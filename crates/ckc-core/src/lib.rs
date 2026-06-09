//! ckc-core — durable typed core for the Clinical Knowledge Compiler (SPEC §5).
//!
//! Rust owns durable semantics: stable IDs, value types, enums, schema structs,
//! canonical JSON bytes, semantic hashes, and validation. This crate is built
//! up unit by unit; `core-ids` seeds it with the SPEC §9 value types [`Id`],
//! [`Hash`], and [`Rational`], and `core-strings` adds the SPEC §10
//! [`StringPolicy`] normalizers.
#![forbid(unsafe_code)]

mod id;
mod strings;

pub use id::{Hash, Id, Rational, RationalRepr, ValidationError};
pub use strings::StringPolicy;
