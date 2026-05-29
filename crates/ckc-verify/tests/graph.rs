//! Gate for task 0.9.12: golden-lock the Phase-0 certificate graph and assert
//! its structural invariants.
//!
//! The graph is built over the full Phase-0 certificate set — the 10 solver/cvc5
//! certs from [`certificates`] plus the SHACL cert (task 0.9.10), and the
//! mirrored 11 witnesses — so the golden records the same graph that 0.9.14's
//! `verify_all` reproduces. The two structural tests are the readable
//! cross-check beside the opaque golden bytes: every certificate is reachable as
//! a node, and the provenance graph is acyclic (Kahn's algorithm drains it).
//!
//! Harness mirrors `crates/ckc-verify/tests/recorded.rs`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;

use ckc_compile::CompileBundle;
use ckc_core::canonical::{content_hash, to_canonical_bytes};
use ckc_verify::{
    Certificate, CertificateGraph, ExecutionWitness, build_graph, certificates, shacl_certificate,
    witnesses,
};

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
// Golden fixture: the full Phase-0 certificate graph (11 certs, 11 witnesses).
// ---------------------------------------------------------------------------

/// The full Phase-0 certificate set and mirrored witness set: the 10
/// solver/cvc5 entries plus the in-process SHACL pair.
fn full_certs_witnesses() -> (Vec<Certificate>, Vec<ExecutionWitness>) {
    let bundle = CompileBundle::load_toy();
    let mut certs = certificates(&bundle);
    let mut ws = witnesses(&bundle);
    let (shacl_cert, shacl_witness) = shacl_certificate(&bundle);
    certs.push(shacl_cert);
    ws.push(shacl_witness);
    (certs, ws)
}

fn golden_certificate_graph() -> CertificateGraph {
    let bundle = CompileBundle::load_toy();
    let (certs, ws) = full_certs_witnesses();
    build_graph(&bundle, &certs, &ws)
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
    gs_certificate_graph,
    CertificateGraph,
    golden_certificate_graph,
    "certificate_graph"
);

// ---------------------------------------------------------------------------
// Structural invariants
// ---------------------------------------------------------------------------

#[test]
fn every_certificate_appears_as_a_node() {
    let (certs, _) = full_certs_witnesses();
    let graph = golden_certificate_graph();
    let node_hashes: HashSet<&str> = graph
        .nodes
        .iter()
        .map(|n| n.artifact_hash.as_str())
        .collect();

    for cert in &certs {
        let hash = content_hash(cert);
        assert!(
            node_hashes.contains(hash.as_str()),
            "certificate {} missing from graph nodes",
            cert.certificate_id.as_str()
        );
    }
}

#[test]
fn graph_is_acyclic() {
    let graph = golden_certificate_graph();

    // Kahn's algorithm: repeatedly drain in-degree-0 nodes. A cycle leaves some
    // node permanently above in-degree 0, so fewer than all nodes drain.
    let mut indegree: HashMap<&str, usize> = graph
        .nodes
        .iter()
        .map(|n| (n.artifact_hash.as_str(), 0usize))
        .collect();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        *indegree
            .get_mut(edge.to.as_str())
            .expect("edge target is a node") += 1;
    }

    let mut queue: Vec<&str> = indegree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(n, _)| *n)
        .collect();
    let mut drained = 0usize;
    while let Some(node) = queue.pop() {
        drained += 1;
        for &succ in adj.get(node).map(Vec::as_slice).unwrap_or_default() {
            let d = indegree.get_mut(succ).unwrap();
            *d -= 1;
            if *d == 0 {
                queue.push(succ);
            }
        }
    }

    assert_eq!(
        drained,
        indegree.len(),
        "certificate graph has a cycle (drained {drained} of {} nodes)",
        indegree.len()
    );
}

// ---------------------------------------------------------------------------
// Regeneration: `cargo test -p ckc-verify --test graph -- --ignored regenerate`
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
    write_type(&golden_certificate_graph(), "certificate_graph");
}
