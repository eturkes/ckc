//! §1.1 step 3 (M0.0.3.3): resolution of every spec-declared reference
//! against the [`crate::symtab`] symbol table, with checker-local sorted
//! diagnostics. §1.1 step 4 (M0.0.3.4.4): [`check_registry`] validates a
//! built registry+manifest pair — two-sided bound coverage over the §3.1
//! type-graph walk and the §1.2 source-support/role rule. Every issue
//! carries [`CheckIssue::CODE`] (`referential_integrity_error`); the §8.7
//! `Diagnostic` artifact wrapper lands with its consuming unit.
//!
//! Checked references: S-decl field types and E-decl alternative/sexp
//! argument types (generic-arity and string-policy aware); tagged-union
//! `Ref` alternatives; builtin <-> §6.2 definition bijection;
//! certificate-class <-> §9.2 obligation bijection; §3.3 gate-table and
//! §7.2 rule-table keys against their canonical enums plus evidence-object
//! and conclusion columns; §3.2 stage-table emissions; §11.1 command
//! operations and outputs; §11.3 unit/gate bijection, dependency closure,
//! and deliverable names; §11.4 unit coverage; §2.1 consumer keys; §3.1
//! inventory names; body-wide `§`/`A.N` anchor references. Table cells mix
//! prose with names, so only single capitalized type-shaped tokens resolve
//! in artifact columns (`schema diagnostics` and `Appendix A accepted
//! artifact inventory` are skipped, never resolution errors).

use std::collections::{BTreeMap, BTreeSet};

use ckc_core::policy::StringPolicy;
use ckc_core::scalar::{FeaturePath, Id};

use crate::bounds::SchemaBoundManifest;
use crate::build::{WalkedLeaf, schema_has_source_support, schema_ident, walk_inventory};
use crate::registry::{SchemaRegistry, SchemaRole};
use crate::spec::{EAlt, SDecl, SpecDecls, TTable, TypeExpr, is_type_name, parse_spec};
use crate::symtab::{
    SymbolKind, SymbolTable, build_symbol_table, command_table, consumer_table, gate_table,
    operation_tokens, reading_table, rule_table, stage_table, unit_table,
};

/// One referential-integrity finding. Ordering (line, anchor, message) is
/// the canonical report order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CheckIssue {
    pub line: usize,
    /// Enclosing section anchor of the offending reference.
    pub anchor: String,
    pub message: String,
}

impl CheckIssue {
    /// §1.1 step 5 diagnostic code shared by every issue.
    pub const CODE: &'static str = "referential_integrity_error";
}

/// Sorted issues; empty means the spec resolves (§1.1 step-5 `ok`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckReport {
    pub issues: Vec<CheckIssue>,
}

impl CheckReport {
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Runs §1.1 steps 1-3 over raw SPEC.md text: parse, build the symbol
/// table, resolve every reference. Step 4 lives in [`check_registry`];
/// step-5 composition lands with M0.0.3.4.7.
pub fn check_spec(text: &str) -> CheckReport {
    let parse = parse_spec(text);
    let decls = &parse.decls;
    let mut issues = Vec::new();
    let mut push = |line: usize, message: String| {
        issues.push(CheckIssue {
            line,
            anchor: enclosing_anchor(decls, line),
            message,
        });
    };

    for p in &parse.issues {
        push(p.line, format!("parse: {}", p.message));
    }

    let (table, duplicates) = build_symbol_table(decls);
    for d in &duplicates {
        push(
            d.line,
            format!(
                "duplicate {} `{}` declared under `{}` and `{}`",
                d.kind.as_str(),
                d.id,
                d.first_anchor,
                d.second_anchor
            ),
        );
    }

    resolve_decl_types(decls, &table, &mut push);
    check_builtin_bijection(decls, &table, &mut push);
    check_certificate_bijection(decls, &table, &mut push);
    check_gate_table(decls, &table, &mut push);
    check_rule_table(decls, &table, &mut push);
    check_stage_table(decls, &table, &mut push);
    check_command_table(decls, &table, &mut push);
    check_build_unit_tables(decls, &table, &mut push);
    check_consumer_table(decls, &table, &mut push);
    check_inventory(decls, &table, &mut push);
    check_anchor_refs(text, &table, &mut push);

    issues.sort();
    issues.dedup();
    CheckReport { issues }
}

/// Last section header at or before `line`.
fn enclosing_anchor(decls: &SpecDecls, line: usize) -> String {
    decls
        .sections
        .iter()
        .rev()
        .find(|s| s.line <= line)
        .map_or_else(|| "title".to_string(), |s| s.anchor.clone())
}

/// §1.1 step-4 bound coverage and the §1.2 source-support/role rule over a
/// built registry+manifest pair. Peer of [`check_spec`]: same issue type
/// and ordering. The §1.2 hash-field-convention and §3.2 producer-mapping
/// checkers plus step-1-2/5 composition land with M0.0.3.4.5-7.
pub fn check_registry(
    text: &str,
    registry: &SchemaRegistry,
    manifest: &SchemaBoundManifest,
) -> CheckReport {
    let parse = parse_spec(text);
    let decls = &parse.decls;
    let inv = inventory_decls(decls);
    let inv_line = decls
        .sections
        .iter()
        .find(|s| s.anchor == "3.1")
        .map_or(0, |s| s.line);
    let line_of = |schema: &str| inv.get(schema).map_or(inv_line, |d| d.line);
    let mut issues = Vec::new();
    let mut push = |line: usize, message: String| {
        issues.push(CheckIssue {
            line,
            anchor: enclosing_anchor(decls, line),
            message,
        });
    };

    // §1.1 step 4, two-sided: every walked non-exempt collection field has
    // exactly one bound row, and every bound row sits on such a field.
    let walked: BTreeSet<(String, String)> = walk_inventory(decls)
        .into_iter()
        .filter(|r| {
            matches!(
                r.leaf,
                WalkedLeaf::Collection {
                    enum_domain_key: false
                }
            )
        })
        .map(|r| (r.schema_id.as_str().to_string(), joined(&r.path)))
        .collect();
    let mut bound_keys: BTreeMap<(String, String), usize> = BTreeMap::new();
    for b in &manifest.schema_collection_bounds {
        *bound_keys
            .entry((b.schema_id.as_str().to_string(), joined(&b.path)))
            .or_insert(0) += 1;
    }
    for key @ (schema, path) in &walked {
        match bound_keys.get(key).copied().unwrap_or(0) {
            1 => {}
            0 => push(
                line_of(schema),
                format!("collection `{schema}/{path}` has no SchemaCollectionBound row"),
            ),
            n => push(
                line_of(schema),
                format!("collection `{schema}/{path}` has {n} SchemaCollectionBound rows"),
            ),
        }
    }
    for key @ (schema, path) in bound_keys.keys() {
        if !walked.contains(key) {
            push(
                line_of(schema),
                format!(
                    "SchemaCollectionBound `{schema}/{path}` matches no bounded collection field"
                ),
            );
        }
    }

    // §1.2 role rule: a schema with neither source-support exposure over
    // its direct fields nor a registered alias must keep a non-semantic
    // `schema_role`.
    let aliased: BTreeSet<&str> = registry
        .source_support_aliases
        .iter()
        .map(|a| a.schema_id.as_str())
        .collect();
    for e in &registry.schema_entries {
        let id = e.schema_id.as_str();
        let Some(decl) = inv.get(id).copied() else {
            push(
                inv_line,
                format!("schema entry `{id}` resolves to no §3.1 inventory S-decl"),
            );
            continue;
        };
        if e.schema_role == SchemaRole::Semantic
            && !schema_has_source_support(decl)
            && !aliased.contains(id)
        {
            push(
                decl.line,
                format!(
                    "schema `{id}` has schema_role `semantic` with neither source-support \
                     field nor registered alias"
                ),
            );
        }
    }

    issues.sort();
    issues.dedup();
    CheckReport { issues }
}

/// schema_id -> §3.1 inventory S-decl (issue lines + role-rule input).
fn inventory_decls(decls: &SpecDecls) -> BTreeMap<String, &SDecl> {
    decls
        .inventory
        .iter()
        .filter_map(|n| Some((schema_ident(n).as_str().to_string(), decls.s_decl(n)?)))
        .collect()
}

/// `/`-joined `FeaturePath` segments (§1.1 diagnostic display convention).
fn joined(path: &FeaturePath) -> String {
    path.segments()
        .iter()
        .map(Id::as_str)
        .collect::<Vec<_>>()
        .join("/")
}

/// Resolves one type expression. `generic` is the enclosing declaration's
/// parameter; `siblings` are S-decl field names for dependent `Text<field>`
/// policies (§1.4 `TextLiteral.policy`).
fn resolve_type(
    ty: &TypeExpr,
    generic: Option<&str>,
    siblings: &[&str],
    table: &SymbolTable,
    line: usize,
    push: &mut impl FnMut(usize, String),
) {
    match ty {
        TypeExpr::Name { name, arg } => {
            if Some(name.as_str()) == generic {
                if arg.is_some() {
                    push(
                        line,
                        format!("generic parameter `{name}` takes no argument"),
                    );
                }
                return;
            }
            let schema = table.get(SymbolKind::Schema, name);
            let enm = table.get(SymbolKind::Enum, name);
            match (schema, enm) {
                (Some(_), Some(_)) => push(
                    line,
                    format!("type `{name}` resolves to both a schema and an enum"),
                ),
                (None, None) => push(line, format!("unresolved type `{name}`")),
                (Some(sym), None) | (None, Some(sym)) => {
                    if sym.generic && arg.is_none() {
                        push(line, format!("generic type `{name}` used without argument"));
                    }
                    if !sym.generic && arg.is_some() {
                        push(line, format!("type `{name}` is not generic"));
                    }
                }
            }
            if let Some(a) = arg {
                resolve_type(a, generic, siblings, table, line, push);
            }
        }
        TypeExpr::Set(x) | TypeExpr::List(x) | TypeExpr::Optional(x) => {
            resolve_type(x, generic, siblings, table, line, push);
        }
        TypeExpr::Map(k, v) => {
            resolve_type(k, generic, siblings, table, line, push);
            resolve_type(v, generic, siblings, table, line, push);
        }
        TypeExpr::Text(p) => {
            if StringPolicy::from_id(p).is_none() && !siblings.contains(&p.as_str()) {
                push(
                    line,
                    format!("unresolved string policy or sibling field `Text<{p}>`"),
                );
            }
        }
    }
}

/// S-decl fields; E-decl named/sexp alternative types; tagged-union `Ref`
/// alternatives must name a type (pure capitalized variants resolve only in
/// label enums, e.g. `E JudgmentKind = ... | NF | ...`).
fn resolve_decl_types(
    decls: &SpecDecls,
    table: &SymbolTable,
    push: &mut impl FnMut(usize, String),
) {
    for d in &decls.s_decls {
        let siblings: Vec<&str> = d.fields.iter().map(|f| f.name.as_str()).collect();
        for f in &d.fields {
            resolve_type(
                &f.ty,
                d.generic_param.as_deref(),
                &siblings,
                table,
                d.line,
                push,
            );
        }
    }
    for d in &decls.e_decls {
        let generic = d.generic_param.as_deref();
        let is_union = d
            .alts
            .iter()
            .any(|a| matches!(a, EAlt::Named { .. } | EAlt::Sexp { .. }));
        for a in &d.alts {
            match a {
                EAlt::Named { ty, .. } => resolve_type(ty, generic, &[], table, d.line, push),
                EAlt::Sexp { args, .. } => {
                    for arg in args {
                        resolve_type(arg, generic, &[], table, d.line, push);
                    }
                }
                EAlt::Ref(t) if is_union && table.type_target(t).is_none() => push(
                    d.line,
                    format!("unresolved union alternative `{t}` in `{}`", d.name),
                ),
                EAlt::Ref(_) | EAlt::Bare(_) => {}
            }
        }
    }
}

/// Two-sided set comparison; emits one issue per unmatched member.
fn check_bijection(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
    left_desc: &str,
    right_desc: &str,
    line: usize,
    push: &mut impl FnMut(usize, String),
) {
    for l in left.difference(right) {
        push(line, format!("{left_desc} `{l}` has no {right_desc}"));
    }
    for r in right.difference(left) {
        push(line, format!("{right_desc} `{r}` has no {left_desc}"));
    }
}

fn kind_ids(table: &SymbolTable, kind: SymbolKind) -> BTreeSet<String> {
    table.ids(kind).map(String::from).collect()
}

/// `E BuiltinName` variants <-> §6.2 builtin definitions.
fn check_builtin_bijection(
    decls: &SpecDecls,
    table: &SymbolTable,
    push: &mut impl FnMut(usize, String),
) {
    let line = decls.e_decl("BuiltinName").map_or(0, |d| d.line);
    check_bijection(
        &kind_ids(table, SymbolKind::Builtin),
        &decls.builtin_defs.iter().cloned().collect(),
        "builtin",
        "§6.2 builtin definition",
        line,
        push,
    );
}

/// `E M0CertificateClass` variants <-> §9.2 obligation labels.
fn check_certificate_bijection(
    decls: &SpecDecls,
    table: &SymbolTable,
    push: &mut impl FnMut(usize, String),
) {
    let line = decls.e_decl("M0CertificateClass").map_or(0, |d| d.line);
    check_bijection(
        &kind_ids(table, SymbolKind::CertificateClass),
        &decls.certificate_classes.iter().cloned().collect(),
        "certificate class",
        "§9.2 certificate obligation",
        line,
        push,
    );
}

fn require_table<'a>(
    found: Option<&'a TTable>,
    desc: &str,
    push: &mut impl FnMut(usize, String),
) -> Option<&'a TTable> {
    if found.is_none() {
        push(0, format!("canonical table missing: {desc}"));
    }
    found
}

/// Comma-separated artifact cell: single capitalized type-shaped tokens
/// resolve as schema/enum names; multi-word or lowercase items are prose.
fn resolve_artifact_cell(
    cell: &str,
    line: usize,
    table: &SymbolTable,
    push: &mut impl FnMut(usize, String),
) {
    for item in cell.split(',').map(str::trim) {
        if is_type_name(item) && table.type_target(item).is_none() {
            push(line, format!("unresolved artifact name `{item}`"));
        }
    }
}

/// §3.3 gate table: keys <-> `E Gate` variants; evidence objects resolve.
fn check_gate_table(decls: &SpecDecls, table: &SymbolTable, push: &mut impl FnMut(usize, String)) {
    let Some(t) = require_table(gate_table(decls), "§3.3 T Gate", push) else {
        return;
    };
    check_bijection(
        &kind_ids(table, SymbolKind::Gate),
        &t.rows.iter().map(|r| r[0].clone()).collect(),
        "gate",
        "§3.3 gate-table row",
        t.line,
        push,
    );
    for (i, row) in t.rows.iter().enumerate() {
        for obj in row[2].split(" and ").map(str::trim) {
            if !table.contains(SymbolKind::Schema, obj) {
                push(
                    t.line + 1 + i,
                    format!("unresolved gate evidence object `{obj}`"),
                );
            }
        }
    }
}

/// §7.2 rule table: keys <-> `E ProofRule` variants; `Emitted for`
/// conclusions resolve as schemas.
fn check_rule_table(decls: &SpecDecls, table: &SymbolTable, push: &mut impl FnMut(usize, String)) {
    let Some(t) = require_table(rule_table(decls), "§7.2 T Rule", push) else {
        return;
    };
    check_bijection(
        &kind_ids(table, SymbolKind::ProofRule),
        &t.rows.iter().map(|r| r[0].clone()).collect(),
        "proof rule",
        "§7.2 rule-table row",
        t.line,
        push,
    );
    for (i, row) in t.rows.iter().enumerate() {
        resolve_artifact_cell(&row[1], t.line + 1 + i, table, push);
    }
}

/// §3.2 stage table: emitted accepted artifacts resolve (operations are
/// declared by this table and consumed by §11.1 resolution).
fn check_stage_table(decls: &SpecDecls, table: &SymbolTable, push: &mut impl FnMut(usize, String)) {
    let Some(t) = require_table(stage_table(decls), "§3.2 T Stage", push) else {
        return;
    };
    for (i, row) in t.rows.iter().enumerate() {
        resolve_artifact_cell(&row[3], t.line + 1 + i, table, push);
    }
}

/// §11.1 command table: every pipeline operation resolves to a §3.2 stage
/// operation (or a `T-*` acceptance gate); primary emitted artifacts
/// resolve.
fn check_command_table(
    decls: &SpecDecls,
    table: &SymbolTable,
    push: &mut impl FnMut(usize, String),
) {
    let Some(t) = require_table(command_table(decls), "§11.1 T Command", push) else {
        return;
    };
    for (i, row) in t.rows.iter().enumerate() {
        let line = t.line + 1 + i;
        for op in operation_tokens(&row[1]) {
            let kind = if op.starts_with("T-") {
                SymbolKind::AcceptanceGate
            } else {
                SymbolKind::Stage
            };
            if !table.contains(kind, op) {
                push(line, format!("unresolved pipeline operation `{op}`"));
            }
        }
        resolve_artifact_cell(&row[2], line, table, push);
    }
}

/// §11.3 unit table: each row's gate resolves and is assigned exactly once,
/// gates biject with obligation rows, dependencies close over unit ids;
/// §11.4 reading rows biject with units. The deliverable column is prose
/// (it names operations and fixtures alongside schemas) and is unchecked.
fn check_build_unit_tables(
    decls: &SpecDecls,
    table: &SymbolTable,
    push: &mut impl FnMut(usize, String),
) {
    let Some(units) = require_table(unit_table(decls), "§11.3 T Unit", push) else {
        return;
    };
    let unit_ids: BTreeSet<String> = units.rows.iter().map(|r| r[0].clone()).collect();
    let mut gates = BTreeSet::new();
    for (i, row) in units.rows.iter().enumerate() {
        let line = units.line + 1 + i;
        if !table.contains(SymbolKind::AcceptanceGate, &row[3]) {
            push(line, format!("unresolved acceptance gate `{}`", row[3]));
        }
        if !gates.insert(row[3].clone()) {
            push(
                line,
                format!("acceptance gate `{}` assigned to multiple units", row[3]),
            );
        }
        if row[2] != "none" && !unit_ids.contains(&row[2]) {
            push(line, format!("unresolved unit dependency `{}`", row[2]));
        }
    }
    check_bijection(
        &gates,
        &kind_ids(table, SymbolKind::AcceptanceGate),
        "unit-table gate",
        "obligation-table gate",
        units.line,
        push,
    );
    if let Some(reading) = require_table(reading_table(decls), "§11.4 T Unit reading map", push) {
        check_bijection(
            &unit_ids,
            &reading.rows.iter().map(|r| r[0].clone()).collect(),
            "§11.3 unit",
            "§11.4 reading-map row",
            reading.line,
            push,
        );
    }
}

/// §2.1 vocabulary-consumer keys resolve as shared enums.
fn check_consumer_table(
    decls: &SpecDecls,
    table: &SymbolTable,
    push: &mut impl FnMut(usize, String),
) {
    let Some(t) = require_table(consumer_table(decls), "§2.1 T Vocabulary", push) else {
        return;
    };
    for (i, row) in t.rows.iter().enumerate() {
        for name in row[0].split(" and ").map(str::trim) {
            if !table.contains(SymbolKind::Enum, name) {
                push(
                    t.line + 1 + i,
                    format!("unresolved vocabulary enum `{name}`"),
                );
            }
        }
    }
}

/// §3.1 required-payload inventory rows resolve as schemas.
fn check_inventory(decls: &SpecDecls, table: &SymbolTable, push: &mut impl FnMut(usize, String)) {
    let line = decls
        .sections
        .iter()
        .find(|s| s.anchor == "3.1")
        .map_or(0, |s| s.line);
    for name in &decls.inventory {
        if !table.contains(SymbolKind::Schema, name) {
            push(line, format!("unresolved inventory payload `{name}`"));
        }
    }
}

/// Body-wide `§N[.M]` / `§§X-Y` / `A.N` references resolve to section
/// anchors. Scans every line, fenced blocks included.
fn check_anchor_refs(text: &str, table: &SymbolTable, push: &mut impl FnMut(usize, String)) {
    for (i, line) in text.lines().enumerate() {
        let lineno = i + 1;
        // Header lines declare anchors rather than referencing them.
        if line.starts_with('#') {
            continue;
        }
        for anchor in anchor_refs(line) {
            if !table.contains(SymbolKind::SectionAnchor, &anchor) {
                push(lineno, format!("unresolved section reference `§{anchor}`"));
            }
        }
    }
}

/// Extracts referenced anchors from one line: `§1.5`, `§§7.3-9.3` (both
/// endpoints), and appendix `A.10` tokens at word boundaries.
fn anchor_refs(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '§' {
            let mut j = i + 1;
            if chars.get(j) == Some(&'§') {
                j += 1;
            }
            if let Some((anchor, next)) = numeric_anchor(&chars, j) {
                out.push(anchor);
                // Range endpoint: `-` followed by a digit.
                if chars.get(next) == Some(&'-')
                    && chars.get(next + 1).is_some_and(char::is_ascii_digit)
                    && let Some((second, next2)) = numeric_anchor(&chars, next + 1)
                {
                    out.push(second);
                    i = next2;
                    continue;
                }
                i = next;
                continue;
            }
        }
        if chars[i] == 'A'
            && (i == 0 || !chars[i - 1].is_ascii_alphanumeric())
            && chars.get(i + 1) == Some(&'.')
            && chars.get(i + 2).is_some_and(char::is_ascii_digit)
        {
            let mut j = i + 2;
            while chars.get(j).is_some_and(char::is_ascii_digit) {
                j += 1;
            }
            out.push(format!("A.{}", chars[i + 2..j].iter().collect::<String>()));
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

/// Parses `digits(.digits)*` at `pos`; a trailing `.` not followed by a
/// digit is punctuation.
fn numeric_anchor(chars: &[char], pos: usize) -> Option<(String, usize)> {
    let mut j = pos;
    while chars.get(j).is_some_and(char::is_ascii_digit) {
        j += 1;
    }
    if j == pos {
        return None;
    }
    while chars.get(j) == Some(&'.') && chars.get(j + 1).is_some_and(char::is_ascii_digit) {
        j += 1;
        while chars.get(j).is_some_and(char::is_ascii_digit) {
            j += 1;
        }
    }
    Some((chars[pos..j].iter().collect(), j))
}

#[cfg(test)]
mod tests {
    use ckc_core::scalar::UInt;

    use crate::bounds::SchemaCollectionBound;
    use crate::build::build_v0_registry;
    use crate::registry::SchemaEntry;

    use super::*;

    fn spec_text() -> String {
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../SPEC.md")).unwrap()
    }

    /// Gate core: the real SPEC resolves clean, and symbol classification
    /// lands every kind (union alternative vs capitalized label variant,
    /// derived builtin/rule/certificate/gate kinds, table-declared kinds).
    #[test]
    fn check_real_spec_resolves_clean() {
        let text = spec_text();
        let report = check_spec(&text);
        assert!(report.is_clean(), "issues: {:#?}", report.issues);

        let (table, duplicates) = build_symbol_table(&parse_spec(&text).decls);
        assert!(duplicates.is_empty(), "{duplicates:#?}");
        for (kind, id) in [
            (SymbolKind::Schema, "SchemaRegistry"),
            (SymbolKind::Enum, "Outcome"),
            (SymbolKind::UnionAlternative, "Term.VarTerm"),
            (SymbolKind::UnionAlternative, "Premise.builtin"),
            (SymbolKind::UnionAlternative, "OperationResult.success"),
            (SymbolKind::EnumVariant, "Effect.Inference"),
            (SymbolKind::EnumVariant, "JudgmentKind.NF"),
            (SymbolKind::EnumVariant, "ClaimTier.S0"),
            (SymbolKind::Builtin, "normalize_context"),
            (SymbolKind::ProofRule, "MECH_OBS"),
            (SymbolKind::CertificateClass, "closed_nf"),
            (SymbolKind::Gate, "G-S3"),
            (SymbolKind::AcceptanceGate, "T-Demo-M0-Replay"),
            (SymbolKind::CliOperation, "ckc demo m0"),
            (SymbolKind::Stage, "BuildMatchClasses"),
            (SymbolKind::Stage, "source_region_closure"),
            (SymbolKind::SectionAnchor, "A.10"),
        ] {
            assert!(table.contains(kind, id), "missing {} `{id}`", kind.as_str());
        }
        // The §7.2 judgment-kind labels stay variants, not dangling refs.
        assert!(table.contains(SymbolKind::UnionAlternative, "JudgmentKind.SourceGraph"));
    }

    /// Synthetic §13 block: duplicate schema under a divergent anchor,
    /// dangling field type, dangling string policy, dangling union
    /// alternatives, dangling `§`/`A.N` references. Issues stay sorted.
    #[test]
    fn check_duplicate_and_dangling_refs_reject() {
        let text = format!(
            "{}\n## 13. Synthetic\n\n```text\nS SchemaRegistry(x:Id)\n\
             S Dangle(a:NoSuchType,b:Text<no_such_policy>)\n\
             E BadUnion = ok:NoSuchPayload | Whoops\n```\n\nSee §99.9 and A.99.\n",
            spec_text()
        );
        let report = check_spec(&text);
        for needle in [
            "duplicate schema `SchemaRegistry` declared under `1.1` and `13`",
            "unresolved type `NoSuchType`",
            "unresolved string policy or sibling field `Text<no_such_policy>`",
            "unresolved type `NoSuchPayload`",
            "unresolved union alternative `Whoops` in `BadUnion`",
            "unresolved section reference `§99.9`",
            "unresolved section reference `§A.99`",
        ] {
            assert!(
                report.issues.iter().any(|i| i.message == needle),
                "missing `{needle}` in {:#?}",
                report.issues
            );
        }
        assert_eq!(report.issues.len(), 7);
        assert!(report.issues.windows(2).all(|w| w[0] <= w[1]));
        assert!(report.issues.iter().all(|i| i.anchor == "13"));
    }

    /// Bijection perturbations: renaming one `E BuiltinName` variant breaks
    /// the §6.2 definition bijection both ways; renaming one unit-table
    /// gate cell breaks resolution and the §11.3 unit/gate bijection.
    #[test]
    fn check_bijection_perturbations_reject() {
        let builtin = spec_text().replace(" = support_of | ", " = support_off | ");
        let report = check_spec(&builtin);
        for needle in [
            "builtin `support_off` has no §6.2 builtin definition",
            "§6.2 builtin definition `support_of` has no builtin",
        ] {
            assert!(report.issues.iter().any(|i| i.message == needle));
        }

        let gate = spec_text().replace("|M0.0.1|T-Canonical-Bytes", "|M0.0.1|T-Canonical-Bytez");
        let report = check_spec(&gate);
        for needle in [
            "unresolved acceptance gate `T-Canonical-Bytez`",
            "unit-table gate `T-Canonical-Bytez` has no obligation-table gate",
            "obligation-table gate `T-Canonical-Bytes` has no unit-table gate",
        ] {
            assert!(
                report.issues.iter().any(|i| i.message == needle),
                "missing `{needle}` in {:#?}",
                report.issues
            );
        }
    }

    /// Column-check sensitivity: a renamed §3.2 emitted artifact and a
    /// renamed §11.1 pipeline operation each produce exactly the targeted
    /// issue (the cell checks scan real rows rather than skipping them).
    #[test]
    fn check_table_cell_perturbations_reject() {
        let artifact = spec_text().replace(
            "MechObsPayload, AnalyzerManifest, MechanicalLexicon",
            "MechObsPayload, AnalyzerManifezt, MechanicalLexicon",
        );
        let report = check_spec(&artifact);
        assert_eq!(
            report
                .issues
                .iter()
                .map(|i| i.message.as_str())
                .collect::<Vec<_>>(),
            ["unresolved artifact name `AnalyzerManifezt`"]
        );

        let operation =
            spec_text().replace("ckc replay | ReplayIdentity |", "ckc replay | Replay |");
        let report = check_spec(&operation);
        assert_eq!(
            report
                .issues
                .iter()
                .map(|i| i.message.as_str())
                .collect::<Vec<_>>(),
            ["unresolved pipeline operation `Replay`"]
        );
    }

    fn messages(report: &CheckReport) -> Vec<&str> {
        report.issues.iter().map(|i| i.message.as_str()).collect()
    }

    /// Gate core (registry side): the built v0 registry+manifest pass the
    /// step-4 and role-rule checks over the real SPEC.
    #[test]
    fn check_registry_real_spec_clean() {
        let text = spec_text();
        let decls = parse_spec(&text).decls;
        let (registry, manifest) = build_v0_registry(&decls, &text);
        let report = check_registry(&text, &registry, &manifest);
        assert!(report.is_clean(), "issues: {:#?}", report.issues);
    }

    /// Step-4 perturbations: dropped, duplicate-keyed, and off-walk bound
    /// rows each produce exactly the targeted issue, anchored at the
    /// owning S-decl's section.
    #[test]
    fn check_registry_bound_perturbations_reject() {
        let text = spec_text();
        let decls = parse_spec(&text).decls;
        let (registry, manifest) = build_v0_registry(&decls, &text);
        let target = manifest
            .schema_collection_bounds
            .iter()
            .find(|b| {
                b.schema_id.as_str() == "schema_registry" && joined(&b.path) == "schema_entries"
            })
            .cloned()
            .unwrap();

        let mut dropped = manifest.clone();
        assert!(dropped.schema_collection_bounds.remove(&target));
        let report = check_registry(&text, &registry, &dropped);
        assert_eq!(
            messages(&report),
            ["collection `schema_registry/schema_entries` has no SchemaCollectionBound row"]
        );
        assert_eq!(report.issues[0].anchor, "1.1");

        let mut duplicated = manifest.clone();
        assert!(
            duplicated
                .schema_collection_bounds
                .insert(SchemaCollectionBound {
                    max_items: UInt::from(7u64),
                    ..target.clone()
                })
        );
        let report = check_registry(&text, &registry, &duplicated);
        assert_eq!(
            messages(&report),
            ["collection `schema_registry/schema_entries` has 2 SchemaCollectionBound rows"]
        );

        let mut extra = manifest.clone();
        assert!(
            extra
                .schema_collection_bounds
                .insert(SchemaCollectionBound {
                    path: FeaturePath::new(vec![Id::new("no_such_field").unwrap()]),
                    ..target
                })
        );
        let report = check_registry(&text, &registry, &extra);
        assert_eq!(
            messages(&report),
            [
                "SchemaCollectionBound `schema_registry/no_such_field` matches no bounded collection field"
            ]
        );
    }

    /// §1.2 role-rule perturbations: a no-support schema flipped to
    /// `semantic` and a nested-alias schema stripped of its registered
    /// alias both reject; a support-bearing schema in a non-semantic role
    /// stays clean (the rule constrains only the no-support direction); an
    /// entry naming no inventory S-decl rejects.
    #[test]
    fn check_registry_role_perturbations() {
        let text = spec_text();
        let decls = parse_spec(&text).decls;
        let (registry, manifest) = build_v0_registry(&decls, &text);
        let reroled = |id: &str, role: SchemaRole| {
            let mut r = registry.clone();
            let e = r
                .schema_entries
                .iter()
                .find(|e| e.schema_id.as_str() == id)
                .cloned()
                .unwrap();
            r.schema_entries.remove(&e);
            r.schema_entries.insert(SchemaEntry {
                schema_role: role,
                ..e
            });
            r
        };

        let report = check_registry(
            &text,
            &reroled("schema_registry", SchemaRole::Semantic),
            &manifest,
        );
        assert_eq!(
            messages(&report),
            [
                "schema `schema_registry` has schema_role `semantic` with neither source-support field nor registered alias"
            ]
        );

        // Registered-alias dependency: air_core_record exposes support only
        // through the nested air_key/support_region_id alias row.
        let mut dealiased = registry.clone();
        let alias = dealiased
            .source_support_aliases
            .iter()
            .find(|a| a.schema_id.as_str() == "air_core_record")
            .cloned()
            .unwrap();
        assert!(dealiased.source_support_aliases.remove(&alias));
        let report = check_registry(&text, &dealiased, &manifest);
        assert_eq!(
            messages(&report),
            [
                "schema `air_core_record` has schema_role `semantic` with neither source-support field nor registered alias"
            ]
        );

        let report = check_registry(&text, &reroled("residual", SchemaRole::ViewOnly), &manifest);
        assert!(report.is_clean(), "{:#?}", report.issues);

        let mut stray = registry.clone();
        let donor = stray.schema_entries.iter().next().cloned().unwrap();
        stray.schema_entries.insert(SchemaEntry {
            schema_id: Id::new("no_such_schema").unwrap(),
            ..donor
        });
        let report = check_registry(&text, &stray, &manifest);
        assert_eq!(
            messages(&report),
            ["schema entry `no_such_schema` resolves to no §3.1 inventory S-decl"]
        );
    }
}
