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
}
