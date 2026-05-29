//! Recorded solver-outcome oracle types (SPEC 13).
//!
//! Phase-0 verification is deterministic by construction: accepted certificates
//! and witnesses are built from authored, canonicalized verdicts — a recorded
//! oracle — rather than live solver stdout, whose model order and proof bytes
//! drift run-to-run and would break the task-0.13 replay-hash acceptance. These
//! types are that oracle's vocabulary. They carry no `Normalize` impl: outcomes
//! are authored in a fixed deterministic form, not canonicalized from raw input.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_core::enums::{CertificateClass, TargetLanguage};

/// Identifier of the solver/checker that produced a recorded verdict (SPEC 13).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SolverId {
    Z3,
    Cvc5,
    Clingo,
    Souffle,
    Lean,
    TlaSany,
    Alloy,
    Shacl,
}

/// Normalized verdict token a Phase-0 backend reports (SPEC 11.1 witness forms,
/// SPEC 13 portfolio). One variant per distinct accepted outcome shape the
/// recorded oracle captures across the toy targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerdictStatus {
    Unsat,
    Sat,
    Satisfiable,
    NoCounterexample,
    EmptyRelation,
    SemanticCheckPassed,
    KernelChecked,
    ViolationsFound,
}

/// One recorded verifier outcome over a single compiled target artifact: the
/// verdict the named solver produces, the salient model/core atoms worth
/// surfacing, an optional optimization objective (MaxSMT), whether a proof
/// artifact accompanies it, and the achieved certificate class.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VerifierOutcome {
    pub target_language: TargetLanguage,
    pub artifact_path: String,
    pub solver: SolverId,
    pub status: VerdictStatus,
    pub salient_atoms: Vec<String>,
    pub objective: Option<i64>,
    pub proof_present: bool,
    pub certificate_class: CertificateClass,
}

/// The recorded oracle: every Phase-0 verifier outcome in one array.
/// Transparent newtype — serializes as the bare array of outcomes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct RecordedOutcomes(pub Vec<VerifierOutcome>);
