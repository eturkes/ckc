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
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_id_roundtrip {
        ($test_name:ident, $ty:ty) => {
            #[test]
            fn $test_name() {
                let id = <$ty>::new("test_value_123");
                let json = serde_json::to_string(&id).unwrap();
                assert_eq!(json, r#""test_value_123""#);
                let rt: $ty = serde_json::from_str(&json).unwrap();
                assert_eq!(id, rt);
                assert_eq!(id.as_str(), "test_value_123");
                assert_eq!(id.to_string(), "test_value_123");
            }
        };
    }

    test_id_roundtrip!(doc_id, DocId);
    test_id_roundtrip!(span_id, SpanId);
    test_id_roundtrip!(extracted_table_id, ExtractedTableId);
    test_id_roundtrip!(concept_id, ConceptId);
    test_id_roundtrip!(claim_id, ClaimId);
    test_id_roundtrip!(rule_id, RuleId);
    test_id_roundtrip!(decision_table_id, DecisionTableId);
    test_id_roundtrip!(decision_row_id, DecisionRowId);
    test_id_roundtrip!(workflow_id, WorkflowId);
    test_id_roundtrip!(case_id, CaseId);
    test_id_roundtrip!(witness_id, WitnessId);
    test_id_roundtrip!(conflict_id, ConflictId);
    test_id_roundtrip!(argument_graph_id, ArgumentGraphId);
    test_id_roundtrip!(certificate_id, CertificateId);
    test_id_roundtrip!(assurance_node_id, AssuranceNodeId);
    test_id_roundtrip!(audit_trace_id, AuditTraceId);
    test_id_roundtrip!(cq_id, CqId);
    test_id_roundtrip!(bundle_id, BundleId);
    test_id_roundtrip!(manifest_id, ManifestId);
    test_id_roundtrip!(egraph_class_id, EGraphClassId);
    test_id_roundtrip!(dmn_export_id, DmnExportId);
}
