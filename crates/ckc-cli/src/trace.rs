//! SPEC §7.1 trace payloads — the trace processing_stage's durable shapes:
//! [`TraceBundle`] (`trace_bundle.json`), the run's derivation DAG plus
//! claim-evidence rows, and [`LineageIndex`] (`lineage_index.json`), its
//! per-(finding, document) query index.
//!
//! This module owns the types, their canonical bytes (every collection a
//! canonical set sorted by [`canonical_sort_key`]), structural validation,
//! and assembly: [`assemble_trace`] builds both payloads over the run's
//! landed processing_stage artifacts, handed off per document as [`DocTrace`] and per
//! test_source group as [`GroupTrace`] by the §8.3 trace processing_stage in
//! [`crate::run`]. The [`command`] submodule is the pair's consumer:
//! `ckc trace` (§8.5 item 7).

pub(crate) mod command;

use std::collections::BTreeMap;

use ckc_core::{
    ArtifactWrapper, CanonError, CanonRead, CanonReadError, Canonical, Hash, Id, IrBundle,
    ObjectEmitter, ObjectReader, Reader, canonical_sort_key, emit_set, emit_string, fieldless_enum,
    read_set, read_string,
};
use ckc_smt::{CompiledArtifact, SolverVerdict, VerifierCategory, VerifierResults};

use crate::shell::static_id;

fieldless_enum! {
    /// SPEC §7.1 derivation-DAG node kind, one per §8.3 durable artifact
    /// class on the source → report chain. Declaration order is
    /// [`rank`](TraceNodeKind::rank) order; edges ascend strictly.
    TraceNodeKind {
        /// A corpus test_source document (raw bytes, corpus-relative path).
        Source => "source",
        /// The extract processing_stage's SourceDocumentGraph artifact.
        SourceDocumentGraph => "source_document_graph",
        /// The segment processing_stage's ClinicalSegments artifact.
        Segments => "segments",
        /// The normalize processing_stage's statement + rule layers artifact.
        Normalization => "normalization",
        /// The assemble processing_stage's five-layer IRBundle artifact.
        IrBundle => "ir_bundle",
        /// The compile processing_stage's per-group CompiledArtifact.
        Compiled => "compiled",
        /// The verify processing_stage's per-group VerifierResults artifact.
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
            TraceNodeKind::SourceDocumentGraph => 1,
            TraceNodeKind::Segments => 2,
            TraceNodeKind::Normalization => 3,
            TraceNodeKind::IrBundle => 4,
            TraceNodeKind::Compiled => 5,
            TraceNodeKind::VerifierResults => 6,
            TraceNodeKind::Report => 7,
        }
    }

    /// The §8.3 processing_stage that produces nodes of this kind — the operation
    /// every incoming edge carries; sources have no producer.
    pub fn operation(self) -> Option<TraceOperation> {
        match self {
            TraceNodeKind::Source => None,
            TraceNodeKind::SourceDocumentGraph => Some(TraceOperation::Extract),
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
    /// SPEC §7.1 edge label: the §8.3 processing_stage kind that derived the target
    /// node, spelled exactly as the run's processing_stage events spell it.
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
/// `content_hash` is the wrapper content hash (sources: the raw-byte
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
/// the `operation` processing_stage.
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
    ///    `finding.<group_id>.<sequence_number>` (sequence_number a canonical decimal)
    ///    with each group's sequence_numbers dense from 0; `query_id` extends
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
        let mut sequence_numbers: BTreeMap<&str, Vec<u64>> = BTreeMap::new();
        for claim in &self.claims {
            let Some(sequence_number) =
                parse_finding_sequence_number(&claim.finding_id, &claim.group_id)
            else {
                return Err(TraceError::FindingIdForm(claim.finding_id.clone()));
            };
            sequence_numbers
                .entry(claim.group_id.as_str())
                .or_default()
                .push(sequence_number);
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
        for (group, mut ords) in sequence_numbers {
            ords.sort_unstable();
            if ords.iter().enumerate().any(|(i, o)| *o != i as u64) {
                return Err(TraceError::FindingSequenceNumbers(
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

/// One document's per-processing_stage landings for [`assemble_trace`] — the run
/// hand-off (the document pipeline fills it as processing_stages land): identity
/// and corpus source_linkage, then each §8.3 landing as an Option in chain
/// order, present exactly when its processing_stage landed the artifact.
#[derive(Debug, Clone)]
pub struct DocTrace {
    pub document_id: Id,
    /// Corpus-relative test_source path (§8.2), the source node's `path`.
    pub test_source_path: String,
    /// Raw source-byte hash, the source node's content hash.
    pub source_hash: Hash,
    /// Landed (artifact id, wrapper content hash) per document processing_stage.
    pub source_document_graph: Option<(Id, Hash)>,
    pub segments: Option<(Id, Hash)>,
    pub normalization: Option<(Id, Hash)>,
    /// The assemble landing rides whole: lineage reads its rule, statement,
    /// and segment layers.
    pub bundle: Option<ArtifactWrapper<IrBundle>>,
}

/// One test_source group's landings for [`assemble_trace`] — the run hand-off
/// (the group pipeline fills it as processing_stages land): the §8.4 member set,
/// then the two group landings riding whole — claims read the compiled
/// plan and assertion map beside the verifier results.
#[derive(Debug, Clone)]
pub struct GroupTrace {
    pub group_id: Id,
    /// Member document ids in §8.4 registry order.
    pub test_sources: Vec<Id>,
    pub compiled: Option<ArtifactWrapper<CompiledArtifact>>,
    pub verifier_results: Option<ArtifactWrapper<VerifierResults>>,
}

/// Assemble the run's [`TraceBundle`] and [`LineageIndex`] over the landed
/// processing_stage artifacts (§7.1), skipping absent pieces.
///
/// DAG: one static report node (id `report`, path `report.json`, hashless)
/// as the sink; per document the §8.3 chain source →extract→ source_document_graph
/// →segment→ segments →normalize→ normalization →assemble→ ir_bundle —
/// node ids the artifact ids, paths the §8.3 run-relative layout, hashes
/// the wrapper content hashes, each present landing edged from its
/// nearest present predecessor; per group every member ir_bundle →compile→
/// compiled →verify→ verifier_results →report→ the report node.
///
/// Claims: one row per verifier result, sequence_number = its index in the group's
/// plan-ordered results vector (§7.2 finding ids; a result resolving to no
/// plan pair contributes nothing, surfacing downstream as an sequence_number gap).
/// Evidence is the recorded unsat core, else the row's query `:named`
/// assertions — `ctx.<rule_id>`/`a.<rule_id>` from the pair's
/// `fc.<rule_id>` constraints; rule and region ids union the
/// assertion-map records bound to the pair's two constraint rules,
/// independent of the evidence set; `conflict_kind` rides on
/// `semantic_contradiction` rows as M1's single kind.
///
/// Lineage: one row per finding × contributing member document — the
/// claim's rules narrowed by the `<document_id>.rule.` prefix, statements
/// via the §5 `rules[k] ← statements[k]` index invariant on the member
/// bundle, segments from those statements, regions from the rule.
///
/// Every collection lands sorted by [`canonical_sort_key`]; the caller
/// runs [`TraceBundle::validate`] / [`LineageIndex::validate`] on the
/// pair.
pub fn assemble_trace(docs: &[DocTrace], groups: &[GroupTrace]) -> (TraceBundle, LineageIndex) {
    let report = static_id("report");
    let mut nodes = vec![TraceNode {
        node_id: report.clone(),
        kind: TraceNodeKind::Report,
        path: "report.json".to_owned(),
        content_hash: None,
    }];
    let mut edges = Vec::new();

    let mut bundle_nodes: BTreeMap<&str, &Id> = BTreeMap::new();
    for doc in docs {
        nodes.push(TraceNode {
            node_id: doc.document_id.clone(),
            kind: TraceNodeKind::Source,
            path: doc.test_source_path.clone(),
            content_hash: Some(doc.source_hash.clone()),
        });
        let dir = format!("artifacts/{}", doc.document_id);
        let mut prev = &doc.document_id;
        for (landing, kind, file) in [
            (
                &doc.source_document_graph,
                TraceNodeKind::SourceDocumentGraph,
                "source_document_graph",
            ),
            (&doc.segments, TraceNodeKind::Segments, "segments"),
            (
                &doc.normalization,
                TraceNodeKind::Normalization,
                "normalization",
            ),
        ] {
            let Some((artifact_id, hash)) = landing else {
                continue;
            };
            nodes.push(TraceNode {
                node_id: artifact_id.clone(),
                kind,
                path: format!("{dir}/{file}.json"),
                content_hash: Some(hash.clone()),
            });
            edges.push(TraceEdge {
                from: prev.clone(),
                operation: kind.operation().expect("landing kinds carry producers"),
                to: artifact_id.clone(),
            });
            prev = artifact_id;
        }
        if let Some(bundle) = &doc.bundle {
            nodes.push(TraceNode {
                node_id: bundle.artifact_id.clone(),
                kind: TraceNodeKind::IrBundle,
                path: format!("{dir}/ir_bundle.json"),
                content_hash: Some(bundle.content_hash.clone()),
            });
            edges.push(TraceEdge {
                from: prev.clone(),
                operation: TraceOperation::Assemble,
                to: bundle.artifact_id.clone(),
            });
            bundle_nodes.insert(doc.document_id.as_str(), &bundle.artifact_id);
        }
    }

    let mut claims = Vec::new();
    let mut rows = Vec::new();
    for group in groups {
        let dir = format!("groups/{}", group.group_id);
        if let Some(compiled) = &group.compiled {
            nodes.push(TraceNode {
                node_id: compiled.artifact_id.clone(),
                kind: TraceNodeKind::Compiled,
                path: format!("{dir}/compiled.json"),
                content_hash: Some(compiled.content_hash.clone()),
            });
            for test_source in &group.test_sources {
                if let Some(bundle_id) = bundle_nodes.get(test_source.as_str()) {
                    edges.push(TraceEdge {
                        from: (*bundle_id).clone(),
                        operation: TraceOperation::Compile,
                        to: compiled.artifact_id.clone(),
                    });
                }
            }
        }
        if let Some(results) = &group.verifier_results {
            nodes.push(TraceNode {
                node_id: results.artifact_id.clone(),
                kind: TraceNodeKind::VerifierResults,
                path: format!("{dir}/verifier_results.json"),
                content_hash: Some(results.content_hash.clone()),
            });
            if let Some(compiled) = &group.compiled {
                edges.push(TraceEdge {
                    from: compiled.artifact_id.clone(),
                    operation: TraceOperation::Verify,
                    to: results.artifact_id.clone(),
                });
            }
            edges.push(TraceEdge {
                from: results.artifact_id.clone(),
                operation: TraceOperation::Report,
                to: report.clone(),
            });
        }

        let (Some(compiled), Some(results)) = (&group.compiled, &group.verifier_results) else {
            continue;
        };
        for (sequence_number, result) in results.payload.results.iter().enumerate() {
            let Some(pair) = compiled.payload.solver_query_plan.iter().find(|p| {
                p.context_overlap_query_id == result.query_id
                    || p.deontic_consistency_query_id == result.query_id
            }) else {
                continue;
            };
            let pair_rules: Vec<&str> = [&pair.constraint_a_id, &pair.constraint_b_id]
                .into_iter()
                .filter_map(|c| c.as_str().strip_prefix("fc."))
                .collect();
            let assertion_ids = match &result.unsat_core {
                Some(core) => core.clone(),
                None => {
                    let prefix = if pair.context_overlap_query_id == result.query_id {
                        "ctx."
                    } else {
                        "a."
                    };
                    canonical_id_set(
                        pair_rules
                            .iter()
                            .map(|rule| {
                                Id::new(format!("{prefix}{rule}")).expect(
                                    "a valid constraint id keeps the Id grammar re-prefixed",
                                )
                            })
                            .collect(),
                    )
                }
            };
            let mut rule_ids = Vec::new();
            let mut region_ids = Vec::new();
            for (assertion_id, record) in &compiled.payload.assertion_to_source_map {
                let bound = assertion_id
                    .as_str()
                    .strip_prefix("ctx.")
                    .or_else(|| assertion_id.as_str().strip_prefix("a."))
                    .is_some_and(|rule| pair_rules.contains(&rule));
                if bound {
                    rule_ids.extend(record.rule_ids.iter().cloned());
                    region_ids.extend(record.region_ids.iter().cloned());
                }
            }
            let rule_ids = canonical_id_set(rule_ids);
            let finding_id = Id::new(format!("finding.{}.{sequence_number}", group.group_id))
                .expect("a valid group id keeps the Id grammar under the finding prefix");

            for test_source in &group.test_sources {
                let Some(bundle) = docs
                    .iter()
                    .find(|d| d.document_id == *test_source)
                    .and_then(|d| d.bundle.as_ref())
                else {
                    continue;
                };
                let rule_prefix = format!("{test_source}.rule.");
                let doc_rules: Vec<Id> = rule_ids
                    .iter()
                    .filter(|r| r.as_str().starts_with(&rule_prefix))
                    .cloned()
                    .collect();
                if doc_rules.is_empty() {
                    continue;
                }
                let mut doc_regions = Vec::new();
                let mut statement_ids = Vec::new();
                let mut segment_ids = Vec::new();
                for rule_id in &doc_rules {
                    let Some(index) = bundle
                        .payload
                        .norm
                        .rules
                        .iter()
                        .position(|rule| rule.rule_id == *rule_id)
                    else {
                        continue;
                    };
                    doc_regions.extend(bundle.payload.norm.rules[index].source_region_ids.clone());
                    if let Some(statement) = bundle.payload.clinical.statements.get(index) {
                        statement_ids.push(statement.statement_id.clone());
                        segment_ids.extend(statement.source_segment_ids.iter().cloned());
                    }
                }
                rows.push(LineageRow {
                    finding_id: finding_id.clone(),
                    document_id: test_source.clone(),
                    region_ids: canonical_id_set(doc_regions),
                    rule_ids: doc_rules,
                    segment_ids: canonical_id_set(segment_ids),
                    statement_ids: canonical_id_set(statement_ids),
                });
            }

            claims.push(ClaimEvidenceRow {
                finding_id,
                group_id: group.group_id.clone(),
                pair_id: pair.pair_id.clone(),
                query_id: result.query_id.clone(),
                category: result.category,
                verdict: result.verdict,
                conflict_kind: (result.category == VerifierCategory::SemanticContradiction)
                    .then_some(ConflictKind::DeonticDirectionConflict),
                assertion_ids,
                rule_ids,
                region_ids: canonical_id_set(region_ids),
                report_ref: report.clone(),
            });
        }
    }

    sort_canonical(&mut nodes);
    sort_canonical(&mut edges);
    sort_canonical(&mut claims);
    sort_canonical(&mut rows);
    (
        TraceBundle {
            nodes,
            edges,
            claims,
        },
        LineageIndex { rows },
    )
}

/// Sort a collection into canonical-set storage order
/// ([`canonical_sort_key`]); trace shapes emit infallibly.
fn sort_canonical<T: Canonical>(items: &mut [T]) {
    items.sort_by_cached_key(|item| {
        canonical_sort_key(item).expect("trace shapes emit canonically")
    });
}

/// Canonical Id set from an unordered pool: byte-sorted, duplicate-free
/// (Id-byte order equals [`canonical_sort_key`] order for identifier
/// strings).
pub(crate) fn canonical_id_set(mut ids: Vec<Id>) -> Vec<Id> {
    ids.sort_unstable_by(|a, b| a.as_str().cmp(b.as_str()));
    ids.dedup();
    ids
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

/// Parse the §7.2 finding-id form `finding.<group_id>.<sequence_number>` against
/// the row's group, yielding the sequence_number; the sequence_number is a canonical
/// decimal — ASCII digits with no leading zero unless exactly `0`.
fn parse_finding_sequence_number(finding_id: &Id, group_id: &Id) -> Option<u64> {
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
    /// The named finding id lacks the §7.2 `finding.<group_id>.<sequence_number>`
    /// form over its row's group.
    FindingIdForm(Id),
    /// The named group's finding sequence_numbers are not dense from 0.
    FindingSequenceNumbers(Id),
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
                "finding id {id} is not finding.<group_id>.<sequence_number> over its group"
            ),
            TraceError::FindingSequenceNumbers(group) => {
                write!(
                    f,
                    "group {group} finding sequence_numbers are not dense from 0"
                )
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
    use ckc_core::{
        ContradictionQueryPair, EvidenceStatus, Origin, Producer, SolverIdentity,
        canonical_payload_bytes, read_strict_canonical,
    };
    use ckc_smt::{AssertionRecord, VerifierResult};

    /// Canonical bytes of `value` as a UTF-8 string, for exact-match assertions.
    fn canon<T: Canonical>(value: &T) -> String {
        String::from_utf8(canonical_payload_bytes(value).unwrap()).unwrap()
    }

    /// Assert `value` survives a canonical write -> read round trip unchanged.
    fn round_trip<T: Canonical + CanonRead + std::fmt::Debug + PartialEq>(value: T) {
        let bytes = canonical_payload_bytes(&value).unwrap();
        let got: T = read_strict_canonical(&bytes).unwrap();
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
            node_id: id("test_source.m1_guideline_a"),
            kind: TraceNodeKind::Source,
            path: "corpus/test_sources/m1_guideline_a.html".to_owned(),
            content_hash: Some(hash('a')),
        }
    }

    /// docA's extract landing: §8.3 run-relative path, wrapper hash.
    fn graph_node() -> TraceNode {
        TraceNode {
            node_id: id("test_source.m1_guideline_a.source_document_graph"),
            kind: TraceNodeKind::SourceDocumentGraph,
            path: "artifacts/test_source.m1_guideline_a/source_document_graph.json".to_owned(),
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
            from: id("test_source.m1_guideline_a"),
            operation: TraceOperation::Extract,
            to: id("test_source.m1_guideline_a.source_document_graph"),
        }
    }

    /// §8.6 overlap row: finding sequence_number 0, sat, no conflict kind, ctx
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
                id("ctx.test_source.m1_guideline_a.rule.0"),
                id("ctx.test_source.m1_guideline_b.rule.0"),
            ],
            rule_ids: vec![
                id("test_source.m1_guideline_a.rule.0"),
                id("test_source.m1_guideline_b.rule.0"),
            ],
            region_ids: vec![id("r.2"), id("r.3")],
            report_ref: id("report"),
        }
    }

    /// §8.6 deontic row: finding sequence_number 1, unsat, the M1 conflict kind,
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
                id("a.test_source.m1_guideline_a.rule.0"),
                id("a.test_source.m1_guideline_b.rule.0"),
            ],
            rule_ids: vec![
                id("test_source.m1_guideline_a.rule.0"),
                id("test_source.m1_guideline_b.rule.0"),
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
            document_id: id("test_source.m1_guideline_a"),
            region_ids: vec![id("r.2"), id("r.3")],
            rule_ids: vec![id("test_source.m1_guideline_a.rule.0")],
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
        assert_eq!(
            canon(&TraceNodeKind::SourceDocumentGraph),
            "\"source_document_graph\""
        );
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
                r#"{"assertion_ids":["a.test_source.m1_guideline_a.rule.0","#,
                r#""a.test_source.m1_guideline_b.rule.0"],"#,
                r#""category":"semantic_contradiction","#,
                r#""conflict_kind":"deontic_direction_conflict","#,
                r#""finding_id":"finding.group.m1_conflict.1","#,
                r#""group_id":"group.m1_conflict","#,
                r#""pair_id":"q.m1_conflict.pair1","#,
                r#""query_id":"q.m1_conflict.pair1.deontic","#,
                r#""region_ids":["r.2","r.3"],"report_ref":"report","#,
                r#""rule_ids":["test_source.m1_guideline_a.rule.0","#,
                r#""test_source.m1_guideline_b.rule.0"],"verdict":"unsat"},"#,
                r#"{"assertion_ids":["ctx.test_source.m1_guideline_a.rule.0","#,
                r#""ctx.test_source.m1_guideline_b.rule.0"],"#,
                r#""category":"semantic_no_conflict","#,
                r#""finding_id":"finding.group.m1_conflict.0","#,
                r#""group_id":"group.m1_conflict","#,
                r#""pair_id":"q.m1_conflict.pair1","#,
                r#""query_id":"q.m1_conflict.pair1.overlap","#,
                r#""region_ids":["r.2","r.3"],"report_ref":"report","#,
                r#""rule_ids":["test_source.m1_guideline_a.rule.0","#,
                r#""test_source.m1_guideline_b.rule.0"],"verdict":"sat"}],"#,
                r#""edges":[{"from":"test_source.m1_guideline_a","operation":"extract","#,
                r#""to":"test_source.m1_guideline_a.source_document_graph"}],"#,
                r#""nodes":[{"content_hash":"#,
                r#""sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","#,
                r#""kind":"source","node_id":"test_source.m1_guideline_a","#,
                r#""path":"corpus/test_sources/m1_guideline_a.html"},"#,
                r#"{"content_hash":"#,
                r#""sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","#,
                r#""kind":"source_document_graph","#,
                r#""node_id":"test_source.m1_guideline_a.source_document_graph","#,
                r#""path":"artifacts/test_source.m1_guideline_a/source_document_graph.json"},"#,
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
                r#"{"rows":[{"document_id":"test_source.m1_guideline_a","#,
                r#""finding_id":"finding.group.m1_conflict.1","#,
                r#""region_ids":["r.2","r.3"],"#,
                r#""rule_ids":["test_source.m1_guideline_a.rule.0"],"#,
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
        dup_id.nodes[1].node_id = id("test_source.m1_guideline_a");
        assert_eq!(
            dup_id.validate(),
            Err(TraceError::DuplicateNodeId(id(
                "test_source.m1_guideline_a"
            )))
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
            Err(TraceError::HashPresence(id("test_source.m1_guideline_a")))
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
            from: id("test_source.m1_guideline_a.source_document_graph"),
            operation: TraceOperation::Extract,
            to: id("test_source.m1_guideline_a"),
        };
        assert_eq!(
            rank.validate(),
            Err(TraceError::EdgeRank {
                from: id("test_source.m1_guideline_a.source_document_graph"),
                to: id("test_source.m1_guideline_a"),
            })
        );
        // Operation must be the target kind's producer.
        let mut op = sample_bundle();
        op.edges[0].operation = TraceOperation::Segment;
        assert_eq!(
            op.validate(),
            Err(TraceError::EdgeOperation {
                to: id("test_source.m1_guideline_a.source_document_graph"),
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
        // Finding id form: leading-zero sequence_number, foreign group, bad prefix.
        for bad in [
            "finding.group.m1_conflict.01",
            "finding.group.m1_no_conflict.0",
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
        // SequenceNumbers dense from 0 per group: sequence_number 1 alone.
        let mut sparse = sample_bundle();
        sparse.claims = vec![deontic_claim()];
        assert_eq!(
            sparse.validate(),
            Err(TraceError::FindingSequenceNumbers(id("group.m1_conflict")))
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
                id("test_source.m1_guideline_b.rule.0"),
                id("test_source.m1_guideline_a.rule.0"),
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
            report_ref: id("test_source.m1_guideline_a"),
            ..overlap_claim()
        }];
        assert_eq!(
            wrong.validate(),
            Err(TraceError::ReportRef {
                finding_id: id("finding.group.m1_conflict.0"),
                report_ref: id("test_source.m1_guideline_a"),
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
                document_id: id("test_source.m1_guideline_a"),
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

    /// Wrapper wrap for hand-built group payloads: assembly reads
    /// `artifact_id`, `content_hash`, and `payload` only, so the metadata
    /// fields carry inert synthetic values and `fill` pins the hash the
    /// node must copy.
    fn wrapper<P>(artifact_id: &str, kind: &str, fill: char, payload: P) -> ArtifactWrapper<P> {
        ArtifactWrapper {
            schema_id: id(&format!("schema.{kind}")),
            artifact_id: id(artifact_id),
            artifact_kind: id(kind),
            producer: Producer {
                pipeline_id: id("cand.test"),
                pipeline_step_id: id("comp.test"),
                toolchain_manifest_hash: hash('f'),
            },
            input_hashes: vec![],
            content_hash: hash(fill),
            canonicalization_policy_hash: hash('f'),
            origin: Origin::DeterministicCompiler,
            evidence_status: EvidenceStatus::CompilerEvidenceStatus,
            external_effects: vec![],
            trace_refs: vec![],
            diagnostics: vec![],
            runtime_metadata: vec![],
            payload,
        }
    }

    /// DocTrace with every landing absent — contributes a bare source node.
    fn bare_doc(doc: &str) -> DocTrace {
        DocTrace {
            document_id: id(doc),
            test_source_path: format!("corpus/test_sources/{doc}.html"),
            source_hash: hash('a'),
            source_document_graph: None,
            segments: None,
            normalization: None,
            bundle: None,
        }
    }

    /// doc.full with the three pre-bundle landings present.
    fn chain_doc() -> DocTrace {
        DocTrace {
            source_document_graph: Some((id("doc.full.source_document_graph"), hash('b'))),
            segments: Some((id("doc.full.segments"), hash('c'))),
            normalization: Some((id("doc.full.normalization"), hash('d'))),
            ..bare_doc("doc.full")
        }
    }

    fn record(rule: &str, regions: &[&str]) -> AssertionRecord {
        AssertionRecord {
            rule_ids: vec![id(rule)],
            region_ids: regions.iter().map(|r| id(r)).collect(),
        }
    }

    /// The synthetic plan: pair doc.a.rule.0 × doc.b.rule.0 with all four
    /// named-assertion records — ctx.doc.a carries an extra region (r.2)
    /// so claim unions distinguish "the pair's records" from "the
    /// evidence's records". Bodies stay empty: assembly reads the plan and
    /// assertion map only.
    fn compiled_payload() -> CompiledArtifact {
        CompiledArtifact {
            target_id: id("target.smtlib2"),
            solver_query_plan: vec![ContradictionQueryPair {
                pair_id: id("q.g1.pair1"),
                action_key: id("act.administer:drug.x"),
                constraint_a_id: id("fc.doc.a.rule.0"),
                constraint_b_id: id("fc.doc.b.rule.0"),
                context_overlap_query_id: id("q.g1.pair1.overlap"),
                deontic_consistency_query_id: id("q.g1.pair1.deontic"),
            }],
            query_bodies: vec![],
            assertion_to_source_map: vec![
                (id("a.doc.a.rule.0"), record("doc.a.rule.0", &["r.1"])),
                (id("a.doc.b.rule.0"), record("doc.b.rule.0", &["r.9"])),
                (
                    id("ctx.doc.a.rule.0"),
                    record("doc.a.rule.0", &["r.1", "r.2"]),
                ),
                (id("ctx.doc.b.rule.0"), record("doc.b.rule.0", &["r.9"])),
            ],
            target_metadata: vec![],
            diagnostics: vec![],
        }
    }

    fn verifier_result(
        query: &str,
        category: VerifierCategory,
        verdict: SolverVerdict,
        core: Option<&[&str]>,
    ) -> VerifierResult {
        VerifierResult {
            query_id: id(query),
            category,
            verdict: Some(verdict),
            model: None,
            unsat_core: core.map(|ids| ids.iter().map(|i| id(i)).collect()),
            solver_identity: SolverIdentity {
                solver_id: id("z3"),
                version: "4.13.4".to_owned(),
            },
            diagnostics: vec![],
        }
    }

    /// group.g1 with both landings present over the synthetic plan.
    fn claims_group(results: Vec<VerifierResult>) -> GroupTrace {
        GroupTrace {
            group_id: id("group.g1"),
            test_sources: vec![id("doc.a"), id("doc.b")],
            compiled: Some(wrapper(
                "group.g1.compiled",
                "compiled",
                'b',
                compiled_payload(),
            )),
            verifier_results: Some(wrapper(
                "group.g1.verifier_results",
                "verifier_results",
                'e',
                VerifierResults { results },
            )),
        }
    }

    fn node<'a>(bundle: &'a TraceBundle, node_id: &str) -> &'a TraceNode {
        bundle
            .nodes
            .iter()
            .find(|n| n.node_id.as_str() == node_id)
            .expect("node present")
    }

    fn claim<'a>(bundle: &'a TraceBundle, finding_id: &str) -> &'a ClaimEvidenceRow {
        bundle
            .claims
            .iter()
            .find(|c| c.finding_id.as_str() == finding_id)
            .expect("claim present")
    }

    fn has_edge(bundle: &TraceBundle, from: &str, operation: TraceOperation, to: &str) -> bool {
        bundle
            .edges
            .iter()
            .any(|e| e.from.as_str() == from && e.operation == operation && e.to.as_str() == to)
    }

    // Empty inputs assemble to the lone report node.
    #[test]
    fn assemble_empty_inputs() {
        let (bundle, index) = assemble_trace(&[], &[]);
        assert_eq!(bundle.validate(), Ok(()));
        assert_eq!(index.validate(), Ok(()));
        assert_eq!(bundle.nodes, vec![report_node()]);
        assert!(bundle.edges.is_empty());
        assert!(bundle.claims.is_empty());
        assert!(index.rows.is_empty());
    }

    // A bundle-less full chain: source + three landings at §8.3 paths with
    // the landing hashes, chain edges in processing_stage order.
    #[test]
    fn assemble_full_chain_doc() {
        let (bundle, index) = assemble_trace(&[chain_doc()], &[]);
        assert_eq!(bundle.validate(), Ok(()));
        assert_eq!(index.validate(), Ok(()));
        assert_eq!(bundle.nodes.len(), 5);
        let source = node(&bundle, "doc.full");
        assert_eq!(source.kind, TraceNodeKind::Source);
        assert_eq!(source.path, "corpus/test_sources/doc.full.html");
        assert_eq!(source.content_hash, Some(hash('a')));
        let segments = node(&bundle, "doc.full.segments");
        assert_eq!(segments.kind, TraceNodeKind::Segments);
        assert_eq!(segments.path, "artifacts/doc.full/segments.json");
        assert_eq!(segments.content_hash, Some(hash('c')));
        assert_eq!(bundle.edges.len(), 3);
        assert!(has_edge(
            &bundle,
            "doc.full",
            TraceOperation::Extract,
            "doc.full.source_document_graph"
        ));
        assert!(has_edge(
            &bundle,
            "doc.full.source_document_graph",
            TraceOperation::Segment,
            "doc.full.segments"
        ));
        assert!(has_edge(
            &bundle,
            "doc.full.segments",
            TraceOperation::Normalize,
            "doc.full.normalization"
        ));
        assert!(bundle.claims.is_empty());
    }

    // Segments absent: one normalize edge bridges source_document_graph →
    // normalization (nearest present predecessor).
    #[test]
    fn assemble_gapped_doc() {
        let mut doc = chain_doc();
        doc.segments = None;
        let (bundle, _) = assemble_trace(&[doc], &[]);
        assert_eq!(bundle.validate(), Ok(()));
        assert_eq!(bundle.nodes.len(), 4);
        assert_eq!(bundle.edges.len(), 2);
        assert!(has_edge(
            &bundle,
            "doc.full.source_document_graph",
            TraceOperation::Normalize,
            "doc.full.normalization"
        ));
    }

    // A group with neither landing contributes nothing.
    #[test]
    fn assemble_bare_group() {
        let group = GroupTrace {
            group_id: id("group.g1"),
            test_sources: vec![id("doc.a")],
            compiled: None,
            verifier_results: None,
        };
        let (bundle, index) = assemble_trace(&[], &[group]);
        assert_eq!(bundle.validate(), Ok(()));
        assert_eq!(bundle.nodes, vec![report_node()]);
        assert!(bundle.edges.is_empty());
        assert!(bundle.claims.is_empty());
        assert!(index.rows.is_empty());
    }

    // Verifier results without the compiled half: the node and its report
    // edge land, no verify edge, no claims.
    #[test]
    fn assemble_results_only_group() {
        let group = GroupTrace {
            group_id: id("group.g1"),
            test_sources: vec![],
            compiled: None,
            verifier_results: Some(wrapper(
                "group.g1.verifier_results",
                "verifier_results",
                'e',
                VerifierResults { results: vec![] },
            )),
        };
        let (bundle, index) = assemble_trace(&[], &[group]);
        assert_eq!(bundle.validate(), Ok(()));
        let results = node(&bundle, "group.g1.verifier_results");
        assert_eq!(results.kind, TraceNodeKind::VerifierResults);
        assert_eq!(results.path, "groups/group.g1/verifier_results.json");
        assert_eq!(results.content_hash, Some(hash('e')));
        assert_eq!(bundle.edges.len(), 1);
        assert!(has_edge(
            &bundle,
            "group.g1.verifier_results",
            TraceOperation::Report,
            "report"
        ));
        assert!(bundle.claims.is_empty());
        assert!(index.rows.is_empty());
    }

    // The hand-built claims battery: sequence_numbers from the results vector,
    // core-verbatim vs ctx.-fallback evidence, assertion-map unions over
    // the pair's two constraints on both rows, conflict_kind exactly on
    // semantic_contradiction — and both lineage skip paths (doc.a rides
    // bundle-less, doc.b has no DocTrace at all → zero rows).
    #[test]
    fn assemble_claims_and_lineage_skips() {
        let results = vec![
            verifier_result(
                "q.g1.pair1.overlap",
                VerifierCategory::SemanticNoConflict,
                SolverVerdict::Sat,
                None,
            ),
            verifier_result(
                "q.g1.pair1.deontic",
                VerifierCategory::SemanticContradiction,
                SolverVerdict::Unsat,
                Some(&["a.doc.a.rule.0", "a.doc.b.rule.0"]),
            ),
            // Resolving to no plan pair: contributes nothing.
            verifier_result(
                "q.g1.unplanned",
                VerifierCategory::SemanticNoConflict,
                SolverVerdict::Sat,
                None,
            ),
        ];
        let (bundle, index) = assemble_trace(&[bare_doc("doc.a")], &[claims_group(results)]);
        assert_eq!(bundle.validate(), Ok(()));
        assert_eq!(index.validate(), Ok(()));
        assert!(index.rows.is_empty());
        assert_eq!(bundle.nodes.len(), 4);
        assert_eq!(
            node(&bundle, "group.g1.compiled").path,
            "groups/group.g1/compiled.json"
        );
        // No compile edges without member ir_bundle nodes.
        assert_eq!(bundle.edges.len(), 2);
        assert!(has_edge(
            &bundle,
            "group.g1.compiled",
            TraceOperation::Verify,
            "group.g1.verifier_results"
        ));
        assert!(has_edge(
            &bundle,
            "group.g1.verifier_results",
            TraceOperation::Report,
            "report"
        ));
        assert_eq!(bundle.claims.len(), 2);
        let overlap = claim(&bundle, "finding.group.g1.0");
        assert_eq!(overlap.group_id, id("group.g1"));
        assert_eq!(overlap.pair_id, id("q.g1.pair1"));
        assert_eq!(overlap.query_id, id("q.g1.pair1.overlap"));
        assert_eq!(overlap.category, VerifierCategory::SemanticNoConflict);
        assert_eq!(overlap.verdict, Some(SolverVerdict::Sat));
        assert_eq!(overlap.conflict_kind, None);
        // Coreless overlap evidence: both constraints fc.-stripped and
        // ctx.-prefixed.
        assert_eq!(
            overlap.assertion_ids,
            vec![id("ctx.doc.a.rule.0"), id("ctx.doc.b.rule.0")]
        );
        let deontic = claim(&bundle, "finding.group.g1.1");
        assert_eq!(deontic.query_id, id("q.g1.pair1.deontic"));
        assert_eq!(
            deontic.conflict_kind,
            Some(ConflictKind::DeonticDirectionConflict)
        );
        // Recorded core rides verbatim.
        assert_eq!(
            deontic.assertion_ids,
            vec![id("a.doc.a.rule.0"), id("a.doc.b.rule.0")]
        );
        // Rule/region unions are evidence-independent: ctx.doc.a's extra
        // r.2 reaches the deontic row too.
        for row in [overlap, deontic] {
            assert_eq!(row.rule_ids, vec![id("doc.a.rule.0"), id("doc.b.rule.0")]);
            assert_eq!(row.region_ids, vec![id("r.1"), id("r.2"), id("r.9")]);
            assert_eq!(row.report_ref, id("report"));
        }
    }

    // A skipped result keeps its index: the lone claim lands at sequence_number 1
    // with the deontic a.-fallback evidence, and the gap surfaces
    // downstream as non-dense sequence_numbers.
    #[test]
    fn assemble_claim_sequence_number_gap_and_deontic_fallback() {
        let results = vec![
            verifier_result(
                "q.g1.unplanned",
                VerifierCategory::SemanticNoConflict,
                SolverVerdict::Sat,
                None,
            ),
            verifier_result(
                "q.g1.pair1.deontic",
                VerifierCategory::SemanticNoConflict,
                SolverVerdict::Sat,
                None,
            ),
        ];
        let (bundle, _) = assemble_trace(&[], &[claims_group(results)]);
        assert_eq!(bundle.claims.len(), 1);
        let row = &bundle.claims[0];
        assert_eq!(row.finding_id, id("finding.group.g1.1"));
        assert_eq!(
            row.assertion_ids,
            vec![id("a.doc.a.rule.0"), id("a.doc.b.rule.0")]
        );
        assert_eq!(
            bundle.validate(),
            Err(TraceError::FindingSequenceNumbers(id("group.g1")))
        );
    }

    // --- live test_source pins (cli-runner.3a.2b) ---

    use std::time::Duration;

    use crate::extract::{ExtractConfig, extract};
    use crate::normalize::{Lexicon, load_lexicon, normalize};
    use crate::segment::segment;
    use ckc_core::{
        DataClass, DiagnosticRecord, DocIr, Provenance, assemble, canonicalization_policy_hash,
        content_hash, hash_bytes,
    };
    use ckc_smt::{Z3Adapter, compile, verify};

    /// §8.2 committed test_source bytes, repository-rooted.
    fn test_source(name: &str) -> Vec<u8> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/test_sources/");
        std::fs::read(format!("{dir}{name}")).unwrap()
    }

    /// The committed §5 lexicon evidence_status, loaded.
    fn committed_lexicon() -> Lexicon {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/lexicon/ja_core.yaml"
        );
        load_lexicon(&std::fs::read(path).unwrap()).unwrap()
    }

    /// Inert processing_stage producer (real producer values are run.rs's concern).
    fn live_producer() -> Producer {
        Producer {
            pipeline_id: id("cand.m1"),
            pipeline_step_id: id("processing_stage.trace"),
            toolchain_manifest_hash: hash('f'),
        }
    }

    /// The generic live wrap: real content/policy hashes over the payload,
    /// every effect/trace/diagnostic/metadata slot empty — assembly reads
    /// `artifact_id`, `content_hash`, and `payload` only.
    fn live_wrapper<P: Canonical>(artifact_id: &str, kind: &str, payload: P) -> ArtifactWrapper<P> {
        ArtifactWrapper {
            schema_id: id(&format!("schema.{kind}")),
            artifact_id: id(artifact_id),
            artifact_kind: id(kind),
            producer: live_producer(),
            input_hashes: vec![],
            content_hash: content_hash(&payload).unwrap(),
            canonicalization_policy_hash: canonicalization_policy_hash(),
            origin: Origin::DeterministicCompiler,
            evidence_status: EvidenceStatus::CompilerEvidenceStatus,
            external_effects: vec![],
            trace_refs: vec![],
            diagnostics: vec![],
            runtime_metadata: vec![],
            payload,
        }
    }

    /// run.rs's document pipeline mirrored through the pub processing_stage surface:
    /// extract → segment → normalize → DocIr + canonical diagnostic union
    /// → [`ckc_core::assemble`] → graph validation, every landing Some.
    fn live_doc(stem: &str, lexicon: &Lexicon) -> DocTrace {
        let document_id = id(&format!("test_source.{stem}"));
        let raw = test_source(&format!("{stem}.html"));
        let config = ExtractConfig {
            document_id: document_id.clone(),
            source_family: id("synthetic_test_source_html"),
            provenance: Provenance::Synthetic,
            data_class: DataClass::None,
            producer: live_producer(),
        };
        let source = extract(&raw, &config).unwrap();
        let segments = segment(&source, &live_producer()).unwrap();
        let normalization = normalize(&source, &segments, lexicon, &live_producer()).unwrap();

        let doc = DocIr::from_graph(&source.payload, source.diagnostics.clone()).unwrap();
        let mut diagnostics: Vec<DiagnosticRecord> = segments
            .diagnostics
            .iter()
            .chain(&normalization.diagnostics)
            .cloned()
            .collect();
        sort_canonical(&mut diagnostics);
        diagnostics
            .dedup_by(|a, b| canonical_sort_key(a).unwrap() == canonical_sort_key(b).unwrap());
        let bundle = assemble(
            doc,
            segments.payload.clone(),
            normalization.payload.clinical.clone(),
            normalization.payload.norm.clone(),
            Vec::new(),
            diagnostics,
        )
        .unwrap();
        bundle.validate(&source.payload).unwrap();

        DocTrace {
            test_source_path: format!("corpus/test_sources/{stem}.html"),
            source_hash: hash_bytes(&raw),
            source_document_graph: Some((source.artifact_id.clone(), source.content_hash.clone())),
            segments: Some((segments.artifact_id.clone(), segments.content_hash.clone())),
            normalization: Some((
                normalization.artifact_id.clone(),
                normalization.content_hash.clone(),
            )),
            bundle: Some(live_wrapper(
                &format!("{document_id}.ir_bundle"),
                "ir_bundle",
                bundle,
            )),
            document_id,
        }
    }

    /// run.rs's group pipeline mirrored: compile the members' (formal,
    /// norm) pairs, verify on live z3 under a generous budget.
    fn live_group(gid: &str, members: &[&DocTrace], adapter: &Z3Adapter) -> GroupTrace {
        let artifact = compile(
            &id(gid),
            members.iter().map(|d| {
                let bundle = &d.bundle.as_ref().unwrap().payload;
                (&bundle.formal, &bundle.norm)
            }),
        );
        let results = verify(adapter, &artifact, Duration::from_secs(30));
        GroupTrace {
            group_id: id(gid),
            test_sources: members.iter().map(|d| d.document_id.clone()).collect(),
            compiled: Some(live_wrapper(
                &format!("{gid}.compiled"),
                "compiled",
                artifact,
            )),
            verifier_results: Some(live_wrapper(
                &format!("{gid}.verifier_results"),
                "verifier_results",
                VerifierResults { results },
            )),
        }
    }

    fn row(
        finding: &str,
        document: &str,
        regions: &[&str],
        rules: &[&str],
        segments: &[&str],
        statements: &[&str],
    ) -> LineageRow {
        let ids = |ss: &[&str]| ss.iter().map(|s| id(s)).collect();
        LineageRow {
            finding_id: id(finding),
            document_id: id(document),
            region_ids: ids(regions),
            rule_ids: ids(rules),
            segment_ids: ids(segments),
            statement_ids: ids(statements),
        }
    }

    // The §8.6 thread live: the three committed test_sources through the
    // mirrored document pipeline, both §8.4 groups through compile +
    // live-z3 verify, assembled into the validating trace pair — census,
    // §8.3 paths, chain/compile/verify/report edges, the three §8.6 claim
    // rows, and the full lineage index pinned.
    #[test]
    fn live_test_source_groups_assemble_full_trace() {
        let lexicon = committed_lexicon();
        let a = live_doc("m1_guideline_a", &lexicon);
        let b = live_doc("m1_guideline_b", &lexicon);
        let control = live_doc("m1_control", &lexicon);
        let adapter = Z3Adapter::new().unwrap();
        let conflict = live_group("group.m1_conflict", &[&a, &b], &adapter);
        let null = live_group("group.m1_no_conflict", &[&a, &control], &adapter);

        let (bundle, index) = assemble_trace(&[a, b, control], &[conflict, null]);
        assert_eq!(bundle.validate(), Ok(()));
        assert_eq!(index.validate(), Ok(()));

        // Node census by kind (1 report + 3 sources + 3×4 document
        // artifacts + 2×2 group artifacts) and the 12 + 4 + 2 + 2 edges.
        for (kind, count) in [
            (TraceNodeKind::Source, 3),
            (TraceNodeKind::SourceDocumentGraph, 3),
            (TraceNodeKind::Segments, 3),
            (TraceNodeKind::Normalization, 3),
            (TraceNodeKind::IrBundle, 3),
            (TraceNodeKind::Compiled, 2),
            (TraceNodeKind::VerifierResults, 2),
            (TraceNodeKind::Report, 1),
        ] {
            let got = bundle.nodes.iter().filter(|n| n.kind == kind).count();
            assert_eq!(got, count, "{}", kind.as_str());
        }
        assert_eq!(bundle.nodes.len(), 20);
        assert_eq!(bundle.edges.len(), 20);

        // §8.3 paths: corpus-relative source (hash = the raw test_source
        // bytes), run-relative artifacts, the static report sink.
        let source = node(&bundle, "test_source.m1_guideline_a");
        assert_eq!(source.kind, TraceNodeKind::Source);
        assert_eq!(source.path, "corpus/test_sources/m1_guideline_a.html");
        assert_eq!(
            source.content_hash,
            Some(hash_bytes(&test_source("m1_guideline_a.html")))
        );
        assert_eq!(
            node(&bundle, "test_source.m1_guideline_a.ir_bundle").path,
            "artifacts/test_source.m1_guideline_a/ir_bundle.json"
        );
        assert_eq!(
            node(&bundle, "group.m1_conflict.compiled").path,
            "groups/group.m1_conflict/compiled.json"
        );
        assert_eq!(
            node(&bundle, "group.m1_no_conflict.verifier_results").path,
            "groups/group.m1_no_conflict/verifier_results.json"
        );
        assert_eq!(node(&bundle, "report"), &report_node());

        // Edge spot checks: docA's full chain, both conflict compile
        // fan-ins, the null group's control fan-in, verify, report.
        for (from, operation, to) in [
            (
                "test_source.m1_guideline_a",
                TraceOperation::Extract,
                "test_source.m1_guideline_a.source_document_graph",
            ),
            (
                "test_source.m1_guideline_a.source_document_graph",
                TraceOperation::Segment,
                "test_source.m1_guideline_a.segments",
            ),
            (
                "test_source.m1_guideline_a.segments",
                TraceOperation::Normalize,
                "test_source.m1_guideline_a.normalization",
            ),
            (
                "test_source.m1_guideline_a.normalization",
                TraceOperation::Assemble,
                "test_source.m1_guideline_a.ir_bundle",
            ),
            (
                "test_source.m1_guideline_a.ir_bundle",
                TraceOperation::Compile,
                "group.m1_conflict.compiled",
            ),
            (
                "test_source.m1_guideline_b.ir_bundle",
                TraceOperation::Compile,
                "group.m1_conflict.compiled",
            ),
            (
                "test_source.m1_control.ir_bundle",
                TraceOperation::Compile,
                "group.m1_no_conflict.compiled",
            ),
            (
                "group.m1_conflict.compiled",
                TraceOperation::Verify,
                "group.m1_conflict.verifier_results",
            ),
            (
                "group.m1_conflict.verifier_results",
                TraceOperation::Report,
                "report",
            ),
        ] {
            assert!(has_edge(&bundle, from, operation, to), "{from} -> {to}");
        }

        // The three §8.6 claim rows.
        assert_eq!(bundle.claims.len(), 3);
        let overlap = claim(&bundle, "finding.group.m1_conflict.0");
        assert_eq!(overlap.pair_id, id("q.m1_conflict.pair1"));
        assert_eq!(overlap.query_id, id("q.m1_conflict.pair1.overlap"));
        assert_eq!(overlap.category, VerifierCategory::SemanticNoConflict);
        assert_eq!(overlap.verdict, Some(SolverVerdict::Sat));
        assert_eq!(overlap.conflict_kind, None);
        // Sat records no core: both ctx.* assertions stand as evidence.
        assert_eq!(
            overlap.assertion_ids,
            vec![
                id("ctx.test_source.m1_guideline_a.rule.0"),
                id("ctx.test_source.m1_guideline_b.rule.0"),
            ]
        );
        let deontic = claim(&bundle, "finding.group.m1_conflict.1");
        assert_eq!(deontic.pair_id, id("q.m1_conflict.pair1"));
        assert_eq!(deontic.query_id, id("q.m1_conflict.pair1.deontic"));
        assert_eq!(deontic.category, VerifierCategory::SemanticContradiction);
        assert_eq!(deontic.verdict, Some(SolverVerdict::Unsat));
        assert_eq!(
            deontic.conflict_kind,
            Some(ConflictKind::DeonticDirectionConflict)
        );
        // The recorded cross-document core, verbatim (§8.6).
        assert_eq!(
            deontic.assertion_ids,
            vec![
                id("a.test_source.m1_guideline_a.rule.0"),
                id("a.test_source.m1_guideline_b.rule.0"),
            ]
        );
        for row in [overlap, deontic] {
            assert_eq!(row.group_id, id("group.m1_conflict"));
            assert_eq!(
                row.rule_ids,
                vec![
                    id("test_source.m1_guideline_a.rule.0"),
                    id("test_source.m1_guideline_b.rule.0"),
                ]
            );
            assert_eq!(row.region_ids, vec![id("r.2"), id("r.3")]);
            assert_eq!(row.report_ref, id("report"));
        }
        // The documented-null overlap row: disjoint intervals answer
        // unsat, no core recorded, ctx fallback over the pair.
        let no_conflict_row = claim(&bundle, "finding.group.m1_no_conflict.0");
        assert_eq!(no_conflict_row.group_id, id("group.m1_no_conflict"));
        assert_eq!(no_conflict_row.pair_id, id("q.m1_no_conflict.pair1"));
        assert_eq!(
            no_conflict_row.query_id,
            id("q.m1_no_conflict.pair1.overlap")
        );
        assert_eq!(
            no_conflict_row.category,
            VerifierCategory::SemanticNoConflict
        );
        assert_eq!(no_conflict_row.verdict, Some(SolverVerdict::Unsat));
        assert_eq!(no_conflict_row.conflict_kind, None);
        assert_eq!(
            no_conflict_row.assertion_ids,
            vec![
                id("ctx.test_source.m1_control.rule.0"),
                id("ctx.test_source.m1_guideline_a.rule.0"),
            ]
        );
        assert_eq!(
            no_conflict_row.rule_ids,
            vec![
                id("test_source.m1_control.rule.0"),
                id("test_source.m1_guideline_a.rule.0"),
            ]
        );
        assert_eq!(no_conflict_row.region_ids, vec![id("r.2"), id("r.3")]);
        assert_eq!(no_conflict_row.report_ref, id("report"));

        // The full lineage index, pinned from observed output: docA rows
        // carry the §8.6 regions and the normalize.rs-pinned statement +
        // segments across all three findings; rows in canonical-set order
        // (document_id first).
        let doc_a = |finding: &str| {
            row(
                finding,
                "test_source.m1_guideline_a",
                &["r.2", "r.3"],
                &["test_source.m1_guideline_a.rule.0"],
                &["seg.2", "seg.3"],
                &["stmt.0"],
            )
        };
        let doc_b = |finding: &str| {
            row(
                finding,
                "test_source.m1_guideline_b",
                &["r.2"],
                &["test_source.m1_guideline_b.rule.0"],
                &["seg.2"],
                &["stmt.0"],
            )
        };
        assert_eq!(
            index.rows,
            vec![
                row(
                    "finding.group.m1_no_conflict.0",
                    "test_source.m1_control",
                    &["r.2"],
                    &["test_source.m1_control.rule.0"],
                    &["seg.2"],
                    &["stmt.0"],
                ),
                doc_a("finding.group.m1_conflict.0"),
                doc_a("finding.group.m1_conflict.1"),
                doc_a("finding.group.m1_no_conflict.0"),
                doc_b("finding.group.m1_conflict.0"),
                doc_b("finding.group.m1_conflict.1"),
            ]
        );
    }
}
