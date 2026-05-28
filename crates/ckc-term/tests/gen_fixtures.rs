use std::fs;

const CONCEPTS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/toy_research_kernel/fixtures/concepts.json"
);
const RULES_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/toy_research_kernel/fixtures/rules.json"
);
const TTL_OUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/toy_research_kernel/fixtures/terminology.ttl"
);
const SHACL_OUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/toy_research_kernel/fixtures/shacl_report.json"
);

#[test]
#[ignore]
fn generate_rdf_and_shacl_fixtures() {
    let concepts_json = fs::read_to_string(CONCEPTS_PATH).unwrap();
    let graph = ckc_term::TerminologyGraph::load_from_json(&concepts_json).unwrap();
    let turtle = ckc_term::rdf::export_skos_turtle(&graph);
    fs::write(TTL_OUT, &turtle).unwrap();
    eprintln!("Wrote {TTL_OUT} ({} bytes)", turtle.len());

    let rules_json = fs::read_to_string(RULES_PATH).unwrap();
    let rules: Vec<ckc_core::clinical::Rule> = serde_json::from_str(&rules_json).unwrap();
    let report = ckc_term::shacl::validate_rules(&rules);
    let report_json = serde_json::to_string_pretty(&report).unwrap();
    fs::write(SHACL_OUT, format!("{report_json}\n")).unwrap();
    eprintln!("Wrote {SHACL_OUT} ({} bytes, conforms={})", report_json.len(), report.conforms);
}
