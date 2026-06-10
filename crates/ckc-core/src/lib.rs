//! ckc-core — durable typed core for the Clinical Knowledge Compiler (SPEC §3).
//!
//! Rust owns durable semantics: stable IDs, value types, enums, schema structs,
//! canonical JSON bytes, semantic hashes, and validation. The crate is built up
//! unit by unit; `core-ids` seeds it with the SPEC §4.1 value types [`Id`],
//! [`Hash`], and [`Rational`]; `core-strings` adds the SPEC §4.2
//! [`StringPolicy`] normalizers; `core-canon-writer` opens the SPEC §4.3
//! canonical JSON writer ([`Canonical`], [`canonical_payload_bytes`],
//! [`ObjectEmitter`]), `core-canon-collections` adds the array/set/map
//! rules ([`emit_array`], [`emit_set`], [`emit_map`], [`MapKey`]), and
//! `core-canon-unions` adds tagged-union emission ([`emit_union`]).
#![forbid(unsafe_code)]

mod canon;
mod id;
mod strings;

pub use canon::{
    CanonError, Canonical, MapKey, ObjectEmitter, canonical_payload_bytes, canonical_sort_key,
    emit_array, emit_int, emit_map, emit_set, emit_string, emit_string_policy, emit_union,
};
pub use id::{Hash, Id, Rational, RationalRepr, ValidationError};
pub use strings::StringPolicy;
