//! §1.1 steps 3-4 substrate (M0.0.3.4.1): the type-graph walk from each
//! §3.1 inventory root, yielding [`WalkedPath`] rows that .4.2 (entry +
//! alias authoring), .4.3 (binding/bound rows), .4.4 (step-4 coverage +
//! §1.2 role rule), and .4.5 (§1.2 hash conventions) consume.
//!
//! Walk semantics:
//! - rows are payload-rooted: `schema_id` is the §3.1 root via
//!   [`schema_ident`]; `path` is the field chain from the payload root
//!   (§1.3 `FeaturePath` is traversed over a schema-validated payload);
//! - `TypeExpr::Name` recurses into nested S-decls by value, with a
//!   visit-stack cut on cycles and one generic binding per level
//!   (`Env[Inner]` walks `Inner` through the parameter field);
//! - E-decl interiors stay terminal: label enums encode as `Id` strings,
//!   and tagged-union alternatives are positional/tag-addressed, which
//!   `FeaturePath` (`List[Id]`) cannot name — union surface is covered by
//!   `SchemaEntry.tagged_union_alternatives_hash`, registry rows bind only
//!   S-decl-reachable positions;
//! - exactly one `Collection` row per collection field: §1.1 step 4 counts
//!   fields, so anonymous nested layers (`List[Set[RegionMember]]`) share
//!   the field's single bound row, while collections in nested S-decl
//!   fields get their own paths;
//! - the enum-domain Map-key flag requires a label-enum domain; decl-level
//!   alias enums (`E RoleName = Id`) are open domains and stay unflagged.

use ckc_core::canon::canonical_payload_bytes;
use ckc_core::policy::StringPolicy;
use ckc_core::scalar::{FeaturePath, Hash, Id, UInt};

use crate::bounds::{BoundOverflowDisposition, SchemaBoundManifest, SchemaCollectionBound};
use crate::registry::{
    SchemaEntry, SchemaRegistry, SchemaRole, SourceSupportAlias, SourceSupportAliasKind,
    StringPolicyBinding,
};
use crate::spec::{SDecl, SpecDecls, TypeExpr};

/// Leaf classification of one walked path. A field can yield several rows
/// (`subject_hashes:Set[Hash]` is `Collection` and `HashNamed`; a
/// `Map[Id,Text<p>]` field is `Collection` and `Text`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WalkedLeaf {
    /// Set/List/Map field; `enum_domain_key` marks the §1.1 step-4 bound
    /// exemption for scalar maps keyed by a label-enum domain.
    Collection { enum_domain_key: bool },
    /// `Text<policy>` site; `policy` is the raw parameter (a §1.4 policy id
    /// or a dependent sibling-field name — .4.3 resolves which).
    Text { policy: String },
    /// Field named `*_hash`/`*_hashes`/`*_digest` (§1.2 conventions; .4.4
    /// classifies applicability).
    HashNamed,
}

/// One row of the §3.1 type-graph walk.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WalkedPath {
    pub schema_id: Id,
    pub path: FeaturePath,
    pub leaf: WalkedLeaf,
}

/// Inventory type name to registry `schema_id`: boundary-split snake_case
/// (`SchemaRegistry` -> `schema_registry`, `CKCGen` -> `ckc_gen`,
/// `AIRCoreRecord` -> `air_core_record`), matching the crate's
/// `canonical_record!` type ids.
pub fn schema_ident(type_name: &str) -> Id {
    let b = type_name.as_bytes();
    let mut out = String::with_capacity(b.len() + 4);
    for (i, &c) in b.iter().enumerate() {
        if c.is_ascii_uppercase() {
            let boundary = i > 0
                && (!b[i - 1].is_ascii_uppercase()
                    || b.get(i + 1).is_some_and(u8::is_ascii_lowercase));
            if boundary {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase() as char);
        } else {
            out.push(c as char);
        }
    }
    Id::new(&out).expect("snake_case of a type name is a valid Id")
}

/// §1.2 fixed-default source-support alias field names and their kinds;
/// .4.2 authors `SourceSupportAlias` rows from matches over walked paths.
pub const SOURCE_SUPPORT_ALIAS_DEFAULTS: &[(&str, SourceSupportAliasKind)] = &[
    ("source_region_id", SourceSupportAliasKind::SingletonRegion),
    ("support_region_id", SourceSupportAliasKind::SingletonRegion),
    ("source_regions", SourceSupportAliasKind::RegionSet),
    (
        "exact_japanese_source_regions",
        SourceSupportAliasKind::RegionSet,
    ),
    ("source_region_ids", SourceSupportAliasKind::RegionSet),
    ("subject_hash", SourceSupportAliasKind::InheritedSubject),
    ("input_hash", SourceSupportAliasKind::InheritedInput),
    (
        "closed_members",
        SourceSupportAliasKind::ClosedRegionMembers,
    ),
];

/// §1.2 role rule input (shared by .4.2 authoring and the .4.4 checker):
/// the payload exposes source support through the canonical
/// `source_support` field or one fixed-default alias field, checked over
/// the decl's direct fields.
pub fn schema_has_source_support(decl: &SDecl) -> bool {
    decl.fields.iter().any(|f| {
        f.name == "source_support"
            || SOURCE_SUPPORT_ALIAS_DEFAULTS
                .iter()
                .any(|(name, _)| *name == f.name)
    })
}

/// Walks every §3.1 inventory root, in inventory order. Roots without an
/// S-decl are skipped (their absence is an M0.0.3.3 resolution issue).
pub fn walk_inventory(decls: &SpecDecls) -> Vec<WalkedPath> {
    decls
        .inventory
        .iter()
        .flat_map(|name| walk_schema(decls, name))
        .collect()
}

/// Walks one root S-decl by type name; rows in field/recursion order.
pub fn walk_schema(decls: &SpecDecls, root: &str) -> Vec<WalkedPath> {
    let Some(decl) = decls.s_decl(root) else {
        return Vec::new();
    };
    let schema_id = schema_ident(root);
    let mut out = Vec::new();
    let mut stack = vec![root.to_string()];
    walk_sdecl(decls, &schema_id, decl, &[], None, &mut stack, &mut out);
    out
}

fn is_hash_named(field: &str) -> bool {
    field.ends_with("_hash") || field.ends_with("_hashes") || field.ends_with("_digest")
}

/// Replaces the enclosing decl's generic parameter; the binding was itself
/// substituted at the call site, so one level suffices.
fn substitute(ty: &TypeExpr, binding: Option<(&str, &TypeExpr)>) -> TypeExpr {
    let Some((param, bound)) = binding else {
        return ty.clone();
    };
    match ty {
        TypeExpr::Name { name, arg: None } if name == param => bound.clone(),
        TypeExpr::Name { name, arg } => TypeExpr::Name {
            name: name.clone(),
            arg: arg
                .as_ref()
                .map(|a| Box::new(substitute(a, Some((param, bound))))),
        },
        TypeExpr::Set(x) => TypeExpr::Set(Box::new(substitute(x, binding))),
        TypeExpr::List(x) => TypeExpr::List(Box::new(substitute(x, binding))),
        TypeExpr::Optional(x) => TypeExpr::Optional(Box::new(substitute(x, binding))),
        TypeExpr::Map(k, v) => TypeExpr::Map(
            Box::new(substitute(k, binding)),
            Box::new(substitute(v, binding)),
        ),
        TypeExpr::Text(_) => ty.clone(),
    }
}

/// Optional is transparent at every level (§1.3 omission).
fn strip_optional(ty: &TypeExpr) -> &TypeExpr {
    match ty {
        TypeExpr::Optional(x) => strip_optional(x),
        _ => ty,
    }
}

/// `Some(enum_domain_key)` when the field type is a collection.
fn collection_flag(decls: &SpecDecls, ty: &TypeExpr) -> Option<bool> {
    match ty {
        TypeExpr::Set(_) | TypeExpr::List(_) => Some(false),
        TypeExpr::Map(k, _) => Some(is_enum_domain(decls, strip_optional(k))),
        _ => None,
    }
}

fn is_enum_domain(decls: &SpecDecls, key: &TypeExpr) -> bool {
    matches!(key, TypeExpr::Name { name, arg: None }
        if decls.e_decl(name).is_some_and(|d| d.alias_target().is_none()))
}

fn walk_sdecl(
    decls: &SpecDecls,
    schema_id: &Id,
    decl: &SDecl,
    prefix: &[Id],
    binding: Option<(&str, &TypeExpr)>,
    stack: &mut Vec<String>,
    out: &mut Vec<WalkedPath>,
) {
    for f in &decl.fields {
        let mut path = prefix.to_vec();
        path.push(Id::new(&f.name).expect("S-decl field name is a valid Id"));
        let ty = substitute(&f.ty, binding);
        if is_hash_named(&f.name) {
            out.push(WalkedPath {
                schema_id: schema_id.clone(),
                path: FeaturePath::new(path.clone()),
                leaf: WalkedLeaf::HashNamed,
            });
        }
        if let Some(enum_domain_key) = collection_flag(decls, strip_optional(&ty)) {
            out.push(WalkedPath {
                schema_id: schema_id.clone(),
                path: FeaturePath::new(path.clone()),
                leaf: WalkedLeaf::Collection { enum_domain_key },
            });
        }
        type_walk(decls, schema_id, &ty, &path, stack, out);
    }
}

/// Text-site emission and S-decl recursion over one (substituted) field
/// type. Collection rows stay field-level in [`walk_sdecl`].
fn type_walk(
    decls: &SpecDecls,
    schema_id: &Id,
    ty: &TypeExpr,
    path: &[Id],
    stack: &mut Vec<String>,
    out: &mut Vec<WalkedPath>,
) {
    match ty {
        TypeExpr::Optional(x) | TypeExpr::Set(x) | TypeExpr::List(x) => {
            type_walk(decls, schema_id, x, path, stack, out);
        }
        TypeExpr::Map(k, v) => {
            type_walk(decls, schema_id, k, path, stack, out);
            type_walk(decls, schema_id, v, path, stack, out);
        }
        TypeExpr::Text(policy) => out.push(WalkedPath {
            schema_id: schema_id.clone(),
            path: FeaturePath::new(path.to_vec()),
            leaf: WalkedLeaf::Text {
                policy: policy.clone(),
            },
        }),
        TypeExpr::Name { name, arg } => {
            if let Some(nested) = decls.s_decl(name) {
                if !stack.iter().any(|s| s == name) {
                    stack.push(name.clone());
                    let binding = nested.generic_param.as_deref().zip(arg.as_deref());
                    walk_sdecl(decls, schema_id, nested, path, binding, stack, out);
                    stack.pop();
                }
            } else if let Some(a) = arg {
                // Non-S-decl generic application: the argument walks through
                // at the same path.
                type_walk(decls, schema_id, a, path, stack, out);
            }
        }
    }
}

/// §1.2 alias-default suppressions: walked paths whose hash field stores
/// raw recorded bytes (§1.2 raw-bytes convention), which no alias kind can
/// resolve to an artifact's support projection. Keys are
/// (schema_id, `/`-joined path).
pub const ALIAS_EXCEPTIONS: &[(&str, &str)] = &[
    // §4.4: sha256 of the exact extraction input bytes, not an envelope
    // reference, so inherited_input does not apply.
    ("extraction_manifest", "input_hash"),
];

/// §1.2 `SourceSupportAlias` rows over the §3.1 inventory, in inventory
/// order. A schema with a direct canonical `source_support` field
/// registers no alias; otherwise its registered row is the
/// highest-priority fixed-default field-name match over walked field
/// paths — priority by ([`SOURCE_SUPPORT_ALIAS_DEFAULTS`] order, then walk
/// order), minus [`ALIAS_EXCEPTIONS`]. One row per schema: §1.2 names one
/// schema-defined alias as the canonical projection.
pub fn alias_rows(decls: &SpecDecls) -> Vec<SourceSupportAlias> {
    let mut out = Vec::new();
    for name in &decls.inventory {
        let Some(decl) = decls.s_decl(name) else {
            continue;
        };
        if decl.fields.iter().any(|f| f.name == "source_support") {
            continue;
        }
        let schema_id = schema_ident(name);
        let mut cands = Vec::new();
        let mut stack = vec![name.to_string()];
        alias_candidates(decls, &schema_id, decl, &[], None, &mut stack, &mut cands);
        if let Some((_, path, alias_kind)) = cands.into_iter().min_by_key(|(idx, ..)| *idx) {
            out.push(SourceSupportAlias {
                schema_id,
                path: FeaturePath::new(path),
                alias_kind,
            });
        }
    }
    out
}

/// Traversal mirrors [`walk_sdecl`], collecting
/// (default-table index, path, kind) per matching field name.
fn alias_candidates(
    decls: &SpecDecls,
    schema_id: &Id,
    decl: &SDecl,
    prefix: &[Id],
    binding: Option<(&str, &TypeExpr)>,
    stack: &mut Vec<String>,
    out: &mut Vec<(usize, Vec<Id>, SourceSupportAliasKind)>,
) {
    for f in &decl.fields {
        let mut path = prefix.to_vec();
        path.push(Id::new(&f.name).expect("S-decl field name is a valid Id"));
        let joined = path.iter().map(Id::as_str).collect::<Vec<_>>().join("/");
        let excepted = ALIAS_EXCEPTIONS
            .iter()
            .any(|(s, p)| *s == schema_id.as_str() && *p == joined);
        if !excepted
            && let Some(idx) = SOURCE_SUPPORT_ALIAS_DEFAULTS
                .iter()
                .position(|(n, _)| *n == f.name)
        {
            out.push((idx, path.clone(), SOURCE_SUPPORT_ALIAS_DEFAULTS[idx].1));
        }
        let ty = substitute(&f.ty, binding);
        alias_type_recurse(decls, schema_id, &ty, &path, stack, out);
    }
}

/// Traversal mirrors [`type_walk`] (recursion only, no leaf emission).
fn alias_type_recurse(
    decls: &SpecDecls,
    schema_id: &Id,
    ty: &TypeExpr,
    path: &[Id],
    stack: &mut Vec<String>,
    out: &mut Vec<(usize, Vec<Id>, SourceSupportAliasKind)>,
) {
    match ty {
        TypeExpr::Optional(x) | TypeExpr::Set(x) | TypeExpr::List(x) => {
            alias_type_recurse(decls, schema_id, x, path, stack, out);
        }
        TypeExpr::Map(k, v) => {
            alias_type_recurse(decls, schema_id, k, path, stack, out);
            alias_type_recurse(decls, schema_id, v, path, stack, out);
        }
        TypeExpr::Text(_) => {}
        TypeExpr::Name { name, arg } => {
            if let Some(nested) = decls.s_decl(name) {
                if !stack.iter().any(|s| s == name) {
                    stack.push(name.clone());
                    let binding = nested.generic_param.as_deref().zip(arg.as_deref());
                    alias_candidates(decls, schema_id, nested, path, binding, stack, out);
                    stack.pop();
                }
            } else if let Some(a) = arg {
                alias_type_recurse(decls, schema_id, a, path, stack, out);
            }
        }
    }
}

/// Authored §1.2 schema_id -> `SchemaRole` rows, one per §3.1 inventory
/// entry in inventory order. Roles derive from §2 Authority values, §3.2
/// producer position, and source-support presence; non-obvious rows carry
/// their rationale inline. The .4.4 checker validates the §1.2 role rule
/// against registered aliases.
pub const SCHEMA_ROLES: &[(&str, SchemaRole)] = &[
    ("schema_registry", SchemaRole::SchemaControl),
    ("schema_bound_manifest", SchemaRole::SchemaControl),
    ("unicode_policy_manifest", SchemaRole::SchemaControl),
    ("toolchain_manifest", SchemaRole::EnvironmentControl),
    ("tool_record", SchemaRole::EnvironmentControl),
    ("environment_profile", SchemaRole::EnvironmentControl),
    ("producer_manifest", SchemaRole::ReplayControl),
    // §1.6 gate-runner control emission referencing replay_manifest_hash.
    ("validation_manifest", SchemaRole::ReplayControl),
    ("source_edition", SchemaRole::SourceOnly),
    ("source_permission_record", SchemaRole::SourceOnly),
    ("corpus_document", SchemaRole::SourceOnly),
    // §3.2 IngestSourceEdition output; input_hash is raw recorded bytes
    // (see ALIAS_EXCEPTIONS), so no support projection.
    ("extraction_manifest", SchemaRole::SourceOnly),
    ("source_graph", SchemaRole::SourceOnly),
    ("source_region", SchemaRole::SourceOnly),
    ("source_span", SchemaRole::SourceOnly),
    ("source_anchor", SchemaRole::SourceOnly),
    // §3.2 stage -30 source_region_closure output (source builders).
    ("region_closure_certificate", SchemaRole::SourceOnly),
    // §4.4 analyzer identity/version/config: ToolRecord-shaped.
    ("analyzer_manifest", SchemaRole::EnvironmentControl),
    // §4.4 analyzer input resource; mechanical_authority, no support.
    ("mechanical_lexicon", SchemaRole::EnvironmentControl),
    ("mech_obs_payload", SchemaRole::Semantic),
    ("pattern_obs", SchemaRole::Semantic),
    ("match", SchemaRole::Semantic),
    // §7.2 quotient structures: support is proof-DAG-inherited (§1.2).
    ("match_class", SchemaRole::ProofStructure),
    ("class_member", SchemaRole::ProofStructure),
    // §6.4 admitted generator program; its emissions carry support, the
    // program itself does not.
    ("ckc_gen", SchemaRole::AdmissionControl),
    // §6.2 authority = evidence_discovery_only.
    ("generator_grammar_artifact", SchemaRole::EvidenceDiscovery),
    // §6.1 fixture-control rows pin §7.1 closure finite domains for
    // deterministic replay; SchemaRole has no fixture_control.
    ("finite_fixture_manifest", SchemaRole::ReplayControl),
    ("frozen_constant", SchemaRole::ReplayControl),
    ("parsed_quantity", SchemaRole::ReplayControl),
    ("diagnostic_tag", SchemaRole::ReplayControl),
    ("accepted_generator_base", SchemaRole::AdmissionControl),
    // Stage-10 term_resource generator emission with region-grounded
    // concept/binding/relation rows.
    ("terminology_resource_set", SchemaRole::Semantic),
    ("terminology_closure", SchemaRole::Semantic),
    // §5.3 admitted policy artifact; semantic keys carry no support.
    ("semantic_policy_set", SchemaRole::AdmissionControl),
    ("resolution_theorem", SchemaRole::Semantic),
    // §6.4 admission machinery; §3.2 "accepted replay-control artifacts"
    // names replay-inclusion behavior, not the registry role.
    ("proposal_record", SchemaRole::AdmissionControl),
    // §6.4 authority = evidence_discovery_only; scores stay evidence-only.
    ("retrieval_proposal_trace", SchemaRole::EvidenceDiscovery),
    ("admission_context", SchemaRole::AdmissionControl),
    ("reviewer_record", SchemaRole::AdmissionControl),
    ("admission_record", SchemaRole::AdmissionControl),
    ("effect_discharge_record", SchemaRole::AdmissionControl),
    ("counterexample_suite", SchemaRole::AdmissionControl),
    (
        "materialized_consequence_manifest",
        SchemaRole::AdmissionControl,
    ),
    // §7.1 closure-run pinning: input/output hash manifests plus the
    // recomputable bound/termination certificate.
    ("closure_input", SchemaRole::ReplayControl),
    ("closure_output", SchemaRole::ReplayControl),
    ("closure_bound_certificate", SchemaRole::ReplayControl),
    ("license", SchemaRole::Semantic),
    ("licensed_reading_set", SchemaRole::Semantic),
    ("air_core_record", SchemaRole::Semantic),
    ("ckc_normal_form", SchemaRole::Semantic),
    ("witness_context", SchemaRole::Semantic),
    // §7.5 view_only authority invariant family.
    ("gloss_template", SchemaRole::ViewOnly),
    ("gloss_view", SchemaRole::ViewOnly),
    ("conflict_theorem", SchemaRole::Semantic),
    ("factual_inconsistency_theorem", SchemaRole::Semantic),
    ("residual", SchemaRole::Semantic),
    ("ambiguity", SchemaRole::Semantic),
    ("incoherence", SchemaRole::Semantic),
    ("diagnostic", SchemaRole::Semantic),
    ("verifier_witness", SchemaRole::Semantic),
    // §9.1 symbol->definition map: kernel structure without support.
    ("symbol_source_map", SchemaRole::ProofStructure),
    ("constraint_core_witness", SchemaRole::Semantic),
    // §8.7 "proof-visible diagnostic trace" affecting accepted semantics
    // only through admitted edits.
    ("repair_set_search_trace", SchemaRole::EvidenceDiscovery),
    ("proof_node", SchemaRole::ProofStructure),
    ("proof_dag", SchemaRole::ProofStructure),
    ("certificate", SchemaRole::Semantic),
    ("claim_record", SchemaRole::Semantic),
    // §9.3 report wording/rendering control, GlossTemplate-shaped.
    ("report_question_template", SchemaRole::ViewOnly),
    ("report_trace_index", SchemaRole::Semantic),
    ("claim_tier_summary", SchemaRole::ViewOnly),
    ("wording_gate_record", SchemaRole::ViewOnly),
    ("review_report", SchemaRole::Semantic),
    ("replay_manifest", SchemaRole::ReplayControl),
    ("replay_identity_check", SchemaRole::ReplayControl),
];

/// [`SCHEMA_ROLES`] lookup.
pub fn schema_role(schema_id: &str) -> Option<SchemaRole> {
    SCHEMA_ROLES
        .iter()
        .find(|(id, _)| *id == schema_id)
        .map(|(_, role)| *role)
}

/// One §1.1 `SchemaEntry` per §3.1 inventory entry, in inventory order.
/// v0 placeholders pending M0.0.4 (T-Schema-Equivalence):
/// `rust_type_hash` = `generated_json_schema_hash` = sha256 of the entry's
/// S-decl line bytes (the §1.1 design authority for the v0 field set);
/// `tagged_union_alternatives_hash` is None because every inventory row is
/// an S-decl record.
pub fn build_schema_entries(decls: &SpecDecls, spec_text: &str) -> Vec<SchemaEntry> {
    let v0 = Id::new("v0").expect("v0 is a valid Id");
    decls
        .inventory
        .iter()
        .filter_map(|name| {
            let decl = decls.s_decl(name)?;
            let schema_id = schema_ident(name);
            let schema_role =
                schema_role(schema_id.as_str()).expect("SCHEMA_ROLES covers the inventory");
            let line = spec_text
                .lines()
                .nth(decl.line - 1)
                .expect("SDecl.line is 1-based into the parsed text");
            let line_hash = Hash::of_bytes(line.as_bytes());
            Some(SchemaEntry {
                schema_id,
                schema_version: v0.clone(),
                schema_role,
                rust_type_hash: line_hash.clone(),
                generated_json_schema_hash: line_hash,
                tagged_union_alternatives_hash: None,
            })
        })
        .collect()
}

/// One §1.4 `StringPolicyBinding` per walked `Text` site, in walk order.
/// Every S-decl-reachable site names a static §1.4 policy id, so
/// `dependent_policy_field` is None across v0: the one §0 dependent-text
/// site (`TextLiteral.value`, `Text<policy>`) sits inside tagged-union
/// alternatives (`E Literal`/`E EvalScalar`), which `FeaturePath` rows
/// cannot address — its dependency lands with the M0.0.4 union surface.
pub fn binding_rows(decls: &SpecDecls) -> Vec<StringPolicyBinding> {
    walk_inventory(decls)
        .into_iter()
        .filter_map(|r| match r.leaf {
            WalkedLeaf::Text { policy } => Some(StringPolicyBinding {
                schema_id: r.schema_id,
                path: r.path,
                policy: StringPolicy::from_id(&policy)
                    .expect("every S-decl-reachable Text parameter is a §1.4 policy id"),
                dependent_policy_field: None,
            }),
            _ => None,
        })
        .collect()
}

/// v0 authored `max_items` default for per-record collections. Every v0
/// disposition is `reject_with_diagnostic` — §1.1 leaves the bounded
/// artifact absent, the safe posture until a §8.7 consumer motivates
/// residual/ambiguity/incoherence emission.
pub const DEFAULT_MAX_ITEMS: u64 = 65_536;

/// Whole-root `max_items` override: every collection walked under these
/// §3.1 roots holds document/terminology/closure-run-scale data.
pub const SCHEMA_MAX_ITEMS: &[(&str, u64)] = &[
    ("source_graph", 1 << 20),
    ("source_region", 1 << 20),
    ("terminology_resource_set", 1 << 20),
    ("terminology_closure", 1 << 20),
    ("closure_output", 1 << 20),
    ("proof_dag", 1 << 20),
];

/// Per-path `max_items` override for corpus-scale collections inside
/// otherwise per-record roots. Keys are (schema_id, `/`-joined path).
pub const PATH_MAX_ITEMS: &[(&str, &str, u64)] = &[
    // §4.3 closure batches accumulate region members over a whole graph.
    (
        "region_closure_certificate",
        "added_member_batches",
        1 << 20,
    ),
    // §9.3 one row per report item across the corpus run.
    ("report_trace_index", "rows", 1 << 20),
];

fn max_items(schema_id: &str, joined: &str) -> u64 {
    PATH_MAX_ITEMS
        .iter()
        .find(|(s, p, _)| *s == schema_id && *p == joined)
        .map(|(.., n)| *n)
        .or_else(|| {
            SCHEMA_MAX_ITEMS
                .iter()
                .find(|(s, _)| *s == schema_id)
                .map(|(_, n)| *n)
        })
        .unwrap_or(DEFAULT_MAX_ITEMS)
}

/// One §1.1 `SchemaCollectionBound` per walked collection path, in walk
/// order, minus the step-4 exemption for enum-domain Map keys.
pub fn bound_rows(decls: &SpecDecls) -> Vec<SchemaCollectionBound> {
    walk_inventory(decls)
        .into_iter()
        .filter_map(|r| match r.leaf {
            WalkedLeaf::Collection {
                enum_domain_key: false,
            } => {
                let joined = r
                    .path
                    .segments()
                    .iter()
                    .map(Id::as_str)
                    .collect::<Vec<_>>()
                    .join("/");
                let max = max_items(r.schema_id.as_str(), &joined);
                Some(SchemaCollectionBound {
                    schema_id: r.schema_id,
                    path: r.path,
                    max_items: UInt::from(max),
                    overflow_disposition: BoundOverflowDisposition::RejectWithDiagnostic,
                })
            }
            _ => None,
        })
        .collect()
}

/// sha256 of the named §-anchor header line: the v0 placeholder referent
/// for `*_hash` fields whose real input lands later.
fn anchor_line_hash(spec_text: &str, prefix: &str) -> Hash {
    let line = spec_text
        .lines()
        .find(|l| l.starts_with(prefix))
        .expect("named §-anchor is a SPEC.md header line");
    Hash::of_bytes(line.as_bytes())
}

/// v0 `SchemaRegistry` + `SchemaBoundManifest` over parsed SPEC decls.
/// Real hashes: `spec_contract_hash` = sha256 of the full SPEC.md bytes
/// (pass the exact file content); `schema_bound_manifest_hash` = sha256 of
/// the built manifest's canonical payload bytes (§1.2 artifact-ref
/// convention; envelope pending M0.0.5 store). Placeholder hashes anchor
/// the defining § header line, per field: `rust_type_manifest_hash` and
/// `generated_json_schema_manifest_hash` -> §1.1 (both equal pending the
/// M0.0.4 T-Schema-Equivalence manifests, like the per-entry pair);
/// `canonicalization_policy_hash` -> §1.5 (policy artifact pending);
/// `generator_static_bound_policy_hash` -> §6.1 (C-GEN-static bound-excess
/// policy); `parser_bound_policy_hash` -> §6.2 (parser state machine);
/// `closure_bound_policy_hash` -> §7.1 (closure bounds).
pub fn build_v0_registry(
    decls: &SpecDecls,
    spec_text: &str,
) -> (SchemaRegistry, SchemaBoundManifest) {
    let manifest = SchemaBoundManifest {
        manifest_id: Id::new("ckc_schema_bound_manifest_v0").expect("authored Id is valid"),
        schema_collection_bounds: bound_rows(decls).into_iter().collect(),
        generator_static_bound_policy_hash: anchor_line_hash(spec_text, "### 6.1 "),
        closure_bound_policy_hash: anchor_line_hash(spec_text, "### 7.1 "),
        parser_bound_policy_hash: anchor_line_hash(spec_text, "### 6.2 "),
    };
    let manifest_bytes =
        canonical_payload_bytes(&manifest).expect("built bound manifest canonicalizes");
    let section_1_1 = anchor_line_hash(spec_text, "### 1.1 ");
    let registry = SchemaRegistry {
        registry_id: Id::new("ckc_schema_registry").expect("authored Id is valid"),
        registry_version: Id::new("v0").expect("v0 is a valid Id"),
        spec_contract_hash: Hash::of_bytes(spec_text.as_bytes()),
        rust_type_manifest_hash: section_1_1.clone(),
        generated_json_schema_manifest_hash: section_1_1,
        canonicalization_policy_hash: anchor_line_hash(spec_text, "### 1.5 "),
        schema_bound_manifest_hash: Hash::of_bytes(&manifest_bytes),
        schema_entries: build_schema_entries(decls, spec_text).into_iter().collect(),
        string_policy_bindings: binding_rows(decls).into_iter().collect(),
        source_support_aliases: alias_rows(decls).into_iter().collect(),
    };
    (registry, manifest)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use ckc_core::canon::from_canonical_bytes;

    use crate::spec::parse_spec;

    use super::*;

    fn spec_text() -> String {
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../SPEC.md")).unwrap()
    }

    fn synthetic(decl_lines: &str) -> SpecDecls {
        let parse = parse_spec(&format!("## 13. Synthetic\n\n```text\n{decl_lines}\n```\n"));
        assert!(parse.issues.is_empty(), "{:#?}", parse.issues);
        parse.decls
    }

    fn row(schema: &str, segments: &[&str], leaf: WalkedLeaf) -> WalkedPath {
        WalkedPath {
            schema_id: Id::new(schema).unwrap(),
            path: FeaturePath::new(
                segments
                    .iter()
                    .map(|s| Id::new(s).unwrap())
                    .collect::<Vec<_>>(),
            ),
            leaf,
        }
    }

    const COLL: WalkedLeaf = WalkedLeaf::Collection {
        enum_domain_key: false,
    };

    fn text(policy: &str) -> WalkedLeaf {
        WalkedLeaf::Text {
            policy: policy.to_string(),
        }
    }

    /// Nested S-decl recursion through a collection, with exact row order
    /// (field order, recursion depth-first).
    #[test]
    fn build_nested_walk_exact_rows() {
        let decls = synthetic(
            "S Root(root_hash:Hash,items:Set[Item],note:Text<semantic_ja>)\n\
             S Item(item_id:Id,payload_digest:Hash,tags:List[Id])",
        );
        assert_eq!(
            walk_schema(&decls, "Root"),
            [
                row("root", &["root_hash"], WalkedLeaf::HashNamed),
                row("root", &["items"], COLL),
                row("root", &["items", "payload_digest"], WalkedLeaf::HashNamed),
                row("root", &["items", "tags"], COLL),
                row("root", &["note"], text("semantic_ja")),
            ]
        );
    }

    /// Cyclic S-decl pair terminates via the visit-stack cut; rows from the
    /// first expansion survive.
    #[test]
    fn build_cyclic_decls_terminate() {
        let decls = synthetic(
            "S Node(next:Edge,node_hash:Hash)\n\
             S Edge(peers:Set[Node])",
        );
        assert_eq!(
            walk_schema(&decls, "Node"),
            [
                row("node", &["next", "peers"], COLL),
                row("node", &["node_hash"], WalkedLeaf::HashNamed),
            ]
        );
    }

    /// Enum-domain Map keys flag the §1.1 step-4 exemption; alias enums
    /// (`E Alias = Id`) and scalar keys stay unflagged; Optional is
    /// transparent around the collection.
    #[test]
    fn build_enum_domain_map_keys() {
        let decls = synthetic(
            "E Color = red | green | blue\n\
             E Alias = Id\n\
             S Board(cells:Map[Color,UInt],named:Map[Alias,UInt],open:Map[Id,UInt]?)",
        );
        assert_eq!(
            walk_schema(&decls, "Board"),
            [
                row(
                    "board",
                    &["cells"],
                    WalkedLeaf::Collection {
                        enum_domain_key: true
                    }
                ),
                row("board", &["named"], COLL),
                row("board", &["open"], COLL),
            ]
        );
    }

    /// Generic argument walks through the parameter field (`Env[Inner]`
    /// binds `T` to `Inner` inside `Env`).
    #[test]
    fn build_generic_args_walk_through() {
        let decls = synthetic(
            "S Env<T>(payload:T,env_hash:Hash)\n\
             S Inner(parts:List[Id],inner_hash:Hash)\n\
             S Holder(boxed:Env[Inner])",
        );
        assert_eq!(
            walk_schema(&decls, "Holder"),
            [
                row("holder", &["boxed", "payload", "parts"], COLL),
                row(
                    "holder",
                    &["boxed", "payload", "inner_hash"],
                    WalkedLeaf::HashNamed
                ),
                row("holder", &["boxed", "env_hash"], WalkedLeaf::HashNamed),
            ]
        );
    }

    #[test]
    fn build_schema_ident_boundaries() {
        for (name, ident) in [
            ("SchemaRegistry", "schema_registry"),
            ("CKCGen", "ckc_gen"),
            ("CKCNormalForm", "ckc_normal_form"),
            ("AIRCoreRecord", "air_core_record"),
            ("MechObsPayload", "mech_obs_payload"),
            ("BBox", "b_box"),
        ] {
            assert_eq!(schema_ident(name).as_str(), ident);
        }
    }

    /// §1.2 source-support detection over real decls: canonical/alias
    /// fields hit, control schemas miss.
    #[test]
    fn build_source_support_detection() {
        let parse = parse_spec(&spec_text());
        for (schema, expect) in [
            ("Residual", true),        // source_regions
            ("MechObsPayload", true),  // source_region_id
            ("SourceRegion", true),    // closed_members
            ("SchemaRegistry", false), // schema_control
            ("UnicodePolicyManifest", false),
        ] {
            assert_eq!(
                schema_has_source_support(parse.decls.s_decl(schema).unwrap()),
                expect,
                "{schema}"
            );
        }
    }

    /// Real-SPEC walk: every inventory root is a walkable S-decl, rows are
    /// unique, per-leaf-family counts are nonzero (and the enum-domain
    /// exemption has no real instance: walked Map keys are Id, Hash, Int,
    /// Var, RoleName), with payload-rooted spot-checks.
    #[test]
    fn build_real_spec_inventory_walk() {
        let parse = parse_spec(&spec_text());
        let decls = &parse.decls;
        assert!(parse.issues.is_empty());
        assert_eq!(
            decls
                .inventory
                .iter()
                .filter(|n| decls.s_decl(n).is_some())
                .count(),
            decls.inventory.len()
        );

        let rows = walk_inventory(decls);
        let unique: BTreeSet<&WalkedPath> = rows.iter().collect();
        assert_eq!(unique.len(), rows.len());

        let count =
            |pred: &dyn Fn(&WalkedLeaf) -> bool| rows.iter().filter(|r| pred(&r.leaf)).count();
        assert!(count(&|l| matches!(l, WalkedLeaf::Collection { .. })) > 200);
        assert!(count(&|l| matches!(l, WalkedLeaf::Text { .. })) > 30);
        assert!(count(&|l| matches!(l, WalkedLeaf::HashNamed)) > 200);
        assert_eq!(
            count(&|l| matches!(
                l,
                WalkedLeaf::Collection {
                    enum_domain_key: true
                }
            )),
            0
        );

        for expected in [
            row("schema_registry", &["schema_entries"], COLL),
            row(
                "schema_registry",
                &["schema_entries", "rust_type_hash"],
                WalkedLeaf::HashNamed,
            ),
            // Map[Id,Text<semantic_ja>]: one Collection and one Text row.
            row("mech_obs_payload", &["fields"], COLL),
            row("mech_obs_payload", &["fields"], text("semantic_ja")),
            row("source_span", &["raw_text"], text("raw_source")),
            // List[Set[RegionMember]]: one field-level Collection row;
            // RegionMember is a tagged union, so no deeper rows.
            row(
                "region_closure_certificate",
                &["added_member_batches"],
                COLL,
            ),
            row("residual", &["source_regions"], COLL),
            row("residual", &["diagnostic"], text("diagnostic_text")),
        ] {
            assert!(rows.contains(&expected), "missing {expected:?}");
        }
        assert!(
            !rows
                .iter()
                .any(|r| r.schema_id.as_str() == "region_closure_certificate"
                    && r.path.segments().len() > 1
                    && r.path.segments()[0].as_str() == "added_member_batches")
        );
    }

    /// SCHEMA_ROLES is the inventory in order; entries derive 1:1 with v0
    /// placeholder hashes (S-decl line bytes, recomputed independently)
    /// and role spot-checks across all nine roles.
    #[test]
    fn build_schema_entries_cover_inventory() {
        let text = spec_text();
        let parse = parse_spec(&text);
        let decls = &parse.decls;
        assert_eq!(SCHEMA_ROLES.len(), decls.inventory.len());
        for (name, (id, _)) in decls.inventory.iter().zip(SCHEMA_ROLES) {
            assert_eq!(schema_ident(name).as_str(), *id);
        }

        let entries = build_schema_entries(decls, &text);
        assert_eq!(entries.len(), decls.inventory.len());
        let registry_line = text
            .lines()
            .find(|l| l.starts_with("S SchemaRegistry("))
            .unwrap();
        assert_eq!(entries[0].schema_id.as_str(), "schema_registry");
        assert_eq!(
            entries[0].rust_type_hash,
            Hash::of_bytes(registry_line.as_bytes())
        );
        for e in &entries {
            assert_eq!(e.schema_version.as_str(), "v0");
            assert_eq!(e.rust_type_hash, e.generated_json_schema_hash);
            assert_eq!(e.tagged_union_alternatives_hash, None);
        }
        for (id, role) in [
            ("pattern_obs", SchemaRole::Semantic),
            ("proof_dag", SchemaRole::ProofStructure),
            ("gloss_view", SchemaRole::ViewOnly),
            ("proposal_record", SchemaRole::AdmissionControl),
            ("retrieval_proposal_trace", SchemaRole::EvidenceDiscovery),
            ("mechanical_lexicon", SchemaRole::EnvironmentControl),
            ("frozen_constant", SchemaRole::ReplayControl),
            ("region_closure_certificate", SchemaRole::SourceOnly),
            ("unicode_policy_manifest", SchemaRole::SchemaControl),
        ] {
            assert_eq!(schema_role(id), Some(role), "{id}");
        }
    }

    /// §1.2 alias rows over the real SPEC: one row per support-bearing
    /// schema, highest-priority candidate, canonical-field and raw-bytes
    /// exclusions hold.
    #[test]
    fn build_alias_rows_real_spec() {
        let parse = parse_spec(&spec_text());
        let rows = alias_rows(&parse.decls);
        let by_schema: BTreeMap<&str, &SourceSupportAlias> =
            rows.iter().map(|r| (r.schema_id.as_str(), r)).collect();
        assert_eq!(by_schema.len(), rows.len());
        assert_eq!(rows.len(), 25);

        use SourceSupportAliasKind::*;
        for (schema, path, kind) in [
            ("mech_obs_payload", "source_region_id", SingletonRegion),
            ("pattern_obs", "support_region_id", SingletonRegion),
            (
                "licensed_reading_set",
                "air_key/support_region_id",
                SingletonRegion,
            ),
            (
                "air_core_record",
                "air_key/support_region_id",
                SingletonRegion,
            ),
            ("source_region", "closed_members", ClosedRegionMembers),
            // source_regions beats subject_hash by default-table order.
            ("residual", "source_regions", RegionSet),
            ("diagnostic", "source_regions", RegionSet),
            ("verifier_witness", "input_hash", InheritedInput),
            ("certificate", "subject_hash", InheritedSubject),
            ("admission_record", "subject_hash", InheritedSubject),
            (
                "terminology_resource_set",
                "concepts/source_region_ids",
                RegionSet,
            ),
            (
                "terminology_closure",
                "normalized_relations/source_region_ids",
                RegionSet,
            ),
            (
                "constraint_core_witness",
                "named_atoms/source_region_ids",
                RegionSet,
            ),
            ("report_trace_index", "rows/source_region_ids", RegionSet),
            (
                "review_report",
                "report_items/exact_japanese_source_regions",
                RegionSet,
            ),
            (
                "finite_fixture_manifest",
                "parsed_quantities/source_region_id",
                SingletonRegion,
            ),
        ] {
            let row = by_schema[schema];
            let joined = row
                .path
                .segments()
                .iter()
                .map(Id::as_str)
                .collect::<Vec<_>>()
                .join("/");
            assert_eq!((joined.as_str(), row.alias_kind), (path, kind), "{schema}");
        }
        // Canonical source_support field or raw-bytes exception -> no row.
        for absent in [
            "license",
            "ckc_normal_form",
            "resolution_theorem",
            "witness_context",
            "extraction_manifest",
            "schema_registry",
            "match_class",
            "proof_dag",
        ] {
            assert!(!by_schema.contains_key(absent), "{absent}");
        }
    }

    /// §1.2 role rule over authored roles + built aliases: semantic
    /// schemas expose support (canonical field or registered alias);
    /// canonical-field schemas register no alias; alias rows point at
    /// inventory schemas.
    #[test]
    fn build_role_support_consistency() {
        let parse = parse_spec(&spec_text());
        let decls = &parse.decls;
        let rows = alias_rows(decls);
        let aliased: BTreeSet<&str> = rows.iter().map(|r| r.schema_id.as_str()).collect();
        let ids: BTreeSet<Id> = decls.inventory.iter().map(|n| schema_ident(n)).collect();
        for r in &rows {
            assert!(ids.contains(&r.schema_id), "{:?}", r.schema_id);
        }
        for name in &decls.inventory {
            let decl = decls.s_decl(name).unwrap();
            let id = schema_ident(name);
            let canonical = decl.fields.iter().any(|f| f.name == "source_support");
            if schema_role(id.as_str()).unwrap() == SchemaRole::Semantic {
                assert!(canonical || aliased.contains(id.as_str()), "{name}");
            }
            if canonical {
                assert!(!aliased.contains(id.as_str()), "{name}");
            }
        }
    }

    /// Test-local depth-0 splitter over one S-decl source line, returning
    /// the field type strings — the independent scan for root-level row
    /// counts (field types contain no parens; `<`/`>` only in `Text<p>`).
    fn root_field_types(line: &str) -> Vec<&str> {
        let inner = &line[line.find('(').unwrap() + 1..line.rfind(')').unwrap()];
        let mut fields = Vec::new();
        let (mut depth, mut start) = (0i32, 0usize);
        for (i, c) in inner.char_indices() {
            match c {
                '[' | '<' => depth += 1,
                ']' | '>' => depth -= 1,
                ',' if depth == 0 => {
                    fields.push(&inner[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        fields.push(&inner[start..]);
        fields
            .iter()
            .map(|f| f.split_once(':').unwrap().1)
            .collect()
    }

    /// Binding/bound totals equal their walked leaf families, and per-root
    /// single-segment counts equal an independent depth-0 scan of each
    /// inventory S-decl source line (every root-line `Text<` is a
    /// single-segment binding; nested-decl sites have longer paths).
    #[test]
    fn build_row_counts_vs_line_scan() {
        let text = spec_text();
        let parse = parse_spec(&text);
        let decls = &parse.decls;
        let bindings = binding_rows(decls);
        let bounds = bound_rows(decls);

        let rows = walk_inventory(decls);
        assert_eq!(
            bindings.len(),
            rows.iter()
                .filter(|r| matches!(r.leaf, WalkedLeaf::Text { .. }))
                .count()
        );
        assert_eq!(
            bounds.len(),
            rows.iter()
                .filter(|r| matches!(
                    r.leaf,
                    WalkedLeaf::Collection {
                        enum_domain_key: false
                    }
                ))
                .count()
        );

        for name in &decls.inventory {
            let decl = decls.s_decl(name).unwrap();
            let line = text.lines().nth(decl.line - 1).unwrap();
            let id = schema_ident(name);
            let n_coll = root_field_types(line)
                .iter()
                .filter(|t| ["Set[", "List[", "Map["].iter().any(|p| t.starts_with(p)))
                .count();
            assert_eq!(
                bounds
                    .iter()
                    .filter(|b| b.schema_id == id && b.path.segments().len() == 1)
                    .count(),
                n_coll,
                "{name} bounds"
            );
            assert_eq!(
                bindings
                    .iter()
                    .filter(|b| b.schema_id == id && b.path.segments().len() == 1)
                    .count(),
                line.matches("Text<").count(),
                "{name} bindings"
            );
        }
    }

    /// Binding policies at root and nested paths (all static, no dependent
    /// sibling fields) and bound max-items tiers (default, whole-root,
    /// per-path) under the uniform v0 disposition.
    #[test]
    fn build_binding_bound_spot_checks() {
        let parse = parse_spec(&spec_text());
        let bindings = binding_rows(&parse.decls);
        let bounds = bound_rows(&parse.decls);
        let joined = |p: &FeaturePath| {
            p.segments()
                .iter()
                .map(Id::as_str)
                .collect::<Vec<_>>()
                .join("/")
        };

        assert!(bindings.iter().all(|b| b.dependent_policy_field.is_none()));
        for (schema, path, policy) in [
            ("source_span", "raw_text", StringPolicy::RawSource),
            ("mech_obs_payload", "fields", StringPolicy::SemanticJa),
            (
                "unicode_policy_manifest",
                "unicode_version",
                StringPolicy::IdentifierAscii,
            ),
            ("source_graph", "spans/raw_text", StringPolicy::RawSource),
            (
                "gloss_template",
                "literal_parts",
                StringPolicy::TemplateLiteral,
            ),
            ("residual", "diagnostic", StringPolicy::DiagnosticText),
        ] {
            let row = bindings
                .iter()
                .find(|b| b.schema_id.as_str() == schema && joined(&b.path) == path)
                .unwrap_or_else(|| panic!("missing binding {schema} {path}"));
            assert_eq!(row.policy, policy, "{schema} {path}");
        }

        assert!(
            bounds
                .iter()
                .all(|b| b.overflow_disposition == BoundOverflowDisposition::RejectWithDiagnostic)
        );
        for (schema, path, max) in [
            ("schema_registry", "schema_entries", DEFAULT_MAX_ITEMS),
            ("closure_output", "match_hashes", 1 << 20),
            ("source_graph", "spans", 1 << 20),
            (
                "region_closure_certificate",
                "added_member_batches",
                1 << 20,
            ),
            ("report_trace_index", "rows", 1 << 20),
        ] {
            let row = bounds
                .iter()
                .find(|b| b.schema_id.as_str() == schema && joined(&b.path) == path)
                .unwrap_or_else(|| panic!("missing bound {schema} {path}"));
            assert_eq!(row.max_items, UInt::from(max), "{schema} {path}");
        }
    }

    /// Step-4 enum-domain Map exemption: flagged keys yield no bound row.
    #[test]
    fn build_bound_enum_domain_exemption() {
        let mut decls = synthetic(
            "E Color = red | green | blue\n\
             S Board(cells:Map[Color,UInt],open:Map[Id,UInt])",
        );
        decls.inventory = vec!["Board".to_string()];
        let rows = bound_rows(&decls);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].path.segments()[0].as_str(), "open");
    }

    /// v0 assembly: real spec-contract and bound-manifest hashes, per-field
    /// §-anchor placeholders, set sizes match the row builders, and the
    /// composed records roundtrip through canonical bytes.
    #[test]
    fn build_v0_registry_assembly_roundtrip() {
        let text = spec_text();
        let parse = parse_spec(&text);
        let decls = &parse.decls;
        let (registry, manifest) = build_v0_registry(decls, &text);

        assert_eq!(registry.spec_contract_hash, Hash::of_bytes(text.as_bytes()));
        let manifest_bytes = canonical_payload_bytes(&manifest).unwrap();
        assert_eq!(
            registry.schema_bound_manifest_hash,
            Hash::of_bytes(&manifest_bytes)
        );
        let anchor =
            |p: &str| Hash::of_bytes(text.lines().find(|l| l.starts_with(p)).unwrap().as_bytes());
        assert_eq!(registry.rust_type_manifest_hash, anchor("### 1.1 "));
        assert_eq!(
            registry.generated_json_schema_manifest_hash,
            anchor("### 1.1 ")
        );
        assert_eq!(registry.canonicalization_policy_hash, anchor("### 1.5 "));
        assert_eq!(
            manifest.generator_static_bound_policy_hash,
            anchor("### 6.1 ")
        );
        assert_eq!(manifest.parser_bound_policy_hash, anchor("### 6.2 "));
        assert_eq!(manifest.closure_bound_policy_hash, anchor("### 7.1 "));

        assert_eq!(registry.schema_entries.len(), decls.inventory.len());
        assert_eq!(
            registry.string_policy_bindings.len(),
            binding_rows(decls).len()
        );
        assert_eq!(
            registry.source_support_aliases.len(),
            alias_rows(decls).len()
        );
        assert_eq!(
            manifest.schema_collection_bounds.len(),
            bound_rows(decls).len()
        );

        assert_eq!(
            from_canonical_bytes::<SchemaRegistry>(&canonical_payload_bytes(&registry).unwrap())
                .unwrap(),
            registry
        );
        assert_eq!(
            from_canonical_bytes::<SchemaBoundManifest>(&manifest_bytes).unwrap(),
            manifest
        );
    }

    /// Alias priority: default-table order picks source_regions over
    /// subject_hash; walk order breaks same-kind ties; canonical
    /// source_support suppresses the row.
    #[test]
    fn build_alias_priority_synthetic() {
        let mut decls = synthetic(
            "S Root(subject_hash:Hash,source_regions:Set[RegionId],note:Id)\n\
             S Pair(a_rows:Set[Sub],b_rows:Set[Sub])\n\
             S Sub(source_region_ids:Set[RegionId])\n\
             S Canon(source_support:Set[RegionId],subject_hash:Hash)",
        );
        decls.inventory = ["Root", "Pair", "Canon"].map(String::from).to_vec();
        let rows = alias_rows(&decls);
        assert_eq!(
            rows,
            [
                SourceSupportAlias {
                    schema_id: Id::new("root").unwrap(),
                    path: FeaturePath::new(vec![Id::new("source_regions").unwrap()]),
                    alias_kind: SourceSupportAliasKind::RegionSet,
                },
                SourceSupportAlias {
                    schema_id: Id::new("pair").unwrap(),
                    path: FeaturePath::new(vec![
                        Id::new("a_rows").unwrap(),
                        Id::new("source_region_ids").unwrap()
                    ]),
                    alias_kind: SourceSupportAliasKind::RegionSet,
                },
            ]
        );
    }
}
