//! CKC verification and certificate orchestration (SPEC 13).
//!
//! Consumes the emit-only compiler portfolio from `ckc-compile` plus a recorded
//! solver-outcome oracle ([`verdict`]), then (across task 0.9) builds
//! certificates, execution witnesses, portfolio-agreement records, a certificate
//! graph, and an assurance seed. Determinism comes from the recorded oracle:
//! live solvers run only in PATH-guarded `tests/live_*.rs` that re-derive
//! verdicts and compare against the oracle.

pub mod verdict;

pub use ckc_core::canonical::{ContentHash, content_hash};
pub use ckc_core::enums::{CertificateClass, TargetLanguage};
pub use verdict::{
    RecordedOutcomes, SolverId, VerdictStatus, VerifierOutcome, load_recorded_outcomes,
};
