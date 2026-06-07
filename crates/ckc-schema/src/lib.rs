//! ckc-schema: SPEC §1.1 schema authority. Registry and bound types (.1),
//! the SPEC.md declaration extractor (.2), the symbol table + resolution
//! checker (.3), and the §3.1 type-graph walker (.4.1) are in place; the
//! v0 registry build (.4.2-.4.5) completes
//! `T-Registry-Referential-Integrity`.

pub mod bounds;
pub mod build;
pub mod check;
pub mod registry;
pub mod spec;
pub mod symtab;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use ckc_core::canon::{canonical_payload_bytes, from_canonical_bytes};
    use ckc_core::policy::StringPolicy;
    use ckc_core::scalar::{FeaturePath, Hash, Id, UInt};

    use crate::bounds::{BoundOverflowDisposition, SchemaBoundManifest, SchemaCollectionBound};
    use crate::registry::{
        SchemaEntry, SchemaRegistry, SchemaRole, SourceSupportAlias, SourceSupportAliasKind,
        StringPolicyBinding,
    };

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn path(segments: &[&str]) -> FeaturePath {
        FeaturePath::new(segments.iter().map(|s| id(s)).collect())
    }

    /// One composed registry+bounds strict roundtrip; the bound-manifest
    /// hash exercises the §1.2 `*_hash` convention compositionally.
    #[test]
    fn composed_registry_and_bounds_roundtrip() {
        let bounds = SchemaBoundManifest {
            manifest_id: id("schema-bound-manifest-v0"),
            schema_collection_bounds: BTreeSet::from([SchemaCollectionBound {
                schema_id: id("schema_registry"),
                path: path(&["schema_entries"]),
                max_items: UInt::from(4096u64),
                overflow_disposition: BoundOverflowDisposition::RejectWithDiagnostic,
            }]),
        };
        let bounds_bytes = canonical_payload_bytes(&bounds).unwrap();
        assert_eq!(
            from_canonical_bytes::<SchemaBoundManifest>(&bounds_bytes).unwrap(),
            bounds
        );

        let registry = SchemaRegistry {
            registry_id: id("ckc-schema-registry"),
            registry_version: id("v0"),
            spec_contract_hash: Hash::of_bytes(b"spec"),
            rust_type_manifest_hash: Hash::of_bytes(b"rust-manifest"),
            generated_json_schema_manifest_hash: Hash::of_bytes(b"json-manifest"),
            canonicalization_policy_hash: Hash::of_bytes(b"canonicalization"),
            schema_bound_manifest_hash: Hash::of_bytes(&bounds_bytes),
            schema_entries: BTreeSet::from([SchemaEntry {
                schema_id: id("unicode_policy_manifest"),
                schema_version: id("v0"),
                schema_role: SchemaRole::SchemaControl,
                rust_type_hash: Hash::of_bytes(b"rust"),
                generated_json_schema_hash: Hash::of_bytes(b"json"),
                tagged_union_alternatives_hash: None,
            }]),
            string_policy_bindings: BTreeSet::from([StringPolicyBinding {
                schema_id: id("unicode_policy_manifest"),
                path: path(&["unicode_version"]),
                policy: StringPolicy::IdentifierAscii,
                dependent_policy_field: None,
            }]),
            source_support_aliases: BTreeSet::from([SourceSupportAlias {
                schema_id: id("residual"),
                path: path(&["source_regions"]),
                alias_kind: SourceSupportAliasKind::RegionSet,
            }]),
        };
        let registry_bytes = canonical_payload_bytes(&registry).unwrap();
        assert_eq!(
            from_canonical_bytes::<SchemaRegistry>(&registry_bytes).unwrap(),
            registry
        );
    }
}
