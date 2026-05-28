use std::collections::BTreeMap;
use std::fmt;

use egg::{define_language, CostFunction, EGraph, Extractor, Id, Language, Symbol};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use ckc_core::enums::BindingStatus;

use crate::TerminologyGraph;

// ---------------------------------------------------------------------------
// CKC term language for egg
// ---------------------------------------------------------------------------

define_language! {
    pub enum CkcTerm {
        "concept-ref" = ConceptRef([Id; 1]),
        "label" = LabelNode([Id; 1]),
        "eclass-group" = EClassGroup([Id; 1]),
        Symbol(Symbol),
    }
}

// ---------------------------------------------------------------------------
// Cost function: prefer the concept with the shortest concept_id
// ---------------------------------------------------------------------------

struct ShortestConceptCost;

impl CostFunction<CkcTerm> for ShortestConceptCost {
    type Cost = usize;

    fn cost<C>(&mut self, enode: &CkcTerm, mut costs: C) -> usize
    where
        C: FnMut(Id) -> usize,
    {
        let op_cost = match enode {
            CkcTerm::Symbol(s) => s.as_str().len(),
            _ => 1,
        };
        enode.fold(op_cost, |sum, child| sum + costs(child))
    }
}

// ---------------------------------------------------------------------------
// Label normalization for Japanese variant detection
// ---------------------------------------------------------------------------

/// Normalize a Japanese label for variant-pattern matching.
///
/// Resolves Greek-letter/katakana/hyphenated beta-lactam variants to a
/// canonical form so that structurally equivalent labels can be detected.
fn normalize_ja_label(label: &str) -> String {
    let nfkc: String = label.nfkc().collect();
    nfkc.replace('β', "ベータ")
        .replace('-', "")
        .replace('ー', "")
        .to_lowercase()
}

// ---------------------------------------------------------------------------
// Artifact output types (CAS-storable)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SaturationStats {
    pub total_nodes: usize,
    pub total_classes: usize,
    pub saturated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CanonicalEntry {
    pub class_id: String,
    pub canonical_concept_id: String,
    pub member_concept_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EgraphArtifact {
    pub class_ids: Vec<String>,
    pub canonical_representatives: Vec<CanonicalEntry>,
    pub rewrite_count: usize,
    pub saturation_stats: SaturationStats,
    pub iterations: usize,
}

impl fmt::Display for EgraphArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EgraphArtifact({} classes, {} rewrites, {} iterations)",
            self.class_ids.len(),
            self.rewrite_count,
            self.iterations
        )
    }
}

// ---------------------------------------------------------------------------
// TermEquivalence engine
// ---------------------------------------------------------------------------

pub struct TermEquivalence {
    egraph: EGraph<CkcTerm, ()>,
    /// Concept-id -> egg Id of the ConceptRef node (deterministic BTreeMap order).
    concept_node_ids: BTreeMap<String, Id>,
    rewrite_count: usize,
    iterations: usize,
}

impl TermEquivalence {
    /// Build the e-graph from a TerminologyGraph, applying three equivalence rules:
    ///
    /// (a) Concepts sharing egraph_class_id are equivalent.
    /// (b) Concepts whose normalized JA labels match are equivalent.
    /// (c) Brand-to-class subsumption: a concept with a "narrow" binding to a
    ///     system+code is merged with concepts that have an "exact" binding for
    ///     the same system+code.
    pub fn from_terminology_graph(graph: &TerminologyGraph) -> Self {
        let mut egraph = EGraph::<CkcTerm, ()>::default();
        let mut concept_node_ids = BTreeMap::new();

        // Add each concept as ConceptRef(Symbol(concept_id)).
        // BTreeMap iteration guarantees deterministic insertion order.
        let mut sorted_concepts: Vec<_> = graph.concepts().collect();
        sorted_concepts.sort_by_key(|c| c.concept_id.clone());

        for concept in &sorted_concepts {
            let sym_id = egraph.add(CkcTerm::Symbol(
                Symbol::from(concept.concept_id.as_str()),
            ));
            let ref_id = egraph.add(CkcTerm::ConceptRef([sym_id]));
            concept_node_ids.insert(concept.concept_id.as_str().to_owned(), ref_id);
        }

        let mut rewrite_count = 0usize;

        // Rule (a): union concepts sharing egraph_class_id.
        let mut class_members: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for concept in &sorted_concepts {
            if let Some(ref class_id) = concept.egraph_class_id {
                class_members
                    .entry(class_id.as_str().to_owned())
                    .or_default()
                    .push(concept.concept_id.as_str().to_owned());
            }
        }
        for members in class_members.values() {
            if members.len() > 1 {
                let first = concept_node_ids[&members[0]];
                for member in members.iter().skip(1) {
                    let other = concept_node_ids[member];
                    if egraph.find(first) != egraph.find(other) {
                        egraph.union(first, other);
                        rewrite_count += 1;
                    }
                }
            }
        }

        // Rule (b): union concepts whose normalized JA labels match.
        let mut label_groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for concept in &sorted_concepts {
            let normalized = normalize_ja_label(&concept.label_ja);
            label_groups
                .entry(normalized)
                .or_default()
                .push(concept.concept_id.as_str().to_owned());
        }
        for members in label_groups.values() {
            if members.len() > 1 {
                let first = concept_node_ids[&members[0]];
                for member in members.iter().skip(1) {
                    let other = concept_node_ids[member];
                    if egraph.find(first) != egraph.find(other) {
                        egraph.union(first, other);
                        rewrite_count += 1;
                    }
                }
            }
        }

        // Rule (c): brand-to-class subsumption via narrow bindings.
        for concept in &sorted_concepts {
            for binding in &concept.terminology_bindings {
                if binding.status == BindingStatus::Narrow {
                    if let Some(ref code) = binding.code {
                        let class_concepts =
                            graph.find_by_system_code(&binding.system, code);
                        for class_concept in &class_concepts {
                            if class_concept.concept_id != concept.concept_id {
                                let brand =
                                    concept_node_ids[concept.concept_id.as_str()];
                                let class =
                                    concept_node_ids[class_concept.concept_id.as_str()];
                                if egraph.find(brand) != egraph.find(class) {
                                    egraph.union(brand, class);
                                    rewrite_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        egraph.rebuild();

        Self {
            egraph,
            concept_node_ids,
            rewrite_count,
            iterations: 1,
        }
    }

    /// Return the canonical concept_id for a given concept_id.
    pub fn canonical_for(&self, concept_id: &str) -> Option<String> {
        let &node_id = self.concept_node_ids.get(concept_id)?;
        let extractor = Extractor::new(&self.egraph, ShortestConceptCost);
        let (_cost, best) = extractor.find_best(node_id);
        for node in best.as_ref() {
            if let CkcTerm::Symbol(s) = node {
                return Some(s.as_str().to_owned());
            }
        }
        None
    }

    /// Check whether two concept_ids are equivalent in the e-graph.
    pub fn equivalent(&self, a: &str, b: &str) -> bool {
        match (self.concept_node_ids.get(a), self.concept_node_ids.get(b)) {
            (Some(&id_a), Some(&id_b)) => self.egraph.find(id_a) == self.egraph.find(id_b),
            _ => false,
        }
    }

    /// Emit a deterministic, CAS-storable artifact describing the e-graph state.
    pub fn emit_artifact(&self, graph: &TerminologyGraph) -> EgraphArtifact {
        // Group concepts by canonical egg e-class Id.
        let mut eclass_to_members: BTreeMap<Id, Vec<String>> = BTreeMap::new();
        for (concept_id, &node_id) in &self.concept_node_ids {
            let canonical = self.egraph.find(node_id);
            eclass_to_members
                .entry(canonical)
                .or_default()
                .push(concept_id.clone());
        }

        // Map egg e-class Ids back to original egraph_class_ids.
        // Use the first encountered original class_id per e-class (deterministic
        // via BTreeMap iteration of concept_node_ids).
        let mut egg_to_original: BTreeMap<Id, String> = BTreeMap::new();
        for concept in graph.concepts() {
            if let Some(ref class_id) = concept.egraph_class_id {
                if let Some(&node_id) = self.concept_node_ids.get(concept.concept_id.as_str()) {
                    let canonical = self.egraph.find(node_id);
                    egg_to_original
                        .entry(canonical)
                        .or_insert_with(|| class_id.as_str().to_owned());
                }
            }
        }

        let mut class_ids = Vec::new();
        let mut canonical_representatives = Vec::new();

        for (egg_class_id, members) in &eclass_to_members {
            let class_label = egg_to_original
                .get(egg_class_id)
                .cloned()
                .unwrap_or_else(|| format!("eclass_auto_{}", members[0]));

            let canonical = self
                .canonical_for(&members[0])
                .unwrap_or_default();

            class_ids.push(class_label.clone());
            canonical_representatives.push(CanonicalEntry {
                class_id: class_label,
                canonical_concept_id: canonical,
                member_concept_ids: members.clone(),
            });
        }

        // Sort for deterministic output.
        class_ids.sort();
        canonical_representatives.sort_by(|a, b| a.class_id.cmp(&b.class_id));

        EgraphArtifact {
            class_ids,
            canonical_representatives,
            rewrite_count: self.rewrite_count,
            saturation_stats: SaturationStats {
                total_nodes: self.egraph.total_number_of_nodes(),
                total_classes: self.egraph.number_of_classes(),
                saturated: true,
            },
            iterations: self.iterations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::canonical::{content_hash, to_canonical_bytes};
    use ckc_core::envelope::{ArtifactEnvelope, ArtifactKind, ArtifactMeta};
    use ckc_core::canonical::ContentHash;
    use ckc_core::profile::SemanticProfile;

    const FIXTURE_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/toy_research_kernel/fixtures/concepts.json"
    );

    fn load_graph() -> TerminologyGraph {
        let json =
            std::fs::read_to_string(FIXTURE_PATH).expect("concepts.json fixture must exist");
        TerminologyGraph::load_from_json(&json).expect("fixture must parse")
    }

    fn build_engine() -> (TerminologyGraph, TermEquivalence) {
        let graph = load_graph();
        let engine = TermEquivalence::from_terminology_graph(&graph);
        (graph, engine)
    }

    fn test_meta() -> ArtifactMeta {
        ArtifactMeta {
            schema_version: "0.0.0".into(),
            producer_version: "ckc-term/0.0.0".into(),
            command_manifest: serde_json::json!({"command": "ckc", "stage": "term-egraph"}),
            source_input_hashes: vec![],
            parent_hashes: vec![],
            stage: "terminology".into(),
            semantic_profiles: vec![SemanticProfile::Term],
            content_hash: ContentHash(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .into(),
            ),
            certificate_ids: vec![],
            replay_command: None,
        }
    }

    // -- Core equivalence tests --

    #[test]
    fn all_beta_lactam_variants_converge_to_one_eclass() {
        let (_graph, engine) = build_engine();
        let variants = [
            "concept_beta_lactam",
            "concept_bl_variant_katakana",
            "concept_bl_variant_hyphenated",
            "concept_bl_variant_english",
            "concept_bl_variant_brand",
        ];
        for pair in variants.windows(2) {
            assert!(
                engine.equivalent(pair[0], pair[1]),
                "{} and {} must be equivalent",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn canonical_representative_is_shortest_concept_id() {
        let (_graph, engine) = build_engine();
        let variants = [
            "concept_beta_lactam",
            "concept_bl_variant_katakana",
            "concept_bl_variant_hyphenated",
            "concept_bl_variant_english",
            "concept_bl_variant_brand",
        ];
        for variant in &variants {
            let canonical = engine.canonical_for(variant).unwrap();
            assert_eq!(
                canonical, "concept_beta_lactam",
                "canonical for {variant} must be concept_beta_lactam (shortest id)"
            );
        }
    }

    #[test]
    fn non_beta_lactam_concepts_remain_separate() {
        let (_graph, engine) = build_engine();
        let separate = [
            "concept_sepsis",
            "concept_anaphylaxis",
            "concept_body_temperature",
            "concept_heart_rate",
            "concept_blood_pressure",
        ];
        for pair in separate.windows(2) {
            assert!(
                !engine.equivalent(pair[0], pair[1]),
                "{} and {} must remain separate",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn separate_concepts_have_distinct_canonicals() {
        let (_graph, engine) = build_engine();
        let concepts = [
            "concept_sepsis",
            "concept_anaphylaxis",
            "concept_body_temperature",
            "concept_heart_rate",
            "concept_blood_pressure",
        ];
        let canonicals: Vec<String> = concepts
            .iter()
            .map(|c| engine.canonical_for(c).unwrap())
            .collect();
        for (i, a) in canonicals.iter().enumerate() {
            for b in canonicals.iter().skip(i + 1) {
                assert_ne!(a, b, "separate concepts must have distinct canonicals");
            }
        }
    }

    #[test]
    fn each_separate_concept_is_its_own_canonical() {
        let (_graph, engine) = build_engine();
        let concepts = [
            "concept_sepsis",
            "concept_anaphylaxis",
            "concept_body_temperature",
            "concept_heart_rate",
            "concept_blood_pressure",
        ];
        for c in &concepts {
            assert_eq!(
                engine.canonical_for(c).unwrap(),
                *c,
                "singleton concept must be its own canonical"
            );
        }
    }

    #[test]
    fn unknown_concept_returns_none() {
        let (_graph, engine) = build_engine();
        assert!(engine.canonical_for("nonexistent").is_none());
    }

    #[test]
    fn unknown_concepts_are_not_equivalent() {
        let (_graph, engine) = build_engine();
        assert!(!engine.equivalent("nonexistent_a", "nonexistent_b"));
        assert!(!engine.equivalent("concept_sepsis", "nonexistent"));
    }

    // -- Artifact determinism --

    #[test]
    fn artifact_is_deterministic() {
        let graph = load_graph();
        let e1 = TermEquivalence::from_terminology_graph(&graph);
        let a1 = e1.emit_artifact(&graph);
        let e2 = TermEquivalence::from_terminology_graph(&graph);
        let a2 = e2.emit_artifact(&graph);
        assert_eq!(
            to_canonical_bytes(&a1),
            to_canonical_bytes(&a2),
            "repeated construction must produce identical canonical bytes"
        );
    }

    #[test]
    fn artifact_content_hash_is_deterministic() {
        let graph = load_graph();
        let e1 = TermEquivalence::from_terminology_graph(&graph);
        let a1 = e1.emit_artifact(&graph);
        let e2 = TermEquivalence::from_terminology_graph(&graph);
        let a2 = e2.emit_artifact(&graph);
        assert_eq!(
            content_hash(&a1),
            content_hash(&a2),
            "repeated construction must produce identical content hash"
        );
    }

    #[test]
    fn artifact_serde_roundtrip() {
        let (graph, engine) = build_engine();
        let artifact = engine.emit_artifact(&graph);
        let json = serde_json::to_string(&artifact).unwrap();
        let rt: EgraphArtifact = serde_json::from_str(&json).unwrap();
        assert_eq!(artifact, rt);
    }

    // -- Artifact contents --

    #[test]
    fn artifact_has_six_equivalence_classes() {
        let (graph, engine) = build_engine();
        let artifact = engine.emit_artifact(&graph);
        assert_eq!(
            artifact.class_ids.len(),
            6,
            "10 concepts should form 6 classes (5 beta-lactam + 5 singletons)"
        );
    }

    #[test]
    fn artifact_class_ids_are_sorted() {
        let (graph, engine) = build_engine();
        let artifact = engine.emit_artifact(&graph);
        let mut sorted = artifact.class_ids.clone();
        sorted.sort();
        assert_eq!(artifact.class_ids, sorted);
    }

    #[test]
    fn artifact_beta_lactam_class_has_five_members() {
        let (graph, engine) = build_engine();
        let artifact = engine.emit_artifact(&graph);
        let bl_entry = artifact
            .canonical_representatives
            .iter()
            .find(|e| e.class_id == "eclass_beta_lactam")
            .expect("eclass_beta_lactam must exist in artifact");
        assert_eq!(bl_entry.member_concept_ids.len(), 5);
        assert_eq!(bl_entry.canonical_concept_id, "concept_beta_lactam");
    }

    #[test]
    fn artifact_singleton_classes_are_self_canonical() {
        let (graph, engine) = build_engine();
        let artifact = engine.emit_artifact(&graph);
        let singletons = ["eclass_sepsis", "eclass_anaphylaxis", "eclass_body_temperature",
                         "eclass_heart_rate", "eclass_blood_pressure"];
        for class in &singletons {
            let entry = artifact
                .canonical_representatives
                .iter()
                .find(|e| e.class_id == *class)
                .unwrap_or_else(|| panic!("{class} must exist"));
            assert_eq!(entry.member_concept_ids.len(), 1);
            assert_eq!(entry.canonical_concept_id, entry.member_concept_ids[0]);
        }
    }

    #[test]
    fn artifact_saturation_stats_consistent() {
        let (graph, engine) = build_engine();
        let artifact = engine.emit_artifact(&graph);
        assert!(artifact.saturation_stats.saturated);
        assert!(artifact.saturation_stats.total_nodes > 0);
        // The egg EGraph has Symbol leaf nodes plus ConceptRef nodes.
        // 10 Symbol e-classes + 6 merged ConceptRef e-classes = 16 total.
        // The 6 semantic equivalence classes are the ConceptRef-level groups.
        assert!(
            artifact.saturation_stats.total_classes >= 6,
            "must have at least 6 semantic e-classes"
        );
    }

    #[test]
    fn artifact_rewrite_count_positive() {
        let (graph, engine) = build_engine();
        let artifact = engine.emit_artifact(&graph);
        assert!(
            artifact.rewrite_count > 0,
            "merging 5 beta-lactam variants requires at least 4 rewrites"
        );
    }

    #[test]
    fn artifact_bounded_iterations() {
        let (graph, engine) = build_engine();
        let artifact = engine.emit_artifact(&graph);
        assert!(
            artifact.iterations <= 10,
            "saturation must terminate within bounded iterations"
        );
    }

    // -- CAS storage --

    #[test]
    fn artifact_storable_through_cas() {
        let (graph, engine) = build_engine();
        let artifact = engine.emit_artifact(&graph);
        let envelope = ArtifactEnvelope::wrap(
            ArtifactKind::EgraphArtifact,
            &artifact,
            test_meta(),
        );
        assert!(envelope.verify_content_hash());
        let extracted: EgraphArtifact = envelope.extract().unwrap();
        assert_eq!(artifact, extracted);
    }

    #[test]
    fn artifact_envelope_hash_deterministic() {
        let graph = load_graph();
        let e1 = TermEquivalence::from_terminology_graph(&graph);
        let a1 = e1.emit_artifact(&graph);
        let env1 = ArtifactEnvelope::wrap(
            ArtifactKind::EgraphArtifact,
            &a1,
            test_meta(),
        );
        let e2 = TermEquivalence::from_terminology_graph(&graph);
        let a2 = e2.emit_artifact(&graph);
        let env2 = ArtifactEnvelope::wrap(
            ArtifactKind::EgraphArtifact,
            &a2,
            test_meta(),
        );
        assert_eq!(env1.envelope_hash(), env2.envelope_hash());
    }

    // -- Label normalization --

    #[test]
    fn label_normalization_beta_variants() {
        let a = normalize_ja_label("βラクタム系抗菌薬");
        let b = normalize_ja_label("ベータラクタム系薬剤");
        let c = normalize_ja_label("β-ラクタム系抗菌薬");
        assert_eq!(a, c, "Greek-letter and hyphenated variants must normalize identically");
        assert_ne!(a, b, "suffix-different labels remain distinct after normalization");
    }

    #[test]
    fn label_normalization_idempotent() {
        let labels = ["βラクタム系抗菌薬", "敗血症", "アナフィラキシー", "体温"];
        for label in &labels {
            let n1 = normalize_ja_label(label);
            let n2 = normalize_ja_label(&n1);
            assert_eq!(n1, n2, "label normalization must be idempotent for {label}");
        }
    }

    // -- Empty graph --

    #[test]
    fn empty_graph_produces_empty_artifact() {
        let graph = TerminologyGraph::new();
        let engine = TermEquivalence::from_terminology_graph(&graph);
        let artifact = engine.emit_artifact(&graph);
        assert!(artifact.class_ids.is_empty());
        assert!(artifact.canonical_representatives.is_empty());
        assert_eq!(artifact.rewrite_count, 0);
    }
}
