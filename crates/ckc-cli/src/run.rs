//! SPEC §3 `ckc run` (cli-runner.2a + .2b + .3a.3 + .4.1b.1): resolve the
//! experiment through the §8.4 registries, drive each corpus document through
//! extract → segment → normalize → assemble into
//! `artifacts/<doc-id>/{source_graph,segments,normalization,ir_bundle}.json`,
//! drive each fixture group through compile → verify into
//! `groups/<gid>/{compiled.json,verifier_results.json,smt/<query-id>.smt2}`,
//! assemble the run-scoped §7.1 trace pair over every landed artifact
//! into `trace_bundle.json` + `lineage_index.json` at the run root, then
//! assemble the §7.2 report over the landed pair into `report.json`
//! — every envelope written as §4.3 canonical bytes and strict-read back at
//! the write boundary ([`land`]), every smt file byte-identical to its
//! [`ckc_smt::QueryBody`] body. Each attempted stage records exactly one
//! §4.6 stage event carrying the artifact's envelope diagnostics (or the
//! failure diagnostic); the §4.4 total outcome is the severity fold over
//! every event and command-scope diagnostic; `report.md` and the run/replay
//! manifests join with cli-runner.4.1b.2.
//!
//! Failure scoping: registry resolution, lexicon loading, solver-adapter
//! construction, and corpus-file reads are command-scope
//! ([`Shell::diagnostic`]); a stage failure rides its stage event and skips
//! the document's (or group's) remaining stages, leaving other documents
//! and groups to proceed (§4.4 valid-remainder rule). A group whose member
//! bundle is missing fails its compile stage rather than compiling a
//! partial group: a cross-document verdict over fewer documents than the
//! group declares would document a null result the fixtures never earned.
//! Producer values are runner-owned: candidate = the experiment's pipeline,
//! component = the registry stage component, toolchain manifest hash = the
//! zero placeholder until cli-runner.4.1b mints run manifests (envelope
//! metadata outside content hashes, so replay identity is unaffected by the
//! swap).

use std::path::Path;
use std::time::Duration;

use ckc_core::{
    ArtifactEnvelope, Authority, CanonError, CanonRead, Canonical, CorpusEntry, DataClass,
    DiagnosticCode, DiagnosticRecord, FixtureGroup, Hash, Id, IrBundle, Normalization, Origin,
    Outcome, Producer, SegmentIr, SolverIdentity, SourceGraph, assemble, canonical_payload_bytes,
    canonical_sort_key, canonicalization_policy_hash, content_hash, hash_bytes, parse_candidates,
    parse_corpora, parse_experiments, read_canonical,
};
use ckc_smt::{VerifierResults, Z3Adapter, compile, verify};

use crate::extract::{ExtractConfig, extract};
use crate::normalize::{Lexicon, load_lexicon, normalize};
use crate::registry_check::{invalid_diagnostic, load};
use crate::report::assemble_report;
use crate::segment::segment;
use crate::shell::{Shell, StageClock, StageEvent, stage_clock, static_id};
use crate::trace::{DocTrace, GroupTrace, LineageIndex, TraceBundle, assemble_trace};

/// §5 lexicon authority the normalize stage consumes (module doc in
/// [`crate::normalize`]), read from the invocation root like the registries.
const LEXICON_FILE: &str = "corpus/lexicon/ja_core.yaml";

/// The eight §8.3 stages this module drives, in chain order, spelled as the
/// registry `kind` tokens the pipeline's stage components declare: four
/// per-document stages, the two per-group stages, then the run-scoped
/// trace and report stages.
const STAGE_KINDS: [&str; 8] = [
    "extract",
    "segment",
    "normalize",
    "assemble",
    "compile",
    "verify",
    "trace",
    "report",
];

/// [`STAGE_KINDS`] indices of the group stages and the run-scoped pair.
const COMPILE: usize = 4;
const VERIFY: usize = 5;
const TRACE: usize = 6;
const REPORT: usize = 7;

/// §8.4 budget counter naming the per-query solver wall-clock cap in
/// milliseconds — the one budget key the M1 vocabulary defines.
const SOLVER_BUDGET_KEY: &str = "solver_ms_per_query";

/// Run `ckc run` rooted at `root` (the invocation working directory: §3
/// anchors `registry/` and corpus paths at the repository root). Evidence,
/// artifacts, and the outcome land entirely in the shell.
pub(crate) fn execute(root: &Path, experiment_id: &Id, shell: &mut Shell) {
    let Some(resolved) = resolve(root, experiment_id, shell) else {
        return;
    };
    // §7.2's lexicon hash rides the raw authority-file bytes (§4.4: the
    // lexicon is a file, not an accepted artifact), taken here where the
    // run already holds them.
    let (lexicon, lexicon_hash) = match std::fs::read(root.join(LEXICON_FILE)) {
        Ok(bytes) => match load_lexicon(&bytes) {
            Ok(lexicon) => (lexicon, hash_bytes(&bytes)),
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
    let mut docs: Vec<DocTrace> = Vec::new();
    let mut graphs: Vec<ArtifactEnvelope<SourceGraph>> = Vec::new();
    for entry in &resolved.documents {
        if let Some((doc, graph)) = document_pipeline(root, entry, &resolved, &lexicon, shell) {
            docs.push(doc);
            graphs.extend(graph);
        }
    }
    // The solver adapter is load-bearing for every group verdict — the
    // run's deliverable — so construction failure is command-scope, after
    // the document artifacts have already landed (§4.4 valid remainder).
    let adapter = match Z3Adapter::new() {
        Ok(adapter) => adapter,
        Err(e) => {
            shell.diagnostic(DiagnosticRecord {
                code: DiagnosticCode::SolverExecutionFailure,
                outcome: Outcome::Invalid,
                payload: vec![(static_id("reason"), format!("solver adapter: {e}"))],
                region_ids: vec![],
                artifact_hashes: vec![],
            });
            return;
        }
    };
    let mut groups: Vec<GroupTrace> = Vec::with_capacity(resolved.groups.len());
    for group in &resolved.groups {
        groups.push(group_pipeline(group, &docs, &resolved, &adapter, shell));
    }
    // Run-scoped chain: a trace-stage failure stops it before the report,
    // the same first-failure rule the document chain runs under.
    let Some((bundle, lineage)) = trace_stage(&docs, &groups, &resolved, shell) else {
        return;
    };
    report_stage(
        &graphs,
        &groups,
        &bundle,
        &lineage,
        &lexicon_hash,
        adapter.identity(),
        &resolved,
        shell,
    );
}

/// The runner's resolved view of one experiment: the pipeline candidate,
/// its stage component ids, the unique corpus documents across the fixture
/// groups in first-appearance order, the groups themselves in evaluation
/// order, and the per-query solver budget.
struct Resolved {
    pipeline_id: Id,
    /// Stage component ids parallel to [`STAGE_KINDS`].
    components: [Id; 8],
    documents: Vec<CorpusEntry>,
    groups: Vec<FixtureGroup>,
    /// §8.4 `solver_ms_per_query` budget value.
    budget_ms: u64,
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

    let mut components: Vec<Id> = Vec::with_capacity(STAGE_KINDS.len());
    for kind in STAGE_KINDS {
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
    let components: [Id; 8] = components
        .try_into()
        .expect("the loop pushes one component per stage kind");

    let Some(&budget_ms) = experiment.budget.get(&static_id(SOLVER_BUDGET_KEY)) else {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("reason"),
            format!("experiment {experiment_id} declares no {SOLVER_BUDGET_KEY} budget"),
        )]));
        return None;
    };

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
        groups: experiment.fixture_groups.clone(),
        budget_ms,
    })
}

/// Drive one corpus document through the four document stages. Every
/// attempted stage lands exactly one stage event; the first failure stops
/// this document and leaves the rest of the run to proceed. Returns the
/// document's [`DocTrace`] — every landing recorded as it happens, the
/// bundle envelope riding whole as the group stages' input — beside its
/// landed source-graph envelope when extract succeeded (the report stage's
/// quoted-span authority), or `None` when the corpus file itself was
/// unreadable (command-scope diagnostic: without source bytes there is no
/// hash to ground a trace node).
fn document_pipeline(
    root: &Path,
    entry: &CorpusEntry,
    resolved: &Resolved,
    lexicon: &Lexicon,
    shell: &mut Shell,
) -> Option<(DocTrace, Option<ArtifactEnvelope<SourceGraph>>)> {
    let html = match std::fs::read(root.join(&entry.path)) {
        Ok(bytes) => bytes,
        Err(e) => {
            shell.diagnostic(invalid_diagnostic(vec![
                (static_id("file"), entry.path.clone()),
                (static_id("reason"), format!("read {}: {e}", entry.path)),
            ]));
            return None;
        }
    };
    let mut trace = DocTrace {
        document_id: entry.id.clone(),
        fixture_path: entry.path.clone(),
        source_hash: hash_bytes(&html),
        source_graph: None,
        segments: None,
        normalization: None,
        bundle: None,
    };
    let dir = format!("artifacts/{}", entry.id);
    let mut graph: Option<ArtifactEnvelope<SourceGraph>> = None;

    'chain: {
        let clock = stage_clock();
        let config = ExtractConfig {
            document_id: entry.id.clone(),
            source_family: static_id("synthetic_fixture_html"),
            provenance: entry.provenance,
            data_class: DataClass::None,
            producer: producer(resolved, 0),
        };
        let built = extract(&html, &config)
            .map_err(|e| stage_diagnostic(0, "document", &entry.id, e.to_string()));
        let rel = format!("{dir}/source_graph.json");
        let Some(source) = close_stage(shell, resolved, 0, clock, Vec::new(), &rel, built) else {
            break 'chain;
        };
        trace.source_graph = Some((source.artifact_id.clone(), source.content_hash.clone()));
        graph = Some(source.clone());

        let clock = stage_clock();
        let built = segment(&source, &producer(resolved, 1))
            .map_err(|e| stage_diagnostic(1, "document", &entry.id, e.to_string()));
        let rel = format!("{dir}/segments.json");
        let inputs = vec![source.content_hash.clone()];
        let Some(segments) = close_stage(shell, resolved, 1, clock, inputs, &rel, built) else {
            break 'chain;
        };
        trace.segments = Some((segments.artifact_id.clone(), segments.content_hash.clone()));

        let clock = stage_clock();
        let built = normalize(&source, &segments, lexicon, &producer(resolved, 2))
            .map_err(|e| stage_diagnostic(2, "document", &entry.id, e.to_string()));
        let rel = format!("{dir}/normalization.json");
        let inputs = vec![source.content_hash.clone(), segments.content_hash.clone()];
        let Some(normalization) = close_stage(shell, resolved, 2, clock, inputs, &rel, built)
        else {
            break 'chain;
        };
        trace.normalization = Some((
            normalization.artifact_id.clone(),
            normalization.content_hash.clone(),
        ));

        let clock = stage_clock();
        let built = assemble_bundle(entry, resolved, &source, &segments, &normalization);
        let rel = format!("{dir}/ir_bundle.json");
        let inputs = vec![
            source.content_hash.clone(),
            segments.content_hash.clone(),
            normalization.content_hash.clone(),
        ];
        trace.bundle = close_stage(shell, resolved, 3, clock, inputs, &rel, built);
    }
    Some((trace, graph))
}

/// Drive one fixture group through compile → verify. Compile loads the
/// members' landed bundles (all must be present — see the module doc's
/// partial-group rule), compiles their (FormalIR, NormIR) pairs into the
/// enveloped [`ckc_smt::CompiledArtifact`] at `groups/<gid>/compiled.json`,
/// and materializes every planned query body byte-identical at
/// `groups/<gid>/smt/<query-id>.smt2`; verify drives the compiled plan
/// through the solver adapter under the experiment's per-query budget into
/// `groups/<gid>/verifier_results.json`. One stage event each; a compile
/// failure skips the group's verify and leaves other groups to proceed.
/// Returns the group's [`GroupTrace`]: the §8.4 member set plus each group
/// landing that happened, riding whole.
fn group_pipeline(
    group: &FixtureGroup,
    docs: &[DocTrace],
    resolved: &Resolved,
    adapter: &Z3Adapter,
    shell: &mut Shell,
) -> GroupTrace {
    let gid = &group.group_id;
    let dir = format!("groups/{gid}");
    let mut trace = GroupTrace {
        group_id: gid.clone(),
        fixtures: group.fixtures.clone(),
        compiled: None,
        verifier_results: None,
    };

    let clock = stage_clock();
    let mut members: Vec<&ArtifactEnvelope<IrBundle>> = Vec::with_capacity(group.fixtures.len());
    for fixture in &group.fixtures {
        let bundle = docs
            .iter()
            .find(|d| d.document_id == *fixture)
            .and_then(|d| d.bundle.as_ref());
        match bundle {
            Some(bundle) => members.push(bundle),
            None => {
                let built = Err(stage_diagnostic(
                    COMPILE,
                    "group",
                    gid,
                    format!("member {fixture} landed no ir_bundle artifact"),
                ));
                finish_stage::<IrBundle>(shell, resolved, COMPILE, clock, Vec::new(), built);
                return trace;
            }
        }
    }
    let inputs: Vec<Hash> = members.iter().map(|m| m.content_hash.clone()).collect();

    let artifact = compile(
        gid,
        members.iter().map(|m| (&m.payload.formal, &m.payload.norm)),
    );
    let built = artifact
        .validate()
        .map_err(|e| stage_diagnostic(COMPILE, "group", gid, format!("compiled artifact: {e}")))
        .and_then(|()| {
            let diagnostics = canonical_diagnostic_set(&artifact.diagnostics)
                .map_err(|e| stage_diagnostic(COMPILE, "group", gid, e.to_string()))?;
            envelope(
                format!("{gid}.compiled"),
                "compiled",
                producer(resolved, COMPILE),
                inputs.clone(),
                Origin::DeterministicCompiler,
                Authority::CompilerAuthority,
                diagnostics,
                artifact,
            )
            .map_err(|e| stage_diagnostic(COMPILE, "group", gid, e.to_string()))
        });
    let landed = built
        .and_then(|env| land(shell, &format!("{dir}/compiled.json"), env))
        .and_then(|env| {
            materialize_queries(shell, &dir, &env)?;
            Ok(env)
        });
    let Some(compiled) = finish_stage(shell, resolved, COMPILE, clock, inputs, landed) else {
        return trace;
    };

    let clock = stage_clock();
    let results = verify(
        adapter,
        &compiled.payload,
        Duration::from_millis(resolved.budget_ms),
    );
    let wrapped = VerifierResults { results };
    let built = wrapped
        .validate()
        .map_err(|e| stage_diagnostic(VERIFY, "group", gid, format!("verifier results: {e}")))
        .and_then(|()| {
            let diagnostics =
                canonical_diagnostic_set(wrapped.results.iter().flat_map(|r| &r.diagnostics))
                    .map_err(|e| stage_diagnostic(VERIFY, "group", gid, e.to_string()))?;
            envelope(
                format!("{gid}.verifier_results"),
                "verifier_results",
                producer(resolved, VERIFY),
                vec![compiled.content_hash.clone()],
                Origin::AdapterGenerated,
                Authority::VerifierAuthority,
                diagnostics,
                wrapped,
            )
            .map_err(|e| stage_diagnostic(VERIFY, "group", gid, e.to_string()))
        });
    let landed = built.and_then(|env| land(shell, &format!("{dir}/verifier_results.json"), env));
    trace.verifier_results = finish_stage(
        shell,
        resolved,
        VERIFY,
        clock,
        vec![compiled.content_hash.clone()],
        landed,
    );
    trace.compiled = Some(compiled);
    trace
}

/// The §8.3 trace stage, run once after the group loop: assemble the §7.1
/// pair over every landed artifact ([`assemble_trace`] skips absent
/// pieces), validate both payloads, and land them at the run root as
/// `trace_bundle.json` + `lineage_index.json`. Both envelopes carry the
/// DAG's node content-hash set as input hashes (each source's raw-byte
/// hash beside every landed envelope hash; the hashless report node
/// contributes nothing). One stage event covers the pair: both content
/// hashes as outputs, or the first failure diagnostic. Returns the landed
/// pair — the report stage's input — or `None` on the recorded failure.
fn trace_stage(
    docs: &[DocTrace],
    groups: &[GroupTrace],
    resolved: &Resolved,
    shell: &mut Shell,
) -> Option<(
    ArtifactEnvelope<TraceBundle>,
    ArtifactEnvelope<LineageIndex>,
)> {
    let clock = stage_clock();
    let (bundle, lineage) = assemble_trace(docs, groups);
    // §4.3 set semantics up front so the in-memory hashes equal every
    // durable view: distinct nodes can share bytes (two structurally
    // identical segments artifacts hash alike), and set emission would
    // collapse them at the boundary anyway.
    let mut input_hashes: Vec<Hash> = bundle
        .nodes
        .iter()
        .filter_map(|n| n.content_hash.clone())
        .collect();
    input_hashes.sort();
    input_hashes.dedup();
    let run_id = shell.run_id().clone();
    let fail = |reason: String| stage_diagnostic(TRACE, "run", &run_id, reason);

    let landed = bundle
        .validate()
        .map_err(|e| fail(format!("trace bundle: {e}")))
        .and_then(|()| {
            lineage
                .validate()
                .map_err(|e| fail(format!("lineage index: {e}")))
        })
        .and_then(|()| {
            let bundle = envelope(
                "trace_bundle".to_owned(),
                "trace_bundle",
                producer(resolved, TRACE),
                input_hashes.clone(),
                Origin::DeterministicCompiler,
                Authority::MechanicalAuthority,
                vec![],
                bundle,
            )
            .map_err(|e| fail(e.to_string()))?;
            let lineage = envelope(
                "lineage_index".to_owned(),
                "lineage_index",
                producer(resolved, TRACE),
                input_hashes.clone(),
                Origin::DeterministicCompiler,
                Authority::MechanicalAuthority,
                vec![],
                lineage,
            )
            .map_err(|e| fail(e.to_string()))?;
            Ok((bundle, lineage))
        })
        .and_then(|(bundle, lineage)| {
            let bundle = land(shell, "trace_bundle.json", bundle)?;
            let lineage = land(shell, "lineage_index.json", lineage)?;
            Ok((bundle, lineage))
        });

    let (started_at, ended_at, duration_ms) = clock.stop();
    let (outcome, diagnostics, output_hashes, pair) = match landed {
        // Both envelopes are built with empty diagnostics (assembly raises
        // nothing of its own), so a landed pair is a clean stage.
        Ok((bundle, lineage)) => (
            Outcome::Ok,
            Vec::new(),
            vec![bundle.content_hash.clone(), lineage.content_hash.clone()],
            Some((bundle, lineage)),
        ),
        Err(diagnostic) => (diagnostic.outcome, vec![diagnostic], Vec::new(), None),
    };
    shell.stage_event(StageEvent {
        candidate_id: resolved.pipeline_id.clone(),
        component_id: resolved.components[TRACE].clone(),
        stage: static_id(STAGE_KINDS[TRACE]),
        started_at,
        ended_at,
        duration_ms,
        input_hashes,
        output_hashes,
        outcome,
        diagnostics,
        budget_counters: Vec::new(),
    });
    pair
}

/// The §8.3 report stage, the run-scoped chain's tail: snapshot the
/// shell's diagnostic ledger (every §7.4 record the run raised before this
/// stage; the report cannot count records that do not yet exist), assemble
/// the §7.2 [`crate::report::Report`] over the landed trace pair, the
/// landed source-graph envelopes, the landed verifier results, the raw
/// lexicon-byte hash, and the adapter's live solver identity, validate it,
/// and land it at the run root as `report.json` — the path the DAG's
/// hashless sink node already names. Input hashes are the §4.3 set of
/// every consumed envelope's content hash; one stage event closes the
/// stage with the report's content hash or the first failure diagnostic.
#[allow(clippy::too_many_arguments)]
fn report_stage(
    graphs: &[ArtifactEnvelope<SourceGraph>],
    groups: &[GroupTrace],
    bundle: &ArtifactEnvelope<TraceBundle>,
    lineage: &ArtifactEnvelope<LineageIndex>,
    lexicon_hash: &Hash,
    solver_identity: &SolverIdentity,
    resolved: &Resolved,
    shell: &mut Shell,
) {
    let clock = stage_clock();
    let ledger = shell.ledger().to_vec();
    let run_id = shell.run_id().clone();
    let fail = |reason: String| stage_diagnostic(REPORT, "run", &run_id, reason);

    let results: Vec<&ArtifactEnvelope<VerifierResults>> = groups
        .iter()
        .filter_map(|g| g.verifier_results.as_ref())
        .collect();
    let mut input_hashes = vec![bundle.content_hash.clone(), lineage.content_hash.clone()];
    input_hashes.extend(graphs.iter().map(|g| g.content_hash.clone()));
    input_hashes.extend(results.iter().map(|r| r.content_hash.clone()));
    input_hashes.sort();
    input_hashes.dedup();

    let landed = assemble_report(
        &bundle.payload,
        &lineage.payload,
        &graphs.iter().map(|g| &g.payload).collect::<Vec<_>>(),
        &results.iter().map(|r| &r.payload).collect::<Vec<_>>(),
        lexicon_hash,
        solver_identity,
        &ledger,
    )
    .map_err(|e| fail(format!("report assembly: {e}")))
    .and_then(|report| {
        report
            .validate()
            .map_err(|e| fail(format!("report invariant: {e}")))?;
        envelope(
            "report".to_owned(),
            "report",
            producer(resolved, REPORT),
            input_hashes.clone(),
            Origin::DeterministicCompiler,
            Authority::MechanicalAuthority,
            vec![],
            report,
        )
        .map_err(|e| fail(e.to_string()))
    })
    .and_then(|report| land(shell, "report.json", report));

    let (started_at, ended_at, duration_ms) = clock.stop();
    let (outcome, diagnostics, output_hashes) = match landed {
        // The envelope is built with empty diagnostics (assembly raises
        // nothing of its own), so a landed report is a clean stage.
        Ok(report) => (Outcome::Ok, Vec::new(), vec![report.content_hash]),
        Err(diagnostic) => (diagnostic.outcome, vec![diagnostic], Vec::new()),
    };
    shell.stage_event(StageEvent {
        candidate_id: resolved.pipeline_id.clone(),
        component_id: resolved.components[REPORT].clone(),
        stage: static_id(STAGE_KINDS[REPORT]),
        started_at,
        ended_at,
        duration_ms,
        input_hashes,
        output_hashes,
        outcome,
        diagnostics,
        budget_counters: Vec::new(),
    });
}

/// Materialize the landed compiled artifact's query bodies as the §8.3
/// `groups/<gid>/smt/<query-id>.smt2` files, each read back and checked
/// byte-identical to its [`ckc_smt::QueryBody`] body — solver-bound text
/// pinned at the same boundary discipline as the envelopes.
fn materialize_queries(
    shell: &Shell,
    dir: &str,
    compiled: &ArtifactEnvelope<ckc_smt::CompiledArtifact>,
) -> Result<(), DiagnosticRecord> {
    for body in &compiled.payload.query_bodies {
        let rel = format!("{dir}/smt/{}.smt2", body.query_id);
        let fail = |reason: String| {
            invalid_diagnostic(vec![
                (static_id("artifact"), rel.clone()),
                (static_id("reason"), reason),
            ])
        };
        let path = shell
            .write_under(&rel, body.body.as_bytes())
            .map_err(|e| fail(e.to_string()))?;
        let read_back = std::fs::read(&path).map_err(|e| fail(format!("read back: {e}")))?;
        if read_back != body.body.as_bytes() {
            return Err(fail("read-back bytes differ from the query body".into()));
        }
    }
    Ok(())
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
    let fail = |reason: String| stage_diagnostic(3, "document", &entry.id, reason);

    let doc = ckc_core::DocIr::from_graph(&source.payload, source.diagnostics.clone())
        .map_err(|e| fail(format!("doc layer: {e}")))?;

    let diagnostics = canonical_diagnostic_set(
        segments
            .diagnostics
            .iter()
            .chain(&normalization.diagnostics),
    )
    .map_err(|e| fail(format!("diagnostic sort key: {e}")))?;

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
/// then record the stage event ([`finish_stage`]).
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
    finish_stage(shell, resolved, stage_index, clock, input_hashes, landed)
}

/// Record one attempted stage's §4.6 event: envelope diagnostics and
/// content hash on success, the failure diagnostic alone otherwise; the
/// verify stage carries its §8.4 budget counter. Returns the landed
/// envelope for the next consumer; `None` means the event recorded a
/// failure.
fn finish_stage<P: Canonical + CanonRead>(
    shell: &mut Shell,
    resolved: &Resolved,
    stage_index: usize,
    clock: StageClock,
    input_hashes: Vec<Hash>,
    landed: Result<ArtifactEnvelope<P>, DiagnosticRecord>,
) -> Option<ArtifactEnvelope<P>> {
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
    let budget_counters = if stage_index == VERIFY {
        vec![(static_id(SOLVER_BUDGET_KEY), resolved.budget_ms)]
    } else {
        Vec::new()
    };
    shell.stage_event(StageEvent {
        candidate_id: resolved.pipeline_id.clone(),
        component_id: resolved.components[stage_index].clone(),
        stage: static_id(STAGE_KINDS[stage_index]),
        started_at,
        ended_at,
        duration_ms,
        input_hashes,
        output_hashes,
        outcome,
        diagnostics,
        budget_counters,
    });
    envelope
}

/// Envelope one group-stage payload under the runner's fixed §4.4 fields:
/// `schema.<kind>` schema id, `<artifact-id>` minted by the caller, content
/// and policy hashes computed here.
#[allow(clippy::too_many_arguments)]
fn envelope<P: Canonical>(
    artifact_id: String,
    kind: &str,
    producer: Producer,
    input_hashes: Vec<Hash>,
    origin: Origin,
    authority: Authority,
    diagnostics: Vec<DiagnosticRecord>,
    payload: P,
) -> Result<ArtifactEnvelope<P>, CanonError> {
    Ok(ArtifactEnvelope {
        schema_id: Id::new(format!("schema.{kind}")).expect("schema.<kind> stays in the grammar"),
        artifact_id: Id::new(artifact_id)
            .expect("the runner mints artifact ids inside the Id grammar"),
        artifact_kind: static_id(kind),
        producer,
        input_hashes,
        content_hash: content_hash(&payload)?,
        canonicalization_policy_hash: canonicalization_policy_hash(),
        origin,
        authority,
        accepted_effects: vec![],
        trace_refs: vec![],
        diagnostics,
        runtime_metadata: vec![],
        payload,
    })
}

/// §4.3 canonical-set view of stage diagnostics: sorted by canonical bytes,
/// byte-identical duplicates collapsed — the envelope `diagnostics` field's
/// storage order.
fn canonical_diagnostic_set<'a>(
    diagnostics: impl IntoIterator<Item = &'a DiagnosticRecord>,
) -> Result<Vec<DiagnosticRecord>, CanonError> {
    let mut keyed: Vec<(Vec<u8>, DiagnosticRecord)> = diagnostics
        .into_iter()
        .map(|d| Ok((canonical_sort_key(d)?, d.clone())))
        .collect::<Result<_, CanonError>>()?;
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    keyed.dedup_by(|a, b| a.0 == b.0);
    Ok(keyed.into_iter().map(|(_, d)| d).collect())
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
/// stage and its subject — `document` for the per-document stages, `group`
/// for the group stages (§4.4 "schema, hash, canonicalization … fails").
fn stage_diagnostic(
    stage_index: usize,
    subject_key: &str,
    subject: &Id,
    reason: String,
) -> DiagnosticRecord {
    invalid_diagnostic(vec![
        (static_id(subject_key), subject.to_string()),
        (static_id("reason"), reason),
        (static_id("stage"), STAGE_KINDS[stage_index].to_owned()),
    ])
}

/// §4.4 producer for one stage execution. The toolchain manifest hash is
/// the zero placeholder until cli-runner.4.1b.2 mints run manifests.
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

    /// Strict-read one landed artifact envelope at `rel` under the run
    /// directory and re-check its mechanical invariants; the §8.5 item 3
    /// per-artifact property, asserted from the consumer side.
    fn strict_at<P: Canonical + CanonRead>(out: &Path, rel: &str) -> ArtifactEnvelope<P> {
        let path = out.join(rel);
        let envelope: ArtifactEnvelope<P> = read_canonical(&std::fs::read(&path).unwrap())
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        envelope.validate().unwrap();
        envelope
    }

    /// [`strict_at`] over the document-artifact layout slot.
    fn strict<P: Canonical + CanonRead>(out: &Path, doc: &str, name: &str) -> ArtifactEnvelope<P> {
        strict_at(out, &format!("artifacts/{doc}/{name}.json"))
    }

    // The unit gate: the document stages over the three fixtures land the
    // twelve §8.3 document artifacts, every one strict-read clean with its
    // input hashes chaining the §8.4 stage order, and the event stream
    // carries one clean stage event per execution before the command event.
    #[test]
    fn document_stages_land_strict_artifacts_over_the_fixtures() {
        let (result, events, diagnostics, out, _tmp) = executed(&repo_root(), "exp.m1_spine");

        // The full §8.3 chain through verify completes clean.
        assert_eq!(result.outcome, Outcome::Ok);
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

        // 4 stage events per document, compile+verify per group, the
        // run-scoped trace and report stages, then the closing command
        // event.
        assert_eq!(events.len(), 19);
        for (n, event) in events.iter().enumerate() {
            assert_eq!(event.event_id, format!("event.{n}").parse::<Id>().unwrap());
            assert_eq!(event.logical_time, n as u64);
            assert_eq!(event.run_id, static_id("m1"));
        }
        for (d, doc) in DOC_IDS.iter().enumerate() {
            for (s, kind) in STAGE_KINDS[..4].iter().enumerate() {
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
        // Group events in evaluation order: conflict then null, compile
        // then verify; only verify carries the §8.4 budget counter.
        for (g, base) in [12, 14].iter().enumerate() {
            let compile = &events[*base];
            assert_eq!(compile.stage, static_id("compile"), "group {g}");
            assert_eq!(compile.component_id, static_id("stage.m1.compile"));
            assert_eq!(compile.outcome, Outcome::Ok);
            assert_eq!(compile.input_hashes.len(), 2);
            assert_eq!(compile.output_hashes.len(), 1);
            assert!(compile.diagnostics.is_empty());
            assert!(compile.budget_counters.is_empty());
            let verify = &events[*base + 1];
            assert_eq!(verify.stage, static_id("verify"), "group {g}");
            assert_eq!(verify.component_id, static_id("stage.m1.verify"));
            assert_eq!(verify.outcome, Outcome::Ok);
            assert_eq!(verify.input_hashes, compile.output_hashes);
            assert_eq!(verify.output_hashes.len(), 1);
            assert!(verify.diagnostics.is_empty());
            assert_eq!(
                verify.budget_counters,
                vec![(static_id("solver_ms_per_query"), 10_000)]
            );
        }
        // The trace stage: the DAG node content-hash set as input — 19
        // hashed nodes (3 sources + 12 document artifacts + 4 group
        // artifacts; the report node is hashless) collapsing to 18 because
        // control's and guideline_b's segments artifacts are byte-identical
        // — and the landed pair as outputs.
        let trace = &events[16];
        assert_eq!(trace.stage, static_id("trace"));
        assert_eq!(trace.candidate_id, static_id("pipe.layered_ckcir_to_smt"));
        assert_eq!(trace.component_id, static_id("stage.m1.trace"));
        assert_eq!(trace.outcome, Outcome::Ok);
        assert_eq!(trace.input_hashes.len(), 18);
        assert_eq!(trace.output_hashes.len(), 2);
        assert!(trace.diagnostics.is_empty());
        assert!(trace.budget_counters.is_empty());
        // The report stage: every consumed envelope's content hash as
        // input — the trace pair, three source graphs, two verifier
        // results — and the landed report as output.
        let report = &events[17];
        assert_eq!(report.stage, static_id("report"));
        assert_eq!(report.candidate_id, static_id("pipe.layered_ckcir_to_smt"));
        assert_eq!(report.component_id, static_id("stage.m1.report"));
        assert_eq!(report.outcome, Outcome::Ok);
        assert_eq!(report.input_hashes.len(), 7);
        assert!(report.input_hashes.contains(&trace.output_hashes[0]));
        assert!(report.input_hashes.contains(&trace.output_hashes[1]));
        assert_eq!(report.output_hashes.len(), 1);
        assert!(report.diagnostics.is_empty());
        assert!(report.budget_counters.is_empty());
        assert!(out.join("report.json").exists());
        let command = &events[18];
        assert_eq!(command.stage, static_id("run"));
        assert_eq!(command.outcome, Outcome::Ok);
    }

    // The group stages over exp.m1_spine: compiled artifacts and verifier
    // results land strict-read clean with hashes chaining bundles →
    // compiled → results and every query body materialized byte-identical
    // under smt/; the §8.6 thread yields the cross-document contradiction
    // in the conflict group and the disjoint-interval documented null in
    // the null group.
    #[test]
    fn group_stages_compile_and_verify_the_fixture_groups() {
        use ckc_smt::{CompiledArtifact, SolverVerdict, VerifierCategory};

        let (_result, _events, _diagnostics, out, _tmp) = executed(&repo_root(), "exp.m1_spine");
        let a: ArtifactEnvelope<IrBundle> = strict(&out, "fixture.m1_guideline_a", "ir_bundle");
        let b: ArtifactEnvelope<IrBundle> = strict(&out, "fixture.m1_guideline_b", "ir_bundle");
        let control: ArtifactEnvelope<IrBundle> = strict(&out, "fixture.m1_control", "ir_bundle");

        // Per group: envelope identity/chaining pins, plan and assertion
        // map shape, and byte-identical smt materialization.
        for (gid, members, rules) in [
            (
                "group.m1_conflict",
                [&a, &b],
                ["a", "b"].map(|d| format!("fixture.m1_guideline_{d}.rule.0")),
            ),
            (
                "group.m1_null",
                [&a, &control],
                [
                    "fixture.m1_control.rule.0".to_owned(),
                    "fixture.m1_guideline_a.rule.0".to_owned(),
                ],
            ),
        ] {
            let compiled: ArtifactEnvelope<CompiledArtifact> =
                strict_at(&out, &format!("groups/{gid}/compiled.json"));
            assert_eq!(compiled.schema_id, static_id("schema.compiled"), "{gid}");
            assert_eq!(
                compiled.artifact_id,
                format!("{gid}.compiled").parse().unwrap()
            );
            assert_eq!(compiled.artifact_kind, static_id("compiled"));
            assert_eq!(
                compiled.producer.component_id,
                static_id("stage.m1.compile")
            );
            assert_eq!(compiled.origin, Origin::DeterministicCompiler);
            assert_eq!(compiled.authority, Authority::CompilerAuthority);
            assert_eq!(
                compiled.input_hashes,
                sorted(&members.map(|m| m.content_hash.clone()))
            );
            assert!(compiled.diagnostics.is_empty());
            assert!(compiled.payload.diagnostics.is_empty());

            let gsuf = gid.strip_prefix("group.").unwrap();
            assert_eq!(compiled.payload.query_plan.len(), 1);
            assert_eq!(
                compiled.payload.query_plan[0].pair_id,
                format!("q.{gsuf}.pair1").parse().unwrap()
            );
            assert_eq!(compiled.payload.query_bodies.len(), 2);
            let mut expected_names: Vec<Id> = Vec::new();
            for prefix in ["a", "ctx"] {
                for rule in &rules {
                    expected_names.push(format!("{prefix}.{rule}").parse().unwrap());
                }
            }
            expected_names.sort();
            let names: Vec<Id> = compiled
                .payload
                .assertion_map
                .iter()
                .map(|(name, _)| name.clone())
                .collect();
            assert_eq!(names, expected_names, "{gid}");

            let smt_dir = out.join(format!("groups/{gid}/smt"));
            let mut files: Vec<String> = std::fs::read_dir(&smt_dir)
                .unwrap()
                .map(|e| e.unwrap().file_name().into_string().unwrap())
                .collect();
            files.sort();
            assert_eq!(
                files,
                [
                    format!("q.{gsuf}.pair1.deontic.smt2"),
                    format!("q.{gsuf}.pair1.overlap.smt2")
                ]
            );
            for body in &compiled.payload.query_bodies {
                let bytes = std::fs::read(smt_dir.join(format!("{}.smt2", body.query_id))).unwrap();
                assert_eq!(bytes, body.body.as_bytes(), "{}", body.query_id);
            }

            let results: ArtifactEnvelope<ckc_smt::VerifierResults> =
                strict_at(&out, &format!("groups/{gid}/verifier_results.json"));
            assert_eq!(results.schema_id, static_id("schema.verifier_results"));
            assert_eq!(
                results.artifact_id,
                format!("{gid}.verifier_results").parse().unwrap()
            );
            assert_eq!(results.artifact_kind, static_id("verifier_results"));
            assert_eq!(results.producer.component_id, static_id("stage.m1.verify"));
            assert_eq!(results.origin, Origin::AdapterGenerated);
            assert_eq!(results.authority, Authority::VerifierAuthority);
            assert_eq!(results.input_hashes, vec![compiled.content_hash.clone()]);
            assert!(results.diagnostics.is_empty());
            results.payload.validate().unwrap();
            for r in &results.payload.results {
                assert!(r.diagnostics.is_empty(), "{gid} {}", r.query_id);
                assert_eq!(r.solver_identity.solver_id, static_id("z3"));
            }
        }

        // Conflict group: Q1 sat with the overlap witness, Q2 unsat with
        // the cross-document core — the §8.6 finding.
        let conflict: ArtifactEnvelope<ckc_smt::VerifierResults> =
            strict_at(&out, "groups/group.m1_conflict/verifier_results.json");
        let rs = &conflict.payload.results;
        assert_eq!(rs.len(), 2);
        assert_eq!(
            rs[0].query_id,
            "q.m1_conflict.pair1.overlap".parse().unwrap()
        );
        assert_eq!(rs[0].category, VerifierCategory::SemanticNoConflict);
        assert_eq!(rs[0].verdict, Some(SolverVerdict::Sat));
        assert!(rs[0].model.is_some());
        assert_eq!(rs[0].unsat_core, None);
        assert_eq!(
            rs[1].query_id,
            "q.m1_conflict.pair1.deontic".parse().unwrap()
        );
        assert_eq!(rs[1].category, VerifierCategory::SemanticContradiction);
        assert_eq!(rs[1].verdict, Some(SolverVerdict::Unsat));
        assert_eq!(rs[1].model, None);
        assert_eq!(
            rs[1].unsat_core,
            Some(vec![
                "a.fixture.m1_guideline_a.rule.0".parse().unwrap(),
                "a.fixture.m1_guideline_b.rule.0".parse().unwrap(),
            ])
        );

        // Null group: the disjoint-interval Q1 answers unsat, closing the
        // pair as the documented null result — no Q2 run, no witness.
        let null: ArtifactEnvelope<ckc_smt::VerifierResults> =
            strict_at(&out, "groups/group.m1_null/verifier_results.json");
        let rs = &null.payload.results;
        assert_eq!(rs.len(), 1);
        assert_eq!(rs[0].query_id, "q.m1_null.pair1.overlap".parse().unwrap());
        assert_eq!(rs[0].category, VerifierCategory::SemanticNoConflict);
        assert_eq!(rs[0].verdict, Some(SolverVerdict::Unsat));
        assert_eq!(rs[0].model, None);
        assert_eq!(rs[0].unsat_core, None);
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
    /// the pipeline declares one component per [`STAGE_KINDS`] entry.
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
    stages:
      [stage.t.extract, stage.t.segment, stage.t.normalize, stage.t.assemble,
       stage.t.compile, stage.t.verify, stage.t.trace, stage.t.report]
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
  - id: stage.t.compile
    kind: compile
    determinism: deterministic
    input_artifact_kinds: [ir_bundle]
    output_artifact_kinds: [compiled, smt_query]
  - id: stage.t.verify
    kind: verify
    determinism: deterministic
    input_artifact_kinds: [compiled, smt_query]
    output_artifact_kinds: [verifier_results]
  - id: stage.t.trace
    kind: trace
    determinism: deterministic
    input_artifact_kinds:
      [source_graph, segments, normalization, ir_bundle, compiled, verifier_results]
    output_artifact_kinds: [trace_bundle, lineage_index]
  - id: stage.t.report
    kind: report
    determinism: deterministic
    input_artifact_kinds:
      [source_graph, ir_bundle, compiled, verifier_results, trace_bundle, lineage_index]
    output_artifact_kinds: [report, run_manifest, replay_manifest]
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
    // stages and lands its artifacts; the group misses a member bundle, so
    // its compile stage fails rather than compiling a partial group, and
    // verify never runs. The trace stage still assembles and lands the pair
    // over what landed — the surviving document's chain, no group artifacts
    // — and the report stage closes the chain over it: no claims, so both
    // partitions are empty, while the corpus row and the ledger rollup
    // still document the degraded run.
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
        // paragraph) riding their stage events, then the group's compile
        // failure naming the bundle-less member.
        assert_eq!(diagnostics.len(), 4);
        assert!(
            diagnostics[0]
                .payload
                .iter()
                .any(|(_, v)| v.contains("gone.html")),
            "{diagnostics:?}"
        );
        assert!(
            diagnostics[3]
                .payload
                .iter()
                .any(|(_, v)| v.contains("fixture.gone")),
            "{diagnostics:?}"
        );

        assert_eq!(events.len(), 8);
        for (s, kind) in STAGE_KINDS[..4].iter().enumerate() {
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
        let compile_event = &events[4];
        assert_eq!(compile_event.stage, static_id("compile"));
        assert_eq!(compile_event.component_id, static_id("stage.t.compile"));
        assert_eq!(compile_event.outcome, Outcome::Invalid);
        assert_eq!(compile_event.diagnostics.len(), 1);
        assert!(compile_event.output_hashes.is_empty());
        let trace_event = &events[5];
        assert_eq!(trace_event.stage, static_id("trace"));
        assert_eq!(trace_event.component_id, static_id("stage.t.trace"));
        assert_eq!(trace_event.outcome, Outcome::Ok);
        // fixture.tiny's source + its four landed artifacts; the
        // bundle-less group contributes no nodes.
        assert_eq!(trace_event.input_hashes.len(), 5);
        assert_eq!(trace_event.output_hashes.len(), 2);
        // The report stage consumes the trace pair and the surviving
        // document's graph (the verdict-less group contributes nothing);
        // its rollup counts the whole four-record ledger.
        let report_event = &events[6];
        assert_eq!(report_event.stage, static_id("report"));
        assert_eq!(report_event.component_id, static_id("stage.t.report"));
        assert_eq!(report_event.outcome, Outcome::Ok);
        assert_eq!(report_event.input_hashes.len(), 3);
        assert_eq!(report_event.output_hashes.len(), 1);
        assert!(out.join("trace_bundle.json").exists());
        assert!(out.join("lineage_index.json").exists());
        assert!(!out.join("artifacts/fixture.gone").exists());
        assert!(!out.join("groups").exists());
        let bundle: ArtifactEnvelope<IrBundle> = strict(&out, "fixture.tiny", "ir_bundle");
        let source: ArtifactEnvelope<SourceGraph> = strict(&out, "fixture.tiny", "source_graph");
        bundle.payload.validate(&source.payload).unwrap();
        let report: ArtifactEnvelope<crate::report::Report> = strict_at(&out, "report.json");
        assert!(report.payload.findings.is_empty());
        assert!(report.payload.null_results.is_empty());
        assert!(report.payload.wording.is_empty());
        assert_eq!(report.payload.corpus_hashes.len(), 1);
        assert_eq!(report.payload.corpus_hashes[0].0, static_id("fixture.tiny"));
        assert_eq!(
            report
                .payload
                .diagnostics_summary
                .iter()
                .map(|(_, n)| n)
                .sum::<u64>(),
            4
        );
    }

    // Budget resolution: an experiment without the §8.4 per-query solver
    // budget key is one command-scope diagnostic and no stage runs.
    #[test]
    fn missing_solver_budget_stops_resolution() {
        let root = tempfile::tempdir().unwrap();
        write_tiny_root(root.path());
        std::fs::write(
            root.path().join("registry/experiments.yaml"),
            "\
- id: exp.tiny
  pipeline: pipe.tiny
  fixture_groups:
    - group_id: group.t
      fixtures: [fixture.tiny]
  seed: 1
  budget: {}
  expected_outcomes: corpus/gold/t.yaml
",
        )
        .unwrap();

        let (result, events, diagnostics, out, _tmp) = executed(root.path(), "exp.tiny");
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(events.len(), 1);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .payload
                .iter()
                .any(|(_, v)| v.contains("solver_ms_per_query")),
            "{diagnostics:?}"
        );
        assert!(!out.join("artifacts").exists());
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
