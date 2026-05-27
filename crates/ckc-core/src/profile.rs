use serde::{Deserialize, Serialize};

/// SPEC §9: semantic profiles that determine allowed syntax, validators,
/// compiler targets, and certificate requirements for each CKC object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    fn profile_serde_name() {
        let json = serde_json::to_string(&SemanticProfile::Text).unwrap();
        assert_eq!(json, "\"CKC-Text\"");

        let back: SemanticProfile = serde_json::from_str("\"CKC-Defeasible\"").unwrap();
        assert_eq!(back, SemanticProfile::Defeasible);
    }

    #[test]
    fn all_profiles_roundtrip() {
        let all = [
            SemanticProfile::Text,
            SemanticProfile::Term,
            SemanticProfile::Evidence,
            SemanticProfile::Classical,
            SemanticProfile::Norm,
            SemanticProfile::Defeasible,
            SemanticProfile::Para,
            SemanticProfile::Temporal,
            SemanticProfile::Event,
            SemanticProfile::Quant,
            SemanticProfile::Decision,
            SemanticProfile::Workflow,
            SemanticProfile::Interop,
            SemanticProfile::Prob,
            SemanticProfile::Cert,
            SemanticProfile::Audit,
        ];
        for p in all {
            let json = serde_json::to_string(&p).unwrap();
            let back: SemanticProfile = serde_json::from_str(&json).unwrap();
            assert_eq!(back, p);
        }
    }
}
