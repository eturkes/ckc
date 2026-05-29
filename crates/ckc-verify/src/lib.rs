//! CKC verification and certificate orchestration (SPEC 13).
//!
//! Consumes the emit-only compiler portfolio from `ckc-compile` plus a recorded
//! solver-outcome oracle ([`verdict`]), then (across task 0.9) builds
//! certificates, execution witnesses, portfolio-agreement records, a certificate
//! graph, and an assurance seed. Determinism comes from the recorded oracle:
//! live solvers run only in PATH-guarded `tests/live_*.rs` that re-derive
//! verdicts (via [`runner`]) and compare against the oracle.

pub mod assurance;
pub mod certificate;
pub mod graph;
pub mod portfolio;
pub mod runner;
pub mod verdict;
pub mod witness;

pub use assurance::{AssuranceSeed, assurance_seed};
pub use certificate::{certificate_for, certificates};
pub use ckc_core::artifact::ExecutionWitness;
pub use ckc_core::canonical::{ContentHash, content_hash};
pub use ckc_core::compile::CompileDiagnostic;
pub use ckc_core::enums::{CertificateClass, TargetLanguage};
pub use ckc_core::verify::{AssuranceNode, Certificate};
pub use graph::{CertEdge, CertNode, CertificateGraph, build_graph};
pub use portfolio::{AgreementRecord, portfolio_check};
pub use verdict::{
    RecordedOutcomes, SolverId, VerdictStatus, VerifierOutcome, load_recorded_outcomes,
};
pub use witness::{shacl_certificate, witness_for, witnesses};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_compile::CompileBundle;

/// The Phase-0 verification report for one compile bundle (SPEC 12–13, 17): every
/// certificate, its execution witness, the portfolio-agreement records and any
/// backend-disagreement diagnostics, the certificate graph, and the assurance
/// seed. A composition of already-normalized artifacts, so it carries no
/// `Normalize` impl of its own — its determinism follows from each builder's, as
/// the downstream committed-artifact and manifest goldens (tasks 0.9.15–0.9.16)
/// depend on.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VerificationReport {
    pub certificates: Vec<Certificate>,
    pub witnesses: Vec<ExecutionWitness>,
    pub agreements: Vec<AgreementRecord>,
    pub disagreements: Vec<CompileDiagnostic>,
    pub graph: CertificateGraph,
    pub assurance: AssuranceSeed,
}

/// Compose the verification report for `bundle` from the recorded solver-outcome
/// oracle (SPEC 12–13, 17), orchestrating the task-0.9.8–0.9.13 builders:
///
/// - `certificates`: the 10 solver/cvc5 certs ([`certificates`]) plus the
///   in-process SHACL grounding cert ([`shacl_certificate`]) — 11;
/// - `witnesses`: the mirrored 10 ([`witnesses`]) plus the SHACL witness — 11;
/// - `agreements`/`disagreements`: [`portfolio_check`] over the recorded oracle;
/// - `graph`: [`build_graph`] over the full 11-cert / 11-witness set;
/// - `assurance`: [`assurance_seed`] over the full 11-cert set.
///
/// The certificate and witness sets — and hence the graph and assurance seed — are
/// exactly the sets the 0.9.12/0.9.13 goldens lock, so this report reproduces them.
pub fn verify_all(bundle: &CompileBundle) -> VerificationReport {
    // Each `let` shadows the same-named builder fn after capturing it on the RHS;
    // call both builders before either name is rebound.
    let mut certificates = certificates(bundle);
    let mut witnesses = witnesses(bundle);
    let (shacl_cert, shacl_witness) = shacl_certificate(bundle);
    certificates.push(shacl_cert);
    witnesses.push(shacl_witness);

    let (agreements, disagreements) = portfolio_check(&load_recorded_outcomes().0);
    let graph = build_graph(bundle, &certificates, &witnesses);
    let assurance = assurance_seed(&certificates);

    VerificationReport {
        certificates,
        witnesses,
        agreements,
        disagreements,
        graph,
        assurance,
    }
}
