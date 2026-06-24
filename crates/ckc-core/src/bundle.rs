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
//! their structural bytes.
//!
//! [`IrBundle::validate`] enforces the §5 IR invariants over a stored bundle
//! and its source document graph in a pinned order — DocIR re-derivation, source_linkage
//! with residuals licensed by `extraction_uncertain` doc diagnostics,
//! per-pool id uniqueness, support/reference resolution, key and interval
//! coherence, the NormIR→FormalIR projection, §6 plan-pair eligibility, and
//! re-derivation of every derived field — yielding a typed [`BundleError`].

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::canon::{
    CanonError, CanonRead, CanonReadError, Canonical, ObjectEmitter, ObjectReader, Reader,
    canonical_sort_key, emit_set, read_set,
};
use crate::enums::{DiagnosticCode, DiagnosticRecord, emit_payload, fieldless_enum, read_payload};
use crate::source_linkage::{SourceLinkageError, SourceDocumentGraph};
use crate::hash::{content_hash, hash_bytes};
use crate::id::{Hash, Id};
use crate::ir::{
    Action, ClinicalIr, ContextAtom, ContextExpr, DocIr, FormalConstraint, FormalIr, IrError,
    NormIr, NormativeRule, QuantityInterval, RefLocalizer, SegmentIr, Structural, directions_opposed,
    emit_structural_record_set, emit_structural_ref_set, structural_hash,
};

fieldless_enum! {
    /// SPEC §5 reusable-component kind. Population and condition reduce to
    /// concept atoms in M1, so `concept` covers them.
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
    pub ir_component_id: Id,
    pub kind: ComponentKind,
    pub structural_hash: Hash,
    pub use_sites: Vec<Id>,
}

impl Canonical for ComponentRecord {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("ir_component_id", |b| self.ir_component_id.emit_canonical(b))?;
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
        let ir_component_id = obj.member("ir_component_id", Id::read)?;
        let kind = obj.member("kind", ComponentKind::read)?;
        let structural_hash = obj.member("structural_hash", Hash::read)?;
        let use_sites = obj.member("use_sites", read_set::<Id>)?;
        obj.close()?;
        Ok(ComponentRecord {
            ir_component_id,
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
/// invariants are [`IrBundle::validate`]'s job.
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
    ir_component_id: Id,
    kind: ComponentKind,
    structural_hash: Hash,
    mut use_sites: Vec<Id>,
) -> ComponentRecord {
    use_sites.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    use_sites.dedup();
    ComponentRecord {
        ir_component_id,
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

/// A SPEC §5 IR-bundle invariant failed ([`IrBundle::validate`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleError {
    /// DocIR derivation from the source document graph failed outright.
    Doc(IrError),
    /// The stored DocIR is not the graph's derivation.
    DocLayerMismatch,
    /// A §4.5 source_linkage invariant failed (residuals licensed first).
    SourceLinkage(SourceLinkageError),
    /// Two entities in one id pool share an id.
    Duplicate { pool: &'static str, id: Id },
    /// A reference names an id its pool does not define.
    Dangling { pool: &'static str, id: Id },
    /// The named segment's region set is empty.
    EmptySupport(Id),
    /// The named statement or rule stores an action key that is not the
    /// `kind:target` derivation.
    KeyMismatch(Id),
    /// The named constraint is missing, out of rule order, or not its rule's
    /// projection.
    ConstraintMismatch(Id),
    /// A quantity interval breaks bound coherence (`rule` names the broken
    /// bound rule).
    Interval { var: Id, rule: &'static str },
    /// A plan pair breaks a §6 eligibility invariant (`rule` names it).
    PairInvalid { pair_id: Id, rule: &'static str },
    /// The stored component index is not the layers' derivation.
    ComponentsMismatch,
    /// A stored layer or bundle hash does not re-derive.
    HashMismatch,
    /// Canonical emission failed while re-deriving.
    Canon(CanonError),
}

impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BundleError::Doc(e) => write!(f, "doc layer derivation: {e}"),
            BundleError::DocLayerMismatch => write!(f, "stored DocIR is not its graph derivation"),
            BundleError::SourceLinkage(e) => write!(f, "source_linkage: {e}"),
            BundleError::Duplicate { pool, id } => write!(f, "duplicate {pool} id {id}"),
            BundleError::Dangling { pool, id } => write!(f, "reference to undefined {pool} {id}"),
            BundleError::EmptySupport(id) => write!(f, "segment {id} supports nothing"),
            BundleError::KeyMismatch(id) => {
                write!(f, "{id} stores an action key that is not kind:target")
            }
            BundleError::ConstraintMismatch(id) => write!(
                f,
                "constraint {id} is missing, out of rule order, or not its rule's projection"
            ),
            BundleError::Interval { var, rule } => write!(f, "interval over {var}: {rule}"),
            BundleError::PairInvalid { pair_id, rule } => write!(f, "plan pair {pair_id}: {rule}"),
            BundleError::ComponentsMismatch => {
                write!(f, "stored components are not the layers' derivation")
            }
            BundleError::HashMismatch => {
                write!(f, "a stored layer or bundle hash does not re-derive")
            }
            BundleError::Canon(e) => write!(f, "canonical emission: {e}"),
        }
    }
}

impl std::error::Error for BundleError {}

impl From<CanonError> for BundleError {
    fn from(e: CanonError) -> Self {
        BundleError::Canon(e)
    }
}

impl IrBundle {
    /// Enforce the SPEC §5 IR invariants over a stored bundle and its source
    /// graph, in a pinned order: (1) the DocIR layer re-derives equal from
    /// `graph`; (2) source_linkage holds, unspanned textual nodes licensed by the
    /// regions of `extraction_uncertain` doc diagnostics; (3) ids are unique
    /// per pool; (4) segment support is nonempty, then every region ref
    /// resolves; (5) statements re-derive keys, cohere intervals, and cite
    /// real segments; (6) rules likewise, exception refs resolving against
    /// statement exceptions; (7) FormalIR is NormIR's total in-order
    /// projection; (8) plan pairs are §6-eligible; (9) the component index
    /// re-derives equal; (10) layer hashes, then the bundle hash, re-derive
    /// equal. [`assemble`] output over a valid graph passes by construction.
    pub fn validate(&self, graph: &SourceDocumentGraph) -> Result<(), BundleError> {
        // (1) DocIR re-derives equal from the graph and the carried
        // extraction diagnostics.
        let derived =
            DocIr::from_graph(graph, self.doc.diagnostics.clone()).map_err(BundleError::Doc)?;
        if derived != self.doc {
            return Err(BundleError::DocLayerMismatch);
        }

        // (2) SourceLinkage, residual textual nodes licensed by the regions of
        // extraction_uncertain doc diagnostics (step 1 resolved them).
        let regions: HashMap<&Id, &[Id]> = graph
            .regions
            .iter()
            .map(|r| (&r.region_id, r.node_ids.as_slice()))
            .collect();
        let residuals: Vec<Id> = self
            .doc
            .diagnostics
            .iter()
            .filter(|d| d.code == DiagnosticCode::ExtractionUncertain)
            .flat_map(|d| &d.region_ids)
            .filter_map(|region_id| regions.get(region_id))
            .flat_map(|nodes| nodes.iter().cloned())
            .collect();
        graph.validate(&residuals).map_err(BundleError::SourceLinkage)?;

        // (3) Id uniqueness per pool.
        unique(
            "segment",
            self.segment.segments.iter().map(|s| &s.segment_id),
        )?;
        unique(
            "binding",
            self.clinical.bindings.iter().map(|b| &b.binding_id),
        )?;
        let statements = &self.clinical.statements;
        unique("statement", statements.iter().map(|s| &s.statement_id))?;
        let exceptions = || statements.iter().flat_map(|s| &s.exceptions);
        unique("exception", exceptions().map(|e| &e.exception_id))?;
        unique("rule", self.norm.rules.iter().map(|r| &r.rule_id))?;
        unique(
            "constraint",
            self.formal.constraints.iter().map(|c| &c.constraint_id),
        )?;
        unique(
            "plan",
            self.formal.plan.iter().flat_map(|p| {
                [
                    &p.pair_id,
                    &p.context_overlap_query_id,
                    &p.deontic_consistency_query_id,
                ]
            }),
        )?;
        unique(
            "assumption",
            self.assumptions.iter().map(|a| &a.assumption_id),
        )?;

        // (4) Segment support is nonempty; every region ref resolves.
        for s in &self.segment.segments {
            if s.region_ids.is_empty() {
                return Err(BundleError::EmptySupport(s.segment_id.clone()));
            }
        }
        let in_regions = |id: &Id| regions.contains_key(id);
        for s in &self.segment.segments {
            resolve("region", in_regions, &s.region_ids)?;
        }
        for b in &self.clinical.bindings {
            resolve("region", in_regions, &b.region_ids)?;
        }
        for e in exceptions() {
            resolve("region", in_regions, &e.region_ids)?;
        }
        for r in &self.norm.rules {
            resolve("region", in_regions, &r.source_region_ids)?;
        }
        for a in &self.assumptions {
            resolve("region", in_regions, &a.region_ids)?;
        }
        for d in &self.diagnostics {
            resolve("region", in_regions, &d.region_ids)?;
        }

        // (5) Statements: stored action keys re-derive, interval atoms
        // cohere, cited segments exist.
        let segment_ids: HashSet<&Id> = self
            .segment
            .segments
            .iter()
            .map(|s| &s.segment_id)
            .collect();
        for s in statements {
            check_key(&s.action, &s.statement_id)?;
            let exception_atoms = s.exceptions.iter().flat_map(|e| &e.atoms);
            check_atoms(
                s.population
                    .iter()
                    .chain(&s.condition)
                    .chain(exception_atoms),
            )?;
            resolve(
                "segment",
                |id| segment_ids.contains(id),
                &s.source_segment_ids,
            )?;
        }

        // (6) Rules: keys, context atoms, exception refs.
        let exception_ids: HashSet<&Id> = exceptions().map(|e| &e.exception_id).collect();
        for r in &self.norm.rules {
            check_key(&r.action, &r.rule_id)?;
            check_atoms(context_atoms(&r.context))?;
            resolve(
                "exception",
                |id| exception_ids.contains(id),
                &r.exception_refs,
            )?;
        }

        // (7) FormalIR is NormIR's projection: every stored constraint names
        // a rule and equals its projection; the sequence is total, in rule
        // order (covers id derivation, omission, and reordering).
        let rules: HashMap<&Id, &NormativeRule> =
            self.norm.rules.iter().map(|r| (&r.rule_id, r)).collect();
        for c in &self.formal.constraints {
            let rule = rules.get(&c.rule_id).ok_or_else(|| BundleError::Dangling {
                pool: "rule",
                id: c.rule_id.clone(),
            })?;
            if FormalConstraint::from_rule(rule) != *c {
                return Err(BundleError::ConstraintMismatch(c.constraint_id.clone()));
            }
        }
        for (i, rule) in self.norm.rules.iter().enumerate() {
            if self
                .formal
                .constraints
                .get(i)
                .is_none_or(|c| c.rule_id != rule.rule_id)
            {
                return Err(BundleError::ConstraintMismatch(
                    FormalConstraint::from_rule(rule).constraint_id,
                ));
            }
        }

        // (8) Plan pairs: §6 eligibility on resolved constraints.
        let constraints: HashMap<&Id, &FormalConstraint> = self
            .formal
            .constraints
            .iter()
            .map(|c| (&c.constraint_id, c))
            .collect();
        for p in &self.formal.plan {
            let lookup = |id: &Id| {
                constraints
                    .get(id)
                    .copied()
                    .ok_or_else(|| BundleError::Dangling {
                        pool: "constraint",
                        id: id.clone(),
                    })
            };
            let a = lookup(&p.constraint_a_id)?;
            let b = lookup(&p.constraint_b_id)?;
            let invalid = |rule: &'static str| BundleError::PairInvalid {
                pair_id: p.pair_id.clone(),
                rule,
            };
            if p.constraint_a_id.as_str() >= p.constraint_b_id.as_str() {
                return Err(invalid("constraint ids not ascending"));
            }
            if p.action_key != a.action.key || p.action_key != b.action.key {
                return Err(invalid("action key mismatch"));
            }
            if !directions_opposed(a.direction, b.direction) {
                return Err(invalid("directions not opposed"));
            }
        }

        // (9) The component index re-derives equal.
        if derive_components(&self.segment, &self.clinical, &self.norm, &self.formal)?
            != self.components
        {
            return Err(BundleError::ComponentsMismatch);
        }

        // (10) Layer hashes, then the bundle hash, re-derive equal.
        let layer_hashes = LayerHashes {
            clinical: structural_hash(&self.clinical)?,
            doc: structural_hash(&self.doc)?,
            formal: structural_hash(&self.formal)?,
            norm: structural_hash(&self.norm)?,
            segment: structural_hash(&self.segment)?,
        };
        if layer_hashes != self.layer_hashes {
            return Err(BundleError::HashMismatch);
        }
        let bundle_hash = hash_bytes(&bundle_structural_bytes(
            &self.doc,
            &self.segment,
            &self.clinical,
            &self.norm,
            &self.formal,
            &self.assumptions,
            &self.diagnostics,
        )?);
        if bundle_hash != self.bundle_hash {
            return Err(BundleError::HashMismatch);
        }
        Ok(())
    }
}

/// One id pool accepts each id once.
fn unique<'a>(pool: &'static str, ids: impl Iterator<Item = &'a Id>) -> Result<(), BundleError> {
    let mut seen: HashSet<&Id> = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(BundleError::Duplicate {
                pool,
                id: id.clone(),
            });
        }
    }
    Ok(())
}

/// Every id in `ids` is defined in `pool`.
fn resolve<'a>(
    pool: &'static str,
    defined: impl Fn(&Id) -> bool,
    ids: impl IntoIterator<Item = &'a Id>,
) -> Result<(), BundleError> {
    for id in ids {
        if !defined(id) {
            return Err(BundleError::Dangling {
                pool,
                id: id.clone(),
            });
        }
    }
    Ok(())
}

/// A stored action key equals its `kind:target` derivation (§5 action
/// sameness rides on keys); `owner` names the holder on failure.
fn check_key(action: &Action, owner: &Id) -> Result<(), BundleError> {
    if Action::new(action.kind.clone(), action.target.clone()).key != action.key {
        return Err(BundleError::KeyMismatch(owner.clone()));
    }
    Ok(())
}

/// Interval atoms cohere; concept atoms carry nothing checkable here.
fn check_atoms<'a>(atoms: impl Iterator<Item = &'a ContextAtom>) -> Result<(), BundleError> {
    for atom in atoms {
        if let ContextAtom::Interval(q) = atom {
            check_interval(q)?;
        }
    }
    Ok(())
}

/// §5 quantity-interval coherence: at least one bound, at most one per side,
/// and a two-sided interval is nonempty over the reals — strict `lo < hi`
/// when either side is strict, else `lo <= hi`.
fn check_interval(q: &QuantityInterval) -> Result<(), BundleError> {
    let fail = |rule: &'static str| BundleError::Interval {
        var: q.var.clone(),
        rule,
    };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon::read_strict_canonical;
    use crate::enums::Direction;
    use crate::source_linkage::{
        DataClass, NodeKind, Provenance, SourceDocument, SourceNode, EvidenceRegion, SourceTextSpan,
    };
    use crate::ir::tests::{
        atom_c, atom_ge, atom_nc, binding_p, canon, diag, dnf1, id, pair_p, round_trip, rule_p,
        statement_p, structural,
    };
    use crate::ir::{
        ClinicalSegment, ContextConjunct, ContradictionQueryPair, SegmentKind, Strength,
    };

    fn pid(p: &str, tail: &str) -> Id {
        id(&format!("{p}{tail}"))
    }

    /// Source document graph behind [`test_source_p`]: document root, CQ heading,
    /// recommendation paragraph, exception sentence — each textual node
    /// spanned — and one region per test_source ref target.
    fn graph_p(p: &str) -> SourceDocumentGraph {
        let node = |kind, tail: &str, parent: Option<&str>| SourceNode {
            node_id: pid(p, tail),
            kind,
            parent_id: parent.map(|t| pid(p, t)),
            attrs: vec![],
        };
        let region = |tail: &str, node_tail: &str, span_tail: &str| EvidenceRegion {
            region_id: pid(p, tail),
            node_ids: vec![pid(p, node_tail)],
            span_ids: vec![pid(p, span_tail)],
            anchor_ids: vec![],
        };
        SourceDocumentGraph {
            document: SourceDocument {
                document_id: pid(p, "doc.a"),
                source_family: id("synthetic_test_source_html"),
                provenance: Provenance::Synthetic,
                raw_hash: hash_bytes(b"raw"),
                content_hash: hash_bytes(b"content"),
                data_class: DataClass::None,
            },
            nodes: vec![
                node(NodeKind::Document, "n.doc", None),
                node(NodeKind::Cq, "n.cq1", Some("n.doc")),
                node(NodeKind::Recommendation, "n.rec", Some("n.doc")),
                node(NodeKind::Paragraph, "n.exc", Some("n.doc")),
            ],
            spans: vec![
                SourceTextSpan::derive(pid(p, "s.cq1"), pid(p, "n.cq1"), 0, "cq1".to_owned(), 1),
                SourceTextSpan::derive(pid(p, "s.rec"), pid(p, "n.rec"), 0, "rec".to_owned(), 2),
                SourceTextSpan::derive(pid(p, "s.exc"), pid(p, "n.exc"), 0, "exc".to_owned(), 3),
            ],
            anchors: vec![],
            regions: vec![
                region("r.cq1", "n.cq1", "s.cq1"),
                region("region.a.cq1.rec", "n.rec", "s.rec"),
                region("region.a.cq1.exc", "n.exc", "s.exc"),
            ],
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
    /// prefix: the DocIR view derived from [`graph_p`], the CQ +
    /// recommendation segments, the ir.rs binding/statement/rule test_sources,
    /// one assumption, one bundle diagnostic.
    fn test_source_p(
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
            DocIr::from_graph(&graph_p(p), vec![]).unwrap(),
            segment,
            clinical,
            norm,
            vec![assumption_p(p)],
            diagnostics,
        )
    }

    fn assemble_p(p: &str) -> IrBundle {
        let (doc, segment, clinical, norm, assumptions, diagnostics) = test_source_p(p);
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
            read_strict_canonical::<Assumption>(unnormalized),
            Err(CanonReadError::Unnormalized(_))
        ));
    }

    // Pins the derived index over the worked test_source: every record's id,
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
                    r.ir_component_id.as_str(),
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
                .find(|r| r.ir_component_id.as_str() == want)
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
                .find(|r| r.ir_component_id.as_str() == want)
                .unwrap()
        };
        // both constraints cite the worked pair
        assert_eq!(
            by_id("fc.rule.a.cq1.r1").use_sites,
            [id("q.m1_conflict.pair1")]
        );
        assert_eq!(
            by_id("fc.rule.b.contra1").use_sites,
            [id("q.m1_conflict.pair1")]
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

        let (doc, segment, clinical, mut norm, assumptions, diagnostics) = test_source_p("");
        norm.rules[0].context = dnf1(vec![
            atom_c("cond.pneumonia"),
            atom_nc("cond.renal_severe"),
            atom_ge("q.age_years", 18),
        ]);
        let swapped = assemble(doc, segment, clinical, norm, assumptions, diagnostics).unwrap();
        assert_ne!(bundle.bundle_hash, swapped.bundle_hash);
    }

    // Pins the ten-field bundle shape byte-exactly over the smallest
    // assemblable bundle, and round-trips the full test_source.
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

    // ---- core-ir.5: validation ------------------------------------------

    /// Re-derive the derived fields after a layer tamper so validation
    /// reaches the targeted invariant instead of tripping on staleness.
    fn restamp(b: &mut IrBundle) {
        b.components = derive_components(&b.segment, &b.clinical, &b.norm, &b.formal).unwrap();
        b.layer_hashes = LayerHashes {
            clinical: structural_hash(&b.clinical).unwrap(),
            doc: structural_hash(&b.doc).unwrap(),
            formal: structural_hash(&b.formal).unwrap(),
            norm: structural_hash(&b.norm).unwrap(),
            segment: structural_hash(&b.segment).unwrap(),
        };
        b.bundle_hash = hash_bytes(
            &bundle_structural_bytes(
                &b.doc,
                &b.segment,
                &b.clinical,
                &b.norm,
                &b.formal,
                &b.assumptions,
                &b.diagnostics,
            )
            .unwrap(),
        );
    }

    /// Tamper the worked bundle, restamp, validate against its graph.
    fn tampered(mutate: impl FnOnce(&mut IrBundle)) -> BundleError {
        let mut b = assemble_p("");
        mutate(&mut b);
        restamp(&mut b);
        b.validate(&graph_p("")).unwrap_err()
    }

    /// Two opposed rules over one action plus the §8.6 worked plan pair —
    /// the smt-emit.2 flow: assemble, fill the plan, restamp.
    fn two_rule_bundle() -> IrBundle {
        let (doc, segment, clinical, mut norm, assumptions, diagnostics) = test_source_p("");
        let mut against = rule_p("");
        against.rule_id = id("rule.b.contra1");
        against.direction = Direction::Against;
        norm.rules.push(against);
        let mut b = assemble(doc, segment, clinical, norm, assumptions, diagnostics).unwrap();
        b.formal.plan = vec![pair_p("")];
        restamp(&mut b);
        b
    }

    #[test]
    fn validate_accepts_assembled_bundles() {
        assemble_p("").validate(&graph_p("")).unwrap();
        assemble_p("x.").validate(&graph_p("x.")).unwrap();
        two_rule_bundle().validate(&graph_p("")).unwrap();
    }

    // §4.5 coverage degrades to typed residuals: an unspanned textual node
    // fails source_linkage bare and passes once an extraction_uncertain doc
    // diagnostic licenses its region's nodes.
    #[test]
    fn validate_licenses_residual_nodes() {
        let mut graph = graph_p("");
        graph.nodes.push(SourceNode {
            node_id: id("n.resid"),
            kind: NodeKind::Paragraph,
            parent_id: Some(id("n.doc")),
            attrs: vec![],
        });
        graph.regions.push(EvidenceRegion {
            region_id: id("r.resid"),
            node_ids: vec![id("n.resid")],
            span_ids: vec![],
            anchor_ids: vec![],
        });
        let (_, segment, clinical, norm, assumptions, diagnostics) = test_source_p("");
        let doc = DocIr::from_graph(&graph, vec![]).unwrap();
        let bare = assemble(doc, segment, clinical, norm, assumptions, diagnostics).unwrap();
        assert_eq!(
            bare.validate(&graph),
            Err(BundleError::SourceLinkage(
                SourceLinkageError::UnspannedTextualNode(id("n.resid"))
            ))
        );
        let (_, segment, clinical, norm, assumptions, diagnostics) = test_source_p("");
        let licensed = vec![diag(DiagnosticCode::ExtractionUncertain, "r.resid")];
        let doc = DocIr::from_graph(&graph, licensed).unwrap();
        assemble(doc, segment, clinical, norm, assumptions, diagnostics)
            .unwrap()
            .validate(&graph)
            .unwrap();
    }

    #[test]
    fn validate_rejects_doc_breaks() {
        assert!(matches!(
            tampered(|b| {
                b.doc.diagnostics = vec![diag(DiagnosticCode::ExtractionUncertain, "r.nope")];
            }),
            BundleError::Doc(IrError::Dangling { .. })
        ));
        assert_eq!(
            tampered(|b| {
                b.doc.blocks.pop();
            }),
            BundleError::DocLayerMismatch
        );
    }

    #[test]
    fn validate_rejects_duplicate_ids() {
        let dup = |pool: &'static str, tail: &str| BundleError::Duplicate { pool, id: id(tail) };
        assert_eq!(
            tampered(|b| {
                let s = b.segment.segments[0].clone();
                b.segment.segments.push(s);
            }),
            dup("segment", "seg.a.cq1")
        );
        assert_eq!(
            tampered(|b| {
                let x = b.clinical.bindings[0].clone();
                b.clinical.bindings.push(x);
            }),
            dup("binding", "bind.a.m1")
        );
        assert_eq!(
            tampered(|b| {
                let s = b.clinical.statements[0].clone();
                b.clinical.statements.push(s);
            }),
            dup("statement", "st.a.cq1.s1")
        );
        assert_eq!(
            tampered(|b| {
                let e = b.clinical.statements[0].exceptions[0].clone();
                b.clinical.statements[0].exceptions.push(e);
            }),
            dup("exception", "exc.a.cq1.e1")
        );
        assert_eq!(
            tampered(|b| {
                let r = b.norm.rules[0].clone();
                b.norm.rules.push(r);
            }),
            dup("rule", "rule.a.cq1.r1")
        );
        assert_eq!(
            tampered(|b| {
                let c = b.formal.constraints[0].clone();
                b.formal.constraints.push(c);
            }),
            dup("constraint", "fc.rule.a.cq1.r1")
        );
        assert_eq!(
            tampered(|b| {
                let a = b.assumptions[0].clone();
                b.assumptions.push(a);
            }),
            dup("assumption", "as.a.1")
        );
        let mut b = two_rule_bundle();
        let p0 = b.formal.plan[0].clone();
        b.formal.plan.push(p0);
        restamp(&mut b);
        assert_eq!(
            b.validate(&graph_p("")).unwrap_err(),
            dup("plan", "q.m1_conflict.pair1")
        );
        let mut b = two_rule_bundle();
        b.formal.plan[0].deontic_consistency_query_id =
            b.formal.plan[0].context_overlap_query_id.clone();
        restamp(&mut b);
        assert_eq!(
            b.validate(&graph_p("")).unwrap_err(),
            dup("plan", "q.m1_conflict.pair1.overlap")
        );
    }

    #[test]
    fn validate_rejects_dangling_refs() {
        let nope = |pool: &'static str, tail: &str| BundleError::Dangling { pool, id: id(tail) };
        let region_break = |got: BundleError| assert_eq!(got, nope("region", "r.nope"));
        region_break(tampered(|b| {
            b.segment.segments[0].region_ids = vec![id("r.nope")];
        }));
        region_break(tampered(|b| {
            b.clinical.bindings[0].region_ids = vec![id("r.nope")];
        }));
        region_break(tampered(|b| {
            b.clinical.statements[0].exceptions[0].region_ids = vec![id("r.nope")];
        }));
        region_break(tampered(|b| {
            b.norm.rules[0].source_region_ids = vec![id("r.nope")];
        }));
        region_break(tampered(|b| {
            b.assumptions[0].region_ids = vec![id("r.nope")];
        }));
        region_break(tampered(|b| {
            b.diagnostics[0].region_ids = vec![id("r.nope")];
        }));
        assert_eq!(
            tampered(|b| {
                b.clinical.statements[0].source_segment_ids = vec![id("seg.nope")];
            }),
            nope("segment", "seg.nope")
        );
        assert_eq!(
            tampered(|b| {
                b.norm.rules[0].exception_refs = vec![id("exc.nope")];
            }),
            nope("exception", "exc.nope")
        );
        assert_eq!(
            tampered(|b| {
                b.formal.constraints[0].rule_id = id("rule.nope");
            }),
            nope("rule", "rule.nope")
        );
        let mut b = two_rule_bundle();
        b.formal.plan[0].constraint_a_id = id("fc.rule.nope");
        restamp(&mut b);
        assert_eq!(
            b.validate(&graph_p("")).unwrap_err(),
            nope("constraint", "fc.rule.nope")
        );
    }

    #[test]
    fn validate_rejects_empty_support() {
        assert_eq!(
            tampered(|b| b.segment.segments[0].region_ids = vec![]),
            BundleError::EmptySupport(id("seg.a.cq1"))
        );
    }

    #[test]
    fn validate_rejects_key_and_projection_tampers() {
        assert_eq!(
            tampered(|b| b.clinical.statements[0].action.key = id("act.administer:drug.other")),
            BundleError::KeyMismatch(id("st.a.cq1.s1"))
        );
        assert_eq!(
            tampered(|b| b.norm.rules[0].action.key = id("act.administer:drug.other")),
            BundleError::KeyMismatch(id("rule.a.cq1.r1"))
        );
        assert_eq!(
            tampered(|b| b.formal.constraints[0].strength = Strength::Weak),
            BundleError::ConstraintMismatch(id("fc.rule.a.cq1.r1"))
        );
        assert_eq!(
            tampered(|b| b.formal.constraints[0].constraint_id = id("fc.wrong")),
            BundleError::ConstraintMismatch(id("fc.wrong"))
        );
        // the projection is total...
        assert_eq!(
            tampered(|b| b.formal.constraints.clear()),
            BundleError::ConstraintMismatch(id("fc.rule.a.cq1.r1"))
        );
        // ...and in rule order
        let mut b = two_rule_bundle();
        b.formal.constraints.swap(0, 1);
        restamp(&mut b);
        assert_eq!(
            b.validate(&graph_p("")).unwrap_err(),
            BundleError::ConstraintMismatch(id("fc.rule.a.cq1.r1"))
        );
    }

    // The §5 interval-coherence table: bound presence, one bound per side,
    // nonemptiness over the reals (strict lo<hi, inclusive lo<=hi).
    #[test]
    fn validate_rejects_incoherent_intervals() {
        let q = |ge, gt, le, lt| QuantityInterval {
            var: id("q.age_years"),
            ge,
            gt,
            le,
            lt,
        };
        let table: [(QuantityInterval, Option<&'static str>); 7] = [
            (q(None, None, None, None), Some("no bound")),
            (q(Some(17), Some(16), None, None), Some("two lower bounds")),
            (q(None, None, Some(65), Some(66)), Some("two upper bounds")),
            (q(Some(18), None, Some(17), None), Some("empty interval")),
            (q(None, Some(18), Some(18), None), Some("empty interval")),
            (q(Some(18), None, Some(18), None), None),
            (q(None, Some(17), None, Some(18)), None),
        ];
        for (interval, want) in table {
            let (doc, segment, clinical, mut norm, assumptions, diagnostics) = test_source_p("");
            norm.rules[0].context = dnf1(vec![ContextAtom::Interval(interval)]);
            let got = assemble(doc, segment, clinical, norm, assumptions, diagnostics)
                .unwrap()
                .validate(&graph_p(""));
            match want {
                None => got.unwrap(),
                Some(rule) => assert_eq!(
                    got.unwrap_err(),
                    BundleError::Interval {
                        var: id("q.age_years"),
                        rule
                    }
                ),
            }
        }
        // statement atoms run the same check (population here)
        let (doc, segment, mut clinical, norm, assumptions, diagnostics) = test_source_p("");
        clinical.statements[0].population = vec![ContextAtom::Interval(q(None, None, None, None))];
        assert_eq!(
            assemble(doc, segment, clinical, norm, assumptions, diagnostics)
                .unwrap()
                .validate(&graph_p(""))
                .unwrap_err(),
            BundleError::Interval {
                var: id("q.age_years"),
                rule: "no bound"
            }
        );
    }

    #[test]
    fn validate_rejects_plan_breaks() {
        let broken = |mutate: fn(&mut ContradictionQueryPair), rule: &'static str| {
            let mut b = two_rule_bundle();
            mutate(&mut b.formal.plan[0]);
            restamp(&mut b);
            assert_eq!(
                b.validate(&graph_p("")).unwrap_err(),
                BundleError::PairInvalid {
                    pair_id: id("q.m1_conflict.pair1"),
                    rule
                }
            );
        };
        broken(
            |p| std::mem::swap(&mut p.constraint_a_id, &mut p.constraint_b_id),
            "constraint ids not ascending",
        );
        broken(
            |p| p.constraint_a_id = p.constraint_b_id.clone(),
            "constraint ids not ascending",
        );
        broken(
            |p| p.action_key = id("act.administer:drug.other"),
            "action key mismatch",
        );
        // both rules `for`: eligibility's direction half fails
        let (doc, segment, clinical, mut norm, assumptions, diagnostics) = test_source_p("");
        let mut second = rule_p("");
        second.rule_id = id("rule.b.contra1");
        norm.rules.push(second);
        let mut b = assemble(doc, segment, clinical, norm, assumptions, diagnostics).unwrap();
        b.formal.plan = vec![pair_p("")];
        restamp(&mut b);
        assert_eq!(
            b.validate(&graph_p("")).unwrap_err(),
            BundleError::PairInvalid {
                pair_id: id("q.m1_conflict.pair1"),
                rule: "directions not opposed"
            }
        );
    }

    // Derived fields gone stale (no restamp) are caught by re-derivation.
    #[test]
    fn validate_rejects_stale_derived_fields() {
        let stale = |mutate: fn(&mut IrBundle), want: BundleError| {
            let mut b = assemble_p("");
            mutate(&mut b);
            assert_eq!(b.validate(&graph_p("")).unwrap_err(), want);
        };
        stale(
            |b| {
                b.components.pop();
            },
            BundleError::ComponentsMismatch,
        );
        stale(
            |b| b.layer_hashes.norm = hash_bytes(b"stale"),
            BundleError::HashMismatch,
        );
        stale(
            |b| b.bundle_hash = hash_bytes(b"stale"),
            BundleError::HashMismatch,
        );
        // layer content tampered without restamping reaches the bundle hash
        stale(
            |b| {
                b.assumptions.push(Assumption {
                    assumption_id: id("as.a.2"),
                    payload: vec![],
                    region_ids: vec![],
                });
            },
            BundleError::HashMismatch,
        );
    }
}
