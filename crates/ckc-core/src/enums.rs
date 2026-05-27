use serde::{Deserialize, Serialize};

/// SPEC §12.2: certificate depth hierarchy.
/// Ord reflects C0 < C1 < ... < C9; higher implies all lower classes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CertificateClass {
    #[serde(rename = "C0-Parsed")]
    C0Parsed,
    #[serde(rename = "C1-Schema")]
    C1Schema,
    #[serde(rename = "C2-Normal")]
    C2Normal,
    #[serde(rename = "C3-Grounded")]
    C3Grounded,
    #[serde(rename = "C4-Executable")]
    C4Executable,
    #[serde(rename = "C5-Portfolio")]
    C5Portfolio,
    #[serde(rename = "C6-ProofObject")]
    C6ProofObject,
    #[serde(rename = "C7-Kernel")]
    C7Kernel,
    #[serde(rename = "C8-Adjudicated")]
    C8Adjudicated,
    #[serde(rename = "C9-Assured")]
    C9Assured,
}

/// SPEC §6.2: terminology binding match quality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingStatus {
    Exact,
    Broad,
    Narrow,
    Related,
    Unmapped,
    Ambiguous,
    Deprecated,
    Incoherent,
}

/// License/permission status for sources and terminology bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseStatus {
    Open,
    Licensed,
    Restricted,
    FairUse,
    Unknown,
    Prohibited,
}

/// ISO 639-1 language tag for bilingual JA/EN outputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Ja,
    En,
}

/// GRADE recommendation direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationDirection {
    For,
    Against,
}

/// GRADE recommendation strength.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationStrength {
    Strong,
    Conditional,
}

/// GRADE evidence certainty. Ord reflects VeryLow < Low < Moderate < High.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCertainty {
    VeryLow,
    Low,
    Moderate,
    High,
}

/// SPEC §6.1: source document classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Guideline,
    Textbook,
    PackageInsert,
    ReviewReport,
    SafetyCommunication,
    LocalPolicy,
}

/// SPEC §6.2: clinical concept semantic type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticType {
    Disease,
    Drug,
    LabTest,
    Procedure,
    AdverseEvent,
    Finding,
    Anatomy,
    Substance,
    Pathway,
    Qualifier,
}

/// Clinical claim type classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimType {
    Factual,
    Normative,
    Definitional,
    Eligibility,
    Quantity,
    Temporal,
    Workflow,
    DecisionTable,
}

/// Processing status of a CKC artifact through the semantic firewall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    Candidate,
    Accepted,
    Rejected,
    Review,
    Deprecated,
}

/// SPEC §9 CKC-Defeasible: rule classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    Strict,
    Defeasible,
    Defeater,
}

/// Deontic modality projection for clinical norms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeonticProjection {
    Obligation,
    Prohibition,
    Permission,
    Recommendation,
}

/// Exception handling policy for norms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExceptionPolicy {
    Defeasible,
    Absolute,
}

/// Prima facie vs. all-things-considered norm scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormScope {
    PrimaFacie,
    AllThingsConsidered,
}

/// Clinical action type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Administer,
    Prescribe,
    Monitor,
    Assess,
    Discontinue,
    Refer,
    Avoid,
}

/// Evidence type classification per GRADE methodology.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    MetaAnalysis,
    SystematicReview,
    Rct,
    Cohort,
    CaseControl,
    CrossSectional,
    CaseSeries,
    CaseReport,
    ExpertOpinion,
    GuidelineRecommendation,
}

/// GRADE outcome importance levels.
/// Ord reflects NotImportant < Important < Critical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeImportance {
    NotImportant,
    Important,
    Critical,
}

/// SKOS-standard mapping relation for terminology bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingRelation {
    ExactMatch,
    CloseMatch,
    BroadMatch,
    NarrowMatch,
    RelatedMatch,
}

/// SPEC §15.4: conflict classification for review triage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictClassification {
    TrueConflict,
    LikelyAmbiguity,
    ExtractionError,
    FormalizationError,
    TerminologyError,
    InteropCompilerError,
    StaleSource,
    NeedsClinicianAdjudication,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certificate_class_ordering() {
        assert!(CertificateClass::C0Parsed < CertificateClass::C9Assured);
        assert!(CertificateClass::C3Grounded < CertificateClass::C4Executable);
        assert!(CertificateClass::C6ProofObject > CertificateClass::C5Portfolio);
    }

    #[test]
    fn certificate_class_serde() {
        let c = CertificateClass::C6ProofObject;
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"C6-ProofObject\"");
        let back: CertificateClass = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn binding_status_serde() {
        let json = serde_json::to_string(&BindingStatus::Exact).unwrap();
        assert_eq!(json, "\"exact\"");
        let back: BindingStatus = serde_json::from_str("\"incoherent\"").unwrap();
        assert_eq!(back, BindingStatus::Incoherent);
    }

    #[test]
    fn evidence_certainty_ordering() {
        assert!(EvidenceCertainty::VeryLow < EvidenceCertainty::Low);
        assert!(EvidenceCertainty::Low < EvidenceCertainty::Moderate);
        assert!(EvidenceCertainty::Moderate < EvidenceCertainty::High);
    }

    #[test]
    fn conflict_classification_serde() {
        let c = ConflictClassification::NeedsClinicianAdjudication;
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"needs_clinician_adjudication\"");
        let back: ConflictClassification = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn language_serde() {
        assert_eq!(serde_json::to_string(&Language::Ja).unwrap(), "\"ja\"");
        assert_eq!(serde_json::to_string(&Language::En).unwrap(), "\"en\"");
    }

    #[test]
    fn recommendation_serde() {
        let json = serde_json::to_string(&RecommendationDirection::For).unwrap();
        assert_eq!(json, "\"for\"");
        let json = serde_json::to_string(&RecommendationStrength::Conditional).unwrap();
        assert_eq!(json, "\"conditional\"");
    }

    #[test]
    fn source_type_serde() {
        assert_eq!(
            serde_json::to_string(&SourceType::Guideline).unwrap(),
            "\"guideline\""
        );
        assert_eq!(
            serde_json::to_string(&SourceType::PackageInsert).unwrap(),
            "\"package_insert\""
        );
        let back: SourceType = serde_json::from_str("\"local_policy\"").unwrap();
        assert_eq!(back, SourceType::LocalPolicy);
    }

    #[test]
    fn semantic_type_serde() {
        assert_eq!(
            serde_json::to_string(&SemanticType::Disease).unwrap(),
            "\"disease\""
        );
        assert_eq!(
            serde_json::to_string(&SemanticType::LabTest).unwrap(),
            "\"lab_test\""
        );
        let back: SemanticType = serde_json::from_str("\"adverse_event\"").unwrap();
        assert_eq!(back, SemanticType::AdverseEvent);
    }

    #[test]
    fn mapping_relation_serde() {
        assert_eq!(
            serde_json::to_string(&MappingRelation::ExactMatch).unwrap(),
            "\"exact_match\""
        );
        let back: MappingRelation = serde_json::from_str("\"broad_match\"").unwrap();
        assert_eq!(back, MappingRelation::BroadMatch);
    }

    #[test]
    fn claim_type_serde() {
        assert_eq!(
            serde_json::to_string(&ClaimType::Normative).unwrap(),
            "\"normative\""
        );
        assert_eq!(
            serde_json::to_string(&ClaimType::DecisionTable).unwrap(),
            "\"decision_table\""
        );
        let back: ClaimType = serde_json::from_str("\"factual\"").unwrap();
        assert_eq!(back, ClaimType::Factual);
    }

    #[test]
    fn claim_status_serde() {
        assert_eq!(
            serde_json::to_string(&ClaimStatus::Candidate).unwrap(),
            "\"candidate\""
        );
        let back: ClaimStatus = serde_json::from_str("\"accepted\"").unwrap();
        assert_eq!(back, ClaimStatus::Accepted);
    }

    #[test]
    fn rule_kind_serde() {
        assert_eq!(
            serde_json::to_string(&RuleKind::Strict).unwrap(),
            "\"strict\""
        );
        assert_eq!(
            serde_json::to_string(&RuleKind::Defeasible).unwrap(),
            "\"defeasible\""
        );
        assert_eq!(
            serde_json::to_string(&RuleKind::Defeater).unwrap(),
            "\"defeater\""
        );
    }

    #[test]
    fn deontic_projection_serde() {
        assert_eq!(
            serde_json::to_string(&DeonticProjection::Obligation).unwrap(),
            "\"obligation\""
        );
        let back: DeonticProjection = serde_json::from_str("\"recommendation\"").unwrap();
        assert_eq!(back, DeonticProjection::Recommendation);
    }

    #[test]
    fn exception_policy_serde() {
        assert_eq!(
            serde_json::to_string(&ExceptionPolicy::Defeasible).unwrap(),
            "\"defeasible\""
        );
        assert_eq!(
            serde_json::to_string(&ExceptionPolicy::Absolute).unwrap(),
            "\"absolute\""
        );
    }

    #[test]
    fn norm_scope_serde() {
        assert_eq!(
            serde_json::to_string(&NormScope::PrimaFacie).unwrap(),
            "\"prima_facie\""
        );
        assert_eq!(
            serde_json::to_string(&NormScope::AllThingsConsidered).unwrap(),
            "\"all_things_considered\""
        );
    }

    #[test]
    fn action_type_serde() {
        assert_eq!(
            serde_json::to_string(&ActionType::Administer).unwrap(),
            "\"administer\""
        );
        let back: ActionType = serde_json::from_str("\"discontinue\"").unwrap();
        assert_eq!(back, ActionType::Discontinue);
    }

    #[test]
    fn evidence_type_serde() {
        assert_eq!(
            serde_json::to_string(&EvidenceType::Rct).unwrap(),
            "\"rct\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceType::MetaAnalysis).unwrap(),
            "\"meta_analysis\""
        );
        let back: EvidenceType = serde_json::from_str("\"systematic_review\"").unwrap();
        assert_eq!(back, EvidenceType::SystematicReview);
    }

    #[test]
    fn outcome_importance_ordering() {
        assert!(OutcomeImportance::NotImportant < OutcomeImportance::Important);
        assert!(OutcomeImportance::Important < OutcomeImportance::Critical);
    }

    #[test]
    fn outcome_importance_serde() {
        assert_eq!(
            serde_json::to_string(&OutcomeImportance::Critical).unwrap(),
            "\"critical\""
        );
        let back: OutcomeImportance = serde_json::from_str("\"not_important\"").unwrap();
        assert_eq!(back, OutcomeImportance::NotImportant);
    }
}
