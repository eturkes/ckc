//! Artifact emission: write a run's artifact bytes under its output directory,
//! creating intermediate directories. Every pipeline stage routes file writes
//! through [`write_artifact`] so path handling stays in one place.

use std::path::Path;

use anyhow::Context as _;

/// Write `bytes` to `rel_path` under `out_dir`, creating any missing parent
/// directories. `rel_path` is a run-relative path such as
/// `logic/smt/norm_conflict.smt2` or `certs/certificates/<id>.json`.
pub fn write_artifact(out_dir: &Path, rel_path: &str, bytes: &[u8]) -> anyhow::Result<()> {
    let path = out_dir.join(rel_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create dir {}", parent.display()))?;
    }
    std::fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}
