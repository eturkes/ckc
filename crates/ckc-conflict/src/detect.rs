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

use std::collections::BTreeMap;

use ckc_core::artifact::{DecisionRow, DecisionTable, EventNarrative};
use ckc_core::clinical::Rule;
use ckc_core::enums::{
    ConflictClassification, HitPolicy, RecommendationDirection, RuleKind, Severity,
};
use ckc_core::nf::normalize_all;
use serde_json::Value;

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

/// The decision-table certificate key (SPEC 15.1 #14): the z3 check over the
/// `dt_vitals_triage` overlap/gap target (`logic/smt/decision_table.smt2`) — the
/// deterministic `cert_z3_decision_table` id the 0.9 builder emits.
/// [`link::witness_for_cert`] and [`link::minimal_artifact_set`] resolve it to the
/// real witness and certificate hash that replace the toy `conflicts.json`
/// placeholders.
const DT_CERT_KEY: &str = "cert_z3_decision_table";

/// Probe step for breakpoint sampling. Each numeric threshold `t` in a field's
/// conditions contributes probe values `t - PROBE_STEP`, `t`, and `t + PROBE_STEP`,
/// sampling just below, at, and just above the breakpoint. `0.5` is exact in f64,
/// so probe arithmetic over the integer/half-integer toy thresholds stays exact.
const PROBE_STEP: f64 = 0.5;

/// Whether one decision-row condition (`{field, op, value}`) holds at `point`.
/// A `*` (wildcard) op always holds; the comparison ops test the point's value for
/// the condition's field against the threshold. A condition over a field absent
/// from `point`, missing its threshold, or carrying an unrecognized op does not
/// hold.
fn condition_holds(cond: &Value, point: &BTreeMap<String, f64>) -> bool {
    let op = cond.get("op").and_then(Value::as_str).unwrap_or_default();
    if op == "*" {
        return true;
    }
    let Some(field) = cond.get("field").and_then(Value::as_str) else {
        return false;
    };
    let Some(&x) = point.get(field) else {
        return false;
    };
    let Some(threshold) = cond.get("value").and_then(Value::as_f64) else {
        return false;
    };
    match op {
        ">=" => x >= threshold,
        ">" => x > threshold,
        "<=" => x <= threshold,
        "<" => x < threshold,
        "==" => x == threshold,
        _ => false,
    }
}

/// Whether every condition of `row` holds at `point` — i.e. the row fires there.
fn row_fires(row: &DecisionRow, point: &BTreeMap<String, f64>) -> bool {
    row.conditions.iter().all(|c| condition_holds(c, point))
}

/// All `(field → value)` probe points for `table`: the cartesian product of each
/// input field's breakpoint probes. Fields (BTreeMap key order) and per-field probe
/// values (sorted, deduped) are deterministic, so the point sequence is stable
/// across runs. Each numeric threshold contributes `t ± PROBE_STEP` and `t`,
/// sampling every region the row conditions induce: a point firing exactly one row
/// is well-covered, an overlap point fires two or more, a gap point fires none.
fn probe_points(table: &DecisionTable) -> Vec<BTreeMap<String, f64>> {
    // Threshold-derived probe values per referenced field.
    let mut by_field: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for row in &table.rows {
        for cond in &row.conditions {
            let Some(field) = cond.get("field").and_then(Value::as_str) else {
                continue;
            };
            let probes = by_field.entry(field.to_string()).or_default();
            if let Some(t) = cond.get("value").and_then(Value::as_f64) {
                probes.extend([t - PROBE_STEP, t, t + PROBE_STEP]);
            }
        }
    }
    for probes in by_field.values_mut() {
        // A field appearing only behind wildcards carries no threshold; one neutral
        // probe keeps it from collapsing the cartesian product to nothing.
        if probes.is_empty() {
            probes.push(0.0);
        }
        probes.sort_by(f64::total_cmp);
        probes.dedup();
    }
    let mut points = vec![BTreeMap::new()];
    for (field, probes) in &by_field {
        let mut next = Vec::with_capacity(points.len() * probes.len());
        for base in &points {
            for &v in probes {
                let mut p = base.clone();
                p.insert(field.clone(), v);
                next.push(p);
            }
        }
        points = next;
    }
    points
}

/// The overlapping row pair of a `Unique`-policy decision table (SPEC 15.1 #14):
/// two rows that fire together at some probe point yet carry different outputs, so
/// the unique hit policy — at most one matching row — is violated. Returns
/// `(first_row, second_row)` in table (document) order, or `None` when no `Unique`
/// table has such a pair. Over the toy bundle this is
/// `(row_temp_high, row_temp_very_high)` of `dt_vitals_triage`, which both fire at
/// `temperature ≥ 38.5` under differing outputs (`administer_antipyretic` vs
/// `initiate_cooling`). Exposed so the scan is checkable independently of the
/// curated oracle (the task-0.10.6 detection-agreement gate).
pub fn decision_table_overlap(bundle: &CompileBundle) -> Option<(&DecisionRow, &DecisionRow)> {
    for table in &bundle.decision_tables {
        if table.hit_policy != HitPolicy::Unique {
            continue;
        }
        let points = probe_points(table);
        for (i, a) in table.rows.iter().enumerate() {
            for b in &table.rows[i + 1..] {
                if a.outputs != b.outputs
                    && points.iter().any(|p| row_fires(a, p) && row_fires(b, p))
                {
                    return Some((a, b));
                }
            }
        }
    }
    None
}

/// A gap point of a `Unique`-policy decision table (SPEC 15.1 #14): an input
/// assignment firing no row, so the table prescribes nothing there. Returns the
/// first such probe point (deterministic probe order), or `None` when every probe
/// point fires at least one row. Over the toy bundle a gap exists in the region
/// `temperature < 38.0 ∧ heart_rate ≤ 90 ∧ systolic_bp ≥ 90` (the curated witness
/// `(37.5, 85, 95)` lies inside it). Exposed alongside [`decision_table_overlap`]
/// for the detection-agreement gate.
pub fn decision_table_gap(bundle: &CompileBundle) -> Option<BTreeMap<String, f64>> {
    for table in &bundle.decision_tables {
        if table.hit_policy != HitPolicy::Unique {
            continue;
        }
        if let Some(point) = probe_points(table)
            .into_iter()
            .find(|p| table.rows.iter().all(|row| !row_fires(row, p)))
        {
            return Some(point);
        }
    }
    None
}

/// Detect the decision-table defect (SPEC 15.1 #14): a `Unique`-policy table whose
/// rows overlap (a pair fires together under differing outputs) and leave a gap (a
/// point firing no row).
///
/// Runs [`decision_table_overlap`] over `bundle.decision_tables`; with no
/// overlapping unique-policy pair the scan reports nothing. On a hit it assembles
/// one [`Conflict`] with the computed verdict — `conflict_type =
/// "decision_table_overlap"`, `classification = TrueConflict`, `severity = Medium`
/// — and the real evidence linkage from `report`: the witness behind
/// [`DT_CERT_KEY`] ([`link::witness_for_cert`]) and that certificate's hash
/// ([`link::minimal_artifact_set`]), replacing the toy `conflicts.json`
/// placeholders. The curated `source_spans`, `confidence`, `normalized_view`
/// (carrying the overlap and gap witnesses), `solver_evidence`, and JA/EN review
/// questions come from the matching oracle; the source-revision
/// [`repair::repair_candidates`] back the conflict. A decision-table defect carries
/// no argument graph (`argument_graph_id = None`). [`normalize_all`] then assigns
/// the content-derived `nf-…` `conflict_id`.
pub fn detect_decision_table_defects(
    bundle: &CompileBundle,
    report: &VerificationReport,
) -> Vec<Conflict> {
    // Detection drives emission: report only when the scan finds an overlapping
    // unique-policy row pair, not merely because the oracle carries the class.
    if decision_table_overlap(bundle).is_none() {
        return Vec::new();
    }
    let oracle = conflict_oracle(bundle, "decision_table_overlap");
    let mut conflict = Conflict {
        conflict_id: oracle.conflict_id.clone(),
        conflict_type: "decision_table_overlap".to_string(),
        severity: Severity::Medium,
        confidence: oracle.confidence,
        minimal_artifact_set: link::minimal_artifact_set(report, &[DT_CERT_KEY]),
        source_spans: oracle.source_spans.clone(),
        normalized_view: oracle.normalized_view.clone(),
        witness: link::witness_for_cert(report, DT_CERT_KEY),
        repair_candidates: repair::repair_candidates("decision_table_overlap", oracle),
        solver_evidence: oracle.solver_evidence.clone(),
        argument_graph_id: None,
        human_review_question_ja: oracle.human_review_question_ja.clone(),
        human_review_question_en: oracle.human_review_question_en.clone(),
        classification: ConflictClassification::TrueConflict,
    };
    normalize_all(&mut conflict);
    vec![conflict]
}

/// The Event-Calculus certificate key (SPEC 15.1 #6/#7): the clingo check over the
/// `logic/asp/event_calculus.lp` narrative target — the deterministic
/// `cert_clingo_event_calculus` id the 0.9 builder emits. [`link::witness_for_cert`]
/// and [`link::minimal_artifact_set`] resolve it to the real EC witness and
/// certificate hash that replace the toy `conflicts.json` placeholders.
const EC_CERT_KEY: &str = "cert_clingo_event_calculus";

/// The violated-constraint name the recorded EC witness carries when the
/// contraindication fluent persists through drug administration — the
/// `violation(<conflict>)` atom argument the 0.9 clingo EC outcome surfaces. The
/// detector cross-checks it against the witness so detection rests on the recorded
/// solver verdict, not merely the oracle carrying the class.
const EC_VIOLATION: &str = "conflict_ec_allergy_persistence";

/// The happens-time of the `administer_drug` event in `narrative` — the moment the
/// drug is given. `None` when the narrative schedules no such event.
fn administer_time(narrative: &EventNarrative) -> Option<f64> {
    narrative
        .happens
        .iter()
        .find(|h| h.get("event").and_then(Value::as_str) == Some("administer_drug"))
        .and_then(|h| h.get("time").and_then(Value::as_f64))
}

/// Whether `fluent` genuinely holds at time `t` under Event-Calculus persistence: an
/// event initiates it at some time at or before `t`, and no event terminates it at a
/// time before `t`. Once initiated and never clipped, the fluent persists to `t`.
fn persists_until(narrative: &EventNarrative, fluent: &str, t: f64) -> bool {
    let initiated = narrative.initiates.iter().any(|i| {
        i.get("fluent").and_then(Value::as_str) == Some(fluent)
            && i.get("time")
                .and_then(Value::as_f64)
                .is_some_and(|ti| ti <= t)
    });
    let cleared = narrative.terminates.iter().any(|term| {
        term.get("fluent").and_then(Value::as_str) == Some(fluent)
            && term
                .get("time")
                .and_then(Value::as_f64)
                .is_some_and(|tt| tt < t)
    });
    initiated && !cleared
}

/// Whether `narrative` exhibits the allergy-persistence temporal violation
/// (SPEC 15.1 #6/#7): a `holds_query` asserts a fluent still holds (`expected = true`)
/// at the `administer_drug` time, and that fluent — initiated earlier and terminated
/// by no event before then ([`persists_until`]) — genuinely persists there, so the
/// contraindication is active when the drug is given. Over the toy bundle the single
/// narrative satisfies this for `allergy_known` at t = 10. Exposed so the bundle-side
/// scan is checkable independently of the curated oracle (part of the task-0.10.7
/// detection-agreement gate).
pub fn allergy_persists_at_administration(narrative: &EventNarrative) -> bool {
    let Some(t_admin) = administer_time(narrative) else {
        return false;
    };
    narrative.holds_queries.iter().any(|q| {
        q.get("expected").and_then(Value::as_bool) == Some(true)
            && q.get("time").and_then(Value::as_f64) == Some(t_admin)
            && q.get("fluent")
                .and_then(Value::as_str)
                .is_some_and(|f| persists_until(narrative, f, t_admin))
    })
}

/// The violated-constraint names the recorded Event-Calculus witness carries — the
/// `violation(<conflict>)` atom arguments the clingo EC run surfaced, behind
/// [`EC_CERT_KEY`]. Over the toy bundle this is `["conflict_ec_allergy_persistence"]`;
/// empty when the report carries no EC witness. Exposed so the report-side cross-check
/// is checkable independently of the curated oracle (part of the task-0.10.7
/// detection-agreement gate).
pub fn ec_violated_constraints(report: &VerificationReport) -> Vec<String> {
    report
        .witnesses
        .iter()
        .find(|w| w.certificate_ids.iter().any(|c| c.as_str() == EC_CERT_KEY))
        .map(|w| w.violated_constraints.clone())
        .unwrap_or_default()
}

/// Detect the Event-Calculus temporal violation (SPEC 15.1 #6/#7): the persistence
/// defect where the contraindication fluent `allergy_known`, initiated at the
/// allergy-detection event and never terminated, still holds when `administer_drug`
/// fires — so the drug is given under an active contraindication.
///
/// Detection is twofold and both halves must agree: the bundle's single narrative
/// must exhibit the persistence ([`allergy_persists_at_administration`]) and the
/// recorded EC witness must name [`EC_VIOLATION`] among its [`ec_violated_constraints`].
/// With either absent the scan reports nothing. On a hit it assembles one [`Conflict`]
/// with the computed verdict — `conflict_type = "temporal_violation"`,
/// `classification = TrueConflict`, `severity = High` — and the real evidence linkage
/// from `report`: the EC witness behind [`EC_CERT_KEY`] ([`link::witness_for_cert`])
/// and that certificate's hash ([`link::minimal_artifact_set`]), replacing the toy
/// `conflicts.json` placeholders. The curated `source_spans`, `confidence`,
/// `normalized_view`, `solver_evidence`, and JA/EN review questions come from the
/// matching oracle — whose `source_spans` cover the narrative's own
/// `source_span_ids` (asserted) — and the source-revision [`repair::repair_candidates`]
/// and normalized Dung [`argument::build_argument_graph`] back the conflict.
/// [`normalize_all`] then assigns the content-derived `nf-…` `conflict_id`.
pub fn detect_temporal_violation(
    bundle: &CompileBundle,
    report: &VerificationReport,
) -> Vec<Conflict> {
    // Detection drives emission: the narrative must show the persistence AND the
    // recorded EC witness must name the violation, not merely the oracle carrying it.
    let Some(narrative) = bundle.event_narratives.first() else {
        return Vec::new();
    };
    if !allergy_persists_at_administration(narrative)
        || !ec_violated_constraints(report)
            .iter()
            .any(|c| c == EC_VIOLATION)
    {
        return Vec::new();
    }
    let oracle = conflict_oracle(bundle, "temporal_violation");

    // The curated oracle's source spans must cover the narrative's own spans, so the
    // conflict's grounding subsumes the narrative it was detected from. A divergent
    // fixture surfaces here as a build-time bug, mirroring `conflict_oracle`.
    for span in &narrative.source_span_ids {
        assert!(
            oracle
                .source_spans
                .iter()
                .any(|s| s.as_str() == span.as_str()),
            "temporal-violation oracle source_spans must cover narrative span {}",
            span.as_str()
        );
    }

    let mut conflict = Conflict {
        conflict_id: oracle.conflict_id.clone(),
        conflict_type: "temporal_violation".to_string(),
        severity: Severity::High,
        confidence: oracle.confidence,
        minimal_artifact_set: link::minimal_artifact_set(report, &[EC_CERT_KEY]),
        source_spans: oracle.source_spans.clone(),
        normalized_view: oracle.normalized_view.clone(),
        witness: link::witness_for_cert(report, EC_CERT_KEY),
        repair_candidates: repair::repair_candidates("temporal_violation", oracle),
        solver_evidence: oracle.solver_evidence.clone(),
        argument_graph_id: Some(argument::build_argument_graph(bundle).argument_graph_id),
        human_review_question_ja: oracle.human_review_question_ja.clone(),
        human_review_question_en: oracle.human_review_question_en.clone(),
        classification: ConflictClassification::TrueConflict,
    };
    normalize_all(&mut conflict);
    vec![conflict]
}
