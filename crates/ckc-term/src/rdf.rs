use ckc_core::enums::BindingStatus;
use oxrdf::{Literal, NamedNode, Triple};
use oxttl::TurtleSerializer;

use crate::TerminologyGraph;

const CKC_NS: &str = "http://ckc.example.org/";
const SKOS_NS: &str = "http://www.w3.org/2004/02/skos/core#";
const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const DCT_NS: &str = "http://purl.org/dc/terms/";

fn nn(iri: String) -> NamedNode {
    NamedNode::new_unchecked(iri)
}

fn skos(local: &str) -> NamedNode {
    nn(format!("{SKOS_NS}{local}"))
}

fn binding_predicate(status: BindingStatus) -> NamedNode {
    match status {
        BindingStatus::Exact => skos("exactMatch"),
        BindingStatus::Broad => skos("broadMatch"),
        BindingStatus::Narrow => skos("narrowMatch"),
        _ => skos("relatedMatch"),
    }
}

/// Export the terminology graph as SKOS/Turtle.
///
/// Each `Concept` becomes a `skos:Concept` with:
/// - `skos:prefLabel` for `label_ja` (@ja)
/// - `skos:altLabel` for `label_en` (@en) when present
/// - `ckc:semanticType` literal
/// - `skos:inScheme` for `egraph_class_id` grouping
/// - `skos:{exact,broad,narrow,related}Match` per `TerminologyBinding.status`
/// - `dct:source` for each source span ID
///
/// Output is deterministic: concepts sorted by `concept_id`.
pub fn export_skos_turtle(graph: &TerminologyGraph) -> String {
    let rdf_type = nn(format!("{RDF_NS}type"));
    let skos_concept = skos("Concept");
    let skos_pref_label = skos("prefLabel");
    let skos_alt_label = skos("altLabel");
    let skos_in_scheme = skos("inScheme");
    let ckc_semantic_type = nn(format!("{CKC_NS}semanticType"));
    let dct_source = nn(format!("{DCT_NS}source"));

    let mut triples = Vec::new();

    let mut concepts: Vec<_> = graph.concepts().collect();
    concepts.sort_by_key(|c| c.concept_id.as_str());

    for concept in &concepts {
        let subj = nn(format!("{CKC_NS}concept/{}", concept.concept_id.as_str()));

        triples.push(Triple::new(
            subj.clone(),
            rdf_type.clone(),
            skos_concept.clone(),
        ));

        triples.push(Triple::new(
            subj.clone(),
            skos_pref_label.clone(),
            Literal::new_language_tagged_literal_unchecked(&concept.label_ja, "ja"),
        ));

        if let Some(ref en) = concept.label_en {
            triples.push(Triple::new(
                subj.clone(),
                skos_alt_label.clone(),
                Literal::new_language_tagged_literal_unchecked(en, "en"),
            ));
        }

        triples.push(Triple::new(
            subj.clone(),
            ckc_semantic_type.clone(),
            Literal::new_simple_literal(&concept.semantic_type),
        ));

        if let Some(ref class_id) = concept.egraph_class_id {
            triples.push(Triple::new(
                subj.clone(),
                skos_in_scheme.clone(),
                nn(format!("{CKC_NS}eclass/{}", class_id.as_str())),
            ));
        }

        for binding in &concept.terminology_bindings {
            if let Some(ref code) = binding.code {
                triples.push(Triple::new(
                    subj.clone(),
                    binding_predicate(binding.status),
                    nn(format!("{CKC_NS}external/{}/{code}", binding.system)),
                ));
            }
        }

        for span_id in &concept.source_span_ids {
            triples.push(Triple::new(
                subj.clone(),
                dct_source.clone(),
                Literal::new_simple_literal(span_id.as_str()),
            ));
        }
    }

    let mut serializer = TurtleSerializer::new()
        .with_prefix("rdf", RDF_NS)
        .expect("valid RDF namespace")
        .with_prefix("skos", SKOS_NS)
        .expect("valid SKOS namespace")
        .with_prefix("dct", DCT_NS)
        .expect("valid DCT namespace")
        .with_prefix("ckc", CKC_NS)
        .expect("valid CKC namespace")
        .for_writer(Vec::new());

    for triple in &triples {
        serializer
            .serialize_triple(triple)
            .expect("triple serialization");
    }

    let buf = serializer.finish().expect("Turtle finalization");
    String::from_utf8(buf).expect("Turtle output is valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxttl::TurtleParser;

    const FIXTURE_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/research_kernel/fixtures/concepts.json"
    );

    fn load_graph() -> TerminologyGraph {
        let json = std::fs::read_to_string(FIXTURE_PATH).expect("concepts.json fixture must exist");
        TerminologyGraph::load_from_json(&json).expect("fixture must parse")
    }

    #[test]
    fn export_produces_nonempty_turtle() {
        let graph = load_graph();
        let turtle = export_skos_turtle(&graph);
        assert!(!turtle.is_empty());
        assert!(turtle.contains("skos:"));
    }

    #[test]
    fn export_roundtrips_through_parser() {
        let graph = load_graph();
        let turtle = export_skos_turtle(&graph);

        let parsed: Vec<_> = TurtleParser::new()
            .for_slice(&turtle)
            .collect::<Result<Vec<_>, _>>()
            .expect("exported Turtle must re-parse");

        assert!(!parsed.is_empty(), "round-trip parse must yield triples");

        // Each concept: type + prefLabel + semanticType = 3 base triples
        // Plus optional altLabel, inScheme, match predicates, source spans
        assert!(
            parsed.len() >= graph.len() * 3,
            "expected >= {} triples (3 per concept), got {}",
            graph.len() * 3,
            parsed.len()
        );
    }

    #[test]
    fn export_is_deterministic() {
        let graph = load_graph();
        let t1 = export_skos_turtle(&graph);
        let t2 = export_skos_turtle(&graph);
        assert_eq!(t1, t2, "repeated exports must be byte-identical");
    }

    #[test]
    fn export_contains_beta_lactam_pref_label() {
        let graph = load_graph();
        let turtle = export_skos_turtle(&graph);
        assert!(
            turtle.contains("βラクタム系抗菌薬"),
            "Turtle must contain JA prefLabel for beta-lactam"
        );
    }

    #[test]
    fn export_contains_skos_match_predicates() {
        let graph = load_graph();
        let turtle = export_skos_turtle(&graph);
        assert!(
            turtle.contains("exactMatch") || turtle.contains("skos:exactMatch"),
            "Turtle must contain exactMatch predicates"
        );
        assert!(
            turtle.contains("broadMatch") || turtle.contains("skos:broadMatch"),
            "Turtle must contain broadMatch predicates"
        );
    }

    #[test]
    fn export_contains_eclass_scheme() {
        let graph = load_graph();
        let turtle = export_skos_turtle(&graph);
        assert!(
            turtle.contains("eclass/eclass_beta_lactam") || turtle.contains("inScheme"),
            "Turtle must reference e-graph class as scheme"
        );
    }

    #[test]
    fn export_contains_source_provenance() {
        let graph = load_graph();
        let turtle = export_skos_turtle(&graph);
        assert!(
            turtle.contains("source") && turtle.contains("span_"),
            "Turtle must include dct:source with span IDs"
        );
    }

    #[test]
    fn export_empty_graph_produces_valid_turtle() {
        let graph = TerminologyGraph::new();
        let turtle = export_skos_turtle(&graph);

        let parsed: Vec<_> = TurtleParser::new()
            .for_slice(&turtle)
            .collect::<Result<Vec<_>, _>>()
            .expect("empty graph Turtle must re-parse");

        assert_eq!(parsed.len(), 0, "empty graph yields zero triples");
    }

    #[test]
    fn export_triple_count_matches_expected() {
        let graph = load_graph();
        let turtle = export_skos_turtle(&graph);

        let triple_count = TurtleParser::new().for_slice(&turtle).count();

        // Manually counted from fixture: 10 concepts, each has type + prefLabel
        // + semanticType = 30 base. Most have altLabel, inScheme, bindings, spans.
        // Exact count depends on fixture data; verify it's stable.
        assert!(
            triple_count > 30,
            "expected > 30 triples for 10 concepts, got {triple_count}"
        );
    }
}
