//! `ckc` CLI library (SPEC 18). The binary (`src/main.rs`) is a thin clap shell;
//! every command's work lives here so each pipeline stage stays unit-testable
//! without spawning a process. The `pub use` block below is the downstream-crate
//! surface the stages and tests drive against.

pub mod emit;
pub mod manifest;
pub mod pipeline;

pub use ckc_compile::{ARTIFACT_PATHS, CompileBundle, compile_all, portfolio_manifest};
pub use ckc_conflict::{conflict_manifest, detect_all};
pub use ckc_core::canonical::{ContentHash, content_hash, to_canonical_bytes};
pub use ckc_retrieve::{RetrievalQuery, RetrievalResult, SparseIndex};
pub use ckc_term::TerminologyGraph;
pub use ckc_term::rdf::export_skos_turtle;
pub use ckc_term::shacl::{ShaclReport, validate_rules};
pub use ckc_verify::{VerificationReport, verification_manifest, verify_all};
