use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_core::enums::BindingStatus;
use ckc_core::id::ConceptId;

use crate::TerminologyGraph;
use crate::egraph::TermEquivalence;

/// Detected incoherence: two concepts in the same e-graph equivalence class
/// have `Exact` terminology bindings to the same system but different codes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AlignmentIncoherence {
    pub concept_a: String,
    pub concept_b: String,
    pub system: String,
    pub code_a: String,
    pub code_b: String,
    pub repair_suggestion: String,
}

/// Check whether any e-graph equivalence class contains concepts with
/// conflicting exact terminology bindings (same system, different code).
///
/// Returns a deterministically ordered list of `AlignmentIncoherence`
/// diagnostics sorted by (concept_a, concept_b, system).
pub fn check_alignment_coherence(
    graph: &TerminologyGraph,
    equiv: &TermEquivalence,
) -> Vec<AlignmentIncoherence> {
    let canonical_map = equiv.to_canonical_map();

    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (concept_id, canonical) in &canonical_map {
        groups
            .entry(canonical.clone())
            .or_default()
            .push(concept_id.clone());
    }

    let mut incoherences = Vec::new();

    for members in groups.values() {
        if members.len() < 2 {
            continue;
        }
        for i in 0..members.len() {
            let Some(a) = graph.get_by_id(&ConceptId::new(&members[i])) else {
                continue;
            };
            for j in (i + 1)..members.len() {
                let Some(b) = graph.get_by_id(&ConceptId::new(&members[j])) else {
                    continue;
                };
                for ba in &a.terminology_bindings {
                    if ba.status != BindingStatus::Exact {
                        continue;
                    }
                    let Some(ref ca) = ba.code else { continue };
                    for bb in &b.terminology_bindings {
                        if bb.status != BindingStatus::Exact || bb.system != ba.system {
                            continue;
                        }
                        let Some(ref cb) = bb.code else { continue };
                        if ca != cb {
                            incoherences.push(AlignmentIncoherence {
                                concept_a: members[i].clone(),
                                concept_b: members[j].clone(),
                                system: ba.system.clone(),
                                code_a: ca.clone(),
                                code_b: cb.clone(),
                                repair_suggestion: format!(
                                    "{} and {} share an equivalence class but have \
                                     conflicting exact {} codes ({} vs {}); \
                                     review the e-graph equivalence or one binding",
                                    members[i], members[j], ba.system, ca, cb
                                ),
                            });
                        }
                    }
                }
            }
        }
    }

    incoherences.sort_by(|a, b| {
        a.concept_a
            .cmp(&b.concept_a)
            .then(a.concept_b.cmp(&b.concept_b))
            .then(a.system.cmp(&b.system))
    });

    incoherences
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::id::EGraphClassId;
    use ckc_core::source::{Concept, TerminologyBinding};

    const FIXTURE_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/research_kernel/fixtures/concepts.json"
    );

    fn load_graph() -> TerminologyGraph {
        let json = std::fs::read_to_string(FIXTURE_PATH).expect("concepts.json fixture must exist");
        TerminologyGraph::load_from_json(&json).expect("fixture must parse")
    }

    /// Insert a concept that shares the β-lactam e-graph class but binds
    /// MEDIS to an incompatible exact code (Y9999 vs the fixture's Y0100).
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

    #[test]
    fn clean_fixture_and_empty_graph_have_no_incoherences() {
        let graph = load_graph();
        let equiv = TermEquivalence::from_terminology_graph(&graph);
        assert!(check_alignment_coherence(&graph, &equiv).is_empty());

        let empty = TerminologyGraph::new();
        let equiv_empty = TermEquivalence::from_terminology_graph(&empty);
        assert!(check_alignment_coherence(&empty, &equiv_empty).is_empty());
    }

    #[test]
    fn planted_incoherence_surfaces_with_repair_text() {
        let mut graph = load_graph();
        plant_incoherent_concept(&mut graph);
        let equiv = TermEquivalence::from_terminology_graph(&graph);
        let result = check_alignment_coherence(&graph, &equiv);
        assert!(!result.is_empty());

        let medis_hits: Vec<_> = result
            .iter()
            .filter(|d| d.system == "MEDIS" && (d.code_a == "Y9999" || d.code_b == "Y9999"))
            .collect();
        assert!(!medis_hits.is_empty());
        for hit in &medis_hits {
            // Every reported MEDIS hit involves the planted concept and
            // carries a non-empty repair suggestion mentioning the system.
            assert!(
                hit.concept_a == "concept_incoherent_bl_test"
                    || hit.concept_b == "concept_incoherent_bl_test"
            );
            assert!(hit.repair_suggestion.contains(&hit.system));
        }
    }
}
