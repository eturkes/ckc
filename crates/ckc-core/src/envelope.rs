use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::canonical::{ContentHash, content_hash};
use crate::id::CertificateId;
use crate::profile::SemanticProfile;

/// Tag enum discriminating all storable CKC artifact types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    CorpusDocument,
    SourceSpan,
    ExtractedTable,
    Concept,
    ClinicalClaim,
    Rule,
    DecisionTable,
    WorkflowFragment,
    EventNarrative,
    PatientCase,
    ExecutionWitness,
    Conflict,
    ArgumentGraph,
    Certificate,
    AssuranceNode,
    AuditTrace,
    StoreManifest,
    EgraphArtifact,
    ShaclReport,
    RdfExport,
    AlignmentDiagnostic,
    RetrievalResult,
    CompiledTarget,
}

/// Pipeline metadata for a stored artifact (SPEC 5.2).
///
/// `content_hash` holds the SHA-256 of the inner artifact's canonical JSON.
/// The envelope's own hash (computed over kind + meta + payload) serves as
/// the content-addressed store key.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactMeta {
    pub schema_version: String,
    pub producer_version: String,
    pub command_manifest: Value,
    pub source_input_hashes: Vec<ContentHash>,
    pub parent_hashes: Vec<ContentHash>,
    pub stage: String,
    pub semantic_profiles: Vec<SemanticProfile>,
    pub content_hash: ContentHash,
    pub certificate_ids: Vec<CertificateId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_command: Option<String>,
}

/// Content-addressed envelope wrapping any CKC artifact with pipeline metadata.
///
/// Store key: `sha256:<hex>` of the envelope's canonical JSON bytes.
/// Inner artifact hash: stored in `meta.content_hash`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactEnvelope {
    pub kind: ArtifactKind,
    pub meta: ArtifactMeta,
    pub payload: Value,
}

impl ArtifactEnvelope {
    /// Build an envelope from a typed artifact. Computes the artifact's
    /// content hash and stores it in `meta.content_hash`, overwriting any
    /// prior value.
    pub fn wrap<T: Serialize>(kind: ArtifactKind, artifact: &T, meta: ArtifactMeta) -> Self {
        let inner_hash = content_hash(artifact);
        let payload = serde_json::to_value(artifact).expect("CKC types must be serializable");
        Self {
            kind,
            meta: ArtifactMeta {
                content_hash: inner_hash,
                ..meta
            },
            payload,
        }
    }

    /// SHA-256 of the entire envelope's canonical JSON bytes (the store key).
    pub fn envelope_hash(&self) -> ContentHash {
        content_hash(self)
    }

    /// Deserialize the payload into a typed artifact.
    pub fn extract<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }

    /// Verify that `meta.content_hash` matches the payload's actual
    /// canonical JSON hash.
    pub fn verify_content_hash(&self) -> bool {
        content_hash(&self.payload) == self.meta.content_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::CorpusDocument;

    /// Minimal Rule-like JSON payload. Avoids constructing a real Rule here:
    /// envelope semantics are independent of payload shape, only canonical
    /// hashing matters.
    fn small_payload() -> serde_json::Value {
        serde_json::json!({"id": "rule_x", "kind": "strict"})
    }

    fn fixture_meta() -> ArtifactMeta {
        ArtifactMeta {
            schema_version: "0.0.0".into(),
            producer_version: "ckc-core/0.0.0".into(),
            command_manifest: serde_json::json!({"command": "ckc"}),
            source_input_hashes: vec![],
            parent_hashes: vec![],
            stage: "normalize".into(),
            semantic_profiles: vec![],
            // Intentional placeholder — wrap() must overwrite.
            content_hash: ContentHash("sha256:00".repeat(32)),
            certificate_ids: vec![],
            replay_command: None,
        }
    }

    /// Every `ArtifactKind` round-trips through its SPEC-mandated wire
    /// string. Catches accidental rename of any variant; golden tests
    /// only pin one variant at a time.
    #[test]
    fn artifact_kind_wire_format() {
        let kinds = [
            (ArtifactKind::CorpusDocument, "corpus_document"),
            (ArtifactKind::SourceSpan, "source_span"),
            (ArtifactKind::ExtractedTable, "extracted_table"),
            (ArtifactKind::Concept, "concept"),
            (ArtifactKind::ClinicalClaim, "clinical_claim"),
            (ArtifactKind::Rule, "rule"),
            (ArtifactKind::DecisionTable, "decision_table"),
            (ArtifactKind::WorkflowFragment, "workflow_fragment"),
            (ArtifactKind::EventNarrative, "event_narrative"),
            (ArtifactKind::PatientCase, "patient_case"),
            (ArtifactKind::ExecutionWitness, "execution_witness"),
            (ArtifactKind::Conflict, "conflict"),
            (ArtifactKind::ArgumentGraph, "argument_graph"),
            (ArtifactKind::Certificate, "certificate"),
            (ArtifactKind::AssuranceNode, "assurance_node"),
            (ArtifactKind::AuditTrace, "audit_trace"),
            (ArtifactKind::StoreManifest, "store_manifest"),
            (ArtifactKind::EgraphArtifact, "egraph_artifact"),
            (ArtifactKind::ShaclReport, "shacl_report"),
            (ArtifactKind::RdfExport, "rdf_export"),
            (ArtifactKind::AlignmentDiagnostic, "alignment_diagnostic"),
            (ArtifactKind::RetrievalResult, "retrieval_result"),
            (ArtifactKind::CompiledTarget, "compiled_target"),
        ];
        for (variant, wire) in kinds {
            let expected = format!("\"{wire}\"");
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected, "wire format for {variant:?}");
        }
    }

    #[test]
    fn wrap_sets_content_hash_to_payload_hash_overwriting_placeholder() {
        let payload = small_payload();
        let expected = content_hash(&payload);
        let envelope = ArtifactEnvelope::wrap(ArtifactKind::Rule, &payload, fixture_meta());
        assert_eq!(envelope.meta.content_hash, expected);
    }

    #[test]
    fn envelope_hash_differs_from_inner_content_hash() {
        let envelope = ArtifactEnvelope::wrap(ArtifactKind::Rule, &small_payload(), fixture_meta());
        assert_ne!(envelope.envelope_hash(), envelope.meta.content_hash);
    }

    #[test]
    fn verify_content_hash_detects_tampering() {
        let mut envelope =
            ArtifactEnvelope::wrap(ArtifactKind::Rule, &small_payload(), fixture_meta());
        assert!(envelope.verify_content_hash());
        envelope.meta.content_hash = ContentHash("sha256:00".repeat(32));
        assert!(!envelope.verify_content_hash());
    }

    #[test]
    fn extract_wrong_type_fails() {
        let envelope = ArtifactEnvelope::wrap(ArtifactKind::Rule, &small_payload(), fixture_meta());
        assert!(envelope.extract::<CorpusDocument>().is_err());
    }

    #[test]
    fn meta_differs_changes_envelope_hash_but_not_content_hash() {
        let payload = small_payload();
        let e1 = ArtifactEnvelope::wrap(ArtifactKind::Rule, &payload, fixture_meta());
        let mut m2 = fixture_meta();
        m2.stage = "compile".into();
        let e2 = ArtifactEnvelope::wrap(ArtifactKind::Rule, &payload, m2);
        assert_eq!(e1.meta.content_hash, e2.meta.content_hash);
        assert_ne!(e1.envelope_hash(), e2.envelope_hash());
    }
}
