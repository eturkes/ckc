//! CKC conflict and inconsistency detection (SPEC 15).
//!
//! Consumes the emit-only compiler portfolio ([`CompileBundle`]) plus the
//! [`VerificationReport`] from `ckc-verify`, then (across task 0.10) links each
//! detected conflict to its real certificate/witness evidence ([`link`]),
//! generates source-revision repair candidates ([`repair`]), assembles the
//! Dung-style argument graph ([`argument`]), and runs the per-class detectors
//! ([`detect`]). Like [`VerificationReport`], every output composes
//! already-normalized artifacts, so the aggregate types carry no `Normalize`
//! impl of their own — their determinism follows from each detector's.

pub mod argument;
pub mod detect;
pub mod link;
pub mod repair;

pub use ckc_compile::CompileBundle;
pub use ckc_core::canonical::{ContentHash, content_hash};
pub use ckc_core::compile::CompileDiagnostic;
pub use ckc_core::verify::{ArgumentGraph, Conflict};
pub use ckc_verify::{VerificationReport, verify_all};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The Phase-0 conflict report for one compile bundle (SPEC 15): every detected
/// [`Conflict`], the [`ArgumentGraph`]s backing the defeasible conflicts, and any
/// backend-disagreement diagnostics passed through from the verification
/// portfolio (SPEC 15.1 #16). A composition of already-normalized artifacts,
/// mirroring [`VerificationReport`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConflictReport {
    pub conflicts: Vec<Conflict>,
    pub argument_graphs: Vec<ArgumentGraph>,
    pub diagnostics: Vec<CompileDiagnostic>,
}

/// One row of the [`ConflictManifest`]: a single committed `certs/` conflict or
/// argument-graph artifact, tagged by kind and byte-locked through the
/// `content_hash` of its canonical form. Mirrors `ckc_verify::VerificationEntry`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConflictManifestEntry {
    pub artifact_kind: String,
    pub artifact_path: String,
    pub content_hash: ContentHash,
}

/// The Phase-0 conflict manifest (SPEC 15, 25): every detected conflict and
/// argument-graph artifact paired with its committed `certs/` path and content
/// hash. One compact golden over this manifest byte-locks the whole conflict
/// artifact set through its hashes. Transparent newtype — serializes as the bare
/// array of entries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct ConflictManifest(pub Vec<ConflictManifestEntry>);

/// Compose the Phase-0 conflict report for `bundle` from its verification `report`
/// (SPEC 15), orchestrating the task-0.10.5–0.10.7 detectors:
///
/// - `conflicts`: the norm-contradiction, decision-table-defect, and
///   temporal-violation passes ([`detect`]) — 3 over the toy bundle;
/// - `argument_graphs`: the single Dung graph backing the defeasible conflicts
///   ([`argument::build_argument_graph`]) — 1;
/// - `diagnostics`: the backend-disagreement diagnostics passed through from the
///   verification portfolio (SPEC 15.1 #16) — 0 on the real oracle, where the
///   portfolio agrees; the synthetic-divergence branch is already covered in
///   `ckc-verify`'s portfolio check.
///
/// Each conflict and the argument graph is already normalized by its builder, so the
/// report composes normalized artifacts and carries no `Normalize` impl of its own,
/// mirroring [`verify_all`].
pub fn detect_all(bundle: &CompileBundle, report: &VerificationReport) -> ConflictReport {
    let mut conflicts = detect::detect_norm_contradiction(bundle, report);
    conflicts.extend(detect::detect_decision_table_defects(bundle, report));
    conflicts.extend(detect::detect_temporal_violation(bundle, report));
    ConflictReport {
        conflicts,
        argument_graphs: vec![argument::build_argument_graph(bundle)],
        diagnostics: report.disagreements.clone(),
    }
}

/// Build the conflict manifest for `bundle` from a live [`detect_all`] run over its
/// verification `report`, pairing each artifact with the canonical `certs/` path task
/// 0.10.9 commits it to and its [`content_hash`]: every detected conflict
/// (`certs/conflicts/<conflict_id>.json`) first, then each argument graph
/// (`certs/argument_graphs/<argument_graph_id>.json`), in that deterministic order.
/// One compact golden over this manifest byte-locks the whole conflict artifact set
/// through its hashes, so a drift in any detector surfaces as a manifest mismatch.
/// Mirrors `ckc_verify::verification_manifest`.
pub fn conflict_manifest(bundle: &CompileBundle, report: &VerificationReport) -> ConflictManifest {
    let detected = detect_all(bundle, report);
    let mut entries = Vec::new();
    for conflict in &detected.conflicts {
        entries.push(ConflictManifestEntry {
            artifact_kind: "conflict".to_string(),
            artifact_path: format!("certs/conflicts/{}.json", conflict.conflict_id.as_str()),
            content_hash: content_hash(conflict),
        });
    }
    for graph in &detected.argument_graphs {
        entries.push(ConflictManifestEntry {
            artifact_kind: "argument_graph".to_string(),
            artifact_path: format!(
                "certs/argument_graphs/{}.json",
                graph.argument_graph_id.as_str()
            ),
            content_hash: content_hash(graph),
        });
    }
    ConflictManifest(entries)
}
