//! Gate for task 0.12.1: the JSON Schema for the bilingual [`Report`] model
//! (SPEC 21, 23) stays byte-stable against the committed
//! `schemas/report.schema.json`. Mirrors the `check_schema` + `#[ignore]
//! regenerate` idiom of `ckc-verify/tests/manifest.rs`; the value-level
//! `report.json` golden arrives with report assembly in task 0.12.4.

use std::path::PathBuf;

use ckc_report::Report;

fn schema_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("schemas")
}

fn check_schema<T: schemars::JsonSchema>(stem: &str) {
    let schema = schemars::schema_for!(T);
    let json = serde_json::to_string_pretty(&schema).unwrap() + "\n";
    let path = schema_dir().join(format!("{stem}.schema.json"));
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read schema {}: {e}", path.display()));
    assert!(json == golden, "schema mismatch for {stem}");
}

#[test]
fn schema() {
    check_schema::<Report>("report");
}

// Regeneration: `cargo test -p ckc-report --test schema -- --ignored`
#[test]
#[ignore]
fn regenerate() {
    let schema = schemars::schema_for!(Report);
    std::fs::write(
        schema_dir().join("report.schema.json"),
        serde_json::to_string_pretty(&schema).unwrap() + "\n",
    )
    .unwrap();
}
