use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! id_newtype {
    ($($(#[doc = $doc:expr])* $name:ident),+ $(,)?) => {$(
        $(#[doc = $doc])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[derive(Serialize, Deserialize, JsonSchema)]
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

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    )+};
}

id_newtype! {
    /// Primary key for `CorpusDocument`.
    DocId,
    /// Primary key for `SourceSpan`.
    SpanId,
    /// Primary key for `ExtractedTable`.
    ExtractedTableId,
    /// Primary key for `Concept`.
    ConceptId,
    /// Primary key for `ClinicalClaim`.
    ClaimId,
    /// Primary key for `Rule`.
    RuleId,
    /// Primary key for `DecisionTable`.
    DecisionTableId,
    /// Primary key for `DecisionRow`.
    DecisionRowId,
    /// Primary key for `WorkflowFragment`.
    WorkflowId,
    /// Primary key for `PatientCase`.
    CaseId,
    /// Primary key for `ExecutionWitness`.
    WitnessId,
    /// Primary key for `Conflict`.
    ConflictId,
    /// Primary key for `ArgumentGraph`.
    ArgumentGraphId,
    /// Primary key for `Certificate`.
    CertificateId,
    /// Primary key for `AssuranceNode`.
    AssuranceNodeId,
    /// Primary key for `AuditTrace`.
    AuditTraceId,
    /// Clinical question identifier.
    CqId,
    /// Artifact bundle identifier.
    BundleId,
    /// Extraction or replay manifest identifier.
    ManifestId,
    /// E-graph equivalence class identifier.
    EGraphClassId,
    /// DMN export artifact identifier.
    DmnExportId,
    /// Primary key for `RetrievalQuery`.
    QueryId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newtype_roundtrip_via_macro() {
        let id = DocId::new("test_value_123");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""test_value_123""#);
        let rt: DocId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, rt);
        assert_eq!(id.as_str(), "test_value_123");
        assert_eq!(id.to_string(), "test_value_123");
    }
}
