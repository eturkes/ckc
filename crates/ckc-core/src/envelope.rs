use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::canonical::{content_hash, ContentHash};
use crate::id::CertificateId;
use crate::profile::SemanticProfile;

/// Tag enum discriminating all storable CKC artifact types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
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
    pub fn wrap<T: Serialize>(
        kind: ArtifactKind,
        artifact: &T,
        meta: ArtifactMeta,
    ) -> Self {
        let inner_hash = content_hash(artifact);
        let payload =
            serde_json::to_value(artifact).expect("CKC types must be serializable");
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
    pub fn extract<T: serde::de::DeserializeOwned>(
        &self,
    ) -> Result<T, serde_json::Error> {
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
    use crate::canonical::to_canonical_bytes;
    use crate::clinical::{Action, Norm, Rule};
    use crate::enums::*;
    use crate::id::*;

    fn fixture_meta() -> ArtifactMeta {
        ArtifactMeta {
            schema_version: "0.0.0".into(),
            producer_version: "ckc-core/0.0.0".into(),
            command_manifest: serde_json::json!({"command": "ckc", "args": ["normalize"]}),
            source_input_hashes: vec![ContentHash(
                "sha256:ee00000000000000000000000000000000000000000000000000000000000001"
                    .into(),
            )],
            parent_hashes: vec![],
            stage: "normalize".into(),
            semantic_profiles: vec![
                SemanticProfile::Norm,
                SemanticProfile::Defeasible,
            ],
            content_hash: ContentHash(
                "sha256:ff00000000000000000000000000000000000000000000000000000000000001"
                    .into(),
            ),
            certificate_ids: vec![],
            replay_command: Some("ckc normalize --bundle test".into()),
        }
    }

    fn fixture_rule() -> Rule {
        Rule {
            rule_id: RuleId::new("rule_sepsis_beta_lactam_001"),
            profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
            kind: RuleKind::Defeasible,
            context: "sepsis AND adult_patient".into(),
            antecedent: "(dx sepsis) AND (adult patient)".into(),
            consequent: "(administer beta_lactam)".into(),
            norm: Some(Norm {
                context: "sepsis in adult patients".into(),
                direction: RecommendationDirection::For,
                action: Action {
                    action_type: "administer".into(),
                    target_concept: ConceptId::new("concept_beta_lactam"),
                    parameters: serde_json::json!({"dose_range": "standard", "route": "iv"}),
                    temporal_constraints: serde_json::json!({"onset": "immediate"}),
                    quantity_constraints: serde_json::json!({"min_dose_mg": 1000}),
                },
                recommendation_strength: RecommendationStrength::Strong,
                evidence_certainty: EvidenceCertainty::Moderate,
                original_modality_phrase_ja: "投与を推奨する".into(),
                deontic_projection: DeonticProjection::Recommended,
                exception_policy: "beta-lactam allergy contraindicates".into(),
                prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
            }),
            priority_over: vec![],
            exceptions: vec!["beta_lactam_allergy".into()],
            temporal_scope: Some("acute_phase".into()),
            population_scope: Some("adult".into()),
            source_span_ids: vec![SpanId::new("span_s1")],
            provenance: "guideline_sepsis_2024_cq1".into(),
            certificate_ids: vec![],
        }
    }

    // -- ArtifactKind tests --

    #[test]
    fn artifact_kind_roundtrip() {
        let kinds = [
            (ArtifactKind::CorpusDocument, "\"corpus_document\""),
            (ArtifactKind::SourceSpan, "\"source_span\""),
            (ArtifactKind::ExtractedTable, "\"extracted_table\""),
            (ArtifactKind::Concept, "\"concept\""),
            (ArtifactKind::ClinicalClaim, "\"clinical_claim\""),
            (ArtifactKind::Rule, "\"rule\""),
            (ArtifactKind::DecisionTable, "\"decision_table\""),
            (ArtifactKind::WorkflowFragment, "\"workflow_fragment\""),
            (ArtifactKind::EventNarrative, "\"event_narrative\""),
            (ArtifactKind::PatientCase, "\"patient_case\""),
            (ArtifactKind::ExecutionWitness, "\"execution_witness\""),
            (ArtifactKind::Conflict, "\"conflict\""),
            (ArtifactKind::ArgumentGraph, "\"argument_graph\""),
            (ArtifactKind::Certificate, "\"certificate\""),
            (ArtifactKind::AssuranceNode, "\"assurance_node\""),
            (ArtifactKind::AuditTrace, "\"audit_trace\""),
            (ArtifactKind::StoreManifest, "\"store_manifest\""),
            (ArtifactKind::EgraphArtifact, "\"egraph_artifact\""),
        ];
        for (variant, expected) in kinds {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected, "serialize {variant:?}");
            let rt: ArtifactKind = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, rt, "roundtrip {variant:?}");
        }
    }

    #[test]
    fn artifact_kind_invalid_rejects() {
        let result = serde_json::from_str::<ArtifactKind>("\"bogus_kind\"");
        assert!(result.is_err());
    }

    // -- ArtifactMeta tests --

    #[test]
    fn artifact_meta_roundtrip() {
        let meta = fixture_meta();
        let json = serde_json::to_string(&meta).unwrap();
        let rt: ArtifactMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(meta, rt);
    }

    #[test]
    fn artifact_meta_replay_command_omitted_when_none() {
        let mut meta = fixture_meta();
        meta.replay_command = None;
        let json = serde_json::to_string(&meta).unwrap();
        assert!(!json.contains("replay_command"));
        let rt: ArtifactMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(meta, rt);
    }

    #[test]
    fn artifact_meta_canonical_stability() {
        let meta = fixture_meta();
        assert_eq!(to_canonical_bytes(&meta), to_canonical_bytes(&meta));
        assert_eq!(content_hash(&meta), content_hash(&meta));
    }

    // -- ArtifactEnvelope tests --

    #[test]
    fn envelope_roundtrip() {
        let rule = fixture_rule();
        let envelope = ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            &rule,
            fixture_meta(),
        );
        let json = serde_json::to_string(&envelope).unwrap();
        let rt: ArtifactEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(envelope, rt);
    }

    #[test]
    fn envelope_canonical_stability() {
        let rule = fixture_rule();
        let envelope = ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            &rule,
            fixture_meta(),
        );
        assert_eq!(
            to_canonical_bytes(&envelope),
            to_canonical_bytes(&envelope)
        );
        assert_eq!(content_hash(&envelope), content_hash(&envelope));
    }

    #[test]
    fn wrap_computes_correct_content_hash() {
        let rule = fixture_rule();
        let rule_hash = content_hash(&rule);
        let envelope = ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            &rule,
            fixture_meta(),
        );
        assert_eq!(
            envelope.meta.content_hash, rule_hash,
            "wrap() must set content_hash to the artifact's canonical hash"
        );
    }

    #[test]
    fn wrap_overwrites_placeholder_content_hash() {
        let rule = fixture_rule();
        let meta = fixture_meta();
        let placeholder = meta.content_hash.clone();
        let envelope = ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            &rule,
            meta,
        );
        assert_ne!(
            envelope.meta.content_hash, placeholder,
            "wrap() must overwrite the placeholder content_hash"
        );
    }

    #[test]
    fn envelope_hash_differs_from_content_hash() {
        let rule = fixture_rule();
        let envelope = ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            &rule,
            fixture_meta(),
        );
        assert_ne!(
            envelope.envelope_hash(),
            envelope.meta.content_hash,
            "envelope hash (store key) must differ from inner content hash"
        );
    }

    #[test]
    fn verify_content_hash_valid() {
        let rule = fixture_rule();
        let envelope = ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            &rule,
            fixture_meta(),
        );
        assert!(
            envelope.verify_content_hash(),
            "freshly wrapped envelope must pass content hash verification"
        );
    }

    #[test]
    fn verify_content_hash_detects_tampering() {
        let rule = fixture_rule();
        let mut envelope = ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            &rule,
            fixture_meta(),
        );
        envelope.meta.content_hash = ContentHash(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                .into(),
        );
        assert!(
            !envelope.verify_content_hash(),
            "tampered content_hash must fail verification"
        );
    }

    #[test]
    fn extract_recovers_typed_artifact() {
        let rule = fixture_rule();
        let envelope = ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            &rule,
            fixture_meta(),
        );
        let extracted: Rule = envelope.extract().unwrap();
        assert_eq!(rule, extracted);
    }

    #[test]
    fn extract_wrong_type_fails() {
        let rule = fixture_rule();
        let envelope = ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            &rule,
            fixture_meta(),
        );
        let result = envelope.extract::<crate::source::CorpusDocument>();
        assert!(result.is_err(), "extracting wrong type must fail");
    }

    #[test]
    fn identical_artifacts_produce_identical_envelopes() {
        let rule = fixture_rule();
        let meta = fixture_meta();
        let e1 = ArtifactEnvelope::wrap(ArtifactKind::Rule, &rule, meta.clone());
        let e2 = ArtifactEnvelope::wrap(ArtifactKind::Rule, &rule, meta);
        assert_eq!(e1.envelope_hash(), e2.envelope_hash());
    }

    #[test]
    fn different_meta_produces_different_envelope_hash() {
        let rule = fixture_rule();
        let mut meta2 = fixture_meta();
        meta2.stage = "compile".into();
        let e1 = ArtifactEnvelope::wrap(ArtifactKind::Rule, &rule, fixture_meta());
        let e2 = ArtifactEnvelope::wrap(ArtifactKind::Rule, &rule, meta2);
        assert_eq!(
            e1.meta.content_hash, e2.meta.content_hash,
            "same artifact must produce same content_hash"
        );
        assert_ne!(
            e1.envelope_hash(),
            e2.envelope_hash(),
            "different meta must produce different envelope hash"
        );
    }
}
