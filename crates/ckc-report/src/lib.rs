//! CKC bilingual report assembly (SPEC 21, 23).
//!
//! Composes the read-only bilingual JA/EN report from the already-normalized
//! Phase-0 artifacts: the compiler `CompileBundle`, the `VerificationReport`,
//! and the `ConflictReport`. The report is the single static input the
//! SvelteKit UI (SPEC 19, 21) renders and the manuscript supplement (SPEC 23)
//! draws on. Like `VerificationReport`/`ConflictReport`, every type here
//! composes already-normalized inputs, so the report carries no `Normalize`
//! impl of its own — its determinism follows from deterministic assembly over
//! those inputs, which the committed `report.json` golden (task 0.12.4)
//! depends on.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use ckc_core::enums::{CertificateClass, ConflictClassification, Severity};

/// The Phase-0 bilingual report (SPEC 21, 23) for one compile bundle: the run
/// command, the producer version, a [`ReportSummary`], and one [`ConflictCard`]
/// per detected conflict in the §21 card order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub command: String,
    pub producer_version: String,
    pub summary: ReportSummary,
    pub conflict_cards: Vec<ConflictCard>,
}

/// Run-level tallies (SPEC 23): corpus/extraction/claim/rule/conflict counts,
/// the certificate-depth distribution, and the conflict-taxonomy counts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReportSummary {
    pub n_documents: usize,
    pub n_spans: usize,
    pub n_claims: usize,
    pub n_rules: usize,
    pub n_conflicts: usize,
    pub certificate_depth_distribution: Vec<DepthCount>,
    pub conflict_taxonomy_counts: Vec<TaxonomyCount>,
}

/// One bucket of the certificate-depth distribution (SPEC 12.2, 23): a
/// certificate class and how many accepted certificates reached it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DepthCount {
    pub certificate_class: CertificateClass,
    pub count: usize,
}

/// One bucket of the conflict-taxonomy counts (SPEC 15, 23): a conflict type
/// and how many detected conflicts carry it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TaxonomyCount {
    pub conflict_type: String,
    pub count: usize,
}

/// One conflict card (SPEC 21) rendered JA-source-first: the conflict identity,
/// its source spans, bilingual glosses and explanation, the normalized CKC
/// view, the witness/model, the per-conflict certificate evidence and depth
/// badge, the repair candidates, the bilingual human-review question, and the
/// adjudication status. Field order follows the §21 card order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConflictCard {
    pub conflict_id: String,
    pub conflict_type: String,
    pub severity: Severity,
    pub classification: ConflictClassification,
    pub source_spans: Vec<CardSpan>,
    pub gloss_ja: String,
    pub gloss_en: String,
    pub normalized_view: Value,
    pub explanation_ja: String,
    pub explanation_en: String,
    pub witness: Vec<Value>,
    pub certificate_evidence: Vec<CardCertificate>,
    pub certificate_depth: Option<CertificateClass>,
    pub repair_candidates: Vec<Value>,
    pub human_review_question_ja: String,
    pub human_review_question_en: String,
    pub adjudication_status: String,
}

/// One source span shown on a [`ConflictCard`] (SPEC 21 element 1): the JA exact
/// source text and its table-cell anchor where present, kept source-first.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CardSpan {
    pub span_id: String,
    pub raw_text: String,
    pub display_text: String,
    pub table_cell: Option<Value>,
    pub language: String,
}

/// One certificate backing a [`ConflictCard`] (SPEC 21 element 6): its id, depth
/// class, and the solver/checker that produced it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CardCertificate {
    pub certificate_id: String,
    pub certificate_class: CertificateClass,
    pub solver_or_checker: String,
}
