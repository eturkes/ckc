//! SPEC §5 IR layers: DocIR ([`DocIr`]) — the layout-preserving text/table
//! view over [`SourceGraph`] refs with extraction diagnostics — SegmentIR
//! ([`SegmentIr`]) — the document's [`ClinicalSegment`]s — ClinicalIR
//! ([`ClinicalIr`]) — [`ClinicalStatement`]s plus [`TerminologyBinding`]s —
//! NormIR ([`NormIr`]) — [`NormRule`]s over [`ContextExpr`] guards — and
//! FormalIR ([`FormalIr`]) — target-independent [`FormalConstraint`]s plus
//! the [`ContradictionQueryPair`] contradiction-query plan.
//! Layers hold references into the graph; the graph stays the byte authority.
//!
//! The module also defines the §4.3 structural-hash machinery the later
//! layers (core-ir.2/.3) reuse: a component's *structural bytes* are its
//! canonical bytes with every reference id replaced by a local index id
//! (`i0`, `i1`, …) assigned at first occurrence — [`Structural`],
//! [`RefLocalizer`], [`structural_hash`] — so the hash is stable under
//! consistent semantic-id renames while names, multiplicity, and order of
//! structure still change it. Scoping rules, which new impls must follow:
//!
//! - One localizer = one component scope; [`structural_hash`] opens a fresh
//!   scope. Layers emit each contained component under its own fresh scope
//!   (the component boundary), so a layer's structural bytes embed each
//!   component's independently hashable structural bytes.
//! - Implementations call [`ObjectEmitter`] members in canonical (sorted-name)
//!   field order, making index assignment follow the byte order of the result.
//! - Reference ids — the component's own id included — go through
//!   [`RefLocalizer::localize`]; ref *sets* localize in byte order of their
//!   semantic ids ([`emit_structural_ref_set`]). Closed-vocabulary
//!   identifiers (enum spellings, map keys), content (text, numbers), and
//!   content-hash fields (they address external bytes, not renameable ids)
//!   emit verbatim, which keeps local index ids collision-free in the bytes.
//! - Lexicon vocabulary is closed too: concept codes, action
//!   kinds/targets/keys, quantity variables, and binding
//!   systems/codes/alternatives emit verbatim (all-vocabulary values —
//!   [`ContextExpr`], [`Action`] — skip [`Structural`] and embed their
//!   canonical bytes), so structural identity means "same structure over the
//!   same concepts" and a concept swap moves the hash.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::canon::{
    CanonError, CanonRead, CanonReadError, Canonical, ObjectEmitter, ObjectReader, Reader,
    emit_array, emit_i64, emit_set, emit_string, emit_u64, emit_union, read_array, read_i64,
    read_set, read_string, read_u64, read_union,
};
use crate::enums::{
    BindingStatus, DiagnosticCode, DiagnosticRecord, Direction, emit_payload, fieldless_enum,
};
use crate::grounding::{NodeKind, RefKind, SourceGraph, SourceNode};
use crate::hash::hash_bytes;
use crate::id::{Hash, Id, ValidationError};

/// Maps semantic ids to local index ids (`i0`, `i1`, …) by first occurrence.
/// One localizer is one structural scope (module doc).
#[derive(Debug, Default)]
pub struct RefLocalizer {
    indices: HashMap<Id, usize>,
}

impl RefLocalizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// The local index id for `id`, assigned at first call.
    pub fn localize(&mut self, id: &Id) -> Id {
        let next = self.indices.len();
        let n = *self.indices.entry(id.clone()).or_insert(next);
        Id::new(format!("i{n}")).expect("index ids satisfy the Id grammar")
    }
}

/// Structural emission: canonical emission with reference ids localized
/// through `ids` (scoping rules in the module doc).
pub trait Structural {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError>;
}

/// SPEC §4.3 structural hash of one component: sha256 over its structural
/// bytes under a fresh scope.
pub fn structural_hash<T: Structural>(value: &T) -> Result<Hash, CanonError> {
    let mut out = Vec::new();
    value.emit_structural(&mut out, &mut RefLocalizer::new())?;
    Ok(hash_bytes(&out))
}

/// Emit reference ids as a §4.3 set localized in the enclosing scope:
/// members localize in byte order of their semantic ids (for [`Id`] the raw
/// bytes are the canonical order — the grammar admits no escapes), and the
/// localized set sorts by localized bytes. Fresh contiguous indices keep the
/// result rename-stable while the set is its scope's only use of its id pool,
/// which M1 component shapes satisfy.
pub fn emit_structural_ref_set(
    out: &mut Vec<u8>,
    ids: &mut RefLocalizer,
    refs: &[Id],
) -> Result<(), CanonError> {
    let mut sorted: Vec<&Id> = refs.iter().collect();
    sorted.sort_by_key(|r| r.as_str().as_bytes());
    let localized: Vec<Id> = sorted.into_iter().map(|r| ids.localize(r)).collect();
    emit_set(out, &localized)
}

/// Emit reference ids as an ordered array localized in the enclosing scope,
/// for refs whose order is semantic ([`NormRule::source_region_ids`]).
/// Localization follows array order, so the structural bytes see the
/// co-reference pattern and count; the order of refs first occurring here is
/// itself erased (always `i0, i1, …`), like every first-occurrence index.
fn emit_structural_ref_array(
    out: &mut Vec<u8>,
    ids: &mut RefLocalizer,
    refs: &[Id],
) -> Result<(), CanonError> {
    let localized: Vec<Id> = refs.iter().map(|r| ids.localize(r)).collect();
    emit_array(out, &localized)
}

/// Emit an ordered run of structural values inside the enclosing scope
/// (arrays whose order is semantic share their component's scope).
pub(crate) fn emit_structural_array<T: Structural>(
    out: &mut Vec<u8>,
    ids: &mut RefLocalizer,
    items: &[T],
) -> Result<(), CanonError> {
    out.push(b'[');
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        item.emit_structural(out, ids)?;
    }
    out.push(b']');
    Ok(())
}

/// Emit components as an ordered array, each under a fresh scope (the
/// component boundary).
pub(crate) fn emit_structural_components<T: Structural>(
    out: &mut Vec<u8>,
    items: &[T],
) -> Result<(), CanonError> {
    out.push(b'[');
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        item.emit_structural(out, &mut RefLocalizer::new())?;
    }
    out.push(b']');
    Ok(())
}

/// Emit records as a §4.3 set in structural form: each record's structural
/// bytes under a fresh scope, sorted byte-lexicographically, byte-identical
/// duplicates collapsed — so set order is decided by structural content,
/// never by semantic names.
pub(crate) fn emit_structural_record_set<T: Structural>(
    out: &mut Vec<u8>,
    items: &[T],
) -> Result<(), CanonError> {
    let mut elems = Vec::with_capacity(items.len());
    for item in items {
        let mut bytes = Vec::new();
        item.emit_structural(&mut bytes, &mut RefLocalizer::new())?;
        elems.push(bytes);
    }
    elems.sort();
    elems.dedup();
    out.push(b'[');
    for (i, e) in elems.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        out.extend_from_slice(e);
    }
    out.push(b']');
    Ok(())
}

impl Structural for DiagnosticRecord {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("artifact_hashes", |b| emit_set(b, &self.artifact_hashes))?;
        obj.member("code", |b| self.code.emit_canonical(b))?;
        obj.member("outcome", |b| self.outcome.emit_canonical(b))?;
        obj.member("payload", |b| emit_payload(b, &self.payload))?;
        obj.member("region_ids", |b| {
            emit_structural_ref_set(b, ids, &self.region_ids)
        })?;
        obj.finish(out)
    }
}

fieldless_enum! {
    /// SPEC §5 clinical segment kind — the seven M1 kinds (spec prose
    /// "table-row" spells `table_row` in the snake_case token family).
    SegmentKind {
        /// Clinical question.
        Cq => "cq",
        Recommendation => "recommendation",
        Evidence => "evidence",
        Exception => "exception",
        Definition => "definition",
        TableRow => "table_row",
        Metadata => "metadata",
    }
}

fieldless_enum! {
    /// Typed `header` relation of a table cell (§8.2): header cells label
    /// their row or column, body cells carry values.
    CellRole {
        Header => "header",
        Body => "body",
    }
}

/// One reading-order unit of the DocIR text view: a span tagged with its
/// node's kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBlock {
    pub kind: NodeKind,
    pub node_id: Id,
    pub span_id: Id,
}

impl Canonical for TextBlock {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("kind", |b| self.kind.emit_canonical(b))?;
        obj.member("node_id", |b| self.node_id.emit_canonical(b))?;
        obj.member("span_id", |b| self.span_id.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for TextBlock {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let kind = obj.member("kind", NodeKind::read)?;
        let node_id = obj.member("node_id", Id::read)?;
        let span_id = obj.member("span_id", Id::read)?;
        obj.close()?;
        Ok(TextBlock {
            kind,
            node_id,
            span_id,
        })
    }
}

impl Structural for TextBlock {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("kind", |b| self.kind.emit_canonical(b))?;
        obj.member("node_id", |b| ids.localize(&self.node_id).emit_canonical(b))?;
        obj.member("span_id", |b| ids.localize(&self.span_id).emit_canonical(b))?;
        obj.finish(out)
    }
}

/// One typed cell of a [`TableView`]: the §8.2 row/column/header relations
/// parsed out of the cell node's raw attrs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell {
    pub node_id: Id,
    pub row: u64,
    pub col: u64,
    pub role: CellRole,
}

impl Canonical for TableCell {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("col", |b| {
            emit_u64(b, self.col);
            Ok(())
        })?;
        obj.member("node_id", |b| self.node_id.emit_canonical(b))?;
        obj.member("role", |b| self.role.emit_canonical(b))?;
        obj.member("row", |b| {
            emit_u64(b, self.row);
            Ok(())
        })?;
        obj.finish(out)
    }
}

impl CanonRead for TableCell {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let col = obj.member("col", read_u64)?;
        let node_id = obj.member("node_id", Id::read)?;
        let role = obj.member("role", CellRole::read)?;
        let row = obj.member("row", read_u64)?;
        obj.close()?;
        Ok(TableCell {
            node_id,
            row,
            col,
            role,
        })
    }
}

impl Structural for TableCell {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("col", |b| {
            emit_u64(b, self.col);
            Ok(())
        })?;
        obj.member("node_id", |b| ids.localize(&self.node_id).emit_canonical(b))?;
        obj.member("role", |b| self.role.emit_canonical(b))?;
        obj.member("row", |b| {
            emit_u64(b, self.row);
            Ok(())
        })?;
        obj.finish(out)
    }
}

/// Typed view of one table node: its cells in row-major order, positions
/// unique. Caption and heading text reach the text view as ordinary blocks;
/// their table linkage stays queryable in the graph (`parent_id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableView {
    pub table_node_id: Id,
    pub cells: Vec<TableCell>,
}

impl Canonical for TableView {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("cells", |b| emit_array(b, &self.cells))?;
        obj.member("table_node_id", |b| self.table_node_id.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for TableView {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let cells = obj.member("cells", read_array::<TableCell>)?;
        let table_node_id = obj.member("table_node_id", Id::read)?;
        obj.close()?;
        Ok(TableView {
            table_node_id,
            cells,
        })
    }
}

impl Structural for TableView {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("cells", |b| emit_structural_array(b, ids, &self.cells))?;
        obj.member("table_node_id", |b| {
            ids.localize(&self.table_node_id).emit_canonical(b)
        })?;
        obj.finish(out)
    }
}

/// SPEC §5 DocIR: the layout-preserving text/table view over [`SourceGraph`]
/// refs, one component per document. [`from_graph`](Self::from_graph) derives
/// it deterministically; DocIR holds references, never copied text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocIr {
    pub document_id: Id,
    /// Every span in reading order, tagged with its node's kind.
    pub blocks: Vec<TextBlock>,
    /// One view per non-residual table node, in document order.
    pub tables: Vec<TableView>,
    /// Extraction diagnostics carried into the layer (set semantics):
    /// `extraction_uncertain` residuals license unspanned textual nodes
    /// (`IrBundle::validate`); `table_structure_uncertain` residuals
    /// withhold their tables from `tables`.
    pub diagnostics: Vec<DiagnosticRecord>,
}

impl DocIr {
    /// Derive the view from a graph its producer already validated
    /// ([`SourceGraph::validate`]) plus the extraction diagnostics raised for
    /// this document. Fails when a diagnostic's region ref dangles or a
    /// non-residual table's cells carry missing, malformed, or colliding
    /// `row`/`col`/`header` attrs.
    pub fn from_graph(
        graph: &SourceGraph,
        diagnostics: Vec<DiagnosticRecord>,
    ) -> Result<DocIr, IrError> {
        let kinds: HashMap<&Id, NodeKind> =
            graph.nodes.iter().map(|n| (&n.node_id, n.kind)).collect();

        let blocks = graph
            .spans
            .iter()
            .map(|s| {
                let kind = kinds.get(&s.node_id).copied().ok_or(IrError::Dangling {
                    kind: RefKind::Node,
                    id: s.node_id.clone(),
                })?;
                Ok(TextBlock {
                    kind,
                    node_id: s.node_id.clone(),
                    span_id: s.span_id.clone(),
                })
            })
            .collect::<Result<Vec<_>, IrError>>()?;

        // Diagnostics come from outside the graph: every region ref must
        // resolve; table_structure_uncertain regions mark their tables
        // residual.
        let regions: HashMap<&Id, &[Id]> = graph
            .regions
            .iter()
            .map(|r| (&r.region_id, r.node_ids.as_slice()))
            .collect();
        let mut residual_tables: HashSet<&Id> = HashSet::new();
        for diag in &diagnostics {
            for region_id in &diag.region_ids {
                let node_ids = regions.get(region_id).ok_or(IrError::Dangling {
                    kind: RefKind::Region,
                    id: region_id.clone(),
                })?;
                if diag.code == DiagnosticCode::TableStructureUncertain {
                    residual_tables.extend(
                        node_ids
                            .iter()
                            .filter(|n| kinds.get(*n) == Some(&NodeKind::Table)),
                    );
                }
            }
        }

        let mut cells_by_table: HashMap<&Id, Vec<&SourceNode>> = HashMap::new();
        for node in &graph.nodes {
            if node.kind == NodeKind::Cell
                && let Some(parent) = &node.parent_id
            {
                cells_by_table.entry(parent).or_default().push(node);
            }
        }
        let mut tables = Vec::new();
        for node in &graph.nodes {
            if node.kind != NodeKind::Table || residual_tables.contains(&node.node_id) {
                continue;
            }
            let mut cells = cells_by_table
                .remove(&node.node_id)
                .unwrap_or_default()
                .into_iter()
                .map(|cell| {
                    Ok(TableCell {
                        node_id: cell.node_id.clone(),
                        row: cell_u64(cell, "row")?,
                        col: cell_u64(cell, "col")?,
                        role: cell_role(cell)?,
                    })
                })
                .collect::<Result<Vec<_>, IrError>>()?;
            cells.sort_by_key(|c| (c.row, c.col));
            if let Some(w) = cells
                .windows(2)
                .find(|w| (w[0].row, w[0].col) == (w[1].row, w[1].col))
            {
                return Err(IrError::CellCollision {
                    table_id: node.node_id.clone(),
                    row: w[0].row,
                    col: w[0].col,
                });
            }
            tables.push(TableView {
                table_node_id: node.node_id.clone(),
                cells,
            });
        }

        Ok(DocIr {
            document_id: graph.document.document_id.clone(),
            blocks,
            tables,
            diagnostics,
        })
    }
}

/// The raw attr value under `key`, if present.
fn attr<'a>(node: &'a SourceNode, key: &str) -> Option<&'a str> {
    node.attrs
        .iter()
        .find(|(k, _)| k.as_str() == key)
        .map(|(_, v)| v.as_str())
}

/// Parse a required decimal cell attr (`row`, `col`).
fn cell_u64(cell: &SourceNode, key: &'static str) -> Result<u64, IrError> {
    attr(cell, key)
        .and_then(|raw| raw.parse().ok())
        .ok_or_else(|| IrError::CellAttr {
            node_id: cell.node_id.clone(),
            attr: key,
        })
}

/// Parse the optional `header` attr (`true`/`false`; absent means body).
fn cell_role(cell: &SourceNode) -> Result<CellRole, IrError> {
    match attr(cell, "header") {
        None | Some("false") => Ok(CellRole::Body),
        Some("true") => Ok(CellRole::Header),
        Some(_) => Err(IrError::CellAttr {
            node_id: cell.node_id.clone(),
            attr: "header",
        }),
    }
}

impl Canonical for DocIr {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("blocks", |b| emit_array(b, &self.blocks))?;
        obj.member("diagnostics", |b| emit_set(b, &self.diagnostics))?;
        obj.member("document_id", |b| self.document_id.emit_canonical(b))?;
        obj.member("tables", |b| emit_array(b, &self.tables))?;
        obj.finish(out)
    }
}

impl CanonRead for DocIr {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let blocks = obj.member("blocks", read_array::<TextBlock>)?;
        let diagnostics = obj.member("diagnostics", read_set::<DiagnosticRecord>)?;
        let document_id = obj.member("document_id", Id::read)?;
        let tables = obj.member("tables", read_array::<TableView>)?;
        obj.close()?;
        Ok(DocIr {
            document_id,
            blocks,
            tables,
            diagnostics,
        })
    }
}

impl Structural for DocIr {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("blocks", |b| emit_structural_array(b, ids, &self.blocks))?;
        obj.member("diagnostics", |b| {
            emit_structural_record_set(b, &self.diagnostics)
        })?;
        obj.member("document_id", |b| {
            ids.localize(&self.document_id).emit_canonical(b)
        })?;
        obj.member("tables", |b| emit_structural_array(b, ids, &self.tables))?;
        obj.finish(out)
    }
}

/// SPEC §5 clinical segment: one classified unit of guideline text, grounded
/// in §4.5 regions (set semantics; `IrBundle::validate` enforces
/// nonemptiness). An independently hashable component (§5 component records).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClinicalSegment {
    pub segment_id: Id,
    pub kind: SegmentKind,
    pub region_ids: Vec<Id>,
}

impl Canonical for ClinicalSegment {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("kind", |b| self.kind.emit_canonical(b))?;
        obj.member("region_ids", |b| emit_set(b, &self.region_ids))?;
        obj.member("segment_id", |b| self.segment_id.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for ClinicalSegment {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let kind = obj.member("kind", SegmentKind::read)?;
        let region_ids = obj.member("region_ids", read_set::<Id>)?;
        let segment_id = obj.member("segment_id", Id::read)?;
        obj.close()?;
        Ok(ClinicalSegment {
            segment_id,
            kind,
            region_ids,
        })
    }
}

impl Structural for ClinicalSegment {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("kind", |b| self.kind.emit_canonical(b))?;
        obj.member("region_ids", |b| {
            emit_structural_ref_set(b, ids, &self.region_ids)
        })?;
        obj.member("segment_id", |b| {
            ids.localize(&self.segment_id).emit_canonical(b)
        })?;
        obj.finish(out)
    }
}

/// SPEC §5 SegmentIR: the document's [`ClinicalSegment`]s in reading order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentIr {
    pub segments: Vec<ClinicalSegment>,
}

impl Canonical for SegmentIr {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("segments", |b| emit_array(b, &self.segments))?;
        obj.finish(out)
    }
}

impl CanonRead for SegmentIr {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let segments = obj.member("segments", read_array::<ClinicalSegment>)?;
        obj.close()?;
        Ok(SegmentIr { segments })
    }
}

impl Structural for SegmentIr {
    fn emit_structural(
        &self,
        out: &mut Vec<u8>,
        _ids: &mut RefLocalizer,
    ) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("segments", |b| {
            emit_structural_components(b, &self.segments)
        })?;
        obj.finish(out)
    }
}

fieldless_enum! {
    /// SPEC §5 recommendation strength.
    Strength {
        Strong => "strong",
        Weak => "weak",
    }
}

fieldless_enum! {
    /// SPEC §5 evidence certainty (GRADE-style four levels).
    Certainty {
        High => "high",
        Moderate => "moderate",
        Low => "low",
        VeryLow => "very_low",
    }
}

/// SPEC §5 action: kind + target concept + the normalized target key that
/// carries action sameness (§5 semantic policy invariants). All three fields
/// are lexicon vocabulary, verbatim in structural bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub kind: Id,
    pub target: Id,
    /// Normalized target key `kind:target` (M1; M2 joins discriminating
    /// slots). [`new`](Self::new) derives it; bundle validation
    /// (`IrBundle::validate`) re-checks a stored key against its derivation,
    /// the [`SourceSpan`](crate::grounding::SourceSpan) precedent for
    /// derived fields.
    pub key: Id,
}

impl Action {
    /// Build an action, deriving the normalized target key.
    pub fn new(kind: Id, target: Id) -> Action {
        let key =
            Id::new(format!("{kind}:{target}")).expect("two ids joined by ':' form a valid id");
        Action { kind, target, key }
    }
}

impl Canonical for Action {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("key", |b| self.key.emit_canonical(b))?;
        obj.member("kind", |b| self.kind.emit_canonical(b))?;
        obj.member("target", |b| self.target.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for Action {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let key = obj.member("key", Id::read)?;
        let kind = obj.member("kind", Id::read)?;
        let target = obj.member("target", Id::read)?;
        obj.close()?;
        Ok(Action { kind, target, key })
    }
}

/// SPEC §5 quantity-interval guard over a lexicon-defined variable
/// (`成人 → age >= 18` yields `{ var: q.age_years, ge: 18 }`). Bounds are
/// canonical decimal integers, each side optional and omitted when absent.
/// Bound coherence (at least one bound, one bound per side, nonempty
/// interval) is `IrBundle::validate`'s job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantityInterval {
    pub var: Id,
    pub ge: Option<i64>,
    pub gt: Option<i64>,
    pub le: Option<i64>,
    pub lt: Option<i64>,
}

impl Canonical for QuantityInterval {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        let bound = |b: &mut Vec<u8>, v: i64| {
            emit_i64(b, v);
            Ok(())
        };
        obj.optional("ge", self.ge, bound)?;
        obj.optional("gt", self.gt, bound)?;
        obj.optional("le", self.le, bound)?;
        obj.optional("lt", self.lt, bound)?;
        obj.member("var", |b| self.var.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for QuantityInterval {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let ge = obj.optional("ge", read_i64)?;
        let gt = obj.optional("gt", read_i64)?;
        let le = obj.optional("le", read_i64)?;
        let lt = obj.optional("lt", read_i64)?;
        let var = obj.member("var", Id::read)?;
        obj.close()?;
        Ok(QuantityInterval {
            var,
            ge,
            gt,
            le,
            lt,
        })
    }
}

/// SPEC §5 context atom, a §4.3 tagged union: concept predicate, negated
/// concept predicate, or quantity interval (M2 adds slot equality and
/// temporal atoms). Concept ids and interval variables are lexicon
/// vocabulary, so atoms have no [`Structural`] impl — enclosing components
/// embed their canonical bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextAtom {
    Concept(Id),
    ConceptNegated(Id),
    Interval(QuantityInterval),
}

impl Canonical for ContextAtom {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        match self {
            ContextAtom::Concept(c) => emit_union(out, "concept", c),
            ContextAtom::ConceptNegated(c) => emit_union(out, "concept_negated", c),
            ContextAtom::Interval(q) => emit_union(out, "interval", q),
        }
    }
}

impl CanonRead for ContextAtom {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let (_, atom) = read_union(r, |tag, r| match tag {
            "concept" => Ok(ContextAtom::Concept(Id::read(r)?)),
            "concept_negated" => Ok(ContextAtom::ConceptNegated(Id::read(r)?)),
            "interval" => Ok(ContextAtom::Interval(QuantityInterval::read(r)?)),
            other => Err(ValidationError::Enum(format!("ContextAtom has no tag {other:?}")).into()),
        })?;
        Ok(atom)
    }
}

/// One DNF conjunct: a §4.3 set of [`ContextAtom`]s (sorted by canonical
/// bytes, byte-identical duplicates collapsed — `A ∧ A = A`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextConjunct {
    pub all: Vec<ContextAtom>,
}

impl Canonical for ContextConjunct {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("all", |b| emit_set(b, &self.all))?;
        obj.finish(out)
    }
}

impl CanonRead for ContextConjunct {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let all = obj.member("all", read_set::<ContextAtom>)?;
        obj.close()?;
        Ok(ContextConjunct { all })
    }
}

/// SPEC §5 context expression: finite DNF — a §4.3 set of
/// [`ContextConjunct`]s. All-vocabulary, so no [`Structural`] impl (module
/// doc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextExpr {
    pub any: Vec<ContextConjunct>,
}

impl Canonical for ContextExpr {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("any", |b| emit_set(b, &self.any))?;
        obj.finish(out)
    }
}

impl CanonRead for ContextExpr {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let any = obj.member("any", read_set::<ContextConjunct>)?;
        obj.close()?;
        Ok(ContextExpr { any })
    }
}

/// SPEC §5 terminology binding: one mention bound to a concept of `system`
/// (M1: `ckc.lex`). `alternatives` carries the competing codes of an
/// `ambiguous` binding (§4.3 set); `region_ids` ground the mention (§4.3
/// set). System, code, and alternatives are terminology vocabulary — verbatim
/// in structural bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminologyBinding {
    pub binding_id: Id,
    pub system: Id,
    pub code: Id,
    pub status: BindingStatus,
    pub alternatives: Vec<Id>,
    pub region_ids: Vec<Id>,
}

impl Canonical for TerminologyBinding {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("alternatives", |b| emit_set(b, &self.alternatives))?;
        obj.member("binding_id", |b| self.binding_id.emit_canonical(b))?;
        obj.member("code", |b| self.code.emit_canonical(b))?;
        obj.member("region_ids", |b| emit_set(b, &self.region_ids))?;
        obj.member("status", |b| self.status.emit_canonical(b))?;
        obj.member("system", |b| self.system.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for TerminologyBinding {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let alternatives = obj.member("alternatives", read_set::<Id>)?;
        let binding_id = obj.member("binding_id", Id::read)?;
        let code = obj.member("code", Id::read)?;
        let region_ids = obj.member("region_ids", read_set::<Id>)?;
        let status = obj.member("status", BindingStatus::read)?;
        let system = obj.member("system", Id::read)?;
        obj.close()?;
        Ok(TerminologyBinding {
            binding_id,
            system,
            code,
            status,
            alternatives,
            region_ids,
        })
    }
}

impl Structural for TerminologyBinding {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("alternatives", |b| emit_set(b, &self.alternatives))?;
        obj.member("binding_id", |b| {
            ids.localize(&self.binding_id).emit_canonical(b)
        })?;
        obj.member("code", |b| self.code.emit_canonical(b))?;
        obj.member("region_ids", |b| {
            emit_structural_ref_set(b, ids, &self.region_ids)
        })?;
        obj.member("status", |b| self.status.emit_canonical(b))?;
        obj.member("system", |b| self.system.emit_canonical(b))?;
        obj.finish(out)
    }
}

/// One exception clause of a [`ClinicalStatement`]: the exception's own
/// (positive) condition atoms plus its grounding. stage-normalize.2 compiles
/// each clause into negated conjuncts of the rule context, the clause regions
/// joining `source_region_ids` (§5); [`NormRule::exception_refs`] cite
/// `exception_id`s for trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExceptionClause {
    pub exception_id: Id,
    /// §4.3 set; atoms are lexicon vocabulary (verbatim in structural bytes).
    pub atoms: Vec<ContextAtom>,
    /// §4.3 set grounding the exception text.
    pub region_ids: Vec<Id>,
}

impl Canonical for ExceptionClause {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("atoms", |b| emit_set(b, &self.atoms))?;
        obj.member("exception_id", |b| self.exception_id.emit_canonical(b))?;
        obj.member("region_ids", |b| emit_set(b, &self.region_ids))?;
        obj.finish(out)
    }
}

impl CanonRead for ExceptionClause {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let atoms = obj.member("atoms", read_set::<ContextAtom>)?;
        let exception_id = obj.member("exception_id", Id::read)?;
        let region_ids = obj.member("region_ids", read_set::<Id>)?;
        obj.close()?;
        Ok(ExceptionClause {
            exception_id,
            atoms,
            region_ids,
        })
    }
}

impl Structural for ExceptionClause {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("atoms", |b| emit_set(b, &self.atoms))?;
        obj.member("exception_id", |b| {
            ids.localize(&self.exception_id).emit_canonical(b)
        })?;
        obj.member("region_ids", |b| {
            emit_structural_ref_set(b, ids, &self.region_ids)
        })?;
        obj.finish(out)
    }
}

/// SPEC §5 clinical statement: the normalized reading of one recommendation —
/// population and condition as atom sets, the action, deontic modality
/// ([`Direction`]), strength, optional certainty, exception clauses in
/// document order, and the source segments it normalizes (§4.3 set).
/// Comparator/outcome/temporal slots stay M2. An independently hashable
/// component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClinicalStatement {
    pub statement_id: Id,
    /// §4.3 set: who the statement covers (concepts, quantity intervals).
    pub population: Vec<ContextAtom>,
    /// §4.3 set: the clinical condition under which it applies.
    pub condition: Vec<ContextAtom>,
    pub action: Action,
    /// Normalized deontic modality (lexicon modality phrase → direction).
    pub modality: Direction,
    pub strength: Strength,
    /// Omitted from canonical bytes when `None` (§5: optional at M1).
    pub certainty: Option<Certainty>,
    /// Exception clauses in document order.
    pub exceptions: Vec<ExceptionClause>,
    /// §4.3 set: the [`ClinicalSegment`]s this statement normalizes.
    pub source_segment_ids: Vec<Id>,
}

impl Canonical for ClinicalStatement {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("action", |b| self.action.emit_canonical(b))?;
        obj.optional("certainty", self.certainty, |b, c| c.emit_canonical(b))?;
        obj.member("condition", |b| emit_set(b, &self.condition))?;
        obj.member("exceptions", |b| emit_array(b, &self.exceptions))?;
        obj.member("modality", |b| self.modality.emit_canonical(b))?;
        obj.member("population", |b| emit_set(b, &self.population))?;
        obj.member("source_segment_ids", |b| {
            emit_set(b, &self.source_segment_ids)
        })?;
        obj.member("statement_id", |b| self.statement_id.emit_canonical(b))?;
        obj.member("strength", |b| self.strength.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for ClinicalStatement {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let action = obj.member("action", Action::read)?;
        let certainty = obj.optional("certainty", Certainty::read)?;
        let condition = obj.member("condition", read_set::<ContextAtom>)?;
        let exceptions = obj.member("exceptions", read_array::<ExceptionClause>)?;
        let modality = obj.member("modality", Direction::read)?;
        let population = obj.member("population", read_set::<ContextAtom>)?;
        let source_segment_ids = obj.member("source_segment_ids", read_set::<Id>)?;
        let statement_id = obj.member("statement_id", Id::read)?;
        let strength = obj.member("strength", Strength::read)?;
        obj.close()?;
        Ok(ClinicalStatement {
            statement_id,
            population,
            condition,
            action,
            modality,
            strength,
            certainty,
            exceptions,
            source_segment_ids,
        })
    }
}

impl Structural for ClinicalStatement {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("action", |b| self.action.emit_canonical(b))?;
        obj.optional("certainty", self.certainty, |b, c| c.emit_canonical(b))?;
        obj.member("condition", |b| emit_set(b, &self.condition))?;
        obj.member("exceptions", |b| {
            emit_structural_array(b, ids, &self.exceptions)
        })?;
        obj.member("modality", |b| self.modality.emit_canonical(b))?;
        obj.member("population", |b| emit_set(b, &self.population))?;
        obj.member("source_segment_ids", |b| {
            emit_structural_ref_set(b, ids, &self.source_segment_ids)
        })?;
        obj.member("statement_id", |b| {
            ids.localize(&self.statement_id).emit_canonical(b)
        })?;
        obj.member("strength", |b| self.strength.emit_canonical(b))?;
        obj.finish(out)
    }
}

/// SPEC §5 ClinicalIR: the document's [`ClinicalStatement`]s and
/// [`TerminologyBinding`]s, each list in reading/derivation order, each
/// element an independently hashable component. CQ/PICO/EtD slots stay
/// optional-M1, unscheduled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClinicalIr {
    pub bindings: Vec<TerminologyBinding>,
    pub statements: Vec<ClinicalStatement>,
}

impl Canonical for ClinicalIr {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("bindings", |b| emit_array(b, &self.bindings))?;
        obj.member("statements", |b| emit_array(b, &self.statements))?;
        obj.finish(out)
    }
}

impl CanonRead for ClinicalIr {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let bindings = obj.member("bindings", read_array::<TerminologyBinding>)?;
        let statements = obj.member("statements", read_array::<ClinicalStatement>)?;
        obj.close()?;
        Ok(ClinicalIr {
            bindings,
            statements,
        })
    }
}

impl Structural for ClinicalIr {
    fn emit_structural(
        &self,
        out: &mut Vec<u8>,
        _ids: &mut RefLocalizer,
    ) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("bindings", |b| {
            emit_structural_components(b, &self.bindings)
        })?;
        obj.member("statements", |b| {
            emit_structural_components(b, &self.statements)
        })?;
        obj.finish(out)
    }
}

/// SPEC §5 norm rule: one guarded deontic constraint. The §8.6 worked rule
/// pins the canonical bytes. An independently hashable component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormRule {
    pub rule_id: Id,
    pub context: ContextExpr,
    pub direction: Direction,
    pub action: Action,
    pub strength: Strength,
    /// Ordered array, not a set (§8.6 pins `[rec, exc]`): the rule's own
    /// regions in source order, exception regions joining as their clauses
    /// compile in (§5).
    pub source_region_ids: Vec<Id>,
    /// Omitted from canonical bytes when `None` (§5: optional at M1).
    pub certainty: Option<Certainty>,
    /// §4.3 set of [`ExceptionClause`] ids, for trace. Empty means absent:
    /// the writer omits an empty set and the reader rejects a present-but-
    /// empty one, keeping one canonical encoding (§5: optional at M1).
    pub exception_refs: Vec<Id>,
}

impl Canonical for NormRule {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("action", |b| self.action.emit_canonical(b))?;
        obj.optional("certainty", self.certainty, |b, c| c.emit_canonical(b))?;
        obj.member("context", |b| self.context.emit_canonical(b))?;
        obj.member("direction", |b| self.direction.emit_canonical(b))?;
        obj.optional(
            "exception_refs",
            (!self.exception_refs.is_empty()).then_some(&self.exception_refs),
            emit_set,
        )?;
        obj.member("rule_id", |b| self.rule_id.emit_canonical(b))?;
        obj.member("source_region_ids", |b| {
            emit_array(b, &self.source_region_ids)
        })?;
        obj.member("strength", |b| self.strength.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for NormRule {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let action = obj.member("action", Action::read)?;
        let certainty = obj.optional("certainty", Certainty::read)?;
        let context = obj.member("context", ContextExpr::read)?;
        let direction = obj.member("direction", Direction::read)?;
        let exception_refs = obj.optional("exception_refs", read_set::<Id>)?;
        if exception_refs.as_ref().is_some_and(Vec::is_empty) {
            return Err(CanonReadError::UnknownField("exception_refs".to_owned()));
        }
        let exception_refs = exception_refs.unwrap_or_default();
        let rule_id = obj.member("rule_id", Id::read)?;
        let source_region_ids = obj.member("source_region_ids", read_array::<Id>)?;
        let strength = obj.member("strength", Strength::read)?;
        obj.close()?;
        Ok(NormRule {
            rule_id,
            context,
            direction,
            action,
            strength,
            source_region_ids,
            certainty,
            exception_refs,
        })
    }
}

impl Structural for NormRule {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("action", |b| self.action.emit_canonical(b))?;
        obj.optional("certainty", self.certainty, |b, c| c.emit_canonical(b))?;
        obj.member("context", |b| self.context.emit_canonical(b))?;
        obj.member("direction", |b| self.direction.emit_canonical(b))?;
        obj.optional(
            "exception_refs",
            (!self.exception_refs.is_empty()).then_some(&self.exception_refs),
            |b, v| emit_structural_ref_set(b, ids, v),
        )?;
        obj.member("rule_id", |b| ids.localize(&self.rule_id).emit_canonical(b))?;
        obj.member("source_region_ids", |b| {
            emit_structural_ref_array(b, ids, &self.source_region_ids)
        })?;
        obj.member("strength", |b| self.strength.emit_canonical(b))?;
        obj.finish(out)
    }
}

/// SPEC §5 NormIR: the document's [`NormRule`]s in derivation order, each an
/// independently hashable component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormIr {
    pub rules: Vec<NormRule>,
}

impl Canonical for NormIr {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("rules", |b| emit_array(b, &self.rules))?;
        obj.finish(out)
    }
}

impl CanonRead for NormIr {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let rules = obj.member("rules", read_array::<NormRule>)?;
        obj.close()?;
        Ok(NormIr { rules })
    }
}

impl Structural for NormIr {
    fn emit_structural(
        &self,
        out: &mut Vec<u8>,
        _ids: &mut RefLocalizer,
    ) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("rules", |b| emit_structural_components(b, &self.rules))?;
        obj.finish(out)
    }
}

/// SPEC §6 direction-group opposition, the direction half of conflict
/// eligibility (the other half is action sameness via normalized keys, §5):
/// true when one direction is in the positive group (`for`/`require`/
/// `permit`) and the other is in the against (`against`/`avoid`) or
/// contraindicating (`contraindicate`/`avoid`) group.
pub fn directions_opposed(a: Direction, b: Direction) -> bool {
    let positive =
        |d: Direction| matches!(d, Direction::For | Direction::Require | Direction::Permit);
    let opposing = |d: Direction| {
        matches!(
            d,
            Direction::Against | Direction::Avoid | Direction::Contraindicate
        )
    };
    (positive(a) && opposing(b)) || (positive(b) && opposing(a))
}

/// SPEC §5 FormalIR constraint: the target-independent projection of one
/// [`NormRule`] — self-contained [`Action`], folded [`ContextExpr`] guard,
/// direction, and the proof-visible strength/certainty annotations (§5:
/// conflict logic consumes direction and normalized action/context). Source
/// regions stay reachable through `rule_id`. An independently hashable
/// component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalConstraint {
    /// `fc.<rule_id>`, derived by [`from_rule`](Self::from_rule); bundle
    /// validation (`IrBundle::validate`) re-checks stored values against
    /// the derivation, like every derived field.
    pub constraint_id: Id,
    pub rule_id: Id,
    pub action: Action,
    pub context: ContextExpr,
    pub direction: Direction,
    pub strength: Strength,
    /// Omitted from canonical bytes when `None` (§5: optional at M1).
    pub certainty: Option<Certainty>,
}

impl FormalConstraint {
    /// Project a rule into its constraint: `constraint_id = fc.<rule_id>`,
    /// every other field a straight copy — [`NormRule::context`] already
    /// folds exceptions into negated conjuncts (§5).
    pub fn from_rule(rule: &NormRule) -> FormalConstraint {
        let constraint_id = Id::new(format!("fc.{}", rule.rule_id))
            .expect("'fc.' before a valid id forms a valid id");
        FormalConstraint {
            constraint_id,
            rule_id: rule.rule_id.clone(),
            action: rule.action.clone(),
            context: rule.context.clone(),
            direction: rule.direction,
            strength: rule.strength,
            certainty: rule.certainty,
        }
    }
}

impl Canonical for FormalConstraint {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("action", |b| self.action.emit_canonical(b))?;
        obj.optional("certainty", self.certainty, |b, c| c.emit_canonical(b))?;
        obj.member("constraint_id", |b| self.constraint_id.emit_canonical(b))?;
        obj.member("context", |b| self.context.emit_canonical(b))?;
        obj.member("direction", |b| self.direction.emit_canonical(b))?;
        obj.member("rule_id", |b| self.rule_id.emit_canonical(b))?;
        obj.member("strength", |b| self.strength.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for FormalConstraint {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let action = obj.member("action", Action::read)?;
        let certainty = obj.optional("certainty", Certainty::read)?;
        let constraint_id = obj.member("constraint_id", Id::read)?;
        let context = obj.member("context", ContextExpr::read)?;
        let direction = obj.member("direction", Direction::read)?;
        let rule_id = obj.member("rule_id", Id::read)?;
        let strength = obj.member("strength", Strength::read)?;
        obj.close()?;
        Ok(FormalConstraint {
            constraint_id,
            rule_id,
            action,
            context,
            direction,
            strength,
            certainty,
        })
    }
}

impl Structural for FormalConstraint {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("action", |b| self.action.emit_canonical(b))?;
        obj.optional("certainty", self.certainty, |b, c| c.emit_canonical(b))?;
        obj.member("constraint_id", |b| {
            ids.localize(&self.constraint_id).emit_canonical(b)
        })?;
        obj.member("context", |b| self.context.emit_canonical(b))?;
        obj.member("direction", |b| self.direction.emit_canonical(b))?;
        obj.member("rule_id", |b| ids.localize(&self.rule_id).emit_canonical(b))?;
        obj.member("strength", |b| self.strength.emit_canonical(b))?;
        obj.finish(out)
    }
}

/// SPEC §5/§6 contradiction-query plan slot: one conflict-eligible
/// constraint pair — same normalized action key, [`directions_opposed`],
/// `constraint_a_id < constraint_b_id` by id bytes (`IrBundle::validate`) —
/// holding the planner-minted ids of its two §6 queries (Q1
/// context_overlap, Q2 deontic_consistency; §8.6 spells them
/// `q.<group>.<pair>.overlap`/`.deontic`). Planning lands in smt-emit.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContradictionQueryPair {
    pub pair_id: Id,
    pub action_key: Id,
    pub constraint_a_id: Id,
    pub constraint_b_id: Id,
    pub context_overlap_query_id: Id,
    pub deontic_consistency_query_id: Id,
}

impl Canonical for ContradictionQueryPair {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("action_key", |b| self.action_key.emit_canonical(b))?;
        obj.member("constraint_a_id", |b| {
            self.constraint_a_id.emit_canonical(b)
        })?;
        obj.member("constraint_b_id", |b| {
            self.constraint_b_id.emit_canonical(b)
        })?;
        obj.member("context_overlap_query_id", |b| {
            self.context_overlap_query_id.emit_canonical(b)
        })?;
        obj.member("deontic_consistency_query_id", |b| {
            self.deontic_consistency_query_id.emit_canonical(b)
        })?;
        obj.member("pair_id", |b| self.pair_id.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for ContradictionQueryPair {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let action_key = obj.member("action_key", Id::read)?;
        let constraint_a_id = obj.member("constraint_a_id", Id::read)?;
        let constraint_b_id = obj.member("constraint_b_id", Id::read)?;
        let context_overlap_query_id = obj.member("context_overlap_query_id", Id::read)?;
        let deontic_consistency_query_id = obj.member("deontic_consistency_query_id", Id::read)?;
        let pair_id = obj.member("pair_id", Id::read)?;
        obj.close()?;
        Ok(ContradictionQueryPair {
            pair_id,
            action_key,
            constraint_a_id,
            constraint_b_id,
            context_overlap_query_id,
            deontic_consistency_query_id,
        })
    }
}

impl Structural for ContradictionQueryPair {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("action_key", |b| self.action_key.emit_canonical(b))?;
        obj.member("constraint_a_id", |b| {
            ids.localize(&self.constraint_a_id).emit_canonical(b)
        })?;
        obj.member("constraint_b_id", |b| {
            ids.localize(&self.constraint_b_id).emit_canonical(b)
        })?;
        obj.member("context_overlap_query_id", |b| {
            ids.localize(&self.context_overlap_query_id)
                .emit_canonical(b)
        })?;
        obj.member("deontic_consistency_query_id", |b| {
            ids.localize(&self.deontic_consistency_query_id)
                .emit_canonical(b)
        })?;
        obj.member("pair_id", |b| ids.localize(&self.pair_id).emit_canonical(b))?;
        obj.finish(out)
    }
}

/// SPEC §5 FormalIR: the document's [`FormalConstraint`]s in rule order plus
/// the contradiction-query plan (ordered array; pairs may cross documents,
/// so [`derive`](Self::derive) leaves it for the planner, smt-emit.2).
/// Constraints emit under per-component scopes; plan entries localize in the
/// layer scope, so cross-pair co-reference stays in the structural bytes
/// (module doc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalIr {
    pub constraints: Vec<FormalConstraint>,
    pub plan: Vec<ContradictionQueryPair>,
}

impl FormalIr {
    /// Derive the layer from NormIR: one constraint per rule, in rule order;
    /// the plan stays empty until planning (smt-emit.2).
    pub fn derive(norm: &NormIr) -> FormalIr {
        FormalIr {
            constraints: norm.rules.iter().map(FormalConstraint::from_rule).collect(),
            plan: Vec::new(),
        }
    }
}

impl Canonical for FormalIr {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("constraints", |b| emit_array(b, &self.constraints))?;
        obj.member("plan", |b| emit_array(b, &self.plan))?;
        obj.finish(out)
    }
}

impl CanonRead for FormalIr {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let constraints = obj.member("constraints", read_array::<FormalConstraint>)?;
        let plan = obj.member("plan", read_array::<ContradictionQueryPair>)?;
        obj.close()?;
        Ok(FormalIr { constraints, plan })
    }
}

impl Structural for FormalIr {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("constraints", |b| {
            emit_structural_components(b, &self.constraints)
        })?;
        obj.member("plan", |b| emit_structural_array(b, ids, &self.plan))?;
        obj.finish(out)
    }
}

/// A DocIR derivation input broke its contract ([`DocIr::from_graph`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrError {
    /// A carried diagnostic references a region the graph does not define, or
    /// a span references an unknown node (impossible on validated graphs).
    Dangling { kind: RefKind, id: Id },
    /// A non-residual table cell's `row`/`col`/`header` attr is missing or
    /// unparsable.
    CellAttr { node_id: Id, attr: &'static str },
    /// Two cells of one table claim the same position.
    CellCollision { table_id: Id, row: u64, col: u64 },
}

impl fmt::Display for IrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrError::Dangling { kind, id } => {
                write!(f, "reference to undefined {} {id}", kind.as_str())
            }
            IrError::CellAttr { node_id, attr } => {
                write!(f, "cell {node_id} attr {attr} is missing or unparsable")
            }
            IrError::CellCollision { table_id, row, col } => {
                write!(f, "table {table_id} has two cells at ({row},{col})")
            }
        }
    }
}

impl std::error::Error for IrError {}

/// Test fixtures shared with sibling modules (bundle): the §8.6 worked-rule
/// family under a rename prefix, atom/DNF builders, and byte-pin helpers.
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::canon::{canonical_payload_bytes, read_canonical};
    use crate::enums::Outcome;
    use crate::grounding::{DataClass, Provenance, SourceDocument, SourceRegion, SourceSpan};
    use crate::hash::content_hash;

    pub(crate) fn canon<T: Canonical>(value: &T) -> String {
        String::from_utf8(canonical_payload_bytes(value).unwrap()).unwrap()
    }

    pub(crate) fn structural<T: Structural>(value: &T) -> String {
        let mut out = Vec::new();
        value
            .emit_structural(&mut out, &mut RefLocalizer::new())
            .unwrap();
        String::from_utf8(out).unwrap()
    }

    pub(crate) fn round_trip<T: Canonical + CanonRead + std::fmt::Debug + PartialEq>(value: T) {
        let bytes = canonical_payload_bytes(&value).unwrap();
        let got: T = read_canonical(&bytes).unwrap();
        assert_eq!(got, value, "round trip changed the value");
    }

    pub(crate) fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn h(digit: char) -> Hash {
        Hash::new(format!("sha256:{}", digit.to_string().repeat(64))).unwrap()
    }

    fn node(kind: NodeKind, nid: &str, parent: Option<&str>) -> SourceNode {
        SourceNode {
            node_id: id(nid),
            kind,
            parent_id: parent.map(id),
            attrs: vec![],
        }
    }

    fn cell(nid: &str, parent: &str, row: &str, col: &str, header: Option<&str>) -> SourceNode {
        let mut attrs = vec![(id("row"), row.to_owned()), (id("col"), col.to_owned())];
        if let Some(v) = header {
            attrs.push((id("header"), v.to_owned()));
        }
        SourceNode {
            node_id: id(nid),
            kind: NodeKind::Cell,
            parent_id: Some(id(parent)),
            attrs,
        }
    }

    /// A definitions-table graph in the §8.2 shape: a CQ heading, a
    /// recommendation paragraph, and a 2×2 table (header row 用語/定義,
    /// body row 成人/18歳以上).
    fn sample_graph() -> SourceGraph {
        let texts: [(&str, &str, &str); 6] = [
            ("s.cq1", "n.cq1", "ＣＱ１：高血圧の治療"),
            ("s.p1", "n.p1", "成人には１０ｍｇを投与する。"),
            ("s.c11", "n.c11", "用語"),
            ("s.c12", "n.c12", "定義"),
            ("s.c21", "n.c21", "成人"),
            ("s.c22", "n.c22", "18歳以上"),
        ];
        SourceGraph {
            document: SourceDocument {
                document_id: id("doc.a"),
                source_family: id("synthetic_fixture_html"),
                provenance: Provenance::Synthetic,
                raw_hash: h('a'),
                content_hash: h('a'),
                data_class: DataClass::None,
            },
            nodes: vec![
                node(NodeKind::Document, "n.doc", None),
                node(NodeKind::Cq, "n.cq1", Some("n.doc")),
                node(NodeKind::Paragraph, "n.p1", Some("n.doc")),
                node(NodeKind::Table, "n.t1", Some("n.doc")),
                cell("n.c11", "n.t1", "1", "1", Some("true")),
                cell("n.c12", "n.t1", "1", "2", Some("true")),
                cell("n.c21", "n.t1", "2", "1", None),
                cell("n.c22", "n.t1", "2", "2", Some("false")),
            ],
            spans: texts
                .iter()
                .enumerate()
                .map(|(i, (sid, nid, raw))| {
                    SourceSpan::derive(id(sid), id(nid), 0, (*raw).to_owned(), i as u64 + 1)
                })
                .collect(),
            anchors: vec![],
            regions: vec![
                SourceRegion {
                    region_id: id("r.cq1"),
                    node_ids: vec![id("n.cq1")],
                    span_ids: vec![id("s.cq1")],
                    anchor_ids: vec![],
                },
                SourceRegion {
                    region_id: id("r.t1"),
                    node_ids: vec![id("n.t1")],
                    span_ids: vec![],
                    anchor_ids: vec![],
                },
            ],
        }
    }

    pub(crate) fn diag(code: DiagnosticCode, region: &str) -> DiagnosticRecord {
        DiagnosticRecord {
            code,
            outcome: Outcome::Residual,
            payload: vec![(id("detail"), "test".to_owned())],
            region_ids: vec![id(region)],
            artifact_hashes: vec![],
        }
    }

    fn segment(sid: &str, kind: SegmentKind, regions: &[&str]) -> ClinicalSegment {
        ClinicalSegment {
            segment_id: id(sid),
            kind,
            region_ids: regions.iter().map(|r| id(r)).collect(),
        }
    }

    // Pins the §5 segment-kind set and the cell-role vocabulary.
    #[test]
    fn segment_and_cell_kind_spellings() {
        let spelled: Vec<&str> = SegmentKind::ALL.iter().map(|&v| v.as_str()).collect();
        assert_eq!(
            spelled,
            [
                "cq",
                "recommendation",
                "evidence",
                "exception",
                "definition",
                "table_row",
                "metadata"
            ]
        );
        let spelled: Vec<&str> = CellRole::ALL.iter().map(|&v| v.as_str()).collect();
        assert_eq!(spelled, ["header", "body"]);
        for v in SegmentKind::ALL {
            round_trip(*v);
        }
        for v in CellRole::ALL {
            round_trip(*v);
        }
    }

    #[test]
    fn clinical_segment_canonical_bytes_and_round_trip() {
        let seg = segment(
            "seg.a.cq1.rec1",
            SegmentKind::Recommendation,
            &["r.b", "r.a"],
        );
        assert_eq!(
            canon(&seg),
            concat!(
                r#"{"kind":"recommendation","region_ids":["r.a","r.b"],"#,
                r#""segment_id":"seg.a.cq1.rec1"}"#
            )
        );
        // the value that round-trips unchanged is the set-sorted one
        round_trip(segment(
            "seg.a.cq1.rec1",
            SegmentKind::Recommendation,
            &["r.a", "r.b"],
        ));
        round_trip(SegmentIr {
            segments: vec![
                segment("seg.a.cq1", SegmentKind::Cq, &["r.cq1"]),
                segment("seg.a.cq1.rec1", SegmentKind::Recommendation, &["r.a"]),
            ],
        });
    }

    #[test]
    fn doc_ir_from_graph_builds_view() {
        let graph = sample_graph();
        graph.validate(&[]).unwrap();
        let ir = DocIr::from_graph(&graph, vec![]).unwrap();
        assert_eq!(ir.document_id, id("doc.a"));
        let blocks: Vec<(&str, &str)> = ir
            .blocks
            .iter()
            .map(|b| (b.span_id.as_str(), b.kind.as_str()))
            .collect();
        assert_eq!(
            blocks,
            [
                ("s.cq1", "cq"),
                ("s.p1", "paragraph"),
                ("s.c11", "cell"),
                ("s.c12", "cell"),
                ("s.c21", "cell"),
                ("s.c22", "cell")
            ]
        );
        assert_eq!(ir.tables.len(), 1);
        let table = &ir.tables[0];
        assert_eq!(table.table_node_id, id("n.t1"));
        let cells: Vec<(u64, u64, &str, &str)> = table
            .cells
            .iter()
            .map(|c| (c.row, c.col, c.role.as_str(), c.node_id.as_str()))
            .collect();
        assert_eq!(
            cells,
            [
                (1, 1, "header", "n.c11"),
                (1, 2, "header", "n.c12"),
                (2, 1, "body", "n.c21"),
                (2, 2, "body", "n.c22")
            ]
        );
        round_trip(ir);
    }

    // Pins the four-field DocIR shape over a minimal graph.
    #[test]
    fn doc_ir_canonical_bytes() {
        let mut graph = sample_graph();
        graph.nodes.truncate(2); // n.doc, n.cq1
        graph.spans.truncate(1);
        graph.regions.truncate(1);
        let ir = DocIr::from_graph(
            &graph,
            vec![diag(DiagnosticCode::ExtractionUncertain, "r.cq1")],
        )
        .unwrap();
        let want = concat!(
            r#"{"blocks":[{"kind":"cq","node_id":"n.cq1","span_id":"s.cq1"}],"#,
            r#""diagnostics":[{"artifact_hashes":[],"code":"extraction_uncertain","#,
            r#""outcome":"residual","payload":{"detail":"test"},"region_ids":["r.cq1"]}],"#,
            r#""document_id":"doc.a","#,
            r#""tables":[]}"#
        );
        assert_eq!(canon(&ir), want);
        round_trip(ir);
    }

    #[test]
    fn from_graph_rejects_bad_cells() {
        // missing col
        let mut graph = sample_graph();
        graph.nodes[6].attrs.retain(|(k, _)| k.as_str() != "col");
        assert_eq!(
            DocIr::from_graph(&graph, vec![]),
            Err(IrError::CellAttr {
                node_id: id("n.c21"),
                attr: "col"
            })
        );

        // unparsable row
        let mut graph = sample_graph();
        graph.nodes[6].attrs[0].1 = "x".to_owned();
        assert_eq!(
            DocIr::from_graph(&graph, vec![]),
            Err(IrError::CellAttr {
                node_id: id("n.c21"),
                attr: "row"
            })
        );

        // junk header value
        let mut graph = sample_graph();
        graph.nodes[7].attrs[2].1 = "yes".to_owned();
        assert_eq!(
            DocIr::from_graph(&graph, vec![]),
            Err(IrError::CellAttr {
                node_id: id("n.c22"),
                attr: "header"
            })
        );

        // colliding position
        let mut graph = sample_graph();
        graph.nodes[6].attrs = graph.nodes[4].attrs.clone();
        assert_eq!(
            DocIr::from_graph(&graph, vec![]),
            Err(IrError::CellCollision {
                table_id: id("n.t1"),
                row: 1,
                col: 1
            })
        );
    }

    #[test]
    fn from_graph_resolves_diagnostic_regions() {
        let graph = sample_graph();
        assert_eq!(
            DocIr::from_graph(
                &graph,
                vec![diag(DiagnosticCode::ExtractionUncertain, "r.zzz")]
            ),
            Err(IrError::Dangling {
                kind: RefKind::Region,
                id: id("r.zzz")
            })
        );
    }

    // A table_structure_uncertain residual withholds its table view; the
    // cell text stays in the blocks and the diagnostic stays in the layer.
    #[test]
    fn residual_table_withheld() {
        let graph = sample_graph();
        let residual = diag(DiagnosticCode::TableStructureUncertain, "r.t1");
        let ir = DocIr::from_graph(&graph, vec![residual.clone()]).unwrap();
        assert_eq!(ir.tables, []);
        assert_eq!(ir.blocks.len(), 6);
        assert_eq!(ir.diagnostics, std::slice::from_ref(&residual));
        // the residual also licenses skipping the cells' attr parsing
        let mut graph = sample_graph();
        graph.nodes[4].attrs.clear();
        DocIr::from_graph(&graph, vec![residual]).unwrap();
    }

    // Pins the structural byte pattern: refs become first-occurrence index
    // ids in canonical field order; each component opens a fresh scope.
    #[test]
    fn structural_bytes_pin() {
        let seg = segment(
            "seg.a.cq1.rec1",
            SegmentKind::Recommendation,
            &["r.b", "r.a"],
        );
        assert_eq!(
            structural(&seg),
            r#"{"kind":"recommendation","region_ids":["i0","i1"],"segment_id":"i2"}"#
        );
        let layer = SegmentIr {
            segments: vec![
                segment("seg.a.cq1", SegmentKind::Cq, &["r.cq1"]),
                segment("seg.a.cq1.rec1", SegmentKind::Recommendation, &["r.a"]),
            ],
        };
        // both segments restart at i0: per-component scopes
        assert_eq!(
            structural(&layer),
            concat!(
                r#"{"segments":[{"kind":"cq","region_ids":["i0"],"segment_id":"i1"},"#,
                r#"{"kind":"recommendation","region_ids":["i0"],"segment_id":"i1"}]}"#
            )
        );
        let record = diag(DiagnosticCode::TableStructureUncertain, "r.t1");
        assert_eq!(
            structural(&record),
            concat!(
                r#"{"artifact_hashes":[],"code":"table_structure_uncertain","#,
                r#""outcome":"residual","payload":{"detail":"test"},"region_ids":["i0"]}"#
            )
        );
    }

    // §4.3: structural hashes are stable under consistent semantic-id
    // renames, even ones that permute set order, while content hashes move.
    #[test]
    fn structural_hash_rename_stable() {
        let layer = SegmentIr {
            segments: vec![
                segment("seg.1", SegmentKind::Exception, &["r.a", "r.b"]),
                segment("seg.2", SegmentKind::Recommendation, &["r.a"]),
            ],
        };
        // r.a -> r.z flips the name order inside seg.1's region set
        let renamed = SegmentIr {
            segments: vec![
                segment("seg.one", SegmentKind::Exception, &["r.z", "r.b"]),
                segment("seg.two", SegmentKind::Recommendation, &["r.z"]),
            ],
        };
        assert_eq!(
            structural_hash(&layer).unwrap(),
            structural_hash(&renamed).unwrap()
        );
        assert_ne!(
            content_hash(&layer).unwrap(),
            content_hash(&renamed).unwrap()
        );
        // per-component hashes agree too
        assert_eq!(
            structural_hash(&layer.segments[0]).unwrap(),
            structural_hash(&renamed.segments[0]).unwrap()
        );

        let graph = sample_graph();
        let diags = vec![diag(DiagnosticCode::ExtractionUncertain, "r.cq1")];
        let ir = DocIr::from_graph(&graph, diags).unwrap();
        // rename every graph id (uniform prefix) and the diagnostic's region
        let prefixed = |i: &Id| Id::new(format!("x.{}", i.as_str())).unwrap();
        let mut renamed_graph = graph.clone();
        renamed_graph.document.document_id = prefixed(&graph.document.document_id);
        for n in &mut renamed_graph.nodes {
            n.node_id = prefixed(&n.node_id);
            n.parent_id = n.parent_id.as_ref().map(&prefixed);
        }
        for s in &mut renamed_graph.spans {
            s.span_id = prefixed(&s.span_id);
            s.node_id = prefixed(&s.node_id);
        }
        for r in &mut renamed_graph.regions {
            r.region_id = prefixed(&r.region_id);
            r.node_ids = r.node_ids.iter().map(&prefixed).collect();
            r.span_ids = r.span_ids.iter().map(&prefixed).collect();
        }
        renamed_graph.validate(&[]).unwrap();
        let renamed_ir = DocIr::from_graph(
            &renamed_graph,
            vec![diag(DiagnosticCode::ExtractionUncertain, "x.r.cq1")],
        )
        .unwrap();
        assert_eq!(
            structural_hash(&ir).unwrap(),
            structural_hash(&renamed_ir).unwrap()
        );
        assert_ne!(
            content_hash(&ir).unwrap(),
            content_hash(&renamed_ir).unwrap()
        );
    }

    // Structure — kinds, multiplicity, co-reference pattern, order — moves
    // the structural hash.
    #[test]
    fn structural_hash_sees_structure() {
        let base = SegmentIr {
            segments: vec![segment("seg.1", SegmentKind::Recommendation, &["r.a"])],
        };
        let kind_changed = SegmentIr {
            segments: vec![segment("seg.1", SegmentKind::Exception, &["r.a"])],
        };
        let region_added = SegmentIr {
            segments: vec![segment(
                "seg.1",
                SegmentKind::Recommendation,
                &["r.a", "r.b"],
            )],
        };
        let duplicated = SegmentIr {
            segments: vec![
                segment("seg.1", SegmentKind::Recommendation, &["r.a"]),
                segment("seg.1", SegmentKind::Recommendation, &["r.a"]),
            ],
        };
        let base_hash = structural_hash(&base).unwrap();
        for other in [&kind_changed, &region_added, &duplicated] {
            assert_ne!(base_hash, structural_hash(other).unwrap());
        }
    }

    // ---- core-ir.2: ClinicalIR + NormIR ------------------------------------

    pub(crate) fn atom_c(c: &str) -> ContextAtom {
        ContextAtom::Concept(id(c))
    }

    pub(crate) fn atom_nc(c: &str) -> ContextAtom {
        ContextAtom::ConceptNegated(id(c))
    }

    pub(crate) fn atom_ge(var: &str, n: i64) -> ContextAtom {
        ContextAtom::Interval(QuantityInterval {
            var: id(var),
            ge: Some(n),
            gt: None,
            le: None,
            lt: None,
        })
    }

    pub(crate) fn dnf1(all: Vec<ContextAtom>) -> ContextExpr {
        ContextExpr {
            any: vec![ContextConjunct { all }],
        }
    }

    /// The §8.6 worked rule `rule.a.cq1.r1`, local ids under prefix `p`.
    pub(crate) fn rule_p(p: &str) -> NormRule {
        NormRule {
            rule_id: id(&format!("{p}rule.a.cq1.r1")),
            context: dnf1(vec![
                atom_c("cond.sepsis"),
                atom_nc("cond.renal_severe"),
                atom_ge("q.age_years", 18),
            ]),
            direction: Direction::For,
            action: Action::new(id("act.administer"), id("drug.abx_a")),
            strength: Strength::Strong,
            source_region_ids: vec![
                id(&format!("{p}region.a.cq1.rec")),
                id(&format!("{p}region.a.cq1.exc")),
            ],
            certainty: None,
            exception_refs: vec![],
        }
    }

    /// The statement the §8.6 rule normalizes from, local ids under `p`.
    pub(crate) fn statement_p(p: &str) -> ClinicalStatement {
        ClinicalStatement {
            statement_id: id(&format!("{p}st.a.cq1.s1")),
            population: vec![atom_ge("q.age_years", 18)],
            condition: vec![atom_c("cond.sepsis")],
            action: Action::new(id("act.administer"), id("drug.abx_a")),
            modality: Direction::For,
            strength: Strength::Strong,
            certainty: Some(Certainty::Moderate),
            exceptions: vec![ExceptionClause {
                exception_id: id(&format!("{p}exc.a.cq1.e1")),
                atoms: vec![atom_c("cond.renal_severe")],
                region_ids: vec![id(&format!("{p}region.a.cq1.exc"))],
            }],
            source_segment_ids: vec![id(&format!("{p}seg.a.cq1.rec1"))],
        }
    }

    pub(crate) fn binding_p(p: &str) -> TerminologyBinding {
        TerminologyBinding {
            binding_id: id(&format!("{p}bind.a.m1")),
            system: id("ckc.lex"),
            code: id("cond.sepsis"),
            status: BindingStatus::Exact,
            alternatives: vec![],
            region_ids: vec![id(&format!("{p}region.a.cq1.rec"))],
        }
    }

    #[test]
    fn strength_certainty_spellings() {
        let spelled: Vec<&str> = Strength::ALL.iter().map(|&v| v.as_str()).collect();
        assert_eq!(spelled, ["strong", "weak"]);
        let spelled: Vec<&str> = Certainty::ALL.iter().map(|&v| v.as_str()).collect();
        assert_eq!(spelled, ["high", "moderate", "low", "very_low"]);
        for v in Strength::ALL {
            round_trip(*v);
        }
        for v in Certainty::ALL {
            round_trip(*v);
        }
    }

    #[test]
    fn action_derives_key() {
        let action = Action::new(id("act.administer"), id("drug.abx_a"));
        assert_eq!(action.key, id("act.administer:drug.abx_a"));
        assert_eq!(
            canon(&action),
            concat!(
                r#"{"key":"act.administer:drug.abx_a","#,
                r#""kind":"act.administer","target":"drug.abx_a"}"#
            )
        );
        round_trip(action);
    }

    // Pins the three §4.3 atom unions and the interval bound encoding.
    #[test]
    fn context_atom_bytes() {
        assert_eq!(
            canon(&atom_c("cond.sepsis")),
            r#"{"tag":"concept","value":"cond.sepsis"}"#
        );
        assert_eq!(
            canon(&atom_nc("cond.renal_severe")),
            r#"{"tag":"concept_negated","value":"cond.renal_severe"}"#
        );
        assert_eq!(
            canon(&atom_ge("q.age_years", 18)),
            r#"{"tag":"interval","value":{"ge":"18","var":"q.age_years"}}"#
        );
        let interval = QuantityInterval {
            var: id("q.egfr"),
            ge: None,
            gt: Some(-5),
            le: None,
            lt: Some(30),
        };
        assert_eq!(canon(&interval), r#"{"gt":"-5","lt":"30","var":"q.egfr"}"#);
        round_trip(ContextAtom::Interval(interval));
        round_trip(atom_c("cond.sepsis"));
        round_trip(atom_nc("cond.renal_severe"));
        assert!(matches!(
            read_canonical::<ContextAtom>(br#"{"tag":"slot_eq","value":"x"}"#),
            Err(CanonReadError::Policy(_))
        ));
    }

    // Conjuncts and disjuncts are §4.3 sets: construction order is erased,
    // byte-identical duplicates collapse.
    #[test]
    fn context_expr_sets() {
        let shuffled = dnf1(vec![
            atom_ge("q.age_years", 18),
            atom_c("cond.sepsis"),
            atom_nc("cond.renal_severe"),
        ]);
        let sorted = dnf1(vec![
            atom_c("cond.sepsis"),
            atom_nc("cond.renal_severe"),
            atom_ge("q.age_years", 18),
        ]);
        assert_eq!(canon(&shuffled), canon(&sorted));
        round_trip(sorted);
        let dup = dnf1(vec![atom_c("cond.sepsis"), atom_c("cond.sepsis")]);
        assert_eq!(
            canon(&dup),
            r#"{"any":[{"all":[{"tag":"concept","value":"cond.sepsis"}]}]}"#
        );
        let two = ContextExpr {
            any: vec![
                ContextConjunct {
                    all: vec![atom_c("cond.b")],
                },
                ContextConjunct {
                    all: vec![atom_c("cond.a")],
                },
            ],
        };
        let bytes = canon(&two);
        assert!(bytes.find("cond.a").unwrap() < bytes.find("cond.b").unwrap());
    }

    // THE unit pin: the §8.6 NormRule canonical payload, byte for byte.
    #[test]
    fn norm_rule_spec86_bytes() {
        let rule = rule_p("");
        assert_eq!(
            canon(&rule),
            concat!(
                r#"{"action":{"key":"act.administer:drug.abx_a","kind":"act.administer","#,
                r#""target":"drug.abx_a"},"context":{"any":[{"all":["#,
                r#"{"tag":"concept","value":"cond.sepsis"},"#,
                r#"{"tag":"concept_negated","value":"cond.renal_severe"},"#,
                r#"{"tag":"interval","value":{"ge":"18","var":"q.age_years"}}]}]},"#,
                r#""direction":"for","rule_id":"rule.a.cq1.r1","#,
                r#""source_region_ids":["region.a.cq1.rec","region.a.cq1.exc"],"#,
                r#""strength":"strong"}"#
            )
        );
        round_trip(rule);
    }

    // certainty and exception_refs emit when present, vanish when absent
    // (§5: optional at M1); a present-but-empty exception_refs is not writer
    // output and is rejected.
    #[test]
    fn norm_rule_optionals() {
        let mut rule = rule_p("");
        rule.certainty = Some(Certainty::Low);
        rule.exception_refs = vec![id("exc.a.cq1.e1")];
        let bytes = canon(&rule);
        assert!(bytes.contains(r#""certainty":"low","context""#));
        assert!(bytes.contains(r#""exception_refs":["exc.a.cq1.e1"],"rule_id""#));
        round_trip(rule);
        let doctored =
            canon(&rule_p("")).replace(r#""rule_id""#, r#""exception_refs":[],"rule_id""#);
        assert!(matches!(
            read_canonical::<NormRule>(doctored.as_bytes()),
            Err(CanonReadError::UnknownField(f)) if f == "exception_refs"
        ));
    }

    #[test]
    fn terminology_binding_bytes() {
        let binding = binding_p("");
        assert_eq!(
            canon(&binding),
            concat!(
                r#"{"alternatives":[],"binding_id":"bind.a.m1","code":"cond.sepsis","#,
                r#""region_ids":["region.a.cq1.rec"],"status":"exact","system":"ckc.lex"}"#
            )
        );
        round_trip(binding);
        let ambiguous = TerminologyBinding {
            binding_id: id("bind.a.m2"),
            system: id("ckc.lex"),
            code: id("cond.flu"),
            status: BindingStatus::Ambiguous,
            alternatives: vec![id("cond.z"), id("cond.a")],
            region_ids: vec![id("r.x")],
        };
        assert!(canon(&ambiguous).contains(r#""alternatives":["cond.a","cond.z"]"#));
    }

    #[test]
    fn clinical_statement_bytes() {
        let statement = statement_p("");
        assert_eq!(
            canon(&statement),
            concat!(
                r#"{"action":{"key":"act.administer:drug.abx_a","kind":"act.administer","#,
                r#""target":"drug.abx_a"},"certainty":"moderate","#,
                r#""condition":[{"tag":"concept","value":"cond.sepsis"}],"#,
                r#""exceptions":[{"atoms":[{"tag":"concept","value":"cond.renal_severe"}],"#,
                r#""exception_id":"exc.a.cq1.e1","region_ids":["region.a.cq1.exc"]}],"#,
                r#""modality":"for","#,
                r#""population":[{"tag":"interval","value":{"ge":"18","var":"q.age_years"}}],"#,
                r#""source_segment_ids":["seg.a.cq1.rec1"],"statement_id":"st.a.cq1.s1","#,
                r#""strength":"strong"}"#
            )
        );
        round_trip(statement);
        let mut bare = statement_p("");
        bare.certainty = None;
        assert!(!canon(&bare).contains("certainty"));
        round_trip(bare);
    }

    #[test]
    fn ir2_layers_round_trip() {
        let clinical = ClinicalIr {
            bindings: vec![binding_p("")],
            statements: vec![statement_p("")],
        };
        assert!(canon(&clinical).starts_with(r#"{"bindings":[{"alternatives""#));
        round_trip(clinical);
        let norm = NormIr {
            rules: vec![rule_p("")],
        };
        assert!(canon(&norm).starts_with(r#"{"rules":[{"action""#));
        round_trip(norm);
    }

    // Structural bytes: local ids localize in canonical field order,
    // vocabulary (concepts, action, interval vars, system/code) stays
    // verbatim, components open fresh scopes.
    #[test]
    fn ir2_structural_bytes_pin() {
        assert_eq!(
            structural(&rule_p("")),
            concat!(
                r#"{"action":{"key":"act.administer:drug.abx_a","kind":"act.administer","#,
                r#""target":"drug.abx_a"},"context":{"any":[{"all":["#,
                r#"{"tag":"concept","value":"cond.sepsis"},"#,
                r#"{"tag":"concept_negated","value":"cond.renal_severe"},"#,
                r#"{"tag":"interval","value":{"ge":"18","var":"q.age_years"}}]}]},"#,
                r#""direction":"for","rule_id":"i0","source_region_ids":["i1","i2"],"#,
                r#""strength":"strong"}"#
            )
        );
        assert_eq!(
            structural(&statement_p("")),
            concat!(
                r#"{"action":{"key":"act.administer:drug.abx_a","kind":"act.administer","#,
                r#""target":"drug.abx_a"},"certainty":"moderate","#,
                r#""condition":[{"tag":"concept","value":"cond.sepsis"}],"#,
                r#""exceptions":[{"atoms":[{"tag":"concept","value":"cond.renal_severe"}],"#,
                r#""exception_id":"i0","region_ids":["i1"]}],"#,
                r#""modality":"for","#,
                r#""population":[{"tag":"interval","value":{"ge":"18","var":"q.age_years"}}],"#,
                r#""source_segment_ids":["i2"],"statement_id":"i3","strength":"strong"}"#
            )
        );
        assert_eq!(
            structural(&binding_p("")),
            concat!(
                r#"{"alternatives":[],"binding_id":"i0","code":"cond.sepsis","#,
                r#""region_ids":["i1"],"status":"exact","system":"ckc.lex"}"#
            )
        );
        // sibling components restart at i0: per-component scopes
        let mut second = rule_p("");
        second.rule_id = id("rule.a.cq1.r2");
        let norm = NormIr {
            rules: vec![rule_p(""), second],
        };
        assert_eq!(structural(&norm).matches(r#""rule_id":"i0""#).count(), 2);
    }

    // §4.3: renaming every document-local id moves the content hash, never
    // the structural hash; swapping lexicon vocabulary moves both.
    #[test]
    fn ir2_structural_hash_rename_stable() {
        let mut rule = rule_p("");
        rule.exception_refs = vec![id("exc.a.cq1.e1")];
        let mut renamed_rule = rule_p("x.");
        renamed_rule.exception_refs = vec![id("x.exc.a.cq1.e1")];
        let norm = NormIr { rules: vec![rule] };
        let renamed_norm = NormIr {
            rules: vec![renamed_rule],
        };
        assert_eq!(
            structural_hash(&norm).unwrap(),
            structural_hash(&renamed_norm).unwrap()
        );
        assert_ne!(
            content_hash(&norm).unwrap(),
            content_hash(&renamed_norm).unwrap()
        );

        let clinical = ClinicalIr {
            bindings: vec![binding_p("")],
            statements: vec![statement_p("")],
        };
        let renamed_clinical = ClinicalIr {
            bindings: vec![binding_p("x.")],
            statements: vec![statement_p("x.")],
        };
        assert_eq!(
            structural_hash(&clinical).unwrap(),
            structural_hash(&renamed_clinical).unwrap()
        );
        assert_ne!(
            content_hash(&clinical).unwrap(),
            content_hash(&renamed_clinical).unwrap()
        );

        // vocabulary is structure: a concept swap moves the structural hash
        let mut other_concept = rule_p("");
        other_concept.context = dnf1(vec![
            atom_c("cond.pneumonia"),
            atom_nc("cond.renal_severe"),
            atom_ge("q.age_years", 18),
        ]);
        assert_ne!(
            structural_hash(&rule_p("")).unwrap(),
            structural_hash(&other_concept).unwrap()
        );
    }

    #[test]
    fn ir2_structural_hash_sees_structure() {
        let base = rule_p("");
        let mut atom_dropped = rule_p("");
        atom_dropped.context = dnf1(vec![atom_c("cond.sepsis"), atom_ge("q.age_years", 18)]);
        let mut flipped = rule_p("");
        flipped.direction = Direction::Against;
        let base_hash = structural_hash(&base).unwrap();
        for other in [&atom_dropped, &flipped] {
            assert_ne!(base_hash, structural_hash(other).unwrap());
        }
        let one = NormIr {
            rules: vec![rule_p("")],
        };
        let doubled = NormIr {
            rules: vec![rule_p(""), rule_p("")],
        };
        assert_ne!(
            structural_hash(&one).unwrap(),
            structural_hash(&doubled).unwrap()
        );
    }

    // ---- core-ir.3: FormalIR ------------------------------------------------

    fn constraint_p(p: &str) -> FormalConstraint {
        FormalConstraint::from_rule(&rule_p(p))
    }

    /// A §8.6-style plan slot for the worked pair, local ids under prefix
    /// `p` (constraint refs follow the `fc.<rule_id>` derivation).
    pub(crate) fn pair_p(p: &str) -> ContradictionQueryPair {
        ContradictionQueryPair {
            pair_id: id(&format!("{p}q.m1_conflict.pair1")),
            action_key: id("act.administer:drug.abx_a"),
            constraint_a_id: id(&format!("fc.{p}rule.a.cq1.r1")),
            constraint_b_id: id(&format!("fc.{p}rule.b.contra1")),
            context_overlap_query_id: id(&format!("{p}q.m1_conflict.pair1.overlap")),
            deontic_consistency_query_id: id(&format!("{p}q.m1_conflict.pair1.deontic")),
        }
    }

    // §6: opposed iff one direction is positive (for/require/permit) and
    // the other is against/avoid or contraindicate/avoid — all 36 cells.
    #[test]
    fn directions_opposed_truth_table() {
        use Direction::*;
        let positive = [For, Require, Permit];
        let opposing = [Against, Avoid, Contraindicate];
        for &a in Direction::ALL {
            for &b in Direction::ALL {
                let want = (positive.contains(&a) && opposing.contains(&b))
                    || (positive.contains(&b) && opposing.contains(&a));
                assert_eq!(directions_opposed(a, b), want, "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn formal_constraint_from_rule_projects() {
        let rule = rule_p("");
        let fc = FormalConstraint::from_rule(&rule);
        assert_eq!(fc.constraint_id, id("fc.rule.a.cq1.r1"));
        assert_eq!(fc.rule_id, rule.rule_id);
        assert_eq!(fc.action, rule.action);
        assert_eq!(fc.context, rule.context);
        assert_eq!(fc.direction, rule.direction);
        assert_eq!(fc.strength, rule.strength);
        assert_eq!(fc.certainty, None);
        // certainty rides along when present; the context copies verbatim —
        // exceptions were already folded into it at normalization (§5)
        let mut with_extras = rule_p("");
        with_extras.certainty = Some(Certainty::Low);
        with_extras.exception_refs = vec![id("exc.a.cq1.e1")];
        let fc = FormalConstraint::from_rule(&with_extras);
        assert_eq!(fc.certainty, Some(Certainty::Low));
        assert_eq!(fc.context, with_extras.context);
    }

    // Canonical byte pins: the §8.6-derived constraint and the worked-pair
    // plan slot; certainty vanishes when absent.
    #[test]
    fn ir3_canonical_bytes_pin() {
        let fc = constraint_p("");
        assert_eq!(
            canon(&fc),
            concat!(
                r#"{"action":{"key":"act.administer:drug.abx_a","kind":"act.administer","#,
                r#""target":"drug.abx_a"},"constraint_id":"fc.rule.a.cq1.r1","#,
                r#""context":{"any":[{"all":["#,
                r#"{"tag":"concept","value":"cond.sepsis"},"#,
                r#"{"tag":"concept_negated","value":"cond.renal_severe"},"#,
                r#"{"tag":"interval","value":{"ge":"18","var":"q.age_years"}}]}]},"#,
                r#""direction":"for","rule_id":"rule.a.cq1.r1","strength":"strong"}"#
            )
        );
        round_trip(fc);
        let mut with_certainty = rule_p("");
        with_certainty.certainty = Some(Certainty::Moderate);
        let fc = FormalConstraint::from_rule(&with_certainty);
        assert!(canon(&fc).contains(r#""certainty":"moderate","constraint_id""#));
        round_trip(fc);

        let pair = pair_p("");
        assert_eq!(
            canon(&pair),
            concat!(
                r#"{"action_key":"act.administer:drug.abx_a","#,
                r#""constraint_a_id":"fc.rule.a.cq1.r1","#,
                r#""constraint_b_id":"fc.rule.b.contra1","#,
                r#""context_overlap_query_id":"q.m1_conflict.pair1.overlap","#,
                r#""deontic_consistency_query_id":"q.m1_conflict.pair1.deontic","#,
                r#""pair_id":"q.m1_conflict.pair1"}"#
            )
        );
        round_trip(pair);
    }

    #[test]
    fn formal_ir_derive_and_round_trip() {
        let mut second = rule_p("");
        second.rule_id = id("rule.a.cq1.r2");
        let norm = NormIr {
            rules: vec![rule_p(""), second.clone()],
        };
        let formal = FormalIr::derive(&norm);
        assert_eq!(
            formal.constraints,
            vec![
                FormalConstraint::from_rule(&norm.rules[0]),
                FormalConstraint::from_rule(&second)
            ]
        );
        assert_eq!(formal.plan, []);
        let bytes = canon(&formal);
        assert!(bytes.starts_with(r#"{"constraints":[{"action""#));
        assert!(bytes.ends_with(r#""plan":[]}"#));
        round_trip(formal);
        round_trip(FormalIr {
            constraints: vec![constraint_p("")],
            plan: vec![pair_p("")],
        });
    }

    // Structural bytes: constraints fresh-scope (constraint_id i0, rule_id
    // i1, action/context/enums verbatim); plan ids localize i0..i4 in the
    // layer scope with action_key verbatim.
    #[test]
    fn ir3_structural_bytes_pin() {
        let formal = FormalIr {
            constraints: vec![constraint_p("")],
            plan: vec![pair_p("")],
        };
        assert_eq!(
            structural(&formal),
            concat!(
                r#"{"constraints":[{"action":{"key":"act.administer:drug.abx_a","#,
                r#""kind":"act.administer","target":"drug.abx_a"},"constraint_id":"i0","#,
                r#""context":{"any":[{"all":["#,
                r#"{"tag":"concept","value":"cond.sepsis"},"#,
                r#"{"tag":"concept_negated","value":"cond.renal_severe"},"#,
                r#"{"tag":"interval","value":{"ge":"18","var":"q.age_years"}}]}]},"#,
                r#""direction":"for","rule_id":"i1","strength":"strong"}],"#,
                r#""plan":[{"action_key":"act.administer:drug.abx_a","#,
                r#""constraint_a_id":"i0","constraint_b_id":"i1","#,
                r#""context_overlap_query_id":"i2","deontic_consistency_query_id":"i3","#,
                r#""pair_id":"i4"}]}"#
            )
        );
        // sibling constraints restart at i0: per-component scopes
        let mut second = rule_p("");
        second.rule_id = id("rule.a.cq1.r2");
        let two = FormalIr {
            constraints: vec![constraint_p(""), FormalConstraint::from_rule(&second)],
            plan: vec![],
        };
        assert_eq!(
            structural(&two).matches(r#""constraint_id":"i0""#).count(),
            2
        );
    }

    // §4.3: renaming document-local ids (rule ids, plan/query ids) keeps the
    // structural hash and moves the content hash; vocabulary swaps,
    // co-reference collapses, direction flips, and multiplicity move the
    // structural hash.
    #[test]
    fn ir3_structural_hash_rename_stable() {
        let formal = FormalIr {
            constraints: vec![constraint_p("")],
            plan: vec![pair_p("")],
        };
        let renamed = FormalIr {
            constraints: vec![constraint_p("x.")],
            plan: vec![pair_p("x.")],
        };
        assert_eq!(
            structural_hash(&formal).unwrap(),
            structural_hash(&renamed).unwrap()
        );
        assert_ne!(
            content_hash(&formal).unwrap(),
            content_hash(&renamed).unwrap()
        );

        // action_key is vocabulary: swapping it moves the structural hash
        let mut other_action = pair_p("");
        other_action.action_key = id("act.administer:drug.abx_b");
        let swapped = FormalIr {
            constraints: vec![constraint_p("")],
            plan: vec![other_action],
        };
        assert_ne!(
            structural_hash(&formal).unwrap(),
            structural_hash(&swapped).unwrap()
        );
        // collapsing the pair's constraint refs changes the co-reference
        // pattern (i0,i0 vs i0,i1)
        let mut collapsed = pair_p("");
        collapsed.constraint_b_id = collapsed.constraint_a_id.clone();
        let collapsed = FormalIr {
            constraints: vec![constraint_p("")],
            plan: vec![collapsed],
        };
        assert_ne!(
            structural_hash(&formal).unwrap(),
            structural_hash(&collapsed).unwrap()
        );
        // direction flips and constraint multiplicity move it too
        let mut flipped_rule = rule_p("");
        flipped_rule.direction = Direction::Against;
        let flipped = FormalIr {
            constraints: vec![FormalConstraint::from_rule(&flipped_rule)],
            plan: vec![],
        };
        let derived = FormalIr::derive(&NormIr {
            rules: vec![rule_p("")],
        });
        let doubled = FormalIr {
            constraints: vec![constraint_p(""), constraint_p("")],
            plan: vec![],
        };
        assert_ne!(
            structural_hash(&derived).unwrap(),
            structural_hash(&flipped).unwrap()
        );
        assert_ne!(
            structural_hash(&derived).unwrap(),
            structural_hash(&doubled).unwrap()
        );
    }
}
