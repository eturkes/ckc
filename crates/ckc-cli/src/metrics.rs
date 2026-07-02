//! SPEC §7.3 route-quality raw-row metrics (M2, metrics-m2.1/.2): pure
//! computation from a recorded run's per-route observations to sorted metric
//! rows, no I/O. Value rules (§7.3): every emitted value is an exact reduced
//! [`Rational`] (counts ride as `<n>/1`); a zero denominator emits
//! [`MetricValue::NotApplicable`]; a metric the observations cannot honestly
//! support is omitted from the rows, explained by one omission diagnostic on
//! [`RouteMetrics::diagnostics`]. Channels: fills project [`ModelFill`]
//! telemetry ([`FillObservation::from_fill`]); groups carry the solver
//! results plus the planned query-pair ids the §6 match criteria key off
//! ([`GroupObservation`]); samples carry the k-sample battery's per-draw
//! group observations (§9 k-sample verdict stability); the reference
//! supplies the §8 conflict + no-conflict expectations (no-conflict groups
//! first-class, §9). [`experiment_metrics`] assembles the routes' raw rows
//! plus one per-metric (route − baseline) delta table per non-baseline
//! route under the experiment's designated baseline;
//! [`ExperimentMetrics::emission_order`] carries the §9
//! raw-rows-before-ranking rendering contract. run-m2.1 wires the
//! observations from the route loop; the report units embed the rows.

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
/// §7.3 `k_sample_convergence` (§9 k-sample verdict stability): mean
/// per-group pairwise verdict agreement across the k recorded samples —
/// agreeing unordered sample pairs over `groups × C(k, 2)`. One draw
/// (k < 2) cannot witness stability and an empty group universe has
/// nothing to agree on: both are zero denominators (`not_applicable`).
pub const K_SAMPLE_CONVERGENCE: &str = "k_sample_convergence";
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

/// One non-baseline route's §7.3 baseline-delta table: per-metric
/// (route − baseline) rows over identical inputs, sorted by metric id,
/// reusing [`MetricRow`]. A delta value is an exact signed [`Rational`];
/// a metric `not_applicable` or omitted on either side lands
/// `not_applicable` (a delta cannot honestly subtract what a route did
/// not measure).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDelta {
    pub pipeline_id: Id,
    pub rows: Vec<MetricRow>,
}

/// One experiment's assembled §7.3 metrics: every route's raw rows plus a
/// (route − baseline) delta table per non-baseline route, the baseline
/// designated by the experiment binding (run-m2.1 passes
/// `ExperimentEntry::baseline()`). The baseline's self-delta is
/// identically zero, so it gets no table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentMetrics {
    pub baseline_pipeline_id: Id,
    /// Raw rows per route, in the caller's (experiment pipeline-set) order.
    pub routes: Vec<RouteMetrics>,
    /// Delta tables for the non-baseline routes, in `routes` order.
    pub deltas: Vec<RouteDelta>,
}

/// One §9-ordered rendering section ([`ExperimentMetrics::emission_order`]).
#[derive(Debug, Clone, Copy)]
pub enum MetricsSection<'a> {
    /// A route's raw rows.
    RawRows(&'a RouteMetrics),
    /// A non-baseline route's (route − baseline) delta table.
    DeltaTable(&'a RouteDelta),
}

impl ExperimentMetrics {
    /// §9 raw-rows-before-ranking: every route's raw rows strictly precede
    /// every delta table. Renderers (the report units) walk this order,
    /// never the fields ad hoc.
    pub fn emission_order(&self) -> Vec<MetricsSection<'_>> {
        self.routes
            .iter()
            .map(MetricsSection::RawRows)
            .chain(self.deltas.iter().map(MetricsSection::DeltaTable))
            .collect()
    }
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

/// One sample's verdict content for a group, projected to what two draws
/// must reproduce to agree (§7.3 convergence = normalized hash agreement,
/// here over the verdict payload): the planned pairs plus each
/// solver-executed result's (query id, §6 category, raw verdict, unsat
/// core), §4.3 array order preserved. Outside the projection: solver
/// identity (run environment, not route output), diagnostics (telemetry),
/// and a sat witness's `model` bytes (a different witness for the same
/// verdict still agrees). Ids are single-line identifier_ascii, so the
/// newline/space framing cannot collide.
fn verdict_fingerprint(group: &GroupObservation) -> String {
    let mut lines = Vec::new();
    for (overlap, deontic) in &group.query_pairs {
        lines.push(format!("pair {overlap} {deontic}"));
    }
    for result in &group.results {
        let verdict = result.verdict.map_or("-", SolverVerdict::as_str);
        let core = result.unsat_core.as_ref().map_or_else(
            || "-".to_owned(),
            |ids| ids.iter().map(Id::as_str).collect::<Vec<_>>().join(","),
        );
        lines.push(format!(
            "result {} {} {verdict} {core}",
            result.query_id,
            result.category.as_str()
        ));
    }
    lines.join("\n")
}

/// §7.3/§9 k-sample convergence over the sample battery: for each group in
/// the union universe, the fraction of unordered sample pairs whose
/// [`verdict_fingerprint`]s agree — a draw without the group carries a
/// distinct no-verdict value, so consistent absence agrees and
/// presence-versus-absence disagrees; the route value is the mean over
/// groups.
fn k_sample_convergence(samples: &[Vec<GroupObservation>]) -> MetricValue {
    let mut universe: BTreeSet<&Id> = BTreeSet::new();
    let mut draws: Vec<BTreeMap<&Id, String>> = Vec::new();
    for sample in samples {
        let mut draw = BTreeMap::new();
        for group in sample {
            assert!(
                draw.insert(&group.group_id, verdict_fingerprint(group))
                    .is_none(),
                "duplicate group observation {} in a sample",
                group.group_id
            );
            universe.insert(&group.group_id);
        }
        draws.push(draw);
    }
    let k = draws.len() as u128;
    let sample_pairs = k * k.saturating_sub(1) / 2;
    let mut agreeing: u128 = 0;
    for gid in &universe {
        for (i, left) in draws.iter().enumerate() {
            for right in &draws[i + 1..] {
                if left.get(gid) == right.get(gid) {
                    agreeing += 1;
                }
            }
        }
    }
    ratio(agreeing, universe.len() as u128 * sample_pairs)
}

/// Fold one route's recorded-run observations into its §7.3 raw rows. Rows
/// emit sorted by metric id; an uncomputable metric is omitted and explained
/// on `diagnostics`; a zero denominator emits `not_applicable`. Accuracy
/// scores every reference group — a reference group with no observation (or
/// an observation with no results) is a miss, an observed group absent from
/// the reference (a rejection-path group) is outside the denominator.
/// `samples` is the k-sample battery's per-draw verdict channel (§9
/// per-sample seeds; whether the base draw is among them is the run's
/// k-sample config) — empty when the run drew no battery.
pub fn route_metrics(
    pipeline_id: &Id,
    fills: &[FillObservation],
    groups: &[GroupObservation],
    samples: &[Vec<GroupObservation>],
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
        row(K_SAMPLE_CONVERGENCE, k_sample_convergence(samples)),
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

/// Per-metric (route − baseline) rows over the union of both row sets,
/// sorted by metric id ([`RouteDelta`]'s value rules).
fn metric_deltas(route: &RouteMetrics, baseline: &RouteMetrics) -> Vec<MetricRow> {
    let metrics: BTreeSet<&Id> = route
        .rows
        .iter()
        .chain(&baseline.rows)
        .map(|r| &r.metric)
        .collect();
    metrics
        .into_iter()
        .map(|metric| {
            let value_in = |side: &RouteMetrics| {
                side.rows
                    .iter()
                    .find(|r| r.metric == *metric)
                    .map(|r| r.value.clone())
            };
            let value = match (value_in(route), value_in(baseline)) {
                (Some(MetricValue::Value(r)), Some(MetricValue::Value(b))) => {
                    MetricValue::Value(r.sub(&b))
                }
                _ => MetricValue::NotApplicable,
            };
            MetricRow {
                metric: metric.clone(),
                value,
            }
        })
        .collect()
}

/// Assemble one experiment's §7.3 metrics from its routes' raw rows.
/// `baseline_pipeline_id` is the experiment's designated baseline
/// (`exp.m2_multihop`: the direct_smt pipeline); it must name exactly one
/// route and the routes must be duplicate-free — a miss is run-loop
/// wiring, not observation, at fault (fail-closed panic, the
/// duplicate-observation convention).
pub fn experiment_metrics(
    routes: Vec<RouteMetrics>,
    baseline_pipeline_id: &Id,
) -> ExperimentMetrics {
    let mut seen: BTreeSet<&Id> = BTreeSet::new();
    for route in &routes {
        assert!(
            seen.insert(&route.pipeline_id),
            "duplicate route metrics {}",
            route.pipeline_id
        );
    }
    let baseline = routes
        .iter()
        .find(|r| r.pipeline_id == *baseline_pipeline_id)
        .unwrap_or_else(|| panic!("baseline pipeline {baseline_pipeline_id} has no route metrics"));
    let deltas: Vec<RouteDelta> = routes
        .iter()
        .filter(|r| r.pipeline_id != *baseline_pipeline_id)
        .map(|route| RouteDelta {
            pipeline_id: route.pipeline_id.clone(),
            rows: metric_deltas(route, baseline),
        })
        .collect();
    ExperimentMetrics {
        baseline_pipeline_id: baseline_pipeline_id.clone(),
        routes,
        deltas,
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
        let metrics = route_metrics(&static_id("pipe.test"), &[], &groups, &[], &[entry]);
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
        let metrics = route_metrics(&static_id("pipe.test"), &fills, &[], &[], &[]);
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
                K_SAMPLE_CONVERGENCE,
                RECORDED_CALL_COUNT,
                REPAIR_COUNT,
                SCHEMA_VALID_RATE,
                TARGET_SYNTACTIC_VALIDITY,
            ]
        );
    }

    #[test]
    fn zero_denominators_emit_not_applicable() {
        let metrics = route_metrics(&static_id("pipe.test"), &[], &[], &[], &[]);
        assert_eq!(
            value_of(&metrics, ACCEPTANCE_RATE),
            MetricValue::NotApplicable
        );
        assert_eq!(
            value_of(&metrics, K_SAMPLE_CONVERGENCE),
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
        assert_eq!(metrics.rows.len(), 7);
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
        let metrics = route_metrics(&static_id("pipe.test"), &[], &groups, &[], &[]);
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
        let metrics = route_metrics(&static_id("pipe.test"), &[], &[], &[], &[entry()]);
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
        assert_eq!(metrics.rows.len(), 6);
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

    /// Two-draw convergence probe: the fingerprint boundary under test,
    /// isolated (one group, so the row is that group's lone pair fraction).
    fn two_draw_convergence(a: GroupObservation, b: GroupObservation) -> MetricValue {
        let samples = vec![vec![a], vec![b]];
        let metrics = route_metrics(&static_id("pipe.test"), &[], &[], &samples, &[]);
        value_of(&metrics, K_SAMPLE_CONVERGENCE)
    }

    #[test]
    fn k_sample_convergence_measures_pairwise_verdict_agreement() {
        // Three draws: g.a identical in all three (3 agreeing pairs), g.b
        // flips in the third (1 agreeing pair) → (3 + 1) / (2 × 3) = 2/3.
        let stable = || {
            group(
                "g.a",
                vec![
                    no_conflict_on("q.o", SolverVerdict::Sat),
                    contradiction_on("q.d", &["a.r1", "a.r2"]),
                ],
            )
        };
        let agreeing = || group("g.b", vec![no_conflict_on("q.o", SolverVerdict::Sat)]);
        let flipped = group("g.b", vec![contradiction_on("q.d", &["a.r1"])]);
        let samples = vec![
            vec![stable(), agreeing()],
            vec![stable(), agreeing()],
            vec![stable(), flipped],
        ];
        let metrics = route_metrics(&static_id("pipe.test"), &[], &[], &samples, &[]);
        assert_eq!(value_of(&metrics, K_SAMPLE_CONVERGENCE), frac("2", "3"));
    }

    #[test]
    fn k_sample_fingerprint_reads_verdict_content_only() {
        let base = || group("g.s", vec![no_conflict_on("q.o", SolverVerdict::Sat)]);

        // Environment and telemetry sit outside the projection: a draw
        // differing only in solver identity, diagnostics, and witness
        // bytes still agrees.
        let mut noisy = base();
        noisy.results[0].solver_identity.version = "2.0.0".to_owned();
        noisy.results[0].model = Some("x1".to_owned());
        noisy.results[0].diagnostics.push(DiagnosticRecord {
            code: DiagnosticCode::AiSchemaViolation,
            outcome: Outcome::Invalid,
            payload: vec![(static_id("reason"), "noise".to_owned())],
            region_ids: Vec::new(),
            artifact_hashes: Vec::new(),
        });
        assert_eq!(two_draw_convergence(base(), noisy), frac("1", "1"));

        // Verdict content is load-bearing: raw verdict, category, unsat
        // core, result query id, planned pairs, §4.3 result order.
        let verdict_flip = group("g.s", vec![no_conflict_on("q.o", SolverVerdict::Unsat)]);
        assert_eq!(two_draw_convergence(base(), verdict_flip), frac("0", "1"));
        let category_flip = group("g.s", vec![contradiction_on("q.o", &["a.r1"])]);
        assert_eq!(two_draw_convergence(base(), category_flip), frac("0", "1"));
        let with_core = |core: &[&str]| group("g.s", vec![contradiction_on("q.d", core)]);
        assert_eq!(
            two_draw_convergence(with_core(&["a.r1"]), with_core(&["a.r2"])),
            frac("0", "1")
        );
        let query_flip = group("g.s", vec![no_conflict_on("q.x", SolverVerdict::Sat)]);
        assert_eq!(two_draw_convergence(base(), query_flip), frac("0", "1"));
        let mut pair_flip = base();
        pair_flip.query_pairs = vec![(static_id("q.o"), static_id("q.z"))];
        assert_eq!(two_draw_convergence(base(), pair_flip), frac("0", "1"));
        let two = |first: &str, second: &str| {
            group(
                "g.s",
                vec![
                    no_conflict_on(first, SolverVerdict::Sat),
                    no_conflict_on(second, SolverVerdict::Sat),
                ],
            )
        };
        assert_eq!(
            two_draw_convergence(two("q.o", "q.d"), two("q.d", "q.o")),
            frac("0", "1")
        );
    }

    #[test]
    fn k_sample_absence_is_a_distinct_fingerprint() {
        // g.x observed only in the first of three draws: two
        // presence-versus-absence pairs disagree, the both-absent pair
        // agrees (consistent absence is stable); g.y agrees everywhere
        // → (1 + 3) / (2 × 3) = 2/3.
        let stable = || group("g.y", vec![no_conflict_on("q.o", SolverVerdict::Sat)]);
        let extra = group("g.x", vec![no_conflict_on("q.o", SolverVerdict::Sat)]);
        let samples = vec![vec![extra, stable()], vec![stable()], vec![stable()]];
        let metrics = route_metrics(&static_id("pipe.test"), &[], &[], &samples, &[]);
        assert_eq!(value_of(&metrics, K_SAMPLE_CONVERGENCE), frac("2", "3"));
    }

    #[test]
    fn k_sample_zero_denominators_emit_not_applicable() {
        // One draw cannot witness stability, however much it observed.
        let one_draw = vec![vec![group(
            "g.a",
            vec![no_conflict_on("q.o", SolverVerdict::Sat)],
        )]];
        let metrics = route_metrics(&static_id("pipe.test"), &[], &[], &one_draw, &[]);
        assert_eq!(
            value_of(&metrics, K_SAMPLE_CONVERGENCE),
            MetricValue::NotApplicable
        );
        // Two draws over an empty group universe have nothing to agree on.
        let empty_draws: Vec<Vec<GroupObservation>> = vec![Vec::new(), Vec::new()];
        let metrics = route_metrics(&static_id("pipe.test"), &[], &[], &empty_draws, &[]);
        assert_eq!(
            value_of(&metrics, K_SAMPLE_CONVERGENCE),
            MetricValue::NotApplicable
        );
    }

    #[test]
    #[should_panic(expected = "in a sample")]
    fn duplicate_group_in_a_sample_is_a_route_bug() {
        let samples = vec![vec![group("g.a", Vec::new()), group("g.a", Vec::new())]];
        route_metrics(&static_id("pipe.test"), &[], &[], &samples, &[]);
    }

    /// Two-route fixture over identical inputs (shared reference): the
    /// route side accepts less, misses the conflict verdict, and carries a
    /// two-draw agreeing battery; the baseline accepts more, matches the
    /// reference, and drew no battery.
    fn delta_fixture() -> (RouteMetrics, RouteMetrics) {
        let reference = vec![conflict_entry("g.c", &["a.r1"])];
        let hit = || {
            group(
                "g.c",
                vec![
                    no_conflict_on("q.o", SolverVerdict::Sat),
                    contradiction_on("q.d", &["a.r1"]),
                ],
            )
        };
        let miss = group(
            "g.c",
            vec![
                no_conflict_on("q.o", SolverVerdict::Sat),
                no_conflict_on("q.d", SolverVerdict::Sat),
            ],
        );
        let miss_again = miss.clone();
        let route = route_metrics(
            &static_id("pipe.route"),
            &[obs(true, 1, 0, 0), obs(false, 3, 2, 2)],
            &[miss],
            &[vec![miss_again.clone()], vec![miss_again]],
            &reference,
        );
        let baseline = route_metrics(
            &static_id("pipe.base"),
            &[
                obs(true, 1, 0, 0),
                obs(true, 2, 1, 1),
                obs(true, 1, 0, 0),
                obs(false, 2, 1, 2),
            ],
            &[hit()],
            &[],
            &reference,
        );
        (route, baseline)
    }

    #[test]
    fn baseline_delta_rows_subtract_exact_and_signed() {
        let (route, baseline) = delta_fixture();
        // Raw sides, for the hand-derivation below: route acceptance 1/2,
        // accuracy 0/1, convergence 1/1, calls 4, repairs 2, schema 1/2,
        // syntactic 1/1; baseline 3/4, 1/1, n/a, 6, 2, 1/2, 1/1.
        let assembled = experiment_metrics(vec![route, baseline], &static_id("pipe.base"));
        assert_eq!(assembled.baseline_pipeline_id, static_id("pipe.base"));
        let [delta] = assembled.deltas.as_slice() else {
            panic!("one non-baseline delta table");
        };
        assert_eq!(delta.pipeline_id, static_id("pipe.route"));
        let row = |metric: &str, num: &str, den: &str| MetricRow {
            metric: static_id(metric),
            value: frac(num, den),
        };
        assert_eq!(
            delta.rows,
            vec![
                row(ACCEPTANCE_RATE, "-1", "4"),
                row(CONFLICT_VERDICT_ACCURACY, "-1", "1"),
                MetricRow {
                    metric: static_id(K_SAMPLE_CONVERGENCE),
                    value: MetricValue::NotApplicable,
                },
                row(RECORDED_CALL_COUNT, "-2", "1"),
                row(REPAIR_COUNT, "0", "1"),
                row(SCHEMA_VALID_RATE, "0", "1"),
                row(TARGET_SYNTACTIC_VALIDITY, "0", "1"),
            ]
        );
    }

    #[test]
    fn delta_union_covers_one_sided_metrics() {
        let side = |pipeline: &str, rows: Vec<(&str, &str, &str)>| RouteMetrics {
            pipeline_id: static_id(pipeline),
            rows: rows
                .into_iter()
                .map(|(metric, num, den)| MetricRow {
                    metric: static_id(metric),
                    value: frac(num, den),
                })
                .collect(),
            diagnostics: Vec::new(),
        };
        let route = side(
            "pipe.route",
            vec![("m.only_route", "1", "2"), ("m.shared", "1", "1")],
        );
        let baseline = side(
            "pipe.base",
            vec![("m.only_base", "2", "1"), ("m.shared", "1", "4")],
        );
        let rows = metric_deltas(&route, &baseline);
        assert_eq!(
            rows,
            vec![
                MetricRow {
                    metric: static_id("m.only_base"),
                    value: MetricValue::NotApplicable,
                },
                MetricRow {
                    metric: static_id("m.only_route"),
                    value: MetricValue::NotApplicable,
                },
                MetricRow {
                    metric: static_id("m.shared"),
                    value: frac("3", "4"),
                },
            ]
        );
    }

    #[test]
    fn raw_rows_emit_before_the_delta_table() {
        let (route, baseline) = delta_fixture();
        let assembled = experiment_metrics(
            vec![route.clone(), baseline.clone()],
            &static_id("pipe.base"),
        );
        // Every route's raw rows survive assembly in caller order; the
        // baseline gets no self-delta.
        assert_eq!(assembled.routes, vec![route, baseline]);
        assert_eq!(assembled.deltas.len(), 1);
        // §9 raw-rows-before-ranking: every raw-row section strictly
        // precedes every delta-table section.
        let sections = assembled.emission_order();
        assert_eq!(sections.len(), 3);
        let last_raw = sections
            .iter()
            .rposition(|s| matches!(s, MetricsSection::RawRows(_)))
            .expect("raw sections present");
        let first_delta = sections
            .iter()
            .position(|s| matches!(s, MetricsSection::DeltaTable(_)))
            .expect("delta section present");
        assert!(last_raw < first_delta, "raw rows precede the delta table");
        let [
            MetricsSection::RawRows(first),
            MetricsSection::RawRows(second),
            MetricsSection::DeltaTable(table),
        ] = sections.as_slice()
        else {
            panic!("two raw sections then one delta table");
        };
        assert_eq!(first.pipeline_id, static_id("pipe.route"));
        assert_eq!(second.pipeline_id, static_id("pipe.base"));
        assert_eq!(table.pipeline_id, static_id("pipe.route"));
    }

    #[test]
    #[should_panic(expected = "has no route metrics")]
    fn missing_baseline_is_a_wiring_bug() {
        let (route, _) = delta_fixture();
        experiment_metrics(vec![route], &static_id("pipe.base"));
    }

    #[test]
    #[should_panic(expected = "duplicate route metrics")]
    fn duplicate_route_metrics_is_a_wiring_bug() {
        let (route, _) = delta_fixture();
        experiment_metrics(vec![route.clone(), route], &static_id("pipe.route"));
    }

    #[test]
    #[should_panic(expected = "duplicate group observation")]
    fn duplicate_group_observation_is_a_route_bug() {
        let groups = vec![group("g.a", Vec::new()), group("g.a", Vec::new())];
        route_metrics(&static_id("pipe.test"), &[], &groups, &[], &[]);
    }

    #[test]
    #[should_panic(expected = "more schema violations than recorded calls")]
    fn violations_beyond_calls_are_a_fill_bug() {
        route_metrics(
            &static_id("pipe.test"),
            &[obs(false, 1, 0, 2)],
            &[],
            &[],
            &[],
        );
    }
}
