use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::artifact::{
    DecisionRow, DecisionTable, EventNarrative, ExecutionWitness, PatientCase, WorkflowFragment,
};
use crate::clinical::{
    Action, ClinicalClaim, ConfidenceInterval, EtDFrame, EvidenceAtom, Norm, PICOFrame, Rule,
};
use crate::source::{
    BBox, Concept, CorpusDocument, ExtractedTable, ExtractorVote, SourceSpan, TableCellRef,
    TerminologyBinding,
};
use crate::verify::{ArgumentGraph, AssuranceNode, AuditTrace, Certificate, Conflict};

// ---------------------------------------------------------------------------
// NF context: rewrite log and diagnostics
// ---------------------------------------------------------------------------

/// Record of a single field rewrite during normalization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NfRewrite {
    pub pass: u8,
    pub field: String,
    pub before: String,
    pub after: String,
}

/// Structured diagnostic emitted during normalization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NfDiagnostic {
    pub stage: String,
    pub code: String,
    pub message: String,
}

/// Accumulated context for the CKC Normal Form rewrite pipeline.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NfContext {
    pub rewrites: Vec<NfRewrite>,
    pub diagnostics: Vec<NfDiagnostic>,
}

impl NfContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply text normalization to a string field. Records a rewrite when
    /// the normalized result differs from the original.
    pub fn normalize_field(&mut self, pass: u8, field: &str, value: &mut String) {
        let normalized = normalize_text(value);
        if *value != normalized {
            self.rewrites.push(NfRewrite {
                pass,
                field: field.into(),
                before: std::mem::take(value),
                after: normalized.clone(),
            });
            *value = normalized;
        }
    }

    /// Apply text normalization to an optional string field.
    pub fn normalize_opt_field(&mut self, pass: u8, field: &str, value: &mut Option<String>) {
        if let Some(s) = value {
            self.normalize_field(pass, field, s);
        }
    }

    /// Apply text normalization to each element of a string vector.
    pub fn normalize_vec_field(&mut self, pass: u8, field: &str, values: &mut Vec<String>) {
        for (i, v) in values.iter_mut().enumerate() {
            let indexed = format!("{field}[{i}]");
            self.normalize_field(pass, &indexed, v);
        }
    }
}

// ---------------------------------------------------------------------------
// Text normalization (Pass 2)
// ---------------------------------------------------------------------------

/// Normalize a text string: Unicode NFKC, ideographic space (U+3000) to
/// ASCII space, whitespace collapse, trim.
///
/// NFKC handles fullwidth ASCII to halfwidth and halfwidth katakana to
/// fullwidth. The ideographic space replacement and whitespace collapse
/// handle remaining Japanese-specific spacing.
#[must_use]
pub fn normalize_text(s: &str) -> String {
    let nfkc: String = s.nfkc().collect();
    let mut result = String::with_capacity(nfkc.len());
    let mut prev_ws = true;
    for ch in nfkc.chars() {
        let ch = if ch == '\u{3000}' { ' ' } else { ch };
        if ch.is_whitespace() {
            if !prev_ws {
                result.push(' ');
            }
            prev_ws = true;
        } else {
            result.push(ch);
            prev_ws = false;
        }
    }
    if result.ends_with(' ') {
        result.pop();
    }
    result
}

/// Convenience: normalize a value in place and return the accumulated context.
pub fn normalize_all<T: Normalize>(value: &mut T) -> NfContext {
    let mut ctx = NfContext::new();
    value.normalize(&mut ctx);
    ctx
}

// ---------------------------------------------------------------------------
// Normalize trait
// ---------------------------------------------------------------------------

/// Normalize a CKC object in place according to the NF pipeline.
///
/// Each type implements passes relevant to its fields. Types with a
/// `profiles` field use their profiles to determine which passes apply;
/// Pass 1-2 text normalization is universal across all profiles.
pub trait Normalize {
    fn normalize(&mut self, ctx: &mut NfContext);
}

impl<T: Normalize> Normalize for Vec<T> {
    fn normalize(&mut self, ctx: &mut NfContext) {
        for item in self.iter_mut() {
            item.normalize(ctx);
        }
    }
}

impl<T: Normalize> Normalize for Option<T> {
    fn normalize(&mut self, ctx: &mut NfContext) {
        if let Some(inner) = self {
            inner.normalize(ctx);
        }
    }
}

// ---------------------------------------------------------------------------
// Pass 1-2 implementations: types with text fields or delegating children
// ---------------------------------------------------------------------------

impl Normalize for SourceSpan {
    fn normalize(&mut self, ctx: &mut NfContext) {
        // Pass 1: raw_text preserved verbatim
        // Pass 2: normalize derived text fields
        ctx.normalize_field(2, "nfkc_text", &mut self.nfkc_text);
        ctx.normalize_field(2, "search_text", &mut self.search_text);
        ctx.normalize_field(2, "display_text", &mut self.display_text);
        self.extractor_votes.normalize(ctx);
    }
}

impl Normalize for CorpusDocument {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "title_ja", &mut self.title_ja);
        ctx.normalize_opt_field(2, "title_en", &mut self.title_en);
    }
}

impl Normalize for Concept {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "label_ja", &mut self.label_ja);
        ctx.normalize_opt_field(2, "label_en", &mut self.label_en);
        self.terminology_bindings.normalize(ctx);
    }
}

impl Normalize for TerminologyBinding {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "label", &mut self.label);
    }
}

impl Normalize for ExtractedTable {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_vec_field(2, "row_headers", &mut self.row_headers);
        ctx.normalize_vec_field(2, "column_headers", &mut self.column_headers);
        self.extraction_votes.normalize(ctx);
    }
}

impl Normalize for ClinicalClaim {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "gloss_ja", &mut self.gloss_ja);
        ctx.normalize_field(2, "gloss_en", &mut self.gloss_en);
        self.pico.normalize(ctx);
        self.etd.normalize(ctx);
        self.evidence_atoms.normalize(ctx);
    }
}

impl Normalize for Rule {
    fn normalize(&mut self, ctx: &mut NfContext) {
        self.norm.normalize(ctx);
    }
}

impl Normalize for Norm {
    fn normalize(&mut self, ctx: &mut NfContext) {
        // Pass 1: original_modality_phrase_ja preserved verbatim
        self.action.normalize(ctx);
    }
}

impl Normalize for DecisionTable {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_vec_field(2, "input_columns", &mut self.input_columns);
        ctx.normalize_vec_field(2, "output_columns", &mut self.output_columns);
        self.rows.normalize(ctx);
    }
}

impl Normalize for Conflict {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "human_review_question_ja", &mut self.human_review_question_ja);
        ctx.normalize_field(2, "human_review_question_en", &mut self.human_review_question_en);
    }
}

impl Normalize for AssuranceNode {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "claim", &mut self.claim);
    }
}

// ---------------------------------------------------------------------------
// No-op implementations: types without text fields targeted in Pass 1-2.
// Future passes will add behavior for structural, domain, and complex
// normalization.
// ---------------------------------------------------------------------------

macro_rules! normalize_noop {
    ($($ty:ty),+ $(,)?) => {
        $(impl Normalize for $ty {
            fn normalize(&mut self, _ctx: &mut NfContext) {}
        })+
    };
}

normalize_noop!(
    ExtractorVote,
    BBox,
    TableCellRef,
    ConfidenceInterval,
    PICOFrame,
    EtDFrame,
    EvidenceAtom,
    Action,
    DecisionRow,
    WorkflowFragment,
    EventNarrative,
    PatientCase,
    ExecutionWitness,
    ArgumentGraph,
    Certificate,
    AuditTrace,
);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::ContentHash;
    use crate::enums::*;
    use crate::id::*;
    use crate::profile::SemanticProfile;

    // -- normalize_text unit tests --

    #[test]
    fn already_normal_unchanged() {
        assert_eq!(normalize_text("敗血症"), "敗血症");
        assert_eq!(normalize_text("hello world"), "hello world");
    }

    #[test]
    fn fullwidth_ascii_to_halfwidth() {
        assert_eq!(normalize_text("ＡＢＣＤ"), "ABCD");
        assert_eq!(normalize_text("１２３"), "123");
        assert_eq!(normalize_text("（ＩＶ）"), "(IV)");
    }

    #[test]
    fn halfwidth_katakana_to_fullwidth() {
        assert_eq!(normalize_text("ｶﾀｶﾅ"), "カタカナ");
    }

    #[test]
    fn ideographic_space_to_ascii() {
        assert_eq!(normalize_text("敗血症\u{3000}治療"), "敗血症 治療");
    }

    #[test]
    fn whitespace_collapse() {
        assert_eq!(normalize_text("a  b   c"), "a b c");
        assert_eq!(normalize_text("  leading"), "leading");
        assert_eq!(normalize_text("trailing  "), "trailing");
    }

    #[test]
    fn mixed_whitespace_types() {
        assert_eq!(normalize_text("a\t\n\u{3000}b"), "a b");
    }

    #[test]
    fn mixed_japanese_normalization() {
        assert_eq!(
            normalize_text("βラクタム系\u{3000}抗菌薬（ＩＶ投与）"),
            "βラクタム系 抗菌薬(IV投与)"
        );
    }

    #[test]
    fn fullwidth_digits_in_title() {
        assert_eq!(
            normalize_text("日本版敗血症診療ガイドライン\u{3000}２０２４"),
            "日本版敗血症診療ガイドライン 2024"
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(normalize_text(""), "");
    }

    #[test]
    fn only_whitespace() {
        assert_eq!(normalize_text("   \t  \u{3000}  "), "");
    }

    // -- SourceSpan: raw preserved, derived normalized --

    #[test]
    fn source_span_raw_preserved_derived_normalized() {
        let raw = "βラクタム系\u{3000}抗菌薬（ＩＶ）";
        let mut span = SourceSpan {
            span_id: SpanId::new("span_test"),
            doc_id: DocId::new("doc_test"),
            section_path: vec![],
            cq_id: None,
            page: None,
            bbox: None,
            table_cell: None,
            raw_text: raw.into(),
            nfkc_text: raw.into(),
            search_text: raw.into(),
            display_text: raw.into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![],
            confidence: 1.0,
        };

        let mut ctx = NfContext::new();
        span.normalize(&mut ctx);

        assert_eq!(span.raw_text, raw);
        let expected = "βラクタム系 抗菌薬(IV)";
        assert_eq!(span.nfkc_text, expected);
        assert_eq!(span.search_text, expected);
        assert_eq!(span.display_text, expected);
        assert_eq!(ctx.rewrites.len(), 3);
        assert!(ctx.rewrites.iter().all(|r| r.pass == 2));
    }

    #[test]
    fn source_span_no_rewrite_when_already_normal() {
        let text = "敗血症にはβラクタム系抗菌薬を投与する";
        let mut span = SourceSpan {
            span_id: SpanId::new("span_test"),
            doc_id: DocId::new("doc_test"),
            section_path: vec![],
            cq_id: None,
            page: None,
            bbox: None,
            table_cell: None,
            raw_text: text.into(),
            nfkc_text: text.into(),
            search_text: text.into(),
            display_text: text.into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![],
            confidence: 1.0,
        };

        let mut ctx = NfContext::new();
        span.normalize(&mut ctx);

        assert!(ctx.rewrites.is_empty());
    }

    // -- ExtractorVote: raw preserved --

    #[test]
    fn extractor_vote_raw_preserved() {
        let raw = "βラクタム系\u{3000}抗菌薬（ＩＶ）";
        let mut vote = ExtractorVote {
            extractor: "pymupdf".into(),
            raw_text: raw.into(),
            confidence: 0.99,
        };

        let mut ctx = NfContext::new();
        vote.normalize(&mut ctx);

        assert_eq!(vote.raw_text, raw);
        assert!(ctx.rewrites.is_empty());
    }

    // -- CorpusDocument: titles normalized --

    #[test]
    fn corpus_document_titles_normalized() {
        let mut doc = CorpusDocument {
            doc_id: DocId::new("doc_test"),
            title_ja: "日本版敗血症診療ガイドライン\u{3000}２０２４".into(),
            title_en: Some("Japanese  Clinical  Practice  Guidelines".into()),
            source_type: "guideline".into(),
            publisher: "test".into(),
            society: "test".into(),
            edition: "2024".into(),
            publication_date: None,
            access_date: None,
            license_status: "permitted".into(),
            content_hash: ContentHash(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .into(),
            ),
            extraction_manifest_id: ManifestId::new("manifest_test"),
            supersedes: None,
            superseded_by: None,
        };

        let mut ctx = NfContext::new();
        doc.normalize(&mut ctx);

        assert_eq!(doc.title_ja, "日本版敗血症診療ガイドライン 2024");
        assert_eq!(
            doc.title_en,
            Some("Japanese Clinical Practice Guidelines".into())
        );
        assert_eq!(ctx.rewrites.len(), 2);
    }

    #[test]
    fn corpus_document_none_title_en_skipped() {
        let mut doc = CorpusDocument {
            doc_id: DocId::new("doc_test"),
            title_ja: "テスト".into(),
            title_en: None,
            source_type: "guideline".into(),
            publisher: "test".into(),
            society: "test".into(),
            edition: "1".into(),
            publication_date: None,
            access_date: None,
            license_status: "permitted".into(),
            content_hash: ContentHash(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .into(),
            ),
            extraction_manifest_id: ManifestId::new("manifest_test"),
            supersedes: None,
            superseded_by: None,
        };

        let mut ctx = NfContext::new();
        doc.normalize(&mut ctx);

        assert!(ctx.rewrites.is_empty());
        assert!(doc.title_en.is_none());
    }

    // -- Concept: labels and child bindings normalized --

    #[test]
    fn concept_labels_and_bindings_normalized() {
        let mut concept = Concept {
            concept_id: ConceptId::new("concept_test"),
            label_ja: "βラクタム系\u{3000}抗菌薬".into(),
            label_en: Some("Beta-Lactam  Antibiotics".into()),
            semantic_type: "drug_class".into(),
            terminology_bindings: vec![TerminologyBinding {
                system: "MEDIS".into(),
                code: None,
                version: None,
                label: "βラクタム系\u{3000}抗菌薬".into(),
                status: BindingStatus::Exact,
                mapping_relation: "equivalent".into(),
                provenance: "test".into(),
                confidence: 1.0,
                license_status: "permitted".into(),
                valid_from: None,
                valid_to: None,
            }],
            egraph_class_id: None,
            source_span_ids: vec![],
        };

        let mut ctx = NfContext::new();
        concept.normalize(&mut ctx);

        assert_eq!(concept.label_ja, "βラクタム系 抗菌薬");
        assert_eq!(concept.label_en, Some("Beta-Lactam Antibiotics".into()));
        assert_eq!(concept.terminology_bindings[0].label, "βラクタム系 抗菌薬");
        assert_eq!(ctx.rewrites.len(), 3);
    }

    // -- ClinicalClaim: glosses normalized --

    #[test]
    fn clinical_claim_glosses_normalized() {
        let mut claim = ClinicalClaim {
            claim_id: ClaimId::new("claim_test"),
            claim_type: "recommendation".into(),
            profiles: vec![SemanticProfile::Norm],
            source_span_ids: vec![],
            pico: None,
            etd: None,
            evidence_atoms: vec![],
            rule_ids: vec![],
            decision_table_ids: vec![],
            workflow_fragment_ids: vec![],
            gloss_ja: "βラクタム系\u{3000}抗菌薬の投与を強く推奨する".into(),
            gloss_en: "Beta-lactam  antibiotics  are  strongly  recommended".into(),
            status: "candidate".into(),
        };

        let mut ctx = NfContext::new();
        claim.normalize(&mut ctx);

        assert_eq!(claim.gloss_ja, "βラクタム系 抗菌薬の投与を強く推奨する");
        assert_eq!(
            claim.gloss_en,
            "Beta-lactam antibiotics are strongly recommended"
        );
        assert_eq!(ctx.rewrites.len(), 2);
    }

    // -- Norm: original_modality_phrase_ja preserved --

    #[test]
    fn norm_preserves_original_modality() {
        let original = "投与を\u{3000}推奨する";
        let mut norm = Norm {
            context: "test".into(),
            direction: RecommendationDirection::For,
            action: Action {
                action_type: "administer".into(),
                target_concept: ConceptId::new("concept_test"),
                parameters: serde_json::Value::Null,
                temporal_constraints: serde_json::Value::Null,
                quantity_constraints: serde_json::Value::Null,
            },
            recommendation_strength: RecommendationStrength::Strong,
            evidence_certainty: EvidenceCertainty::Moderate,
            original_modality_phrase_ja: original.into(),
            deontic_projection: DeonticProjection::Recommended,
            exception_policy: "none".into(),
            prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
        };

        let mut ctx = NfContext::new();
        norm.normalize(&mut ctx);

        assert_eq!(norm.original_modality_phrase_ja, original);
        assert!(ctx.rewrites.is_empty());
    }

    // -- DecisionTable: column labels normalized --

    #[test]
    fn decision_table_columns_normalized() {
        let mut dt = DecisionTable {
            table_id: DecisionTableId::new("dt_test"),
            hit_policy: HitPolicy::Unique,
            input_columns: vec!["体温\u{3000}（℃）".into(), "心拍数".into()],
            output_columns: vec!["アラート\u{3000}レベル".into()],
            rows: vec![],
            source_table_id: None,
            dmn_export_id: None,
            certificate_ids: vec![],
        };

        let mut ctx = NfContext::new();
        dt.normalize(&mut ctx);

        assert_eq!(dt.input_columns[0], "体温 (\u{00B0}C)");
        assert_eq!(dt.input_columns[1], "心拍数");
        assert_eq!(dt.output_columns[0], "アラート レベル");
        assert_eq!(ctx.rewrites.len(), 2);
    }

    // -- ExtractedTable: headers normalized --

    #[test]
    fn extracted_table_headers_normalized() {
        let mut table = ExtractedTable {
            table_id: ExtractedTableId::new("tbl_test"),
            doc_id: DocId::new("doc_test"),
            caption_span_id: None,
            cell_span_ids: vec![],
            row_headers: vec!["体温\u{3000}".into(), "血圧".into()],
            column_headers: vec!["項目\u{3000}名".into()],
            reading_order: vec![],
            extraction_votes: vec![],
            normalized_table_hash: ContentHash(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .into(),
            ),
        };

        let mut ctx = NfContext::new();
        table.normalize(&mut ctx);

        assert_eq!(table.row_headers[0], "体温");
        assert_eq!(table.row_headers[1], "血圧");
        assert_eq!(table.column_headers[0], "項目 名");
        assert_eq!(ctx.rewrites.len(), 2);
    }

    // -- normalize_all convenience --

    #[test]
    fn normalize_all_returns_context() {
        let mut span = SourceSpan {
            span_id: SpanId::new("span_test"),
            doc_id: DocId::new("doc_test"),
            section_path: vec![],
            cq_id: None,
            page: None,
            bbox: None,
            table_cell: None,
            raw_text: "raw".into(),
            nfkc_text: "  extra  spaces  ".into(),
            search_text: "ok".into(),
            display_text: "ok".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![],
            confidence: 1.0,
        };

        let ctx = normalize_all(&mut span);

        assert_eq!(span.nfkc_text, "extra spaces");
        assert_eq!(ctx.rewrites.len(), 1);
        assert_eq!(ctx.rewrites[0].field, "nfkc_text");
    }

    // -- Rewrite records before and after values --

    #[test]
    fn rewrite_records_before_and_after() {
        let mut ctx = NfContext::new();
        let mut value = "ＡＢＣ".to_string();
        ctx.normalize_field(2, "test_field", &mut value);

        assert_eq!(value, "ABC");
        assert_eq!(ctx.rewrites.len(), 1);
        assert_eq!(ctx.rewrites[0].before, "ＡＢＣ");
        assert_eq!(ctx.rewrites[0].after, "ABC");
        assert_eq!(ctx.rewrites[0].pass, 2);
        assert_eq!(ctx.rewrites[0].field, "test_field");
    }

    // -- Vec<String> field normalization --

    #[test]
    fn vec_field_indexed_rewrites() {
        let mut ctx = NfContext::new();
        let mut values = vec![
            "ＡＢＣ".to_string(),
            "normal".to_string(),
            "ＸＹＺ".to_string(),
        ];
        ctx.normalize_vec_field(2, "cols", &mut values);

        assert_eq!(values, vec!["ABC", "normal", "XYZ"]);
        assert_eq!(ctx.rewrites.len(), 2);
        assert_eq!(ctx.rewrites[0].field, "cols[0]");
        assert_eq!(ctx.rewrites[1].field, "cols[2]");
    }
}
