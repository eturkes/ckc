//! Extract stage core (SPEC §8.3): fixture HTML → grounded
//! [`SourceGraph`] in its §4.4 envelope, `source_graph.json`'s payload.
//!
//! `stage-extract.1` lands the flow walker. html5ever parses the bytes
//! (guaranteeing the html/head/body skeleton); the body walk mints
//! counter ids `n.<k>`/`s.<k>`/`r.<k>` in walk order under the document
//! root: h1–h6 drive a section stack, p maps to paragraph, ul/ol map to
//! list with li children as paragraphs. Every nonempty trimmed textual
//! unit gets one span at offset 0 with strictly increasing reading order
//! plus one {node,span} region; whitespace-only units mint nothing;
//! anchors stay empty (§4.5 subspan anchors belong to later stages).
//! Parse errors and unknown flow content become `extraction_uncertain`
//! residuals; tables ride that residual path until `stage-extract.2`
//! lands the real arm.

use std::collections::HashMap;
use std::fmt;

use ckc_core::{
    ArtifactEnvelope, Authority, CanonError, DataClass, DiagnosticCode, DiagnosticRecord,
    GroundingError, Id, NodeKind, Origin, Outcome, Producer, Provenance, SourceDocument,
    SourceGraph, SourceNode, SourceRegion, SourceSpan, StringPolicy, canonicalization_policy_hash,
    content_hash, hash_bytes,
};
use ego_tree::NodeRef;
use scraper::Html;
use scraper::node::Node;

use crate::shell::static_id;

/// Extractor-fixed identity for one document: everything the §4.5
/// [`SourceDocument`] and the §4.4 envelope need beyond the input bytes.
#[derive(Debug, Clone)]
pub struct ExtractConfig {
    pub document_id: Id,
    /// Open vocabulary, e.g. `synthetic_fixture_html` (§4.5).
    pub source_family: Id,
    pub provenance: Provenance,
    pub data_class: DataClass,
    /// Rides the envelope verbatim; the runner owns its values.
    pub producer: Producer,
}

/// Extraction failed before an envelope could form. Parse trouble is
/// never an error — html5ever recovers and the walker emits
/// `extraction_uncertain` residuals — so the variants are the §4.4/§4.5
/// mechanical invariants plus input decoding.
#[derive(Debug)]
pub enum ExtractError {
    /// Input bytes are not UTF-8 (V1 fixtures carry no charset layer).
    Utf8(std::str::Utf8Error),
    /// The built graph violates a §4.5 invariant (extractor bug).
    Grounding(GroundingError),
    /// Canonical emission failed while hashing the payload.
    Canon(CanonError),
}

impl fmt::Display for ExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractError::Utf8(e) => write!(f, "input is not UTF-8: {e}"),
            ExtractError::Grounding(e) => write!(f, "grounding invariant: {e}"),
            ExtractError::Canon(e) => write!(f, "canonical emission: {e}"),
        }
    }
}

impl std::error::Error for ExtractError {}

impl From<GroundingError> for ExtractError {
    fn from(e: GroundingError) -> Self {
        ExtractError::Grounding(e)
    }
}

impl From<CanonError> for ExtractError {
    fn from(e: CanonError) -> Self {
        ExtractError::Canon(e)
    }
}

/// Parse `html` and build the enveloped source graph: walk the body,
/// validate the graph (residual-licensed), and wrap it per §4.4 —
/// `schema.source_graph`, artifact id `<document_id>.source_graph`,
/// `deterministic_compiler` origin under `mechanical_authority`, empty
/// input/effect/trace/runtime sets, payload hashes computed here.
pub fn extract(
    html: &[u8],
    config: &ExtractConfig,
) -> Result<ArtifactEnvelope<SourceGraph>, ExtractError> {
    let text = std::str::from_utf8(html).map_err(ExtractError::Utf8)?;
    let parsed = Html::parse_document(text);

    let mut walker = Walker::default();
    let doc_node = walker.mint_node(NodeKind::Document, None);
    for error in &parsed.errors {
        walker.residual(format!("parse error: {error}"), &doc_node);
    }
    walker.walk_body(find_body(&parsed), &doc_node);

    let graph = SourceGraph {
        document: SourceDocument {
            document_id: config.document_id.clone(),
            source_family: config.source_family.clone(),
            provenance: config.provenance,
            // No transport or charset decoding applies to fixture bytes,
            // so acquired and consumed bytes coincide (§4.5).
            raw_hash: hash_bytes(html),
            content_hash: hash_bytes(html),
            data_class: config.data_class,
        },
        nodes: walker.nodes,
        spans: walker.spans,
        anchors: vec![],
        regions: walker.regions,
    };
    graph.validate(&walker.residual_nodes)?;

    let artifact_id = Id::new(format!("{}.source_graph", config.document_id))
        .expect("a valid document id keeps the Id grammar under a suffix");
    Ok(ArtifactEnvelope {
        schema_id: static_id("schema.source_graph"),
        artifact_id,
        artifact_kind: static_id("source_graph"),
        producer: config.producer.clone(),
        input_hashes: vec![],
        content_hash: content_hash(&graph)?,
        canonicalization_policy_hash: canonicalization_policy_hash(),
        origin: Origin::DeterministicCompiler,
        authority: Authority::MechanicalAuthority,
        accepted_effects: vec![],
        trace_refs: vec![],
        diagnostics: walker.diagnostics,
        runtime_metadata: vec![],
        payload: graph,
    })
}

/// The body element node. html5ever's tree builder always synthesizes
/// the html/head/body skeleton, so absence is unreachable.
fn find_body(html: &Html) -> NodeRef<'_, Node> {
    fn is_element(n: &NodeRef<'_, Node>, name: &str) -> bool {
        n.value().as_element().is_some_and(|e| e.name() == name)
    }
    let root = html
        .tree
        .root()
        .children()
        .find(|n| is_element(n, "html"))
        .expect("html5ever guarantees the html element");
    root.children()
        .find(|n| is_element(n, "body"))
        .expect("html5ever guarantees the body element")
}

/// `h1`–`h6` → 1–6; `None` for every other element name.
fn heading_level(name: &str) -> Option<u8> {
    match name {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

/// Concatenated descendant text of `node` in document order; phrasing
/// markup inside a textual unit flattens to its text.
fn collect_text(node: NodeRef<'_, Node>) -> String {
    let mut text = String::new();
    for descendant in node.descendants() {
        if let Node::Text(t) = descendant.value() {
            text.push_str(&t.text);
        }
    }
    text
}

/// Walk state: the four §4.5 pools under construction plus the residual
/// bookkeeping `extract` feeds to [`SourceGraph::validate`]. Counter ids
/// derive from pool lengths, so mint order is id order.
#[derive(Default)]
struct Walker {
    nodes: Vec<SourceNode>,
    spans: Vec<SourceSpan>,
    regions: Vec<SourceRegion>,
    diagnostics: Vec<DiagnosticRecord>,
    /// Memoized node-only regions: one region per node grounds every
    /// residual naming that node.
    node_regions: HashMap<Id, Id>,
    /// Nodes named by residual regions, licensing §4.5 coverage.
    residual_nodes: Vec<Id>,
}

impl Walker {
    fn mint_node(&mut self, kind: NodeKind, parent: Option<&Id>) -> Id {
        let node_id = counter_id("n", self.nodes.len());
        self.nodes.push(SourceNode {
            node_id: node_id.clone(),
            kind,
            parent_id: parent.cloned(),
            attrs: vec![],
        });
        node_id
    }

    /// One span over the whole textual unit (offset 0) with its
    /// {node,span} region; reading order is the span counter.
    fn mint_span(&mut self, node_id: &Id, raw_text: String) {
        let k = self.spans.len();
        let span_id = counter_id("s", k);
        self.spans.push(SourceSpan::derive(
            span_id.clone(),
            node_id.clone(),
            0,
            raw_text,
            k as u64,
        ));
        let region_id = counter_id("r", self.regions.len());
        self.regions.push(SourceRegion {
            region_id,
            node_ids: vec![node_id.clone()],
            span_ids: vec![span_id],
            anchor_ids: vec![],
        });
    }

    /// The memoized node-only region for `node_id`, minting on first use.
    fn node_region(&mut self, node_id: &Id) -> Id {
        if let Some(region_id) = self.node_regions.get(node_id) {
            return region_id.clone();
        }
        let region_id = counter_id("r", self.regions.len());
        self.regions.push(SourceRegion {
            region_id: region_id.clone(),
            node_ids: vec![node_id.clone()],
            span_ids: vec![],
            anchor_ids: vec![],
        });
        self.node_regions.insert(node_id.clone(), region_id.clone());
        region_id
    }

    /// One `extraction_uncertain` residual grounded in `node_id`'s
    /// memoized region, licensing that node for §4.5 coverage.
    fn residual(&mut self, detail: String, node_id: &Id) {
        let region_id = self.node_region(node_id);
        self.residual_nodes.push(node_id.clone());
        let detail = StringPolicy::DiagnosticText
            .normalize(&detail)
            .expect("diagnostic_text is infallible");
        self.diagnostics.push(DiagnosticRecord {
            code: DiagnosticCode::ExtractionUncertain,
            outcome: Outcome::Residual,
            payload: vec![(static_id("detail"), detail)],
            region_ids: vec![region_id],
            artifact_hashes: vec![],
        });
    }

    /// Walk the body's children: headings drive the section stack (pop
    /// depths >= level; the heading text spans the section node itself),
    /// known flow elements mint their nodes, everything else is residual.
    fn walk_body(&mut self, body: NodeRef<'_, Node>, doc: &Id) {
        let mut stack: Vec<(u8, Id)> = Vec::new();
        for child in body.children() {
            match child.value() {
                Node::Element(element) => {
                    let name = element.name();
                    if let Some(level) = heading_level(name) {
                        while stack.last().is_some_and(|&(depth, _)| depth >= level) {
                            stack.pop();
                        }
                        let parent = stack.last().map_or(doc, |(_, id)| id).clone();
                        let section = self.mint_node(NodeKind::Section, Some(&parent));
                        self.span_text(child, &section);
                        stack.push((level, section));
                    } else {
                        let parent = stack.last().map_or(doc, |(_, id)| id).clone();
                        self.flow_element(child, name, &parent);
                    }
                }
                Node::Text(t) => {
                    let parent = stack.last().map_or(doc, |(_, id)| id).clone();
                    self.stray_text(&t.text, &parent);
                }
                _ => {}
            }
        }
    }

    /// One non-heading flow element at structural level. Unknown names —
    /// tables included until `stage-extract.2` — leave one residual and
    /// skip the subtree.
    fn flow_element(&mut self, node: NodeRef<'_, Node>, name: &str, parent: &Id) {
        match name {
            "p" => self.textual_node(node, NodeKind::Paragraph, parent),
            "ul" | "ol" => self.list(node, parent),
            _ => self.residual(format!("unknown flow element: {name}"), parent),
        }
    }

    /// A textual-kind node: minted only when its trimmed text is
    /// nonempty, so every textual node carries its span by construction.
    fn textual_node(&mut self, node: NodeRef<'_, Node>, kind: NodeKind, parent: &Id) {
        let text = collect_text(node);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        let node_id = self.mint_node(kind, Some(parent));
        self.mint_span(&node_id, trimmed.to_owned());
    }

    /// Span `node`'s trimmed text onto an already-minted structural node
    /// (a section holding its heading text); whitespace-only mints
    /// nothing and structural kinds carry no coverage obligation.
    fn span_text(&mut self, node: NodeRef<'_, Node>, node_id: &Id) {
        let text = collect_text(node);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            self.mint_span(node_id, trimmed.to_owned());
        }
    }

    /// ul/ol: a list node whose li children are paragraphs; any other
    /// non-whitespace content inside the list is residual.
    fn list(&mut self, node: NodeRef<'_, Node>, parent: &Id) {
        let list_id = self.mint_node(NodeKind::List, Some(parent));
        for child in node.children() {
            match child.value() {
                Node::Element(element) if element.name() == "li" => {
                    self.textual_node(child, NodeKind::Paragraph, &list_id);
                }
                Node::Element(element) => {
                    self.residual(
                        format!("unknown flow element: {}", element.name()),
                        &list_id,
                    );
                }
                Node::Text(t) => self.stray_text(&t.text, &list_id),
                _ => {}
            }
        }
    }

    /// Non-whitespace text outside any textual unit: one residual
    /// grounded in the enclosing node; whitespace-only mints nothing.
    fn stray_text(&mut self, text: &str, parent: &Id) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            self.residual(format!("stray text: {trimmed}"), parent);
        }
    }
}

/// `<prefix>.<k>`: the walk-order counter ids of §8.3 extract.
fn counter_id(prefix: &str, k: usize) -> Id {
    Id::new(format!("{prefix}.{k}")).expect("counter ids match the Id grammar")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::{Hash, canonical_payload_bytes, read_canonical};

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn config() -> ExtractConfig {
        ExtractConfig {
            document_id: id("doc.test"),
            source_family: id("synthetic_fixture_html"),
            provenance: Provenance::Synthetic,
            data_class: DataClass::None,
            producer: Producer {
                candidate_id: id("cand.v1"),
                component_id: id("stage.extract"),
                toolchain_manifest_hash: Hash::new(format!("sha256:{}", "0".repeat(64))).unwrap(),
            },
        }
    }

    /// Extract `html`, returning the payload graph and the envelope
    /// diagnostics.
    fn graph(html: &str) -> (SourceGraph, Vec<DiagnosticRecord>) {
        let envelope = extract(html.as_bytes(), &config()).unwrap();
        (envelope.payload, envelope.diagnostics)
    }

    fn node_shape(g: &SourceGraph) -> Vec<(Id, NodeKind, Option<Id>)> {
        g.nodes
            .iter()
            .map(|n| (n.node_id.clone(), n.kind, n.parent_id.clone()))
            .collect()
    }

    fn span_shape(g: &SourceGraph) -> Vec<(Id, Id, String, u64)> {
        g.spans
            .iter()
            .map(|s| {
                (
                    s.span_id.clone(),
                    s.node_id.clone(),
                    s.raw_text.clone(),
                    s.reading_order,
                )
            })
            .collect()
    }

    fn region_shape(g: &SourceGraph) -> Vec<(Id, Vec<Id>, Vec<Id>)> {
        g.regions
            .iter()
            .map(|r| (r.region_id.clone(), r.node_ids.clone(), r.span_ids.clone()))
            .collect()
    }

    fn detail(d: &DiagnosticRecord) -> &str {
        assert_eq!(d.code, DiagnosticCode::ExtractionUncertain);
        assert_eq!(d.outcome, Outcome::Residual);
        assert!(d.artifact_hashes.is_empty());
        let [(key, value)] = d.payload.as_slice() else {
            panic!("payload is a single detail entry, got {:?}", d.payload);
        };
        assert_eq!(*key, id("detail"));
        value
    }

    // Sections stack by heading level (h1 pops the h1+h2 pair), p and
    // ul/li mint under the open section, every nonempty unit gets one
    // offset-0 span with strictly increasing reading order and one
    // {node,span} region.
    #[test]
    fn walk_shape_sections_paragraphs_lists() {
        let (g, diags) = graph(concat!(
            "<!DOCTYPE html><html><body>",
            "<h1>ＣＱ１：高血圧の治療</h1>",
            "<p>成人には降圧薬を推奨する。</p>",
            "<h2>詳細</h2>",
            "<p>小児には注意を要する。</p>",
            "<h1>第二章</h1>",
            "<ul><li>項目一</li><li>項目二</li></ul>",
            "</body></html>"
        ));
        assert!(
            diags.is_empty(),
            "clean fixture HTML extracts residual-free"
        );
        assert_eq!(
            node_shape(&g),
            vec![
                (id("n.0"), NodeKind::Document, None),
                (id("n.1"), NodeKind::Section, Some(id("n.0"))),
                (id("n.2"), NodeKind::Paragraph, Some(id("n.1"))),
                (id("n.3"), NodeKind::Section, Some(id("n.1"))),
                (id("n.4"), NodeKind::Paragraph, Some(id("n.3"))),
                (id("n.5"), NodeKind::Section, Some(id("n.0"))),
                (id("n.6"), NodeKind::List, Some(id("n.5"))),
                (id("n.7"), NodeKind::Paragraph, Some(id("n.6"))),
                (id("n.8"), NodeKind::Paragraph, Some(id("n.6"))),
            ]
        );
        assert_eq!(
            span_shape(&g),
            vec![
                (id("s.0"), id("n.1"), "ＣＱ１：高血圧の治療".to_owned(), 0),
                (
                    id("s.1"),
                    id("n.2"),
                    "成人には降圧薬を推奨する。".to_owned(),
                    1
                ),
                (id("s.2"), id("n.3"), "詳細".to_owned(), 2),
                (id("s.3"), id("n.4"), "小児には注意を要する。".to_owned(), 3),
                (id("s.4"), id("n.5"), "第二章".to_owned(), 4),
                (id("s.5"), id("n.7"), "項目一".to_owned(), 5),
                (id("s.6"), id("n.8"), "項目二".to_owned(), 6),
            ]
        );
        let want: Vec<(Id, Vec<Id>, Vec<Id>)> = [
            ("r.0", "n.1", "s.0"),
            ("r.1", "n.2", "s.1"),
            ("r.2", "n.3", "s.2"),
            ("r.3", "n.4", "s.3"),
            ("r.4", "n.5", "s.4"),
            ("r.5", "n.7", "s.5"),
            ("r.6", "n.8", "s.6"),
        ]
        .iter()
        .map(|(r, n, s)| (id(r), vec![id(n)], vec![id(s)]))
        .collect();
        assert_eq!(region_shape(&g), want);
        assert!(g.spans.iter().all(|s| s.start == 0));
        assert!(g.anchors.is_empty());
    }

    // Whitespace-only units mint nothing: an empty paragraph leaves no
    // node, an empty heading still opens its (structural, spanless)
    // section, whitespace text mints no residual.
    #[test]
    fn whitespace_only_units_mint_nothing() {
        let (g, diags) = graph(concat!(
            "<!DOCTYPE html><html><body>",
            "<h2>   </h2><p>  </p><p>本文。</p><ul> </ul>  ",
            "</body></html>"
        ));
        assert!(diags.is_empty());
        assert_eq!(
            node_shape(&g),
            vec![
                (id("n.0"), NodeKind::Document, None),
                (id("n.1"), NodeKind::Section, Some(id("n.0"))),
                (id("n.2"), NodeKind::Paragraph, Some(id("n.1"))),
                (id("n.3"), NodeKind::List, Some(id("n.1"))),
            ]
        );
        assert_eq!(
            span_shape(&g),
            vec![(id("s.0"), id("n.2"), "本文。".to_owned(), 0)]
        );
        assert_eq!(
            region_shape(&g),
            vec![(id("r.0"), vec![id("n.2")], vec![id("s.0")])]
        );
    }

    // One residual per Html::errors entry ("Duplicate attribute" pinned
    // from observed html5ever output), all grounded in one memoized
    // whole-document region minted before any walk region.
    #[test]
    fn parse_errors_share_the_memoized_document_region() {
        let (g, diags) = graph(concat!(
            "<!DOCTYPE html><html><body>",
            r#"<p id="a" id="a">甲</p><p id="b" id="b">乙</p>"#,
            "</body></html>"
        ));
        assert_eq!(diags.len(), 2);
        for d in &diags {
            assert_eq!(detail(d), "parse error: Duplicate attribute");
            assert_eq!(d.region_ids, vec![id("r.0")]);
        }
        assert_eq!(
            region_shape(&g),
            vec![
                (id("r.0"), vec![id("n.0")], vec![]),
                (id("r.1"), vec![id("n.1")], vec![id("s.0")]),
                (id("r.2"), vec![id("n.2")], vec![id("s.1")]),
            ]
        );
    }

    // Unknown flow elements (tables until stage-extract.2) and stray
    // text leave one residual each, grounded in the parent node's
    // memoized region; their subtrees mint nothing.
    #[test]
    fn unknown_flow_and_stray_text_residuals() {
        let (g, diags) = graph(concat!(
            "<!DOCTYPE html><html><body>",
            "<h1>見出し</h1>",
            "はぐれた本文",
            "<table><tr><td>表</td></tr></table>",
            "<div><p>内部</p></div>",
            "</body></html>"
        ));
        let details: Vec<&str> = diags.iter().map(detail).collect();
        assert_eq!(
            details,
            vec![
                "stray text: はぐれた本文",
                "unknown flow element: table",
                "unknown flow element: div",
            ]
        );
        for d in &diags {
            assert_eq!(d.region_ids, vec![id("r.1")], "memoized parent region");
        }
        // the skipped subtrees mint no nodes
        assert_eq!(
            node_shape(&g),
            vec![
                (id("n.0"), NodeKind::Document, None),
                (id("n.1"), NodeKind::Section, Some(id("n.0"))),
            ]
        );
        assert_eq!(
            region_shape(&g),
            vec![
                (id("r.0"), vec![id("n.1")], vec![id("s.0")]),
                (id("r.1"), vec![id("n.1")], vec![]),
            ]
        );
    }

    // §4.4 envelope shape: ids, deterministic_compiler origin under
    // mechanical_authority, empty sets, both derived hashes valid, and
    // the §4.5 document identity hashing the input bytes raw.
    #[test]
    fn envelope_shape() {
        let html = "<!DOCTYPE html><html><body><p>本文。</p></body></html>";
        let envelope = extract(html.as_bytes(), &config()).unwrap();
        assert_eq!(envelope.schema_id, id("schema.source_graph"));
        assert_eq!(envelope.artifact_id, id("doc.test.source_graph"));
        assert_eq!(envelope.artifact_kind, id("source_graph"));
        assert_eq!(envelope.producer, config().producer);
        assert_eq!(envelope.origin, Origin::DeterministicCompiler);
        assert_eq!(envelope.authority, Authority::MechanicalAuthority);
        assert!(envelope.input_hashes.is_empty());
        assert!(envelope.accepted_effects.is_empty());
        assert!(envelope.trace_refs.is_empty());
        assert!(envelope.runtime_metadata.is_empty());
        envelope.validate().unwrap();
        let document = &envelope.payload.document;
        assert_eq!(document.document_id, id("doc.test"));
        assert_eq!(document.source_family, id("synthetic_fixture_html"));
        assert_eq!(document.provenance, Provenance::Synthetic);
        assert_eq!(document.data_class, DataClass::None);
        assert_eq!(document.raw_hash, hash_bytes(html.as_bytes()));
        assert_eq!(document.content_hash, document.raw_hash);
    }

    // §4.5 determinism: identical bytes and config give byte-identical
    // envelopes, and the bytes survive a strict read → re-emit cycle.
    #[test]
    fn double_extract_byte_identical_and_strict_reads() {
        let html = concat!(
            "<!DOCTYPE html><html><body>",
            "<h1>ＣＱ１：高血圧の治療</h1>",
            "<p>成人には降圧薬を推奨する。</p>",
            "<table><tr><td>表</td></tr></table>",
            "<ul><li>項目一</li><li>項目二</li></ul>",
            "</body></html>"
        );
        let first = canonical_payload_bytes(&extract(html.as_bytes(), &config()).unwrap()).unwrap();
        let second =
            canonical_payload_bytes(&extract(html.as_bytes(), &config()).unwrap()).unwrap();
        assert_eq!(first, second, "double extract is byte-identical");
        let reread: ArtifactEnvelope<SourceGraph> = read_canonical(&first).unwrap();
        assert_eq!(
            canonical_payload_bytes(&reread).unwrap(),
            first,
            "strict read re-emits the same bytes"
        );
    }

    #[test]
    fn non_utf8_input_is_utf8_error() {
        let err = extract(&[0xff, 0xfe, b'<', b'p', b'>'], &config()).unwrap_err();
        assert!(matches!(err, ExtractError::Utf8(_)), "got {err}");
    }
}
