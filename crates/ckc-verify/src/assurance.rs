//! Assurance-seed builder (SPEC 10 AssuranceNode, 17 assurance and governance).
//!
//! Assembles the Phase-0 GSN/SACM-style assurance seed: a root goal asserting the
//! SPEC §17 top research assurance claim, supported by three strategy nodes —
//! grounding, determinism, and formal checkability — each citing the certificates
//! that evidence its sub-claim. Determinism is structural: [`normalize_all`] sorts
//! every node's `evidence_artifact_ids`, so the seed's content hash is stable
//! across runs and machines, which the downstream verification-manifest golden
//! depends on.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_core::canonical::{ContentHash, content_hash};
use ckc_core::id::AssuranceNodeId;
use ckc_core::nf::normalize_all;
use ckc_core::verify::{AssuranceNode, Certificate};

/// The Phase-0 assurance seed: a GSN-style goal/strategy tree over the
/// verification certificates. Transparent newtype — serializes as the bare array
/// of nodes (the root goal first, then its strategy children in declaration
/// order). Authored deterministically, so it carries no `Normalize` impl; each
/// node is normalized individually in [`assurance_seed`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct AssuranceSeed(pub Vec<AssuranceNode>);

/// The `solver_or_checker` label of the in-process SHACL grounding validator (set
/// by [`shacl_rules_certificate`](crate::certificate)). It marks the one
/// provenance/grounding certificate apart from the external solver/prover backends.
const SHACL_CHECKER: &str = "ckc-shacl";

/// SPEC §17 top research assurance claim — the root goal's claim.
const TOP_CLAIM: &str = "Accepted CKC artifacts are source-grounded, deterministic, \
    formally checkable, replayable, and suitable for identifying source text \
    requiring review.";

/// Grounding strategy claim — supported by the SHACL provenance validator.
const GROUNDING_CLAIM: &str = "Source grounding holds under closed-world SHACL over \
    CKC rules: every accepted rule carries source-span and provenance support.";

/// Determinism strategy claim — supported by every certificate's replay status.
const DETERMINISM_CLAIM: &str = "Determinism and replay hold for every accepted \
    artifact: each certificate records a passing replay status over \
    content-addressed canonical bytes.";

/// Formal-checkability strategy claim — supported by the external backends.
const FORMAL_CLAIM: &str = "Formal checkability holds through independent solver and \
    prover backends: SMT (z3, cvc5), ASP (clingo), Datalog (souffle), model \
    checking (TLA+ SANY, Alloy), and a Lean kernel proof.";

/// True for the in-process SHACL grounding certificate, false for every external
/// solver/prover certificate. The discriminator is the cert's own
/// `solver_or_checker` label, so the split is derived from the certificate slice.
fn is_shacl(cert: &Certificate) -> bool {
    cert.solver_or_checker == SHACL_CHECKER
}

/// Build one normalized strategy/goal [`AssuranceNode`] with `status="supported"`.
/// [`normalize_all`] sorts `evidence_artifact_ids` (construction order is free) and
/// preserves `children` order (a goal lists its strategies in declaration order).
fn node(
    id: &str,
    node_type: &str,
    claim: &str,
    evidence: Vec<ContentHash>,
    children: &[&str],
) -> AssuranceNode {
    let mut node = AssuranceNode {
        node_id: AssuranceNodeId::new(id),
        node_type: node_type.to_string(),
        claim: claim.to_string(),
        evidence_artifact_ids: evidence,
        status: "supported".to_string(),
        children: children.iter().copied().map(AssuranceNodeId::new).collect(),
    };
    normalize_all(&mut node);
    node
}

/// Build the Phase-0 assurance seed from the verification `certs` (SPEC 17). The
/// root goal asserts the top research assurance claim and delegates to three
/// strategy nodes:
///
/// - grounding: evidenced by the in-process SHACL grounding/provenance validator;
/// - determinism: evidenced by every certificate (each records a passing replay
///   status over content-addressed bytes — a cross-cutting property, hence the
///   deliberate overlap with the other two strategies);
/// - formal checkability: evidenced by the external solver and prover backends.
///
/// Each strategy cites the relevant certificate `content_hash`es. Pass the full
/// Phase-0 certificate set (the solver/cvc5 certs plus the SHACL cert) so every
/// strategy's evidence is non-empty.
pub fn assurance_seed(certs: &[Certificate]) -> AssuranceSeed {
    let grounding: Vec<ContentHash> = certs
        .iter()
        .filter(|cert| is_shacl(cert))
        .map(content_hash)
        .collect();
    let formal: Vec<ContentHash> = certs
        .iter()
        .filter(|cert| !is_shacl(cert))
        .map(content_hash)
        .collect();
    let determinism: Vec<ContentHash> = certs.iter().map(content_hash).collect();

    AssuranceSeed(vec![
        node(
            "asn_root",
            "goal",
            TOP_CLAIM,
            Vec::new(),
            &[
                "asn_grounding",
                "asn_determinism",
                "asn_formal_checkability",
            ],
        ),
        node("asn_grounding", "strategy", GROUNDING_CLAIM, grounding, &[]),
        node(
            "asn_determinism",
            "strategy",
            DETERMINISM_CLAIM,
            determinism,
            &[],
        ),
        node(
            "asn_formal_checkability",
            "strategy",
            FORMAL_CLAIM,
            formal,
            &[],
        ),
    ])
}
