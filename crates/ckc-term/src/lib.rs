pub mod alignment;
pub mod egraph;
pub mod rdf;
pub mod shacl;

use std::collections::HashMap;

use ckc_core::id::{ConceptId, EGraphClassId};
use ckc_core::source::Concept;

/// In-memory terminology graph with indexed concept lookups.
///
/// Maintains secondary indexes on egraph_class_id and (system, code) pairs.
/// Label search uses normalized substring matching over JA and EN labels.
pub struct TerminologyGraph {
    concepts: HashMap<ConceptId, Concept>,
    by_egraph_class: HashMap<EGraphClassId, Vec<ConceptId>>,
    by_system_code: HashMap<(String, String), Vec<ConceptId>>,
}

impl TerminologyGraph {
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
            by_egraph_class: HashMap::new(),
            by_system_code: HashMap::new(),
        }
    }

    pub fn insert(&mut self, concept: Concept) {
        let id = concept.concept_id.clone();

        if let Some(ref class_id) = concept.egraph_class_id {
            self.by_egraph_class
                .entry(class_id.clone())
                .or_default()
                .push(id.clone());
        }

        for binding in &concept.terminology_bindings {
            if let Some(ref code) = binding.code {
                self.by_system_code
                    .entry((binding.system.clone(), code.clone()))
                    .or_default()
                    .push(id.clone());
            }
        }

        self.concepts.insert(id, concept);
    }

    pub fn get_by_id(&self, id: &ConceptId) -> Option<&Concept> {
        self.concepts.get(id)
    }

    pub fn get_by_egraph_class(&self, class_id: &str) -> Vec<&Concept> {
        let key = EGraphClassId::new(class_id);
        self.by_egraph_class
            .get(&key)
            .map(|ids| ids.iter().filter_map(|id| self.concepts.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_by_system_code(&self, system: &str, code: &str) -> Vec<&Concept> {
        let key = (system.to_owned(), code.to_owned());
        self.by_system_code
            .get(&key)
            .map(|ids| ids.iter().filter_map(|id| self.concepts.get(id)).collect())
            .unwrap_or_default()
    }

    /// Substring search over JA and EN labels (case-insensitive ASCII,
    /// kana-aware through direct Unicode contains).
    pub fn search_by_label(&self, query: &str) -> Vec<&Concept> {
        let query_lower = query.to_lowercase();
        self.concepts
            .values()
            .filter(|c| {
                c.label_ja.to_lowercase().contains(&query_lower)
                    || c.label_en
                        .as_ref()
                        .is_some_and(|en| en.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn load_from_json(json: &str) -> Result<Self, serde_json::Error> {
        let concepts: Vec<Concept> = serde_json::from_str(json)?;
        let mut graph = Self::new();
        for concept in concepts {
            graph.insert(concept);
        }
        Ok(graph)
    }

    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    pub fn concepts(&self) -> impl Iterator<Item = &Concept> {
        self.concepts.values()
    }
}

impl Default for TerminologyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const FIXTURE_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/research_kernel/fixtures/concepts.json"
    );

    fn load_fixture() -> TerminologyGraph {
        let json = std::fs::read_to_string(FIXTURE_PATH).expect("concepts.json fixture must exist");
        TerminologyGraph::load_from_json(&json).expect("fixture must parse")
    }

    /// All 10 toy concepts populate every secondary index and are reachable
    /// via id + egraph-class + system-code + JA-label + EN-label lookups.
    #[test]
    fn toy_fixture_round_trip() {
        let graph = load_fixture();
        assert_eq!(graph.len(), 10);

        // Spot-check ID -> label content.
        let sepsis = graph.get_by_id(&ConceptId::new("concept_sepsis")).unwrap();
        assert_eq!(sepsis.label_ja, "敗血症");
        assert_eq!(sepsis.label_en.as_deref(), Some("sepsis"));

        // Every fixture e-graph class returns its expected member count.
        for (class_id, expected) in [
            ("eclass_beta_lactam", 5),
            ("eclass_sepsis", 1),
            ("eclass_anaphylaxis", 1),
            ("eclass_body_temperature", 1),
            ("eclass_heart_rate", 1),
            ("eclass_blood_pressure", 1),
        ] {
            assert_eq!(graph.get_by_egraph_class(class_id).len(), expected);
        }

        // β-lactam variants share their e-graph class.
        let variant_ids: HashSet<&str> = graph
            .get_by_egraph_class("eclass_beta_lactam")
            .iter()
            .map(|c| c.concept_id.as_str())
            .collect();
        for id in [
            "concept_beta_lactam",
            "concept_bl_variant_katakana",
            "concept_bl_variant_hyphenated",
            "concept_bl_variant_english",
            "concept_bl_variant_brand",
        ] {
            assert!(variant_ids.contains(id));
        }
    }

    /// MEDIS Y0100 is shared by the primary β-lactam concept and three
    /// variants (katakana/hyphenated/brand). JLAC11 codes are unique.
    #[test]
    fn find_by_system_code_groups_variants_under_shared_codes() {
        let graph = load_fixture();
        let medis_y0100: HashSet<&str> = graph
            .find_by_system_code("MEDIS", "Y0100")
            .iter()
            .map(|c| c.concept_id.as_str())
            .collect();
        assert_eq!(medis_y0100.len(), 4);
        for id in [
            "concept_beta_lactam",
            "concept_bl_variant_katakana",
            "concept_bl_variant_hyphenated",
            "concept_bl_variant_brand",
        ] {
            assert!(medis_y0100.contains(id));
        }

        let jlac = graph.find_by_system_code("JLAC11", "9N611000000000001");
        assert_eq!(jlac.len(), 1);
        assert_eq!(jlac[0].concept_id.as_str(), "concept_body_temperature");
    }

    /// JA substring match, EN substring match, and case-insensitive EN
    /// match all surface the right concepts; `concept_bl_variant_hyphenated`
    /// uses Greek β in EN so an ASCII "beta-lactam" search must skip it.
    #[test]
    fn search_by_label_covers_ja_en_case_insensitive() {
        let graph = load_fixture();

        let ja_ids: HashSet<&str> = graph
            .search_by_label("ラクタム")
            .iter()
            .map(|c| c.concept_id.as_str())
            .collect();
        assert!(ja_ids.len() >= 3);
        assert!(ja_ids.contains("concept_beta_lactam"));

        let en_ids: HashSet<&str> = graph
            .search_by_label("beta-lactam")
            .iter()
            .map(|c| c.concept_id.as_str())
            .collect();
        assert_eq!(en_ids.len(), 3);
        assert!(en_ids.contains("concept_bl_variant_english"));
        assert!(!en_ids.contains("concept_bl_variant_hyphenated"));

        assert!(
            graph
                .search_by_label("Sepsis")
                .iter()
                .any(|c| c.concept_id.as_str() == "concept_sepsis")
        );

        let bp = graph.search_by_label("血圧");
        assert_eq!(bp.len(), 1);
        assert_eq!(bp[0].concept_id.as_str(), "concept_blood_pressure");
    }
}
