use serde::{Deserialize, Serialize};

use crate::enums::{BindingStatus, LicenseStatus, MappingRelation, SemanticType};
use crate::id::{ConceptId, EGraphClassId, SpanId};

/// SPEC §6.2, §10: explicit mapping between a CKC concept and a
/// terminology system entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TerminologyBinding {
    pub system: String,
    pub code: Option<String>,
    pub version: Option<String>,
    pub label: String,
    pub status: BindingStatus,
    pub mapping_relation: MappingRelation,
    pub provenance: String,
    pub confidence: f64,
    pub license_status: LicenseStatus,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
}

/// SPEC §10: clinical concept with terminology bindings and e-graph class.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Concept {
    pub concept_id: ConceptId,
    pub label_ja: String,
    pub label_en: Option<String>,
    pub semantic_type: SemanticType,
    pub terminology_bindings: Vec<TerminologyBinding>,
    pub egraph_class_id: Option<EGraphClassId>,
    pub source_span_ids: Vec<SpanId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::content_hash_of;

    #[test]
    fn terminology_binding_serde_roundtrip() {
        let binding = TerminologyBinding {
            system: "MEDIS".into(),
            code: Some("I10-J189".into()),
            version: Some("2024".into()),
            label: "敗血症".into(),
            status: BindingStatus::Exact,
            mapping_relation: MappingRelation::ExactMatch,
            provenance: "manual".into(),
            confidence: 1.0,
            license_status: LicenseStatus::Licensed,
            valid_from: Some("2024-01-01".into()),
            valid_to: None,
        };
        let json = serde_json::to_string(&binding).unwrap();
        let back: TerminologyBinding = serde_json::from_str(&json).unwrap();
        assert_eq!(back, binding);
    }

    #[test]
    fn concept_serde_roundtrip() {
        let concept = Concept {
            concept_id: ConceptId::new("c-sepsis"),
            label_ja: "敗血症".into(),
            label_en: Some("sepsis".into()),
            semantic_type: SemanticType::Disease,
            terminology_bindings: vec![],
            egraph_class_id: Some(EGraphClassId::new("eg-001")),
            source_span_ids: vec![SpanId::new("span-001")],
        };
        let json = serde_json::to_string(&concept).unwrap();
        let back: Concept = serde_json::from_str(&json).unwrap();
        assert_eq!(back, concept);
    }

    #[test]
    fn concept_with_bindings_canonical_hash() {
        let concept = Concept {
            concept_id: ConceptId::new("c-beta-lactam"),
            label_ja: "βラクタム系抗菌薬".into(),
            label_en: Some("beta-lactam antibiotics".into()),
            semantic_type: SemanticType::Drug,
            terminology_bindings: vec![TerminologyBinding {
                system: "HOT".into(),
                code: Some("H001".into()),
                version: None,
                label: "βラクタム系".into(),
                status: BindingStatus::Broad,
                mapping_relation: MappingRelation::BroadMatch,
                provenance: "automated".into(),
                confidence: 0.85,
                license_status: LicenseStatus::Licensed,
                valid_from: None,
                valid_to: None,
            }],
            egraph_class_id: None,
            source_span_ids: vec![SpanId::new("span-010"), SpanId::new("span-011")],
        };
        let h1 = content_hash_of(&concept).unwrap();
        let h2 = content_hash_of(&concept).unwrap();
        assert_eq!(h1, h2);
    }
}
