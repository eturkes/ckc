//! SPEC §5 IRBundle: the five IR layers assembled into one per-document
//! artifact — [`IrBundle`] — plus the derived fields [`assemble`] computes:
//! [`FormalIr`] (via [`FormalIr::derive`]), the reusable-component index
//! ([`ComponentRecord`]s, [`derive_components`]), per-layer structural hashes
//! ([`LayerHashes`]), and the whole-bundle hash.
//!
//! Hash scope: `bundle_hash` is sha256 over ONE structural emission of the
//! five layers plus assumptions and bundle diagnostics under a single fresh
//! scope (the bundle is the component) — the derived fields (`components`,
//! `layer_hashes`, `bundle_hash` itself) stay outside, so the hash is
//! rename-stable like every structural hash. Assumption/diagnostic sets emit
//! per-record fresh scopes ([`emit_structural_record_set`]); the §4.3 IR
//! invariant "Assumptions and uncertainty are explicit payload fields" rides
//! in as [`Assumption`]'s §7.4-style payload.
//!
//! Component records are a derived index: [`Canonical`]/[`CanonRead`] only,
//! no [`Structural`] impl. Structural-hash records cover segments, bindings,
//! statements, rules, and constraints; vocabulary records (actions by
//! normalized key, concepts by id) carry [`content_hash`] — all-vocabulary
//! values emit verbatim in structural bytes, so their canonical bytes ARE
//! their structural bytes. §5 IR invariants (grounding, references, policy
//! completeness) land with `validate` in core-ir.5.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::canon::{
    CanonError, CanonRead, CanonReadError, Canonical, ObjectEmitter, ObjectReader, Reader,
    canonical_sort_key, emit_set, emit_string, read_set, read_string,
};
use crate::enums::{DiagnosticRecord, emit_payload, fieldless_enum, read_payload};
use crate::hash::{content_hash, hash_bytes};
use crate::id::{Hash, Id, ValidationError};
use crate::ir::{
    Action, ClinicalIr, ContextAtom, ContextExpr, DocIr, FormalIr, NormIr, RefLocalizer, SegmentIr,
    Structural, emit_structural_record_set, emit_structural_ref_set, structural_hash,
};

fieldless_enum! {
    /// SPEC §5 reusable-component kind. Population and condition reduce to
    /// concept atoms in V1, so `concept` covers them.
    ComponentKind {
        Concept => "concept",
        Action => "action",
        Segment => "segment",
        Binding => "binding",
        Statement => "statement",
        Rule => "rule",
        Constraint => "constraint",
    }
}

/// SPEC §5 component record: one reusable component's stable id, kind,
/// normalized hash, and use sites (§4.3 set of owner ids). A derived index
/// over the layers ([`derive_components`]) — no [`Structural`] impl;
/// `structural_hash` holds [`content_hash`] for vocabulary kinds
/// (`action`/`concept`), whose canonical bytes are their structural bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentRecord {
    pub component_id: Id,
    pub kind: ComponentKind,
    pub structural_hash: Hash,
    pub use_sites: Vec<Id>,
}

impl Canonical for ComponentRecord {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("component_id", |b| self.component_id.emit_canonical(b))?;
        obj.member("kind", |b| self.kind.emit_canonical(b))?;
        obj.member("structural_hash", |b| {
            self.structural_hash.emit_canonical(b)
        })?;
        obj.member("use_sites", |b| emit_set(b, &self.use_sites))?;
        obj.finish(out)
    }
}

impl CanonRead for ComponentRecord {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let component_id = obj.member("component_id", Id::read)?;
        let kind = obj.member("kind", ComponentKind::read)?;
        let structural_hash = obj.member("structural_hash", Hash::read)?;
        let use_sites = obj.member("use_sites", read_set::<Id>)?;
        obj.close()?;
        Ok(ComponentRecord {
            component_id,
            kind,
            structural_hash,
            use_sites,
        })
    }
}

/// SPEC §5 explicit assumption: a §7.4-style structured payload grounded in
/// §4.5 regions (§4.3 set). Ids localize in structural bytes, payload emits
/// verbatim (assumption content is content, not a renameable reference).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assumption {
    pub assumption_id: Id,
    /// Identifier keys to `diagnostic_text` values (§7.4 payload form).
    pub payload: Vec<(Id, String)>,
    pub region_ids: Vec<Id>,
}

impl Canonical for Assumption {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("assumption_id", |b| self.assumption_id.emit_canonical(b))?;
        obj.member("payload", |b| emit_payload(b, &self.payload))?;
        obj.member("region_ids", |b| emit_set(b, &self.region_ids))?;
        obj.finish(out)
    }
}

impl CanonRead for Assumption {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let assumption_id = obj.member("assumption_id", Id::read)?;
        let payload = obj.member("payload", read_payload)?;
        let region_ids = obj.member("region_ids", read_set::<Id>)?;
        obj.close()?;
        Ok(Assumption {
            assumption_id,
            payload,
            region_ids,
        })
    }
}

impl Structural for Assumption {
    fn emit_structural(&self, out: &mut Vec<u8>, ids: &mut RefLocalizer) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("assumption_id", |b| {
            ids.localize(&self.assumption_id).emit_canonical(b)
        })?;
        obj.member("payload", |b| emit_payload(b, &self.payload))?;
        obj.member("region_ids", |b| {
            emit_structural_ref_set(b, ids, &self.region_ids)
        })?;
        obj.finish(out)
    }
}

/// SPEC §5 per-layer structural hashes, one per IR layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerHashes {
    pub clinical: Hash,
    pub doc: Hash,
    pub formal: Hash,
    pub norm: Hash,
    pub segment: Hash,
}

impl Canonical for LayerHashes {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("clinical", |b| self.clinical.emit_canonical(b))?;
        obj.member("doc", |b| self.doc.emit_canonical(b))?;
        obj.member("formal", |b| self.formal.emit_canonical(b))?;
        obj.member("norm", |b| self.norm.emit_canonical(b))?;
        obj.member("segment", |b| self.segment.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for LayerHashes {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let clinical = obj.member("clinical", Hash::read)?;
        let doc = obj.member("doc", Hash::read)?;
        let formal = obj.member("formal", Hash::read)?;
        let norm = obj.member("norm", Hash::read)?;
        let segment = obj.member("segment", Hash::read)?;
        obj.close()?;
        Ok(LayerHashes {
            clinical,
            doc,
            formal,
            norm,
            segment,
        })
    }
}

/// SPEC §5 IRBundle: the five layers plus component records, assumptions,
/// bundle-level diagnostics (extraction diagnostics stay in [`DocIr`]), and
/// the derived hashes. [`assemble`] builds it; stored set fields hold
/// canonical order, so a bundle round-trips byte-identically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrBundle {
    pub assumptions: Vec<Assumption>,
    pub bundle_hash: Hash,
    pub clinical: ClinicalIr,
    pub components: Vec<ComponentRecord>,
    pub diagnostics: Vec<DiagnosticRecord>,
    pub doc: DocIr,
    pub formal: FormalIr,
    pub layer_hashes: LayerHashes,
    pub norm: NormIr,
    pub segment: SegmentIr,
}

impl Canonical for IrBundle {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("assumptions", |b| emit_set(b, &self.assumptions))?;
        obj.member("bundle_hash", |b| self.bundle_hash.emit_canonical(b))?;
        obj.member("clinical", |b| self.clinical.emit_canonical(b))?;
        obj.member("components", |b| emit_set(b, &self.components))?;
        obj.member("diagnostics", |b| emit_set(b, &self.diagnostics))?;
        obj.member("doc", |b| self.doc.emit_canonical(b))?;
        obj.member("formal", |b| self.formal.emit_canonical(b))?;
        obj.member("layer_hashes", |b| self.layer_hashes.emit_canonical(b))?;
        obj.member("norm", |b| self.norm.emit_canonical(b))?;
        obj.member("segment", |b| self.segment.emit_canonical(b))?;
        obj.finish(out)
    }
}

impl CanonRead for IrBundle {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let assumptions = obj.member("assumptions", read_set::<Assumption>)?;
        let bundle_hash = obj.member("bundle_hash", Hash::read)?;
        let clinical = obj.member("clinical", ClinicalIr::read)?;
        let components = obj.member("components", read_set::<ComponentRecord>)?;
        let diagnostics = obj.member("diagnostics", read_set::<DiagnosticRecord>)?;
        let doc = obj.member("doc", DocIr::read)?;
        let formal = obj.member("formal", FormalIr::read)?;
        let layer_hashes = obj.member("layer_hashes", LayerHashes::read)?;
        let norm = obj.member("norm", NormIr::read)?;
        let segment = obj.member("segment", SegmentIr::read)?;
        obj.close()?;
        Ok(IrBundle {
            assumptions,
            bundle_hash,
            clinical,
            components,
            diagnostics,
            doc,
            formal,
            layer_hashes,
            norm,
            segment,
        })
    }
}

/// Assemble one document's bundle from its produced layers: derive
/// [`FormalIr`] from NormIR, the component index, the per-layer hashes, and
/// the whole-bundle hash (module doc). Cross-reference and re-derivation
/// invariants are `validate`'s job (core-ir.5).
pub fn assemble(
    doc: DocIr,
    segment: SegmentIr,
    clinical: ClinicalIr,
    norm: NormIr,
    assumptions: Vec<Assumption>,
    diagnostics: Vec<DiagnosticRecord>,
) -> Result<IrBundle, CanonError> {
    let formal = FormalIr::derive(&norm);
    let components = derive_components(&segment, &clinical, &norm, &formal)?;
    let layer_hashes = LayerHashes {
        clinical: structural_hash(&clinical)?,
        doc: structural_hash(&doc)?,
        formal: structural_hash(&formal)?,
        norm: structural_hash(&norm)?,
        segment: structural_hash(&segment)?,
    };
    let bundle_hash = hash_bytes(&bundle_structural_bytes(
        &doc,
        &segment,
        &clinical,
        &norm,
        &formal,
        &assumptions,
        &diagnostics,
    )?);
    Ok(IrBundle {
        assumptions,
        bundle_hash,
        clinical,
        components,
        diagnostics,
        doc,
        formal,
        layer_hashes,
        norm,
        segment,
    })
}

/// One structural emission of the bundle's non-derived content under a
/// single fresh scope; `bundle_hash` = sha256 over these bytes.
fn bundle_structural_bytes(
    doc: &DocIr,
    segment: &SegmentIr,
    clinical: &ClinicalIr,
    norm: &NormIr,
    formal: &FormalIr,
    assumptions: &[Assumption],
    diagnostics: &[DiagnosticRecord],
) -> Result<Vec<u8>, CanonError> {
    let mut ids = RefLocalizer::new();
    let mut obj = ObjectEmitter::new();
    obj.member("assumptions", |b| {
        emit_structural_record_set(b, assumptions)
    })?;
    obj.member("clinical", |b| clinical.emit_structural(b, &mut ids))?;
    obj.member("diagnostics", |b| {
        emit_structural_record_set(b, diagnostics)
    })?;
    obj.member("doc", |b| doc.emit_structural(b, &mut ids))?;
    obj.member("formal", |b| formal.emit_structural(b, &mut ids))?;
    obj.member("norm", |b| norm.emit_structural(b, &mut ids))?;
    obj.member("segment", |b| segment.emit_structural(b, &mut ids))?;
    let mut out = Vec::new();
    obj.finish(&mut out)?;
    Ok(out)
}

/// Derive the §5 component index from the layers. Structural-hash records:
/// segments (use sites = statements citing them via `source_segment_ids`),
/// bindings and statements (empty), rules (constraints via `rule_id`),
/// constraints (plan pairs). Vocabulary records under [`content_hash`]:
/// actions keyed by `Action::key`, concepts drawn from binding
/// code+alternatives, statement population/condition/exception atoms, and
/// rule/constraint context atoms. Use sites are sorted+deduped owner ids;
/// records sort by [`canonical_sort_key`], so stored order is canonical set
/// order.
pub fn derive_components(
    segment: &SegmentIr,
    clinical: &ClinicalIr,
    norm: &NormIr,
    formal: &FormalIr,
) -> Result<Vec<ComponentRecord>, CanonError> {
    let mut records = Vec::new();

    for seg in &segment.segments {
        let owners = clinical
            .statements
            .iter()
            .filter(|s| s.source_segment_ids.contains(&seg.segment_id))
            .map(|s| s.statement_id.clone())
            .collect();
        records.push(record(
            seg.segment_id.clone(),
            ComponentKind::Segment,
            structural_hash(seg)?,
            owners,
        ));
    }
    for binding in &clinical.bindings {
        records.push(record(
            binding.binding_id.clone(),
            ComponentKind::Binding,
            structural_hash(binding)?,
            Vec::new(),
        ));
    }
    for statement in &clinical.statements {
        records.push(record(
            statement.statement_id.clone(),
            ComponentKind::Statement,
            structural_hash(statement)?,
            Vec::new(),
        ));
    }
    for rule in &norm.rules {
        let owners = formal
            .constraints
            .iter()
            .filter(|c| c.rule_id == rule.rule_id)
            .map(|c| c.constraint_id.clone())
            .collect();
        records.push(record(
            rule.rule_id.clone(),
            ComponentKind::Rule,
            structural_hash(rule)?,
            owners,
        ));
    }
    for constraint in &formal.constraints {
        let owners = formal
            .plan
            .iter()
            .filter(|p| {
                p.constraint_a_id == constraint.constraint_id
                    || p.constraint_b_id == constraint.constraint_id
            })
            .map(|p| p.pair_id.clone())
            .collect();
        records.push(record(
            constraint.constraint_id.clone(),
            ComponentKind::Constraint,
            structural_hash(constraint)?,
            owners,
        ));
    }

    // Vocabulary occurrences; intermediate map order is irrelevant — every
    // output ordering comes from the sorts below.
    let mut actions: HashMap<Id, (Hash, Vec<Id>)> = HashMap::new();
    let mut concepts: HashMap<Id, Vec<Id>> = HashMap::new();
    for s in &clinical.statements {
        add_action(&mut actions, &s.action, &s.statement_id)?;
        let exception_atoms = s.exceptions.iter().flat_map(|e| &e.atoms);
        for atom in s
            .population
            .iter()
            .chain(&s.condition)
            .chain(exception_atoms)
        {
            add_concept(&mut concepts, atom, &s.statement_id);
        }
    }
    for b in &clinical.bindings {
        for code in std::iter::once(&b.code).chain(&b.alternatives) {
            concepts
                .entry(code.clone())
                .or_default()
                .push(b.binding_id.clone());
        }
    }
    for r in &norm.rules {
        add_action(&mut actions, &r.action, &r.rule_id)?;
        for atom in context_atoms(&r.context) {
            add_concept(&mut concepts, atom, &r.rule_id);
        }
    }
    for c in &formal.constraints {
        add_action(&mut actions, &c.action, &c.constraint_id)?;
        for atom in context_atoms(&c.context) {
            add_concept(&mut concepts, atom, &c.constraint_id);
        }
    }
    for (key, (hash, owners)) in actions {
        records.push(record(key, ComponentKind::Action, hash, owners));
    }
    for (concept, owners) in concepts {
        let hash = content_hash(&concept)?;
        records.push(record(concept, ComponentKind::Concept, hash, owners));
    }

    let mut keyed = records
        .into_iter()
        .map(|r| Ok((canonical_sort_key(&r)?, r)))
        .collect::<Result<Vec<_>, CanonError>>()?;
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(keyed.into_iter().map(|(_, r)| r).collect())
}

/// Build a record with use sites sorted by id bytes and deduped.
fn record(
    component_id: Id,
    kind: ComponentKind,
    structural_hash: Hash,
    mut use_sites: Vec<Id>,
) -> ComponentRecord {
    use_sites.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    use_sites.dedup();
    ComponentRecord {
        component_id,
        kind,
        structural_hash,
        use_sites,
    }
}

/// Record one action occurrence under its normalized key, hashing the value
/// on first sight.
fn add_action(
    map: &mut HashMap<Id, (Hash, Vec<Id>)>,
    action: &Action,
    owner: &Id,
) -> Result<(), CanonError> {
    let slot = match map.entry(action.key.clone()) {
        Entry::Occupied(e) => e.into_mut(),
        Entry::Vacant(v) => v.insert((content_hash(action)?, Vec::new())),
    };
    slot.1.push(owner.clone());
    Ok(())
}

/// Record a concept occurrence; interval atoms carry quantity variables, not
/// concepts.
fn add_concept(map: &mut HashMap<Id, Vec<Id>>, atom: &ContextAtom, owner: &Id) {
    if let ContextAtom::Concept(c) | ContextAtom::ConceptNegated(c) = atom {
        map.entry(c.clone()).or_default().push(owner.clone());
    }
}

/// Every atom of a DNF context, conjunct by conjunct.
fn context_atoms(expr: &ContextExpr) -> impl Iterator<Item = &ContextAtom> {
    expr.any.iter().flat_map(|c| &c.all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon::read_canonical;
    use crate::enums::DiagnosticCode;
    use crate::grounding::NodeKind;
    use crate::ir::tests::{
        atom_c, atom_ge, atom_nc, binding_p, canon, diag, dnf1, id, pair_p, round_trip, rule_p,
        statement_p, structural,
    };
    use crate::ir::{ClinicalSegment, ContextConjunct, SegmentKind, TextBlock};

    fn pid(p: &str, tail: &str) -> Id {
        id(&format!("{p}{tail}"))
    }

    fn doc_p(p: &str) -> DocIr {
        DocIr {
            document_id: pid(p, "doc.a"),
            blocks: vec![TextBlock {
                kind: NodeKind::Cq,
                node_id: pid(p, "n.cq1"),
                span_id: pid(p, "s.cq1"),
            }],
            tables: vec![],
            diagnostics: vec![],
        }
    }

    fn seg_p(p: &str, tail: &str, kind: SegmentKind, region_tail: &str) -> ClinicalSegment {
        ClinicalSegment {
            segment_id: pid(p, tail),
            kind,
            region_ids: vec![pid(p, region_tail)],
        }
    }

    fn assumption_p(p: &str) -> Assumption {
        Assumption {
            assumption_id: pid(p, "as.a.1"),
            payload: vec![(id("note"), "age unit assumed years".to_owned())],
            region_ids: vec![pid(p, "r.cq1")],
        }
    }

    /// The §8.6 worked-rule family as one coherent document under a rename
    /// prefix: DocIR view, the CQ + recommendation segments, the ir.rs
    /// binding/statement/rule fixtures, one assumption, one bundle
    /// diagnostic.
    fn fixture_p(
        p: &str,
    ) -> (
        DocIr,
        SegmentIr,
        ClinicalIr,
        NormIr,
        Vec<Assumption>,
        Vec<DiagnosticRecord>,
    ) {
        let segment = SegmentIr {
            segments: vec![
                seg_p(p, "seg.a.cq1", SegmentKind::Cq, "r.cq1"),
                seg_p(
                    p,
                    "seg.a.cq1.rec1",
                    SegmentKind::Recommendation,
                    "region.a.cq1.rec",
                ),
            ],
        };
        let clinical = ClinicalIr {
            bindings: vec![binding_p(p)],
            statements: vec![statement_p(p)],
        };
        let norm = NormIr {
            rules: vec![rule_p(p)],
        };
        let diagnostics = vec![diag(
            DiagnosticCode::SemanticSlotMissing,
            &format!("{p}r.cq1"),
        )];
        (
            doc_p(p),
            segment,
            clinical,
            norm,
            vec![assumption_p(p)],
            diagnostics,
        )
    }

    fn assemble_p(p: &str) -> IrBundle {
        let (doc, segment, clinical, norm, assumptions, diagnostics) = fixture_p(p);
        assemble(doc, segment, clinical, norm, assumptions, diagnostics).unwrap()
    }

    #[test]
    fn component_kind_spellings() {
        let spelled: Vec<&str> = ComponentKind::ALL.iter().map(|&v| v.as_str()).collect();
        assert_eq!(
            spelled,
            [
                "concept",
                "action",
                "segment",
                "binding",
                "statement",
                "rule",
                "constraint"
            ]
        );
        for v in ComponentKind::ALL {
            round_trip(*v);
        }
    }

    // Pins the assumption's canonical and structural bytes: payload verbatim,
    // ids localized; payload values are diagnostic_text on read.
    #[test]
    fn assumption_bytes() {
        let assumption = assumption_p("");
        assert_eq!(
            canon(&assumption),
            concat!(
                r#"{"assumption_id":"as.a.1","payload":{"note":"age unit assumed years"},"#,
                r#""region_ids":["r.cq1"]}"#
            )
        );
        assert_eq!(
            structural(&assumption),
            concat!(
                r#"{"assumption_id":"i0","payload":{"note":"age unit assumed years"},"#,
                r#""region_ids":["i1"]}"#
            )
        );
        round_trip(assumption);
        let unnormalized = br#"{"assumption_id":"a","payload":{"k":"a  b"},"region_ids":[]}"#;
        assert!(matches!(
            read_canonical::<Assumption>(unnormalized),
            Err(CanonReadError::Unnormalized(_))
        ));
    }

    // Pins the derived index over the worked fixture: every record's id,
    // kind, and use sites, in stored (= canonical set) order, plus the hash
    // vocabulary split (structural for components, content for vocabulary).
    #[test]
    fn assemble_pins_use_sites() {
        let bundle = assemble_p("");
        let got: Vec<(&str, &str, Vec<&str>)> = bundle
            .components
            .iter()
            .map(|r| {
                (
                    r.component_id.as_str(),
                    r.kind.as_str(),
                    r.use_sites.iter().map(Id::as_str).collect(),
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                (
                    "act.administer:drug.abx_a",
                    "action",
                    vec!["fc.rule.a.cq1.r1", "rule.a.cq1.r1", "st.a.cq1.s1"]
                ),
                ("bind.a.m1", "binding", vec![]),
                (
                    "cond.renal_severe",
                    "concept",
                    vec!["fc.rule.a.cq1.r1", "rule.a.cq1.r1", "st.a.cq1.s1"]
                ),
                (
                    "cond.sepsis",
                    "concept",
                    vec![
                        "bind.a.m1",
                        "fc.rule.a.cq1.r1",
                        "rule.a.cq1.r1",
                        "st.a.cq1.s1"
                    ]
                ),
                ("fc.rule.a.cq1.r1", "constraint", vec![]),
                ("rule.a.cq1.r1", "rule", vec!["fc.rule.a.cq1.r1"]),
                ("seg.a.cq1", "segment", vec![]),
                ("seg.a.cq1.rec1", "segment", vec!["st.a.cq1.s1"]),
                ("st.a.cq1.s1", "statement", vec![]),
            ]
        );
        // hash split: structural for layer components, content for vocabulary
        let by_id = |want: &str| {
            bundle
                .components
                .iter()
                .find(|r| r.component_id.as_str() == want)
                .unwrap()
        };
        assert_eq!(
            by_id("seg.a.cq1.rec1").structural_hash,
            structural_hash(&bundle.segment.segments[1]).unwrap()
        );
        assert_eq!(
            by_id("rule.a.cq1.r1").structural_hash,
            structural_hash(&bundle.norm.rules[0]).unwrap()
        );
        assert_eq!(
            by_id("act.administer:drug.abx_a").structural_hash,
            content_hash(&bundle.norm.rules[0].action).unwrap()
        );
        assert_eq!(
            by_id("cond.sepsis").structural_hash,
            content_hash(&id("cond.sepsis")).unwrap()
        );
        // layer hashes re-derive from the stored layers
        assert_eq!(
            bundle.layer_hashes,
            LayerHashes {
                clinical: structural_hash(&bundle.clinical).unwrap(),
                doc: structural_hash(&bundle.doc).unwrap(),
                formal: structural_hash(&bundle.formal).unwrap(),
                norm: structural_hash(&bundle.norm).unwrap(),
                segment: structural_hash(&bundle.segment).unwrap(),
            }
        );
        // assemble derives formal from norm with an empty plan
        assert_eq!(bundle.formal, FormalIr::derive(&bundle.norm));
    }

    // The plan wires constraint use sites once a planner fills it
    // (smt-emit.2); duplicate in-owner occurrences dedup.
    #[test]
    fn derive_components_plan_and_dedup() {
        let mut against = rule_p("");
        against.rule_id = id("rule.b.contra1");
        against.context = ContextExpr {
            any: vec![
                ContextConjunct {
                    all: vec![atom_c("cond.sepsis")],
                },
                ContextConjunct {
                    all: vec![atom_c("cond.sepsis"), atom_nc("cond.renal_severe")],
                },
            ],
        };
        let norm = NormIr {
            rules: vec![rule_p(""), against],
        };
        let mut formal = FormalIr::derive(&norm);
        formal.plan = vec![pair_p("")];
        let empty_segment = SegmentIr { segments: vec![] };
        let empty_clinical = ClinicalIr {
            bindings: vec![],
            statements: vec![],
        };
        let records = derive_components(&empty_segment, &empty_clinical, &norm, &formal).unwrap();
        let by_id = |want: &str| {
            records
                .iter()
                .find(|r| r.component_id.as_str() == want)
                .unwrap()
        };
        // both constraints cite the worked pair
        assert_eq!(
            by_id("fc.rule.a.cq1.r1").use_sites,
            [id("q.v1_conflict.pair1")]
        );
        assert_eq!(
            by_id("fc.rule.b.contra1").use_sites,
            [id("q.v1_conflict.pair1")]
        );
        // cond.sepsis occurs twice inside rule.b's context and once in its
        // constraint: owners dedup to one entry each
        assert_eq!(
            by_id("cond.sepsis").use_sites,
            [
                id("fc.rule.a.cq1.r1"),
                id("fc.rule.b.contra1"),
                id("rule.a.cq1.r1"),
                id("rule.b.contra1")
            ]
        );
        // one action record spans every owner
        assert_eq!(
            by_id("act.administer:drug.abx_a").use_sites,
            [
                id("fc.rule.a.cq1.r1"),
                id("fc.rule.b.contra1"),
                id("rule.a.cq1.r1"),
                id("rule.b.contra1")
            ]
        );
    }

    // §4.3: a uniform rename of document-local ids keeps every layer hash and
    // the bundle hash while the content hash moves; vocabulary is structure,
    // so a concept swap moves the bundle hash.
    #[test]
    fn bundle_hashes_rename_stable() {
        let bundle = assemble_p("");
        let renamed = assemble_p("x.");
        assert_eq!(bundle.layer_hashes, renamed.layer_hashes);
        assert_eq!(bundle.bundle_hash, renamed.bundle_hash);
        assert_ne!(
            content_hash(&bundle).unwrap(),
            content_hash(&renamed).unwrap()
        );

        let (doc, segment, clinical, mut norm, assumptions, diagnostics) = fixture_p("");
        norm.rules[0].context = dnf1(vec![
            atom_c("cond.pneumonia"),
            atom_nc("cond.renal_severe"),
            atom_ge("q.age_years", 18),
        ]);
        let swapped = assemble(doc, segment, clinical, norm, assumptions, diagnostics).unwrap();
        assert_ne!(bundle.bundle_hash, swapped.bundle_hash);
    }

    // Pins the ten-field bundle shape byte-exactly over the smallest
    // assemblable bundle, and round-trips the full fixture.
    #[test]
    fn bundle_round_trip_and_canonical_shape() {
        let minimal = assemble(
            DocIr {
                document_id: id("doc.a"),
                blocks: vec![],
                tables: vec![],
                diagnostics: vec![],
            },
            SegmentIr { segments: vec![] },
            ClinicalIr {
                bindings: vec![],
                statements: vec![],
            },
            NormIr { rules: vec![] },
            vec![],
            vec![],
        )
        .unwrap();
        let want = format!(
            concat!(
                r#"{{"assumptions":[],"bundle_hash":"{}","#,
                r#""clinical":{{"bindings":[],"statements":[]}},"components":[],"#,
                r#""diagnostics":[],"#,
                r#""doc":{{"blocks":[],"diagnostics":[],"document_id":"doc.a","tables":[]}},"#,
                r#""formal":{{"constraints":[],"plan":[]}},"#,
                r#""layer_hashes":{{"clinical":"{}","doc":"{}","formal":"{}","norm":"{}","#,
                r#""segment":"{}"}},"norm":{{"rules":[]}},"segment":{{"segments":[]}}}}"#
            ),
            minimal.bundle_hash.as_str(),
            minimal.layer_hashes.clinical.as_str(),
            minimal.layer_hashes.doc.as_str(),
            minimal.layer_hashes.formal.as_str(),
            minimal.layer_hashes.norm.as_str(),
            minimal.layer_hashes.segment.as_str(),
        );
        assert_eq!(canon(&minimal), want);
        round_trip(minimal);
        round_trip(assemble_p(""));
    }
}
