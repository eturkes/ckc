//! Normalize stage, first half: lexicon loading and mention binding
//! (SPEC §8.3 normalize row).
//!
//! [`load_lexicon`] strict-deserializes `corpus/lexicon/ja_core.yaml` — the
//! §5 V1 terminology and modality authority (system `ckc.lex`) — and
//! validates it into the typed [`Lexicon`], content-hash versioned over its
//! raw file bytes (§4.4 raw-byte hashing) for every run manifest. Every
//! surface form is stored normalized under [`StringPolicy::SemanticJa`]
//! with `surfaces[0]` the representative, so downstream binding compares
//! pre-folded text. Validation rejects duplicate ids (one pool across
//! concepts and actions), empty or duplicate per-entry surface lists,
//! surfaces folding empty (a zero-length surface would zero-byte-loop the
//! binding scan), duplicate modality/certainty surfaces, and intervals
//! breaking the §5 bound-coherence rule; one surface shared across
//! concepts stays legal — it is the ambiguity source mention binding
//! consumes.
//!
//! [`bind_segments`] is the binding pass: recommendation and exception
//! segments scanned in document order for concept mentions —
//! longest-match over span `search_text` (semantic_ja, the surface normal
//! form) — minting one [`TerminologyBinding`] per (segment, candidate
//! set); singleton sets bind `exact`/`synonym`, shared surfaces bind
//! `ambiguous` with a `terminology_ambiguous` record (§5). The statement
//! builder (`stage-normalize.1c`) completes the stage's first half.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::Deserialize;

use ckc_core::{
    BindingStatus, Certainty, DiagnosticCode, DiagnosticRecord, Direction, Hash, Id, Outcome,
    QuantityInterval, SegmentIr, SegmentKind, SourceGraph, SourceRegion, SourceSpan, Strength,
    StringPolicy, TerminologyBinding, hash_bytes,
};

use crate::shell::static_id;

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
            if surface.is_empty() {
                return Err(LexiconError::Invalid("empty modality surface".to_owned()));
            }
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
            if surface.is_empty() {
                return Err(LexiconError::Invalid("empty certainty surface".to_owned()));
            }
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
        if s.is_empty() {
            return Err(LexiconError::Invalid(format!("{owner}: empty surface")));
        }
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

/// SPEC §5 mention binding (`stage-normalize.1b`): scan `segments`'
/// recommendation and exception segments, in document order, for lexicon
/// concept mentions, minting one [`TerminologyBinding`] per (segment,
/// candidate set) under document-wide counter ids `bind.<k>`.
///
/// Each segment's regions resolve to their spans, scanned in reading order
/// (a span named by several regions scans once, every naming region
/// grounding its matches); each span's `search_text` is scanned greedy
/// left-to-right longest-match against the concept surfaces, a match
/// consuming its bytes. The candidate set is every concept sharing the
/// matched surface: a singleton binds its concept — `exact` when the
/// representative matched, else `synonym`; a multi set binds `ambiguous` to
/// the byte-lowest candidate, every candidate an alternative, plus one
/// `terminology_ambiguous` Ambiguity record grounded in the binding's
/// regions (§5). A later match of a set already pending in the segment only
/// extends `region_ids`; alternatives and region_ids store in canonical set
/// order. Unmapped text mints nothing here — demand-side residuals are the
/// statement builder's (`stage-normalize.1c`). Dangling region or span refs
/// are the bundle validator's domain (§5 invariants) and skip silently.
pub fn bind_segments(
    graph: &SourceGraph,
    segments: &SegmentIr,
    lexicon: &Lexicon,
) -> (Vec<TerminologyBinding>, Vec<DiagnosticRecord>) {
    let spans: HashMap<&Id, &SourceSpan> = graph.spans.iter().map(|s| (&s.span_id, s)).collect();
    let regions: HashMap<&Id, &SourceRegion> =
        graph.regions.iter().map(|r| (&r.region_id, r)).collect();

    // Concept surface table. Surfaces try longest first, so the scan's
    // first hit is the longest match; equal-length distinct surfaces cannot
    // both match one position, so the byte tiebreak only fixes iteration
    // order.
    let mut candidates: HashMap<&str, Vec<&Id>> = HashMap::new();
    for concept in &lexicon.concepts {
        for surface in &concept.surfaces {
            candidates
                .entry(surface)
                .or_default()
                .push(&concept.concept_id);
        }
    }
    let mut surfaces: Vec<&str> = candidates.keys().copied().collect();
    surfaces.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    let representative: HashMap<&Id, &str> = lexicon
        .concepts
        .iter()
        .map(|c| (&c.concept_id, c.surfaces[0].as_str()))
        .collect();

    let mut bindings: Vec<TerminologyBinding> = Vec::new();
    let mut diagnostics: Vec<DiagnosticRecord> = Vec::new();
    for segment in &segments.segments {
        if !matches!(
            segment.kind,
            SegmentKind::Recommendation | SegmentKind::Exception
        ) {
            continue;
        }
        // The segment's spans and, per span, the regions naming it.
        let mut span_regions: HashMap<&Id, Vec<&Id>> = HashMap::new();
        for region_id in &segment.region_ids {
            let Some(region) = regions.get(region_id) else {
                continue;
            };
            for span_id in &region.span_ids {
                span_regions.entry(span_id).or_default().push(region_id);
            }
        }
        let mut segment_spans: Vec<&SourceSpan> = span_regions
            .keys()
            .filter_map(|span_id| spans.get(*span_id).copied())
            .collect();
        segment_spans.sort_by_key(|s| s.reading_order);

        let mut pending: Vec<PendingBinding> = Vec::new();
        let mut by_key: HashMap<Vec<Id>, usize> = HashMap::new();
        for span in segment_spans {
            let text = span.search_text.as_str();
            let mut at = 0;
            while at < text.len() {
                let Some(&surface) = surfaces.iter().find(|s| text[at..].starts_with(**s)) else {
                    at += text[at..]
                        .chars()
                        .next()
                        .expect("at sits on a char boundary")
                        .len_utf8();
                    continue;
                };
                // Id byte order equals canonical set order (id chars never
                // escape), so one sort serves the dedupe key, the
                // byte-lowest code, and the stored alternatives.
                let mut key: Vec<Id> = candidates[surface].iter().map(|&id| id.clone()).collect();
                key.sort();
                let i = *by_key.entry(key.clone()).or_insert_with(|| {
                    pending.push(PendingBinding::open(&key, surface, &representative));
                    pending.len() - 1
                });
                pending[i]
                    .region_ids
                    .extend(span_regions[&span.span_id].iter().map(|&id| id.clone()));
                at += surface.len();
            }
        }

        for p in pending {
            let PendingBinding {
                code,
                status,
                alternatives,
                mut region_ids,
                surface,
            } = p;
            let binding_id = Id::new(format!("bind.{}", bindings.len()))
                .expect("counter ids match the Id grammar");
            region_ids.sort();
            region_ids.dedup();
            if status == BindingStatus::Ambiguous {
                diagnostics.push(ambiguity(&surface, &alternatives, &region_ids));
            }
            bindings.push(TerminologyBinding {
                binding_id,
                system: lexicon.system.clone(),
                code,
                status,
                alternatives,
                region_ids,
            });
        }
    }
    (bindings, diagnostics)
}

/// One in-flight binding of [`bind_segments`]: a distinct candidate set
/// first matched in the owning segment, accumulating mention regions until
/// the segment's scan completes. `region_ids` holds raw accumulation;
/// finalization sorts and dedupes.
struct PendingBinding {
    code: Id,
    status: BindingStatus,
    alternatives: Vec<Id>,
    region_ids: Vec<Id>,
    /// The first-matched surface, naming the mention in the ambiguity
    /// record.
    surface: String,
}

impl PendingBinding {
    /// Fix code, status, and alternatives from the first match of a
    /// candidate set (`key` ascending by id bytes).
    fn open(key: &[Id], surface: &str, representative: &HashMap<&Id, &str>) -> PendingBinding {
        let (code, status, alternatives) = match key {
            [concept] => {
                let status = if representative[concept] == surface {
                    BindingStatus::Exact
                } else {
                    BindingStatus::Synonym
                };
                (concept.clone(), status, vec![])
            }
            _ => (key[0].clone(), BindingStatus::Ambiguous, key.to_vec()),
        };
        PendingBinding {
            code,
            status,
            alternatives,
            region_ids: vec![],
            surface: surface.to_owned(),
        }
    }
}

/// One `terminology_ambiguous` Ambiguity record for an ambiguous binding
/// (§5): the first-matched surface and the candidate set as the detail,
/// grounded in the binding's regions.
fn ambiguity(surface: &str, alternatives: &[Id], region_ids: &[Id]) -> DiagnosticRecord {
    let candidates = alternatives
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let detail = StringPolicy::DiagnosticText
        .normalize(&format!("ambiguous surface {surface}: {candidates}"))
        .expect("diagnostic_text is infallible");
    DiagnosticRecord {
        code: DiagnosticCode::TerminologyAmbiguous,
        outcome: Outcome::Ambiguity,
        payload: vec![(static_id("detail"), detail)],
        region_ids: region_ids.to_vec(),
        artifact_hashes: vec![],
    }
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

    // Surfaces folding empty under semantic_ja are rejected at load: a
    // zero-length surface would zero-byte-loop the binding scan.
    #[test]
    fn empty_surface_rejected() {
        let concept = concat!(
            "system: ckc.lex\n",
            "concepts:\n",
            "  - id: x.a\n",
            "    surface: [\" \"]\n",
            "actions: []\n",
            "modality: []\n",
            "certainty: []\n",
        );
        assert_eq!(invalid(concept), "x.a: empty surface");
        let modality = concat!(
            "system: ckc.lex\n",
            "concepts: []\n",
            "actions: []\n",
            "modality:\n",
            "  - surface: \"\"\n",
            "    direction: for\n",
            "    strength: strong\n",
            "certainty: []\n",
        );
        assert_eq!(invalid(modality), "empty modality surface");
        let certainty = concat!(
            "system: ckc.lex\n",
            "concepts: []\n",
            "actions: []\n",
            "modality: []\n",
            "certainty:\n",
            "  - surface: \" \"\n",
            "    value: high\n",
        );
        assert_eq!(invalid(certainty), "empty certainty surface");
    }

    // --- bind_segments ---

    use crate::extract::{ExtractConfig, extract};
    use crate::segment::segment;
    use ckc_core::{ArtifactEnvelope, ClinicalSegment, DataClass, Producer, Provenance};

    fn producer() -> Producer {
        Producer {
            candidate_id: id("cand.v1"),
            component_id: id("stage.normalize"),
            toolchain_manifest_hash: Hash::new(format!("sha256:{}", "0".repeat(64))).unwrap(),
        }
    }

    fn extracted(html: &[u8]) -> ArtifactEnvelope<SourceGraph> {
        let config = ExtractConfig {
            document_id: id("doc.test"),
            source_family: id("synthetic_fixture_html"),
            provenance: Provenance::Synthetic,
            data_class: DataClass::None,
            producer: producer(),
        };
        extract(html, &config).unwrap()
    }

    fn fixture(name: &str) -> Vec<u8> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/fixtures/");
        std::fs::read(format!("{dir}{name}")).unwrap()
    }

    /// Extract → segment → bind `html` under `lexicon`.
    fn bound(html: &[u8], lexicon: &Lexicon) -> (Vec<TerminologyBinding>, Vec<DiagnosticRecord>) {
        let source = extracted(html);
        let segments = segment(&source, &producer()).unwrap();
        bind_segments(&source.payload, &segments.payload, lexicon)
    }

    fn binding(
        binding_id: &str,
        code: &str,
        status: BindingStatus,
        alternatives: &[&str],
        region_ids: &[&str],
    ) -> TerminologyBinding {
        TerminologyBinding {
            binding_id: id(binding_id),
            system: id("ckc.lex"),
            code: id(code),
            status,
            alternatives: alternatives.iter().map(|s| id(s)).collect(),
            region_ids: region_ids.iter().map(|s| id(s)).collect(),
        }
    }

    // The committed fixtures bind their recommendation and exception
    // mentions against the committed lexicon — all exact (every fixture
    // mention is a representative surface), diagnostic-free, ids
    // document-wide across segments. Unscanned kinds prove the segment
    // filter: a's CQ heading and definition rows and every metadata
    // heading mention concepts yet mint nothing.
    #[test]
    fn committed_fixtures_bind_exact() {
        let lexicon = load_lexicon(&committed()).unwrap();
        let cases: [(&str, Vec<TerminologyBinding>); 3] = [
            (
                "v1_guideline_a.html",
                vec![
                    binding("bind.0", "pop.adult", BindingStatus::Exact, &[], &["r.2"]),
                    binding("bind.1", "cond.sepsis", BindingStatus::Exact, &[], &["r.2"]),
                    binding("bind.2", "drug.abx_a", BindingStatus::Exact, &[], &["r.2"]),
                    binding(
                        "bind.3",
                        "cond.renal_severe",
                        BindingStatus::Exact,
                        &[],
                        &["r.3"],
                    ),
                ],
            ),
            (
                "v1_guideline_b.html",
                vec![
                    binding("bind.0", "pop.adult", BindingStatus::Exact, &[], &["r.2"]),
                    binding("bind.1", "cond.sepsis", BindingStatus::Exact, &[], &["r.2"]),
                    binding(
                        "bind.2",
                        "cond.pregnancy",
                        BindingStatus::Exact,
                        &[],
                        &["r.2"],
                    ),
                    binding("bind.3", "drug.abx_a", BindingStatus::Exact, &[], &["r.2"]),
                ],
            ),
            (
                "v1_control.html",
                vec![
                    binding("bind.0", "pop.child", BindingStatus::Exact, &[], &["r.2"]),
                    binding("bind.1", "cond.sepsis", BindingStatus::Exact, &[], &["r.2"]),
                    binding("bind.2", "drug.abx_a", BindingStatus::Exact, &[], &["r.2"]),
                ],
            ),
        ];
        for (name, want) in cases {
            let (bindings, diagnostics) = bound(&fixture(name), &lexicon);
            assert!(
                diagnostics.is_empty(),
                "{name} binds diagnostic-free, got {diagnostics:?}"
            );
            assert_eq!(bindings, want, "{name}");
        }
    }

    // A surface shared by two concepts binds ambiguous: code the
    // byte-lowest candidate, all candidates as alternatives, one
    // terminology_ambiguous Ambiguity record grounded in the binding's
    // regions.
    #[test]
    fn shared_surface_binds_ambiguous() {
        let lexicon = load_lexicon(
            concat!(
                "system: ckc.lex\n",
                "concepts:\n",
                "  - id: x.b\n",
                "    surface: [甲]\n",
                "  - id: x.a\n",
                "    surface: [乙, 甲]\n",
                "actions: []\n",
                "modality: []\n",
                "certainty: []\n",
            )
            .as_bytes(),
        )
        .unwrap();
        let (bindings, diagnostics) = bound(
            "<!DOCTYPE html><html><body><p>甲を推奨する。</p></body></html>".as_bytes(),
            &lexicon,
        );
        assert_eq!(
            bindings,
            vec![binding(
                "bind.0",
                "x.a",
                BindingStatus::Ambiguous,
                &["x.a", "x.b"],
                &["r.0"],
            )]
        );
        let [diag] = diagnostics.as_slice() else {
            panic!("one ambiguity record, got {diagnostics:?}");
        };
        assert_eq!(diag.code, DiagnosticCode::TerminologyAmbiguous);
        assert_eq!(diag.outcome, Outcome::Ambiguity);
        assert_eq!(
            diag.payload,
            vec![(id("detail"), "ambiguous surface 甲: x.a x.b".to_owned())]
        );
        assert_eq!(diag.region_ids, vec![id("r.0")]);
        assert!(diag.artifact_hashes.is_empty());
    }

    // Scan semantics on inline lexicons: a synonym first match fixes
    // status synonym and a later representative match of the same
    // candidate set only extends regions (one binding, no status change);
    // the longest surface wins its position and consumes its bytes, the
    // shorter surface binding only where it stands alone.
    #[test]
    fn scan_is_longest_match_with_first_match_status() {
        let synonym_first = load_lexicon(
            concat!(
                "system: ckc.lex\n",
                "concepts:\n",
                "  - id: x.a\n",
                "    surface: [甲, 乙]\n",
                "actions: []\n",
                "modality: []\n",
                "certainty: []\n",
            )
            .as_bytes(),
        )
        .unwrap();
        let (bindings, diagnostics) = bound(
            "<!DOCTYPE html><html><body><p>乙は甲を推奨する。</p></body></html>".as_bytes(),
            &synonym_first,
        );
        assert!(diagnostics.is_empty());
        assert_eq!(
            bindings,
            vec![binding(
                "bind.0",
                "x.a",
                BindingStatus::Synonym,
                &[],
                &["r.0"],
            )]
        );

        let nested = load_lexicon(
            concat!(
                "system: ckc.lex\n",
                "concepts:\n",
                "  - id: x.a\n",
                "    surface: [甲]\n",
                "  - id: x.ab\n",
                "    surface: [甲乙]\n",
                "actions: []\n",
                "modality: []\n",
                "certainty: []\n",
            )
            .as_bytes(),
        )
        .unwrap();
        let (bindings, diagnostics) = bound(
            "<!DOCTYPE html><html><body><p>甲乙のあとに甲を推奨する。</p></body></html>".as_bytes(),
            &nested,
        );
        assert!(diagnostics.is_empty());
        assert_eq!(
            bindings,
            vec![
                binding("bind.0", "x.ab", BindingStatus::Exact, &[], &["r.0"]),
                binding("bind.1", "x.a", BindingStatus::Exact, &[], &["r.0"]),
            ]
        );
    }

    // A hand-built segment over two regions: the same concept mentioned
    // in both spans yields one binding whose region_ids accumulate every
    // mention's region in canonical set order.
    #[test]
    fn multi_region_segment_accumulates_regions() {
        let lexicon = load_lexicon(
            concat!(
                "system: ckc.lex\n",
                "concepts:\n",
                "  - id: x.a\n",
                "    surface: [甲]\n",
                "actions: []\n",
                "modality: []\n",
                "certainty: []\n",
            )
            .as_bytes(),
        )
        .unwrap();
        let source = extracted(
            "<!DOCTYPE html><html><body><p>甲を推奨する。</p><p>甲は対象である。</p></body></html>"
                .as_bytes(),
        );
        let segments = SegmentIr {
            segments: vec![ClinicalSegment {
                segment_id: id("seg.0"),
                kind: SegmentKind::Recommendation,
                region_ids: vec![id("r.0"), id("r.1")],
            }],
        };
        let (bindings, diagnostics) = bind_segments(&source.payload, &segments, &lexicon);
        assert!(diagnostics.is_empty());
        assert_eq!(
            bindings,
            vec![binding(
                "bind.0",
                "x.a",
                BindingStatus::Exact,
                &[],
                &["r.0", "r.1"],
            )]
        );
    }
}
