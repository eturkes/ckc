//! Normalize stage, first half: lexicon loading (SPEC §8.3 normalize row).
//!
//! [`load_lexicon`] strict-deserializes `corpus/lexicon/ja_core.yaml` — the
//! §5 V1 terminology and modality authority (system `ckc.lex`) — and
//! validates it into the typed [`Lexicon`], content-hash versioned over its
//! raw file bytes (§4.4 raw-byte hashing) for every run manifest. Every
//! surface form is stored normalized under [`StringPolicy::SemanticJa`]
//! with `surfaces[0]` the representative, so downstream binding compares
//! pre-folded text. Validation rejects duplicate ids (one pool across
//! concepts and actions), empty or duplicate per-entry surface lists,
//! duplicate modality/certainty surfaces, and intervals breaking the §5
//! bound-coherence rule; one surface shared across concepts stays legal —
//! it is the ambiguity source mention binding (`stage-normalize.1b`)
//! consumes.

use std::collections::HashSet;
use std::fmt;

use serde::Deserialize;

use ckc_core::{
    Certainty, Direction, Hash, Id, QuantityInterval, Strength, StringPolicy, hash_bytes,
};

/// SPEC §5 V1 lexicon: the typed, validated view of
/// `corpus/lexicon/ja_core.yaml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lexicon {
    /// Terminology system the entries bind under (V1: `ckc.lex`).
    pub system: Id,
    /// [`hash_bytes`] over the raw file bytes (§4.4 raw-byte hashing),
    /// versioning the lexicon in every run manifest.
    pub content_hash: Hash,
    pub concepts: Vec<LexiconConcept>,
    pub actions: Vec<LexiconAction>,
    pub modality: Vec<LexiconModality>,
    pub certainty: Vec<LexiconCertainty>,
}

/// Concept entry: surfaces plus optional §5 interval semantics over a
/// quantity variable (成人 → `q.age_years >= 18`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconConcept {
    pub concept_id: Id,
    /// semantic_ja surface forms; `surfaces[0]` is the representative.
    pub surfaces: Vec<String>,
    pub interval: Option<QuantityInterval>,
}

/// Action-kind entry: the verb surfaces a recommendation sentence carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconAction {
    pub action_id: Id,
    /// semantic_ja surface forms; `surfaces[0]` is the representative.
    pub surfaces: Vec<String>,
}

/// Modality phrase → (direction, strength) per §5; `implies_action` names
/// the action kind a drug-targeted rule takes when the sentence carries no
/// action verb (§8.2 control: 抗菌薬Aは禁忌である → `act.administer`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconModality {
    /// semantic_ja phrase.
    pub surface: String,
    pub direction: Direction,
    pub strength: Strength,
    pub implies_action: Option<Id>,
}

/// Certainty phrase → §5 GRADE-style level, feeding statement certainty
/// when present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconCertainty {
    /// semantic_ja phrase.
    pub surface: String,
    pub value: Certainty,
}

/// Lexicon loading failed: the bytes do not parse into the declared YAML
/// shape (`Yaml`) or parse but break a §5 validation rule (`Invalid`).
#[derive(Debug)]
pub enum LexiconError {
    Yaml(String),
    Invalid(String),
}

impl fmt::Display for LexiconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexiconError::Yaml(detail) => write!(f, "lexicon yaml: {detail}"),
            LexiconError::Invalid(detail) => write!(f, "lexicon invalid: {detail}"),
        }
    }
}

impl std::error::Error for LexiconError {}

// Raw file shape, deserialization-only: `deny_unknown_fields` keeps the
// YAML surface closed so typos fail loud (the registry-loader precedent);
// Id and enum grammars are enforced by their serde impls during this pass.

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLexicon {
    system: Id,
    concepts: Vec<RawConcept>,
    actions: Vec<RawAction>,
    modality: Vec<RawModality>,
    certainty: Vec<RawCertainty>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConcept {
    id: Id,
    surface: Vec<String>,
    interval: Option<RawInterval>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInterval {
    var: Id,
    ge: Option<i64>,
    gt: Option<i64>,
    le: Option<i64>,
    lt: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAction {
    id: Id,
    surface: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawModality {
    surface: String,
    direction: Direction,
    strength: Strength,
    implies_action: Option<Id>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCertainty {
    surface: String,
    value: Certainty,
}

/// Load and validate the lexicon from its raw file bytes.
pub fn load_lexicon(bytes: &[u8]) -> Result<Lexicon, LexiconError> {
    let raw: RawLexicon =
        serde_saphyr::from_slice(bytes).map_err(|e| LexiconError::Yaml(e.to_string()))?;

    let mut pool = HashSet::new();
    for id in raw
        .concepts
        .iter()
        .map(|c| &c.id)
        .chain(raw.actions.iter().map(|a| &a.id))
    {
        if !pool.insert(id.clone()) {
            return Err(LexiconError::Invalid(format!("duplicate id {id}")));
        }
    }

    let concepts: Vec<LexiconConcept> = raw
        .concepts
        .into_iter()
        .map(|c| {
            let surfaces = normalize_surfaces(&c.id, &c.surface)?;
            let interval = c.interval.map(|i| QuantityInterval {
                var: i.var,
                ge: i.ge,
                gt: i.gt,
                le: i.le,
                lt: i.lt,
            });
            if let Some(q) = &interval {
                check_interval(&c.id, q)?;
            }
            Ok(LexiconConcept {
                concept_id: c.id,
                surfaces,
                interval,
            })
        })
        .collect::<Result<_, LexiconError>>()?;

    let actions: Vec<LexiconAction> = raw
        .actions
        .into_iter()
        .map(|a| {
            Ok(LexiconAction {
                surfaces: normalize_surfaces(&a.id, &a.surface)?,
                action_id: a.id,
            })
        })
        .collect::<Result<_, LexiconError>>()?;

    let mut modality_pool = HashSet::new();
    let modality: Vec<LexiconModality> = raw
        .modality
        .into_iter()
        .map(|m| {
            let surface = semantic_ja(&m.surface);
            if !modality_pool.insert(surface.clone()) {
                return Err(LexiconError::Invalid(format!(
                    "duplicate modality surface {surface}"
                )));
            }
            Ok(LexiconModality {
                surface,
                direction: m.direction,
                strength: m.strength,
                implies_action: m.implies_action,
            })
        })
        .collect::<Result<_, LexiconError>>()?;

    let mut certainty_pool = HashSet::new();
    let certainty: Vec<LexiconCertainty> = raw
        .certainty
        .into_iter()
        .map(|c| {
            let surface = semantic_ja(&c.surface);
            if !certainty_pool.insert(surface.clone()) {
                return Err(LexiconError::Invalid(format!(
                    "duplicate certainty surface {surface}"
                )));
            }
            Ok(LexiconCertainty {
                surface,
                value: c.value,
            })
        })
        .collect::<Result<_, LexiconError>>()?;

    Ok(Lexicon {
        system: raw.system,
        content_hash: hash_bytes(bytes),
        concepts,
        actions,
        modality,
        certainty,
    })
}

fn semantic_ja(input: &str) -> String {
    StringPolicy::SemanticJa
        .normalize(input)
        .expect("semantic_ja is infallible")
}

/// Normalize an entry's surface list, requiring it nonempty and free of
/// (post-normalization) duplicates; order is preserved so `surfaces[0]`
/// stays the representative.
fn normalize_surfaces(owner: &Id, raw: &[String]) -> Result<Vec<String>, LexiconError> {
    if raw.is_empty() {
        return Err(LexiconError::Invalid(format!(
            "{owner}: empty surface list"
        )));
    }
    let mut seen = HashSet::new();
    let mut surfaces = Vec::with_capacity(raw.len());
    for s in raw {
        let s = semantic_ja(s);
        if !seen.insert(s.clone()) {
            return Err(LexiconError::Invalid(format!(
                "{owner}: duplicate surface {s}"
            )));
        }
        surfaces.push(s);
    }
    Ok(surfaces)
}

/// §5 bound coherence at load time (the `IrBundle::validate` rule): at
/// least one bound, at most one per side, nonempty over the reals — strict
/// `lo < hi` when either side is strict, else `lo <= hi`.
fn check_interval(owner: &Id, q: &QuantityInterval) -> Result<(), LexiconError> {
    let fail =
        |rule: &str| LexiconError::Invalid(format!("{owner}: interval over {}: {rule}", q.var));
    let (lo, hi) = (q.ge.or(q.gt), q.le.or(q.lt));
    if lo.is_none() && hi.is_none() {
        return Err(fail("no bound"));
    }
    if q.ge.is_some() && q.gt.is_some() {
        return Err(fail("two lower bounds"));
    }
    if q.le.is_some() && q.lt.is_some() {
        return Err(fail("two upper bounds"));
    }
    if let (Some(lo), Some(hi)) = (lo, hi) {
        let strict = q.gt.is_some() || q.lt.is_some();
        if if strict { lo >= hi } else { lo > hi } {
            return Err(fail("empty interval"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn committed() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/lexicon/ja_core.yaml"
        );
        std::fs::read(path).unwrap()
    }

    fn invalid(yaml: &str) -> String {
        match load_lexicon(yaml.as_bytes()) {
            Err(LexiconError::Invalid(detail)) => detail,
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    // The committed lexicon loads; the whole typed value is pinned from
    // observed output (semantic_ja leaves every committed surface
    // byte-identical: no NFKC-affected codepoints, foldable whitespace, or
    // CJK terminal punctuation).
    #[test]
    fn committed_lexicon_loads() {
        let bytes = committed();
        let lexicon = load_lexicon(&bytes).unwrap();
        let age = |id: Id| QuantityInterval {
            var: id,
            ge: None,
            gt: None,
            le: None,
            lt: None,
        };
        let concept =
            |cid: &str, surface: &str, interval: Option<QuantityInterval>| LexiconConcept {
                concept_id: id(cid),
                surfaces: vec![surface.to_owned()],
                interval,
            };
        let phrase = |surface: &str,
                      direction: Direction,
                      strength: Strength,
                      implies_action: Option<Id>| {
            LexiconModality {
                surface: surface.to_owned(),
                direction,
                strength,
                implies_action,
            }
        };
        let level = |surface: &str, value: Certainty| LexiconCertainty {
            surface: surface.to_owned(),
            value,
        };
        assert_eq!(
            lexicon,
            Lexicon {
                system: id("ckc.lex"),
                content_hash: hash_bytes(&bytes),
                concepts: vec![
                    concept("cond.sepsis", "敗血症", None),
                    concept("cond.renal_severe", "重度腎機能障害", None),
                    concept("cond.pregnancy", "妊娠中", None),
                    concept("drug.abx_a", "抗菌薬A", None),
                    concept(
                        "pop.adult",
                        "成人",
                        Some(QuantityInterval {
                            ge: Some(18),
                            ..age(id("q.age_years"))
                        })
                    ),
                    concept(
                        "pop.child",
                        "小児",
                        Some(QuantityInterval {
                            lt: Some(18),
                            ..age(id("q.age_years"))
                        })
                    ),
                ],
                actions: vec![LexiconAction {
                    action_id: id("act.administer"),
                    surfaces: vec!["投与する".to_owned(), "投与".to_owned()],
                }],
                modality: vec![
                    phrase("推奨する", Direction::For, Strength::Strong, None),
                    phrase("提案する", Direction::For, Strength::Weak, None),
                    phrase("考慮してもよい", Direction::Permit, Strength::Weak, None),
                    phrase("推奨しない", Direction::Against, Strength::Strong, None),
                    phrase("提案しない", Direction::Against, Strength::Weak, None),
                    phrase(
                        "投与しないこと",
                        Direction::Contraindicate,
                        Strength::Strong,
                        None
                    ),
                    phrase(
                        "禁忌",
                        Direction::Contraindicate,
                        Strength::Strong,
                        Some(id("act.administer"))
                    ),
                ],
                certainty: vec![
                    level("エビデンスの確実性:高", Certainty::High),
                    level("エビデンスの確実性:中", Certainty::Moderate),
                    level("エビデンスの確実性:低", Certainty::Low),
                    level("エビデンスの確実性:非常に低", Certainty::VeryLow),
                ],
            }
        );
    }

    // One id pool across concepts and actions.
    #[test]
    fn duplicate_id_rejected() {
        let detail = invalid(concat!(
            "system: ckc.lex\n",
            "concepts:\n",
            "  - id: x.a\n",
            "    surface: [甲]\n",
            "actions:\n",
            "  - id: x.a\n",
            "    surface: [行う]\n",
            "modality: []\n",
            "certainty: []\n",
        ));
        assert_eq!(detail, "duplicate id x.a");
    }

    #[test]
    fn empty_surface_list_rejected() {
        let detail = invalid(concat!(
            "system: ckc.lex\n",
            "concepts:\n",
            "  - id: x.a\n",
            "    surface: []\n",
            "actions: []\n",
            "modality: []\n",
            "certainty: []\n",
        ));
        assert_eq!(detail, "x.a: empty surface list");
    }

    // Duplicates are judged post-normalization: full-width Ａ folds onto
    // ASCII A under semantic_ja.
    #[test]
    fn duplicate_entry_surface_rejected() {
        let detail = invalid(concat!(
            "system: ckc.lex\n",
            "concepts:\n",
            "  - id: x.a\n",
            "    surface: [抗菌薬A, 抗菌薬Ａ]\n",
            "actions: []\n",
            "modality: []\n",
            "certainty: []\n",
        ));
        assert_eq!(detail, "x.a: duplicate surface 抗菌薬A");
    }

    #[test]
    fn duplicate_modality_surface_rejected() {
        let detail = invalid(concat!(
            "system: ckc.lex\n",
            "concepts: []\n",
            "actions: []\n",
            "modality:\n",
            "  - surface: 推奨する\n",
            "    direction: for\n",
            "    strength: strong\n",
            "  - surface: 推奨する\n",
            "    direction: against\n",
            "    strength: weak\n",
            "certainty: []\n",
        ));
        assert_eq!(detail, "duplicate modality surface 推奨する");
    }

    #[test]
    fn duplicate_certainty_surface_rejected() {
        let detail = invalid(concat!(
            "system: ckc.lex\n",
            "concepts: []\n",
            "actions: []\n",
            "modality: []\n",
            "certainty:\n",
            "  - surface: 高い\n",
            "    value: high\n",
            "  - surface: 高い\n",
            "    value: moderate\n",
        ));
        assert_eq!(detail, "duplicate certainty surface 高い");
    }

    // §5 bound coherence, one rejection per broken rule; the point
    // interval at the closed boundary stays legal.
    #[test]
    fn incoherent_intervals_rejected() {
        let base = |interval: &str| {
            format!(
                concat!(
                    "system: ckc.lex\n",
                    "concepts:\n",
                    "  - id: x.a\n",
                    "    surface: [甲]\n",
                    "    interval: {}\n",
                    "actions: []\n",
                    "modality: []\n",
                    "certainty: []\n",
                ),
                interval
            )
        };
        let cases = [
            ("{ var: q.v }", "no bound"),
            ("{ var: q.v, ge: 1, gt: 2 }", "two lower bounds"),
            ("{ var: q.v, le: 1, lt: 2 }", "two upper bounds"),
            ("{ var: q.v, ge: 5, le: 4 }", "empty interval"),
            ("{ var: q.v, ge: 5, lt: 5 }", "empty interval"),
        ];
        for (interval, rule) in cases {
            assert_eq!(
                invalid(&base(interval)),
                format!("x.a: interval over q.v: {rule}"),
                "{interval}"
            );
        }
        let point = load_lexicon(base("{ var: q.v, ge: 5, le: 5 }").as_bytes()).unwrap();
        assert_eq!(
            point.concepts[0].interval,
            Some(QuantityInterval {
                var: id("q.v"),
                ge: Some(5),
                gt: None,
                le: Some(5),
                lt: None,
            })
        );
    }

    // Parse-layer failures are Yaml: malformed YAML, unknown fields
    // (deny_unknown_fields), out-of-grammar enum tokens, non-UTF8 bytes.
    #[test]
    fn parse_failures_are_yaml_errors() {
        for bytes in [
            &b"[unclosed"[..],
            b"system: ckc.lex\nconcepts: []\nactions: []\nmodality: []\ncertainty: []\nextra: 1\n",
            b"system: ckc.lex\nconcepts: []\nactions: []\ncertainty: []\nmodality:\n  - surface: x\n    direction: sideways\n    strength: strong\n",
            b"\xff\xfe",
        ] {
            assert!(
                matches!(load_lexicon(bytes), Err(LexiconError::Yaml(_))),
                "expected Yaml error for {:?}",
                String::from_utf8_lossy(bytes)
            );
        }
    }
}
