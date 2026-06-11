//! ckc-cli — the `ckc` binary (SPEC §3): pipeline stages, runner,
//! trace/report/replay, registry check. Surface by module (unit lineage:
//! roadmap stubs + `git log`):
//!
//! - `dispatch`/`shell` ([`run_cli`], [`CliExit`]) — the §3 four-command
//!   surface with validated argument shapes and the once-wired CLI
//!   invariants: containment-guarded writes, §4.6 JSONL event/diagnostic
//!   streams, exactly one §4.4 total operation result, outcome-mapped exit
//!   codes; pending command bodies return typed `unsupported` results.
//! - `registry_check` — `ckc registry check` (§8.5 item 2): the §8.4
//!   registry surface plus experiment-referenced gold strict-loaded from the
//!   invocation root and validated as one cross-referenced set, every load
//!   failure and finding a §7.4 `schema_invalid` diagnostic.
//! - [`extract`] — html5ever-parsed fixture HTML walked into an enveloped
//!   §4.5 [`ckc_core::SourceGraph`]: sections/paragraphs/lists spanned with
//!   {node,span} regions, tables as literal cell grids (`row`/`col`/`header`
//!   attrs; rejected tables withheld as `table_structure_uncertain`), parse
//!   errors and unknown flow content as `extraction_uncertain` residuals.
//! - [`segment`] — the graph's spans classified in reading order into §5
//!   ClinicalSegments (cq, recommendation, exception, definition, table-row,
//!   evidence, metadata), misses as `segmentation_boundary_error` residuals.
//! - [`normalize`] — the §5 lexicon authority `corpus/lexicon/ja_core.yaml`
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
//!   clause as a Residual); the stage entry ([`normalize::normalize`]:
//!   statement and rule layers as one enveloped `schema.normalization`
//!   artifact, input hashes `[source, segments]`).
//! - [`rules`] — NormRule derivation (`stage-normalize.2b`):
//!   [`rules::derive_norm_ir`] lowering `statements[k]` to `rules[k]` under
//!   the §8.6 id scheme: one DNF conjunct per rule, population/condition
//!   concepts interval-lowered through the lexicon, exception clauses
//!   joined as negated concept atoms, clause regions and ids landing in
//!   `source_region_ids`/`exception_refs`.
//! - `run` — `ckc run`'s document half (`cli-runner.2a`): the experiment
//!   resolved through the §8.4 registries, each corpus document driven
//!   extract → segment → normalize → assemble (the core-ir.4/.5 wrapper)
//!   into `artifacts/<doc-id>/` of the §8.3 layout, every artifact written
//!   canonical and strict-read back at the boundary, one §4.6 stage event
//!   per attempted stage; group stages and the total outcome land with
//!   cli-runner.2b.
#![forbid(unsafe_code)]

pub mod extract;
pub mod normalize;
pub mod rules;
pub mod segment;

mod dispatch;
mod registry_check;
mod run;
mod shell;

pub use dispatch::{CliExit, run_cli};
