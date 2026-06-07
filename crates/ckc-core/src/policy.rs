//! SPEC §1.4 string policies, the `Text<P>` dependent text type, and
//! `UnicodePolicyManifest` fingerprints.
//!
//! Construction (`Text::new`) normalizes; ingestion (`Text::from_canonical`,
//! serde `Deserialize`) validates that bytes are already normalized and
//! rejects non-canonical input. Every policy is idempotent and byte-stable
//! (gate `T-Unicode-Idempotency`).

use std::fmt;
use std::marker::PhantomData;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use unicode_normalization::UnicodeNormalization;

use crate::scalar::{Hash, Id};

// ---------------------------------------------------------------------------
// Fold tables (§1.4 semantic_ja)
// ---------------------------------------------------------------------------

/// Whitespace code points folded to U+0020, sorted ascending. NFKC already
/// folds U+00A0, U+2000..U+200A, U+202F, U+205F, and U+3000; they stay listed
/// so the table matches §1.4 verbatim and folds pre-NFKC residue defensively.
const WHITESPACE_FOLD: &[char] = &[
    '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{00A0}', '\u{1680}', '\u{2000}',
    '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}',
    '\u{2009}', '\u{200A}', '\u{2028}', '\u{2029}', '\u{202F}', '\u{205F}', '\u{3000}',
];

/// Punctuation fold applied after NFKC, sorted ascending by code point.
/// Entries NFKC already maps to ASCII (U+FF0C, U+FF0E, …) stay listed for
/// the same verbatim-table reason.
const PUNCT_FOLD: &[(char, &str)] = &[
    ('\u{2010}', "-"),
    ('\u{2011}', "-"),
    ('\u{2012}', "-"),
    ('\u{2013}', "-"),
    ('\u{2014}', "-"),
    ('\u{2015}', "-"),
    ('\u{2212}', "-"),
    ('\u{2264}', "<="),
    ('\u{2265}', ">="),
    ('\u{2266}', "<="),
    ('\u{2267}', ">="),
    ('\u{3001}', ","),
    ('\u{3002}', "."),
    ('\u{3010}', "["),
    ('\u{3011}', "]"),
    ('\u{FF08}', "("),
    ('\u{FF09}', ")"),
    ('\u{FF0C}', ","),
    ('\u{FF0D}', "-"),
    ('\u{FF0E}', "."),
    ('\u{FF1A}', ":"),
    ('\u{FF1B}', ";"),
    ('\u{FF1C}', "<"),
    ('\u{FF1E}', ">"),
    ('\u{FF3B}', "["),
    ('\u{FF3D}', "]"),
];

fn nfkc(input: &str) -> String {
    input.nfkc().collect()
}

fn fold_whitespace(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if WHITESPACE_FOLD.binary_search(&c).is_ok() {
                ' '
            } else {
                c
            }
        })
        .collect()
}

fn fold_punct(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match PUNCT_FOLD.binary_search_by_key(&c, |&(k, _)| k) {
            Ok(i) => out.push_str(PUNCT_FOLD[i].1),
            Err(_) => out.push(c),
        }
    }
    out
}

/// Collapse each maximal run of U+0020 to one U+0020, then trim leading and
/// trailing U+0020 (§1.4 semantic_ja).
fn collapse_and_trim_spaces(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_space = false;
    for c in input.chars() {
        if c == ' ' {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim_matches(' ').to_owned()
}

fn semantic_fold(input: &str) -> String {
    collapse_and_trim_spaces(&fold_punct(&fold_whitespace(&nfkc(input))))
}

fn diagnostic_fold(input: &str) -> String {
    collapse_and_trim_spaces(&fold_whitespace(&nfkc(input)))
}

/// §1.4 semantic_en lowercases ASCII letters only inside
/// controlled-vocabulary identifier fields; those field sites apply this
/// helper after `semantic_en` normalization. ASCII lowercase output is
/// NFKC- and fold-stable, so the composition stays idempotent.
pub fn ascii_lowercase_controlled_vocab(input: &str) -> String {
    input.to_ascii_lowercase()
}

fn is_identifier_ascii(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b':' | b'.' | b'/' | b'-'))
}

// ---------------------------------------------------------------------------
// StringPolicy
// ---------------------------------------------------------------------------

/// §1.4 `E StringPolicy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringPolicy {
    RawSource,
    SourceNfkc,
    SemanticJa,
    SemanticEn,
    IdentifierAscii,
    TemplateLiteral,
    DiagnosticText,
    ViewText,
}

impl StringPolicy {
    pub const ALL: [Self; 8] = [
        Self::RawSource,
        Self::SourceNfkc,
        Self::SemanticJa,
        Self::SemanticEn,
        Self::IdentifierAscii,
        Self::TemplateLiteral,
        Self::DiagnosticText,
        Self::ViewText,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::RawSource => "raw_source",
            Self::SourceNfkc => "source_nfkc",
            Self::SemanticJa => "semantic_ja",
            Self::SemanticEn => "semantic_en",
            Self::IdentifierAscii => "identifier_ascii",
            Self::TemplateLiteral => "template_literal",
            Self::DiagnosticText => "diagnostic_text",
            Self::ViewText => "view_text",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.id() == id)
    }

    /// Apply the policy algorithm (§1.4). `template_literal` gloss-template
    /// grammar validation lands with §7.5 at M0.4.8; until then the policy
    /// normalizes via NFKC, which preserves slot markers.
    pub fn normalize(self, input: &str) -> Result<String, PolicyError> {
        match self {
            Self::RawSource => Ok(input.to_owned()),
            Self::SourceNfkc | Self::TemplateLiteral | Self::ViewText => Ok(nfkc(input)),
            Self::SemanticJa | Self::SemanticEn => Ok(semantic_fold(input)),
            Self::DiagnosticText => Ok(diagnostic_fold(input)),
            Self::IdentifierAscii => {
                if is_identifier_ascii(input) {
                    Ok(input.to_owned())
                } else {
                    Err(PolicyError {
                        policy: self,
                        kind: PolicyErrorKind::IdentifierAsciiCharset,
                    })
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyError {
    pub policy: StringPolicy,
    pub kind: PolicyErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyErrorKind {
    /// Input violates `[a-z0-9_:./-]+`.
    IdentifierAsciiCharset,
    /// Ingested bytes differ from their policy normalization.
    NonCanonical,
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            PolicyErrorKind::IdentifierAsciiCharset => {
                write!(f, "{} charset violation", self.policy.id())
            }
            PolicyErrorKind::NonCanonical => {
                write!(f, "{} non-canonical bytes", self.policy.id())
            }
        }
    }
}

impl std::error::Error for PolicyError {}

// ---------------------------------------------------------------------------
// Text<P>
// ---------------------------------------------------------------------------

/// Marker types tying a `Text` field to its declared policy.
pub trait PolicyMarker: 'static {
    const POLICY: StringPolicy;
}

macro_rules! policy_markers {
    ($($marker:ident => $variant:ident;)+) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $marker;
            impl PolicyMarker for $marker {
                const POLICY: StringPolicy = StringPolicy::$variant;
            }
        )+
    };
}

policy_markers! {
    RawSource => RawSource;
    SourceNfkc => SourceNfkc;
    SemanticJa => SemanticJa;
    SemanticEn => SemanticEn;
    IdentifierAscii => IdentifierAscii;
    TemplateLiteral => TemplateLiteral;
    DiagnosticText => DiagnosticText;
    ViewText => ViewText;
}

/// `Text<P>`: UTF-8 string normalized by `StringPolicy` `P` (§1.3).
pub struct Text<P: PolicyMarker> {
    value: String,
    _policy: PhantomData<fn() -> P>,
}

impl<P: PolicyMarker> Text<P> {
    /// Normalize `input` under `P`.
    pub fn new(input: &str) -> Result<Self, PolicyError> {
        Ok(Self {
            value: P::POLICY.normalize(input)?,
            _policy: PhantomData,
        })
    }

    /// Accept already-canonical bytes; reject input its policy would change.
    pub fn from_canonical(input: &str) -> Result<Self, PolicyError> {
        let normalized = P::POLICY.normalize(input)?;
        if normalized == input {
            Ok(Self {
                value: normalized,
                _policy: PhantomData,
            })
        } else {
            Err(PolicyError {
                policy: P::POLICY,
                kind: PolicyErrorKind::NonCanonical,
            })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_string(self) -> String {
        self.value
    }

    pub fn policy() -> StringPolicy {
        P::POLICY
    }
}

impl<P: PolicyMarker> fmt::Debug for Text<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Text<{}>({:?})", P::POLICY.id(), self.value)
    }
}

impl<P: PolicyMarker> Clone for Text<P> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _policy: PhantomData,
        }
    }
}

impl<P: PolicyMarker> PartialEq for Text<P> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<P: PolicyMarker> Eq for Text<P> {}

impl<P: PolicyMarker> PartialOrd for Text<P> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<P: PolicyMarker> Ord for Text<P> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<P: PolicyMarker> std::hash::Hash for Text<P> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<'de, P: PolicyMarker> Deserialize<'de> for Text<P> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Text::from_canonical(&s).map_err(D::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// UnicodePolicyManifest (§1.4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnicodePolicyManifest {
    pub manifest_id: Id,
    pub normalization_table_hash: Hash,
    pub policy_test_hash: Hash,
    pub punctuation_table_hash: Hash,
    pub unicode_version: Text<IdentifierAscii>,
}

/// Unicode version of the NFKC tables in use, e.g. `"16.0.0"`.
pub fn unicode_version_string() -> String {
    let (major, minor, micro) = unicode_normalization::UNICODE_VERSION;
    format!("{major}.{minor}.{micro}")
}

/// Behavioral fingerprint of the NFKC tables: hash of the canonical text
/// listing every scalar value's non-identity NFKC image over the full
/// Unicode range. Any table drift (mapping added, removed, or changed)
/// moves this hash.
pub fn normalization_table_fingerprint() -> Hash {
    use fmt::Write;
    let mut content = String::new();
    let mut buf = String::new();
    for cp in 0u32..=0x10FFFF {
        let Some(c) = char::from_u32(cp) else {
            continue; // surrogate range
        };
        buf.clear();
        buf.extend(std::iter::once(c).nfkc());
        if buf.len() == c.len_utf8() && buf.starts_with(c) {
            continue; // identity image
        }
        writeln!(content, "{cp:06x}:{buf}").expect("write to String is infallible");
    }
    Hash::of_bytes(content.as_bytes())
}

/// Fingerprint of the canonical text form of the §1.4 whitespace and
/// punctuation fold tables.
pub fn punctuation_table_fingerprint() -> Hash {
    use fmt::Write;
    let mut content = String::from("ws\n");
    for c in WHITESPACE_FOLD {
        writeln!(content, "{:06x}", *c as u32).expect("write to String is infallible");
    }
    content.push_str("punct\n");
    for (c, replacement) in PUNCT_FOLD {
        writeln!(content, "{:06x}:{replacement}", *c as u32)
            .expect("write to String is infallible");
    }
    Hash::of_bytes(content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_tables_are_sorted_for_binary_search() {
        assert!(WHITESPACE_FOLD.is_sorted());
        assert!(PUNCT_FOLD.iter().map(|&(c, _)| c).is_sorted());
    }

    #[test]
    fn identifier_ascii_charset() {
        for ok in [
            "ckc-core",
            "runs/m0",
            "1.96.0",
            "sha256:ab",
            "--out",
            "16.0.0",
        ] {
            assert!(StringPolicy::IdentifierAscii.normalize(ok).is_ok(), "{ok}");
        }
        for bad in ["", "A", "a b", "ＡＢ", "a=b", "a\n"] {
            assert!(
                StringPolicy::IdentifierAscii.normalize(bad).is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn semantic_vs_diagnostic_punctuation() {
        // semantic_ja folds U+3002; diagnostic_text preserves it.
        assert_eq!(StringPolicy::SemanticJa.normalize("あ。").unwrap(), "あ.");
        assert_eq!(
            StringPolicy::DiagnosticText.normalize("あ。").unwrap(),
            "あ。"
        );
        // Both fold whitespace runs.
        assert_eq!(
            StringPolicy::DiagnosticText
                .normalize("a\u{3000} \tb")
                .unwrap(),
            "a b"
        );
    }

    #[test]
    fn controlled_vocab_lowercase_is_ascii_only() {
        assert_eq!(ascii_lowercase_controlled_vocab("AbC-Β"), "abc-Β");
    }

    #[test]
    fn from_canonical_rejects_unnormalized_bytes() {
        assert!(Text::<SemanticJa>::from_canonical("ａ").is_err());
        assert!(Text::<SemanticJa>::from_canonical("a").is_ok());
        assert!(Text::<RawSource>::from_canonical("ａ \u{3000}").is_ok());
    }
}
