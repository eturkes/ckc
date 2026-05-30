//! Run-pipeline stages (SPEC 18): load the bundle, then drive compile / verify /
//! conflicts / substrate emission and the `demo` orchestration. Task 0.11.1
//! lands the load entry point and the stage signatures; tasks 0.11.2–0.11.6
//! fill in each stage body.

use std::path::Path;

use anyhow::bail;

use crate::emit::write_artifact;
use crate::manifest::{RunManifest, RunManifestEntry};
use ckc_compile::{ARTIFACT_PATHS, CompileBundle, compile_all, portfolio_manifest};
use ckc_core::canonical::to_canonical_bytes;
use ckc_verify::{VerificationReport, verification_manifest, verify_all};

/// The recorded cvc5 proof artifact, embedded so [`run_verify`] can place it
/// beside the `cert_cvc5_norm_conflict` certificate that references its content
/// hash. It is recorded evidence (reached via the cert's `proof_artifact_hashes`),
/// not a `verify_all` artifact, so it carries no run-manifest entry of its own.
const CVC5_PROOF: &[u8] =
    include_bytes!("../../../examples/research_kernel/fixtures/cvc5_norm_conflict.proof");

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
///
/// Writes each [`compile_all`] target's `artifact_text` to its canonical
/// [`ARTIFACT_PATHS`] slot under `out_dir`, returning one `compile`-stage
/// [`RunManifestEntry`] per target. Each entry carries the `content_hash` that
/// [`portfolio_manifest`] byte-locks its target with, so the run manifest pins
/// every emitted artifact by hash.
pub fn run_compile(
    bundle: &CompileBundle,
    out_dir: &Path,
) -> anyhow::Result<Vec<RunManifestEntry>> {
    let targets = compile_all(bundle);
    let manifest = portfolio_manifest(bundle);
    let mut entries = Vec::with_capacity(targets.len());
    for ((target, rel), entry) in targets.iter().zip(ARTIFACT_PATHS).zip(manifest.0) {
        write_artifact(out_dir, rel, target.artifact_text.as_bytes())?;
        entries.push(RunManifestEntry {
            stage: "compile".to_string(),
            artifact_kind: "compiled_target".to_string(),
            artifact_path: rel.to_string(),
            content_hash: entry.content_hash,
        });
    }
    Ok(entries)
}

/// Verify stage (task 0.11.3): emit the SPEC-12/13 verification artifact set under
/// `out_dir`, returning the in-memory report alongside its run-manifest entries.
///
/// Writes each [`Certificate`](ckc_verify::Certificate) and
/// [`ExecutionWitness`](ckc_verify::ExecutionWitness) as `to_canonical_bytes` to
/// its `certs/{certificates,witnesses}/<id>.json` slot, the certificate graph and
/// assurance seed to their `certs/` files, and the recorded [`CVC5_PROOF`] beside
/// them — mirroring task 0.9.15's `report_artifacts` layout byte-for-byte. The
/// returned `verify`-stage entries come from [`verification_manifest`], so each
/// entry's `content_hash` byte-locks the file emitted at its path (24 entries: 11
/// certificates, 11 witnesses, the graph, the assurance seed); the proof copy is
/// recorded evidence and carries no entry.
pub fn run_verify(
    bundle: &CompileBundle,
    out_dir: &Path,
) -> anyhow::Result<(VerificationReport, Vec<RunManifestEntry>)> {
    let report = verify_all(bundle);
    for cert in &report.certificates {
        let rel = format!("certs/certificates/{}.json", cert.certificate_id.as_str());
        write_artifact(out_dir, &rel, &to_canonical_bytes(cert))?;
    }
    for witness in &report.witnesses {
        let rel = format!("certs/witnesses/{}.json", witness.witness_id.as_str());
        write_artifact(out_dir, &rel, &to_canonical_bytes(witness))?;
    }
    write_artifact(
        out_dir,
        "certs/certificate_graph.json",
        &to_canonical_bytes(&report.graph),
    )?;
    write_artifact(
        out_dir,
        "certs/assurance_seed.json",
        &to_canonical_bytes(&report.assurance),
    )?;
    write_artifact(out_dir, "certs/cvc5_norm_conflict.proof", CVC5_PROOF)?;

    let entries = verification_manifest(bundle)
        .0
        .into_iter()
        .map(|e| RunManifestEntry {
            stage: "verify".to_string(),
            artifact_kind: e.artifact_kind,
            artifact_path: e.artifact_path,
            content_hash: e.content_hash,
        })
        .collect();
    Ok((report, entries))
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
