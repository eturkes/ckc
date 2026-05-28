//! Integration test: toy source corpus fixtures for SPEC 20 Phase 0 scenarios.
//!
//! Constructs 3 CorpusDocuments, 16 SourceSpans, and 1 ExtractedTable.
//! Validates cross-referential consistency, span chains, table cell coverage,
//! and canonical hash determinism.
//!
//! Scenario coverage (SPEC 20 Phase 0):
//!   1. norm conflict       — span_rec_sepsis_bl + span_contra_bl_allergy
//!   2. terminology variants — span_term_bl_greek, span_term_bl_katakana,
//!      span_allergy_history (contains beta-ラクタム)
//!   3. decision table      — table cell spans (r0-r3, c0-c1) with overlaps
//!   4. Event Calculus      — span_allergy_history + span_contra_bl_allergy
//!   5. repair candidates   — same as 1 + 3 (conflict sources for MaxSMT/ASP)
//!   6. SHACL provenance    — span_provenance_meta (proper provenance example)
//!   7. Lean proof          — same as 1 + 3 (conflict input for formal proof)
//!   8. replay determinism  — all fixtures participate in hash verification

use ckc_core::canonical::{ContentHash, content_hash, to_canonical_bytes};
use ckc_core::enums::Language;
use ckc_core::id::*;
use ckc_core::source::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;

// =========================================================================
// ID constants (reusable by later subtask tests via JSON loading)
// =========================================================================

const DOC_SEPSIS: &str = "doc_sepsis_gl";
const DOC_ALLERGY: &str = "doc_allergy_ref";
const DOC_VITALS: &str = "doc_vitals_protocol";

const TBL_VITALS: &str = "tbl_vitals";

const SPAN_REC_SEPSIS: &str = "span_rec_sepsis_bl";
const SPAN_EVIDENCE: &str = "span_evidence_sepsis";
const SPAN_TERM_GREEK: &str = "span_term_bl_greek";
const SPAN_TERM_KATAKANA: &str = "span_term_bl_katakana";
const SPAN_CONTRA: &str = "span_contra_bl_allergy";
const SPAN_ALLERGY_HIST: &str = "span_allergy_history";
const SPAN_PROVENANCE: &str = "span_provenance_meta";
const SPAN_VITALS_CAPTION: &str = "span_vitals_caption";
const SPAN_CELL_R0C0: &str = "span_cell_r0c0";
const SPAN_CELL_R0C1: &str = "span_cell_r0c1";
const SPAN_CELL_R1C0: &str = "span_cell_r1c0";
const SPAN_CELL_R1C1: &str = "span_cell_r1c1";
const SPAN_CELL_R2C0: &str = "span_cell_r2c0";
const SPAN_CELL_R2C1: &str = "span_cell_r2c1";
const SPAN_CELL_R3C0: &str = "span_cell_r3c0";
const SPAN_CELL_R3C1: &str = "span_cell_r3c1";

// =========================================================================
// Fixture constructors
// =========================================================================

fn toy_documents() -> Vec<CorpusDocument> {
    vec![
        CorpusDocument {
            doc_id: DocId::new(DOC_SEPSIS),
            title_ja: "敗血症診療ガイドライン2024".into(),
            title_en: Some("Sepsis Management Guideline 2024".into()),
            source_type: "guideline".into(),
            publisher: "日本集中治療医学会".into(),
            society: "JSICM".into(),
            edition: "5".into(),
            publication_date: Some("2024-03-01".into()),
            access_date: Some("2026-05-01".into()),
            license_status: "permitted_research".into(),
            content_hash: content_hash(&"source_content:doc_sepsis_gl"),
            extraction_manifest_id: ManifestId::new("manifest_sepsis_gl"),
            supersedes: None,
            superseded_by: None,
        },
        CorpusDocument {
            doc_id: DocId::new(DOC_ALLERGY),
            title_ja: "薬物アレルギー・アナフィラキシー対応マニュアル".into(),
            title_en: Some("Drug Allergy and Anaphylaxis Management Manual".into()),
            source_type: "reference".into(),
            publisher: "日本アレルギー学会".into(),
            society: "JSA".into(),
            edition: "3".into(),
            publication_date: Some("2023-06-15".into()),
            access_date: Some("2026-05-01".into()),
            license_status: "permitted_research".into(),
            content_hash: content_hash(&"source_content:doc_allergy_ref"),
            extraction_manifest_id: ManifestId::new("manifest_allergy_ref"),
            supersedes: None,
            superseded_by: None,
        },
        CorpusDocument {
            doc_id: DocId::new(DOC_VITALS),
            title_ja: "バイタルサイン評価プロトコル".into(),
            title_en: Some("Vital Sign Assessment Protocol".into()),
            source_type: "protocol".into(),
            publisher: "日本集中治療医学会".into(),
            society: "JSICM".into(),
            edition: "1".into(),
            publication_date: Some("2024-01-01".into()),
            access_date: Some("2026-05-01".into()),
            license_status: "permitted_research".into(),
            content_hash: content_hash(&"source_content:doc_vitals_protocol"),
            extraction_manifest_id: ManifestId::new("manifest_vitals_protocol"),
            supersedes: None,
            superseded_by: None,
        },
    ]
}

fn toy_spans() -> Vec<SourceSpan> {
    let pymupdf = |text: &str| ExtractorVote {
        extractor: "pymupdf".into(),
        raw_text: text.into(),
        confidence: 0.99,
    };
    let yomitoku = |text: &str| ExtractorVote {
        extractor: "yomitoku".into(),
        raw_text: text.into(),
        confidence: 0.95,
    };

    vec![
        // == Document A: Sepsis guideline — 5 chained spans ==

        // 1. Recommendation: beta-lactam for sepsis (scenario 1)
        //    raw_text uses full-width （）： which NFKC normalizes to ASCII ():.
        SourceSpan {
            span_id: SpanId::new(SPAN_REC_SEPSIS),
            doc_id: DocId::new(DOC_SEPSIS),
            section_path: vec!["CQ1".into(), "\u{63a8}\u{5968}".into()],
            cq_id: Some(CqId::new("cq_sepsis_abx")),
            page: Some(12),
            bbox: Some(BBox {
                x: 72.0,
                y: 200.0,
                width: 468.0,
                height: 28.0,
            }),
            table_cell: None,
            raw_text: "敗血症患者にはβラクタム系抗菌薬の投与を推奨する\
                       （推奨の強さ：強、エビデンスの確実性：中）"
                .into(),
            nfkc_text: "敗血症患者にはβラクタム系抗菌薬の投与を推奨する\
                        (推奨の強さ:強、エビデンスの確実性:中)"
                .into(),
            search_text: "敗血症患者にはβラクタム系抗菌薬の投与を推奨する\
                          (推奨の強さ:強、エビデンスの確実性:中)"
                .into(),
            display_text: "敗血症患者にはβラクタム系抗菌薬の投与を推奨する\
                           (推奨の強さ:強、エビデンスの確実性:中)"
                .into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: Some(SpanId::new(SPAN_EVIDENCE)),
            extractor_votes: vec![pymupdf(
                "敗血症患者にはβラクタム系抗菌薬の投与を推奨する\
                 （推奨の強さ：強、エビデンスの確実性：中）",
            )],
            confidence: 0.99,
        },
        // 2. Evidence summary for the sepsis recommendation
        //    raw_text uses full-width （） for CI parentheses.
        SourceSpan {
            span_id: SpanId::new(SPAN_EVIDENCE),
            doc_id: DocId::new(DOC_SEPSIS),
            section_path: vec!["CQ1".into(), "エビデンス".into()],
            cq_id: Some(CqId::new("cq_sepsis_abx")),
            page: Some(12),
            bbox: Some(BBox {
                x: 72.0,
                y: 240.0,
                width: 468.0,
                height: 42.0,
            }),
            table_cell: None,
            raw_text: "複数のランダム化比較試験のメタアナリシスにより、\
                       βラクタム系抗菌薬は敗血症患者の28日死亡率を\
                       有意に低下させた（RR 0.75, 95%CI 0.63-0.89）"
                .into(),
            nfkc_text: "複数のランダム化比較試験のメタアナリシスにより、\
                        βラクタム系抗菌薬は敗血症患者の28日死亡率を\
                        有意に低下させた(RR 0.75, 95%CI 0.63-0.89)"
                .into(),
            search_text: "複数のランダム化比較試験のメタアナリシスにより、\
                          βラクタム系抗菌薬は敗血症患者の28日死亡率を\
                          有意に低下させた(RR 0.75, 95%CI 0.63-0.89)"
                .into(),
            display_text: "複数のランダム化比較試験のメタアナリシスにより、\
                           βラクタム系抗菌薬は敗血症患者の28日死亡率を\
                           有意に低下させた(RR 0.75, 95%CI 0.63-0.89)"
                .into(),
            language: Language::Ja,
            previous_span_id: Some(SpanId::new(SPAN_REC_SEPSIS)),
            next_span_id: Some(SpanId::new(SPAN_TERM_GREEK)),
            extractor_votes: vec![pymupdf(
                "複数のランダム化比較試験のメタアナリシスにより、\
                 βラクタム系抗菌薬は敗血症患者の28日死亡率を\
                 有意に低下させた（RR 0.75, 95%CI 0.63-0.89）",
            )],
            confidence: 0.99,
        },
        // 3. Terminology: βラクタム variant (scenario 2)
        SourceSpan {
            span_id: SpanId::new(SPAN_TERM_GREEK),
            doc_id: DocId::new(DOC_SEPSIS),
            section_path: vec!["CQ1".into(), "用語解説".into()],
            cq_id: None,
            page: Some(13),
            bbox: Some(BBox {
                x: 72.0,
                y: 100.0,
                width: 468.0,
                height: 28.0,
            }),
            table_cell: None,
            raw_text: "βラクタム系抗菌薬は広域スペクトラムの抗菌薬であり、\
                       ペニシリン系、セフェム系、カルバペネム系を含む"
                .into(),
            nfkc_text: "βラクタム系抗菌薬は広域スペクトラムの抗菌薬であり、\
                        ペニシリン系、セフェム系、カルバペネム系を含む"
                .into(),
            search_text: "βラクタム系抗菌薬は広域スペクトラムの抗菌薬であり、\
                          ペニシリン系、セフェム系、カルバペネム系を含む"
                .into(),
            display_text: "βラクタム系抗菌薬は広域スペクトラムの抗菌薬であり、\
                           ペニシリン系、セフェム系、カルバペネム系を含む"
                .into(),
            language: Language::Ja,
            previous_span_id: Some(SpanId::new(SPAN_EVIDENCE)),
            next_span_id: Some(SpanId::new(SPAN_TERM_KATAKANA)),
            extractor_votes: vec![pymupdf(
                "βラクタム系抗菌薬は広域スペクトラムの抗菌薬であり、\
                 ペニシリン系、セフェム系、カルバペネム系を含む",
            )],
            confidence: 0.98,
        },
        // 4. Terminology: ベータラクタム variant (scenario 2)
        SourceSpan {
            span_id: SpanId::new(SPAN_TERM_KATAKANA),
            doc_id: DocId::new(DOC_SEPSIS),
            section_path: vec!["CQ1".into(), "用語解説".into()],
            cq_id: None,
            page: Some(13),
            bbox: Some(BBox {
                x: 72.0,
                y: 140.0,
                width: 468.0,
                height: 28.0,
            }),
            table_cell: None,
            raw_text: "ベータラクタム系薬剤はその化学構造にベータラクタム環を有する".into(),
            nfkc_text: "ベータラクタム系薬剤はその化学構造にベータラクタム環を有する".into(),
            search_text: "ベータラクタム系薬剤はその化学構造にベータラクタム環を有する".into(),
            display_text: "ベータラクタム系薬剤はその化学構造にベータラクタム環を有する".into(),
            language: Language::Ja,
            previous_span_id: Some(SpanId::new(SPAN_TERM_GREEK)),
            next_span_id: Some(SpanId::new(SPAN_PROVENANCE)),
            extractor_votes: vec![pymupdf(
                "ベータラクタム系薬剤はその化学構造にベータラクタム環を有する",
            )],
            confidence: 0.98,
        },
        // 5. Provenance metadata (scenario 6 — proper provenance example)
        SourceSpan {
            span_id: SpanId::new(SPAN_PROVENANCE),
            doc_id: DocId::new(DOC_SEPSIS),
            section_path: vec!["出典情報".into()],
            cq_id: None,
            page: Some(1),
            bbox: Some(BBox {
                x: 72.0,
                y: 680.0,
                width: 468.0,
                height: 14.0,
            }),
            table_cell: None,
            raw_text: "本ガイドラインは日本集中治療医学会により\
                       2024年3月に発行された第5版である"
                .into(),
            nfkc_text: "本ガイドラインは日本集中治療医学会により\
                        2024年3月に発行された第5版である"
                .into(),
            search_text: "本ガイドラインは日本集中治療医学会により\
                          2024年3月に発行された第5版である"
                .into(),
            display_text: "本ガイドラインは日本集中治療医学会により\
                           2024年3月に発行された第5版である"
                .into(),
            language: Language::Ja,
            previous_span_id: Some(SpanId::new(SPAN_TERM_KATAKANA)),
            next_span_id: None,
            extractor_votes: vec![pymupdf(
                "本ガイドラインは日本集中治療医学会により\
                 2024年3月に発行された第5版である",
            )],
            confidence: 0.99,
        },
        // == Document B: Allergy reference — 2 chained spans ==

        // 6. Contraindication: beta-lactam in anaphylaxis history (scenario 1)
        SourceSpan {
            span_id: SpanId::new(SPAN_CONTRA),
            doc_id: DocId::new(DOC_ALLERGY),
            section_path: vec!["禁忌事項".into()],
            cq_id: None,
            page: Some(5),
            bbox: Some(BBox {
                x: 72.0,
                y: 150.0,
                width: 468.0,
                height: 28.0,
            }),
            table_cell: None,
            raw_text: "βラクタム系抗菌薬にアナフィラキシーの既往がある患者には、\
                       同系統の抗菌薬の投与は禁忌である"
                .into(),
            nfkc_text: "βラクタム系抗菌薬にアナフィラキシーの既往がある患者には、\
                        同系統の抗菌薬の投与は禁忌である"
                .into(),
            search_text: "βラクタム系抗菌薬にアナフィラキシーの既往がある患者には、\
                          同系統の抗菌薬の投与は禁忌である"
                .into(),
            display_text: "βラクタム系抗菌薬にアナフィラキシーの既往がある患者には、\
                           同系統の抗菌薬の投与は禁忌である"
                .into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: Some(SpanId::new(SPAN_ALLERGY_HIST)),
            extractor_votes: vec![pymupdf(
                "βラクタム系抗菌薬にアナフィラキシーの既往がある患者には、\
                 同系統の抗菌薬の投与は禁忌である",
            )],
            confidence: 0.99,
        },
        // 7. Allergy history with β-ラクタム variant (scenarios 2, 4)
        SourceSpan {
            span_id: SpanId::new(SPAN_ALLERGY_HIST),
            doc_id: DocId::new(DOC_ALLERGY),
            section_path: vec!["アレルギー歴確認".into()],
            cq_id: None,
            page: Some(8),
            bbox: Some(BBox {
                x: 72.0,
                y: 100.0,
                width: 468.0,
                height: 42.0,
            }),
            table_cell: None,
            raw_text: "薬物アレルギー歴の確認において、過去に\
                       β-ラクタム系抗菌薬によるアナフィラキシーが\
                       認められた場合は、当該薬剤群を回避する"
                .into(),
            nfkc_text: "薬物アレルギー歴の確認において、過去に\
                        β-ラクタム系抗菌薬によるアナフィラキシーが\
                        認められた場合は、当該薬剤群を回避する"
                .into(),
            search_text: "薬物アレルギー歴の確認において、過去に\
                          β-ラクタム系抗菌薬によるアナフィラキシーが\
                          認められた場合は、当該薬剤群を回避する"
                .into(),
            display_text: "薬物アレルギー歴の確認において、過去に\
                           β-ラクタム系抗菌薬によるアナフィラキシーが\
                           認められた場合は、当該薬剤群を回避する"
                .into(),
            language: Language::Ja,
            previous_span_id: Some(SpanId::new(SPAN_CONTRA)),
            next_span_id: None,
            extractor_votes: vec![pymupdf(
                "薬物アレルギー歴の確認において、過去に\
                 β-ラクタム系抗菌薬によるアナフィラキシーが\
                 認められた場合は、当該薬剤群を回避する",
            )],
            confidence: 0.98,
        },
        // == Document C: Vitals protocol — 1 caption + 8 table cells ==

        // 8. Table caption (full-width digits in raw: １ → 1 under NFKC)
        SourceSpan {
            span_id: SpanId::new(SPAN_VITALS_CAPTION),
            doc_id: DocId::new(DOC_VITALS),
            section_path: vec!["バイタルサイン".into(), "表1".into()],
            cq_id: None,
            page: Some(3),
            bbox: Some(BBox {
                x: 72.0,
                y: 100.0,
                width: 468.0,
                height: 14.0,
            }),
            table_cell: None,
            raw_text: "表１：バイタルサイン評価基準と対応".into(),
            nfkc_text: "表1:バイタルサイン評価基準と対応".into(),
            search_text: "表1:バイタルサイン評価基準と対応".into(),
            display_text: "表1:バイタルサイン評価基準と対応".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![yomitoku("表１：バイタルサイン評価基準と対応")],
            confidence: 0.97,
        },
        // Row 0: temperature >= 38.0 -> antipyretic
        // Full-width digits ３８.０ in raw, normalized to 38.0.
        SourceSpan {
            span_id: SpanId::new(SPAN_CELL_R0C0),
            doc_id: DocId::new(DOC_VITALS),
            section_path: vec!["バイタルサイン".into(), "表1".into()],
            cq_id: None,
            page: Some(3),
            bbox: Some(BBox {
                x: 100.0,
                y: 130.0,
                width: 200.0,
                height: 14.0,
            }),
            table_cell: Some(TableCellRef {
                table_id: ExtractedTableId::new(TBL_VITALS),
                row: 0,
                col: 0,
            }),
            raw_text: "体温 ≧ ３８.０℃".into(),
            nfkc_text: "体温 ≧ 38.0℃".into(),
            search_text: "体温 ≧ 38.0℃".into(),
            display_text: "体温 ≧ 38.0℃".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![yomitoku("体温 ≧ ３８.０℃")],
            confidence: 0.96,
        },
        SourceSpan {
            span_id: SpanId::new(SPAN_CELL_R0C1),
            doc_id: DocId::new(DOC_VITALS),
            section_path: vec!["バイタルサイン".into(), "表1".into()],
            cq_id: None,
            page: Some(3),
            bbox: Some(BBox {
                x: 310.0,
                y: 130.0,
                width: 200.0,
                height: 14.0,
            }),
            table_cell: Some(TableCellRef {
                table_id: ExtractedTableId::new(TBL_VITALS),
                row: 0,
                col: 1,
            }),
            raw_text: "解熱薬投与を検討".into(),
            nfkc_text: "解熱薬投与を検討".into(),
            search_text: "解熱薬投与を検討".into(),
            display_text: "解熱薬投与を検討".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![yomitoku("解熱薬投与を検討")],
            confidence: 0.96,
        },
        // Row 1: temperature >= 38.5 -> cooling (OVERLAPS with row 0)
        SourceSpan {
            span_id: SpanId::new(SPAN_CELL_R1C0),
            doc_id: DocId::new(DOC_VITALS),
            section_path: vec!["バイタルサイン".into(), "表1".into()],
            cq_id: None,
            page: Some(3),
            bbox: Some(BBox {
                x: 100.0,
                y: 150.0,
                width: 200.0,
                height: 14.0,
            }),
            table_cell: Some(TableCellRef {
                table_id: ExtractedTableId::new(TBL_VITALS),
                row: 1,
                col: 0,
            }),
            raw_text: "体温 ≧ ３８.５℃".into(),
            nfkc_text: "体温 ≧ 38.5℃".into(),
            search_text: "体温 ≧ 38.5℃".into(),
            display_text: "体温 ≧ 38.5℃".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![yomitoku("体温 ≧ ３８.５℃")],
            confidence: 0.96,
        },
        SourceSpan {
            span_id: SpanId::new(SPAN_CELL_R1C1),
            doc_id: DocId::new(DOC_VITALS),
            section_path: vec!["バイタルサイン".into(), "表1".into()],
            cq_id: None,
            page: Some(3),
            bbox: Some(BBox {
                x: 310.0,
                y: 150.0,
                width: 200.0,
                height: 14.0,
            }),
            table_cell: Some(TableCellRef {
                table_id: ExtractedTableId::new(TBL_VITALS),
                row: 1,
                col: 1,
            }),
            raw_text: "冷却処置を開始".into(),
            nfkc_text: "冷却処置を開始".into(),
            search_text: "冷却処置を開始".into(),
            display_text: "冷却処置を開始".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![yomitoku("冷却処置を開始")],
            confidence: 0.96,
        },
        // Row 2: heart rate > 90/min -> enhanced monitoring
        // Full-width ＞ and ９０ in raw, normalized to > and 90.
        SourceSpan {
            span_id: SpanId::new(SPAN_CELL_R2C0),
            doc_id: DocId::new(DOC_VITALS),
            section_path: vec!["バイタルサイン".into(), "表1".into()],
            cq_id: None,
            page: Some(3),
            bbox: Some(BBox {
                x: 100.0,
                y: 170.0,
                width: 200.0,
                height: 14.0,
            }),
            table_cell: Some(TableCellRef {
                table_id: ExtractedTableId::new(TBL_VITALS),
                row: 2,
                col: 0,
            }),
            raw_text: "心拍数 ＞ ９０回/分".into(),
            nfkc_text: "心拍数 > 90回/分".into(),
            search_text: "心拍数 > 90回/分".into(),
            display_text: "心拍数 > 90回/分".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![yomitoku("心拍数 ＞ ９０回/分")],
            confidence: 0.95,
        },
        SourceSpan {
            span_id: SpanId::new(SPAN_CELL_R2C1),
            doc_id: DocId::new(DOC_VITALS),
            section_path: vec!["バイタルサイン".into(), "表1".into()],
            cq_id: None,
            page: Some(3),
            bbox: Some(BBox {
                x: 310.0,
                y: 170.0,
                width: 200.0,
                height: 14.0,
            }),
            table_cell: Some(TableCellRef {
                table_id: ExtractedTableId::new(TBL_VITALS),
                row: 2,
                col: 1,
            }),
            raw_text: "経過観察を強化".into(),
            nfkc_text: "経過観察を強化".into(),
            search_text: "経過観察を強化".into(),
            display_text: "経過観察を強化".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![yomitoku("経過観察を強化")],
            confidence: 0.95,
        },
        // Row 3: systolic BP < 90mmHg -> fluid resuscitation
        // Full-width ＜ and ９０ in raw, normalized to < and 90.
        SourceSpan {
            span_id: SpanId::new(SPAN_CELL_R3C0),
            doc_id: DocId::new(DOC_VITALS),
            section_path: vec!["バイタルサイン".into(), "表1".into()],
            cq_id: None,
            page: Some(3),
            bbox: Some(BBox {
                x: 100.0,
                y: 190.0,
                width: 200.0,
                height: 14.0,
            }),
            table_cell: Some(TableCellRef {
                table_id: ExtractedTableId::new(TBL_VITALS),
                row: 3,
                col: 0,
            }),
            raw_text: "収縮期血圧 ＜ ９０mmHg".into(),
            nfkc_text: "収縮期血圧 < 90mmHg".into(),
            search_text: "収縮期血圧 < 90mmHg".into(),
            display_text: "収縮期血圧 < 90mmHg".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![yomitoku("収縮期血圧 ＜ ９０mmHg")],
            confidence: 0.95,
        },
        SourceSpan {
            span_id: SpanId::new(SPAN_CELL_R3C1),
            doc_id: DocId::new(DOC_VITALS),
            section_path: vec!["バイタルサイン".into(), "表1".into()],
            cq_id: None,
            page: Some(3),
            bbox: Some(BBox {
                x: 310.0,
                y: 190.0,
                width: 200.0,
                height: 14.0,
            }),
            table_cell: Some(TableCellRef {
                table_id: ExtractedTableId::new(TBL_VITALS),
                row: 3,
                col: 1,
            }),
            raw_text: "輸液負荷を開始".into(),
            nfkc_text: "輸液負荷を開始".into(),
            search_text: "輸液負荷を開始".into(),
            display_text: "輸液負荷を開始".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![yomitoku("輸液負荷を開始")],
            confidence: 0.95,
        },
    ]
}

fn toy_tables() -> Vec<ExtractedTable> {
    let cell_ids: Vec<SpanId> = [
        SPAN_CELL_R0C0,
        SPAN_CELL_R0C1,
        SPAN_CELL_R1C0,
        SPAN_CELL_R1C1,
        SPAN_CELL_R2C0,
        SPAN_CELL_R2C1,
        SPAN_CELL_R3C0,
        SPAN_CELL_R3C1,
    ]
    .iter()
    .map(|s| SpanId::new(*s))
    .collect();

    vec![ExtractedTable {
        table_id: ExtractedTableId::new(TBL_VITALS),
        doc_id: DocId::new(DOC_VITALS),
        caption_span_id: Some(SpanId::new(SPAN_VITALS_CAPTION)),
        cell_span_ids: cell_ids.clone(),
        row_headers: vec![
            "体温基準1".into(),
            "体温基準2".into(),
            "心拍数基準".into(),
            "血圧基準".into(),
        ],
        column_headers: vec!["基準".into(), "対応".into()],
        reading_order: cell_ids,
        extraction_votes: vec![ExtractorVote {
            extractor: "yomitoku".into(),
            raw_text: "体温≧38.0℃|解熱薬投与\n\
                       体温≧38.5℃|冷却処置\n\
                       心拍数>90|経過観察\n\
                       血圧<90|輸液負荷"
                .into(),
            confidence: 0.94,
        }],
        normalized_table_hash: content_hash(
            &"normalized_cells:tbl_vitals:体温≧38.0|解熱薬|体温≧38.5|冷却|心拍数>90|観察|血圧<90|輸液",
        ),
    }]
}

// =========================================================================
// Fixture directory helpers
// =========================================================================

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/toy_research_kernel/fixtures")
}

// =========================================================================
// Cross-referential consistency tests
// =========================================================================

#[test]
fn spans_reference_existing_documents() {
    let doc_ids: HashSet<String> = toy_documents().iter().map(|d| d.doc_id.0.clone()).collect();
    for span in &toy_spans() {
        assert!(
            doc_ids.contains(&span.doc_id.0),
            "span {} references unknown doc_id {}",
            span.span_id,
            span.doc_id
        );
    }
}

#[test]
fn span_chains_are_consistent() {
    let spans = toy_spans();
    let by_id: HashMap<&str, &SourceSpan> = spans.iter().map(|s| (s.span_id.as_str(), s)).collect();

    for span in &spans {
        if let Some(ref next_id) = span.next_span_id {
            let next = by_id.get(next_id.as_str()).unwrap_or_else(|| {
                panic!(
                    "span {} has next_span_id {} which does not exist",
                    span.span_id, next_id
                )
            });
            assert_eq!(
                next.previous_span_id.as_ref().map(|s| s.as_str()),
                Some(span.span_id.as_str()),
                "span chain broken: {}.next = {} but {}.prev != {}",
                span.span_id,
                next_id,
                next_id,
                span.span_id
            );
        }
        if let Some(ref prev_id) = span.previous_span_id {
            let prev = by_id.get(prev_id.as_str()).unwrap_or_else(|| {
                panic!(
                    "span {} has previous_span_id {} which does not exist",
                    span.span_id, prev_id
                )
            });
            assert_eq!(
                prev.next_span_id.as_ref().map(|s| s.as_str()),
                Some(span.span_id.as_str()),
                "span chain broken: {}.prev = {} but {}.next != {}",
                span.span_id,
                prev_id,
                prev_id,
                span.span_id
            );
        }
    }
}

#[test]
fn span_chains_stay_within_same_document() {
    let spans = toy_spans();
    let by_id: HashMap<&str, &SourceSpan> = spans.iter().map(|s| (s.span_id.as_str(), s)).collect();

    for span in &spans {
        if let Some(ref next_id) = span.next_span_id {
            let next = &by_id[next_id.as_str()];
            assert_eq!(
                span.doc_id, next.doc_id,
                "span chain crosses documents: {} (doc {}) -> {} (doc {})",
                span.span_id, span.doc_id, next_id, next.doc_id
            );
        }
    }
}

#[test]
fn table_cell_spans_exist_in_span_set() {
    let span_ids: HashSet<String> = toy_spans().iter().map(|s| s.span_id.0.clone()).collect();
    for table in &toy_tables() {
        for cell_id in &table.cell_span_ids {
            assert!(
                span_ids.contains(&cell_id.0),
                "table {} references cell span {} which does not exist",
                table.table_id,
                cell_id
            );
        }
        if let Some(ref cap_id) = table.caption_span_id {
            assert!(
                span_ids.contains(&cap_id.0),
                "table {} references caption span {} which does not exist",
                table.table_id,
                cap_id
            );
        }
    }
}

#[test]
fn table_cell_refs_reference_existing_table() {
    let table_ids: HashSet<String> = toy_tables().iter().map(|t| t.table_id.0.clone()).collect();
    for span in &toy_spans() {
        if let Some(ref cell_ref) = span.table_cell {
            assert!(
                table_ids.contains(&cell_ref.table_id.0),
                "span {} has table_cell referencing unknown table {}",
                span.span_id,
                cell_ref.table_id
            );
        }
    }
}

#[test]
fn table_cell_positions_cover_expected_grid() {
    let spans = toy_spans();
    let cells: Vec<&SourceSpan> = spans
        .iter()
        .filter(|s| {
            s.table_cell
                .as_ref()
                .is_some_and(|c| c.table_id.as_str() == TBL_VITALS)
        })
        .collect();
    let mut positions: HashSet<(u32, u32)> = HashSet::new();
    for span in &cells {
        let cell = span.table_cell.as_ref().unwrap();
        positions.insert((cell.row, cell.col));
    }
    // 4 rows x 2 columns = 8 positions
    assert_eq!(positions.len(), 8, "expected 4x2 grid of table cells");
    for row in 0..4 {
        for col in 0..2 {
            assert!(
                positions.contains(&(row, col)),
                "missing table cell at row={row}, col={col}"
            );
        }
    }
}

#[test]
fn all_span_ids_are_unique() {
    let spans = toy_spans();
    let mut seen = HashSet::new();
    for span in &spans {
        assert!(
            seen.insert(span.span_id.as_str()),
            "duplicate span_id: {}",
            span.span_id
        );
    }
}

#[test]
fn all_doc_ids_are_unique() {
    let docs = toy_documents();
    let mut seen = HashSet::new();
    for doc in &docs {
        assert!(
            seen.insert(doc.doc_id.as_str()),
            "duplicate doc_id: {}",
            doc.doc_id
        );
    }
}

// =========================================================================
// Hash determinism tests
// =========================================================================

#[test]
fn canonical_hashes_deterministic_across_construction() {
    let h1_docs = content_hash(&toy_documents());
    let h2_docs = content_hash(&toy_documents());
    assert_eq!(h1_docs, h2_docs, "document fixture hashes must be stable");

    let h1_spans = content_hash(&toy_spans());
    let h2_spans = content_hash(&toy_spans());
    assert_eq!(h1_spans, h2_spans, "span fixture hashes must be stable");

    let h1_tables = content_hash(&toy_tables());
    let h2_tables = content_hash(&toy_tables());
    assert_eq!(h1_tables, h2_tables, "table fixture hashes must be stable");
}

#[test]
fn individual_spans_have_distinct_hashes() {
    let spans = toy_spans();
    let hashes: Vec<ContentHash> = spans.iter().map(content_hash).collect();
    let unique: HashSet<&str> = hashes.iter().map(|h| h.as_str()).collect();
    assert_eq!(
        unique.len(),
        hashes.len(),
        "each span fixture must produce a unique content hash"
    );
}

#[test]
fn individual_documents_have_distinct_hashes() {
    let docs = toy_documents();
    let hashes: Vec<ContentHash> = docs.iter().map(content_hash).collect();
    let unique: HashSet<&str> = hashes.iter().map(|h| h.as_str()).collect();
    assert_eq!(
        unique.len(),
        hashes.len(),
        "each document fixture must produce a unique content hash"
    );
}

// =========================================================================
// Scenario coverage test
// =========================================================================

#[test]
fn scenario_coverage_all_eight() {
    let spans = toy_spans();
    let by_id: HashMap<&str, &SourceSpan> = spans.iter().map(|s| (s.span_id.as_str(), s)).collect();

    // Scenario 1: norm conflict — recommendation + contraindication spans
    assert!(by_id.contains_key(SPAN_REC_SEPSIS));
    assert!(by_id.contains_key(SPAN_CONTRA));

    // Scenario 2: terminology variants — three different beta-lactam spellings
    let term_greek = &by_id[SPAN_TERM_GREEK];
    assert!(term_greek.raw_text.contains("βラクタム"));
    let term_katakana = &by_id[SPAN_TERM_KATAKANA];
    assert!(term_katakana.raw_text.contains("ベータラクタム"));
    let allergy = &by_id[SPAN_ALLERGY_HIST];
    assert!(allergy.raw_text.contains("β-ラクタム"));

    // Scenario 3: decision table — cell spans with overlapping conditions
    assert!(by_id.contains_key(SPAN_CELL_R0C0));
    assert!(by_id.contains_key(SPAN_CELL_R1C0));
    let r0 = &by_id[SPAN_CELL_R0C0];
    let r1 = &by_id[SPAN_CELL_R1C0];
    assert!(
        r0.nfkc_text.contains("38.0") && r1.nfkc_text.contains("38.5"),
        "rows 0 and 1 must have overlapping temperature conditions"
    );

    // Scenario 4: Event Calculus — allergy history span
    assert!(by_id.contains_key(SPAN_ALLERGY_HIST));

    // Scenario 5: repair — uses same conflict sources as 1 and 3
    // (no additional spans required)

    // Scenario 6: SHACL provenance — provenance metadata span
    assert!(by_id.contains_key(SPAN_PROVENANCE));

    // Scenario 7: Lean proof — uses same conflict input as 1 and 3
    // (no additional spans required)

    // Scenario 8: replay — all fixtures participate in hash determinism
    assert_eq!(
        spans.len(),
        16,
        "expected 16 spans for full scenario coverage"
    );
}

// =========================================================================
// Committed fixture file tests
// =========================================================================

#[test]
fn committed_documents_match() {
    let path = fixtures_dir().join("documents.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture file missing: {}\nRun: cargo test -p ckc-core \
             --test toy_source_corpus regen_fixtures -- --ignored\n\
             Error: {e}",
            path.display()
        )
    });
    let expected = to_canonical_bytes(&toy_documents());
    assert_eq!(
        bytes, expected,
        "committed documents.json differs from constructor output; \
         regenerate with the regen_fixtures test"
    );
}

#[test]
fn committed_spans_match() {
    let path = fixtures_dir().join("spans.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture file missing: {}\nRun: cargo test -p ckc-core \
             --test toy_source_corpus regen_fixtures -- --ignored\n\
             Error: {e}",
            path.display()
        )
    });
    let expected = to_canonical_bytes(&toy_spans());
    assert_eq!(
        bytes, expected,
        "committed spans.json differs from constructor output; \
         regenerate with the regen_fixtures test"
    );
}

#[test]
fn committed_tables_match() {
    let path = fixtures_dir().join("tables.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture file missing: {}\nRun: cargo test -p ckc-core \
             --test toy_source_corpus regen_fixtures -- --ignored\n\
             Error: {e}",
            path.display()
        )
    });
    let expected = to_canonical_bytes(&toy_tables());
    assert_eq!(
        bytes, expected,
        "committed tables.json differs from constructor output; \
         regenerate with the regen_fixtures test"
    );
}

#[test]
fn committed_fixtures_deserialize_correctly() {
    let dir = fixtures_dir();
    let docs: Vec<CorpusDocument> =
        serde_json::from_slice(&std::fs::read(dir.join("documents.json")).unwrap())
            .expect("documents.json must deserialize");
    let spans: Vec<SourceSpan> =
        serde_json::from_slice(&std::fs::read(dir.join("spans.json")).unwrap())
            .expect("spans.json must deserialize");
    let tables: Vec<ExtractedTable> =
        serde_json::from_slice(&std::fs::read(dir.join("tables.json")).unwrap())
            .expect("tables.json must deserialize");

    assert_eq!(docs.len(), 3);
    assert_eq!(spans.len(), 16);
    assert_eq!(tables.len(), 1);
}

// =========================================================================
// Fixture regeneration (run with --ignored)
// =========================================================================

#[test]
#[ignore]
fn regen_fixtures() {
    let dir = fixtures_dir();
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(
        dir.join("documents.json"),
        to_canonical_bytes(&toy_documents()),
    )
    .unwrap();
    std::fs::write(dir.join("spans.json"), to_canonical_bytes(&toy_spans())).unwrap();
    std::fs::write(dir.join("tables.json"), to_canonical_bytes(&toy_tables())).unwrap();

    eprintln!("Regenerated fixtures in {}", dir.display());
}
