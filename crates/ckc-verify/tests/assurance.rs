//! Gate for task 0.9.13: golden-lock the Phase-0 assurance seed and assert its
//! GSN structural invariants.
//!
//! The seed is built over the full Phase-0 certificate set — the 10 solver/cvc5
//! certs from [`certificates`] plus the SHACL cert (task 0.9.10) — so the golden
//! records the same seed that 0.9.14's `verify_all` reproduces. The two
//! structural tests are the readable cross-check beside the opaque golden bytes:
//! the root goal carries children, and every strategy node cites evidence.
//!
//! Harness mirrors `crates/ckc-verify/tests/graph.rs`.

use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;

use ckc_compile::CompileBundle;
use ckc_core::canonical::{content_hash, to_canonical_bytes};
use ckc_verify::{AssuranceSeed, Certificate, assurance_seed, certificates, shacl_certificate};

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
// Golden fixture: the Phase-0 assurance seed over the full certificate set.
// ---------------------------------------------------------------------------

/// The full Phase-0 certificate set: the 10 solver/cvc5 certs plus the in-process
/// SHACL grounding cert — the same set 0.9.14's `verify_all` feeds `assurance_seed`.
fn full_certs() -> Vec<Certificate> {
    let bundle = CompileBundle::load_toy();
    let mut certs = certificates(&bundle);
    let (shacl_cert, _) = shacl_certificate(&bundle);
    certs.push(shacl_cert);
    certs
}

fn golden_assurance_seed() -> AssuranceSeed {
    assurance_seed(&full_certs())
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
    gs_assurance_seed,
    AssuranceSeed,
    golden_assurance_seed,
    "assurance_seed"
);

// ---------------------------------------------------------------------------
// Structural invariants
// ---------------------------------------------------------------------------

#[test]
fn root_goal_has_children() {
    let AssuranceSeed(nodes) = golden_assurance_seed();
    let root = nodes.first().expect("assurance seed has a root node");
    assert_eq!(root.node_type, "goal", "first node is the root goal");
    assert!(
        !root.children.is_empty(),
        "root goal {} has no children",
        root.node_id.as_str()
    );
}

#[test]
fn every_strategy_node_cites_evidence() {
    let AssuranceSeed(nodes) = golden_assurance_seed();
    let strategies: Vec<_> = nodes.iter().filter(|n| n.node_type == "strategy").collect();
    assert!(!strategies.is_empty(), "assurance seed has strategy nodes");
    for strategy in strategies {
        assert!(
            !strategy.evidence_artifact_ids.is_empty(),
            "strategy {} cites no evidence",
            strategy.node_id.as_str()
        );
    }
}

// ---------------------------------------------------------------------------
// Regeneration: `cargo test -p ckc-verify --test assurance -- --ignored regenerate`
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
    write_type(&golden_assurance_seed(), "assurance_seed");
}
