use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::canonical::ContentHash;
use crate::enums::TargetLanguage;
use crate::id::SpanId;

/// One CKC node id mapped to a target-language symbol with source grounding
/// (SPEC 14 CompilationMap entry).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SymbolMapping {
    pub ckc_node_id: String,
    pub target_symbol: String,
    pub source_span_ids: Vec<SpanId>,
}

/// CKC-node-id → target-symbol mapping for one compiled artifact (SPEC 14).
/// Transparent newtype: serializes as the bare array of mappings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct CompilationMap(pub Vec<SymbolMapping>);

/// Structured compiler diagnostic with bilingual messages and source spans
/// (SPEC 12.1, 14 deterministic diagnostics).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CompileDiagnostic {
    pub code: String,
    pub message_ja: String,
    pub message_en: String,
    pub source_span_ids: Vec<SpanId>,
}

/// A compiled target artifact: target text plus the CKC→target symbol map,
/// diagnostics, source provenance, and a replay command (SPEC 14).
///
/// Produced from an already-normalized bundle and not itself re-normalized,
/// so this type carries no `Normalize` impl. `certificate_class` is
/// intentionally absent: certificates are assigned in the verification stage
/// via `Certificate.certificate_class` referencing this artifact's content
/// hash.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CompiledTarget {
    pub target_language: TargetLanguage,
    pub artifact_text: String,
    pub compilation_map: CompilationMap,
    pub diagnostics: Vec<CompileDiagnostic>,
    pub source_artifact_hashes: Vec<ContentHash>,
    pub replay_command: String,
    /// `Some` when a target-language parser/tool was on PATH and run as an
    /// emit-time sanity check; `None` when no such tool was available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_parse_ok: Option<bool>,
}
