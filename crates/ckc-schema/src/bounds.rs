//! SPEC §1.1 `SchemaBoundManifest` rows: collection bounds and overflow
//! dispositions.
//!
//! Types only (M0.0.3.1): `HandleBoundOverflow` emission (§1.1 bound
//! overflow convention) lands with its first §8.7 consumer.

use std::collections::BTreeSet;

use ckc_core::scalar::{FeaturePath, Hash, Id, UInt};
use ckc_core::{bare_enum, canonical_record};
use serde::Deserialize;

bare_enum! {
    /// §1.1 `E BoundOverflowDisposition`; the per-disposition emission
    /// dispatch is the §1.1 bound-overflow convention table.
    #[derive(PartialOrd, Ord)]
    pub enum BoundOverflowDisposition: "bound_overflow_disposition" {
        RejectWithDiagnostic = "reject_with_diagnostic",
        EmitResidual = "emit_residual",
        EmitAmbiguity = "emit_ambiguity",
        EmitIncoherence = "emit_incoherence",
    }
}

/// §1.1 `S SchemaCollectionBound`: one row per Set/List/Map field in every
/// accepted schema (§1.1 step 4, enum-domain scalar maps exempt).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaCollectionBound {
    pub schema_id: Id,
    pub path: FeaturePath,
    pub max_items: UInt,
    pub overflow_disposition: BoundOverflowDisposition,
}

canonical_record!(SchemaCollectionBound: "schema_collection_bound",
    fields { schema_id, path, max_items, overflow_disposition });

/// §1.1 `S SchemaBoundManifest`; `SchemaRegistry.schema_bound_manifest_hash`
/// points at it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaBoundManifest {
    pub manifest_id: Id,
    pub schema_collection_bounds: BTreeSet<SchemaCollectionBound>,
    pub generator_static_bound_policy_hash: Hash,
    pub closure_bound_policy_hash: Hash,
    pub parser_bound_policy_hash: Hash,
}

canonical_record!(SchemaBoundManifest: "schema_bound_manifest",
    fields { manifest_id, generator_static_bound_policy_hash, closure_bound_policy_hash, parser_bound_policy_hash },
    sets { schema_collection_bounds });
