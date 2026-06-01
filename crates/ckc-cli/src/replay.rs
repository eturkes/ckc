//! Replay diagnostic types (SPEC 18): the structured outcome of re-deriving a
//! committed [`RunManifest`](crate::manifest::RunManifest) and comparing every
//! emitted artifact's `content_hash` against the recorded run. The comparison
//! logic (`compare_manifests`, `run_replay`) arrives in task 0.13.3; this module
//! defines the serializable report those functions produce.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_core::canonical::ContentHash;
use ckc_core::enums::ReplayStatus;

/// One artifact whose replayed `content_hash` diverged from the committed
/// manifest, or that appears on only one side. `expected_hash` is `None` when
/// the path appears only in the replayed run; `actual_hash` is `None` when it
/// appears only in the committed manifest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplayMismatch {
    pub artifact_path: String,
    pub expected_hash: Option<ContentHash>,
    pub actual_hash: Option<ContentHash>,
}

/// The structured result of a `ckc replay` run (SPEC 18): the replayed command,
/// the overall [`ReplayStatus`], how many committed entries were compared
/// (`n_entries`) and matched byte-for-byte (`n_matched`), and one
/// [`ReplayMismatch`] per divergent artifact. `Passed` with an empty
/// `mismatches` list is the deterministic-replay success witness.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReplayReport {
    pub manifest_command: String,
    pub status: ReplayStatus,
    pub n_entries: usize,
    pub n_matched: usize,
    pub mismatches: Vec<ReplayMismatch>,
}
