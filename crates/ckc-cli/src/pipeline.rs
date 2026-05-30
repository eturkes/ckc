//! Run-pipeline stages (SPEC 18): load the bundle, then drive compile / verify /
//! conflicts / substrate emission and the `demo` orchestration. Task 0.11.1
//! lands the load entry point and the stage signatures; tasks 0.11.2–0.11.6
//! fill in each stage body.

use std::path::Path;

use anyhow::bail;

use crate::manifest::{RunManifest, RunManifestEntry};
use ckc_compile::CompileBundle;
use ckc_verify::VerificationReport;

/// Load the [`CompileBundle`] for a run. Phase-0 serves only the committed
/// research-kernel toy bundle ([`CompileBundle::load_toy`]); any other path is a
/// not-yet-supported corpus and fails fast.
pub fn load_bundle(path: &str) -> anyhow::Result<CompileBundle> {
    match path {
        "examples/research_kernel" => Ok(CompileBundle::load_toy()),
        other => {
            bail!("unsupported bundle path {other:?}; Phase-0 serves examples/research_kernel")
        }
    }
}

/// Compile stage (task 0.11.2): emit the SPEC-14 target portfolio under `out_dir`.
pub fn run_compile(
    bundle: &CompileBundle,
    out_dir: &Path,
) -> anyhow::Result<Vec<RunManifestEntry>> {
    let _ = (bundle, out_dir);
    bail!("pending")
}

/// Verify stage (task 0.11.3): emit certificates, witnesses, the certificate
/// graph, and the assurance seed under `out_dir`, returning the in-memory report.
pub fn run_verify(
    bundle: &CompileBundle,
    out_dir: &Path,
) -> anyhow::Result<(VerificationReport, Vec<RunManifestEntry>)> {
    let _ = (bundle, out_dir);
    bail!("pending")
}

/// Conflicts stage (task 0.11.4): emit detected conflicts and argument graphs
/// under `out_dir`.
pub fn run_conflicts(
    bundle: &CompileBundle,
    report: &VerificationReport,
    out_dir: &Path,
) -> anyhow::Result<Vec<RunManifestEntry>> {
    let _ = (bundle, report, out_dir);
    bail!("pending")
}

/// Substrate stage (task 0.11.5): emit the RDF/SHACL terminology artifacts and
/// the retrieval results under `out_dir`.
pub fn run_substrate(
    bundle: &CompileBundle,
    out_dir: &Path,
) -> anyhow::Result<Vec<RunManifestEntry>> {
    let _ = (bundle, out_dir);
    bail!("pending")
}

/// Demo orchestration (task 0.11.6): run every stage under `out_dir`, assemble
/// the [`RunManifest`], and — when `replay` — prove the run hashes identically a
/// second time.
pub fn run_demo(scenario: &str, replay: bool, out_dir: &Path) -> anyhow::Result<RunManifest> {
    let _ = (scenario, replay, out_dir);
    bail!("pending")
}
