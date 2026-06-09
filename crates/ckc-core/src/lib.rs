//! ckc-core — durable typed core for the Clinical Knowledge Compiler (SPEC §5).
//!
//! Rust owns durable semantics: stable IDs, value types, enums, schema structs,
//! canonical JSON bytes, semantic hashes, and validation. This crate is built
//! up unit by unit; `core-ids` seeds it with the foundational value types of
//! SPEC §9: [`Id`], [`Hash`], and [`Rational`].
#![forbid(unsafe_code)]

mod id;

pub use id::{Hash, Id, Rational, RationalRepr, ValidationError};
