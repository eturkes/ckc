//! Certificate builder (SPEC 10 Certificate, 12.2 certificate classes).
//!
//! Turns each compiled target plus its recorded oracle verdict ([`crate::verdict`])
//! into a [`Certificate`]: the deepest achieved certificate class, the input
//! artifact hashes it covers, the solver invocation that produced it, and its
//! replay status. Determinism is structural — the command manifest records
//! repo-relative artifact paths and bare jar names (never an absolute `$HOME` jar
//! path), and [`normalize_all`] sorts the hash vectors — so a certificate's content
//! hash is stable across runs and machines, which the downstream certificate-graph
//! and verification-manifest goldens depend on.

use serde::Serialize;
use serde_json::{Value, json};

use ckc_compile::{CompileBundle, CompiledTarget, compile_all};
use ckc_core::canonical::content_hash;
use ckc_core::enums::{CertificateClass, ReplayStatus};
use ckc_core::id::CertificateId;
use ckc_core::nf::normalize_all;
use ckc_core::verify::Certificate;
use ckc_term::shacl::ShaclReport;

use crate::verdict::{
    RecordedOutcomes, SolverId, VerdictStatus, VerifierOutcome, load_recorded_outcomes,
};

/// The committed cvc5 proof of the norm-conflict unsat — recorded evidence (the
/// live cvc5 re-run in task 0.9.4 is structural, not byte-equal). Its content hash
/// is the cvc5 certificate's single proof-artifact hash.
fn cvc5_proof() -> &'static str {
    include_str!("../../../examples/research_kernel/fixtures/cvc5_norm_conflict.proof")
}

/// Snake_case wire token of a Copy enum, read from its `Serialize` impl so the
/// string never drifts from the recorded oracle's wire form. Drives both the
/// `cert_<solver>_…` id prefix / `solver_or_checker` field (over [`SolverId`]) and
/// the `result` verdict token (over `VerdictStatus`).
fn wire_token<T: Serialize>(value: T) -> String {
    match serde_json::to_value(value) {
        Ok(Value::String(s)) => s,
        _ => unreachable!("oracle enums serialize to a JSON string"),
    }
}

/// Snake_case file stem of a repo-relative artifact path — the `<artifact-stem>`
/// segment of a certificate id. Takes the basename, strips the final extension,
/// and lowercases PascalCase into snake_case so the SMT, cvc5, and Lean
/// norm-conflict artifacts (`norm_conflict.smt2`, `NormConflict.lean`) all yield
/// the stem `norm_conflict`, leaving only the solver prefix to tell their certs
/// apart.
fn snake_stem(artifact_path: &str) -> String {
    let base = artifact_path.rsplit('/').next().unwrap_or(artifact_path);
    let stem = base.rsplit_once('.').map_or(base, |(stem, _)| stem);
    let mut out = String::with_capacity(stem.len());
    for (i, c) in stem.chars().enumerate() {
        if c.is_ascii_uppercase() && i != 0 && !out.ends_with('_') {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// Machine-independent descriptor of the solver invocation behind `outcome`: the
/// runner's command plus its args, using the repo-relative artifact path and bare
/// jar names so the enclosing certificate's content hash stays stable across
/// machines. Mirrors the live [`crate::runner`] invocations.
fn command_manifest(outcome: &VerifierOutcome) -> Value {
    let path = outcome.artifact_path.as_str();
    let (command, args): (&str, Vec<&str>) = match outcome.solver {
        SolverId::Z3 => ("z3", vec![path]),
        SolverId::Cvc5 => ("cvc5", vec!["--produce-proofs", "--dump-proofs", path]),
        SolverId::Clingo => ("clingo", vec![path]),
        SolverId::Souffle => ("souffle", vec![path]),
        SolverId::Lean => ("lean", vec![path]),
        SolverId::TlaSany => ("java", vec!["-cp", "tla2tools.jar", "tla2sany.SANY", path]),
        SolverId::Alloy => ("java", vec!["-jar", "alloy.jar", "exec", path]),
        // SHACL flows through `shacl_certificate` (task 0.9.10) with its own
        // `ckc-shacl` checker label; this arm only keeps the match exhaustive.
        SolverId::Shacl => ("ckc-shacl", vec![path]),
    };
    json!({ "command": command, "args": args })
}

/// Build the [`Certificate`] for one compiled `target` under its recorded
/// `outcome`. The id is `cert_<solver>_<artifact-stem>`; `input_artifact_hashes` is
/// the target's own content hash followed by its source-artifact provenance (the
/// Normalize impl sorts it, so construction order is free); the `result` is the
/// verdict token, with the MaxSMT objective appended for the repair target
/// (mirroring the `violations_found:2` shape of the SHACL cert); and the proof
/// hash is present only when the oracle flags a proof object (the cvc5 cert).
pub fn certificate_for(target: &CompiledTarget, outcome: &VerifierOutcome) -> Certificate {
    let solver = wire_token(outcome.solver);
    let certificate_id = CertificateId::new(format!(
        "cert_{solver}_{}",
        snake_stem(&outcome.artifact_path)
    ));

    let mut input_artifact_hashes = vec![content_hash(target)];
    input_artifact_hashes.extend(target.source_artifact_hashes.iter().cloned());

    let result = match outcome.objective {
        Some(objective) => format!("{}:{objective}", wire_token(outcome.status)),
        None => wire_token(outcome.status),
    };

    let proof_artifact_hashes = if outcome.proof_present {
        let proof = cvc5_proof();
        vec![content_hash(&proof)]
    } else {
        Vec::new()
    };

    let mut certificate = Certificate {
        certificate_id,
        certificate_class: outcome.certificate_class,
        input_artifact_hashes,
        compiler_hash: None,
        solver_or_checker: solver,
        command_manifest: command_manifest(outcome),
        result,
        proof_artifact_hashes,
        replay_status: ReplayStatus::Passed,
        diagnostics: Vec::new(),
    };
    normalize_all(&mut certificate);
    certificate
}

/// Build the Phase-0 certificate set: one cert per `compile_all` target paired with
/// its recorded outcome (9, in `ARTIFACT_PATHS` order), plus the standalone cvc5
/// proof-object cert over the same norm-conflict SMT target — 10 in all. The three
/// norm-conflict certs (z3, cvc5, lean) share the `norm_conflict` stem and stay
/// distinct only through their solver prefix.
pub fn certificates(bundle: &CompileBundle) -> Vec<Certificate> {
    let targets = compile_all(bundle);
    let RecordedOutcomes(outcomes) = load_recorded_outcomes();

    // The first 9 outcomes align with `compile_all`/`ARTIFACT_PATHS`
    // element-for-element; zip stops at the 9 targets, leaving the cvc5 and SHACL
    // entries for the standalone certs below.
    let mut certs: Vec<Certificate> = targets
        .iter()
        .zip(outcomes.iter())
        .map(|(target, outcome)| certificate_for(target, outcome))
        .collect();

    // Standalone cvc5 cert: a second, proof-producing checker over the
    // norm-conflict target (`targets[0]`, `ARTIFACT_PATHS[0]`), carrying C6.
    let cvc5 = outcomes
        .iter()
        .find(|o| o.solver == SolverId::Cvc5)
        .expect("recorded oracle includes the cvc5 outcome");
    certs.push(certificate_for(&targets[0], cvc5));

    certs
}

/// Build the SHACL rule-shape certificate (task 0.9.10, scenario 6). Wraps the
/// in-process [`ckc_term::shacl::validate_rules`] `report` in a `C6-ProofObject`
/// certificate whose single proof artifact is the report itself. This checker runs
/// in-process and deterministically, so the report is built live (by the caller in
/// [`crate::witness::shacl_certificate`]) rather than read from the recorded oracle;
/// `input_artifact_hashes` covers every validated rule and `result` records the
/// `violations_found` token with the violation count (the `violations_found:2` shape
/// the eight solver certs' `result` tokens mirror via their appended objectives).
pub(crate) fn shacl_rules_certificate(report: &ShaclReport, bundle: &CompileBundle) -> Certificate {
    let input_artifact_hashes = bundle.rules.iter().map(content_hash).collect();
    let result = format!(
        "{}:{}",
        wire_token(VerdictStatus::ViolationsFound),
        report.violations.len()
    );

    let mut certificate = Certificate {
        certificate_id: CertificateId::new("cert_shacl_rules"),
        certificate_class: CertificateClass::C6ProofObject,
        input_artifact_hashes,
        compiler_hash: None,
        solver_or_checker: "ckc-shacl".to_string(),
        command_manifest: json!({ "command": "ckc-shacl", "args": ["validate_rules"] }),
        result,
        proof_artifact_hashes: vec![content_hash(report)],
        replay_status: ReplayStatus::Passed,
        diagnostics: Vec::new(),
    };
    normalize_all(&mut certificate);
    certificate
}
