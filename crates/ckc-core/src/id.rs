use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! id_newtype {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    )+};
}

id_newtype! {
    DocId,
    SpanId,
    ExtractedTableId,
    ConceptId,
    ClaimId,
    CqId,
    RuleId,
    DecisionTableId,
    DecisionRowId,
    WorkflowId,
    CaseId,
    WitnessId,
    BundleId,
    ConflictId,
    ArgGraphId,
    CertId,
    AssuranceNodeId,
    TraceId,
    EGraphClassId,
    ManifestId,
    DmnExportId,
}

/// Content-addressed artifact identifier: `sha256:<hex>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentHash(pub String);

impl ContentHash {
    pub fn from_bytes(data: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(data);
        Self(format!("sha256:{}", hex::encode(hash)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_display() {
        let id = DocId::new("sepsis-guideline-2024");
        assert_eq!(id.to_string(), "sepsis-guideline-2024");
    }

    #[test]
    fn id_serde_roundtrip() {
        let id = SpanId::new("span-001");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"span-001\"");
        let back: SpanId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn content_hash_deterministic() {
        let h1 = ContentHash::from_bytes(b"test data");
        let h2 = ContentHash::from_bytes(b"test data");
        assert_eq!(h1, h2);
        assert!(h1.as_str().starts_with("sha256:"));
    }

    #[test]
    fn content_hash_known_value() {
        let h = ContentHash::from_bytes(b"");
        assert_eq!(
            h.as_str(),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
