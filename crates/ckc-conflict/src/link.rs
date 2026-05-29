//! Evidence-linkage resolver (task 0.10.2): replaces the placeholder
//! cross-references authored in the toy `conflicts.json` with the real
//! `ckc_verify::verify_all` certificate and execution-witness artifacts.

use ckc_core::canonical::{ContentHash, content_hash};
use ckc_core::id::WitnessId;

use crate::VerificationReport;

/// The `nf-…` `witness_id` of the
/// [`ExecutionWitness`](ckc_core::artifact::ExecutionWitness) checked by the
/// certificate `cert_id` — the real witness that replaces the toy `conflicts.json`
/// placeholder `witness:"witness_norm_conflict"`. `None` when no recorded witness
/// carries that certificate. Each Phase-0 solver witness links exactly one
/// certificate, so the first match is unambiguous; mirrors the `by_cert` lookup of
/// the `ckc-verify` witness gate.
pub fn witness_for_cert(report: &VerificationReport, cert_id: &str) -> Option<WitnessId> {
    report
        .witnesses
        .iter()
        .find(|w| w.certificate_ids.iter().any(|c| c.as_str() == cert_id))
        .map(|w| w.witness_id.clone())
}

/// The sorted, deduplicated [`content_hash`] of each
/// [`Certificate`](ckc_core::verify::Certificate) whose `certificate_id` is in
/// `cert_ids` — the real minimal artifact set that replaces the toy
/// `conflicts.json` placeholder `minimal_artifact_set:["sha256:a00…"]`. Sorting and
/// dedup make the set order-independent and collapse any duplicate a repeated
/// `cert_id` would introduce, so the result is deterministic.
pub fn minimal_artifact_set(report: &VerificationReport, cert_ids: &[&str]) -> Vec<ContentHash> {
    let mut hashes: Vec<ContentHash> = report
        .certificates
        .iter()
        .filter(|c| cert_ids.iter().any(|id| *id == c.certificate_id.as_str()))
        .map(content_hash)
        .collect();
    hashes.sort();
    hashes.dedup();
    hashes
}
