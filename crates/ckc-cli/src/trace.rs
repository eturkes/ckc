//! SPEC §7.1 trace payloads — the trace stage's durable shapes:
//! [`TraceBundle`] (`trace_bundle.json`), the run's derivation DAG plus
//! claim-evidence rows, and [`LineageIndex`] (`lineage_index.json`), its
//! per-(finding, document) query index.
//!
//! This module owns the types, their canonical bytes (every collection a
//! canonical set sorted by [`canonical_sort_key`]), and structural
//! validation; assembly over landed stage artifacts arrives with
//! cli-runner.3a.2 and the run wiring with cli-runner.3a.3 (§8.3 trace
//! stage row).

use std::collections::BTreeMap;

use ckc_core::{
    CanonError, CanonRead, CanonReadError, Canonical, Hash, Id, ObjectEmitter, ObjectReader,
    Reader, canonical_sort_key, emit_set, emit_string, fieldless_enum, read_set, read_string,
};
use ckc_smt::{SolverVerdict, VerifierCategory};

fieldless_enum! {
    /// SPEC §7.1 derivation-DAG node kind, one per §8.3 durable artifact
    /// class on the source → report chain. Declaration order is
    /// [`rank`](TraceNodeKind::rank) order; edges ascend strictly.
    TraceNodeKind {
        /// A corpus fixture document (raw bytes, corpus-relative path).
        Source => "source",
        /// The extract stage's SourceGraph artifact.
        SourceGraph => "source_graph",
        /// The segment stage's ClinicalSegments artifact.
        Segments => "segments",
        /// The normalize stage's statement + rule layers artifact.
        Normalization => "normalization",
        /// The assemble stage's five-layer IRBundle artifact.
        IrBundle => "ir_bundle",
        /// The compile stage's per-group CompiledArtifact.
        Compiled => "compiled",
        /// The verify stage's per-group VerifierResults artifact.
        VerifierResults => "verifier_results",
        /// The run's one report node (`report.json`), the DAG sink — the
        /// only kind without a content hash.
        Report => "report",
    }
}

impl TraceNodeKind {
    /// Pipeline depth, 0..=7 in declaration order; edges go strictly
    /// rank-upward.
    pub fn rank(self) -> u8 {
        match self {
            TraceNodeKind::Source => 0,
            TraceNodeKind::SourceGraph => 1,
            TraceNodeKind::Segments => 2,
            TraceNodeKind::Normalization => 3,
            TraceNodeKind::IrBundle => 4,
            TraceNodeKind::Compiled => 5,
            TraceNodeKind::VerifierResults => 6,
            TraceNodeKind::Report => 7,
        }
    }

    /// The §8.3 stage that produces nodes of this kind — the operation
    /// every incoming edge carries; sources have no producer.
    pub fn operation(self) -> Option<TraceOperation> {
        match self {
            TraceNodeKind::Source => None,
            TraceNodeKind::SourceGraph => Some(TraceOperation::Extract),
            TraceNodeKind::Segments => Some(TraceOperation::Segment),
            TraceNodeKind::Normalization => Some(TraceOperation::Normalize),
            TraceNodeKind::IrBundle => Some(TraceOperation::Assemble),
            TraceNodeKind::Compiled => Some(TraceOperation::Compile),
            TraceNodeKind::VerifierResults => Some(TraceOperation::Verify),
            TraceNodeKind::Report => Some(TraceOperation::Report),
        }
    }
}

fieldless_enum! {
    /// SPEC §7.1 edge label: the §8.3 stage kind that derived the target
    /// node, spelled exactly as the run's stage events spell it.
    TraceOperation {
        Extract => "extract",
        Segment => "segment",
        Normalize => "normalize",
        Assemble => "assemble",
        Compile => "compile",
        Verify => "verify",
        Report => "report",
    }
}

fieldless_enum! {
    /// SPEC §6 conflict kind carried by `semantic_contradiction` findings;
    /// M1's single kind (opposed direction groups under satisfiable shared
    /// context).
    ConflictKind {
        DeonticDirectionConflict => "deontic_direction_conflict",
    }
}

/// SPEC §7.1 DAG node: one durable artifact (or source document) at its
/// run-relative path (§8.3 layout; sources corpus-relative).
/// `content_hash` is the envelope content hash (sources: the raw-byte
/// hash) and is absent exactly on the report node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceNode {
    pub node_id: Id,
    pub kind: TraceNodeKind,
    pub path: String,
    pub content_hash: Option<Hash>,
}

impl Canonical for TraceNode {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.optional("content_hash", self.content_hash.as_ref(), |b, h| {
            h.emit_canonical(b)
        })?;
        obj.member("kind", |b| self.kind.emit_canonical(b))?;
        obj.member("node_id", |b| self.node_id.emit_canonical(b))?;
        obj.member("path", |b| {
            emit_string(b, &self.path);
            Ok(())
        })?;
        obj.finish(out)
    }
}

impl CanonRead for TraceNode {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let content_hash = obj.optional("content_hash", Hash::read)?;
        let kind = obj.member("kind", TraceNodeKind::read)?;
        let node_id = obj.member("node_id", Id::read)?;
        let path = obj.member("path", read_string)?;
        obj.close()?;
        Ok(TraceNode {
            node_id,
            kind,
            path,
            content_hash,
        })
    }
}

/// SPEC §7.1 operation-labeled DAG edge: `to` was derived from `from` by
/// the `operation` stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEdge {
    pub from: Id,
    pub operation: TraceOperation,
    pub to: Id,
}

impl Canonical for TraceEdge {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("from", |b| self.from.emit_canonical(b))?;
        obj.member("operation", |b| self.operation.emit_canonical(b))?;
        obj.member("to", |b| self.to.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for TraceEdge {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let from = obj.member("from", Id::read)?;
        let operation = obj.member("operation", TraceOperation::read)?;
        let to = obj.member("to", Id::read)?;
        obj.close()?;
        Ok(TraceEdge {
            from,
            operation,
            to,
        })
    }
}

/// SPEC §7.1 claim-evidence row: one verifier result traced from its §7.2
/// finding id through its query, named assertions, rules, and regions to
/// the report node. Optionals omit canonically when absent: `verdict`
/// follows the §6 category coherence rules and `conflict_kind` rides
/// exactly on `semantic_contradiction` rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimEvidenceRow {
    pub finding_id: Id,
    pub group_id: Id,
    pub pair_id: Id,
    pub query_id: Id,
    pub category: VerifierCategory,
    pub verdict: Option<SolverVerdict>,
    pub conflict_kind: Option<ConflictKind>,
    pub assertion_ids: Vec<Id>,
    pub rule_ids: Vec<Id>,
    pub region_ids: Vec<Id>,
    pub report_ref: Id,
}

impl Canonical for ClaimEvidenceRow {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("assertion_ids", |b| emit_set(b, &self.assertion_ids))?;
        obj.member("category", |b| self.category.emit_canonical(b))?;
        obj.optional("conflict_kind", self.conflict_kind, |b, k| {
            k.emit_canonical(b)
        })?;
        obj.member("finding_id", |b| self.finding_id.emit_canonical(b))?;
        obj.member("group_id", |b| self.group_id.emit_canonical(b))?;
        obj.member("pair_id", |b| self.pair_id.emit_canonical(b))?;
        obj.member("query_id", |b| self.query_id.emit_canonical(b))?;
        obj.member("region_ids", |b| emit_set(b, &self.region_ids))?;
        obj.member("report_ref", |b| self.report_ref.emit_canonical(b))?;
        obj.member("rule_ids", |b| emit_set(b, &self.rule_ids))?;
        obj.optional("verdict", self.verdict, |b, v| v.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for ClaimEvidenceRow {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let assertion_ids = obj.member("assertion_ids", read_set::<Id>)?;
        let category = obj.member("category", VerifierCategory::read)?;
        let conflict_kind = obj.optional("conflict_kind", ConflictKind::read)?;
        let finding_id = obj.member("finding_id", Id::read)?;
        let group_id = obj.member("group_id", Id::read)?;
        let pair_id = obj.member("pair_id", Id::read)?;
        let query_id = obj.member("query_id", Id::read)?;
        let region_ids = obj.member("region_ids", read_set::<Id>)?;
        let report_ref = obj.member("report_ref", Id::read)?;
        let rule_ids = obj.member("rule_ids", read_set::<Id>)?;
        let verdict = obj.optional("verdict", SolverVerdict::read)?;
        obj.close()?;
        Ok(ClaimEvidenceRow {
            finding_id,
            group_id,
            pair_id,
            query_id,
            category,
            verdict,
            conflict_kind,
            assertion_ids,
            rule_ids,
            region_ids,
            report_ref,
        })
    }
}

/// SPEC §5 `TraceBundle`: the derivation DAG (nodes + operation-labeled
/// edges) and the claim-evidence rows — `trace_bundle.json` in the §8.3
/// run layout. All three collections are canonical sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceBundle {
    pub nodes: Vec<TraceNode>,
    pub edges: Vec<TraceEdge>,
    pub claims: Vec<ClaimEvidenceRow>,
}

impl TraceBundle {
    /// Structural invariants, first break wins:
    ///
    /// 1. `nodes`, `edges`, and `claims` are stored as canonical sets
    ///    (sorted by [`canonical_sort_key`], duplicate-free) — so stored
    ///    values equal their strict-read round trip; duplicate edges break
    ///    here.
    /// 2. Nodes: `node_id`s unique; `path`s non-empty; `content_hash`
    ///    absent exactly on the report kind; at most one report node (the
    ///    report node).
    /// 3. Edges: both endpoints resolve to nodes; strictly rank-upward
    ///    (`rank(from) < rank(to)`); `operation` is the target kind's
    ///    producing operation.
    /// 4. Claims: `finding_id` takes the §7.2 form
    ///    `finding.<group_id>.<ordinal>` (ordinal a canonical decimal)
    ///    with each group's ordinals dense from 0; `query_id` extends
    ///    `pair_id` by a non-empty `.`-suffix; category ↔ verdict per the
    ///    §6 rules and `conflict_kind` present exactly on
    ///    `semantic_contradiction` rows; evidence sets non-empty canonical
    ///    sets; `report_ref` is the report node.
    pub fn validate(&self) -> Result<(), TraceError> {
        check_canonical_set("nodes", &self.nodes)?;
        let mut kinds: BTreeMap<&str, TraceNodeKind> = BTreeMap::new();
        let mut report_id: Option<&Id> = None;
        for node in &self.nodes {
            if kinds.insert(node.node_id.as_str(), node.kind).is_some() {
                return Err(TraceError::DuplicateNodeId(node.node_id.clone()));
            }
            if node.path.is_empty() {
                return Err(TraceError::EmptyPath(node.node_id.clone()));
            }
            if node.content_hash.is_none() != (node.kind == TraceNodeKind::Report) {
                return Err(TraceError::HashPresence(node.node_id.clone()));
            }
            if node.kind == TraceNodeKind::Report {
                if report_id.is_some() {
                    return Err(TraceError::SecondReportNode(node.node_id.clone()));
                }
                report_id = Some(&node.node_id);
            }
        }
        check_canonical_set("edges", &self.edges)?;
        for edge in &self.edges {
            let Some(&from_kind) = kinds.get(edge.from.as_str()) else {
                return Err(TraceError::EdgeUnresolved {
                    end: "from",
                    id: edge.from.clone(),
                });
            };
            let Some(&to_kind) = kinds.get(edge.to.as_str()) else {
                return Err(TraceError::EdgeUnresolved {
                    end: "to",
                    id: edge.to.clone(),
                });
            };
            if from_kind.rank() >= to_kind.rank() {
                return Err(TraceError::EdgeRank {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                });
            }
            if to_kind.operation() != Some(edge.operation) {
                return Err(TraceError::EdgeOperation {
                    to: edge.to.clone(),
                    operation: edge.operation,
                });
            }
        }
        check_canonical_set("claims", &self.claims)?;
        let mut ordinals: BTreeMap<&str, Vec<u64>> = BTreeMap::new();
        for claim in &self.claims {
            let Some(ordinal) = parse_finding_ordinal(&claim.finding_id, &claim.group_id) else {
                return Err(TraceError::FindingIdForm(claim.finding_id.clone()));
            };
            ordinals
                .entry(claim.group_id.as_str())
                .or_default()
                .push(ordinal);
            let (query, pair) = (claim.query_id.as_str(), claim.pair_id.as_str());
            if query.len() <= pair.len() + 1
                || !query.starts_with(pair)
                || query.as_bytes()[pair.len()] != b'.'
            {
                return Err(TraceError::QueryOutsidePair {
                    query_id: claim.query_id.clone(),
                    pair_id: claim.pair_id.clone(),
                });
            }
            let (coherent, rule) = category_verdict_rule(claim.category, claim.verdict);
            if !coherent {
                return Err(TraceError::CategoryVerdict {
                    category: claim.category,
                    rule,
                });
            }
            if claim.conflict_kind.is_some()
                != (claim.category == VerifierCategory::SemanticContradiction)
            {
                return Err(TraceError::ConflictKindPresence(claim.finding_id.clone()));
            }
            for (pool, set) in [
                ("assertion_ids", &claim.assertion_ids),
                ("region_ids", &claim.region_ids),
                ("rule_ids", &claim.rule_ids),
            ] {
                if set.is_empty() {
                    return Err(TraceError::EmptyEvidence {
                        owner: claim.finding_id.clone(),
                        pool,
                    });
                }
                check_canonical_set(pool, set.iter())?;
            }
            if report_id.map(Id::as_str) != Some(claim.report_ref.as_str()) {
                return Err(TraceError::ReportRef {
                    finding_id: claim.finding_id.clone(),
                    report_ref: claim.report_ref.clone(),
                });
            }
        }
        for (group, mut ords) in ordinals {
            ords.sort_unstable();
            if ords.iter().enumerate().any(|(i, o)| *o != i as u64) {
                return Err(TraceError::FindingOrdinals(
                    Id::new(group).expect("group ids are valid"),
                ));
            }
        }
        Ok(())
    }
}

impl Canonical for TraceBundle {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("claims", |b| emit_set(b, &self.claims))?;
        obj.member("edges", |b| emit_set(b, &self.edges))?;
        obj.member("nodes", |b| emit_set(b, &self.nodes))?;
        obj.finish(out)
    }
}

impl CanonRead for TraceBundle {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let claims = obj.member("claims", read_set::<ClaimEvidenceRow>)?;
        let edges = obj.member("edges", read_set::<TraceEdge>)?;
        let nodes = obj.member("nodes", read_set::<TraceNode>)?;
        obj.close()?;
        Ok(TraceBundle {
            nodes,
            edges,
            claims,
        })
    }
}

/// SPEC §5 `LineageIndex` row: one finding's references within one member
/// document — regions, rules, segments, statements — the `ckc trace`
/// resolution unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageRow {
    pub finding_id: Id,
    pub document_id: Id,
    pub region_ids: Vec<Id>,
    pub rule_ids: Vec<Id>,
    pub segment_ids: Vec<Id>,
    pub statement_ids: Vec<Id>,
}

impl Canonical for LineageRow {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("document_id", |b| self.document_id.emit_canonical(b))?;
        obj.member("finding_id", |b| self.finding_id.emit_canonical(b))?;
        obj.member("region_ids", |b| emit_set(b, &self.region_ids))?;
        obj.member("rule_ids", |b| emit_set(b, &self.rule_ids))?;
        obj.member("segment_ids", |b| emit_set(b, &self.segment_ids))?;
        obj.member("statement_ids", |b| emit_set(b, &self.statement_ids))?;
        obj.finish(out)
    }
}

impl CanonRead for LineageRow {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let document_id = obj.member("document_id", Id::read)?;
        let finding_id = obj.member("finding_id", Id::read)?;
        let region_ids = obj.member("region_ids", read_set::<Id>)?;
        let rule_ids = obj.member("rule_ids", read_set::<Id>)?;
        let segment_ids = obj.member("segment_ids", read_set::<Id>)?;
        let statement_ids = obj.member("statement_ids", read_set::<Id>)?;
        obj.close()?;
        Ok(LineageRow {
            finding_id,
            document_id,
            region_ids,
            rule_ids,
            segment_ids,
            statement_ids,
        })
    }
}

/// SPEC §5 `LineageIndex`: the TraceBundle's query index, one row per
/// (finding, member document) — `lineage_index.json` in the §8.3 run
/// layout. `rows` is a canonical set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageIndex {
    pub rows: Vec<LineageRow>,
}

impl LineageIndex {
    /// Structural invariants, first break wins: `rows` is a canonical set;
    /// each row's four reference sets are non-empty canonical sets; rows
    /// unique per (finding_id, document_id).
    pub fn validate(&self) -> Result<(), TraceError> {
        check_canonical_set("rows", &self.rows)?;
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for row in &self.rows {
            let key = (row.finding_id.as_str(), row.document_id.as_str());
            if seen.contains(&key) {
                return Err(TraceError::DuplicateLineageRow {
                    finding_id: row.finding_id.clone(),
                    document_id: row.document_id.clone(),
                });
            }
            seen.push(key);
            for (pool, set) in [
                ("region_ids", &row.region_ids),
                ("rule_ids", &row.rule_ids),
                ("segment_ids", &row.segment_ids),
                ("statement_ids", &row.statement_ids),
            ] {
                if set.is_empty() {
                    return Err(TraceError::EmptyEvidence {
                        owner: row.finding_id.clone(),
                        pool,
                    });
                }
                check_canonical_set(pool, set.iter())?;
            }
        }
        Ok(())
    }
}

impl Canonical for LineageIndex {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("rows", |b| emit_set(b, &self.rows))?;
        obj.finish(out)
    }
}

impl CanonRead for LineageIndex {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let rows = obj.member("rows", read_set::<LineageRow>)?;
        obj.close()?;
        Ok(LineageIndex { rows })
    }
}

/// Enforce canonical-set storage — strictly ascending by
/// [`canonical_sort_key`], duplicate-free — so stored values equal their
/// strict-read round trip (`pool` names the set in errors, `index` the
/// offending element).
pub fn check_canonical_set<'a, T: Canonical + 'a>(
    pool: &'static str,
    items: impl IntoIterator<Item = &'a T>,
) -> Result<(), TraceError> {
    let mut prev: Option<Vec<u8>> = None;
    for (index, item) in items.into_iter().enumerate() {
        let key = canonical_sort_key(item)?;
        if let Some(p) = &prev {
            match p.as_slice().cmp(key.as_slice()) {
                std::cmp::Ordering::Equal => return Err(TraceError::Duplicate { pool, index }),
                std::cmp::Ordering::Greater => return Err(TraceError::Unsorted { pool, index }),
                std::cmp::Ordering::Less => {}
            }
        }
        prev = Some(key);
    }
    Ok(())
}

/// The §6 category ↔ verdict coherence rule (the verifier-result rules
/// restated over claim rows): whether `verdict` satisfies `category`, and
/// the rule's name for errors.
fn category_verdict_rule(
    category: VerifierCategory,
    verdict: Option<SolverVerdict>,
) -> (bool, &'static str) {
    match category {
        VerifierCategory::SemanticContradiction => (
            verdict == Some(SolverVerdict::Unsat),
            "semantic_contradiction requires verdict unsat",
        ),
        VerifierCategory::SemanticNoConflict => (
            matches!(
                verdict,
                Some(SolverVerdict::Sat) | Some(SolverVerdict::Unsat)
            ),
            "semantic_no_conflict requires verdict sat or unsat",
        ),
        VerifierCategory::Unknown => (
            matches!(
                verdict,
                Some(SolverVerdict::Unknown) | Some(SolverVerdict::Timeout)
            ),
            "unknown requires verdict unknown or timeout",
        ),
        VerifierCategory::SchemaFailure
        | VerifierCategory::CompilerFailure
        | VerifierCategory::TargetSyntaxFailure
        | VerifierCategory::SolverExecutionFailure
        | VerifierCategory::UnsupportedFragment => {
            (verdict.is_none(), "failure categories carry no verdict")
        }
    }
}

/// Parse the §7.2 finding-id form `finding.<group_id>.<ordinal>` against
/// the row's group, yielding the ordinal; the ordinal is a canonical
/// decimal — ASCII digits with no leading zero unless exactly `0`.
fn parse_finding_ordinal(finding_id: &Id, group_id: &Id) -> Option<u64> {
    let digits = finding_id
        .as_str()
        .strip_prefix("finding.")?
        .strip_prefix(group_id.as_str())?
        .strip_prefix('.')?;
    let canonical = !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && (digits.len() == 1 || !digits.starts_with('0'));
    if canonical { digits.parse().ok() } else { None }
}

/// A trace payload broke a structural invariant
/// ([`TraceBundle::validate`] / [`LineageIndex::validate`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceError {
    /// Canonical emission failed while computing a sort key.
    Canon(CanonError),
    /// Two set elements share canonical bytes (`pool` names the set,
    /// `index` the second element).
    Duplicate { pool: &'static str, index: usize },
    /// A set stores the element at `index` out of canonical order (`pool`
    /// names it).
    Unsorted { pool: &'static str, index: usize },
    /// Two nodes share the named `node_id`.
    DuplicateNodeId(Id),
    /// The named node's path is empty.
    EmptyPath(Id),
    /// The named node's content-hash presence disagrees with its kind
    /// (absent exactly on report).
    HashPresence(Id),
    /// A second report-kind node (the named one) joined the DAG.
    SecondReportNode(Id),
    /// An edge endpoint (`end` names which) resolves to no node.
    EdgeUnresolved { end: &'static str, id: Id },
    /// The edge `from` → `to` is not strictly rank-upward.
    EdgeRank { from: Id, to: Id },
    /// The edge into `to` carries an operation other than its kind's
    /// producing operation.
    EdgeOperation { to: Id, operation: TraceOperation },
    /// The named finding id lacks the §7.2 `finding.<group_id>.<ordinal>`
    /// form over its row's group.
    FindingIdForm(Id),
    /// The named group's finding ordinals are not dense from 0.
    FindingOrdinals(Id),
    /// The row's query id does not extend its pair id.
    QueryOutsidePair { query_id: Id, pair_id: Id },
    /// The row's category disagrees with its verdict (`rule` names the §6
    /// expectation).
    CategoryVerdict {
        category: VerifierCategory,
        rule: &'static str,
    },
    /// The named finding's `conflict_kind` presence disagrees with its
    /// category (present exactly on `semantic_contradiction`).
    ConflictKindPresence(Id),
    /// The named owner's evidence set `pool` is empty.
    EmptyEvidence { owner: Id, pool: &'static str },
    /// The named finding's `report_ref` is not the report node.
    ReportRef { finding_id: Id, report_ref: Id },
    /// Two lineage rows share (finding_id, document_id).
    DuplicateLineageRow { finding_id: Id, document_id: Id },
}

impl From<CanonError> for TraceError {
    fn from(e: CanonError) -> Self {
        TraceError::Canon(e)
    }
}

impl std::fmt::Display for TraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceError::Canon(e) => write!(f, "canonical emission failed: {e}"),
            TraceError::Duplicate { pool, index } => {
                write!(f, "{pool}[{index}] duplicates an earlier element")
            }
            TraceError::Unsorted { pool, index } => {
                write!(f, "{pool}[{index}] is stored out of canonical order")
            }
            TraceError::DuplicateNodeId(id) => write!(f, "duplicate node id {id}"),
            TraceError::EmptyPath(id) => write!(f, "node {id} has an empty path"),
            TraceError::HashPresence(id) => {
                write!(
                    f,
                    "node {id} breaks: hash absent exactly on the report node"
                )
            }
            TraceError::SecondReportNode(id) => write!(f, "second report node {id}"),
            TraceError::EdgeUnresolved { end, id } => {
                write!(f, "edge {end} {id} resolves to no node")
            }
            TraceError::EdgeRank { from, to } => {
                write!(f, "edge {from} -> {to} is not strictly rank-upward")
            }
            TraceError::EdgeOperation { to, operation } => write!(
                f,
                "edge into {to} carries {} instead of its producing operation",
                operation.as_str()
            ),
            TraceError::FindingIdForm(id) => write!(
                f,
                "finding id {id} is not finding.<group_id>.<ordinal> over its group"
            ),
            TraceError::FindingOrdinals(group) => {
                write!(f, "group {group} finding ordinals are not dense from 0")
            }
            TraceError::QueryOutsidePair { query_id, pair_id } => {
                write!(f, "query {query_id} is not under pair {pair_id}")
            }
            TraceError::CategoryVerdict { category, rule } => {
                write!(f, "category {} breaks: {rule}", category.as_str())
            }
            TraceError::ConflictKindPresence(id) => write!(
                f,
                "finding {id} breaks: conflict_kind present exactly on semantic_contradiction"
            ),
            TraceError::EmptyEvidence { owner, pool } => {
                write!(f, "{owner} has an empty {pool} set")
            }
            TraceError::ReportRef {
                finding_id,
                report_ref,
            } => write!(
                f,
                "finding {finding_id} report_ref {report_ref} is not the report node"
            ),
            TraceError::DuplicateLineageRow {
                finding_id,
                document_id,
            } => write!(f, "duplicate lineage row for ({finding_id}, {document_id})"),
        }
    }
}

impl std::error::Error for TraceError {}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::{canonical_payload_bytes, read_canonical};

    /// Canonical bytes of `value` as a UTF-8 string, for exact-match assertions.
    fn canon<T: Canonical>(value: &T) -> String {
        String::from_utf8(canonical_payload_bytes(value).unwrap()).unwrap()
    }

    /// Assert `value` survives a canonical write -> read round trip unchanged.
    fn round_trip<T: Canonical + CanonRead + std::fmt::Debug + PartialEq>(value: T) {
        let bytes = canonical_payload_bytes(&value).unwrap();
        let got: T = read_canonical(&bytes).unwrap();
        assert_eq!(got, value, "round trip changed the value");
    }

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn hash(fill: char) -> Hash {
        Hash::new(format!("sha256:{}", fill.to_string().repeat(64))).unwrap()
    }

    /// docA's source node: corpus-relative path, raw-byte hash.
    fn source_node() -> TraceNode {
        TraceNode {
            node_id: id("fixture.m1_guideline_a"),
            kind: TraceNodeKind::Source,
            path: "corpus/fixtures/m1_guideline_a.html".to_owned(),
            content_hash: Some(hash('a')),
        }
    }

    /// docA's extract landing: §8.3 run-relative path, envelope hash.
    fn graph_node() -> TraceNode {
        TraceNode {
            node_id: id("fixture.m1_guideline_a.source_graph"),
            kind: TraceNodeKind::SourceGraph,
            path: "artifacts/fixture.m1_guideline_a/source_graph.json".to_owned(),
            content_hash: Some(hash('b')),
        }
    }

    /// The run's one report node: static id, no hash.
    fn report_node() -> TraceNode {
        TraceNode {
            node_id: id("report"),
            kind: TraceNodeKind::Report,
            path: "report.json".to_owned(),
            content_hash: None,
        }
    }

    fn extract_edge() -> TraceEdge {
        TraceEdge {
            from: id("fixture.m1_guideline_a"),
            operation: TraceOperation::Extract,
            to: id("fixture.m1_guideline_a.source_graph"),
        }
    }

    /// §8.6 overlap row: finding ordinal 0, sat, no conflict kind, ctx
    /// assertions.
    fn overlap_claim() -> ClaimEvidenceRow {
        ClaimEvidenceRow {
            finding_id: id("finding.group.m1_conflict.0"),
            group_id: id("group.m1_conflict"),
            pair_id: id("q.m1_conflict.pair1"),
            query_id: id("q.m1_conflict.pair1.overlap"),
            category: VerifierCategory::SemanticNoConflict,
            verdict: Some(SolverVerdict::Sat),
            conflict_kind: None,
            assertion_ids: vec![
                id("ctx.fixture.m1_guideline_a.rule.0"),
                id("ctx.fixture.m1_guideline_b.rule.0"),
            ],
            rule_ids: vec![
                id("fixture.m1_guideline_a.rule.0"),
                id("fixture.m1_guideline_b.rule.0"),
            ],
            region_ids: vec![id("r.2"), id("r.3")],
            report_ref: id("report"),
        }
    }

    /// §8.6 deontic row: finding ordinal 1, unsat, the M1 conflict kind,
    /// the cross-document core as assertions.
    fn deontic_claim() -> ClaimEvidenceRow {
        ClaimEvidenceRow {
            finding_id: id("finding.group.m1_conflict.1"),
            group_id: id("group.m1_conflict"),
            pair_id: id("q.m1_conflict.pair1"),
            query_id: id("q.m1_conflict.pair1.deontic"),
            category: VerifierCategory::SemanticContradiction,
            verdict: Some(SolverVerdict::Unsat),
            conflict_kind: Some(ConflictKind::DeonticDirectionConflict),
            assertion_ids: vec![
                id("a.fixture.m1_guideline_a.rule.0"),
                id("a.fixture.m1_guideline_b.rule.0"),
            ],
            rule_ids: vec![
                id("fixture.m1_guideline_a.rule.0"),
                id("fixture.m1_guideline_b.rule.0"),
            ],
            region_ids: vec![id("r.2"), id("r.3")],
            report_ref: id("report"),
        }
    }

    /// Canonical-set storage order: claims by leading assertion id (a.* <
    /// ctx.*), nodes by hash then the hashless report last.
    fn sample_bundle() -> TraceBundle {
        TraceBundle {
            nodes: vec![source_node(), graph_node(), report_node()],
            edges: vec![extract_edge()],
            claims: vec![deontic_claim(), overlap_claim()],
        }
    }

    fn empty_bundle() -> TraceBundle {
        TraceBundle {
            nodes: vec![],
            edges: vec![],
            claims: vec![],
        }
    }

    fn sample_row() -> LineageRow {
        LineageRow {
            finding_id: id("finding.group.m1_conflict.1"),
            document_id: id("fixture.m1_guideline_a"),
            region_ids: vec![id("r.2"), id("r.3")],
            rule_ids: vec![id("fixture.m1_guideline_a.rule.0")],
            segment_ids: vec![id("seg.3")],
            statement_ids: vec![id("st.0")],
        }
    }

    fn sample_index() -> LineageIndex {
        LineageIndex {
            rows: vec![sample_row()],
        }
    }

    // Pins rank order, the producing-operation mapping, and the canonical
    // token spellings.
    #[test]
    fn kind_rank_and_operation() {
        let ranks: Vec<u8> = TraceNodeKind::ALL.iter().map(|k| k.rank()).collect();
        assert_eq!(ranks, (0..=7).collect::<Vec<u8>>());
        assert_eq!(TraceNodeKind::Source.operation(), None);
        let produced: Vec<TraceOperation> = TraceNodeKind::ALL[1..]
            .iter()
            .map(|k| k.operation().unwrap())
            .collect();
        assert_eq!(produced, TraceOperation::ALL.to_vec());
        assert_eq!(canon(&TraceNodeKind::SourceGraph), "\"source_graph\"");
        assert_eq!(canon(&TraceOperation::Extract), "\"extract\"");
        assert_eq!(
            canon(&ConflictKind::DeonticDirectionConflict),
            "\"deontic_direction_conflict\""
        );
        assert_eq!(
            TraceOperation::parse("verify").unwrap(),
            TraceOperation::Verify
        );
    }

    // Pins the §7.1 TraceBundle canonical shape over §8.6 values:
    // byte-sorted members, set-ordered collections, optional-omit
    // conflict_kind/verdict/content_hash.
    #[test]
    fn trace_bundle_canonical_bytes() {
        assert_eq!(
            canon(&sample_bundle()),
            concat!(
                r#"{"claims":["#,
                r#"{"assertion_ids":["a.fixture.m1_guideline_a.rule.0","#,
                r#""a.fixture.m1_guideline_b.rule.0"],"#,
                r#""category":"semantic_contradiction","#,
                r#""conflict_kind":"deontic_direction_conflict","#,
                r#""finding_id":"finding.group.m1_conflict.1","#,
                r#""group_id":"group.m1_conflict","#,
                r#""pair_id":"q.m1_conflict.pair1","#,
                r#""query_id":"q.m1_conflict.pair1.deontic","#,
                r#""region_ids":["r.2","r.3"],"report_ref":"report","#,
                r#""rule_ids":["fixture.m1_guideline_a.rule.0","#,
                r#""fixture.m1_guideline_b.rule.0"],"verdict":"unsat"},"#,
                r#"{"assertion_ids":["ctx.fixture.m1_guideline_a.rule.0","#,
                r#""ctx.fixture.m1_guideline_b.rule.0"],"#,
                r#""category":"semantic_no_conflict","#,
                r#""finding_id":"finding.group.m1_conflict.0","#,
                r#""group_id":"group.m1_conflict","#,
                r#""pair_id":"q.m1_conflict.pair1","#,
                r#""query_id":"q.m1_conflict.pair1.overlap","#,
                r#""region_ids":["r.2","r.3"],"report_ref":"report","#,
                r#""rule_ids":["fixture.m1_guideline_a.rule.0","#,
                r#""fixture.m1_guideline_b.rule.0"],"verdict":"sat"}],"#,
                r#""edges":[{"from":"fixture.m1_guideline_a","operation":"extract","#,
                r#""to":"fixture.m1_guideline_a.source_graph"}],"#,
                r#""nodes":[{"content_hash":"#,
                r#""sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","#,
                r#""kind":"source","node_id":"fixture.m1_guideline_a","#,
                r#""path":"corpus/fixtures/m1_guideline_a.html"},"#,
                r#"{"content_hash":"#,
                r#""sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","#,
                r#""kind":"source_graph","#,
                r#""node_id":"fixture.m1_guideline_a.source_graph","#,
                r#""path":"artifacts/fixture.m1_guideline_a/source_graph.json"},"#,
                r#"{"kind":"report","node_id":"report","path":"report.json"}]}"#
            )
        );
        assert_eq!(
            canon(&empty_bundle()),
            r#"{"claims":[],"edges":[],"nodes":[]}"#
        );
    }

    // Pins the §7.1 LineageIndex canonical shape: one row, byte-sorted
    // members, Id sets.
    #[test]
    fn lineage_index_canonical_bytes() {
        assert_eq!(
            canon(&sample_index()),
            concat!(
                r#"{"rows":[{"document_id":"fixture.m1_guideline_a","#,
                r#""finding_id":"finding.group.m1_conflict.1","#,
                r#""region_ids":["r.2","r.3"],"#,
                r#""rule_ids":["fixture.m1_guideline_a.rule.0"],"#,
                r#""segment_ids":["seg.3"],"statement_ids":["st.0"]}]}"#
            )
        );
        assert_eq!(canon(&LineageIndex { rows: vec![] }), r#"{"rows":[]}"#);
    }

    #[test]
    fn round_trips() {
        round_trip(sample_bundle());
        round_trip(empty_bundle());
        round_trip(sample_index());
        round_trip(LineageIndex { rows: vec![] });
    }

    #[test]
    fn validation_accepts_samples() {
        assert_eq!(sample_bundle().validate(), Ok(()));
        assert_eq!(empty_bundle().validate(), Ok(()));
        assert_eq!(sample_index().validate(), Ok(()));
        assert_eq!(LineageIndex { rows: vec![] }.validate(), Ok(()));
    }

    #[test]
    fn validation_rejects_node_breaks() {
        // Set storage: nodes out of canonical order / duplicated.
        let mut unsorted = sample_bundle();
        unsorted.nodes.swap(0, 1);
        assert_eq!(
            unsorted.validate(),
            Err(TraceError::Unsorted {
                pool: "nodes",
                index: 1,
            })
        );
        let mut dup = sample_bundle();
        dup.nodes.insert(1, source_node());
        assert_eq!(
            dup.validate(),
            Err(TraceError::Duplicate {
                pool: "nodes",
                index: 1,
            })
        );
        // Unique node ids: the graph node renamed onto the source id (set
        // order survives — bytes sort by hash).
        let mut dup_id = sample_bundle();
        dup_id.nodes[1].node_id = id("fixture.m1_guideline_a");
        assert_eq!(
            dup_id.validate(),
            Err(TraceError::DuplicateNodeId(id("fixture.m1_guideline_a")))
        );
        // Non-empty paths.
        let mut empty_path = sample_bundle();
        empty_path.nodes[2].path = String::new();
        assert_eq!(
            empty_path.validate(),
            Err(TraceError::EmptyPath(id("report")))
        );
        // Hash presence: a hash on the report node...
        let mut hashed_report = sample_bundle();
        hashed_report.nodes[2].content_hash = Some(hash('c'));
        assert_eq!(
            hashed_report.validate(),
            Err(TraceError::HashPresence(id("report")))
        );
        // ...and a hashless non-report node (stored re-sorted: the bare
        // node's bytes start at "kind", after the hashed node's).
        let mut bare = sample_bundle();
        let mut hashless = source_node();
        hashless.content_hash = None;
        bare.nodes = vec![graph_node(), report_node(), hashless];
        assert_eq!(
            bare.validate(),
            Err(TraceError::HashPresence(id("fixture.m1_guideline_a")))
        );
        // At most one report node.
        let mut second = sample_bundle();
        second.nodes.push(TraceNode {
            node_id: id("report2"),
            kind: TraceNodeKind::Report,
            path: "report2.json".to_owned(),
            content_hash: None,
        });
        assert_eq!(
            second.validate(),
            Err(TraceError::SecondReportNode(id("report2")))
        );
    }

    #[test]
    fn validation_rejects_edge_breaks() {
        // Unresolved endpoints.
        let mut from = sample_bundle();
        from.edges[0].from = id("zzz");
        assert_eq!(
            from.validate(),
            Err(TraceError::EdgeUnresolved {
                end: "from",
                id: id("zzz"),
            })
        );
        let mut to = sample_bundle();
        to.edges[0].to = id("zzz");
        assert_eq!(
            to.validate(),
            Err(TraceError::EdgeUnresolved {
                end: "to",
                id: id("zzz"),
            })
        );
        // Strict rank ascent: graph (1) -> source (0).
        let mut rank = sample_bundle();
        rank.edges[0] = TraceEdge {
            from: id("fixture.m1_guideline_a.source_graph"),
            operation: TraceOperation::Extract,
            to: id("fixture.m1_guideline_a"),
        };
        assert_eq!(
            rank.validate(),
            Err(TraceError::EdgeRank {
                from: id("fixture.m1_guideline_a.source_graph"),
                to: id("fixture.m1_guideline_a"),
            })
        );
        // Operation must be the target kind's producer.
        let mut op = sample_bundle();
        op.edges[0].operation = TraceOperation::Segment;
        assert_eq!(
            op.validate(),
            Err(TraceError::EdgeOperation {
                to: id("fixture.m1_guideline_a.source_graph"),
                operation: TraceOperation::Segment,
            })
        );
        // Duplicate edges break as set storage.
        let mut dup = sample_bundle();
        dup.edges.push(extract_edge());
        assert_eq!(
            dup.validate(),
            Err(TraceError::Duplicate {
                pool: "edges",
                index: 1,
            })
        );
    }

    #[test]
    fn validation_rejects_claim_form_breaks() {
        // Finding id form: leading-zero ordinal, foreign group, bad prefix.
        for bad in [
            "finding.group.m1_conflict.01",
            "finding.group.m1_null.0",
            "found.group.m1_conflict.0",
        ] {
            let mut bundle = sample_bundle();
            bundle.claims = vec![ClaimEvidenceRow {
                finding_id: id(bad),
                ..overlap_claim()
            }];
            assert_eq!(
                bundle.validate(),
                Err(TraceError::FindingIdForm(id(bad))),
                "{bad}"
            );
        }
        // Ordinals dense from 0 per group: ordinal 1 alone.
        let mut sparse = sample_bundle();
        sparse.claims = vec![deontic_claim()];
        assert_eq!(
            sparse.validate(),
            Err(TraceError::FindingOrdinals(id("group.m1_conflict")))
        );
        // Query outside its pair.
        let mut outside = sample_bundle();
        outside.claims = vec![ClaimEvidenceRow {
            query_id: id("q.m1_conflict.pair2.overlap"),
            ..overlap_claim()
        }];
        assert_eq!(
            outside.validate(),
            Err(TraceError::QueryOutsidePair {
                query_id: id("q.m1_conflict.pair2.overlap"),
                pair_id: id("q.m1_conflict.pair1"),
            })
        );
        // Claims stored out of canonical order.
        let mut unsorted = sample_bundle();
        unsorted.claims.swap(0, 1);
        assert_eq!(
            unsorted.validate(),
            Err(TraceError::Unsorted {
                pool: "claims",
                index: 1,
            })
        );
    }

    #[test]
    fn validation_rejects_claim_coherence_breaks() {
        // Category-verdict: contradiction with sat.
        let mut sat = sample_bundle();
        sat.claims = vec![ClaimEvidenceRow {
            finding_id: id("finding.group.m1_conflict.0"),
            verdict: Some(SolverVerdict::Sat),
            ..deontic_claim()
        }];
        assert_eq!(
            sat.validate(),
            Err(TraceError::CategoryVerdict {
                category: VerifierCategory::SemanticContradiction,
                rule: "semantic_contradiction requires verdict unsat",
            })
        );
        // Conflict kind missing on a contradiction row...
        let mut missing = sample_bundle();
        missing.claims = vec![ClaimEvidenceRow {
            finding_id: id("finding.group.m1_conflict.0"),
            conflict_kind: None,
            ..deontic_claim()
        }];
        assert_eq!(
            missing.validate(),
            Err(TraceError::ConflictKindPresence(id(
                "finding.group.m1_conflict.0"
            )))
        );
        // ...and present on a no-conflict row.
        let mut extra = sample_bundle();
        extra.claims = vec![ClaimEvidenceRow {
            conflict_kind: Some(ConflictKind::DeonticDirectionConflict),
            ..overlap_claim()
        }];
        assert_eq!(
            extra.validate(),
            Err(TraceError::ConflictKindPresence(id(
                "finding.group.m1_conflict.0"
            )))
        );
    }

    #[test]
    fn validation_rejects_claim_evidence_breaks() {
        // Empty evidence set.
        let mut empty = sample_bundle();
        empty.claims = vec![ClaimEvidenceRow {
            assertion_ids: vec![],
            ..overlap_claim()
        }];
        assert_eq!(
            empty.validate(),
            Err(TraceError::EmptyEvidence {
                owner: id("finding.group.m1_conflict.0"),
                pool: "assertion_ids",
            })
        );
        // Evidence set stored unsorted.
        let mut unsorted = sample_bundle();
        unsorted.claims = vec![ClaimEvidenceRow {
            rule_ids: vec![
                id("fixture.m1_guideline_b.rule.0"),
                id("fixture.m1_guideline_a.rule.0"),
            ],
            ..overlap_claim()
        }];
        assert_eq!(
            unsorted.validate(),
            Err(TraceError::Unsorted {
                pool: "rule_ids",
                index: 1,
            })
        );
        // report_ref must be the report node — a non-report node fails...
        let mut wrong = sample_bundle();
        wrong.claims = vec![ClaimEvidenceRow {
            report_ref: id("fixture.m1_guideline_a"),
            ..overlap_claim()
        }];
        assert_eq!(
            wrong.validate(),
            Err(TraceError::ReportRef {
                finding_id: id("finding.group.m1_conflict.0"),
                report_ref: id("fixture.m1_guideline_a"),
            })
        );
        // ...as does a claim with no report node in the DAG.
        let mut missing = sample_bundle();
        missing.nodes = vec![source_node(), graph_node()];
        missing.claims = vec![overlap_claim()];
        assert_eq!(
            missing.validate(),
            Err(TraceError::ReportRef {
                finding_id: id("finding.group.m1_conflict.0"),
                report_ref: id("report"),
            })
        );
    }

    #[test]
    fn validation_rejects_lineage_breaks() {
        // Set storage on rows.
        let mut dup = sample_index();
        dup.rows.push(sample_row());
        assert_eq!(
            dup.validate(),
            Err(TraceError::Duplicate {
                pool: "rows",
                index: 1,
            })
        );
        // Distinct rows for the same (finding, document).
        let mut pair_dup = sample_index();
        pair_dup.rows.push(LineageRow {
            segment_ids: vec![id("seg.4")],
            ..sample_row()
        });
        assert_eq!(
            pair_dup.validate(),
            Err(TraceError::DuplicateLineageRow {
                finding_id: id("finding.group.m1_conflict.1"),
                document_id: id("fixture.m1_guideline_a"),
            })
        );
        // Empty reference set.
        let mut empty = sample_index();
        empty.rows[0].statement_ids = vec![];
        assert_eq!(
            empty.validate(),
            Err(TraceError::EmptyEvidence {
                owner: id("finding.group.m1_conflict.1"),
                pool: "statement_ids",
            })
        );
        // Reference set stored unsorted.
        let mut unsorted = sample_index();
        unsorted.rows[0].region_ids = vec![id("r.3"), id("r.2")];
        assert_eq!(
            unsorted.validate(),
            Err(TraceError::Unsorted {
                pool: "region_ids",
                index: 1,
            })
        );
    }
}
