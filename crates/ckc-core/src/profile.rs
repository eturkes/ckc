use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// CKC semantic profile (SPEC 9). Every CKC object declares which profiles
/// apply, determining allowed syntax, validators, compiler targets, and
/// certificate requirements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Serialize, Deserialize, JsonSchema)]
pub enum SemanticProfile {
    #[serde(rename = "CKC-Text")]
    Text,
    #[serde(rename = "CKC-Term")]
    Term,
    #[serde(rename = "CKC-Evidence")]
    Evidence,
    #[serde(rename = "CKC-Classical")]
    Classical,
    #[serde(rename = "CKC-Norm")]
    Norm,
    #[serde(rename = "CKC-Defeasible")]
    Defeasible,
    #[serde(rename = "CKC-Para")]
    Para,
    #[serde(rename = "CKC-Temporal")]
    Temporal,
    #[serde(rename = "CKC-Event")]
    Event,
    #[serde(rename = "CKC-Quant")]
    Quant,
    #[serde(rename = "CKC-Decision")]
    Decision,
    #[serde(rename = "CKC-Workflow")]
    Workflow,
    #[serde(rename = "CKC-Interop")]
    Interop,
    #[serde(rename = "CKC-Prob")]
    Prob,
    #[serde(rename = "CKC-Cert")]
    Cert,
    #[serde(rename = "CKC-Audit")]
    Audit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_all_profiles() {
        let profiles = [
            (SemanticProfile::Text, r#""CKC-Text""#),
            (SemanticProfile::Term, r#""CKC-Term""#),
            (SemanticProfile::Evidence, r#""CKC-Evidence""#),
            (SemanticProfile::Classical, r#""CKC-Classical""#),
            (SemanticProfile::Norm, r#""CKC-Norm""#),
            (SemanticProfile::Defeasible, r#""CKC-Defeasible""#),
            (SemanticProfile::Para, r#""CKC-Para""#),
            (SemanticProfile::Temporal, r#""CKC-Temporal""#),
            (SemanticProfile::Event, r#""CKC-Event""#),
            (SemanticProfile::Quant, r#""CKC-Quant""#),
            (SemanticProfile::Decision, r#""CKC-Decision""#),
            (SemanticProfile::Workflow, r#""CKC-Workflow""#),
            (SemanticProfile::Interop, r#""CKC-Interop""#),
            (SemanticProfile::Prob, r#""CKC-Prob""#),
            (SemanticProfile::Cert, r#""CKC-Cert""#),
            (SemanticProfile::Audit, r#""CKC-Audit""#),
        ];
        for (variant, expected_json) in profiles {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let rt: SemanticProfile = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, rt, "roundtrip {variant:?}");
        }
    }

    #[test]
    fn profile_array_roundtrip() {
        let profiles = vec![
            SemanticProfile::Norm,
            SemanticProfile::Defeasible,
            SemanticProfile::Temporal,
        ];
        let json = serde_json::to_string(&profiles).unwrap();
        assert_eq!(json, r#"["CKC-Norm","CKC-Defeasible","CKC-Temporal"]"#);
        let rt: Vec<SemanticProfile> = serde_json::from_str(&json).unwrap();
        assert_eq!(profiles, rt);
    }
}
