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

/// One row of the [`VerificationManifest`]: a single committed `certs/` artifact,
/// tagged by kind and byte-locked through the `content_hash` of its canonical
/// form.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VerificationEntry {
    pub artifact_kind: String,
    pub artifact_path: String,
    pub content_hash: ContentHash,
}

/// The Phase-0 verification manifest (SPEC 12–13, 25): every [`verify_all`]
/// artifact paired with its committed `certs/` path and content hash, in the
/// deterministic order task 0.9.15 writes the files — the 11 certificates, the 11
/// witnesses, the certificate graph, then the assurance seed. One compact golden
/// over this manifest byte-locks the whole verification artifact set through its
/// hashes, so a drift in any builder surfaces as a manifest mismatch. Transparent
/// newtype — serializes as the bare array of entries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct VerificationManifest(pub Vec<VerificationEntry>);

/// Build the verification manifest for `bundle` from a live [`verify_all`] run,
/// pairing each artifact with the canonical `certs/` path task 0.9.15 commits it
/// to and its [`content_hash`]. The path layout mirrors that task's
/// `report_artifacts` exactly; the cvc5 proof copy is recorded evidence rather
/// than a `verify_all` artifact, so it carries no manifest entry.
pub fn verification_manifest(bundle: &CompileBundle) -> VerificationManifest {
    let report = verify_all(bundle);
    let mut entries = Vec::new();
    for cert in &report.certificates {
        entries.push(VerificationEntry {
            artifact_kind: "certificate".to_string(),
            artifact_path: format!("certs/certificates/{}.json", cert.certificate_id.as_str()),
            content_hash: content_hash(cert),
        });
    }
    for witness in &report.witnesses {
        entries.push(VerificationEntry {
            artifact_kind: "execution_witness".to_string(),
            artifact_path: format!("certs/witnesses/{}.json", witness.witness_id.as_str()),
            content_hash: content_hash(witness),
        });
    }
    entries.push(VerificationEntry {
        artifact_kind: "certificate_graph".to_string(),
        artifact_path: "certs/certificate_graph.json".to_string(),
        content_hash: content_hash(&report.graph),
    });
    entries.push(VerificationEntry {
        artifact_kind: "assurance_seed".to_string(),
        artifact_path: "certs/assurance_seed.json".to_string(),
        content_hash: content_hash(&report.assurance),
    });
    VerificationManifest(entries)
}
