//! Gate for task 0.9.2: golden-lock the recorded solver-outcome oracle.
//!
//! One compact golden over `RecordedOutcomes` from `load_recorded_outcomes()`
//! byte-locks every Phase-0 verifier verdict — the 9 `compile_all` targets, the
//! cvc5 proof outcome, and the SHACL validation outcome. The load test is the
//! readable cross-check beside the opaque golden bytes: it asserts the oracle
//! decodes to 11 entries with the expected per-entry `(solver, status,
//! certificate_class)`, plus the two semantically load-bearing fields the triple
//! alone misses (the MaxSMT repair objective and the cvc5 proof flag).
//!
//! Harness mirrors `crates/ckc-compile/tests/manifest.rs`.

use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;

use ckc_core::canonical::{content_hash, to_canonical_bytes};
use ckc_verify::{
    CertificateClass, RecordedOutcomes, SolverId, VerdictStatus, load_recorded_outcomes,
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
// Golden fixture: the real recorded oracle, so the golden records the verdict
// of every verifier rather than a hand-authored stand-in.
// ---------------------------------------------------------------------------

fn golden_verifier_outcomes() -> RecordedOutcomes {
    load_recorded_outcomes()
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
    gs_verifier_outcomes,
    RecordedOutcomes,
    golden_verifier_outcomes,
    "verifier_outcomes"
);

// ---------------------------------------------------------------------------
// Load test: the oracle decodes to the expected 11 entries, in order.
// ---------------------------------------------------------------------------

#[test]
fn recorded_oracle_has_eleven_expected_entries() {
    use CertificateClass::{C4Executable, C6ProofObject, C7Kernel};
    use SolverId::{Alloy, Clingo, Cvc5, Lean, Shacl, Souffle, TlaSany, Z3};
    use VerdictStatus::{
        EmptyRelation, KernelChecked, NoCounterexample, Sat, Satisfiable, SemanticCheckPassed,
        Unsat, ViolationsFound,
    };

    // The 9 `compile_all` targets in ARTIFACT_PATHS order, then cvc5, then SHACL.
    let expected: [(SolverId, VerdictStatus, CertificateClass); 11] = [
        (Z3, Unsat, C4Executable),
        (Z3, Sat, C4Executable),
        (Z3, Sat, C4Executable),
        (Clingo, Satisfiable, C4Executable),
        (Clingo, Satisfiable, C4Executable),
        (Souffle, EmptyRelation, C4Executable),
        (Lean, KernelChecked, C7Kernel),
        (TlaSany, SemanticCheckPassed, C4Executable),
        (Alloy, NoCounterexample, C4Executable),
        (Cvc5, Unsat, C6ProofObject),
        (Shacl, ViolationsFound, C6ProofObject),
    ];

    let RecordedOutcomes(outcomes) = load_recorded_outcomes();
    assert_eq!(outcomes.len(), 11, "oracle must record 11 outcomes");

    for (i, (o, (solver, status, class))) in outcomes.iter().zip(expected).enumerate() {
        assert_eq!(o.solver, solver, "solver mismatch at entry {i}");
        assert_eq!(o.status, status, "status mismatch at entry {i}");
        assert_eq!(
            o.certificate_class, class,
            "certificate_class mismatch at entry {i}"
        );
    }

    // The two fields the (solver, status, class) triple cannot distinguish:
    // entries 1 and 2 are both (Z3, Sat, C4) — only the MaxSMT objective on the
    // repair target separates them — and cvc5 alone carries a proof.
    assert_eq!(
        outcomes[2].objective,
        Some(1),
        "repair_maxsmt must record objective 1"
    );
    assert!(
        outcomes[9].proof_present,
        "cvc5 outcome must record a proof"
    );
}

// ---------------------------------------------------------------------------
// Regeneration: `cargo test -p ckc-verify --test recorded -- --ignored`
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
    write_type(&golden_verifier_outcomes(), "verifier_outcomes");
}
