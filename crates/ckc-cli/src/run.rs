//! SPEC §3 `ckc run`, document half (cli-runner.2a): resolve the experiment
//! through the §8.4 registries, then drive each corpus document through
//! extract → segment → normalize → assemble into the §8.3 run layout —
//! `artifacts/<doc-id>/{source_graph,segments,normalization,ir_bundle}.json`
//! — every artifact written as §4.3 canonical envelope bytes and strict-read
//! back at the write boundary ([`land`]). Each attempted stage records
//! exactly one §4.6 stage event carrying the artifact's envelope
//! diagnostics (or the failure diagnostic); group stages (compile/verify)
//! and the total-outcome assembly land with cli-runner.2b, marked by the
//! pending `unsupported` merge at their slot.
//!
//! Failure scoping: registry resolution, lexicon loading, and corpus-file
//! reads are command-scope ([`Shell::diagnostic`]); a stage failure rides
//! its stage event and skips the document's remaining stages, leaving other
//! documents to proceed (§4.4 valid-remainder rule). Producer values are
//! runner-owned: candidate = the experiment's pipeline, component = the
//! registry stage component, toolchain manifest hash = the zero placeholder
//! until cli-runner.4.1b mints run manifests (envelope metadata outside
//! content hashes, so replay identity is unaffected by the swap).

use std::path::Path;

use ckc_core::{
    ArtifactEnvelope, Authority, CanonError, CanonRead, Canonical, CorpusEntry, DataClass,
    DiagnosticRecord, Hash, Id, IrBundle, Normalization, Origin, Outcome, Producer, SegmentIr,
    SourceGraph, assemble, canonical_payload_bytes, canonical_sort_key,
    canonicalization_policy_hash, content_hash, parse_candidates, parse_corpora, parse_experiments,
    read_canonical,
};

use crate::extract::{ExtractConfig, extract};
use crate::normalize::{Lexicon, load_lexicon, normalize};
use crate::registry_check::{invalid_diagnostic, load};
use crate::segment::segment;
use crate::shell::{Shell, StageClock, StageEvent, stage_clock, static_id};

/// §5 lexicon authority the normalize stage consumes (module doc in
/// [`crate::normalize`]), read from the invocation root like the registries.
const LEXICON_FILE: &str = "corpus/lexicon/ja_core.yaml";

/// The four §8.3 document stages this unit drives, in chain order, spelled
/// as the registry `kind` tokens the pipeline's stage components declare.
const DOCUMENT_STAGE_KINDS: [&str; 4] = ["extract", "segment", "normalize", "assemble"];

/// Run `ckc run` rooted at `root` (the invocation working directory: §3
/// anchors `registry/` and corpus paths at the repository root). Evidence,
/// artifacts, and the outcome land entirely in the shell.
pub(crate) fn execute(root: &Path, experiment_id: &Id, shell: &mut Shell) {
    let Some(resolved) = resolve(root, experiment_id, shell) else {
        return;
    };
    let lexicon = match std::fs::read(root.join(LEXICON_FILE)) {
        Ok(bytes) => match load_lexicon(&bytes) {
            Ok(lexicon) => lexicon,
            Err(e) => {
                shell.diagnostic(invalid_diagnostic(vec![
                    (static_id("file"), LEXICON_FILE.to_owned()),
                    (static_id("reason"), e.to_string()),
                ]));
                return;
            }
        },
        Err(e) => {
            shell.diagnostic(invalid_diagnostic(vec![
                (static_id("file"), LEXICON_FILE.to_owned()),
                (static_id("reason"), format!("read {LEXICON_FILE}: {e}")),
            ]));
            return;
        }
    };
    for entry in &resolved.documents {
        document_pipeline(root, entry, &resolved, &lexicon, shell);
    }
    // Group stages (compile/verify) and the total-outcome assembly are
    // cli-runner.2b's deliverable, replacing this pending marker in place.
    shell.merge(Outcome::Unsupported);
}

/// The runner's resolved view of one experiment: the pipeline candidate,
/// its document-stage component ids, and the unique corpus documents across
/// the fixture groups in first-appearance order.
struct Resolved {
    pipeline_id: Id,
    /// Stage component ids parallel to [`DOCUMENT_STAGE_KINDS`].
    components: [Id; 4],
    documents: Vec<CorpusEntry>,
}

/// Resolve `experiment_id` against the §8.4 registry surface. Whole-set
/// semantic validation is `ckc registry check`'s job; resolution diagnoses
/// exactly the references this run needs, each failure one command-scope
/// `schema_invalid` diagnostic.
fn resolve(root: &Path, experiment_id: &Id, shell: &mut Shell) -> Option<Resolved> {
    let corpora = load(root, "registry/corpora.yaml", parse_corpora, shell);
    let candidates = load(root, "registry/candidates.yaml", parse_candidates, shell);
    let experiments = load(root, "registry/experiments.yaml", parse_experiments, shell);
    let (Some(corpora), Some(candidates), Some(experiments)) = (corpora, candidates, experiments)
    else {
        return None;
    };

    let Some(experiment) = experiments.iter().find(|e| e.id == *experiment_id) else {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("reason"),
            format!("experiment {experiment_id} is not in registry/experiments.yaml"),
        )]));
        return None;
    };
    let Some(pipeline) = candidates
        .pipelines
        .iter()
        .find(|p| p.id == experiment.pipeline)
    else {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("reason"),
            format!(
                "experiment {experiment_id} names undefined pipeline {}",
                experiment.pipeline
            ),
        )]));
        return None;
    };

    let mut components: Vec<Id> = Vec::with_capacity(DOCUMENT_STAGE_KINDS.len());
    for kind in DOCUMENT_STAGE_KINDS {
        let found = pipeline.stages.iter().find_map(|stage_id| {
            candidates
                .stages
                .iter()
                .find(|s| s.id == *stage_id && s.kind == static_id(kind))
        });
        match found {
            Some(stage) => components.push(stage.id.clone()),
            None => {
                shell.diagnostic(invalid_diagnostic(vec![(
                    static_id("reason"),
                    format!(
                        "pipeline {} declares no {kind} stage component",
                        pipeline.id
                    ),
                )]));
                return None;
            }
        }
    }
    let components: [Id; 4] = components
        .try_into()
        .expect("the loop pushes one component per document stage kind");

    let mut documents: Vec<CorpusEntry> = Vec::new();
    let mut unresolved = false;
    for group in &experiment.fixture_groups {
        for fixture in &group.fixtures {
            if documents.iter().any(|d| d.id == *fixture) {
                continue;
            }
            match corpora.iter().find(|c| c.id == *fixture) {
                Some(entry) => documents.push(entry.clone()),
                None => {
                    shell.diagnostic(invalid_diagnostic(vec![(
                        static_id("reason"),
                        format!(
                            "group {} names fixture {fixture} undefined in registry/corpora.yaml",
                            group.group_id
                        ),
                    )]));
                    unresolved = true;
                }
            }
        }
    }
    if unresolved {
        return None;
    }
    Some(Resolved {
        pipeline_id: pipeline.id.clone(),
        components,
        documents,
    })
}

/// Drive one corpus document through the four document stages. Every
/// attempted stage lands exactly one stage event; the first failure stops
/// this document and leaves the rest of the run to proceed.
fn document_pipeline(
    root: &Path,
    entry: &CorpusEntry,
    resolved: &Resolved,
    lexicon: &Lexicon,
    shell: &mut Shell,
) {
    let html = match std::fs::read(root.join(&entry.path)) {
        Ok(bytes) => bytes,
        Err(e) => {
            shell.diagnostic(invalid_diagnostic(vec![
                (static_id("file"), entry.path.clone()),
                (static_id("reason"), format!("read {}: {e}", entry.path)),
            ]));
            return;
        }
    };
    let dir = format!("artifacts/{}", entry.id);

    let clock = stage_clock();
    let config = ExtractConfig {
        document_id: entry.id.clone(),
        source_family: static_id("synthetic_fixture_html"),
        provenance: entry.provenance,
        data_class: DataClass::None,
        producer: producer(resolved, 0),
    };
    let built = extract(&html, &config).map_err(|e| stage_diagnostic(0, &entry.id, e.to_string()));
    let rel = format!("{dir}/source_graph.json");
    let Some(source) = close_stage(shell, resolved, 0, clock, Vec::new(), &rel, built) else {
        return;
    };

    let clock = stage_clock();
    let built = segment(&source, &producer(resolved, 1))
        .map_err(|e| stage_diagnostic(1, &entry.id, e.to_string()));
    let rel = format!("{dir}/segments.json");
    let inputs = vec![source.content_hash.clone()];
    let Some(segments) = close_stage(shell, resolved, 1, clock, inputs, &rel, built) else {
        return;
    };

    let clock = stage_clock();
    let built = normalize(&source, &segments, lexicon, &producer(resolved, 2))
        .map_err(|e| stage_diagnostic(2, &entry.id, e.to_string()));
    let rel = format!("{dir}/normalization.json");
    let inputs = vec![source.content_hash.clone(), segments.content_hash.clone()];
    let Some(normalization) = close_stage(shell, resolved, 2, clock, inputs, &rel, built) else {
        return;
    };

    let clock = stage_clock();
    let built = assemble_bundle(entry, resolved, &source, &segments, &normalization);
    let rel = format!("{dir}/ir_bundle.json");
    let inputs = vec![
        source.content_hash.clone(),
        segments.content_hash.clone(),
        normalization.content_hash.clone(),
    ];
    close_stage(shell, resolved, 3, clock, inputs, &rel, built);
}

/// The §8.3 assemble stage, the thin core-ir.4/.5 wrapper: derive the DocIR
/// view from the source graph and its extraction diagnostics, assemble the
/// five-layer bundle (bundle-level diagnostics = canonical-set union of the
/// segments and normalization envelope diagnostics; extraction diagnostics
/// stay in DocIr per the §5 bundle row; M1 fixtures inject no assumptions),
/// validate it against the graph, and envelope it.
fn assemble_bundle(
    entry: &CorpusEntry,
    resolved: &Resolved,
    source: &ArtifactEnvelope<SourceGraph>,
    segments: &ArtifactEnvelope<SegmentIr>,
    normalization: &ArtifactEnvelope<Normalization>,
) -> Result<ArtifactEnvelope<IrBundle>, DiagnosticRecord> {
    let fail = |reason: String| stage_diagnostic(3, &entry.id, reason);

    let doc = ckc_core::DocIr::from_graph(&source.payload, source.diagnostics.clone())
        .map_err(|e| fail(format!("doc layer: {e}")))?;

    let mut keyed: Vec<(Vec<u8>, DiagnosticRecord)> = segments
        .diagnostics
        .iter()
        .chain(&normalization.diagnostics)
        .map(|d| Ok((canonical_sort_key(d)?, d.clone())))
        .collect::<Result<_, CanonError>>()
        .map_err(|e| fail(format!("diagnostic sort key: {e}")))?;
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    keyed.dedup_by(|a, b| a.0 == b.0);
    let diagnostics = keyed.into_iter().map(|(_, d)| d).collect();

    let bundle = assemble(
        doc,
        segments.payload.clone(),
        normalization.payload.clinical.clone(),
        normalization.payload.norm.clone(),
        Vec::new(),
        diagnostics,
    )
    .map_err(|e| fail(format!("assembly: {e}")))?;
    bundle
        .validate(&source.payload)
        .map_err(|e| fail(format!("bundle invariant: {e}")))?;

    Ok(ArtifactEnvelope {
        schema_id: static_id("schema.ir_bundle"),
        artifact_id: Id::new(format!("{}.ir_bundle", entry.id))
            .expect("a valid document id keeps the Id grammar under a suffix"),
        artifact_kind: static_id("ir_bundle"),
        producer: producer(resolved, 3),
        input_hashes: vec![
            source.content_hash.clone(),
            segments.content_hash.clone(),
            normalization.content_hash.clone(),
        ],
        content_hash: content_hash(&bundle).map_err(|e| fail(format!("content hash: {e}")))?,
        canonicalization_policy_hash: canonicalization_policy_hash(),
        origin: Origin::DeterministicCompiler,
        authority: Authority::MechanicalAuthority,
        accepted_effects: vec![],
        trace_refs: vec![],
        // Assembly raised nothing of its own: layer diagnostics live in the
        // payload, stage failures never reach an envelope.
        diagnostics: vec![],
        runtime_metadata: vec![],
        payload: bundle,
    })
}

/// Close one attempted stage: land the built envelope at `rel` on success,
/// then record the stage event (envelope diagnostics and content hash on
/// success, the failure diagnostic alone otherwise). Returns the read-back
/// envelope for the next stage; `None` means the event recorded a failure.
fn close_stage<P: Canonical + CanonRead>(
    shell: &mut Shell,
    resolved: &Resolved,
    stage_index: usize,
    clock: StageClock,
    input_hashes: Vec<Hash>,
    rel: &str,
    built: Result<ArtifactEnvelope<P>, DiagnosticRecord>,
) -> Option<ArtifactEnvelope<P>> {
    let landed = built.and_then(|envelope| land(shell, rel, envelope));
    let (started_at, ended_at, duration_ms) = clock.stop();
    let (outcome, diagnostics, output_hashes, envelope) = match landed {
        Ok(envelope) => (
            severity(&envelope.diagnostics),
            envelope.diagnostics.clone(),
            vec![envelope.content_hash.clone()],
            Some(envelope),
        ),
        Err(diagnostic) => (diagnostic.outcome, vec![diagnostic], Vec::new(), None),
    };
    shell.stage_event(StageEvent {
        candidate_id: resolved.pipeline_id.clone(),
        component_id: resolved.components[stage_index].clone(),
        stage: static_id(DOCUMENT_STAGE_KINDS[stage_index]),
        started_at,
        ended_at,
        duration_ms,
        input_hashes,
        output_hashes,
        outcome,
        diagnostics,
        budget_counters: Vec::new(),
    });
    envelope
}

/// The write boundary: validate the produced envelope, write its canonical
/// bytes under `rel`, strict-read the file back, re-validate, and return
/// the read-back value — §8.5 item 3's per-artifact property enforced at
/// production time. Downstream stages chain the read-back value, never the
/// in-memory precursor: the §4.4 accepted artifact is the canonical bytes
/// on disk, and §4.3 set emission sorts what some producers store in
/// creation order (e.g. SourceGraph regions), so only disk truth keeps
/// every consumer — here, cli-runner.2b/.2c, replay — seeing one value.
/// Failures come back as the stage's diagnostic.
fn land<P: Canonical + CanonRead>(
    shell: &Shell,
    rel: &str,
    envelope: ArtifactEnvelope<P>,
) -> Result<ArtifactEnvelope<P>, DiagnosticRecord> {
    let fail = |reason: String| {
        invalid_diagnostic(vec![
            (static_id("artifact"), rel.to_owned()),
            (static_id("reason"), reason),
        ])
    };
    envelope
        .validate()
        .map_err(|e| fail(format!("envelope invariant: {e}")))?;
    let bytes =
        canonical_payload_bytes(&envelope).map_err(|e| fail(format!("canonical emission: {e}")))?;
    let path = shell
        .write_under(rel, &bytes)
        .map_err(|e| fail(e.to_string()))?;
    let read_back = std::fs::read(&path).map_err(|e| fail(format!("read back: {e}")))?;
    let parsed: ArtifactEnvelope<P> =
        read_canonical(&read_back).map_err(|e| fail(format!("strict read: {e}")))?;
    parsed
        .validate()
        .map_err(|e| fail(format!("read-back invariant: {e}")))?;
    Ok(parsed)
}

/// §4.4 stage outcome: severity max over the artifact's diagnostics.
fn severity(diagnostics: &[DiagnosticRecord]) -> Outcome {
    diagnostics
        .iter()
        .map(|d| d.outcome)
        .fold(Outcome::Ok, Outcome::max)
}

/// Stage-failure diagnostic: `schema_invalid`/`invalid` naming the §8.3
/// stage and the document (§4.4 "schema, hash, canonicalization … fails").
fn stage_diagnostic(stage_index: usize, document_id: &Id, reason: String) -> DiagnosticRecord {
    invalid_diagnostic(vec![
        (static_id("document"), document_id.to_string()),
        (static_id("reason"), reason),
        (
            static_id("stage"),
            DOCUMENT_STAGE_KINDS[stage_index].to_owned(),
        ),
    ])
}

/// §4.4 producer for one stage execution. The toolchain manifest hash is
/// the zero placeholder until cli-runner.4.1b mints run manifests.
fn producer(resolved: &Resolved, stage_index: usize) -> Producer {
    Producer {
        candidate_id: resolved.pipeline_id.clone(),
        component_id: resolved.components[stage_index].clone(),
        toolchain_manifest_hash: Hash::new(format!("sha256:{}", "0".repeat(64)))
            .expect("the zero digest matches the Hash grammar"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use ckc_core::{EventRecord, TotalOperationResult, read_jsonl};

    /// Repository root: two levels above the ckc-cli manifest, where the §3
    /// `registry/` and `corpus/` trees live.
    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("crates/ckc-cli sits two levels under the repo root")
            .to_path_buf()
    }

    /// Execute `experiment` from `root` into a fresh `<tmp>/m1` run
    /// directory and finish the shell: the §4.4 result, the parsed event
    /// stream, the parsed diagnostics stream, and the run directory (with
    /// its tempdir guard).
    fn executed(
        root: &Path,
        experiment: &str,
    ) -> (
        TotalOperationResult,
        Vec<EventRecord>,
        Vec<DiagnosticRecord>,
        PathBuf,
        tempfile::TempDir,
    ) {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("m1");
        std::fs::create_dir_all(&out).unwrap();
        let mut shell = Shell::open(static_id("run"), static_id("m1"), Some(out.clone()));
        execute(root, &experiment.parse().unwrap(), &mut shell);
        let finished = shell.finish().unwrap();
        let events = read_jsonl(&std::fs::read(out.join("logs/events.jsonl")).unwrap()).unwrap();
        let diagnostics =
            read_jsonl(&std::fs::read(out.join("logs/diagnostics.jsonl")).unwrap()).unwrap();
        (finished.result, events, diagnostics, out, tmp)
    }

    const DOC_IDS: [&str; 3] = [
        "fixture.m1_guideline_a",
        "fixture.m1_guideline_b",
        "fixture.m1_control",
    ];

    /// §4.3 set emission canonically sorts envelope input hashes, so the
    /// chain expectations compare as sorted sets (ASCII `sha256:` text
    /// orders identically under derived `Ord` and the canonical byte key).
    fn sorted(hashes: &[Hash]) -> Vec<Hash> {
        let mut hashes = hashes.to_vec();
        hashes.sort();
        hashes
    }

    /// Strict-read one landed artifact envelope and re-check its mechanical
    /// invariants; the §8.5 item 3 per-artifact property, asserted from the
    /// consumer side.
    fn strict<P: Canonical + CanonRead>(out: &Path, doc: &str, name: &str) -> ArtifactEnvelope<P> {
        let path = out.join(format!("artifacts/{doc}/{name}.json"));
        let envelope: ArtifactEnvelope<P> = read_canonical(&std::fs::read(&path).unwrap())
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        envelope.validate().unwrap();
        envelope
    }

    // The unit gate: the document stages over the three fixtures land the
    // twelve §8.3 document artifacts, every one strict-read clean with its
    // input hashes chaining the §8.4 stage order, and the event stream
    // carries one clean stage event per execution before the command event.
    #[test]
    fn document_stages_land_strict_artifacts_over_the_fixtures() {
        let (result, events, diagnostics, out, _tmp) = executed(&repo_root(), "exp.m1_spine");

        // Group stages are pending (cli-runner.2b): unsupported, clean.
        assert_eq!(result.outcome, Outcome::Unsupported);
        assert!(result.diagnostic_hashes.is_empty());
        assert!(diagnostics.is_empty());

        for doc in DOC_IDS {
            let source: ArtifactEnvelope<SourceGraph> = strict(&out, doc, "source_graph");
            let segments: ArtifactEnvelope<SegmentIr> = strict(&out, doc, "segments");
            let normalization: ArtifactEnvelope<Normalization> = strict(&out, doc, "normalization");
            let bundle: ArtifactEnvelope<IrBundle> = strict(&out, doc, "ir_bundle");

            assert_eq!(
                source.artifact_id,
                format!("{doc}.source_graph").parse().unwrap()
            );
            assert_eq!(source.input_hashes, Vec::new());
            assert_eq!(segments.input_hashes, vec![source.content_hash.clone()]);
            assert_eq!(
                normalization.input_hashes,
                sorted(&[source.content_hash.clone(), segments.content_hash.clone()])
            );
            assert_eq!(
                bundle.input_hashes,
                sorted(&[
                    source.content_hash.clone(),
                    segments.content_hash.clone(),
                    normalization.content_hash.clone()
                ])
            );
            assert_eq!(
                bundle.producer.candidate_id,
                static_id("pipe.layered_ckcir_to_smt")
            );
            assert_eq!(bundle.producer.component_id, static_id("stage.m1.assemble"));

            // The bundle re-validates against its graph and carries the §8.6
            // rule the gold core expects from each fixture.
            bundle.payload.validate(&source.payload).unwrap();
            assert_eq!(bundle.payload.norm.rules.len(), 1);
            assert_eq!(
                bundle.payload.norm.rules[0].rule_id,
                format!("{doc}.rule.0").parse().unwrap()
            );
        }

        // 4 stage events per document, then the closing command event.
        assert_eq!(events.len(), 13);
        for (n, event) in events.iter().enumerate() {
            assert_eq!(event.event_id, format!("event.{n}").parse::<Id>().unwrap());
            assert_eq!(event.logical_time, n as u64);
            assert_eq!(event.run_id, static_id("m1"));
        }
        for (d, doc) in DOC_IDS.iter().enumerate() {
            for (s, kind) in DOCUMENT_STAGE_KINDS.iter().enumerate() {
                let event = &events[d * 4 + s];
                assert_eq!(event.stage, static_id(kind), "{doc}");
                assert_eq!(event.candidate_id, static_id("pipe.layered_ckcir_to_smt"));
                assert_eq!(
                    event.component_id,
                    format!("stage.m1.{kind}").parse().unwrap()
                );
                assert_eq!(event.outcome, Outcome::Ok);
                assert_eq!(event.input_hashes.len(), s);
                assert_eq!(event.output_hashes.len(), 1);
                assert!(event.diagnostics.is_empty());
            }
        }
        let command = &events[12];
        assert_eq!(command.stage, static_id("run"));
        assert_eq!(command.outcome, Outcome::Unsupported);
    }

    // Resolution failures are command-scope diagnostics: a root without
    // registries reports every unreadable file; a real root with an unknown
    // experiment names it. No artifacts land either way.
    #[test]
    fn resolution_failures_diagnose() {
        let bare = tempfile::tempdir().unwrap();
        let (result, events, diagnostics, out, _tmp) = executed(bare.path(), "exp.m1_spine");
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 3);
        assert_eq!(events.len(), 1);
        assert!(!out.join("artifacts").exists());

        let (result, events, diagnostics, _out, _tmp) = executed(&repo_root(), "exp.bogus");
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .payload
                .iter()
                .any(|(_, v)| v.contains("exp.bogus")),
            "{diagnostics:?}"
        );
    }

    /// Write a minimal two-fixture registry trio under `root`: `fixture.gone`
    /// points at a missing file, `fixture.tiny` at a minimal HTML document;
    /// the pipeline declares exactly the four document stages.
    fn write_tiny_root(root: &Path) {
        let write = |rel: &str, text: &str| {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        write(
            "registry/corpora.yaml",
            "\
- id: fixture.gone
  path: corpus/fixtures/gone.html
  origin: ai_generated
  authority: source_authority
  provenance: synthetic
- id: fixture.tiny
  path: corpus/fixtures/tiny.html
  origin: ai_generated
  authority: source_authority
  provenance: synthetic
",
        );
        write(
            "registry/candidates.yaml",
            "\
pipelines:
  - id: pipe.tiny
    stages: [stage.t.extract, stage.t.segment, stage.t.normalize, stage.t.assemble]
stages:
  - id: stage.t.extract
    kind: extract
    determinism: deterministic
    input_artifact_kinds: []
    output_artifact_kinds: [source_graph]
  - id: stage.t.segment
    kind: segment
    determinism: deterministic
    input_artifact_kinds: [source_graph]
    output_artifact_kinds: [segments]
  - id: stage.t.normalize
    kind: normalize
    determinism: deterministic
    input_artifact_kinds: [source_graph, segments]
    output_artifact_kinds: [normalization]
  - id: stage.t.assemble
    kind: assemble
    determinism: deterministic
    input_artifact_kinds: [source_graph, segments, normalization]
    output_artifact_kinds: [ir_bundle]
",
        );
        write(
            "registry/experiments.yaml",
            "\
- id: exp.tiny
  pipeline: pipe.tiny
  fixture_groups:
    - group_id: group.t
      fixtures: [fixture.gone, fixture.tiny]
  seed: 1
  budget:
    solver_ms_per_query: 1000
  expected_outcomes: corpus/gold/t.yaml
",
        );
        write(
            "corpus/fixtures/tiny.html",
            "<html><body><p>本文。</p></body></html>",
        );
    }

    // §4.4 valid remainder: a document whose corpus file is missing takes a
    // command-scope diagnostic while the other document still runs all four
    // stages and lands its artifacts.
    #[test]
    fn missing_corpus_file_keeps_other_documents() {
        let root = tempfile::tempdir().unwrap();
        write_tiny_root(root.path());
        let lexicon_target = root.path().join(LEXICON_FILE);
        std::fs::create_dir_all(lexicon_target.parent().unwrap()).unwrap();
        std::fs::copy(repo_root().join(LEXICON_FILE), lexicon_target).unwrap();

        let (result, events, diagnostics, out, _tmp) = executed(root.path(), "exp.tiny");
        assert_eq!(result.outcome, Outcome::Invalid);
        // Ledger: the command-scope read failure first, then the tiny
        // document's stage residuals (extract parse error, unclassified
        // paragraph) riding their stage events.
        assert_eq!(diagnostics.len(), 3);
        assert!(
            diagnostics[0]
                .payload
                .iter()
                .any(|(_, v)| v.contains("gone.html")),
            "{diagnostics:?}"
        );

        assert_eq!(events.len(), 5);
        for (s, kind) in DOCUMENT_STAGE_KINDS.iter().enumerate() {
            assert_eq!(events[s].stage, static_id(kind));
            assert_eq!(
                events[s].component_id,
                format!("stage.t.{kind}").parse().unwrap()
            );
            assert_eq!(
                events[s].output_hashes.len(),
                1,
                "{kind} landed its artifact"
            );
        }
        assert!(!out.join("artifacts/fixture.gone").exists());
        let bundle: ArtifactEnvelope<IrBundle> = strict(&out, "fixture.tiny", "ir_bundle");
        let source: ArtifactEnvelope<SourceGraph> = strict(&out, "fixture.tiny", "source_graph");
        bundle.payload.validate(&source.payload).unwrap();
    }

    // The lexicon is load-bearing for the whole run: an unreadable file is
    // one command-scope diagnostic and no document runs.
    #[test]
    fn missing_lexicon_stops_the_run() {
        let root = tempfile::tempdir().unwrap();
        write_tiny_root(root.path());
        let (result, events, diagnostics, out, _tmp) = executed(root.path(), "exp.tiny");
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .payload
                .iter()
                .any(|(_, v)| v.contains(LEXICON_FILE)),
            "{diagnostics:?}"
        );
        assert_eq!(events.len(), 1);
        assert!(!out.join("artifacts").exists());
    }
}
