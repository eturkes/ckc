//! SPEC §5 IR layers, first pair: DocIR ([`DocIr`]) — the layout-preserving
//! text/table view over [`SourceGraph`] refs with extraction diagnostics —
//! and SegmentIR ([`SegmentIr`]) — the document's [`ClinicalSegment`]s.
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

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::canon::{
    CanonError, CanonRead, CanonReadError, Canonical, ObjectEmitter, ObjectReader, Reader,
    emit_array, emit_set, emit_string, emit_u64, read_array, read_set, read_string, read_u64,
};
use crate::enums::{DiagnosticCode, DiagnosticRecord, emit_payload, fieldless_enum};
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
/// which V1 component shapes satisfy.
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

/// Emit an ordered run of structural values inside the enclosing scope
/// (arrays whose order is semantic share their component's scope).
fn emit_structural_array<T: Structural>(
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
fn emit_structural_components<T: Structural>(
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
fn emit_structural_record_set<T: Structural>(
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
    /// SPEC §5 clinical segment kind — the seven V1 kinds (spec prose
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
    /// (bundle validation, core-ir.3); `table_structure_uncertain` residuals
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
/// in §4.5 regions (set semantics; nonemptiness lands with bundle validation,
/// core-ir.3). An independently hashable component (§5 component records).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon::{canonical_payload_bytes, read_canonical};
    use crate::enums::Outcome;
    use crate::grounding::{DataClass, Provenance, SourceDocument, SourceRegion, SourceSpan};
    use crate::hash::content_hash;

    fn canon<T: Canonical>(value: &T) -> String {
        String::from_utf8(canonical_payload_bytes(value).unwrap()).unwrap()
    }

    fn structural<T: Structural>(value: &T) -> String {
        let mut out = Vec::new();
        value
            .emit_structural(&mut out, &mut RefLocalizer::new())
            .unwrap();
        String::from_utf8(out).unwrap()
    }

    fn round_trip<T: Canonical + CanonRead + std::fmt::Debug + PartialEq>(value: T) {
        let bytes = canonical_payload_bytes(&value).unwrap();
        let got: T = read_canonical(&bytes).unwrap();
        assert_eq!(got, value, "round trip changed the value");
    }

    fn id(s: &str) -> Id {
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

    fn diag(code: DiagnosticCode, region: &str) -> DiagnosticRecord {
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
}
