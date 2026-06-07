//! SPEC.md declaration extractor (M0.0.3.2): one line-oriented pass over the
//! specification text yielding the §1.1 step-1 symbol-source inventory —
//! S/E declarations with parsed [`TypeExpr`] field/alternative types,
//! T-tables keyed by section, markdown section anchors, the §3.1 payload
//! inventory, §6.2 builtin-definition names, and §9.2 certificate classes.
//!
//! Parse-level only: every name lookup — including whether a
//! type-name-shaped E alternative is a type reference or a bare variant —
//! belongs to the M0.0.3.3 symbol table. §1.3 scalars enter resolution
//! through [`SCALAR_AXIOMS`]. Lines that are not declaration-shaped (prose,
//! numbered algorithm steps, pipeline arrows) are ignored; a line that
//! starts like a declaration but fails its grammar becomes a [`ParseIssue`],
//! and the M0.0.3.2 gate holds issues at zero spec-wide.

use std::fmt;

/// §1.3 scalar-table names, predeclared as resolution axioms (anchor §1.3).
/// `Text`, `Set`, `List`, and `Map` are the generic constructors; `Rational`
/// also has an `S Rational` declaration in §1.3 (same anchor, so the
/// M0.0.3.3 duplicate check tolerates the overlap by kind or anchor).
pub const SCALAR_AXIOMS: &[&str] = &[
    "Id",
    "ProofId",
    "RegionId",
    "FeaturePath",
    "Hash",
    "Bool",
    "UInt",
    "Int",
    "Rational",
    "Text",
    "Set",
    "List",
    "Map",
];

/// Field or alternative type: `Base | Base? | Set[X] | List[X] | Map[K,V] |
/// Text<p> | Name[T]`, nested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    /// Base name, or generic application `Name[T]` when `arg` is present.
    Name {
        name: String,
        arg: Option<Box<TypeExpr>>,
    },
    Set(Box<TypeExpr>),
    List(Box<TypeExpr>),
    Map(Box<TypeExpr>, Box<TypeExpr>),
    /// `Text<p>`: a string-policy name, or a sibling-field reference
    /// (`S TextLiteral` uses `value:Text<policy>`).
    Text(String),
    /// Trailing `?`: optional field, represented by omission (§1.3).
    Optional(Box<TypeExpr>),
}

impl TypeExpr {
    /// Parses one complete type expression; rejects trailing bytes.
    pub fn parse(s: &str) -> Result<TypeExpr, String> {
        let b = s.as_bytes();
        let (ty, pos) = parse_type_at(b, 0)?;
        if pos != b.len() {
            return Err(format!("trailing bytes after type in `{s}`"));
        }
        Ok(ty)
    }
}

impl fmt::Display for TypeExpr {
    /// Renders back to spec syntax (M0.0.3.3 diagnostic text).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeExpr::Name { name, arg: None } => write!(f, "{name}"),
            TypeExpr::Name { name, arg: Some(a) } => write!(f, "{name}[{a}]"),
            TypeExpr::Set(x) => write!(f, "Set[{x}]"),
            TypeExpr::List(x) => write!(f, "List[{x}]"),
            TypeExpr::Map(k, v) => write!(f, "Map[{k},{v}]"),
            TypeExpr::Text(p) => write!(f, "Text<{p}>"),
            TypeExpr::Optional(x) => write!(f, "{x}?"),
        }
    }
}

fn parse_type_at(b: &[u8], pos: usize) -> Result<(TypeExpr, usize), String> {
    let rest = &b[pos..];
    let (base, p) = if rest.starts_with(b"Set[") {
        let (inner, p) = parse_type_at(b, pos + 4)?;
        (TypeExpr::Set(Box::new(inner)), expect(b, p, b']')?)
    } else if rest.starts_with(b"List[") {
        let (inner, p) = parse_type_at(b, pos + 5)?;
        (TypeExpr::List(Box::new(inner)), expect(b, p, b']')?)
    } else if rest.starts_with(b"Map[") {
        let (k, p) = parse_type_at(b, pos + 4)?;
        let p = expect(b, p, b',')?;
        let (v, p) = parse_type_at(b, p)?;
        (TypeExpr::Map(Box::new(k), Box::new(v)), expect(b, p, b']')?)
    } else if rest.starts_with(b"Text<") {
        let start = pos + 5;
        let mut p = start;
        while p < b.len() && (b[p].is_ascii_lowercase() || b[p].is_ascii_digit() || b[p] == b'_') {
            p += 1;
        }
        if p == start {
            return Err(format!("empty Text<> policy at byte {start}"));
        }
        let param = String::from_utf8(b[start..p].to_vec()).expect("ascii");
        (TypeExpr::Text(param), expect(b, p, b'>')?)
    } else {
        let mut p = pos;
        if p >= b.len() || !b[p].is_ascii_uppercase() {
            return Err(format!("expected type name at byte {pos}"));
        }
        while p < b.len() && b[p].is_ascii_alphanumeric() {
            p += 1;
        }
        let name = String::from_utf8(b[pos..p].to_vec()).expect("ascii");
        let arg = if b.get(p) == Some(&b'[') {
            let (a, q) = parse_type_at(b, p + 1)?;
            p = expect(b, q, b']')?;
            Some(Box::new(a))
        } else {
            None
        };
        (TypeExpr::Name { name, arg }, p)
    };
    if b.get(p) == Some(&b'?') {
        return Ok((TypeExpr::Optional(Box::new(base)), p + 1));
    }
    Ok((base, p))
}

fn expect(b: &[u8], pos: usize, c: u8) -> Result<usize, String> {
    if b.get(pos) == Some(&c) {
        Ok(pos + 1)
    } else {
        Err(format!("expected `{}` at byte {pos}", c as char))
    }
}

/// `S Name(field:Type,...)` or `S Name<T>(...)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SDecl {
    pub name: String,
    pub generic_param: Option<String>,
    pub fields: Vec<Field>,
    /// Enclosing section anchor, e.g. `1.1`.
    pub section: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: TypeExpr,
}

/// `E Name = alt | alt | ...` or `E Name[T] = ...`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EDecl {
    pub name: String,
    pub generic_param: Option<String>,
    pub alts: Vec<EAlt>,
    pub section: String,
    pub line: usize,
}

impl EDecl {
    /// Decl-level alias (`E RoleName = Id`): exactly one type-name-shaped
    /// alternative.
    pub fn alias_target(&self) -> Option<&str> {
        match self.alts.as_slice() {
            [EAlt::Ref(name)] => Some(name),
            _ => None,
        }
    }
}

/// One E-decl alternative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EAlt {
    /// Lowercase or otherwise non-type-shaped token (`semantic`, `air.term`,
    /// `G-EXTRACTOR-ADAPTER`).
    Bare(String),
    /// `name:Type` (`success:List[T]`).
    Named { name: String, ty: TypeExpr },
    /// `(head ArgType ...)` (`(eq FeaturePath Literal)`).
    Sexp { head: String, args: Vec<TypeExpr> },
    /// Type-name-shaped token: a type reference (`E Term = VarTerm | ...`)
    /// or a capitalized bare variant (`E Effect = Inference | ...`,
    /// `E ClaimTier = S0 | ...`) — M0.0.3.3 resolution decides.
    Ref(String),
}

/// `T col|col|...` header plus its `|`-separated rows, keyed by section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TTable {
    pub section: String,
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub line: usize,
}

/// Markdown header: `### 1.1 Schema authority` → anchor `1.1`; the level-1
/// title line → anchor `title`; `## Appendix A. ...` → `A`; `### A.10 ...`
/// → `A.10`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionAnchor {
    pub anchor: String,
    pub title: String,
    pub level: u8,
    pub line: usize,
}

/// Everything M0.0.3.3-4 consume from one SPEC.md pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpecDecls {
    pub sections: Vec<SectionAnchor>,
    pub s_decls: Vec<SDecl>,
    pub e_decls: Vec<EDecl>,
    pub t_tables: Vec<TTable>,
    /// §3.1 required-payload inventory block, in spec order.
    pub inventory: Vec<String>,
    /// §6.2 builtin-definition names (bijective with `E BuiltinName` under
    /// the M0.0.3.3 check).
    pub builtin_defs: Vec<String>,
    /// §9.2 certificate-class obligation labels (bijective with
    /// `E M0CertificateClass` under the M0.0.3.3 check).
    pub certificate_classes: Vec<String>,
}

impl SpecDecls {
    pub fn s_decl(&self, name: &str) -> Option<&SDecl> {
        self.s_decls.iter().find(|d| d.name == name)
    }

    pub fn e_decl(&self, name: &str) -> Option<&EDecl> {
        self.e_decls.iter().find(|d| d.name == name)
    }

    pub fn tables_in<'a>(&'a self, section: &'a str) -> impl Iterator<Item = &'a TTable> {
        self.t_tables.iter().filter(move |t| t.section == section)
    }
}

/// Declaration-shaped line that failed its grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIssue {
    pub line: usize,
    pub message: String,
}

/// Parse result; the gate requires `issues` to be empty over the real SPEC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecParse {
    pub decls: SpecDecls,
    pub issues: Vec<ParseIssue>,
}

/// One pass over SPEC.md text. Declarations are recognized only inside
/// ``` fenced blocks; section anchors and block preambles (the last prose
/// line before a fence, which keys the §6.2 builtin and §9.2 certificate
/// blocks) are tracked outside them.
pub fn parse_spec(text: &str) -> SpecParse {
    let mut decls = SpecDecls::default();
    let mut issues = Vec::new();
    let mut in_fence = false;
    let mut section = String::from("title");
    let mut preamble = String::new();
    let mut block: Vec<(usize, &str)> = Vec::new();

    for (i, line) in text.lines().enumerate() {
        let lineno = i + 1;
        if line.starts_with("```") {
            if in_fence {
                process_block(&block, &section, &preamble, &mut decls, &mut issues);
                block.clear();
            }
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            block.push((lineno, line));
            continue;
        }
        if let Some((level, anchor, title)) = parse_header(line) {
            section = anchor.clone();
            decls.sections.push(SectionAnchor {
                anchor,
                title,
                level,
                line: lineno,
            });
            continue;
        }
        if !line.trim().is_empty() {
            preamble = line.to_string();
        }
    }
    if in_fence {
        issues.push(ParseIssue {
            line: text.lines().count(),
            message: "unclosed code fence".into(),
        });
    }
    SpecParse { decls, issues }
}

fn parse_header(line: &str) -> Option<(u8, String, String)> {
    let level = line.bytes().take_while(|&c| c == b'#').count();
    if level == 0 || level > 6 || line.as_bytes().get(level) != Some(&b' ') {
        return None;
    }
    let rest = line[level + 1..].trim();
    let (anchor, title) = if level == 1 {
        ("title".to_string(), rest.to_string())
    } else if let Some(t) = rest.strip_prefix("Appendix A.") {
        ("A".to_string(), t.trim().to_string())
    } else {
        let (tok, title) = rest.split_once(' ').unwrap_or((rest, ""));
        (tok.trim_end_matches('.').to_string(), title.to_string())
    };
    Some((level as u8, anchor, title))
}

fn process_block(
    block: &[(usize, &str)],
    section: &str,
    preamble: &str,
    decls: &mut SpecDecls,
    issues: &mut Vec<ParseIssue>,
) {
    // §3.1 inventory: the block whose every line is a bare type name (the
    // sibling pipeline block fails on its `->` lines).
    if section == "3.1" && !block.is_empty() && block.iter().all(|(_, l)| is_type_name(l)) {
        decls
            .inventory
            .extend(block.iter().map(|(_, l)| l.to_string()));
        return;
    }
    // §9.2 certificate classes: `class_name:` labels at column 0.
    if section == "9.2" && preamble == "Certificate classes:" {
        for (_, l) in block {
            if let Some(name) = l.strip_suffix(':')
                && is_lower_name(name)
            {
                decls.certificate_classes.push(name.to_string());
            }
        }
        return;
    }
    // §6.2 builtin definitions: `name(args):` / `name and name:` lines at
    // column 0 (indented continuations skipped); the block also declares
    // `E BuiltinEval`, so fall through to the declaration scan. Lines that
    // fail the name shape are skipped silently — the M0.0.3.3 bijection
    // against `E BuiltinName` catches real omissions.
    if section == "6.2" && preamble.starts_with("Builtin definitions") {
        for (_, l) in block {
            if l.is_empty() || l.starts_with(' ') || l.starts_with("E ") {
                continue;
            }
            let Some((head, _)) = l.split_once(':') else {
                continue;
            };
            let names: Vec<&str> = head
                .split(" and ")
                .map(|p| p.split_once('(').map_or(p, |(base, _)| base))
                .collect();
            if names.iter().all(|n| is_lower_name(n)) {
                decls
                    .builtin_defs
                    .extend(names.into_iter().map(String::from));
            }
        }
    }

    let mut i = 0;
    while i < block.len() {
        let (lineno, l) = block[i];
        if l.len() > 2 && l.starts_with("S ") && l.as_bytes()[2].is_ascii_uppercase() {
            match parse_s_decl(l, lineno, section) {
                Ok(d) => decls.s_decls.push(d),
                Err(message) => issues.push(ParseIssue {
                    line: lineno,
                    message,
                }),
            }
        } else if l.len() > 2 && l.starts_with("E ") && l.as_bytes()[2].is_ascii_uppercase() {
            match parse_e_decl(l, lineno, section) {
                Ok(d) => decls.e_decls.push(d),
                Err(message) => issues.push(ParseIssue {
                    line: lineno,
                    message,
                }),
            }
        } else if l.starts_with("T ") && l.contains('|') {
            let header = l[2..].split('|').map(|c| c.trim().to_string()).collect();
            let mut rows = Vec::new();
            let mut j = i + 1;
            while j < block.len() && block[j].1.contains('|') {
                rows.push(
                    block[j]
                        .1
                        .split('|')
                        .map(|c| c.trim().to_string())
                        .collect(),
                );
                j += 1;
            }
            decls.t_tables.push(TTable {
                section: section.to_string(),
                header,
                rows,
                line: lineno,
            });
            i = j;
            continue;
        }
        i += 1;
    }
}

fn parse_s_decl(line: &str, lineno: usize, section: &str) -> Result<SDecl, String> {
    let rest = &line[2..];
    let b = rest.as_bytes();
    let name_end = b.iter().take_while(|c| c.is_ascii_alphanumeric()).count();
    let name = &rest[..name_end];
    let mut p = name_end;
    let generic_param = if b.get(p) == Some(&b'<') {
        let close = rest[p..]
            .find('>')
            .ok_or_else(|| format!("unclosed `<` in S-decl `{name}`"))?
            + p;
        let param = &rest[p + 1..close];
        if !is_type_name(param) {
            return Err(format!("invalid generic param `{param}` in `{name}`"));
        }
        p = close + 1;
        Some(param.to_string())
    } else {
        None
    };
    if b.get(p) != Some(&b'(') || !rest.ends_with(')') {
        return Err(format!("S-decl `{name}` body must be `(fields)`"));
    }
    let body = &rest[p + 1..rest.len() - 1];
    let mut fields = Vec::new();
    for f in split_top_level(body, b',') {
        let (fname, fty) = f
            .split_once(':')
            .ok_or_else(|| format!("field `{f}` in `{name}` lacks `:`"))?;
        if !is_lower_name(fname) {
            return Err(format!("invalid field name `{fname}` in `{name}`"));
        }
        let ty = TypeExpr::parse(fty).map_err(|e| format!("field `{fname}` in `{name}`: {e}"))?;
        fields.push(Field {
            name: fname.to_string(),
            ty,
        });
    }
    Ok(SDecl {
        name: name.to_string(),
        generic_param,
        fields,
        section: section.to_string(),
        line: lineno,
    })
}

fn parse_e_decl(line: &str, lineno: usize, section: &str) -> Result<EDecl, String> {
    let rest = &line[2..];
    let b = rest.as_bytes();
    let name_end = b.iter().take_while(|c| c.is_ascii_alphanumeric()).count();
    let name = &rest[..name_end];
    let mut p = name_end;
    let generic_param = if b.get(p) == Some(&b'[') {
        let close = rest[p..]
            .find(']')
            .ok_or_else(|| format!("unclosed `[` in E-decl `{name}`"))?
            + p;
        let param = &rest[p + 1..close];
        if !is_type_name(param) {
            return Err(format!("invalid generic param `{param}` in `{name}`"));
        }
        p = close + 1;
        Some(param.to_string())
    } else {
        None
    };
    let body = rest[p..]
        .strip_prefix(" = ")
        .ok_or_else(|| format!("E-decl `{name}` lacks ` = `"))?;
    let mut alts = Vec::new();
    for alt in body.split('|').map(str::trim) {
        alts.push(parse_e_alt(alt).map_err(|e| format!("in `{name}`: {e}"))?);
    }
    Ok(EDecl {
        name: name.to_string(),
        generic_param,
        alts,
        section: section.to_string(),
        line: lineno,
    })
}

fn parse_e_alt(alt: &str) -> Result<EAlt, String> {
    if let Some(inner) = alt.strip_prefix('(') {
        let inner = inner
            .strip_suffix(')')
            .ok_or_else(|| format!("unclosed sexp `{alt}`"))?;
        let mut parts = inner.split_whitespace();
        let head = parts.next().ok_or_else(|| "empty sexp".to_string())?;
        if !is_sexp_head(head) {
            return Err(format!("invalid sexp head `{head}`"));
        }
        let args = parts
            .map(TypeExpr::parse)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("in sexp `{head}`: {e}"))?;
        return Ok(EAlt::Sexp {
            head: head.to_string(),
            args,
        });
    }
    if let Some((n, t)) = alt.split_once(':') {
        if !is_lower_name(n) {
            return Err(format!("invalid alternative name `{n}`"));
        }
        return Ok(EAlt::Named {
            name: n.to_string(),
            ty: TypeExpr::parse(t)?,
        });
    }
    if is_type_name(alt) {
        return Ok(EAlt::Ref(alt.to_string()));
    }
    if is_bare_token(alt) {
        return Ok(EAlt::Bare(alt.to_string()));
    }
    Err(format!("unparseable alternative `{alt}`"))
}

/// Splits on `sep` at bracket depth zero (`[`/`<` open, `]`/`>` close).
fn split_top_level(s: &str, sep: u8) -> Vec<&str> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, &c) in b.iter().enumerate() {
        match c {
            b'[' | b'<' => depth += 1,
            b']' | b'>' => depth -= 1,
            c if c == sep && depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// `[A-Z][A-Za-z0-9]*`.
pub(crate) fn is_type_name(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty() && b[0].is_ascii_uppercase() && b.iter().all(|c| c.is_ascii_alphanumeric())
}

/// `[a-z_][a-z0-9_]*`.
fn is_lower_name(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && (b[0].is_ascii_lowercase() || b[0] == b'_')
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_')
}

/// `[a-z][a-z0-9-]*` (`bounded-path`).
fn is_sexp_head(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && b[0].is_ascii_lowercase()
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

/// Variant token: alphanumerics plus `_`, `.`, `-` (`air.term`,
/// `G-EXTRACTOR-ADAPTER`).
fn is_bare_token(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec_text() -> String {
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../SPEC.md")).unwrap()
    }

    fn name(n: &str) -> TypeExpr {
        TypeExpr::Name {
            name: n.to_string(),
            arg: None,
        }
    }

    /// Gate core: zero unparseable declarations spec-wide, and S/E decl
    /// counts equal an independent flat line scan (fence-state-free).
    #[test]
    fn spec_parses_clean_with_counts_matching_line_scan() {
        let text = spec_text();
        let parse = parse_spec(&text);
        assert!(parse.issues.is_empty(), "issues: {:#?}", parse.issues);

        let s_scan = text
            .lines()
            .filter(|l| {
                l.strip_prefix("S ").is_some_and(|r| {
                    let n = r.bytes().take_while(u8::is_ascii_alphanumeric).count();
                    n > 0
                        && r.as_bytes()[0].is_ascii_uppercase()
                        && matches!(r.as_bytes().get(n), Some(b'(') | Some(b'<'))
                        && r.ends_with(')')
                })
            })
            .count();
        let e_scan = text
            .lines()
            .filter(|l| {
                l.strip_prefix("E ").is_some_and(|r| {
                    let n = r.bytes().take_while(u8::is_ascii_alphanumeric).count();
                    n > 0 && r.as_bytes()[0].is_ascii_uppercase() && r[n..].contains(" = ")
                })
            })
            .count();
        assert_eq!(parse.decls.s_decls.len(), s_scan);
        assert_eq!(parse.decls.e_decls.len(), e_scan);
        assert!(s_scan > 200, "s_scan={s_scan}");
        assert!(e_scan > 100, "e_scan={e_scan}");
    }

    /// Spot-check: `S SchemaRegistry` field names, a nested field type, and
    /// the `S ArtifactEnvelope<T>` generic param.
    #[test]
    fn spec_schema_registry_fields() {
        let parse = parse_spec(&spec_text());
        let d = parse.decls.s_decl("SchemaRegistry").unwrap();
        assert_eq!(d.section, "1.1");
        assert_eq!(
            d.fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
            [
                "registry_id",
                "registry_version",
                "spec_contract_hash",
                "rust_type_manifest_hash",
                "generated_json_schema_manifest_hash",
                "canonicalization_policy_hash",
                "schema_bound_manifest_hash",
                "schema_entries",
                "string_policy_bindings",
                "source_support_aliases",
            ]
        );
        assert_eq!(d.fields[7].ty, TypeExpr::Set(Box::new(name("SchemaEntry"))));

        let env = parse.decls.s_decl("ArtifactEnvelope").unwrap();
        assert_eq!(env.generic_param.as_deref(), Some("T"));
        assert_eq!(env.fields.last().unwrap().ty, name("T"));

        let lit = parse.decls.s_decl("TextLiteral").unwrap();
        assert_eq!(lit.fields[1].ty, TypeExpr::Text("policy".to_string()));
    }

    /// Spot-check: `E Premise` is all-sexp with typed args.
    #[test]
    fn spec_premise_sexps() {
        let parse = parse_spec(&spec_text());
        let d = parse.decls.e_decl("Premise").unwrap();
        assert_eq!(d.alts.len(), 17);
        assert!(d.alts.iter().all(|a| matches!(a, EAlt::Sexp { .. })));
        assert!(d.alts.contains(&EAlt::Sexp {
            head: "builtin".to_string(),
            args: vec![
                name("BuiltinName"),
                TypeExpr::List(Box::new(name("Term"))),
                name("OutputVars"),
            ],
        }));
        assert!(d.alts.iter().any(
            |a| matches!(a, EAlt::Sexp { head, args } if head == "bounded-path" && args.len() == 4)
        ));
    }

    /// Spot-check: `E OperationResult[T]` generic union, alternative shapes,
    /// and decl-level aliases.
    #[test]
    fn spec_operation_result_and_alt_shapes() {
        let parse = parse_spec(&spec_text());
        let d = parse.decls.e_decl("OperationResult").unwrap();
        assert_eq!(d.section, "1.7");
        assert_eq!(d.generic_param.as_deref(), Some("T"));
        assert_eq!(d.alts.len(), 6);
        assert_eq!(
            d.alts[0],
            EAlt::Named {
                name: "success".to_string(),
                ty: TypeExpr::List(Box::new(name("T"))),
            }
        );

        let role = parse.decls.e_decl("RoleName").unwrap();
        assert_eq!(role.alias_target(), Some("Id"));
        let effect = parse.decls.e_decl("Effect").unwrap();
        assert_eq!(effect.alias_target(), None);
        assert_eq!(effect.alts[0], EAlt::Ref("Inference".to_string()));
        let gate = parse.decls.e_decl("Gate").unwrap();
        assert_eq!(gate.alts[0], EAlt::Bare("G-EXTRACTOR-ADAPTER".to_string()));
    }

    /// The M0.0.3.3-4 consumption surfaces: section anchors, the named
    /// T-tables, §3.1 inventory, §6.2 builtins, §9.2 certificate classes.
    #[test]
    fn spec_tables_anchors_inventory_builtins_certificates() {
        let parse = parse_spec(&spec_text());
        let d = &parse.decls;

        let s113 = d.sections.iter().find(|s| s.anchor == "11.3").unwrap();
        assert_eq!(s113.level, 3);
        assert!(d.sections.iter().any(|s| s.anchor == "A.10"));
        assert_eq!(d.sections[0].anchor, "title");

        let unit_table = d
            .tables_in("11.3")
            .find(|t| t.header == ["Unit", "Deliverable", "Depends on", "Acceptance gate"])
            .unwrap();
        assert_eq!(unit_table.rows[0][0], "M0.0.1");
        assert_eq!(unit_table.rows.last().unwrap()[0], "GATED.1");
        let obligation_table = d
            .tables_in("11.3")
            .find(|t| t.header == ["Acceptance gate", "Canonical obligation"])
            .unwrap();
        assert_eq!(unit_table.rows.len(), obligation_table.rows.len());

        let rule_table = d.tables_in("7.2").next().unwrap();
        assert_eq!(rule_table.header[0], "Rule");
        assert_eq!(rule_table.rows.len(), 15);
        assert_eq!(rule_table.rows[0][0], "SOURCE");

        let command_table = d.tables_in("11.1").next().unwrap();
        assert_eq!(command_table.header[0], "Command");
        assert_eq!(command_table.rows.len(), 21);
        let stage_table = d.tables_in("3.2").find(|t| t.header[0] == "Stage").unwrap();
        assert!(stage_table.rows.iter().any(|r| r[1] == "CloseM0"));

        assert_eq!(d.inventory.len(), 74);
        assert_eq!(d.inventory[0], "SchemaRegistry");
        assert_eq!(d.inventory.last().unwrap(), "ReplayIdentityCheck");

        assert_eq!(d.builtin_defs.len(), 14);
        assert!(d.builtin_defs.contains(&"ctx_compatible".to_string()));
        assert!(d.builtin_defs.contains(&"dependency_minimize".to_string()));

        assert_eq!(
            d.certificate_classes,
            [
                "source_graph",
                "mech_observed",
                "admitted_base",
                "closed_nf",
                "finite_checked",
                "report_replay",
            ]
        );

        assert!(SCALAR_AXIOMS.contains(&"FeaturePath"));
    }

    /// `TypeExpr` Display renders back to spec syntax.
    #[test]
    fn spec_type_expr_display_roundtrip() {
        for s in [
            "Id",
            "Hash?",
            "Set[SchemaEntry]",
            "Map[Id,Text<semantic_ja>]",
            "List[Set[RegionMember]]",
            "Text<raw_source>?",
            "OperationResult[Certificate]",
            "T",
        ] {
            assert_eq!(TypeExpr::parse(s).unwrap().to_string(), s);
        }
    }
}
