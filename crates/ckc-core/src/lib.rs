//! ckc-core — durable typed core for the Clinical Knowledge Compiler (SPEC §3).
//!
//! Rust owns durable semantics: stable IDs, value types, enums, schema structs,
//! canonical JSON bytes, semantic hashes, and validation. The crate is built up
//! unit by unit; `core-ids` seeds it with the SPEC §4.1 value types [`Id`],
//! [`Hash`], and [`Rational`]; `core-strings` adds the SPEC §4.2
//! [`StringPolicy`] normalizers; `core-canon-writer` opens the SPEC §4.3
//! canonical JSON writer ([`Canonical`], [`canonical_payload_bytes`],
//! [`ObjectEmitter`]), `core-canon-collections` adds the array/set/map
//! rules ([`emit_array`], [`emit_set`], [`emit_map`], [`MapKey`]),
//! `core-canon-unions` adds tagged-union emission ([`emit_union`]), and
//! `core-canon-reader` adds the strict canonical reader ([`read_canonical`],
//! [`CanonRead`], [`CanonReadError`]) that admits only the writer's bytes.
//! `core-canon-hash` then seals a payload into its content hash and pins the
//! policy version ([`content_hash`], [`hash_bytes`],
//! [`canonicalization_policy_hash`]). `core-enums-envelope.1` adds the SPEC
//! §4.4 fieldless enum family ([`Outcome`] severity-ordered) with the §7.4
//! [`DiagnosticRecord`] and the §4.4 [`TotalOperationResult`];
//! `core-enums-envelope.2` wraps payloads in the §4.4 [`ArtifactEnvelope`]
//! and frames §4.6 [`EventRecord`]s and diagnostics as canonical JSONL
//! ([`write_jsonl`], [`read_jsonl`]). `core-grounding` adds the SPEC §4.5
//! source-grounding layer ([`SourceGraph`] with its document, nodes, spans,
//! anchors, and regions, validated by [`SourceGraph::validate`]).
#![forbid(unsafe_code)]

mod canon;
mod enums;
mod envelope;
mod grounding;
mod hash;
mod id;
mod strings;

pub use canon::{
    CanonError, CanonRead, CanonReadError, Canonical, MapKey, ObjectEmitter, ObjectReader, Reader,
    canonical_payload_bytes, canonical_sort_key, emit_array, emit_int, emit_map, emit_set,
    emit_string, emit_string_policy, emit_union, read_array, read_canonical, read_int, read_map,
    read_set, read_string, read_string_policy, read_union,
};
pub use enums::{
    AttemptClassification, Authority, BindingStatus, ClaimTier, DiagnosticCode, DiagnosticRecord,
    Direction, Origin, Outcome, PromotionDecision, PromotionScope, ReviewClassification,
    TotalOperationResult,
};
pub use envelope::{
    ArtifactEnvelope, Effect, EnvelopeError, EventRecord, Producer, SCHEMA_VERSION, jsonl_line,
    read_jsonl, write_jsonl,
};
pub use grounding::{
    AnchorKind, DataClass, GroundingError, NodeKind, Provenance, RefKind, SourceAnchor,
    SourceDocument, SourceGraph, SourceNode, SourceRegion, SourceSpan,
};
pub use hash::{CanonicalizationPolicy, canonicalization_policy_hash, content_hash, hash_bytes};
pub use id::{Hash, Id, Rational, RationalRepr, ValidationError};
pub use strings::StringPolicy;
