//! ckc-cli — the `ckc` binary (SPEC §3): pipeline processing_stages, runner,
//! trace/report/replay, registry check. Surface by module (unit lineage:
//! roadmap stubs + `git log`):
//!
//! - `dispatch`/`shell` ([`run_cli`], [`CliExit`]) — the §3 four-command
//!   surface with validated argument shapes and the once-wired CLI
//!   invariants: containment-conditioned writes, §4.6 JSONL event/diagnostic
//!   streams, exactly one §4.4 total operation result, outcome-mapped exit
//!   codes.
//! - `registry_check` — `ckc registry check` (§8.5 item 2): the §8.4
//!   registry surface plus experiment-referenced reference strict-loaded from the
//!   invocation root and validated as one cross-referenced set, every load
//!   failure and finding a §7.4 `schema_invalid` diagnostic.
//! - [`extract`] — html5ever-parsed test_source HTML walked into an wrapped
//!   §4.5 [`ckc_core::SourceDocumentGraph`]: sections/paragraphs/lists spanned with
//!   {node,span} regions, tables as literal cell grids (`row`/`col`/`header`
//!   attrs; rejected tables withheld as `table_structure_uncertain`), parse
//!   errors and unknown flow content as `extraction_uncertain` residuals.
//! - [`segment`] — the graph's spans classified in reading order into §5
//!   ClinicalSegments (cq, recommendation, exception, definition, table-row,
//!   evidence, metadata), misses as `segmentation_boundary_error` residuals.
//! - [`normalize`] — the §5 lexicon evidence_status `corpus/lexicon/ja_core.yaml`
//!   strict-loaded into the typed [`normalize::Lexicon`] (semantic_ja
//!   surfaces, `surfaces[0]` representative, §5-coherent intervals, raw-byte
//!   content hash for manifests); mention binding
//!   ([`normalize::bind_segments`]: recommendation/exception spans scanned
//!   longest-match, singleton candidate sets binding `exact`/`synonym`,
//!   shared surfaces `ambiguous` with a `terminology_ambiguous` record);
//!   statement building ([`normalize::clinical_ir`]: slot readings building
//!   at most one §5 ClinicalStatement per recommendation segment, misses and
//!   ambiguities withholding it as §7.4 records; exception segments
//!   attaching their concepts as ExceptionClauses to the nearest preceding
//!   kept statement, concept-free or statement-less exceptions dropping the
//!   clause as a Residual); the processing_stage entry ([`normalize::normalize`]:
//!   statement and rule layers as one wrapped `schema.normalization`
//!   artifact, input hashes `[source, segments]`).
//! - [`rules`] — NormativeRule derivation (`stage-normalize.2b`):
//!   [`rules::derive_norm_ir`] lowering `statements[k]` to `rules[k]` under
//!   the §8.6 id scheme: one DNF conjunct per rule, population/condition
//!   concepts interval-lowered through the lexicon, exception clauses
//!   joined as negated concept atoms, clause regions and ids landing in
//!   `source_region_ids`/`exception_refs`.
//! - `run` — `ckc run` (`cli-runner.2a`/`.2b`): the experiment
//!   resolved through the §8.4 registries, each corpus document driven
//!   extract → segment → normalize → assemble (the core-ir.4/.5 wrapper)
//!   into `artifacts/<doc-id>/` of the §8.3 layout, then each test_source
//!   group driven compile → verify (the ckc-smt processing_stage cores under the live
//!   Z3 adapter and the experiment's per-query budget) into
//!   `groups/<gid>/` with every query body byte-identical under `smt/`;
//!   every artifact written canonical and strict-read back at the
//!   boundary, one §4.6 processing_stage event per attempted processing_stage, the §4.4 total
//!   outcome the severity fold over the whole run; after the group loop
//!   the run-scoped trace processing_stage assembles and lands the §7.1 pair at the
//!   run root (`cli-runner.3a.3`), then the report processing_stage assembles the
//!   §7.2 report over the landed pair, the source-graph wrappers, the
//!   verifier results, the raw lexicon-byte hash, the live solver
//!   identity, and the diagnostic ledger, landing `report.json` beside it
//!   (`cli-runner.4.1b.1`); `report_en.md` and the manifests join with
//!   cli-runner.4.1b.2b.
//! - [`trace`] — §7.1 trace-processing_stage payloads (`cli-runner.3a.1`–`.3a.3`):
//!   [`trace::TraceBundle`], the derivation DAG (eight ranked
//!   [`trace::TraceNodeKind`]s, operation-labeled strictly rank-upward
//!   edges) plus §7.2 claim-evidence rows, and [`trace::LineageIndex`],
//!   its per-(finding, document) query index — every collection a
//!   canonical set, structural validation via [`trace::TraceError`],
//!   assembly via [`trace::assemble_trace`] over the run's [`trace::DocTrace`]
//!   / [`trace::GroupTrace`] hand-offs; `trace::command` is `ckc trace`
//!   (`cli-runner.3b`, §8.5 item 7): the pair strict-read from the run
//!   root, one finding resolved through claim row + lineage rows + source
//!   nodes under a pair-agreement check, the chain rendered in both
//!   directions as the command's stdout body.
//! - [`report`] — §7.2 report payload (`cli-runner.4.1a.1`–`.4.1b.1`):
//!   [`report::Report`], the canonical `report.json` shape (corpus/lexicon
//!   hashes, finding and documented-null partitions as
//!   [`report::ReportFinding`] rows with quoted spans, code-keyed
//!   diagnostics summary, solver identity, replay status, §0 wording) with
//!   structural validation via [`report::ReportError`] and assembly via
//!   [`report::assemble_report`] over the run's validated artifacts, plus
//!   [`report::render_markdown`], the deterministic §7.2 derived view
//!   (`report_en.md` body, cli-runner.4.1b.2a).
//! - [`manifests`] — §5/§4.6 manifest assembly (cli-runner.4.1b.2a):
//!   [`manifests::assemble_manifests`] building RunManifest +
//!   ReplayManifest over the caller-supplied [`manifests::ManifestInputs`]
//!   run state (no I/O): §5 plan-hash linkage, canonical set/map storage,
//!   shared fields and expected-vs-output hashes mirrored between the two
//!   records; the run landing writes both with cli-runner.4.1b.2b.
//! - [`replay`] — §4.6 replay core (cli-runner.4.2a), no shell contact:
//!   [`replay::execute`] strict-reads a prior run's `replay_manifest.json`,
//!   re-executes the recorded experiment through the full run pipeline
//!   into an empty scratch directory (a fresh §8.3 layout under a
//!   replay-owned internal shell), and compares accepted-artifact content
//!   hashes, runtime metadata excluded by construction — a match is the
//!   §4.6 re-run-equals-prior property, divergence a `replay_mismatch`
//!   carrying the symmetric difference, a missing solver
//!   `replay_identity_unsupported`; `replay::command` is `ckc replay`
//!   (cli-runner.4.2b, §8.5 item 8): the prior run re-executed into the
//!   sibling `<run-dir>.replay` scratch layout, a match rendered as the
//!   command's stdout body (rerun outcome and layout, every accepted
//!   artifact hash), every failure one command-scope §7.4 diagnostic.
#![forbid(unsafe_code)]

pub mod extract;
pub mod manifests;
pub mod normalize;
pub mod replay;
pub mod report;
pub mod rules;
pub mod segment;
pub mod trace;

mod dispatch;
mod registry_check;
mod run;
mod shell;

pub use dispatch::{CliExit, run_cli};
