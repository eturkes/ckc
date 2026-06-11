//! SPEC §7.2 report payload — the canonical [`Report`], the run layout's
//! `report.json` (§8.3) and the derivation DAG's sink. Contents at M1:
//! findings and documented null results partitioned from the §7.1
//! claim-evidence rows, quoted Japanese source spans resolved per member
//! document, a code-keyed §7.4 diagnostics rollup, corpus and lexicon
//! hashes, solver identity, the replay-status slot, and §0-vocabulary
//! wording. This module owns the types, their canonical bytes, and
//! structural validation; assembly over the run's validated artifacts
//! lands with cli-runner.4.1a.2, the markdown rendering, the run/replay
//! manifests, and the `ckc run` report-stage wiring with cli-runner.4.1b.
//!
//! Partition (M1's two-query §6 plan, roles spelled by the §8.6 query-id
//! suffixes `.overlap`/`.deontic`):
//!
//! - `semantic_contradiction` (deontic, unsat) → a finding, carrying its
//!   §6 conflict kind and recorded core.
//! - `semantic_no_conflict` closing its pair — overlap unsat (disjoint
//!   contexts) or deontic sat (shared context, consistent directions) → a
//!   documented null result, carrying neither (Q1 runs produce-models, so
//!   even its unsat records no core).
//! - `semantic_no_conflict` on a sat overlap probe: the pair-eligibility
//!   witness, not an outcome — no report row.
//! - Failure/unknown categories: no row; their evidence is the §7.4
//!   records rolled up in `diagnostics_summary`.

use std::cmp::Ordering;

use ckc_core::{
    CanonError, CanonRead, CanonReadError, Canonical, ClaimTier, Hash, Id, ObjectEmitter,
    ObjectReader, Reader, SolverIdentity, canonical_sort_key, emit_map, emit_set, emit_string,
    emit_u64_map, fieldless_enum, read_map, read_set, read_string, read_u64_map,
};
use ckc_smt::SolverVerdict;

use crate::trace::ConflictKind;

fieldless_enum! {
    /// SPEC §7.2 replay-status slot. Assembly always writes
    /// [`NotReplayed`](ReplayStatus::NotReplayed): `report.json` is hashed
    /// into the replay comparison, so `ckc replay` (cli-runner.4.2) reports
    /// against this slot rather than mutating it; the remaining values are
    /// the §7.4-aligned verdicts that command vocabulary names.
    ReplayStatus {
        NotReplayed => "not_replayed",
        ReplayMatch => "replay_match",
        ReplayMismatch => "replay_mismatch",
        ReplayIdentityUnsupported => "replay_identity_unsupported",
    }
}

fieldless_enum! {
    /// SPEC §0 report vocabulary, spelled exactly as §0 prints it — the
    /// closed label set report wording draws from (§7.2: report wording
    /// stays within the §0 vocabulary). M1 rows use
    /// [`SyntheticFixtureMeasurement`](Wording::SyntheticFixtureMeasurement)
    /// (findings) and
    /// [`DocumentedNullResult`](Wording::DocumentedNullResult) (null
    /// results); the rest join with their milestones.
    Wording {
        Candidate => "candidate",
        DocumentedNullResult => "documented null result",
        FormalizationQa => "formalization-QA",
        LockedMeasurement => "locked measurement",
        RawBenchmarkOutput => "raw benchmark output",
        Replayable => "replayable",
        RequiresHumanAdjudication => "requires human adjudication",
        ResearchHarness => "research harness",
        ReviewCandidate => "review candidate",
        SchemaValid => "schema-valid",
        SourceGrounded => "source-grounded",
        SyntheticFixtureMeasurement => "synthetic fixture measurement",
        TextQualityAnalysis => "text-quality analysis",
        VerifierChecked => "verifier-checked",
    }
}

/// One quoted source span (§7.2 "quoted spans", §8.5 item 9): a member
/// document's region resolved through its SourceGraph to one span's raw
/// fixture bytes. Region and span ids are document-local (§8.6), so every
/// row carries its `document_id`; a multi-span region yields one row per
/// span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotedSpan {
    pub document_id: Id,
    pub region_id: Id,
    pub span_id: Id,
    /// The span's `raw_text` — fixture bytes exactly as extracted.
    pub text: String,
}

impl Canonical for QuotedSpan {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("document_id", |b| self.document_id.emit_canonical(b))?;
        obj.member("region_id", |b| self.region_id.emit_canonical(b))?;
        obj.member("span_id", |b| self.span_id.emit_canonical(b))?;
        obj.member("text", |b| {
            emit_string(b, &self.text);
            Ok(())
        })?;
        obj.finish(out)
    }
}

impl CanonRead for QuotedSpan {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let document_id = obj.member("document_id", Id::read)?;
        let region_id = obj.member("region_id", Id::read)?;
        let span_id = obj.member("span_id", Id::read)?;
        let text = obj.member("text", read_string)?;
        obj.close()?;
        Ok(QuotedSpan {
            document_id,
            region_id,
            span_id,
            text,
        })
    }
}

/// SPEC §7.2 report row, used by both partitions: a finding (conflict kind
/// and core present, verdict unsat) or a documented null result (both
/// absent). Keyed by its §7.1 trace finding id; evidence sets ride from
/// the claim row (`assertion_ids` the claim-evidence set: the recorded
/// core when one exists, else the query's named assertions), `core` is the
/// solver-recorded unsat core itself, and `quoted_spans` ground the row in
/// fixture bytes. At M1 every row is claim tier `s1_admitted` (§8.6: built
/// from artifacts that passed deterministic admission) with its partition's
/// §0 wording label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportFinding {
    pub assertion_ids: Vec<Id>,
    pub claim_tier: ClaimTier,
    pub conflict_kind: Option<ConflictKind>,
    pub core: Option<Vec<Id>>,
    pub finding_id: Id,
    pub query_id: Id,
    pub quoted_spans: Vec<QuotedSpan>,
    pub region_ids: Vec<Id>,
    pub rule_ids: Vec<Id>,
    pub verdict: SolverVerdict,
    pub wording: Wording,
}

impl Canonical for ReportFinding {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("assertion_ids", |b| emit_set(b, &self.assertion_ids))?;
        obj.member("claim_tier", |b| self.claim_tier.emit_canonical(b))?;
        obj.optional("conflict_kind", self.conflict_kind, |b, k| {
            k.emit_canonical(b)
        })?;
        obj.optional("core", self.core.as_ref(), emit_set)?;
        obj.member("finding_id", |b| self.finding_id.emit_canonical(b))?;
        obj.member("query_id", |b| self.query_id.emit_canonical(b))?;
        obj.member("quoted_spans", |b| emit_set(b, &self.quoted_spans))?;
        obj.member("region_ids", |b| emit_set(b, &self.region_ids))?;
        obj.member("rule_ids", |b| emit_set(b, &self.rule_ids))?;
        obj.member("verdict", |b| self.verdict.emit_canonical(b))?;
        obj.member("wording", |b| self.wording.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for ReportFinding {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let assertion_ids = obj.member("assertion_ids", read_set::<Id>)?;
        let claim_tier = obj.member("claim_tier", ClaimTier::read)?;
        let conflict_kind = obj.optional("conflict_kind", ConflictKind::read)?;
        let core = obj.optional("core", read_set::<Id>)?;
        let finding_id = obj.member("finding_id", Id::read)?;
        let query_id = obj.member("query_id", Id::read)?;
        let quoted_spans = obj.member("quoted_spans", read_set::<QuotedSpan>)?;
        let region_ids = obj.member("region_ids", read_set::<Id>)?;
        let rule_ids = obj.member("rule_ids", read_set::<Id>)?;
        let verdict = obj.member("verdict", SolverVerdict::read)?;
        let wording = obj.member("wording", Wording::read)?;
        obj.close()?;
        Ok(ReportFinding {
            assertion_ids,
            claim_tier,
            conflict_kind,
            core,
            finding_id,
            query_id,
            quoted_spans,
            region_ids,
            rule_ids,
            verdict,
            wording,
        })
    }
}

/// SPEC §5 `Report`: the canonical `report.json` payload (§7.2 contents at
/// M1). `corpus_hashes` maps each member document to its raw source-byte
/// hash; `diagnostics_summary` is the code-keyed count rollup over the
/// run's §7.4 records; `findings` and `null_results` are the two claim
/// partitions; `wording` is the canonical set of §0 labels the rows carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub corpus_hashes: Vec<(Id, Hash)>,
    pub diagnostics_summary: Vec<(Id, u64)>,
    pub findings: Vec<ReportFinding>,
    pub lexicon_hash: Hash,
    pub null_results: Vec<ReportFinding>,
    pub replay_status: ReplayStatus,
    pub solver_identity: SolverIdentity,
    pub wording: Vec<Wording>,
}

impl Report {
    /// Structural invariants, first break wins:
    ///
    /// 1. `corpus_hashes` and `diagnostics_summary` are §4.3 maps (keys
    ///    strictly ascending by id bytes); summary counts are positive.
    /// 2. `findings`, `null_results`, and `wording` are canonical sets
    ///    (sorted by [`canonical_sort_key`], duplicate-free).
    /// 3. Rows: `conflict_kind` and `core` are present exactly on findings
    ///    (§6: a contradiction is an unsat deontic query with its recorded
    ///    core; Q1-unsat and deontic-sat nulls carry neither); finding
    ///    verdicts are `unsat` and null verdicts `sat` or `unsat`; the
    ///    three id pools, `quoted_spans`, and `core` are non-empty
    ///    canonical sets; quoted texts are non-empty.
    /// 4. Finding ids are unique across both partitions.
    pub fn validate(&self) -> Result<(), ReportError> {
        check_map_order("corpus_hashes", self.corpus_hashes.iter().map(|(k, _)| k))?;
        check_map_order(
            "diagnostics_summary",
            self.diagnostics_summary.iter().map(|(k, _)| k),
        )?;
        if let Some((code, _)) = self.diagnostics_summary.iter().find(|(_, n)| *n == 0) {
            return Err(ReportError::ZeroCount(code.clone()));
        }
        check_canonical_set("findings", &self.findings)?;
        check_canonical_set("null_results", &self.null_results)?;
        check_canonical_set("wording", &self.wording)?;
        let mut seen: Vec<&str> = Vec::new();
        for (rows, is_finding) in [(&self.findings, true), (&self.null_results, false)] {
            for row in rows {
                if seen.contains(&row.finding_id.as_str()) {
                    return Err(ReportError::DuplicateFindingId(row.finding_id.clone()));
                }
                seen.push(row.finding_id.as_str());
                if row.conflict_kind.is_some() != is_finding {
                    return Err(ReportError::ConflictKindPresence(row.finding_id.clone()));
                }
                if row.core.is_some() != is_finding {
                    return Err(ReportError::CorePresence(row.finding_id.clone()));
                }
                let verdict_ok = if is_finding {
                    row.verdict == SolverVerdict::Unsat
                } else {
                    matches!(row.verdict, SolverVerdict::Sat | SolverVerdict::Unsat)
                };
                if !verdict_ok {
                    return Err(ReportError::VerdictRule {
                        finding_id: row.finding_id.clone(),
                        rule: if is_finding {
                            "findings require verdict unsat"
                        } else {
                            "null results require verdict sat or unsat"
                        },
                    });
                }
                for (pool, set) in [
                    ("assertion_ids", &row.assertion_ids),
                    ("region_ids", &row.region_ids),
                    ("rule_ids", &row.rule_ids),
                ] {
                    if set.is_empty() {
                        return Err(ReportError::EmptyEvidence {
                            finding_id: row.finding_id.clone(),
                            pool,
                        });
                    }
                    check_canonical_set(pool, set.iter())?;
                }
                if row.quoted_spans.is_empty() {
                    return Err(ReportError::EmptyEvidence {
                        finding_id: row.finding_id.clone(),
                        pool: "quoted_spans",
                    });
                }
                check_canonical_set("quoted_spans", &row.quoted_spans)?;
                if let Some(span) = row.quoted_spans.iter().find(|s| s.text.is_empty()) {
                    return Err(ReportError::EmptyQuotedText {
                        finding_id: row.finding_id.clone(),
                        span_id: span.span_id.clone(),
                    });
                }
                if let Some(core) = &row.core {
                    if core.is_empty() {
                        return Err(ReportError::EmptyEvidence {
                            finding_id: row.finding_id.clone(),
                            pool: "core",
                        });
                    }
                    check_canonical_set("core", core.iter())?;
                }
            }
        }
        Ok(())
    }
}

impl Canonical for Report {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("corpus_hashes", |b| {
            emit_map(b, self.corpus_hashes.iter().map(|(k, v)| (k, v)))
        })?;
        obj.member("diagnostics_summary", |b| {
            emit_u64_map(b, &self.diagnostics_summary)
        })?;
        obj.member("findings", |b| emit_set(b, &self.findings))?;
        obj.member("lexicon_hash", |b| self.lexicon_hash.emit_canonical(b))?;
        obj.member("null_results", |b| emit_set(b, &self.null_results))?;
        obj.member("replay_status", |b| self.replay_status.emit_canonical(b))?;
        obj.member("solver_identity", |b| {
            self.solver_identity.emit_canonical(b)
        })?;
        obj.member("wording", |b| emit_set(b, &self.wording))?;
        obj.finish(out)
    }
}

impl CanonRead for Report {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let corpus_hashes = obj.member("corpus_hashes", read_map::<Id, Hash>)?;
        let diagnostics_summary = obj.member("diagnostics_summary", read_u64_map)?;
        let findings = obj.member("findings", read_set::<ReportFinding>)?;
        let lexicon_hash = obj.member("lexicon_hash", Hash::read)?;
        let null_results = obj.member("null_results", read_set::<ReportFinding>)?;
        let replay_status = obj.member("replay_status", ReplayStatus::read)?;
        let solver_identity = obj.member("solver_identity", SolverIdentity::read)?;
        let wording = obj.member("wording", read_set::<Wording>)?;
        obj.close()?;
        Ok(Report {
            corpus_hashes,
            diagnostics_summary,
            findings,
            lexicon_hash,
            null_results,
            replay_status,
            solver_identity,
            wording,
        })
    }
}

/// Enforce canonical-set storage — strictly ascending by
/// [`canonical_sort_key`], duplicate-free (`pool` names the set in
/// errors).
fn check_canonical_set<'a, T: Canonical + 'a>(
    pool: &'static str,
    items: impl IntoIterator<Item = &'a T>,
) -> Result<(), ReportError> {
    let mut prev: Option<Vec<u8>> = None;
    for item in items {
        let key = canonical_sort_key(item)?;
        if let Some(prev) = &prev {
            match prev.as_slice().cmp(&key) {
                Ordering::Less => {}
                Ordering::Equal => return Err(ReportError::SetDuplicate { pool }),
                Ordering::Greater => return Err(ReportError::SetOrder { pool }),
            }
        }
        prev = Some(key);
    }
    Ok(())
}

/// Enforce §4.3 map storage: keys strictly ascending by id bytes (covers
/// duplicates).
fn check_map_order<'a>(
    pool: &'static str,
    keys: impl IntoIterator<Item = &'a Id>,
) -> Result<(), ReportError> {
    let mut prev: Option<&str> = None;
    for key in keys {
        if let Some(prev) = prev
            && prev >= key.as_str()
        {
            return Err(ReportError::MapOrder { pool });
        }
        prev = Some(key.as_str());
    }
    Ok(())
}

/// Structural failure taxonomy for [`Report`] validation; every variant
/// names its offending id or pool (assembly variants join with
/// cli-runner.4.1a.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportError {
    /// A set field is out of canonical order.
    SetOrder { pool: &'static str },
    /// A set field holds byte-identical duplicates.
    SetDuplicate { pool: &'static str },
    /// A map field's keys are not strictly ascending.
    MapOrder { pool: &'static str },
    /// A diagnostics-summary count is zero.
    ZeroCount(Id),
    /// A finding id appears in both partitions or twice in one.
    DuplicateFindingId(Id),
    /// `conflict_kind` is absent on a finding or present on a null result.
    ConflictKindPresence(Id),
    /// `core` is absent on a finding or present on a null result.
    CorePresence(Id),
    /// A row's verdict breaks its partition's §6 rule.
    VerdictRule { finding_id: Id, rule: &'static str },
    /// A row's evidence pool is empty.
    EmptyEvidence { finding_id: Id, pool: &'static str },
    /// A quoted span carries no text.
    EmptyQuotedText { finding_id: Id, span_id: Id },
    /// Canonical emission failed while sorting or checking.
    Canon(CanonError),
}

impl From<CanonError> for ReportError {
    fn from(e: CanonError) -> Self {
        ReportError::Canon(e)
    }
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportError::SetOrder { pool } => {
                write!(f, "{pool} is not in canonical set order")
            }
            ReportError::SetDuplicate { pool } => {
                write!(f, "{pool} holds byte-identical duplicates")
            }
            ReportError::MapOrder { pool } => {
                write!(f, "{pool} keys are not strictly ascending")
            }
            ReportError::ZeroCount(code) => {
                write!(f, "diagnostics_summary count for {code} is zero")
            }
            ReportError::DuplicateFindingId(id) => {
                write!(f, "finding id {id} appears in more than one row")
            }
            ReportError::ConflictKindPresence(id) => {
                write!(
                    f,
                    "row {id} breaks conflict-kind presence: findings carry it, null results omit it"
                )
            }
            ReportError::CorePresence(id) => {
                write!(
                    f,
                    "row {id} breaks core presence: findings carry the unsat core, null results omit it"
                )
            }
            ReportError::VerdictRule { finding_id, rule } => {
                write!(f, "row {finding_id}: {rule}")
            }
            ReportError::EmptyEvidence { finding_id, pool } => {
                write!(f, "row {finding_id} has an empty {pool}")
            }
            ReportError::EmptyQuotedText {
                finding_id,
                span_id,
            } => {
                write!(f, "row {finding_id} quotes span {span_id} with no text")
            }
            ReportError::Canon(e) => write!(f, "canonical emission failed: {e:?}"),
        }
    }
}

impl std::error::Error for ReportError {}

#[cfg(test)]
mod tests {
    use ckc_core::{canonical_payload_bytes, read_canonical};

    use super::*;

    fn id(text: &str) -> Id {
        Id::new(text.to_owned()).unwrap()
    }

    fn hash(seed: char) -> Hash {
        Hash::new(format!("sha256:{}", seed.to_string().repeat(64))).unwrap()
    }

    fn span(doc: &str, region: &str, span_id: &str, text: &str) -> QuotedSpan {
        QuotedSpan {
            document_id: id(doc),
            region_id: id(region),
            span_id: id(span_id),
            text: text.to_owned(),
        }
    }

    /// Hand-built §8.6-shaped report: one deontic finding with its core,
    /// one Q1-unsat null result, every set and map hand-sorted into
    /// canonical order (ASCII span texts keep the byte pin stable).
    fn valid_report() -> Report {
        Report {
            corpus_hashes: vec![(id("fixture.a"), hash('1')), (id("fixture.b"), hash('2'))],
            diagnostics_summary: vec![(id("schema_invalid"), 1), (id("solver_timeout"), 2)],
            findings: vec![ReportFinding {
                assertion_ids: vec![id("a.fixture.a.rule.0"), id("a.fixture.b.rule.0")],
                claim_tier: ClaimTier::S1Admitted,
                conflict_kind: Some(ConflictKind::DeonticDirectionConflict),
                core: Some(vec![id("a.fixture.a.rule.0"), id("a.fixture.b.rule.0")]),
                finding_id: id("finding.group.g1.1"),
                query_id: id("q.g1.pair1.deontic"),
                quoted_spans: vec![
                    span("fixture.a", "r.0", "s.0", "administer drug A"),
                    span("fixture.b", "r.0", "s.0", "withhold drug A"),
                ],
                region_ids: vec![id("r.0")],
                rule_ids: vec![id("fixture.a.rule.0"), id("fixture.b.rule.0")],
                verdict: SolverVerdict::Unsat,
                wording: Wording::SyntheticFixtureMeasurement,
            }],
            lexicon_hash: hash('f'),
            null_results: vec![ReportFinding {
                assertion_ids: vec![id("ctx.fixture.a.rule.1"), id("ctx.fixture.b.rule.1")],
                claim_tier: ClaimTier::S1Admitted,
                conflict_kind: None,
                core: None,
                finding_id: id("finding.group.g2.0"),
                query_id: id("q.g2.pair1.overlap"),
                quoted_spans: vec![
                    span("fixture.a", "r.1", "s.1", "adults eighteen and over"),
                    span("fixture.b", "r.1", "s.1", "children under eighteen"),
                ],
                region_ids: vec![id("r.1")],
                rule_ids: vec![id("fixture.a.rule.1"), id("fixture.b.rule.1")],
                verdict: SolverVerdict::Unsat,
                wording: Wording::DocumentedNullResult,
            }],
            replay_status: ReplayStatus::NotReplayed,
            solver_identity: SolverIdentity {
                solver_id: id("z3"),
                version: "4.13.0".to_owned(),
            },
            wording: vec![
                Wording::DocumentedNullResult,
                Wording::SyntheticFixtureMeasurement,
            ],
        }
    }

    #[test]
    fn wording_spells_the_fourteen_section0_labels() {
        let labels: Vec<&str> = Wording::ALL.iter().map(|w| w.as_str()).collect();
        assert_eq!(
            labels,
            vec![
                "candidate",
                "documented null result",
                "formalization-QA",
                "locked measurement",
                "raw benchmark output",
                "replayable",
                "requires human adjudication",
                "research harness",
                "review candidate",
                "schema-valid",
                "source-grounded",
                "synthetic fixture measurement",
                "text-quality analysis",
                "verifier-checked",
            ]
        );
    }

    #[test]
    fn replay_status_spells_the_four_verdicts() {
        let labels: Vec<&str> = ReplayStatus::ALL.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            labels,
            vec![
                "not_replayed",
                "replay_match",
                "replay_mismatch",
                "replay_identity_unsupported",
            ]
        );
    }

    #[test]
    fn report_round_trips_canonically() {
        let report = valid_report();
        report.validate().unwrap();
        let bytes = canonical_payload_bytes(&report).unwrap();
        let read: Report = read_canonical(&bytes).unwrap();
        assert_eq!(read, report);
        read.validate().unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), PINNED_REPORT);
    }

    /// The full canonical bytes of [`valid_report`], pinned from observed
    /// output: alphabetical members, optionals omitted on the null row,
    /// u64 counts as §4.3 string-wrapped integers.
    const PINNED_REPORT: &str = r#"{"corpus_hashes":{"fixture.a":"sha256:1111111111111111111111111111111111111111111111111111111111111111","fixture.b":"sha256:2222222222222222222222222222222222222222222222222222222222222222"},"diagnostics_summary":{"schema_invalid":"1","solver_timeout":"2"},"findings":[{"assertion_ids":["a.fixture.a.rule.0","a.fixture.b.rule.0"],"claim_tier":"s1_admitted","conflict_kind":"deontic_direction_conflict","core":["a.fixture.a.rule.0","a.fixture.b.rule.0"],"finding_id":"finding.group.g1.1","query_id":"q.g1.pair1.deontic","quoted_spans":[{"document_id":"fixture.a","region_id":"r.0","span_id":"s.0","text":"administer drug A"},{"document_id":"fixture.b","region_id":"r.0","span_id":"s.0","text":"withhold drug A"}],"region_ids":["r.0"],"rule_ids":["fixture.a.rule.0","fixture.b.rule.0"],"verdict":"unsat","wording":"synthetic fixture measurement"}],"lexicon_hash":"sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff","null_results":[{"assertion_ids":["ctx.fixture.a.rule.1","ctx.fixture.b.rule.1"],"claim_tier":"s1_admitted","finding_id":"finding.group.g2.0","query_id":"q.g2.pair1.overlap","quoted_spans":[{"document_id":"fixture.a","region_id":"r.1","span_id":"s.1","text":"adults eighteen and over"},{"document_id":"fixture.b","region_id":"r.1","span_id":"s.1","text":"children under eighteen"}],"region_ids":["r.1"],"rule_ids":["fixture.a.rule.1","fixture.b.rule.1"],"verdict":"unsat","wording":"documented null result"}],"replay_status":"not_replayed","solver_identity":{"solver_id":"z3","version":"4.13.0"},"wording":["documented null result","synthetic fixture measurement"]}"#;

    #[test]
    fn null_row_omits_conflict_kind_and_core_bytes() {
        let report = valid_report();
        let bytes = canonical_payload_bytes(&report.null_results[0]).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("conflict_kind"));
        assert!(!text.contains("core"));
        let read: ReportFinding = read_canonical(text.as_bytes()).unwrap();
        assert_eq!(read, report.null_results[0]);
    }

    #[test]
    fn deontic_sat_null_verdict_is_valid() {
        let mut report = valid_report();
        report.null_results[0].verdict = SolverVerdict::Sat;
        report.validate().unwrap();
    }

    #[test]
    fn rejects_map_order_breaks() {
        let mut report = valid_report();
        report.corpus_hashes.swap(0, 1);
        assert_eq!(
            report.validate(),
            Err(ReportError::MapOrder {
                pool: "corpus_hashes"
            })
        );

        let mut report = valid_report();
        report.corpus_hashes[1].0 = id("fixture.a");
        assert_eq!(
            report.validate(),
            Err(ReportError::MapOrder {
                pool: "corpus_hashes"
            })
        );

        let mut report = valid_report();
        report.diagnostics_summary.swap(0, 1);
        assert_eq!(
            report.validate(),
            Err(ReportError::MapOrder {
                pool: "diagnostics_summary"
            })
        );
    }

    #[test]
    fn rejects_zero_counts() {
        let mut report = valid_report();
        report.diagnostics_summary[0].1 = 0;
        assert_eq!(
            report.validate(),
            Err(ReportError::ZeroCount(id("schema_invalid")))
        );
    }

    #[test]
    fn rejects_set_order_breaks() {
        let mut report = valid_report();
        let mut second = report.findings[0].clone();
        second.finding_id = id("finding.group.g0.0");
        report.findings.push(second);
        assert_eq!(
            report.validate(),
            Err(ReportError::SetOrder { pool: "findings" })
        );

        let mut report = valid_report();
        let copy = report.null_results[0].clone();
        report.null_results.push(copy);
        assert_eq!(
            report.validate(),
            Err(ReportError::SetDuplicate {
                pool: "null_results"
            })
        );

        let mut report = valid_report();
        report.wording.swap(0, 1);
        assert_eq!(
            report.validate(),
            Err(ReportError::SetOrder { pool: "wording" })
        );
    }

    #[test]
    fn rejects_duplicate_finding_ids_across_partitions() {
        let mut report = valid_report();
        report.null_results[0].finding_id = report.findings[0].finding_id.clone();
        assert_eq!(
            report.validate(),
            Err(ReportError::DuplicateFindingId(id("finding.group.g1.1")))
        );
    }

    #[test]
    fn rejects_conflict_kind_presence_breaks() {
        let mut report = valid_report();
        report.findings[0].conflict_kind = None;
        assert_eq!(
            report.validate(),
            Err(ReportError::ConflictKindPresence(id("finding.group.g1.1")))
        );

        let mut report = valid_report();
        report.null_results[0].conflict_kind = Some(ConflictKind::DeonticDirectionConflict);
        assert_eq!(
            report.validate(),
            Err(ReportError::ConflictKindPresence(id("finding.group.g2.0")))
        );
    }

    #[test]
    fn rejects_core_presence_breaks() {
        let mut report = valid_report();
        report.findings[0].core = None;
        assert_eq!(
            report.validate(),
            Err(ReportError::CorePresence(id("finding.group.g1.1")))
        );

        let mut report = valid_report();
        report.null_results[0].core = Some(vec![id("a.fixture.a.rule.1")]);
        assert_eq!(
            report.validate(),
            Err(ReportError::CorePresence(id("finding.group.g2.0")))
        );
    }

    #[test]
    fn rejects_verdict_breaks() {
        let mut report = valid_report();
        report.findings[0].verdict = SolverVerdict::Sat;
        assert_eq!(
            report.validate(),
            Err(ReportError::VerdictRule {
                finding_id: id("finding.group.g1.1"),
                rule: "findings require verdict unsat",
            })
        );

        let mut report = valid_report();
        report.null_results[0].verdict = SolverVerdict::Timeout;
        assert_eq!(
            report.validate(),
            Err(ReportError::VerdictRule {
                finding_id: id("finding.group.g2.0"),
                rule: "null results require verdict sat or unsat",
            })
        );
    }

    #[test]
    fn rejects_empty_evidence() {
        for pool in ["assertion_ids", "region_ids", "rule_ids"] {
            let mut report = valid_report();
            match pool {
                "assertion_ids" => report.findings[0].assertion_ids.clear(),
                "region_ids" => report.findings[0].region_ids.clear(),
                _ => report.findings[0].rule_ids.clear(),
            }
            assert_eq!(
                report.validate(),
                Err(ReportError::EmptyEvidence {
                    finding_id: id("finding.group.g1.1"),
                    pool,
                })
            );
        }

        let mut report = valid_report();
        report.findings[0].quoted_spans.clear();
        assert_eq!(
            report.validate(),
            Err(ReportError::EmptyEvidence {
                finding_id: id("finding.group.g1.1"),
                pool: "quoted_spans",
            })
        );

        let mut report = valid_report();
        report.findings[0].core = Some(vec![]);
        assert_eq!(
            report.validate(),
            Err(ReportError::EmptyEvidence {
                finding_id: id("finding.group.g1.1"),
                pool: "core",
            })
        );
    }

    #[test]
    fn rejects_row_set_breaks() {
        let mut report = valid_report();
        report.findings[0].rule_ids.swap(0, 1);
        assert_eq!(
            report.validate(),
            Err(ReportError::SetOrder { pool: "rule_ids" })
        );

        let mut report = valid_report();
        report.findings[0].assertion_ids = vec![id("a.fixture.a.rule.0"), id("a.fixture.a.rule.0")];
        assert_eq!(
            report.validate(),
            Err(ReportError::SetDuplicate {
                pool: "assertion_ids"
            })
        );

        let mut report = valid_report();
        report.findings[0].quoted_spans.swap(0, 1);
        assert_eq!(
            report.validate(),
            Err(ReportError::SetOrder {
                pool: "quoted_spans"
            })
        );

        let mut report = valid_report();
        report.findings[0].core = Some(vec![id("a.fixture.b.rule.0"), id("a.fixture.a.rule.0")]);
        assert_eq!(
            report.validate(),
            Err(ReportError::SetOrder { pool: "core" })
        );
    }

    #[test]
    fn rejects_empty_quoted_text() {
        let mut report = valid_report();
        report.null_results[0].quoted_spans[0].text = String::new();
        assert_eq!(
            report.validate(),
            Err(ReportError::EmptyQuotedText {
                finding_id: id("finding.group.g2.0"),
                span_id: id("s.1"),
            })
        );
    }
}
