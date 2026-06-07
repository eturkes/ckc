//! SPEC §1.5 canonical JSON payload bytes: writer core.
//!
//! Single emission authority for canonical bytes (§0 one canonical
//! representation per fact): artifact bytes are produced here; serde
//! `Deserialize` impls are the validating read side. `TYPE_ID` values are
//! provisional symbol ids pending the M0.0.3 `SchemaRegistry`. M0.0.2.2
//! adds `canonical_sort_key` and Set/List/Map encodings; M0.0.2.3 adds
//! tagged unions and strict canonical reading.

use std::fmt;

use crate::policy::{PolicyMarker, StringPolicy, Text, UnicodePolicyManifest};
use crate::scalar::{Hash, Id, Int, Rational, UInt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonError {
    DuplicateField { name: String },
}

impl fmt::Display for CanonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateField { name } => write!(f, "duplicate object field: {name:?}"),
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

    pub fn finish(mut self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        self.members.sort_by(|a, b| a.0.cmp(&b.0));
        if let Some(w) = self.members.windows(2).find(|w| w[0].0 == w[1].0) {
            return Err(CanonError::DuplicateField {
                name: w[0].0.clone(),
            });
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
