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
//! `stage-extract.1` opens the extract stage ([`extract`]): html5ever-parsed
//! fixture HTML walked into an enveloped §4.5 [`ckc_core::SourceGraph`] —
//! sections/paragraphs/lists spanned with {node,span} regions, parse errors
//! and unknown flow content as `extraction_uncertain` residuals (tables ride
//! that path until `stage-extract.2`).
#![forbid(unsafe_code)]

pub mod extract;

mod dispatch;
mod registry_check;
mod shell;

pub use dispatch::{CliExit, run_cli};
