//! SPEC §5 `CompiledArtifact` — the compile processing_stage's durable payload: target
//! id, contradiction-query plan, per-query SMT-LIB bodies, the
//! named-assertion map, target metadata, diagnostics.
//!
//! Query text is embedded here and materialized byte-identically under
//! `groups/<gid>/smt/` for solver consumption (§6 SMT profile); §8.5 item 4
//! audits the assertion map. Planning over FormalIR lives in
//! [`plan_queries`](crate::plan_queries), §8.6-pinned emission in
//! [`emit_overlap_query`](crate::emit_overlap_query) /
//! [`emit_deontic_query`](crate::emit_deontic_query); this module owns the
//! shapes, their canonical bytes, and structural validation.

use ckc_core::{
    CanonError, CanonRead, CanonReadError, Canonical, ContradictionQueryPair, DiagnosticRecord, Id,
    ObjectEmitter, ObjectReader, Reader, emit_array, emit_map, emit_set, emit_string,
    fieldless_enum, read_array, read_map, read_set, read_string,
};

use crate::{SetBreak, first_set_break};

fieldless_enum! {
    /// SPEC §6 query logic: the narrowest sufficient SMT-LIB logic, recorded
    /// per query. Canonical and YAML form is the lowercase identifier_ascii
    /// spelling (§4.4); [`smt_token`](SmtLogic::smt_token) yields the
    /// uppercase standard token `(set-logic …)` lines print. M1's set:
    /// `QF_LRA` for context queries (Bool constants + linear-real interval
    /// atoms), `QF_UF` for deontic polarity queries; M4 adds difference
    /// logic, and anything richer stays behind declared target profiles
    /// (otherwise `unsupported_fragment`).
    SmtLogic {
        /// Quantifier-free linear real arithmetic.
        QfLra => "qf_lra",
        /// Quantifier-free uninterpreted functions.
        QfUf => "qf_uf",
    }
}

impl SmtLogic {
    /// The uppercase SMT-LIB standard token for emitted query text.
    pub fn smt_token(self) -> &'static str {
        match self {
            SmtLogic::QfLra => "QF_LRA",
            SmtLogic::QfUf => "QF_UF",
        }
    }
}

/// SPEC §5 "query bodies" element: one §6 query's id, its recorded logic,
/// and the exact SMT-LIB text. `body` rides byte-exact — no §4.2 policy
/// touches solver-bound text (JSON string escaping only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryBody {
    pub query_id: Id,
    pub logic: SmtLogic,
    pub body: String,
}

impl Canonical for QueryBody {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("body", |b| {
            emit_string(b, &self.body);
            Ok(())
        })?;
        obj.member("logic", |b| self.logic.emit_canonical(b))?;
        obj.member("query_id", |b| self.query_id.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for QueryBody {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let body = obj.member("body", read_string)?;
        let logic = obj.member("logic", SmtLogic::read)?;
        let query_id = obj.member("query_id", Id::read)?;
        obj.close()?;
        Ok(QueryBody {
            query_id,
            logic,
            body,
        })
    }
}

/// SPEC §5 named-assertion record — the value under its assertion-id key in
/// [`CompiledArtifact::assertion_to_source_map`], binding the assertion to IR rule ids
/// and source region ids (§8.5 item 4 audits exactly this). Both lists are
/// canonical sets: sorted by Id bytes, duplicate-free, non-empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionRecord {
    pub rule_ids: Vec<Id>,
    pub region_ids: Vec<Id>,
}

impl Canonical for AssertionRecord {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("region_ids", |b| emit_set(b, &self.region_ids))?;
        obj.member("rule_ids", |b| emit_set(b, &self.rule_ids))?;
        obj.finish(out)
    }
}

impl CanonRead for AssertionRecord {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let region_ids = obj.member("region_ids", read_set::<Id>)?;
        let rule_ids = obj.member("rule_ids", read_set::<Id>)?;
        obj.close()?;
        Ok(AssertionRecord {
            rule_ids,
            region_ids,
        })
    }
}

/// A raw-text map value: emit/read with no §4.2 policy, the crate-local twin
/// of ckc-core's internal raw-string wrapper.
struct RawValue(String);

impl Canonical for RawValue {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string(out, &self.0);
        Ok(())
    }
}

impl CanonRead for RawValue {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        Ok(RawValue(read_string(r)?))
    }
}

/// SPEC §5 `CompiledArtifact`: target id, logic, query plan, query bodies,
/// named-assertion records, diagnostics — plus the roadmap's target
/// metadata.
///
/// - `solver_query_plan` reuses the §5 [`ContradictionQueryPair`] slots planned
///   over FormalIR (minted by [`plan_queries`](crate::plan_queries)).
/// - `query_bodies` holds the §6 two-query texts in plan order: pair `k`'s
///   `context_overlap` then `deontic_consistency`.
/// - `assertion_to_source_map` binds §6 assertion names — `ctx.<rule_id>` (context)
///   and `a.<rule_id>` (polarity/factual) — to rule and region ids; map
///   semantics, stored key-sorted.
/// - `target_metadata` records identifier-keyed raw-text target facts; map
///   semantics, stored key-sorted.
/// - `diagnostics` carries the processing_stage's §7.4 records in production order.
///
/// [`validate`](Self::validate) enforces the structural invariants; emission
/// policy (which logic per slot, the §8.6 byte shapes) lives with the emit
/// module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledArtifact {
    pub target_id: Id,
    pub solver_query_plan: Vec<ContradictionQueryPair>,
    pub query_bodies: Vec<QueryBody>,
    pub assertion_to_source_map: Vec<(Id, AssertionRecord)>,
    pub target_metadata: Vec<(Id, String)>,
    pub diagnostics: Vec<DiagnosticRecord>,
}

impl CompiledArtifact {
    /// Structural invariants, first break wins:
    ///
    /// 1. Plan pairs: `constraint_a_id < constraint_b_id` by id bytes (§5
    ///    bundle invariant restated locally); pair ids and query ids each
    ///    unique across the plan.
    /// 2. Bodies ↔ plan: exactly two bodies per pair, in plan order
    ///    (`context_overlap` then `deontic_consistency`), none empty.
    /// 3. Assertion map: keys stored in canonical map order; every record's
    ///    `rule_ids`/`region_ids` non-empty canonical sets; every assertion
    ///    id is `ctx.<r>` or `a.<r>` with `r` among its record's `rule_ids`
    ///    (§6 assertion-id forms).
    /// 4. Target metadata: keys stored in canonical map order.
    pub fn validate(&self) -> Result<(), ArtifactError> {
        let mut pair_ids: Vec<&str> = Vec::new();
        let mut query_ids: Vec<&str> = Vec::new();
        for pair in &self.solver_query_plan {
            if pair.constraint_a_id.as_str() >= pair.constraint_b_id.as_str() {
                return Err(ArtifactError::PairInvalid {
                    pair_id: pair.pair_id.clone(),
                    rule: "constraint_a_id < constraint_b_id",
                });
            }
            if pair_ids.contains(&pair.pair_id.as_str()) {
                return Err(ArtifactError::Duplicate {
                    pool: "pair_id",
                    id: pair.pair_id.clone(),
                });
            }
            pair_ids.push(pair.pair_id.as_str());
            for qid in [
                &pair.context_overlap_query_id,
                &pair.deontic_consistency_query_id,
            ] {
                if query_ids.contains(&qid.as_str()) {
                    return Err(ArtifactError::Duplicate {
                        pool: "query_id",
                        id: qid.clone(),
                    });
                }
                query_ids.push(qid.as_str());
            }
        }
        if self.query_bodies.len() != query_ids.len() {
            return Err(ArtifactError::QueryCount {
                expected: query_ids.len(),
                found: self.query_bodies.len(),
            });
        }
        for (index, (body, want)) in self.query_bodies.iter().zip(&query_ids).enumerate() {
            if body.query_id.as_str() != *want {
                return Err(ArtifactError::QuerySlotMismatch {
                    index,
                    expected: Id::new(*want).expect("plan ids are valid"),
                    found: body.query_id.clone(),
                });
            }
            if body.body.is_empty() {
                return Err(ArtifactError::EmptyBody(body.query_id.clone()));
            }
        }
        check_set(
            "assertion_to_source_map",
            self.assertion_to_source_map.iter().map(|(k, _)| k),
        )?;
        for (assertion_id, record) in &self.assertion_to_source_map {
            if record.rule_ids.is_empty() || record.region_ids.is_empty() {
                return Err(ArtifactError::EmptyAssertionRefs(assertion_id.clone()));
            }
            check_set("rule_ids", record.rule_ids.iter())?;
            check_set("region_ids", record.region_ids.iter())?;
            let suffix = assertion_id
                .as_str()
                .strip_prefix("ctx.")
                .or_else(|| assertion_id.as_str().strip_prefix("a."));
            let bound = suffix.is_some_and(|r| record.rule_ids.iter().any(|id| id.as_str() == r));
            if !bound {
                return Err(ArtifactError::AssertionForm(assertion_id.clone()));
            }
        }
        check_set(
            "target_metadata",
            self.target_metadata.iter().map(|(k, _)| k),
        )?;
        Ok(())
    }
}

impl Canonical for CompiledArtifact {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("assertion_to_source_map", |b| {
            emit_map(b, self.assertion_to_source_map.iter().map(|(k, v)| (k, v)))
        })?;
        obj.member("diagnostics", |b| emit_array(b, &self.diagnostics))?;
        obj.member("query_bodies", |b| emit_array(b, &self.query_bodies))?;
        obj.member("solver_query_plan", |b| {
            emit_array(b, &self.solver_query_plan)
        })?;
        obj.member("target_id", |b| self.target_id.emit_canonical(b))?;
        obj.member("target_metadata", |b| {
            let texts: Vec<RawValue> = self
                .target_metadata
                .iter()
                .map(|(_, v)| RawValue(v.clone()))
                .collect();
            emit_map(b, self.target_metadata.iter().map(|(k, _)| k).zip(&texts))
        })?;
        obj.finish(out)
    }
}

impl CanonRead for CompiledArtifact {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let assertion_to_source_map =
            obj.member("assertion_to_source_map", read_map::<Id, AssertionRecord>)?;
        let diagnostics = obj.member("diagnostics", read_array::<DiagnosticRecord>)?;
        let query_bodies = obj.member("query_bodies", read_array::<QueryBody>)?;
        let solver_query_plan =
            obj.member("solver_query_plan", read_array::<ContradictionQueryPair>)?;
        let target_id = obj.member("target_id", Id::read)?;
        let target_metadata = obj.member("target_metadata", |r| {
            Ok(read_map::<Id, RawValue>(r)?
                .into_iter()
                .map(|(k, v)| (k, v.0))
                .collect::<Vec<(Id, String)>>())
        })?;
        obj.close()?;
        Ok(CompiledArtifact {
            target_id,
            solver_query_plan,
            query_bodies,
            assertion_to_source_map,
            target_metadata,
            diagnostics,
        })
    }
}

/// Enforce canonical-set/map-key storage so stored values equal their
/// strict-read round trip.
fn check_set<'a>(
    pool: &'static str,
    ids: impl IntoIterator<Item = &'a Id>,
) -> Result<(), ArtifactError> {
    match first_set_break(ids) {
        None => Ok(()),
        Some((SetBreak::Duplicate, id)) => Err(ArtifactError::Duplicate {
            pool,
            id: id.clone(),
        }),
        Some((SetBreak::Unsorted, id)) => Err(ArtifactError::Unsorted {
            pool,
            id: id.clone(),
        }),
    }
}

/// A [`CompiledArtifact`] broke a structural invariant
/// ([`CompiledArtifact::validate`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactError {
    /// `query_bodies` length differs from two per plan pair.
    QueryCount { expected: usize, found: usize },
    /// The body at `index` names a query id other than its plan slot's.
    QuerySlotMismatch {
        index: usize,
        expected: Id,
        found: Id,
    },
    /// The named query's body text is empty.
    EmptyBody(Id),
    /// A plan pair breaks a §6 eligibility invariant (`rule` names it).
    PairInvalid { pair_id: Id, rule: &'static str },
    /// Two entities in one id pool share an id (`pool` names it).
    Duplicate { pool: &'static str, id: Id },
    /// A stored set or map keeps `id` out of canonical order (`pool` names
    /// it).
    Unsorted { pool: &'static str, id: Id },
    /// The named assertion id lacks the §6 `a.<r>`/`ctx.<r>` form over its
    /// record's `rule_ids`.
    AssertionForm(Id),
    /// The named assertion record's `rule_ids` or `region_ids` set is empty.
    EmptyAssertionRefs(Id),
}

impl std::fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactError::QueryCount { expected, found } => {
                write!(
                    f,
                    "query_bodies holds {found} bodies, plan demands {expected}"
                )
            }
            ArtifactError::QuerySlotMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "query_bodies[{index}] is {found}, plan slot demands {expected}"
            ),
            ArtifactError::EmptyBody(id) => write!(f, "query {id} has an empty body"),
            ArtifactError::PairInvalid { pair_id, rule } => {
                write!(f, "plan pair {pair_id} breaks {rule}")
            }
            ArtifactError::Duplicate { pool, id } => write!(f, "duplicate {pool} id {id}"),
            ArtifactError::Unsorted { pool, id } => {
                write!(f, "{pool} stores {id} out of canonical order")
            }
            ArtifactError::AssertionForm(id) => {
                write!(f, "assertion id {id} is not a./ctx. over its rule_ids")
            }
            ArtifactError::EmptyAssertionRefs(id) => {
                write!(f, "assertion {id} has empty rule_ids or region_ids")
            }
        }
    }
}

impl std::error::Error for ArtifactError {}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::{DiagnosticCode, Hash, Outcome, canonical_payload_bytes, read_strict_canonical};

    /// Canonical bytes of `value` as a UTF-8 string, for exact-match assertions.
    fn canon<T: Canonical>(value: &T) -> String {
        String::from_utf8(canonical_payload_bytes(value).unwrap()).unwrap()
    }

    /// Assert `value` survives a canonical write -> read round trip unchanged.
    fn round_trip<T: Canonical + CanonRead + std::fmt::Debug + PartialEq>(value: T) {
        let bytes = canonical_payload_bytes(&value).unwrap();
        let got: T = read_strict_canonical(&bytes).unwrap();
        assert_eq!(got, value, "round trip changed the value");
    }

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    /// The §8.6 worked-thread pair: docA rule.0 (for) × docB rule.0
    /// (contraindicate) over the shared administer-abx_a action key.
    fn sample_pair() -> ContradictionQueryPair {
        ContradictionQueryPair {
            pair_id: id("q.m1_conflict.pair1"),
            action_key: id("act.administer:drug.abx_a"),
            constraint_a_id: id("fc.test_source.m1_guideline_a.rule.0"),
            constraint_b_id: id("fc.test_source.m1_guideline_b.rule.0"),
            context_overlap_query_id: id("q.m1_conflict.pair1.overlap"),
            deontic_consistency_query_id: id("q.m1_conflict.pair1.deontic"),
        }
    }

    /// A second well-formed pair for uniqueness checks.
    fn other_pair() -> ContradictionQueryPair {
        ContradictionQueryPair {
            pair_id: id("q.m1_conflict.pair2"),
            action_key: id("act.administer:drug.abx_a"),
            constraint_a_id: id("fc.test_source.m1_guideline_a.rule.1"),
            constraint_b_id: id("fc.test_source.m1_guideline_b.rule.1"),
            context_overlap_query_id: id("q.m1_conflict.pair2.overlap"),
            deontic_consistency_query_id: id("q.m1_conflict.pair2.deontic"),
        }
    }

    fn record(rule: &str, regions: &[&str]) -> AssertionRecord {
        AssertionRecord {
            rule_ids: vec![id(rule)],
            region_ids: regions.iter().map(|r| id(r)).collect(),
        }
    }

    /// §8.6-grounded sample: the worked-thread pair, two small query bodies
    /// in plan order, all four named assertions mapped to their rules and
    /// regions (docA grounds in r.2+r.3 — recommendation + exception spans —
    /// docB in r.2).
    fn sample_artifact() -> CompiledArtifact {
        let rule_a = "test_source.m1_guideline_a.rule.0";
        let rule_b = "test_source.m1_guideline_b.rule.0";
        CompiledArtifact {
            target_id: id("target.smtlib2"),
            solver_query_plan: vec![sample_pair()],
            query_bodies: vec![
                QueryBody {
                    query_id: id("q.m1_conflict.pair1.overlap"),
                    logic: SmtLogic::QfLra,
                    body: "(set-logic QF_LRA) (check-sat)".to_owned(),
                },
                QueryBody {
                    query_id: id("q.m1_conflict.pair1.deontic"),
                    logic: SmtLogic::QfUf,
                    body: "(set-logic QF_UF) (check-sat)".to_owned(),
                },
            ],
            assertion_to_source_map: vec![
                (
                    id("a.test_source.m1_guideline_a.rule.0"),
                    record(rule_a, &["r.2", "r.3"]),
                ),
                (
                    id("a.test_source.m1_guideline_b.rule.0"),
                    record(rule_b, &["r.2"]),
                ),
                (
                    id("ctx.test_source.m1_guideline_a.rule.0"),
                    record(rule_a, &["r.2", "r.3"]),
                ),
                (
                    id("ctx.test_source.m1_guideline_b.rule.0"),
                    record(rule_b, &["r.2"]),
                ),
            ],
            target_metadata: vec![(id("smtlib_version"), "2.6".to_owned())],
            diagnostics: vec![],
        }
    }

    fn empty_artifact() -> CompiledArtifact {
        CompiledArtifact {
            target_id: id("target.smtlib2"),
            solver_query_plan: vec![],
            query_bodies: vec![],
            assertion_to_source_map: vec![],
            target_metadata: vec![],
            diagnostics: vec![],
        }
    }

    // Pins the lowercase canonical spelling against the uppercase SMT-LIB
    // token: the payload stays identifier_ascii while emitted text prints
    // the standard form.
    #[test]
    fn logic_spellings() {
        assert_eq!(canon(&SmtLogic::QfLra), "\"qf_lra\"");
        assert_eq!(canon(&SmtLogic::QfUf), "\"qf_uf\"");
        assert_eq!(SmtLogic::QfLra.smt_token(), "QF_LRA");
        assert_eq!(SmtLogic::QfUf.smt_token(), "QF_UF");
        assert_eq!(SmtLogic::parse("qf_lra").unwrap(), SmtLogic::QfLra);
        // The uppercase token is not a canonical value.
        assert!(SmtLogic::parse("QF_LRA").is_err());
    }

    // Pins the §5 CompiledArtifact canonical shape over §8.6 values: every
    // field, byte-sorted members, map-form assertion records, plan-order
    // bodies with JSON-escaped newlines.
    #[test]
    fn compiled_artifact_canonical_bytes() {
        assert_eq!(
            canon(&sample_artifact()),
            concat!(
                r#"{"assertion_to_source_map":{"#,
                r#""a.test_source.m1_guideline_a.rule.0":{"region_ids":["r.2","r.3"],"#,
                r#""rule_ids":["test_source.m1_guideline_a.rule.0"]},"#,
                r#""a.test_source.m1_guideline_b.rule.0":{"region_ids":["r.2"],"#,
                r#""rule_ids":["test_source.m1_guideline_b.rule.0"]},"#,
                r#""ctx.test_source.m1_guideline_a.rule.0":{"region_ids":["r.2","r.3"],"#,
                r#""rule_ids":["test_source.m1_guideline_a.rule.0"]},"#,
                r#""ctx.test_source.m1_guideline_b.rule.0":{"region_ids":["r.2"],"#,
                r#""rule_ids":["test_source.m1_guideline_b.rule.0"]}},"#,
                r#""diagnostics":[],"#,
                r#""query_bodies":["#,
                r#"{"body":"(set-logic QF_LRA) (check-sat)","logic":"qf_lra","#,
                r#""query_id":"q.m1_conflict.pair1.overlap"},"#,
                r#"{"body":"(set-logic QF_UF) (check-sat)","logic":"qf_uf","#,
                r#""query_id":"q.m1_conflict.pair1.deontic"}],"#,
                r#""solver_query_plan":[{"action_key":"act.administer:drug.abx_a","#,
                r#""constraint_a_id":"fc.test_source.m1_guideline_a.rule.0","#,
                r#""constraint_b_id":"fc.test_source.m1_guideline_b.rule.0","#,
                r#""context_overlap_query_id":"q.m1_conflict.pair1.overlap","#,
                r#""deontic_consistency_query_id":"q.m1_conflict.pair1.deontic","#,
                r#""pair_id":"q.m1_conflict.pair1"}],"#,
                r#""target_id":"target.smtlib2","#,
                r#""target_metadata":{"smtlib_version":"2.6"}}"#
            )
        );
        // Empty collections keep their type-guided forms ({} maps, [] arrays).
        assert_eq!(
            canon(&empty_artifact()),
            concat!(
                r#"{"assertion_to_source_map":{},"diagnostics":[],"query_bodies":[],"#,
                r#""solver_query_plan":[],"target_id":"target.smtlib2","target_metadata":{}}"#
            )
        );
    }

    #[test]
    fn compiled_artifact_round_trips() {
        round_trip(sample_artifact());
        round_trip(empty_artifact());
        // With multi-line body text (the §4.3 control-character escape path),
        // a diagnostic, and a second metadata row.
        let mut full = sample_artifact();
        full.query_bodies[0].body = "(set-logic QF_LRA)\n(check-sat)\n(get-model)\n".to_owned();
        full.target_metadata = vec![
            (id("profile"), "m1".to_owned()),
            (id("smtlib_version"), "2.6".to_owned()),
        ];
        full.diagnostics = vec![DiagnosticRecord {
            code: DiagnosticCode::UnsupportedIrFragment,
            outcome: Outcome::Unsupported,
            payload: vec![(id("note"), "sample".to_owned())],
            region_ids: vec![id("r.2")],
            artifact_hashes: vec![Hash::new(format!("sha256:{}", "a".repeat(64))).unwrap()],
        }];
        round_trip(full);
    }

    #[test]
    fn validation_accepts_samples() {
        assert_eq!(sample_artifact().validate(), Ok(()));
        assert_eq!(empty_artifact().validate(), Ok(()));
        let mut two = sample_artifact();
        two.solver_query_plan.push(other_pair());
        two.query_bodies.extend([
            QueryBody {
                query_id: id("q.m1_conflict.pair2.overlap"),
                logic: SmtLogic::QfLra,
                body: "(check-sat)\n".to_owned(),
            },
            QueryBody {
                query_id: id("q.m1_conflict.pair2.deontic"),
                logic: SmtLogic::QfUf,
                body: "(check-sat)\n".to_owned(),
            },
        ]);
        assert_eq!(two.validate(), Ok(()));
    }

    #[test]
    fn validation_rejects_plan_breaks() {
        // Constraint order: a >= b.
        let mut swapped = sample_artifact();
        let pair = &mut swapped.solver_query_plan[0];
        std::mem::swap(&mut pair.constraint_a_id, &mut pair.constraint_b_id);
        assert_eq!(
            swapped.validate(),
            Err(ArtifactError::PairInvalid {
                pair_id: id("q.m1_conflict.pair1"),
                rule: "constraint_a_id < constraint_b_id",
            })
        );
        // Duplicate pair id (second pair reuses pair1's id).
        let mut dup_pair = sample_artifact();
        let mut clone = other_pair();
        clone.pair_id = id("q.m1_conflict.pair1");
        dup_pair.solver_query_plan.push(clone);
        assert_eq!(
            dup_pair.validate(),
            Err(ArtifactError::Duplicate {
                pool: "pair_id",
                id: id("q.m1_conflict.pair1"),
            })
        );
        // Duplicate query id across pairs.
        let mut dup_query = sample_artifact();
        let mut clone = other_pair();
        clone.context_overlap_query_id = id("q.m1_conflict.pair1.overlap");
        dup_query.solver_query_plan.push(clone);
        assert_eq!(
            dup_query.validate(),
            Err(ArtifactError::Duplicate {
                pool: "query_id",
                id: id("q.m1_conflict.pair1.overlap"),
            })
        );
    }

    #[test]
    fn validation_rejects_body_breaks() {
        // Count: one body dropped.
        let mut short = sample_artifact();
        short.query_bodies.pop();
        assert_eq!(
            short.validate(),
            Err(ArtifactError::QueryCount {
                expected: 2,
                found: 1,
            })
        );
        // Order: bodies swapped out of plan order.
        let mut swapped = sample_artifact();
        swapped.query_bodies.swap(0, 1);
        assert_eq!(
            swapped.validate(),
            Err(ArtifactError::QuerySlotMismatch {
                index: 0,
                expected: id("q.m1_conflict.pair1.overlap"),
                found: id("q.m1_conflict.pair1.deontic"),
            })
        );
        // Empty body text.
        let mut empty = sample_artifact();
        empty.query_bodies[1].body.clear();
        assert_eq!(
            empty.validate(),
            Err(ArtifactError::EmptyBody(id("q.m1_conflict.pair1.deontic")))
        );
    }

    #[test]
    fn validation_rejects_assertion_breaks() {
        // Keys stored out of canonical map order.
        let mut unsorted = sample_artifact();
        unsorted.assertion_to_source_map.swap(0, 1);
        assert_eq!(
            unsorted.validate(),
            Err(ArtifactError::Unsorted {
                pool: "assertion_to_source_map",
                id: id("a.test_source.m1_guideline_a.rule.0"),
            })
        );
        // Duplicate key.
        let mut dup = sample_artifact();
        let entry = dup.assertion_to_source_map[0].clone();
        dup.assertion_to_source_map.insert(1, entry);
        assert_eq!(
            dup.validate(),
            Err(ArtifactError::Duplicate {
                pool: "assertion_to_source_map",
                id: id("a.test_source.m1_guideline_a.rule.0"),
            })
        );
        // Prefix outside a./ctx. (key sits in sorted position, so the form
        // check itself fires).
        let mut form = sample_artifact();
        form.assertion_to_source_map[3].0 = id("z.test_source.m1_guideline_b.rule.0");
        assert_eq!(
            form.validate(),
            Err(ArtifactError::AssertionForm(id(
                "z.test_source.m1_guideline_b.rule.0"
            )))
        );
        // Suffix rule absent from rule_ids.
        let mut unbound = sample_artifact();
        unbound.assertion_to_source_map[0].1.rule_ids =
            vec![id("test_source.m1_guideline_b.rule.0")];
        assert_eq!(
            unbound.validate(),
            Err(ArtifactError::AssertionForm(id(
                "a.test_source.m1_guideline_a.rule.0"
            )))
        );
        // Empty refs.
        let mut refs = sample_artifact();
        refs.assertion_to_source_map[0].1.region_ids.clear();
        assert_eq!(
            refs.validate(),
            Err(ArtifactError::EmptyAssertionRefs(id(
                "a.test_source.m1_guideline_a.rule.0"
            )))
        );
        // Region set stored unsorted.
        let mut regions = sample_artifact();
        regions.assertion_to_source_map[0].1.region_ids = vec![id("r.3"), id("r.2")];
        assert_eq!(
            regions.validate(),
            Err(ArtifactError::Unsorted {
                pool: "region_ids",
                id: id("r.2"),
            })
        );
    }

    #[test]
    fn validation_rejects_metadata_breaks() {
        let mut unsorted = sample_artifact();
        unsorted.target_metadata = vec![
            (id("smtlib_version"), "2.6".to_owned()),
            (id("profile"), "m1".to_owned()),
        ];
        assert_eq!(
            unsorted.validate(),
            Err(ArtifactError::Unsorted {
                pool: "target_metadata",
                id: id("profile"),
            })
        );
    }
}
