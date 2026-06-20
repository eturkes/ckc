//! The seven string policies of SPEC §4.2 as deterministic normalizers.
//!
//! A schema declares the [`StringPolicy`] each string field carries, and the
//! policy is applied before the bytes enter a content hash. Every policy is a
//! pure, deterministic `&str -> String` map; only [`StringPolicy::IdentifierAscii`]
//! can reject its input, so [`StringPolicy::normalize`] returns a uniform
//! `Result` across all variants.
//!
//! ```text
//! raw_source       identity (provenance byte hash recorded by the producer)
//! source_nfkc      NFKC
//! semantic_ja      NFKC, fold whitespace to U+0020, collapse runs, trim, fold CJK punctuation
//! semantic_en      semantic_ja, then lowercase ASCII letters only
//! identifier_ascii require non-empty [a-z0-9_:./-]+; store bytes exactly (fallible)
//! diagnostic_text  NFKC, fold whitespace to U+0020, collapse runs, trim
//! rendered_text        NFKC (renderer provenance recorded by the producer)
//! ```
//!
//! NFKC (the only nontrivial Unicode operation) is delegated to
//! `unicode-normalization`; the small CJK-punctuation fold table is owned here
//! as the deterministic source of truth (see [`PUNCT_FOLDS`]).

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::ValidationError;

/// A SPEC §4.2 string policy. Serializes to the spec's snake_case policy name so
/// it can be recorded directly in schema string-policy bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StringPolicy {
    /// Preserve the extractor-emitted Unicode scalar sequence exactly.
    RawSource,
    /// Apply Unicode NFKC only.
    SourceNfkc,
    /// NFKC, semantic whitespace folding, then CJK-punctuation folding.
    SemanticJa,
    /// [`SemanticJa`](StringPolicy::SemanticJa) followed by ASCII-only lowercasing.
    SemanticEn,
    /// Require a non-empty `[a-z0-9_:./-]+` string; store its bytes exactly.
    IdentifierAscii,
    /// NFKC plus semantic whitespace folding (no case or punctuation folding).
    DiagnosticText,
    /// NFKC display text (renderer provenance recorded separately).
    RenderedText,
}

impl StringPolicy {
    /// Normalize `input` under this policy. Infallible for every policy except
    /// [`StringPolicy::IdentifierAscii`], which rejects bytes outside its grammar.
    pub fn normalize(&self, input: &str) -> Result<String, ValidationError> {
        let out = match self {
            StringPolicy::RawSource => input.to_string(),
            StringPolicy::SourceNfkc | StringPolicy::RenderedText => nfkc(input),
            StringPolicy::DiagnosticText => fold_whitespace(&nfkc(input)),
            StringPolicy::SemanticJa => fold_punctuation(&fold_whitespace(&nfkc(input))),
            StringPolicy::SemanticEn => {
                fold_punctuation(&fold_whitespace(&nfkc(input))).to_ascii_lowercase()
            }
            StringPolicy::IdentifierAscii => return identifier_ascii(input),
        };
        Ok(out)
    }
}

/// Unicode NFKC normalization.
fn nfkc(input: &str) -> String {
    input.nfkc().collect()
}

/// Fold every Unicode whitespace scalar to U+0020, collapse runs to a single
/// space, and trim leading and trailing space. `str::split_whitespace` already
/// treats any whitespace run as one separator and drops leading/trailing runs,
/// so re-joining its tokens with a single space realizes all three steps.
fn fold_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for word in input.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    out
}

/// CJK punctuation folded to its ASCII equivalent. NFKC runs first and already
/// folds the fullwidth Latin block (e.g. U+FF0C `，` -> `,`, U+3000 ideographic
/// space -> U+0020), so this table carries only the CJK Symbols-block marks NFKC
/// leaves untouched yet which have one unambiguous ASCII counterpart. It is the
/// deterministic source of truth for `semantic_ja`/`semantic_en` and the
/// intended extension point as more marks acquire agreed ASCII equivalents.
const PUNCT_FOLDS: &[(char, char)] = &[
    ('\u{3001}', ','), // 、 ideographic comma
    ('\u{3002}', '.'), // 。 ideographic full stop
];

/// Fold the [`PUNCT_FOLDS`] marks to ASCII, leaving every other scalar intact.
fn fold_punctuation(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            PUNCT_FOLDS
                .iter()
                .find_map(|&(from, to)| (from == c).then_some(to))
                .unwrap_or(c)
        })
        .collect()
}

/// Validate the `identifier_ascii` grammar and return the bytes unchanged.
/// Distinct from [`crate::Id`]: this grammar allows a leading digit and `/` and
/// omits the leading-`[a-z]` requirement (SPEC §4.2 vs §4.1).
fn identifier_ascii(input: &str) -> Result<String, ValidationError> {
    if input.is_empty() {
        return Err(ValidationError::StringPolicy(
            "identifier_ascii: empty string".to_string(),
        ));
    }
    if let Some(c) = input
        .chars()
        .find(|&c| !matches!(c, 'a'..='z' | '0'..='9' | '_' | ':' | '.' | '/' | '-'))
    {
        return Err(ValidationError::StringPolicy(format!(
            "identifier_ascii: character {c:?} not in [a-z0-9_:./-]: {input:?}"
        )));
    }
    Ok(input.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_names_match_spec() {
        let cases = [
            (StringPolicy::RawSource, "\"raw_source\""),
            (StringPolicy::SourceNfkc, "\"source_nfkc\""),
            (StringPolicy::SemanticJa, "\"semantic_ja\""),
            (StringPolicy::SemanticEn, "\"semantic_en\""),
            (StringPolicy::IdentifierAscii, "\"identifier_ascii\""),
            (StringPolicy::DiagnosticText, "\"diagnostic_text\""),
            (StringPolicy::RenderedText, "\"rendered_text\""),
        ];
        for (policy, name) in cases {
            let json = serde_json::to_string(&policy).unwrap();
            assert_eq!(json, name);
            assert_eq!(serde_json::from_str::<StringPolicy>(&json).unwrap(), policy);
        }
    }

    #[test]
    fn raw_source_is_identity() {
        // Fullwidth digits and ideographic comma are NOT normalized away.
        let input = "Ａ１、　ｂ";
        assert_eq!(StringPolicy::RawSource.normalize(input).unwrap(), input);
    }

    #[test]
    fn source_nfkc_applies_nfkc_only() {
        // Fullwidth -> ASCII, ligature decomposition, halfwidth kana -> fullwidth.
        assert_eq!(StringPolicy::SourceNfkc.normalize("Ａ１").unwrap(), "A1");
        assert_eq!(
            StringPolicy::SourceNfkc.normalize("\u{FB01}").unwrap(),
            "fi"
        );
        // Whitespace and the ideographic comma survive (no folding beyond NFKC).
        assert_eq!(
            StringPolicy::SourceNfkc.normalize("a  、b").unwrap(),
            "a  、b"
        );
    }

    #[test]
    fn semantic_ja_runs_full_pipeline() {
        // Ideographic space + fullwidth letters + ideographic punctuation +
        // padded whitespace -> NFKC, whitespace fold/collapse/trim, punct fold.
        let input = "\u{3000}Ａ、\u{3000}Ｂ。 ";
        assert_eq!(StringPolicy::SemanticJa.normalize(input).unwrap(), "A, B.");
    }

    #[test]
    fn semantic_en_lowercases_ascii_only() {
        // ASCII letters lowered; the non-ASCII Ä stays uppercase (ASCII-only).
        let input = "ＨＥＬＬＯ、 Ä World";
        assert_eq!(
            StringPolicy::SemanticEn.normalize(input).unwrap(),
            "hello, Ä world"
        );
    }

    #[test]
    fn diagnostic_text_folds_whitespace_not_punct_or_case() {
        // Distinguishes diagnostic_text from semantic_ja: comma and case survive.
        let input = "Ａ  、\tB";
        assert_eq!(
            StringPolicy::DiagnosticText.normalize(input).unwrap(),
            "A 、 B"
        );
    }

    #[test]
    fn rendered_text_matches_source_nfkc() {
        for input in ["Ａ１", "café\u{3000}x", "\u{FB01}ne"] {
            assert_eq!(
                StringPolicy::RenderedText.normalize(input).unwrap(),
                StringPolicy::SourceNfkc.normalize(input).unwrap()
            );
        }
    }

    #[test]
    fn identifier_ascii_accepts_and_stores_exactly() {
        for s in ["a", "0", "schema.ir_bundle", "a/b", "x:y", "1abc-d_e.f/g"] {
            assert_eq!(
                StringPolicy::IdentifierAscii.normalize(s).unwrap(),
                s,
                "{s:?} should pass unchanged"
            );
        }
    }

    #[test]
    fn identifier_ascii_rejects_out_of_grammar() {
        for s in ["", "A", "café", "has space", "a*b", "под"] {
            assert!(
                matches!(
                    StringPolicy::IdentifierAscii.normalize(s),
                    Err(ValidationError::StringPolicy(_))
                ),
                "{s:?} should be rejected"
            );
        }
    }

    #[test]
    fn folding_policies_are_idempotent() {
        let input = "\u{3000}Ａ、　Ｂ。 ＣＡＴ";
        for policy in [
            StringPolicy::SemanticJa,
            StringPolicy::SemanticEn,
            StringPolicy::DiagnosticText,
        ] {
            let once = policy.normalize(input).unwrap();
            let twice = policy.normalize(&once).unwrap();
            assert_eq!(once, twice, "{policy:?} should be idempotent");
        }
    }
}
