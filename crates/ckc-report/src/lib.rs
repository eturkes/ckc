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

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use ckc_compile::CompileBundle;
use ckc_conflict::{Conflict, ConflictReport};
use ckc_core::canonical::content_hash;
use ckc_core::clinical::ClinicalClaim;
use ckc_core::source::{CorpusDocument, SourceSpan};
use ckc_verify::VerificationReport;

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

const CLAIMS_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/claims.json");
const DOCUMENTS_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/documents.json");

/// Load the Phase-0 toy [`ClinicalClaim`] set from the committed fixture
/// embedded at compile time, mirroring [`ckc_compile::CompileBundle::load_toy`].
/// Panics only when the committed fixture stops matching its `ckc-core` type,
/// which is a build-time bug rather than a runtime condition.
pub fn load_claims() -> Vec<ClinicalClaim> {
    serde_json::from_str(CLAIMS_JSON).expect("toy claims.json must deserialize")
}

/// Load the Phase-0 toy [`CorpusDocument`] set from the committed fixture
/// embedded at compile time, mirroring [`load_claims`].
pub fn load_documents() -> Vec<CorpusDocument> {
    serde_json::from_str(DOCUMENTS_JSON).expect("toy documents.json must deserialize")
}

/// Assemble the [`ReportSummary`] (SPEC 23) for one compile bundle: the
/// corpus/extraction/claim/rule/conflict counts, the certificate-depth
/// distribution tallied over `verification.certificates`, and the
/// conflict-taxonomy counts tallied over `conflicts.conflicts`. Both
/// distributions are emitted in a deterministic order — depth ascending by
/// [`CertificateClass`] (`C0 < … < C9`), taxonomy ascending by `conflict_type` —
/// via `BTreeMap` accumulation, so the summary inherits the determinism of its
/// already-normalized inputs.
pub fn build_summary(
    bundle: &CompileBundle,
    claims: &[ClinicalClaim],
    documents: &[CorpusDocument],
    verification: &VerificationReport,
    conflicts: &ConflictReport,
) -> ReportSummary {
    let mut depth: BTreeMap<CertificateClass, usize> = BTreeMap::new();
    for cert in &verification.certificates {
        *depth.entry(cert.certificate_class).or_insert(0) += 1;
    }
    let certificate_depth_distribution = depth
        .into_iter()
        .map(|(certificate_class, count)| DepthCount {
            certificate_class,
            count,
        })
        .collect();

    let mut taxonomy: BTreeMap<String, usize> = BTreeMap::new();
    for conflict in &conflicts.conflicts {
        *taxonomy.entry(conflict.conflict_type.clone()).or_insert(0) += 1;
    }
    let conflict_taxonomy_counts = taxonomy
        .into_iter()
        .map(|(conflict_type, count)| TaxonomyCount {
            conflict_type,
            count,
        })
        .collect();

    ReportSummary {
        n_documents: documents.len(),
        n_spans: bundle.spans.len(),
        n_claims: claims.len(),
        n_rules: bundle.rules.len(),
        n_conflicts: conflicts.conflicts.len(),
        certificate_depth_distribution,
        conflict_taxonomy_counts,
    }
}

/// Render a serde string-valued enum ([`Severity`], [`ConflictClassification`],
/// `ckc_core::enums::Language`) as its canonical snake_case token. Serializing
/// through serde keeps the card's string fields in lockstep with the schema
/// rather than a hand-maintained mapping that could drift.
fn token<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// Assemble one [`ConflictCard`] (SPEC 21) for `conflict`, realizing the
/// 10-element §21 card order over the already-normalized Phase-0 artifacts:
///
/// 1. `source_spans`: each `conflict.source_spans` id resolved to its
///    [`SourceSpan`] in `spans`, in conflict order, JA source-first.
/// 2. `gloss_ja`/`gloss_en`: glosses of every `claims` entry whose
///    `source_span_ids` intersect the conflict's spans, joined by `" / "`
///    (empty when none, e.g. decision-table cells with no owning claim).
/// 3. `normalized_view`: the conflict's normalized CKC view verbatim.
/// 4. `explanation_ja`/`explanation_en`: a deterministic bilingual description
///    of type/severity/classification, distinct from the element-9 question.
/// 5. `witness`: the conflict's inline `solver_evidence` (sat-model / EC-model /
///    overlap-witness), kept inline with no join to `ExecutionWitness`.
/// 6. `certificate_evidence`: every `verification.certificates` entry whose
///    `content_hash` is in `conflict.minimal_artifact_set` (the real per-conflict
///    linkage from `ckc_conflict::link::minimal_artifact_set`), sorted by id.
/// 7. `certificate_depth`: the deepest [`CertificateClass`] in (6).
/// 8. `repair_candidates`: the conflict's source-revision repair candidates.
/// 9. `human_review_question_ja`/`_en`: the conflict's review questions.
/// 10. `adjudication_status`: the initial `"pending_adjudication"`.
pub fn build_conflict_card(
    conflict: &Conflict,
    spans: &[SourceSpan],
    claims: &[ClinicalClaim],
    verification: &VerificationReport,
) -> ConflictCard {
    // (1) source spans, resolved in conflict order, JA source-first.
    let source_spans: Vec<CardSpan> = conflict
        .source_spans
        .iter()
        .filter_map(|sid| spans.iter().find(|span| span.span_id == *sid))
        .map(|span| CardSpan {
            span_id: span.span_id.as_str().to_string(),
            raw_text: span.raw_text.clone(),
            display_text: span.display_text.clone(),
            table_cell: span
                .table_cell
                .as_ref()
                .map(|cell| serde_json::to_value(cell).expect("TableCellRef must serialize")),
            language: token(&span.language),
        })
        .collect();

    // (2) bilingual glosses from claims whose spans intersect the conflict's,
    // joined in claim (fixture) order so the join stays deterministic.
    let matched: Vec<&ClinicalClaim> = claims
        .iter()
        .filter(|claim| {
            claim
                .source_span_ids
                .iter()
                .any(|sid| conflict.source_spans.contains(sid))
        })
        .collect();
    let gloss_ja = matched
        .iter()
        .map(|claim| claim.gloss_ja.as_str())
        .collect::<Vec<_>>()
        .join(" / ");
    let gloss_en = matched
        .iter()
        .map(|claim| claim.gloss_en.as_str())
        .collect::<Vec<_>>()
        .join(" / ");

    // (4) deterministic bilingual description, distinct from the (9) question.
    let severity = token(&conflict.severity);
    let classification = token(&conflict.classification);
    let explanation_en = format!(
        "Detected a {} conflict with {} severity, classified as {}.",
        conflict.conflict_type, severity, classification
    );
    let explanation_ja = format!(
        "{} の不整合を検出しました。重大度は {}、分類は {} です。",
        conflict.conflict_type, severity, classification
    );

    // (6) certificate evidence: the certificates the conflict's minimal
    // artifact set already names, resolved against the verification portfolio.
    let mut certificate_evidence: Vec<CardCertificate> = verification
        .certificates
        .iter()
        .filter(|cert| conflict.minimal_artifact_set.contains(&content_hash(cert)))
        .map(|cert| CardCertificate {
            certificate_id: cert.certificate_id.as_str().to_string(),
            certificate_class: cert.certificate_class,
            solver_or_checker: cert.solver_or_checker.clone(),
        })
        .collect();
    certificate_evidence.sort_by(|a, b| a.certificate_id.cmp(&b.certificate_id));

    // (7) deepest achieved certificate class over the evidence.
    let certificate_depth = certificate_evidence
        .iter()
        .map(|cert| cert.certificate_class)
        .max();

    ConflictCard {
        conflict_id: conflict.conflict_id.as_str().to_string(),
        conflict_type: conflict.conflict_type.clone(),
        severity: conflict.severity,
        classification: conflict.classification,
        source_spans,
        gloss_ja,
        gloss_en,
        normalized_view: conflict.normalized_view.clone(),
        explanation_ja,
        explanation_en,
        witness: conflict.solver_evidence.clone(),
        certificate_evidence,
        certificate_depth,
        repair_candidates: conflict.repair_candidates.clone(),
        human_review_question_ja: conflict.human_review_question_ja.clone(),
        human_review_question_en: conflict.human_review_question_en.clone(),
        adjudication_status: "pending_adjudication".to_string(),
    }
}
