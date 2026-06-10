//! SPEC §4.5 source grounding: the typed layer every semantic claim resolves
//! back to.
//!
//! [`SourceGraph`] is the one-per-document artifact extract emits: a finite
//! node tree plus the [`SourceSpan`]s, [`SourceAnchor`]s, and [`SourceRegion`]s
//! that make document text addressable. [`validate`](SourceGraph::validate)
//! enforces the §4.5 invariants that are mechanical at this layer: references
//! resolve, spans agree with their derived texts and hashes, and every textual
//! node is spanned or named by an `extraction_uncertain` residual. The
//! claim-side half of §4.5 (`source_region_ids` / `synthetic_fixture_id` on
//! semantic claims) lands with the IR layers; envelope-wrapping the graph as
//! `source_graph.json` is the extract stage's job.
//!
//! Offsets are UTF-8 byte offsets, half-open `[start, end)`: a span addresses
//! its parent node's extracted text, an anchor addresses its parent span's
//! `raw_text`.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::canon::{
    CanonError, CanonRead, CanonReadError, Canonical, ObjectEmitter, ObjectReader, Reader,
    emit_array, emit_raw_map, emit_set, emit_string, emit_string_policy, emit_u64, read_array,
    read_raw_map, read_set, read_string, read_string_policy, read_u64,
};
use crate::enums::fieldless_enum;
use crate::hash::hash_bytes;
use crate::id::{Hash, Id, ValidationError};
use crate::strings::StringPolicy;

fieldless_enum! {
    /// SPEC §4.5 node kind: the closed set of structural units a
    /// [`SourceGraph`] may contain.
    NodeKind {
        Document => "document",
        Section => "section",
        Paragraph => "paragraph",
        List => "list",
        Table => "table",
        Cell => "cell",
        Caption => "caption",
        Footnote => "footnote",
        /// Clinical question.
        Cq => "cq",
        Recommendation => "recommendation",
    }
}

impl NodeKind {
    /// Kinds that always bear extracted text. The §4.5 coverage invariant
    /// binds here: a textual node carries at least one [`SourceSpan`] or is
    /// named by a typed `extraction_uncertain` residual. The structural kinds
    /// (`document`, `section`, `list`, `table`) hold any text they own —
    /// e.g. a section heading — through spans attached directly to them, which
    /// stays the producer's contract (§8.3).
    pub fn is_textual(self) -> bool {
        matches!(
            self,
            NodeKind::Paragraph
                | NodeKind::Cell
                | NodeKind::Caption
                | NodeKind::Footnote
                | NodeKind::Cq
                | NodeKind::Recommendation
        )
    }
}

fieldless_enum! {
    /// SPEC §4.5 document provenance.
    Provenance {
        Synthetic => "synthetic",
        Public => "public",
    }
}

fieldless_enum! {
    /// SPEC §4.5 `data_class`: `none` is the default and the only V1 value;
    /// §15 source-permission gates add variants as real corpora land.
    DataClass {
        None => "none",
    }
}

fieldless_enum! {
    /// SPEC §4.5 anchor kind: what a [`SourceAnchor`] marks inside its span.
    AnchorKind {
        Mention => "mention",
        Quantity => "quantity",
        Modality => "modality",
        Negation => "negation",
        TemporalCue => "temporal_cue",
        TableValue => "table_value",
    }
}

/// SPEC §4.5 document identity. Both hashes are §4.4 `_hash` fields declared
/// over raw bytes (a source document is an external file, not an accepted
/// artifact): `raw_hash` covers the bytes as acquired, `content_hash` the
/// decoded content extract consumed — equal when no transport or charset
/// decoding applies, as with the V1 fixtures. Distinct from the envelope's
/// canonical-payload `content_hash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDocument {
    pub document_id: Id,
    /// Open vocabulary (e.g. `synthetic_fixture_html`); §15 gates new families.
    pub source_family: Id,
    pub provenance: Provenance,
    pub raw_hash: Hash,
    pub content_hash: Hash,
    pub data_class: DataClass,
}

impl Canonical for SourceDocument {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("content_hash", |b| self.content_hash.emit_canonical(b))?;
        obj.member("data_class", |b| self.data_class.emit_canonical(b))?;
        obj.member("document_id", |b| self.document_id.emit_canonical(b))?;
        obj.member("provenance", |b| self.provenance.emit_canonical(b))?;
        obj.member("raw_hash", |b| self.raw_hash.emit_canonical(b))?;
        obj.member("source_family", |b| self.source_family.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for SourceDocument {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let content_hash = obj.member("content_hash", Hash::read)?;
        let data_class = obj.member("data_class", DataClass::read)?;
        let document_id = obj.member("document_id", Id::read)?;
        let provenance = obj.member("provenance", Provenance::read)?;
        let raw_hash = obj.member("raw_hash", Hash::read)?;
        let source_family = obj.member("source_family", Id::read)?;
        obj.close()?;
        Ok(SourceDocument {
            document_id,
            source_family,
            provenance,
            raw_hash,
            content_hash,
            data_class,
        })
    }
}

/// One node of the §4.5 graph. Array position in [`SourceGraph::nodes`] is
/// document order; the unique `document`-kind root comes first and every other
/// node's parent precedes it (which keeps the tree acyclic by construction).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNode {
    pub node_id: Id,
    pub kind: NodeKind,
    /// `None` only on the root document node.
    pub parent_id: Option<Id>,
    /// Extractor-emitted structure the kind list cannot carry, as identifier
    /// keys to raw-text values — e.g. `row`/`col`/`header` on a `cell` node
    /// preserve the §8.2 table relations. Map semantics; the producing stage
    /// owns the vocabulary.
    pub attrs: Vec<(Id, String)>,
}

impl Canonical for SourceNode {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("attrs", |b| emit_raw_map(b, &self.attrs))?;
        obj.member("kind", |b| self.kind.emit_canonical(b))?;
        obj.member("node_id", |b| self.node_id.emit_canonical(b))?;
        obj.optional("parent_id", self.parent_id.as_ref(), |b, p| {
            p.emit_canonical(b)
        })?;
        obj.finish(out)
    }
}

impl CanonRead for SourceNode {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let attrs = obj.member("attrs", read_raw_map)?;
        let kind = obj.member("kind", NodeKind::read)?;
        let node_id = obj.member("node_id", Id::read)?;
        let parent_id = obj.optional("parent_id", Id::read)?;
        obj.close()?;
        Ok(SourceNode {
            node_id,
            kind,
            parent_id,
            attrs,
        })
    }
}

/// SPEC §4.5 stable text span: the addressable unit of document text.
/// `nfkc_text`, `search_text`, `text_hash`, and the offset width are pure
/// derivations of `raw_text` ([`derive`](Self::derive) computes them;
/// [`SourceGraph::validate`] re-checks them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub span_id: Id,
    /// The node whose extracted text this span addresses.
    pub node_id: Id,
    /// UTF-8 byte offsets into the node's extracted text, half-open;
    /// `end - start` equals `raw_text`'s byte length.
    pub start: u64,
    pub end: u64,
    /// The addressed bytes exactly (§4.2 `raw_source`).
    pub raw_text: String,
    /// `raw_text` under §4.2 `source_nfkc`.
    pub nfkc_text: String,
    /// `raw_text` under §4.2 `semantic_ja`, the V1 search normal form.
    pub search_text: String,
    /// Position in the document's total reading order; strictly increasing
    /// along [`SourceGraph::spans`].
    pub reading_order: u64,
    /// §4.4 `_hash` over `raw_text`'s raw bytes.
    pub text_hash: Hash,
}

impl SourceSpan {
    /// Build a span from the extractor-known fields, computing every derived
    /// field: `end` from `raw_text`'s byte length, the two normalized texts,
    /// and `text_hash`.
    pub fn derive(
        span_id: Id,
        node_id: Id,
        start: u64,
        raw_text: String,
        reading_order: u64,
    ) -> SourceSpan {
        let nfkc_text = StringPolicy::SourceNfkc
            .normalize(&raw_text)
            .expect("source_nfkc is infallible");
        let search_text = StringPolicy::SemanticJa
            .normalize(&raw_text)
            .expect("semantic_ja is infallible");
        let text_hash = hash_bytes(raw_text.as_bytes());
        let end = start + raw_text.len() as u64;
        SourceSpan {
            span_id,
            node_id,
            start,
            end,
            raw_text,
            nfkc_text,
            search_text,
            reading_order,
            text_hash,
        }
    }

    /// Check every derived field against its derivation from `raw_text`.
    fn check_derived(&self) -> Result<(), GroundingError> {
        let len = self.raw_text.len() as u64;
        if self.start >= self.end || self.end - self.start != len {
            return Err(GroundingError::SpanOffsets {
                span_id: self.span_id.clone(),
                start: self.start,
                end: self.end,
                len,
            });
        }
        let nfkc = StringPolicy::SourceNfkc
            .normalize(&self.raw_text)
            .expect("source_nfkc is infallible");
        if self.nfkc_text != nfkc {
            return Err(GroundingError::SpanDerived {
                span_id: self.span_id.clone(),
                field: "nfkc_text",
            });
        }
        let search = StringPolicy::SemanticJa
            .normalize(&self.raw_text)
            .expect("semantic_ja is infallible");
        if self.search_text != search {
            return Err(GroundingError::SpanDerived {
                span_id: self.span_id.clone(),
                field: "search_text",
            });
        }
        if self.text_hash != hash_bytes(self.raw_text.as_bytes()) {
            return Err(GroundingError::SpanDerived {
                span_id: self.span_id.clone(),
                field: "text_hash",
            });
        }
        Ok(())
    }
}

impl Canonical for SourceSpan {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("end", |b| {
            emit_u64(b, self.end);
            Ok(())
        })?;
        obj.member("nfkc_text", |b| {
            emit_string_policy(b, StringPolicy::SourceNfkc, &self.nfkc_text)
        })?;
        obj.member("node_id", |b| self.node_id.emit_canonical(b))?;
        obj.member("raw_text", |b| {
            emit_string_policy(b, StringPolicy::RawSource, &self.raw_text)
        })?;
        obj.member("reading_order", |b| {
            emit_u64(b, self.reading_order);
            Ok(())
        })?;
        obj.member("search_text", |b| {
            emit_string_policy(b, StringPolicy::SemanticJa, &self.search_text)
        })?;
        obj.member("span_id", |b| self.span_id.emit_canonical(b))?;
        obj.member("start", |b| {
            emit_u64(b, self.start);
            Ok(())
        })?;
        obj.member("text_hash", |b| self.text_hash.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for SourceSpan {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let end = obj.member("end", read_u64)?;
        let nfkc_text = obj.member("nfkc_text", |r| {
            read_string_policy(r, StringPolicy::SourceNfkc)
        })?;
        let node_id = obj.member("node_id", Id::read)?;
        let raw_text = obj.member("raw_text", |r| {
            read_string_policy(r, StringPolicy::RawSource)
        })?;
        let reading_order = obj.member("reading_order", read_u64)?;
        let search_text = obj.member("search_text", |r| {
            read_string_policy(r, StringPolicy::SemanticJa)
        })?;
        let span_id = obj.member("span_id", Id::read)?;
        let start = obj.member("start", read_u64)?;
        let text_hash = obj.member("text_hash", Hash::read)?;
        obj.close()?;
        Ok(SourceSpan {
            span_id,
            node_id,
            start,
            end,
            raw_text,
            nfkc_text,
            search_text,
            reading_order,
            text_hash,
        })
    }
}

/// SPEC §4.5 subspan anchor: marks a mention, quantity, modality, negation,
/// temporal cue, or table value inside one [`SourceSpan`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAnchor {
    pub anchor_id: Id,
    pub span_id: Id,
    pub kind: AnchorKind,
    /// UTF-8 byte offsets into the parent span's `raw_text`, half-open,
    /// nonempty, on character boundaries.
    pub start: u64,
    pub end: u64,
}

impl SourceAnchor {
    /// The anchored slice of `span`'s `raw_text`; `None` when the offsets fall
    /// out of range or off character boundaries (valid graphs never do).
    pub fn text_in<'a>(&self, span: &'a SourceSpan) -> Option<&'a str> {
        let start = usize::try_from(self.start).ok()?;
        let end = usize::try_from(self.end).ok()?;
        span.raw_text.get(start..end)
    }
}

impl Canonical for SourceAnchor {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("anchor_id", |b| self.anchor_id.emit_canonical(b))?;
        obj.member("end", |b| {
            emit_u64(b, self.end);
            Ok(())
        })?;
        obj.member("kind", |b| self.kind.emit_canonical(b))?;
        obj.member("span_id", |b| self.span_id.emit_canonical(b))?;
        obj.member("start", |b| {
            emit_u64(b, self.start);
            Ok(())
        })?;
        obj.finish(out)
    }
}

impl CanonRead for SourceAnchor {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let anchor_id = obj.member("anchor_id", Id::read)?;
        let end = obj.member("end", read_u64)?;
        let kind = obj.member("kind", AnchorKind::read)?;
        let span_id = obj.member("span_id", Id::read)?;
        let start = obj.member("start", read_u64)?;
        obj.close()?;
        Ok(SourceAnchor {
            anchor_id,
            span_id,
            kind,
            start,
            end,
        })
    }
}

/// SPEC §4.5 region: the unit of evidence — a closed support set over nodes
/// (cells included), spans, and anchors. Closure — naming everything the
/// supported claim relies on — is the producer's contract; this layer enforces
/// that the set is nonempty and every named ref resolves, which is what makes
/// region ids quotable in reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRegion {
    pub region_id: Id,
    /// Set semantics, as are the two below.
    pub node_ids: Vec<Id>,
    pub span_ids: Vec<Id>,
    pub anchor_ids: Vec<Id>,
}

impl Canonical for SourceRegion {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("anchor_ids", |b| emit_set(b, &self.anchor_ids))?;
        obj.member("node_ids", |b| emit_set(b, &self.node_ids))?;
        obj.member("region_id", |b| self.region_id.emit_canonical(b))?;
        obj.member("span_ids", |b| emit_set(b, &self.span_ids))?;
        obj.finish(out)
    }
}

impl CanonRead for SourceRegion {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let anchor_ids = obj.member("anchor_ids", read_set::<Id>)?;
        let node_ids = obj.member("node_ids", read_set::<Id>)?;
        let region_id = obj.member("region_id", Id::read)?;
        let span_ids = obj.member("span_ids", read_set::<Id>)?;
        obj.close()?;
        Ok(SourceRegion {
            region_id,
            node_ids,
            span_ids,
            anchor_ids,
        })
    }
}

/// Which id pool a [`GroundingError`] reference names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Node,
    Span,
    Anchor,
    Region,
}

impl RefKind {
    /// Crate-visible so sibling error types (ir) name pools the same way.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            RefKind::Node => "node",
            RefKind::Span => "span",
            RefKind::Anchor => "anchor",
            RefKind::Region => "region",
        }
    }
}

/// A SPEC §4.5 grounding invariant failed ([`SourceGraph::validate`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundingError {
    /// The node array's root shape is wrong (carries the violated rule).
    Root(&'static str),
    /// A second `document`-kind node appeared.
    ExtraDocumentNode(Id),
    /// A non-root node carries no parent.
    MissingParent(Id),
    /// Two entities in one pool share an id.
    Duplicate { kind: RefKind, id: Id },
    /// `from` references `id`, which is not defined — for node parents, not
    /// defined earlier in document order (which also rejects cycles).
    Dangling { kind: RefKind, id: Id, from: Id },
    /// `reading_order` fails to increase strictly along the span array.
    ReadingOrder { span_id: Id },
    /// Span offsets are empty, reversed, or disagree with `raw_text`'s byte
    /// length.
    SpanOffsets {
        span_id: Id,
        start: u64,
        end: u64,
        len: u64,
    },
    /// A derived span field does not match its derivation from `raw_text`.
    SpanDerived { span_id: Id, field: &'static str },
    /// Anchor offsets are empty, out of range, or off character boundaries.
    AnchorOffsets { anchor_id: Id, start: u64, end: u64 },
    /// A region supports nothing: every ref list is empty.
    EmptyRegion(Id),
    /// A textual node has no span and no `extraction_uncertain` residual.
    UnspannedTextualNode(Id),
}

impl fmt::Display for GroundingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroundingError::Root(rule) => write!(f, "node tree: {rule}"),
            GroundingError::ExtraDocumentNode(id) => write!(f, "second document node {id}"),
            GroundingError::MissingParent(id) => write!(f, "non-root node {id} carries no parent"),
            GroundingError::Duplicate { kind, id } => {
                write!(f, "duplicate {} id {id}", kind.as_str())
            }
            GroundingError::Dangling { kind, id, from } => {
                write!(f, "{from} references undefined {} {id}", kind.as_str())
            }
            GroundingError::ReadingOrder { span_id } => {
                write!(f, "span {span_id} breaks strictly increasing reading_order")
            }
            GroundingError::SpanOffsets {
                span_id,
                start,
                end,
                len,
            } => write!(
                f,
                "span {span_id} offsets [{start},{end}) disagree with raw_text byte length {len}"
            ),
            GroundingError::SpanDerived { span_id, field } => {
                write!(
                    f,
                    "span {span_id} field {field} does not derive from raw_text"
                )
            }
            GroundingError::AnchorOffsets {
                anchor_id,
                start,
                end,
            } => write!(
                f,
                "anchor {anchor_id} offsets [{start},{end}) fall outside its span's raw_text or off character boundaries"
            ),
            GroundingError::EmptyRegion(id) => write!(f, "region {id} supports nothing"),
            GroundingError::UnspannedTextualNode(id) => write!(
                f,
                "textual node {id} has no span and no extraction_uncertain residual"
            ),
        }
    }
}

impl std::error::Error for GroundingError {}

/// SPEC §4.5 source graph: one artifact per document, emitted by extract.
/// `nodes` and `spans` are arrays whose order is semantic (document order,
/// reading order); `anchors` and `regions` are sets, so input order never
/// reaches the canonical bytes — identical source bytes and extraction config
/// give identical graph bytes provided the producer derives ids and order
/// deterministically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceGraph {
    pub document: SourceDocument,
    pub nodes: Vec<SourceNode>,
    pub spans: Vec<SourceSpan>,
    pub anchors: Vec<SourceAnchor>,
    pub regions: Vec<SourceRegion>,
}

impl SourceGraph {
    /// Enforce the §4.5 invariants that are mechanical at this layer.
    /// `residual_node_ids` names the nodes the producer covered with typed
    /// `extraction_uncertain` residual diagnostics (which live in the
    /// envelope, not the payload): a textual node must carry a span or appear
    /// there. The remaining checks need no outside input: the node array is a
    /// rooted tree in document order, ids are pool-unique, every reference
    /// resolves, span offsets/texts/hashes are self-consistent, reading order
    /// strictly increases, anchors address real subspans, and regions are
    /// nonempty.
    pub fn validate(&self, residual_node_ids: &[Id]) -> Result<(), GroundingError> {
        match self.nodes.first() {
            None => return Err(GroundingError::Root("nodes is empty")),
            Some(root) => {
                if root.kind != NodeKind::Document {
                    return Err(GroundingError::Root("first node must be the document node"));
                }
                if root.parent_id.is_some() {
                    return Err(GroundingError::Root("the document node carries a parent"));
                }
            }
        }
        let mut node_pos: HashMap<&Id, usize> = HashMap::new();
        for (i, node) in self.nodes.iter().enumerate() {
            if node_pos.insert(&node.node_id, i).is_some() {
                return Err(GroundingError::Duplicate {
                    kind: RefKind::Node,
                    id: node.node_id.clone(),
                });
            }
            if i == 0 {
                continue;
            }
            if node.kind == NodeKind::Document {
                return Err(GroundingError::ExtraDocumentNode(node.node_id.clone()));
            }
            let Some(parent) = &node.parent_id else {
                return Err(GroundingError::MissingParent(node.node_id.clone()));
            };
            if node_pos.get(parent).is_none_or(|&p| p >= i) {
                return Err(GroundingError::Dangling {
                    kind: RefKind::Node,
                    id: parent.clone(),
                    from: node.node_id.clone(),
                });
            }
        }

        let mut spans_by_id: HashMap<&Id, &SourceSpan> = HashMap::new();
        let mut spanned_nodes: HashSet<&Id> = HashSet::new();
        let mut prev_order: Option<u64> = None;
        for span in &self.spans {
            if spans_by_id.insert(&span.span_id, span).is_some() {
                return Err(GroundingError::Duplicate {
                    kind: RefKind::Span,
                    id: span.span_id.clone(),
                });
            }
            if !node_pos.contains_key(&span.node_id) {
                return Err(GroundingError::Dangling {
                    kind: RefKind::Node,
                    id: span.node_id.clone(),
                    from: span.span_id.clone(),
                });
            }
            spanned_nodes.insert(&span.node_id);
            if prev_order.is_some_and(|p| span.reading_order <= p) {
                return Err(GroundingError::ReadingOrder {
                    span_id: span.span_id.clone(),
                });
            }
            prev_order = Some(span.reading_order);
            span.check_derived()?;
        }

        let mut anchor_ids: HashSet<&Id> = HashSet::new();
        for anchor in &self.anchors {
            if !anchor_ids.insert(&anchor.anchor_id) {
                return Err(GroundingError::Duplicate {
                    kind: RefKind::Anchor,
                    id: anchor.anchor_id.clone(),
                });
            }
            let Some(span) = spans_by_id.get(&anchor.span_id) else {
                return Err(GroundingError::Dangling {
                    kind: RefKind::Span,
                    id: anchor.span_id.clone(),
                    from: anchor.anchor_id.clone(),
                });
            };
            if anchor.start >= anchor.end || anchor.text_in(span).is_none() {
                return Err(GroundingError::AnchorOffsets {
                    anchor_id: anchor.anchor_id.clone(),
                    start: anchor.start,
                    end: anchor.end,
                });
            }
        }

        let mut region_ids: HashSet<&Id> = HashSet::new();
        for region in &self.regions {
            if !region_ids.insert(&region.region_id) {
                return Err(GroundingError::Duplicate {
                    kind: RefKind::Region,
                    id: region.region_id.clone(),
                });
            }
            if region.node_ids.is_empty()
                && region.span_ids.is_empty()
                && region.anchor_ids.is_empty()
            {
                return Err(GroundingError::EmptyRegion(region.region_id.clone()));
            }
            let dangling = |kind: RefKind, id: &Id| GroundingError::Dangling {
                kind,
                id: id.clone(),
                from: region.region_id.clone(),
            };
            for id in &region.node_ids {
                if !node_pos.contains_key(id) {
                    return Err(dangling(RefKind::Node, id));
                }
            }
            for id in &region.span_ids {
                if !spans_by_id.contains_key(id) {
                    return Err(dangling(RefKind::Span, id));
                }
            }
            for id in &region.anchor_ids {
                if !anchor_ids.contains(id) {
                    return Err(dangling(RefKind::Anchor, id));
                }
            }
        }

        let residuals: HashSet<&Id> = residual_node_ids.iter().collect();
        for node in &self.nodes {
            if node.kind.is_textual()
                && !spanned_nodes.contains(&node.node_id)
                && !residuals.contains(&node.node_id)
            {
                return Err(GroundingError::UnspannedTextualNode(node.node_id.clone()));
            }
        }
        Ok(())
    }
}

impl Canonical for SourceGraph {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("anchors", |b| emit_set(b, &self.anchors))?;
        obj.member("document", |b| self.document.emit_canonical(b))?;
        obj.member("nodes", |b| emit_array(b, &self.nodes))?;
        obj.member("regions", |b| emit_set(b, &self.regions))?;
        obj.member("spans", |b| emit_array(b, &self.spans))?;
        obj.finish(out)
    }
}

impl CanonRead for SourceGraph {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let anchors = obj.member("anchors", read_set::<SourceAnchor>)?;
        let document = obj.member("document", SourceDocument::read)?;
        let nodes = obj.member("nodes", read_array::<SourceNode>)?;
        let regions = obj.member("regions", read_set::<SourceRegion>)?;
        let spans = obj.member("spans", read_array::<SourceSpan>)?;
        obj.close()?;
        Ok(SourceGraph {
            document,
            nodes,
            spans,
            anchors,
            regions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon::{canonical_payload_bytes, read_canonical};
    use crate::hash::content_hash;

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

    /// A valid [`Hash`] built from one repeated hex digit.
    fn h(digit: char) -> Hash {
        Hash::new(format!("sha256:{}", digit.to_string().repeat(64))).unwrap()
    }

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn node(kind: NodeKind, nid: &str, parent: Option<&str>) -> SourceNode {
        SourceNode {
            node_id: id(nid),
            kind,
            parent_id: parent.map(id),
            attrs: vec![],
        }
    }

    fn sample_document() -> SourceDocument {
        SourceDocument {
            document_id: id("doc.a"),
            source_family: id("synthetic_fixture_html"),
            provenance: Provenance::Synthetic,
            raw_hash: h('a'),
            content_hash: h('a'),
            data_class: DataClass::None,
        }
    }

    // Fullwidth digits/letters NFKC-fold to ASCII; 。 folds to '.' only under
    // semantic_ja. The quantity １０ｍｇ sits at raw bytes [12, 24).
    const P1_RAW: &str = "成人には１０ｍｇを投与する。";
    const CQ1_RAW: &str = "ＣＱ１：高血圧の治療";

    fn sample_graph() -> SourceGraph {
        SourceGraph {
            document: sample_document(),
            nodes: vec![
                node(NodeKind::Document, "n.doc", None),
                node(NodeKind::Section, "n.sec", Some("n.doc")),
                node(NodeKind::Cq, "n.cq1", Some("n.sec")),
                node(NodeKind::Paragraph, "n.p1", Some("n.sec")),
            ],
            spans: vec![
                SourceSpan::derive(id("s.cq1"), id("n.cq1"), 0, CQ1_RAW.to_owned(), 1),
                SourceSpan::derive(id("s.p1"), id("n.p1"), 0, P1_RAW.to_owned(), 2),
            ],
            anchors: vec![
                SourceAnchor {
                    anchor_id: id("a.m1"),
                    span_id: id("s.cq1"),
                    kind: AnchorKind::Mention,
                    start: 12,
                    end: 24,
                },
                SourceAnchor {
                    anchor_id: id("a.q1"),
                    span_id: id("s.p1"),
                    kind: AnchorKind::Quantity,
                    start: 12,
                    end: 24,
                },
            ],
            regions: vec![
                SourceRegion {
                    region_id: id("r.cq1"),
                    node_ids: vec![id("n.cq1")],
                    span_ids: vec![id("s.cq1")],
                    anchor_ids: vec![id("a.m1")],
                },
                SourceRegion {
                    region_id: id("r.p1"),
                    node_ids: vec![],
                    span_ids: vec![id("s.p1")],
                    anchor_ids: vec![id("a.q1")],
                },
            ],
        }
    }

    /// `sample_graph` with `spans[1]` (s.p1) replaced.
    fn with_p1_span(span: SourceSpan) -> SourceGraph {
        let mut graph = sample_graph();
        graph.spans[1] = span;
        graph
    }

    // Pins the §4.5 node-kind set: spelling, count, and the textual/structural
    // partition the coverage invariant binds on.
    #[test]
    fn node_kind_spellings_and_textual_partition() {
        let spelled: Vec<&str> = NodeKind::ALL.iter().map(|&v| v.as_str()).collect();
        assert_eq!(
            spelled,
            [
                "document",
                "section",
                "paragraph",
                "list",
                "table",
                "cell",
                "caption",
                "footnote",
                "cq",
                "recommendation"
            ]
        );
        let textual: Vec<&str> = NodeKind::ALL
            .iter()
            .filter(|k| k.is_textual())
            .map(|&v| v.as_str())
            .collect();
        assert_eq!(
            textual,
            [
                "paragraph",
                "cell",
                "caption",
                "footnote",
                "cq",
                "recommendation"
            ]
        );
        for v in NodeKind::ALL {
            round_trip(*v);
        }
    }

    #[test]
    fn anchor_kind_provenance_data_class_spellings() {
        let spelled: Vec<&str> = AnchorKind::ALL.iter().map(|&v| v.as_str()).collect();
        assert_eq!(
            spelled,
            [
                "mention",
                "quantity",
                "modality",
                "negation",
                "temporal_cue",
                "table_value"
            ]
        );
        let spelled: Vec<&str> = Provenance::ALL.iter().map(|&v| v.as_str()).collect();
        assert_eq!(spelled, ["synthetic", "public"]);
        let spelled: Vec<&str> = DataClass::ALL.iter().map(|&v| v.as_str()).collect();
        assert_eq!(spelled, ["none"]);
        for v in AnchorKind::ALL {
            round_trip(*v);
        }
    }

    // Pins the §4.5 document-identity shape, data_class defaulting to none.
    #[test]
    fn document_canonical_bytes() {
        let want = format!(
            concat!(
                r#"{{"content_hash":"{0}","data_class":"none","document_id":"doc.a","#,
                r#""provenance":"synthetic","raw_hash":"{0}","#,
                r#""source_family":"synthetic_fixture_html"}}"#
            ),
            h('a').as_str(),
        );
        assert_eq!(canon(&sample_document()), want);
    }

    // Pins optional-omit on the root's parent_id and the sorted raw-text attrs
    // map carrying §8.2 cell relations.
    #[test]
    fn node_canonical_bytes_and_attrs() {
        let root = node(NodeKind::Document, "n.doc", None);
        assert_eq!(
            canon(&root),
            r#"{"attrs":{},"kind":"document","node_id":"n.doc"}"#
        );
        let cell = SourceNode {
            node_id: id("n.c1"),
            kind: NodeKind::Cell,
            parent_id: Some(id("n.t1")),
            attrs: vec![
                (id("row"), "1".to_owned()),
                (id("col"), "2".to_owned()),
                (id("header"), "true".to_owned()),
            ],
        };
        assert_eq!(
            canon(&cell),
            concat!(
                r#"{"attrs":{"col":"2","header":"true","row":"1"},"#,
                r#""kind":"cell","node_id":"n.c1","parent_id":"n.t1"}"#
            )
        );
        round_trip(root);
        // reading returns the map's canonical key order, so the value that
        // round-trips unchanged is the key-sorted one
        round_trip(SourceNode {
            attrs: vec![
                (id("col"), "2".to_owned()),
                (id("header"), "true".to_owned()),
                (id("row"), "1".to_owned()),
            ],
            ..cell
        });
    }

    // derive() computes end and the three derived fields; the canonical bytes
    // carry Japanese text as raw UTF-8 (no escaping beyond the minimal set).
    #[test]
    fn span_derive_and_canonical_bytes() {
        let span = SourceSpan::derive(id("s.p1"), id("n.p1"), 12, P1_RAW.to_owned(), 2);
        assert_eq!(span.end, 12 + P1_RAW.len() as u64);
        assert_eq!(span.nfkc_text, "成人には10mgを投与する。");
        assert_eq!(span.search_text, "成人には10mgを投与する.");
        assert_eq!(span.text_hash, hash_bytes(P1_RAW.as_bytes()));
        let want = format!(
            concat!(
                r#"{{"end":"54","nfkc_text":"成人には10mgを投与する。","node_id":"n.p1","#,
                r#""raw_text":"成人には１０ｍｇを投与する。","reading_order":"2","#,
                r#""search_text":"成人には10mgを投与する.","span_id":"s.p1","#,
                r#""start":"12","text_hash":"{}"}}"#
            ),
            hash_bytes(P1_RAW.as_bytes()).as_str(),
        );
        assert_eq!(canon(&span), want);
        round_trip(span);
    }

    // Pins the graph's five-field shape with empty collections.
    #[test]
    fn graph_canonical_bytes_minimal() {
        let graph = SourceGraph {
            document: sample_document(),
            nodes: vec![node(NodeKind::Document, "n.doc", None)],
            spans: vec![],
            anchors: vec![],
            regions: vec![],
        };
        let want = format!(
            concat!(
                r#"{{"anchors":[],"document":{},"#,
                r#""nodes":[{{"attrs":{{}},"kind":"document","node_id":"n.doc"}}],"#,
                r#""regions":[],"spans":[]}}"#
            ),
            canon(&sample_document()),
        );
        assert_eq!(canon(&graph), want);
    }

    #[test]
    fn graph_round_trips_fully_populated() {
        round_trip(sample_graph());
    }

    // §4.5: identical source bytes and config give identical canonical bytes —
    // at this layer, set-field input order never reaches the bytes.
    #[test]
    fn set_field_input_order_never_reaches_the_bytes() {
        let graph = sample_graph();
        let mut permuted = sample_graph();
        permuted.anchors.reverse();
        permuted.regions.reverse();
        permuted.regions[0].anchor_ids = vec![id("a.q1")];
        permuted.regions[1].anchor_ids = vec![id("a.m1")];
        assert_eq!(
            canonical_payload_bytes(&graph).unwrap(),
            canonical_payload_bytes(&permuted).unwrap()
        );
        assert_eq!(
            content_hash(&graph).unwrap(),
            content_hash(&permuted).unwrap()
        );
    }

    #[test]
    fn validate_accepts_the_sample_graph() {
        sample_graph().validate(&[]).unwrap();
    }

    #[test]
    fn validate_rejects_root_shape_violations() {
        let mut graph = sample_graph();
        graph.nodes.clear();
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::Root("nodes is empty"))
        );

        let mut graph = sample_graph();
        graph.nodes[0].kind = NodeKind::Section;
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::Root("first node must be the document node"))
        );

        let mut graph = sample_graph();
        graph.nodes[0].parent_id = Some(id("n.sec"));
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::Root("the document node carries a parent"))
        );

        let mut graph = sample_graph();
        graph.nodes[2] = node(NodeKind::Document, "n.doc2", Some("n.doc"));
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::ExtraDocumentNode(id("n.doc2")))
        );

        let mut graph = sample_graph();
        graph.nodes[1].parent_id = None;
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::MissingParent(id("n.sec")))
        );
    }

    #[test]
    fn validate_rejects_duplicate_ids() {
        let mut graph = sample_graph();
        graph.nodes[3] = node(NodeKind::Paragraph, "n.cq1", Some("n.sec"));
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::Duplicate {
                kind: RefKind::Node,
                id: id("n.cq1")
            })
        );

        let mut graph = sample_graph();
        graph.spans[1].span_id = id("s.cq1");
        assert!(matches!(
            graph.validate(&[]),
            Err(GroundingError::Duplicate {
                kind: RefKind::Span,
                ..
            })
        ));

        let mut graph = sample_graph();
        graph.anchors[1].anchor_id = id("a.m1");
        assert!(matches!(
            graph.validate(&[]),
            Err(GroundingError::Duplicate {
                kind: RefKind::Anchor,
                ..
            })
        ));

        let mut graph = sample_graph();
        graph.regions[1].region_id = id("r.cq1");
        assert!(matches!(
            graph.validate(&[]),
            Err(GroundingError::Duplicate {
                kind: RefKind::Region,
                ..
            })
        ));
    }

    #[test]
    fn validate_rejects_dangling_refs_and_empty_regions() {
        // forward parent ref: a child preceding its parent is undefined-so-far
        let mut graph = sample_graph();
        graph.nodes.swap(1, 2);
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::Dangling {
                kind: RefKind::Node,
                id: id("n.sec"),
                from: id("n.cq1")
            })
        );

        let mut graph = sample_graph();
        graph.spans[1].node_id = id("n.zzz");
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::Dangling {
                kind: RefKind::Node,
                id: id("n.zzz"),
                from: id("s.p1")
            })
        );

        let mut graph = sample_graph();
        graph.anchors[1].span_id = id("s.zzz");
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::Dangling {
                kind: RefKind::Span,
                id: id("s.zzz"),
                from: id("a.q1")
            })
        );

        for (field, kind) in [
            ("node_ids", RefKind::Node),
            ("span_ids", RefKind::Span),
            ("anchor_ids", RefKind::Anchor),
        ] {
            let mut graph = sample_graph();
            let region = &mut graph.regions[1];
            match field {
                "node_ids" => region.node_ids = vec![id("x.zzz")],
                "span_ids" => region.span_ids = vec![id("x.zzz")],
                _ => region.anchor_ids = vec![id("x.zzz")],
            }
            assert_eq!(
                graph.validate(&[]),
                Err(GroundingError::Dangling {
                    kind,
                    id: id("x.zzz"),
                    from: id("r.p1")
                }),
                "dangling {field} should be rejected"
            );
        }

        let mut graph = sample_graph();
        graph.regions[1] = SourceRegion {
            region_id: id("r.empty"),
            node_ids: vec![],
            span_ids: vec![],
            anchor_ids: vec![],
        };
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::EmptyRegion(id("r.empty")))
        );
    }

    #[test]
    fn validate_rejects_span_inconsistency() {
        // reading_order must strictly increase along the array
        let mut span = sample_graph().spans[1].clone();
        span.reading_order = 1;
        assert_eq!(
            with_p1_span(span).validate(&[]),
            Err(GroundingError::ReadingOrder {
                span_id: id("s.p1")
            })
        );

        let mut span = sample_graph().spans[1].clone();
        span.end += 1;
        assert!(matches!(
            with_p1_span(span).validate(&[]),
            Err(GroundingError::SpanOffsets { .. })
        ));

        // empty offsets are rejected before the length comparison
        let mut span = sample_graph().spans[1].clone();
        span.end = span.start;
        assert!(matches!(
            with_p1_span(span).validate(&[]),
            Err(GroundingError::SpanOffsets { .. })
        ));

        for (field, tamper) in [
            (
                "nfkc_text",
                &(|s: &mut SourceSpan| s.nfkc_text = s.raw_text.clone())
                    as &dyn Fn(&mut SourceSpan),
            ),
            ("search_text", &|s: &mut SourceSpan| s.search_text.push('!')),
            ("text_hash", &|s: &mut SourceSpan| s.text_hash = h('b')),
        ] {
            let mut span = sample_graph().spans[1].clone();
            tamper(&mut span);
            assert_eq!(
                with_p1_span(span).validate(&[]),
                Err(GroundingError::SpanDerived {
                    span_id: id("s.p1"),
                    field
                })
            );
        }
    }

    #[test]
    fn validate_rejects_bad_anchor_offsets() {
        // past the end of the span's raw_text
        let mut graph = sample_graph();
        graph.anchors[1].end = 100;
        assert!(matches!(
            graph.validate(&[]),
            Err(GroundingError::AnchorOffsets { .. })
        ));

        // byte 13 sits inside the three-byte １ — not a character boundary
        let mut graph = sample_graph();
        graph.anchors[1].start = 13;
        assert!(matches!(
            graph.validate(&[]),
            Err(GroundingError::AnchorOffsets { .. })
        ));

        // empty anchors mark nothing
        let mut graph = sample_graph();
        graph.anchors[1].end = graph.anchors[1].start;
        assert!(matches!(
            graph.validate(&[]),
            Err(GroundingError::AnchorOffsets { .. })
        ));
    }

    // §4.5: every extracted textual unit has a span or a typed
    // extraction_uncertain residual; structural kinds need neither.
    #[test]
    fn validate_enforces_textual_coverage() {
        let mut graph = sample_graph();
        graph
            .nodes
            .push(node(NodeKind::Paragraph, "n.p2", Some("n.sec")));
        assert_eq!(
            graph.validate(&[]),
            Err(GroundingError::UnspannedTextualNode(id("n.p2")))
        );
        graph.validate(&[id("n.p2")]).unwrap();

        // structural nodes carry no coverage obligation
        let mut graph = sample_graph();
        graph
            .nodes
            .push(node(NodeKind::Table, "n.t1", Some("n.sec")));
        graph.validate(&[]).unwrap();
    }

    #[test]
    fn anchor_text_quotes_the_raw_bytes() {
        let graph = sample_graph();
        let quantity = &graph.anchors[1];
        assert_eq!(quantity.text_in(&graph.spans[1]), Some("１０ｍｇ"));
        let mention = &graph.anchors[0];
        assert_eq!(mention.text_in(&graph.spans[0]), Some("高血圧の"));
    }
}
