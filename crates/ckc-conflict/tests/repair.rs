//! Repair-candidate gate for task 0.10.3.
//!
//! Each per-class generator produces exactly the SPEC 15 minimal repairs the
//! authored `conflicts.json` curates — `add_priority`/`add_exception` for the norm
//! contradiction, `refine_condition`/`change_hit_policy` for the decision-table
//! overlap, `add_temporal_guard` for the temporal violation — and the generated set
//! agrees with the curated oracle byte-for-byte under canonical serialization
//! (detection-agreement). The MaxSMT minimal-repair objective recovered from the
//! verification report is 1: one repair restores satisfiability.

use ckc_conflict::repair::{minimal_repair_objective, repair_candidates};
use ckc_conflict::{CompileBundle, Conflict, verify_all};
use ckc_core::canonical::to_canonical_bytes;
use serde_json::Value;

/// The three Phase-0 conflict classes that carry repair candidates.
const CONFLICT_TYPES: [&str; 3] = [
    "norm_contradiction",
    "decision_table_overlap",
    "temporal_violation",
];

/// The authored `conflicts.json` entry for `conflict_type` — the content oracle a
/// detector reuses (here the source of the `rationale` prose and the agreement
/// target).
fn oracle(conflict_type: &str) -> Conflict {
    CompileBundle::load_toy()
        .conflicts
        .into_iter()
        .find(|c| c.conflict_type == conflict_type)
        .unwrap_or_else(|| panic!("toy conflicts.json carries a {conflict_type} entry"))
}

/// The `type` of each generated repair candidate, in order.
fn repair_types(candidates: &[Value]) -> Vec<&str> {
    candidates
        .iter()
        .map(|rc| {
            rc.get("type")
                .and_then(Value::as_str)
                .expect("each repair candidate carries a string `type`")
        })
        .collect()
}

#[test]
fn norm_yields_priority_and_exception() {
    let oracle = oracle("norm_contradiction");
    let candidates = repair_candidates("norm_contradiction", &oracle);
    assert_eq!(
        repair_types(&candidates),
        vec!["add_priority", "add_exception"]
    );
}

#[test]
fn decision_table_yields_refine_and_hit_policy() {
    let oracle = oracle("decision_table_overlap");
    let candidates = repair_candidates("decision_table_overlap", &oracle);
    assert_eq!(
        repair_types(&candidates),
        vec!["refine_condition", "change_hit_policy"]
    );
}

#[test]
fn temporal_yields_temporal_guard() {
    let oracle = oracle("temporal_violation");
    let candidates = repair_candidates("temporal_violation", &oracle);
    assert_eq!(repair_types(&candidates), vec!["add_temporal_guard"]);
}

#[test]
fn generated_sets_match_authored_oracle_bytes() {
    for conflict_type in CONFLICT_TYPES {
        let oracle = oracle(conflict_type);
        let generated = repair_candidates(conflict_type, &oracle);
        assert_eq!(
            to_canonical_bytes(&generated),
            to_canonical_bytes(&oracle.repair_candidates),
            "generated {conflict_type} repair candidates diverge from the authored conflicts.json set"
        );
    }
}

#[test]
fn minimal_repair_objective_is_one() {
    let report = verify_all(&CompileBundle::load_toy());
    assert_eq!(
        minimal_repair_objective(&report),
        Some(1),
        "the MaxSMT minimal-repair objective recorded for the norm conflict is 1"
    );
}
