//! Gate for task 0.12.2: the report summary (SPEC 23) tallies the Phase-0 toy
//! bundle's corpus/extraction/claim/rule/conflict counts, certificate-depth
//! distribution, and conflict taxonomy deterministically over
//! `CompileBundle::load_toy()` + `verify_all` + `detect_all`.

use ckc_compile::CompileBundle;
use ckc_conflict::detect_all;
use ckc_report::{
    CertificateClass, DepthCount, TaxonomyCount, build_summary, load_claims, load_documents,
};
use ckc_verify::verify_all;

#[test]
fn summary_tallies_toy_bundle() {
    let bundle = CompileBundle::load_toy();
    let claims = load_claims();
    let documents = load_documents();
    let verification = verify_all(&bundle);
    let conflicts = detect_all(&bundle, &verification);

    let summary = build_summary(&bundle, &claims, &documents, &verification, &conflicts);

    assert_eq!(summary.n_documents, 3);
    assert_eq!(summary.n_spans, 16);
    assert_eq!(summary.n_claims, 2);
    assert_eq!(summary.n_rules, 3);
    assert_eq!(summary.n_conflicts, 3);

    // Depth distribution covers the 11 accepted certificates and includes the
    // single C7-Kernel Lean certificate.
    let depth_sum: usize = summary
        .certificate_depth_distribution
        .iter()
        .map(|d| d.count)
        .sum();
    assert_eq!(depth_sum, 11);
    assert!(
        summary
            .certificate_depth_distribution
            .contains(&DepthCount {
                certificate_class: CertificateClass::C7Kernel,
                count: 1,
            })
    );

    // Taxonomy is emitted in ascending conflict_type order.
    assert_eq!(
        summary.conflict_taxonomy_counts,
        vec![
            TaxonomyCount {
                conflict_type: "decision_table_overlap".to_string(),
                count: 1,
            },
            TaxonomyCount {
                conflict_type: "norm_contradiction".to_string(),
                count: 1,
            },
            TaxonomyCount {
                conflict_type: "temporal_violation".to_string(),
                count: 1,
            },
        ]
    );
}
