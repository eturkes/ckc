//! CKC verification and certificate orchestration (SPEC 13).
//!
//! Consumes the emit-only compiler portfolio from `ckc-compile` plus a recorded
//! solver-outcome oracle ([`verdict`]), then (across task 0.9) builds
//! certificates, execution witnesses, portfolio-agreement records, a certificate
//! graph, and an assurance seed. Determinism comes from the recorded oracle:
//! live solvers run only in PATH-guarded `tests/live_*.rs` that re-derive
//! verdicts (via [`runner`]) and compare against the oracle.

pub mod certificate;
pub mod graph;
pub mod portfolio;
pub mod runner;
pub mod verdict;
pub mod witness;

pub use certificate::{certificate_for, certificates};
pub use ckc_core::artifact::ExecutionWitness;
pub use ckc_core::canonical::{ContentHash, content_hash};
pub use ckc_core::enums::{CertificateClass, TargetLanguage};
pub use ckc_core::verify::Certificate;
pub use graph::{CertEdge, CertNode, CertificateGraph, build_graph};
pub use portfolio::{AgreementRecord, portfolio_check};
pub use verdict::{
    RecordedOutcomes, SolverId, VerdictStatus, VerifierOutcome, load_recorded_outcomes,
};
pub use witness::{shacl_certificate, witness_for, witnesses};
