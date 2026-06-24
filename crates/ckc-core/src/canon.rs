//! Canonical JSON payload bytes — writer core (SPEC §4.3).
//!
//! [`canonical_payload_bytes`] is the single function that turns a typed value
//! into the deterministic UTF-8 bytes hashed into an artifact's `content_hash`.
//! `core-canon-writer` delivered the scalar + object writer core,
//! `core-canon-collections` added the array, set, and map rules,
//! `core-canon-unions` added tagged unions, and `core-canon-reader` adds the
//! strict inverse ([`read_strict_canonical`], [`CanonRead`], [`CanonReadError`]) that
//! accepts only these bytes, and `core-canon-hash` seals them into an artifact's
//! content hash.
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
//! union    {"tag":<identifier_ascii>,"value":<payload>} — exactly two members
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

/// Failure while emitting canonical bytes (SPEC §4.3).
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

/// Type-guided canonical emission (SPEC §4.3): a value appends its canonical
/// UTF-8 bytes to `out`. Composite values build their fields through
/// [`ObjectEmitter`].
pub trait Canonical {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError>;
}

/// SPEC §4.3 `canonical_payload_bytes`: the deterministic bytes a later unit
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

/// Append an integer as its canonical decimal string, e.g. `"-42"` (SPEC §4.3:
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
    /// (SPEC §4.3 optional-omit; an absent field is never emitted as null).
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
// Canonical collections (SPEC §4.3)
// ---------------------------------------------------------------------------

/// SPEC §4.3 `canonical_sort_key`: the total order [`emit_set`] and pair-array
/// [`emit_map`] sort by. For CKC it is exactly the element's
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

/// SPEC §4.3 array: elements in their given semantic order, each emitted through
/// [`Canonical`]. Order is preserved verbatim; callers needing order-free set
/// semantics use [`emit_set`].
pub fn emit_array<'a, T: Canonical + 'a>(
    out: &mut Vec<u8>,
    items: impl IntoIterator<Item = &'a T>,
) -> Result<(), CanonError> {
    emit_bracketed(out, items, |b, item| item.emit_canonical(b))
}

/// SPEC §4.3 set: an array sorted by [`canonical_sort_key`] with byte-identical
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

/// A type usable as a SPEC §4.3 map key. `IDENTIFIER_ASCII` is a type-level
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

/// SPEC §4.3 map. identifier_ascii keys (`K::IDENTIFIER_ASCII`) encode as an
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

// ---------------------------------------------------------------------------
// Tagged unions (SPEC §4.3)
// ---------------------------------------------------------------------------

/// SPEC §4.3 union: a sum value as the tagged object `{"tag":…,"value":…}` with
/// exactly those two members. `tag` is the variant discriminant, normalized
/// through `identifier_ascii` (so it is a controlled, escape-free identifier);
/// `value` is the variant payload, emitted through [`Canonical`]. Field order
/// is byte-sorted by [`ObjectEmitter`] (`tag` 0x74 < `value` 0x76), giving one
/// fixed encoding. Fails only when `tag` violates the identifier_ascii grammar
/// or the payload's own emission fails.
pub fn emit_union<V: Canonical>(out: &mut Vec<u8>, tag: &str, value: &V) -> Result<(), CanonError> {
    let mut obj = ObjectEmitter::new();
    obj.member("tag", |b| {
        emit_string_policy(b, StringPolicy::IdentifierAscii, tag)
    })?;
    obj.member("value", |b| value.emit_canonical(b))?;
    obj.finish(out)
}

// ---------------------------------------------------------------------------
// Strict canonical reading (SPEC §4.3) — the writer's inverse
// ---------------------------------------------------------------------------

/// Failure while strictly reading canonical bytes (SPEC §4.3). Every variant
/// marks an encoding the writer never produces, so [`read_strict_canonical`] accepts
/// exactly the canonical form and nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonReadError {
    /// A structural byte was missing or wrong; the `&str` names what was
    /// expected (e.g. `":"`, `"}"`, `"["`).
    Syntax(&'static str),
    /// Input ended in the middle of a value.
    Eof,
    /// Bytes remained after the top-level value (canonical bytes are exact).
    Trailing,
    /// A non-canonical JSON token sat where a value was expected — a bare number,
    /// or `null`/`true`/`false`. Canonical form encodes integers as strings and
    /// rationals as objects and never emits null or booleans.
    Token,
    /// A string was not in canonical form: a shorthand or non-minimal escape, an
    /// unescaped control byte, or invalid UTF-8. The `&str` notes which.
    Str(&'static str),
    /// An integer string was not canonical decimal (leading zero, `+`, or `-0`).
    Integer(String),
    /// An object carried a field no type expected (also how a duplicate or an
    /// out-of-order field surfaces — it appears where none belongs).
    UnknownField(String),
    /// A required field was absent, or the next field did not match it.
    MissingField(&'static str),
    /// Set or map elements were not strictly ascending by [`canonical_sort_key`]
    /// (covering both mis-ordering and a repeated element).
    Unsorted,
    /// A rational object was not exact-reduced with a positive denominator.
    RationalNotReduced,
    /// A string under a declared policy was not already normalized to that
    /// policy's canonical bytes.
    Unnormalized(StringPolicy),
    /// A parsed value failed its own grammar (`Id`, `Hash`, or a string policy).
    Policy(ValidationError),
}

impl fmt::Display for CanonReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonReadError::Syntax(w) => write!(f, "expected {w}"),
            CanonReadError::Eof => write!(f, "unexpected end of input"),
            CanonReadError::Trailing => write!(f, "trailing bytes after canonical value"),
            CanonReadError::Token => write!(f, "non-canonical JSON token at a value position"),
            CanonReadError::Str(w) => write!(f, "non-canonical string: {w}"),
            CanonReadError::Integer(s) => write!(f, "non-canonical integer string: {s:?}"),
            CanonReadError::UnknownField(n) => write!(f, "unexpected object field: {n:?}"),
            CanonReadError::MissingField(n) => write!(f, "missing object field: {n:?}"),
            CanonReadError::Unsorted => write!(f, "set/map elements not strictly ascending"),
            CanonReadError::RationalNotReduced => write!(f, "rational not exact-reduced"),
            CanonReadError::Unnormalized(p) => write!(f, "string not normalized under {p:?}"),
            CanonReadError::Policy(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CanonReadError {}

impl From<ValidationError> for CanonReadError {
    fn from(e: ValidationError) -> Self {
        CanonReadError::Policy(e)
    }
}

/// Re-emitting a value just read can in principle surface a writer error; fold it
/// into the reader's domain so the ordering checks (which re-emit each element to
/// derive its sort key) compose with `?`.
impl From<CanonError> for CanonReadError {
    fn from(e: CanonError) -> Self {
        match e {
            CanonError::Policy(v) => CanonReadError::Policy(v),
            CanonError::DuplicateField(n) | CanonError::DuplicateMapKey(n) => {
                CanonReadError::UnknownField(n)
            }
        }
    }
}

/// A cursor over canonical bytes that [`CanonRead`] implementations pull from.
/// Callers normally go through [`read_strict_canonical`] rather than driving it.
pub struct Reader<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Open a cursor at the first byte of `input`.
    pub fn new(input: &'a [u8]) -> Self {
        Reader { input, pos: 0 }
    }

    /// The next byte without consuming it.
    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    /// Consume and return the next byte, or fail at end of input.
    fn bump(&mut self) -> Result<u8, CanonReadError> {
        let b = self.peek().ok_or(CanonReadError::Eof)?;
        self.pos += 1;
        Ok(b)
    }

    /// Consume `b`, or fail with `ctx` naming what was expected.
    fn expect(&mut self, b: u8, ctx: &'static str) -> Result<(), CanonReadError> {
        if self.peek() == Some(b) {
            self.pos += 1;
            Ok(())
        } else {
            Err(CanonReadError::Syntax(ctx))
        }
    }

    /// True once every byte has been consumed.
    fn at_end(&self) -> bool {
        self.pos >= self.input.len()
    }
}

/// Strictly read a `T` from canonical bytes, requiring the whole input to be
/// consumed (SPEC §4.3): the inverse of [`canonical_payload_bytes`]. Any
/// deviation from the writer's encoding is rejected with a [`CanonReadError`].
pub fn read_strict_canonical<T: CanonRead>(bytes: &[u8]) -> Result<T, CanonReadError> {
    let mut r = Reader::new(bytes);
    let value = T::read(&mut r)?;
    if !r.at_end() {
        return Err(CanonReadError::Trailing);
    }
    Ok(value)
}

/// Type-guided strict reading (SPEC §4.3): the inverse of [`Canonical`]. A value
/// pulls exactly its canonical bytes from the [`Reader`], rejecting every other
/// encoding; composite values drive their fields through [`ObjectReader`].
pub trait CanonRead: Sized {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError>;
}

/// Classify a wrong leading byte where a value was expected: a bare number,
/// `null`, `true`, or `false` start is a non-canonical [`CanonReadError::Token`];
/// anything else is a structural mismatch against `expected`.
fn token_or_syntax(b: u8, expected: &'static str) -> CanonReadError {
    match b {
        b'-' | b'0'..=b'9' | b'n' | b't' | b'f' => CanonReadError::Token,
        _ => CanonReadError::Syntax(expected),
    }
}

/// Read exactly four lowercase-hex digits, returning their value.
fn read_hex4(r: &mut Reader<'_>) -> Result<u32, CanonReadError> {
    let mut cp = 0u32;
    for _ in 0..4 {
        let v = match r.bump()? {
            d @ b'0'..=b'9' => d - b'0',
            d @ b'a'..=b'f' => d - b'a' + 10,
            _ => return Err(CanonReadError::Str("escape needs lowercase hex")),
        };
        cp = cp * 16 + u32::from(v);
    }
    Ok(cp)
}

/// Read a canonical JSON string `"…"` and return its decoded text, accepting
/// only the writer's minimal escaping: `\"`, `\\`, and `\u00xx` (lowercase hex)
/// for U+0000..U+001F. Shorthand escapes (`\n`), `\u` escapes for scalars that
/// pass through raw, uppercase hex, unescaped control bytes, and invalid UTF-8
/// are each rejected — one representation per string.
pub fn read_string(r: &mut Reader<'_>) -> Result<String, CanonReadError> {
    match r.peek() {
        Some(b'"') => r.pos += 1,
        Some(b) => return Err(token_or_syntax(b, "\"")),
        None => return Err(CanonReadError::Eof),
    }
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match r.bump()? {
            b'"' => break,
            b'\\' => match r.bump()? {
                b'"' => buf.push(b'"'),
                b'\\' => buf.push(b'\\'),
                b'u' => {
                    let cp = read_hex4(r)?;
                    if cp > 0x1f {
                        return Err(CanonReadError::Str("non-minimal \\u escape"));
                    }
                    buf.push(cp as u8);
                }
                _ => return Err(CanonReadError::Str("unknown escape")),
            },
            0x00..=0x1f => return Err(CanonReadError::Str("unescaped control byte")),
            b => buf.push(b),
        }
    }
    String::from_utf8(buf).map_err(|_| CanonReadError::Str("invalid UTF-8"))
}

/// Read a canonical integer: a string token holding a canonical decimal (SPEC
/// §4.3 integers are decimal strings, never bare JSON numbers). A leading zero,
/// `+`, or `-0` fails [`CanonReadError::Integer`]; a bare number fails
/// [`CanonReadError::Token`].
pub fn read_int(r: &mut Reader<'_>) -> Result<BigInt, CanonReadError> {
    let s = read_string(r)?;
    let value: BigInt = s.parse().map_err(|_| CanonReadError::Integer(s.clone()))?;
    if value.to_string() != s {
        return Err(CanonReadError::Integer(s));
    }
    Ok(value)
}

/// Read a canonical string and confirm it is already normalized under `policy`
/// (SPEC §4.3 strings carry their declared policy's bytes). A string whose
/// normalization differs, or which fails `identifier_ascii`'s grammar, is
/// rejected.
pub fn read_string_policy(
    r: &mut Reader<'_>,
    policy: StringPolicy,
) -> Result<String, CanonReadError> {
    let s = read_string(r)?;
    let normalized = policy.normalize(&s)?;
    if normalized != s {
        return Err(CanonReadError::Unnormalized(policy));
    }
    Ok(s)
}

impl CanonRead for Id {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        Ok(Id::new(read_string(r)?)?)
    }
}

impl CanonRead for Hash {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        Ok(Hash::new(read_string(r)?)?)
    }
}

impl CanonRead for BigInt {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        read_int(r)
    }
}

/// Inverse of the [`Canonical`] impl: read `{"den":…,"num":…}`, then confirm the
/// parts are exact-reduced with a positive denominator — re-deriving the rational
/// must reproduce the very parts read, else the bytes were a non-reduced or
/// negative-denominator encoding the writer never emits.
impl CanonRead for Rational {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let den = obj.member("den", read_int)?;
        let num = obj.member("num", read_int)?;
        obj.close()?;
        let (num_s, den_s) = (num.to_string(), den.to_string());
        let rational = Rational::new(num, den)?;
        if rational.numer().to_string() != num_s || rational.denom().to_string() != den_s {
            return Err(CanonReadError::RationalNotReduced);
        }
        Ok(rational)
    }
}

/// Reader-side mirror of [`ObjectEmitter`]: a type pulls its fields by name in
/// ascending byte order via [`member`](Self::member)/[`optional`](Self::optional),
/// then [`close`](Self::close)s. Because canonical fields are byte-sorted and a
/// type requests them in that order, any out-of-order, duplicate, or unknown
/// field appears where none is expected and is rejected.
pub struct ObjectReader<'a, 'r> {
    r: &'r mut Reader<'a>,
    started: bool,
}

impl<'a, 'r> ObjectReader<'a, 'r> {
    /// Consume the opening `{`.
    pub fn open(r: &'r mut Reader<'a>) -> Result<Self, CanonReadError> {
        r.expect(b'{', "{")?;
        Ok(ObjectReader { r, started: false })
    }

    /// Decoded name of the next member, leaving the cursor unmoved; `None` at the
    /// closing `}`.
    fn peek_name(&mut self) -> Result<Option<String>, CanonReadError> {
        let save = self.r.pos;
        if self.started {
            match self.r.peek() {
                Some(b'}') => return Ok(None),
                Some(b',') => self.r.pos += 1,
                _ => return Err(CanonReadError::Syntax(", or }")),
            }
        } else if self.r.peek() == Some(b'}') {
            return Ok(None);
        }
        let name = read_string(self.r)?;
        self.r.expect(b':', ":")?;
        self.r.pos = save;
        Ok(Some(name))
    }

    /// Consume the next member's separator, name, and `:`, returning the name.
    fn take_name(&mut self) -> Result<String, CanonReadError> {
        if self.started {
            self.r.expect(b',', ",")?;
        }
        let name = read_string(self.r)?;
        self.r.expect(b':', ":")?;
        self.started = true;
        Ok(name)
    }

    /// Read the required field `name`, which must be next in ascending order.
    pub fn member<T>(
        &mut self,
        name: &'static str,
        read: impl FnOnce(&mut Reader<'_>) -> Result<T, CanonReadError>,
    ) -> Result<T, CanonReadError> {
        match self.peek_name()? {
            Some(n) if n == name => {
                self.take_name()?;
                read(self.r)
            }
            _ => Err(CanonReadError::MissingField(name)),
        }
    }

    /// Read the optional field `name`, returning `None` when absent (SPEC §4.3
    /// optional-omit). A field sorting before `name` that was never consumed is
    /// an unknown field.
    pub fn optional<T>(
        &mut self,
        name: &'static str,
        read: impl FnOnce(&mut Reader<'_>) -> Result<T, CanonReadError>,
    ) -> Result<Option<T>, CanonReadError> {
        match self.peek_name()? {
            Some(n) if n == name => {
                self.take_name()?;
                Ok(Some(read(self.r)?))
            }
            Some(n) if n.as_str() < name => Err(CanonReadError::UnknownField(n)),
            _ => Ok(None),
        }
    }

    /// Consume the closing `}`, rejecting any unexpected trailing field.
    pub fn close(mut self) -> Result<(), CanonReadError> {
        match self.peek_name()? {
            None => self.r.expect(b'}', "}"),
            Some(n) => Err(CanonReadError::UnknownField(n)),
        }
    }
}

/// Shared `[…]` framing: read elements with `read` until `]`, requiring single
/// `,` separators and rejecting a trailing comma.
fn read_bracketed<T>(
    r: &mut Reader<'_>,
    mut read: impl FnMut(&mut Reader<'_>) -> Result<T, CanonReadError>,
) -> Result<Vec<T>, CanonReadError> {
    r.expect(b'[', "[")?;
    let mut items = Vec::new();
    if r.peek() == Some(b']') {
        r.pos += 1;
        return Ok(items);
    }
    loop {
        items.push(read(r)?);
        match r.bump()? {
            b']' => break,
            b',' => {}
            _ => return Err(CanonReadError::Syntax(", or ]")),
        }
    }
    Ok(items)
}

/// Read a canonical array `[…]`, each element via `T::read`, preserving order
/// (SPEC §4.3 arrays keep their semantic order).
pub fn read_array<T: CanonRead>(r: &mut Reader<'_>) -> Result<Vec<T>, CanonReadError> {
    read_bracketed(r, T::read)
}

/// Read a canonical set: an array strictly ascending by [`canonical_sort_key`]
/// with no repeats (SPEC §4.3). The writer's [`emit_set`] sorts and dedups, so
/// any mis-ordering or duplicate is non-canonical and rejected.
pub fn read_set<T: CanonRead + Canonical>(r: &mut Reader<'_>) -> Result<Vec<T>, CanonReadError> {
    let mut prev: Option<Vec<u8>> = None;
    read_bracketed(r, |r| {
        let item = T::read(r)?;
        let key = canonical_sort_key(&item)?;
        if prev.as_ref().is_some_and(|p| key <= *p) {
            return Err(CanonReadError::Unsorted);
        }
        prev = Some(key);
        Ok(item)
    })
}

/// Read a canonical map (SPEC §4.3), the inverse of [`emit_map`]. An
/// identifier_ascii key type reads the object form (member names are the keys);
/// any other key type reads the pair-array form `[{"key":K,"value":V},…]`. Keys
/// must be strictly ascending by [`canonical_sort_key`] in both forms, so
/// mis-ordering and duplicate keys are rejected.
pub fn read_map<K: MapKey + CanonRead, V: CanonRead>(
    r: &mut Reader<'_>,
) -> Result<Vec<(K, V)>, CanonReadError> {
    let mut prev: Option<Vec<u8>> = None;
    if K::IDENTIFIER_ASCII {
        r.expect(b'{', "{")?;
        let mut entries = Vec::new();
        if r.peek() == Some(b'}') {
            r.pos += 1;
            return Ok(entries);
        }
        loop {
            let key = K::read(r)?;
            let key_bytes = canonical_sort_key(&key)?;
            if prev.as_ref().is_some_and(|p| key_bytes <= *p) {
                return Err(CanonReadError::Unsorted);
            }
            prev = Some(key_bytes);
            r.expect(b':', ":")?;
            let value = V::read(r)?;
            entries.push((key, value));
            match r.bump()? {
                b'}' => break,
                b',' => {}
                _ => return Err(CanonReadError::Syntax(", or }")),
            }
        }
        Ok(entries)
    } else {
        read_bracketed(r, |r| {
            let mut obj = ObjectReader::open(r)?;
            let key: K = obj.member("key", K::read)?;
            let value: V = obj.member("value", V::read)?;
            obj.close()?;
            let key_bytes = canonical_sort_key(&key)?;
            if prev.as_ref().is_some_and(|p| key_bytes <= *p) {
                return Err(CanonReadError::Unsorted);
            }
            prev = Some(key_bytes);
            Ok((key, value))
        })
    }
}

/// Read a canonical tagged union `{"tag":…,"value":…}` (SPEC §4.3), the inverse
/// of [`emit_union`]: exactly the members `tag` (an identifier_ascii string) then
/// `value`. `read_value` receives the decoded tag to pick the payload reader.
/// Any other shape — missing, extra, reordered, or misnamed members, or a
/// non-identifier tag — is rejected.
pub fn read_union<V>(
    r: &mut Reader<'_>,
    read_value: impl FnOnce(&str, &mut Reader<'_>) -> Result<V, CanonReadError>,
) -> Result<(String, V), CanonReadError> {
    let mut obj = ObjectReader::open(r)?;
    let tag = obj.member("tag", |r| {
        read_string_policy(r, StringPolicy::IdentifierAscii)
    })?;
    let value = obj.member("value", |r| read_value(&tag, r))?;
    obj.close()?;
    Ok((tag, value))
}

/// A runtime-evidence or extractor-emitted string carried as raw bytes — no
/// §4.2 policy normalization on write, none enforced on read. Crate-internal
/// wrapper so raw strings can ride the [`Canonical`]/[`CanonRead`] generics
/// (map values, set elements).
pub(crate) struct RawText(pub(crate) String);

impl Canonical for RawText {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string(out, &self.0);
        Ok(())
    }
}

impl CanonRead for RawText {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        Ok(RawText(read_string(r)?))
    }
}

/// Emit a `u64` as the §4.3 decimal-string integer.
pub(crate) fn emit_u64(out: &mut Vec<u8>, value: u64) {
    emit_int(out, &BigInt::from(value));
}

/// Read a canonical integer and bound it to `u64`; negative or oversized
/// values surface as [`CanonReadError::Integer`].
pub(crate) fn read_u64(r: &mut Reader<'_>) -> Result<u64, CanonReadError> {
    let n = read_int(r)?;
    u64::try_from(&n).map_err(|_| CanonReadError::Integer(n.to_string()))
}

/// Emit an `i64` as the §4.3 decimal-string integer.
pub(crate) fn emit_i64(out: &mut Vec<u8>, value: i64) {
    emit_int(out, &BigInt::from(value));
}

/// Read a canonical integer and bound it to `i64`; oversized values surface
/// as [`CanonReadError::Integer`].
pub(crate) fn read_i64(r: &mut Reader<'_>) -> Result<i64, CanonReadError> {
    let n = read_int(r)?;
    i64::try_from(&n).map_err(|_| CanonReadError::Integer(n.to_string()))
}

/// Emit `entries` as a §4.3 map of identifier keys to raw-text values.
pub(crate) fn emit_raw_map(out: &mut Vec<u8>, entries: &[(Id, String)]) -> Result<(), CanonError> {
    let texts: Vec<RawText> = entries.iter().map(|(_, v)| RawText(v.clone())).collect();
    emit_map(out, entries.iter().map(|(k, _)| k).zip(&texts))
}

/// Inverse of [`emit_raw_map`].
pub(crate) fn read_raw_map(r: &mut Reader<'_>) -> Result<Vec<(Id, String)>, CanonReadError> {
    let entries = read_map::<Id, RawText>(r)?;
    Ok(entries.into_iter().map(|(k, v)| (k, v.0)).collect())
}

/// A canonical unsigned counter: the §4.3 decimal-string integer constrained
/// to `u64`. Crate-internal wrapper so counters ride the map generics.
pub(crate) struct Count(pub(crate) u64);

impl Canonical for Count {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_u64(out, self.0);
        Ok(())
    }
}

impl CanonRead for Count {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        Ok(Count(read_u64(r)?))
    }
}

/// Emit `entries` as a §4.3 map of identifier keys to `u64` counter values
/// (event `resource_counters`, plan `budget`, report `diagnostics_summary`).
pub fn emit_u64_map(out: &mut Vec<u8>, entries: &[(Id, u64)]) -> Result<(), CanonError> {
    let counts: Vec<Count> = entries.iter().map(|&(_, v)| Count(v)).collect();
    emit_map(out, entries.iter().map(|(k, _)| k).zip(&counts))
}

/// Inverse of [`emit_u64_map`].
pub fn read_u64_map(r: &mut Reader<'_>) -> Result<Vec<(Id, u64)>, CanonReadError> {
    let entries = read_map::<Id, Count>(r)?;
    Ok(entries.into_iter().map(|(k, v)| (k, v.0)).collect())
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
            emit_string_policy(b, StringPolicy::RenderedText, "1")
        })
        .unwrap();
        o.member("dup", |b| {
            emit_string_policy(b, StringPolicy::RenderedText, "2")
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
    /// object, so [`emit_map`] must fall to the SPEC §4.3 pair-array form.
    #[derive(Debug, PartialEq)]
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

    #[test]
    fn unions_emit_exactly_tag_and_value_byte_ordered() {
        // SPEC §4.3: a sum value is the tagged object {"tag","value"} with
        // exactly two members, byte-ordered by ObjectEmitter ("tag" before
        // "value"). A composite payload is emitted through its Canonical impl.
        assert_eq!(
            emitted(
                |o| emit_union(o, "rational", &Rational::from_parts("2", "4").unwrap()).unwrap()
            ),
            r#"{"tag":"rational","value":{"den":"2","num":"1"}}"#
        );
        // A scalar payload and a punctuated identifier_ascii tag pass through.
        assert_eq!(
            emitted(|o| emit_union(o, "ir.doc_segment", &BigInt::from(7)).unwrap()),
            r#"{"tag":"ir.doc_segment","value":"7"}"#
        );
    }

    #[test]
    fn union_tag_must_be_identifier_ascii() {
        // The tag is normalized through identifier_ascii; an out-of-grammar tag
        // (uppercase and space) is rejected before any bytes are emitted.
        let mut out = Vec::new();
        assert!(matches!(
            emit_union(&mut out, "Bad Tag", &BigInt::from(1)),
            Err(CanonError::Policy(ValidationError::StringPolicy(_)))
        ));
    }

    // -----------------------------------------------------------------------
    // Strict reader (SPEC §4.3) — the writer's inverse
    // -----------------------------------------------------------------------

    /// Read a `T` from the whole of `bytes`, requiring full consumption: the
    /// free-function analogue of [`read_strict_canonical`] for the collection, union,
    /// and policy-string readers that are not [`CanonRead`] impls.
    fn read_all<T>(
        bytes: &[u8],
        read: impl FnOnce(&mut Reader<'_>) -> Result<T, CanonReadError>,
    ) -> Result<T, CanonReadError> {
        let mut r = Reader::new(bytes);
        let value = read(&mut r)?;
        if !r.at_end() {
            return Err(CanonReadError::Trailing);
        }
        Ok(value)
    }

    /// Assert `value` survives a canonical write -> read round trip unchanged.
    fn round_trip<T: Canonical + CanonRead + std::fmt::Debug + PartialEq>(value: T) {
        let bytes = canonical_payload_bytes(&value).unwrap();
        let got: T = read_strict_canonical(&bytes).unwrap();
        assert_eq!(got, value, "round trip changed the value");
    }

    impl CanonRead for Span {
        fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
            let mut obj = ObjectReader::open(r)?;
            let hi = obj.member("hi", read_int)?;
            let lo = obj.member("lo", read_int)?;
            obj.close()?;
            Ok(Span {
                lo: lo.to_string().parse().unwrap(),
                hi: hi.to_string().parse().unwrap(),
            })
        }
    }

    /// A record with a required and an optional field, exercising the
    /// [`ObjectEmitter`]/[`ObjectReader`] member/optional symmetry.
    #[derive(Debug, PartialEq)]
    struct Rec {
        count: BigInt,
        label: Option<Id>,
    }

    impl Canonical for Rec {
        fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
            let mut o = ObjectEmitter::new();
            o.member("count", |b| {
                emit_int(b, &self.count);
                Ok(())
            })?;
            o.optional("label", self.label.as_ref(), |b, v: &Id| {
                v.emit_canonical(b)
            })?;
            o.finish(out)
        }
    }

    impl CanonRead for Rec {
        fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
            let mut obj = ObjectReader::open(r)?;
            let count = obj.member("count", read_int)?;
            let label = obj.optional("label", Id::read)?;
            obj.close()?;
            Ok(Rec { count, label })
        }
    }

    #[test]
    fn round_trips_scalars_and_records() {
        round_trip(Id::new("pipe.layered_ckcir_to_smt").unwrap());
        round_trip(
            Hash::new("sha256:0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        );
        for n in [0_i64, -42, 1_000_000] {
            round_trip(BigInt::from(n));
        }
        round_trip("123456789012345678901234567890".parse::<BigInt>().unwrap());
        round_trip(Rational::from_parts("2", "4").unwrap());
        round_trip(Rational::from_parts("3", "-2").unwrap());
        round_trip(Rec {
            count: BigInt::from(7),
            label: Some(Id::new("pipe.x").unwrap()),
        });
        round_trip(Rec {
            count: BigInt::from(0),
            label: None,
        });
    }

    #[test]
    fn round_trips_collections_and_unions() {
        // array — semantic order preserved verbatim
        let arr = vec![
            Id::new("c").unwrap(),
            Id::new("a").unwrap(),
            Id::new("b").unwrap(),
        ];
        let mut bytes = Vec::new();
        emit_array(&mut bytes, &arr).unwrap();
        assert_eq!(read_all(&bytes, read_array::<Id>).unwrap(), arr);

        // set — writer sorts and dedups; the canonical bytes read back ascending
        let set = [
            Id::new("c").unwrap(),
            Id::new("a").unwrap(),
            Id::new("b").unwrap(),
            Id::new("a").unwrap(),
        ];
        let mut bytes = Vec::new();
        emit_set(&mut bytes, &set).unwrap();
        assert_eq!(
            read_all(&bytes, read_set::<Id>).unwrap(),
            vec![
                Id::new("a").unwrap(),
                Id::new("b").unwrap(),
                Id::new("c").unwrap(),
            ]
        );

        // identifier-keyed map — object form, members ascending by key
        let imap = [
            (Id::new("b").unwrap(), BigInt::from(2)),
            (Id::new("a").unwrap(), BigInt::from(1)),
        ];
        let mut bytes = Vec::new();
        emit_map(&mut bytes, imap.iter().map(|(k, v)| (k, v))).unwrap();
        assert_eq!(
            read_all(&bytes, read_map::<Id, BigInt>).unwrap(),
            vec![
                (Id::new("a").unwrap(), BigInt::from(1)),
                (Id::new("b").unwrap(), BigInt::from(2)),
            ]
        );

        // structured-keyed map — pair-array form, pairs ascending by key bytes
        let smap = [
            (Span { lo: 5, hi: 9 }, Id::new("y").unwrap()),
            (Span { lo: 1, hi: 2 }, Id::new("x").unwrap()),
        ];
        let mut bytes = Vec::new();
        emit_map(&mut bytes, smap.iter().map(|(k, v)| (k, v))).unwrap();
        assert_eq!(
            read_all(&bytes, read_map::<Span, Id>).unwrap(),
            vec![
                (Span { lo: 1, hi: 2 }, Id::new("x").unwrap()),
                (Span { lo: 5, hi: 9 }, Id::new("y").unwrap()),
            ]
        );

        // union — the decoded tag selects the payload reader
        let mut bytes = Vec::new();
        emit_union(
            &mut bytes,
            "rational",
            &Rational::from_parts("2", "4").unwrap(),
        )
        .unwrap();
        let (tag, value) = read_all(&bytes, |r| read_union(r, |_, r| Rational::read(r))).unwrap();
        assert_eq!(tag, "rational");
        assert_eq!(value, Rational::from_parts("1", "2").unwrap());
    }

    #[test]
    fn rejects_trailing_bytes_null_and_bare_numbers() {
        // bytes left over after a complete value
        assert_eq!(
            read_strict_canonical::<Id>(br#""a"x"#),
            Err(CanonReadError::Trailing)
        );
        // null / booleans are never canonical
        assert_eq!(
            read_strict_canonical::<Id>(b"null"),
            Err(CanonReadError::Token)
        );
        // a bare number where an integer string is required
        assert_eq!(
            read_strict_canonical::<BigInt>(b"42"),
            Err(CanonReadError::Token)
        );
        // bare numbers inside a rational object (parts are integer strings)
        assert_eq!(
            read_strict_canonical::<Rational>(br#"{"den":4,"num":2}"#),
            Err(CanonReadError::Token)
        );
    }

    #[test]
    fn rejects_non_canonical_strings() {
        // a JSON shorthand escape (canonical form uses the six-byte u-escape)
        assert!(matches!(
            read_strict_canonical::<Id>(&[b'"', 0x5c, b'n', b'"']),
            Err(CanonReadError::Str(_))
        ));
        // a u-escape for a scalar that must pass through raw (uppercase 'A')
        assert!(matches!(
            read_strict_canonical::<Id>(&[b'"', 0x5c, b'u', b'0', b'0', b'4', b'1', b'"']),
            Err(CanonReadError::Str(_))
        ));
        // a u-escape written with uppercase hex digits
        assert!(matches!(
            read_strict_canonical::<Id>(&[b'"', 0x5c, b'u', b'0', b'0', b'0', b'A', b'"']),
            Err(CanonReadError::Str(_))
        ));
        // an unescaped control byte
        assert!(matches!(
            read_strict_canonical::<Id>(&[b'"', 0x01, b'"']),
            Err(CanonReadError::Str(_))
        ));
        // invalid UTF-8
        assert!(matches!(
            read_strict_canonical::<Id>(&[b'"', 0xff, b'"']),
            Err(CanonReadError::Str(_))
        ));
    }

    #[test]
    fn rejects_non_canonical_integers_and_rationals() {
        // leading zero, explicit +, and -0 are non-canonical decimals
        for s in [r#""007""#, r#""+1""#, r#""-0""#] {
            assert!(matches!(
                read_strict_canonical::<BigInt>(s.as_bytes()),
                Err(CanonReadError::Integer(_))
            ));
        }
        // 2/4 reduces to 1/2, so its written parts are non-canonical
        assert_eq!(
            read_strict_canonical::<Rational>(br#"{"den":"4","num":"2"}"#),
            Err(CanonReadError::RationalNotReduced)
        );
        // a negative denominator is non-canonical (sign rides the numerator)
        assert_eq!(
            read_strict_canonical::<Rational>(br#"{"den":"-2","num":"1"}"#),
            Err(CanonReadError::RationalNotReduced)
        );
    }

    #[test]
    fn rejects_object_field_violations() {
        // a field no type expects (also how dupes / mis-order surface)
        assert_eq!(
            read_strict_canonical::<Rec>(br#"{"count":"1","zzz":"2"}"#),
            Err(CanonReadError::UnknownField("zzz".to_string()))
        );
        // a required field absent (only the optional present)
        assert_eq!(
            read_strict_canonical::<Rec>(br#"{"label":"x"}"#),
            Err(CanonReadError::MissingField("count"))
        );
    }

    #[test]
    fn rejects_set_and_map_misordering() {
        // set elements descending
        assert_eq!(
            read_all(br#"["b","a"]"#, read_set::<Id>),
            Err(CanonReadError::Unsorted)
        );
        // a repeated set element
        assert_eq!(
            read_all(br#"["a","a"]"#, read_set::<Id>),
            Err(CanonReadError::Unsorted)
        );
        // map keys descending
        assert_eq!(
            read_all(br#"{"b":"1","a":"2"}"#, read_map::<Id, BigInt>),
            Err(CanonReadError::Unsorted)
        );
    }

    #[test]
    fn rejects_malformed_unions() {
        // value member missing
        assert_eq!(
            read_all(br#"{"tag":"x"}"#, |r| read_union(r, |_, r| BigInt::read(r))),
            Err(CanonReadError::MissingField("value"))
        );
        // an extra member after value
        assert_eq!(
            read_all(br#"{"tag":"x","value":"1","z":"2"}"#, |r| read_union(
                r,
                |_, r| BigInt::read(r)
            )),
            Err(CanonReadError::UnknownField("z".to_string()))
        );
        // members reordered: tag is not first
        assert_eq!(
            read_all(br#"{"value":"1","tag":"x"}"#, |r| read_union(r, |_, r| {
                BigInt::read(r)
            })),
            Err(CanonReadError::MissingField("tag"))
        );
        // a tag outside identifier_ascii
        assert!(matches!(
            read_all(br#"{"tag":"X","value":"1"}"#, |r| read_union(r, |_, r| {
                BigInt::read(r)
            })),
            Err(CanonReadError::Policy(ValidationError::StringPolicy(_)))
        ));
    }

    #[test]
    fn rejects_unnormalized_policy_strings() {
        // semantic_en would fold and trim this, so the raw bytes are non-canonical
        assert_eq!(
            read_all(br#""  Hi  ""#, |r| read_string_policy(
                r,
                StringPolicy::SemanticEn
            )),
            Err(CanonReadError::Unnormalized(StringPolicy::SemanticEn))
        );
        // identifier_ascii rejects the space outright
        assert!(matches!(
            read_all(br#""Bad Id""#, |r| read_string_policy(
                r,
                StringPolicy::IdentifierAscii
            )),
            Err(CanonReadError::Policy(ValidationError::StringPolicy(_)))
        ));
    }
}
