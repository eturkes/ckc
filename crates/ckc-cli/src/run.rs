//! SPEC §3 `ckc run` (cli-runner.2a + .2b + .3a.3 + .4.1b): resolve the
//! experiment through the §8.4 registries, drive each corpus document through
//! extract → segment → normalize → assemble into
//! `artifacts/<doc-id>/{source_document_graph,segments,normalization,ir_bundle}.json`,
//! drive each test_source group through compile → verify into
//! `groups/<gid>/{compiled.json,verifier_results.json,smt/<query-id>.smt2}`,
//! assemble the run-scoped §7.1 trace pair over every landed artifact
//! into `trace_bundle.json` + `lineage_index.json` at the run root, then
//! close with the §8.3 report processing_stage: `report.json` (§7.2), its rendered
//! `report_en.md` view, and the §5/§4.6 provenance pair `manifest.json` +
//! `replay_manifest.json` over the run's recorded state
//! — every wrapper written as §4.3 canonical bytes and strict-read back at
//! the write boundary ([`land`]), every smt file byte-identical to its
//! [`ckc_smt::QueryBody`] body, every bare record re-read equal
//! ([`land_record`]). Each attempted processing_stage records exactly one
//! §4.6 processing_stage event carrying the artifact's wrapper diagnostics (or the
//! failure diagnostic); the §4.4 total outcome is the severity fold over
//! every event and command-scope diagnostic.
//!
//! Failure scoping: registry resolution, lexicon loading, solver-adapter
//! construction, and corpus-file reads are command-scope
//! ([`Shell::diagnostic`]); a processing_stage failure rides its processing_stage event and skips
//! the document's (or group's) remaining processing_stages, leaving other documents
//! and groups to proceed (§4.4 valid-remainder rule). A group whose member
//! bundle is missing fails its compile processing_stage rather than compiling a
//! partial group: a cross-document verdict over fewer documents than the
//! group declares would document a no-conflict result the test_sources never earned.
//! Producer values are runner-owned: pipeline_id = the experiment's pipeline,
//! pipeline_step_id = the registry processing_stage entry, toolchain manifest hash = the
//! §4.4 raw-byte hash of [`TOOLCHAIN_FILE`], read once at resolution and
//! shared verbatim with the §5/§4.6 manifests.

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use ckc_core::{
    ArtifactWrapper, CanonError, CanonRead, Canonical, ClinicalIr, CorpusEntry, DataClass,
    DiagnosticCode, DiagnosticRecord, EvidenceStatus, Hash, Id, IrBundle, Normalization, Origin,
    Outcome, Producer, RunPlan, SegmentIr, SolverIdentity, SourceDocumentGraph, TestSourceGroup,
    assemble, canonical_payload_bytes, canonical_sort_key, canonicalization_policy_hash,
    content_hash, hash_bytes, parse_candidates, parse_corpora, parse_experiments,
    read_strict_canonical,
};
use ckc_smt::{
    QueryBody, SmtLogic, VerifierResults, Z3Adapter, compile, verify, verify_query_pairs,
};

use crate::cassette::{CassetteKey, CassetteStore};
use crate::extract::{ExtractConfig, extract};
use crate::manifests::{ManifestInputs, assemble_manifests};
use crate::model_fill::{FillReject, FillSource, model_fill};
use crate::normalize::{Lexicon, load_lexicon, normalize};
use crate::registry_check::{invalid_diagnostic, load};
use crate::report::{Report, assemble_report, render_markdown};
use crate::segment::segment;
use crate::shell::{
    ProcessingStageClock, ProcessingStageEvent, Shell, processing_stage_clock, static_id,
};
use crate::trace::{DocTrace, GroupTrace, LineageIndex, TraceBundle, assemble_trace};

/// §5 lexicon reference file the normalize processing_stage consumes (module doc in
/// [`crate::normalize`]), read from the invocation root like the registries.
const LEXICON_FILE: &str = "corpus/lexicon/ja_core.yaml";

/// §4.4 toolchain manifest in force — the workspace's pinned-toolchain
/// file, raw-byte hashed into every producer and both §5/§4.6 manifests.
const TOOLCHAIN_FILE: &str = "rust-toolchain.toml";

/// §8.4 corpus registry; its raw bytes hash into the manifests'
/// `corpus_hash` (the §5 "content hash versioning the corpus in force").
const CORPORA_FILE: &str = "registry/corpora.yaml";

/// The workspace lockfile, raw-byte hashed under the manifests'
/// `lockfile_hashes` key `cargo.lock`.
const LOCKFILE: &str = "Cargo.lock";

/// §5 `git_commit`: the repository commit this binary was built at, baked
/// in by the ckc-cli build script (provenance of the build, not of the
/// invocation directory).
const GIT_COMMIT: &str = env!("CKC_GIT_COMMIT");

/// The eight §8.3 processing_stages this module drives, in chain order, spelled as the
/// registry `kind` tokens the pipeline's processing_stage entries declare: four
/// per-document processing_stages, the two per-group processing_stages, then the run-scoped
/// trace and report processing_stages.
const PROCESSING_STAGE_KINDS: [&str; 8] = [
    "extract",
    "segment",
    "normalize",
    "assemble",
    "compile",
    "verify",
    "trace",
    "report",
];

/// [`PROCESSING_STAGE_KINDS`] indices of the group processing_stages and the run-scoped pair.
const COMPILE: usize = 4;
const VERIFY: usize = 5;
const TRACE: usize = 6;
const REPORT: usize = 7;

/// The direct route's `verify_smt` slot in `pipe.m2_direct_smt`'s 4-stage step
/// list (extract, segment, model_fill_smt, verify_smt) — distinct from the M1
/// [`VERIFY`] slot (5), which is inert padding in the direct fixture's `[Id; 8]`.
const DIRECT_VERIFY: usize = 3;

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
    // §7.2's lexicon hash rides the raw reference-file bytes (§4.4: the
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
    let mut graphs: Vec<ArtifactWrapper<SourceDocumentGraph>> = Vec::new();
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
    // Run-scoped chain: a trace-processing_stage failure stops it before the report,
    // the same first-failure rule the document chain runs under.
    let Some((bundle, lineage)) = trace_processing_stage(&docs, &groups, &resolved, shell) else {
        return;
    };
    report_processing_stage(
        root,
        &docs,
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
/// its pipeline step ids, the unique corpus documents across the test_source
/// groups in first-appearance order, the groups themselves in evaluation
/// order, the per-query solver resource limit, the §5 plan the run executes, and
/// the toolchain manifest hash every producer carries.
struct Resolved {
    pipeline_id: Id,
    /// Pipeline step ids parallel to [`PROCESSING_STAGE_KINDS`].
    pipeline_step_ids: [Id; 8],
    documents: Vec<CorpusEntry>,
    groups: Vec<TestSourceGroup>,
    /// §8.4 `solver_ms_per_query` budget value.
    budget_ms: u64,
    /// §5 run plan built from the experiment entry; its content hash is
    /// the manifest's `run_plan_hash`.
    plan: RunPlan,
    /// §4.4 raw-byte hash of [`TOOLCHAIN_FILE`].
    toolchain_manifest_hash: Hash,
}

/// Resolve `experiment_id` against the §8.4 registry surface. Whole-set
/// semantic validation is `ckc registry check`'s job; resolution diagnoses
/// exactly the references this run needs, each failure one command-scope
/// `schema_invalid` diagnostic. The toolchain manifest read rides last:
/// every producer (and later both manifests) carries its hash, so a run
/// that cannot attest its toolchain mints nothing.
fn resolve(root: &Path, experiment_id: &Id, shell: &mut Shell) -> Option<Resolved> {
    let corpora = load(root, CORPORA_FILE, parse_corpora, shell);
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
    // M2 generalizes the experiment to a pipeline set + §7.3 baseline; multi-route
    // execution lands in run-m2.1. Today the run drives the single baseline pipeline.
    let Some(baseline) = experiment.baseline() else {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("reason"),
            format!("experiment {experiment_id} has no valid pipeline binding"),
        )]));
        return None;
    };
    let Some(pipeline) = candidates.pipelines.iter().find(|p| p.id == *baseline) else {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("reason"),
            format!("experiment {experiment_id} names undefined pipeline {baseline}"),
        )]));
        return None;
    };

    let mut pipeline_step_ids: Vec<Id> = Vec::with_capacity(PROCESSING_STAGE_KINDS.len());
    for kind in PROCESSING_STAGE_KINDS {
        let found = pipeline
            .processing_stages
            .iter()
            .find_map(|processing_stage_id| {
                candidates
                    .processing_stages
                    .iter()
                    .find(|s| s.id == *processing_stage_id && s.kind == static_id(kind))
            });
        match found {
            Some(processing_stage) => pipeline_step_ids.push(processing_stage.id.clone()),
            None => {
                shell.diagnostic(invalid_diagnostic(vec![(
                    static_id("reason"),
                    format!(
                        "pipeline {} declares no {kind} processing_stage entry",
                        pipeline.id
                    ),
                )]));
                return None;
            }
        }
    }
    let pipeline_step_ids: [Id; 8] = pipeline_step_ids
        .try_into()
        .expect("the loop pushes one pipeline step id per processing_stage kind");

    let Some(&budget_ms) = experiment.budget.get(&static_id(SOLVER_BUDGET_KEY)) else {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("reason"),
            format!("experiment {experiment_id} declares no {SOLVER_BUDGET_KEY} budget"),
        )]));
        return None;
    };

    let mut documents: Vec<CorpusEntry> = Vec::new();
    let mut unresolved = false;
    for group in &experiment.test_source_groups {
        for test_source in &group.test_sources {
            if documents.iter().any(|d| d.id == *test_source) {
                continue;
            }
            match corpora.iter().find(|c| c.id == *test_source) {
                Some(entry) => documents.push(entry.clone()),
                None => {
                    shell.diagnostic(invalid_diagnostic(vec![(
                        static_id("reason"),
                        format!(
                            "group {} names test_source {test_source} undefined in registry/corpora.yaml",
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
    let toolchain_manifest_hash = match std::fs::read(root.join(TOOLCHAIN_FILE)) {
        Ok(bytes) => hash_bytes(&bytes),
        Err(e) => {
            shell.diagnostic(invalid_diagnostic(vec![
                (static_id("file"), TOOLCHAIN_FILE.to_owned()),
                (static_id("reason"), format!("read {TOOLCHAIN_FILE}: {e}")),
            ]));
            return None;
        }
    };
    Some(Resolved {
        pipeline_id: pipeline.id.clone(),
        pipeline_step_ids,
        documents,
        groups: experiment.test_source_groups.clone(),
        budget_ms,
        plan: RunPlan {
            experiment_id: experiment.id.clone(),
            test_source_groups: experiment
                .test_source_groups
                .iter()
                .map(|g| g.group_id.clone())
                .collect(),
            pipelines: vec![baseline.clone()],
            seed: experiment.seed,
            budget: experiment
                .budget
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
        },
        toolchain_manifest_hash,
    })
}

/// Drive one corpus document through the four document processing_stages. Every
/// attempted processing_stage lands exactly one processing_stage event; the first failure stops
/// this document and leaves the rest of the run to proceed. Returns the
/// document's [`DocTrace`] — every landing recorded as it happens, the
/// bundle wrapper riding whole as the group processing_stages' input — beside its
/// landed source-graph wrapper when extract succeeded (the report processing_stage's
/// quoted-span source), or `None` when the corpus file itself was
/// unreadable (command-scope diagnostic: without source bytes there is no
/// hash to ground a trace node).
fn document_pipeline(
    root: &Path,
    entry: &CorpusEntry,
    resolved: &Resolved,
    lexicon: &Lexicon,
    shell: &mut Shell,
) -> Option<(DocTrace, Option<ArtifactWrapper<SourceDocumentGraph>>)> {
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
        test_source_path: entry.path.clone(),
        source_hash: hash_bytes(&html),
        source_document_graph: None,
        segments: None,
        normalization: None,
        bundle: None,
    };
    let dir = format!("artifacts/{}", entry.id);
    let mut graph: Option<ArtifactWrapper<SourceDocumentGraph>> = None;

    'chain: {
        let clock = processing_stage_clock();
        let config = ExtractConfig {
            document_id: entry.id.clone(),
            source_family: static_id("synthetic_test_source_html"),
            provenance: entry.provenance,
            data_class: DataClass::None,
            producer: producer(resolved, 0),
        };
        let built = extract(&html, &config)
            .map_err(|e| processing_stage_diagnostic(0, "document", &entry.id, e.to_string()));
        let rel = format!("{dir}/source_document_graph.json");
        let Some(source) =
            close_processing_stage(shell, resolved, 0, clock, Vec::new(), &rel, built)
        else {
            break 'chain;
        };
        trace.source_document_graph =
            Some((source.artifact_id.clone(), source.content_hash.clone()));
        graph = Some(source.clone());

        let clock = processing_stage_clock();
        let built = segment(&source, &producer(resolved, 1))
            .map_err(|e| processing_stage_diagnostic(1, "document", &entry.id, e.to_string()));
        let rel = format!("{dir}/segments.json");
        let inputs = vec![source.content_hash.clone()];
        let Some(segments) = close_processing_stage(shell, resolved, 1, clock, inputs, &rel, built)
        else {
            break 'chain;
        };
        trace.segments = Some((segments.artifact_id.clone(), segments.content_hash.clone()));

        let clock = processing_stage_clock();
        let built = normalize(&source, &segments, lexicon, &producer(resolved, 2))
            .map_err(|e| processing_stage_diagnostic(2, "document", &entry.id, e.to_string()));
        let rel = format!("{dir}/normalization.json");
        let inputs = vec![source.content_hash.clone(), segments.content_hash.clone()];
        let Some(normalization) =
            close_processing_stage(shell, resolved, 2, clock, inputs, &rel, built)
        else {
            break 'chain;
        };
        trace.normalization = Some((
            normalization.artifact_id.clone(),
            normalization.content_hash.clone(),
        ));

        let clock = processing_stage_clock();
        let built = assemble_bundle(entry, resolved, &source, &segments, &normalization);
        let rel = format!("{dir}/ir_bundle.json");
        let inputs = vec![
            source.content_hash.clone(),
            segments.content_hash.clone(),
            normalization.content_hash.clone(),
        ];
        trace.bundle = close_processing_stage(shell, resolved, 3, clock, inputs, &rel, built);
    }
    Some((trace, graph))
}

/// Drive one test_source group through compile → verify. Compile loads the
/// members' landed bundles (all must be present — see the module doc's
/// partial-group rule), compiles their (FormalIR, NormIR) pairs into the
/// wrapped [`ckc_smt::CompiledArtifact`] at `groups/<gid>/compiled.json`,
/// and materializes every planned query body byte-identical at
/// `groups/<gid>/smt/<query-id>.smt2`; verify drives the compiled plan
/// through the solver adapter under the experiment's per-query budget into
/// `groups/<gid>/verifier_results.json`. One processing_stage event each; a compile
/// failure skips the group's verify and leaves other groups to proceed.
/// Returns the group's [`GroupTrace`]: the §8.4 member set plus each group
/// landing that happened, riding whole.
fn group_pipeline(
    group: &TestSourceGroup,
    docs: &[DocTrace],
    resolved: &Resolved,
    adapter: &Z3Adapter,
    shell: &mut Shell,
) -> GroupTrace {
    let gid = &group.group_id;
    let dir = format!("groups/{gid}");
    let mut trace = GroupTrace {
        group_id: gid.clone(),
        test_sources: group.test_sources.clone(),
        compiled: None,
        verifier_results: None,
    };

    let clock = processing_stage_clock();
    let mut members: Vec<&ArtifactWrapper<IrBundle>> = Vec::with_capacity(group.test_sources.len());
    for test_source in &group.test_sources {
        let bundle = docs
            .iter()
            .find(|d| d.document_id == *test_source)
            .and_then(|d| d.bundle.as_ref());
        match bundle {
            Some(bundle) => members.push(bundle),
            None => {
                let built = Err(processing_stage_diagnostic(
                    COMPILE,
                    "group",
                    gid,
                    format!("member {test_source} landed no ir_bundle artifact"),
                ));
                finish_processing_stage::<IrBundle>(
                    shell,
                    resolved,
                    COMPILE,
                    clock,
                    Vec::new(),
                    built,
                );
                return trace;
            }
        }
    }
    let (compiled, verifier_results) =
        compile_verify_group(gid, &dir, &members, clock, resolved, adapter, shell);
    trace.compiled = compiled;
    trace.verifier_results = verifier_results;
    trace
}

/// The compile → verify back end over member [`IrBundle`]s, split from
/// [`group_pipeline`] so a route stage can feed its own validated bundles +
/// artifact `dir`. The caller fixes `dir` and opens the COMPILE `clock` ahead
/// of the timed body, so the compile interval spans the same work as the inline
/// form; opens a fresh clock for verify. Each tuple slot is `None` on that
/// processing_stage's failure; a compile failure skips verify.
fn compile_verify_group(
    group_id: &Id,
    dir: &str,
    members: &[&ArtifactWrapper<IrBundle>],
    clock: ProcessingStageClock,
    resolved: &Resolved,
    adapter: &Z3Adapter,
    shell: &mut Shell,
) -> (
    Option<ArtifactWrapper<ckc_smt::CompiledArtifact>>,
    Option<ArtifactWrapper<VerifierResults>>,
) {
    let gid = group_id;
    let inputs: Vec<Hash> = members.iter().map(|m| m.content_hash.clone()).collect();

    let artifact = compile(
        gid,
        members.iter().map(|m| (&m.payload.formal, &m.payload.norm)),
    );
    let built = artifact
        .validate()
        .map_err(|e| {
            processing_stage_diagnostic(COMPILE, "group", gid, format!("compiled artifact: {e}"))
        })
        .and_then(|()| {
            let diagnostics = canonical_diagnostic_set(&artifact.diagnostics)
                .map_err(|e| processing_stage_diagnostic(COMPILE, "group", gid, e.to_string()))?;
            wrapper(
                format!("{gid}.compiled"),
                "compiled",
                producer(resolved, COMPILE),
                inputs.clone(),
                Origin::DeterministicCompiler,
                EvidenceStatus::CompilerEvidenceStatus,
                diagnostics,
                artifact,
            )
            .map_err(|e| processing_stage_diagnostic(COMPILE, "group", gid, e.to_string()))
        });
    let landed = built
        .and_then(|env| land(shell, &format!("{dir}/compiled.json"), env))
        .and_then(|env| {
            materialize_queries(shell, dir, &env)?;
            Ok(env)
        });
    let Some(compiled) = finish_processing_stage(shell, resolved, COMPILE, clock, inputs, landed)
    else {
        return (None, None);
    };

    let clock = processing_stage_clock();
    let results = verify(
        adapter,
        &compiled.payload,
        Duration::from_millis(resolved.budget_ms),
    );
    let wrapped = VerifierResults { results };
    let built = wrapped
        .validate()
        .map_err(|e| {
            processing_stage_diagnostic(VERIFY, "group", gid, format!("verifier results: {e}"))
        })
        .and_then(|()| {
            let diagnostics =
                canonical_diagnostic_set(wrapped.results.iter().flat_map(|r| &r.diagnostics))
                    .map_err(|e| {
                        processing_stage_diagnostic(VERIFY, "group", gid, e.to_string())
                    })?;
            wrapper(
                format!("{gid}.verifier_results"),
                "verifier_results",
                producer(resolved, VERIFY),
                vec![compiled.content_hash.clone()],
                Origin::ExternalAdapterGenerated,
                EvidenceStatus::VerifierEvidenceStatus,
                diagnostics,
                wrapped,
            )
            .map_err(|e| processing_stage_diagnostic(VERIFY, "group", gid, e.to_string()))
        });
    let landed = built.and_then(|env| land(shell, &format!("{dir}/verifier_results.json"), env));
    let verifier_results = finish_processing_stage(
        shell,
        resolved,
        VERIFY,
        clock,
        vec![compiled.content_hash.clone()],
        landed,
    );
    (Some(compiled), verifier_results)
}

/// The `route.single_ir` §4 acceptance closure over one model output: strict-read
/// the bytes as a [`ClinicalIr`] — a parse failure is a repairable
/// [`FillReject::Schema`] carrying the reason — then ground every cited upstream
/// id against the document's region and segment id-universes (the
/// [`IrBundle::validate`] reference checks, run here before assembly). A binding
/// or exception `region_id` outside `regions`, or a statement `source_segment_id`
/// outside `segments`, is a terminal [`FillReject::Grounding`] carrying the absent
/// ids; an empty absent set accepts and yields the parsed `ClinicalIr`. Closing
/// over pre-built id sets lets a route step (and its tests) classify a model
/// output with no live pipeline. [`single_ir_fill`] wires it into the route.
#[allow(dead_code)]
fn single_ir_accept<'a>(
    regions: &'a HashSet<&'a Id>,
    segments: &'a HashSet<&'a Id>,
) -> impl Fn(&[u8]) -> Result<ClinicalIr, FillReject> + 'a {
    move |bytes| {
        let clinical: ClinicalIr =
            read_strict_canonical(bytes).map_err(|e| FillReject::Schema(e.to_string()))?;
        let mut absent: Vec<Id> = Vec::new();
        for binding in &clinical.bindings {
            for region_id in &binding.region_ids {
                if !regions.contains(region_id) {
                    absent.push(region_id.clone());
                }
            }
        }
        for statement in &clinical.statements {
            for exception in &statement.exceptions {
                for region_id in &exception.region_ids {
                    if !regions.contains(region_id) {
                        absent.push(region_id.clone());
                    }
                }
            }
            for segment_id in &statement.source_segment_ids {
                if !segments.contains(segment_id) {
                    absent.push(segment_id.clone());
                }
            }
        }
        if absent.is_empty() {
            Ok(clinical)
        } else {
            Err(FillReject::Grounding(absent))
        }
    }
}

/// The single_ir route's per-document fill back end: drive a corpus document
/// through extract → segment, replay its committed model cassette through
/// [`model_fill`] under [`single_ir_accept`], and compile the accepted
/// [`ClinicalIr`] over the deterministic upstream into an [`IrBundle`] — the same
/// five-layer assembly [`assemble_bundle`] produces, but with the model's clinical
/// layer and a norm [`derive_norm_ir`](crate::rules::derive_norm_ir)-recomputed
/// over it in place of the deterministic normalizer's. The grounding scaffold is
/// the deterministic head: extract + segment mint the real region and segment ids
/// the accept closure grounds the model's references against, so a hallucinated
/// reference surfaces as `ai_hallucinated_source` rather than corrupting the
/// bundle. Each failure rides a shell diagnostic and yields `None`; wired into the
/// experiment run by run-m2.1.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn single_ir_fill(
    root: &Path,
    entry: &CorpusEntry,
    lexicon: &Lexicon,
    store: &CassetteStore,
    seed: u64,
    resolved: &Resolved,
    repair_limit: u32,
    shell: &mut Shell,
) -> Option<ArtifactWrapper<IrBundle>> {
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
    let config = ExtractConfig {
        document_id: entry.id.clone(),
        source_family: static_id("synthetic_test_source_html"),
        provenance: entry.provenance,
        data_class: DataClass::None,
        producer: producer(resolved, 0),
    };
    let source = match extract(&html, &config) {
        Ok(source) => source,
        Err(e) => {
            shell.diagnostic(processing_stage_diagnostic(
                0,
                "document",
                &entry.id,
                e.to_string(),
            ));
            return None;
        }
    };
    let segments = match segment(&source, &producer(resolved, 1)) {
        Ok(segments) => segments,
        Err(e) => {
            shell.diagnostic(processing_stage_diagnostic(
                1,
                "document",
                &entry.id,
                e.to_string(),
            ));
            return None;
        }
    };

    // Ground the model's references against the deterministic head's real ids
    // (§4 acceptance steps 4 + 5), then replay the committed cassette.
    let regions: HashSet<&Id> = source
        .payload
        .regions
        .iter()
        .map(|r| &r.region_id)
        .collect();
    let segment_ids: HashSet<&Id> = segments
        .payload
        .segments
        .iter()
        .map(|s| &s.segment_id)
        .collect();
    let accept = single_ir_accept(&regions, &segment_ids);
    let key = CassetteKey {
        route: static_id("route.single_ir"),
        source: entry.id.clone(),
        seed,
    };
    let fill = match model_fill(store, &key, FillSource::Replay, repair_limit, accept) {
        Ok(fill) => fill,
        Err(e) => {
            shell.diagnostic(invalid_diagnostic(vec![
                (static_id("cassette"), entry.id.to_string()),
                (static_id("reason"), e.to_string()),
                (static_id("processing_stage"), "model_fill".to_owned()),
            ]));
            return None;
        }
    };
    // Surface the §7.4 fill diagnostics (schema violations, terminal grounding or
    // repair-limit), then a terminal reject (no accepted target) ends the route.
    for diagnostic in &fill.diagnostics {
        shell.diagnostic(diagnostic.clone());
    }
    let clinical = fill.target?;

    // Deterministic tail mirroring [`assemble_bundle`], substituting the model's
    // clinical layer and a norm recomputed over it; the model-fill route runs no
    // normalizer, so the bundle diagnostics are the segments' alone — equal to M1's
    // segments ∪ normalization set because the normalizer adds none for grounded
    // output (route-single-ir.2b's reproduce-M1 gate proves the equality).
    let norm = crate::rules::derive_norm_ir(
        &source.payload.document.document_id,
        &clinical,
        &segments.payload,
        lexicon,
    );
    let doc = match ckc_core::DocIr::from_graph(&source.payload, source.diagnostics.clone()) {
        Ok(doc) => doc,
        Err(e) => {
            shell.diagnostic(processing_stage_diagnostic(
                3,
                "document",
                &entry.id,
                format!("doc layer: {e}"),
            ));
            return None;
        }
    };
    let diagnostics = match canonical_diagnostic_set(segments.diagnostics.iter()) {
        Ok(diagnostics) => diagnostics,
        Err(e) => {
            shell.diagnostic(processing_stage_diagnostic(
                3,
                "document",
                &entry.id,
                format!("diagnostic sort key: {e}"),
            ));
            return None;
        }
    };
    let bundle = match assemble(
        doc,
        segments.payload.clone(),
        clinical,
        norm,
        Vec::new(),
        diagnostics,
    ) {
        Ok(bundle) => bundle,
        Err(e) => {
            shell.diagnostic(processing_stage_diagnostic(
                3,
                "document",
                &entry.id,
                format!("assembly: {e}"),
            ));
            return None;
        }
    };
    if let Err(e) = bundle.validate(&source.payload) {
        shell.diagnostic(processing_stage_diagnostic(
            3,
            "document",
            &entry.id,
            format!("bundle invariant: {e}"),
        ));
        return None;
    }

    // Wrap; the single_ir route has no normalization wrapper, so the bundle cites
    // source + segments (run-m2.1 adds the replayed cassette hash for provenance).
    // These wrapper-level fields do not reach the payload-only `content_hash`.
    match wrapper(
        format!("{}.ir_bundle", entry.id),
        "ir_bundle",
        producer(resolved, 3),
        vec![source.content_hash.clone(), segments.content_hash.clone()],
        Origin::DeterministicCompiler,
        EvidenceStatus::MechanicalEvidenceStatus,
        Vec::new(),
        bundle,
    ) {
        Ok(wrapped) => Some(wrapped),
        Err(e) => {
            shell.diagnostic(processing_stage_diagnostic(
                3,
                "document",
                &entry.id,
                format!("content hash: {e}"),
            ));
            None
        }
    }
}

/// The direct_smt route's acceptance check: a shallow SMT surface-marker check only —
/// valid UTF-8 with a `(set-logic ...)` head and a `(check-sat)` command — mapping
/// any shortfall to a repairable [`FillReject::Schema`]. Unlike [`single_ir_accept`]
/// there is no [`FillReject::Grounding`]: the direct route emits SMT over the raw
/// guideline text and carries no source linkage, so the solver is the syntactic
/// authority — a marker-passing but unparseable query surfaces as `target_syntax_failure`
/// at verify (route-direct-smt.5), never here.
#[allow(dead_code)]
fn direct_smt_accept() -> impl Fn(&[u8]) -> Result<String, FillReject> {
    |bytes| {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| FillReject::Schema(format!("not utf-8: {e}")))?;
        if !text.trim_start().starts_with("(set-logic") {
            return Err(FillReject::Schema(
                "expected a (set-logic ...) head".to_owned(),
            ));
        }
        if !text.contains("(check-sat)") {
            return Err(FillReject::Schema(
                "expected a (check-sat) command".to_owned(),
            ));
        }
        Ok(text.to_owned())
    }
}

/// The direct_smt route's per-group fill back end: extract + segment each member for
/// provenance (references ungrounded — the direct route emits raw SMT, not an IR),
/// then replay the group's two committed model cassettes through [`model_fill`] under
/// [`direct_smt_accept`] — the overlap query keyed under `<gid>.overlap`, the deontic
/// query under `<gid>.deontic`, both at the base seed. The sources are role-namespaced
/// so a Q2 repair never aliases Q1's: [`model_fill`] reads attempt `i` under
/// `derive_seed(base, i)` on the one source, so a shared source would collide. Each
/// accepted body is wrapped as an `smt_query` [`ArtifactWrapper`] carrying the raw
/// model output ([`Origin::AiGenerated`] + [`EvidenceStatus::AcceptedEvidenceStatus`],
/// no external effects and no deterministic transform — distinct from single_ir's
/// mechanical `ir_bundle`). Returns the pair's two wrappers (each `content_hash` is what
/// the verdict tail cites as its `verify_smt` input) or `None` on a recorded failure;
/// wired into the experiment run by run-m2.1.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn direct_smt_fill(
    root: &Path,
    gid: &Id,
    members: &[&CorpusEntry],
    store: &CassetteStore,
    seed: u64,
    resolved: &Resolved,
    repair_limit: u32,
    shell: &mut Shell,
) -> Option<(ArtifactWrapper<QueryBody>, ArtifactWrapper<QueryBody>)> {
    // The cassette-role design mints exactly one (overlap, deontic) pair per group — the
    // shape of the M1 planner's single constraint-pair over a minimal pair of members.
    // Fail closed on any other cardinality so an expanded registry cannot silently emit a
    // two-query artifact that under-covers a 3+-member group's pairwise queries.
    if members.len() != 2 {
        shell.diagnostic(invalid_diagnostic(vec![
            (static_id("group"), gid.to_string()),
            (
                static_id("reason"),
                format!("expected a 2-member minimal pair, got {}", members.len()),
            ),
            (static_id("processing_stage"), "model_fill_smt".to_owned()),
        ]));
        return None;
    }
    // Provenance inputs: extract + segment each member (the M1-chain head, references
    // ungrounded). The wrapper cites these in member order; provenance-only, so they do
    // not reach the payload-only `content_hash`.
    let mut input_hashes: Vec<Hash> = Vec::new();
    for entry in members {
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
        let config = ExtractConfig {
            document_id: entry.id.clone(),
            source_family: static_id("synthetic_test_source_html"),
            provenance: entry.provenance,
            data_class: DataClass::None,
            producer: producer(resolved, 0),
        };
        let source = match extract(&html, &config) {
            Ok(source) => source,
            Err(e) => {
                shell.diagnostic(processing_stage_diagnostic(
                    0,
                    "document",
                    &entry.id,
                    e.to_string(),
                ));
                return None;
            }
        };
        let segments = match segment(&source, &producer(resolved, 1)) {
            Ok(segments) => segments,
            Err(e) => {
                shell.diagnostic(processing_stage_diagnostic(
                    1,
                    "document",
                    &entry.id,
                    e.to_string(),
                ));
                return None;
            }
        };
        input_hashes.push(source.content_hash.clone());
        input_hashes.push(segments.content_hash.clone());
    }

    // Replay the pair's two role-namespaced cassettes at the base seed, wrapping each
    // shallow-accepted body as the raw-AI `smt_query` the verdict tail consumes.
    let mut pair: Vec<ArtifactWrapper<QueryBody>> = Vec::new();
    for (role, logic) in [("overlap", SmtLogic::QfLra), ("deontic", SmtLogic::QfUf)] {
        let source = static_id(&format!("{gid}.{role}"));
        let key = CassetteKey {
            route: static_id("route.direct_smt"),
            source: source.clone(),
            seed,
        };
        let fill = match model_fill(
            store,
            &key,
            FillSource::Replay,
            repair_limit,
            direct_smt_accept(),
        ) {
            Ok(fill) => fill,
            Err(e) => {
                shell.diagnostic(invalid_diagnostic(vec![
                    (static_id("cassette"), source.to_string()),
                    (static_id("reason"), e.to_string()),
                    (static_id("processing_stage"), "model_fill_smt".to_owned()),
                ]));
                return None;
            }
        };
        // Surface §7.4 fill diagnostics (schema violations, repair-limit), then a
        // terminal reject (no accepted target) ends the route.
        for diagnostic in &fill.diagnostics {
            shell.diagnostic(diagnostic.clone());
        }
        let body = fill.target?;
        let payload = QueryBody {
            query_id: source,
            logic,
            body,
        };
        match wrapper(
            format!("{gid}.{role}.smt_query"),
            "smt_query",
            producer(resolved, 2),
            input_hashes.clone(),
            Origin::AiGenerated,
            EvidenceStatus::AcceptedEvidenceStatus,
            Vec::new(),
            payload,
        ) {
            Ok(wrapped) => pair.push(wrapped),
            Err(e) => {
                shell.diagnostic(invalid_diagnostic(vec![
                    (static_id("group"), gid.to_string()),
                    (static_id("artifact"), format!("{gid}.{role}.smt_query")),
                    (static_id("reason"), format!("wrap: {e}")),
                    (static_id("processing_stage"), "model_fill_smt".to_owned()),
                ]));
                return None;
            }
        }
    }
    let mut pair = pair.into_iter();
    let overlap = pair.next().expect("overlap query wrapped");
    let deontic = pair.next().expect("deontic query wrapped");
    Some((overlap, deontic))
}

/// The direct_smt route's per-group verdict tail: run the pair's two model-emitted
/// SMT bodies (route-direct-smt.3b's `smt_query` wrappers) through the shared
/// caller-minted verdict engine [`verify_query_pairs`] — no
/// [`CompiledArtifact`](ckc_smt::CompiledArtifact), since the direct route emits raw
/// SMT and builds no IR (the region-id wall) — then validate, land, and event the
/// `verifier_results`. The results wrapper cites the two `smt_query` wrapper
/// `content_hash`es (the upstream artifact, as single_ir's verify cites `compiled`).
///
/// The 4-stage `pipe.m2_direct_smt` places `verify_smt` at slot [`DIRECT_VERIFY`], so
/// the §4.6 event is minted here rather than via [`finish_processing_stage`]: that
/// derives the kind from `PROCESSING_STAGE_KINDS[3]` (`assemble`) and gates the
/// solver-budget counter on the M1 [`VERIFY`] slot (5, inert padding in the direct
/// fixture) — this stamps the `verify` kind, the slot-3 `m2.verify_smt` step id
/// ([`producer`] uses the same), and the solver budget counter unconditionally. Wired
/// into the experiment run by run-m2.1.
#[allow(dead_code)]
fn direct_smt_verify_group(
    gid: &Id,
    dir: &str,
    overlap: &ArtifactWrapper<QueryBody>,
    deontic: &ArtifactWrapper<QueryBody>,
    resolved: &Resolved,
    adapter: &Z3Adapter,
    shell: &mut Shell,
) -> Option<ArtifactWrapper<VerifierResults>> {
    // Gather the pair's hashes and bodies before the clock so only the solver run and
    // artifact production fall inside the timed interval (compile_verify_group's
    // discipline; the M2.14 clock-boundary lesson).
    let inputs = vec![overlap.content_hash.clone(), deontic.content_hash.clone()];
    let pairs = [(
        (
            overlap.payload.query_id.clone(),
            overlap.payload.body.clone(),
        ),
        (
            deontic.payload.query_id.clone(),
            deontic.payload.body.clone(),
        ),
    )];
    let clock = processing_stage_clock();
    let results = verify_query_pairs(adapter, &pairs, Duration::from_millis(resolved.budget_ms));
    let wrapped = VerifierResults { results };
    let built = wrapped
        .validate()
        .map_err(|e| {
            processing_stage_diagnostic(VERIFY, "group", gid, format!("verifier results: {e}"))
        })
        .and_then(|()| {
            let diagnostics =
                canonical_diagnostic_set(wrapped.results.iter().flat_map(|r| &r.diagnostics))
                    .map_err(|e| {
                        processing_stage_diagnostic(VERIFY, "group", gid, e.to_string())
                    })?;
            wrapper(
                format!("{gid}.verifier_results"),
                "verifier_results",
                producer(resolved, DIRECT_VERIFY),
                inputs.clone(),
                Origin::ExternalAdapterGenerated,
                EvidenceStatus::VerifierEvidenceStatus,
                diagnostics,
                wrapped,
            )
            .map_err(|e| processing_stage_diagnostic(VERIFY, "group", gid, e.to_string()))
        });
    let landed = built.and_then(|env| land(shell, &format!("{dir}/verifier_results.json"), env));

    // Emit the §4.6 verify event directly (see the doc comment): the direct pipeline's
    // slot-3 verify_smt cannot go through the index-coupled finish_processing_stage.
    let (started_at, ended_at, duration_ms) = clock.stop();
    let (outcome, diagnostics, output_hashes, verifier_results) = match landed {
        Ok(wrapper) => (
            severity(&wrapper.diagnostics),
            wrapper.diagnostics.clone(),
            vec![wrapper.content_hash.clone()],
            Some(wrapper),
        ),
        Err(diagnostic) => (diagnostic.outcome, vec![diagnostic], Vec::new(), None),
    };
    shell.processing_stage_event(ProcessingStageEvent {
        pipeline_id: resolved.pipeline_id.clone(),
        pipeline_step_id: resolved.pipeline_step_ids[DIRECT_VERIFY].clone(),
        processing_stage: static_id("verify"),
        started_at,
        ended_at,
        duration_ms,
        input_hashes: inputs,
        output_hashes,
        outcome,
        diagnostics,
        resource_counters: vec![(static_id(SOLVER_BUDGET_KEY), resolved.budget_ms)],
    });
    verifier_results
}

/// The §8.3 trace processing_stage, run once after the group loop: assemble the §7.1
/// pair over every landed artifact ([`assemble_trace`] skips absent
/// pieces), validate both payloads, and land them at the run root as
/// `trace_bundle.json` + `lineage_index.json`. Both wrappers carry the
/// DAG's node content-hash set as input hashes (each source's raw-byte
/// hash beside every landed wrapper hash; the hashless report node
/// contributes nothing). One processing_stage event covers the pair: both content
/// hashes as outputs, or the first failure diagnostic. Returns the landed
/// pair — the report processing_stage's input — or `None` on the recorded failure.
fn trace_processing_stage(
    docs: &[DocTrace],
    groups: &[GroupTrace],
    resolved: &Resolved,
    shell: &mut Shell,
) -> Option<(ArtifactWrapper<TraceBundle>, ArtifactWrapper<LineageIndex>)> {
    let clock = processing_stage_clock();
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
    let fail = |reason: String| processing_stage_diagnostic(TRACE, "run", &run_id, reason);

    let landed = bundle
        .validate()
        .map_err(|e| fail(format!("trace bundle: {e}")))
        .and_then(|()| {
            lineage
                .validate()
                .map_err(|e| fail(format!("lineage index: {e}")))
        })
        .and_then(|()| {
            let bundle = wrapper(
                "trace_bundle".to_owned(),
                "trace_bundle",
                producer(resolved, TRACE),
                input_hashes.clone(),
                Origin::DeterministicCompiler,
                EvidenceStatus::MechanicalEvidenceStatus,
                vec![],
                bundle,
            )
            .map_err(|e| fail(e.to_string()))?;
            let lineage = wrapper(
                "lineage_index".to_owned(),
                "lineage_index",
                producer(resolved, TRACE),
                input_hashes.clone(),
                Origin::DeterministicCompiler,
                EvidenceStatus::MechanicalEvidenceStatus,
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
        // Both wrappers are built with empty diagnostics (assembly raises
        // nothing of its own), so a landed pair is a clean processing_stage.
        Ok((bundle, lineage)) => (
            Outcome::Ok,
            Vec::new(),
            vec![bundle.content_hash.clone(), lineage.content_hash.clone()],
            Some((bundle, lineage)),
        ),
        Err(diagnostic) => (diagnostic.outcome, vec![diagnostic], Vec::new(), None),
    };
    shell.processing_stage_event(ProcessingStageEvent {
        pipeline_id: resolved.pipeline_id.clone(),
        pipeline_step_id: resolved.pipeline_step_ids[TRACE].clone(),
        processing_stage: static_id(PROCESSING_STAGE_KINDS[TRACE]),
        started_at,
        ended_at,
        duration_ms,
        input_hashes,
        output_hashes,
        outcome,
        diagnostics,
        resource_counters: Vec::new(),
    });
    pair
}

/// The §8.3 report processing_stage, the run-scoped chain's tail: snapshot the
/// shell's diagnostic ledger (every §7.4 record the run raised before this
/// processing_stage; the report cannot count records that do not yet exist), assemble
/// the §7.2 [`crate::report::Report`] over the landed trace pair, the
/// landed source-graph wrappers, the landed verifier results, the raw
/// lexicon-byte hash, and the adapter's live solver identity, validate it,
/// and land it at the run root as `report.json` — the path the DAG's
/// hashless sink node already names. The landed payload then renders as
/// `report_en.md` (read back byte-identical, the [`materialize_queries`]
/// discipline), and the §5/§4.6 provenance pair lands as `manifest.json` +
/// `replay_manifest.json` over [`manifest_inputs`]' gathered run state.
/// Input hashes are the §4.3 set of every consumed wrapper's content
/// hash; one processing_stage event closes the processing_stage with the report's content hash
/// (the manifests and the view attest or derive from accepted artifacts;
/// they carry no content hash of their own) or the first failure
/// diagnostic.
#[allow(clippy::too_many_arguments)]
fn report_processing_stage(
    root: &Path,
    docs: &[DocTrace],
    graphs: &[ArtifactWrapper<SourceDocumentGraph>],
    groups: &[GroupTrace],
    bundle: &ArtifactWrapper<TraceBundle>,
    lineage: &ArtifactWrapper<LineageIndex>,
    lexicon_hash: &Hash,
    solver_identity: &SolverIdentity,
    resolved: &Resolved,
    shell: &mut Shell,
) {
    let clock = processing_stage_clock();
    let ledger = shell.ledger().to_vec();
    let run_id = shell.run_id().clone();
    let fail = |reason: String| processing_stage_diagnostic(REPORT, "run", &run_id, reason);

    let results: Vec<&ArtifactWrapper<VerifierResults>> = groups
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
        wrapper(
            "report".to_owned(),
            "report",
            producer(resolved, REPORT),
            input_hashes.clone(),
            Origin::DeterministicCompiler,
            EvidenceStatus::MechanicalEvidenceStatus,
            vec![],
            report,
        )
        .map_err(|e| fail(e.to_string()))
    })
    .and_then(|report| land(shell, "report.json", report))
    .and_then(|report| {
        // §7.2 derived view: rendered from the read-back payload (disk
        // truth), landed beside the canonical record.
        let body = render_markdown(&report.payload);
        let path = shell
            .write_under("report_en.md", body.as_bytes())
            .map_err(|e| fail(format!("report_en.md: {e}")))?;
        let read_back =
            std::fs::read(&path).map_err(|e| fail(format!("report_en.md: read back: {e}")))?;
        if read_back != body.as_bytes() {
            return Err(fail(
                "report_en.md: read back diverges from the rendering".to_owned(),
            ));
        }
        Ok(report)
    })
    .and_then(|report| {
        let inputs = manifest_inputs(
            root,
            docs,
            groups,
            bundle,
            lineage,
            &report,
            lexicon_hash,
            solver_identity,
            resolved,
            shell,
        )
        .map_err(&fail)?;
        let (manifest, replay) =
            assemble_manifests(&inputs).map_err(|e| fail(format!("manifest assembly: {e}")))?;
        land_record(shell, "manifest.json", &manifest)?;
        land_record(shell, "replay_manifest.json", &replay)?;
        Ok(report)
    });

    let (started_at, ended_at, duration_ms) = clock.stop();
    let (outcome, diagnostics, output_hashes) = match landed {
        // The wrapper is built with empty diagnostics (assembly raises
        // nothing of its own), so a landed report is a clean processing_stage.
        Ok(report) => (Outcome::Ok, Vec::new(), vec![report.content_hash]),
        Err(diagnostic) => (diagnostic.outcome, vec![diagnostic], Vec::new()),
    };
    shell.processing_stage_event(ProcessingStageEvent {
        pipeline_id: resolved.pipeline_id.clone(),
        pipeline_step_id: resolved.pipeline_step_ids[REPORT].clone(),
        processing_stage: static_id(PROCESSING_STAGE_KINDS[REPORT]),
        started_at,
        ended_at,
        duration_ms,
        input_hashes,
        output_hashes,
        outcome,
        diagnostics,
        resource_counters: Vec::new(),
    });
}

/// Gather the run state the §5/§4.6 manifests attest, one value per
/// [`ManifestInputs`] fact (failures name their reason; the report processing_stage
/// scopes them): the resolved §5 plan and toolchain hash; the §4.6 replay
/// command reconstructed in semantic order from the experiment id and the
/// shell's `--out` token; the build-baked [`GIT_COMMIT`]; raw-byte hashes
/// of [`LOCKFILE`] and [`CORPORA_FILE`] read at the invocation root; the
/// lexicon hash and live solver identity the run already holds; `os` +
/// `arch` environment facts; input hashes = the corpus documents' raw
/// source-byte hashes; output hashes = every landed wrapper's content
/// hash — the document layers, the group pair, the trace pair, and the
/// report itself ([`assemble_manifests`] owns canonical ordering).
#[allow(clippy::too_many_arguments)]
fn manifest_inputs(
    root: &Path,
    docs: &[DocTrace],
    groups: &[GroupTrace],
    bundle: &ArtifactWrapper<TraceBundle>,
    lineage: &ArtifactWrapper<LineageIndex>,
    report: &ArtifactWrapper<Report>,
    lexicon_hash: &Hash,
    solver_identity: &SolverIdentity,
    resolved: &Resolved,
    shell: &Shell,
) -> Result<ManifestInputs, String> {
    let Some(out_dir) = shell.out_dir() else {
        return Err("manifests: run shell has no output directory".to_owned());
    };
    let mut command: Vec<String> = ["ckc", "run", "--experiment"].map(str::to_owned).to_vec();
    command.push(resolved.plan.experiment_id.to_string());
    command.push("--out".to_owned());
    command.push(out_dir.display().to_string());

    let lockfile = std::fs::read(root.join(LOCKFILE))
        .map_err(|e| format!("manifests: read {LOCKFILE}: {e}"))?;
    let corpora = std::fs::read(root.join(CORPORA_FILE))
        .map_err(|e| format!("manifests: read {CORPORA_FILE}: {e}"))?;

    let mut output_hashes: Vec<Hash> = vec![
        bundle.content_hash.clone(),
        lineage.content_hash.clone(),
        report.content_hash.clone(),
    ];
    for doc in docs {
        output_hashes.extend(doc.source_document_graph.iter().map(|(_, h)| h.clone()));
        output_hashes.extend(doc.segments.iter().map(|(_, h)| h.clone()));
        output_hashes.extend(doc.normalization.iter().map(|(_, h)| h.clone()));
        output_hashes.extend(doc.bundle.iter().map(|b| b.content_hash.clone()));
    }
    for group in groups {
        output_hashes.extend(group.compiled.iter().map(|c| c.content_hash.clone()));
        output_hashes.extend(
            group
                .verifier_results
                .iter()
                .map(|v| v.content_hash.clone()),
        );
    }

    Ok(ManifestInputs {
        plan: resolved.plan.clone(),
        command,
        git_commit: GIT_COMMIT.to_owned(),
        toolchain_manifest_hash: resolved.toolchain_manifest_hash.clone(),
        lockfile_hashes: vec![(static_id("cargo.lock"), hash_bytes(&lockfile))],
        corpus_hash: hash_bytes(&corpora),
        lexicon_hash: lexicon_hash.clone(),
        environment_profile: vec![
            (static_id("arch"), std::env::consts::ARCH.to_owned()),
            (static_id("os"), std::env::consts::OS.to_owned()),
        ],
        solver_identity: solver_identity.clone(),
        input_hashes: docs.iter().map(|d| d.source_hash.clone()).collect(),
        output_hashes,
    })
}

/// The manifests' write boundary: emit the bare §5/§4.6 record as
/// canonical bytes under `rel`, strict-read the file back, and require the
/// read-back value equal — [`land`]'s discipline for records that carry no
/// wrapper (the manifests attest wrappers; nothing wraps them).
fn land_record<P: Canonical + CanonRead + PartialEq>(
    shell: &Shell,
    rel: &str,
    record: &P,
) -> Result<(), DiagnosticRecord> {
    let fail = |reason: String| {
        invalid_diagnostic(vec![
            (static_id("artifact"), rel.to_owned()),
            (static_id("reason"), reason),
        ])
    };
    let bytes =
        canonical_payload_bytes(record).map_err(|e| fail(format!("canonical emission: {e}")))?;
    let path = shell
        .write_under(rel, &bytes)
        .map_err(|e| fail(e.to_string()))?;
    let read_back = std::fs::read(&path).map_err(|e| fail(format!("read back: {e}")))?;
    let parsed: P =
        read_strict_canonical(&read_back).map_err(|e| fail(format!("strict read: {e}")))?;
    if parsed != *record {
        return Err(fail("read-back value diverges from the record".to_owned()));
    }
    Ok(())
}

/// Materialize the landed compiled artifact's query bodies as the §8.3
/// `groups/<gid>/smt/<query-id>.smt2` files, each read back and checked
/// byte-identical to its [`ckc_smt::QueryBody`] body — solver-bound text
/// pinned at the same boundary discipline as the wrappers.
fn materialize_queries(
    shell: &Shell,
    dir: &str,
    compiled: &ArtifactWrapper<ckc_smt::CompiledArtifact>,
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

/// The §8.3 assemble processing_stage, the thin core-ir.4/.5 wrapper: derive the DocIR
/// view from the source document graph and its extraction diagnostics, assemble the
/// five-layer bundle (bundle-level diagnostics = canonical-set union of the
/// segments and normalization wrapper diagnostics; extraction diagnostics
/// stay in DocIr per the §5 bundle row; M1 test_sources inject no assumptions),
/// validate it against the graph, and wrap it.
fn assemble_bundle(
    entry: &CorpusEntry,
    resolved: &Resolved,
    source: &ArtifactWrapper<SourceDocumentGraph>,
    segments: &ArtifactWrapper<SegmentIr>,
    normalization: &ArtifactWrapper<Normalization>,
) -> Result<ArtifactWrapper<IrBundle>, DiagnosticRecord> {
    let fail = |reason: String| processing_stage_diagnostic(3, "document", &entry.id, reason);

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

    Ok(ArtifactWrapper {
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
        evidence_status: EvidenceStatus::MechanicalEvidenceStatus,
        external_effects: vec![],
        trace_refs: vec![],
        // Assembly raised nothing of its own: layer diagnostics live in the
        // payload, processing_stage failures never reach an wrapper.
        diagnostics: vec![],
        runtime_metadata: vec![],
        payload: bundle,
    })
}

/// Close one attempted processing_stage: land the built wrapper at `rel` on success,
/// then record the processing_stage event ([`finish_processing_stage`]).
fn close_processing_stage<P: Canonical + CanonRead>(
    shell: &mut Shell,
    resolved: &Resolved,
    processing_stage_index: usize,
    clock: ProcessingStageClock,
    input_hashes: Vec<Hash>,
    rel: &str,
    built: Result<ArtifactWrapper<P>, DiagnosticRecord>,
) -> Option<ArtifactWrapper<P>> {
    let landed = built.and_then(|wrapper| land(shell, rel, wrapper));
    finish_processing_stage(
        shell,
        resolved,
        processing_stage_index,
        clock,
        input_hashes,
        landed,
    )
}

/// Record one attempted processing_stage's §4.6 event: wrapper diagnostics and
/// content hash on success, the failure diagnostic alone otherwise; the
/// verify processing_stage carries its §8.4 budget counter. Returns the landed
/// wrapper for the next consumer; `None` means the event recorded a
/// failure.
fn finish_processing_stage<P: Canonical + CanonRead>(
    shell: &mut Shell,
    resolved: &Resolved,
    processing_stage_index: usize,
    clock: ProcessingStageClock,
    input_hashes: Vec<Hash>,
    landed: Result<ArtifactWrapper<P>, DiagnosticRecord>,
) -> Option<ArtifactWrapper<P>> {
    let (started_at, ended_at, duration_ms) = clock.stop();
    let (outcome, diagnostics, output_hashes, wrapper) = match landed {
        Ok(wrapper) => (
            severity(&wrapper.diagnostics),
            wrapper.diagnostics.clone(),
            vec![wrapper.content_hash.clone()],
            Some(wrapper),
        ),
        Err(diagnostic) => (diagnostic.outcome, vec![diagnostic], Vec::new(), None),
    };
    let resource_counters = if processing_stage_index == VERIFY {
        vec![(static_id(SOLVER_BUDGET_KEY), resolved.budget_ms)]
    } else {
        Vec::new()
    };
    shell.processing_stage_event(ProcessingStageEvent {
        pipeline_id: resolved.pipeline_id.clone(),
        pipeline_step_id: resolved.pipeline_step_ids[processing_stage_index].clone(),
        processing_stage: static_id(PROCESSING_STAGE_KINDS[processing_stage_index]),
        started_at,
        ended_at,
        duration_ms,
        input_hashes,
        output_hashes,
        outcome,
        diagnostics,
        resource_counters,
    });
    wrapper
}

/// Wrapper one group-processing_stage payload under the runner's fixed §4.4 fields:
/// `schema.<kind>` schema id, `<artifact-id>` minted by the caller, content
/// and policy hashes computed here.
#[allow(clippy::too_many_arguments)]
fn wrapper<P: Canonical>(
    artifact_id: String,
    kind: &str,
    producer: Producer,
    input_hashes: Vec<Hash>,
    origin: Origin,
    evidence_status: EvidenceStatus,
    diagnostics: Vec<DiagnosticRecord>,
    payload: P,
) -> Result<ArtifactWrapper<P>, CanonError> {
    Ok(ArtifactWrapper {
        schema_id: Id::new(format!("schema.{kind}")).expect("schema.<kind> stays in the grammar"),
        artifact_id: Id::new(artifact_id)
            .expect("the runner mints artifact ids inside the Id grammar"),
        artifact_kind: static_id(kind),
        producer,
        input_hashes,
        content_hash: content_hash(&payload)?,
        canonicalization_policy_hash: canonicalization_policy_hash(),
        origin,
        evidence_status,
        external_effects: vec![],
        trace_refs: vec![],
        diagnostics,
        runtime_metadata: vec![],
        payload,
    })
}

/// §4.3 canonical-set view of processing_stage diagnostics: sorted by canonical bytes,
/// byte-identical duplicates collapsed — the wrapper `diagnostics` field's
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

/// The write boundary: validate the produced wrapper, write its canonical
/// bytes under `rel`, strict-read the file back, re-validate, and return
/// the read-back value — §8.5 item 3's per-artifact property enforced at
/// production time. Downstream processing_stages chain the read-back value, never the
/// in-memory precursor: the §4.4 accepted artifact is the canonical bytes
/// on disk, and §4.3 set emission sorts what some producers store in
/// creation order (e.g. SourceDocumentGraph regions), so only disk truth keeps
/// every consumer — here, cli-runner.2b/.2c, replay — seeing one value.
/// Failures come back as the processing_stage's diagnostic.
fn land<P: Canonical + CanonRead>(
    shell: &Shell,
    rel: &str,
    wrapper: ArtifactWrapper<P>,
) -> Result<ArtifactWrapper<P>, DiagnosticRecord> {
    let fail = |reason: String| {
        invalid_diagnostic(vec![
            (static_id("artifact"), rel.to_owned()),
            (static_id("reason"), reason),
        ])
    };
    wrapper
        .validate()
        .map_err(|e| fail(format!("wrapper invariant: {e}")))?;
    let bytes =
        canonical_payload_bytes(&wrapper).map_err(|e| fail(format!("canonical emission: {e}")))?;
    let path = shell
        .write_under(rel, &bytes)
        .map_err(|e| fail(e.to_string()))?;
    let read_back = std::fs::read(&path).map_err(|e| fail(format!("read back: {e}")))?;
    let parsed: ArtifactWrapper<P> =
        read_strict_canonical(&read_back).map_err(|e| fail(format!("strict read: {e}")))?;
    parsed
        .validate()
        .map_err(|e| fail(format!("read-back invariant: {e}")))?;
    Ok(parsed)
}

/// §4.4 processing_stage outcome: severity max over the artifact's diagnostics.
fn severity(diagnostics: &[DiagnosticRecord]) -> Outcome {
    diagnostics
        .iter()
        .map(|d| d.outcome)
        .fold(Outcome::Ok, Outcome::max)
}

/// ProcessingStage-failure diagnostic: `schema_invalid`/`invalid` naming the §8.3
/// processing_stage and its subject — `document` for the per-document processing_stages, `group`
/// for the group processing_stages (§4.4 "schema, hash, canonicalization … fails").
fn processing_stage_diagnostic(
    processing_stage_index: usize,
    subject_key: &str,
    subject: &Id,
    reason: String,
) -> DiagnosticRecord {
    invalid_diagnostic(vec![
        (static_id(subject_key), subject.to_string()),
        (static_id("reason"), reason),
        (
            static_id("processing_stage"),
            PROCESSING_STAGE_KINDS[processing_stage_index].to_owned(),
        ),
    ])
}

/// §4.4 producer for one processing_stage execution; the toolchain manifest hash is
/// the [`TOOLCHAIN_FILE`] raw-byte hash resolution recorded.
fn producer(resolved: &Resolved, processing_stage_index: usize) -> Producer {
    Producer {
        pipeline_id: resolved.pipeline_id.clone(),
        pipeline_step_id: resolved.pipeline_step_ids[processing_stage_index].clone(),
        toolchain_manifest_hash: resolved.toolchain_manifest_hash.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use ckc_core::{
        Action, BindingStatus, CassettePayload, ClinicalStatement, Direction, EventRecord,
        ExceptionClause, ModelIdentity, Strength, TerminologyBinding, TotalOperationResult,
        parse_reference, read_jsonl,
    };

    use crate::normalize::clinical_ir;

    /// [`single_ir_accept`] classifies a model output with no live pipeline:
    /// malformed bytes are a repairable [`FillReject::Schema`]; an empty
    /// [`ClinicalIr`] cites no ids and accepts; an output citing ids absent
    /// upstream is a terminal [`FillReject::Grounding`] naming exactly those ids,
    /// and the same output accepts once the universes hold them. The cited output
    /// exercises every grounded site together — a binding `region_id`, an
    /// exception `region_id`, and a statement `source_segment_id` — in both
    /// directions; the no-false-reject half is the property route-single-ir.2b's
    /// reproduce-M1 gate leans on.
    #[test]
    fn single_ir_accept_classifies() {
        let none: HashSet<&Id> = HashSet::new();
        let accept = single_ir_accept(&none, &none);

        // (1) bytes that are not canonical `ClinicalIr` → repairable schema reject.
        assert!(matches!(
            accept(b"not-canonical"),
            Err(FillReject::Schema(_))
        ));

        // (2) an empty `ClinicalIr` cites no upstream ids → accepted.
        let empty = ClinicalIr {
            bindings: vec![],
            statements: vec![],
        };
        let empty_bytes = canonical_payload_bytes(&empty).unwrap();
        assert!(accept(&empty_bytes).is_ok());

        // An output citing every grounded site: a binding region, an exception
        // region, and a statement source segment.
        let region_b = static_id("region.binding");
        let region_e = static_id("region.exception");
        let segment_s = static_id("segment.statement");
        let cited = ClinicalIr {
            bindings: vec![TerminologyBinding {
                binding_id: static_id("bind.0"),
                system: static_id("ckc.lex"),
                code: static_id("cond.sepsis"),
                status: BindingStatus::Exact,
                alternatives: vec![],
                region_ids: vec![region_b.clone()],
            }],
            statements: vec![ClinicalStatement {
                statement_id: static_id("stmt.0"),
                population: vec![],
                condition: vec![],
                action: Action::new(static_id("act.start"), static_id("drug.x")),
                modality: Direction::Require,
                strength: Strength::Strong,
                certainty: None,
                exceptions: vec![ExceptionClause {
                    exception_id: static_id("exc.0"),
                    atoms: vec![],
                    region_ids: vec![region_e.clone()],
                }],
                source_segment_ids: vec![segment_s.clone()],
            }],
        };
        let cited_bytes = canonical_payload_bytes(&cited).unwrap();

        // (3) empty universes → terminal grounding naming exactly the three cited
        // ids (the downstream diagnostic sorts and dedups, so compare as a set).
        match accept(&cited_bytes) {
            Err(FillReject::Grounding(ids)) => {
                let mut got: Vec<&Id> = ids.iter().collect();
                got.sort();
                let mut want = vec![&region_b, &region_e, &segment_s];
                want.sort();
                assert_eq!(got, want);
            }
            other => panic!("expected Grounding naming all cited ids, got {other:?}"),
        }

        // (4) universes holding exactly those ids → the same output accepts.
        let regions: HashSet<&Id> = HashSet::from([&region_b, &region_e]);
        let segments: HashSet<&Id> = HashSet::from([&segment_s]);
        assert!(single_ir_accept(&regions, &segments)(&cited_bytes).is_ok());
    }

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
        "test_source.m1_guideline_a",
        "test_source.m1_guideline_b",
        "test_source.m1_control",
    ];

    /// §4.3 set emission canonically sorts wrapper input hashes, so the
    /// chain expectations compare as sorted sets (ASCII `sha256:` text
    /// orders identically under derived `Ord` and the canonical byte key).
    fn sorted(hashes: &[Hash]) -> Vec<Hash> {
        let mut hashes = hashes.to_vec();
        hashes.sort();
        hashes
    }

    /// Strict-read one landed artifact wrapper at `rel` under the run
    /// directory and re-check its mechanical invariants; the §8.5 item 3
    /// per-artifact property, asserted from the consumer side.
    fn strict_at<P: Canonical + CanonRead>(out: &Path, rel: &str) -> ArtifactWrapper<P> {
        let path = out.join(rel);
        let wrapper: ArtifactWrapper<P> = read_strict_canonical(&std::fs::read(&path).unwrap())
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        wrapper.validate().unwrap();
        wrapper
    }

    /// [`strict_at`] over the document-artifact layout slot.
    fn strict<P: Canonical + CanonRead>(out: &Path, doc: &str, name: &str) -> ArtifactWrapper<P> {
        strict_at(out, &format!("artifacts/{doc}/{name}.json"))
    }

    // The unit gate: the document processing_stages over the three test_sources land the
    // twelve §8.3 document artifacts, every one strict-read clean with its
    // input hashes chaining the §8.4 processing_stage order, and the event stream
    // carries one clean processing_stage event per execution before the command event.
    #[test]
    fn document_processing_stages_land_strict_artifacts_over_the_test_sources() {
        let (result, events, diagnostics, out, _tmp) = executed(&repo_root(), "exp.m1_scaffold");

        // The full §8.3 chain through verify completes clean.
        assert_eq!(result.outcome, Outcome::Ok);
        assert!(result.diagnostic_hashes.is_empty());
        assert!(diagnostics.is_empty());

        for doc in DOC_IDS {
            let source: ArtifactWrapper<SourceDocumentGraph> =
                strict(&out, doc, "source_document_graph");
            let segments: ArtifactWrapper<SegmentIr> = strict(&out, doc, "segments");
            let normalization: ArtifactWrapper<Normalization> = strict(&out, doc, "normalization");
            let bundle: ArtifactWrapper<IrBundle> = strict(&out, doc, "ir_bundle");

            assert_eq!(
                source.artifact_id,
                format!("{doc}.source_document_graph").parse().unwrap()
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
                bundle.producer.pipeline_id,
                static_id("pipe.layered_ckcir_to_smt")
            );
            assert_eq!(
                bundle.producer.pipeline_step_id,
                static_id("processing_stage.m1.assemble")
            );

            // The bundle re-validates against its graph and carries the §8.6
            // rule the reference core expects from each test_source.
            bundle.payload.validate(&source.payload).unwrap();
            assert_eq!(bundle.payload.norm.rules.len(), 1);
            assert_eq!(
                bundle.payload.norm.rules[0].rule_id,
                format!("{doc}.rule.0").parse().unwrap()
            );
        }

        // 4 processing_stage events per document, compile+verify per group, the
        // run-scoped trace and report processing_stages, then the closing command
        // event.
        assert_eq!(events.len(), 19);
        for (n, event) in events.iter().enumerate() {
            assert_eq!(event.event_id, format!("event.{n}").parse::<Id>().unwrap());
            assert_eq!(event.event_sequence_number, n as u64);
            assert_eq!(event.run_id, static_id("m1"));
        }
        for (d, doc) in DOC_IDS.iter().enumerate() {
            for (s, kind) in PROCESSING_STAGE_KINDS[..4].iter().enumerate() {
                let event = &events[d * 4 + s];
                assert_eq!(event.processing_stage, static_id(kind), "{doc}");
                assert_eq!(event.pipeline_id, static_id("pipe.layered_ckcir_to_smt"));
                assert_eq!(
                    event.pipeline_step_id,
                    format!("processing_stage.m1.{kind}").parse().unwrap()
                );
                assert_eq!(event.outcome, Outcome::Ok);
                assert_eq!(event.input_hashes.len(), s);
                assert_eq!(event.output_hashes.len(), 1);
                assert!(event.diagnostics.is_empty());
            }
        }
        // Group events in evaluation order: conflict then no-conflict, compile
        // then verify; only verify carries the §8.4 budget counter.
        for (g, base) in [12, 14].iter().enumerate() {
            let compile = &events[*base];
            assert_eq!(compile.processing_stage, static_id("compile"), "group {g}");
            assert_eq!(
                compile.pipeline_step_id,
                static_id("processing_stage.m1.compile")
            );
            assert_eq!(compile.outcome, Outcome::Ok);
            assert_eq!(compile.input_hashes.len(), 2);
            assert_eq!(compile.output_hashes.len(), 1);
            assert!(compile.diagnostics.is_empty());
            assert!(compile.resource_counters.is_empty());
            let verify = &events[*base + 1];
            assert_eq!(verify.processing_stage, static_id("verify"), "group {g}");
            assert_eq!(
                verify.pipeline_step_id,
                static_id("processing_stage.m1.verify")
            );
            assert_eq!(verify.outcome, Outcome::Ok);
            assert_eq!(verify.input_hashes, compile.output_hashes);
            assert_eq!(verify.output_hashes.len(), 1);
            assert!(verify.diagnostics.is_empty());
            assert_eq!(
                verify.resource_counters,
                vec![(static_id("solver_ms_per_query"), 10_000)]
            );
        }
        // The trace processing_stage: the DAG node content-hash set as input — 19
        // hashed nodes (3 sources + 12 document artifacts + 4 group
        // artifacts; the report node is hashless) collapsing to 18 because
        // control's and guideline_b's segments artifacts are byte-identical
        // — and the landed pair as outputs.
        let trace = &events[16];
        assert_eq!(trace.processing_stage, static_id("trace"));
        assert_eq!(trace.pipeline_id, static_id("pipe.layered_ckcir_to_smt"));
        assert_eq!(
            trace.pipeline_step_id,
            static_id("processing_stage.m1.trace")
        );
        assert_eq!(trace.outcome, Outcome::Ok);
        assert_eq!(trace.input_hashes.len(), 18);
        assert_eq!(trace.output_hashes.len(), 2);
        assert!(trace.diagnostics.is_empty());
        assert!(trace.resource_counters.is_empty());
        // The report processing_stage: every consumed wrapper's content hash as
        // input — the trace pair, three source document graphs, two verifier
        // results — and the landed report as output.
        let report = &events[17];
        assert_eq!(report.processing_stage, static_id("report"));
        assert_eq!(report.pipeline_id, static_id("pipe.layered_ckcir_to_smt"));
        assert_eq!(
            report.pipeline_step_id,
            static_id("processing_stage.m1.report")
        );
        assert_eq!(report.outcome, Outcome::Ok);
        assert_eq!(report.input_hashes.len(), 7);
        assert!(report.input_hashes.contains(&trace.output_hashes[0]));
        assert!(report.input_hashes.contains(&trace.output_hashes[1]));
        assert_eq!(report.output_hashes.len(), 1);
        assert!(report.diagnostics.is_empty());
        assert!(report.resource_counters.is_empty());
        assert!(out.join("report.json").exists());
        let command = &events[18];
        assert_eq!(command.processing_stage, static_id("run"));
        assert_eq!(command.outcome, Outcome::Ok);
    }

    // The group processing_stages over exp.m1_scaffold: compiled artifacts and verifier
    // results land strict-read clean with hashes chaining bundles →
    // compiled → results and every query body materialized byte-identical
    // under smt/; the §8.6 thread yields the cross-document contradiction
    // in the conflict group and the disjoint-interval documented no-conflict
    // result in the no-conflict group.
    #[test]
    fn group_processing_stages_compile_and_verify_the_test_source_groups() {
        use ckc_smt::{CompiledArtifact, SolverVerdict, VerifierCategory};

        let (_result, _events, _diagnostics, out, _tmp) = executed(&repo_root(), "exp.m1_scaffold");
        let a: ArtifactWrapper<IrBundle> = strict(&out, "test_source.m1_guideline_a", "ir_bundle");
        let b: ArtifactWrapper<IrBundle> = strict(&out, "test_source.m1_guideline_b", "ir_bundle");
        let control: ArtifactWrapper<IrBundle> =
            strict(&out, "test_source.m1_control", "ir_bundle");

        // Per group: wrapper identity/chaining pins, plan and assertion
        // map shape, and byte-identical smt materialization.
        for (gid, members, rules) in [
            (
                "group.m1_conflict",
                [&a, &b],
                ["a", "b"].map(|d| format!("test_source.m1_guideline_{d}.rule.0")),
            ),
            (
                "group.m1_no_conflict",
                [&a, &control],
                [
                    "test_source.m1_control.rule.0".to_owned(),
                    "test_source.m1_guideline_a.rule.0".to_owned(),
                ],
            ),
        ] {
            let compiled: ArtifactWrapper<CompiledArtifact> =
                strict_at(&out, &format!("groups/{gid}/compiled.json"));
            assert_eq!(compiled.schema_id, static_id("schema.compiled"), "{gid}");
            assert_eq!(
                compiled.artifact_id,
                format!("{gid}.compiled").parse().unwrap()
            );
            assert_eq!(compiled.artifact_kind, static_id("compiled"));
            assert_eq!(
                compiled.producer.pipeline_step_id,
                static_id("processing_stage.m1.compile")
            );
            assert_eq!(compiled.origin, Origin::DeterministicCompiler);
            assert_eq!(
                compiled.evidence_status,
                EvidenceStatus::CompilerEvidenceStatus
            );
            assert_eq!(
                compiled.input_hashes,
                sorted(&members.map(|m| m.content_hash.clone()))
            );
            assert!(compiled.diagnostics.is_empty());
            assert!(compiled.payload.diagnostics.is_empty());

            let gsuf = gid.strip_prefix("group.").unwrap();
            assert_eq!(compiled.payload.solver_query_plan.len(), 1);
            assert_eq!(
                compiled.payload.solver_query_plan[0].pair_id,
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
                .assertion_to_source_map
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

            let results: ArtifactWrapper<ckc_smt::VerifierResults> =
                strict_at(&out, &format!("groups/{gid}/verifier_results.json"));
            assert_eq!(results.schema_id, static_id("schema.verifier_results"));
            assert_eq!(
                results.artifact_id,
                format!("{gid}.verifier_results").parse().unwrap()
            );
            assert_eq!(results.artifact_kind, static_id("verifier_results"));
            assert_eq!(
                results.producer.pipeline_step_id,
                static_id("processing_stage.m1.verify")
            );
            assert_eq!(results.origin, Origin::ExternalAdapterGenerated);
            assert_eq!(
                results.evidence_status,
                EvidenceStatus::VerifierEvidenceStatus
            );
            assert_eq!(results.input_hashes, vec![compiled.content_hash.clone()]);
            assert!(results.diagnostics.is_empty());
            results.payload.validate().unwrap();
            for r in &results.payload.results {
                assert!(r.diagnostics.is_empty(), "{gid} {}", r.query_id);
                assert_eq!(r.solver_identity.solver_id, static_id("z3"));
            }
        }

        // Conflict group: Q1 sat with the overlap satisfying_example, Q2 unsat with
        // the cross-document core — the §8.6 finding.
        let conflict: ArtifactWrapper<ckc_smt::VerifierResults> =
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
                "a.test_source.m1_guideline_a.rule.0".parse().unwrap(),
                "a.test_source.m1_guideline_b.rule.0".parse().unwrap(),
            ])
        );

        // No-conflict group: the disjoint-interval Q1 answers unsat, closing the
        // pair as the documented no-conflict result — no Q2 run, no satisfying_example.
        let no_conflict: ArtifactWrapper<ckc_smt::VerifierResults> =
            strict_at(&out, "groups/group.m1_no_conflict/verifier_results.json");
        let rs = &no_conflict.payload.results;
        assert_eq!(rs.len(), 1);
        assert_eq!(
            rs[0].query_id,
            "q.m1_no_conflict.pair1.overlap".parse().unwrap()
        );
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
        let (result, events, diagnostics, out, _tmp) = executed(bare.path(), "exp.m1_scaffold");
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

    /// Write a minimal two-test_source registry trio under `root`: `test_source.gone`
    /// points at a missing file, `test_source.tiny` at a minimal HTML document;
    /// the pipeline declares one processing_stage per [`PROCESSING_STAGE_KINDS`] entry.
    fn write_tiny_root(root: &Path) {
        let write = |rel: &str, text: &str| {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        write(
            "registry/corpora.yaml",
            "\
- id: test_source.gone
  path: corpus/test_sources/gone.html
  origin: ai_generated
  evidence_status: source_evidence_status
  provenance: synthetic
- id: test_source.tiny
  path: corpus/test_sources/tiny.html
  origin: ai_generated
  evidence_status: source_evidence_status
  provenance: synthetic
",
        );
        write(
            "registry/candidates.yaml",
            "\
pipelines:
  - id: pipe.tiny
    processing_stages:
      [processing_stage.t.extract, processing_stage.t.segment, processing_stage.t.normalize, processing_stage.t.assemble,
       processing_stage.t.compile, processing_stage.t.verify, processing_stage.t.trace, processing_stage.t.report]
processing_stages:
  - id: processing_stage.t.extract
    kind: extract
    determinism: deterministic
    input_artifact_kinds: []
    output_artifact_kinds: [source_document_graph]
  - id: processing_stage.t.segment
    kind: segment
    determinism: deterministic
    input_artifact_kinds: [source_document_graph]
    output_artifact_kinds: [segments]
  - id: processing_stage.t.normalize
    kind: normalize
    determinism: deterministic
    input_artifact_kinds: [source_document_graph, segments]
    output_artifact_kinds: [normalization]
  - id: processing_stage.t.assemble
    kind: assemble
    determinism: deterministic
    input_artifact_kinds: [source_document_graph, segments, normalization]
    output_artifact_kinds: [ir_bundle]
  - id: processing_stage.t.compile
    kind: compile
    determinism: deterministic
    input_artifact_kinds: [ir_bundle]
    output_artifact_kinds: [compiled, smt_query]
  - id: processing_stage.t.verify
    kind: verify
    determinism: deterministic
    input_artifact_kinds: [compiled, smt_query]
    output_artifact_kinds: [verifier_results]
  - id: processing_stage.t.trace
    kind: trace
    determinism: deterministic
    input_artifact_kinds:
      [source_document_graph, segments, normalization, ir_bundle, compiled, verifier_results]
    output_artifact_kinds: [trace_bundle, lineage_index]
  - id: processing_stage.t.report
    kind: report
    determinism: deterministic
    input_artifact_kinds:
      [source_document_graph, ir_bundle, compiled, verifier_results, trace_bundle, lineage_index]
    output_artifact_kinds: [report, run_manifest, replay_manifest]
",
        );
        write(
            "registry/experiments.yaml",
            "\
- id: exp.tiny
  pipeline: pipe.tiny
  test_source_groups:
    - group_id: group.t
      test_sources: [test_source.gone, test_source.tiny]
  seed: 1
  budget:
    solver_ms_per_query: 1000
  expected_outcomes: corpus/reference/t.yaml
",
        );
        write(
            "corpus/test_sources/tiny.html",
            "<html><body><p>本文。</p></body></html>",
        );
        // Provenance files the resolved producer and the report processing_stage's
        // manifests hash (raw bytes; never parsed).
        write(TOOLCHAIN_FILE, "[toolchain]\nchannel = \"test\"\n");
        write(LOCKFILE, "# staged lockfile\n");
    }

    // §4.4 valid remainder: a document whose corpus file is missing takes a
    // command-scope diagnostic while the other document still runs all four
    // processing_stages and lands its artifacts; the group misses a member bundle, so
    // its compile processing_stage fails rather than compiling a partial group, and
    // verify never runs. The trace processing_stage still assembles and lands the pair
    // over what landed — the surviving document's chain, no group artifacts
    // — and the report processing_stage closes the chain over it: no claims, so both
    // partitions are empty, while the corpus row and the ledger summary
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
        // document's processing_stage residuals (extract parse error, unclassified
        // paragraph) riding their processing_stage events, then the group's compile
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
                .any(|(_, v)| v.contains("test_source.gone")),
            "{diagnostics:?}"
        );

        assert_eq!(events.len(), 8);
        for (s, kind) in PROCESSING_STAGE_KINDS[..4].iter().enumerate() {
            assert_eq!(events[s].processing_stage, static_id(kind));
            assert_eq!(
                events[s].pipeline_step_id,
                format!("processing_stage.t.{kind}").parse().unwrap()
            );
            assert_eq!(
                events[s].output_hashes.len(),
                1,
                "{kind} landed its artifact"
            );
        }
        let compile_event = &events[4];
        assert_eq!(compile_event.processing_stage, static_id("compile"));
        assert_eq!(
            compile_event.pipeline_step_id,
            static_id("processing_stage.t.compile")
        );
        assert_eq!(compile_event.outcome, Outcome::Invalid);
        assert_eq!(compile_event.diagnostics.len(), 1);
        assert!(compile_event.output_hashes.is_empty());
        let trace_event = &events[5];
        assert_eq!(trace_event.processing_stage, static_id("trace"));
        assert_eq!(
            trace_event.pipeline_step_id,
            static_id("processing_stage.t.trace")
        );
        assert_eq!(trace_event.outcome, Outcome::Ok);
        // test_source.tiny's source + its four landed artifacts; the
        // bundle-less group contributes no nodes.
        assert_eq!(trace_event.input_hashes.len(), 5);
        assert_eq!(trace_event.output_hashes.len(), 2);
        // The report processing_stage consumes the trace pair and the surviving
        // document's graph (the verdict-less group contributes nothing);
        // its summary counts the whole four-record ledger.
        let report_event = &events[6];
        assert_eq!(report_event.processing_stage, static_id("report"));
        assert_eq!(
            report_event.pipeline_step_id,
            static_id("processing_stage.t.report")
        );
        assert_eq!(report_event.outcome, Outcome::Ok);
        assert_eq!(report_event.input_hashes.len(), 3);
        assert_eq!(report_event.output_hashes.len(), 1);
        assert!(out.join("trace_bundle.json").exists());
        assert!(out.join("lineage_index.json").exists());
        assert!(!out.join("artifacts/test_source.gone").exists());
        assert!(!out.join("groups").exists());
        let bundle: ArtifactWrapper<IrBundle> = strict(&out, "test_source.tiny", "ir_bundle");
        let source: ArtifactWrapper<SourceDocumentGraph> =
            strict(&out, "test_source.tiny", "source_document_graph");
        bundle.payload.validate(&source.payload).unwrap();
        let report: ArtifactWrapper<crate::report::Report> = strict_at(&out, "report.json");
        assert!(report.payload.findings.is_empty());
        assert!(report.payload.no_conflict_results.is_empty());
        assert!(report.payload.wording.is_empty());
        assert_eq!(report.payload.corpus_hashes.len(), 1);
        assert_eq!(
            report.payload.corpus_hashes[0].0,
            static_id("test_source.tiny")
        );
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
    // budget key is one command-scope diagnostic and no processing_stage runs.
    #[test]
    fn missing_solver_budget_stops_resolution() {
        let root = tempfile::tempdir().unwrap();
        write_tiny_root(root.path());
        std::fs::write(
            root.path().join("registry/experiments.yaml"),
            "\
- id: exp.tiny
  pipeline: pipe.tiny
  test_source_groups:
    - group_id: group.t
      test_sources: [test_source.tiny]
  seed: 1
  budget: {}
  expected_outcomes: corpus/reference/t.yaml
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

    /// The §5 lexicon the single_ir route's deterministic head consumes, loaded
    /// from the repository root like the run does.
    fn single_ir_lexicon() -> Lexicon {
        load_lexicon(&std::fs::read(repo_root().join(LEXICON_FILE)).unwrap()).unwrap()
    }

    /// The three M1 corpus documents ([`DOC_IDS`]) resolved from the committed
    /// `registry/corpora.yaml`, in [`DOC_IDS`] order.
    fn single_ir_corpus() -> Vec<CorpusEntry> {
        let text = std::fs::read_to_string(repo_root().join(CORPORA_FILE)).unwrap();
        let corpora = parse_corpora(&text).unwrap();
        DOC_IDS
            .iter()
            .map(|id| {
                let id: Id = id.parse().unwrap();
                corpora
                    .iter()
                    .find(|e| e.id == id)
                    .unwrap_or_else(|| panic!("registry/corpora.yaml is missing {id}"))
                    .clone()
            })
            .collect()
    }

    /// A minimal [`Resolved`] for the single_ir route over the M1 inputs. `Resolved`
    /// carries a fixed `[Id; 8]`; slots `[0]`–`[5]` are `pipe.m2_single_ir`'s six stages
    /// (extract, segment, model_fill, assemble, compile, verify), so the route's producer
    /// stamps (the fill head, the cassette wrapper, the verdict tail) are faithful; slots
    /// `[6]`/`[7]` (trace, report) are unread M1-shaped placeholders the verdict tail never
    /// reaches. `documents` / `groups` / `plan` go unread so stay empty and the toolchain
    /// hash stays synthetic — the bundle `content_hash` gate is payload-only; `budget_ms`
    /// is exp.m1_scaffold's §8.4 `solver_ms_per_query`, the verdict tail's z3 cap.
    fn single_ir_resolved() -> Resolved {
        Resolved {
            pipeline_id: static_id("pipe.m2_single_ir"),
            pipeline_step_ids: [
                static_id("processing_stage.m1.extract"),
                static_id("processing_stage.m1.segment"),
                static_id("processing_stage.m2.model_fill"),
                static_id("processing_stage.m2.assemble"),
                static_id("processing_stage.m1.compile"),
                static_id("processing_stage.m1.verify"),
                static_id("processing_stage.m1.trace"),
                static_id("processing_stage.m1.report"),
            ],
            documents: vec![],
            groups: vec![],
            budget_ms: 10_000,
            plan: RunPlan {
                experiment_id: static_id("exp.m2_multihop"),
                test_source_groups: vec![],
                pipelines: vec![],
                seed: 42,
                budget: vec![],
            },
            toolchain_manifest_hash: hash_bytes(b"single-ir-fixture-toolchain"),
        }
    }

    /// Craft one golden `route.single_ir` cassette: wrap `output` (the bytes the
    /// model "emits") under the §4.4 provenance contract
    /// [`CassetteStore::build_wrapper`] enforces, with a synthetic model identity (the
    /// crafted-fixture rule — no real engine, quant, or model-format token), and
    /// persist it at the store key. Factored so route-single-ir.4 reuses it to seed
    /// rejection cassettes at other seeds.
    fn write_single_ir_cassette(
        entry: &CorpusEntry,
        resolved: &Resolved,
        store: &CassetteStore,
        seed: u64,
        output: &[u8],
    ) {
        let key = CassetteKey {
            route: static_id("route.single_ir"),
            source: entry.id.clone(),
            seed,
        };
        let payload = CassettePayload::from_output(
            static_id("route.single_ir"),
            entry.id.clone(),
            seed,
            "Fill the ClinicalIR schema grounded in the cited document.".to_owned(),
            hash_bytes(b"single-ir-constraint"),
            hash_bytes(b"single-ir-prompt-template"),
            ModelIdentity {
                model_id: static_id("model.baseline"),
                quant: "fixture_quant".to_owned(),
                runtime_version: "1.0.0".to_owned(),
            },
            output,
        );
        let wrapper = store
            .build_wrapper(&key, payload, producer(resolved, 2))
            .unwrap();
        store.persist(&key, wrapper).unwrap();
    }

    /// Bless the three committed golden `route.single_ir` cassettes: for each M1
    /// document, run the deterministic head (extract → segment → [`clinical_ir`]) and
    /// record the canonical [`ClinicalIr`] bytes as the model output. The premise the
    /// reproduce-M1 gate leans on — `clinical_ir` raises no diagnostics for these three
    /// documents, so the route's segments-only bundle diagnostics equal M1's segments ∪
    /// normalization set — is asserted here. `#[ignore]`d: run it to regenerate, then
    /// commit the three json. Regenerate with
    /// `cargo test -p ckc-cli bless_single_ir_cassettes -- --ignored --exact`.
    #[test]
    #[ignore = "regenerates the committed golden cassettes"]
    fn bless_single_ir_cassettes() {
        let root = repo_root();
        let lexicon = single_ir_lexicon();
        let resolved = single_ir_resolved();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        for entry in single_ir_corpus() {
            let html = std::fs::read(root.join(&entry.path)).unwrap();
            let config = ExtractConfig {
                document_id: entry.id.clone(),
                source_family: static_id("synthetic_test_source_html"),
                provenance: entry.provenance,
                data_class: DataClass::None,
                producer: producer(&resolved, 0),
            };
            let source = extract(&html, &config).unwrap();
            let segments = segment(&source, &producer(&resolved, 1)).unwrap();
            let (clinical, norm_diags) = clinical_ir(&source.payload, &segments.payload, &lexicon);
            assert!(
                norm_diags.is_empty(),
                "{}: clinical_ir raised diagnostics, breaking the reproduce-M1 premise: {norm_diags:?}",
                entry.id
            );
            let output = canonical_payload_bytes(&clinical).unwrap();
            write_single_ir_cassette(&entry, &resolved, &store, 42, &output);
        }
    }

    /// Bless the committed `route.single_ir` rejection cassettes route-single-ir.4
    /// replays, all keyed under guideline_a (its real grounding universe) and
    /// synthetic-identity (the crafted-fixture rule). A HALLUCINATED output (seed 99) is
    /// the golden ClinicalIr with one statement `source_segment_id` rebound to an id
    /// absent upstream — still canonical, so it parses and the absent reference surfaces
    /// at grounding. A MALFORMED output (seed 98) is not canonical `ClinicalIr`, so the
    /// parse/schema check fails. A VALID recovery output (the golden ClinicalIr) sits at
    /// the first repair seed `derive_seed(98, 1)`, so a malformed base repairs to an
    /// accepted fill. A MALFORMED multi-attempt pair (base seed 97 and its first repair
    /// seed `derive_seed(97, 1)`, both non-canonical) drives a `repair_limit = 1` fill
    /// through the re-prompt path to a terminal `repair_limit_exceeded`. `#[ignore]`d: run
    /// to regenerate, then commit the five json. Regenerate with
    /// `cargo test -p ckc-cli bless_single_ir_rejection_cassettes -- --ignored --exact`.
    #[test]
    #[ignore = "regenerates the committed rejection cassettes"]
    fn bless_single_ir_rejection_cassettes() {
        let root = repo_root();
        let resolved = single_ir_resolved();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        let guideline_a = single_ir_corpus()
            .into_iter()
            .find(|e| e.id == static_id("test_source.m1_guideline_a"))
            .expect("guideline_a in the corpus");

        // The golden ClinicalIr the route reproduces for guideline_a, decoded from its
        // committed seed-42 cassette.
        let golden_bytes = store
            .replay(&CassetteKey {
                route: static_id("route.single_ir"),
                source: guideline_a.id.clone(),
                seed: 42,
            })
            .unwrap()
            .payload
            .output_bytes()
            .unwrap();

        // HALLUCINATED (seed 99): rebind one statement source segment to a fresh id absent
        // from guideline_a's segment universe; canonical re-encoding keeps it parseable.
        let mut hallucinated: ClinicalIr = read_strict_canonical(&golden_bytes).unwrap();
        hallucinated.statements[0].source_segment_ids[0] = static_id("seg.hallucinated_absent");
        let hallucinated_bytes = canonical_payload_bytes(&hallucinated).unwrap();
        write_single_ir_cassette(&guideline_a, &resolved, &store, 99, &hallucinated_bytes);

        // MALFORMED (seed 98): not canonical `ClinicalIr`, so the parse/schema check fails.
        write_single_ir_cassette(
            &guideline_a,
            &resolved,
            &store,
            98,
            b"not a canonical ClinicalIr payload",
        );

        // VALID recovery at the first repair seed: the golden ClinicalIr.
        write_single_ir_cassette(
            &guideline_a,
            &resolved,
            &store,
            crate::model::derive_seed(98, 1),
            &golden_bytes,
        );

        // MULTI-ATTEMPT EXHAUSTION (base seed 97 + its first repair seed): both
        // non-canonical, so a `repair_limit = 1` fill schema-fails at the base, re-prompts
        // under `derive_seed(97, 1)`, schema-fails again, and exhausts → a terminal
        // `repair_limit_exceeded` after traversing the re-prompt path (the zero-budget
        // boundary stays `model_fill.rs`'s coverage).
        write_single_ir_cassette(
            &guideline_a,
            &resolved,
            &store,
            97,
            b"not a canonical ClinicalIr payload (base)",
        );
        write_single_ir_cassette(
            &guideline_a,
            &resolved,
            &store,
            crate::model::derive_seed(97, 1),
            b"not a canonical ClinicalIr payload (repair 1)",
        );
    }

    /// The reproduce-M1 gate: for each M1 document, [`single_ir_fill`] replaying the
    /// committed golden cassette compiles a bundle byte-identical (by the payload-only
    /// `content_hash`) to the M1 deterministic [`assemble_bundle`] bundle.
    /// Runtime-absent — the cassette IS the recorded model output. Structural equality
    /// is asserted too (the clearer failure should the route's deterministic tail ever
    /// diverge).
    #[test]
    fn single_ir_fill_reproduces_m1_bundles() {
        let root = repo_root();
        let lexicon = single_ir_lexicon();
        let resolved = single_ir_resolved();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        for entry in single_ir_corpus() {
            // M1 reference: the deterministic extract → segment → normalize → assemble
            // chain (z3-free), the same `assemble_bundle` the M1 run drives.
            let html = std::fs::read(root.join(&entry.path)).unwrap();
            let config = ExtractConfig {
                document_id: entry.id.clone(),
                source_family: static_id("synthetic_test_source_html"),
                provenance: entry.provenance,
                data_class: DataClass::None,
                producer: producer(&resolved, 0),
            };
            let source = extract(&html, &config).unwrap();
            let segments = segment(&source, &producer(&resolved, 1)).unwrap();
            let normalization =
                normalize(&source, &segments, &lexicon, &producer(&resolved, 2)).unwrap();
            let m1 =
                assemble_bundle(&entry, &resolved, &source, &segments, &normalization).unwrap();

            // The single_ir route bundle: replay the committed golden cassette and
            // compile over the same deterministic head.
            let tmp = tempfile::tempdir().unwrap();
            let out = tmp.path().join("m2");
            std::fs::create_dir_all(&out).unwrap();
            let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out));
            let route = single_ir_fill(
                &root, &entry, &lexicon, &store, 42, &resolved, 0, &mut shell,
            )
            .unwrap_or_else(|| panic!("{}: single_ir_fill yielded no bundle", entry.id));

            assert_eq!(route.payload, m1.payload, "{} payload", entry.id);
            assert_eq!(
                route.content_hash, m1.content_hash,
                "{} content_hash",
                entry.id
            );
        }
    }

    /// route-single-ir.3 — the single_ir route's verdict half over the M1 groups
    /// (z3 present, model-runtime-absent). Fill each M1 document through
    /// [`single_ir_fill`] replaying its committed golden cassette, resolve the groups
    /// and reference from `exp.m1_scaffold`, then drive each group's member bundles
    /// through [`compile_verify_group`] and score the verdicts with the `run_oracle.rs`
    /// `assert_group_matches_reference` logic. The reproduce-M1 gate proved the route
    /// bundles equal M1's, so the model-filled clinical layer reaches M1's verdicts —
    /// the conflict pair contradicts on its deontic query (cross-document unsat core),
    /// the no-conflict pair closes on a Q1-unsat overlap with its deontic query skipped.
    #[test]
    fn single_ir_route_scores_m1_groups() {
        use std::collections::{BTreeMap, BTreeSet};

        use ckc_smt::{SolverVerdict, VerifierCategory};

        let root = repo_root();
        let lexicon = single_ir_lexicon();
        let resolved = single_ir_resolved();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        let adapter = Z3Adapter::new().expect("z3 adapter on PATH");

        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("m2");
        std::fs::create_dir_all(&out).unwrap();
        let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out));

        // Fill every M1 document once, keyed by test_source id, so the groups and the
        // reference resolve from `exp.m1_scaffold` (the run_oracle registry-driven
        // shape) rather than a hardcoded membership list.
        let mut bundles = BTreeMap::new();
        for entry in single_ir_corpus() {
            let bundle = single_ir_fill(
                &root, &entry, &lexicon, &store, 42, &resolved, 0, &mut shell,
            )
            .unwrap_or_else(|| panic!("{}: single_ir_fill yielded no bundle", entry.id));
            bundles.insert(entry.id.clone(), bundle);
        }

        let experiments = parse_experiments(
            &std::fs::read_to_string(root.join("registry/experiments.yaml")).unwrap(),
        )
        .unwrap();
        let exp = experiments
            .iter()
            .find(|e| e.id == static_id("exp.m1_scaffold"))
            .expect("exp.m1_scaffold");
        let reference =
            parse_reference(&std::fs::read_to_string(root.join(&exp.expected_outcomes)).unwrap())
                .unwrap();
        assert_eq!(
            reference.len(),
            exp.test_source_groups.len(),
            "one reference entry per test_source group"
        );

        for group in &exp.test_source_groups {
            let gid = group.group_id.clone();
            let members: Vec<_> = group
                .test_sources
                .iter()
                .map(|s| {
                    bundles
                        .get(s)
                        .unwrap_or_else(|| panic!("{gid}: unfilled member {s}"))
                })
                .collect();
            let (compiled, results) = compile_verify_group(
                &gid,
                &format!("groups/{gid}"),
                &members,
                processing_stage_clock(),
                &resolved,
                &adapter,
                &mut shell,
            );
            let compiled = compiled.unwrap_or_else(|| panic!("{gid}: no compiled artifact"));
            let results = results.unwrap_or_else(|| panic!("{gid}: no verifier results"));

            // The verdict tail stamps `pipe.m2_single_ir`'s real compile and verify steps.
            assert_eq!(
                compiled.producer.pipeline_step_id,
                static_id("processing_stage.m1.compile"),
                "{gid}"
            );
            assert_eq!(
                results.producer.pipeline_step_id,
                static_id("processing_stage.m1.verify"),
                "{gid}"
            );

            // Score vs the reference (run_oracle's assert_group_matches_reference shape).
            let entry = reference
                .iter()
                .find(|e| e.group_id == gid)
                .unwrap_or_else(|| panic!("{gid}: no reference entry"));
            let contradictions: Vec<_> = results
                .payload
                .results
                .iter()
                .filter(|r| r.category == VerifierCategory::SemanticContradiction)
                .collect();
            if entry.expected_outcome == static_id("semantic_contradiction") {
                assert_eq!(contradictions.len(), 1, "{gid}: exactly one contradiction");
                let hit = contradictions[0];
                assert!(
                    compiled
                        .payload
                        .solver_query_plan
                        .iter()
                        .any(|p| p.deontic_consistency_query_id == hit.query_id),
                    "{gid}: the contradiction rides a deontic-consistency query"
                );
                assert_eq!(
                    entry.expected_conflict_kind,
                    Some(static_id("deontic_direction_conflict")),
                    "{gid}: a deontic Q2 contradiction is the deontic_direction_conflict kind"
                );
                let core: BTreeSet<Id> = hit
                    .unsat_core
                    .clone()
                    .expect("an unsat verdict carries its core")
                    .into_iter()
                    .collect();
                assert_eq!(
                    core, entry.expected_unsat_core,
                    "{gid}: unsat core as a set"
                );
            } else if entry.expected_outcome == static_id("semantic_no_conflict") {
                assert!(contradictions.is_empty(), "{gid}: no contradiction");
                assert!(
                    results
                        .payload
                        .results
                        .iter()
                        .all(|r| r.category == VerifierCategory::SemanticNoConflict),
                    "{gid}: every query closed semantic_no_conflict"
                );
                if entry.expected_no_conflict_result {
                    // §6 no-conflict closure: an overlap query answered unsat and the
                    // pair's deontic query never ran.
                    let closed: Vec<_> = compiled
                        .payload
                        .solver_query_plan
                        .iter()
                        .filter(|p| {
                            results.payload.results.iter().any(|r| {
                                r.query_id == p.context_overlap_query_id
                                    && r.verdict == Some(SolverVerdict::Unsat)
                            })
                        })
                        .collect();
                    assert!(
                        !closed.is_empty(),
                        "{gid}: a pair closed as documented no-conflict result"
                    );
                    for pair in &closed {
                        assert!(
                            results
                                .payload
                                .results
                                .iter()
                                .all(|r| r.query_id != pair.deontic_consistency_query_id),
                            "{gid}: closed pair {} skipped its deontic query",
                            pair.pair_id
                        );
                    }
                }
            } else {
                panic!(
                    "{gid}: unhandled expected_outcome {}",
                    entry.expected_outcome
                );
            }
        }
    }

    /// route-single-ir.4 — the single_ir route's §7.4 rejection codes wire through
    /// [`model_fill`] under [`single_ir_accept`], model-runtime-absent (z3-free). Over
    /// guideline_a's real grounding universe (the deterministic extract → segment head),
    /// replaying the committed bad cassettes: a HALLUCINATED output — a source-segment
    /// reference absent upstream, still canonical — is a terminal `ai_hallucinated_source`
    /// naming exactly the absent id, with no target and no repair spent even under a repair
    /// budget (a hallucination is not a schema defect); a MALFORMED output is a repairable
    /// `ai_schema_violation` that, with a valid recovery cassette at the first repair seed,
    /// recovers an accepted target — and the full route compiles it to an `IrBundle` — or,
    /// with no repair budget, terminates in `repair_limit_exceeded`. The repair-loop
    /// mechanics live in `model_fill.rs`; this pins the route accept closure → §7.4 code
    /// selection end-to-end.
    #[test]
    fn single_ir_route_rejection_codes() {
        use ckc_core::{DiagnosticCode, Outcome};

        let root = repo_root();
        let lexicon = single_ir_lexicon();
        let resolved = single_ir_resolved();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        let guideline_a = single_ir_corpus()
            .into_iter()
            .find(|e| e.id == static_id("test_source.m1_guideline_a"))
            .expect("guideline_a in the corpus");
        let source_id = guideline_a.id.clone();
        let key = |seed| CassetteKey {
            route: static_id("route.single_ir"),
            source: source_id.clone(),
            seed,
        };

        // guideline_a's deterministic grounding universe: the real region and segment ids
        // the accept closure grounds the model's references against (the head
        // `single_ir_fill` runs internally).
        let html = std::fs::read(root.join(&guideline_a.path)).unwrap();
        let config = ExtractConfig {
            document_id: guideline_a.id.clone(),
            source_family: static_id("synthetic_test_source_html"),
            provenance: guideline_a.provenance,
            data_class: DataClass::None,
            producer: producer(&resolved, 0),
        };
        let source = extract(&html, &config).unwrap();
        let segments = segment(&source, &producer(&resolved, 1)).unwrap();
        let regions: HashSet<&Id> = source
            .payload
            .regions
            .iter()
            .map(|r| &r.region_id)
            .collect();
        let segment_ids: HashSet<&Id> = segments
            .payload
            .segments
            .iter()
            .map(|s| &s.segment_id)
            .collect();

        // The §7.4 schema-violation shape, asserted wherever a malformed output surfaces
        // one: the parse reason rides the payload under the `reason` key, with no resolved
        // refs (symmetric to the hallucinated/exceeded payload pins below).
        let assert_schema_shape = |d: &DiagnosticRecord| {
            assert_eq!(d.code, DiagnosticCode::AiSchemaViolation);
            assert_eq!(d.outcome, Outcome::Invalid);
            assert!(d.region_ids.is_empty());
            assert!(d.artifact_hashes.is_empty());
            assert_eq!(d.payload.len(), 1);
            assert_eq!(d.payload[0].0, static_id("reason"));
            assert!(!d.payload[0].1.is_empty(), "the parse reason is recorded");
        };

        // guideline_a's golden ClinicalIr, decoded from its committed seed-42 cassette: the
        // recovery cassette replays these exact bytes, so a repaired fill recovers to them.
        let golden_bytes = store
            .replay(&key(42))
            .unwrap()
            .payload
            .output_bytes()
            .unwrap();

        // (a) HALLUCINATED (seed 99): canonical output citing an absent segment → a
        // terminal `ai_hallucinated_source` naming exactly the rebound id, no target, and
        // no repair spent despite the budget (grounding does not consume repairs).
        let fill = model_fill(
            &store,
            &key(99),
            FillSource::Replay,
            2,
            single_ir_accept(&regions, &segment_ids),
        )
        .unwrap();
        assert!(fill.target.is_none());
        assert_eq!(fill.repairs, 0);
        assert_eq!(fill.recorded_calls, 1);
        assert_eq!(fill.diagnostics.len(), 1);
        let d = &fill.diagnostics[0];
        assert_eq!(d.code, DiagnosticCode::AiHallucinatedSource);
        assert_eq!(d.outcome, Outcome::Invalid);
        assert!(d.region_ids.is_empty());
        assert!(d.artifact_hashes.is_empty());
        assert_eq!(
            d.payload,
            vec![(
                static_id("absent_source_ids"),
                "seg.hallucinated_absent".to_owned()
            )]
        );

        // (b) MALFORMED (seed 98) with a repair budget → one `ai_schema_violation`, then
        // the valid recovery at `derive_seed(98, 1)` is accepted — and the recovered IR is
        // exactly guideline_a's golden ClinicalIr, not merely some grounded fill.
        let fill = model_fill(
            &store,
            &key(98),
            FillSource::Replay,
            1,
            single_ir_accept(&regions, &segment_ids),
        )
        .unwrap();
        assert_eq!(fill.repairs, 1);
        assert_eq!(fill.recorded_calls, 2);
        assert_eq!(fill.diagnostics.len(), 1);
        assert_schema_shape(&fill.diagnostics[0]);
        let recovered = fill
            .target
            .expect("the repair seed's valid output is accepted");
        assert_eq!(canonical_payload_bytes(&recovered).unwrap(), golden_bytes);

        // The route fn surfaces the §7.4 fill diagnostics to its shell ledger: the
        // malformed→repair path lands one schema violation, then recovers to a bundle.
        {
            let tmp = tempfile::tempdir().unwrap();
            let out = tmp.path().join("m2");
            std::fs::create_dir_all(&out).unwrap();
            let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out));
            let bundle = single_ir_fill(
                &root,
                &guideline_a,
                &lexicon,
                &store,
                98,
                &resolved,
                1,
                &mut shell,
            );
            assert!(
                bundle.is_some(),
                "the malformed base repairs to an accepted bundle"
            );
            assert_eq!(shell.ledger().len(), 1);
            assert_schema_shape(&shell.ledger()[0]);
        }

        // The route fn yields `None` on a terminal reject and still surfaces its
        // diagnostic: the hallucinated cassette (seed 99) ends the route with
        // `ai_hallucinated_source` on the ledger and no bundle.
        {
            let tmp = tempfile::tempdir().unwrap();
            let out = tmp.path().join("m2");
            std::fs::create_dir_all(&out).unwrap();
            let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out));
            let bundle = single_ir_fill(
                &root,
                &guideline_a,
                &lexicon,
                &store,
                99,
                &resolved,
                2,
                &mut shell,
            );
            assert!(bundle.is_none(), "a hallucinated reference ends the route");
            assert_eq!(shell.ledger().len(), 1);
            assert_eq!(shell.ledger()[0].code, DiagnosticCode::AiHallucinatedSource);
        }

        // (c) MALFORMED at the base AND its first repair seed (97 + `derive_seed(97, 1)`)
        // with `repair_limit = 1` → the re-prompt path is traversed: a schema violation per
        // attempt, then a terminal `repair_limit_exceeded` naming the exhausted limit, no
        // target. (The zero-budget boundary is `model_fill.rs`'s coverage.)
        let fill = model_fill(
            &store,
            &key(97),
            FillSource::Replay,
            1,
            single_ir_accept(&regions, &segment_ids),
        )
        .unwrap();
        assert!(fill.target.is_none());
        assert_eq!(fill.repairs, 1);
        assert_eq!(fill.recorded_calls, 2);
        assert_eq!(fill.diagnostics.len(), 3);
        assert_schema_shape(&fill.diagnostics[0]);
        assert_schema_shape(&fill.diagnostics[1]);
        let last = &fill.diagnostics[2];
        assert_eq!(last.code, DiagnosticCode::RepairLimitExceeded);
        assert_eq!(last.outcome, Outcome::Invalid);
        assert_eq!(
            last.payload,
            vec![(static_id("repair_limit"), "1".to_owned())]
        );
    }

    /// route-direct-smt.3a — a minimal [`Resolved`] for the direct_smt route over
    /// the M1 inputs. `pipe.m2_direct_smt` is four stages (extract, segment,
    /// model_fill_smt, verify_smt), filling slots `[0]`–`[3]` of the fixed `[Id; 8]`;
    /// only `producer(resolved, 0..=3)` is ever read, so slots `[4]`–`[7]` hold an inert `processing_stage.unused`
    /// sentinel, a non-stage id, so an accidental read surfaces obviously rather
    /// than posing as a real verify stage. `documents` / `groups` / `plan` go unread and
    /// stay empty; `budget_ms` is exp.m1_scaffold's §8.4 `solver_ms_per_query`.
    fn direct_smt_resolved() -> Resolved {
        Resolved {
            pipeline_id: static_id("pipe.m2_direct_smt"),
            pipeline_step_ids: [
                static_id("processing_stage.m1.extract"),
                static_id("processing_stage.m1.segment"),
                static_id("processing_stage.m2.model_fill_smt"),
                static_id("processing_stage.m2.verify_smt"),
                static_id("processing_stage.unused"),
                static_id("processing_stage.unused"),
                static_id("processing_stage.unused"),
                static_id("processing_stage.unused"),
            ],
            documents: vec![],
            groups: vec![],
            budget_ms: 10_000,
            plan: RunPlan {
                experiment_id: static_id("exp.m2_multihop"),
                test_source_groups: vec![],
                pipelines: vec![],
                seed: 42,
                budget: vec![],
            },
            toolchain_manifest_hash: hash_bytes(b"direct-smt-fixture-toolchain"),
        }
    }

    /// The M1 reference query bodies per `exp.m1_scaffold` group: build each
    /// member's deterministic M1 bundle (the z3-free extract → segment → normalize
    /// → assemble chain, the same `assemble_bundle` the M1 run drives), then
    /// [`compile`] the group's members into `query_bodies` (overlap at `[0]`,
    /// deontic at `[1]` per planned pair). The golden `route.direct_smt` cassettes
    /// record these exact bytes, so the route's `:named a.<rule_id>` labels match the
    /// reference `expected_unsat_core` (route-direct-smt.4 scoring). Shared by the
    /// bless helper and the `.3b` fill gate.
    fn m1_reference_query_bodies(
        root: &Path,
        lexicon: &Lexicon,
    ) -> Vec<(Id, Vec<ckc_smt::QueryBody>)> {
        use std::collections::BTreeMap;

        let resolved = single_ir_resolved();
        // Build every M1 document's reference bundle once, keyed by test_source id.
        let mut bundles = BTreeMap::new();
        for entry in single_ir_corpus() {
            let html = std::fs::read(root.join(&entry.path)).unwrap();
            let config = ExtractConfig {
                document_id: entry.id.clone(),
                source_family: static_id("synthetic_test_source_html"),
                provenance: entry.provenance,
                data_class: DataClass::None,
                producer: producer(&resolved, 0),
            };
            let source = extract(&html, &config).unwrap();
            let segments = segment(&source, &producer(&resolved, 1)).unwrap();
            let normalization =
                normalize(&source, &segments, lexicon, &producer(&resolved, 2)).unwrap();
            let m1 =
                assemble_bundle(&entry, &resolved, &source, &segments, &normalization).unwrap();
            bundles.insert(entry.id.clone(), m1);
        }

        let experiments = parse_experiments(
            &std::fs::read_to_string(root.join("registry/experiments.yaml")).unwrap(),
        )
        .unwrap();
        let exp = experiments
            .iter()
            .find(|e| e.id == static_id("exp.m1_scaffold"))
            .expect("exp.m1_scaffold");

        exp.test_source_groups
            .iter()
            .map(|group| {
                let gid = group.group_id.clone();
                let members: Vec<_> = group
                    .test_sources
                    .iter()
                    .map(|s| {
                        bundles
                            .get(s)
                            .unwrap_or_else(|| panic!("{gid}: unfilled member {s}"))
                    })
                    .collect();
                let compiled = compile(
                    &gid,
                    members.iter().map(|m| (&m.payload.formal, &m.payload.norm)),
                );
                // Each exp.m1_scaffold group is one overlap+deontic pair, so `compile`
                // yields exactly two query bodies ([0]=overlap, [1]=deontic). Fail loudly
                // if a group ever yields another shape, rather than indexing past [1] or
                // silently dropping later pairs when bless/self-check take only [0]/[1].
                assert_eq!(
                    compiled.query_bodies.len(),
                    2,
                    "{gid}: expected 2 query bodies (one overlap+deontic pair), got {}",
                    compiled.query_bodies.len()
                );
                (gid, compiled.query_bodies)
            })
            .collect()
    }

    /// Craft one golden `route.direct_smt` cassette: wrap the raw SMT `output`
    /// bytes (the query text the model "emits") under §4.4 provenance via
    /// [`CassetteStore::build_wrapper`], keyed by the minted `<gid>.<role>` source.
    /// Synthetic model identity — the crafted-fixture rule.
    fn write_direct_smt_cassette(
        source_id: Id,
        resolved: &Resolved,
        store: &CassetteStore,
        seed: u64,
        output: &[u8],
    ) {
        let key = CassetteKey {
            route: static_id("route.direct_smt"),
            source: source_id.clone(),
            seed,
        };
        let payload = CassettePayload::from_output(
            static_id("route.direct_smt"),
            source_id,
            seed,
            "Emit the SMT-LIB query for the cited guideline pair.".to_owned(),
            hash_bytes(b"direct-smt-constraint"),
            hash_bytes(b"direct-smt-prompt-template"),
            ModelIdentity {
                model_id: static_id("model.baseline"),
                quant: "fixture_quant".to_owned(),
                runtime_version: "1.0.0".to_owned(),
            },
            output,
        );
        let wrapper = store
            .build_wrapper(&key, payload, producer(resolved, 2))
            .unwrap();
        store.persist(&key, wrapper).unwrap();
    }

    /// Regenerate the committed golden `route.direct_smt` cassettes: for each
    /// `exp.m1_scaffold` group, record its M1 `compile()` overlap query
    /// (`query_bodies[0]`) under `<gid>.overlap` and its deontic query
    /// (`query_bodies[1]`) under `<gid>.deontic`, both at seed 42, the raw SMT body
    /// bytes as recorded output. Run:
    /// `cargo test -p ckc-cli bless_direct_smt_cassettes -- --ignored` (a substring
    /// filter; `--exact` would need the full `run::tests::…` path and match nothing).
    #[test]
    #[ignore = "regenerates committed golden cassettes"]
    fn bless_direct_smt_cassettes() {
        let root = repo_root();
        let lexicon = single_ir_lexicon();
        let resolved = direct_smt_resolved();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        for (gid, qbodies) in m1_reference_query_bodies(&root, &lexicon) {
            write_direct_smt_cassette(
                static_id(&format!("{gid}.overlap")),
                &resolved,
                &store,
                42,
                qbodies[0].body.as_bytes(),
            );
            write_direct_smt_cassette(
                static_id(&format!("{gid}.deontic")),
                &resolved,
                &store,
                42,
                qbodies[1].body.as_bytes(),
            );
        }
    }

    /// route-direct-smt.3a self-check (model-runtime-absent): every committed golden
    /// `route.direct_smt` cassette replays to its group's freshly-compiled M1 query
    /// body — overlap under `<gid>.overlap`, deontic under `<gid>.deontic`. The `.3b`
    /// fill gate then proves the route reconstructs these wrappers.
    #[test]
    fn direct_smt_cassettes_carry_m1_query_bodies() {
        let root = repo_root();
        let lexicon = single_ir_lexicon();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        for (gid, qbodies) in m1_reference_query_bodies(&root, &lexicon) {
            for (role, i) in [("overlap", 0), ("deontic", 1)] {
                let key = CassetteKey {
                    route: static_id("route.direct_smt"),
                    source: static_id(&format!("{gid}.{role}")),
                    seed: 42,
                };
                let wrapper = store.replay(&key).unwrap();
                assert_eq!(
                    wrapper.payload.output_bytes().unwrap(),
                    qbodies[i].body.as_bytes(),
                    "{gid}.{role}"
                );
            }
        }
    }

    /// route-direct-smt.3b — the direct_smt route reconstructs the M1 query bodies
    /// (model-runtime-absent). For every `exp.m1_scaffold` group, [`direct_smt_fill`]
    /// replays the committed golden cassettes and rewraps the pair; each `QueryBody`
    /// body matches the freshly-compiled M1 body byte-for-byte, carrying the minted
    /// `<gid>.<role>` id, the role's logic, and the pinned raw-AI `smt_query`
    /// provenance the verdict tail (route-direct-smt.4) cites.
    #[test]
    fn direct_smt_fill_reproduces_m1_query_bodies() {
        use std::collections::BTreeMap;

        let root = repo_root();
        let lexicon = single_ir_lexicon();
        let resolved = direct_smt_resolved();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));

        // The M1 reference bodies, and the corpus entries keyed by test_source id so the
        // route's members resolve from `exp.m1_scaffold` (never a hardcoded membership).
        let refs = m1_reference_query_bodies(&root, &lexicon);
        let corpus: BTreeMap<Id, CorpusEntry> = single_ir_corpus()
            .into_iter()
            .map(|entry| (entry.id.clone(), entry))
            .collect();

        let experiments = parse_experiments(
            &std::fs::read_to_string(root.join("registry/experiments.yaml")).unwrap(),
        )
        .unwrap();
        let exp = experiments
            .iter()
            .find(|e| e.id == static_id("exp.m1_scaffold"))
            .expect("exp.m1_scaffold");

        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("m2");
        std::fs::create_dir_all(&out).unwrap();
        let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out));

        for group in &exp.test_source_groups {
            let gid = group.group_id.clone();
            let members: Vec<&CorpusEntry> = group
                .test_sources
                .iter()
                .map(|s| {
                    corpus
                        .get(s)
                        .unwrap_or_else(|| panic!("{gid}: unknown member {s}"))
                })
                .collect();
            let (overlap, deontic) =
                direct_smt_fill(&root, &gid, &members, &store, 42, &resolved, 1, &mut shell)
                    .unwrap_or_else(|| panic!("{gid}: direct_smt_fill yielded no pair"));

            let want = &refs
                .iter()
                .find(|(g, _)| *g == gid)
                .unwrap_or_else(|| panic!("{gid}: no reference query bodies"))
                .1;

            assert_eq!(overlap.payload.body, want[0].body, "{gid} overlap body");
            assert_eq!(
                overlap.payload.query_id,
                static_id(&format!("{gid}.overlap")),
                "{gid} overlap id"
            );
            assert_eq!(
                overlap.payload.logic,
                SmtLogic::QfLra,
                "{gid} overlap logic"
            );
            assert_eq!(deontic.payload.body, want[1].body, "{gid} deontic body");
            assert_eq!(
                deontic.payload.query_id,
                static_id(&format!("{gid}.deontic")),
                "{gid} deontic id"
            );
            assert_eq!(deontic.payload.logic, SmtLogic::QfUf, "{gid} deontic logic");

            // Provenance inputs: re-derive [source, segments] per member in group order —
            // the payload-only content_hash cannot catch a wrong or missing input hash, so
            // pin the full input_hashes independently of the body equality above.
            let mut want_inputs: Vec<Hash> = Vec::new();
            for m in &members {
                let html = std::fs::read(root.join(&m.path)).unwrap();
                let config = ExtractConfig {
                    document_id: m.id.clone(),
                    source_family: static_id("synthetic_test_source_html"),
                    provenance: m.provenance,
                    data_class: DataClass::None,
                    producer: producer(&resolved, 0),
                };
                let source = extract(&html, &config).unwrap();
                let segments = segment(&source, &producer(&resolved, 1)).unwrap();
                want_inputs.push(source.content_hash.clone());
                want_inputs.push(segments.content_hash.clone());
            }

            // The pinned raw-AI `smt_query` provenance: `validate()` enforces only the
            // effects↔status rule, so pin origin / status / kind / producer here.
            for w in [&overlap, &deontic] {
                assert_eq!(w.artifact_kind, static_id("smt_query"), "{gid} kind");
                assert_eq!(w.schema_id, static_id("schema.smt_query"), "{gid} schema");
                assert_eq!(w.origin, Origin::AiGenerated, "{gid} origin");
                assert_eq!(
                    w.evidence_status,
                    EvidenceStatus::AcceptedEvidenceStatus,
                    "{gid} status"
                );
                assert!(w.external_effects.is_empty(), "{gid} effects");
                assert_eq!(
                    w.producer.pipeline_step_id,
                    static_id("processing_stage.m2.model_fill_smt"),
                    "{gid} producer"
                );
                assert_eq!(w.input_hashes, want_inputs, "{gid} input_hashes");
                w.validate()
                    .unwrap_or_else(|e| panic!("{gid} wrapper validate: {e:?}"));
            }
        }
    }

    /// route-direct-smt.3b fail-closed guard: the cassette-role design mints exactly one
    /// (overlap, deontic) pair per group, so a non-pair member set must yield no artifact
    /// rather than a two-query wrapper that silently under-covers the group. The guard
    /// short-circuits ahead of any cassette or filesystem access, so repeating one corpus
    /// entry to reach a given cardinality is sufficient.
    #[test]
    fn direct_smt_fill_rejects_non_pair_group() {
        let root = repo_root();
        let resolved = direct_smt_resolved();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        let corpus = single_ir_corpus();
        let gid = static_id("group.malformed");
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("m2");
        std::fs::create_dir_all(&out).unwrap();
        for members in [vec![&corpus[0]], vec![&corpus[0], &corpus[0], &corpus[0]]] {
            let n = members.len();
            let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out.clone()));
            let got = direct_smt_fill(&root, &gid, &members, &store, 42, &resolved, 1, &mut shell);
            assert!(got.is_none(), "non-pair group (len {n}) must fail closed");
        }
    }

    /// route-direct-smt.4 — the direct_smt route scores the M1 conflict and no-conflict
    /// groups against `exp.m1_scaffold`'s reference (model-runtime-absent fill, live z3
    /// verdict). For every group [`direct_smt_fill`] replays the golden cassettes into
    /// the pair's two SMT bodies, [`direct_smt_verify_group`] runs the shared verdict
    /// engine over them, and the results match the reference the same way run_oracle's
    /// `assert_group_matches_reference` decides M1: a conflict group yields exactly one
    /// `semantic_contradiction` whose unsat core equals `expected_unsat_core` and rides
    /// the pair's deontic (`<gid>.deontic`) query; a no-conflict group yields no
    /// contradiction and every query `semantic_no_conflict`, with a documented
    /// no-conflict result closing on an unsat overlap query whose deontic query never
    /// ran. The direct route mints its own `<gid>.<role>` query ids (no
    /// `solver_query_plan`), so the no-conflict closure keys off those ids.
    #[test]
    fn direct_smt_route_scores_m1_groups() {
        use std::collections::{BTreeMap, BTreeSet};

        use ckc_smt::{SolverVerdict, VerifierCategory};

        let root = repo_root();
        let resolved = direct_smt_resolved();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        let adapter = Z3Adapter::new().expect("z3 adapter on PATH");

        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("m2");
        std::fs::create_dir_all(&out).unwrap();
        let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out));

        // Corpus entries keyed by test_source id so the groups and the reference resolve
        // from `exp.m1_scaffold` (the run_oracle registry-driven shape), never a
        // hardcoded membership list.
        let corpus: BTreeMap<Id, CorpusEntry> = single_ir_corpus()
            .into_iter()
            .map(|entry| (entry.id.clone(), entry))
            .collect();

        let experiments = parse_experiments(
            &std::fs::read_to_string(root.join("registry/experiments.yaml")).unwrap(),
        )
        .unwrap();
        let exp = experiments
            .iter()
            .find(|e| e.id == static_id("exp.m1_scaffold"))
            .expect("exp.m1_scaffold");
        let reference =
            parse_reference(&std::fs::read_to_string(root.join(&exp.expected_outcomes)).unwrap())
                .unwrap();
        assert_eq!(
            reference.len(),
            exp.test_source_groups.len(),
            "one reference entry per test_source group"
        );

        for group in &exp.test_source_groups {
            let gid = group.group_id.clone();
            let members: Vec<&CorpusEntry> = group
                .test_sources
                .iter()
                .map(|s| {
                    corpus
                        .get(s)
                        .unwrap_or_else(|| panic!("{gid}: unknown member {s}"))
                })
                .collect();
            let (overlap, deontic) =
                direct_smt_fill(&root, &gid, &members, &store, 42, &resolved, 0, &mut shell)
                    .unwrap_or_else(|| panic!("{gid}: direct_smt_fill yielded no pair"));
            let results = direct_smt_verify_group(
                &gid,
                &format!("groups/{gid}"),
                &overlap,
                &deontic,
                &resolved,
                &adapter,
                &mut shell,
            )
            .unwrap_or_else(|| panic!("{gid}: no verifier results"));

            // The verdict tail stamps `pipe.m2_direct_smt`'s slot-3 verify_smt step.
            assert_eq!(
                results.producer.pipeline_step_id,
                static_id("processing_stage.m2.verify_smt"),
                "{gid}"
            );

            // Wrapper provenance: the verifier_results cite the pair's two
            // smt_query bodies (not single_ir's one `compiled`), stamped
            // external-adapter verifier evidence. input_hashes are a §4.3
            // canonical set (the landed wrapper sorts them by hash), so compare
            // as a set rather than pinning the emitted order.
            let pair_inputs =
                BTreeSet::from([overlap.content_hash.clone(), deontic.content_hash.clone()]);
            assert_eq!(
                results
                    .input_hashes
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                pair_inputs,
                "{gid}: verifier_results cite the pair's two smt_query bodies"
            );
            assert_eq!(results.origin, Origin::ExternalAdapterGenerated, "{gid}");
            assert_eq!(
                results.evidence_status,
                EvidenceStatus::VerifierEvidenceStatus,
                "{gid}"
            );
            assert_eq!(
                results.artifact_kind,
                static_id("verifier_results"),
                "{gid}"
            );

            // The directly-emitted §4.6 verify event: the slot-3 verify_smt step
            // is stamped `verify` (not slot-3's `assemble`) and carries the solver
            // budget counter — the two deviations from the index-coupled
            // finish_processing_stage this tail hand-rolls to avoid.
            let event = shell.events().last().expect("a verify event");
            assert_eq!(
                event.processing_stage,
                static_id("verify"),
                "{gid}: event kind"
            );
            assert_eq!(
                event.pipeline_id, resolved.pipeline_id,
                "{gid}: event pipeline"
            );
            assert_eq!(
                event.pipeline_step_id,
                static_id("processing_stage.m2.verify_smt"),
                "{gid}: event step"
            );
            assert_eq!(
                event.resource_counters,
                vec![(static_id(SOLVER_BUDGET_KEY), resolved.budget_ms)],
                "{gid}: solver budget counter rides the direct verify event"
            );
            assert_eq!(
                event.input_hashes.iter().cloned().collect::<BTreeSet<_>>(),
                pair_inputs,
                "{gid}: event inputs = the two smt_query bodies (as a set)"
            );
            assert_eq!(
                event.output_hashes,
                vec![results.content_hash.clone()],
                "{gid}: event output = the landed verifier_results"
            );
            assert_eq!(event.outcome, Outcome::Ok, "{gid}: a clean verdict");

            let overlap_id = static_id(&format!("{gid}.overlap"));
            let deontic_id = static_id(&format!("{gid}.deontic"));

            // Score vs the reference (run_oracle's assert_group_matches_reference shape).
            let entry = reference
                .iter()
                .find(|e| e.group_id == gid)
                .unwrap_or_else(|| panic!("{gid}: no reference entry"));
            let contradictions: Vec<_> = results
                .payload
                .results
                .iter()
                .filter(|r| r.category == VerifierCategory::SemanticContradiction)
                .collect();
            if entry.expected_outcome == static_id("semantic_contradiction") {
                assert_eq!(contradictions.len(), 1, "{gid}: exactly one contradiction");
                let hit = contradictions[0];
                assert_eq!(
                    hit.query_id, deontic_id,
                    "{gid}: the contradiction rides the pair's deontic query"
                );
                assert_eq!(
                    entry.expected_conflict_kind,
                    Some(static_id("deontic_direction_conflict")),
                    "{gid}: a deontic Q2 contradiction is the deontic_direction_conflict kind"
                );
                let core: BTreeSet<Id> = hit
                    .unsat_core
                    .clone()
                    .expect("an unsat verdict carries its core")
                    .into_iter()
                    .collect();
                assert_eq!(
                    core, entry.expected_unsat_core,
                    "{gid}: unsat core as a set"
                );
            } else if entry.expected_outcome == static_id("semantic_no_conflict") {
                assert!(contradictions.is_empty(), "{gid}: no contradiction");
                assert!(
                    results
                        .payload
                        .results
                        .iter()
                        .all(|r| r.category == VerifierCategory::SemanticNoConflict),
                    "{gid}: every query closed semantic_no_conflict"
                );
                if entry.expected_no_conflict_result {
                    // §6 no-conflict closure: the pair's overlap query answered unsat and
                    // its deontic query never ran. The direct route lacks a
                    // solver_query_plan, so key off the minted <gid>.overlap/.deontic ids.
                    let overlap_result = results
                        .payload
                        .results
                        .iter()
                        .find(|r| r.query_id == overlap_id)
                        .unwrap_or_else(|| panic!("{gid}: no overlap result"));
                    assert_eq!(
                        overlap_result.verdict,
                        Some(SolverVerdict::Unsat),
                        "{gid}: the documented no-conflict overlap query closed unsat"
                    );
                    assert!(
                        results
                            .payload
                            .results
                            .iter()
                            .all(|r| r.query_id != deontic_id),
                        "{gid}: the closed pair skipped its deontic query"
                    );
                }
            } else {
                panic!(
                    "{gid}: unhandled expected_outcome {}",
                    entry.expected_outcome
                );
            }
        }
    }

    /// Bless the committed `route.direct_smt` rejection cassettes route-direct-smt.5
    /// replays. Both crafted groups are keyed under the minted `<gid>.<role>` sources
    /// [`direct_smt_fill`] reads. `group.m2_direct_schema` drives §7.4 schema exhaustion:
    /// its overlap source carries a non-SMT base (seed 91) and a non-SMT first-repair
    /// output (`derive_seed(91, 1)`), so a `repair_limit = 1` fill schema-fails at the
    /// base, re-prompts under the derived seed, schema-fails again, and terminates in
    /// `repair_limit_exceeded` (its deontic source stays unwritten — the overlap query
    /// exhausts first). `group.m2_direct_syntax` drives the direct-route-unique
    /// `target_syntax_failure`: its overlap source (seed 90) shallow-accepts — valid
    /// utf-8, a `(set-logic` head, a `(check-sat)` substring — yet z3 rejects the
    /// unbalanced parens with an `(error …)` and no verdict; its deontic source (seed 90)
    /// is a minimal valid query that never runs (Q1 fails first). Synthetic-identity
    /// crafted-fixture cassettes, so the engine-agnostic audit applies. `#[ignore]`d: run
    /// to regenerate, then commit the four json. Regenerate with
    /// `cargo test -p ckc-cli bless_direct_smt_rejection_cassettes -- --ignored`.
    #[test]
    #[ignore = "regenerates the committed rejection cassettes"]
    fn bless_direct_smt_rejection_cassettes() {
        let resolved = direct_smt_resolved();
        let store = CassetteStore::new(repo_root().join("crates/ckc-cli/tests/fixtures"));

        // SCHEMA EXHAUSTION (group.m2_direct_schema.overlap): a non-SMT base and a
        // non-SMT first-repair output, both failing direct_smt_accept's shallow check,
        // drive a repair_limit = 1 fill to a terminal repair_limit_exceeded.
        write_direct_smt_cassette(
            static_id("group.m2_direct_schema.overlap"),
            &resolved,
            &store,
            91,
            b"not an smt query (base)\n",
        );
        write_direct_smt_cassette(
            static_id("group.m2_direct_schema.overlap"),
            &resolved,
            &store,
            crate::model::derive_seed(91, 1),
            b"not an smt query (repair 1)\n",
        );

        // SYNTAX FAILURE (group.m2_direct_syntax): the overlap source shallow-accepts
        // (utf-8, a (set-logic head, a (check-sat) substring) yet z3 rejects its
        // unbalanced parens → target_syntax_failure at verify. The deontic source is a
        // minimal valid query that never runs (Q1 fails first).
        write_direct_smt_cassette(
            static_id("group.m2_direct_syntax.overlap"),
            &resolved,
            &store,
            90,
            b"(set-logic QF_UF)\n(declare-const x Bool)\n(assert (and x\n(check-sat)\n",
        );
        write_direct_smt_cassette(
            static_id("group.m2_direct_syntax.deontic"),
            &resolved,
            &store,
            90,
            b"(set-logic QF_UF)\n(check-sat)\n",
        );
    }

    /// route-direct-smt.5 — the direct_smt route's §7.4 rejection codes wire through
    /// [`model_fill`] under [`direct_smt_accept`] and the §4.6 verify event (z3 present,
    /// model-runtime-absent). Replaying the committed bad cassettes: (a) a
    /// schema-exhaustion group whose overlap base (seed 91) and first-repair
    /// (`derive_seed(91, 1)`) outputs both fail the shallow SMT accept — a
    /// `repair_limit = 1` fill re-prompts once and terminates in `repair_limit_exceeded`,
    /// surfaced both at the [`model_fill`] boundary and on the route fn's shell ledger,
    /// with no target and the deontic source never read (Q1 exhausts first); (b) a
    /// syntax-failure group — direct-route-unique, no single_ir analogue: the overlap
    /// body shallow-accepts (so [`direct_smt_fill`] lands the pair with an empty ledger)
    /// yet z3 rejects its unbalanced parens with an `(error …)` and no verdict, so
    /// [`direct_smt_verify_group`] mints a lone `target_syntax_failure` /
    /// `target_parse_error` result and the §4.6 verify event surfaces it to the ledger.
    /// The repair-loop mechanics live in `model_fill.rs`; this pins the direct route's
    /// accept → §7.4 code selection end-to-end.
    #[test]
    fn direct_smt_route_rejection_codes() {
        use ckc_core::{DiagnosticCode, Outcome};
        use ckc_smt::VerifierCategory;

        let root = repo_root();
        let resolved = direct_smt_resolved();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        let corpus = single_ir_corpus();
        let guideline_a = corpus
            .iter()
            .find(|e| e.id == static_id("test_source.m1_guideline_a"))
            .expect("guideline_a in the corpus");
        let guideline_b = corpus
            .iter()
            .find(|e| e.id == static_id("test_source.m1_guideline_b"))
            .expect("guideline_b in the corpus");
        let members = vec![guideline_a, guideline_b];
        let adapter = Z3Adapter::new().expect("z3 adapter on PATH");

        // The schema group's overlap source — the first cassette direct_smt_fill reads —
        // keyed at a given seed (the golden test's key-construction shape).
        let schema_key = |seed| CassetteKey {
            route: static_id("route.direct_smt"),
            source: static_id("group.m2_direct_schema.overlap"),
            seed,
        };

        // (a) SCHEMA EXHAUSTION at the model_fill boundary: base (seed 91) and first
        // repair (derive_seed(91, 1)) both fail direct_smt_accept's shallow check, so a
        // repair_limit = 1 fill re-prompts once, then exhausts — no target, one repair,
        // two recorded calls, two schema violations, then the terminal repair-limit code.
        let fill = model_fill(
            &store,
            &schema_key(91),
            FillSource::Replay,
            1,
            direct_smt_accept(),
        )
        .unwrap();
        assert!(fill.target.is_none());
        assert_eq!(fill.repairs, 1);
        assert_eq!(fill.recorded_calls, 2);
        assert_eq!(fill.diagnostics.len(), 3);
        // Pin BOTH schema violations to the same shape (mirrors single_ir_route_rejection_codes).
        let assert_schema_shape = |d: &DiagnosticRecord| {
            assert_eq!(d.code, DiagnosticCode::AiSchemaViolation);
            assert_eq!(d.outcome, Outcome::Invalid);
            assert!(d.region_ids.is_empty());
            assert!(d.artifact_hashes.is_empty());
            assert_eq!(d.payload.len(), 1);
            assert_eq!(d.payload[0].0, static_id("reason"));
            assert!(!d.payload[0].1.is_empty(), "the parse reason is recorded");
        };
        assert_schema_shape(&fill.diagnostics[0]);
        assert_schema_shape(&fill.diagnostics[1]);
        let last = &fill.diagnostics[2];
        assert_eq!(last.code, DiagnosticCode::RepairLimitExceeded);
        assert_eq!(last.outcome, Outcome::Invalid);
        assert_eq!(
            last.payload,
            vec![(static_id("repair_limit"), "1".to_owned())]
        );

        // The route fn surfaces the same §7.4 codes on its shell ledger, then yields None
        // (the overlap query exhausts before the deontic source is read).
        {
            let tmp = tempfile::tempdir().unwrap();
            let out = tmp.path().join("m2");
            std::fs::create_dir_all(&out).unwrap();
            let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out));
            let filled = direct_smt_fill(
                &root,
                &static_id("group.m2_direct_schema"),
                &members,
                &store,
                91,
                &resolved,
                1,
                &mut shell,
            );
            assert!(filled.is_none(), "schema exhaustion ends the route");
            let codes: Vec<_> = shell.ledger().iter().map(|d| d.code).collect();
            assert_eq!(
                codes,
                vec![
                    DiagnosticCode::AiSchemaViolation,
                    DiagnosticCode::AiSchemaViolation,
                    DiagnosticCode::RepairLimitExceeded,
                ]
            );
        }

        // (b) SYNTAX FAILURE (direct-route-unique terminal, no repair): the overlap body
        // shallow-accepts, so direct_smt_fill lands the pair with an empty ledger; z3 then
        // rejects the unbalanced parens at verify.
        let syntax_gid = static_id("group.m2_direct_syntax");
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("m2");
        std::fs::create_dir_all(&out).unwrap();
        let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out));

        let (overlap, deontic) = direct_smt_fill(
            &root,
            &syntax_gid,
            &members,
            &store,
            90,
            &resolved,
            0,
            &mut shell,
        )
        .expect("the shallow-accepting pair fills");
        assert!(
            shell.ledger().is_empty(),
            "a shallow-accepting fill raises no diagnostics"
        );

        let results = direct_smt_verify_group(
            &syntax_gid,
            &format!("groups/{syntax_gid}"),
            &overlap,
            &deontic,
            &resolved,
            &adapter,
            &mut shell,
        )
        .expect("the verify tail yields results");
        assert_eq!(
            results.payload.results.len(),
            1,
            "Q1's syntax failure closes the pair before Q2 runs"
        );
        let result = &results.payload.results[0];
        assert_eq!(result.query_id, static_id("group.m2_direct_syntax.overlap"));
        assert_eq!(result.category, VerifierCategory::TargetSyntaxFailure);
        assert_eq!(result.diagnostics[0].code, DiagnosticCode::TargetParseError);
        assert_eq!(result.diagnostics[0].outcome, Outcome::Invalid);

        // The §4.6 verify event extended the ledger with the parse-error diagnostic.
        let codes: Vec<_> = shell.ledger().iter().map(|d| d.code).collect();
        assert_eq!(codes, vec![DiagnosticCode::TargetParseError]);
    }
}
