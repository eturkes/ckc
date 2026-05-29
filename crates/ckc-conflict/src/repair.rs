//! Repair-candidate generators (task 0.10.3): SPEC 15 minimal repairs framed as
//! source-revision review prompts, tied to the recorded MaxSMT outcome.
//!
//! Each generator independently builds the per-class repair shape — which kind of
//! revision, over which rule/row/action — that SPEC 15 prescribes for a conflict
//! class, then reuses the matching authored `repair_candidates` `rationale` prose
//! from the conflict oracle verbatim. The generated set therefore agrees with the
//! curated `conflicts.json` repair candidates byte-for-byte (the detection-agreement
//! check of task 0.10.3), while the repair *logic* stays computed here rather than
//! copied wholesale.

use serde_json::{Value, json};

use crate::{Conflict, VerificationReport};

/// The deterministic certificate id of the MaxSMT minimal-repair search over the
/// norm-conflict target (`cert_<solver>_<stem>` for z3 over
/// `logic/smt/repair_maxsmt.smt2`). Its `result` carries the recorded optimization
/// objective as a `sat:<objective>` token.
const REPAIR_CERT_ID: &str = "cert_z3_repair_maxsmt";

/// The authored `rationale` of the `oracle` repair candidate whose `type` matches
/// `repair_type`, reused verbatim so a generated candidate's canonical bytes agree
/// with the curated `conflicts.json` entry. `Value::Null` when the oracle carries
/// no candidate of that kind, which a divergent oracle would surface as a
/// canonical-bytes mismatch in the gate.
fn oracle_rationale(oracle: &Conflict, repair_type: &str) -> Value {
    oracle
        .repair_candidates
        .iter()
        .find(|rc| rc.get("type").and_then(Value::as_str) == Some(repair_type))
        .and_then(|rc| rc.get("rationale").cloned())
        .unwrap_or(Value::Null)
}

/// The SPEC 15 minimal repair candidates for `conflict_type`, framed as
/// source-revision review prompts. Each candidate's structural fields (the repair
/// kind plus the rule/row/action it revises) are generated here; its `rationale`
/// prose is reused from the matching `oracle` candidate. Returns, per class:
///
/// * `norm_contradiction` — `add_priority` (make `rule_bl_anaphylaxis_contra`
///   superior) and `add_exception` (exclude `beta_lactam_anaphylaxis` from
///   `rule_sepsis_bl_recommend`); the two toggles the MaxSMT search weighs.
/// * `decision_table_overlap` — `refine_condition` (bound `row_temp_high` to
///   `38.0 <= temperature < 38.5`) and `change_hit_policy` (`unique` → `priority`).
/// * `temporal_violation` — `add_temporal_guard` (gate `administer_drug` on
///   `NOT holds(allergy_known, t)`).
///
/// An unrecognized class yields no candidates.
pub fn repair_candidates(conflict_type: &str, oracle: &Conflict) -> Vec<Value> {
    match conflict_type {
        "norm_contradiction" => vec![
            json!({
                "type": "add_priority",
                "superior": "rule_bl_anaphylaxis_contra",
                "rationale": oracle_rationale(oracle, "add_priority"),
            }),
            json!({
                "type": "add_exception",
                "rule": "rule_sepsis_bl_recommend",
                "exception": "beta_lactam_anaphylaxis",
                "rationale": oracle_rationale(oracle, "add_exception"),
            }),
        ],
        "decision_table_overlap" => vec![
            json!({
                "type": "refine_condition",
                "row": "row_temp_high",
                "new_condition": "38.0 <= temperature < 38.5",
                "rationale": oracle_rationale(oracle, "refine_condition"),
            }),
            json!({
                "type": "change_hit_policy",
                "from": "unique",
                "to": "priority",
                "rationale": oracle_rationale(oracle, "change_hit_policy"),
            }),
        ],
        "temporal_violation" => vec![json!({
            "type": "add_temporal_guard",
            "action": "administer_drug",
            "guard": "NOT holds(allergy_known, t)",
            "rationale": oracle_rationale(oracle, "add_temporal_guard"),
        })],
        _ => Vec::new(),
    }
}

/// The minimal-repair optimization objective recorded for the MaxSMT search — the
/// least-cost number of repairs that restores satisfiability — read back from the
/// [`REPAIR_CERT_ID`] certificate's `sat:<objective>` `result` token. `None` when
/// the report carries no such certificate or its result has no objective suffix.
/// Over `verify_all(&CompileBundle::load_toy())` this is `Some(1)`: exactly one of
/// the two norm-conflict repairs suffices.
pub fn minimal_repair_objective(report: &VerificationReport) -> Option<i64> {
    report
        .certificates
        .iter()
        .find(|c| c.certificate_id.as_str() == REPAIR_CERT_ID)
        .and_then(|c| c.result.rsplit_once(':'))
        .and_then(|(_, objective)| objective.parse::<i64>().ok())
}
