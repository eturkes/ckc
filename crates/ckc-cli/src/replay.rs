//! Replay diagnostic (SPEC 18): the structured outcome of re-deriving a committed
//! [`RunManifest`](crate::manifest::RunManifest) and comparing every emitted
//! artifact's `content_hash` against the recorded run. [`ReplayReport`] /
//! [`ReplayMismatch`] are the serializable result; [`compare_manifests`] is the
//! pure diff and [`run_replay`] drives the `ckc replay` command (re-run the
//! pipeline, diff, persist the report, fail on mismatch).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context as _, bail};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::emit::write_artifact;
use crate::manifest::RunManifest;
use crate::pipeline;
use ckc_core::canonical::{ContentHash, to_canonical_bytes};
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

/// Diff a committed `expected` [`RunManifest`] against a freshly re-derived
/// `actual` one (SPEC 18). Pure. Indexes both entry lists by `artifact_path`
/// (unique within a run — each stage emits distinct paths) and emits one
/// [`ReplayMismatch`] per path whose `content_hash` differs or that appears on
/// only one side (`None` hash for the absent side), sorted by `artifact_path`.
/// `n_matched` counts the `expected` entries reproduced byte-for-byte;
/// `status` is `Passed` exactly when no mismatch exists.
pub fn compare_manifests(expected: &RunManifest, actual: &RunManifest) -> ReplayReport {
    let expected_by_path: BTreeMap<&str, &ContentHash> = expected
        .entries
        .iter()
        .map(|e| (e.artifact_path.as_str(), &e.content_hash))
        .collect();
    let actual_by_path: BTreeMap<&str, &ContentHash> = actual
        .entries
        .iter()
        .map(|e| (e.artifact_path.as_str(), &e.content_hash))
        .collect();

    // Union of every path, iterated in sorted order so `mismatches` is sorted.
    let all_paths: BTreeSet<&str> = expected_by_path
        .keys()
        .chain(actual_by_path.keys())
        .copied()
        .collect();

    let mut mismatches = Vec::new();
    let mut n_matched = 0usize;
    for path in all_paths {
        let expected_hash = expected_by_path.get(path).copied();
        let actual_hash = actual_by_path.get(path).copied();
        match (expected_hash, actual_hash) {
            (Some(e), Some(a)) if e == a => n_matched += 1,
            (e, a) => mismatches.push(ReplayMismatch {
                artifact_path: path.to_string(),
                expected_hash: e.cloned(),
                actual_hash: a.cloned(),
            }),
        }
    }

    let status = if mismatches.is_empty() {
        ReplayStatus::Passed
    } else {
        ReplayStatus::Failed
    };
    ReplayReport {
        manifest_command: expected.command.clone(),
        status,
        n_entries: expected.entries.len(),
        n_matched,
        mismatches,
    }
}

/// Replay the run recorded in the committed manifest at `manifest_path` (SPEC
/// 18): re-derive the Phase-0 pipeline under `out_dir`, diff it against the
/// recorded manifest with [`compare_manifests`], persist the [`ReplayReport`] to
/// `out_dir/replay_report.json`, and fail when any artifact diverged.
///
/// The expected manifest's `command` must equal [`pipeline::DEMO_COMMAND`]:
/// Phase-0 replay serves only the research-kernel scenario, so this
/// command-equality check stands in for scenario selection. The report is written
/// before the mismatch `bail!`, so a failed replay still leaves its diagnostic on
/// disk for review.
pub fn run_replay(manifest_path: &Path, out_dir: &Path) -> anyhow::Result<ReplayReport> {
    let bytes = std::fs::read(manifest_path)
        .with_context(|| format!("read replay manifest {}", manifest_path.display()))?;
    let expected: RunManifest = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse replay manifest {}", manifest_path.display()))?;
    if expected.command != pipeline::DEMO_COMMAND {
        bail!(
            "unsupported replay manifest command {:?}; Phase-0 replay serves {:?}",
            expected.command,
            pipeline::DEMO_COMMAND
        );
    }

    let actual = pipeline::run_pipeline("research-kernel", out_dir)?;
    let report = compare_manifests(&expected, &actual);
    write_artifact(out_dir, "replay_report.json", &to_canonical_bytes(&report))?;

    if report.status != ReplayStatus::Passed {
        bail!(
            "replay mismatch: {} of {} artifacts diverged ({})",
            report.mismatches.len(),
            report.n_entries,
            report
                .mismatches
                .iter()
                .map(|m| m.artifact_path.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(report)
}
