use std::fs;

const CONCEPTS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/research_kernel/fixtures/concepts.json"
);
const RULES_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/research_kernel/fixtures/rules.json"
);
const TTL_OUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/research_kernel/fixtures/terminology.ttl"
);
const SHACL_OUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/research_kernel/fixtures/shacl_report.json"
);
const EGRAPH_OUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/research_kernel/fixtures/egraph_equivalence.json"
);

#[test]
#[ignore]
fn generate_substrate_fixtures() {
    let concepts_json = fs::read_to_string(CONCEPTS_PATH).unwrap();
    let graph = ckc_term::TerminologyGraph::load_from_json(&concepts_json).unwrap();
    let turtle = ckc_term::rdf::export_skos_turtle(&graph);
    fs::write(TTL_OUT, &turtle).unwrap();
    eprintln!("Wrote {TTL_OUT} ({} bytes)", turtle.len());

    // Canonical-JSON e-graph equivalence artifact. It carries no f64, so the
    // committed fixture is byte-stable and ckc-cli's substrate gate byte-compares
    // it (the .ttl path stays raw Turtle; the other JSON fixtures are pretty).
    let artifact =
        ckc_term::egraph::TermEquivalence::from_terminology_graph(&graph).emit_artifact(&graph);
    let egraph_bytes = ckc_core::canonical::to_canonical_bytes(&artifact);
    fs::write(EGRAPH_OUT, &egraph_bytes).unwrap();
    eprintln!("Wrote {EGRAPH_OUT} ({} bytes)", egraph_bytes.len());

    let rules_json = fs::read_to_string(RULES_PATH).unwrap();
    let rules: Vec<ckc_core::clinical::Rule> = serde_json::from_str(&rules_json).unwrap();
    let report = ckc_term::shacl::validate_rules(&rules);
    let report_json = serde_json::to_string_pretty(&report).unwrap();
    fs::write(SHACL_OUT, format!("{report_json}\n")).unwrap();
    eprintln!(
        "Wrote {SHACL_OUT} ({} bytes, conforms={})",
        report_json.len(),
        report.conforms
    );
}
