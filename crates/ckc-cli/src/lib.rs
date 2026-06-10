//! ckc-cli — the `ckc` binary (SPEC §3): pipeline stages, runner,
//! trace/report/replay, registry check. Built up unit by unit;
//! `cli-runner.1.1` lands the command shell: dispatch over the §3
//! four-command surface with validated argument shapes, the once-wired CLI
//! invariants (containment-guarded writes, §4.6 JSONL event/diagnostic
//! streams, exactly one §4.4 total operation result, outcome-mapped exit
//! codes), and pending command bodies returning typed `unsupported` results.
//! `cli-runner.1.2` implements `registry check`: the §8.4 registry surface
//! plus experiment-referenced gold documents strict-loaded from the
//! invocation root and validated as one cross-referenced set, every load
//! failure and finding a §7.4 `schema_invalid` diagnostic (§8.5 item 2).
//! `stage-extract.1`/`.2` land the extract stage ([`extract`]): html5ever-
//! parsed fixture HTML walked into an enveloped §4.5
//! [`ckc_core::SourceGraph`] — sections/paragraphs/lists spanned with
//! {node,span} regions, tables as literal cell grids (`row`/`col`/`header`
//! attrs; rejected tables withheld as `table_structure_uncertain`), parse
//! errors and unknown flow content as `extraction_uncertain` residuals.
//! `stage-segment` adds the segment stage ([`segment`]): the graph's spans
//! classified in reading order into §5 ClinicalSegments (cq, recommendation,
//! exception, definition, table-row, evidence, metadata), misses as
//! `segmentation_boundary_error` residuals. `stage-normalize.1a` opens the
//! normalize stage ([`normalize`]): the §5 lexicon authority
//! `corpus/lexicon/ja_core.yaml` strict-loaded and validated into the typed
//! [`normalize::Lexicon`] — semantic_ja surfaces with `surfaces[0]` the
//! representative, §5-coherent intervals, raw-byte content hash for
//! manifests. `stage-normalize.1b` adds mention binding
//! ([`normalize::bind_segments`]): recommendation/exception spans scanned
//! longest-match for concept mentions, singleton candidate sets binding
//! `exact`/`synonym`, shared surfaces binding `ambiguous` with a
//! `terminology_ambiguous` Ambiguity record. `stage-normalize.1d` adds
//! statement building ([`normalize::clinical_ir`]): the binding core run
//! per segment, slot readings (binding namespaces; verb, modality, and
//! certainty scans) building at most one §5 ClinicalStatement per
//! recommendation segment, misses and ambiguities withholding it as §7.4
//! records.
#![forbid(unsafe_code)]

pub mod extract;
pub mod normalize;
pub mod segment;

mod dispatch;
mod registry_check;
mod shell;

pub use dispatch::{CliExit, run_cli};
