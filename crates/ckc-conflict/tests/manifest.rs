//! Gate for task 0.10.10: one compact golden over the conflict manifest
//! byte-locks every `detect_all` artifact through its `content_hash`, alongside
//! round-trip determinism and JSON Schema stability for `ConflictManifest`.
//! Drift in any detector or the argument-graph builder shifts that entry's hash
//! and breaks `canonical`. Mirrors `crates/ckc-verify/tests/manifest.rs`.

use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;

use ckc_conflict::{CompileBundle, ConflictManifest, conflict_manifest, verify_all};
use ckc_core::canonical::{content_hash, to_canonical_bytes};

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

fn schema_dir() -> PathBuf {
    workspace_root().join("schemas")
}

fn golden_dir() -> PathBuf {
    workspace_root().join("schemas").join("golden")
}

// ---------------------------------------------------------------------------
// Assertion helpers (same contract as ckc-verify/tests/manifest.rs)
// ---------------------------------------------------------------------------

fn check_golden<T: Serialize>(fixture: &T, stem: &str) {
    let bytes = to_canonical_bytes(fixture);
    let path = golden_dir().join(format!("{stem}.json"));
    let golden =
        std::fs::read(&path).unwrap_or_else(|e| panic!("read golden {}: {e}", path.display()));
    assert!(
        bytes == golden,
        "canonical bytes mismatch for {stem} (got {} bytes, golden {} bytes)",
        bytes.len(),
        golden.len()
    );
}

fn check_roundtrip<T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug>(
    fixture: &T,
    stem: &str,
) {
    let bytes1 = to_canonical_bytes(fixture);
    let hash1 = content_hash(fixture);
    let rt: T =
        serde_json::from_slice(&bytes1).unwrap_or_else(|e| panic!("deserialize {stem}: {e}"));
    let bytes2 = to_canonical_bytes(&rt);
    let hash2 = content_hash(&rt);
    assert_eq!(bytes1, bytes2, "bytes differ after roundtrip for {stem}");
    assert_eq!(hash1, hash2, "hash differs after roundtrip for {stem}");
    assert_eq!(*fixture, rt, "value differs after roundtrip for {stem}");
}

fn check_schema<T: schemars::JsonSchema>(stem: &str) {
    let schema = schemars::schema_for!(T);
    let json = serde_json::to_string_pretty(&schema).unwrap() + "\n";
    let path = schema_dir().join(format!("{stem}.schema.json"));
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read schema {}: {e}", path.display()));
    assert!(json == golden, "schema mismatch for {stem}");
}

// ---------------------------------------------------------------------------
// Golden fixture: the real toy conflict manifest, so the golden records the
// content hash of every detected conflict + argument-graph artifact rather than
// a stand-in.
// ---------------------------------------------------------------------------

fn golden_conflict_manifest() -> ConflictManifest {
    let bundle = CompileBundle::load_toy();
    conflict_manifest(&bundle, &verify_all(&bundle))
}

// ---------------------------------------------------------------------------
// Test macro
// ---------------------------------------------------------------------------

macro_rules! golden_suite {
    ($mod_name:ident, $type:ty, $fixture_fn:ident, $stem:literal) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn canonical() {
                check_golden(&$fixture_fn(), $stem);
            }

            #[test]
            fn roundtrip() {
                check_roundtrip::<$type>(&$fixture_fn(), $stem);
            }

            #[test]
            fn schema() {
                check_schema::<$type>($stem);
            }
        }
    };
}

golden_suite!(
    gs_conflict_manifest,
    ConflictManifest,
    golden_conflict_manifest,
    "conflict_manifest"
);

// ---------------------------------------------------------------------------
// Integrity: every manifest entry names a real committed `certs/` artifact (task
// 0.10.9), so the compact hash-lock and the on-disk conflict tree stay coupled —
// a path the golden happily records yet that no file backs is caught.
// ---------------------------------------------------------------------------

#[test]
fn manifest_paths_resolve_to_committed_files() {
    let manifest = golden_conflict_manifest();
    assert_eq!(manifest.0.len(), 4, "3 conflicts + 1 argument graph");
    let root = workspace_root();
    for entry in &manifest.0 {
        let path = root.join(&entry.artifact_path);
        assert!(
            path.is_file(),
            "manifest path {} has no committed file at {}",
            entry.artifact_path,
            path.display()
        );
    }
}

// ---------------------------------------------------------------------------
// Regeneration: `cargo test -p ckc-conflict --test manifest -- --ignored`
// ---------------------------------------------------------------------------

fn write_type<T: Serialize + schemars::JsonSchema>(fixture: &T, stem: &str) {
    std::fs::write(
        golden_dir().join(format!("{stem}.json")),
        to_canonical_bytes(fixture),
    )
    .unwrap();
    let schema = schemars::schema_for!(T);
    std::fs::write(
        schema_dir().join(format!("{stem}.schema.json")),
        serde_json::to_string_pretty(&schema).unwrap() + "\n",
    )
    .unwrap();
}

#[test]
#[ignore]
fn regenerate() {
    std::fs::create_dir_all(golden_dir()).unwrap();
    write_type(&golden_conflict_manifest(), "conflict_manifest");
}
