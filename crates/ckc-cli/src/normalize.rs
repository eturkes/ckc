//! Normalize processing_stage (SPEC §8.3 normalize row): lexicon loading, mention
//! binding, statement building, and the wrapped processing_stage entry;
//! [`NormativeRule`](ckc_core::NormativeRule) derivation lives in [`crate::rules`].
//!
//! [`load_lexicon`] strict-deserializes `corpus/lexicon/ja_core.yaml` — the
//! §5 M1 terminology and modality evidence_status (system `ckc.lex`) — and
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
//! `ambiguous` with a `terminology_ambiguous` record (§5).
//!
//! [`clinical_ir`] (`stage-normalize.1d`) is the statement builder: the
//! binding core re-run per segment (byte-equal to [`bind_segments`]),
//! then at most one [`ClinicalStatement`] per recommendation segment from
//! the slot readings — binding codes split by namespace (`pop.*`
//! population, `drug.*` action target, else condition) and span-text
//! scans for verbs, modality phrases (reading = (direction, strength);
//! `implies_action` the kind fallback), and certainty phrases. Slot
//! misses and ambiguities withhold the statement as §7.4 records;
//! ambiguous certainty keeps it, certainty-free. Exception segments
//! (`stage-normalize.1e`) attach their exact/synonym concepts as
//! [`ExceptionClause`]s (`exc.<k>` counting attached clauses) to the
//! nearest preceding kept statement, whose `source_segment_ids` gains the
//! segment; a concept-free exception or one with no preceding statement
//! records a Residual and drops the clause, bindings emitting either way.
//!
//! [`normalize`] (`stage-normalize.2b`) is the processing_stage entry: [`clinical_ir`]
//! plus [`crate::rules::derive_norm_ir`] under one §4.4 wrapper —
//! `schema.normalization`, artifact id `<document_id>.normalization`,
//! `deterministic_compiler` origin under `mechanical_evidence_status`, the
//! consumed source-graph and segments wrappers' content hashes as the
//! input hashes in that order, the statement pass's diagnostics in the
//! wrapper, payload hashes computed here.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::Deserialize;

use ckc_core::{
    Action, ArtifactWrapper, EvidenceStatus, BindingStatus, CanonError, Certainty, ClinicalIr,
    ClinicalSegment, ClinicalStatement, ContextAtom, DiagnosticCode, DiagnosticRecord, Direction,
    ExceptionClause, Hash, Id, Normalization, Origin, Outcome, Producer, QuantityInterval,
    SegmentIr, SegmentKind, SourceDocumentGraph, EvidenceRegion, SourceTextSpan, Strength, StringPolicy,
    TerminologyBinding, canonicalization_policy_hash, content_hash, hash_bytes,
};

use crate::shell::static_id;

/// SPEC §5 M1 lexicon: the typed, validated view of
/// `corpus/lexicon/ja_core.yaml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lexicon {
    /// Terminology system the entries bind under (M1: `ckc.lex`).
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
/// source_linkage its matches); each span's `search_text` is scanned greedy
/// left-to-right longest-match against the concept surfaces, a match
/// consuming its bytes. The candidate set is every concept sharing the
/// matched surface: a singleton binds its concept — `exact` when the
/// representative matched, else `synonym`; a multi set binds `ambiguous` to
/// the byte-lowest candidate, every candidate an alternative, plus one
/// `terminology_ambiguous` Ambiguity record grounded in the binding's
/// regions (§5). A later match of a set already pending in the segment only
/// extends `region_ids`; alternatives and region_ids store in canonical set
/// order. Unmapped text mints nothing here — demand-side residuals are the
/// statement builder's (`stage-normalize.1d`). Dangling region or span refs
/// are the bundle validator's domain (§5 invariants) and skip silently.
pub fn bind_segments(
    graph: &SourceDocumentGraph,
    segments: &SegmentIr,
    lexicon: &Lexicon,
) -> (Vec<TerminologyBinding>, Vec<DiagnosticRecord>) {
    let index = GraphIndex::new(graph);
    let table = ConceptTable::new(lexicon);

    let mut bindings: Vec<TerminologyBinding> = Vec::new();
    let mut diagnostics: Vec<DiagnosticRecord> = Vec::new();
    for segment in &segments.segments {
        if !matches!(
            segment.kind,
            SegmentKind::Recommendation | SegmentKind::Exception
        ) {
            continue;
        }
        let spans = index.segment_spans(segment);
        let (mut b, mut d) = bind_segment(&spans, &table, &lexicon.system, bindings.len());
        bindings.append(&mut b);
        diagnostics.append(&mut d);
    }
    (bindings, diagnostics)
}

/// One-pass id indexes over a [`SourceDocumentGraph`]: the span and region lookups
/// behind [`GraphIndex::segment_spans`], built once per binding run.
struct GraphIndex<'a> {
    spans: HashMap<&'a Id, &'a SourceTextSpan>,
    regions: HashMap<&'a Id, &'a EvidenceRegion>,
}

impl<'a> GraphIndex<'a> {
    fn new(graph: &'a SourceDocumentGraph) -> GraphIndex<'a> {
        GraphIndex {
            spans: graph.spans.iter().map(|s| (&s.span_id, s)).collect(),
            regions: graph.regions.iter().map(|r| (&r.region_id, r)).collect(),
        }
    }

    /// A segment's spans, deduped and sorted by reading order — a span
    /// named by several regions appears once — each paired with the region
    /// ids naming it (raw accumulation; consumers sort and dedupe what
    /// they keep). Dangling region and span refs skip silently per the
    /// [`bind_segments`] requirements.
    fn segment_spans(&self, segment: &ClinicalSegment) -> Vec<(&'a SourceTextSpan, Vec<&'a Id>)> {
        let mut span_regions: HashMap<&'a Id, Vec<&'a Id>> = HashMap::new();
        for region_id in &segment.region_ids {
            let Some(&region) = self.regions.get(region_id) else {
                continue;
            };
            for span_id in &region.span_ids {
                span_regions
                    .entry(span_id)
                    .or_default()
                    .push(&region.region_id);
            }
        }
        let mut spans: Vec<(&'a SourceTextSpan, Vec<&'a Id>)> = span_regions
            .into_iter()
            .filter_map(|(span_id, regions)| self.spans.get(span_id).map(|&span| (span, regions)))
            .collect();
        spans.sort_by_key(|(span, _)| span.reading_order);
        spans
    }
}

/// Concept surface table built once per binding run: the longest-first
/// scan list, surface → candidate concept ids, and each concept's
/// representative surface for exact-vs-synonym status.
struct ConceptTable<'a> {
    /// Scan list, ordered by [`longest_first`].
    surfaces: Vec<&'a str>,
    /// Surface → concepts sharing it (lexicon order).
    candidates: HashMap<&'a str, Vec<&'a Id>>,
    /// Concept → `surfaces[0]`, the exact-status surface.
    representative: HashMap<&'a Id, &'a str>,
}

impl<'a> ConceptTable<'a> {
    fn new(lexicon: &'a Lexicon) -> ConceptTable<'a> {
        let mut candidates: HashMap<&'a str, Vec<&'a Id>> = HashMap::new();
        for concept in &lexicon.concepts {
            for surface in &concept.surfaces {
                candidates
                    .entry(surface)
                    .or_default()
                    .push(&concept.concept_id);
            }
        }
        ConceptTable {
            surfaces: longest_first(candidates.keys().copied().collect()),
            candidates,
            representative: lexicon
                .concepts
                .iter()
                .map(|c| (&c.concept_id, c.surfaces[0].as_str()))
                .collect(),
        }
    }
}

/// Order a surface list for [`scan`]: byte-length descending so a scan's
/// first prefix hit is the longest match, ties ascending by bytes —
/// equal-length distinct surfaces cannot both match one position, so the
/// tiebreak only fixes iteration order.
fn longest_first(mut surfaces: Vec<&str>) -> Vec<&str> {
    surfaces.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    surfaces
}

/// Greedy left-to-right longest-match scan: the first `surfaces` entry
/// (pre-ordered by [`longest_first`]) prefixing the cursor matches and
/// consumes its bytes, unmatched text advances one char. Matches return
/// in scan order, repeats kept.
fn scan<'s>(text: &str, surfaces: &[&'s str]) -> Vec<&'s str> {
    let mut matches = Vec::new();
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
        matches.push(surface);
        at += surface.len();
    }
    matches
}

/// Drop later duplicates in place, keeping first occurrences in order —
/// the reading-list hygiene [`build_statement`] applies to every slot.
fn dedup_keep_first<T: PartialEq>(items: &mut Vec<T>) {
    let mut i = 0;
    while i < items.len() {
        if items[..i].contains(&items[i]) {
            items.remove(i);
        } else {
            i += 1;
        }
    }
}

/// The per-segment binding core (`stage-normalize.1b` semantics): scan the
/// segment's `spans` — [`GraphIndex::segment_spans`] pairs — against
/// `table`, minting one binding per distinct candidate set in first-match
/// order under ids `bind.<next + local index>`, plus one ambiguity record
/// per ambiguous binding.
fn bind_segment(
    spans: &[(&SourceTextSpan, Vec<&Id>)],
    table: &ConceptTable<'_>,
    system: &Id,
    next: usize,
) -> (Vec<TerminologyBinding>, Vec<DiagnosticRecord>) {
    let mut pending: Vec<PendingBinding> = Vec::new();
    let mut by_key: HashMap<Vec<Id>, usize> = HashMap::new();
    for (span, regions) in spans {
        for surface in scan(&span.search_text, &table.surfaces) {
            // Id byte order equals canonical set order (id chars never
            // escape), so one sort serves the dedupe key, the byte-lowest
            // code, and the stored alternatives.
            let mut key: Vec<Id> = table.candidates[surface]
                .iter()
                .map(|&id| id.clone())
                .collect();
            key.sort();
            let i = *by_key.entry(key.clone()).or_insert_with(|| {
                pending.push(PendingBinding::open(&key, surface, &table.representative));
                pending.len() - 1
            });
            pending[i]
                .region_ids
                .extend(regions.iter().map(|&id| id.clone()));
        }
    }

    let mut bindings: Vec<TerminologyBinding> = Vec::new();
    let mut diagnostics: Vec<DiagnosticRecord> = Vec::new();
    for (k, p) in pending.into_iter().enumerate() {
        let PendingBinding {
            code,
            status,
            alternatives,
            mut region_ids,
            surface,
        } = p;
        let binding_id =
            Id::new(format!("bind.{}", next + k)).expect("counter ids match the Id grammar");
        region_ids.sort();
        region_ids.dedup();
        if status == BindingStatus::Ambiguous {
            diagnostics.push(ambiguity(&surface, &alternatives, &region_ids));
        }
        bindings.push(TerminologyBinding {
            binding_id,
            system: system.clone(),
            code,
            status,
            alternatives,
            region_ids,
        });
    }
    (bindings, diagnostics)
}

/// One in-flight binding of [`bind_segment`]: a distinct candidate set
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

/// SPEC §5 statement building (`stage-normalize.1d`): run the binding core
/// per recommendation and exception segment in document order — bindings
/// and binding diagnostics byte-equal to [`bind_segments`] — and build at
/// most one [`ClinicalStatement`] per recommendation segment. Exception
/// segments bind only (clause attachment is `stage-normalize.1e`), so
/// every statement ships `exceptions: []`.
///
/// Slot readings per segment: its `exact`/`synonym` binding codes split by
/// namespace — `pop.*` population, `drug.*` action target, else condition
/// (ambiguous bindings contribute nothing) — plus scans over its span
/// texts for action verbs, modality phrases, and certainty phrases. A
/// modality reading is the (direction, strength) pair only;
/// `implies_action` ids collect separately as the kind fallback (kinds =
/// verbs if any verb matched, else implied). Every list dedupes keeping
/// first occurrences. The slot rule (`slot_diag` records, grounded in
/// the segment's regions): zero modality readings or zero kinds →
/// `semantic_slot_missing`; zero targets → `terminology_unmapped`; more
/// than one distinct modality/kind/target reading →
/// `terminology_ambiguous` — each withholds the statement while bindings
/// and diagnostics still emit. Ambiguous certainty records
/// `terminology_ambiguous` but keeps the statement with no certainty.
/// Diagnostics accumulate in document order, binding diagnostics before
/// the owning segment's slot diagnostics.
///
/// Kept statements mint `stmt.<k>` counting kept statements:
/// population/condition as [`ContextAtom::Concept`] sets (interval
/// lowering is `stage-normalize.2`), the action from the single kind and
/// target, modality/strength from the single reading, and
/// `source_segment_ids = {segment}`.
pub fn clinical_ir(
    graph: &SourceDocumentGraph,
    segments: &SegmentIr,
    lexicon: &Lexicon,
) -> (ClinicalIr, Vec<DiagnosticRecord>) {
    let index = GraphIndex::new(graph);
    let concepts = ConceptTable::new(lexicon);
    let tables = StatementTables::new(lexicon);

    let mut bindings: Vec<TerminologyBinding> = Vec::new();
    let mut statements: Vec<ClinicalStatement> = Vec::new();
    let mut diagnostics: Vec<DiagnosticRecord> = Vec::new();
    for segment in &segments.segments {
        if !matches!(
            segment.kind,
            SegmentKind::Recommendation | SegmentKind::Exception
        ) {
            continue;
        }
        let spans = index.segment_spans(segment);
        let (mut b, mut d) = bind_segment(&spans, &concepts, &lexicon.system, bindings.len());
        diagnostics.append(&mut d);
        if segment.kind == SegmentKind::Recommendation {
            if let Some(statement) = build_statement(
                segment,
                &spans,
                &b,
                &tables,
                statements.len(),
                &mut diagnostics,
            ) {
                statements.push(statement);
            }
        } else {
            attach_exception(segment, &b, &mut statements, &mut diagnostics);
        }
        bindings.append(&mut b);
    }
    (
        ClinicalIr {
            bindings,
            statements,
        },
        diagnostics,
    )
}

/// Verb, modality, and certainty scan tables built once per
/// [`clinical_ir`] run; every scan list is [`longest_first`]-ordered.
struct StatementTables<'a> {
    verb_surfaces: Vec<&'a str>,
    /// Verb surface → the action kinds carrying it (one surface may serve
    /// several actions; lexicon order).
    verbs: HashMap<&'a str, Vec<&'a Id>>,
    modality_surfaces: Vec<&'a str>,
    modality: HashMap<&'a str, &'a LexiconModality>,
    certainty_surfaces: Vec<&'a str>,
    certainty: HashMap<&'a str, Certainty>,
}

impl<'a> StatementTables<'a> {
    fn new(lexicon: &'a Lexicon) -> StatementTables<'a> {
        let mut verbs: HashMap<&'a str, Vec<&'a Id>> = HashMap::new();
        for action in &lexicon.actions {
            for surface in &action.surfaces {
                verbs.entry(surface).or_default().push(&action.action_id);
            }
        }
        let modality: HashMap<&'a str, &'a LexiconModality> = lexicon
            .modality
            .iter()
            .map(|m| (m.surface.as_str(), m))
            .collect();
        let certainty: HashMap<&'a str, Certainty> = lexicon
            .certainty
            .iter()
            .map(|c| (c.surface.as_str(), c.value))
            .collect();
        StatementTables {
            verb_surfaces: longest_first(verbs.keys().copied().collect()),
            verbs,
            modality_surfaces: longest_first(modality.keys().copied().collect()),
            modality,
            certainty_surfaces: longest_first(certainty.keys().copied().collect()),
            certainty,
        }
    }
}

/// The per-segment statement builder behind [`clinical_ir`]: read the §5
/// slots from the segment's fresh `bindings` and scans over its `spans`,
/// apply the slot rule pushing records onto `diagnostics`, and mint
/// `stmt.<next>` when every required slot reads single.
fn build_statement(
    segment: &ClinicalSegment,
    spans: &[(&SourceTextSpan, Vec<&Id>)],
    bindings: &[TerminologyBinding],
    tables: &StatementTables<'_>,
    next: usize,
    diagnostics: &mut Vec<DiagnosticRecord>,
) -> Option<ClinicalStatement> {
    let mut population: Vec<Id> = Vec::new();
    let mut condition: Vec<Id> = Vec::new();
    let mut targets: Vec<Id> = Vec::new();
    for binding in bindings {
        if !matches!(
            binding.status,
            BindingStatus::Exact | BindingStatus::Synonym
        ) {
            continue;
        }
        let code = binding.code.clone();
        if code.as_str().starts_with("pop.") {
            population.push(code);
        } else if code.as_str().starts_with("drug.") {
            targets.push(code);
        } else {
            condition.push(code);
        }
    }

    let mut verbs: Vec<Id> = Vec::new();
    let mut readings: Vec<(Direction, Strength)> = Vec::new();
    let mut implied: Vec<Id> = Vec::new();
    let mut certainties: Vec<Certainty> = Vec::new();
    for (span, _) in spans {
        for surface in scan(&span.search_text, &tables.verb_surfaces) {
            verbs.extend(tables.verbs[surface].iter().map(|&id| id.clone()));
        }
        for surface in scan(&span.search_text, &tables.modality_surfaces) {
            let phrase = tables.modality[surface];
            readings.push((phrase.direction, phrase.strength));
            implied.extend(phrase.implies_action.clone());
        }
        for surface in scan(&span.search_text, &tables.certainty_surfaces) {
            certainties.push(tables.certainty[surface]);
        }
    }
    for list in [
        &mut population,
        &mut condition,
        &mut targets,
        &mut verbs,
        &mut implied,
    ] {
        dedup_keep_first(list);
    }
    dedup_keep_first(&mut readings);
    dedup_keep_first(&mut certainties);
    let kinds = if verbs.is_empty() { implied } else { verbs };

    let mut withheld = false;
    match readings.len() {
        1 => {}
        0 => {
            withheld = true;
            diagnostics.push(slot_diag(
                DiagnosticCode::SemanticSlotMissing,
                Outcome::Residual,
                "no modality phrase".to_owned(),
                segment,
            ));
        }
        _ => {
            withheld = true;
            diagnostics.push(slot_diag(
                DiagnosticCode::TerminologyAmbiguous,
                Outcome::Ambiguity,
                ambiguous_detail(
                    "modality",
                    readings
                        .iter()
                        .map(|(d, s)| format!("{} {}", d.as_str(), s.as_str())),
                ),
                segment,
            ));
        }
    }
    match kinds.len() {
        1 => {}
        0 => {
            withheld = true;
            diagnostics.push(slot_diag(
                DiagnosticCode::SemanticSlotMissing,
                Outcome::Residual,
                "no action kind".to_owned(),
                segment,
            ));
        }
        _ => {
            withheld = true;
            diagnostics.push(slot_diag(
                DiagnosticCode::TerminologyAmbiguous,
                Outcome::Ambiguity,
                ambiguous_detail("kind", kinds.iter().map(|k| k.to_string())),
                segment,
            ));
        }
    }
    match targets.len() {
        1 => {}
        0 => {
            withheld = true;
            diagnostics.push(slot_diag(
                DiagnosticCode::TerminologyUnmapped,
                Outcome::Residual,
                "no action target".to_owned(),
                segment,
            ));
        }
        _ => {
            withheld = true;
            diagnostics.push(slot_diag(
                DiagnosticCode::TerminologyAmbiguous,
                Outcome::Ambiguity,
                ambiguous_detail("target", targets.iter().map(|t| t.to_string())),
                segment,
            ));
        }
    }
    let certainty = match certainties.as_slice() {
        [] => None,
        [one] => Some(*one),
        _ => {
            diagnostics.push(slot_diag(
                DiagnosticCode::TerminologyAmbiguous,
                Outcome::Ambiguity,
                ambiguous_detail(
                    "certainty",
                    certainties.iter().map(|c| c.as_str().to_owned()),
                ),
                segment,
            ));
            None
        }
    };
    if withheld {
        return None;
    }

    // Concept-atom canonical bytes order by concept id (id chars never
    // escape), so an id sort yields canonical set order.
    population.sort();
    condition.sort();
    Some(ClinicalStatement {
        statement_id: Id::new(format!("stmt.{next}")).expect("counter ids match the Id grammar"),
        population: population.into_iter().map(ContextAtom::Concept).collect(),
        condition: condition.into_iter().map(ContextAtom::Concept).collect(),
        action: Action::new(kinds[0].clone(), targets[0].clone()),
        modality: readings[0].0,
        strength: readings[0].1,
        certainty,
        exceptions: vec![],
        source_segment_ids: vec![segment.segment_id.clone()],
    })
}

/// The per-segment exception attacher behind [`clinical_ir`]: the exception
/// segment's exact/synonym binding codes become the clause atoms of
/// `exc.<k>` (`k` counting attached clauses document-wide), pushed onto the
/// nearest preceding kept statement, whose `source_segment_ids` gains the
/// segment. Zero concepts (`terminology_unmapped`) or no preceding statement
/// (`semantic_slot_missing`) record one Residual and drop the clause; the
/// segment's bindings emit either way.
fn attach_exception(
    segment: &ClinicalSegment,
    bindings: &[TerminologyBinding],
    statements: &mut [ClinicalStatement],
    diagnostics: &mut Vec<DiagnosticRecord>,
) {
    // Id sorts yield canonical set order (id chars never escape).
    let mut codes: Vec<Id> = bindings
        .iter()
        .filter(|b| matches!(b.status, BindingStatus::Exact | BindingStatus::Synonym))
        .map(|b| b.code.clone())
        .collect();
    codes.sort();
    codes.dedup();
    if codes.is_empty() {
        diagnostics.push(slot_diag(
            DiagnosticCode::TerminologyUnmapped,
            Outcome::Residual,
            "no exception concept".to_owned(),
            segment,
        ));
        return;
    }
    let next: usize = statements.iter().map(|s| s.exceptions.len()).sum();
    let Some(statement) = statements.last_mut() else {
        diagnostics.push(slot_diag(
            DiagnosticCode::SemanticSlotMissing,
            Outcome::Residual,
            "no preceding statement".to_owned(),
            segment,
        ));
        return;
    };
    let mut region_ids = segment.region_ids.clone();
    region_ids.sort();
    region_ids.dedup();
    statement.exceptions.push(ExceptionClause {
        exception_id: Id::new(format!("exc.{next}")).expect("counter ids match the Id grammar"),
        atoms: codes.into_iter().map(ContextAtom::Concept).collect(),
        region_ids,
    });
    statement
        .source_segment_ids
        .push(segment.segment_id.clone());
    statement.source_segment_ids.sort();
}

/// Render an ambiguous-slot detail: the distinct readings sorted by their
/// rendered bytes, comma-joined.
fn ambiguous_detail(slot: &str, values: impl Iterator<Item = String>) -> String {
    let mut values: Vec<String> = values.collect();
    values.sort();
    format!("ambiguous {slot}: {}", values.join(", "))
}

/// One slot record of [`build_statement`] (§7.4): the detail and the
/// owning segment in the payload, grounded in the segment's regions.
fn slot_diag(
    code: DiagnosticCode,
    outcome: Outcome,
    detail: String,
    segment: &ClinicalSegment,
) -> DiagnosticRecord {
    let detail = StringPolicy::DiagnosticText
        .normalize(&detail)
        .expect("diagnostic_text is infallible");
    let mut region_ids = segment.region_ids.clone();
    region_ids.sort();
    region_ids.dedup();
    DiagnosticRecord {
        code,
        outcome,
        payload: vec![
            (static_id("detail"), detail),
            (static_id("segment"), segment.segment_id.to_string()),
        ],
        region_ids,
        artifact_hashes: vec![],
    }
}

/// Normalization failed mechanically. Binding gaps, slot misses, and
/// dropped clauses are diagnostics, never errors, so canonical emission
/// while hashing the payload is the sole variant.
#[derive(Debug)]
pub enum NormalizeError {
    Canon(CanonError),
}

impl fmt::Display for NormalizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NormalizeError::Canon(e) => write!(f, "canonical emission: {e}"),
        }
    }
}

impl std::error::Error for NormalizeError {}

impl From<CanonError> for NormalizeError {
    fn from(e: CanonError) -> Self {
        NormalizeError::Canon(e)
    }
}

/// Normalize `source` under `segments` and wrap the result per §4.4
/// (module doc): the §5 statement layer beside the rule layer derived
/// from it, `schema.normalization`.
pub fn normalize(
    source: &ArtifactWrapper<SourceDocumentGraph>,
    segments: &ArtifactWrapper<SegmentIr>,
    lexicon: &Lexicon,
    producer: &Producer,
) -> Result<ArtifactWrapper<Normalization>, NormalizeError> {
    let (clinical, diagnostics) = clinical_ir(&source.payload, &segments.payload, lexicon);
    let document_id = &source.payload.document.document_id;
    let norm = crate::rules::derive_norm_ir(document_id, &clinical, &segments.payload, lexicon);
    let payload = Normalization { clinical, norm };

    let artifact_id = Id::new(format!("{document_id}.normalization"))
        .expect("a valid document id keeps the Id grammar under a suffix");
    Ok(ArtifactWrapper {
        schema_id: static_id("schema.normalization"),
        artifact_id,
        artifact_kind: static_id("normalization"),
        producer: producer.clone(),
        input_hashes: vec![source.content_hash.clone(), segments.content_hash.clone()],
        content_hash: content_hash(&payload)?,
        canonicalization_policy_hash: canonicalization_policy_hash(),
        origin: Origin::DeterministicCompiler,
        evidence_status: EvidenceStatus::MechanicalEvidenceStatus,
        external_effects: vec![],
        trace_refs: vec![],
        diagnostics,
        runtime_metadata: vec![],
        payload,
    })
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
    use ckc_core::{
        ArtifactWrapper, ClinicalSegment, DataClass, Producer, Provenance,
        canonical_payload_bytes, read_strict_canonical,
    };

    fn producer() -> Producer {
        Producer {
            pipeline_id: id("cand.m1"),
            pipeline_step_id: id("processing_stage.normalize"),
            toolchain_manifest_hash: Hash::new(format!("sha256:{}", "0".repeat(64))).unwrap(),
        }
    }

    fn extracted(html: &[u8]) -> ArtifactWrapper<SourceDocumentGraph> {
        let config = ExtractConfig {
            document_id: id("doc.test"),
            source_family: id("synthetic_test_source_html"),
            provenance: Provenance::Synthetic,
            data_class: DataClass::None,
            producer: producer(),
        };
        extract(html, &config).unwrap()
    }

    fn test_source(name: &str) -> Vec<u8> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/test_sources/");
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

    // The committed test_sources bind their recommendation and exception
    // mentions against the committed lexicon — all exact (every test_source
    // mention is a representative surface), diagnostic-free, ids
    // document-wide across segments. Unscanned kinds prove the segment
    // filter: a's CQ heading and definition rows and every metadata
    // heading mention concepts yet mint nothing.
    #[test]
    fn committed_test_sources_bind_exact() {
        let lexicon = load_lexicon(&committed()).unwrap();
        let cases: [(&str, Vec<TerminologyBinding>); 3] = [
            (
                "m1_guideline_a.html",
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
                "m1_guideline_b.html",
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
                "m1_control.html",
                vec![
                    binding("bind.0", "pop.child", BindingStatus::Exact, &[], &["r.2"]),
                    binding("bind.1", "cond.sepsis", BindingStatus::Exact, &[], &["r.2"]),
                    binding("bind.2", "drug.abx_a", BindingStatus::Exact, &[], &["r.2"]),
                ],
            ),
        ];
        for (name, want) in cases {
            let (bindings, diagnostics) = bound(&test_source(name), &lexicon);
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

    // --- clinical_ir ---

    /// Extract → segment → derive ClinicalIR over `html` under `lexicon`.
    fn derived(html: &[u8], lexicon: &Lexicon) -> (ClinicalIr, Vec<DiagnosticRecord>) {
        let source = extracted(html);
        let segments = segment(&source, &producer()).unwrap();
        clinical_ir(&source.payload, &segments.payload, lexicon)
    }

    // The statement builder's binding core reproduces bind_segments
    // byte-for-byte on every committed test_source: same bindings, same
    // (empty) diagnostics.
    #[test]
    fn clinical_ir_bindings_equal_bind_segments() {
        let lexicon = load_lexicon(&committed()).unwrap();
        for name in [
            "m1_guideline_a.html",
            "m1_guideline_b.html",
            "m1_control.html",
        ] {
            let source = extracted(&test_source(name));
            let segments = segment(&source, &producer()).unwrap();
            let (bindings, diagnostics) =
                bind_segments(&source.payload, &segments.payload, &lexicon);
            let (ir, ir_diagnostics) = clinical_ir(&source.payload, &segments.payload, &lexicon);
            assert_eq!(ir.bindings, bindings, "{name}");
            assert_eq!(ir_diagnostics, diagnostics, "{name}");
        }
    }

    // The two single-recommendation test_sources build their statements,
    // pinned from observed output: in guideline_b the verb 投与 matches
    // inside 投与しないこと and the two contraindicate phrases dedupe to
    // one (contraindicate, strong) reading; in control no verb matches
    // and the kind arrives via 禁忌's implies_action.
    #[test]
    fn committed_test_sources_build_statements() {
        let lexicon = load_lexicon(&committed()).unwrap();
        let statement = |population: &str, condition: &[&str]| ClinicalStatement {
            statement_id: id("stmt.0"),
            population: vec![ContextAtom::Concept(id(population))],
            condition: condition
                .iter()
                .map(|c| ContextAtom::Concept(id(c)))
                .collect(),
            action: Action::new(id("act.administer"), id("drug.abx_a")),
            modality: Direction::Contraindicate,
            strength: Strength::Strong,
            certainty: None,
            exceptions: vec![],
            source_segment_ids: vec![id("seg.2")],
        };
        let cases = [
            (
                "m1_guideline_b.html",
                statement("pop.adult", &["cond.pregnancy", "cond.sepsis"]),
            ),
            ("m1_control.html", statement("pop.child", &["cond.sepsis"])),
        ];
        for (name, want) in cases {
            let (ir, diagnostics) = derived(&test_source(name), &lexicon);
            assert!(
                diagnostics.is_empty(),
                "{name} derives diagnostic-free, got {diagnostics:?}"
            );
            assert_eq!(ir.statements, vec![want], "{name}");
        }
    }

    // One inline case per withhold class — the statement is withheld
    // while the segment's bindings still emit, one §7.4 record naming the
    // slot: 考慮してもよい classifies the segment recommendation while the
    // inline lexicon omits it from modality (zero modality readings); no
    // verb and no implies_action (zero kinds); no drug.* binding (zero
    // targets); two distinct (direction, strength) readings (ambiguous
    // modality, values sorted and rendered "<direction> <strength>").
    #[test]
    fn slot_misses_withhold_statement() {
        let drug_and_verb = concat!(
            "system: ckc.lex\n",
            "concepts:\n",
            "  - id: drug.x\n",
            "    surface: [薬X]\n",
            "actions:\n",
            "  - id: act.give\n",
            "    surface: [投与する]\n",
        );
        let cases: [(String, &str, DiagnosticCode, Outcome, &str); 4] = [
            (
                format!("{drug_and_verb}modality: []\ncertainty: []\n"),
                "<p>薬Xを投与する。考慮してもよい。</p>",
                DiagnosticCode::SemanticSlotMissing,
                Outcome::Residual,
                "no modality phrase",
            ),
            (
                concat!(
                    "system: ckc.lex\n",
                    "concepts:\n",
                    "  - id: drug.x\n",
                    "    surface: [薬X]\n",
                    "actions: []\n",
                    "modality:\n",
                    "  - surface: 推奨する\n",
                    "    direction: for\n",
                    "    strength: strong\n",
                    "certainty: []\n",
                )
                .to_owned(),
                "<p>薬Xを推奨する。</p>",
                DiagnosticCode::SemanticSlotMissing,
                Outcome::Residual,
                "no action kind",
            ),
            (
                concat!(
                    "system: ckc.lex\n",
                    "concepts:\n",
                    "  - id: cond.a\n",
                    "    surface: [甲]\n",
                    "actions:\n",
                    "  - id: act.give\n",
                    "    surface: [投与する]\n",
                    "modality:\n",
                    "  - surface: 推奨する\n",
                    "    direction: for\n",
                    "    strength: strong\n",
                    "certainty: []\n",
                )
                .to_owned(),
                "<p>甲に投与するを推奨する。</p>",
                DiagnosticCode::TerminologyUnmapped,
                Outcome::Residual,
                "no action target",
            ),
            (
                format!(
                    concat!(
                        "{}modality:\n",
                        "  - surface: 推奨する\n",
                        "    direction: for\n",
                        "    strength: strong\n",
                        "  - surface: 提案する\n",
                        "    direction: for\n",
                        "    strength: weak\n",
                        "certainty: []\n",
                    ),
                    drug_and_verb
                ),
                "<p>薬Xを投与するを推奨する。提案する。</p>",
                DiagnosticCode::TerminologyAmbiguous,
                Outcome::Ambiguity,
                "ambiguous modality: for strong, for weak",
            ),
        ];
        for (yaml, body, code, outcome, detail) in cases {
            let lexicon = load_lexicon(yaml.as_bytes()).unwrap();
            let html = format!("<!DOCTYPE html><html><body>{body}</body></html>");
            let (ir, diagnostics) = derived(html.as_bytes(), &lexicon);
            assert!(ir.statements.is_empty(), "{detail}: statement withheld");
            assert_eq!(ir.bindings.len(), 1, "{detail}: binding still emits");
            let [diag] = diagnostics.as_slice() else {
                panic!("{detail}: one record, got {diagnostics:?}");
            };
            assert_eq!(diag.code, code, "{detail}");
            assert_eq!(diag.outcome, outcome, "{detail}");
            assert_eq!(
                diag.payload,
                vec![
                    (id("detail"), detail.to_owned()),
                    (id("segment"), "seg.0".to_owned()),
                ],
            );
            assert_eq!(diag.region_ids, vec![id("r.0")], "{detail}");
            assert!(diag.artifact_hashes.is_empty(), "{detail}");
        }
    }

    // Certainty paths: one phrase reads Some; two distinct phrases record
    // ambiguous certainty while the statement stays KEPT, certainty-free.
    #[test]
    fn certainty_reads_and_ambiguity_keeps_statement() {
        let lexicon = load_lexicon(
            concat!(
                "system: ckc.lex\n",
                "concepts:\n",
                "  - id: drug.x\n",
                "    surface: [薬X]\n",
                "actions:\n",
                "  - id: act.give\n",
                "    surface: [投与する]\n",
                "modality:\n",
                "  - surface: 推奨する\n",
                "    direction: for\n",
                "    strength: strong\n",
                "certainty:\n",
                "  - surface: 確実性は高い\n",
                "    value: high\n",
                "  - surface: 確実性は低い\n",
                "    value: low\n",
            )
            .as_bytes(),
        )
        .unwrap();
        let derive = |sentence: &str| {
            let html = format!("<!DOCTYPE html><html><body><p>{sentence}</p></body></html>");
            derived(html.as_bytes(), &lexicon)
        };

        let (ir, diagnostics) = derive("薬Xを投与するを推奨する。確実性は高い。");
        assert!(diagnostics.is_empty(), "got {diagnostics:?}");
        let [statement] = ir.statements.as_slice() else {
            panic!("one statement, got {:?}", ir.statements);
        };
        assert_eq!(statement.certainty, Some(Certainty::High));

        let (ir, diagnostics) = derive("薬Xを投与するを推奨する。確実性は高いが確実性は低い。");
        let [diag] = diagnostics.as_slice() else {
            panic!("one record, got {diagnostics:?}");
        };
        assert_eq!(diag.code, DiagnosticCode::TerminologyAmbiguous);
        assert_eq!(diag.outcome, Outcome::Ambiguity);
        assert_eq!(
            diag.payload,
            vec![
                (id("detail"), "ambiguous certainty: high, low".to_owned()),
                (id("segment"), "seg.0".to_owned()),
            ],
        );
        let [statement] = ir.statements.as_slice() else {
            panic!("statement kept, got {:?}", ir.statements);
        };
        assert_eq!(statement.certainty, None);
        assert_eq!(statement.statement_id, id("stmt.0"));
    }

    // Within a segment, binding diagnostics precede slot diagnostics: the
    // shared surface binds ambiguous first (contributing to no slot), then
    // the slot rule reports the modality, kind, and target misses in
    // order; bind_segments reproduces exactly the binding prefix.
    #[test]
    fn binding_diagnostics_precede_slot_diagnostics() {
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
        let source =
            extracted("<!DOCTYPE html><html><body><p>甲を推奨する。</p></body></html>".as_bytes());
        let segments = segment(&source, &producer()).unwrap();
        let (bindings, bind_diags) = bind_segments(&source.payload, &segments.payload, &lexicon);
        let (ir, diagnostics) = clinical_ir(&source.payload, &segments.payload, &lexicon);
        assert_eq!(ir.bindings, bindings);
        assert!(ir.statements.is_empty());
        assert_eq!(diagnostics[..1], bind_diags[..]);
        let details: Vec<&str> = diagnostics
            .iter()
            .map(|d| d.payload[0].1.as_str())
            .collect();
        assert_eq!(
            details,
            vec![
                "ambiguous surface 甲: x.a x.b",
                "no modality phrase",
                "no action kind",
                "no action target",
            ]
        );
    }

    // --- exception clauses ---

    // guideline_a's full statement, pinned from observed output: the
    // exception segment's cond.renal_severe becomes exc.0 on the kept
    // statement, whose source_segment_ids span the recommendation and
    // exception segments.
    #[test]
    fn committed_guideline_a_attaches_exception() {
        let lexicon = load_lexicon(&committed()).unwrap();
        let (ir, diagnostics) = derived(&test_source("m1_guideline_a.html"), &lexicon);
        assert!(
            diagnostics.is_empty(),
            "derives diagnostic-free, got {diagnostics:?}"
        );
        assert_eq!(
            ir.statements,
            vec![ClinicalStatement {
                statement_id: id("stmt.0"),
                population: vec![ContextAtom::Concept(id("pop.adult"))],
                condition: vec![ContextAtom::Concept(id("cond.sepsis"))],
                action: Action::new(id("act.administer"), id("drug.abx_a")),
                modality: Direction::For,
                strength: Strength::Strong,
                certainty: None,
                exceptions: vec![ExceptionClause {
                    exception_id: id("exc.0"),
                    atoms: vec![ContextAtom::Concept(id("cond.renal_severe"))],
                    region_ids: vec![id("r.3")],
                }],
                source_segment_ids: vec![id("seg.2"), id("seg.3")],
            }]
        );
    }

    // One inline case per miss — the clause drops while the segments'
    // bindings still emit, one §7.4 Residual naming the miss: an
    // unknown-text exception after a kept statement leaves it
    // exception-free (terminology_unmapped); a lone exception with no
    // preceding statement binds but attaches nowhere
    // (semantic_slot_missing).
    #[test]
    fn exception_misses_drop_clause() {
        let lexicon = load_lexicon(
            concat!(
                "system: ckc.lex\n",
                "concepts:\n",
                "  - id: drug.x\n",
                "    surface: [薬X]\n",
                "actions:\n",
                "  - id: act.give\n",
                "    surface: [投与する]\n",
                "modality:\n",
                "  - surface: 推奨する\n",
                "    direction: for\n",
                "    strength: strong\n",
                "certainty: []\n",
            )
            .as_bytes(),
        )
        .unwrap();
        let source = extracted(
            "<!DOCTYPE html><html><body><p>薬Xを投与するを推奨する。</p><p>不明の場合を除く。</p></body></html>"
                .as_bytes(),
        );
        let segments = SegmentIr {
            segments: vec![
                ClinicalSegment {
                    segment_id: id("seg.0"),
                    kind: SegmentKind::Recommendation,
                    region_ids: vec![id("r.0")],
                },
                ClinicalSegment {
                    segment_id: id("seg.1"),
                    kind: SegmentKind::Exception,
                    region_ids: vec![id("r.1")],
                },
            ],
        };
        let (ir, diagnostics) = clinical_ir(&source.payload, &segments, &lexicon);
        let [statement] = ir.statements.as_slice() else {
            panic!("statement kept, got {:?}", ir.statements);
        };
        assert!(statement.exceptions.is_empty(), "left exception-free");
        assert_eq!(statement.source_segment_ids, vec![id("seg.0")]);
        assert_eq!(ir.bindings.len(), 1, "recommendation binding emits");
        let [diag] = diagnostics.as_slice() else {
            panic!("one record, got {diagnostics:?}");
        };
        assert_eq!(diag.code, DiagnosticCode::TerminologyUnmapped);
        assert_eq!(diag.outcome, Outcome::Residual);
        assert_eq!(
            diag.payload,
            vec![
                (id("detail"), "no exception concept".to_owned()),
                (id("segment"), "seg.1".to_owned()),
            ],
        );
        assert_eq!(diag.region_ids, vec![id("r.1")]);

        let lexicon = load_lexicon(
            concat!(
                "system: ckc.lex\n",
                "concepts:\n",
                "  - id: cond.a\n",
                "    surface: [甲]\n",
                "actions: []\n",
                "modality: []\n",
                "certainty: []\n",
            )
            .as_bytes(),
        )
        .unwrap();
        let source = extracted(
            "<!DOCTYPE html><html><body><p>甲の場合を除く。</p></body></html>".as_bytes(),
        );
        let segments = SegmentIr {
            segments: vec![ClinicalSegment {
                segment_id: id("seg.0"),
                kind: SegmentKind::Exception,
                region_ids: vec![id("r.0")],
            }],
        };
        let (ir, diagnostics) = clinical_ir(&source.payload, &segments, &lexicon);
        assert!(ir.statements.is_empty());
        assert_eq!(
            ir.bindings,
            vec![binding(
                "bind.0",
                "cond.a",
                BindingStatus::Exact,
                &[],
                &["r.0"]
            )],
            "exception binding emits"
        );
        let [diag] = diagnostics.as_slice() else {
            panic!("one record, got {diagnostics:?}");
        };
        assert_eq!(diag.code, DiagnosticCode::SemanticSlotMissing);
        assert_eq!(diag.outcome, Outcome::Residual);
        assert_eq!(
            diag.payload,
            vec![
                (id("detail"), "no preceding statement".to_owned()),
                (id("segment"), "seg.0".to_owned()),
            ],
        );
        assert_eq!(diag.region_ids, vec![id("r.0")]);
    }

    // Accumulation: one recommendation then two exception segments — the
    // clauses land as exc.0 and exc.1 on the one statement, each with its
    // own atoms and regions, source_segment_ids accumulating all three
    // segments in canonical set order.
    #[test]
    fn exceptions_accumulate_on_one_statement() {
        let lexicon = load_lexicon(
            concat!(
                "system: ckc.lex\n",
                "concepts:\n",
                "  - id: cond.a\n",
                "    surface: [甲]\n",
                "  - id: cond.b\n",
                "    surface: [乙]\n",
                "  - id: drug.x\n",
                "    surface: [薬X]\n",
                "actions:\n",
                "  - id: act.give\n",
                "    surface: [投与する]\n",
                "modality:\n",
                "  - surface: 推奨する\n",
                "    direction: for\n",
                "    strength: strong\n",
                "certainty: []\n",
            )
            .as_bytes(),
        )
        .unwrap();
        let source = extracted(
            "<!DOCTYPE html><html><body><p>薬Xを投与するを推奨する。</p><p>甲を除く。</p><p>乙を除く。</p></body></html>"
                .as_bytes(),
        );
        let segments = SegmentIr {
            segments: vec![
                ClinicalSegment {
                    segment_id: id("seg.0"),
                    kind: SegmentKind::Recommendation,
                    region_ids: vec![id("r.0")],
                },
                ClinicalSegment {
                    segment_id: id("seg.1"),
                    kind: SegmentKind::Exception,
                    region_ids: vec![id("r.1")],
                },
                ClinicalSegment {
                    segment_id: id("seg.2"),
                    kind: SegmentKind::Exception,
                    region_ids: vec![id("r.2")],
                },
            ],
        };
        let (ir, diagnostics) = clinical_ir(&source.payload, &segments, &lexicon);
        assert!(diagnostics.is_empty(), "got {diagnostics:?}");
        let [statement] = ir.statements.as_slice() else {
            panic!("one statement, got {:?}", ir.statements);
        };
        assert_eq!(
            statement.exceptions,
            vec![
                ExceptionClause {
                    exception_id: id("exc.0"),
                    atoms: vec![ContextAtom::Concept(id("cond.a"))],
                    region_ids: vec![id("r.1")],
                },
                ExceptionClause {
                    exception_id: id("exc.1"),
                    atoms: vec![ContextAtom::Concept(id("cond.b"))],
                    region_ids: vec![id("r.2")],
                },
            ]
        );
        assert_eq!(
            statement.source_segment_ids,
            vec![id("seg.0"), id("seg.1"), id("seg.2")]
        );
    }

    // Double derivation over the exception-bearing test_source is
    // byte-identical, and the bytes strict-read back to the derived value.
    #[test]
    fn clinical_ir_derivation_is_deterministic() {
        let lexicon = load_lexicon(&committed()).unwrap();
        let html = test_source("m1_guideline_a.html");
        let (first, _) = derived(&html, &lexicon);
        let (second, _) = derived(&html, &lexicon);
        let bytes = canonical_payload_bytes(&first).unwrap();
        assert_eq!(bytes, canonical_payload_bytes(&second).unwrap());
        let read: ClinicalIr = read_strict_canonical(&bytes).unwrap();
        assert_eq!(read, first);
    }
}
