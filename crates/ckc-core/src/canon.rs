//! SPEC §1.5 canonical JSON payload bytes: writer core, union encoding,
//! strict canonical reading.
//!
//! Single emission authority for canonical bytes (§0 one canonical
//! representation per fact): artifact bytes are produced here; serde
//! `Deserialize` impls are the validating read side, entered at byte level
//! through [`from_canonical_bytes`]. `TYPE_ID` values are provisional
//! symbol ids pending the M0.0.3 `SchemaRegistry`.
//!
//! Enum-vs-union encoding (decided M0.0.2.3): a union whose alternatives
//! are all bare — a §2-style `E` enum such as `Outcome`, `Effect`,
//! `SchemaRole` — encodes as the bare JSON string of its variant symbol
//! exactly as SPEC writes it (`"ok"`, `"Inference"`, `"G-RET-PARITY"`).
//! §1.1 keeps enum variants and tagged-union alternatives as distinct
//! symbol kinds, and variant symbols like `Inference`/`S0`/`G-RET-PARITY`
//! violate the `Id` charset, so the encoding is a plain string holding the
//! variant symbol, not an `Id`. A union with at least one payload-carrying
//! alternative (e.g. §1.7 `OperationResult[T]`) encodes as the §1.5
//! `{"tag","value"}` object; its bare alternatives carry value `{}`.
//!
//! `bare_enum!` and `canonical_record!` (exported at crate root) mechanize
//! the `E`/`S` declaration patterns; new enums and records route through
//! them.

use std::borrow::Cow;
use std::fmt;

use crate::policy::{
    IdentifierAscii, PolicyMarker, StringPolicy, Text, UnicodePolicyManifest, is_identifier_ascii,
};
use crate::scalar::{FeaturePath, Hash, Id, Int, Rational, UInt};

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

/// §1.5 tagged union: an object with exactly two members, `"tag"` and
/// `"value"`, written directly in sorted member order. `emit_value` writes
/// the payload object, array, or scalar; [`emit_union_bare`] covers a bare
/// tag or an absent optional payload. Constructor tags are unique within a
/// union by Rust enum construction; the read side ([`read_union`]) rejects
/// duplicate tag members.
pub fn emit_union(
    out: &mut Vec<u8>,
    tag: &str,
    emit_value: impl FnOnce(&mut Vec<u8>) -> Result<(), CanonError>,
) -> Result<(), CanonError> {
    out.extend_from_slice(b"{\"tag\":");
    emit_string(out, tag);
    out.extend_from_slice(b",\"value\":");
    emit_value(out)?;
    out.push(b'}');
    Ok(())
}

/// §1.5 bare tag: `{"tag":<tag>,"value":{}}`.
pub fn emit_union_bare(out: &mut Vec<u8>, tag: &str) -> Result<(), CanonError> {
    emit_union(out, tag, |b| {
        b.extend_from_slice(b"{}");
        Ok(())
    })
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
// Strict canonical reading (§1.5)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadError {
    /// Typed parse rejected the bytes: malformed JSON, JSON null, JSON
    /// numeric token, duplicate object field, unknown field, unknown or
    /// duplicate or misordered union tag, or a scalar/policy canonicality
    /// violation (`Text::from_canonical`, `Rational::from_canonical_parts`).
    Parse { detail: String },
    /// Parsed value re-serializes to different bytes: non-canonical member
    /// order, whitespace, duplicate map keys or set elements that collapsed
    /// on read, or non-canonical string escapes.
    NonCanonical,
    /// Re-serialization of the parsed value failed; reachable when a read
    /// container holds entries the emitter rejects (e.g. duplicate
    /// pair-array map keys read into a `Vec`).
    Emit(CanonError),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { detail } => write!(f, "canonical parse rejected: {detail}"),
            Self::NonCanonical => f.write_str("bytes differ from canonical re-serialization"),
            Self::Emit(e) => write!(f, "re-serialization failed: {e}"),
        }
    }
}

impl std::error::Error for ReadError {}

/// §1.5 strict canonical read: typed parse, then re-serialize-compare.
/// Accepts exactly the bytes [`canonical_payload_bytes`] emits for a
/// schema-valid value of `T`.
pub fn from_canonical_bytes<T>(bytes: &[u8]) -> Result<T, ReadError>
where
    T: Canonical + serde::de::DeserializeOwned,
{
    let value: T = serde_json::from_slice(bytes).map_err(|e| ReadError::Parse {
        detail: e.to_string(),
    })?;
    let reserialized = canonical_payload_bytes(&value).map_err(ReadError::Emit)?;
    if reserialized == bytes {
        Ok(value)
    } else {
        Err(ReadError::NonCanonical)
    }
}

/// Strict §1.5 union-read driver for manual `Deserialize` impls: enforces
/// the exact `{"tag","value"}` two-member shape in canonical member order,
/// rejecting empty objects, duplicate tag members, unknown members, and
/// member counts above two. `parse` maps the tag to a value via
/// `map.next_value::<…>()` and errors on unknown tags; bare tags read
/// [`BareValue`].
pub fn read_union<'de, A, V>(
    mut map: A,
    parse: impl FnOnce(&str, &mut A) -> Result<V, A::Error>,
) -> Result<V, A::Error>
where
    A: serde::de::MapAccess<'de>,
{
    use serde::de::Error as _;
    match map.next_key::<String>()? {
        Some(k) if k == "tag" => {}
        Some(k) => {
            return Err(A::Error::custom(format!(
                "union member 1 is \"tag\", found {k:?}"
            )));
        }
        None => return Err(A::Error::custom("union object is empty")),
    }
    let tag: String = map.next_value()?;
    match map.next_key::<String>()? {
        Some(k) if k == "value" => {}
        Some(k) if k == "tag" => return Err(A::Error::custom("duplicate union tag member")),
        Some(k) => {
            return Err(A::Error::custom(format!(
                "union member 2 is \"value\", found {k:?}"
            )));
        }
        None => return Err(A::Error::custom("union object misses \"value\"")),
    }
    let value = parse(&tag, &mut map)?;
    if map.next_key::<String>()?.is_some() {
        return Err(A::Error::custom("union object has exactly two members"));
    }
    Ok(value)
}

/// §1.5 bare-tag payload: deserializes exactly `{}`.
pub struct BareValue;

impl<'de> serde::Deserialize<'de> for BareValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = BareValue;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bare union value {}")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<BareValue, A::Error> {
                use serde::de::Error as _;
                if map.next_key::<String>()?.is_some() {
                    return Err(A::Error::custom("bare union value is exactly {}"));
                }
                Ok(BareValue)
            }
        }
        deserializer.deserialize_map(V)
    }
}

// ---------------------------------------------------------------------------
// Declaration macros (§1.1 E/S forms)
// ---------------------------------------------------------------------------

/// Declares a §1.1 all-bare `E` enum (the `Outcome` pattern): SPEC
/// listing-order `IDS`/`ALL`, variant-symbol `id`/`from_id`, bare-string
/// §1.5 canonical encoding, strict `Deserialize`. Base derives match
/// `Outcome` — `Ord` is opt-in (`#[derive(PartialOrd, Ord)]` in the
/// invocation) because a derived listing order can shadow an explicit
/// semantic rank (§1.7 `Outcome::primacy`); opt in when the enum sits in a
/// `BTreeSet`/`BTreeMap` element.
#[macro_export]
macro_rules! bare_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident : $type_id:literal {
            $($(#[$vmeta:meta])* $variant:ident = $id:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name {
            $($(#[$vmeta])* $variant),+
        }

        impl $name {
            /// Variant symbols exactly as SPEC writes them, in listing order.
            pub const IDS: &'static [&'static str] = &[$($id),+];

            /// SPEC listing order.
            pub const ALL: [Self; Self::IDS.len()] = [$(Self::$variant),+];

            /// Variant symbol exactly as SPEC writes it.
            pub fn id(self) -> &'static str {
                Self::IDS[self as usize]
            }

            pub fn from_id(id: &str) -> Option<Self> {
                Self::ALL.into_iter().find(|x| x.id() == id)
            }
        }

        impl $crate::canon::Canonical for $name {
            const TYPE_ID: &'static str = $type_id;

            fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), $crate::canon::CanonError> {
                $crate::canon::emit_string(out, self.id());
                Ok(())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let s = <String as ::serde::Deserialize>::deserialize(deserializer)?;
                Self::from_id(&s)
                    .ok_or_else(|| <D::Error as ::serde::de::Error>::unknown_variant(&s, Self::IDS))
            }
        }
    };
}

/// Implements §1.5 `Canonical` for a §1.1 `S` record (the
/// `UnicodePolicyManifest` pattern): JSON member names are the Rust field
/// names, `ObjectEmitter` sorts them; `sets` members emit through
/// [`emit_set`]; `optional` members (`Option<T>`) are omitted when `None`.
/// Pair with `#[derive(Deserialize)] #[serde(deny_unknown_fields)]` for the
/// strict read side.
#[macro_export]
macro_rules! canonical_record {
    (
        $name:ident : $type_id:literal,
        fields { $($field:ident),* $(,)? }
        $(, sets { $($set:ident),* $(,)? })?
        $(, optional { $($opt:ident),* $(,)? })?
        $(,)?
    ) => {
        impl $crate::canon::Canonical for $name {
            const TYPE_ID: &'static str = $type_id;

            fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), $crate::canon::CanonError> {
                let mut obj = $crate::canon::ObjectEmitter::new();
                $(obj.member(stringify!($field), |b| {
                    $crate::canon::Canonical::emit_canonical(&self.$field, b)
                })?;)*
                $($(obj.member(stringify!($set), |b| $crate::canon::emit_set(b, &self.$set))?;)*)?
                $($(if let Some(v) = &self.$opt {
                    obj.member(stringify!($opt), |b| $crate::canon::Canonical::emit_canonical(v, b))?;
                })*)?
                obj.finish(out)
            }
        }
    };
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

impl Canonical for FeaturePath {
    const TYPE_ID: &'static str = "feature_path";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_list(out, self.segments())
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

canonical_record!(UnicodePolicyManifest: "unicode_policy_manifest",
    fields { manifest_id, normalization_table_hash, policy_test_hash, punctuation_table_hash, unicode_version });

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
    #[test]
    fn union_encoding_tag_then_value() {
        let mut out = Vec::new();
        emit_union(&mut out, "one", |b| UInt::from(2u64).emit_canonical(b)).unwrap();
        assert_eq!(out, br#"{"tag":"one","value":"2"}"#);
        let mut bare = Vec::new();
        emit_union_bare(&mut bare, "empty").unwrap();
        assert_eq!(bare, br#"{"tag":"empty","value":{}}"#);
    }
}
