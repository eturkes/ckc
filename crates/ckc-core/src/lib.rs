//! ckc-core — durable typed core for the Clinical Knowledge Compiler
//! (SPEC §3): stable IDs, value types, enums, schema structs, canonical JSON
//! bytes, semantic hashes, and validation. Surface by module (unit lineage:
//! roadmap stubs + `git log`):
//!
//! - `id` — §4.1 value types [`Id`], [`struct@Hash`], [`Rational`].
//! - `strings` — the seven §4.2 [`StringPolicy`] normalizers.
//! - `canon` — §4.3 canonical bytes: writer ([`Canonical`],
//!   [`canonical_payload_bytes`], [`ObjectEmitter`], [`emit_array`],
//!   [`emit_set`], [`emit_map`], [`emit_union`], [`MapKey`]) and the strict
//!   reader admitting only the writer's bytes ([`read_canonical`],
//!   [`CanonRead`], [`CanonReadError`]).
//! - `hash` — content-hash seal + policy pin ([`content_hash`],
//!   [`hash_bytes`], [`canonicalization_policy_hash`]).
//! - `enums` — §4.4 fieldless enum family ([`Outcome`] severity-ordered),
//!   §7.4 [`DiagnosticRecord`], §4.4 [`TotalOperationResult`].
//! - `envelope` — §4.4 [`ArtifactEnvelope`]; §4.6 [`EventRecord`]; canonical
//!   JSONL framing ([`write_jsonl`], [`read_jsonl`]).
//! - `grounding` — §4.5 [`SourceGraph`] (document, nodes, spans, anchors,
//!   regions; [`SourceGraph::validate`]).
//! - `ir` — the §5 layers [`DocIr`]/[`SegmentIr`]/[`ClinicalIr`]/[`NormIr`]/
//!   [`FormalIr`] over [`ClinicalSegment`], [`ClinicalStatement`],
//!   [`TerminologyBinding`], [`NormRule`] guarded by [`ContextExpr`] DNF of
//!   [`ContextAtom`]s on an [`Action`], [`FormalConstraint`],
//!   [`ContradictionQueryPair`], [`directions_opposed`], the §8.3
//!   normalize-stage payload [`Normalization`]; plus the §4.3
//!   rename-stable structural-hash pattern ([`Structural`],
//!   [`structural_hash`], [`RefLocalizer`]).
//! - `bundle` — §5 [`IrBundle`] assembly and sealing: [`assemble`],
//!   [`derive_components`] ([`ComponentRecord`]), [`Assumption`],
//!   [`LayerHashes`], rename-stable whole-bundle hash, and
//!   [`IrBundle::validate`] ([`BundleError`]) enforcing the §5 grounding,
//!   reference, coherence, and re-derivation invariants.
//! - `plans` — §5/§4.6 run records [`RunPlan`] ([`RunPlan::plan_hash`]),
//!   [`RunManifest`], [`ReplayManifest`], [`SolverIdentity`].
//! - `registry` — §8.4 registries + §8.2 gold, strict-loaded
//!   ([`parse_corpora`], [`parse_candidates`], [`parse_experiments`],
//!   [`parse_gold`]; [`CorpusEntry`], [`Candidates`], [`PipelineEntry`],
//!   [`StageEntry`], [`ExperimentEntry`], [`FixtureGroup`], [`GoldEntry`])
//!   and cross-validated as one set ([`validate_registries`] collecting
//!   [`RegistryFinding`]s: uniqueness, resolution, the stage-chain rule).
#![forbid(unsafe_code)]

mod bundle;
mod canon;
mod enums;
mod envelope;
mod grounding;
mod hash;
mod id;
mod ir;
mod plans;
mod registry;
mod strings;

pub use bundle::{
    Assumption, BundleError, ComponentKind, ComponentRecord, IrBundle, LayerHashes, assemble,
    derive_components,
};
pub use canon::{
    CanonError, CanonRead, CanonReadError, Canonical, MapKey, ObjectEmitter, ObjectReader, Reader,
    canonical_payload_bytes, canonical_sort_key, emit_array, emit_int, emit_map, emit_set,
    emit_string, emit_string_policy, emit_u64_map, emit_union, read_array, read_canonical,
    read_int, read_map, read_set, read_string, read_string_policy, read_u64_map, read_union,
};
pub use enums::{
    AttemptClassification, Authority, BindingStatus, ClaimTier, DiagnosticCode, DiagnosticRecord,
    Direction, Origin, Outcome, PromotionDecision, PromotionScope, ReviewClassification,
    TotalOperationResult,
};
pub use envelope::{
    ArtifactEnvelope, Effect, EnvelopeError, EventRecord, Producer, SCHEMA_VERSION, jsonl_line,
    read_jsonl, write_jsonl,
};
pub use grounding::{
    AnchorKind, DataClass, GroundingError, NodeKind, Provenance, RefKind, SourceAnchor,
    SourceDocument, SourceGraph, SourceNode, SourceRegion, SourceSpan,
};
pub use hash::{CanonicalizationPolicy, canonicalization_policy_hash, content_hash, hash_bytes};
pub use id::{Hash, Id, Rational, RationalRepr, ValidationError};
pub use ir::{
    Action, CellRole, Certainty, ClinicalIr, ClinicalSegment, ClinicalStatement, ContextAtom,
    ContextConjunct, ContextExpr, ContradictionQueryPair, DocIr, ExceptionClause, FormalConstraint,
    FormalIr, IrError, NormIr, NormRule, Normalization, QuantityInterval, RefLocalizer, SegmentIr,
    SegmentKind, Strength, Structural, TableCell, TableView, TerminologyBinding, TextBlock,
    directions_opposed, emit_structural_ref_set, structural_hash,
};
pub use plans::{ReplayManifest, RunManifest, RunPlan, SolverIdentity};
pub use registry::{
    Candidates, CorpusEntry, Determinism, ExperimentEntry, FixtureGroup, GoldEntry, PipelineEntry,
    RegistryError, RegistryFinding, StageEntry, parse_candidates, parse_corpora, parse_experiments,
    parse_gold, to_yaml, validate_registries,
};
pub use strings::StringPolicy;
