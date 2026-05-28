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
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.concepts.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn find_by_system_code(&self, system: &str, code: &str) -> Vec<&Concept> {
        let key = (system.to_owned(), code.to_owned());
        self.by_system_code
            .get(&key)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.concepts.get(id))
                    .collect()
            })
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
        "/../../examples/toy_research_kernel/fixtures/concepts.json"
    );

    fn load_fixture() -> TerminologyGraph {
        let json = std::fs::read_to_string(FIXTURE_PATH)
            .expect("concepts.json fixture must exist");
        TerminologyGraph::load_from_json(&json).expect("fixture must parse")
    }

    #[test]
    fn load_all_toy_concepts() {
        let graph = load_fixture();
        assert_eq!(graph.len(), 10);
    }

    #[test]
    fn get_by_id_returns_known_concept() {
        let graph = load_fixture();
        let sepsis = graph
            .get_by_id(&ConceptId::new("concept_sepsis"))
            .expect("concept_sepsis must exist");
        assert_eq!(sepsis.label_ja, "敗血症");
        assert_eq!(sepsis.label_en.as_deref(), Some("sepsis"));
    }

    #[test]
    fn get_by_id_returns_none_for_unknown() {
        let graph = load_fixture();
        assert!(graph.get_by_id(&ConceptId::new("nonexistent")).is_none());
    }

    #[test]
    fn beta_lactam_variants_share_egraph_class() {
        let graph = load_fixture();
        let variants = graph.get_by_egraph_class("eclass_beta_lactam");
        assert_eq!(variants.len(), 5);

        let ids: HashSet<&str> = variants.iter().map(|c| c.concept_id.as_str()).collect();
        assert!(ids.contains("concept_beta_lactam"));
        assert!(ids.contains("concept_bl_variant_katakana"));
        assert!(ids.contains("concept_bl_variant_hyphenated"));
        assert!(ids.contains("concept_bl_variant_english"));
        assert!(ids.contains("concept_bl_variant_brand"));
    }

    #[test]
    fn egraph_class_returns_empty_for_unknown() {
        let graph = load_fixture();
        assert!(graph.get_by_egraph_class("nonexistent").is_empty());
    }

    #[test]
    fn find_by_system_code_medis_y0100() {
        let graph = load_fixture();
        let results = graph.find_by_system_code("MEDIS", "Y0100");
        // Primary beta-lactam + katakana variant + hyphenated variant + brand (narrower)
        assert_eq!(results.len(), 4);
        let ids: HashSet<&str> = results.iter().map(|c| c.concept_id.as_str()).collect();
        assert!(ids.contains("concept_beta_lactam"));
        assert!(ids.contains("concept_bl_variant_katakana"));
        assert!(ids.contains("concept_bl_variant_hyphenated"));
        assert!(ids.contains("concept_bl_variant_brand"));
    }

    #[test]
    fn find_by_system_code_jlac11() {
        let graph = load_fixture();
        let results = graph.find_by_system_code("JLAC11", "9N611000000000001");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].concept_id.as_str(), "concept_body_temperature");
    }

    #[test]
    fn find_by_system_code_returns_empty_for_unknown() {
        let graph = load_fixture();
        assert!(graph.find_by_system_code("FAKE", "000").is_empty());
    }

    #[test]
    fn search_by_label_ja_partial() {
        let graph = load_fixture();
        // "ラクタム" matches all beta-lactam JA variants containing that substring
        let results = graph.search_by_label("ラクタム");
        assert!(results.len() >= 3, "expected >= 3 JA matches, got {}", results.len());

        let ids: HashSet<&str> = results.iter().map(|c| c.concept_id.as_str()).collect();
        assert!(ids.contains("concept_beta_lactam"));
        assert!(ids.contains("concept_bl_variant_katakana"));
        assert!(ids.contains("concept_bl_variant_hyphenated"));
    }

    #[test]
    fn search_by_label_en_partial() {
        let graph = load_fixture();
        // "beta-lactam" matches EN labels with ASCII "beta"; concept_bl_variant_hyphenated
        // uses Greek β in its EN label so it correctly does not match here.
        let results = graph.search_by_label("beta-lactam");
        let ids: HashSet<&str> = results.iter().map(|c| c.concept_id.as_str()).collect();
        assert!(ids.contains("concept_beta_lactam"));
        assert!(ids.contains("concept_bl_variant_katakana"));
        assert!(ids.contains("concept_bl_variant_english"));
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn search_by_label_case_insensitive() {
        let graph = load_fixture();
        let results = graph.search_by_label("Sepsis");
        assert!(
            results.iter().any(|c| c.concept_id.as_str() == "concept_sepsis"),
            "case-insensitive EN search must find sepsis"
        );
    }

    #[test]
    fn search_by_label_ja_vital_sign() {
        let graph = load_fixture();
        let results = graph.search_by_label("血圧");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].concept_id.as_str(), "concept_blood_pressure");
    }

    #[test]
    fn each_concept_queryable_by_id() {
        let graph = load_fixture();
        let expected_ids = [
            "concept_beta_lactam",
            "concept_bl_variant_katakana",
            "concept_bl_variant_hyphenated",
            "concept_bl_variant_english",
            "concept_bl_variant_brand",
            "concept_sepsis",
            "concept_anaphylaxis",
            "concept_body_temperature",
            "concept_heart_rate",
            "concept_blood_pressure",
        ];
        for id_str in &expected_ids {
            assert!(
                graph.get_by_id(&ConceptId::new(*id_str)).is_some(),
                "concept {id_str} must be queryable by ID"
            );
        }
    }

    #[test]
    fn each_egraph_class_returns_correct_count() {
        let graph = load_fixture();
        let expected = [
            ("eclass_beta_lactam", 5),
            ("eclass_sepsis", 1),
            ("eclass_anaphylaxis", 1),
            ("eclass_body_temperature", 1),
            ("eclass_heart_rate", 1),
            ("eclass_blood_pressure", 1),
        ];
        for (class_id, count) in &expected {
            let results = graph.get_by_egraph_class(class_id);
            assert_eq!(
                results.len(),
                *count,
                "egraph class {class_id} expected {count} concepts, got {}",
                results.len()
            );
        }
    }

    #[test]
    fn insert_then_query() {
        let mut graph = TerminologyGraph::new();
        let concept = Concept {
            concept_id: ConceptId::new("test_concept"),
            label_ja: "テスト".into(),
            label_en: Some("test".into()),
            semantic_type: "test".into(),
            terminology_bindings: vec![],
            egraph_class_id: Some(EGraphClassId::new("eclass_test")),
            source_span_ids: vec![],
        };
        graph.insert(concept);
        assert_eq!(graph.len(), 1);
        assert!(graph.get_by_id(&ConceptId::new("test_concept")).is_some());
        assert_eq!(graph.get_by_egraph_class("eclass_test").len(), 1);
    }

    #[test]
    fn empty_graph() {
        let graph = TerminologyGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);
        assert!(graph.get_by_id(&ConceptId::new("x")).is_none());
        assert!(graph.get_by_egraph_class("x").is_empty());
        assert!(graph.find_by_system_code("x", "y").is_empty());
        assert!(graph.search_by_label("x").is_empty());
    }
}
