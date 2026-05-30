//! Run-manifest types: the deterministic record of every artifact a `ckc` run
//! emits (SPEC 2.2, 18). Mirrors the per-crate `*Manifest` shape
//! (`ckc_compile::PortfolioManifest`, `ckc_verify::VerificationManifest`) at the
//! run level — a flat, timestamp-free entry list, so two runs of the same
//! command hash identically (the CAS `stored_at_epoch` non-determinism noted in
//! agent memory is the field deliberately absent here).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_core::canonical::ContentHash;

/// One emitted artifact in a [`RunManifest`]: the pipeline stage that produced
/// it, its artifact kind, its run-relative path, and the `content_hash` that
/// byte-locks it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunManifestEntry {
    pub stage: String,
    pub artifact_kind: String,
    pub artifact_path: String,
    pub content_hash: ContentHash,
}

/// The deterministic manifest of a whole `ckc` run (SPEC 18): the command that
/// produced it, the `producer_version` (`CARGO_PKG_VERSION` at the construction
/// site), and every emitted artifact. Carries no timestamp, so repeated runs of
/// the same command hash identically.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunManifest {
    pub command: String,
    pub producer_version: String,
    pub entries: Vec<RunManifestEntry>,
}
