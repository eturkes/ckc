//! SPEC §3 `ckc run` (cli-runner.2a + .2b + .3a.3 + .4.1b): resolve the
//! experiment through the §8.4 registries, drive each corpus document through
//! extract → segment → normalize → assemble into
//! `artifacts/<doc-id>/{source_document_graph,segments,normalization,ir_bundle}.json`,
//! drive each test_source group through compile → verify into
//! `groups/<gid>/{compiled.json,verifier_results.json,smt/<query-id>.smt2}`,
//! assemble the run-scoped §7.1 trace pair over the landed trace-DAG artifacts
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
//! A model-route experiment (SPEC §9) instead runs each route pipeline into its
//! own `routes/<pipeline_id>/{artifacts,groups}/` subtree under one shared run
//! out ([`execute_routes`]); the run-level trace/report/manifest tails then run
//! once over both routes' collected docs, source graphs, and group traces,
//! landing the run artifacts at the bare run root.
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
//! pipeline_step_id = the registry processing_stage entry (the model-route run-level
//! tails span both routes, so they carry the baseline route's pipeline_id paired with a
//! synthetic run-level step id — `RUN_TRACE_STEP`/`RUN_REPORT_STEP`, neither a route
//! pipeline's declared stage nor the padded [`UNUSED_STAGE`] slot the tails used before
//! run-m2.1e-A; this step id is write-only provenance, never read back for logic),
//! toolchain manifest hash = the §4.4 raw-byte hash of [`TOOLCHAIN_FILE`], read once at
//! resolution and shared verbatim with the §5/§4.6 manifests.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ckc_core::{
    ArtifactWrapper, Candidates, CanonError, CanonRead, Canonical, ClinicalIr, ClinicalSegment,
    CorpusEntry, DataClass, DiagnosticCode, DiagnosticRecord, EvidenceStatus, Hash, Id, IrBundle,
    ModelIdentity, Normalization, Origin, Outcome, PipelineEntry, Producer, PromptEntry, RunPlan,
    SchemaEntry, SegmentIr, SolverIdentity, SourceDocumentGraph, TestSourceGroup, assemble,
    canonical_payload_bytes, canonical_sort_key, canonicalization_policy_hash, content_hash,
    hash_bytes, parse_candidates, parse_corpora, parse_experiments, parse_prompts, parse_reference,
    parse_schemas, read_strict_canonical, validate_model_registry,
};
use ckc_smt::{
    QueryBody, SmtLogic, VerifierResults, Z3Adapter, compile, verify, verify_query_pairs,
};

use crate::cassette::{CassetteKey, CassetteStore, RecordContext};
use crate::extract::{ExtractConfig, extract};
use crate::manifests::{ManifestInputs, assemble_manifests};
use crate::metrics::{FillObservation, GroupObservation, RouteMetrics, route_metrics};
use crate::model::ModelAdapter;
use crate::model_fill::{
    FillReject, FillSource, ModelFill, RECORDED_CALLS_COUNTER, REPAIRS_COUNTER, model_fill,
};
use crate::normalize::{Lexicon, load_lexicon, normalize};
use crate::registry_check::{invalid_diagnostic, load};
use crate::report::{
    ModelRunSections, Report, assemble_report, render_markdown, render_markdown_ja,
};
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

/// §8.4 experiment registry; resolved for the experiment binding and, on a
/// model-route run, raw-byte read for the §9 `reference_hash` lookup.
const EXPERIMENTS_FILE: &str = "registry/experiments.yaml";

/// §14 schema registry; a model-route run reads it for the §9 `schema_hash`
/// over the route-relevant `schemas/` entries.
const SCHEMAS_FILE: &str = "registry/schemas.yaml";

/// §14 prompt registry; a model-route run reads it for the §9
/// `prompt_template_hash` over the route-relevant prompt entries.
const PROMPTS_FILE: &str = "registry/prompts.yaml";

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

/// The model_fill slot shared by both model-route step lists
/// ([`SINGLE_IR_STAGE_KINDS`] and [`DIRECT_SMT_STAGE_KINDS`]). The M1 kind
/// table holds `normalize` at index 2, so the index-coupled event helpers
/// cannot serve this slot: the fill stage emits its §4.6 event directly
/// ([`direct_smt_verify_group`]'s pattern).
const MODEL_FILL: usize = 2;

/// Declared processing-stage kind sequences the two §9 model-route pipelines
/// fingerprint to in [`resolve_route`]: single_ir hands the model-filled IR
/// to the M1 assemble→compile→verify back end; direct_smt goes straight from
/// the model's smt_query output to verification (no IR, no compile).
const SINGLE_IR_STAGE_KINDS: [&str; 6] = [
    "extract",
    "segment",
    "model_fill",
    "assemble",
    "compile",
    "verify",
];
const DIRECT_SMT_STAGE_KINDS: [&str; 4] = ["extract", "segment", "model_fill", "verify"];

/// Inert padding id for the `[Id; 8]` slots past a route's declared stages —
/// a non-stage sentinel, so an accidental read of an unused slot surfaces as
/// an obviously wrong producer instead of a real stage id.
const UNUSED_STAGE: &str = "processing_stage.unused";

/// Run-level tail step ids: a model-route run's trace/report tails run once
/// over every route, owned by no single route pipeline (their per-route
/// `pipeline_step_ids[TRACE|REPORT]` slot is the inert [`UNUSED_STAGE`]
/// sentinel), so the landed wrappers carry a run-scoped step id instead. The
/// M1 baseline keeps its real per-pipeline trace/report step.
const RUN_TRACE_STEP: &str = "processing_stage.run.trace";
const RUN_REPORT_STEP: &str = "processing_stage.run.report";

/// §8.4 budget counter naming the per-query solver wall-clock cap in
/// milliseconds — the one budget key the M1 vocabulary defines.
const SOLVER_BUDGET_KEY: &str = "solver_ms_per_query";

/// §9 budget key naming the model-route repair cap: how many `derive_seed`
/// re-prompts one fill may spend before `repair_limit_exceeded`. [`resolve`]
/// requires it whenever the experiment binds a model route; the M1 layered
/// shape never reads it.
const MODEL_REPAIR_LIMIT_KEY: &str = "model_repair_limit";

/// §9 budget key naming the per-fill sample count. Replay executes exactly
/// one recorded draw per attempt, so on a model-route binding [`resolve`]
/// accepts only the value 1 when the key is present — a larger count would
/// promise samples the recorded run never draws (k-sample convergence waits
/// on a sampling config, a downstream decision).
const MODEL_SAMPLE_COUNT_KEY: &str = "model_sample_count";

/// §9 budget key naming the per-invocation model wall-clock cap in
/// milliseconds — the recording counterpart of [`SOLVER_BUDGET_KEY`].
/// [`resolve`] reads it unconditionally into an `Option`; only a `--record`
/// run requires it ([`build_record_parts`] fails loudly without it), since
/// replay never invokes the runtime.
const MODEL_MS_PER_CALL_KEY: &str = "model_ms_per_call";

/// Run `ckc run` rooted at `root` (the invocation working directory: §3
/// anchors `registry/` and corpus paths at the repository root). Evidence,
/// artifacts, and the outcome land entirely in the shell.
pub(crate) fn execute(root: &Path, experiment_id: &Id, record: bool, shell: &mut Shell) {
    let Some(mut views) = resolve(root, experiment_id, shell) else {
        return;
    };
    // run-m2.1d5a dispatch on the resolved route set. A lone layered M1 view runs
    // today's path below verbatim; a model-route-only set runs the route loop; a set
    // mixing the layered M1 pipeline with model routes has no defined joint execution
    // and fails closed with one command diagnostic, landing nothing.
    if views.len() != 1 || views[0].shape != RouteShape::M1Layered {
        if views.iter().any(|v| v.shape == RouteShape::M1Layered) {
            shell.diagnostic(invalid_diagnostic(vec![(
                static_id("reason"),
                format!(
                    "experiment {experiment_id} mixes the layered M1 pipeline with model \
                     routes; the run command executes a single-shape route set"
                ),
            )]));
        } else {
            execute_routes(root, &views, record, shell);
        }
        return;
    }
    let resolved = views.pop().expect("single layered M1 view checked above");
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
    let Some((bundle, lineage)) = trace_processing_stage(&docs, &groups, &resolved, shell, true)
    else {
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
        true,
        None,
        &[],
        &[],
    );
}

/// One model route's collected observations, held for the §7.3 metrics
/// (run-m2.1e): the route's pipeline id, its slice of the run ledger, the per-fill
/// observations, the per-group verdict observations, and the k-sample battery — one
/// recorded draw here, so `samples` holds a single `groups` snapshot.
#[allow(dead_code)]
struct RouteRun {
    pipeline_id: Id,
    ledger: Vec<DiagnosticRecord>,
    fills: Vec<FillObservation>,
    groups: Vec<GroupObservation>,
    samples: Vec<Vec<GroupObservation>>,
}

/// Fold one fill's model identity into the run's agreed identity: a fill with no
/// identity agrees vacuously; the first `Some` sets it; a later differing `Some` is a
/// fail-closed conflict — the run's routes must attest one evaluator — recorded as one
/// command diagnostic. Returns `false` on conflict so the caller stops the run. The
/// golden cassettes all agree, so the clean route loop never trips this.
fn agree_model_identity(
    agreed: &mut Option<ModelIdentity>,
    candidate: Option<ModelIdentity>,
    shell: &mut Shell,
) -> bool {
    let Some(candidate) = candidate else {
        return true;
    };
    match agreed {
        Some(existing) if *existing != candidate => {
            shell.diagnostic(invalid_diagnostic(vec![(
                static_id("reason"),
                "model routes disagree on the model identity attesting the run".to_owned(),
            )]));
            false
        }
        Some(_) => true,
        None => {
            *agreed = Some(candidate);
            true
        }
    }
}

/// The model-route execution loop (run-m2.1d5a): run each resolved model route
/// (single_ir, direct_smt) over the shared locked inputs, landing its per-route
/// artifacts under `routes/{pipeline_id}/`. [`execute`] reaches here only for a set
/// whose every view is a model route — it runs a lone layered M1 view inline and
/// fails a mixed set closed — so a layered M1 view here is unreachable.
///
/// Each route reuses the already-built route processing-stages: [`route_document_head`]
/// lands the deterministic head, then single_ir fills one ClinicalIR bundle per
/// document ([`single_ir_fill`]) and compiles + verifies each group
/// ([`compile_verify_group`]), while direct_smt fills one overlap/deontic SMT pair per
/// group ([`direct_smt_fill`]) and verifies it ([`direct_smt_verify_group`]). Group
/// artifacts land route-namespaced under `routes/{pipeline_id}/groups/{gid}` (the
/// head-namespacing mirror), so the two routes never collide under the one run out.
///
/// The per-route observations collect into a [`RouteRun`] each for the §7.3
/// metrics (run-m2.1e); the run-level trace/report tails over both routes then
/// run once, over every route's docs, source graphs, and group traces.
fn execute_routes(root: &Path, views: &[Resolved], record: bool, shell: &mut Shell) {
    // §7.2's lexicon hash rides the raw reference-file bytes, mirroring the M1 path;
    // the run-level report tail below hashes them.
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
    // The cassette store points at `<root>/cassettes/` (infallible); the solver
    // adapter is load-bearing for every group verdict, so its failure is command-scope.
    let store = CassetteStore::new(root);
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
    // Record mode (run-m2.1f): `ckc run --record` invokes the runtime once per
    // model fill and persists each cassette. Probe the live adapter and load the
    // validated committed model registry once here; a replay run (the default)
    // builds none of this and never touches the runtime.
    let record_setup = if record {
        match build_record_setup(root, shell) {
            Some(setup) => Some(setup),
            None => return,
        }
    } else {
        None
    };
    // The plan (and its seed) is shared across the resolved set (= experiment.seed).
    let seed = views[0].plan.seed;

    // One agreed model identity across every route's fills (see [`agree_model_identity`]).
    let mut agreed: Option<ModelIdentity> = None;
    let mut route_runs: Vec<RouteRun> = Vec::with_capacity(views.len());
    // The realized shape of each route, handed to the run-level manifest for its
    // §9 route-relevant schema/prompt selection.
    let mut model_routes: Vec<RouteShape> = Vec::with_capacity(views.len());

    // The run-level trace/report tails run once over every route's docs, source
    // graphs, and group traces; both routes chain in parallel under the one run
    // trace, so collect them across the view loop below.
    let mut all_docs: Vec<DocTrace> = Vec::new();
    let mut all_graphs: Vec<ArtifactWrapper<SourceDocumentGraph>> = Vec::new();
    let mut all_group_traces: Vec<GroupTrace> = Vec::new();

    for resolved in views {
        let ledger_start = shell.ledger().len();
        let repair_limit = resolved
            .repair_limit
            .expect("a model route resolves Some repair limit");
        let mut fills: Vec<FillObservation> = Vec::new();
        let mut groups: Vec<GroupObservation> = Vec::new();

        // This route's record-mode inputs (schema/prompt/context), assembled once
        // per view for both fills below; `None` on a replay run, so the fills
        // default to cassette replay. A registry gap here is command-scope.
        let route_record = match &record_setup {
            Some(setup) => match build_route_record(root, setup, resolved) {
                Ok(record) => Some(record),
                Err(reason) => {
                    shell.diagnostic(invalid_diagnostic(vec![(static_id("reason"), reason)]));
                    return;
                }
            },
            None => None,
        };

        match resolved.shape {
            RouteShape::SingleIr => {
                // One ClinicalIR bundle per unique document, keyed by document id for
                // the group compile below; a document whose head fails is skipped (its
                // diagnostic already raised), leaving its groups member-short.
                let mut bundles: BTreeMap<Id, ArtifactWrapper<IrBundle>> = BTreeMap::new();
                for entry in &resolved.documents {
                    let Some(head) = route_document_head(root, entry, resolved, shell) else {
                        continue;
                    };
                    let rd = single_ir_fill(
                        head,
                        &lexicon,
                        &store,
                        seed,
                        resolved,
                        repair_limit,
                        route_record.as_ref(),
                        shell,
                    );
                    if !agree_model_identity(&mut agreed, rd.identity, shell) {
                        return;
                    }
                    fills.extend(rd.fill);
                    // Clone the bundle into the compile map so `rd.trace` keeps it for
                    // the run trace's lineage row.
                    if let Some(bundle) = &rd.trace.bundle {
                        bundles.insert(entry.id.clone(), bundle.clone());
                    }
                    all_graphs.push(rd.graph);
                    all_docs.push(rd.trace);
                }
                for group in &resolved.groups {
                    // Compile needs every member bundle. A member-short group fails its
                    // compile processing_stage (the module's partial-group rule, mirroring
                    // M1's `group_pipeline`) rather than scoring a partial verdict, then the
                    // loop proceeds; the short member's own upstream failure already raised
                    // its diagnostic.
                    let mut members: Vec<&ArtifactWrapper<IrBundle>> =
                        Vec::with_capacity(group.test_sources.len());
                    let mut member_short = false;
                    for s in &group.test_sources {
                        match bundles.get(s) {
                            Some(bundle) => members.push(bundle),
                            None => {
                                let built = Err(processing_stage_diagnostic(
                                    COMPILE,
                                    "group",
                                    &group.group_id,
                                    format!("member {s} landed no ir_bundle artifact"),
                                ));
                                finish_processing_stage::<IrBundle>(
                                    shell,
                                    resolved,
                                    COMPILE,
                                    processing_stage_clock(),
                                    Vec::new(),
                                    built,
                                );
                                member_short = true;
                                break;
                            }
                        }
                    }
                    if member_short {
                        continue;
                    }
                    let dir = route_group_dir(resolved, &group.group_id);
                    let member_bundles: Vec<Id> =
                        members.iter().map(|m| m.artifact_id.clone()).collect();
                    let (compiled, results) = compile_verify_group(
                        &group.group_id,
                        &dir,
                        &members,
                        processing_stage_clock(),
                        resolved,
                        &adapter,
                        shell,
                    );
                    if let Some(compiled) = compiled {
                        // A verified group feeds the run report's group row.
                        if let Some(results) = &results {
                            groups.push(GroupObservation {
                                group_id: group.group_id.clone(),
                                query_pairs: compiled
                                    .payload
                                    .solver_query_plan
                                    .iter()
                                    .map(|p| {
                                        (
                                            p.context_overlap_query_id.clone(),
                                            p.deontic_consistency_query_id.clone(),
                                        )
                                    })
                                    .collect(),
                                results: results.payload.results.clone(),
                            });
                        }
                        // The compiled group chains into the run trace whenever the
                        // compiled artifact landed, so a landed-but-unverified group
                        // stays replay-covered (single_ir mints no smt_query pair).
                        all_group_traces.push(GroupTrace {
                            group_id: group.group_id.clone(),
                            test_sources: group.test_sources.clone(),
                            member_bundles,
                            dir,
                            smt_queries: Vec::new(),
                            compiled: Some(compiled),
                            verifier_results: results,
                        });
                    }
                }
            }
            RouteShape::DirectSmt => {
                // Head prepass: build each unique document's deterministic head once
                // in first-appearance order (`resolved.documents`), reused across the
                // groups that share a member.
                let mut heads: BTreeMap<Id, DocHead> = BTreeMap::new();
                for entry in &resolved.documents {
                    if let Some(head) = route_document_head(root, entry, resolved, shell) {
                        heads.insert(entry.id.clone(), head);
                    }
                }
                for group in &resolved.groups {
                    let gid = group.group_id.clone();
                    // The pair fill needs every member head; a member-short group is
                    // skipped (the direct route mints no compiled artifact, so the module's
                    // compile-stage partial-group rule does not apply — the short member's
                    // head failure already raised its diagnostic upstream).
                    let Some(head_refs) = group
                        .test_sources
                        .iter()
                        .map(|s| heads.get(s))
                        .collect::<Option<Vec<&DocHead>>>()
                    else {
                        continue;
                    };
                    let df = direct_smt_fill(
                        &gid,
                        &head_refs,
                        &store,
                        seed,
                        resolved,
                        repair_limit,
                        route_record.as_ref(),
                        shell,
                    );
                    fills.extend(df.fills);
                    for identity in df.identities {
                        if !agree_model_identity(&mut agreed, Some(identity), shell) {
                            return;
                        }
                    }
                    if !df.smt_queries.is_empty() {
                        let dir = route_group_dir(resolved, &gid);
                        // Verify only a complete pair; a lone landed role (a partial fill)
                        // still lands its smt_query, which the GroupTrace carries into the
                        // manifest so replay covers the group even without a verdict.
                        let results = df.pair.as_ref().and_then(|(overlap, deontic)| {
                            direct_smt_verify_group(
                                &gid, &dir, overlap, deontic, resolved, &adapter, shell,
                            )
                        });
                        // A verified group feeds the run report's group row.
                        if let Some(results) = &results {
                            groups.push(GroupObservation {
                                group_id: gid.clone(),
                                query_pairs: vec![(
                                    static_id(&format!("{gid}.overlap")),
                                    static_id(&format!("{gid}.deontic")),
                                )],
                                results: results.payload.results.clone(),
                            });
                        }
                        // The direct route compiles no IR, so the group trace carries no
                        // member bundles and no compiled artifact — the landed smt_query
                        // wrappers keep replay covering the group even when verification did
                        // not land, and the verifier results (when present) feed the run
                        // report's group row.
                        all_group_traces.push(GroupTrace {
                            group_id: gid.clone(),
                            test_sources: group.test_sources.clone(),
                            member_bundles: Vec::new(),
                            dir,
                            smt_queries: df.smt_queries,
                            compiled: None,
                            verifier_results: results,
                        });
                    }
                }
                // Each unique document's head chains into the run trace as a parallel
                // (bundle-less) branch beside the single_ir route's IR chain.
                for (_, head) in heads {
                    all_graphs.push(head.source);
                    all_docs.push(head.trace);
                }
            }
            RouteShape::M1Layered => {
                unreachable!("execute_routes runs only model-route views")
            }
        }

        let ledger = shell.ledger()[ledger_start..].to_vec();
        route_runs.push(RouteRun {
            pipeline_id: resolved.pipeline_id.clone(),
            ledger,
            fills,
            samples: vec![groups.clone()],
            groups,
        });
        model_routes.push(resolved.shape);
    }

    // The run-level tails run once over both routes. Every route builds the same
    // source graph per document (payload-identical, so equal content hashes), so keep
    // one per document for the report; the run trace instead keeps every route's
    // parallel chain (deduping only the shared source node).
    let mut seen_docs: BTreeSet<Id> = BTreeSet::new();
    all_graphs.retain(|g| seen_docs.insert(g.payload.document.document_id.clone()));
    // assemble_trace's lineage lookup takes the first doc by id; the direct route is
    // bundle-less and resolves first in the set, so surface the bundle-bearing
    // single_ir doc first while the stable sort keeps both routes' chain nodes.
    all_docs.sort_by_key(|d| d.bundle.is_none());

    // The route pipelines declare no trace/report step, so the run-level tails run as
    // the baseline route's steps with the census event suppressed (`emit_event`
    // false); a failed tail still fails the run closed by raising its diagnostic.
    let baseline_resolved = views
        .iter()
        .find(|v| v.is_baseline)
        .expect("the resolved set names a baseline pipeline");
    let Some((bundle, lineage)) = trace_processing_stage(
        &all_docs,
        &all_group_traces,
        baseline_resolved,
        shell,
        false,
    ) else {
        return;
    };
    report_processing_stage(
        root,
        &all_docs,
        &all_graphs,
        &all_group_traces,
        &bundle,
        &lineage,
        &lexicon_hash,
        adapter.identity(),
        baseline_resolved,
        shell,
        false,
        agreed,
        &model_routes,
        &route_runs,
    );
}

/// The §9 route family a pipeline's declared processing-stage sequence
/// fingerprints to ([`resolve_route`]): the eight-stage layered M1 pipeline,
/// the six-stage single_ir route, or the four-stage direct_smt route.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RouteShape {
    M1Layered,
    SingleIr,
    DirectSmt,
}

/// The runner's resolved view of one experiment pipeline: the pipeline
/// candidate, its pipeline step ids and route shape, the unique corpus
/// documents across the test_source groups in first-appearance order, the
/// groups themselves in evaluation order, the per-query solver resource
/// limit, the §5 plan the run executes, and the toolchain manifest hash every
/// producer carries. [`resolve`] returns one view per bound pipeline in set
/// order; documents, groups, budget, plan, and toolchain hash are shared
/// across the set, while the repair budget and baseline flag are per-view.
struct Resolved {
    pipeline_id: Id,
    /// Pipeline step ids: the pipeline's declared stages filling slots
    /// `0..n` in declared order, padded to eight with [`UNUSED_STAGE`]. The
    /// M1 shape fills all eight parallel to [`PROCESSING_STAGE_KINDS`].
    pipeline_step_ids: [Id; 8],
    /// The route family the pipeline fingerprinted to.
    shape: RouteShape,
    documents: Vec<CorpusEntry>,
    groups: Vec<TestSourceGroup>,
    /// §8.4 `solver_ms_per_query` budget value.
    budget_ms: u64,
    /// §9 `model_repair_limit` budget value: `Some` on the model routes
    /// (resolution fails without it) and `None` on the M1 layered shape,
    /// which spends no repairs. run-m2.1d3/.1d4 read it in the route loop.
    #[allow(dead_code)]
    repair_limit: Option<u32>,
    /// §9 `model_ms_per_call` budget value, read unconditionally from the
    /// experiment budget map (`None` when undeclared). Required only at
    /// record time — [`build_record_parts`] turns `None` into a loud
    /// command-scope failure; a replay run never invokes the runtime, so
    /// M1 and replay bindings run without the key.
    model_ms_per_call: Option<u64>,
    /// Whether this view's pipeline is the experiment's §7.3 delta baseline
    /// (the M1 legacy binding's single view is its own baseline).
    /// run-m2.1d5's tails read it.
    #[allow(dead_code)]
    is_baseline: bool,
    /// §5 run plan built from the experiment entry; its content hash is
    /// the manifest's `run_plan_hash`.
    plan: RunPlan,
    /// §4.4 raw-byte hash of [`TOOLCHAIN_FILE`].
    toolchain_manifest_hash: Hash,
}

/// Resolve `experiment_id` against the §8.4 registry surface into one
/// [`Resolved`] view per bound pipeline, in set order (the M1 legacy binding
/// resolves to its single pipeline). Whole-set semantic validation is `ckc
/// registry check`'s job; resolution diagnoses exactly what this run
/// consumes — every reference it follows plus member uniqueness (duplicate
/// members would mint colliding views) — each failure one command-scope
/// `schema_invalid` diagnostic.
/// Per-view step ids and [`RouteShape`] come from [`resolve_route`]'s
/// fingerprint of each pipeline's declared stages. The toolchain manifest
/// read rides last: every producer (and later both manifests) carries its
/// hash, so a run that cannot attest its toolchain mints nothing.
fn resolve(root: &Path, experiment_id: &Id, shell: &mut Shell) -> Option<Vec<Resolved>> {
    let corpora = load(root, CORPORA_FILE, parse_corpora, shell);
    let candidates = load(root, "registry/candidates.yaml", parse_candidates, shell);
    let experiments = load(root, EXPERIMENTS_FILE, parse_experiments, shell);
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
    // The shape-aware baseline accessor mirrors `registry check`'s binding
    // validation: a malformed binding (neither form, both, or a stray or
    // out-of-set baseline) resolves nothing.
    let Some(baseline_id) = experiment.baseline().cloned() else {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("reason"),
            format!("experiment {experiment_id} has no valid pipeline binding"),
        )]));
        return None;
    };
    let pipelines = experiment.resolved_pipelines();
    let mut routes: Vec<(Id, [Id; 8], RouteShape)> = Vec::with_capacity(pipelines.len());
    for member in &pipelines {
        if routes.iter().any(|(id, _, _)| id == member) {
            shell.diagnostic(invalid_diagnostic(vec![(
                static_id("reason"),
                format!("experiment {experiment_id} binds duplicate pipeline {member}"),
            )]));
            return None;
        }
        let Some(pipeline) = candidates.pipelines.iter().find(|p| p.id == *member) else {
            shell.diagnostic(invalid_diagnostic(vec![(
                static_id("reason"),
                format!("experiment {experiment_id} names undefined pipeline {member}"),
            )]));
            return None;
        };
        let (pipeline_step_ids, shape) = resolve_route(pipeline, &candidates, shell)?;
        routes.push((pipeline.id.clone(), pipeline_step_ids, shape));
    }

    let Some(&budget_ms) = experiment.budget.get(&static_id(SOLVER_BUDGET_KEY)) else {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("reason"),
            format!("experiment {experiment_id} declares no {SOLVER_BUDGET_KEY} budget"),
        )]));
        return None;
    };
    // §9 model-invocation cap: read unconditionally (any binding may declare
    // it); only `--record` requires it, at `build_record_parts`.
    let model_ms_per_call = experiment
        .budget
        .get(&static_id(MODEL_MS_PER_CALL_KEY))
        .copied();
    // §9 model-route budget: a binding that resolves a model route must cap
    // the repair loop explicitly, and a declared sample count other than 1
    // would promise draws single-draw replay never makes.
    let model_repair_limit = if routes.iter().any(|(_, _, s)| *s != RouteShape::M1Layered) {
        let Some(&raw) = experiment.budget.get(&static_id(MODEL_REPAIR_LIMIT_KEY)) else {
            shell.diagnostic(invalid_diagnostic(vec![(
                static_id("reason"),
                format!("experiment {experiment_id} declares no {MODEL_REPAIR_LIMIT_KEY} budget"),
            )]));
            return None;
        };
        let Ok(limit) = u32::try_from(raw) else {
            shell.diagnostic(invalid_diagnostic(vec![(
                static_id("reason"),
                format!(
                    "experiment {experiment_id} declares a {MODEL_REPAIR_LIMIT_KEY} budget \
                     beyond u32 ({raw})"
                ),
            )]));
            return None;
        };
        if let Some(&samples) = experiment.budget.get(&static_id(MODEL_SAMPLE_COUNT_KEY))
            && samples != 1
        {
            shell.diagnostic(invalid_diagnostic(vec![(
                static_id("reason"),
                format!(
                    "experiment {experiment_id} declares {MODEL_SAMPLE_COUNT_KEY} {samples}; \
                     replay draws exactly one sample"
                ),
            )]));
            return None;
        }
        Some(limit)
    } else {
        None
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
    // ONE §5 plan shared by every view: the plan describes the run, not a
    // route (M1's single binding keeps its exact prior plan bytes).
    let plan = RunPlan {
        experiment_id: experiment.id.clone(),
        test_source_groups: experiment
            .test_source_groups
            .iter()
            .map(|g| g.group_id.clone())
            .collect(),
        pipelines,
        seed: experiment.seed,
        budget: experiment
            .budget
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect(),
    };
    Some(
        routes
            .into_iter()
            .map(|(pipeline_id, pipeline_step_ids, shape)| Resolved {
                is_baseline: pipeline_id == baseline_id,
                pipeline_id,
                pipeline_step_ids,
                shape,
                documents: documents.clone(),
                groups: experiment.test_source_groups.clone(),
                budget_ms,
                repair_limit: match shape {
                    RouteShape::M1Layered => None,
                    RouteShape::SingleIr | RouteShape::DirectSmt => model_repair_limit,
                },
                model_ms_per_call,
                plan: plan.clone(),
                toolchain_manifest_hash: toolchain_manifest_hash.clone(),
            })
            .collect(),
    )
}

/// Fingerprint one pipeline's declared processing stages against the route
/// shapes this runner drives. Each declared stage id must name a registry
/// entry; the declared kind sequence — plus the model_fill stage's output
/// artifact kinds, which split the two §9 routes — must match exactly one of
/// [`PROCESSING_STAGE_KINDS`] (M1 layered), [`SINGLE_IR_STAGE_KINDS`] with a
/// `clinical_ir` fill, or [`DIRECT_SMT_STAGE_KINDS`] with an `smt_query`
/// fill. Declared order fills slots `0..n` of the fixed `[Id; 8]`; the
/// remainder pads with [`UNUSED_STAGE`]. Any failure is one command-scope
/// diagnostic + `None`.
fn resolve_route(
    pipeline: &PipelineEntry,
    candidates: &Candidates,
    shell: &mut Shell,
) -> Option<([Id; 8], RouteShape)> {
    let mut kinds: Vec<&str> = Vec::with_capacity(pipeline.processing_stages.len());
    let mut model_fill_outputs: &[Id] = &[];
    for stage_id in &pipeline.processing_stages {
        let Some(stage) = candidates
            .processing_stages
            .iter()
            .find(|s| s.id == *stage_id)
        else {
            shell.diagnostic(invalid_diagnostic(vec![(
                static_id("reason"),
                format!(
                    "pipeline {} declares undefined processing_stage {stage_id}",
                    pipeline.id
                ),
            )]));
            return None;
        };
        kinds.push(stage.kind.as_str());
        if stage.kind == static_id("model_fill") {
            model_fill_outputs = &stage.output_artifact_kinds;
        }
    }
    let shape = if kinds == PROCESSING_STAGE_KINDS {
        RouteShape::M1Layered
    } else if kinds == SINGLE_IR_STAGE_KINDS && *model_fill_outputs == [static_id("clinical_ir")] {
        RouteShape::SingleIr
    } else if kinds == DIRECT_SMT_STAGE_KINDS && *model_fill_outputs == [static_id("smt_query")] {
        RouteShape::DirectSmt
    } else {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("reason"),
            format!(
                "pipeline {} declares an unsupported processing-stage sequence [{}]",
                pipeline.id,
                kinds.join(", ")
            ),
        )]));
        return None;
    };
    let mut pipeline_step_ids: Vec<Id> = pipeline.processing_stages.clone();
    pipeline_step_ids.resize(PROCESSING_STAGE_KINDS.len(), static_id(UNUSED_STAGE));
    let pipeline_step_ids: [Id; 8] = pipeline_step_ids
        .try_into()
        .expect("a fingerprinted sequence declares at most eight stages and pads to exactly eight");
    Some((pipeline_step_ids, shape))
}

/// Route namespace for run-minted wrapper artifact ids: empty on the M1
/// layered shape (its artifact ids keep their exact M1 bytes) and
/// `"{pipeline_id}."` on the model routes, so two routes landing the same
/// document or group never mint colliding wrapper ids. Payload-level ids —
/// compile query ids, the `{gid}.overlap`/`{gid}.deontic` cassette sources —
/// stay unprefixed inside their route-prefixed wrappers. Finding ids stay
/// unprefixed structurally: trace assembly mints `finding.{gid}.{seq}` only
/// for groups carrying compiled + verifier_results, and the direct route
/// lands no compiled artifact (SPEC §9: the model emits SMT-LIB directly —
/// no assertion-to-source provenance), so exactly one view (single_ir)
/// feeds the §7.1 findings body and duplicate ids never reach
/// `Report::validate` (`is_baseline` still marks the baseline for .1d5's
/// tail wrapper producer). The single_ir consumers hold it now —
/// [`route_document_head`]'s two head wrappers, [`single_ir_fill`]'s bundle,
/// and [`compile_verify_group`]'s two group wrappers; run-m2.1d4a applies it
/// to the direct route's minted wrappers.
fn route_id_prefix(resolved: &Resolved) -> String {
    match resolved.shape {
        RouteShape::M1Layered => String::new(),
        RouteShape::SingleIr | RouteShape::DirectSmt => format!("{}.", resolved.pipeline_id),
    }
}

/// The route-namespaced group landing dir `routes/<pipeline_id>/groups/<gid>`,
/// mirroring the route head namespacing `routes/<pipeline_id>/artifacts/<doc>`.
/// Both model routes run through one shared run out ([`execute_routes`]), so each
/// route's group artifacts nest under their route and never collide; the M1
/// layered path keeps its bare `groups/<gid>` ([`group_pipeline`]).
fn route_group_dir(resolved: &Resolved, gid: &Id) -> String {
    format!("routes/{}/groups/{}", resolved.pipeline_id, gid)
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
    let dir = format!("artifacts/{}", entry.id);
    let mut trace = DocTrace {
        document_id: entry.id.clone(),
        test_source_path: entry.path.clone(),
        source_hash: hash_bytes(&html),
        dir: dir.clone(),
        source_document_graph: None,
        segments: None,
        normalization: None,
        bundle: None,
    };
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
/// Returns the group's [`GroupTrace`]: the §8.4 member set, the member
/// bundle ids compile consumed, plus each group landing that happened,
/// riding whole.
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
        member_bundles: Vec::new(),
        dir: dir.clone(),
        smt_queries: Vec::new(),
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
    trace.member_bundles = members.iter().map(|m| m.artifact_id.clone()).collect();
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
/// processing_stage's failure; a compile failure skips verify. Both wrapper
/// ids carry the caller's [`route_id_prefix`] — empty on the M1 layered
/// shape, so M1 artifact ids and byte pins hold unchanged.
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
    let prefix = route_id_prefix(resolved);
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
                format!("{prefix}{gid}.compiled"),
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
                format!("{prefix}{gid}.verifier_results"),
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

/// One document's deterministic route head: the [`DocTrace`] with the
/// extract + segment landings recorded under the route's artifact dir,
/// beside the two landed wrappers the fill stage consumes — the source
/// graph and the segments, whose payloads carry the grounding id universes.
/// Built by [`route_document_head`]; consumed by [`single_ir_fill`]
/// (run-m2.1d4a feeds the direct route's per-group fill from the same
/// heads).
#[allow(dead_code)]
struct DocHead {
    trace: DocTrace,
    source: ArtifactWrapper<SourceDocumentGraph>,
    segments: ArtifactWrapper<SegmentIr>,
}

/// One document's completed route passage: the [`DocTrace`] holding every
/// landing, the source graph riding whole (the report processing_stage's
/// quoted-span source), and the fill stage's §7.3 telemetry — `fill` is
/// `None` only on a cassette IO/contract failure (no completed fill, per
/// [`FillObservation`]'s contract), `identity` the last attempt's cassette
/// [`ModelIdentity`] (run-m2.1d5a checks cross-route identity agreement
/// against it).
#[allow(dead_code)]
struct RouteDoc {
    trace: DocTrace,
    graph: ArtifactWrapper<SourceDocumentGraph>,
    fill: Option<FillObservation>,
    identity: Option<ModelIdentity>,
}

/// Re-mint a wrapper's artifact id under the route's [`route_id_prefix`].
/// `content_hash` is payload-only, so re-minting never disturbs byte pins;
/// the M1 shape's empty prefix re-mints the id to itself.
#[allow(dead_code)]
fn route_minted<P>(mut wrapper: ArtifactWrapper<P>, prefix: &str) -> ArtifactWrapper<P> {
    wrapper.artifact_id = Id::new(format!("{prefix}{}", wrapper.artifact_id))
        .expect("a grammatical artifact id stays grammatical under a pipeline-id prefix");
    wrapper
}

/// [`document_pipeline`]'s read + extract + segment half for a model route:
/// land the deterministic head under `routes/{pipeline_id}/artifacts/{doc}`
/// with route-minted wrapper ids, one §4.6 event per attempted stage through
/// the index-coupled helpers (the declared slot-0/1 kinds equal M1's). An
/// unreadable corpus file is the same command-scope diagnostic + `None`; a
/// failed stage records its event and yields `None` (the fill stage needs
/// both wrappers). run-m2.1d5a drives it once per unique document.
#[allow(dead_code)]
fn route_document_head(
    root: &Path,
    entry: &CorpusEntry,
    resolved: &Resolved,
    shell: &mut Shell,
) -> Option<DocHead> {
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
    let prefix = route_id_prefix(resolved);
    let dir = format!("routes/{}/artifacts/{}", resolved.pipeline_id, entry.id);
    let mut trace = DocTrace {
        document_id: entry.id.clone(),
        test_source_path: entry.path.clone(),
        source_hash: hash_bytes(&html),
        dir: dir.clone(),
        source_document_graph: None,
        segments: None,
        normalization: None,
        bundle: None,
    };

    let rel = format!("{dir}/source_document_graph.json");
    let clock = processing_stage_clock();
    let config = ExtractConfig {
        document_id: entry.id.clone(),
        source_family: static_id("synthetic_test_source_html"),
        provenance: entry.provenance,
        data_class: DataClass::None,
        producer: producer(resolved, 0),
    };
    let built = extract(&html, &config)
        .map(|w| route_minted(w, &prefix))
        .map_err(|e| processing_stage_diagnostic(0, "document", &entry.id, e.to_string()));
    let source = close_processing_stage(shell, resolved, 0, clock, Vec::new(), &rel, built)?;
    trace.source_document_graph = Some((source.artifact_id.clone(), source.content_hash.clone()));

    let rel = format!("{dir}/segments.json");
    let inputs = vec![source.content_hash.clone()];
    let clock = processing_stage_clock();
    let built = segment(&source, &producer(resolved, 1))
        .map(|w| route_minted(w, &prefix))
        .map_err(|e| processing_stage_diagnostic(1, "document", &entry.id, e.to_string()));
    let segments = close_processing_stage(shell, resolved, 1, clock, inputs, &rel, built)?;
    trace.segments = Some((segments.artifact_id.clone(), segments.content_hash.clone()));

    Some(DocHead {
        trace,
        source,
        segments,
    })
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

/// The single_ir route's per-document fill stage over a landed [`DocHead`]:
/// replay the document's committed model cassette through [`model_fill`]
/// under [`single_ir_accept`], emit the fill's §4.6 event directly (the M1
/// kind table holds `normalize` at [`MODEL_FILL`]'s slot, so the
/// index-coupled helpers cannot serve it; the §7.4 fill diagnostics ride
/// the event only, which ledgers them), and compile an accepted
/// [`ClinicalIr`] over the head into an [`IrBundle`] — the same five-layer
/// assembly [`assemble_bundle`] produces, but with the model's clinical
/// layer and a norm [`derive_norm_ir`](crate::rules::derive_norm_ir)-
/// recomputed over it in place of the deterministic normalizer's — landed
/// under slot 3's fail-closure. The head is the grounding scaffold: its
/// wrappers carry the real region and segment ids the accept closure
/// grounds the model's references against, so a hallucinated reference
/// surfaces as `ai_hallucinated_source` rather than corrupting the bundle.
/// A cassette IO/contract failure is a command-scope diagnostic with no
/// event (infrastructure, not a stage outcome). Returns the document's
/// [`RouteDoc`]; run-m2.1d5a drives it per document.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn single_ir_fill(
    head: DocHead,
    lexicon: &Lexicon,
    store: &CassetteStore,
    seed: u64,
    resolved: &Resolved,
    repair_limit: u32,
    record: Option<&RouteRecord>,
    shell: &mut Shell,
) -> RouteDoc {
    let DocHead {
        mut trace,
        source,
        segments,
    } = head;
    let doc_id = trace.document_id.clone();
    let prefix = route_id_prefix(resolved);

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
        source: doc_id.clone(),
        seed,
    };
    // The fill event's inputs, cloned ahead of the clock (the M2.7 boundary
    // discipline: pure setup stays outside the timed interval).
    let fill_inputs = vec![source.content_hash.clone(), segments.content_hash.clone()];
    // Compose the record prompt outside the timed interval (the M2.7 clock
    // boundary): pure setup stays out of the fill's measured span. A replay run
    // composes nothing and sends `FillSource::Replay`.
    let record_prompt = record.map(|r| {
        single_ir_prompt(
            &r.template,
            &doc_id,
            &source.payload,
            &segments.payload.segments,
            lexicon,
        )
    });
    let clock = processing_stage_clock();
    let source_fill = match record.zip(record_prompt.as_deref()) {
        Some((r, prompt)) => FillSource::Record {
            adapter: r.adapter,
            prompt,
            constraint: &r.constraint,
            ctx: &r.ctx,
        },
        None => FillSource::Replay,
    };
    let fill = match model_fill(store, &key, source_fill, repair_limit, accept) {
        Ok(fill) => fill,
        Err(e) => {
            shell.diagnostic(invalid_diagnostic(vec![
                (static_id("cassette"), doc_id.to_string()),
                (static_id("reason"), e.to_string()),
                (static_id("processing_stage"), "model_fill".to_owned()),
            ]));
            return RouteDoc {
                trace,
                graph: source,
                fill: None,
                identity: None,
            };
        }
    };
    let (started_at, ended_at, duration_ms) = clock.stop();
    let observation = FillObservation::from_fill(&fill);
    let ModelFill {
        target,
        accepted_cassette_hash,
        model_identity,
        diagnostics,
        recorded_calls,
        repairs,
    } = fill;
    // The fill's §4.6 event, emitted directly (see the doc comment): the
    // accepted attempt's cassette wrapper hash is the stage output (empty iff
    // no attempt was accepted), the two §7.3 counters ride resource_counters,
    // and the §7.4 fill diagnostics ride the event only —
    // processing_stage_event extends the ledger with them. `outcome` is
    // written above `diagnostics`: struct-literal fields evaluate in written
    // order, so the borrow ends before the move.
    shell.processing_stage_event(ProcessingStageEvent {
        pipeline_id: resolved.pipeline_id.clone(),
        pipeline_step_id: resolved.pipeline_step_ids[MODEL_FILL].clone(),
        processing_stage: static_id(SINGLE_IR_STAGE_KINDS[MODEL_FILL]),
        started_at,
        ended_at,
        duration_ms,
        input_hashes: fill_inputs,
        output_hashes: accepted_cassette_hash.iter().cloned().collect(),
        outcome: severity(&diagnostics),
        diagnostics,
        resource_counters: vec![
            (static_id(RECORDED_CALLS_COUNTER), recorded_calls),
            (static_id(REPAIRS_COUNTER), repairs),
        ],
    });
    let Some(clinical) = target else {
        // A terminal reject (no accepted target) ends the route: the trace
        // keeps its head landings, the bundle slot stays empty.
        return RouteDoc {
            trace,
            graph: source,
            fill: Some(observation),
            identity: model_identity,
        };
    };
    // §9 attestation: an accepted fill always carries the accepted attempt's
    // cassette wrapper hash (`Some` iff `target` is — model_fill's contract);
    // the bundle wrapper cites it below.
    let cassette_hash =
        accepted_cassette_hash.expect("accepted fill carries its cassette wrapper hash");

    // Deterministic tail mirroring [`assemble_bundle`], substituting the model's
    // clinical layer and a norm recomputed over it; the model-fill route runs no
    // normalizer, so the bundle diagnostics are the segments' alone — equal to M1's
    // segments ∪ normalization set because the normalizer adds none for grounded
    // output (route-single-ir.2b's reproduce-M1 gate proves the equality). Landed
    // under slot 3's fail-closure: the declared kind equals M1's `assemble`, so
    // the index-coupled close serves, and any tail failure records its event.
    let inputs = vec![
        source.content_hash.clone(),
        segments.content_hash.clone(),
        cassette_hash,
    ];
    let rel = format!("{}/ir_bundle.json", trace.dir);
    let fail = |reason: String| processing_stage_diagnostic(3, "document", &doc_id, reason);
    let clock = processing_stage_clock();
    let norm = crate::rules::derive_norm_ir(
        &source.payload.document.document_id,
        &clinical,
        &segments.payload,
        lexicon,
    );
    let built = ckc_core::DocIr::from_graph(&source.payload, source.diagnostics.clone())
        .map_err(|e| fail(format!("doc layer: {e}")))
        .and_then(|doc| {
            let diagnostics = canonical_diagnostic_set(segments.diagnostics.iter())
                .map_err(|e| fail(format!("diagnostic sort key: {e}")))?;
            let bundle = assemble(
                doc,
                segments.payload.clone(),
                clinical,
                norm,
                Vec::new(),
                diagnostics,
            )
            .map_err(|e| fail(format!("assembly: {e}")))?;
            bundle
                .validate(&source.payload)
                .map_err(|e| fail(format!("bundle invariant: {e}")))?;
            // The single_ir route has no normalization wrapper, so the bundle
            // cites source + segments + the accepted attempt's cassette (§9
            // attestation: the artifact graph names the exact recording the
            // clinical layer replayed from). Wrapper-level fields stay off the
            // payload-only `content_hash`.
            wrapper(
                format!("{prefix}{doc_id}.ir_bundle"),
                "ir_bundle",
                producer(resolved, 3),
                inputs.clone(),
                Origin::DeterministicCompiler,
                EvidenceStatus::MechanicalEvidenceStatus,
                Vec::new(),
                bundle,
            )
            .map_err(|e| fail(format!("content hash: {e}")))
        });
    trace.bundle = close_processing_stage(shell, resolved, 3, clock, inputs, &rel, built);
    RouteDoc {
        trace,
        graph: source,
        fill: Some(observation),
        identity: model_identity,
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

/// The pair fill's outcome for one direct_smt group: `pair` is `Some` only when both
/// roles accepted and landed, while `fills` and `identities` survive a terminal role
/// reject so the run-m2.1d5a orchestrator can fold their §7.3 telemetry and check
/// cross-route model-identity agreement. `smt_queries` carries every landed role — a
/// prefix of `[overlap, deontic]` (`[]`, `[overlap]`, or the full pair, since a role
/// reject breaks the loop before the next) — so a lone landed role that cannot verify
/// still reaches the run manifest's `output_hashes` and stays replay-covered (run-m2.1e-A);
/// `pair` is that prefix's two-element case, cloned as a type-safe verify input.
#[allow(dead_code)]
struct DirectFill {
    pair: Option<(ArtifactWrapper<QueryBody>, ArtifactWrapper<QueryBody>)>,
    smt_queries: Vec<ArtifactWrapper<QueryBody>>,
    fills: Vec<FillObservation>,
    identities: Vec<ModelIdentity>,
}

/// The direct_smt route's per-group fill back end over the group's two member
/// [`DocHead`]s: the direct route grounds nothing (it emits raw SMT, not an IR), so the
/// heads carry only member provenance forward. Replays each role's committed cassette
/// through [`model_fill`] under [`direct_smt_accept`] — the overlap query keyed under
/// `<gid>.overlap`, the deontic query under `<gid>.deontic`, both at the base seed. The
/// sources are role-namespaced so a Q2 repair never aliases Q1's: [`model_fill`] reads
/// attempt `i` under `derive_seed(base, i)` on the one source, so a shared source would
/// collide. Each accepted body is wrapped as an `smt_query` [`ArtifactWrapper`] carrying
/// the raw model output ([`Origin::AiGenerated`] + [`EvidenceStatus::AcceptedEvidenceStatus`],
/// no external effects and no deterministic transform — distinct from single_ir's
/// mechanical `ir_bundle`), citing the member provenance plus its own accepted cassette
/// hash, then lands it under `routes/<pipeline_id>/groups/<gid>`. Direct-emits one group
/// model_fill §4.6 event; a terminal role reject or a wrap/land failure breaks the loop but
/// still rides that event.
/// An `Err(CassetteError)` cassette IO/contract failure records a command diagnostic and
/// aborts event-less while nothing has landed (pure infra); once a role has landed it rides
/// the one event (like a wrap/land failure) so the landed artifact stays attested and its
/// counters are not dropped. The returned [`DirectFill`] carries the pair (`Some` only when
/// both roles landed) beside the `fills`/`identities` that survive a reject; wired into the
/// experiment run by run-m2.1.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn direct_smt_fill(
    gid: &Id,
    heads: &[&DocHead],
    store: &CassetteStore,
    seed: u64,
    resolved: &Resolved,
    repair_limit: u32,
    record: Option<&RouteRecord>,
    shell: &mut Shell,
) -> DirectFill {
    // The cassette-role design mints exactly one (overlap, deontic) pair per group, so a
    // non-pair head set fails closed with a command diagnostic and no event.
    if heads.len() != 2 {
        shell.diagnostic(invalid_diagnostic(vec![
            (static_id("group"), gid.to_string()),
            (
                static_id("reason"),
                format!("expected 2 member heads, got {}", heads.len()),
            ),
            (static_id("processing_stage"), "model_fill_smt".to_owned()),
        ]));
        return DirectFill {
            pair: None,
            smt_queries: Vec::new(),
            fills: Vec::new(),
            identities: Vec::new(),
        };
    }
    let prefix = route_id_prefix(resolved);
    // Route-namespaced ([`route_group_dir`]) so both routes' groups never collide
    // under the one shared run out when [`execute_routes`] runs them.
    let dir = route_group_dir(resolved, gid);
    // Member-order provenance, gathered before the clock so only the fill work falls
    // inside the timed interval (the M2.7 clock-boundary discipline).
    let mut input_hashes: Vec<Hash> = Vec::new();
    for head in heads {
        input_hashes.push(head.source.content_hash.clone());
        input_hashes.push(head.segments.content_hash.clone());
    }
    // Member (doc-id, source graph) pairs for the record prompt, gathered once
    // (both roles share them) and only when recording; a replay run gathers none.
    let record_members: Option<Vec<(&Id, &SourceDocumentGraph)>> = record.map(|_| {
        heads
            .iter()
            .map(|h| (&h.trace.document_id, &h.source.payload))
            .collect()
    });
    let clock = processing_stage_clock();
    let mut fills: Vec<FillObservation> = Vec::new();
    let mut identities: Vec<ModelIdentity> = Vec::new();
    let mut diagnostics: Vec<DiagnosticRecord> = Vec::new();
    let mut recorded_calls: u64 = 0;
    let mut repairs: u64 = 0;
    let mut landed: Vec<ArtifactWrapper<QueryBody>> = Vec::new();
    // Replay the pair's two role-namespaced cassettes at the base seed, wrapping and
    // landing each shallow-accepted body as a raw-AI `smt_query` the verdict tail consumes.
    for (role, logic) in [("overlap", SmtLogic::QfLra), ("deontic", SmtLogic::QfUf)] {
        let source = static_id(&format!("{gid}.{role}"));
        let key = CassetteKey {
            route: static_id("route.direct_smt"),
            source: source.clone(),
            seed,
        };
        // Per-role record prompt (both roles share the members); a replay run
        // composes none and sends `FillSource::Replay`.
        let record_prompt = record
            .zip(record_members.as_deref())
            .map(|(r, members)| direct_smt_prompt(&r.template, gid, role, members));
        let role_fill = match record.zip(record_prompt.as_deref()) {
            Some((r, prompt)) => FillSource::Record {
                adapter: r.adapter,
                prompt,
                constraint: &r.constraint,
                ctx: &r.ctx,
            },
            None => FillSource::Replay,
        };
        let fill = match model_fill(store, &key, role_fill, repair_limit, direct_smt_accept()) {
            Ok(fill) => fill,
            Err(e) => {
                // A cassette IO/contract failure is infrastructure, not a stage outcome.
                // While nothing has landed it aborts event-less (a command diagnostic on the
                // ledger); once a role has landed it rides the one event like a wrap/land
                // failure, so the landed artifact stays attested and its counters ride out.
                let cassette_diag = invalid_diagnostic(vec![
                    (static_id("cassette"), source.to_string()),
                    (static_id("reason"), e.to_string()),
                    (static_id("processing_stage"), "model_fill_smt".to_owned()),
                ]);
                if landed.is_empty() {
                    shell.diagnostic(cassette_diag);
                    return DirectFill {
                        pair: None,
                        smt_queries: Vec::new(),
                        fills,
                        identities,
                    };
                }
                diagnostics.push(cassette_diag);
                break;
            }
        };
        fills.push(FillObservation::from_fill(&fill));
        let ModelFill {
            target,
            accepted_cassette_hash,
            model_identity,
            diagnostics: role_diagnostics,
            recorded_calls: role_recorded_calls,
            repairs: role_repairs,
        } = fill;
        identities.extend(model_identity);
        diagnostics.extend(role_diagnostics);
        recorded_calls += role_recorded_calls;
        repairs += role_repairs;
        let Some(body) = target else {
            // A terminal reject stops the pair: the overlap query exhausts before the
            // deontic source is read, so its diagnostics ride the one event below.
            break;
        };
        // §9 attestation: cite this role's accepted cassette alongside the shared member
        // provenance (`Some` iff `target` is — model_fill's contract).
        let mut role_inputs = input_hashes.clone();
        role_inputs
            .push(accepted_cassette_hash.expect("accepted fill carries its cassette wrapper hash"));
        let payload = QueryBody {
            query_id: source,
            logic,
            body,
        };
        match wrapper(
            format!("{prefix}{gid}.{role}.smt_query"),
            "smt_query",
            producer(resolved, MODEL_FILL),
            role_inputs,
            Origin::AiGenerated,
            EvidenceStatus::AcceptedEvidenceStatus,
            Vec::new(),
            payload,
        ) {
            Ok(env) => match land(shell, &format!("{dir}/{role}.smt_query.json"), env) {
                Ok(w) => landed.push(w),
                Err(d) => {
                    // A landing failure is fail-closed: record it on the event and stop, so
                    // the pair never reaches two and this group yields no pair.
                    diagnostics.push(d);
                    break;
                }
            },
            Err(e) => {
                diagnostics.push(invalid_diagnostic(vec![
                    (static_id("group"), gid.to_string()),
                    (
                        static_id("artifact"),
                        format!("{prefix}{gid}.{role}.smt_query"),
                    ),
                    (static_id("reason"), format!("wrap: {e}")),
                    (static_id("processing_stage"), "model_fill_smt".to_owned()),
                ]));
                break;
            }
        }
    }
    let (started_at, ended_at, duration_ms) = clock.stop();
    let output_hashes: Vec<_> = landed.iter().map(|w| w.content_hash.clone()).collect();
    // One directly-emitted model_fill §4.6 event covers the group: the pair's member
    // provenance as inputs, the landed smt_query bodies as outputs, and the summed
    // recorded-call / repair counters. `outcome` reads `diagnostics` before it is moved.
    shell.processing_stage_event(ProcessingStageEvent {
        pipeline_id: resolved.pipeline_id.clone(),
        pipeline_step_id: resolved.pipeline_step_ids[MODEL_FILL].clone(),
        processing_stage: static_id(DIRECT_SMT_STAGE_KINDS[MODEL_FILL]),
        started_at,
        ended_at,
        duration_ms,
        input_hashes,
        output_hashes,
        outcome: severity(&diagnostics),
        diagnostics,
        resource_counters: vec![
            (static_id(RECORDED_CALLS_COUNTER), recorded_calls),
            (static_id(REPAIRS_COUNTER), repairs),
        ],
    });
    // `pair` is the two-element case of `landed` (a prefix of [overlap, deontic]); clone
    // it as the type-safe verify input while `smt_queries` keeps every landed role for
    // manifest replay coverage, so a lone landed role survives though it cannot verify.
    let pair = match landed.as_slice() {
        [overlap, deontic] => Some((overlap.clone(), deontic.clone())),
        _ => None,
    };
    DirectFill {
        pair,
        smt_queries: landed,
        fills,
        identities,
    }
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
    let prefix = route_id_prefix(resolved);
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
                format!("{prefix}{gid}.verifier_results"),
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
    emit_event: bool,
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
                tail_producer(resolved, TRACE, emit_event),
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
                tail_producer(resolved, TRACE, emit_event),
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
        Err(diagnostic) => {
            // A route-pipeline tail (`emit_event` false) declares no census event, so
            // the event's `diagnostics` field never reaches the shell; raise the
            // diagnostic directly to keep a failed tail fail-closed.
            if !emit_event {
                shell.diagnostic(diagnostic.clone());
            }
            (diagnostic.outcome, vec![diagnostic], Vec::new(), None)
        }
    };
    if emit_event {
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
    }
    pair
}

/// (run-m2.1e-C1) Per-route §7.3 metrics for the run-level report's model
/// sections, or `None` on an M1 baseline run (empty `model_routes`). Mirrors
/// [`manifest_inputs`]' experiment lookup — resolve the run's experiment, parse its
/// `expected_outcomes` reference, and score each route's collected fills, groups, and
/// k-sample battery against it via [`route_metrics`].
fn model_route_metrics(
    root: &Path,
    resolved: &Resolved,
    route_runs: &[RouteRun],
    model_routes: &[RouteShape],
) -> Result<Option<Vec<RouteMetrics>>, String> {
    if model_routes.is_empty() {
        return Ok(None);
    }
    let experiments = parse_experiments(
        &std::fs::read_to_string(root.join(EXPERIMENTS_FILE)).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let exp = experiments
        .into_iter()
        .find(|e| e.id == resolved.plan.experiment_id)
        .ok_or_else(|| {
            format!(
                "experiment {} absent from registry",
                resolved.plan.experiment_id
            )
        })?;
    let reference = parse_reference(
        &std::fs::read_to_string(root.join(&exp.expected_outcomes)).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(Some(
        route_runs
            .iter()
            .map(|rr| {
                route_metrics(
                    &rr.pipeline_id,
                    &rr.fills,
                    &rr.groups,
                    &rr.samples,
                    &reference,
                )
            })
            .collect(),
    ))
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
    emit_event: bool,
    agreed: Option<ModelIdentity>,
    model_routes: &[RouteShape],
    route_runs: &[RouteRun],
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

    // The run-level report's model sections (§7.4 taxonomy + §7.3 metrics) exist only
    // on a model route; an M1 baseline run passes `None` and stays byte-identical.
    // `route_diagnostics` lends each route's ledger slice to the per-route taxonomy.
    let route_diagnostics: Vec<(Id, &[DiagnosticRecord])> = route_runs
        .iter()
        .map(|rr| (rr.pipeline_id.clone(), rr.ledger.as_slice()))
        .collect();

    // Report model sections need an attested identity — `ModelRunSections` carries a
    // non-optional one — so score the per-route §7.3 metrics only when the routes
    // agreed on an identity. A model route that attested none yields a section-less
    // report, whereas the §9 manifest still records that run's setup hashes with a
    // null identity (it gates only on `model_routes` non-empty): the two views differ
    // on a degraded route by design, a §7.x view declining to attribute results to an
    // unknown evaluator. Gating on `agreed` here also skips a reference parse the
    // `None` arm would discard — whose failure would otherwise sink a run that
    // `assemble_report(None)` completes.
    let route_metrics = match agreed.as_ref() {
        Some(_) => model_route_metrics(root, resolved, route_runs, model_routes),
        None => Ok(None),
    };

    let landed = route_metrics
        .map_err(|reason| fail(format!("report model sections: {reason}")))
        .and_then(|route_metrics| {
            // Sections are all-or-nothing: built only when the metrics and the agreed
            // identity are both present (the gate above pairs them).
            let model_run = match (route_metrics, agreed.as_ref()) {
                (Some(metrics), Some(model_identity)) => Some(ModelRunSections {
                    route_diagnostics: &route_diagnostics,
                    route_metrics: metrics,
                    baseline_pipeline_id: &resolved.pipeline_id,
                    model_identity,
                }),
                _ => None,
            };
            assemble_report(
                &bundle.payload,
                &lineage.payload,
                &graphs.iter().map(|g| &g.payload).collect::<Vec<_>>(),
                &results.iter().map(|r| &r.payload).collect::<Vec<_>>(),
                lexicon_hash,
                solver_identity,
                &ledger,
                model_run,
            )
            .map_err(|e| fail(format!("report assembly: {e}")))
        })
        .and_then(|report| {
            report
                .validate()
                .map_err(|e| fail(format!("report invariant: {e}")))?;
            wrapper(
                "report".to_owned(),
                "report",
                tail_producer(resolved, REPORT, emit_event),
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
            // §7.2 derived view: the JA rendering of the same read-back payload,
            // landed beside report_en.md in the shared stage (both routes).
            let body = render_markdown_ja(&report.payload);
            let path = shell
                .write_under("report_ja.md", body.as_bytes())
                .map_err(|e| fail(format!("report_ja.md: {e}")))?;
            let read_back =
                std::fs::read(&path).map_err(|e| fail(format!("report_ja.md: read back: {e}")))?;
            if read_back != body.as_bytes() {
                return Err(fail(
                    "report_ja.md: read back diverges from the rendering".to_owned(),
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
                agreed.as_ref(),
                model_routes,
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
        Err(diagnostic) => {
            // A route-pipeline tail (`emit_event` false) declares no census event, so
            // the event's `diagnostics` field never reaches the shell; raise the
            // diagnostic directly to keep a failed tail fail-closed.
            if !emit_event {
                shell.diagnostic(diagnostic.clone());
            }
            (diagnostic.outcome, vec![diagnostic], Vec::new())
        }
    };
    if emit_event {
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
}

/// Combine a set of hash strings into one §4.4 raw-byte hash — sort and
/// dedup first (so route order and per-route duplicates never move the
/// result), join under newlines, then hash the bytes.
fn aggregate_hashes(mut v: Vec<String>) -> Hash {
    v.sort();
    v.dedup();
    hash_bytes(v.join("\n").as_bytes())
}

/// Aggregate the §9 route hash over exactly the registry entries a run's
/// routes consume: gather the wanted entries' hashes, then require every
/// wanted id to have resolved. A drifted registry — a route id matching no
/// entry — fails the run rather than silently locking [`aggregate_hashes`]'s
/// empty-set hash into a measurement record whose whole purpose is
/// attestation. `want` is non-empty by construction (a model route drove it),
/// so a full miss surfaces as a missing id, never a silent empty lock.
fn select_route_hashes<'a>(
    want: &BTreeSet<Id>,
    entries: impl Iterator<Item = (&'a Id, &'a Hash)>,
    kind: &str,
) -> Result<Hash, String> {
    let mut found: BTreeSet<Id> = BTreeSet::new();
    let mut hashes: Vec<String> = Vec::new();
    for (id, hash) in entries {
        if want.contains(id) {
            found.insert(id.clone());
            hashes.push(hash.as_str().to_owned());
        }
    }
    if &found != want {
        let missing: Vec<&str> = want.difference(&found).map(Id::as_str).collect();
        return Err(format!(
            "manifests: {kind} registry missing route id(s): {}",
            missing.join(", ")
        ));
    }
    Ok(aggregate_hashes(hashes))
}

// ── record-mode prompt composition (run-m2.1f) ──────────────────────────────
// Pure record-mode helpers: pick the committed schema and prompt entry a model
// route records against (keyed by entry `id`, mirroring [`manifest_inputs`]'
// want-set), and compose the first-draft prompt a route sends the runtime from
// its template plus the source graph's guideline text. run-m2.1f2 threads these
// into `execute_routes`; run-m2.2 refines the composed wording against a live
// recording.

/// The `registry/schemas.yaml` entry a model route records against, keyed by
/// entry `id` like [`manifest_inputs`]' want-set: single_ir constrains against
/// `schema.clinical_ir`, direct_smt against `schema.smt_query`. The layered M1
/// shape records no model fill, so it selects nothing.
fn select_record_schema(schemas: &[SchemaEntry], shape: RouteShape) -> Option<&SchemaEntry> {
    let want = match shape {
        RouteShape::SingleIr => "schema.clinical_ir",
        RouteShape::DirectSmt => "schema.smt_query",
        RouteShape::M1Layered => return None,
    };
    schemas.iter().find(|s| s.id.as_str() == want)
}

/// The `registry/prompts.yaml` entry a model route records against, keyed by
/// entry `id` like [`manifest_inputs`]' want-set: single_ir sends
/// `prompt.single_ir`, direct_smt sends `prompt.direct_smt`. The layered M1
/// shape records no model fill, so it selects nothing.
fn select_record_prompt(prompts: &[PromptEntry], shape: RouteShape) -> Option<&PromptEntry> {
    let want = match shape {
        RouteShape::SingleIr => "prompt.single_ir",
        RouteShape::DirectSmt => "prompt.direct_smt",
        RouteShape::M1Layered => return None,
    };
    prompts.iter().find(|p| p.id.as_str() == want)
}

/// One source graph's guideline text: its span `raw_text`s in `reading_order`,
/// one line each. The §4.5 graph already stores spans in reading order, but the
/// composer sorts explicitly so a partial or reordered graph still yields a
/// deterministic prompt.
fn reading_order_text(graph: &SourceDocumentGraph) -> Vec<String> {
    let mut spans: Vec<_> = graph.spans.iter().collect();
    spans.sort_by_key(|s| s.reading_order);
    spans.into_iter().map(|s| s.raw_text.clone()).collect()
}

/// Compose the single_ir record prompt: the route `template`, a document-id
/// line, the grounding scaffold — one `segment:` line per upstream segment
/// (id, kind, covered region ids), a `regions:` line naming every evidence
/// region id (read from `graph.regions`, the same set the accept closure
/// grounds against), and the lexicon vocabulary block (`system:` /
/// `concept:` with `var=` interval-variable marks / `action:` lines) — then
/// the document's guideline text in `reading_order`, newline-joined. The
/// supplied lines are exactly the template's promised inputs
/// (`registry/prompts/single_ir.txt`); the enumerated fields the template
/// leaves to the schema (binding status, modality, strength, certainty)
/// stay unsupplied. Deterministic: every block rides committed-artifact
/// order (segments/regions as landed, lexicon in file order, spans sorted
/// by `reading_order`).
fn single_ir_prompt(
    template: &str,
    doc_id: &Id,
    graph: &SourceDocumentGraph,
    segments: &[ClinicalSegment],
    lexicon: &Lexicon,
) -> String {
    let joined = |ids: &[Id]| ids.iter().map(Id::to_string).collect::<Vec<_>>().join(",");
    let mut lines = vec![template.to_owned(), format!("document: {doc_id}")];
    for segment in segments {
        lines.push(format!(
            "segment: {} kind={} regions={}",
            segment.segment_id,
            segment.kind.as_str(),
            joined(&segment.region_ids)
        ));
    }
    let region_ids: Vec<Id> = graph.regions.iter().map(|r| r.region_id.clone()).collect();
    lines.push(format!("regions: {}", joined(&region_ids)));
    lines.push(format!("system: {}", lexicon.system));
    for concept in &lexicon.concepts {
        lines.push(match &concept.interval {
            Some(interval) => format!("concept: {} var={}", concept.concept_id, interval.var),
            None => format!("concept: {}", concept.concept_id),
        });
    }
    for action in &lexicon.actions {
        lines.push(format!("action: {}", action.action_id));
    }
    lines.extend(reading_order_text(graph));
    lines.join("\n")
}

/// Compose the direct_smt record prompt for one verifier `role`
/// (`overlap`/`deontic`) of group `gid`: the route `template`, group and role
/// lines, then each member's document-id line followed by its guideline text
/// in `reading_order`, newline-joined. The template supplies the rest
/// in-text: the role-sensitive `:named` label scheme derives every label
/// from a member's `document:` id plus a rule number, so the composer sends
/// no label list.
fn direct_smt_prompt(
    template: &str,
    gid: &Id,
    role: &str,
    members: &[(&Id, &SourceDocumentGraph)],
) -> String {
    let mut lines = vec![
        template.to_owned(),
        format!("group: {gid}"),
        format!("role: {role}"),
    ];
    for &(doc_id, graph) in members {
        lines.push(format!("document: {doc_id}"));
        lines.extend(reading_order_text(graph));
    }
    lines.join("\n")
}

/// The shared record-mode inputs assembled once per `ckc run --record` model
/// run: the live runtime adapter and the validated committed model registry.
/// Each view then selects its route-relevant schema and prompt from these (see
/// [`build_route_record`]).
struct RecordSetup {
    adapter: ModelAdapter,
    schemas: Vec<SchemaEntry>,
    prompts: Vec<PromptEntry>,
}

/// One model route's record-mode inputs, assembled per view: a borrow of the
/// shared runtime adapter, the route prompt template text (its `inline` text or
/// the read `path` file), the constraint (schema/grammar) path handed to the
/// runtime, and the recording context (producer, template hash, budget). The
/// fills compose the per-document/per-role prompt from `template` and pass a
/// borrow of this value as [`FillSource::Record`].
struct RouteRecord<'a> {
    adapter: &'a ModelAdapter,
    template: String,
    constraint: PathBuf,
    ctx: RecordContext,
}

/// Assemble the shared record-mode inputs once for a `ckc run --record` model
/// run: probe the live model-runtime adapter, then load and validate the
/// committed model registry (`schemas.yaml` / `prompts.yaml`). Any failure is
/// command-scope — the adapter is load-bearing for every fill, and an invalid
/// registry cannot be joined safely — so it lands one diagnostic and returns
/// `None`, stopping the run. The adapter failure is a plain `invalid`
/// diagnostic, not a solver code: no model-runtime diagnostic code exists yet
/// (a dedicated §7.4 code is a later concern).
fn build_record_setup(root: &Path, shell: &mut Shell) -> Option<RecordSetup> {
    let adapter = match ModelAdapter::new() {
        Ok(adapter) => adapter,
        Err(e) => {
            shell.diagnostic(invalid_diagnostic(vec![(
                static_id("reason"),
                format!("model adapter: {e}"),
            )]));
            return None;
        }
    };
    let schemas = load(root, SCHEMAS_FILE, parse_schemas, shell)?;
    let prompts = load(root, PROMPTS_FILE, parse_prompts, shell)?;
    let findings = validate_model_registry(&schemas, &prompts);
    if !findings.is_empty() {
        for finding in findings {
            shell.diagnostic(invalid_diagnostic(vec![(
                static_id("finding"),
                finding.to_string(),
            )]));
        }
        return None;
    }
    Some(RecordSetup {
        adapter,
        schemas,
        prompts,
    })
}

/// The adapter-free slice of one view's record-mode inputs: what
/// [`build_record_parts`] assembles and [`build_route_record`] pairs with the
/// shared adapter borrow. Split out so the deterministic assembly —
/// selection, template load, byte-verification, budget requirement — is
/// testable without probing a live runtime.
#[derive(Debug)]
struct RecordParts {
    template: String,
    constraint: PathBuf,
    ctx: RecordContext,
}

/// Assemble one view's adapter-free record inputs: the route-relevant schema
/// and prompt entry ([`select_record_schema`] / [`select_record_prompt`],
/// mirroring the §9 manifest want-set), the prompt template (its `inline`
/// text or the read `path` file), the constraint path, and the recording
/// context. Failures are loud (`Err`), never a silent replay: a missing
/// wanted entry — a layered M1 shape reaching here, or a registry gap
/// [`validate_model_registry`] did not catch; a missing `model_ms_per_call`
/// budget (recording invokes the runtime, so the cap is required here even
/// though replay ignores it); or a byte-verification miss. The
/// byte-verification closes the declared-hash trust gap before any cassette
/// records against those hashes: the selected template bytes must hash to
/// the entry's `template_hash` and the constraint file bytes to the entry's
/// `schema_hash` — `registry check` is otherwise the sole byte-verifier, and
/// a recording made against drifted bytes would stamp a hash its inputs
/// never had. The `path` joins are safe: [`build_record_setup`] validated
/// every path before this.
fn build_record_parts(
    root: &Path,
    schemas: &[SchemaEntry],
    prompts: &[PromptEntry],
    resolved: &Resolved,
) -> Result<RecordParts, String> {
    let shape = resolved.shape;
    let schema = select_record_schema(schemas, shape)
        .ok_or_else(|| format!("record: no committed schema for route shape {shape:?}"))?;
    let prompt = select_record_prompt(prompts, shape)
        .ok_or_else(|| format!("record: no committed prompt for route shape {shape:?}"))?;
    let budget_ms = resolved.model_ms_per_call.ok_or_else(|| {
        format!(
            "record: experiment {} declares no {MODEL_MS_PER_CALL_KEY} budget",
            resolved.plan.experiment_id
        )
    })?;
    let template = match (&prompt.inline, &prompt.path) {
        (Some(inline), _) => inline.clone(),
        (None, Some(path)) => std::fs::read_to_string(root.join(path))
            .map_err(|e| format!("record: read prompt {path}: {e}"))?,
        (None, None) => {
            return Err(format!(
                "record: prompt {} has neither inline nor path",
                prompt.id
            ));
        }
    };
    let template_hash = hash_bytes(template.as_bytes());
    if template_hash != prompt.template_hash {
        return Err(format!(
            "record: prompt {} template bytes hash {template_hash}, declared template_hash {}",
            prompt.id, prompt.template_hash
        ));
    }
    let constraint = root.join(&schema.path);
    let schema_bytes = std::fs::read(&constraint)
        .map_err(|e| format!("record: read schema {}: {e}", schema.path))?;
    let schema_hash = hash_bytes(&schema_bytes);
    if schema_hash != schema.schema_hash {
        return Err(format!(
            "record: schema {} file bytes hash {schema_hash}, declared schema_hash {}",
            schema.id, schema.schema_hash
        ));
    }
    Ok(RecordParts {
        template,
        constraint,
        ctx: RecordContext {
            producer: producer(resolved, MODEL_FILL),
            prompt_template_hash: prompt.template_hash.clone(),
            budget: Duration::from_millis(budget_ms),
        },
    })
}

/// Assemble one view's [`RouteRecord`]: [`build_record_parts`] plus a borrow
/// of the shared live adapter.
fn build_route_record<'a>(
    root: &Path,
    setup: &'a RecordSetup,
    resolved: &Resolved,
) -> Result<RouteRecord<'a>, String> {
    let RecordParts {
        template,
        constraint,
        ctx,
    } = build_record_parts(root, &setup.schemas, &setup.prompts, resolved)?;
    Ok(RouteRecord {
        adapter: &setup.adapter,
        template,
        constraint,
        ctx,
    })
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
    agreed: Option<&ModelIdentity>,
    model_routes: &[RouteShape],
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
        output_hashes.extend(group.smt_queries.iter().map(|q| q.content_hash.clone()));
        output_hashes.extend(group.compiled.iter().map(|c| c.content_hash.clone()));
        output_hashes.extend(
            group
                .verifier_results
                .iter()
                .map(|v| v.content_hash.clone()),
        );
    }

    // §9 M2 measurement record. A model-route run locks its outputs against
    // the run's actual inputs (not the whole registry): the identity the
    // route fills agreed on, the route-expanded test-source hashes, the
    // reference, and the route-relevant schema + prompt-template hashes. The
    // deterministic M1 baseline runs no model route (`model_routes` empty), so
    // every field is omitted (None) and its manifest bytes stay unchanged. The
    // model and runtime are environment bare-name commands with no committed
    // bytes, so their raw-byte hashes stay None (identity rides `model_identity`).
    let (model_identity, test_source_hash, reference_hash, schema_hash, prompt_template_hash) =
        if model_routes.is_empty() {
            (None, None, None, None, None)
        } else {
            let experiments = parse_experiments(
                &std::fs::read_to_string(root.join(EXPERIMENTS_FILE))
                    .map_err(|e| format!("manifests: read {EXPERIMENTS_FILE}: {e}"))?,
            )
            .map_err(|e| format!("manifests: parse {EXPERIMENTS_FILE}: {e}"))?;
            let exp = experiments
                .into_iter()
                .find(|e| e.id == resolved.plan.experiment_id)
                .ok_or_else(|| "manifests: experiment absent from registry".to_owned())?;
            let reference = std::fs::read(root.join(&exp.expected_outcomes))
                .map_err(|e| format!("manifests: read reference {}: {e}", exp.expected_outcomes))?;

            // Each model route locks only the schema and prompt its target kind
            // consumes (single_ir → ClinicalIR, direct_smt → SMT query); the
            // prompt registry is 1:1 with the route. A layered route carries no
            // model-fill schema/prompt, so its presence in the model-route set
            // is a caller contract violation — fail loudly rather than lock an
            // empty selection.
            let mut want_schema: BTreeSet<Id> = BTreeSet::new();
            let mut want_prompt: BTreeSet<Id> = BTreeSet::new();
            for shape in model_routes {
                match shape {
                    RouteShape::SingleIr => {
                        want_schema.insert(static_id("schema.clinical_ir"));
                        want_prompt.insert(static_id("prompt.single_ir"));
                    }
                    RouteShape::DirectSmt => {
                        want_schema.insert(static_id("schema.smt_query"));
                        want_prompt.insert(static_id("prompt.direct_smt"));
                    }
                    RouteShape::M1Layered => {
                        return Err(
                            "manifests: layered route carries no §9 model schema/prompt".to_owned()
                        );
                    }
                }
            }
            let schemas = parse_schemas(
                &std::fs::read_to_string(root.join(SCHEMAS_FILE))
                    .map_err(|e| format!("manifests: read {SCHEMAS_FILE}: {e}"))?,
            )
            .map_err(|e| format!("manifests: parse {SCHEMAS_FILE}: {e}"))?;
            let prompts = parse_prompts(
                &std::fs::read_to_string(root.join(PROMPTS_FILE))
                    .map_err(|e| format!("manifests: read {PROMPTS_FILE}: {e}"))?,
            )
            .map_err(|e| format!("manifests: parse {PROMPTS_FILE}: {e}"))?;
            (
                agreed.cloned(),
                Some(aggregate_hashes(
                    docs.iter()
                        .map(|d| d.source_hash.as_str().to_owned())
                        .collect(),
                )),
                Some(hash_bytes(&reference)),
                Some(select_route_hashes(
                    &want_schema,
                    schemas.iter().map(|s| (&s.id, &s.schema_hash)),
                    "schema",
                )?),
                Some(select_route_hashes(
                    &want_prompt,
                    prompts.iter().map(|p| (&p.id, &p.template_hash)),
                    "prompt",
                )?),
            )
        };

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
        // §9 M2 measurement record (computed above): populated on a
        // model-route run, omitted (None) on the deterministic M1 baseline.
        model_identity,
        test_source_hash,
        reference_hash,
        schema_hash,
        prompt_template_hash,
        model_hash: None,
        runtime_hash: None,
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

/// §4.4 producer for a run-level tail wrapper. The M1 baseline (`emit_event`
/// true) owns real trace/report pipeline steps, so it keeps [`producer`]'s
/// route step. A model-route run (`emit_event` false) runs the tail once over
/// every route with no owning pipeline step — the route's
/// `pipeline_step_ids[idx]` slot is the inert [`UNUSED_STAGE`] sentinel — so it
/// carries a run-level step id (`processing_stage.run.{trace,report}`) instead.
/// This id rides the wrapper's content-hash-excluded [`Producer`], so it
/// re-blesses only the emitted trace_bundle/lineage_index/report.json wrapper
/// bytes, never a payload hash, layout, census, or determinism.
fn tail_producer(resolved: &Resolved, processing_stage_index: usize, emit_event: bool) -> Producer {
    if emit_event {
        return producer(resolved, processing_stage_index);
    }
    let step = match processing_stage_index {
        TRACE => RUN_TRACE_STEP,
        REPORT => RUN_REPORT_STEP,
        _ => unreachable!("run-level tail producer runs only the trace and report stages"),
    };
    Producer {
        pipeline_id: resolved.pipeline_id.clone(),
        pipeline_step_id: static_id(step),
        toolchain_manifest_hash: resolved.toolchain_manifest_hash.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use ckc_core::{
        Action, BindingStatus, CassettePayload, ClinicalStatement, Direction, EventRecord,
        EvidenceRegion, ExceptionClause, ModelIdentity, Provenance, QuantityInterval, SegmentKind,
        SourceDocument, SourceTextSpan, Strength, TerminologyBinding, TotalOperationResult,
        parse_reference, read_jsonl,
    };

    use crate::normalize::{LexiconAction, LexiconConcept, clinical_ir};

    /// A test [`Hash`] over a repeated seed character (value is arbitrary — the
    /// record-prompt helpers key on ids and span text, never on these hashes).
    fn prompt_hash(seed: char) -> Hash {
        Hash::new(format!("sha256:{}", seed.to_string().repeat(64))).unwrap()
    }

    /// A `registry/schemas.yaml` entry with the given id; `target_kind` is
    /// distinct from `id` so a selection-by-id test cannot pass by accident.
    fn schema_entry(id: &str, target_kind: &str) -> SchemaEntry {
        SchemaEntry {
            id: static_id(id),
            path: format!("schemas/{id}.json"),
            schema_hash: prompt_hash('c'),
            target_kind: static_id(target_kind),
        }
    }

    /// A `registry/prompts.yaml` entry with the given id and route.
    fn prompt_entry(id: &str, route: &str) -> PromptEntry {
        PromptEntry {
            id: static_id(id),
            path: Some(format!("registry/prompts/{id}.txt")),
            inline: None,
            template_hash: prompt_hash('d'),
            route: static_id(route),
        }
    }

    /// A minimal [`SourceDocumentGraph`] for prompt-composition tests: `spans`
    /// carry `(reading_order, raw_text)` in the given (possibly unsorted) array
    /// order, so a composer's `reading_order` sort is observable; the node,
    /// anchor, and region pools start empty (the single_ir scaffold test pushes
    /// its own regions).
    fn prompt_graph(doc: &str, spans: &[(u64, &str)]) -> SourceDocumentGraph {
        SourceDocumentGraph {
            document: SourceDocument {
                document_id: static_id(doc),
                source_family: static_id("family.test_source_html"),
                provenance: Provenance::Synthetic,
                raw_hash: prompt_hash('a'),
                content_hash: prompt_hash('b'),
                data_class: DataClass::None,
            },
            nodes: vec![],
            spans: spans
                .iter()
                .enumerate()
                .map(|(k, (order, text))| {
                    SourceTextSpan::derive(
                        static_id(&format!("span.{k}")),
                        static_id("node.0"),
                        0,
                        (*text).to_owned(),
                        *order,
                    )
                })
                .collect(),
            anchors: vec![],
            regions: vec![],
        }
    }

    /// [`select_record_schema`] keys the committed schema entry off the route
    /// shape by `id`: single_ir → `schema.clinical_ir`, direct_smt →
    /// `schema.smt_query`; the layered M1 shape and an absent id both miss.
    #[test]
    fn select_record_schema_by_route_shape() {
        let schemas = [
            schema_entry("schema.clinical_ir", "clinical_ir"),
            schema_entry("schema.smt_query", "smt_query"),
        ];
        assert_eq!(
            select_record_schema(&schemas, RouteShape::SingleIr).map(|s| s.id.as_str()),
            Some("schema.clinical_ir")
        );
        assert_eq!(
            select_record_schema(&schemas, RouteShape::DirectSmt).map(|s| s.id.as_str()),
            Some("schema.smt_query")
        );
        assert!(select_record_schema(&schemas, RouteShape::M1Layered).is_none());
        // Miss: the wanted id absent from the registry selects nothing.
        assert!(select_record_schema(&schemas[..1], RouteShape::DirectSmt).is_none());
    }

    /// [`select_record_prompt`] keys the committed prompt entry off the route
    /// shape by `id`: single_ir → `prompt.single_ir`, direct_smt →
    /// `prompt.direct_smt`; the layered M1 shape and an absent id both miss.
    #[test]
    fn select_record_prompt_by_route_shape() {
        let prompts = [
            prompt_entry("prompt.single_ir", "route.single_ir"),
            prompt_entry("prompt.direct_smt", "route.direct_smt"),
        ];
        assert_eq!(
            select_record_prompt(&prompts, RouteShape::SingleIr).map(|p| p.id.as_str()),
            Some("prompt.single_ir")
        );
        assert_eq!(
            select_record_prompt(&prompts, RouteShape::DirectSmt).map(|p| p.id.as_str()),
            Some("prompt.direct_smt")
        );
        assert!(select_record_prompt(&prompts, RouteShape::M1Layered).is_none());
        // Miss: the wanted id absent from the registry selects nothing.
        assert!(select_record_prompt(&prompts[..1], RouteShape::DirectSmt).is_none());
    }

    /// [`single_ir_prompt`] emits the template, a document line, the grounding
    /// scaffold (segment lines with kind and covered regions, the graph's
    /// region-id line, the lexicon vocabulary block with `var=` marks only on
    /// interval concepts), then the document's spans in `reading_order` — the
    /// spans supplied out of order to prove the composer sorts, the regions
    /// supplied in non-sorted array order to pin that scaffold lines ride
    /// artifact order untouched.
    #[test]
    fn single_ir_prompt_composes_grounding_scaffold_and_orders_spans() {
        let mut graph = prompt_graph("doc.a", &[(1, "second line"), (0, "first line")]);
        graph.regions = ["region.r1", "region.r0"]
            .map(|r| EvidenceRegion {
                region_id: static_id(r),
                node_ids: vec![],
                span_ids: vec![],
                anchor_ids: vec![],
            })
            .to_vec();
        let segments = [
            ClinicalSegment {
                segment_id: static_id("seg.s0"),
                kind: SegmentKind::Recommendation,
                region_ids: vec![static_id("region.r1")],
            },
            ClinicalSegment {
                segment_id: static_id("seg.s1"),
                kind: SegmentKind::Exception,
                region_ids: vec![static_id("region.r0"), static_id("region.r1")],
            },
        ];
        let lexicon = Lexicon {
            system: static_id("lex.test"),
            content_hash: prompt_hash('c'),
            concepts: vec![
                LexiconConcept {
                    concept_id: static_id("pop.adult"),
                    surfaces: vec![],
                    interval: Some(QuantityInterval {
                        var: static_id("q.age_years"),
                        ge: Some(18),
                        gt: None,
                        le: None,
                        lt: None,
                    }),
                },
                LexiconConcept {
                    concept_id: static_id("drug.x"),
                    surfaces: vec![],
                    interval: None,
                },
            ],
            actions: vec![LexiconAction {
                action_id: static_id("act.administer"),
                surfaces: vec![],
            }],
            modality: vec![],
            certainty: vec![],
        };
        let prompt = single_ir_prompt("TEMPLATE", &static_id("doc.a"), &graph, &segments, &lexicon);
        assert_eq!(
            prompt,
            "TEMPLATE\n\
             document: doc.a\n\
             segment: seg.s0 kind=recommendation regions=region.r1\n\
             segment: seg.s1 kind=exception regions=region.r0,region.r1\n\
             regions: region.r1,region.r0\n\
             system: lex.test\n\
             concept: pop.adult var=q.age_years\n\
             concept: drug.x\n\
             action: act.administer\n\
             first line\n\
             second line"
        );
    }

    /// [`direct_smt_prompt`] emits the template, group and role lines, then each
    /// member's document line followed by its spans in `reading_order` — `doc.a`
    /// supplied out of order to prove the composer sorts each member.
    #[test]
    fn direct_smt_prompt_lays_out_role_and_members() {
        let a = prompt_graph("doc.a", &[(1, "a2"), (0, "a1")]);
        let b = prompt_graph("doc.b", &[(0, "b1")]);
        let members: Vec<(&Id, &SourceDocumentGraph)> =
            vec![(&a.document.document_id, &a), (&b.document.document_id, &b)];
        let prompt = direct_smt_prompt("TEMPLATE", &static_id("group.g0"), "overlap", &members);
        assert_eq!(
            prompt,
            "TEMPLATE\ngroup: group.g0\nrole: overlap\ndocument: doc.a\na1\na2\ndocument: doc.b\nb1"
        );
    }

    /// The committed model registry parsed from the repository root, for
    /// [`build_record_parts`] tests (the run path loads the same files).
    fn committed_model_registry() -> (Vec<SchemaEntry>, Vec<PromptEntry>) {
        let root = repo_root();
        let schemas =
            parse_schemas(&std::fs::read_to_string(root.join(SCHEMAS_FILE)).unwrap()).unwrap();
        let prompts =
            parse_prompts(&std::fs::read_to_string(root.join(PROMPTS_FILE)).unwrap()).unwrap();
        (schemas, prompts)
    }

    /// [`build_record_parts`] over the committed registry byte-verifies clean
    /// on both model routes: each selected template survives the
    /// `template_hash` check (so the committed prompt files and
    /// `registry/prompts.yaml` declare matching bytes), each constraint file
    /// survives the `schema_hash` check, and the recording context carries
    /// the declared template hash, the route's model_fill producer, and the
    /// §9 `model_ms_per_call` budget — not the solver cap.
    #[test]
    fn build_record_parts_verifies_committed_bytes_and_budget() {
        let root = repo_root();
        let (schemas, prompts) = committed_model_registry();

        let single = build_record_parts(&root, &schemas, &prompts, &single_ir_resolved())
            .expect("committed single_ir registry bytes verify");
        let template = std::fs::read_to_string(root.join("registry/prompts/single_ir.txt"))
            .expect("committed single_ir template");
        assert_eq!(single.template, template);
        assert_eq!(
            single.constraint,
            root.join("schemas/clinical_ir.schema.json")
        );
        assert_eq!(
            single.ctx.prompt_template_hash,
            hash_bytes(template.as_bytes())
        );
        assert_eq!(single.ctx.budget, Duration::from_millis(600_000));
        assert_eq!(
            single.ctx.producer.pipeline_id,
            static_id("pipe.m2_single_ir")
        );
        assert_eq!(
            single.ctx.producer.pipeline_step_id,
            static_id("processing_stage.m2.model_fill")
        );

        let direct = build_record_parts(&root, &schemas, &prompts, &direct_smt_resolved())
            .expect("committed direct_smt registry bytes verify");
        assert_eq!(direct.constraint, root.join("schemas/smt_query.grammar"));
        assert_eq!(direct.ctx.budget, Duration::from_millis(600_000));
    }

    /// A model-route view without the §9 `model_ms_per_call` budget fails
    /// [`build_record_parts`] loudly, naming the key — recording invokes the
    /// runtime, so the cap is required even though replay ignores it.
    #[test]
    fn build_record_parts_requires_model_ms_per_call() {
        let (schemas, prompts) = committed_model_registry();
        let mut resolved = single_ir_resolved();
        resolved.model_ms_per_call = None;
        let err = build_record_parts(&repo_root(), &schemas, &prompts, &resolved)
            .expect_err("a record view without the model budget is rejected");
        assert!(
            err.contains("declares no model_ms_per_call budget"),
            "{err}"
        );
    }

    /// Template bytes that hash away from the entry's declared
    /// `template_hash` fail the pre-record byte-verification (here via the
    /// `inline` template arm, which the check covers like a `path` read).
    #[test]
    fn build_record_parts_rejects_template_hash_drift() {
        let (schemas, _) = committed_model_registry();
        let prompts = [PromptEntry {
            id: static_id("prompt.single_ir"),
            path: None,
            inline: Some("TEMPLATE".to_owned()),
            template_hash: prompt_hash('d'),
            route: static_id("route.single_ir"),
        }];
        let err = build_record_parts(&repo_root(), &schemas, &prompts, &single_ir_resolved())
            .expect_err("drifted template bytes are rejected");
        assert!(err.contains("template bytes hash"), "{err}");
    }

    /// A constraint file that hashes away from the entry's declared
    /// `schema_hash` fails the pre-record byte-verification.
    #[test]
    fn build_record_parts_rejects_schema_hash_drift() {
        let (mut schemas, prompts) = committed_model_registry();
        schemas
            .iter_mut()
            .find(|s| s.id.as_str() == "schema.clinical_ir")
            .expect("committed registry declares schema.clinical_ir")
            .schema_hash = prompt_hash('e');
        let err = build_record_parts(&repo_root(), &schemas, &prompts, &single_ir_resolved())
            .expect_err("drifted schema bytes are rejected");
        assert!(err.contains("file bytes hash"), "{err}");
    }

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
        execute(root, &experiment.parse().unwrap(), false, &mut shell);
        let finished = shell.finish().unwrap();
        let events = read_jsonl(&std::fs::read(out.join("logs/events.jsonl")).unwrap()).unwrap();
        let diagnostics =
            read_jsonl(&std::fs::read(out.join("logs/diagnostics.jsonl")).unwrap()).unwrap();
        (finished.result, events, diagnostics, out, tmp)
    }

    /// A resolution/gate failure mints nothing: the run dir holds exactly
    /// the shell's `logs/`, so any other minted path fails the pin.
    fn assert_only_logs(out: &Path) {
        let mut entries: Vec<String> = std::fs::read_dir(out)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        entries.sort();
        assert_eq!(entries, ["logs"]);
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

    /// run-m2.1b mutation lever: string-replace one anchored slice of a
    /// written registry file, so each rejection case derives from
    /// [`write_tiny_root`]'s valid trio by exactly one byte-level edit.
    fn mutate(root: &Path, rel: &str, from: &str, to: &str) {
        let path = root.join(rel);
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            text.match_indices(from).count(),
            1,
            "{rel} must hold exactly one mutation anchor {from:?}"
        );
        std::fs::write(path, text.replacen(from, to, 1)).unwrap();
    }

    /// Mirror the committed registry surface [`resolve`] reads — the three
    /// registry files plus the toolchain provenance bytes — into `root`, so a
    /// mutation test degrades the real `exp.m2_multihop` binding by one edit.
    fn copy_committed_registry(root: &Path) {
        let repo = repo_root();
        for rel in [
            CORPORA_FILE,
            "registry/candidates.yaml",
            EXPERIMENTS_FILE,
            SCHEMAS_FILE,
            PROMPTS_FILE,
            TOOLCHAIN_FILE,
        ] {
            let target = root.join(rel);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(repo.join(rel), target).unwrap();
        }
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

    // Duplicate set members would mint colliding views (`registry check`
    // flags the same invariant at the whole-set level), so the member loop
    // rejects them before any view lands.
    #[test]
    fn duplicate_set_member_stops_resolution() {
        let root = tempfile::tempdir().unwrap();
        write_tiny_root(root.path());
        std::fs::write(
            root.path().join("registry/experiments.yaml"),
            "\
- id: exp.tiny
  pipelines: [pipe.tiny, pipe.tiny]
  baseline_pipeline: pipe.tiny
  test_source_groups:
    - group_id: group.t
      test_sources: [test_source.tiny]
  seed: 1
  budget: {solver_ms_per_query: 10000}
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
                .any(|(_, v)| v.contains("binds duplicate pipeline pipe.tiny")),
            "{diagnostics:?}"
        );
        assert_only_logs(&out);
    }

    // run-m2.1b (a) — dropping the declared normalize stage leaves a 7-kind
    // sequence matching no supported route shape; the diagnostic names the
    // kinds the pipeline actually declares.
    #[test]
    fn unsupported_stage_sequence_stops_resolution() {
        let root = tempfile::tempdir().unwrap();
        write_tiny_root(root.path());
        mutate(
            root.path(),
            "registry/candidates.yaml",
            "processing_stage.t.normalize, ",
            "",
        );

        let (result, events, diagnostics, out, _tmp) = executed(root.path(), "exp.tiny");
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(events.len(), 1);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0].payload.iter().any(|(_, v)| {
                v.contains(
                    "pipeline pipe.tiny declares an unsupported processing-stage sequence \
                     [extract, segment, assemble, compile, verify, trace, report]",
                )
            }),
            "{diagnostics:?}"
        );
        assert_only_logs(&out);
    }

    // run-m2.1b (b) — the declared list references a stage id the registry
    // never defines.
    #[test]
    fn undefined_processing_stage_stops_resolution() {
        let root = tempfile::tempdir().unwrap();
        write_tiny_root(root.path());
        mutate(
            root.path(),
            "registry/candidates.yaml",
            "processing_stage.t.normalize, ",
            "processing_stage.t.ghost, ",
        );

        let (result, events, diagnostics, out, _tmp) = executed(root.path(), "exp.tiny");
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(events.len(), 1);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0].payload.iter().any(|(_, v)| {
                v.contains(
                    "pipeline pipe.tiny declares undefined processing_stage \
                     processing_stage.t.ghost",
                )
            }),
            "{diagnostics:?}"
        );
        assert_only_logs(&out);
    }

    // run-m2.1b (c) — the experiment names a pipeline id the registry never
    // defines.
    #[test]
    fn undefined_pipeline_stops_resolution() {
        let root = tempfile::tempdir().unwrap();
        write_tiny_root(root.path());
        mutate(
            root.path(),
            "registry/experiments.yaml",
            "pipeline: pipe.tiny",
            "pipeline: pipe.ghost",
        );

        let (result, events, diagnostics, out, _tmp) = executed(root.path(), "exp.tiny");
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(events.len(), 1);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .payload
                .iter()
                .any(|(_, v)| v.contains("experiment exp.tiny names undefined pipeline pipe.ghost")),
            "{diagnostics:?}"
        );
        assert_only_logs(&out);
    }

    // run-m2.1b (d) — set form plus a stray legacy `pipeline:` key is a
    // malformed binding: the shape-aware `baseline()` resolves nothing, so
    // the run rejects exactly what `registry check` rejects.
    #[test]
    fn malformed_binding_stops_resolution() {
        let root = tempfile::tempdir().unwrap();
        write_tiny_root(root.path());
        mutate(
            root.path(),
            "registry/experiments.yaml",
            "  pipeline: pipe.tiny\n",
            "  pipeline: pipe.tiny\n  pipelines: [pipe.tiny]\n  baseline_pipeline: pipe.tiny\n",
        );

        let (result, events, diagnostics, out, _tmp) = executed(root.path(), "exp.tiny");
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(events.len(), 1);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .payload
                .iter()
                .any(|(_, v)| v.contains("experiment exp.tiny has no valid pipeline binding")),
            "{diagnostics:?}"
        );
        assert_only_logs(&out);
    }

    // run-m2.1a — the §9 two-route experiment resolves one view per set
    // member over the committed registry: set order [direct_smt, single_ir],
    // declared stages filling the fixed slots in declared order with the
    // unused-sentinel padding, both views sharing the M1 corpus documents,
    // groups, budget, and the one §5 plan.
    #[test]
    fn m2_experiment_resolves_one_view_per_route() {
        let root = repo_root();
        let mut shell = Shell::open(static_id("run"), static_id("m2"), None);
        let views = resolve(&root, &"exp.m2_multihop".parse().unwrap(), &mut shell)
            .expect("exp.m2_multihop resolves over the committed registry");
        assert!(shell.ledger().is_empty(), "{:?}", shell.ledger());

        assert_eq!(views.len(), 2);
        let direct = &views[0];
        assert_eq!(direct.pipeline_id, static_id("pipe.m2_direct_smt"));
        assert_eq!(direct.shape, RouteShape::DirectSmt);
        assert_eq!(direct.repair_limit, Some(1));
        assert!(direct.is_baseline, "direct_smt is the §9 delta baseline");
        assert_eq!(route_id_prefix(direct), "pipe.m2_direct_smt.");
        assert_eq!(
            direct.pipeline_step_ids,
            [
                static_id("processing_stage.m1.extract"),
                static_id("processing_stage.m1.segment"),
                static_id("processing_stage.m2.model_fill_smt"),
                static_id("processing_stage.m2.verify_smt"),
                static_id(UNUSED_STAGE),
                static_id(UNUSED_STAGE),
                static_id(UNUSED_STAGE),
                static_id(UNUSED_STAGE),
            ]
        );
        let single_ir = &views[1];
        assert_eq!(single_ir.pipeline_id, static_id("pipe.m2_single_ir"));
        assert_eq!(single_ir.shape, RouteShape::SingleIr);
        assert_eq!(single_ir.repair_limit, Some(1));
        assert!(!single_ir.is_baseline);
        assert_eq!(route_id_prefix(single_ir), "pipe.m2_single_ir.");
        assert_eq!(
            single_ir.pipeline_step_ids,
            [
                static_id("processing_stage.m1.extract"),
                static_id("processing_stage.m1.segment"),
                static_id("processing_stage.m2.model_fill"),
                static_id("processing_stage.m2.assemble"),
                static_id("processing_stage.m1.compile"),
                static_id("processing_stage.m1.verify"),
                static_id(UNUSED_STAGE),
                static_id(UNUSED_STAGE),
            ]
        );
        for view in &views {
            assert_eq!(view.budget_ms, 10_000);
            assert_eq!(view.model_ms_per_call, Some(600_000));
            assert_eq!(
                view.documents
                    .iter()
                    .map(|d| d.id.clone())
                    .collect::<Vec<_>>(),
                DOC_IDS.map(static_id)
            );
            assert_eq!(
                view.groups
                    .iter()
                    .map(|g| g.group_id.clone())
                    .collect::<Vec<_>>(),
                [
                    static_id("group.m1_conflict"),
                    static_id("group.m1_no_conflict")
                ]
            );
            assert_eq!(view.plan.experiment_id, static_id("exp.m2_multihop"));
            assert_eq!(
                view.plan.pipelines,
                [
                    static_id("pipe.m2_direct_smt"),
                    static_id("pipe.m2_single_ir")
                ]
            );
            assert_eq!(view.plan.seed, 42);
            assert_eq!(
                view.plan.budget,
                [
                    (static_id("model_ms_per_call"), 600_000),
                    (static_id("model_repair_limit"), 1),
                    (static_id("model_sample_count"), 1),
                    (static_id("solver_ms_per_query"), 10_000),
                ]
            );
        }
    }

    // run-m2.1d2 — the M1 legacy binding's single view is its own baseline,
    // spends no repairs, and keeps its unprefixed artifact-id namespace, so
    // M1 behavior is unchanged by the model-route resolve extension.
    #[test]
    fn m1_view_resolves_unprefixed_baseline() {
        let root = tempfile::tempdir().unwrap();
        write_tiny_root(root.path());
        let mut shell = Shell::open(static_id("run"), static_id("t"), None);
        let views = resolve(root.path(), &"exp.tiny".parse().unwrap(), &mut shell)
            .expect("exp.tiny resolves over the tiny root");
        assert!(shell.ledger().is_empty(), "{:?}", shell.ledger());
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].shape, RouteShape::M1Layered);
        assert_eq!(views[0].repair_limit, None);
        // No `model_ms_per_call` key in the binding → honest `None` (the
        // record path, not resolution, requires the key).
        assert_eq!(views[0].model_ms_per_call, None);
        assert!(views[0].is_baseline);
        assert_eq!(route_id_prefix(&views[0]), "");
    }

    // run-m2.1d2 — a model-route binding without the §9 repair budget stops
    // resolution: the repair loop needs an explicit cap before a route runs.
    #[test]
    fn missing_model_repair_limit_stops_resolution() {
        let root = tempfile::tempdir().unwrap();
        copy_committed_registry(root.path());
        mutate(
            root.path(),
            "registry/experiments.yaml",
            "    model_repair_limit: 1\n",
            "",
        );
        let mut shell = Shell::open(static_id("run"), static_id("m2"), None);
        assert!(resolve(root.path(), &"exp.m2_multihop".parse().unwrap(), &mut shell).is_none());
        let ledger = shell.ledger();
        assert_eq!(ledger.len(), 1);
        assert!(
            ledger[0].payload.iter().any(|(_, v)| v
                .contains("experiment exp.m2_multihop declares no model_repair_limit budget")),
            "{ledger:?}"
        );
    }

    // run-m2.1d2 — a repair budget beyond u32 stops resolution rather than
    // truncating the cap.
    #[test]
    fn model_repair_limit_beyond_u32_stops_resolution() {
        let root = tempfile::tempdir().unwrap();
        copy_committed_registry(root.path());
        mutate(
            root.path(),
            "registry/experiments.yaml",
            "model_repair_limit: 1\n",
            "model_repair_limit: 4294967296\n",
        );
        let mut shell = Shell::open(static_id("run"), static_id("m2"), None);
        assert!(resolve(root.path(), &"exp.m2_multihop".parse().unwrap(), &mut shell).is_none());
        let ledger = shell.ledger();
        assert_eq!(ledger.len(), 1);
        assert!(
            ledger[0].payload.iter().any(|(_, v)| v.contains(
                "experiment exp.m2_multihop declares a model_repair_limit budget beyond u32 \
                 (4294967296)"
            )),
            "{ledger:?}"
        );
    }

    // run-m2.1d2 — a declared sample count other than 1 stops resolution:
    // replay executes exactly one recorded draw per attempt, so a larger
    // count would promise samples the recorded run never draws.
    #[test]
    fn model_sample_count_beyond_one_stops_resolution() {
        let root = tempfile::tempdir().unwrap();
        copy_committed_registry(root.path());
        mutate(
            root.path(),
            "registry/experiments.yaml",
            "model_sample_count: 1",
            "model_sample_count: 2",
        );
        let mut shell = Shell::open(static_id("run"), static_id("m2"), None);
        assert!(resolve(root.path(), &"exp.m2_multihop".parse().unwrap(), &mut shell).is_none());
        let ledger = shell.ledger();
        assert_eq!(ledger.len(), 1);
        assert!(
            ledger[0].payload.iter().any(|(_, v)| v.contains(
                "experiment exp.m2_multihop declares model_sample_count 2; \
                 replay draws exactly one sample"
            )),
            "{ledger:?}"
        );
    }

    /// Mirror the committed inputs `exp.m2_multihop` reads into a fresh `root` so the
    /// route loop runs model-runtime-absent: the three registry files and toolchain
    /// provenance ([`copy_committed_registry`]), plus `Cargo.lock`, the lexicon, the
    /// three M1 corpus documents, and the shared reference — and, under
    /// `<root>/cassettes/` where [`CassetteStore::new`] reads them, the seven golden
    /// seed-42 cassettes both routes replay (three single_ir ClinicalIR fills, two
    /// direct_smt overlap/deontic pairs), sourced from the committed test fixtures.
    fn write_m2_root(root: &Path) {
        let repo = repo_root();
        let copy = |rel: &str, from: PathBuf| {
            let target = root.join(rel);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(from, target).unwrap();
        };
        copy_committed_registry(root);
        for rel in [
            LOCKFILE,
            LEXICON_FILE,
            "corpus/test_sources/m1_guideline_a.html",
            "corpus/test_sources/m1_guideline_b.html",
            "corpus/test_sources/m1_control.html",
            "corpus/reference/m1_expected.yaml",
        ] {
            copy(rel, repo.join(rel));
        }
        let fixtures = repo.join("crates/ckc-cli/tests/fixtures");
        for rel in [
            "cassettes/route.single_ir/test_source.m1_control/seed-42.json",
            "cassettes/route.single_ir/test_source.m1_guideline_a/seed-42.json",
            "cassettes/route.single_ir/test_source.m1_guideline_b/seed-42.json",
            "cassettes/route.direct_smt/group.m1_conflict.overlap/seed-42.json",
            "cassettes/route.direct_smt/group.m1_conflict.deontic/seed-42.json",
            "cassettes/route.direct_smt/group.m1_no_conflict.overlap/seed-42.json",
            "cassettes/route.direct_smt/group.m1_no_conflict.deontic/seed-42.json",
        ] {
            copy(rel, fixtures.join(rel));
        }
    }

    // run-m2.1d5a-1 — the model-route loop executes `exp.m2_multihop`'s two routes over
    // one shared run out, landing each route's artifacts route-namespaced under
    // `out/routes/{pipeline_id}/` so the two never collide: single_ir lands three
    // document heads (each with its ClinicalIR bundle) and both groups' compiled +
    // verifier_results; direct_smt lands three heads and both groups' overlap/deontic
    // smt_query pair + verifier_results. A clean replay raises no command diagnostics
    // and leaves no bare `out/groups/`. The unified trace/report tails then land the
    // run-level artifacts at the bare run root; the two-run determinism and event
    // census pins land in run-m2.1d5b.
    #[test]
    fn m2_route_loop_lands_both_routes_namespaced() {
        let root = tempfile::tempdir().unwrap();
        write_m2_root(root.path());

        let (_result, _events, diagnostics, out, _tmp) = executed(root.path(), "exp.m2_multihop");
        assert!(diagnostics.is_empty(), "clean route loop: {diagnostics:?}");

        let listing = |path: &Path| -> Vec<String> {
            let mut names: Vec<String> = std::fs::read_dir(path)
                .unwrap()
                .map(|e| e.unwrap().file_name().into_string().unwrap())
                .collect();
            names.sort();
            names
        };

        // The shared run out holds the event log, both route subtrees, and the
        // run-level tail artifacts the unified tails land at the bare root: the trace
        // bundle + lineage index, the report + its EN render, and the run + replay
        // manifests. No bare artifacts/ or groups/ at the root.
        assert_eq!(
            listing(&out),
            [
                "lineage_index.json",
                "logs",
                "manifest.json",
                "replay_manifest.json",
                "report.json",
                "report_en.md",
                "report_ja.md",
                "routes",
                "trace_bundle.json"
            ]
        );
        // The run trace binds both routes' parallel chains under one bundle; its three
        // §8.3 claims mint from the single_ir route's compiled groups (the direct route
        // lands no compiled artifact, so it mints no claim).
        let bundle = strict_at::<TraceBundle>(&out, "trace_bundle.json");
        assert_eq!(bundle.payload.claims.len(), 3, "run trace claims");
        // Both routes' chains reach the one run trace, so the tail ran over every
        // route rather than single_ir alone (which `claims.len()` alone would allow):
        // each route's nodes carry its route-prefixed path, and the direct route —
        // claim-less, since it lands no compiled artifact — still binds its verdicts.
        let node_paths: Vec<&str> = bundle
            .payload
            .nodes
            .iter()
            .map(|n| n.path.as_str())
            .collect();
        assert!(
            node_paths
                .iter()
                .any(|p| p.starts_with("routes/pipe.m2_single_ir/")),
            "run trace chains the single_ir route"
        );
        assert!(
            node_paths.iter().any(|p| {
                p.starts_with("routes/pipe.m2_direct_smt/groups/")
                    && p.ends_with("/verifier_results.json")
            }),
            "run trace binds the direct route's verifier_results"
        );

        // The run out holds exactly the two route subtrees.
        assert_eq!(
            listing(&out.join("routes")),
            ["pipe.m2_direct_smt", "pipe.m2_single_ir"]
        );

        // single_ir: three document heads, each carrying its ClinicalIR bundle beside
        // the extract + segment landings, and both groups' compiled + verifier_results.
        assert_eq!(
            listing(&out.join("routes/pipe.m2_single_ir")),
            ["artifacts", "groups"]
        );
        for doc in DOC_IDS {
            assert_eq!(
                listing(&out.join(format!("routes/pipe.m2_single_ir/artifacts/{doc}"))),
                [
                    "ir_bundle.json",
                    "segments.json",
                    "source_document_graph.json"
                ],
                "single_ir {doc}"
            );
        }
        for gid in ["group.m1_conflict", "group.m1_no_conflict"] {
            // `compile` lands the emitted SMT bodies under an `smt/` subdir beside the
            // compiled artifact, as in the M1 group layout.
            assert_eq!(
                listing(&out.join(format!("routes/pipe.m2_single_ir/groups/{gid}"))),
                ["compiled.json", "smt", "verifier_results.json"],
                "single_ir {gid}"
            );
        }

        // direct_smt: three document heads (no bundle — raw SMT) and both groups'
        // overlap/deontic smt_query pair plus verifier_results.
        assert_eq!(
            listing(&out.join("routes/pipe.m2_direct_smt")),
            ["artifacts", "groups"]
        );
        for doc in DOC_IDS {
            assert_eq!(
                listing(&out.join(format!("routes/pipe.m2_direct_smt/artifacts/{doc}"))),
                ["segments.json", "source_document_graph.json"],
                "direct {doc}"
            );
        }
        for gid in ["group.m1_conflict", "group.m1_no_conflict"] {
            assert_eq!(
                listing(&out.join(format!("routes/pipe.m2_direct_smt/groups/{gid}"))),
                [
                    "deontic.smt_query.json",
                    "overlap.smt_query.json",
                    "verifier_results.json"
                ],
                "direct {gid}"
            );
        }

        // The cross-route namespacing moved every group under its route: no bare dir.
        assert!(
            !out.join("groups").exists(),
            "no bare out/groups/ under the shared run out"
        );

        // run-m2.1e-A — the direct route's landed smt_query pair is replay-covered:
        // both wrapper content hashes sit in the run manifest's `output_hashes`, the
        // exact set replay re-derives and diffs. The pair lands whether or not
        // verification does; the clean run here verifies both.
        use ckc_core::RunManifest;
        let overlap = strict_at::<QueryBody>(
            &out,
            "routes/pipe.m2_direct_smt/groups/group.m1_conflict/overlap.smt_query.json",
        );
        let deontic = strict_at::<QueryBody>(
            &out,
            "routes/pipe.m2_direct_smt/groups/group.m1_conflict/deontic.smt_query.json",
        );
        let manifest: RunManifest =
            read_strict_canonical(&std::fs::read(out.join("manifest.json")).unwrap()).unwrap();
        assert!(
            manifest.output_hashes.contains(&overlap.content_hash),
            "manifest output_hashes cover the direct overlap smt_query"
        );
        assert!(
            manifest.output_hashes.contains(&deontic.content_hash),
            "manifest output_hashes cover the direct deontic smt_query"
        );

        // run-m2.1e-B2b — the run-level manifest carries the §9 measurement record (populated
        // because the run drives a model route, so `manifest_inputs` computes it rather than
        // gating to None): the agreed evaluator identity (the golden cassettes' synthetic
        // `model.baseline`), the four input hashes over THIS run's actual inputs — the
        // aggregated test sources, the route-relevant schema and prompt entries, and the raw
        // reference bytes — and model_hash/runtime_hash left None, since the runtime is an
        // environment bare-name command committing no bytes (identity rides `model_identity`).
        // Values blessed from the observed run.
        assert_eq!(
            manifest.model_identity,
            Some(ModelIdentity {
                model_id: static_id("model.baseline"),
                quant: "fixture_quant".to_owned(),
                runtime_version: "1.0.0".to_owned(),
            }),
            "run manifest §9 evaluator identity"
        );
        assert!(
            manifest.model_hash.is_none() && manifest.runtime_hash.is_none(),
            "the env bare-name runtime commits no model/runtime bytes"
        );
        assert_eq!(
            [
                manifest.test_source_hash.as_ref().map(|h| h.as_str()),
                manifest.reference_hash.as_ref().map(|h| h.as_str()),
                manifest.schema_hash.as_ref().map(|h| h.as_str()),
                manifest.prompt_template_hash.as_ref().map(|h| h.as_str()),
            ],
            [
                Some("sha256:52023a235277950b672288f1c196550139e2c1a8c32a1c559509ad35aba0d7f8"),
                Some("sha256:7192125b87593d1731795a1757c8f37f417061dc15603595f4a9178aeefee82f"),
                Some("sha256:c814fdbdef45361a17fba8e924190b57eda8a4bab91f58e61a215d9a201497c6"),
                Some("sha256:ad888a9c11b9b65094a5d2ff8c501cbe6345ca2f28b0ea323dfb9bb09614511f"),
            ],
            "run manifest §9 measurement hashes (test_source / reference / schema / prompt_template)"
        );

        // run-m2.1e-B2b — the replay manifest carries the identical §9 record: pin the full
        // seven-tuple equality so both records stay in lockstep without recomputing each hash.
        use ckc_core::ReplayManifest;
        let replay_manifest: ReplayManifest =
            read_strict_canonical(&std::fs::read(out.join("replay_manifest.json")).unwrap())
                .unwrap();
        assert_eq!(
            (
                &replay_manifest.model_identity,
                &replay_manifest.test_source_hash,
                &replay_manifest.reference_hash,
                &replay_manifest.schema_hash,
                &replay_manifest.prompt_template_hash,
                &replay_manifest.model_hash,
                &replay_manifest.runtime_hash,
            ),
            (
                &manifest.model_identity,
                &manifest.test_source_hash,
                &manifest.reference_hash,
                &manifest.schema_hash,
                &manifest.prompt_template_hash,
                &manifest.model_hash,
                &manifest.runtime_hash,
            ),
            "replay manifest §9 record matches the run manifest"
        );

        // run-m2.1e-A — the run-level trace/report tails carry a synthetic producer: the
        // baseline route's `pipeline_id` (`pipe.m2_direct_smt`, kept deliberately — the tails
        // span both routes) paired with a run-level step id, NOT the route's inert
        // UNUSED_STAGE slot. Both producer fields are content_hash-excluded, so a regression
        // back to UNUSED_STAGE (or drift in the carried pipeline_id) keeps every hash and the
        // layout stable and slips past the other pins — pin the full synthetic producer here.
        assert_eq!(
            bundle.producer.pipeline_id,
            static_id("pipe.m2_direct_smt"),
            "run trace tail carries the baseline route's pipeline_id"
        );
        assert_eq!(
            bundle.producer.pipeline_step_id,
            static_id(RUN_TRACE_STEP),
            "run trace tail mints the run-level trace step"
        );
        let lineage = strict_at::<LineageIndex>(&out, "lineage_index.json");
        assert_eq!(
            lineage.producer.pipeline_id,
            static_id("pipe.m2_direct_smt"),
            "run lineage tail carries the baseline route's pipeline_id"
        );
        assert_eq!(
            lineage.producer.pipeline_step_id,
            static_id(RUN_TRACE_STEP),
            "run lineage tail shares the run-level trace step"
        );
        let report = strict_at::<crate::report::Report>(&out, "report.json");
        assert_eq!(
            report.producer.pipeline_id,
            static_id("pipe.m2_direct_smt"),
            "run report tail carries the baseline route's pipeline_id"
        );
        assert_eq!(
            report.producer.pipeline_step_id,
            static_id(RUN_REPORT_STEP),
            "run report tail mints the run-level report step"
        );

        // run-m2.1e-C2 — the model route populates report.json's three M2 sections
        // (failure_taxonomy, metrics, model_identity) and both rendered bodies;
        // C1's `strict_at` typed-validity read left their VALUES unpinned.
        let payload = &report.payload;
        // (a) report.json section values. A clean replay raises no §7.4 rejection,
        // so each route's taxonomy code map is empty; both routes are still named
        // (a clean route is a present, empty-map route, Report::validate rule 5).
        let taxonomy_shape: Vec<(&str, usize)> = payload
            .failure_taxonomy
            .as_ref()
            .expect("model run populates failure_taxonomy")
            .routes
            .iter()
            .map(|(pid, codes)| (pid.as_str(), codes.len()))
            .collect();
        assert_eq!(
            taxonomy_shape,
            vec![("pipe.m2_direct_smt", 0), ("pipe.m2_single_ir", 0)],
            "clean run: both routes named with empty §7.4 code maps"
        );
        // metrics walk raw rows for both routes before the single baseline-delta
        // table (ExperimentMetrics::emission_order), baselined on the direct route.
        let metrics = payload
            .metrics
            .as_ref()
            .expect("model run populates metrics");
        assert_eq!(
            metrics.baseline_pipeline_id.as_str(),
            "pipe.m2_direct_smt",
            "direct route is the §7.3 delta baseline"
        );
        let order: Vec<(&str, bool)> = metrics
            .emission_order()
            .iter()
            .map(|section| match section {
                crate::metrics::MetricsSection::RawRows(r) => (r.pipeline_id.as_str(), false),
                crate::metrics::MetricsSection::DeltaTable(d) => (d.pipeline_id.as_str(), true),
            })
            .collect();
        assert_eq!(
            order,
            vec![
                ("pipe.m2_direct_smt", false),
                ("pipe.m2_single_ir", false),
                ("pipe.m2_single_ir", true),
            ],
            "raw rows for both routes precede the lone baseline-delta table"
        );
        // A single k=1 draw cannot converge, so every route reports
        // k_sample_convergence as not_applicable.
        for route in &metrics.routes {
            let k = route
                .rows
                .iter()
                .find(|row| row.metric.as_str() == crate::metrics::K_SAMPLE_CONVERGENCE)
                .expect("k_sample_convergence row present");
            assert_eq!(
                k.value,
                crate::metrics::MetricValue::NotApplicable,
                "single k=1 draw: k_sample_convergence not_applicable on {}",
                route.pipeline_id.as_str()
            );
        }
        // §9 evaluator identity is the golden cassettes' agreed synthetic identity.
        assert_eq!(
            payload.model_identity.as_ref().map(|m| (
                m.model_id.as_str(),
                m.quant.as_str(),
                m.runtime_version.as_str()
            )),
            Some(("model.baseline", "fixture_quant", "1.0.0")),
            "report §9 identity is the agreed evaluator identity"
        );

        // (b)/(c) full rendered bodies. The solver version is live-parsed from the
        // z3 binary's `--version` (env-dependent, so no live-run body is const-pinned) —
        // normalize that one token to `Z3_VERSION` and pin every other rendered
        // byte, guarding the M2 sections' rendering over this run's actual values.
        // Each body must carry the version exactly once, so the normalization is
        // unambiguous: a future version colliding with another rendered token (the
        // fixture model runtime_version `1.0.0`, a fraction) fails here, loud, not
        // by silently rewriting the collided token.
        assert_eq!(payload.solver_identity.solver_id, static_id("z3"));
        assert!(!payload.solver_identity.version.is_empty());
        let z3_version = payload.solver_identity.version.as_str();
        let report_en = std::fs::read_to_string(out.join("report_en.md")).unwrap();
        let report_ja = std::fs::read_to_string(out.join("report_ja.md")).unwrap();
        assert_eq!(
            report_en.matches(z3_version).count(),
            1,
            "solver version appears once in report_en.md"
        );
        assert_eq!(
            report_ja.matches(z3_version).count(),
            1,
            "solver version appears once in report_ja.md"
        );
        assert_eq!(
            report_en.replace(z3_version, "Z3_VERSION"),
            RUN_M2_REPORT_EN,
            "report_en.md renders the run's M2 sections"
        );
        assert_eq!(
            report_ja.replace(z3_version, "Z3_VERSION"),
            RUN_M2_REPORT_JA,
            "report_ja.md renders the run's M2 sections under JA labels"
        );
    }

    // run-m2.1e-C2 — the model route's two rendered bodies, blessed from an
    // observed `write_m2_root` run; the live z3 `--version` token is normalized
    // to `Z3_VERSION` (see the pin asserts) so the pins stay env-independent.
    const RUN_M2_REPORT_EN: &str = r#"# CKC report

wording: documented no-conflict result, synthetic test source measurement

## Corpus

| document | source hash |
| --- | --- |
| `test_source.m1_control` | `sha256:860dcd4a77c4412a251126e1097d1936ca11fe6b7ca8a72b67b1ac73a693b320` |
| `test_source.m1_guideline_a` | `sha256:dd6018d8daced58ab0ca55c313836ab9be15f572baac44f85d5ef2592cbd1ee8` |
| `test_source.m1_guideline_b` | `sha256:8789f89e86d6eb61612a6e113b3b02d351af5ea13f4ef7aa83c2527b9bed79ec` |

lexicon hash: `sha256:cc6b482aa3a1516ae9fc46d68e40a9a950799b184da4547e4bdf42f1f1da0159`

## Findings

### `finding.group.m1_conflict.1`

synthetic test source measurement; claim tier `s1_accepted`.

- conflict kind: `deontic_direction_conflict`
- query: `q.m1_conflict.pair1.deontic`, verdict `unsat`
- rules: `test_source.m1_guideline_a.rule.0`, `test_source.m1_guideline_b.rule.0`
- regions: `r.2`, `r.3`
- assertions: `a.test_source.m1_guideline_a.rule.0`, `a.test_source.m1_guideline_b.rule.0`
- core: `a.test_source.m1_guideline_a.rule.0`, `a.test_source.m1_guideline_b.rule.0`
- quoted spans:
  - `test_source.m1_guideline_a` `r.2` `s.2`: 成人(18歳以上)の敗血症患者には抗菌薬Aを投与することを推奨する(強い推奨)。
  - `test_source.m1_guideline_a` `r.3` `s.3`: ただし、重度腎機能障害のある患者を除く。
  - `test_source.m1_guideline_b` `r.2` `s.2`: 成人の敗血症患者のうち、妊娠中の患者には抗菌薬Aを投与しないこと(禁忌)。

## Documented no-conflict results

### `finding.group.m1_no_conflict.0`

documented no-conflict result; claim tier `s1_accepted`.

- query: `q.m1_no_conflict.pair1.overlap`, verdict `unsat`
- rules: `test_source.m1_control.rule.0`, `test_source.m1_guideline_a.rule.0`
- regions: `r.2`, `r.3`
- assertions: `ctx.test_source.m1_control.rule.0`, `ctx.test_source.m1_guideline_a.rule.0`
- quoted spans:
  - `test_source.m1_control` `r.2` `s.2`: 小児(18歳未満)の敗血症患者には抗菌薬Aは禁忌である。
  - `test_source.m1_guideline_a` `r.2` `s.2`: 成人(18歳以上)の敗血症患者には抗菌薬Aを投与することを推奨する(強い推奨)。
  - `test_source.m1_guideline_a` `r.3` `s.3`: ただし、重度腎機能障害のある患者を除く。

## Diagnostics summary

none.

## Failure taxonomy

### `pipe.m2_direct_smt`

none.

### `pipe.m2_single_ir`

none.

## Metrics

raw benchmark output (locked measurement); raw rows precede every baseline-delta table. baseline: `pipe.m2_direct_smt`.

### Raw rows: `pipe.m2_direct_smt`

| metric | value |
| --- | --- |
| `acceptance_rate` | 1/1 |
| `conflict_verdict_accuracy` | 1/1 |
| `k_sample_convergence` | not_applicable |
| `recorded_call_count` | 4/1 |
| `repair_count` | 0/1 |
| `schema_valid_rate` | 1/1 |
| `target_syntactic_validity` | 1/1 |

### Raw rows: `pipe.m2_single_ir`

| metric | value |
| --- | --- |
| `acceptance_rate` | 1/1 |
| `conflict_verdict_accuracy` | 1/1 |
| `k_sample_convergence` | not_applicable |
| `recorded_call_count` | 3/1 |
| `repair_count` | 0/1 |
| `schema_valid_rate` | 1/1 |
| `target_syntactic_validity` | 1/1 |

### Baseline delta: `pipe.m2_single_ir` - `pipe.m2_direct_smt`

| metric | value |
| --- | --- |
| `acceptance_rate` | 0/1 |
| `conflict_verdict_accuracy` | 0/1 |
| `k_sample_convergence` | not_applicable |
| `recorded_call_count` | -1/1 |
| `repair_count` | 0/1 |
| `schema_valid_rate` | 0/1 |
| `target_syntactic_validity` | 0/1 |

## Solver identity

`z3` version `Z3_VERSION`

## Model identity

`model.baseline` quant `fixture_quant` runtime version `1.0.0`

## Replay status

`not_replayed`
"#;

    const RUN_M2_REPORT_JA: &str = r#"# CKC レポート

語彙: documented no-conflict result、synthetic test source measurement

## コーパス

| 文書 | ソースハッシュ |
| --- | --- |
| `test_source.m1_control` | `sha256:860dcd4a77c4412a251126e1097d1936ca11fe6b7ca8a72b67b1ac73a693b320` |
| `test_source.m1_guideline_a` | `sha256:dd6018d8daced58ab0ca55c313836ab9be15f572baac44f85d5ef2592cbd1ee8` |
| `test_source.m1_guideline_b` | `sha256:8789f89e86d6eb61612a6e113b3b02d351af5ea13f4ef7aa83c2527b9bed79ec` |

レキシコンハッシュ: `sha256:cc6b482aa3a1516ae9fc46d68e40a9a950799b184da4547e4bdf42f1f1da0159`

## 所見

### `finding.group.m1_conflict.1`

synthetic test source measurement。主張階層 `s1_accepted`。

- 矛盾種別: `deontic_direction_conflict`
- クエリ: `q.m1_conflict.pair1.deontic`、判定 `unsat`
- 規則: `test_source.m1_guideline_a.rule.0`、`test_source.m1_guideline_b.rule.0`
- 領域: `r.2`、`r.3`
- アサーション: `a.test_source.m1_guideline_a.rule.0`、`a.test_source.m1_guideline_b.rule.0`
- コア: `a.test_source.m1_guideline_a.rule.0`、`a.test_source.m1_guideline_b.rule.0`
- 引用スパン:
  - `test_source.m1_guideline_a` `r.2` `s.2`: 成人(18歳以上)の敗血症患者には抗菌薬Aを投与することを推奨する(強い推奨)。
  - `test_source.m1_guideline_a` `r.3` `s.3`: ただし、重度腎機能障害のある患者を除く。
  - `test_source.m1_guideline_b` `r.2` `s.2`: 成人の敗血症患者のうち、妊娠中の患者には抗菌薬Aを投与しないこと(禁忌)。

## 文書化された無矛盾結果

### `finding.group.m1_no_conflict.0`

documented no-conflict result。主張階層 `s1_accepted`。

- クエリ: `q.m1_no_conflict.pair1.overlap`、判定 `unsat`
- 規則: `test_source.m1_control.rule.0`、`test_source.m1_guideline_a.rule.0`
- 領域: `r.2`、`r.3`
- アサーション: `ctx.test_source.m1_control.rule.0`、`ctx.test_source.m1_guideline_a.rule.0`
- 引用スパン:
  - `test_source.m1_control` `r.2` `s.2`: 小児(18歳未満)の敗血症患者には抗菌薬Aは禁忌である。
  - `test_source.m1_guideline_a` `r.2` `s.2`: 成人(18歳以上)の敗血症患者には抗菌薬Aを投与することを推奨する(強い推奨)。
  - `test_source.m1_guideline_a` `r.3` `s.3`: ただし、重度腎機能障害のある患者を除く。

## 診断サマリ

なし。

## 失敗分類

### `pipe.m2_direct_smt`

なし。

### `pipe.m2_single_ir`

なし。

## 指標

raw benchmark output(locked measurement)。生の指標行はすべてのベースライン差分表に先行する。ベースライン: `pipe.m2_direct_smt`。

### 生の指標行: `pipe.m2_direct_smt`

| 指標 | 値 |
| --- | --- |
| `acceptance_rate` | 1/1 |
| `conflict_verdict_accuracy` | 1/1 |
| `k_sample_convergence` | not_applicable |
| `recorded_call_count` | 4/1 |
| `repair_count` | 0/1 |
| `schema_valid_rate` | 1/1 |
| `target_syntactic_validity` | 1/1 |

### 生の指標行: `pipe.m2_single_ir`

| 指標 | 値 |
| --- | --- |
| `acceptance_rate` | 1/1 |
| `conflict_verdict_accuracy` | 1/1 |
| `k_sample_convergence` | not_applicable |
| `recorded_call_count` | 3/1 |
| `repair_count` | 0/1 |
| `schema_valid_rate` | 1/1 |
| `target_syntactic_validity` | 1/1 |

### ベースライン差分: `pipe.m2_single_ir` - `pipe.m2_direct_smt`

| 指標 | 値 |
| --- | --- |
| `acceptance_rate` | 0/1 |
| `conflict_verdict_accuracy` | 0/1 |
| `k_sample_convergence` | not_applicable |
| `recorded_call_count` | -1/1 |
| `repair_count` | 0/1 |
| `schema_valid_rate` | 0/1 |
| `target_syntactic_validity` | 0/1 |

## ソルバー識別情報

`z3` バージョン `Z3_VERSION`

## モデル識別情報

`model.baseline` 量子化 `fixture_quant` ランタイムバージョン `1.0.0`

## リプレイ状態

`not_replayed`
"#;

    // run-m2.1e-A — a partial direct fill (one role lands, the other role's cassette is
    // absent) still reaches the run manifest, so replay covers the lone landed role.
    // Dropping `group.m1_conflict.deontic` lands its overlap, then the absent deontic
    // raises a mid-pair CassetteError; the §4.4 valid-remainder run lands its provenance
    // pair over the recorded state despite that event-scoped failure.
    #[test]
    fn m2_direct_partial_landing_is_replay_covered() {
        let root = tempfile::tempdir().unwrap();
        write_m2_root(root.path());
        std::fs::remove_file(
            root.path()
                .join("cassettes/route.direct_smt/group.m1_conflict.deontic/seed-42.json"),
        )
        .unwrap();

        let (_result, _events, _diagnostics, out, _tmp) = executed(root.path(), "exp.m2_multihop");

        // The partial group landed its overlap alone: no deontic role, no verdict.
        let gdir = out.join("routes/pipe.m2_direct_smt/groups/group.m1_conflict");
        assert!(
            gdir.join("overlap.smt_query.json").exists(),
            "the overlap role landed"
        );
        assert!(
            !gdir.join("deontic.smt_query.json").exists(),
            "the deontic role never landed"
        );
        assert!(
            !gdir.join("verifier_results.json").exists(),
            "a lone role cannot verify"
        );

        // The lone landed overlap's content hash sits in the manifest's output_hashes,
        // the set replay re-derives — so a partial landing stays replay-covered.
        use ckc_core::RunManifest;
        let overlap = strict_at::<QueryBody>(
            &out,
            "routes/pipe.m2_direct_smt/groups/group.m1_conflict/overlap.smt_query.json",
        );
        let manifest: RunManifest =
            read_strict_canonical(&std::fs::read(out.join("manifest.json")).unwrap()).unwrap();
        assert!(
            manifest.output_hashes.contains(&overlap.content_hash),
            "manifest output_hashes cover the lone landed overlap"
        );
    }

    // run-m2.1d5b — two-run determinism over the model-route run. The same
    // locked inputs (`exp.m2_multihop`, the replayed golden cassettes) execute
    // twice into two out dirs — `executed` fixes run_id, so the two runs differ
    // only in their out-path prefix: every landed artifact is byte-equal across
    // the runs, the §5/§4.6 provenance manifests agree once the run-specific out
    // path is normalized wherever it appears (`land_record` writes plain
    // canonical records — no self-hash — so `manifest_inputs.command`'s embedded
    // `out_dir.display()` is the manifests' sole non-deterministic bytes; the
    // tempdir path holds no char the canonical string encoder escapes, so a raw
    // substring replace matches its on-disk bytes), and the event stream agrees
    // on its non-timing projection (only the §4.6 wall-clock fields
    // started_at/ended_at/duration_ms may differ run to run).
    #[test]
    fn m2_route_run_is_deterministic_across_two_runs() {
        let root = tempfile::tempdir().unwrap();
        write_m2_root(root.path());
        let (_, events_a, diag_a, out_a, _tmp_a) = executed(root.path(), "exp.m2_multihop");
        let (_, events_b, diag_b, out_b, _tmp_b) = executed(root.path(), "exp.m2_multihop");
        assert!(
            diag_a.is_empty() && diag_b.is_empty(),
            "clean run both times"
        );

        // Both runs land the identical run-relative file set.
        fn walk(base: &Path, dir: &Path, acc: &mut Vec<String>) {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect();
            entries.sort();
            for p in entries {
                if p.is_dir() {
                    walk(base, &p, acc);
                } else {
                    acc.push(p.strip_prefix(base).unwrap().to_string_lossy().into_owned());
                }
            }
        }
        let files = |out: &Path| {
            let mut acc = Vec::new();
            walk(out, out, &mut acc);
            acc.sort();
            acc
        };
        let paths = files(&out_a);
        assert_eq!(paths, files(&out_b), "both runs land the same file set");

        // The walk reached the run-level tails and both route subtrees, so the
        // byte-equal / manifest-normalize / events arms below run non-vacuously.
        for want in [
            "trace_bundle.json",
            "manifest.json",
            "replay_manifest.json",
            "logs/events.jsonl",
        ] {
            assert!(paths.iter().any(|p| p == want), "walk covers {want}");
        }
        for route in ["routes/pipe.m2_single_ir/", "routes/pipe.m2_direct_smt/"] {
            assert!(
                paths.iter().any(|p| p.starts_with(route)),
                "walk covers {route}"
            );
        }

        // Normalize the run-specific out path (wherever it appears) before
        // comparing the two manifests.
        let norm = |bytes: Vec<u8>, out: &Path| -> Vec<u8> {
            String::from_utf8(bytes)
                .unwrap()
                .replace(&out.display().to_string(), "<OUT>")
                .into_bytes()
        };
        for rel in &paths {
            let a = std::fs::read(out_a.join(rel)).unwrap();
            let b = std::fs::read(out_b.join(rel)).unwrap();
            match rel.as_str() {
                // The §4.6 wall-clock fields ride here; compared as a projection below.
                "logs/events.jsonl" => {}
                "manifest.json" | "replay_manifest.json" => {
                    assert_eq!(norm(a, &out_a), norm(b, &out_b), "{rel} modulo --out");
                }
                _ => assert_eq!(a, b, "{rel} byte-equal across runs"),
            }
        }

        // Events agree on every field but the three §4.6 wall-clock fields
        // (event_id/event_sequence_number are slot-derived, so deterministic).
        let untimed = |e: &EventRecord| -> EventRecord {
            let mut e = e.clone();
            e.started_at.clear();
            e.ended_at.clear();
            e.duration_ms = 0;
            e
        };
        let proj_a: Vec<EventRecord> = events_a.iter().map(untimed).collect();
        let proj_b: Vec<EventRecord> = events_b.iter().map(untimed).collect();
        assert_eq!(
            proj_a, proj_b,
            "events agree on their non-timing projection"
        );
    }

    // run-m2.1d5b — the model-route run's §4.6 event census. Both routes execute
    // under one shared run out: single_ir heads three documents through four
    // processing_stages each (extract/segment/model_fill/assemble) and
    // compiles+verifies two groups (3×4 + 2×2 = 16); direct heads three documents
    // through two (extract/segment) and fills+verifies two groups (3×2 + 2×2 = 10);
    // the run command closes with one event. The run-level trace/report tails run
    // with `emit_event` false, so they contribute no census event — 27 total. The
    // M1 baseline is a separate single-route run with its own 19-event census.
    #[test]
    fn m2_route_run_event_census() {
        let root = tempfile::tempdir().unwrap();
        write_m2_root(root.path());
        let (_, events, diagnostics, _out, _tmp) = executed(root.path(), "exp.m2_multihop");
        assert!(diagnostics.is_empty(), "clean run");

        assert_eq!(events.len(), 27, "M2 two-route census");
        let by_pipeline = |pid: &str| {
            events
                .iter()
                .filter(|e| e.pipeline_id == static_id(pid))
                .count()
        };
        assert_eq!(by_pipeline("pipe.m2_single_ir"), 16, "single_ir 3×4 + 2×2");
        assert_eq!(by_pipeline("pipe.m2_direct_smt"), 10, "direct 3×2 + 2×2");
        assert_eq!(by_pipeline("cli"), 1, "one run command event");

        // The run-level tails declare no census event under the route pipelines'
        // padded UNUSED_STAGE slots.
        assert_eq!(
            events
                .iter()
                .filter(
                    |e| e.processing_stage == static_id(PROCESSING_STAGE_KINDS[TRACE])
                        || e.processing_stage == static_id(PROCESSING_STAGE_KINDS[REPORT])
                )
                .count(),
            0,
            "run-level tails emit no census event",
        );

        let fills = |pid: &str, step: &str| -> Vec<&EventRecord> {
            events
                .iter()
                .filter(|e| {
                    e.pipeline_id == static_id(pid) && e.pipeline_step_id == static_id(step)
                })
                .collect()
        };

        // Per-stage census decomposition: each declared stage emits one event per
        // unit it heads — documents for extract/segment/model_fill(_smt), groups
        // for assemble/compile/verify(_smt). The 16 and 10 pipeline products above
        // hold only if these per-stage factors do; a regression trading one stage's
        // events for another's could preserve a pipeline total but not this table.
        for (step, n) in [
            ("processing_stage.m1.extract", 3),
            ("processing_stage.m1.segment", 3),
            ("processing_stage.m2.model_fill", 3),
            ("processing_stage.m2.assemble", 3),
            ("processing_stage.m1.compile", 2),
            ("processing_stage.m1.verify", 2),
        ] {
            assert_eq!(
                fills("pipe.m2_single_ir", step).len(),
                n,
                "single_ir stage {step}"
            );
        }
        for (step, n) in [
            ("processing_stage.m1.extract", 3),
            ("processing_stage.m1.segment", 3),
            ("processing_stage.m2.model_fill_smt", 2),
            ("processing_stage.m2.verify_smt", 2),
        ] {
            assert_eq!(
                fills("pipe.m2_direct_smt", step).len(),
                n,
                "direct stage {step}"
            );
        }

        // model_fill resource_counters (the .1d3a/.1d4a contract): single_ir draws
        // one sample per document (1/0); direct draws two per group, one per
        // overlap/deontic role summed onto the one group event (2/0).
        for e in fills("pipe.m2_single_ir", "processing_stage.m2.model_fill") {
            assert_eq!(
                e.resource_counters,
                vec![
                    (static_id(RECORDED_CALLS_COUNTER), 1),
                    (static_id(REPAIRS_COUNTER), 0)
                ],
            );
        }
        for e in fills("pipe.m2_direct_smt", "processing_stage.m2.model_fill_smt") {
            assert_eq!(
                e.resource_counters,
                vec![
                    (static_id(RECORDED_CALLS_COUNTER), 2),
                    (static_id(REPAIRS_COUNTER), 0)
                ],
            );
        }

        // The M1 baseline is a separate single-route run whose 19-event census
        // stands unchanged: its execute() body emits every stage event, the
        // trace/report tails included, unlike the suppressed M2 tails.
        let (_, m1_events, m1_diag, _, _m1_tmp) = executed(&repo_root(), "exp.m1_scaffold");
        assert!(m1_diag.is_empty(), "M1 baseline runs clean");
        assert_eq!(m1_events.len(), 19, "M1 baseline census unchanged");
    }

    // run-m2.1d5a-2b — error-path battery over the .1d5a-1 model-route loop's
    // already-landed branches (independent of .1d5a-2's tails wiring).

    // A single_ir document whose model_fill reads no cassette takes a command-scope
    // read diagnostic, leaving its bundle absent; every group holding that member then
    // shorts its compile processing_stage rather than compiling a partial group. The
    // two diagnostics co-occur — a compile short is always preceded by its member's own
    // upstream fill failure. Dropping guideline_b's cassette shorts exactly
    // group.m1_conflict (its sole group); m1_no_conflict (guideline_a + control) still
    // fills and compiles clean, and the direct route — cassettes keyed by group, not
    // source — is untouched, so the run-level tails complete over what landed.
    #[test]
    fn m2_single_ir_member_short_group_co_emits_fill_and_compile_diagnostics() {
        let root = tempfile::tempdir().unwrap();
        write_m2_root(root.path());
        std::fs::remove_file(
            root.path()
                .join("cassettes/route.single_ir/test_source.m1_guideline_b/seed-42.json"),
        )
        .unwrap();

        let (result, events, diagnostics, out, _tmp) = executed(root.path(), "exp.m2_multihop");
        assert_eq!(result.outcome, Outcome::Invalid);

        // The fill loop runs before the group loop, so the member's model_fill read
        // failure is diagnosed first, then the group's partial-group compile short.
        assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
        let fill = &diagnostics[0];
        assert!(
            fill.payload
                .iter()
                .any(|(k, v)| k.as_str() == "cassette" && v == "test_source.m1_guideline_b"),
            "{fill:#?}"
        );
        assert!(
            fill.payload
                .iter()
                .any(|(k, v)| k.as_str() == "processing_stage" && v == "model_fill"),
            "{fill:#?}"
        );
        let compile = &diagnostics[1];
        assert!(
            compile
                .payload
                .iter()
                .any(|(k, v)| k.as_str() == "group" && v == "group.m1_conflict"),
            "{compile:#?}"
        );
        assert!(
            compile
                .payload
                .iter()
                .any(|(k, v)| k.as_str() == "processing_stage" && v == "compile"),
            "{compile:#?}"
        );
        assert!(
            compile
                .payload
                .iter()
                .any(|(k, v)| k.as_str() == "reason" && v.contains("landed no ir_bundle artifact")),
            "{compile:#?}"
        );

        // The fill failure is command-scope: guideline_b's cassette-read diagnostic rides
        // the closing command event (the last event `finish` appends, which carries the
        // command-scope diagnostics), not a model_fill stage event — single_ir_fill raises
        // it directly and returns before the fill event. A regression turning it into a
        // stage outcome would move it off the command event onto the fill's own event.
        let command_event = events
            .last()
            .expect("the run appends a closing command event");
        assert!(
            command_event.diagnostics.iter().any(|d| {
                d.payload
                    .iter()
                    .any(|(k, v)| k.as_str() == "cassette" && v == "test_source.m1_guideline_b")
            }),
            "the cassette-read failure rides the command event"
        );

        // The compile short rides its §4.6 event too: exactly one Invalid `compile`
        // event — the member-short group's — carrying exactly that diagnostic and landing
        // no artifact (m1_no_conflict compiles clean; the direct route runs no compile
        // processing_stage), so a second shorted group would break this count.
        let invalid_compiles: Vec<_> = events
            .iter()
            .filter(|e| e.processing_stage == static_id("compile") && e.outcome == Outcome::Invalid)
            .collect();
        assert_eq!(
            invalid_compiles.len(),
            1,
            "exactly one group shorts its compile"
        );
        let compile_event = invalid_compiles[0];
        assert_eq!(compile_event.diagnostics.len(), 1);
        assert!(compile_event.output_hashes.is_empty());
        assert!(
            compile_event.diagnostics[0]
                .payload
                .iter()
                .any(|(k, v)| k.as_str() == "group" && v == "group.m1_conflict"),
            "{compile_event:#?}"
        );

        // The run recovers past the short group: the run-level tails run to completion
        // over what landed (the clean m1_no_conflict group + both routes' docs, the
        // bundle-less guideline_b doc included), landing the run-root trace + report. A
        // regression returning on the short instead of continuing would leave these
        // absent.
        assert!(
            out.join("trace_bundle.json").exists(),
            "the tails land the run trace"
        );
        assert!(
            out.join("report.json").exists(),
            "the tails land the run report"
        );
    }

    // A route set mixing the layered M1 pipeline with a model route has no defined
    // joint execution: execute()'s dispatch fails it closed with one command diagnostic
    // and lands nothing (only the shell's logs/). Craft the mix off the committed
    // exp.m2_multihop binding — swap the direct route for the layered M1 pipeline in the
    // set and re-point the baseline to the surviving in-set model route, so the binding
    // stays valid (baseline ∈ set) and resolution reaches the shape dispatch.
    #[test]
    fn m2_mixed_shape_route_set_fails_closed_landing_nothing() {
        let root = tempfile::tempdir().unwrap();
        write_m2_root(root.path());
        let experiments = root.path().join("registry/experiments.yaml");
        let mixed = std::fs::read_to_string(&experiments)
            .unwrap()
            .replace(
                "pipe.m2_direct_smt, pipe.m2_single_ir",
                "pipe.layered_ckcir_to_smt, pipe.m2_single_ir",
            )
            .replace(
                "baseline_pipeline: pipe.m2_direct_smt",
                "baseline_pipeline: pipe.m2_single_ir",
            );
        std::fs::write(&experiments, mixed).unwrap();

        let (result, _events, diagnostics, out, _tmp) = executed(root.path(), "exp.m2_multihop");
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert!(
            diagnostics[0]
                .payload
                .iter()
                .any(|(_, v)| v.contains("mixes the layered M1 pipeline with model routes")),
            "{diagnostics:#?}"
        );
        assert_only_logs(&out);
    }

    // run-m2.1e-B2a codex follow-up — the §9 route-hash selection is coverage-checked:
    // a wanted registry id resolving to no entry fails loudly (naming the gap) rather
    // than silently locking `aggregate_hashes(Vec::new())`'s empty-set hash into an
    // attestation record. Guards run.rs's hardcoded route→id map drifting from the
    // committed registry independently of the fill path — a registry rename breaks the
    // fill first, but a hardcoded-id typo would not, so the manifest needs its own guard.
    #[test]
    fn select_route_hashes_fails_on_unresolved_want_id() {
        let clinical = static_id("schema.clinical_ir");
        let smt = static_id("schema.smt_query");
        let h1 = hash_bytes(b"clinical-schema-bytes");
        let entries = [(clinical.clone(), h1.clone())];

        // A fully covered want-set aggregates over exactly the wanted hash.
        let covered: BTreeSet<Id> = [clinical.clone()].into_iter().collect();
        assert_eq!(
            select_route_hashes(&covered, entries.iter().map(|(i, h)| (i, h)), "schema").unwrap(),
            aggregate_hashes(vec![h1.as_str().to_owned()]),
        );

        // Route-relevance, not whole-registry: an entry the want-set omits is dropped,
        // never folded into the aggregate — so a registry that later grows an unused entry
        // cannot silently rewrite this run's §9 attestation hash. Dropping the
        // `want.contains` filter (hashing the whole registry) is caught here alone: the
        // covered and missing cases both pass without it, since neither feeds an extra entry.
        let smt_extra = hash_bytes(b"smt-schema-bytes");
        let with_extra = [(clinical.clone(), h1.clone()), (smt.clone(), smt_extra)];
        assert_eq!(
            select_route_hashes(&covered, with_extra.iter().map(|(i, h)| (i, h)), "schema")
                .unwrap(),
            aggregate_hashes(vec![h1.as_str().to_owned()]),
        );

        // A wanted id no entry supplies fails, naming the gap — never the empty-set lock.
        let short: BTreeSet<Id> = [clinical, smt.clone()].into_iter().collect();
        let err =
            select_route_hashes(&short, entries.iter().map(|(i, h)| (i, h)), "schema").unwrap_err();
        assert!(
            err.contains("missing route id(s)") && err.contains(smt.as_str()),
            "unexpected error: {err}"
        );
    }

    // Both routes must attest one evaluator identity: a second, disagreeing model
    // identity fails the run closed with one command diagnostic. Re-bless guideline_a's
    // single_ir cassette with a divergent synthetic identity (the crafted-fixture rule —
    // no real engine/quant/format token). The direct route runs first and establishes
    // the agreed identity from its four golden cassettes, so single_ir's guideline_a fill
    // is the divergent second attestation, tripping the fail-closed return.
    #[test]
    fn m2_model_identity_disagreement_fails_the_run_closed() {
        let root = tempfile::tempdir().unwrap();
        write_m2_root(root.path());
        let store = CassetteStore::new(root.path());
        let key = CassetteKey {
            route: static_id("route.single_ir"),
            source: static_id("test_source.m1_guideline_a"),
            seed: 42,
        };
        // Keep the recorded output (still accepts + grounds); swap only the identity, so
        // the fill succeeds and the disagreement — not a fill failure — is the sole event.
        let mut payload = store.replay(&key).unwrap().payload;
        payload.model_identity = ModelIdentity {
            model_id: static_id("model.other"),
            quant: "fixture_quant".to_owned(),
            runtime_version: "1.0.0".to_owned(),
        };
        let wrapper = store
            .build_wrapper(&key, payload, producer(&single_ir_resolved(), 2))
            .unwrap();
        store.persist(&key, wrapper).unwrap();

        let (result, _events, diagnostics, out, _tmp) = executed(root.path(), "exp.m2_multihop");
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert!(
            diagnostics[0]
                .payload
                .iter()
                .any(|(_, v)| v.contains("model routes disagree on the model identity")),
            "{diagnostics:#?}"
        );

        // Fail-closed: the disagreement returns from the route loop before the run-level
        // tails, so no run-root tail artifact lands — the run out holds only `logs` and
        // `routes`. Asserting outcome + message alone would pass even if execution
        // wrongly continued into the tails; the absent tail set pins the stop.
        let mut root_entries: Vec<String> = std::fs::read_dir(&out)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        root_entries.sort();
        assert_eq!(root_entries, ["logs", "routes"], "no run-level tail landed");
        // The direct route (first in the set) had already run to completion, establishing
        // the agreed identity, so its subtree is fully landed; single_ir's guideline_a
        // fill is the divergent second attestation. A flipped set order would run
        // single_ir first and leave the direct subtree absent, breaking this.
        assert!(
            out.join("routes/pipe.m2_direct_smt/groups/group.m1_no_conflict/verifier_results.json")
                .exists(),
            "the direct route attests first, landing its subtree before the stop"
        );
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
    /// `[6]`/`[7]` pad with the inert [`UNUSED_STAGE`] sentinel the resolver mints — the
    /// verdict tail never reads them. `documents` / `groups` / `plan` go unread so stay
    /// empty and the toolchain hash stays synthetic — the bundle `content_hash` gate is
    /// payload-only; `budget_ms` is exp.m1_scaffold's §8.4 `solver_ms_per_query`, the
    /// verdict tail's z3 cap.
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
                static_id(UNUSED_STAGE),
                static_id(UNUSED_STAGE),
            ],
            shape: RouteShape::SingleIr,
            documents: vec![],
            groups: vec![],
            budget_ms: 10_000,
            repair_limit: Some(1),
            model_ms_per_call: Some(600_000),
            is_baseline: false,
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

    /// [`single_ir_resolved`]'s hardcoded step ids + shape stay bound to the
    /// committed `pipe.m2_single_ir` declaration through the production
    /// [`resolve_route`] — registry drift (either side) breaks here, never
    /// silently. The reproduce-M1 event battery pins the same ids as
    /// literals, closing the chain events → fixture → registry.
    #[test]
    fn single_ir_resolved_matches_committed_registry() {
        let text = std::fs::read_to_string(repo_root().join("registry/candidates.yaml")).unwrap();
        let candidates = parse_candidates(&text).unwrap();
        let pipeline = candidates
            .pipelines
            .iter()
            .find(|p| p.id == static_id("pipe.m2_single_ir"))
            .expect("registry/candidates.yaml declares pipe.m2_single_ir");
        let mut shell = Shell::open(static_id("run"), static_id("m2"), None);
        let (step_ids, shape) =
            resolve_route(pipeline, &candidates, &mut shell).expect("route resolves");
        let fixture = single_ir_resolved();
        assert_eq!(step_ids, fixture.pipeline_step_ids);
        assert_eq!(shape, fixture.shape);
    }

    /// The reproduce-M1 gate: for each M1 document, [`single_ir_fill`] replaying the
    /// committed golden cassette compiles a bundle byte-identical (by the payload-only
    /// `content_hash`) to the M1 deterministic [`assemble_bundle`] bundle.
    /// Runtime-absent — the cassette IS the recorded model output. Structural equality
    /// is asserted too (the clearer failure should the route's deterministic tail ever
    /// diverge).
    #[test]
    fn single_ir_fill_reproduces_m1_bundles() {
        use std::collections::BTreeSet;

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
            let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out.clone()));
            let head = route_document_head(&root, &entry, &resolved, &mut shell)
                .unwrap_or_else(|| panic!("{}: no deterministic route head", entry.id));
            let route_doc =
                single_ir_fill(head, &lexicon, &store, 42, &resolved, 0, None, &mut shell);

            // §7.3 telemetry + §9 identity ride the RouteDoc whole: one clean
            // recorded call, and the goldens' synthetic identity.
            assert_eq!(
                route_doc.fill,
                Some(FillObservation {
                    accepted: true,
                    recorded_calls: 1,
                    repairs: 0,
                    schema_violations: 0,
                }),
                "{} fill observation",
                entry.id
            );
            assert_eq!(
                route_doc.identity,
                Some(ModelIdentity {
                    model_id: static_id("model.baseline"),
                    quant: "fixture_quant".to_string(),
                    runtime_version: "1.0.0".to_string(),
                }),
                "{} identity",
                entry.id
            );
            let route = route_doc
                .trace
                .bundle
                .unwrap_or_else(|| panic!("{}: single_ir_fill yielded no bundle", entry.id));

            assert_eq!(route.payload, m1.payload, "{} payload", entry.id);
            assert_eq!(
                route.content_hash, m1.content_hash,
                "{} content_hash",
                entry.id
            );

            // Wrapper provenance: source + segments + the replayed cassette (§9
            // attestation) — the payload-only `content_hash` cannot catch a wrong
            // or missing input hash. input_hashes canonicalize as a §4.3 set, so
            // compare as a set, never the emitted order.
            let cassette = store
                .replay(&CassetteKey {
                    route: static_id("route.single_ir"),
                    source: entry.id.clone(),
                    seed: 42,
                })
                .unwrap();
            assert_eq!(route.input_hashes.len(), 3, "{} input count", entry.id);
            assert_eq!(
                route.input_hashes.iter().collect::<BTreeSet<_>>(),
                [
                    &source.content_hash,
                    &segments.content_hash,
                    &cassette.content_hash
                ]
                .into_iter()
                .collect::<BTreeSet<_>>(),
                "{} input_hashes",
                entry.id
            );

            // Close the shell and pin the LANDED §4.6 stream + layout
            // (run-m2.1d3b): every directory level listed exact down to the
            // three route artifacts, each strict-read clean under a
            // route-prefixed artifact id.
            let finished = shell.finish().unwrap();
            assert_eq!(finished.result.outcome, Outcome::Ok, "{}", entry.id);
            assert!(finished.result.diagnostic_hashes.is_empty(), "{}", entry.id);
            let doc_dir = format!("routes/pipe.m2_single_ir/artifacts/{}", entry.id);
            let listing = |path: &Path| -> Vec<String> {
                let mut names: Vec<String> = std::fs::read_dir(path)
                    .unwrap()
                    .map(|e| e.unwrap().file_name().into_string().unwrap())
                    .collect();
                names.sort();
                names
            };
            assert_eq!(listing(&out), ["logs", "routes"], "{}", entry.id);
            assert_eq!(
                listing(&out.join("logs")),
                ["diagnostics.jsonl", "events.jsonl"],
                "{}",
                entry.id
            );
            assert_eq!(
                listing(&out.join("routes")),
                ["pipe.m2_single_ir"],
                "{}",
                entry.id
            );
            assert_eq!(
                listing(&out.join("routes/pipe.m2_single_ir")),
                ["artifacts"],
                "{}",
                entry.id
            );
            assert_eq!(
                listing(&out.join("routes/pipe.m2_single_ir/artifacts")),
                [entry.id.to_string()],
                "{}",
                entry.id
            );
            assert_eq!(
                listing(&out.join(&doc_dir)),
                [
                    "ir_bundle.json",
                    "segments.json",
                    "source_document_graph.json"
                ],
                "{}",
                entry.id
            );
            let landed_source: ArtifactWrapper<SourceDocumentGraph> =
                strict_at(&out, &format!("{doc_dir}/source_document_graph.json"));
            let landed_segments: ArtifactWrapper<SegmentIr> =
                strict_at(&out, &format!("{doc_dir}/segments.json"));
            let landed_bundle: ArtifactWrapper<IrBundle> =
                strict_at(&out, &format!("{doc_dir}/ir_bundle.json"));
            assert_eq!(
                landed_source.artifact_id,
                format!("pipe.m2_single_ir.{}.source_document_graph", entry.id)
                    .parse()
                    .unwrap()
            );
            assert_eq!(
                landed_segments.artifact_id,
                format!("pipe.m2_single_ir.{}.segments", entry.id)
                    .parse()
                    .unwrap()
            );
            assert_eq!(
                landed_bundle.artifact_id,
                format!("pipe.m2_single_ir.{}.ir_bundle", entry.id)
                    .parse()
                    .unwrap()
            );
            assert_eq!(
                landed_bundle.content_hash, route.content_hash,
                "{}",
                entry.id
            );

            // The §4.6 event battery: four stage events (extract, segment,
            // model_fill, assemble) then the closing command event, hashes
            // cross-checked against the landed wrappers. Read-back input and
            // output hashes canonicalize as §4.3 sets, so multi-input slots
            // compare as sets.
            let events: Vec<EventRecord> =
                read_jsonl(&std::fs::read(out.join("logs/events.jsonl")).unwrap()).unwrap();
            assert_eq!(events.len(), 5, "{} event census", entry.id);
            for (n, event) in events.iter().enumerate() {
                assert_eq!(event.event_id, format!("event.{n}").parse::<Id>().unwrap());
                assert_eq!(event.event_sequence_number, n as u64);
                assert_eq!(event.run_id, static_id("m2"));
            }
            let step_ids = [
                "processing_stage.m1.extract",
                "processing_stage.m1.segment",
                "processing_stage.m2.model_fill",
                "processing_stage.m2.assemble",
            ];
            for (s, kind) in SINGLE_IR_STAGE_KINDS[..4].iter().enumerate() {
                let event = &events[s];
                assert_eq!(event.processing_stage, static_id(kind), "{}", entry.id);
                assert_eq!(event.pipeline_id, static_id("pipe.m2_single_ir"), "{kind}");
                assert_eq!(event.pipeline_step_id, static_id(step_ids[s]), "{kind}");
                assert_eq!(event.outcome, Outcome::Ok, "{kind}");
                assert_eq!(event.log_level, static_id("info"), "{kind}");
                assert!(event.diagnostics.is_empty(), "{kind}");
                assert_eq!(event.output_hashes.len(), 1, "{kind}");
            }
            // Slots 0/1/3 output the landed wrapper; slot 2 (model_fill,
            // direct-emitted) outputs the accepted cassette wrapper and
            // carries both §7.3 counters.
            assert!(events[0].input_hashes.is_empty());
            assert_eq!(
                events[0].output_hashes,
                std::slice::from_ref(&landed_source.content_hash)
            );
            assert!(events[0].resource_counters.is_empty());
            assert_eq!(
                events[1].input_hashes,
                std::slice::from_ref(&landed_source.content_hash)
            );
            assert_eq!(
                events[1].output_hashes,
                std::slice::from_ref(&landed_segments.content_hash)
            );
            assert!(events[1].resource_counters.is_empty());
            assert_eq!(
                events[2].input_hashes.iter().collect::<BTreeSet<_>>(),
                [&landed_source.content_hash, &landed_segments.content_hash]
                    .into_iter()
                    .collect::<BTreeSet<_>>()
            );
            assert_eq!(
                events[2].output_hashes,
                std::slice::from_ref(&cassette.content_hash)
            );
            assert_eq!(
                events[2].resource_counters,
                vec![(static_id("recorded_calls"), 1), (static_id("repairs"), 0)]
            );
            assert_eq!(
                events[3].input_hashes.iter().collect::<BTreeSet<_>>(),
                [
                    &landed_source.content_hash,
                    &landed_segments.content_hash,
                    &cassette.content_hash
                ]
                .into_iter()
                .collect::<BTreeSet<_>>()
            );
            assert_eq!(
                events[3].output_hashes,
                std::slice::from_ref(&landed_bundle.content_hash)
            );
            assert!(events[3].resource_counters.is_empty());
            let command = &events[4];
            assert_eq!(command.processing_stage, static_id("run"));
            assert_eq!(command.pipeline_id, static_id("cli"));
            assert_eq!(command.pipeline_step_id, "cli.run".parse::<Id>().unwrap());
            assert_eq!(command.log_level, static_id("info"));
            assert_eq!(command.outcome, Outcome::Ok);
            assert!(command.diagnostics.is_empty());
            assert!(command.input_hashes.is_empty());
            assert!(command.output_hashes.is_empty());
            assert!(command.resource_counters.is_empty());
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
            let head = route_document_head(&root, &entry, &resolved, &mut shell)
                .unwrap_or_else(|| panic!("{}: no deterministic route head", entry.id));
            let bundle = single_ir_fill(head, &lexicon, &store, 42, &resolved, 0, None, &mut shell)
                .trace
                .bundle
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
            let head = route_document_head(&root, &guideline_a, &resolved, &mut shell)
                .expect("guideline_a lands its deterministic route head");
            let bundle = single_ir_fill(head, &lexicon, &store, 98, &resolved, 1, None, &mut shell)
                .trace
                .bundle
                .expect("the malformed base repairs to an accepted bundle");
            // §9 attestation follows the ACCEPTED attempt: the bundle cites the
            // recovery (derived-seed) cassette, never the rejected base recording.
            let recovery = store
                .replay(&key(crate::model::derive_seed(98, 1)))
                .unwrap();
            assert!(
                bundle.input_hashes.contains(&recovery.content_hash),
                "the bundle cites the recovery cassette"
            );
            assert!(
                !bundle
                    .input_hashes
                    .contains(&store.replay(&key(98)).unwrap().content_hash),
                "the rejected base recording stays uncited"
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
            let head = route_document_head(&root, &guideline_a, &resolved, &mut shell)
                .expect("guideline_a lands its deterministic route head");
            let route = single_ir_fill(head, &lexicon, &store, 99, &resolved, 2, None, &mut shell);
            assert!(
                route.trace.bundle.is_none(),
                "a hallucinated reference ends the route"
            );
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

    /// run-m2.1d4a test convenience: rebuild the OLD per-group fill signature over the
    /// new [`DocHead`]-consuming [`direct_smt_fill`] so the call sites only rename and read
    /// `.pair`. Builds each member's [`DocHead`] via [`route_document_head`], then fills the
    /// group. A single-group convenience with no cross-group head dedup, so a member shared
    /// across groups (under one shell) heads once per group — harmless (the re-land
    /// overwrites and no assert here counts head events); run-m2.1d4b's reproduce / scores
    /// pair swaps onto a per-route head prepass where a pin counts head events.
    #[allow(clippy::too_many_arguments)]
    fn direct_fill_group(
        root: &Path,
        gid: &Id,
        members: &[&CorpusEntry],
        store: &CassetteStore,
        seed: u64,
        resolved: &Resolved,
        repair_limit: u32,
        shell: &mut Shell,
    ) -> DirectFill {
        let mut heads: Vec<DocHead> = Vec::new();
        for &m in members {
            heads.push(
                route_document_head(root, m, resolved, shell)
                    .unwrap_or_else(|| panic!("{gid}: no head for {}", m.id)),
            );
        }
        let head_refs: Vec<&DocHead> = heads.iter().collect();
        direct_smt_fill(
            gid,
            &head_refs,
            store,
            seed,
            resolved,
            repair_limit,
            None,
            shell,
        )
    }

    /// route-direct-smt.3a — a minimal [`Resolved`] for the direct_smt route over
    /// the M1 inputs. `pipe.m2_direct_smt` is four stages (extract, segment,
    /// model_fill_smt, verify_smt), filling slots `[0]`–`[3]` of the fixed `[Id; 8]`;
    /// only `producer(resolved, 0..=3)` is ever read, so slots `[4]`–`[7]` hold the inert
    /// [`UNUSED_STAGE`] sentinel, a non-stage id, so an accidental read surfaces obviously
    /// rather than posing as a real verify stage. `documents` / `groups` / `plan` go unread
    /// and stay empty; `budget_ms` is exp.m1_scaffold's §8.4 `solver_ms_per_query`.
    fn direct_smt_resolved() -> Resolved {
        Resolved {
            pipeline_id: static_id("pipe.m2_direct_smt"),
            pipeline_step_ids: [
                static_id("processing_stage.m1.extract"),
                static_id("processing_stage.m1.segment"),
                static_id("processing_stage.m2.model_fill_smt"),
                static_id("processing_stage.m2.verify_smt"),
                static_id(UNUSED_STAGE),
                static_id(UNUSED_STAGE),
                static_id(UNUSED_STAGE),
                static_id(UNUSED_STAGE),
            ],
            shape: RouteShape::DirectSmt,
            documents: vec![],
            groups: vec![],
            budget_ms: 10_000,
            repair_limit: Some(1),
            model_ms_per_call: Some(600_000),
            is_baseline: true,
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
        use std::collections::{BTreeMap, BTreeSet};

        use ckc_core::Outcome;

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
        let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out.clone()));

        // Per-route head prepass: build each UNIQUE member's DocHead once through
        // `route_document_head` (test_source.m1_guideline_a is in both groups) so a shared
        // document heads once per route — the per-route dedup shape .1d5a's orchestrator
        // adopts in execute(), pinned here at the fill boundary by the head-event census
        // below (execute()'s own route loop is pinned in .1d5a). Each group fills the heads.
        let mut unique_members: Vec<Id> = Vec::new();
        for group in &exp.test_source_groups {
            for s in &group.test_sources {
                if !unique_members.contains(s) {
                    unique_members.push(s.clone());
                }
            }
        }
        let mut heads: BTreeMap<Id, DocHead> = BTreeMap::new();
        for id in &unique_members {
            let entry = corpus
                .get(id)
                .unwrap_or_else(|| panic!("unknown member {id}"));
            let head = route_document_head(&root, entry, &resolved, &mut shell)
                .unwrap_or_else(|| panic!("{id}: no deterministic route head"));
            heads.insert(id.clone(), head);
        }

        // Each group's landed (overlap, deontic) pair, kept for the post-run layout and
        // §4.6 event pins after the loop.
        let mut landed_pairs: BTreeMap<
            Id,
            (ArtifactWrapper<QueryBody>, ArtifactWrapper<QueryBody>),
        > = BTreeMap::new();

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
            let head_refs: Vec<&DocHead> = members
                .iter()
                .map(|m| {
                    heads
                        .get(&m.id)
                        .unwrap_or_else(|| panic!("{gid}: prepass built no head for {}", m.id))
                })
                .collect();
            let (overlap, deontic) =
                direct_smt_fill(&gid, &head_refs, &store, 42, &resolved, 1, None, &mut shell)
                    .pair
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
            // pin the full input_hashes independently of the body equality above. Each
            // role's wrapper adds its OWN accepted cassette hash (§9 attestation);
            // input_hashes canonicalize as a §4.3 set, so compare as a set, never the
            // emitted order.
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
            for (w, role) in [(&overlap, "overlap"), (&deontic, "deontic")] {
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
                let cassette = store
                    .replay(&CassetteKey {
                        route: static_id("route.direct_smt"),
                        source: static_id(&format!("{gid}.{role}")),
                        seed: 42,
                    })
                    .unwrap();
                assert_eq!(w.input_hashes.len(), 5, "{gid} {role} input count");
                assert_eq!(
                    w.input_hashes.iter().collect::<BTreeSet<_>>(),
                    want_inputs
                        .iter()
                        .chain([&cassette.content_hash])
                        .collect::<BTreeSet<_>>(),
                    "{gid} {role} input_hashes"
                );
                w.validate()
                    .unwrap_or_else(|e| panic!("{gid} wrapper validate: {e:?}"));
            }

            landed_pairs.insert(gid, (overlap, deontic));
        }

        // Close the run and pin the landed §4.6 stream + on-disk layout (run-m2.1d4b),
        // mirroring the single_ir reproduce battery (.1d3b): the per-route head prepass
        // heads each unique document once, each group lands its two smt_query bodies, and
        // one model_fill event per group attests them.
        let finished = shell.finish().unwrap();
        assert_eq!(finished.result.outcome, Outcome::Ok);
        assert!(finished.result.diagnostic_hashes.is_empty());

        let listing = |path: &Path| -> Vec<String> {
            let mut names: Vec<String> = std::fs::read_dir(path)
                .unwrap()
                .map(|e| e.unwrap().file_name().into_string().unwrap())
                .collect();
            names.sort();
            names
        };

        // Layout: the head prepass lands under routes/<pipeline>/artifacts/<doc> and the
        // fills under routes/<pipeline>/groups/<gid>; guideline_a appears once under
        // artifacts (headed once).
        assert_eq!(listing(&out), ["logs", "routes"]);
        assert_eq!(
            listing(&out.join("logs")),
            ["diagnostics.jsonl", "events.jsonl"]
        );
        assert_eq!(listing(&out.join("routes")), ["pipe.m2_direct_smt"]);
        assert_eq!(
            listing(&out.join("routes/pipe.m2_direct_smt")),
            ["artifacts", "groups"]
        );
        assert_eq!(
            listing(&out.join("routes/pipe.m2_direct_smt/artifacts")),
            [
                "test_source.m1_control",
                "test_source.m1_guideline_a",
                "test_source.m1_guideline_b"
            ]
        );
        for doc in [
            "test_source.m1_control",
            "test_source.m1_guideline_a",
            "test_source.m1_guideline_b",
        ] {
            assert_eq!(
                listing(&out.join(format!("routes/pipe.m2_direct_smt/artifacts/{doc}"))),
                ["segments.json", "source_document_graph.json"],
                "{doc}"
            );
        }
        assert_eq!(
            listing(&out.join("routes/pipe.m2_direct_smt/groups")),
            ["group.m1_conflict", "group.m1_no_conflict"]
        );

        // Per group: the two landed smt_query wrappers strict-read clean, their ids
        // route-prefixed, their content hashes equal the fill's returned pair. Group
        // artifacts land route-namespaced (the head-namespacing mirror).
        for (gid, (overlap, deontic)) in &landed_pairs {
            let group_dir = format!("routes/pipe.m2_direct_smt/groups/{gid}");
            assert_eq!(
                listing(&out.join(&group_dir)),
                ["deontic.smt_query.json", "overlap.smt_query.json"],
                "{gid}"
            );
            let overlap_landed: ArtifactWrapper<QueryBody> =
                strict_at(&out, &format!("{group_dir}/overlap.smt_query.json"));
            let deontic_landed: ArtifactWrapper<QueryBody> =
                strict_at(&out, &format!("{group_dir}/deontic.smt_query.json"));
            assert_eq!(
                overlap_landed.artifact_id,
                format!("pipe.m2_direct_smt.{gid}.overlap.smt_query")
                    .parse()
                    .unwrap(),
                "{gid} overlap id"
            );
            assert_eq!(
                deontic_landed.artifact_id,
                format!("pipe.m2_direct_smt.{gid}.deontic.smt_query")
                    .parse()
                    .unwrap(),
                "{gid} deontic id"
            );
            assert_eq!(
                overlap_landed.content_hash, overlap.content_hash,
                "{gid} overlap landed hash"
            );
            assert_eq!(
                deontic_landed.content_hash, deontic.content_hash,
                "{gid} deontic landed hash"
            );
        }

        // The §4.6 event stream: six head events (extract + segment for the three unique
        // documents — guideline_a once, not twice) then one model_fill event per group,
        // then the closing command event.
        let events: Vec<EventRecord> =
            read_jsonl(&std::fs::read(out.join("logs/events.jsonl")).unwrap()).unwrap();
        assert_eq!(events.len(), 9, "event census");
        for (n, event) in events.iter().enumerate() {
            assert_eq!(event.event_id, format!("event.{n}").parse::<Id>().unwrap());
            assert_eq!(event.event_sequence_number, n as u64);
            assert_eq!(event.run_id, static_id("m2"));
        }
        // Head events: three extract + three segment, one per unique document — a
        // per-group build would head the shared guideline_a twice (four extract events).
        assert_eq!(
            events
                .iter()
                .filter(|e| e.processing_stage == static_id("extract"))
                .count(),
            3,
            "one extract per unique document (guideline_a once)"
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| e.processing_stage == static_id("segment"))
                .count(),
            3,
            "one segment per unique document"
        );
        for event in events.iter().take(6) {
            assert!(
                event.processing_stage == static_id("extract")
                    || event.processing_stage == static_id("segment"),
                "the first six events are the head prepass"
            );
            assert_eq!(event.pipeline_id, static_id("pipe.m2_direct_smt"));
            assert_eq!(event.outcome, Outcome::Ok);
            assert_eq!(event.log_level, static_id("info"));
            assert!(event.diagnostics.is_empty());
            assert_eq!(event.output_hashes.len(), 1);
        }

        // One model_fill event per group (events[6] = m1_conflict, events[7] =
        // m1_no_conflict, in test_source_groups order): the direct route's model_fill kind
        // and model_fill_smt step, both roles' recorded calls summed, the pair's member
        // source+segments as inputs, and the two landed smt_query bodies as outputs
        // (input/output hashes canonicalize as §4.3 sets, so compare each as a set).
        for (i, group) in exp.test_source_groups.iter().enumerate() {
            let event = &events[6 + i];
            let (overlap, deontic) = &landed_pairs[&group.group_id];
            assert_eq!(
                event.processing_stage,
                static_id(DIRECT_SMT_STAGE_KINDS[MODEL_FILL]),
                "{}: model_fill kind",
                group.group_id
            );
            assert_eq!(
                event.pipeline_id,
                static_id("pipe.m2_direct_smt"),
                "{}",
                group.group_id
            );
            assert_eq!(
                event.pipeline_step_id,
                static_id("processing_stage.m2.model_fill_smt"),
                "{}: model_fill step",
                group.group_id
            );
            assert_eq!(event.outcome, Outcome::Ok, "{}", group.group_id);
            assert_eq!(event.log_level, static_id("info"), "{}", group.group_id);
            assert!(event.diagnostics.is_empty(), "{}", group.group_id);
            assert_eq!(
                event.resource_counters,
                vec![
                    (static_id(RECORDED_CALLS_COUNTER), 2),
                    (static_id(REPAIRS_COUNTER), 0)
                ],
                "{}: both roles' recorded calls summed",
                group.group_id
            );
            let mut want_event_inputs: BTreeSet<&Hash> = BTreeSet::new();
            for s in &group.test_sources {
                let h = &heads[s];
                want_event_inputs.insert(&h.source.content_hash);
                want_event_inputs.insert(&h.segments.content_hash);
            }
            assert_eq!(
                event.input_hashes.iter().collect::<BTreeSet<_>>(),
                want_event_inputs,
                "{}: the pair's member source+segments provenance (no cassette hashes)",
                group.group_id
            );
            assert_eq!(
                event.output_hashes.iter().collect::<BTreeSet<_>>(),
                [&overlap.content_hash, &deontic.content_hash]
                    .into_iter()
                    .collect::<BTreeSet<_>>(),
                "{}: the two landed smt_query bodies",
                group.group_id
            );
        }

        // The closing command event.
        let command = &events[8];
        assert_eq!(command.processing_stage, static_id("run"));
        assert_eq!(command.pipeline_id, static_id("cli"));
        assert_eq!(command.pipeline_step_id, "cli.run".parse::<Id>().unwrap());
        assert_eq!(command.outcome, Outcome::Ok);
        assert!(command.diagnostics.is_empty());
    }

    /// route-direct-smt.3b fail-closed guard: the cassette-role design mints exactly one
    /// (overlap, deontic) pair per group, so a non-pair member set must yield no pair
    /// rather than a two-query wrapper that silently under-covers the group.
    /// [`direct_fill_group`] lands each member's [`DocHead`] first (both cases reuse the
    /// valid `corpus[0]`), but [`direct_smt_fill`]'s head-count guard still precedes any
    /// cassette access, so no role fill runs and `fills` stays empty.
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
            let got =
                direct_fill_group(&root, &gid, &members, &store, 42, &resolved, 1, &mut shell);
            assert!(
                got.pair.is_none(),
                "non-pair group (len {n}) must fail closed"
            );
            assert!(
                got.fills.is_empty(),
                "the guard precedes any cassette access — no role fill runs"
            );
        }
    }

    /// route-direct-smt.3b partial-landing attestation (codex-review .1d4a follow-up): a
    /// deontic cassette IO failure AFTER the overlap role has landed still rides the one
    /// model_fill §4.6 event, like a wrap/land failure, so the orphaned overlap `smt_query`
    /// stays attested and overlap's counters are not dropped. Regression guard: the pre-fix
    /// arm early-returned event-less, leaving a landed artifact covered by no event.
    #[test]
    fn direct_smt_fill_landed_then_missing_deontic_still_emits_event() {
        use ckc_core::Outcome;
        use std::collections::BTreeMap;

        let root = repo_root();
        let lexicon = single_ir_lexicon();
        let resolved = direct_smt_resolved();

        // A store holding ONLY one group's overlap cassette: overlap replays and lands,
        // then the absent deontic cassette raises a CassetteError mid-pair.
        let tmp = tempfile::tempdir().unwrap();
        let store = CassetteStore::new(tmp.path().join("fixtures"));
        let (gid, qbodies) = m1_reference_query_bodies(&root, &lexicon)
            .into_iter()
            .next()
            .expect("at least one exp.m1_scaffold group");
        write_direct_smt_cassette(
            static_id(&format!("{gid}.overlap")),
            &resolved,
            &store,
            42,
            qbodies[0].body.as_bytes(),
        );

        // Resolve the group's real members from exp.m1_scaffold (route_document_head reads
        // their HTML), never a hardcoded membership.
        let corpus: BTreeMap<Id, CorpusEntry> = single_ir_corpus()
            .into_iter()
            .map(|entry| (entry.id.clone(), entry))
            .collect();
        let experiments = parse_experiments(
            &std::fs::read_to_string(root.join("registry/experiments.yaml")).unwrap(),
        )
        .unwrap();
        let group = experiments
            .iter()
            .find(|e| e.id == static_id("exp.m1_scaffold"))
            .expect("exp.m1_scaffold")
            .test_source_groups
            .iter()
            .find(|g| g.group_id == gid)
            .expect("the group is in exp.m1_scaffold");
        let members: Vec<&CorpusEntry> = group
            .test_sources
            .iter()
            .map(|s| {
                corpus
                    .get(s)
                    .unwrap_or_else(|| panic!("{gid}: unknown member {s}"))
            })
            .collect();

        let out = tmp.path().join("m2");
        std::fs::create_dir_all(&out).unwrap();
        let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out.clone()));
        let got = direct_fill_group(&root, &gid, &members, &store, 42, &resolved, 1, &mut shell);

        // No pair (deontic never landed), yet overlap is observed and the group's one
        // model_fill event is emitted, attesting the landed overlap and folding its counter.
        assert!(
            got.pair.is_none(),
            "a missing deontic cassette yields no pair"
        );
        // run-m2.1e-A — the landed overlap is retained (not discarded with the absent
        // pair), so execute_routes carries it into the run manifest's output_hashes and
        // a lone landed role stays replay-covered.
        assert_eq!(
            got.smt_queries.len(),
            1,
            "the lone landed overlap survives for manifest coverage"
        );
        assert_eq!(
            got.fills.len(),
            1,
            "the overlap role is observed before the deontic failure"
        );
        let model_fill_events: Vec<_> = shell
            .events()
            .iter()
            .filter(|e| e.processing_stage == static_id(DIRECT_SMT_STAGE_KINDS[MODEL_FILL]))
            .collect();
        assert_eq!(
            model_fill_events.len(),
            1,
            "exactly one model_fill event rides despite the infra failure"
        );
        let event = model_fill_events[0];
        assert_eq!(
            event.output_hashes.len(),
            1,
            "the landed overlap smt_query is attested by the event"
        );
        assert_eq!(
            event.outcome,
            Outcome::Invalid,
            "the deontic cassette failure marks the stage invalid"
        );
        let recorded = event
            .resource_counters
            .iter()
            .find(|(k, _)| *k == static_id(RECORDED_CALLS_COUNTER))
            .map(|(_, v)| *v)
            .expect("recorded_calls counter present");
        assert!(
            recorded >= 1,
            "overlap's recorded call folds into the event, not dropped"
        );
        assert!(
            out.join(format!(
                "routes/pipe.m2_direct_smt/groups/{gid}/overlap.smt_query.json"
            ))
            .exists(),
            "overlap landed on disk before the deontic cassette failed"
        );
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

        // Per-route head prepass: build each unique member's DocHead once (guideline_a is
        // in both groups), the .1d5a orchestrator's dedup — a shared document heads once
        // per route, pinned by the head-event census after the loop.
        let mut unique_members: Vec<Id> = Vec::new();
        for group in &exp.test_source_groups {
            for s in &group.test_sources {
                if !unique_members.contains(s) {
                    unique_members.push(s.clone());
                }
            }
        }
        let mut heads: BTreeMap<Id, DocHead> = BTreeMap::new();
        for id in &unique_members {
            let entry = corpus
                .get(id)
                .unwrap_or_else(|| panic!("unknown member {id}"));
            let head = route_document_head(&root, entry, &resolved, &mut shell)
                .unwrap_or_else(|| panic!("{id}: no deterministic route head"));
            heads.insert(id.clone(), head);
        }

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
            let head_refs: Vec<&DocHead> = members
                .iter()
                .map(|m| {
                    heads
                        .get(&m.id)
                        .unwrap_or_else(|| panic!("{gid}: prepass built no head for {}", m.id))
                })
                .collect();
            let (overlap, deontic) =
                direct_smt_fill(&gid, &head_refs, &store, 42, &resolved, 0, None, &mut shell)
                    .pair
                    .expect("the scored group fills a pair");
            let results = direct_smt_verify_group(
                &gid,
                &route_group_dir(&resolved, &gid),
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

        // The per-route head prepass heads each unique document once: three extract and
        // three segment events (guideline_a, shared by both groups, heads once — a
        // per-group build would extract it twice), then one model_fill and one verify
        // event per group.
        let stage_count = |kind: &str| {
            shell
                .events()
                .iter()
                .filter(|e| e.processing_stage == static_id(kind))
                .count()
        };
        assert_eq!(
            stage_count("extract"),
            3,
            "one extract per unique document (guideline_a once)"
        );
        assert_eq!(stage_count("segment"), 3, "one segment per unique document");
        assert_eq!(
            stage_count("model_fill"),
            2,
            "one model_fill event per group"
        );
        assert_eq!(stage_count("verify"), 2, "one verify event per group");
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
    /// is a minimal valid filler, unverified once Q1's failure closes the pair. Synthetic-identity
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
        // minimal valid filler, left unverified once Q1's failure closes the pair.
        write_direct_smt_cassette(
            static_id("group.m2_direct_syntax.overlap"),
            &resolved,
            &store,
            90,
            b"(set-logic QF_LRA)\n(declare-const x Bool)\n(assert (and x\n(check-sat)\n",
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
    /// model-runtime-absent). Replaying the committed rejection-path cassettes: (a) a
    /// schema-exhaustion group whose overlap base (seed 91) and first-repair
    /// (`derive_seed(91, 1)`) outputs both fail the shallow SMT accept — a
    /// `repair_limit = 1` fill re-prompts once and terminates in `repair_limit_exceeded`,
    /// surfaced both at the [`model_fill`] boundary and on the route fn's shell ledger,
    /// with no target and the deontic source never read (Q1 exhausts first); (b) a
    /// syntax-failure group — direct-route-unique, no single_ir analogue: the overlap
    /// body shallow-accepts (so [`direct_smt_fill`] lands the pair with an empty ledger)
    /// yet z3 rejects its unbalanced parens with an `(error …)` and no verdict, so
    /// [`direct_smt_verify_group`] mints a lone `target_syntax_failure` /
    /// `target_parse_error` result that rides the directly-emitted §4.6 verify event.
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
            let filled = direct_fill_group(
                &root,
                &static_id("group.m2_direct_schema"),
                &members,
                &store,
                91,
                &resolved,
                1,
                &mut shell,
            );
            assert!(filled.pair.is_none(), "schema exhaustion ends the route");
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

        let (overlap, deontic) = direct_fill_group(
            &root,
            &syntax_gid,
            &members,
            &store,
            90,
            &resolved,
            0,
            &mut shell,
        )
        .pair
        .expect("the shallow-accepting pair fills");
        assert!(
            shell.ledger().is_empty(),
            "a shallow-accepting fill raises no diagnostics"
        );

        let results = direct_smt_verify_group(
            &syntax_gid,
            &route_group_dir(&resolved, &syntax_gid),
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
            "Q1's syntax failure closes the pair — no Q2 result"
        );
        let result = &results.payload.results[0];
        assert_eq!(result.query_id, static_id("group.m2_direct_syntax.overlap"));
        assert_eq!(result.category, VerifierCategory::TargetSyntaxFailure);
        assert_eq!(result.diagnostics[0].code, DiagnosticCode::TargetParseError);
        assert_eq!(result.diagnostics[0].outcome, Outcome::Invalid);

        // §7.4 surfaces on the aggregate ledger …
        let codes: Vec<_> = shell.ledger().iter().map(|d| d.code).collect();
        assert_eq!(codes, vec![DiagnosticCode::TargetParseError]);
        // … and rides the directly-emitted §4.6 verify event itself (not a side-channel
        // push): slot-3 verify_smt, Invalid, carrying the parse-error diagnostic and the
        // solver-budget counter.
        let event = shell.events().last().expect("a verify event");
        assert_eq!(event.processing_stage, static_id("verify"));
        assert_eq!(
            event.pipeline_step_id,
            static_id("processing_stage.m2.verify_smt")
        );
        assert_eq!(event.outcome, Outcome::Invalid);
        let event_codes: Vec<_> = event.diagnostics.iter().map(|d| d.code).collect();
        assert_eq!(event_codes, vec![DiagnosticCode::TargetParseError]);
        assert_eq!(
            event.resource_counters,
            vec![(static_id(SOLVER_BUDGET_KEY), resolved.budget_ms)],
            "the solver-budget counter rides the direct verify event"
        );
    }

    /// metrics-m2.1 — fold both routes' recorded-run observation channels into
    /// their §7.3 raw rows over the committed cassettes (z3 present,
    /// model-runtime-absent). Each arm replays its route's real machinery rather
    /// than hand-built observations: single_ir observes six fills (the three
    /// golden documents at seed 42 plus guideline_a's hallucinated / repaired /
    /// exhausted rejection seeds) and the two exp.m1_scaffold group verdict
    /// tails; direct_smt observes seven role fills (four golden, the
    /// schema-exhaustion overlap, the shallow-accepting syntax pair) and three
    /// verified groups — the schema-exhaustion group fills terminally so lands
    /// fill-only, and the syntax group's lone `TargetSyntaxFailure` is a
    /// solver-executed parse failure inside the syntactic denominator while
    /// sitting outside the reference (accuracy scores the two M1 groups alone).
    /// Pins each arm's FULL id-sorted row vector to hand-derived exact reduced
    /// fractions, with no omission diagnostics.
    #[test]
    fn route_metrics_score_recorded_two_route_run() {
        use std::collections::BTreeMap;

        use ckc_core::Rational;

        use crate::metrics::{
            ACCEPTANCE_RATE, CONFLICT_VERDICT_ACCURACY, FillObservation, GroupObservation,
            K_SAMPLE_CONVERGENCE, MetricRow, MetricValue, RECORDED_CALL_COUNT, REPAIR_COUNT,
            SCHEMA_VALID_RATE, TARGET_SYNTACTIC_VALIDITY, route_metrics,
        };

        let root = repo_root();
        let lexicon = single_ir_lexicon();
        let store = CassetteStore::new(root.join("crates/ckc-cli/tests/fixtures"));
        let adapter = Z3Adapter::new().expect("z3 adapter on PATH");

        // Both arms score against the same locked M1 reference.
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

        let row = |metric: &str, num: &str, den: &str| MetricRow {
            metric: static_id(metric),
            value: MetricValue::Value(Rational::from_parts(num, den).unwrap()),
        };
        // The recorded run replays one draw per fill — no k-sample battery,
        // so convergence is honestly not applicable on both arms.
        let na = |metric: &str| MetricRow {
            metric: static_id(metric),
            value: MetricValue::NotApplicable,
        };

        // ARM A — pipe.m2_single_ir.
        {
            let resolved = single_ir_resolved();
            let tmp = tempfile::tempdir().unwrap();
            let out = tmp.path().join("m2");
            std::fs::create_dir_all(&out).unwrap();
            let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out));

            // Fill channel: one observed model_fill per (document, seed), each over
            // the document's real grounding universe (the deterministic extract →
            // segment head, the single_ir_route_rejection_codes shape).
            let observe = |entry: &CorpusEntry, seed: u64| {
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
                let fill = model_fill(
                    &store,
                    &CassetteKey {
                        route: static_id("route.single_ir"),
                        source: entry.id.clone(),
                        seed,
                    },
                    FillSource::Replay,
                    1,
                    single_ir_accept(&regions, &segment_ids),
                )
                .unwrap();
                FillObservation::from_fill(&fill)
            };
            let corpus = single_ir_corpus();
            let guideline_a = corpus
                .iter()
                .find(|e| e.id == static_id("test_source.m1_guideline_a"))
                .expect("guideline_a in the corpus");
            let mut fills: Vec<FillObservation> =
                corpus.iter().map(|entry| observe(entry, 42)).collect();
            for seed in [99, 98, 97] {
                fills.push(observe(guideline_a, seed));
            }
            assert_eq!(fills.len(), 6);

            // Verdict channel: fill every document once, then run each
            // exp.m1_scaffold group through the compile → verify tail (the
            // single_ir_route_scores_m1_groups shape).
            let mut bundles = BTreeMap::new();
            for entry in &corpus {
                let head = route_document_head(&root, entry, &resolved, &mut shell)
                    .unwrap_or_else(|| panic!("{}: no deterministic route head", entry.id));
                let bundle =
                    single_ir_fill(head, &lexicon, &store, 42, &resolved, 1, None, &mut shell)
                        .trace
                        .bundle
                        .unwrap_or_else(|| {
                            panic!("{}: single_ir_fill yielded no bundle", entry.id)
                        });
                bundles.insert(entry.id.clone(), bundle);
            }
            let mut groups = Vec::new();
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
                groups.push(GroupObservation {
                    group_id: gid,
                    query_pairs: compiled
                        .payload
                        .solver_query_plan
                        .iter()
                        .map(|p| {
                            (
                                p.context_overlap_query_id.clone(),
                                p.deontic_consistency_query_id.clone(),
                            )
                        })
                        .collect(),
                    results: results.payload.results.clone(),
                });
            }

            assert_eq!(groups.len(), 2);
            let metrics = route_metrics(
                &static_id("pipe.m2_single_ir"),
                &fills,
                &groups,
                &[],
                &reference,
            );
            assert_eq!(metrics.pipeline_id, static_id("pipe.m2_single_ir"));
            assert_eq!(
                metrics.rows,
                vec![
                    row(ACCEPTANCE_RATE, "2", "3"),
                    row(CONFLICT_VERDICT_ACCURACY, "1", "1"),
                    na(K_SAMPLE_CONVERGENCE),
                    row(RECORDED_CALL_COUNT, "8", "1"),
                    row(REPAIR_COUNT, "2", "1"),
                    row(SCHEMA_VALID_RATE, "5", "8"),
                    row(TARGET_SYNTACTIC_VALIDITY, "1", "1"),
                ]
            );
            assert!(metrics.diagnostics.is_empty());
        }

        // ARM B — pipe.m2_direct_smt.
        {
            let resolved = direct_smt_resolved();
            let tmp = tempfile::tempdir().unwrap();
            let out = tmp.path().join("m2");
            std::fs::create_dir_all(&out).unwrap();
            let mut shell = Shell::open(static_id("run"), static_id("m2"), Some(out));

            // Fill channel: one observed model_fill per (minted role source, seed) —
            // the four golden seed-42 role fills plus the rejection sources
            // (schema-exhausted overlap 91, shallow-accepting syntax pair 90).
            let observe = |source: &str, seed: u64| {
                let fill = model_fill(
                    &store,
                    &CassetteKey {
                        route: static_id("route.direct_smt"),
                        source: static_id(source),
                        seed,
                    },
                    FillSource::Replay,
                    1,
                    direct_smt_accept(),
                )
                .unwrap();
                FillObservation::from_fill(&fill)
            };
            let fills = vec![
                observe("group.m1_conflict.overlap", 42),
                observe("group.m1_conflict.deontic", 42),
                observe("group.m1_no_conflict.overlap", 42),
                observe("group.m1_no_conflict.deontic", 42),
                observe("group.m2_direct_schema.overlap", 91),
                observe("group.m2_direct_syntax.overlap", 90),
                observe("group.m2_direct_syntax.deontic", 90),
            ];

            // Verdict channel: the two golden exp.m1_scaffold groups at seed 42 plus
            // the syntax-failure group at seed 90 (the direct_smt_route_rejection_codes
            // members), each through the fill → verify tail. The schema-exhaustion
            // group fills terminally — no pair, no group observation.
            let corpus: BTreeMap<Id, CorpusEntry> = single_ir_corpus()
                .into_iter()
                .map(|entry| (entry.id.clone(), entry))
                .collect();
            let member = |id: &str| {
                corpus
                    .get(&static_id(id))
                    .unwrap_or_else(|| panic!("{id} in the corpus"))
            };
            let worklist: Vec<(Id, Vec<&CorpusEntry>, u64)> = exp
                .test_source_groups
                .iter()
                .map(|group| {
                    (
                        group.group_id.clone(),
                        group
                            .test_sources
                            .iter()
                            .map(|s| {
                                corpus
                                    .get(s)
                                    .unwrap_or_else(|| panic!("unknown member {s}"))
                            })
                            .collect(),
                        42,
                    )
                })
                .chain(std::iter::once((
                    static_id("group.m2_direct_syntax"),
                    vec![
                        member("test_source.m1_guideline_a"),
                        member("test_source.m1_guideline_b"),
                    ],
                    90,
                )))
                .collect();
            let mut groups = Vec::new();
            for (gid, members, seed) in worklist {
                let (overlap, deontic) = direct_fill_group(
                    &root, &gid, &members, &store, seed, &resolved, 1, &mut shell,
                )
                .pair
                .unwrap_or_else(|| panic!("{gid}: direct_fill_group yielded no pair"));
                let results = direct_smt_verify_group(
                    &gid,
                    &route_group_dir(&resolved, &gid),
                    &overlap,
                    &deontic,
                    &resolved,
                    &adapter,
                    &mut shell,
                )
                .unwrap_or_else(|| panic!("{gid}: no verifier results"));
                let query_pairs = vec![(
                    static_id(&format!("{gid}.overlap")),
                    static_id(&format!("{gid}.deontic")),
                )];
                groups.push(GroupObservation {
                    group_id: gid,
                    query_pairs,
                    results: results.payload.results.clone(),
                });
            }

            assert_eq!(groups.len(), 3);
            let metrics = route_metrics(
                &static_id("pipe.m2_direct_smt"),
                &fills,
                &groups,
                &[],
                &reference,
            );
            assert_eq!(metrics.pipeline_id, static_id("pipe.m2_direct_smt"));
            assert_eq!(
                metrics.rows,
                vec![
                    row(ACCEPTANCE_RATE, "6", "7"),
                    row(CONFLICT_VERDICT_ACCURACY, "1", "1"),
                    na(K_SAMPLE_CONVERGENCE),
                    row(RECORDED_CALL_COUNT, "8", "1"),
                    row(REPAIR_COUNT, "1", "1"),
                    row(SCHEMA_VALID_RATE, "3", "4"),
                    row(TARGET_SYNTACTIC_VALIDITY, "3", "4"),
                ]
            );
            assert!(metrics.diagnostics.is_empty());
        }
    }
}
