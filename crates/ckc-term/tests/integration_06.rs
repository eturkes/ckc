use std::fs;

use ckc_core::canonical::{ContentHash, content_hash, to_canonical_bytes};
use ckc_core::clinical::Rule;
use ckc_core::enums::BindingStatus;
use ckc_core::envelope::{ArtifactEnvelope, ArtifactKind, ArtifactMeta};
use ckc_core::id::{ConceptId, EGraphClassId};
use ckc_core::nf::normalize_with_terms;
use ckc_core::profile::SemanticProfile;
use ckc_core::source::{Concept, TerminologyBinding};
use ckc_store::ContentStore;
use ckc_term::TerminologyGraph;
use ckc_term::alignment::{AlignmentIncoherence, check_alignment_coherence};
use ckc_term::egraph::TermEquivalence;
use ckc_term::rdf::export_skos_turtle;
use ckc_term::shacl::{ShaclReport, validate_rules};
use tempfile::TempDir;

const CONCEPTS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/research_kernel/fixtures/concepts.json"
);
const RULES_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/research_kernel/fixtures/rules.json"
);

fn load_fixtures() -> (TerminologyGraph, Vec<Rule>) {
    let concepts_json = fs::read_to_string(CONCEPTS_PATH).unwrap();
    let rules_json = fs::read_to_string(RULES_PATH).unwrap();
    let graph = TerminologyGraph::load_from_json(&concepts_json).unwrap();
    let rules: Vec<Rule> = serde_json::from_str(&rules_json).unwrap();
    (graph, rules)
}

fn test_meta(stage: &str, profiles: Vec<SemanticProfile>) -> ArtifactMeta {
    ArtifactMeta {
        schema_version: "0.0.0".into(),
        producer_version: "ckc-term/integration".into(),
        command_manifest: serde_json::json!({"command": "ckc", "stage": stage}),
        source_input_hashes: vec![],
        parent_hashes: vec![],
        stage: stage.into(),
        semantic_profiles: profiles,
        content_hash: ContentHash(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
        ),
        certificate_ids: vec![],
        replay_command: None,
    }
}

fn plant_incoherent_concept(graph: &mut TerminologyGraph) {
    graph.insert(Concept {
        concept_id: ConceptId::new("concept_incoherent_bl_test"),
        label_ja: "βラクタム系テスト".into(),
        label_en: Some("beta-lactam test".into()),
        semantic_type: "drug_class".into(),
        terminology_bindings: vec![TerminologyBinding {
            system: "MEDIS".into(),
            code: Some("Y9999".into()),
            version: Some("2024".into()),
            label: "テスト薬剤".into(),
            status: BindingStatus::Exact,
            mapping_relation: "equivalent".into(),
            provenance: "test_incoherent".into(),
            confidence: 0.95,
            license_status: "permitted".into(),
            valid_from: None,
            valid_to: None,
        }],
        egraph_class_id: Some(EGraphClassId::new("eclass_beta_lactam")),
        source_span_ids: vec![],
    });
}

/// Wrapper for storing RDF Turtle export as a CAS artifact.
#[derive(serde::Serialize, serde::Deserialize)]
struct RdfExportPayload {
    turtle: String,
    concept_count: usize,
}

/// Run the full 0.6 pipeline and store all artifacts in CAS.
/// Returns the store, its temp dir (for lifetime), and the manifest.
fn run_pipeline(
    graph: &TerminologyGraph,
    rules: &[Rule],
) -> (ContentStore, TempDir, ckc_store::StoreManifest) {
    // 1. E-graph saturation
    let equiv = TermEquivalence::from_terminology_graph(graph);
    let egraph_artifact = equiv.emit_artifact(graph);

    // 2. NF pass 8 on all Rules
    let term_map = equiv.to_canonical_map();
    let mut rules_nf: Vec<Rule> = rules.to_vec();
    for rule in &mut rules_nf {
        normalize_with_terms(rule, term_map.clone());
    }

    // 3. RDF/SKOS export
    let turtle = export_skos_turtle(graph);
    let rdf_payload = RdfExportPayload {
        concept_count: graph.len(),
        turtle,
    };

    // 4. SHACL validate
    let shacl_report = validate_rules(&rules_nf);

    // 5. Coherence check
    let incoherences = check_alignment_coherence(graph, &equiv);

    // 6. Store all artifacts via CAS
    let tmp = TempDir::new().unwrap();
    let store = ContentStore::new(tmp.path());

    let meta_term = test_meta("terminology", vec![SemanticProfile::Term]);
    let meta_norm = test_meta("normalize", vec![SemanticProfile::Norm]);

    store
        .put(&ArtifactEnvelope::wrap(
            ArtifactKind::EgraphArtifact,
            &egraph_artifact,
            meta_term.clone(),
        ))
        .unwrap();

    store
        .put(&ArtifactEnvelope::wrap(
            ArtifactKind::RdfExport,
            &rdf_payload,
            meta_term.clone(),
        ))
        .unwrap();

    store
        .put(&ArtifactEnvelope::wrap(
            ArtifactKind::ShaclReport,
            &shacl_report,
            meta_norm.clone(),
        ))
        .unwrap();

    store
        .put(&ArtifactEnvelope::wrap(
            ArtifactKind::AlignmentDiagnostic,
            &incoherences,
            meta_term.clone(),
        ))
        .unwrap();

    // Store normalized rules
    for rule in &rules_nf {
        store
            .put(&ArtifactEnvelope::wrap(
                ArtifactKind::Rule,
                rule,
                meta_norm.clone(),
            ))
            .unwrap();
    }

    let manifest = store.generate_manifest().unwrap();
    (store, tmp, manifest)
}

// -----------------------------------------------------------------------
// Pipeline integration tests
// -----------------------------------------------------------------------

#[test]
fn full_pipeline_clean_fixtures() {
    let (graph, rules) = load_fixtures();
    let equiv = TermEquivalence::from_terminology_graph(&graph);

    // E-graph produces expected classes
    let artifact = equiv.emit_artifact(&graph);
    assert_eq!(artifact.class_ids.len(), 6);

    // NF pass 8 converges variant rules
    let term_map = equiv.to_canonical_map();
    let mut rules_nf = rules.clone();
    for rule in &mut rules_nf {
        normalize_with_terms(rule, term_map.clone());
    }

    // RDF export is non-empty and parseable
    let turtle = export_skos_turtle(&graph);
    assert!(turtle.contains("skos:"));
    assert!(turtle.contains("βラクタム系抗菌薬"));

    // SHACL catches the incomplete rule (2 violations: source_span_ids + provenance)
    // NF pass 12 renames rule_id to a stable nf-<hash>, so match on count only.
    let report = validate_rules(&rules_nf);
    assert!(!report.conforms);
    assert_eq!(report.violations.len(), 2);

    // Clean fixtures have no alignment incoherences
    let incoherences = check_alignment_coherence(&graph, &equiv);
    assert!(
        incoherences.is_empty(),
        "clean fixtures must have zero alignment incoherences"
    );
}

#[test]
fn planted_incoherence_detected_in_pipeline() {
    let (mut graph, _rules) = load_fixtures();
    plant_incoherent_concept(&mut graph);
    let equiv = TermEquivalence::from_terminology_graph(&graph);
    let incoherences = check_alignment_coherence(&graph, &equiv);

    assert!(
        !incoherences.is_empty(),
        "planted incoherence must be detected"
    );
    let medis_hit = incoherences
        .iter()
        .any(|d| d.system == "MEDIS" && (d.code_a == "Y9999" || d.code_b == "Y9999"));
    assert!(medis_hit, "MEDIS Y9999 incoherence must appear");
}

#[test]
fn pipeline_artifacts_stored_in_cas() {
    let (graph, rules) = load_fixtures();
    let (store, _tmp, manifest) = run_pipeline(&graph, &rules);

    // 3 rules + egraph + rdf + shacl + alignment = 7 artifacts
    assert_eq!(
        manifest.entries.len(),
        7,
        "expected 7 artifacts in CAS: {:?}",
        manifest
            .entries
            .iter()
            .map(|e| format!("{:?}", e.kind))
            .collect::<Vec<_>>()
    );

    // Every stored object verifies
    for entry in &manifest.entries {
        assert!(
            store.verify(&entry.hash).unwrap(),
            "artifact {:?} must verify",
            entry.kind
        );
    }
}

#[test]
fn manifest_deterministic_across_reruns() {
    let (graph, rules) = load_fixtures();

    // Run 1: fresh store
    let (_store1, _tmp1, m1) = run_pipeline(&graph, &rules);

    // Run 2: independent fresh store
    let (_store2, _tmp2, m2) = run_pipeline(&graph, &rules);

    // Content hashes (excluding stored_at_epoch) must match
    let hashes1: Vec<_> = m1.entries.iter().map(|e| &e.hash).collect();
    let hashes2: Vec<_> = m2.entries.iter().map(|e| &e.hash).collect();
    assert_eq!(
        hashes1, hashes2,
        "two independent pipeline runs must produce identical artifact hashes"
    );
}

#[test]
fn manifest_stable_on_idempotent_rerun() {
    let (graph, rules) = load_fixtures();

    // Run into one store twice
    let tmp = TempDir::new().unwrap();
    let store = ContentStore::new(tmp.path());

    let equiv = TermEquivalence::from_terminology_graph(&graph);
    let egraph_artifact = equiv.emit_artifact(&graph);
    let term_map = equiv.to_canonical_map();
    let mut rules_nf: Vec<Rule> = rules.to_vec();
    for rule in &mut rules_nf {
        normalize_with_terms(rule, term_map.clone());
    }
    let turtle = export_skos_turtle(&graph);
    let rdf_payload = RdfExportPayload {
        concept_count: graph.len(),
        turtle,
    };
    let shacl_report = validate_rules(&rules_nf);
    let incoherences = check_alignment_coherence(&graph, &equiv);

    let meta_term = test_meta("terminology", vec![SemanticProfile::Term]);
    let meta_norm = test_meta("normalize", vec![SemanticProfile::Norm]);

    let put_all = || {
        store
            .put(&ArtifactEnvelope::wrap(
                ArtifactKind::EgraphArtifact,
                &egraph_artifact,
                meta_term.clone(),
            ))
            .unwrap();
        store
            .put(&ArtifactEnvelope::wrap(
                ArtifactKind::RdfExport,
                &rdf_payload,
                meta_term.clone(),
            ))
            .unwrap();
        store
            .put(&ArtifactEnvelope::wrap(
                ArtifactKind::ShaclReport,
                &shacl_report,
                meta_norm.clone(),
            ))
            .unwrap();
        store
            .put(&ArtifactEnvelope::wrap(
                ArtifactKind::AlignmentDiagnostic,
                &incoherences,
                meta_term.clone(),
            ))
            .unwrap();
        for rule in &rules_nf {
            store
                .put(&ArtifactEnvelope::wrap(
                    ArtifactKind::Rule,
                    rule,
                    meta_norm.clone(),
                ))
                .unwrap();
        }
    };

    put_all();
    let m1 = store.generate_manifest().unwrap();

    put_all();
    let m2 = store.generate_manifest().unwrap();

    assert_eq!(
        to_canonical_bytes(&m1),
        to_canonical_bytes(&m2),
        "idempotent CAS puts must produce identical manifests"
    );
    assert_eq!(content_hash(&m1), content_hash(&m2));
}

#[test]
fn shacl_report_storable_and_extractable() {
    let (_graph, rules) = load_fixtures();
    let report = validate_rules(&rules);
    let meta = test_meta("validate", vec![SemanticProfile::Norm]);
    let envelope = ArtifactEnvelope::wrap(ArtifactKind::ShaclReport, &report, meta);
    assert!(envelope.verify_content_hash());
    let extracted: ShaclReport = envelope.extract().unwrap();
    assert_eq!(report, extracted);
}

#[test]
fn alignment_diagnostic_storable_and_extractable() {
    let mut graph = load_fixtures().0;
    plant_incoherent_concept(&mut graph);
    let equiv = TermEquivalence::from_terminology_graph(&graph);
    let diags = check_alignment_coherence(&graph, &equiv);

    let meta = test_meta("terminology", vec![SemanticProfile::Term]);
    let envelope = ArtifactEnvelope::wrap(ArtifactKind::AlignmentDiagnostic, &diags, meta);
    assert!(envelope.verify_content_hash());
    let extracted: Vec<AlignmentIncoherence> = envelope.extract().unwrap();
    assert_eq!(diags, extracted);
}
