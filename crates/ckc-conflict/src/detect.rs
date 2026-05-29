//! Per-class conflict detectors (tasks 0.10.5–0.10.7): SPEC 15.1
//! norm-contradiction, decision-table-defect, and temporal-violation passes,
//! each assembling one source-grounded [`crate::Conflict`].
//!
//! A detector computes the conflict's structural verdict — its `conflict_type`,
//! `classification`, `severity`, and the artifact/witness linkage from the
//! verification report — while reusing the curated prose, source spans, and
//! solver-evidence summary of the matching authored `conflicts.json` entry (the
//! content oracle). Detection drives emission: each detector first runs its scan
//! over the bundle and reports a conflict only when the scan succeeds, so the
//! oracle supplies wording rather than the decision to flag.

use ckc_core::clinical::Rule;
use ckc_core::enums::{ConflictClassification, RecommendationDirection, RuleKind, Severity};
use ckc_core::nf::normalize_all;

use crate::{CompileBundle, Conflict, VerificationReport, argument, link, repair};

/// The four norm-contradiction certificate keys (SPEC 15.1 #1/#2): the z3, cvc5,
/// and Lean checks over the norm-conflict target plus the clingo defeasible check —
/// the deterministic `cert_<solver>_<stem>` ids the 0.9 builder emits.
/// [`link::minimal_artifact_set`] resolves them to the real certificate content
/// hashes that replace the toy `conflicts.json` placeholder set; the first,
/// `cert_z3_norm_conflict`, also names the primary witness.
const NORM_CERT_KEYS: [&str; 4] = [
    "cert_z3_norm_conflict",
    "cert_cvc5_norm_conflict",
    "cert_lean_norm_conflict",
    "cert_clingo_defeasible",
];

/// The authored `conflicts.json` entry whose `conflict_type` matches — the content
/// oracle a detector reuses for its curated `source_spans`, `confidence`,
/// `normalized_view`, `solver_evidence`, and JA/EN review questions. Panics when
/// the committed fixture stops carrying that class, a build-time bug mirroring
/// [`CompileBundle::load_toy`] and `ckc_compile::find_rule`.
fn conflict_oracle<'a>(bundle: &'a CompileBundle, conflict_type: &str) -> &'a Conflict {
    bundle
        .conflicts
        .iter()
        .find(|c| c.conflict_type == conflict_type)
        .unwrap_or_else(|| panic!("toy bundle must contain a {conflict_type} conflict"))
}

/// The defeasible for/against rule pair sharing one `norm.action.target_concept`
/// (SPEC 15.1 #1/#2): a recommendation (`direction = for`) and a contraindication
/// (`direction = against`) that project opposite directions onto the same action
/// target under a satisfiable shared context. Returns `(for_rule, against_rule)`,
/// or `None` when no such pair exists. Over the toy bundle this is
/// `(rule_sepsis_bl_recommend, rule_bl_anaphylaxis_contra)`, both targeting
/// `concept_beta_lactam`. Exposed so the detection scan is checkable independently
/// of the curated oracle (the task-0.10.5 detection-agreement gate).
pub fn norm_contradiction_pair(bundle: &CompileBundle) -> Option<(&Rule, &Rule)> {
    // Both endpoints of a norm contradiction are defeasible rules carrying a
    // clinical norm; each projects a direction onto a norm action target.
    let norm_rules: Vec<&Rule> = bundle
        .rules
        .iter()
        .filter(|r| r.kind == RuleKind::Defeasible && r.norm.is_some())
        .collect();
    for &for_rule in &norm_rules {
        let for_norm = for_rule
            .norm
            .as_ref()
            .expect("filtered to norm-carrying rules");
        if for_norm.direction != RecommendationDirection::For {
            continue;
        }
        for &against_rule in &norm_rules {
            let against_norm = against_rule
                .norm
                .as_ref()
                .expect("filtered to norm-carrying rules");
            if against_norm.direction == RecommendationDirection::Against
                && against_norm.action.target_concept == for_norm.action.target_concept
            {
                return Some((for_rule, against_rule));
            }
        }
    }
    None
}

/// Detect the norm contradiction (SPEC 15.1 #1/#2): a defeasible recommendation and
/// contraindication projecting opposite directions onto the same action target
/// under a satisfiable shared context.
///
/// Runs [`norm_contradiction_pair`] over `bundle.rules`; with no such pair the scan
/// reports nothing. On a hit it assembles one [`Conflict`] with the computed verdict
/// — `conflict_type = "norm_contradiction"`, `classification = TrueConflict`,
/// `severity = High` — and the real evidence linkage from `report`: the primary
/// witness behind `cert_z3_norm_conflict` ([`link::witness_for_cert`]) and the
/// [`NORM_CERT_KEYS`] certificate hashes ([`link::minimal_artifact_set`]), both
/// replacing the toy `conflicts.json` placeholders. The curated `source_spans`,
/// `confidence`, `normalized_view`, `solver_evidence`, and JA/EN review questions
/// come from the matching oracle; the source-revision [`repair::repair_candidates`]
/// and the normalized Dung [`argument::build_argument_graph`] back the conflict.
/// [`normalize_all`] then assigns the content-derived `nf-…` `conflict_id`.
pub fn detect_norm_contradiction(
    bundle: &CompileBundle,
    report: &VerificationReport,
) -> Vec<Conflict> {
    // Detection drives emission: report a conflict only when the scan finds the
    // for/against pair, not merely because the oracle carries the class.
    if norm_contradiction_pair(bundle).is_none() {
        return Vec::new();
    }
    let oracle = conflict_oracle(bundle, "norm_contradiction");
    let mut conflict = Conflict {
        conflict_id: oracle.conflict_id.clone(),
        conflict_type: "norm_contradiction".to_string(),
        severity: Severity::High,
        confidence: oracle.confidence,
        minimal_artifact_set: link::minimal_artifact_set(report, &NORM_CERT_KEYS),
        source_spans: oracle.source_spans.clone(),
        normalized_view: oracle.normalized_view.clone(),
        witness: link::witness_for_cert(report, "cert_z3_norm_conflict"),
        repair_candidates: repair::repair_candidates("norm_contradiction", oracle),
        solver_evidence: oracle.solver_evidence.clone(),
        argument_graph_id: Some(argument::build_argument_graph(bundle).argument_graph_id),
        human_review_question_ja: oracle.human_review_question_ja.clone(),
        human_review_question_en: oracle.human_review_question_en.clone(),
        classification: ConflictClassification::TrueConflict,
    };
    normalize_all(&mut conflict);
    vec![conflict]
}
