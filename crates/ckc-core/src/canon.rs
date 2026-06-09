//! Canonical JSON payload bytes — writer core (SPEC §10).
//!
//! [`canonical_payload_bytes`] is the single authority that turns a typed value
//! into the deterministic UTF-8 bytes hashed into an artifact's `content_hash`.
//! `core-canon-writer` delivered the scalar + object writer core and
//! `core-canon-collections` adds the array, set, and map rules; tagged unions,
//! strict reading, and the content hash itself are layered on by later units.
//!
//! ```text
//! object   field names sorted by UTF-8 byte order; duplicate names rejected
//! optional absent field omitted (the canonical form has no null)
//! string   declared StringPolicy applied, then JSON-escaped (UTF-8 passthrough)
//! integer  decimal string, e.g. "42" (bare JSON number tokens are never emitted)
//! rational {"den":"<den>","num":"<num>"} with den positive and parts reduced
//! array    elements in given semantic order
//! set      elements sorted by canonical_sort_key, byte-identical dups collapsed
//! map      identifier_ascii keys -> sorted object; else sorted {"key","value"} pairs
//! ```
//!
//! Canonical string escaping is minimal and fixed: escape U+0022 `"` as `\"`,
//! U+005C `\` as `\\`, and U+0000..U+001F as lowercase `\u00xx`; every other
//! scalar passes through as its raw UTF-8 bytes (shorthand escapes such as `\n`
//! are non-canonical). One representation per string.

use std::borrow::Cow;
use std::fmt;

use num_bigint::BigInt;

use crate::{Hash, Id, Rational, StringPolicy, ValidationError};

/// Failure while emitting canonical bytes (SPEC §10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonError {
    /// Two object members shared a field name; an object holds at most one value
    /// per name, so the canonical form would be undefined.
    DuplicateField(String),
    /// Two map entries shared a key; a map binds each key once, so the canonical
    /// value would be ambiguous. Carries the object member name (identifier_ascii
    /// form) or the key's canonical bytes (pair-array form).
    DuplicateMapKey(String),
    /// A string field failed its declared [`StringPolicy`]; only
    /// `identifier_ascii` can reject its input.
    Policy(ValidationError),
}

impl fmt::Display for CanonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonError::DuplicateField(name) => write!(f, "duplicate object field: {name:?}"),
            CanonError::DuplicateMapKey(key) => write!(f, "duplicate map key: {key:?}"),
            CanonError::Policy(e) => write!(f, "string policy: {e}"),
        }
    }
}

impl std::error::Error for CanonError {}

impl From<ValidationError> for CanonError {
    fn from(e: ValidationError) -> Self {
        CanonError::Policy(e)
    }
}

/// Type-guided canonical emission (SPEC §10): a value appends its canonical
/// UTF-8 bytes to `out`. Composite values build their fields through
/// [`ObjectEmitter`].
pub trait Canonical {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError>;
}

/// SPEC §10 `canonical_payload_bytes`: the deterministic bytes a later unit
/// hashes into an artifact's `content_hash`.
pub fn canonical_payload_bytes<T: Canonical>(value: &T) -> Result<Vec<u8>, CanonError> {
    let mut out = Vec::new();
    value.emit_canonical(&mut out)?;
    Ok(out)
}

/// Append a canonical JSON string: `"…"` with the module header's minimal fixed
/// escaping. The caller supplies already-policy-normalized text; field names and
/// identifiers (ASCII, escape-free) pass through unchanged.
pub fn emit_string(out: &mut Vec<u8>, s: &str) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.push(b'"');
    for &b in s.as_bytes() {
        match b {
            b'"' => out.extend_from_slice(b"\\\""),
            b'\\' => out.extend_from_slice(b"\\\\"),
            0x00..=0x1f => out.extend_from_slice(&[
                b'\\',
                b'u',
                b'0',
                b'0',
                HEX[usize::from(b >> 4)],
                HEX[usize::from(b & 0xf)],
            ]),
            _ => out.push(b),
        }
    }
    out.push(b'"');
}

/// Append an integer as its canonical decimal string, e.g. `"-42"` (SPEC §10:
/// integers are decimal strings; bare JSON number tokens are never emitted).
pub fn emit_int(out: &mut Vec<u8>, value: &BigInt) {
    emit_string(out, &value.to_string());
}

/// Append a string normalized under `policy`, then JSON-escaped. Fails only when
/// `policy` is [`StringPolicy::IdentifierAscii`] and `raw` violates its grammar.
pub fn emit_string_policy(
    out: &mut Vec<u8>,
    policy: StringPolicy,
    raw: &str,
) -> Result<(), CanonError> {
    let normalized = policy.normalize(raw)?;
    emit_string(out, &normalized);
    Ok(())
}

/// Builds a canonical object: members are buffered, then on [`finish`] sorted by
/// UTF-8 field-name bytes, checked for duplicate names, and written as
/// `{"name":value,…}`. Optional fields are omitted by not adding them (see
/// [`optional`]); the canonical form has no null.
///
/// [`finish`]: ObjectEmitter::finish
/// [`optional`]: ObjectEmitter::optional
#[derive(Default)]
pub struct ObjectEmitter {
    members: Vec<(String, Vec<u8>)>,
}

impl ObjectEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Buffer one required member; `emit` writes the member's value bytes.
    pub fn member(
        &mut self,
        name: &str,
        emit: impl FnOnce(&mut Vec<u8>) -> Result<(), CanonError>,
    ) -> Result<(), CanonError> {
        let mut value = Vec::new();
        emit(&mut value)?;
        self.members.push((name.to_owned(), value));
        Ok(())
    }

    /// Buffer an optional member, omitting it entirely when `value` is `None`
    /// (SPEC §10 optional-omit; an absent field is never emitted as null).
    pub fn optional<T>(
        &mut self,
        name: &str,
        value: Option<T>,
        emit: impl FnOnce(&mut Vec<u8>, T) -> Result<(), CanonError>,
    ) -> Result<(), CanonError> {
        match value {
            Some(v) => self.member(name, |b| emit(b, v)),
            None => Ok(()),
        }
    }

    /// Sort buffered members by field-name bytes, reject duplicate names with
    /// [`CanonError::DuplicateField`], and append the object to `out`.
    pub fn finish(self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        self.finish_with(out, CanonError::DuplicateField)
    }

    /// [`finish`](Self::finish) with a caller-chosen duplicate-name error, so a
    /// map in object form reports [`CanonError::DuplicateMapKey`] for a repeated
    /// key while a record reports a duplicate field.
    fn finish_with(
        mut self,
        out: &mut Vec<u8>,
        duplicate: impl FnOnce(String) -> CanonError,
    ) -> Result<(), CanonError> {
        self.members.sort_by(|a, b| a.0.cmp(&b.0));
        if let Some(w) = self.members.windows(2).find(|w| w[0].0 == w[1].0) {
            return Err(duplicate(w[0].0.clone()));
        }
        out.push(b'{');
        for (i, (name, value)) in self.members.iter().enumerate() {
            if i > 0 {
                out.push(b',');
            }
            emit_string(out, name);
            out.push(b':');
            out.extend_from_slice(value);
        }
        out.push(b'}');
        Ok(())
    }
}

impl Canonical for Id {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string(out, self.as_str());
        Ok(())
    }
}

impl Canonical for Hash {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string(out, self.as_str());
        Ok(())
    }
}

impl Canonical for BigInt {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_int(out, self);
        Ok(())
    }
}

/// `{"den":"<den>","num":"<num>"}`: a two-field object whose names are already in
/// byte order (`den` < `num`), each part a canonical integer string. The
/// denominator is positive and the fraction reduced by [`Rational`]'s
/// construction invariants.
impl Canonical for Rational {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        out.extend_from_slice(b"{\"den\":");
        emit_int(out, self.denom());
        out.extend_from_slice(b",\"num\":");
        emit_int(out, self.numer());
        out.push(b'}');
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Canonical collections (SPEC §10)
// ---------------------------------------------------------------------------

/// SPEC §10 `canonical_sort_key`: the total order [`emit_set`] and pair-array
/// [`emit_map`] sort by. For spec03 it is exactly the element's
/// [`canonical_payload_bytes`]; within a Rust-homogeneous collection those bytes
/// are injective, so their byte-lexicographic order is total and deterministic,
/// and no separate type tag is needed (every element shares one type).
pub fn canonical_sort_key<T: Canonical>(x: &T) -> Result<Vec<u8>, CanonError> {
    canonical_payload_bytes(x)
}

/// Frame `items` as a canonical JSON array `[…]`, writing each through `emit`
/// with `,` separators. Shared core of [`emit_array`], [`emit_set`], and the
/// pair-array map form.
fn emit_bracketed<T>(
    out: &mut Vec<u8>,
    items: impl IntoIterator<Item = T>,
    mut emit: impl FnMut(&mut Vec<u8>, T) -> Result<(), CanonError>,
) -> Result<(), CanonError> {
    out.push(b'[');
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        emit(out, item)?;
    }
    out.push(b']');
    Ok(())
}

/// SPEC §10 array: elements in their given semantic order, each emitted through
/// [`Canonical`]. Order is preserved verbatim; callers needing order-free set
/// semantics use [`emit_set`].
pub fn emit_array<'a, T: Canonical + 'a>(
    out: &mut Vec<u8>,
    items: impl IntoIterator<Item = &'a T>,
) -> Result<(), CanonError> {
    emit_bracketed(out, items, |b, item| item.emit_canonical(b))
}

/// SPEC §10 set: an array sorted by [`canonical_sort_key`] with byte-identical
/// duplicates collapsed (a set holds each element once; canonical bytes are
/// injective, so byte-equal means equal). Input order never affects the result.
pub fn emit_set<'a, T: Canonical + 'a>(
    out: &mut Vec<u8>,
    items: impl IntoIterator<Item = &'a T>,
) -> Result<(), CanonError> {
    let mut keys: Vec<Vec<u8>> = items
        .into_iter()
        .map(canonical_sort_key)
        .collect::<Result<_, _>>()?;
    keys.sort_unstable();
    keys.dedup();
    // A sort key is the element's own canonical bytes, emitted verbatim.
    emit_bracketed(out, keys, |b, k| {
        b.extend_from_slice(&k);
        Ok(())
    })
}

/// A type usable as a SPEC §10 map key. `IDENTIFIER_ASCII` is a type-level
/// promise that every value's canonical form is an identifier_ascii JSON
/// string; it picks the map encoding for the whole map (even when empty), so it
/// must not vary by value.
pub trait MapKey: Canonical {
    /// `true` selects the object form (keys as sorted member names); `false`
    /// selects the pair-array form for structured/other keys.
    const IDENTIFIER_ASCII: bool;

    /// The unquoted object-form member name: `Some` exactly when
    /// `IDENTIFIER_ASCII`. The default suits non-identifier keys.
    fn key_str(&self) -> Option<Cow<'_, str>> {
        None
    }
}

/// `Id` grammar `[a-z][a-z0-9_.:-]*` ⊂ identifier_ascii `[a-z0-9_:./-]+`.
impl MapKey for Id {
    const IDENTIFIER_ASCII: bool = true;
    fn key_str(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(self.as_str()))
    }
}

/// `Hash` form `sha256:` + lowercase hex stays within identifier_ascii.
impl MapKey for Hash {
    const IDENTIFIER_ASCII: bool = true;
    fn key_str(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(self.as_str()))
    }
}

/// SPEC §10 map. identifier_ascii keys (`K::IDENTIFIER_ASCII`) encode as an
/// object with members sorted by key-name bytes; all other keys encode as an
/// array of `{"key":K,"value":V}` pairs sorted by [`canonical_sort_key`] of the
/// key. Duplicate keys are rejected in both forms (a map binds each key once),
/// and an empty map keeps its type-guided form (`{}` vs `[]`).
pub fn emit_map<'a, K: MapKey + 'a, V: Canonical + 'a>(
    out: &mut Vec<u8>,
    entries: impl IntoIterator<Item = (&'a K, &'a V)>,
) -> Result<(), CanonError> {
    if K::IDENTIFIER_ASCII {
        let mut obj = ObjectEmitter::new();
        for (k, v) in entries {
            let name = k.key_str().expect("IDENTIFIER_ASCII MapKey yields key_str");
            obj.member(name.as_ref(), |b| v.emit_canonical(b))?;
        }
        obj.finish_with(out, CanonError::DuplicateMapKey)
    } else {
        let mut pairs: Vec<(Vec<u8>, &V)> = entries
            .into_iter()
            .map(|(k, v)| Ok((canonical_sort_key(k)?, v)))
            .collect::<Result<_, CanonError>>()?;
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        if let Some(w) = pairs.windows(2).find(|w| w[0].0 == w[1].0) {
            return Err(CanonError::DuplicateMapKey(
                String::from_utf8_lossy(&w[0].0).into_owned(),
            ));
        }
        emit_bracketed(out, pairs, |b, (key, v)| {
            b.extend_from_slice(b"{\"key\":");
            b.extend_from_slice(&key);
            b.extend_from_slice(b",\"value\":");
            v.emit_canonical(b)?;
            b.push(b'}');
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Canonical bytes of `value` as a UTF-8 string, for exact-match assertions.
    fn canon<T: Canonical>(value: &T) -> String {
        String::from_utf8(canonical_payload_bytes(value).unwrap()).unwrap()
    }

    fn emitted(emit: impl FnOnce(&mut Vec<u8>)) -> String {
        let mut out = Vec::new();
        emit(&mut out);
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn string_escapes_minimal_set_and_passes_utf8() {
        // `"` -> \" and `\` -> \\ ; nothing else in this pair escapes.
        assert_eq!(emitted(|o| emit_string(o, "\"\\")), "\"\\\"\\\\\"");
        // C0 controls become lowercase \u00xx, never shorthand (\n, \t).
        assert_eq!(emitted(|o| emit_string(o, "\n\t")), "\"\\u000a\\u0009\"");
        assert_eq!(
            emitted(|o| emit_string(o, "\u{0000}\u{001f}")),
            "\"\\u0000\\u001f\""
        );
        // Non-ASCII UTF-8 (and `/`) pass through unescaped.
        assert_eq!(emitted(|o| emit_string(o, "café漢/字")), "\"café漢/字\"");
    }

    #[test]
    fn integers_are_decimal_strings() {
        for (n, want) in [(0_i64, "\"0\""), (42, "\"42\""), (-42, "\"-42\"")] {
            assert_eq!(canon(&BigInt::from(n)), want);
        }
        let big = "123456789012345678901234567890";
        assert_eq!(canon(&big.parse::<BigInt>().unwrap()), format!("\"{big}\""));
    }

    #[test]
    fn rationals_are_reduced_byte_sorted_objects() {
        assert_eq!(
            canon(&Rational::from_parts("2", "4").unwrap()),
            r#"{"den":"2","num":"1"}"#
        );
        // Sign normalizes onto the numerator; denominator stays positive.
        assert_eq!(
            canon(&Rational::from_parts("3", "-2").unwrap()),
            r#"{"den":"2","num":"-3"}"#
        );
    }

    #[test]
    fn ids_and_hashes_emit_as_json_strings() {
        assert_eq!(
            canon(&Id::new("pipe.layered_ckcir_to_smt").unwrap()),
            "\"pipe.layered_ckcir_to_smt\""
        );
        let h = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(canon(&Hash::new(h).unwrap()), format!("\"{h}\""));
    }

    #[test]
    fn object_fields_sort_by_byte_order() {
        let mut o = ObjectEmitter::new();
        // Insert out of order; finish must sort by field-name bytes.
        o.member("b", |b| Id::new("y").unwrap().emit_canonical(b))
            .unwrap();
        o.member("a", |b| Id::new("x").unwrap().emit_canonical(b))
            .unwrap();
        o.member("Z", |b| Id::new("z").unwrap().emit_canonical(b))
            .unwrap();
        // Uppercase 'Z' (0x5a) sorts before lowercase 'a' (0x61).
        assert_eq!(
            emitted(|out| o.finish(out).unwrap()),
            r#"{"Z":"z","a":"x","b":"y"}"#
        );
    }

    #[test]
    fn object_rejects_duplicate_field() {
        let mut o = ObjectEmitter::new();
        o.member("dup", |b| {
            emit_string_policy(b, StringPolicy::ViewText, "1")
        })
        .unwrap();
        o.member("dup", |b| {
            emit_string_policy(b, StringPolicy::ViewText, "2")
        })
        .unwrap();
        let mut out = Vec::new();
        assert!(matches!(o.finish(&mut out), Err(CanonError::DuplicateField(n)) if n == "dup"));
    }

    #[test]
    fn optional_member_is_omitted_when_absent() {
        let mut o = ObjectEmitter::new();
        o.optional("present", Some(&BigInt::from(1)), |b, v: &BigInt| {
            emit_int(b, v);
            Ok(())
        })
        .unwrap();
        o.optional("absent", None, |b, v: &BigInt| {
            emit_int(b, v);
            Ok(())
        })
        .unwrap();
        // The absent field is gone entirely — no "absent" key, no null token.
        assert_eq!(emitted(|out| o.finish(out).unwrap()), r#"{"present":"1"}"#);
    }

    #[test]
    fn string_policy_normalizes_before_escaping() {
        // semantic_en folds whitespace and lowercases ASCII before emission.
        assert_eq!(
            emitted(|o| emit_string_policy(o, StringPolicy::SemanticEn, "  HELLO  Ä ").unwrap()),
            "\"hello Ä\""
        );
        // identifier_ascii passes valid bytes through and rejects out-of-grammar.
        assert_eq!(
            emitted(
                |o| emit_string_policy(o, StringPolicy::IdentifierAscii, "schema.ir_bundle")
                    .unwrap()
            ),
            "\"schema.ir_bundle\""
        );
        let mut out = Vec::new();
        assert!(matches!(
            emit_string_policy(&mut out, StringPolicy::IdentifierAscii, "Bad Id"),
            Err(CanonError::Policy(ValidationError::StringPolicy(_)))
        ));
    }

    #[test]
    fn mixed_object_is_deterministic_end_to_end() {
        let mut o = ObjectEmitter::new();
        o.member("ratio", |b| {
            Rational::from_parts("2", "4").unwrap().emit_canonical(b)
        })
        .unwrap();
        o.member("count", |b| {
            emit_int(b, &BigInt::from(7));
            Ok(())
        })
        .unwrap();
        o.member("id", |b| Id::new("pipe.x").unwrap().emit_canonical(b))
            .unwrap();
        o.member("label", |b| {
            emit_string_policy(b, StringPolicy::SemanticEn, "  Hello  ")
        })
        .unwrap();
        assert_eq!(
            emitted(|out| o.finish(out).unwrap()),
            r#"{"count":"7","id":"pipe.x","label":"hello","ratio":{"den":"2","num":"1"}}"#
        );
    }

    /// A structured (non-identifier_ascii) map key: its canonical form is an
    /// object, so [`emit_map`] must fall to the SPEC §10 pair-array form.
    struct Span {
        lo: i64,
        hi: i64,
    }

    impl Canonical for Span {
        fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
            let mut o = ObjectEmitter::new();
            o.member("hi", |b| {
                emit_int(b, &BigInt::from(self.hi));
                Ok(())
            })?;
            o.member("lo", |b| {
                emit_int(b, &BigInt::from(self.lo));
                Ok(())
            })?;
            o.finish(out)
        }
    }

    impl MapKey for Span {
        const IDENTIFIER_ASCII: bool = false;
    }

    #[test]
    fn arrays_preserve_semantic_order() {
        let items = [
            Id::new("c").unwrap(),
            Id::new("a").unwrap(),
            Id::new("b").unwrap(),
        ];
        // Order is meaningful: it is kept verbatim, never sorted.
        assert_eq!(
            emitted(|o| emit_array(o, &items).unwrap()),
            r#"["c","a","b"]"#
        );
        let empty: [Id; 0] = [];
        assert_eq!(emitted(|o| emit_array(o, &empty).unwrap()), "[]");
    }

    #[test]
    fn sets_sort_and_collapse_by_canonical_bytes() {
        let items = [
            Id::new("c").unwrap(),
            Id::new("a").unwrap(),
            Id::new("b").unwrap(),
            Id::new("a").unwrap(),
        ];
        // Sorted by canonical bytes; the repeated "a" collapses to one element.
        assert_eq!(
            emitted(|o| emit_set(o, &items).unwrap()),
            r#"["a","b","c"]"#
        );
        // The order is byte-lexicographic over canonical bytes, not numeric:
        // "10" precedes "9" because '1' (0x31) sorts before '9' (0x39).
        let nums = [BigInt::from(9), BigInt::from(10)];
        assert_eq!(emitted(|o| emit_set(o, &nums).unwrap()), r#"["10","9"]"#);
        let empty: [Id; 0] = [];
        assert_eq!(emitted(|o| emit_set(o, &empty).unwrap()), "[]");
    }

    #[test]
    fn identifier_keyed_maps_emit_sorted_objects() {
        let entries = [
            (Id::new("b").unwrap(), BigInt::from(2)),
            (Id::new("a").unwrap(), BigInt::from(1)),
        ];
        // identifier_ascii keys -> object, members sorted by key-name bytes.
        assert_eq!(
            emitted(|o| emit_map(o, entries.iter().map(|(k, v)| (k, v))).unwrap()),
            r#"{"a":"1","b":"2"}"#
        );
        // An empty identifier-keyed map keeps the object form.
        assert_eq!(
            emitted(|o| emit_map(o, std::iter::empty::<(&Id, &BigInt)>()).unwrap()),
            "{}"
        );
        // A repeated key is ambiguous and rejected.
        let dup = [
            (Id::new("k").unwrap(), BigInt::from(1)),
            (Id::new("k").unwrap(), BigInt::from(2)),
        ];
        let mut out = Vec::new();
        assert!(matches!(
            emit_map(&mut out, dup.iter().map(|(k, v)| (k, v))),
            Err(CanonError::DuplicateMapKey(k)) if k == "k"
        ));
    }

    #[test]
    fn structured_keyed_maps_emit_sorted_pair_arrays() {
        let entries = [
            (Span { lo: 5, hi: 9 }, Id::new("y").unwrap()),
            (Span { lo: 1, hi: 2 }, Id::new("x").unwrap()),
        ];
        // Non-identifier keys -> array of {"key","value"} pairs sorted by the
        // key's canonical bytes ({"hi":"2",…} precedes {"hi":"9",…}).
        assert_eq!(
            emitted(|o| emit_map(o, entries.iter().map(|(k, v)| (k, v))).unwrap()),
            r#"[{"key":{"hi":"2","lo":"1"},"value":"x"},{"key":{"hi":"9","lo":"5"},"value":"y"}]"#
        );
        // An empty structured-keyed map keeps the pair-array form.
        assert_eq!(
            emitted(|o| emit_map(o, std::iter::empty::<(&Span, &Id)>()).unwrap()),
            "[]"
        );
        // Byte-identical keys are rejected here too.
        let dup = [
            (Span { lo: 1, hi: 2 }, Id::new("x").unwrap()),
            (Span { lo: 1, hi: 2 }, Id::new("y").unwrap()),
        ];
        let mut out = Vec::new();
        assert!(matches!(
            emit_map(&mut out, dup.iter().map(|(k, v)| (k, v))),
            Err(CanonError::DuplicateMapKey(_))
        ));
    }

    #[test]
    fn canonical_sort_key_is_payload_bytes() {
        let id = Id::new("pipe.x").unwrap();
        assert_eq!(
            canonical_sort_key(&id).unwrap(),
            canonical_payload_bytes(&id).unwrap()
        );
    }
}
