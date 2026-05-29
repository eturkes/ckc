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
