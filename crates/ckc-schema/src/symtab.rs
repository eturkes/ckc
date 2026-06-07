//! §1.1 steps 1-2 (M0.0.3.3): the symbol table over every name SPEC.md
//! declares, sorted by `(kind, id, anchor)`, with duplicate
//! `(kind, id)`-divergent-anchor rejection. Step-3 resolution lives in
//! [`crate::check`].
//!
//! Declaration sources are fixed: S-decls and §1.3 scalar axioms declare
//! `schema`; E-decls declare `enum` plus their qualified variants or
//! alternatives; the four canonical enums `BuiltinName`, `ProofRule`,
//! `M0CertificateClass`, and `Gate` additionally declare the unqualified
//! `builtin`/`proof_rule`/`certificate_class`/`gate` symbols; the §11.3
//! obligation table declares `acceptance_gate`; the §11.1 command table
//! declares `cli_operation`; the §3.2 stage-producer table declares `stage`
//! operations; markdown headers declare `section_anchor`. Other occurrences
//! of these names are references checked by [`crate::check`].

use std::collections::BTreeMap;

use crate::spec::{EAlt, SCALAR_AXIOMS, SpecDecls, TTable};

/// Symbol kinds in canonical sort order (§1.1 step 2 sorts by kind first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolKind {
    Schema,
    Enum,
    EnumVariant,
    UnionAlternative,
    Builtin,
    ProofRule,
    CertificateClass,
    Gate,
    AcceptanceGate,
    CliOperation,
    Stage,
    SectionAnchor,
}

impl SymbolKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SymbolKind::Schema => "schema",
            SymbolKind::Enum => "enum",
            SymbolKind::EnumVariant => "enum_variant",
            SymbolKind::UnionAlternative => "union_alternative",
            SymbolKind::Builtin => "builtin",
            SymbolKind::ProofRule => "proof_rule",
            SymbolKind::CertificateClass => "certificate_class",
            SymbolKind::Gate => "gate",
            SymbolKind::AcceptanceGate => "acceptance_gate",
            SymbolKind::CliOperation => "cli_operation",
            SymbolKind::Stage => "stage",
            SymbolKind::SectionAnchor => "section_anchor",
        }
    }
}

/// One declared symbol. `anchor` is the definition anchor of §1.1 step 2:
/// the enclosing section anchor, except `section_anchor` symbols, whose id
/// is their own anchor and whose definition anchor is the header title (so
/// two same-numbered sections with different titles diverge).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub id: String,
    pub anchor: String,
    pub line: usize,
    /// Schema/enum declared with a generic parameter (`ArtifactEnvelope<T>`,
    /// `OperationResult[T]`); reference arity must match.
    pub generic: bool,
}

/// Duplicate `(kind, id)` with divergent definition anchors (§1.1 step 2).
/// Same-anchor redeclaration is tolerated (the §1.3 `S Rational` decl
/// overlaps the scalar axiom at the same anchor).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateSymbol {
    pub kind: SymbolKind,
    pub id: String,
    pub first_anchor: String,
    pub second_anchor: String,
    pub line: usize,
}

/// Symbols keyed by `(kind, id)`; `BTreeMap` order is the §1.1 step-2 sort.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SymbolTable {
    symbols: BTreeMap<(SymbolKind, String), Symbol>,
}

impl SymbolTable {
    pub fn get(&self, kind: SymbolKind, id: &str) -> Option<&Symbol> {
        self.symbols.get(&(kind, id.to_string()))
    }

    pub fn contains(&self, kind: SymbolKind, id: &str) -> bool {
        self.get(kind, id).is_some()
    }

    /// Schema-or-enum type target for field/argument type resolution.
    pub fn type_target(&self, name: &str) -> Option<&Symbol> {
        self.get(SymbolKind::Schema, name)
            .or_else(|| self.get(SymbolKind::Enum, name))
    }

    pub fn ids(&self, kind: SymbolKind) -> impl Iterator<Item = &str> {
        self.symbols
            .range((kind, String::new())..)
            .take_while(move |((k, _), _)| *k == kind)
            .map(|((_, id), _)| id.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }

    fn insert(&mut self, sym: Symbol, duplicates: &mut Vec<DuplicateSymbol>) {
        let key = (sym.kind, sym.id.clone());
        match self.symbols.get(&key) {
            Some(first) if first.anchor != sym.anchor => duplicates.push(DuplicateSymbol {
                kind: sym.kind,
                id: sym.id,
                first_anchor: first.anchor.clone(),
                second_anchor: sym.anchor,
                line: sym.line,
            }),
            Some(_) => {}
            None => {
                self.symbols.insert(key, sym);
            }
        }
    }
}

/// Canonical-table lookups shared with [`crate::check`]; `None` means the
/// table is absent from the parsed spec (itself a check failure).
pub fn unit_table(decls: &SpecDecls) -> Option<&TTable> {
    decls
        .tables_in("11.3")
        .find(|t| t.header == ["Unit", "Deliverable", "Depends on", "Acceptance gate"])
}

pub fn obligation_table(decls: &SpecDecls) -> Option<&TTable> {
    decls
        .tables_in("11.3")
        .find(|t| t.header == ["Acceptance gate", "Canonical obligation"])
}

pub fn reading_table(decls: &SpecDecls) -> Option<&TTable> {
    decls
        .tables_in("11.4")
        .find(|t| t.header == ["Unit", "Required sections", "Required Appendix A slice"])
}

pub fn command_table(decls: &SpecDecls) -> Option<&TTable> {
    decls
        .tables_in("11.1")
        .find(|t| t.header == ["Command", "Pipeline operation", "Primary emitted artifacts"])
}

pub fn stage_table(decls: &SpecDecls) -> Option<&TTable> {
    decls.tables_in("3.2").find(|t| {
        t.header
            == [
                "Stage",
                "Producing operation",
                "Generator profiles or builders",
                "Emitted accepted artifacts",
            ]
    })
}

pub fn rule_table(decls: &SpecDecls) -> Option<&TTable> {
    decls
        .tables_in("7.2")
        .find(|t| t.header == ["Rule", "Emitted for", "Checker side condition"])
}

pub fn gate_table(decls: &SpecDecls) -> Option<&TTable> {
    decls.tables_in("3.3").find(|t| {
        t.header
            == [
                "Gate",
                "Trigger",
                "Required evidence object",
                "Claims enabled",
            ]
    })
}

pub fn consumer_table(decls: &SpecDecls) -> Option<&TTable> {
    decls
        .tables_in("2.1")
        .find(|t| t.header == ["Vocabulary", "M0 consumer or emitter"])
}

/// Strips the `[internal:CloseM0]` marker and splits ` and `-joined
/// operation cells (`BuildMatches and BuildMatchClasses`).
pub fn operation_tokens(cell: &str) -> impl Iterator<Item = &str> {
    let base = cell.split(" [").next().unwrap_or(cell);
    base.split(" and ").map(str::trim)
}

/// §1.1 step 1 + step-2 duplicate detection. The returned table holds the
/// first declaration of each `(kind, id)`; divergent redeclarations are
/// reported, in declaration order, for [`crate::check`] to surface.
pub fn build_symbol_table(decls: &SpecDecls) -> (SymbolTable, Vec<DuplicateSymbol>) {
    let mut table = SymbolTable::default();
    let mut duplicates = Vec::new();
    let ins = &mut duplicates;

    // section_anchor: id = anchor token, definition anchor = header title.
    for s in &decls.sections {
        table.insert(
            Symbol {
                kind: SymbolKind::SectionAnchor,
                id: s.anchor.clone(),
                anchor: s.title.clone(),
                line: s.line,
                generic: false,
            },
            ins,
        );
    }

    // schema: §1.3 scalar axioms, then S-decls.
    let scalar_line = decls
        .sections
        .iter()
        .find(|s| s.anchor == "1.3")
        .map_or(0, |s| s.line);
    for name in SCALAR_AXIOMS {
        table.insert(
            Symbol {
                kind: SymbolKind::Schema,
                id: (*name).to_string(),
                anchor: "1.3".to_string(),
                line: scalar_line,
                generic: false,
            },
            ins,
        );
    }
    for d in &decls.s_decls {
        table.insert(
            Symbol {
                kind: SymbolKind::Schema,
                id: d.name.clone(),
                anchor: d.section.clone(),
                line: d.line,
                generic: d.generic_param.is_some(),
            },
            ins,
        );
    }

    // enum + qualified variants/alternatives. A `Ref`-shaped alternative is
    // a type reference when it names a declared schema/enum, otherwise a
    // capitalized bare variant (`E Effect = Inference | ...`); inside a
    // tagged union (any Named/Sexp alternative) every alternative is a
    // union_alternative and check.rs requires Ref targets to resolve.
    for d in &decls.e_decls {
        table.insert(
            Symbol {
                kind: SymbolKind::Enum,
                id: d.name.clone(),
                anchor: d.section.clone(),
                line: d.line,
                generic: d.generic_param.is_some(),
            },
            ins,
        );
    }
    for d in &decls.e_decls {
        let is_union = d
            .alts
            .iter()
            .any(|a| matches!(a, EAlt::Named { .. } | EAlt::Sexp { .. }));
        for a in &d.alts {
            let (kind, tok) = match a {
                EAlt::Named { name, .. } => (SymbolKind::UnionAlternative, name.as_str()),
                EAlt::Sexp { head, .. } => (SymbolKind::UnionAlternative, head.as_str()),
                EAlt::Ref(t) if is_union || table.type_target(t).is_some() => {
                    (SymbolKind::UnionAlternative, t.as_str())
                }
                EAlt::Ref(t) => (SymbolKind::EnumVariant, t.as_str()),
                EAlt::Bare(t) if is_union => (SymbolKind::UnionAlternative, t.as_str()),
                EAlt::Bare(t) => (SymbolKind::EnumVariant, t.as_str()),
            };
            table.insert(
                Symbol {
                    kind,
                    id: format!("{}.{tok}", d.name),
                    anchor: d.section.clone(),
                    line: d.line,
                    generic: false,
                },
                ins,
            );
        }
    }

    // builtin / proof_rule / certificate_class / gate: unqualified ids from
    // the four canonical enums.
    for (decl_name, kind) in [
        ("BuiltinName", SymbolKind::Builtin),
        ("ProofRule", SymbolKind::ProofRule),
        ("M0CertificateClass", SymbolKind::CertificateClass),
        ("Gate", SymbolKind::Gate),
    ] {
        if let Some(d) = decls.e_decl(decl_name) {
            for a in &d.alts {
                let (EAlt::Bare(tok) | EAlt::Ref(tok)) = a else {
                    continue;
                };
                table.insert(
                    Symbol {
                        kind,
                        id: tok.clone(),
                        anchor: d.section.clone(),
                        line: d.line,
                        generic: false,
                    },
                    ins,
                );
            }
        }
    }

    // acceptance_gate: §11.3 obligation-table rows.
    if let Some(t) = obligation_table(decls) {
        for (i, row) in t.rows.iter().enumerate() {
            table.insert(
                Symbol {
                    kind: SymbolKind::AcceptanceGate,
                    id: row[0].clone(),
                    anchor: t.section.clone(),
                    line: t.line + 1 + i,
                    generic: false,
                },
                ins,
            );
        }
    }

    // cli_operation: §11.1 command-table rows.
    if let Some(t) = command_table(decls) {
        for (i, row) in t.rows.iter().enumerate() {
            table.insert(
                Symbol {
                    kind: SymbolKind::CliOperation,
                    id: row[0].clone(),
                    anchor: t.section.clone(),
                    line: t.line + 1 + i,
                    generic: false,
                },
                ins,
            );
        }
    }

    // stage: §3.2 producing operations (repeats across stages share the
    // anchor, so re-insertion is tolerated).
    if let Some(t) = stage_table(decls) {
        for (i, row) in t.rows.iter().enumerate() {
            for op in operation_tokens(&row[1]) {
                table.insert(
                    Symbol {
                        kind: SymbolKind::Stage,
                        id: op.to_string(),
                        anchor: t.section.clone(),
                        line: t.line + 1 + i,
                        generic: false,
                    },
                    ins,
                );
            }
        }
    }

    (table, duplicates)
}
