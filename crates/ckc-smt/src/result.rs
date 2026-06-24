//! SPEC §5 `VerifierResult` — the verify processing_stage's durable per-query payload:
//! §6 category, raw solver verdict kept distinct, satisfying_example model or canonical
//! unsat core, solver identity, diagnostics.
//!
//! The §6 mapping from raw solver outcomes to categories is the verify
//! adapter's job (smt-verify); this module owns the value sets, the record
//! shape, its canonical bytes, and the coherence rules every record obeys
//! regardless of producer (§6: model on sat where relevant, unsat core on
//! unsat, cores as canonical Id sets sorted by canonical_sort_key).

use ckc_core::{
    CanonError, CanonRead, CanonReadError, Canonical, DiagnosticRecord, Id, ObjectEmitter,
    ObjectReader, Reader, SolverIdentity, emit_array, emit_set, emit_string, fieldless_enum,
    read_array, read_set, read_string,
};

use crate::{SetBreak, first_set_break};

fieldless_enum! {
    /// SPEC §6 verifier result category — the per-query status every
    /// [`VerifierResult`] carries, kept distinct from the raw
    /// [`SolverVerdict`] token.
    VerifierCategory {
        /// An input artifact failed schema or canonical acceptance.
        SchemaFailure => "schema_failure",
        /// The compile processing_stage failed to produce the query.
        CompilerFailure => "compiler_failure",
        /// The solver rejected the query text (parse error).
        TargetSyntaxFailure => "target_syntax_failure",
        /// The solver process failed to execute or crashed.
        SolverExecutionFailure => "solver_execution_failure",
        /// The query completed without witnessing a contradiction.
        SemanticNoConflict => "semantic_no_conflict",
        /// The query's unsat verdict witnesses a contradiction; the unsat
        /// core names the contributing assertions.
        SemanticContradiction => "semantic_contradiction",
        /// The solver returned unknown or hit its per-query budget (the
        /// §7.4 solver_unknown/solver_timeout diagnostics stay distinct).
        Unknown => "unknown",
        /// An out-of-profile construct was gated before solving (§6 SMT
        /// profile).
        UnsupportedFragment => "unsupported_fragment",
    }
}

fieldless_enum! {
    /// SPEC §6 raw solver verdict, preserved distinctly from the category:
    /// `sat`/`unsat`/`unknown` as the solver printed it, `timeout` when the
    /// per-query budget killed the run before a token.
    SolverVerdict {
        Sat => "sat",
        Unsat => "unsat",
        Unknown => "unknown",
        Timeout => "timeout",
    }
}

/// SPEC §5 `VerifierResult`: per-query status (§6 categories), model or
/// unsat core, solver identity, diagnostics. One record per planned §6
/// query; aggregation into the run layout lands with the verify and run
/// units.
///
/// Optional fields omit canonically when absent (§4.3): a failure category
/// carries no verdict, a sat verdict may carry a satisfying_example `model` (§6 "where
/// relevant"), an unsat verdict may carry `unsat_core` — assertion names
/// normalized to a canonical Id set sorted by canonical_sort_key. `model`
/// rides byte-exact as the solver printed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifierResult {
    pub query_id: Id,
    pub category: VerifierCategory,
    pub verdict: Option<SolverVerdict>,
    pub model: Option<String>,
    pub unsat_core: Option<Vec<Id>>,
    pub solver_identity: SolverIdentity,
    pub diagnostics: Vec<DiagnosticRecord>,
}

impl VerifierResult {
    /// Coherence invariants, first break wins:
    ///
    /// 1. `model` only with verdict `sat`; non-empty.
    /// 2. `unsat_core` only with verdict `unsat`; a non-empty canonical set
    ///    (sorted by Id bytes, duplicate-free).
    /// 3. Category ↔ verdict: `semantic_contradiction` requires `unsat`;
    ///    `semantic_no_conflict` requires `sat` or `unsat`; `unknown`
    ///    requires `unknown` or `timeout`; the failure categories
    ///    (`schema_failure`, `compiler_failure`, `target_syntax_failure`,
    ///    `solver_execution_failure`, `unsupported_fragment`) carry no
    ///    verdict.
    pub fn validate(&self) -> Result<(), VerifierError> {
        if self.model.is_some() && self.verdict != Some(SolverVerdict::Sat) {
            return Err(VerifierError::ModelWithoutSat);
        }
        if let Some(model) = &self.model
            && model.is_empty()
        {
            return Err(VerifierError::EmptyModel);
        }
        if self.unsat_core.is_some() && self.verdict != Some(SolverVerdict::Unsat) {
            return Err(VerifierError::CoreWithoutUnsat);
        }
        if let Some(core) = &self.unsat_core {
            if core.is_empty() {
                return Err(VerifierError::EmptyCore);
            }
            match first_set_break(core) {
                None => {}
                Some((SetBreak::Duplicate, id)) => {
                    return Err(VerifierError::Duplicate {
                        pool: "unsat_core",
                        id: id.clone(),
                    });
                }
                Some((SetBreak::Unsorted, id)) => {
                    return Err(VerifierError::Unsorted {
                        pool: "unsat_core",
                        id: id.clone(),
                    });
                }
            }
        }
        let (coherent, rule) = match self.category {
            VerifierCategory::SemanticContradiction => (
                self.verdict == Some(SolverVerdict::Unsat),
                "semantic_contradiction requires verdict unsat",
            ),
            VerifierCategory::SemanticNoConflict => (
                matches!(
                    self.verdict,
                    Some(SolverVerdict::Sat) | Some(SolverVerdict::Unsat)
                ),
                "semantic_no_conflict requires verdict sat or unsat",
            ),
            VerifierCategory::Unknown => (
                matches!(
                    self.verdict,
                    Some(SolverVerdict::Unknown) | Some(SolverVerdict::Timeout)
                ),
                "unknown requires verdict unknown or timeout",
            ),
            VerifierCategory::SchemaFailure
            | VerifierCategory::CompilerFailure
            | VerifierCategory::TargetSyntaxFailure
            | VerifierCategory::SolverExecutionFailure
            | VerifierCategory::UnsupportedFragment => (
                self.verdict.is_none(),
                "failure categories carry no verdict",
            ),
        };
        if !coherent {
            return Err(VerifierError::CategoryVerdict {
                category: self.category,
                rule,
            });
        }
        Ok(())
    }
}

impl Canonical for VerifierResult {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("category", |b| self.category.emit_canonical(b))?;
        obj.member("diagnostics", |b| emit_array(b, &self.diagnostics))?;
        obj.optional("model", self.model.as_deref(), |b, m| {
            emit_string(b, m);
            Ok(())
        })?;
        obj.member("query_id", |b| self.query_id.emit_canonical(b))?;
        obj.member("solver_identity", |b| {
            self.solver_identity.emit_canonical(b)
        })?;
        obj.optional("unsat_core", self.unsat_core.as_deref(), |b, core| {
            emit_set(b, core)
        })?;
        obj.optional("verdict", self.verdict, |b, v| v.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for VerifierResult {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let category = obj.member("category", VerifierCategory::read)?;
        let diagnostics = obj.member("diagnostics", read_array::<DiagnosticRecord>)?;
        let model = obj.optional("model", read_string)?;
        let query_id = obj.member("query_id", Id::read)?;
        let solver_identity = obj.member("solver_identity", SolverIdentity::read)?;
        let unsat_core = obj.optional("unsat_core", read_set::<Id>)?;
        let verdict = obj.optional("verdict", SolverVerdict::read)?;
        obj.close()?;
        Ok(VerifierResult {
            query_id,
            category,
            verdict,
            model,
            unsat_core,
            solver_identity,
            diagnostics,
        })
    }
}

/// SPEC §8.3 verify-processing_stage durable artifact: one test_source group's results in
/// plan order — pair k's `context_overlap`, then its `deontic_consistency`
/// when Q1 answered sat — the payload of the run layout's
/// `groups/<gid>/verifier_results.json`. Order is producer-owned:
/// [`verify`](crate::verify) yields it by construction, and the compiled
/// artifact's plan stays the order source of truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifierResults {
    pub results: Vec<VerifierResult>,
}

impl VerifierResults {
    /// Every member satisfies [`VerifierResult::validate`]; first break
    /// wins.
    pub fn validate(&self) -> Result<(), VerifierError> {
        self.results.iter().try_for_each(VerifierResult::validate)
    }
}

impl Canonical for VerifierResults {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("results", |b| emit_array(b, &self.results))?;
        obj.finish(out)
    }
}

impl CanonRead for VerifierResults {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let results = obj.member("results", read_array::<VerifierResult>)?;
        obj.close()?;
        Ok(VerifierResults { results })
    }
}

/// A [`VerifierResult`] broke a coherence invariant
/// ([`VerifierResult::validate`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifierError {
    /// A satisfying_example model rides on a verdict other than `sat`.
    ModelWithoutSat,
    /// An unsat core rides on a verdict other than `unsat`.
    CoreWithoutUnsat,
    /// The satisfying_example model text is empty.
    EmptyModel,
    /// The unsat core set is empty.
    EmptyCore,
    /// Two core entries share an id (`pool` names the set).
    Duplicate { pool: &'static str, id: Id },
    /// A stored set keeps `id` out of canonical order (`pool` names it).
    Unsorted { pool: &'static str, id: Id },
    /// The category disagrees with the verdict (`rule` names the §6
    /// expectation).
    CategoryVerdict {
        category: VerifierCategory,
        rule: &'static str,
    },
}

impl std::fmt::Display for VerifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifierError::ModelWithoutSat => write!(f, "model without a sat verdict"),
            VerifierError::CoreWithoutUnsat => write!(f, "unsat core without an unsat verdict"),
            VerifierError::EmptyModel => write!(f, "empty satisfying_example model"),
            VerifierError::EmptyCore => write!(f, "empty unsat core"),
            VerifierError::Duplicate { pool, id } => write!(f, "duplicate {pool} id {id}"),
            VerifierError::Unsorted { pool, id } => {
                write!(f, "{pool} stores {id} out of canonical order")
            }
            VerifierError::CategoryVerdict { category, rule } => {
                write!(f, "category {} breaks: {rule}", category.as_str())
            }
        }
    }
}

impl std::error::Error for VerifierError {}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::{DiagnosticCode, Outcome, canonical_payload_bytes, read_strict_canonical};

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

    fn z3() -> SolverIdentity {
        SolverIdentity {
            solver_id: id("z3"),
            version: "4.13.4".to_owned(),
        }
    }

    /// §8.6 Q1: context overlap sat, satisfying_example model recorded.
    fn q1_sat() -> VerifierResult {
        VerifierResult {
            query_id: id("q.m1_conflict.pair1.overlap"),
            category: VerifierCategory::SemanticNoConflict,
            verdict: Some(SolverVerdict::Sat),
            model: Some("((define-fun |q.age_years| () Real 18.0))".to_owned()),
            unsat_core: None,
            solver_identity: z3(),
            diagnostics: vec![],
        }
    }

    /// §8.6 Q2: deontic consistency unsat, the cross-document core.
    fn q2_unsat() -> VerifierResult {
        VerifierResult {
            query_id: id("q.m1_conflict.pair1.deontic"),
            category: VerifierCategory::SemanticContradiction,
            verdict: Some(SolverVerdict::Unsat),
            model: None,
            unsat_core: Some(vec![
                id("a.test_source.m1_guideline_a.rule.0"),
                id("a.test_source.m1_guideline_b.rule.0"),
            ]),
            solver_identity: z3(),
            diagnostics: vec![],
        }
    }

    /// A failure-category record: no verdict, diagnostics carry the detail.
    fn syntax_failure() -> VerifierResult {
        VerifierResult {
            query_id: id("q.m1_conflict.pair1.overlap"),
            category: VerifierCategory::TargetSyntaxFailure,
            verdict: None,
            model: None,
            unsat_core: None,
            solver_identity: z3(),
            diagnostics: vec![DiagnosticRecord {
                code: DiagnosticCode::TargetParseError,
                outcome: Outcome::Invalid,
                payload: vec![(id("line"), "3".to_owned())],
                region_ids: vec![],
                artifact_hashes: vec![],
            }],
        }
    }

    // Pins the §5 VerifierResult canonical shape over the §8.6 outcomes:
    // byte-sorted members, optional-omit fields, the core as a canonical set.
    #[test]
    fn verifier_result_canonical_bytes() {
        assert_eq!(
            canon(&q1_sat()),
            concat!(
                r#"{"category":"semantic_no_conflict","diagnostics":[],"#,
                r#""model":"((define-fun |q.age_years| () Real 18.0))","#,
                r#""query_id":"q.m1_conflict.pair1.overlap","#,
                r#""solver_identity":{"solver_id":"z3","version":"4.13.4"},"#,
                r#""verdict":"sat"}"#
            )
        );
        assert_eq!(
            canon(&q2_unsat()),
            concat!(
                r#"{"category":"semantic_contradiction","diagnostics":[],"#,
                r#""query_id":"q.m1_conflict.pair1.deontic","#,
                r#""solver_identity":{"solver_id":"z3","version":"4.13.4"},"#,
                r#""unsat_core":["a.test_source.m1_guideline_a.rule.0","#,
                r#""a.test_source.m1_guideline_b.rule.0"],"verdict":"unsat"}"#
            )
        );
    }

    #[test]
    fn verifier_result_round_trips() {
        round_trip(q1_sat());
        round_trip(q2_unsat());
        round_trip(syntax_failure());
    }

    #[test]
    fn validation_accepts_samples() {
        assert_eq!(q1_sat().validate(), Ok(()));
        assert_eq!(q2_unsat().validate(), Ok(()));
        assert_eq!(syntax_failure().validate(), Ok(()));
        // A documented no-conflict result Q1 unsat (disjoint intervals): no model,
        // no core.
        let mut no_conflict = q1_sat();
        no_conflict.verdict = Some(SolverVerdict::Unsat);
        no_conflict.model = None;
        assert_eq!(no_conflict.validate(), Ok(()));
        // Unknown via timeout keeps the raw token distinct.
        let mut timeout = q1_sat();
        timeout.category = VerifierCategory::Unknown;
        timeout.verdict = Some(SolverVerdict::Timeout);
        timeout.model = None;
        assert_eq!(timeout.validate(), Ok(()));
    }

    #[test]
    fn validation_rejects_evidence_breaks() {
        // Model on unsat.
        let mut model_unsat = q2_unsat();
        model_unsat.model = Some("(model)".to_owned());
        assert_eq!(model_unsat.validate(), Err(VerifierError::ModelWithoutSat));
        // Model with no verdict at all.
        let mut model_none = syntax_failure();
        model_none.model = Some("(model)".to_owned());
        assert_eq!(model_none.validate(), Err(VerifierError::ModelWithoutSat));
        // Core on sat.
        let mut core_sat = q1_sat();
        core_sat.model = None;
        core_sat.unsat_core = Some(vec![id("a.test_source.m1_guideline_a.rule.0")]);
        assert_eq!(core_sat.validate(), Err(VerifierError::CoreWithoutUnsat));
        // Empty model / empty core.
        let mut empty_model = q1_sat();
        empty_model.model = Some(String::new());
        assert_eq!(empty_model.validate(), Err(VerifierError::EmptyModel));
        let mut empty_core = q2_unsat();
        empty_core.unsat_core = Some(vec![]);
        assert_eq!(empty_core.validate(), Err(VerifierError::EmptyCore));
        // Core stored unsorted / with duplicates.
        let mut unsorted = q2_unsat();
        unsorted.unsat_core = Some(vec![
            id("a.test_source.m1_guideline_b.rule.0"),
            id("a.test_source.m1_guideline_a.rule.0"),
        ]);
        assert_eq!(
            unsorted.validate(),
            Err(VerifierError::Unsorted {
                pool: "unsat_core",
                id: id("a.test_source.m1_guideline_a.rule.0"),
            })
        );
        let mut dup = q2_unsat();
        dup.unsat_core = Some(vec![
            id("a.test_source.m1_guideline_a.rule.0"),
            id("a.test_source.m1_guideline_a.rule.0"),
        ]);
        assert_eq!(
            dup.validate(),
            Err(VerifierError::Duplicate {
                pool: "unsat_core",
                id: id("a.test_source.m1_guideline_a.rule.0"),
            })
        );
    }

    #[test]
    fn validation_rejects_category_verdict_breaks() {
        // semantic_contradiction with sat.
        let mut contra_sat = q2_unsat();
        contra_sat.verdict = Some(SolverVerdict::Sat);
        contra_sat.unsat_core = None;
        assert_eq!(
            contra_sat.validate(),
            Err(VerifierError::CategoryVerdict {
                category: VerifierCategory::SemanticContradiction,
                rule: "semantic_contradiction requires verdict unsat",
            })
        );
        // semantic_no_conflict with no verdict.
        let mut nc_none = q1_sat();
        nc_none.verdict = None;
        nc_none.model = None;
        assert_eq!(
            nc_none.validate(),
            Err(VerifierError::CategoryVerdict {
                category: VerifierCategory::SemanticNoConflict,
                rule: "semantic_no_conflict requires verdict sat or unsat",
            })
        );
        // unknown with sat.
        let mut unk_sat = q1_sat();
        unk_sat.category = VerifierCategory::Unknown;
        assert_eq!(
            unk_sat.validate(),
            Err(VerifierError::CategoryVerdict {
                category: VerifierCategory::Unknown,
                rule: "unknown requires verdict unknown or timeout",
            })
        );
        // A failure category with a verdict.
        let mut fail_sat = syntax_failure();
        fail_sat.verdict = Some(SolverVerdict::Sat);
        assert_eq!(
            fail_sat.validate(),
            Err(VerifierError::CategoryVerdict {
                category: VerifierCategory::TargetSyntaxFailure,
                rule: "failure categories carry no verdict",
            })
        );
    }

    // The group-level wrapper: one member array, round-tripping, member
    // validation delegated (first break wins).
    #[test]
    fn verifier_results_wrap_and_validate_members() {
        let results = VerifierResults {
            results: vec![q1_sat(), q2_unsat()],
        };
        results.validate().unwrap();
        assert!(canon(&results).starts_with("{\"results\":[{\"category\":"));
        round_trip(results.clone());
        round_trip(VerifierResults { results: vec![] });

        let mut broken = results;
        broken.results[1].model = Some("(model)".to_owned());
        assert_eq!(broken.validate(), Err(VerifierError::ModelWithoutSat));
    }
}
