//! Gate for task 0.8.15: one compact golden over the portfolio hash manifest
//! byte-locks every `compile_all` target through its `content_hash`, alongside
//! round-trip determinism and JSON Schema stability for `PortfolioManifest`.
//! Emitter drift in any target shifts that target's hash and breaks `canonical`.

use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;

use ckc_compile::{CompileBundle, PortfolioManifest, portfolio_manifest};
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
// Assertion helpers (same contract as ckc-core/tests/golden.rs)
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
// Golden fixture: the real toy portfolio manifest, so the golden records the
// content hash of every emitted target rather than a hand-authored stand-in.
// ---------------------------------------------------------------------------

fn golden_compile_portfolio_manifest() -> PortfolioManifest {
    portfolio_manifest(&CompileBundle::load_toy())
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
    gs_compile_portfolio_manifest,
    PortfolioManifest,
    golden_compile_portfolio_manifest,
    "compile_portfolio_manifest"
);

// ---------------------------------------------------------------------------
// Regeneration: `cargo test -p ckc-compile --test manifest -- --ignored`
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
    write_type(
        &golden_compile_portfolio_manifest(),
        "compile_portfolio_manifest",
    );
}
