//! SPEC §7.3 route-quality raw-row metrics (M2, metrics-m2.1): pure
//! computation from a recorded run's per-route observations to sorted metric
//! rows, no I/O. Value rules (§7.3): every emitted value is an exact reduced
//! [`Rational`] (counts ride as `<n>/1`); a zero denominator emits
//! [`MetricValue::NotApplicable`]; a metric the observations cannot honestly
//! support is omitted from the rows, explained by one omission diagnostic on
//! [`RouteMetrics::diagnostics`]. Channels: fills project [`ModelFill`]
//! telemetry ([`FillObservation::from_fill`]); groups carry the solver
//! results plus the planned query-pair ids the §6 match criteria key off
//! ([`GroupObservation`]); the reference supplies the §8 conflict +
//! no-conflict expectations (no-conflict groups first-class, §9). run-m2.1
//! wires the observations from the route loop; the report units embed the
//! rows (raw rows precede any ranking, §9).

use std::collections::{BTreeMap, BTreeSet};

use ckc_core::{DiagnosticCode, DiagnosticRecord, Id, Outcome, Rational, ReferenceEntry};
use ckc_smt::{SolverVerdict, VerifierCategory, VerifierResult};

use crate::model_fill::ModelFill;
use crate::shell::static_id;

/// §7.3 `acceptance_rate`: fills whose target passed the route's §4
/// acceptance checks, over completed fills.
pub const ACCEPTANCE_RATE: &str = "acceptance_rate";
/// §7.3/§9 `conflict_verdict_accuracy`: reference groups whose verdicts match
/// their §8 expectation, over all reference groups (a group the route
/// produced no verdict for counts as a miss).
pub const CONFLICT_VERDICT_ACCURACY: &str = "conflict_verdict_accuracy";
/// §7.3 `recorded_call_count`: total recorded model invocations, as `<n>/1`.
pub const RECORDED_CALL_COUNT: &str = "recorded_call_count";
/// §7.3 `repair_count`: total repair re-prompts spent, as `<n>/1`.
pub const REPAIR_COUNT: &str = "repair_count";
/// §7.3 `schema_valid_rate`: schema-valid model outputs over recorded calls
/// (a grounding rejection parses, so it stays schema-valid).
pub const SCHEMA_VALID_RATE: &str = "schema_valid_rate";
/// §9 `target_syntactic_validity`: solver-parsed queries over
/// solver-executed queries (a query never executed is not counted).
pub const TARGET_SYNTACTIC_VALIDITY: &str = "target_syntactic_validity";

/// One completed model-fill invocation's §7.3 telemetry — the fill channel's
/// unit (single_ir: one per document; direct_smt: one per query role). A
/// cassette IO/contract failure (`Err(CassetteError)`) completes no fill and
/// yields no observation; the route surfaces that failure through its own
/// diagnostics instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillObservation {
    /// The fill ended with an accepted target (§4 acceptance passed).
    pub accepted: bool,
    /// Model invocations the fill recorded (attempt 0 plus repairs).
    pub recorded_calls: u64,
    /// Repair re-prompts spent.
    pub repairs: u64,
    /// `ai_schema_violation` diagnostics among the attempts.
    pub schema_violations: u64,
}

impl FillObservation {
    /// Project a [`ModelFill`]'s §7.3 telemetry.
    pub fn from_fill<T>(fill: &ModelFill<T>) -> Self {
        FillObservation {
            accepted: fill.target.is_some(),
            recorded_calls: fill.recorded_calls,
            repairs: fill.repairs,
            schema_violations: fill
                .diagnostics
                .iter()
                .filter(|d| d.code == DiagnosticCode::AiSchemaViolation)
                .count() as u64,
        }
    }
}

/// One group's verdict-channel observation: the solver results the route
/// landed plus the planned `(context_overlap_query_id,
/// deontic_consistency_query_id)` pairs the §6 match criteria key off
/// (single_ir: the compiled `solver_query_plan`; direct_smt: the minted
/// `<gid>.overlap`/`<gid>.deontic` ids). `results` holds solver-executed
/// queries only — a Q1-unsat closure skips its Q2, which therefore never
/// appears here.
#[derive(Debug, Clone)]
pub struct GroupObservation {
    pub group_id: Id,
    pub query_pairs: Vec<(Id, Id)>,
    pub results: Vec<VerifierResult>,
}

/// §7.3 metric value: an exact reduced fraction, or `not_applicable` when
/// the metric's denominator is zero on this run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricValue {
    Value(Rational),
    NotApplicable,
}

/// One §7.3 raw row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRow {
    pub metric: Id,
    pub value: MetricValue,
}

/// One route's §7.3 raw rows, sorted by metric id, plus one omission
/// diagnostic per metric this run's observations could not honestly support.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteMetrics {
    pub pipeline_id: Id,
    pub rows: Vec<MetricRow>,
    pub diagnostics: Vec<DiagnosticRecord>,
}

/// Exact reduced fraction from decimal parts; the caller guarantees a
/// positive denominator.
fn exact(num: u128, den: u128) -> Rational {
    Rational::from_parts(&num.to_string(), &den.to_string())
        .expect("decimal integer parts with a positive denominator")
}

/// §7.3 fraction rule: a zero denominator emits `not_applicable`.
fn ratio(num: u128, den: u128) -> MetricValue {
    if den == 0 {
        MetricValue::NotApplicable
    } else {
        MetricValue::Value(exact(num, den))
    }
}

/// §7.3 count rule: counts are always available, as `<n>/1`.
fn count(n: u128) -> MetricValue {
    MetricValue::Value(exact(n, 1))
}

fn row(metric: &str, value: MetricValue) -> MetricRow {
    MetricRow {
        metric: static_id(metric),
        value,
    }
}

/// §7.3 omission diagnostic: the named metric is absent from the rows
/// because its inputs are off their documented shape — `schema_invalid`
/// (the input, not the route, is at fault), `Outcome::Invalid`, reason in
/// the payload, no resolved refs (the codebase-wide schema-code convention).
fn omitted_metric_diagnostic(metric: &str, reason: String) -> DiagnosticRecord {
    DiagnosticRecord {
        code: DiagnosticCode::SchemaInvalid,
        outcome: Outcome::Invalid,
        payload: vec![
            (static_id("metric"), metric.to_owned()),
            (static_id("reason"), reason),
        ],
        region_ids: Vec::new(),
        artifact_hashes: Vec::new(),
    }
}

/// Does one group's observation match its §8 reference expectation? Mirrors
/// the `run_oracle` scoring shape as a predicate: a `semantic_contradiction`
/// group needs exactly one [`VerifierCategory::SemanticContradiction`]
/// result, riding a planned pair's deontic-consistency query — the §6
/// `deontic_direction_conflict` kind, so the reference must expect exactly
/// that kind — whose `unsat_core` set-equals the expected core; a
/// `semantic_no_conflict` group needs at least one result, zero
/// contradictions, every result [`VerifierCategory::SemanticNoConflict`]
/// — and, when the reference documents the no-conflict result, at least one
/// pair closed by an unsat overlap answer whose deontic query never ran.
/// One deliberate strengthening over the oracle (an assertion helper that
/// only ever sees real, non-empty runs): an empty result set never matches
/// — a scorer stays fail-closed. The caller rejects any other
/// `expected_outcome` before calling.
fn group_matches_reference(group: &GroupObservation, entry: &ReferenceEntry) -> bool {
    let results = &group.results;
    let contradictions: Vec<&VerifierResult> = results
        .iter()
        .filter(|r| r.category == VerifierCategory::SemanticContradiction)
        .collect();
    if entry.expected_outcome == static_id("semantic_contradiction") {
        let [hit] = contradictions.as_slice() else {
            return false;
        };
        let rides_deontic = group.query_pairs.iter().any(|(_, d)| *d == hit.query_id);
        let kind_expected =
            entry.expected_conflict_kind == Some(static_id("deontic_direction_conflict"));
        let core: Option<BTreeSet<&Id>> = hit.unsat_core.as_ref().map(|c| c.iter().collect());
        let expected: BTreeSet<&Id> = entry.expected_unsat_core.iter().collect();
        rides_deontic && kind_expected && core == Some(expected)
    } else {
        // semantic_no_conflict — the only other recognized outcome. An empty
        // result set is a route that produced no verdict, never a match.
        if results.is_empty() || !contradictions.is_empty() {
            return false;
        }
        if !results
            .iter()
            .all(|r| r.category == VerifierCategory::SemanticNoConflict)
        {
            return false;
        }
        if entry.expected_no_conflict_result {
            let closed: Vec<&(Id, Id)> = group
                .query_pairs
                .iter()
                .filter(|(overlap, _)| {
                    results
                        .iter()
                        .any(|r| r.query_id == *overlap && r.verdict == Some(SolverVerdict::Unsat))
                })
                .collect();
            !closed.is_empty()
                && closed
                    .iter()
                    .all(|(_, deontic)| results.iter().all(|r| r.query_id != *deontic))
        } else {
            true
        }
    }
}

/// §7.3/§9 conflict-verdict accuracy over the reference groups, or the
/// omission diagnostic when any reference entry expects an outcome outside
/// the §8 conflict/no-conflict vocabulary (fail-closed: a silently shrunken
/// or misread denominator would misreport the route).
fn conflict_verdict_accuracy(
    groups: &BTreeMap<&Id, &GroupObservation>,
    reference: &[ReferenceEntry],
) -> Result<MetricValue, DiagnosticRecord> {
    let recognized = [
        static_id("semantic_contradiction"),
        static_id("semantic_no_conflict"),
    ];
    let unrecognized: Vec<String> = reference
        .iter()
        .filter(|e| !recognized.contains(&e.expected_outcome))
        .map(|e| format!("{}={}", e.group_id, e.expected_outcome))
        .collect();
    if !unrecognized.is_empty() {
        return Err(omitted_metric_diagnostic(
            CONFLICT_VERDICT_ACCURACY,
            format!(
                "unrecognized expected_outcome(s): {}",
                unrecognized.join(", ")
            ),
        ));
    }
    let matched = reference
        .iter()
        .filter(|e| {
            groups
                .get(&e.group_id)
                .is_some_and(|g| group_matches_reference(g, e))
        })
        .count();
    Ok(ratio(matched as u128, reference.len() as u128))
}

/// Fold one route's recorded-run observations into its §7.3 raw rows. Rows
/// emit sorted by metric id; an uncomputable metric is omitted and explained
/// on `diagnostics`; a zero denominator emits `not_applicable`. Accuracy
/// scores every reference group — a reference group with no observation (or
/// an observation with no results) is a miss, an observed group absent from
/// the reference (a rejection-path group) is outside the denominator.
pub fn route_metrics(
    pipeline_id: &Id,
    fills: &[FillObservation],
    groups: &[GroupObservation],
    reference: &[ReferenceEntry],
) -> RouteMetrics {
    let calls: u128 = fills.iter().map(|f| u128::from(f.recorded_calls)).sum();
    let violations: u128 = fills.iter().map(|f| u128::from(f.schema_violations)).sum();
    assert!(
        violations <= calls,
        "fill observations record more schema violations than recorded calls"
    );
    let repairs: u128 = fills.iter().map(|f| u128::from(f.repairs)).sum();
    let accepted = fills.iter().filter(|f| f.accepted).count() as u128;

    let executed: u128 = groups.iter().map(|g| g.results.len() as u128).sum();
    let parsed = groups
        .iter()
        .flat_map(|g| &g.results)
        .filter(|r| r.category != VerifierCategory::TargetSyntaxFailure)
        .count() as u128;

    let mut by_group: BTreeMap<&Id, &GroupObservation> = BTreeMap::new();
    for group in groups {
        assert!(
            by_group.insert(&group.group_id, group).is_none(),
            "duplicate group observation {}",
            group.group_id
        );
    }

    let mut rows = vec![
        row(ACCEPTANCE_RATE, ratio(accepted, fills.len() as u128)),
        row(RECORDED_CALL_COUNT, count(calls)),
        row(REPAIR_COUNT, count(repairs)),
        row(SCHEMA_VALID_RATE, ratio(calls - violations, calls)),
        row(TARGET_SYNTACTIC_VALIDITY, ratio(parsed, executed)),
    ];
    let mut diagnostics = Vec::new();
    match conflict_verdict_accuracy(&by_group, reference) {
        Ok(value) => rows.push(row(CONFLICT_VERDICT_ACCURACY, value)),
        Err(diagnostic) => diagnostics.push(diagnostic),
    }
    rows.sort_by(|a, b| a.metric.cmp(&b.metric));
    RouteMetrics {
        pipeline_id: pipeline_id.clone(),
        rows,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use ckc_core::{RationalRepr, SolverIdentity};

    use super::*;

    fn obs(accepted: bool, calls: u64, repairs: u64, violations: u64) -> FillObservation {
        FillObservation {
            accepted,
            recorded_calls: calls,
            repairs,
            schema_violations: violations,
        }
    }

    fn vr(
        query: &str,
        category: VerifierCategory,
        verdict: Option<SolverVerdict>,
        core: Option<&[&str]>,
    ) -> VerifierResult {
        VerifierResult {
            query_id: static_id(query),
            category,
            verdict,
            model: None,
            unsat_core: core.map(|ids| ids.iter().map(|s| static_id(s)).collect()),
            solver_identity: SolverIdentity {
                solver_id: static_id("solver.fixture"),
                version: "1.0.0".to_owned(),
            },
            diagnostics: Vec::new(),
        }
    }

    fn group(gid: &str, results: Vec<VerifierResult>) -> GroupObservation {
        GroupObservation {
            group_id: static_id(gid),
            query_pairs: vec![(static_id("q.o"), static_id("q.d"))],
            results,
        }
    }

    fn conflict_entry(gid: &str, core: &[&str]) -> ReferenceEntry {
        ReferenceEntry {
            group_id: static_id(gid),
            expected_outcome: static_id("semantic_contradiction"),
            expected_conflict_kind: Some(static_id("deontic_direction_conflict")),
            expected_unsat_core: core.iter().map(|s| static_id(s)).collect(),
            expected_no_conflict_result: false,
        }
    }

    fn no_conflict_entry(gid: &str, documented: bool) -> ReferenceEntry {
        ReferenceEntry {
            group_id: static_id(gid),
            expected_outcome: static_id("semantic_no_conflict"),
            expected_conflict_kind: None,
            expected_unsat_core: BTreeSet::new(),
            expected_no_conflict_result: documented,
        }
    }

    fn frac(num: &str, den: &str) -> MetricValue {
        MetricValue::Value(Rational::from_parts(num, den).unwrap())
    }

    fn value_of(metrics: &RouteMetrics, metric: &str) -> MetricValue {
        metrics
            .rows
            .iter()
            .find(|r| r.metric == static_id(metric))
            .unwrap_or_else(|| panic!("row {metric} present"))
            .value
            .clone()
    }

    /// One-group accuracy probe: the match criteria under test, isolated.
    fn accuracy(results: Vec<VerifierResult>, entry: ReferenceEntry) -> MetricValue {
        let groups = vec![group(entry.group_id.as_str(), results)];
        let metrics = route_metrics(&static_id("pipe.test"), &[], &groups, &[entry]);
        value_of(&metrics, CONFLICT_VERDICT_ACCURACY)
    }

    fn contradiction_on(query: &str, core: &[&str]) -> VerifierResult {
        vr(
            query,
            VerifierCategory::SemanticContradiction,
            Some(SolverVerdict::Unsat),
            Some(core),
        )
    }

    fn no_conflict_on(query: &str, verdict: SolverVerdict) -> VerifierResult {
        vr(
            query,
            VerifierCategory::SemanticNoConflict,
            Some(verdict),
            None,
        )
    }

    #[test]
    fn fill_rows_are_exact_reduced_fractions() {
        let fills = vec![
            obs(true, 1, 0, 0),
            obs(true, 2, 1, 1),
            obs(false, 2, 1, 2),
            obs(true, 1, 0, 0),
        ];
        let metrics = route_metrics(&static_id("pipe.test"), &fills, &[], &[]);
        assert_eq!(value_of(&metrics, ACCEPTANCE_RATE), frac("3", "4"));
        assert_eq!(value_of(&metrics, RECORDED_CALL_COUNT), frac("6", "1"));
        assert_eq!(value_of(&metrics, REPAIR_COUNT), frac("2", "1"));
        // 3 valid / 6 calls reduces: the wire form is the §4.1 reduced pair.
        let MetricValue::Value(rate) = value_of(&metrics, SCHEMA_VALID_RATE) else {
            panic!("schema_valid_rate applicable");
        };
        let repr: RationalRepr = rate.into();
        assert_eq!((repr.num.as_str(), repr.den.as_str()), ("1", "2"));
        assert!(metrics.diagnostics.is_empty());
        // Rows emit sorted by metric id.
        let ids: Vec<&str> = metrics.rows.iter().map(|r| r.metric.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                ACCEPTANCE_RATE,
                CONFLICT_VERDICT_ACCURACY,
                RECORDED_CALL_COUNT,
                REPAIR_COUNT,
                SCHEMA_VALID_RATE,
                TARGET_SYNTACTIC_VALIDITY,
            ]
        );
    }

    #[test]
    fn zero_denominators_emit_not_applicable() {
        let metrics = route_metrics(&static_id("pipe.test"), &[], &[], &[]);
        assert_eq!(
            value_of(&metrics, ACCEPTANCE_RATE),
            MetricValue::NotApplicable
        );
        assert_eq!(
            value_of(&metrics, SCHEMA_VALID_RATE),
            MetricValue::NotApplicable
        );
        assert_eq!(
            value_of(&metrics, TARGET_SYNTACTIC_VALIDITY),
            MetricValue::NotApplicable
        );
        assert_eq!(
            value_of(&metrics, CONFLICT_VERDICT_ACCURACY),
            MetricValue::NotApplicable
        );
        // Counts stay available on an empty run.
        assert_eq!(value_of(&metrics, RECORDED_CALL_COUNT), frac("0", "1"));
        assert_eq!(value_of(&metrics, REPAIR_COUNT), frac("0", "1"));
        assert_eq!(metrics.rows.len(), 6);
        assert!(metrics.diagnostics.is_empty());
    }

    #[test]
    fn target_syntactic_validity_counts_solver_parses() {
        let groups = vec![
            group(
                "g.a",
                vec![
                    no_conflict_on("q.o", SolverVerdict::Sat),
                    vr("q.d", VerifierCategory::TargetSyntaxFailure, None, None),
                ],
            ),
            group("g.b", vec![no_conflict_on("q.o", SolverVerdict::Unsat)]),
        ];
        let metrics = route_metrics(&static_id("pipe.test"), &[], &groups, &[]);
        assert_eq!(
            value_of(&metrics, TARGET_SYNTACTIC_VALIDITY),
            frac("2", "3")
        );
    }

    #[test]
    fn conflict_match_criteria() {
        let entry = || conflict_entry("g.c", &["a.r1", "a.r2"]);
        // Exactly one contradiction, on the deontic query, core set-equal.
        let hit = vec![
            no_conflict_on("q.o", SolverVerdict::Sat),
            contradiction_on("q.d", &["a.r2", "a.r1"]),
        ];
        assert_eq!(accuracy(hit, entry()), frac("1", "1"));
        // Wrong core.
        let wrong_core = vec![
            no_conflict_on("q.o", SolverVerdict::Sat),
            contradiction_on("q.d", &["a.r1"]),
        ];
        assert_eq!(accuracy(wrong_core, entry()), frac("0", "1"));
        // The reference expects a different conflict kind — a deontic-Q2
        // contradiction is the §6 deontic_direction_conflict kind, no other.
        let wrong_kind = ReferenceEntry {
            expected_conflict_kind: Some(static_id("kind.other")),
            ..entry()
        };
        assert_eq!(
            accuracy(
                vec![
                    no_conflict_on("q.o", SolverVerdict::Sat),
                    contradiction_on("q.d", &["a.r2", "a.r1"]),
                ],
                wrong_kind
            ),
            frac("0", "1")
        );
        // The reference carries no expected kind at all.
        let kindless = ReferenceEntry {
            expected_conflict_kind: None,
            ..entry()
        };
        assert_eq!(
            accuracy(
                vec![
                    no_conflict_on("q.o", SolverVerdict::Sat),
                    contradiction_on("q.d", &["a.r2", "a.r1"]),
                ],
                kindless
            ),
            frac("0", "1")
        );
        // The contradiction rides the overlap query, not a deontic one.
        let rides_overlap = vec![contradiction_on("q.o", &["a.r1", "a.r2"])];
        assert_eq!(accuracy(rides_overlap, entry()), frac("0", "1"));
        // Two contradictions.
        let two = vec![
            contradiction_on("q.o", &["a.r1", "a.r2"]),
            contradiction_on("q.d", &["a.r1", "a.r2"]),
        ];
        assert_eq!(accuracy(two, entry()), frac("0", "1"));
        // No verdict at all.
        assert_eq!(accuracy(Vec::new(), entry()), frac("0", "1"));
        // No observation for the reference group at all.
        let metrics = route_metrics(&static_id("pipe.test"), &[], &[], &[entry()]);
        assert_eq!(
            value_of(&metrics, CONFLICT_VERDICT_ACCURACY),
            frac("0", "1")
        );
    }

    #[test]
    fn no_conflict_match_criteria() {
        // Undocumented: both queries ran and closed no-conflict.
        let open = vec![
            no_conflict_on("q.o", SolverVerdict::Sat),
            no_conflict_on("q.d", SolverVerdict::Sat),
        ];
        assert_eq!(
            accuracy(open, no_conflict_entry("g.n", false)),
            frac("1", "1")
        );
        // An empty result set is no verdict, never a vacuous match.
        assert_eq!(
            accuracy(Vec::new(), no_conflict_entry("g.n", false)),
            frac("0", "1")
        );
        // A contradiction breaks the group.
        let contradicted = vec![
            no_conflict_on("q.o", SolverVerdict::Sat),
            contradiction_on("q.d", &["a.r1"]),
        ];
        assert_eq!(
            accuracy(contradicted, no_conflict_entry("g.n", false)),
            frac("0", "1")
        );
        // A solver-rejected query breaks the group.
        let rejected = vec![vr("q.o", VerifierCategory::TargetSyntaxFailure, None, None)];
        assert_eq!(
            accuracy(rejected, no_conflict_entry("g.n", false)),
            frac("0", "1")
        );
        // Documented: the overlap query answered unsat and the pair's deontic
        // query never ran.
        let closed = vec![no_conflict_on("q.o", SolverVerdict::Unsat)];
        assert_eq!(
            accuracy(closed, no_conflict_entry("g.n", true)),
            frac("1", "1")
        );
        // Documented, but the deontic query ran anyway.
        let leaked = vec![
            no_conflict_on("q.o", SolverVerdict::Unsat),
            no_conflict_on("q.d", SolverVerdict::Sat),
        ];
        assert_eq!(
            accuracy(leaked, no_conflict_entry("g.n", true)),
            frac("0", "1")
        );
        // Documented, but no pair closed unsat.
        let unclosed = vec![
            no_conflict_on("q.o", SolverVerdict::Sat),
            no_conflict_on("q.d", SolverVerdict::Sat),
        ];
        assert_eq!(
            accuracy(unclosed, no_conflict_entry("g.n", true)),
            frac("0", "1")
        );
    }

    #[test]
    fn unscoreable_reference_omits_accuracy_with_diagnostic() {
        let mut alien = no_conflict_entry("g.x", false);
        alien.expected_outcome = static_id("semantic_gibberish");
        let reference = vec![conflict_entry("g.c", &["a.r1"]), alien];
        let metrics = route_metrics(
            &static_id("pipe.test"),
            &[obs(true, 1, 0, 0)],
            &[],
            &reference,
        );
        // The accuracy row is omitted; every other row still emits.
        assert!(
            metrics
                .rows
                .iter()
                .all(|r| r.metric != static_id(CONFLICT_VERDICT_ACCURACY))
        );
        assert_eq!(metrics.rows.len(), 5);
        assert_eq!(value_of(&metrics, ACCEPTANCE_RATE), frac("1", "1"));
        // One omission diagnostic, the schema-code shape: reason in the
        // payload, no resolved refs.
        let [diagnostic] = metrics.diagnostics.as_slice() else {
            panic!("one omission diagnostic");
        };
        assert_eq!(diagnostic.code, DiagnosticCode::SchemaInvalid);
        assert_eq!(diagnostic.outcome, Outcome::Invalid);
        assert!(diagnostic.region_ids.is_empty());
        assert!(diagnostic.artifact_hashes.is_empty());
        assert_eq!(diagnostic.payload.len(), 2);
        assert_eq!(
            diagnostic.payload[0],
            (static_id("metric"), CONFLICT_VERDICT_ACCURACY.to_owned())
        );
        assert_eq!(diagnostic.payload[1].0, static_id("reason"));
        assert_eq!(
            diagnostic.payload[1].1,
            "unrecognized expected_outcome(s): g.x=semantic_gibberish"
        );
    }

    #[test]
    fn from_fill_projects_model_fill_telemetry() {
        let violation = DiagnosticRecord {
            code: DiagnosticCode::AiSchemaViolation,
            outcome: Outcome::Invalid,
            payload: vec![(static_id("reason"), "malformed".to_owned())],
            region_ids: Vec::new(),
            artifact_hashes: Vec::new(),
        };
        let recovered = ModelFill {
            target: Some(()),
            diagnostics: vec![violation.clone()],
            recorded_calls: 2,
            repairs: 1,
        };
        assert_eq!(FillObservation::from_fill(&recovered), obs(true, 2, 1, 1));
        // A hallucination parses: no schema violation counted.
        let hallucinated = ModelFill::<()> {
            target: None,
            diagnostics: vec![DiagnosticRecord {
                code: DiagnosticCode::AiHallucinatedSource,
                outcome: Outcome::Invalid,
                payload: vec![(static_id("absent_source_ids"), "seg.x".to_owned())],
                region_ids: Vec::new(),
                artifact_hashes: Vec::new(),
            }],
            recorded_calls: 1,
            repairs: 0,
        };
        assert_eq!(
            FillObservation::from_fill(&hallucinated),
            obs(false, 1, 0, 0)
        );
        // Exhaustion: the terminal repair_limit_exceeded is no violation.
        let exhausted = ModelFill::<()> {
            target: None,
            diagnostics: vec![
                violation.clone(),
                violation,
                DiagnosticRecord {
                    code: DiagnosticCode::RepairLimitExceeded,
                    outcome: Outcome::Invalid,
                    payload: vec![(static_id("repair_limit"), "1".to_owned())],
                    region_ids: Vec::new(),
                    artifact_hashes: Vec::new(),
                },
            ],
            recorded_calls: 2,
            repairs: 1,
        };
        assert_eq!(FillObservation::from_fill(&exhausted), obs(false, 2, 1, 2));
    }

    #[test]
    #[should_panic(expected = "duplicate group observation")]
    fn duplicate_group_observation_is_a_route_bug() {
        let groups = vec![group("g.a", Vec::new()), group("g.a", Vec::new())];
        route_metrics(&static_id("pipe.test"), &[], &groups, &[]);
    }

    #[test]
    #[should_panic(expected = "more schema violations than recorded calls")]
    fn violations_beyond_calls_are_a_fill_bug() {
        route_metrics(&static_id("pipe.test"), &[obs(false, 1, 0, 2)], &[], &[]);
    }
}
