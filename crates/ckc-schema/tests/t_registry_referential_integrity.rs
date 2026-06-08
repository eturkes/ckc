//! Gate `T-Registry-Referential-Integrity` (§11.3 M0.0.3, completed by
//! M0.0.3.4.7). [`check_registry_referential_integrity`] is clean over the
//! real SPEC plus the built v0 registry (§1.1 step-5 `ok`), and rejects a
//! representative perturbation of each checked surface: a dropped
//! collection bound (step 4), a wrong schema role (§1.2), a duplicate
//! registry entry (steps 1-2), an unmapped payload (§3.2), and a missing
//! local-bound dispatch (§1.1 line-87 rule).

use ckc_core::scalar::Id;
use ckc_schema::bounds::SchemaCollectionBound;
use ckc_schema::build::build_v0_registry;
use ckc_schema::check::{CheckReport, check_registry_referential_integrity};
use ckc_schema::registry::{SchemaEntry, SchemaRole};
use ckc_schema::spec::parse_spec;

fn spec_text() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../SPEC.md")).unwrap()
}

fn messages(report: &CheckReport) -> Vec<&str> {
    report.issues.iter().map(|i| i.message.as_str()).collect()
}

fn rejects(report: &CheckReport, needle: &str) {
    assert!(
        messages(report).contains(&needle),
        "expected `{needle}` in {:#?}",
        report.issues
    );
}

/// Locates a SchemaCollectionBound by schema_id and `/`-joined path.
fn path_is(b: &SchemaCollectionBound, schema: &str, segments: &[&str]) -> bool {
    b.schema_id.as_str() == schema
        && b.path
            .segments()
            .iter()
            .map(Id::as_str)
            .eq(segments.iter().copied())
}

/// Replaces a schema entry's role and returns the mutated registry clone.
fn reroled(
    registry: &ckc_schema::registry::SchemaRegistry,
    schema_id: &str,
    role: SchemaRole,
) -> ckc_schema::registry::SchemaRegistry {
    let mut r = registry.clone();
    let e = r
        .schema_entries
        .iter()
        .find(|e| e.schema_id.as_str() == schema_id)
        .cloned()
        .unwrap();
    r.schema_entries.remove(&e);
    r.schema_entries.insert(SchemaEntry {
        schema_role: role,
        ..e
    });
    r
}

/// The real SPEC and the built v0 registry+manifest resolve clean.
#[test]
fn real_spec_and_registry_clean() {
    let text = spec_text();
    let decls = parse_spec(&text).decls;
    let (registry, manifest) = build_v0_registry(&decls, &text);
    let report = check_registry_referential_integrity(&text, &registry, &manifest);
    assert!(report.is_clean(), "issues: {:#?}", report.issues);
}

/// Step 4: dropping a bound row leaves its collection field uncovered.
#[test]
fn dropped_bound_row_rejects() {
    let text = spec_text();
    let decls = parse_spec(&text).decls;
    let (registry, mut manifest) = build_v0_registry(&decls, &text);
    let target = manifest
        .schema_collection_bounds
        .iter()
        .find(|b| path_is(b, "schema_registry", &["schema_entries"]))
        .cloned()
        .unwrap();
    assert!(manifest.schema_collection_bounds.remove(&target));
    let report = check_registry_referential_integrity(&text, &registry, &manifest);
    rejects(
        &report,
        "collection `schema_registry/schema_entries` has no SchemaCollectionBound row",
    );
}

/// §1.2 role rule: a no-support schema flipped to `semantic` rejects.
#[test]
fn wrong_role_rejects() {
    let text = spec_text();
    let decls = parse_spec(&text).decls;
    let (registry, manifest) = build_v0_registry(&decls, &text);
    let report = check_registry_referential_integrity(
        &text,
        &reroled(&registry, "schema_registry", SchemaRole::Semantic),
        &manifest,
    );
    rejects(
        &report,
        "schema `schema_registry` has schema_role `semantic` with neither source-support \
         field nor registered alias",
    );
}

/// Steps 1-2: a second SchemaEntry for an existing schema_id is a
/// divergent redeclaration (distinct set member via a new schema_version).
#[test]
fn duplicate_entry_rejects() {
    let text = spec_text();
    let decls = parse_spec(&text).decls;
    let (registry, manifest) = build_v0_registry(&decls, &text);
    let mut dup = registry.clone();
    let e = dup
        .schema_entries
        .iter()
        .find(|e| e.schema_id.as_str() == "schema_registry")
        .cloned()
        .unwrap();
    assert!(dup.schema_entries.insert(SchemaEntry {
        schema_version: Id::new("v1").unwrap(),
        ..e
    }));
    let report = check_registry_referential_integrity(&text, &dup, &manifest);
    rejects(
        &report,
        "registry declares schema_id `schema_registry` in multiple SchemaEntry rows",
    );
}

/// §3.2: dequalifying a payload's only stage-table mention to prose drops
/// its producer mapping.
#[test]
fn unmapped_payload_rejects() {
    let text = spec_text().replace("| obs_pattern | PatternObs", "| obs_pattern | patterns");
    let decls = parse_spec(&text).decls;
    let (registry, manifest) = build_v0_registry(&decls, &text);
    let report = check_registry_referential_integrity(&text, &registry, &manifest);
    rejects(
        &report,
        "payload `PatternObs` names no §3.2 producing operation or control emission",
    );
}

/// §1.1 line-87 rule: renaming the §6.2 collect dispatch leaves the
/// CollectBound local bound object without a named overflow dispatch.
#[test]
fn missing_local_bound_dispatch_rejects() {
    let text = spec_text().replace("collect_bound_overflow", "collect_overflow");
    let decls = parse_spec(&text).decls;
    let (registry, manifest) = build_v0_registry(&decls, &text);
    let report = check_registry_referential_integrity(&text, &registry, &manifest);
    rejects(
        &report,
        "local bound object `CollectBound` names no overflow dispatch `collect_bound_overflow`",
    );
}
