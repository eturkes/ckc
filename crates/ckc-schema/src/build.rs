//! §1.1 steps 3-4 substrate (M0.0.3.4.1): the type-graph walk from each
//! §3.1 inventory root, yielding [`WalkedPath`] rows that .4.2 (entry +
//! alias authoring), .4.3 (binding/bound rows), and .4.4 (step-4 coverage
//! and §1.2 hash-convention checks) consume.
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

use ckc_core::scalar::{FeaturePath, Id};

use crate::registry::SourceSupportAliasKind;
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

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
}
