//! SPEC §1.1 `SchemaRegistry` rows; §1.2 source-support alias kinds.
//!
//! Types only (M0.0.3.1): the spec extractor, symbol-table resolution, and
//! v0 build that complete `T-Registry-Referential-Integrity` land with
//! M0.0.3.2-4.

use std::collections::BTreeSet;

use ckc_core::policy::StringPolicy;
use ckc_core::scalar::{FeaturePath, Hash, Id};
use ckc_core::{bare_enum, canonical_record};
use serde::Deserialize;

bare_enum! {
    /// §1.1 `E SchemaRole`.
    #[derive(PartialOrd, Ord)]
    pub enum SchemaRole: "schema_role" {
        Semantic = "semantic",
        SourceOnly = "source_only",
        SchemaControl = "schema_control",
        ReplayControl = "replay_control",
        EnvironmentControl = "environment_control",
        AdmissionControl = "admission_control",
        EvidenceDiscovery = "evidence_discovery",
        ViewOnly = "view_only",
        ProofStructure = "proof_structure",
    }
}

bare_enum! {
    /// §1.1 `E SourceSupportAliasKind`; projection semantics and the fixed
    /// field-name defaults live in the §1.2 alias table.
    #[derive(PartialOrd, Ord)]
    pub enum SourceSupportAliasKind: "source_support_alias_kind" {
        SingletonRegion = "singleton_region",
        RegionSet = "region_set",
        InheritedSubject = "inherited_subject",
        InheritedInput = "inherited_input",
        ClosedRegionMembers = "closed_region_members",
    }
}

/// §1.1 `S SchemaEntry`. Hash fields follow the §1.2 conventions;
/// `tagged_union_alternatives_hash` is present exactly for tagged-union
/// schemas.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaEntry {
    pub schema_id: Id,
    pub schema_version: Id,
    pub schema_role: SchemaRole,
    pub rust_type_hash: Hash,
    pub generated_json_schema_hash: Hash,
    pub tagged_union_alternatives_hash: Option<Hash>,
}

canonical_record!(SchemaEntry: "schema_entry",
    fields { schema_id, schema_version, schema_role, rust_type_hash, generated_json_schema_hash },
    optional { tagged_union_alternatives_hash });

/// §1.1 `S StringPolicyBinding`: one row per `Text<P>` field site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StringPolicyBinding {
    pub schema_id: Id,
    pub path: FeaturePath,
    pub policy: StringPolicy,
    pub dependent_policy_field: Option<FeaturePath>,
}

canonical_record!(StringPolicyBinding: "string_policy_binding",
    fields { schema_id, path, policy },
    optional { dependent_policy_field });

/// §1.1 `S SourceSupportAlias`: registers `path` as the canonical
/// source-support projection for `schema_id` (§1.2).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSupportAlias {
    pub schema_id: Id,
    pub path: FeaturePath,
    pub alias_kind: SourceSupportAliasKind,
}

canonical_record!(SourceSupportAlias: "source_support_alias",
    fields { schema_id, path, alias_kind });

/// §1.1 `S SchemaRegistry`: the executable schema authority for accepted
/// artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaRegistry {
    pub registry_id: Id,
    pub registry_version: Id,
    pub spec_contract_hash: Hash,
    pub rust_type_manifest_hash: Hash,
    pub generated_json_schema_manifest_hash: Hash,
    pub canonicalization_policy_hash: Hash,
    pub schema_bound_manifest_hash: Hash,
    pub schema_entries: BTreeSet<SchemaEntry>,
    pub string_policy_bindings: BTreeSet<StringPolicyBinding>,
    pub source_support_aliases: BTreeSet<SourceSupportAlias>,
}

canonical_record!(SchemaRegistry: "schema_registry",
    fields { registry_id, registry_version, spec_contract_hash, rust_type_manifest_hash, generated_json_schema_manifest_hash, canonicalization_policy_hash, schema_bound_manifest_hash },
    sets { schema_entries, string_policy_bindings, source_support_aliases });

#[cfg(test)]
mod tests {
    use ckc_core::canon::{canonical_payload_bytes, from_canonical_bytes};

    use super::*;

    /// Optional-field omission (§1.3) plus golden member order and bare-enum
    /// string encoding for one entry.
    #[test]
    fn schema_entry_optional_field_omission() {
        let mut entry = SchemaEntry {
            schema_id: Id::new("unicode_policy_manifest").unwrap(),
            schema_version: Id::new("v0").unwrap(),
            schema_role: SchemaRole::SchemaControl,
            rust_type_hash: Hash::of_bytes(b"rust"),
            generated_json_schema_hash: Hash::of_bytes(b"json"),
            tagged_union_alternatives_hash: None,
        };
        let omitted = canonical_payload_bytes(&entry).unwrap();
        assert_eq!(
            String::from_utf8(omitted.clone()).unwrap(),
            format!(
                "{{\"generated_json_schema_hash\":\"{}\",\"rust_type_hash\":\"{}\",\
                 \"schema_id\":\"unicode_policy_manifest\",\"schema_role\":\"schema_control\",\
                 \"schema_version\":\"v0\"}}",
                Hash::of_bytes(b"json"),
                Hash::of_bytes(b"rust"),
            )
        );
        assert_eq!(
            from_canonical_bytes::<SchemaEntry>(&omitted).unwrap(),
            entry
        );

        entry.tagged_union_alternatives_hash = Some(Hash::of_bytes(b"alts"));
        let present = canonical_payload_bytes(&entry).unwrap();
        assert!(
            String::from_utf8(present.clone())
                .unwrap()
                .contains(&format!(
                    "\"tagged_union_alternatives_hash\":\"{}\"",
                    Hash::of_bytes(b"alts")
                ))
        );
        assert_eq!(
            from_canonical_bytes::<SchemaEntry>(&present).unwrap(),
            entry
        );
    }
}
