use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Terminology binding status (SPEC 6.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
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

/// Certificate depth class (SPEC 12.2). Higher classes imply lower.
/// Variant order matches depth: C0 < C1 < … < C9.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Serialize, Deserialize, JsonSchema)]
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

/// Conflict classification (SPEC 15.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
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

/// Conflict / diagnostic severity. Variant order matches escalation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Rule kind (SPEC 9 CKC-Defeasible).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    Strict,
    Defeasible,
    Defeater,
}

/// Patient case provenance (SPEC 10 PatientCase).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseType {
    Synthetic,
    Fixture,
    DeidentifiedLater,
    LiveLater,
}

/// DMN hit policy for decision tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HitPolicy {
    Unique,
    Any,
    Priority,
    First,
    OutputOrder,
    RuleOrder,
    Collect,
}

/// Recommendation direction (SPEC 10 EtDFrame, Norm).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationDirection {
    For,
    Against,
}

/// GRADE recommendation strength (SPEC 10 Norm).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationStrength {
    Strong,
    Weak,
    Conditional,
}

/// GRADE evidence certainty (SPEC 10 Norm, EtDFrame, EvidenceAtom).
/// Variant order matches certainty level: VeryLow < Low < Moderate < High.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCertainty {
    VeryLow,
    Low,
    Moderate,
    High,
}

/// Document language tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Ja,
    En,
}

/// Deontic projection of a clinical norm (SPEC 10 Norm).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeonticProjection {
    Obligatory,
    Recommended,
    Permitted,
    Prohibited,
    Optional,
}

/// Certificate replay status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReplayStatus {
    Passed,
    Failed,
    Pending,
    Skipped,
}

/// Whether a norm is prima-facie (defeasible) or all-things-considered
/// (survived defeasible reasoning).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NormCommitment {
    PrimaFacie,
    AllThingsConsidered,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_status_snake_case() {
        assert_eq!(
            serde_json::to_string(&BindingStatus::Exact).unwrap(),
            r#""exact""#
        );
        assert_eq!(
            serde_json::to_string(&BindingStatus::Incoherent).unwrap(),
            r#""incoherent""#
        );
        let rt: BindingStatus = serde_json::from_str(r#""ambiguous""#).unwrap();
        assert_eq!(rt, BindingStatus::Ambiguous);
    }

    #[test]
    fn certificate_class_custom_names() {
        assert_eq!(
            serde_json::to_string(&CertificateClass::C0Parsed).unwrap(),
            r#""C0-Parsed""#
        );
        assert_eq!(
            serde_json::to_string(&CertificateClass::C9Assured).unwrap(),
            r#""C9-Assured""#
        );
        let rt: CertificateClass = serde_json::from_str(r#""C6-ProofObject""#).unwrap();
        assert_eq!(rt, CertificateClass::C6ProofObject);
    }

    #[test]
    fn certificate_class_ordering() {
        assert!(CertificateClass::C0Parsed < CertificateClass::C9Assured);
        assert!(CertificateClass::C4Executable < CertificateClass::C7Kernel);
    }

    #[test]
    fn conflict_classification_snake_case() {
        assert_eq!(
            serde_json::to_string(&ConflictClassification::NeedsClinicianAdjudication).unwrap(),
            r#""needs_clinician_adjudication""#
        );
        assert_eq!(
            serde_json::to_string(&ConflictClassification::InteropCompilerError).unwrap(),
            r#""interop_compiler_error""#
        );
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Critical);
        assert!(Severity::Low < Severity::High);
    }

    #[test]
    fn evidence_certainty_ordering() {
        assert!(EvidenceCertainty::VeryLow < EvidenceCertainty::High);
        assert!(EvidenceCertainty::Low < EvidenceCertainty::Moderate);
    }

    #[test]
    fn case_type_multiword_variant() {
        assert_eq!(
            serde_json::to_string(&CaseType::DeidentifiedLater).unwrap(),
            r#""deidentified_later""#
        );
    }

    #[test]
    fn deontic_roundtrip() {
        let d = DeonticProjection::Prohibited;
        let json = serde_json::to_string(&d).unwrap();
        let rt: DeonticProjection = serde_json::from_str(&json).unwrap();
        assert_eq!(d, rt);
    }

    #[test]
    fn norm_commitment_roundtrip() {
        assert_eq!(
            serde_json::to_string(&NormCommitment::PrimaFacie).unwrap(),
            r#""prima_facie""#
        );
        assert_eq!(
            serde_json::to_string(&NormCommitment::AllThingsConsidered).unwrap(),
            r#""all_things_considered""#
        );
    }
}
