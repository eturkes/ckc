//! SPEC §7.2 report payload — the canonical [`Report`], the run layout's
//! `report.json` (§8.3) and the derivation DAG's sink. Contents at M1:
//! findings and documented no-conflict results partitioned from the §7.1
//! claim-evidence rows, quoted Japanese source text spans resolved per member
//! document, a code-keyed §7.4 diagnostics summary, corpus and lexicon
//! hashes, solver identity, the replay-status slot, and §0-vocabulary
//! wording. This module owns the types, their canonical bytes, structural
//! validation, and assembly ([`assemble_report`]) over the run's validated
//! artifacts; `ckc run` drives it from the report processing_stage (cli-runner.4.1b.1);
//! [`render_markdown`] is the deterministic §7.2 derived view, the
//! `report_en.md` body (cli-runner.4.1b.2a); the run/replay manifest landings
//! arrive with .4.1b.2b (manifest assembly lives in `crate::manifests`).
//!
//! Partition (M1's two-query §6 plan, roles spelled by the §8.6 query-id
//! suffixes `.overlap`/`.deontic`):
//!
//! - `semantic_contradiction` (deontic, unsat) → a finding, carrying its
//!   §6 conflict kind and recorded core.
//! - `semantic_no_conflict` closing its pair — overlap unsat (disjoint
//!   contexts) or deontic sat (shared context, consistent directions) → a
//!   documented no-conflict result, carrying neither (Q1 runs produce-models, so
//!   even its unsat records no core).
//! - `semantic_no_conflict` on a sat overlap probe: the pair-eligibility
//!   satisfying_example, not an outcome — no report row.
//! - Failure/unknown categories: no row; their evidence is the §7.4
//!   records rolled up in `diagnostics_summary`.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use ckc_core::{
    CanonError, CanonRead, CanonReadError, Canonical, ClaimTier, DiagnosticRecord, Hash, Id,
    ObjectEmitter, ObjectReader, Reader, SolverIdentity, SourceDocumentGraph, canonical_sort_key,
    emit_map, emit_set, emit_string, emit_u64_map, fieldless_enum, read_map, read_set, read_string,
    read_u64_map,
};
use ckc_smt::{SolverVerdict, VerifierCategory, VerifierResult, VerifierResults};

use crate::trace::{
    ConflictKind, LineageIndex, LineageRow, TraceBundle, TraceNodeKind, canonical_id_set,
};

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
    /// [`SyntheticTestSourceMeasurement`](Wording::SyntheticTestSourceMeasurement)
    /// (findings) and
    /// [`DocumentedNoConflictResult`](Wording::DocumentedNoConflictResult) (no-conflict
    /// results); the rest join with their milestones.
    Wording {
        Candidate => "candidate",
        DocumentedNoConflictResult => "documented no-conflict result",
        FormalizationQa => "formalization-QA",
        LockedMeasurement => "locked measurement",
        RawBenchmarkOutput => "raw benchmark output",
        Replayable => "replayable",
        RequiresHumanReview => "requires human review",
        ResearchHarness => "research harness",
        ReviewCandidate => "review candidate",
        SchemaValid => "schema-valid",
        SourceGrounded => "source-grounded",
        SyntheticTestSourceMeasurement => "synthetic test source measurement",
        TextQualityAnalysis => "text-quality analysis",
        VerifierChecked => "verifier-checked",
    }
}

/// One quoted source text span (§7.2 "quoted spans", §8.5 item 9): a member
/// document's region resolved through its SourceDocumentGraph to one span's raw
/// test_source bytes. Region and span ids are document-local (§8.6), so every
/// row carries its `document_id`; a multi-span region yields one row per
/// span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotedSpan {
    pub document_id: Id,
    pub region_id: Id,
    pub span_id: Id,
    /// The span's `raw_text` — test_source bytes exactly as extracted.
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
/// and core present, verdict unsat) or a documented no-conflict result (both
/// absent). Keyed by its §7.1 trace finding id; evidence sets ride from
/// the claim row (`assertion_ids` the claim-evidence set: the recorded
/// core when one exists, else the query's named assertions), `core` is the
/// solver-recorded unsat core itself, and `quoted_spans` ground the row in
/// test_source bytes. At M1 every row is claim tier `s1_accepted` (§8.6: built
/// from artifacts that passed deterministic acceptance) with its partition's
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
/// hash; `diagnostics_summary` is the code-keyed count summary over the
/// run's §7.4 records; `findings` and `no_conflict_results` are the two claim
/// partitions; `wording` is the canonical set of §0 labels the rows carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub corpus_hashes: Vec<(Id, Hash)>,
    pub diagnostics_summary: Vec<(Id, u64)>,
    pub findings: Vec<ReportFinding>,
    /// Raw-byte hash of the §5 lexicon reference file (§4.4: the lexicon
    /// is a file, not an accepted artifact).
    pub lexicon_hash: Hash,
    pub no_conflict_results: Vec<ReportFinding>,
    pub replay_status: ReplayStatus,
    pub solver_identity: SolverIdentity,
    pub wording: Vec<Wording>,
}

impl Report {
    /// Structural invariants, first break wins:
    ///
    /// 1. `corpus_hashes` and `diagnostics_summary` are §4.3 maps (keys
    ///    strictly ascending by id bytes); summary counts are positive.
    /// 2. `findings`, `no_conflict_results`, and `wording` are canonical sets
    ///    (sorted by [`canonical_sort_key`], duplicate-free).
    /// 3. Rows: `conflict_kind` and `core` are present exactly on findings
    ///    (§6: a contradiction is an unsat deontic query with its recorded
    ///    core; Q1-unsat and deontic-sat no_conflict_results carry neither); finding
    ///    verdicts are `unsat` and no-conflict verdicts `sat` or `unsat`; the
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
        check_canonical_set("no_conflict_results", &self.no_conflict_results)?;
        check_canonical_set("wording", &self.wording)?;
        let mut seen: Vec<&str> = Vec::new();
        for (rows, is_finding) in [(&self.findings, true), (&self.no_conflict_results, false)] {
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
                            "no-conflict results require verdict sat or unsat"
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
        obj.member("no_conflict_results", |b| {
            emit_set(b, &self.no_conflict_results)
        })?;
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
        let no_conflict_results = obj.member("no_conflict_results", read_set::<ReportFinding>)?;
        let replay_status = obj.member("replay_status", ReplayStatus::read)?;
        let solver_identity = obj.member("solver_identity", SolverIdentity::read)?;
        let wording = obj.member("wording", read_set::<Wording>)?;
        obj.close()?;
        Ok(Report {
            corpus_hashes,
            diagnostics_summary,
            findings,
            lexicon_hash,
            no_conflict_results,
            replay_status,
            solver_identity,
            wording,
        })
    }
}

/// §6 query role inside its contradiction pair, recovered from the §8.6
/// query-id suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Overlap,
    Deontic,
}

/// Assemble the run's [`Report`] over the validated run artifact set:
/// the strict-read §7.1 pair (claims and source nodes from `bundle`,
/// per-document reference rows from `lineage`), each member document's
/// SourceDocumentGraph, each group's VerifierResults, the run's lexicon hash and
/// solver identity, and every §7.4 record the run emitted.
///
/// Inputs must already satisfy their own validation (the §8.5 item 3 bar);
/// breaks of those per-artifact guarantees panic as caller bugs. Errors
/// are cross-artifact gaps: a claim with no lineage rows, verifier result,
/// or resolvable span chain; a pair disagreement between claim and lineage
/// evidence; duplicate or missing per-document/per-query inputs.
///
/// Rows: claims partition per the module rules; every row resolves its
/// quoted spans per member document through the lineage rows (region and
/// span ids are document-local), takes `core` from its verifier result's
/// recorded unsat core, and carries the M1 constants (`s1_accepted`, the
/// partition's §0 label). `corpus_hashes` come from the bundle's source
/// nodes; `diagnostics_summary` counts `diagnostics` by §7.4 code;
/// `replay_status` starts [`NotReplayed`](ReplayStatus::NotReplayed). The
/// caller runs [`Report::validate`] on the value (the boundary discipline
/// stays with the landing).
pub fn assemble_report(
    bundle: &TraceBundle,
    lineage: &LineageIndex,
    graphs: &[&SourceDocumentGraph],
    results: &[&VerifierResults],
    lexicon_hash: &Hash,
    solver_identity: &SolverIdentity,
    diagnostics: &[DiagnosticRecord],
) -> Result<Report, ReportError> {
    let mut graph_index: BTreeMap<&str, &SourceDocumentGraph> = BTreeMap::new();
    for &graph in graphs {
        let document_id = &graph.document.document_id;
        if graph_index.insert(document_id.as_str(), graph).is_some() {
            return Err(ReportError::DuplicateGraph(document_id.clone()));
        }
    }
    let mut result_index: BTreeMap<&str, &VerifierResult> = BTreeMap::new();
    for result in results.iter().flat_map(|r| &r.results) {
        if result_index
            .insert(result.query_id.as_str(), result)
            .is_some()
        {
            return Err(ReportError::DuplicateResult(result.query_id.clone()));
        }
    }

    let mut corpus_hashes: Vec<(Id, Hash)> = bundle
        .nodes
        .iter()
        .filter(|n| n.kind == TraceNodeKind::Source)
        .map(|n| {
            let hash = n
                .content_hash
                .clone()
                .expect("validated bundles: source nodes carry hashes");
            (n.node_id.clone(), hash)
        })
        .collect();
    corpus_hashes.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

    let mut counts: BTreeMap<&'static str, u64> = BTreeMap::new();
    for record in diagnostics {
        *counts.entry(record.code.as_str()).or_insert(0) += 1;
    }
    let diagnostics_summary: Vec<(Id, u64)> = counts
        .into_iter()
        .map(|(code, n)| {
            let code = Id::new(code.to_owned()).expect("§7.4 code tokens are valid ids");
            (code, n)
        })
        .collect();

    let mut findings = Vec::new();
    let mut no_conflict_results = Vec::new();
    for claim in &bundle.claims {
        if !matches!(
            claim.category,
            VerifierCategory::SemanticContradiction | VerifierCategory::SemanticNoConflict
        ) {
            continue;
        }
        let suffix = claim
            .query_id
            .as_str()
            .strip_prefix(claim.pair_id.as_str())
            .expect("validated claims: query_id extends pair_id");
        let role = match suffix {
            ".overlap" => Role::Overlap,
            ".deontic" => Role::Deontic,
            _ => return Err(ReportError::UnknownQueryRole(claim.query_id.clone())),
        };
        let verdict = claim
            .verdict
            .expect("validated claims: semantic rows carry verdicts");
        let is_finding = match (claim.category, role, verdict) {
            (VerifierCategory::SemanticContradiction, Role::Deontic, _) => true,
            (VerifierCategory::SemanticContradiction, Role::Overlap, _)
            | (VerifierCategory::SemanticNoConflict, Role::Deontic, SolverVerdict::Unsat) => {
                return Err(ReportError::RoleVerdict(claim.query_id.clone()));
            }
            (VerifierCategory::SemanticNoConflict, Role::Overlap, SolverVerdict::Sat) => continue,
            (VerifierCategory::SemanticNoConflict, Role::Overlap, SolverVerdict::Unsat)
            | (VerifierCategory::SemanticNoConflict, Role::Deontic, SolverVerdict::Sat) => false,
            _ => unreachable!("validated claims: §6 category-verdict coherence"),
        };
        let result = result_index
            .get(claim.query_id.as_str())
            .copied()
            .ok_or_else(|| ReportError::MissingResult(claim.query_id.clone()))?;
        if result.category != claim.category || result.verdict != claim.verdict {
            return Err(ReportError::ResultMismatch(claim.query_id.clone()));
        }
        let rows: Vec<&LineageRow> = lineage
            .rows
            .iter()
            .filter(|r| r.finding_id == claim.finding_id)
            .collect();
        if rows.is_empty() {
            return Err(ReportError::MissingLineage(claim.finding_id.clone()));
        }
        let region_union =
            canonical_id_set(rows.iter().flat_map(|r| &r.region_ids).cloned().collect());
        let rule_union = canonical_id_set(rows.iter().flat_map(|r| &r.rule_ids).cloned().collect());
        for (pool, claim_set, union) in [
            ("region_ids", &claim.region_ids, region_union),
            ("rule_ids", &claim.rule_ids, rule_union),
        ] {
            if *claim_set != union {
                return Err(ReportError::PairDisagreement {
                    finding_id: claim.finding_id.clone(),
                    pool,
                });
            }
        }
        let mut quoted = Vec::new();
        for row in &rows {
            let graph = graph_index
                .get(row.document_id.as_str())
                .copied()
                .ok_or_else(|| ReportError::MissingGraph(row.document_id.clone()))?;
            for region_id in &row.region_ids {
                let region = graph
                    .regions
                    .iter()
                    .find(|r| r.region_id == *region_id)
                    .ok_or_else(|| ReportError::MissingRegion {
                        document_id: row.document_id.clone(),
                        region_id: region_id.clone(),
                    })?;
                for span_id in &region.span_ids {
                    let span = graph
                        .spans
                        .iter()
                        .find(|s| s.span_id == *span_id)
                        .ok_or_else(|| ReportError::MissingSpan {
                            document_id: row.document_id.clone(),
                            span_id: span_id.clone(),
                        })?;
                    quoted.push(QuotedSpan {
                        document_id: row.document_id.clone(),
                        region_id: region_id.clone(),
                        span_id: span_id.clone(),
                        text: span.raw_text.clone(),
                    });
                }
            }
        }
        let row = ReportFinding {
            assertion_ids: claim.assertion_ids.clone(),
            claim_tier: ClaimTier::S1Accepted,
            conflict_kind: claim.conflict_kind,
            core: result.unsat_core.clone(),
            finding_id: claim.finding_id.clone(),
            query_id: claim.query_id.clone(),
            quoted_spans: canonical_set(quoted)?,
            region_ids: claim.region_ids.clone(),
            rule_ids: claim.rule_ids.clone(),
            verdict,
            wording: if is_finding {
                Wording::SyntheticTestSourceMeasurement
            } else {
                Wording::DocumentedNoConflictResult
            },
        };
        if is_finding {
            findings.push(row);
        } else {
            no_conflict_results.push(row);
        }
    }
    let findings = canonical_set(findings)?;
    let no_conflict_results = canonical_set(no_conflict_results)?;
    // §0-label bytes order the pair: "documented no-conflict result" sorts before
    // "synthetic test source measurement".
    let mut wording = Vec::new();
    if !no_conflict_results.is_empty() {
        wording.push(Wording::DocumentedNoConflictResult);
    }
    if !findings.is_empty() {
        wording.push(Wording::SyntheticTestSourceMeasurement);
    }
    Ok(Report {
        corpus_hashes,
        diagnostics_summary,
        findings,
        lexicon_hash: lexicon_hash.clone(),
        no_conflict_results,
        replay_status: ReplayStatus::NotReplayed,
        solver_identity: solver_identity.clone(),
        wording,
    })
}

/// SPEC §7.2 `report_en.md` body: the deterministic markdown rendering of a
/// validated [`Report`] — a pure function of the canonical value, §7.2
/// contents in §7.2 prose order (corpus and lexicon hashes, findings,
/// documented no-conflict results, diagnostics summary, solver identity, replay
/// status), headings and labels drawn from §7.2/§0 vocabulary, ids and
/// hashes verbatim in code spans, quoted-span texts verbatim as plain
/// text (§8.5 item 9 resolves them to test_source bytes). Empty content slots
/// render as `none.` so every §7.2 slot stays visible. The caller
/// validates first (the [`assemble_report`] boundary discipline) and
/// writes the returned body as `report_en.md` (cli-runner.4.1b.2b).
pub fn render_markdown(report: &Report) -> String {
    let mut md = String::new();
    md.push_str("# CKC report\n\n");
    md.push_str(&format!("wording: {}\n", join_labels(&report.wording)));

    md.push_str("\n## Corpus\n\n");
    if report.corpus_hashes.is_empty() {
        md.push_str("none.\n");
    } else {
        md.push_str("| document | source hash |\n| --- | --- |\n");
        for (document_id, hash) in &report.corpus_hashes {
            md.push_str(&format!("| `{document_id}` | `{hash}` |\n"));
        }
    }
    md.push_str(&format!("\nlexicon hash: `{}`\n", report.lexicon_hash));

    md.push_str("\n## Findings\n");
    render_rows(&mut md, &report.findings);

    md.push_str("\n## Documented no-conflict results\n");
    render_rows(&mut md, &report.no_conflict_results);

    md.push_str("\n## Diagnostics summary\n\n");
    if report.diagnostics_summary.is_empty() {
        md.push_str("none.\n");
    } else {
        md.push_str("| code | count |\n| --- | --- |\n");
        for (code, count) in &report.diagnostics_summary {
            md.push_str(&format!("| `{code}` | {count} |\n"));
        }
    }

    md.push_str("\n## Solver identity\n\n");
    md.push_str(&format!(
        "`{}` version `{}`\n",
        report.solver_identity.solver_id, report.solver_identity.version
    ));

    md.push_str("\n## Replay status\n\n");
    md.push_str(&format!("`{}`\n", report.replay_status.as_str()));
    md
}

/// One partition's rows under its §7.2 heading: each row a `###` section
/// headed by its finding id, its §0 label and claim tier as the lead
/// sentence, evidence pools as backticked id lists, optionals
/// (`conflict_kind`, `core`) rendered exactly when present — the
/// partition rules make that findings-only.
fn render_rows(md: &mut String, rows: &[ReportFinding]) {
    if rows.is_empty() {
        md.push_str("\nnone.\n");
        return;
    }
    for row in rows {
        md.push_str(&format!("\n### `{}`\n\n", row.finding_id));
        md.push_str(&format!(
            "{}; claim tier `{}`.\n\n",
            row.wording.as_str(),
            row.claim_tier.as_str()
        ));
        if let Some(kind) = row.conflict_kind {
            md.push_str(&format!("- conflict kind: `{}`\n", kind.as_str()));
        }
        md.push_str(&format!(
            "- query: `{}`, verdict `{}`\n",
            row.query_id,
            row.verdict.as_str()
        ));
        md.push_str(&format!("- rules: {}\n", join_ids(&row.rule_ids)));
        md.push_str(&format!("- regions: {}\n", join_ids(&row.region_ids)));
        md.push_str(&format!("- assertions: {}\n", join_ids(&row.assertion_ids)));
        if let Some(core) = &row.core {
            md.push_str(&format!("- core: {}\n", join_ids(core)));
        }
        md.push_str("- quoted spans:\n");
        for span in &row.quoted_spans {
            md.push_str(&format!(
                "  - `{}` `{}` `{}`: {}\n",
                span.document_id, span.region_id, span.span_id, span.text
            ));
        }
    }
}

/// Backticked, comma-joined id list (sets render in storage order).
fn join_ids(ids: &[Id]) -> String {
    ids.iter()
        .map(|id| format!("`{id}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Comma-joined §0 labels as plain prose; the empty set renders `none.`.
fn join_labels(labels: &[Wording]) -> String {
    if labels.is_empty() {
        return "none.".to_owned();
    }
    labels
        .iter()
        .map(|w| w.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Sort `items` into canonical-set storage order by [`canonical_sort_key`]
/// (assembly produces no byte-identical duplicates: quoted-span rows are
/// keyed by per-document lineage triples, partition rows by finding id).
fn canonical_set<T: Canonical>(items: Vec<T>) -> Result<Vec<T>, ReportError> {
    let mut keyed: Vec<(Vec<u8>, T)> = items
        .into_iter()
        .map(|item| Ok((canonical_sort_key(&item)?, item)))
        .collect::<Result<_, CanonError>>()?;
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(keyed.into_iter().map(|(_, item)| item).collect())
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

/// Structural/assembly failure taxonomy for [`Report`]; every variant
/// names its offending id or pool.
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
    /// `conflict_kind` is absent on a finding or present on a no-conflict result.
    ConflictKindPresence(Id),
    /// `core` is absent on a finding or present on a no-conflict result.
    CorePresence(Id),
    /// A row's verdict breaks its partition's §6 rule.
    VerdictRule { finding_id: Id, rule: &'static str },
    /// A row's evidence pool is empty.
    EmptyEvidence { finding_id: Id, pool: &'static str },
    /// A quoted span carries no text.
    EmptyQuotedText { finding_id: Id, span_id: Id },
    /// Two input graphs claim one document.
    DuplicateGraph(Id),
    /// Two input results claim one query.
    DuplicateResult(Id),
    /// A lineage row's document has no input SourceDocumentGraph.
    MissingGraph(Id),
    /// A lineage region is absent from its document's graph.
    MissingRegion { document_id: Id, region_id: Id },
    /// A region's span is absent from its document's graph.
    MissingSpan { document_id: Id, span_id: Id },
    /// A partitioned claim's query has no verifier result.
    MissingResult(Id),
    /// A partitioned claim has no lineage rows.
    MissingLineage(Id),
    /// The claim's evidence set disagrees with its lineage-row union.
    PairDisagreement { finding_id: Id, pool: &'static str },
    /// A semantic claim's query-id suffix names no §8.6 role.
    UnknownQueryRole(Id),
    /// A semantic claim's category contradicts its query role and verdict.
    RoleVerdict(Id),
    /// The verifier result disagrees with its claim row.
    ResultMismatch(Id),
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
                    "row {id} breaks conflict-kind presence: findings carry it, no-conflict results omit it"
                )
            }
            ReportError::CorePresence(id) => {
                write!(
                    f,
                    "row {id} breaks core presence: findings carry the unsat core, no-conflict results omit it"
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
            ReportError::DuplicateGraph(id) => {
                write!(f, "two source document graphs claim document {id}")
            }
            ReportError::DuplicateResult(id) => {
                write!(f, "two verifier results claim query {id}")
            }
            ReportError::MissingGraph(id) => {
                write!(f, "document {id} has no input source document graph")
            }
            ReportError::MissingRegion {
                document_id,
                region_id,
            } => {
                write!(
                    f,
                    "region {region_id} is absent from document {document_id}"
                )
            }
            ReportError::MissingSpan {
                document_id,
                span_id,
            } => {
                write!(f, "span {span_id} is absent from document {document_id}")
            }
            ReportError::MissingResult(id) => {
                write!(f, "query {id} has no verifier result")
            }
            ReportError::MissingLineage(id) => {
                write!(f, "finding {id} has no lineage rows")
            }
            ReportError::PairDisagreement { finding_id, pool } => {
                write!(f, "finding {finding_id}: the pair disagrees on {pool}")
            }
            ReportError::UnknownQueryRole(id) => {
                write!(f, "query {id} names no §8.6 role suffix")
            }
            ReportError::RoleVerdict(id) => {
                write!(f, "query {id}: category contradicts its role and verdict")
            }
            ReportError::ResultMismatch(id) => {
                write!(
                    f,
                    "query {id}: verifier result disagrees with its claim row"
                )
            }
            ReportError::Canon(e) => write!(f, "canonical emission failed: {e:?}"),
        }
    }
}

impl std::error::Error for ReportError {}

#[cfg(test)]
mod tests {
    use ckc_core::{
        DataClass, DiagnosticCode, EvidenceRegion, Outcome, Provenance, SourceDocument,
        SourceTextSpan, canonical_payload_bytes, read_strict_canonical,
    };

    use super::*;
    use crate::trace::{ClaimEvidenceRow, TraceNode};

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
    /// one Q1-unsat no-conflict result, every set and map hand-sorted into
    /// canonical order (ASCII span texts keep the byte pin stable).
    fn valid_report() -> Report {
        Report {
            corpus_hashes: vec![
                (id("test_source.a"), hash('1')),
                (id("test_source.b"), hash('2')),
            ],
            diagnostics_summary: vec![(id("schema_invalid"), 1), (id("solver_timeout"), 2)],
            findings: vec![ReportFinding {
                assertion_ids: vec![id("a.test_source.a.rule.0"), id("a.test_source.b.rule.0")],
                claim_tier: ClaimTier::S1Accepted,
                conflict_kind: Some(ConflictKind::DeonticDirectionConflict),
                core: Some(vec![
                    id("a.test_source.a.rule.0"),
                    id("a.test_source.b.rule.0"),
                ]),
                finding_id: id("finding.group.g1.1"),
                query_id: id("q.g1.pair1.deontic"),
                quoted_spans: vec![
                    span("test_source.a", "r.0", "s.0", "administer drug A"),
                    span("test_source.b", "r.0", "s.0", "withhold drug A"),
                ],
                region_ids: vec![id("r.0")],
                rule_ids: vec![id("test_source.a.rule.0"), id("test_source.b.rule.0")],
                verdict: SolverVerdict::Unsat,
                wording: Wording::SyntheticTestSourceMeasurement,
            }],
            lexicon_hash: hash('f'),
            no_conflict_results: vec![ReportFinding {
                assertion_ids: vec![
                    id("ctx.test_source.a.rule.1"),
                    id("ctx.test_source.b.rule.1"),
                ],
                claim_tier: ClaimTier::S1Accepted,
                conflict_kind: None,
                core: None,
                finding_id: id("finding.group.g2.0"),
                query_id: id("q.g2.pair1.overlap"),
                quoted_spans: vec![
                    span("test_source.a", "r.1", "s.1", "adults eighteen and over"),
                    span("test_source.b", "r.1", "s.1", "children under eighteen"),
                ],
                region_ids: vec![id("r.1")],
                rule_ids: vec![id("test_source.a.rule.1"), id("test_source.b.rule.1")],
                verdict: SolverVerdict::Unsat,
                wording: Wording::DocumentedNoConflictResult,
            }],
            replay_status: ReplayStatus::NotReplayed,
            solver_identity: SolverIdentity {
                solver_id: id("z3"),
                version: "4.13.0".to_owned(),
            },
            wording: vec![
                Wording::DocumentedNoConflictResult,
                Wording::SyntheticTestSourceMeasurement,
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
                "documented no-conflict result",
                "formalization-QA",
                "locked measurement",
                "raw benchmark output",
                "replayable",
                "requires human review",
                "research harness",
                "review candidate",
                "schema-valid",
                "source-grounded",
                "synthetic test source measurement",
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
        let read: Report = read_strict_canonical(&bytes).unwrap();
        assert_eq!(read, report);
        read.validate().unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), PINNED_REPORT);
    }

    /// The full canonical bytes of [`valid_report`], pinned from observed
    /// output: alphabetical members, optionals omitted on the no-conflict row,
    /// u64 counts as §4.3 string-wrapped integers.
    const PINNED_REPORT: &str = r#"{"corpus_hashes":{"test_source.a":"sha256:1111111111111111111111111111111111111111111111111111111111111111","test_source.b":"sha256:2222222222222222222222222222222222222222222222222222222222222222"},"diagnostics_summary":{"schema_invalid":"1","solver_timeout":"2"},"findings":[{"assertion_ids":["a.test_source.a.rule.0","a.test_source.b.rule.0"],"claim_tier":"s1_accepted","conflict_kind":"deontic_direction_conflict","core":["a.test_source.a.rule.0","a.test_source.b.rule.0"],"finding_id":"finding.group.g1.1","query_id":"q.g1.pair1.deontic","quoted_spans":[{"document_id":"test_source.a","region_id":"r.0","span_id":"s.0","text":"administer drug A"},{"document_id":"test_source.b","region_id":"r.0","span_id":"s.0","text":"withhold drug A"}],"region_ids":["r.0"],"rule_ids":["test_source.a.rule.0","test_source.b.rule.0"],"verdict":"unsat","wording":"synthetic test source measurement"}],"lexicon_hash":"sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff","no_conflict_results":[{"assertion_ids":["ctx.test_source.a.rule.1","ctx.test_source.b.rule.1"],"claim_tier":"s1_accepted","finding_id":"finding.group.g2.0","query_id":"q.g2.pair1.overlap","quoted_spans":[{"document_id":"test_source.a","region_id":"r.1","span_id":"s.1","text":"adults eighteen and over"},{"document_id":"test_source.b","region_id":"r.1","span_id":"s.1","text":"children under eighteen"}],"region_ids":["r.1"],"rule_ids":["test_source.a.rule.1","test_source.b.rule.1"],"verdict":"unsat","wording":"documented no-conflict result"}],"replay_status":"not_replayed","solver_identity":{"solver_id":"z3","version":"4.13.0"},"wording":["documented no-conflict result","synthetic test source measurement"]}"#;

    #[test]
    fn no_conflict_row_omits_conflict_kind_and_core_bytes() {
        let report = valid_report();
        let bytes = canonical_payload_bytes(&report.no_conflict_results[0]).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("conflict_kind"));
        assert!(!text.contains("core"));
        let read: ReportFinding = read_strict_canonical(text.as_bytes()).unwrap();
        assert_eq!(read, report.no_conflict_results[0]);
    }

    #[test]
    fn deontic_sat_no_conflict_verdict_is_valid() {
        let mut report = valid_report();
        report.no_conflict_results[0].verdict = SolverVerdict::Sat;
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
        report.corpus_hashes[1].0 = id("test_source.a");
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
        let copy = report.no_conflict_results[0].clone();
        report.no_conflict_results.push(copy);
        assert_eq!(
            report.validate(),
            Err(ReportError::SetDuplicate {
                pool: "no_conflict_results"
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
        report.no_conflict_results[0].finding_id = report.findings[0].finding_id.clone();
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
        report.no_conflict_results[0].conflict_kind = Some(ConflictKind::DeonticDirectionConflict);
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
        report.no_conflict_results[0].core = Some(vec![id("a.test_source.a.rule.1")]);
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
        report.no_conflict_results[0].verdict = SolverVerdict::Timeout;
        assert_eq!(
            report.validate(),
            Err(ReportError::VerdictRule {
                finding_id: id("finding.group.g2.0"),
                rule: "no-conflict results require verdict sat or unsat",
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
        report.findings[0].assertion_ids =
            vec![id("a.test_source.a.rule.0"), id("a.test_source.a.rule.0")];
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
        report.findings[0].core = Some(vec![
            id("a.test_source.b.rule.0"),
            id("a.test_source.a.rule.0"),
        ]);
        assert_eq!(
            report.validate(),
            Err(ReportError::SetOrder { pool: "core" })
        );
    }

    #[test]
    fn rejects_empty_quoted_text() {
        let mut report = valid_report();
        report.no_conflict_results[0].quoted_spans[0].text = String::new();
        assert_eq!(
            report.validate(),
            Err(ReportError::EmptyQuotedText {
                finding_id: id("finding.group.g2.0"),
                span_id: id("s.1"),
            })
        );
    }

    fn identity() -> SolverIdentity {
        SolverIdentity {
            solver_id: id("z3"),
            version: "4.13.0".to_owned(),
        }
    }

    /// A lookup-minimal SourceDocumentGraph: the spans and regions assembly
    /// resolves, document identity for the index key, empty node/anchor
    /// pools.
    fn graph(
        doc: &str,
        spans: &[(&str, &str)],
        regions: &[(&str, &[&str])],
    ) -> SourceDocumentGraph {
        SourceDocumentGraph {
            document: SourceDocument {
                document_id: id(doc),
                source_family: id("family.test_source_html"),
                provenance: Provenance::Synthetic,
                raw_hash: hash('a'),
                content_hash: hash('b'),
                data_class: DataClass::None,
            },
            nodes: vec![],
            spans: spans
                .iter()
                .enumerate()
                .map(|(k, (span_id, text))| {
                    SourceTextSpan::derive(id(span_id), id("n.0"), 0, (*text).to_owned(), k as u64)
                })
                .collect(),
            anchors: vec![],
            regions: regions
                .iter()
                .map(|(region_id, span_ids)| EvidenceRegion {
                    region_id: id(region_id),
                    node_ids: vec![],
                    span_ids: span_ids.iter().map(|s| id(s)).collect(),
                    anchor_ids: vec![],
                })
                .collect(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn claim(
        finding: &str,
        group: &str,
        pair: &str,
        query: &str,
        category: VerifierCategory,
        verdict: Option<SolverVerdict>,
        conflict_kind: Option<ConflictKind>,
        assertions: &[&str],
        rules: &[&str],
        regions: &[&str],
    ) -> ClaimEvidenceRow {
        ClaimEvidenceRow {
            finding_id: id(finding),
            group_id: id(group),
            pair_id: id(pair),
            query_id: id(query),
            category,
            verdict,
            conflict_kind,
            assertion_ids: assertions.iter().map(|a| id(a)).collect(),
            rule_ids: rules.iter().map(|r| id(r)).collect(),
            region_ids: regions.iter().map(|r| id(r)).collect(),
            report_ref: id("report"),
        }
    }

    fn result(
        query: &str,
        category: VerifierCategory,
        verdict: Option<SolverVerdict>,
        core: Option<&[&str]>,
    ) -> VerifierResult {
        VerifierResult {
            query_id: id(query),
            category,
            verdict,
            model: None,
            unsat_core: core.map(|ids| ids.iter().map(|c| id(c)).collect()),
            solver_identity: identity(),
            diagnostics: vec![],
        }
    }

    fn lineage_row(finding: &str, doc: &str, regions: &[&str], rules: &[&str]) -> LineageRow {
        LineageRow {
            finding_id: id(finding),
            document_id: id(doc),
            region_ids: regions.iter().map(|r| id(r)).collect(),
            rule_ids: rules.iter().map(|r| id(r)).collect(),
            segment_ids: vec![id("seg.0")],
            statement_ids: vec![id("st.0")],
        }
    }

    fn sorted<T: Canonical>(items: Vec<T>) -> Vec<T> {
        canonical_set(items).unwrap()
    }

    /// The §8.6-shaped synthetic run: two documents whose region ids
    /// collide (document-local counters), a conflict group whose overlap
    /// probe answered sat (sequence_number 0, no report row) and whose deontic
    /// query cored (sequence_number 1, the finding), and a no-conflict group
    /// closed by an unsat overlap probe (sequence_number 0, the documented
    /// no-conflict result).
    struct World {
        bundle: TraceBundle,
        lineage: LineageIndex,
        graphs: Vec<SourceDocumentGraph>,
        results: Vec<VerifierResults>,
        diagnostics: Vec<DiagnosticRecord>,
    }

    fn world() -> World {
        let nodes = sorted(vec![
            TraceNode {
                node_id: id("report"),
                kind: TraceNodeKind::Report,
                path: "report.json".to_owned(),
                content_hash: None,
            },
            TraceNode {
                node_id: id("test_source.a"),
                kind: TraceNodeKind::Source,
                path: "corpus/test_sources/a.html".to_owned(),
                content_hash: Some(hash('1')),
            },
            TraceNode {
                node_id: id("test_source.b"),
                kind: TraceNodeKind::Source,
                path: "corpus/test_sources/b.html".to_owned(),
                content_hash: Some(hash('2')),
            },
        ]);
        let claims = sorted(vec![
            claim(
                "finding.group.g1.0",
                "group.g1",
                "q.g1.pair1",
                "q.g1.pair1.overlap",
                VerifierCategory::SemanticNoConflict,
                Some(SolverVerdict::Sat),
                None,
                &["ctx.test_source.a.rule.0", "ctx.test_source.b.rule.0"],
                &["test_source.a.rule.0", "test_source.b.rule.0"],
                &["r.0"],
            ),
            claim(
                "finding.group.g1.1",
                "group.g1",
                "q.g1.pair1",
                "q.g1.pair1.deontic",
                VerifierCategory::SemanticContradiction,
                Some(SolverVerdict::Unsat),
                Some(ConflictKind::DeonticDirectionConflict),
                &["a.test_source.a.rule.0", "a.test_source.b.rule.0"],
                &["test_source.a.rule.0", "test_source.b.rule.0"],
                &["r.0"],
            ),
            claim(
                "finding.group.g2.0",
                "group.g2",
                "q.g2.pair1",
                "q.g2.pair1.overlap",
                VerifierCategory::SemanticNoConflict,
                Some(SolverVerdict::Unsat),
                None,
                &["ctx.test_source.a.rule.0", "ctx.test_source.b.rule.0"],
                &["test_source.a.rule.0", "test_source.b.rule.0"],
                &["r.0"],
            ),
        ]);
        let bundle = TraceBundle {
            nodes,
            edges: vec![],
            claims,
        };
        let lineage = LineageIndex {
            rows: sorted(vec![
                lineage_row(
                    "finding.group.g1.0",
                    "test_source.a",
                    &["r.0"],
                    &["test_source.a.rule.0"],
                ),
                lineage_row(
                    "finding.group.g1.0",
                    "test_source.b",
                    &["r.0"],
                    &["test_source.b.rule.0"],
                ),
                lineage_row(
                    "finding.group.g1.1",
                    "test_source.a",
                    &["r.0"],
                    &["test_source.a.rule.0"],
                ),
                lineage_row(
                    "finding.group.g1.1",
                    "test_source.b",
                    &["r.0"],
                    &["test_source.b.rule.0"],
                ),
                lineage_row(
                    "finding.group.g2.0",
                    "test_source.a",
                    &["r.0"],
                    &["test_source.a.rule.0"],
                ),
                lineage_row(
                    "finding.group.g2.0",
                    "test_source.b",
                    &["r.0"],
                    &["test_source.b.rule.0"],
                ),
            ]),
        };
        let graphs = vec![
            graph(
                "test_source.a",
                &[("s.0", "投与を推奨する")],
                &[("r.0", &["s.0"])],
            ),
            graph(
                "test_source.b",
                &[("s.0", "投与しないこと")],
                &[("r.0", &["s.0"])],
            ),
        ];
        let results = vec![
            VerifierResults {
                results: vec![
                    result(
                        "q.g1.pair1.overlap",
                        VerifierCategory::SemanticNoConflict,
                        Some(SolverVerdict::Sat),
                        None,
                    ),
                    result(
                        "q.g1.pair1.deontic",
                        VerifierCategory::SemanticContradiction,
                        Some(SolverVerdict::Unsat),
                        Some(&["a.test_source.a.rule.0", "a.test_source.b.rule.0"]),
                    ),
                ],
            },
            VerifierResults {
                results: vec![result(
                    "q.g2.pair1.overlap",
                    VerifierCategory::SemanticNoConflict,
                    Some(SolverVerdict::Unsat),
                    None,
                )],
            },
        ];
        let diagnostic = |code: DiagnosticCode, outcome: Outcome| DiagnosticRecord {
            code,
            outcome,
            payload: vec![],
            region_ids: vec![],
            artifact_hashes: vec![],
        };
        let diagnostics = vec![
            diagnostic(DiagnosticCode::SolverTimeout, Outcome::Residual),
            diagnostic(DiagnosticCode::SchemaInvalid, Outcome::Invalid),
            diagnostic(DiagnosticCode::SolverTimeout, Outcome::Residual),
        ];
        World {
            bundle,
            lineage,
            graphs,
            results,
            diagnostics,
        }
    }

    fn assemble(world: &World) -> Result<Report, ReportError> {
        assemble_report(
            &world.bundle,
            &world.lineage,
            &world.graphs.iter().collect::<Vec<_>>(),
            &world.results.iter().collect::<Vec<_>>(),
            &hash('f'),
            &identity(),
            &world.diagnostics,
        )
    }

    #[test]
    fn assembles_the_m1_shaped_report() {
        let report = assemble(&world()).unwrap();
        report.validate().unwrap();

        assert_eq!(
            report.corpus_hashes,
            vec![
                (id("test_source.a"), hash('1')),
                (id("test_source.b"), hash('2'))
            ]
        );
        assert_eq!(report.lexicon_hash, hash('f'));
        assert_eq!(
            report.diagnostics_summary,
            vec![(id("schema_invalid"), 1), (id("solver_timeout"), 2)]
        );
        assert_eq!(report.replay_status, ReplayStatus::NotReplayed);
        assert_eq!(report.solver_identity, identity());
        assert_eq!(
            report.wording,
            vec![
                Wording::DocumentedNoConflictResult,
                Wording::SyntheticTestSourceMeasurement
            ]
        );

        // The overlap-sat precondition row reaches neither partition.
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.no_conflict_results.len(), 1);

        let finding = &report.findings[0];
        assert_eq!(finding.finding_id, id("finding.group.g1.1"));
        assert_eq!(finding.query_id, id("q.g1.pair1.deontic"));
        assert_eq!(
            finding.conflict_kind,
            Some(ConflictKind::DeonticDirectionConflict)
        );
        assert_eq!(finding.verdict, SolverVerdict::Unsat);
        assert_eq!(finding.claim_tier, ClaimTier::S1Accepted);
        assert_eq!(finding.wording, Wording::SyntheticTestSourceMeasurement);
        assert_eq!(
            finding.assertion_ids,
            vec![id("a.test_source.a.rule.0"), id("a.test_source.b.rule.0")]
        );
        assert_eq!(
            finding.core,
            Some(vec![
                id("a.test_source.a.rule.0"),
                id("a.test_source.b.rule.0")
            ])
        );
        assert_eq!(
            finding.rule_ids,
            vec![id("test_source.a.rule.0"), id("test_source.b.rule.0")]
        );
        assert_eq!(finding.region_ids, vec![id("r.0")]);
        // The colliding document-local region id resolves per document.
        assert_eq!(
            finding.quoted_spans,
            vec![
                QuotedSpan {
                    document_id: id("test_source.a"),
                    region_id: id("r.0"),
                    span_id: id("s.0"),
                    text: "投与を推奨する".to_owned(),
                },
                QuotedSpan {
                    document_id: id("test_source.b"),
                    region_id: id("r.0"),
                    span_id: id("s.0"),
                    text: "投与しないこと".to_owned(),
                },
            ]
        );

        let no_conflict = &report.no_conflict_results[0];
        assert_eq!(no_conflict.finding_id, id("finding.group.g2.0"));
        assert_eq!(no_conflict.query_id, id("q.g2.pair1.overlap"));
        assert_eq!(no_conflict.conflict_kind, None);
        assert_eq!(no_conflict.core, None);
        assert_eq!(no_conflict.verdict, SolverVerdict::Unsat);
        assert_eq!(no_conflict.wording, Wording::DocumentedNoConflictResult);
        assert_eq!(
            no_conflict.assertion_ids,
            vec![
                id("ctx.test_source.a.rule.0"),
                id("ctx.test_source.b.rule.0")
            ]
        );
        assert_eq!(no_conflict.quoted_spans.len(), 2);
    }

    #[test]
    fn assembled_report_round_trips_canonically() {
        let report = assemble(&world()).unwrap();
        let bytes = canonical_payload_bytes(&report).unwrap();
        let read: Report = read_strict_canonical(&bytes).unwrap();
        assert_eq!(read, report);
        read.validate().unwrap();
        // Pin the §0 spellings and the code-keyed summary form.
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains(
            r#""wording":["documented no-conflict result","synthetic test source measurement"]"#
        ));
        assert!(
            text.contains(r#""diagnostics_summary":{"schema_invalid":"1","solver_timeout":"2"}"#)
        );
        assert!(text.contains(r#""replay_status":"not_replayed""#));
    }

    #[test]
    fn deontic_sat_closes_as_a_no_conflict_result() {
        let mut world = world();
        // Rebuild g1: overlap sat, then a consistent (sat) deontic query.
        world.bundle.claims = sorted(vec![
            claim(
                "finding.group.g1.0",
                "group.g1",
                "q.g1.pair1",
                "q.g1.pair1.overlap",
                VerifierCategory::SemanticNoConflict,
                Some(SolverVerdict::Sat),
                None,
                &["ctx.test_source.a.rule.0", "ctx.test_source.b.rule.0"],
                &["test_source.a.rule.0", "test_source.b.rule.0"],
                &["r.0"],
            ),
            claim(
                "finding.group.g1.1",
                "group.g1",
                "q.g1.pair1",
                "q.g1.pair1.deontic",
                VerifierCategory::SemanticNoConflict,
                Some(SolverVerdict::Sat),
                None,
                &["a.test_source.a.rule.0", "a.test_source.b.rule.0"],
                &["test_source.a.rule.0", "test_source.b.rule.0"],
                &["r.0"],
            ),
        ]);
        world
            .lineage
            .rows
            .retain(|r| !r.finding_id.as_str().starts_with("finding.group.g2."));
        world.results = vec![VerifierResults {
            results: vec![
                result(
                    "q.g1.pair1.overlap",
                    VerifierCategory::SemanticNoConflict,
                    Some(SolverVerdict::Sat),
                    None,
                ),
                result(
                    "q.g1.pair1.deontic",
                    VerifierCategory::SemanticNoConflict,
                    Some(SolverVerdict::Sat),
                    None,
                ),
            ],
        }];
        let report = assemble(&world).unwrap();
        report.validate().unwrap();
        assert!(report.findings.is_empty());
        assert_eq!(report.no_conflict_results.len(), 1);
        let no_conflict = &report.no_conflict_results[0];
        assert_eq!(no_conflict.finding_id, id("finding.group.g1.1"));
        assert_eq!(no_conflict.verdict, SolverVerdict::Sat);
        assert_eq!(no_conflict.wording, Wording::DocumentedNoConflictResult);
        assert_eq!(report.wording, vec![Wording::DocumentedNoConflictResult]);
    }

    #[test]
    fn missing_graph_is_an_assembly_error() {
        let mut world = world();
        world
            .graphs
            .retain(|g| g.document.document_id.as_str() != "test_source.b");
        assert_eq!(
            assemble(&world),
            Err(ReportError::MissingGraph(id("test_source.b")))
        );
    }

    #[test]
    fn missing_region_is_an_assembly_error() {
        let mut world = world();
        world.graphs[0].regions.clear();
        assert_eq!(
            assemble(&world),
            Err(ReportError::MissingRegion {
                document_id: id("test_source.a"),
                region_id: id("r.0"),
            })
        );
    }

    #[test]
    fn missing_span_is_an_assembly_error() {
        let mut world = world();
        world.graphs[0].spans.clear();
        assert_eq!(
            assemble(&world),
            Err(ReportError::MissingSpan {
                document_id: id("test_source.a"),
                span_id: id("s.0"),
            })
        );
    }

    #[test]
    fn missing_result_is_an_assembly_error() {
        let mut world = world();
        world.results[0]
            .results
            .retain(|r| r.query_id.as_str() != "q.g1.pair1.deontic");
        assert_eq!(
            assemble(&world),
            Err(ReportError::MissingResult(id("q.g1.pair1.deontic")))
        );
    }

    #[test]
    fn missing_lineage_is_an_assembly_error() {
        let mut world = world();
        world
            .lineage
            .rows
            .retain(|r| r.finding_id.as_str() != "finding.group.g1.1");
        assert_eq!(
            assemble(&world),
            Err(ReportError::MissingLineage(id("finding.group.g1.1")))
        );
    }

    #[test]
    fn duplicate_graph_is_an_assembly_error() {
        let mut world = world();
        let copy = world.graphs[0].clone();
        world.graphs.push(copy);
        assert_eq!(
            assemble(&world),
            Err(ReportError::DuplicateGraph(id("test_source.a")))
        );
    }

    #[test]
    fn duplicate_result_is_an_assembly_error() {
        let mut world = world();
        let copy = world.results[1].results[0].clone();
        world.results[1].results.push(copy);
        assert_eq!(
            assemble(&world),
            Err(ReportError::DuplicateResult(id("q.g2.pair1.overlap")))
        );
    }

    #[test]
    fn unknown_query_suffix_is_an_assembly_error() {
        let mut world = world();
        for claim in &mut world.bundle.claims {
            if claim.finding_id.as_str() == "finding.group.g2.0" {
                claim.query_id = id("q.g2.pair1.probe");
            }
        }
        assert_eq!(
            assemble(&world),
            Err(ReportError::UnknownQueryRole(id("q.g2.pair1.probe")))
        );
    }

    #[test]
    fn deontic_unsat_no_conflict_is_a_role_error() {
        let mut world = world();
        for claim in &mut world.bundle.claims {
            if claim.finding_id.as_str() == "finding.group.g1.1" {
                claim.category = VerifierCategory::SemanticNoConflict;
                claim.conflict_kind = None;
            }
        }
        assert_eq!(
            assemble(&world),
            Err(ReportError::RoleVerdict(id("q.g1.pair1.deontic")))
        );
    }

    #[test]
    fn pair_disagreement_is_an_assembly_error() {
        let mut world = world();
        for row in &mut world.lineage.rows {
            if row.finding_id.as_str() == "finding.group.g1.1"
                && row.document_id.as_str() == "test_source.a"
            {
                row.region_ids = vec![id("r.0"), id("r.9")];
            }
        }
        assert_eq!(
            assemble(&world),
            Err(ReportError::PairDisagreement {
                finding_id: id("finding.group.g1.1"),
                pool: "region_ids",
            })
        );
    }

    #[test]
    fn result_mismatch_is_an_assembly_error() {
        let mut world = world();
        world.results[1].results[0].verdict = Some(SolverVerdict::Sat);
        assert_eq!(
            assemble(&world),
            Err(ReportError::ResultMismatch(id("q.g2.pair1.overlap")))
        );
    }

    /// The full `report_en.md` body of [`valid_report`], pinned from
    /// observed output: §7.2 prose order, code-spanned ids and hashes,
    /// the finding-only optionals (conflict kind, core) absent on the
    /// no-conflict row, span texts plain and verbatim.
    const PINNED_MARKDOWN: &str = r#"# CKC report

wording: documented no-conflict result, synthetic test source measurement

## Corpus

| document | source hash |
| --- | --- |
| `test_source.a` | `sha256:1111111111111111111111111111111111111111111111111111111111111111` |
| `test_source.b` | `sha256:2222222222222222222222222222222222222222222222222222222222222222` |

lexicon hash: `sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff`

## Findings

### `finding.group.g1.1`

synthetic test source measurement; claim tier `s1_accepted`.

- conflict kind: `deontic_direction_conflict`
- query: `q.g1.pair1.deontic`, verdict `unsat`
- rules: `test_source.a.rule.0`, `test_source.b.rule.0`
- regions: `r.0`
- assertions: `a.test_source.a.rule.0`, `a.test_source.b.rule.0`
- core: `a.test_source.a.rule.0`, `a.test_source.b.rule.0`
- quoted spans:
  - `test_source.a` `r.0` `s.0`: administer drug A
  - `test_source.b` `r.0` `s.0`: withhold drug A

## Documented no-conflict results

### `finding.group.g2.0`

documented no-conflict result; claim tier `s1_accepted`.

- query: `q.g2.pair1.overlap`, verdict `unsat`
- rules: `test_source.a.rule.1`, `test_source.b.rule.1`
- regions: `r.1`
- assertions: `ctx.test_source.a.rule.1`, `ctx.test_source.b.rule.1`
- quoted spans:
  - `test_source.a` `r.1` `s.1`: adults eighteen and over
  - `test_source.b` `r.1` `s.1`: children under eighteen

## Diagnostics summary

| code | count |
| --- | --- |
| `schema_invalid` | 1 |
| `solver_timeout` | 2 |

## Solver identity

`z3` version `4.13.0`

## Replay status

`not_replayed`
"#;

    #[test]
    fn render_markdown_pins_the_derived_view() {
        let report = valid_report();
        report.validate().unwrap();
        let md = render_markdown(&report);
        assert_eq!(md, render_markdown(&report));
        assert_eq!(md, PINNED_MARKDOWN);
    }

    /// Every §7.2 content slot stays visible when empty — pinned from
    /// observed output over the all-empty report.
    #[test]
    fn render_markdown_marks_empty_slots() {
        let report = Report {
            corpus_hashes: vec![],
            diagnostics_summary: vec![],
            findings: vec![],
            lexicon_hash: hash('f'),
            no_conflict_results: vec![],
            replay_status: ReplayStatus::NotReplayed,
            solver_identity: SolverIdentity {
                solver_id: id("z3"),
                version: "4.13.0".to_owned(),
            },
            wording: vec![],
        };
        report.validate().unwrap();
        assert_eq!(
            render_markdown(&report),
            r#"# CKC report

wording: none.

## Corpus

none.

lexicon hash: `sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff`

## Findings

none.

## Documented no-conflict results

none.

## Diagnostics summary

none.

## Solver identity

`z3` version `4.13.0`

## Replay status

`not_replayed`
"#
        );
    }
}
