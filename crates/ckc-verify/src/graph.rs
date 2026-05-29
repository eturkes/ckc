//! Certificate-graph assembly (SPEC 10 certificate graph, 17 trace links).
//!
//! Links the Phase-0 verification artifacts into one provenance DAG: every
//! [`Certificate`] node points at the [`CompiledTarget`](ckc_compile::CompiledTarget)
//! it `checks`, each checked target points at every source artifact it is
//! `derived_from`, and every certificate points at the [`ExecutionWitness`] that
//! `witnessed_by` it. A node's identity is the artifact's `content_hash`, so a
//! target checked by several certs (the norm-conflict SMT target, checked by both
//! z3 and cvc5) and a source artifact shared by several targets each collapse to
//! one node — which is why nodes and edges are explicitly deduplicated (the NF
//! sort helpers sort without dedup). The deterministic sort over both vectors
//! keeps the graph's content hash stable across runs, which the certificate-graph
//! golden depends on.

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_compile::{CompileBundle, CompiledTarget, compile_all};
use ckc_core::artifact::ExecutionWitness;
use ckc_core::canonical::{ContentHash, content_hash};
use ckc_core::enums::CertificateClass;
use ckc_core::verify::Certificate;

/// A node in the certificate graph, identified by the `content_hash` of the
/// artifact it stands for. `certificate_class` is `Some` only for certificate
/// nodes; compiled-target, source-artifact, and witness nodes leave it absent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CertNode {
    pub artifact_hash: ContentHash,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_class: Option<CertificateClass>,
}

/// A directed edge: `from`'s artifact references `to`'s under `relation` — one of
/// `checks`, `derived_from`, or `witnessed_by`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CertEdge {
    pub from: ContentHash,
    pub to: ContentHash,
    pub relation: String,
}

/// The Phase-0 certificate graph: a deterministic, deduplicated, acyclic
/// provenance DAG over verification artifacts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CertificateGraph {
    pub nodes: Vec<CertNode>,
    pub edges: Vec<CertEdge>,
}

const KIND_CERTIFICATE: &str = "certificate";
const KIND_COMPILED_TARGET: &str = "compiled_target";
const KIND_SOURCE_ARTIFACT: &str = "source_artifact";
const KIND_EXECUTION_WITNESS: &str = "execution_witness";

const REL_CHECKS: &str = "checks";
const REL_DERIVED_FROM: &str = "derived_from";
const REL_WITNESSED_BY: &str = "witnessed_by";

/// Assemble the certificate graph over `certs` and `witnesses` (SPEC 17 trace
/// links). Each certificate becomes a node and is linked to: the compiled target
/// it `checks` (the one target hash among its `input_artifact_hashes` — none for
/// the in-process SHACL cert, whose inputs are rule artifacts); each source
/// artifact that target is `derived_from`; and the witness that records it as
/// `witnessed_by` (matched by `certificate_id`). Compiled-target and
/// source-artifact nodes/edges are emitted once per checking certificate and then
/// deduplicated, so a multiply-checked target collapses to a single node with
/// several incoming `checks` edges.
pub fn build_graph(
    bundle: &CompileBundle,
    certs: &[Certificate],
    witnesses: &[ExecutionWitness],
) -> CertificateGraph {
    // Resolve a certificate's input hashes to the target it checks: a cert's
    // other input hashes are source artifacts, never compiled targets, so a hit
    // in this map is exactly the checked target.
    let targets = compile_all(bundle);
    let target_by_hash: HashMap<ContentHash, &CompiledTarget> =
        targets.iter().map(|t| (content_hash(t), t)).collect();

    let mut nodes: Vec<CertNode> = Vec::new();
    let mut edges: Vec<CertEdge> = Vec::new();

    for cert in certs {
        let cert_hash = content_hash(cert);
        nodes.push(CertNode {
            artifact_hash: cert_hash.clone(),
            kind: KIND_CERTIFICATE.to_string(),
            certificate_class: Some(cert.certificate_class),
        });

        // checks: cert → the compiled target among its input hashes, plus that
        // target's derived_from chain to each source artifact.
        for input in &cert.input_artifact_hashes {
            let Some(target) = target_by_hash.get(input) else {
                continue;
            };
            edges.push(CertEdge {
                from: cert_hash.clone(),
                to: input.clone(),
                relation: REL_CHECKS.to_string(),
            });
            nodes.push(CertNode {
                artifact_hash: input.clone(),
                kind: KIND_COMPILED_TARGET.to_string(),
                certificate_class: None,
            });
            for source in &target.source_artifact_hashes {
                nodes.push(CertNode {
                    artifact_hash: source.clone(),
                    kind: KIND_SOURCE_ARTIFACT.to_string(),
                    certificate_class: None,
                });
                edges.push(CertEdge {
                    from: input.clone(),
                    to: source.clone(),
                    relation: REL_DERIVED_FROM.to_string(),
                });
            }
        }

        // witnessed_by: cert → each witness that names it.
        for witness in witnesses {
            if witness.certificate_ids.contains(&cert.certificate_id) {
                edges.push(CertEdge {
                    from: cert_hash.clone(),
                    to: content_hash(witness),
                    relation: REL_WITNESSED_BY.to_string(),
                });
            }
        }
    }

    for witness in witnesses {
        nodes.push(CertNode {
            artifact_hash: content_hash(witness),
            kind: KIND_EXECUTION_WITNESS.to_string(),
            certificate_class: None,
        });
    }

    // A node's hash is unique to its kind, so sorting by (hash, kind) places
    // identical nodes adjacently for dedup and yields a total order; likewise
    // (from, to, relation) for the derived_from edges a multiply-checked target
    // emits once per checking cert.
    nodes.sort_by(|a, b| {
        a.artifact_hash
            .cmp(&b.artifact_hash)
            .then(a.kind.cmp(&b.kind))
    });
    nodes.dedup();
    edges.sort_by(|a, b| {
        a.from
            .cmp(&b.from)
            .then(a.to.cmp(&b.to))
            .then(a.relation.cmp(&b.relation))
    });
    edges.dedup();

    CertificateGraph { nodes, edges }
}
