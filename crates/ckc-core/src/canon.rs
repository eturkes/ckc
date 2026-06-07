//! SPEC §1.5 canonical JSON payload bytes: writer core.
//!
//! Single emission authority for canonical bytes (§0 one canonical
//! representation per fact): artifact bytes are produced here; serde
//! `Deserialize` impls are the validating read side. `TYPE_ID` values are
//! provisional symbol ids pending the M0.0.3 `SchemaRegistry`. Tagged
//! unions and strict canonical reading land in M0.0.2.3.

use std::borrow::Cow;
use std::fmt;

use crate::policy::{
    IdentifierAscii, PolicyMarker, StringPolicy, Text, UnicodePolicyManifest, is_identifier_ascii,
};
use crate::scalar::{Hash, Id, Int, Rational, UInt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonError {
    DuplicateField {
        name: String,
    },
    /// §1.5 duplicate map keys are rejected (unlike set duplicates, which
    /// collapse): two bindings for one key have no canonical value. `key`
    /// holds the member name (object form) or canonical key bytes (pair
    /// form).
    DuplicateMapKey {
        key: String,
    },
}

impl fmt::Display for CanonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateField { name } => write!(f, "duplicate object field: {name:?}"),
            Self::DuplicateMapKey { key } => write!(f, "duplicate map key: {key:?}"),
        }
    }
}

impl std::error::Error for CanonError {}

// ---------------------------------------------------------------------------
// Byte emitters (§1.5 grammar)
// ---------------------------------------------------------------------------

/// §1.5 string: UTF-8 passthrough; escape exactly U+0022 as `\"`, U+005C as
/// `\\`, and U+0000..U+001F as lowercase `\u00xx` (shorthand escapes like
/// `\n` are non-canonical). Control code points are single UTF-8 bytes, so
/// the byte loop is exact.
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

/// §1.5 bool: `true` or `false`.
pub fn emit_bool(out: &mut Vec<u8>, value: bool) {
    out.extend_from_slice(if value { b"true" } else { b"false" });
}

/// §1.5 array: values in semantic order, comma-separated.
pub fn emit_array<T>(
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

/// §1.5 object: buffers members, sorts by UTF-8 field-name bytes (`str`
/// order), rejects duplicate names, writes `{"name":value,…}`. Omitted
/// optional fields are absent: skip `member` for `None`.
#[derive(Default)]
pub struct ObjectEmitter {
    members: Vec<(String, Vec<u8>)>,
}

impl ObjectEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Buffer one member; `emit` writes the member's value bytes.
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

    pub fn finish(self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        self.finish_with(out, |name| CanonError::DuplicateField { name })
    }

    /// Shared sort/duplicate/write tail; `dup` shapes the duplicate-name
    /// error (record field vs map key).
    fn finish_with(
        mut self,
        out: &mut Vec<u8>,
        dup: impl FnOnce(String) -> CanonError,
    ) -> Result<(), CanonError> {
        self.members.sort_by(|a, b| a.0.cmp(&b.0));
        if let Some(w) = self.members.windows(2).find(|w| w[0].0 == w[1].0) {
            return Err(dup(w[0].0.clone()));
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

// ---------------------------------------------------------------------------
// Canonical trait
// ---------------------------------------------------------------------------

/// Type-guided canonical emission (§1.5).
pub trait Canonical {
    /// §1.5 `declared_type_id`, first input of `canonical_sort_key`
    /// (M0.0.2.2).
    const TYPE_ID: &'static str;

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError>;
}

/// §1.5 `canonical_payload_bytes`: deterministic injection over schema-valid
/// typed payloads.
pub fn canonical_payload_bytes<T: Canonical>(value: &T) -> Result<Vec<u8>, CanonError> {
    let mut out = Vec::new();
    value.emit_canonical(&mut out)?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// Sort keys (§1.5)
// ---------------------------------------------------------------------------

/// §1.5 `canonical_sort_key(x) = (declared_type_id, canonical_payload_bytes(x))`
/// for inline values. Derived `Ord` is total: byte order on the provisional
/// `TYPE_ID`, then on payload bytes (byte-lexicographic, not numeric).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalSortKey {
    pub type_id: &'static str,
    pub payload: Vec<u8>,
}

pub fn canonical_sort_key<T: Canonical>(x: &T) -> Result<CanonicalSortKey, CanonError> {
    Ok(CanonicalSortKey {
        type_id: T::TYPE_ID,
        payload: canonical_payload_bytes(x)?,
    })
}

/// §1.5 accepted-object-reference sort key: referenced `artifact_hash`, then
/// `schema_id`, then declared reference field name. Reference collections
/// sort by this at the schema layer (envelopes land M0.0.5); inline values
/// sort by [`canonical_sort_key`].
pub fn accepted_reference_sort_key<'a>(
    artifact_hash: &'a Hash,
    schema_id: &'a Id,
    reference_field_name: &'a str,
) -> (&'a str, &'a str, &'a str) {
    (
        artifact_hash.as_str(),
        schema_id.as_str(),
        reference_field_name,
    )
}

// ---------------------------------------------------------------------------
// Collections (§1.3 Set/List/Map, §1.5 set/map rules)
// ---------------------------------------------------------------------------

/// §1.5 set: array sorted by `canonical_sort_key(element)` with byte-identical
/// duplicates collapsed (set semantics; canonical bytes are injective, so
/// byte-equal means semantically equal). Input order never matters.
pub fn emit_set<'a, T: Canonical + 'a>(
    out: &mut Vec<u8>,
    items: impl IntoIterator<Item = &'a T>,
) -> Result<(), CanonError> {
    let mut keys: Vec<CanonicalSortKey> = items
        .into_iter()
        .map(canonical_sort_key)
        .collect::<Result<_, _>>()?;
    keys.sort_unstable();
    keys.dedup();
    emit_array(out, keys, |b, k| {
        b.extend_from_slice(&k.payload);
        Ok(())
    })
}

/// §1.5 array rule for `List[T]`: semantic order.
pub fn emit_list<'a, T: Canonical + 'a>(
    out: &mut Vec<u8>,
    items: impl IntoIterator<Item = &'a T>,
) -> Result<(), CanonError> {
    emit_array(out, items, |b, t| t.emit_canonical(b))
}

/// Map-key capability (§1.5 map rule). `IDENTIFIER_ASCII` is a type-level
/// guarantee that every value's canonical encoding is a JSON string whose
/// content satisfies §1.4 `is_identifier_ascii`; such `Map[K,V]` encode as
/// objects keyed by `key_str`. All other key types (e.g. record-encoded
/// keys like §6.2 `Var`) encode as a pair array sorted by
/// `canonical_sort_key(key)`.
pub trait MapKey: Canonical {
    const IDENTIFIER_ASCII: bool;

    /// Object-form member name; `Some` exactly when `IDENTIFIER_ASCII`.
    fn key_str(&self) -> Option<Cow<'_, str>> {
        None
    }
}

/// `Id` charset `[a-z][a-z0-9_:-]*` ⊂ identifier_ascii charset.
impl MapKey for Id {
    const IDENTIFIER_ASCII: bool = true;

    fn key_str(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(self.as_str()))
    }
}

/// `Hash` form `sha256:` + lowercase hex stays in `[a-z0-9:]`.
impl MapKey for Hash {
    const IDENTIFIER_ASCII: bool = true;

    fn key_str(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(self.as_str()))
    }
}

/// `Int` decimal form: optional `-` plus digits (§7.1 `stage_bounds` keys).
impl MapKey for Int {
    const IDENTIFIER_ASCII: bool = true;

    fn key_str(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Owned(self.to_decimal()))
    }
}

impl MapKey for Text<IdentifierAscii> {
    const IDENTIFIER_ASCII: bool = true;

    fn key_str(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(self.as_str()))
    }
}

/// §1.5 map: object when `K::IDENTIFIER_ASCII` (members sorted by name
/// bytes), else array of `{"key":K,"value":V}` pairs sorted by
/// `canonical_sort_key(key)`; duplicate keys are rejected in both forms. An
/// empty map keeps its type-guided form: `{}` vs `[]`.
pub fn emit_map<'a, K: MapKey + 'a, V: Canonical + 'a>(
    out: &mut Vec<u8>,
    entries: impl IntoIterator<Item = (&'a K, &'a V)>,
) -> Result<(), CanonError> {
    if K::IDENTIFIER_ASCII {
        let mut obj = ObjectEmitter::new();
        for (k, v) in entries {
            let name = k
                .key_str()
                .expect("IDENTIFIER_ASCII MapKey returns key_str");
            debug_assert!(
                is_identifier_ascii(&name),
                "MapKey emitted non-identifier object key: {name:?}"
            );
            obj.member(&name, |b| v.emit_canonical(b))?;
        }
        obj.finish_with(out, |key| CanonError::DuplicateMapKey { key })
    } else {
        let mut pairs: Vec<(CanonicalSortKey, &V)> = entries
            .into_iter()
            .map(|(k, v)| Ok((canonical_sort_key(k)?, v)))
            .collect::<Result<_, CanonError>>()?;
        pairs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        if let Some(w) = pairs.windows(2).find(|w| w[0].0 == w[1].0) {
            return Err(CanonError::DuplicateMapKey {
                key: String::from_utf8_lossy(&w[0].0.payload).into_owned(),
            });
        }
        emit_array(out, pairs, |b, (key, v)| {
            b.extend_from_slice(b"{\"key\":");
            b.extend_from_slice(&key.payload);
            b.extend_from_slice(b",\"value\":");
            v.emit_canonical(b)?;
            b.push(b'}');
            Ok(())
        })
    }
}

// ---------------------------------------------------------------------------
// Impls: §1.3 scalars, Text<P>, UnicodePolicyManifest
// ---------------------------------------------------------------------------

impl Canonical for bool {
    const TYPE_ID: &'static str = "bool";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_bool(out, *self);
        Ok(())
    }
}

impl Canonical for Id {
    const TYPE_ID: &'static str = "id";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string(out, self.as_str());
        Ok(())
    }
}

impl Canonical for Hash {
    const TYPE_ID: &'static str = "hash";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string(out, self.as_str());
        Ok(())
    }
}

impl Canonical for UInt {
    const TYPE_ID: &'static str = "uint";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string(out, &self.to_decimal());
        Ok(())
    }
}

impl Canonical for Int {
    const TYPE_ID: &'static str = "int";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string(out, &self.to_decimal());
        Ok(())
    }
}

impl Canonical for Rational {
    const TYPE_ID: &'static str = "rational";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("den", |b| self.den().emit_canonical(b))?;
        obj.member("num", |b| self.num().emit_canonical(b))?;
        obj.finish(out)
    }
}

impl<P: PolicyMarker> Canonical for Text<P> {
    /// Provisional `text:<policy_id>` symbol ids.
    const TYPE_ID: &'static str = match P::POLICY {
        StringPolicy::RawSource => "text:raw_source",
        StringPolicy::SourceNfkc => "text:source_nfkc",
        StringPolicy::SemanticJa => "text:semantic_ja",
        StringPolicy::SemanticEn => "text:semantic_en",
        StringPolicy::IdentifierAscii => "text:identifier_ascii",
        StringPolicy::TemplateLiteral => "text:template_literal",
        StringPolicy::DiagnosticText => "text:diagnostic_text",
        StringPolicy::ViewText => "text:view_text",
    };

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string(out, self.as_str());
        Ok(())
    }
}

impl Canonical for UnicodePolicyManifest {
    const TYPE_ID: &'static str = "unicode_policy_manifest";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("manifest_id", |b| self.manifest_id.emit_canonical(b))?;
        obj.member("normalization_table_hash", |b| {
            self.normalization_table_hash.emit_canonical(b)
        })?;
        obj.member("policy_test_hash", |b| {
            self.policy_test_hash.emit_canonical(b)
        })?;
        obj.member("punctuation_table_hash", |b| {
            self.punctuation_table_hash.emit_canonical(b)
        })?;
        obj.member("unicode_version", |b| {
            self.unicode_version.emit_canonical(b)
        })?;
        obj.finish(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{RawSource, SemanticJa};

    fn bytes_of<T: Canonical>(x: &T) -> String {
        String::from_utf8(canonical_payload_bytes(x).expect("emits")).expect("utf-8")
    }

    #[test]
    fn string_minimal_escape() {
        let mut out = Vec::new();
        emit_string(&mut out, "a\"b\\c\u{0}\u{1f}\nあ\u{7f}");
        // U+007F is not a §1.5 control escape; it passes through raw.
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "\"a\\\"b\\\\c\\u0000\\u001f\\u000aあ\u{7f}\""
        );
    }

    #[test]
    fn object_sorts_by_field_name_bytes_and_rejects_duplicates() {
        let mut obj = ObjectEmitter::new();
        for name in ["あ", "z", "a"] {
            obj.member(name, |b| {
                emit_bool(b, true);
                Ok(())
            })
            .unwrap();
        }
        let mut out = Vec::new();
        obj.finish(&mut out).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "{\"a\":true,\"z\":true,\"あ\":true}"
        );

        let mut dup = ObjectEmitter::new();
        for name in ["a", "a"] {
            dup.member(name, |b| {
                emit_bool(b, true);
                Ok(())
            })
            .unwrap();
        }
        assert_eq!(
            dup.finish(&mut Vec::new()),
            Err(CanonError::DuplicateField {
                name: "a".to_owned()
            })
        );
    }

    #[test]
    fn array_keeps_semantic_order() {
        let mut out = Vec::new();
        emit_array(&mut out, [3u64, 1, 2].map(UInt::from), |b, v| {
            v.emit_canonical(b)
        })
        .unwrap();
        assert_eq!(out, br#"["3","1","2"]"#);

        let mut empty = Vec::new();
        emit_array(&mut empty, std::iter::empty::<UInt>(), |b, v| {
            v.emit_canonical(b)
        })
        .unwrap();
        assert_eq!(empty, b"[]");
    }

    #[test]
    fn sort_key_orders_by_type_id_then_payload_bytes() {
        let int = canonical_sort_key(&Int::from(38)).unwrap();
        let uint = canonical_sort_key(&UInt::from(38u64)).unwrap();
        assert_eq!(int.payload, uint.payload);
        assert!(int < uint, "provisional id \"int\" < \"uint\"");
        // Payload order is byte-lexicographic, not numeric: "10" < "2".
        let ten = canonical_sort_key(&UInt::from(10u64)).unwrap();
        let two = canonical_sort_key(&UInt::from(2u64)).unwrap();
        assert!(ten < two);
    }

    #[test]
    fn accepted_reference_sort_key_priority() {
        let lo = Hash::parse(&format!("sha256:{}", "0".repeat(64))).unwrap();
        let hi = Hash::parse(&format!("sha256:{}", "f".repeat(64))).unwrap();
        let (a, b) = (Id::new("a").unwrap(), Id::new("b").unwrap());
        // artifact_hash dominates schema_id; schema_id dominates field name.
        assert!(
            accepted_reference_sort_key(&lo, &b, "z") < accepted_reference_sort_key(&hi, &a, "a")
        );
        assert!(
            accepted_reference_sort_key(&lo, &a, "z") < accepted_reference_sort_key(&lo, &b, "a")
        );
        assert!(
            accepted_reference_sort_key(&lo, &a, "a") < accepted_reference_sort_key(&lo, &a, "b")
        );
    }

    #[test]
    fn set_sorts_dedups_and_list_keeps_order() {
        let items: Vec<UInt> = [10u64, 2, 38, 2].map(UInt::from).to_vec();
        let mut set = Vec::new();
        emit_set(&mut set, &items).unwrap();
        assert_eq!(set, br#"["10","2","38"]"#);
        let mut list = Vec::new();
        emit_list(&mut list, &items).unwrap();
        assert_eq!(list, br#"["10","2","38","2"]"#);
        let mut empty = Vec::new();
        emit_set(&mut empty, std::iter::empty::<&UInt>()).unwrap();
        assert_eq!(empty, b"[]");
    }

    #[test]
    fn map_identifier_keys_encode_as_object() {
        let entries = [
            (Id::new("b").unwrap(), UInt::from(2u64)),
            (Id::new("a").unwrap(), UInt::from(1u64)),
        ];
        let mut out = Vec::new();
        emit_map(&mut out, entries.iter().map(|(k, v)| (k, v))).unwrap();
        assert_eq!(out, br#"{"a":"1","b":"2"}"#);

        // Int keys: decimal-string member names, sorted by name bytes.
        let stages = [
            (Int::from(2), UInt::from(1u64)),
            (Int::from(-10), UInt::from(1u64)),
        ];
        let mut out = Vec::new();
        emit_map(&mut out, stages.iter().map(|(k, v)| (k, v))).unwrap();
        assert_eq!(out, br#"{"-10":"1","2":"1"}"#);

        let dup = [
            (Id::new("a").unwrap(), UInt::from(1u64)),
            (Id::new("a").unwrap(), UInt::from(2u64)),
        ];
        assert_eq!(
            emit_map(&mut Vec::new(), dup.iter().map(|(k, v)| (k, v))),
            Err(CanonError::DuplicateMapKey {
                key: "a".to_owned()
            })
        );
    }

    /// Record-encoded key (shape of §6.2 `Var`), exercising the pair-array
    /// form ahead of M0.3.4.
    struct VarKey(Id);

    impl Canonical for VarKey {
        const TYPE_ID: &'static str = "var";

        fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
            let mut obj = ObjectEmitter::new();
            obj.member("var_id", |b| self.0.emit_canonical(b))?;
            obj.finish(out)
        }
    }

    impl MapKey for VarKey {
        const IDENTIFIER_ASCII: bool = false;
    }

    #[test]
    fn map_other_keys_encode_as_sorted_pair_array() {
        let id = |s| Id::new(s).unwrap();
        let entries = [
            (VarKey(id("y")), UInt::from(2u64)),
            (VarKey(id("x")), UInt::from(1u64)),
        ];
        let mut out = Vec::new();
        emit_map(&mut out, entries.iter().map(|(k, v)| (k, v))).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            r#"[{"key":{"var_id":"x"},"value":"1"},{"key":{"var_id":"y"},"value":"2"}]"#
        );

        let dup = [
            (VarKey(id("x")), UInt::from(1u64)),
            (VarKey(id("x")), UInt::from(2u64)),
        ];
        assert_eq!(
            emit_map(&mut Vec::new(), dup.iter().map(|(k, v)| (k, v))),
            Err(CanonError::DuplicateMapKey {
                key: r#"{"var_id":"x"}"#.to_owned()
            })
        );

        let mut empty = Vec::new();
        emit_map(&mut empty, std::iter::empty::<(&VarKey, &UInt)>()).unwrap();
        assert_eq!(empty, b"[]", "empty map keeps its type-guided form");
    }

    #[test]
    fn scalar_canonical_bytes() {
        assert_eq!(bytes_of(&true), "true");
        assert_eq!(bytes_of(&Id::new("upm-m0").unwrap()), "\"upm-m0\"");
        assert_eq!(
            bytes_of(&Hash::of_bytes(b"")),
            "\"sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\""
        );
        assert_eq!(bytes_of(&UInt::from(38u64)), "\"38\"");
        assert_eq!(bytes_of(&Int::from(-7)), "\"-7\"");
        assert_eq!(
            bytes_of(&Rational::from_decimal_str("38.5").unwrap()),
            r#"{"den":"2","num":"77"}"#
        );
        assert_eq!(
            bytes_of(&Text::<SemanticJa>::new("ａ\u{3000}b").unwrap()),
            "\"a b\""
        );
        // Divergence from serde_json shorthand escaping: tab emits as \u0009.
        assert_eq!(
            bytes_of(&Text::<RawSource>::new("x\ty").unwrap()),
            r#""x\u0009y""#
        );
    }
}
