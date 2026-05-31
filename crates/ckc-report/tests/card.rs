//! Gate for task 0.12.3: each detected Phase-0 conflict assembles into a
//! `ConflictCard` in the SPEC-21 card order — JA source spans first, bilingual
//! glosses, the real per-conflict certificate evidence with its depth badge,
//! the inline witness, and the repair candidates — over
//! `CompileBundle::load_toy()` + `verify_all` + `detect_all`. Conflicts are keyed
//! on `conflict_type` because their ids are content-addressed (`nf-…`).

use ckc_compile::CompileBundle;
use ckc_conflict::{Conflict, detect_all};
use ckc_core::clinical::ClinicalClaim;
use ckc_core::source::SourceSpan;
use ckc_report::{CertificateClass, ConflictCard, build_conflict_card, load_claims};
use ckc_verify::{VerificationReport, verify_all};

/// Build the card for the single conflict of `conflict_type`.
fn card_for(
    conflicts: &[Conflict],
    spans: &[SourceSpan],
    claims: &[ClinicalClaim],
    verification: &VerificationReport,
    conflict_type: &str,
) -> ConflictCard {
    let conflict = conflicts
        .iter()
        .find(|c| c.conflict_type == conflict_type)
        .unwrap_or_else(|| panic!("conflict type {conflict_type} present"));
    build_conflict_card(conflict, spans, claims, verification)
}

/// The certificate ids backing a card, in card (id-sorted) order.
fn cert_ids(card: &ConflictCard) -> Vec<&str> {
    card.certificate_evidence
        .iter()
        .map(|c| c.certificate_id.as_str())
        .collect()
}

/// The solvers backing a card, in card (id-sorted) order.
fn cert_solvers(card: &ConflictCard) -> Vec<&str> {
    card.certificate_evidence
        .iter()
        .map(|c| c.solver_or_checker.as_str())
        .collect()
}

#[test]
fn conflict_cards_follow_spec21_order() {
    let bundle = CompileBundle::load_toy();
    let claims = load_claims();
    let verification = verify_all(&bundle);
    let conflicts = detect_all(&bundle, &verification).conflicts;

    // --- norm_contradiction: the sepsis/anaphylaxis β-lactam pair ---
    let norm = card_for(
        &conflicts,
        &bundle.spans,
        &claims,
        &verification,
        "norm_contradiction",
    );
    assert_eq!(norm.source_spans.len(), 2);
    let sepsis = norm
        .source_spans
        .iter()
        .find(|s| s.span_id == "span_rec_sepsis_bl")
        .expect("sepsis recommendation span present");
    assert!(sepsis.raw_text.contains("敗血症"));
    // The two norm claims both intersect the conflict, so the gloss is non-empty.
    assert!(!norm.gloss_en.is_empty());
    assert_eq!(
        cert_ids(&norm),
        vec![
            "cert_clingo_defeasible",
            "cert_cvc5_norm_conflict",
            "cert_lean_norm_conflict",
            "cert_z3_norm_conflict",
        ]
    );
    assert_eq!(cert_solvers(&norm), vec!["clingo", "cvc5", "lean", "z3"]);
    assert_eq!(norm.certificate_depth, Some(CertificateClass::C7Kernel));
    assert_eq!(norm.witness.len(), 1);
    assert_eq!(norm.repair_candidates.len(), 2);

    // --- decision_table_overlap: vital-sign rows over cell spans, no claim ---
    let dt = card_for(
        &conflicts,
        &bundle.spans,
        &claims,
        &verification,
        "decision_table_overlap",
    );
    assert!(dt.gloss_en.is_empty());
    assert_eq!(cert_ids(&dt), vec!["cert_z3_decision_table"]);
    assert_eq!(cert_solvers(&dt), vec!["z3"]);
    assert_eq!(dt.certificate_depth, Some(CertificateClass::C4Executable));
    assert_eq!(dt.repair_candidates.len(), 2);

    // --- temporal_violation: allergy-history Event Calculus narrative ---
    let ec = card_for(
        &conflicts,
        &bundle.spans,
        &claims,
        &verification,
        "temporal_violation",
    );
    assert_eq!(cert_ids(&ec), vec!["cert_clingo_event_calculus"]);
    assert_eq!(cert_solvers(&ec), vec!["clingo"]);
    assert_eq!(ec.certificate_depth, Some(CertificateClass::C4Executable));
    assert_eq!(ec.repair_candidates.len(), 1);
}
