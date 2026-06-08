//! §1.1 step 3 (M0.0.3.3): resolution of every spec-declared reference
//! against the [`crate::symtab`] symbol table, with checker-local sorted
//! diagnostics. §1.1 step 4 (M0.0.3.4.4): [`check_registry`] validates a
//! built registry+manifest pair — two-sided bound coverage over the §3.1
//! type-graph walk and the §1.2 source-support/role rule. §1.2 hash
//! conventions (M0.0.3.4.5): [`classify_hash_fields`] assigns every
//! walked `HashNamed`/`HashValued` row a [`HashFieldClass`] via suffix
//! defaults plus the authored [`HASH_FIELD_EXCEPTIONS`] table;
//! [`check_registry`] rejects `Unresolved` rows. §3.2 reverse producer
//! coverage (M0.0.3.4.6): [`producer_mapping_issues`] requires every §3.1
//! payload to be named by a stage-table emission or the control-emission
//! rule. Each issue carries [`CheckIssue::CODE`] or
//! [`CheckIssue::PRODUCER_MAPPING`]; the §8.7 `Diagnostic` artifact
//! wrapper lands with its consuming unit.
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

/// One checker finding. Ordering (line, anchor, message) is the canonical
/// report order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CheckIssue {
    pub line: usize,
    /// Enclosing section anchor of the offending reference.
    pub anchor: String,
    pub message: String,
    /// [`Self::CODE`] for resolution/registry issues,
    /// [`Self::PRODUCER_MAPPING`] for §3.2 coverage issues.
    pub code: &'static str,
}

impl CheckIssue {
    /// §1.1 step 5 diagnostic code shared by resolution/registry issues.
    pub const CODE: &'static str = "referential_integrity_error";
    /// §3.2: a §3.1 payload names no producing operation or control
    /// emission.
    pub const PRODUCER_MAPPING: &'static str = "producer_mapping_error";
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
            code: CheckIssue::CODE,
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

    for (line, message) in producer_mapping_issues(decls) {
        issues.push(CheckIssue {
            line,
            anchor: enclosing_anchor(decls, line),
            message,
            code: CheckIssue::PRODUCER_MAPPING,
        });
    }

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

/// §1.1 step-4 bound coverage, the §1.2 source-support/role rule, and the
/// §1.2 hash-field conventions over a built registry+manifest pair: every
/// walked hash leaf must classify to a non-`Unresolved`
/// [`HashFieldClass`]. Peer of [`check_spec`]: same issue type and
/// ordering. Step-1-2/5 composition (.4.7) remains.
pub fn check_registry(
    text: &str,
    registry: &SchemaRegistry,
    manifest: &SchemaBoundManifest,
) -> CheckReport {
    check_registry_with(text, registry, manifest, HASH_FIELD_EXCEPTIONS)
}

/// [`check_registry`] over an explicit hash-exception table (perturbation
/// tests inject lingering-`Unresolved` rows here).
fn check_registry_with(
    text: &str,
    registry: &SchemaRegistry,
    manifest: &SchemaBoundManifest,
    exceptions: &[(&str, HashFieldClass, &str)],
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
            code: CheckIssue::CODE,
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

    // §1.2 hash conventions: an `Unresolved` classification means no
    // convention or field-specific computation applies — the schema entry
    // is invalid.
    for r in classify_with(decls, exceptions) {
        if r.class == HashFieldClass::Unresolved {
            let schema = r.schema_id.as_str();
            push(
                line_of(schema),
                format!(
                    "hash-valued field `{schema}/{}` resolves to no §1.2 convention or \
                     field-specific computation",
                    joined(&r.path)
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

/// §1.2 hash-field-convention class of one walked `HashNamed` path
/// (M0.0.3.4.5.1): which computation produces the stored hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HashFieldClass {
    /// `*_hash`/`*_hashes` naming an accepted artifact's envelope hash.
    ArtifactRef,
    /// `*_digest`/`*_digests`: sha256 of canonical payload bytes of a
    /// payload defined beside the field.
    NamedPayloadDigest,
    /// `sha256(exact_recorded_bytes)` of raw-source/executable/external-
    /// manifest/index-fingerprint bytes, supplied by an accompanying
    /// manifest.
    RawRecordedBytes,
    /// Field-specific computation defined beside the field in its section.
    FieldSpecific,
    /// No §1.2 convention or field-specific computation applies —
    /// [`check_registry`] rejects every such path as
    /// `referential_integrity_error`. Reached by `HashValued` rows
    /// without an exception row and by suffix drift between build.rs
    /// `is_hash_named` and [`classify_hash_fields`]'s defaults.
    Unresolved,
}

/// One classified §3.1 walk row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedHashField {
    pub schema_id: Id,
    pub path: FeaturePath,
    pub class: HashFieldClass,
}

/// Suffix defaults (§1.2): `*_hash`/`*_hashes` reference accepted
/// artifacts; `*_digest`/`*_digests` digest a payload named beside the
/// field; any other terminal (`HashValued` rows, or suffix drift between
/// build.rs `is_hash_named` and this map) is `Unresolved` pending an
/// exception row.
fn hash_suffix_default(terminal: &str) -> HashFieldClass {
    if terminal.ends_with("_hash") || terminal.ends_with("_hashes") {
        HashFieldClass::ArtifactRef
    } else if terminal.ends_with("_digest") || terminal.ends_with("_digests") {
        HashFieldClass::NamedPayloadDigest
    } else {
        HashFieldClass::Unresolved
    }
}

/// Authored terminal-name exceptions to the suffix defaults, sorted; rows
/// are (terminal field name, class, one-line rationale). Every walked
/// terminal name is judged against its S-decl context (.5.1.1 a-l,
/// .5.1.2 m-z; .5.2.x burned down all 42 Unresolved rows via SPEC
/// corrections); a suffix-named terminal absent here means the default
/// survived judgment — WordingGateRecord.literal_part_digests rides
/// NamedPayloadDigest with its payload defined beside the field (§9.3).
/// `HashValued` terminals (no suffix, default Unresolved) each REQUIRE a
/// row naming the §1.2 field-specific computation; the §7.1 bound-map
/// key sentence anchors the ClosureBoundCertificate family.
pub const HASH_FIELD_EXCEPTIONS: &[(&str, HashFieldClass, &str)] = &[
    (
        "actual_output_hashes",
        HashFieldClass::FieldSpecific,
        "§1.6 ReplayIdentity step 3 defines the recomputed-stratum set; equals accepted hashes exactly on replay_identity_pass",
    ),
    (
        "adapter_manifest_hash",
        HashFieldClass::RawRecordedBytes,
        "external extraction-adapter manifest bytes; the owning ExtractionManifest supplies them",
    ),
    (
        "alternatives",
        HashFieldClass::ArtifactRef,
        "§7.3/§8.7 Ambiguity alternatives store the competing accepted reading envelope hashes under a collective name",
    ),
    (
        "applies_to",
        HashFieldClass::FieldSpecific,
        "§8.4 resolution_subject_ids fixes members: operand artifact_hash, CKCNormalForm semantic_digest, or contributing License hashes",
    ),
    (
        "axis_path_bounds",
        HashFieldClass::NamedPayloadDigest,
        "§7.1 bound-map keys digest the bounded bounded-path form's canonical bytes",
    ),
    (
        "build_input_hashes",
        HashFieldClass::RawRecordedBytes,
        "toolchain build-input bytes recorded by the owning ToolchainManifest",
    ),
    (
        "class_signature_hash",
        HashFieldClass::FieldSpecific,
        "sha256 over §7.2 proof_visible_signature canonical bytes",
    ),
    (
        "collect_bounds",
        HashFieldClass::NamedPayloadDigest,
        "§7.1 bound-map keys digest the bounded collect form's canonical bytes",
    ),
    (
        "config_hash",
        HashFieldClass::RawRecordedBytes,
        "tool/analyzer configuration bytes recorded by the enclosing manifest",
    ),
    (
        "content_hash",
        HashFieldClass::RawRecordedBytes,
        "raw document content bytes; sibling extraction_manifest_hash names the supplier",
    ),
    (
        "context_clause_bounds",
        HashFieldClass::NamedPayloadDigest,
        "§7.1 bound-map keys digest the bounded context-clause form's canonical bytes",
    ),
    (
        "decoding_policy_hash",
        HashFieldClass::RawRecordedBytes,
        "external decoding-policy configuration bytes (§6.4 evidence-discovery provenance)",
    ),
    (
        "dense_retriever_manifest_hash",
        HashFieldClass::RawRecordedBytes,
        "external dense-retriever manifest bytes (evidence-discovery trace)",
    ),
    (
        "diagnostics",
        HashFieldClass::ArtifactRef,
        "CKCNormalForm.diagnostics stores accepted Diagnostic envelope hashes under a collective name (§7.4)",
    ),
    (
        "environment_variable_hashes",
        HashFieldClass::RawRecordedBytes,
        "recorded environment-variable bytes; EnvironmentProfile is the supplying manifest",
    ),
    (
        "executable_hash",
        HashFieldClass::RawRecordedBytes,
        "§1.2 names executable bytes explicitly",
    ),
    (
        "external_backend_core_hash",
        HashFieldClass::RawRecordedBytes,
        "replayed external-solver core bytes; the recorded solver proof is the supplying manifest (§8.1)",
    ),
    (
        "fusion_policy_hash",
        HashFieldClass::RawRecordedBytes,
        "external fusion-policy configuration bytes; the sibling family enum names the kind",
    ),
    (
        "generated_json_schema_hash",
        HashFieldClass::FieldSpecific,
        "§1.1 T-Schema-Equivalence canonicalize-and-compare computation; M0.0.4 implements",
    ),
    (
        "generated_json_schema_manifest_hash",
        HashFieldClass::FieldSpecific,
        "§1.1 T-Schema-Equivalence, registry-wide manifest level; M0.0.4 implements",
    ),
    (
        "generator_env_bounds",
        HashFieldClass::ArtifactRef,
        "§7.1 bound-map keys are accepted generator envelope hashes",
    ),
    (
        "generator_materialized_counts",
        HashFieldClass::ArtifactRef,
        "§7.1 bound-map keys are accepted generator envelope hashes",
    ),
    (
        "graph_retrieval_manifest_hash",
        HashFieldClass::RawRecordedBytes,
        "external graph-retrieval manifest bytes (evidence-discovery trace)",
    ),
    (
        "implementation_unit_hashes",
        HashFieldClass::RawRecordedBytes,
        "producing-implementation code bytes (executable-bytes family) beside ProducerManifest.toolchain_manifest_hash",
    ),
    (
        "index_fingerprint_hashes",
        HashFieldClass::RawRecordedBytes,
        "§1.2 names index fingerprints explicitly; §6.4 requires replayable fingerprints",
    ),
    (
        "input_bytes_hash",
        HashFieldClass::RawRecordedBytes,
        "raw extraction-adapter input bytes; the owning ExtractionManifest supplies them (§4.4)",
    ),
    (
        "input_context_hashes",
        HashFieldClass::RawRecordedBytes,
        "recorded generator input-context bytes (§6.4 evidence-discovery provenance)",
    ),
    (
        "late_interaction_manifest_hash",
        HashFieldClass::RawRecordedBytes,
        "external late-interaction manifest bytes (evidence-discovery trace)",
    ),
    (
        "minimal_generator_dependency_set",
        HashFieldClass::ArtifactRef,
        "§8.7 dependency_minimize returns an inclusion-minimal subset of accepted generator envelope hashes",
    ),
    (
        "minimal_theorem_set",
        HashFieldClass::ArtifactRef,
        "§8.7 theorem_minimize returns an inclusion-minimal subset of accepted theorem envelope hashes",
    ),
    (
        "normalization_table_hash",
        HashFieldClass::RawRecordedBytes,
        "Unicode normalization-table bytes; UnicodePolicyManifest supplies them (M0.0.1 table fingerprint)",
    ),
    (
        "output_bytes_hash",
        HashFieldClass::RawRecordedBytes,
        "raw generator output bytes (§6.4 proposal_bytes_hash family)",
    ),
    (
        "permission_evidence_hash",
        HashFieldClass::RawRecordedBytes,
        "external rights-evidence document bytes; the owning SourcePermissionRecord records access_ref (§4.1)",
    ),
    (
        "policy_test_hash",
        HashFieldClass::RawRecordedBytes,
        "policy test-vector file bytes; UnicodePolicyManifest supplies them (M0.0.1 policy_vectors.json)",
    ),
    (
        "prompt_template_hash",
        HashFieldClass::RawRecordedBytes,
        "external prompt-template bytes (§6.4 evidence-discovery provenance)",
    ),
    (
        "proposal_bytes_hash",
        HashFieldClass::RawRecordedBytes,
        "recorded pre-discharge candidate bytes (§6.4 DischargeProposal candidate_bytes input)",
    ),
    (
        "punctuation_table_hash",
        HashFieldClass::RawRecordedBytes,
        "punctuation-table bytes; UnicodePolicyManifest supplies them (M0.0.1 table fingerprint)",
    ),
    (
        "query_decomposition_hash",
        HashFieldClass::RawRecordedBytes,
        "recorded decomposed-query bytes (§6.4 evidence-discovery trace)",
    ),
    (
        "query_hash",
        HashFieldClass::RawRecordedBytes,
        "recorded retrieval-query bytes; the trace's retriever manifests supply the issuing context (§6.4)",
    ),
    (
        "reranker_manifest_hash",
        HashFieldClass::RawRecordedBytes,
        "external reranker manifest bytes (evidence-discovery trace)",
    ),
    (
        "returned_sets",
        HashFieldClass::ArtifactRef,
        "§8.7 repair-set sets draw from candidate_dependency_hashes (accepted envelope hashes); diagnostic-only trace",
    ),
    (
        "rust_type_hash",
        HashFieldClass::FieldSpecific,
        "§1.1 T-Schema-Equivalence canonicalize-and-compare computation; M0.0.4 implements",
    ),
    (
        "rust_type_manifest_hash",
        HashFieldClass::FieldSpecific,
        "§1.1 T-Schema-Equivalence, registry-wide manifest level; M0.0.4 implements",
    ),
    (
        "score_record_hashes",
        HashFieldClass::RawRecordedBytes,
        "recorded external score-record bytes; the trace's *_manifest_hash fields name the suppliers (§6.4 keeps scores evidence-only)",
    ),
    (
        "sequence_bounds",
        HashFieldClass::NamedPayloadDigest,
        "§7.1 bound-map keys digest the bounded seq form's canonical bytes",
    ),
    (
        "source_hash",
        HashFieldClass::RawRecordedBytes,
        "raw source-document bytes; sibling extraction_manifest_hash names the supplier",
    ),
    (
        "sparse_retriever_manifest_hash",
        HashFieldClass::RawRecordedBytes,
        "external sparse-retriever manifest bytes (evidence-discovery trace)",
    ),
    (
        "spec_contract_hash",
        HashFieldClass::RawRecordedBytes,
        "specification-document bytes (sha256 over SPEC.md, built by .4.3); §1.1 compares it under T-Schema-Equivalence",
    ),
    (
        "structured_output_schema_hash",
        HashFieldClass::RawRecordedBytes,
        "external constrained-decoding schema bytes (§6.4 evidence-discovery provenance)",
    ),
    (
        "subject_hashes",
        HashFieldClass::FieldSpecific,
        "subject-identity hashes defined beside §8.7 Incoherence: envelope artifact_hash for enveloped subjects, else canonical-payload digest; §1.1 overflow_member_hash instantiates the rule",
    ),
    (
        "tagged_union_alternatives_hash",
        HashFieldClass::FieldSpecific,
        "§1.1 T-Schema-Equivalence canonicalizes the union-alternative set; M0.0.4 implements",
    ),
    (
        "tool_manifest_hashes",
        HashFieldClass::RawRecordedBytes,
        "external function-calling tool manifest bytes (§6.4 evidence-discovery provenance)",
    ),
    (
        "witness_hash",
        HashFieldClass::FieldSpecific,
        "§8.5/§8.6 fix the binding per kind: WitnessContext/ConstraintCoreWitness artifact refs, except terminology_mapping_incoherence digests the referenced incoherence-hash set",
    ),
];

/// Classifies every `HashNamed` and `HashValued` row of the §3.1 walk:
/// authored exception by terminal field name, else suffix default. Total —
/// every row gets a class; `Unresolved` rejects via [`check_registry`].
pub fn classify_hash_fields(decls: &SpecDecls) -> Vec<ClassifiedHashField> {
    classify_with(decls, HASH_FIELD_EXCEPTIONS)
}

/// [`classify_hash_fields`] over an explicit exception table (perturbation
/// tests inject `Unresolved` rows here).
fn classify_with(
    decls: &SpecDecls,
    exceptions: &[(&str, HashFieldClass, &str)],
) -> Vec<ClassifiedHashField> {
    walk_inventory(decls)
        .into_iter()
        .filter(|r| matches!(r.leaf, WalkedLeaf::HashNamed | WalkedLeaf::HashValued))
        .map(|r| {
            let terminal = r
                .path
                .segments()
                .last()
                .expect("hash-leaf path is nonempty")
                .as_str();
            let class = exceptions
                .iter()
                .find(|(name, _, _)| *name == terminal)
                .map_or_else(|| hash_suffix_default(terminal), |(_, class, _)| *class);
            ClassifiedHashField {
                schema_id: r.schema_id,
                path: r.path,
                class,
            }
        })
        .collect()
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

/// §3.2 cross-cutting control-emission rule: payloads accepted as command
/// wrapper/gate-runner emissions, authored inputs, or control rows within
/// them, rather than stage-producer emissions.
const CONTROL_EMISSIONS: [&str; 9] = [
    "DiagnosticTag",
    "EnvironmentProfile",
    "FiniteFixtureManifest",
    "FrozenConstant",
    "ParsedQuantity",
    "ProducerManifest",
    "ToolRecord",
    "ToolchainManifest",
    "ValidationManifest",
];

/// §3.2 reverse coverage: every §3.1 inventory payload is named in a
/// stage-table emitted-artifacts cell or by [`CONTROL_EMISSIONS`]. Cell
/// handling extends [`resolve_artifact_cell`] token shape to qualified
/// mentions: any type-shaped token names its payload (`admitted CKCGen`,
/// `TerminologyResourceSet fragments`, `ResolutionTheorem artifacts` are
/// emission mappings; item-level reading would drop them). Returns
/// `(line, message)` rows for [`CheckIssue::PRODUCER_MAPPING`] issues,
/// attributed to the §3.1 block like [`check_inventory`].
fn producer_mapping_issues(decls: &SpecDecls) -> Vec<(usize, String)> {
    // A missing stage table is check_stage_table's finding.
    let Some(t) = stage_table(decls) else {
        return Vec::new();
    };
    let emitted: BTreeSet<&str> = t
        .rows
        .iter()
        .flat_map(|r| r[3].split([',', ' ']))
        .filter(|s| is_type_name(s))
        .collect();
    let line = decls
        .sections
        .iter()
        .find(|s| s.anchor == "3.1")
        .map_or(0, |s| s.line);
    decls
        .inventory
        .iter()
        .filter(|name| {
            !emitted.contains(name.as_str()) && !CONTROL_EMISSIONS.contains(&name.as_str())
        })
        .map(|name| {
            (
                line,
                format!("payload `{name}` names no §3.2 producing operation or control emission"),
            )
        })
        .collect()
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
    use ckc_core::scalar::{Hash, UInt};

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
    /// The renamed artifact additionally unmaps its payload.
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
            [
                "payload `AnalyzerManifest` names no §3.2 producing operation or control emission",
                "unresolved artifact name `AnalyzerManifezt`"
            ]
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

    /// §3.2 reverse coverage: dequalifying a payload's only stage-table
    /// mention to prose rejects with `producer_mapping_error` (and nothing
    /// else — the prose cell itself stays resolution-clean).
    #[test]
    fn check_producer_mapping_unmapped_payload_rejects() {
        let text = spec_text().replace("| obs_pattern | PatternObs", "| obs_pattern | patterns");
        let report = check_spec(&text);
        assert_eq!(
            messages(&report),
            ["payload `PatternObs` names no §3.2 producing operation or control emission"]
        );
        assert_eq!(report.issues[0].code, CheckIssue::PRODUCER_MAPPING);
        assert_eq!(report.issues[0].anchor, "3.1");
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

    /// §1.2 hash-convention perturbations: a hash-valued field outside
    /// the naming suffixes with no exception row (synthetic mini-spec)
    /// and a lingering `Unresolved` exception row over the real SPEC
    /// both reject.
    #[test]
    fn check_registry_hash_perturbations_reject() {
        let mini = "### 3.1 Synthetic\n\n```text\nRoot\n```\n\n\
                    ```text\nS Root(root_id:Id,fingerprint:Hash)\n```\n";
        let h = Hash::of_bytes(b"");
        let registry = SchemaRegistry {
            registry_id: Id::new("r").unwrap(),
            registry_version: Id::new("v0").unwrap(),
            spec_contract_hash: h.clone(),
            rust_type_manifest_hash: h.clone(),
            generated_json_schema_manifest_hash: h.clone(),
            canonicalization_policy_hash: h.clone(),
            schema_bound_manifest_hash: h,
            schema_entries: BTreeSet::new(),
            string_policy_bindings: BTreeSet::new(),
            source_support_aliases: BTreeSet::new(),
        };
        let empty_manifest = SchemaBoundManifest {
            manifest_id: Id::new("m").unwrap(),
            schema_collection_bounds: BTreeSet::new(),
        };
        let report = check_registry(mini, &registry, &empty_manifest);
        assert_eq!(
            messages(&report),
            [
                "hash-valued field `root/fingerprint` resolves to no §1.2 convention or field-specific computation"
            ]
        );
        assert_eq!(report.issues[0].anchor, "3.1");

        let text = spec_text();
        let decls = parse_spec(&text).decls;
        let (registry, manifest) = build_v0_registry(&decls, &text);
        let report = check_registry_with(
            &text,
            &registry,
            &manifest,
            &[("subject_hash", HashFieldClass::Unresolved, "perturbation")],
        );
        assert!(!report.is_clean());
        assert!(
            report
                .issues
                .iter()
                .all(|i| i.message.contains("resolves to no §1.2 convention"))
        );
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.message.contains("subject_hash"))
        );
    }

    fn terminal(row: &ClassifiedHashField) -> &str {
        row.path.segments().last().unwrap().as_str()
    }

    /// Totality + final per-class path counts over the real SPEC (240
    /// HashNamed + 14 HashValued paths). The .5.2.x burn-down is complete
    /// (42 of 42 names); Unresolved is empty and [`check_registry`]
    /// rejects any recurrence.
    #[test]
    fn check_hash_real_spec_totality_and_counts() {
        let text = spec_text();
        let decls = parse_spec(&text).decls;
        let rows = classify_hash_fields(&decls);
        let hash_leaves = walk_inventory(&decls)
            .into_iter()
            .filter(|r| matches!(r.leaf, WalkedLeaf::HashNamed | WalkedLeaf::HashValued))
            .count();
        assert_eq!(rows.len(), hash_leaves);
        assert_eq!(rows.len(), 254);

        let names: BTreeSet<&str> = rows.iter().map(terminal).collect();
        assert_eq!(names.len(), 166);
        assert_eq!(names.iter().filter(|n| **n < "m").count(), 84);

        let count = |class: HashFieldClass| rows.iter().filter(|r| r.class == class).count();
        assert_eq!(count(HashFieldClass::ArtifactRef), 180);
        assert_eq!(count(HashFieldClass::NamedPayloadDigest), 26);
        assert_eq!(count(HashFieldClass::RawRecordedBytes), 35);
        assert_eq!(count(HashFieldClass::FieldSpecific), 13);
        assert_eq!(count(HashFieldClass::Unresolved), 0);
    }

    /// Exception-table hygiene: rows sorted and unique, each names a
    /// walked terminal, never restates a suffix default, and carries a
    /// rationale.
    #[test]
    fn check_hash_exception_table_hygiene() {
        assert!(HASH_FIELD_EXCEPTIONS.windows(2).all(|w| w[0].0 < w[1].0));

        let text = spec_text();
        let decls = parse_spec(&text).decls;
        let rows = classify_hash_fields(&decls);
        let walked: BTreeSet<&str> = rows.iter().map(terminal).collect();
        for (name, class, rationale) in HASH_FIELD_EXCEPTIONS {
            assert!(walked.contains(name), "dead exception row `{name}`");
            assert_ne!(
                *class,
                hash_suffix_default(name),
                "redundant exception row `{name}`"
            );
            assert!(!rationale.is_empty());
        }
    }

    /// Spot checks across both judged halves: suffix defaults that
    /// survived judgment and rows per exception class, keyed
    /// (schema_id, /-joined path).
    #[test]
    fn check_hash_spot_checks() {
        let text = spec_text();
        let decls = parse_spec(&text).decls;
        let by_key: BTreeMap<(String, String), HashFieldClass> = classify_hash_fields(&decls)
            .into_iter()
            .map(|r| ((r.schema_id.as_str().to_string(), joined(&r.path)), r.class))
            .collect();
        for (schema, path, class) in [
            (
                "admission_context",
                "accepted_base_hash",
                HashFieldClass::ArtifactRef,
            ),
            (
                "closure_output",
                "claim_record_hashes",
                HashFieldClass::ArtifactRef,
            ),
            (
                "conflict_theorem",
                "left_artifact_hash",
                HashFieldClass::ArtifactRef,
            ),
            (
                "gloss_view",
                "combined_slot_digest",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "proof_node",
                "environment_digest",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "tool_record",
                "executable_hash",
                HashFieldClass::RawRecordedBytes,
            ),
            (
                "toolchain_manifest",
                "tool_records/config_hash",
                HashFieldClass::RawRecordedBytes,
            ),
            (
                "retrieval_proposal_trace",
                "index_fingerprint_hashes",
                HashFieldClass::RawRecordedBytes,
            ),
            (
                "replay_identity_check",
                "actual_output_hashes",
                HashFieldClass::FieldSpecific,
            ),
            (
                "match_class",
                "class_signature_hash",
                HashFieldClass::FieldSpecific,
            ),
            (
                "incoherence",
                "subject_hashes",
                HashFieldClass::FieldSpecific,
            ),
            (
                "validation_manifest",
                "validated_artifact_hashes",
                HashFieldClass::ArtifactRef,
            ),
            (
                "schema_registry",
                "canonicalization_policy_hash",
                HashFieldClass::ArtifactRef,
            ),
            (
                "source_region",
                "closure_certificate_hash",
                HashFieldClass::ArtifactRef,
            ),
            (
                "region_closure_certificate",
                "seed_members_digest",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "extraction_manifest",
                "input_bytes_hash",
                HashFieldClass::RawRecordedBytes,
            ),
            (
                "verifier_witness",
                "subject_hash",
                HashFieldClass::ArtifactRef,
            ),
            (
                "generator_grammar_artifact",
                "parser_state_machine/states/lr_items_digest",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "closure_output",
                "match_hashes",
                HashFieldClass::ArtifactRef,
            ),
            (
                "verifier_witness",
                "symbol_source_map_hash",
                HashFieldClass::ArtifactRef,
            ),
            ("review_report", "trace_hash", HashFieldClass::ArtifactRef),
            (
                "ckc_normal_form",
                "semantic_digest",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "proof_node",
                "support_digest",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "source_edition",
                "source_hash",
                HashFieldClass::RawRecordedBytes,
            ),
            (
                "unicode_policy_manifest",
                "normalization_table_hash",
                HashFieldClass::RawRecordedBytes,
            ),
            (
                "schema_registry",
                "spec_contract_hash",
                HashFieldClass::RawRecordedBytes,
            ),
            (
                "schema_registry",
                "schema_entries/rust_type_hash",
                HashFieldClass::FieldSpecific,
            ),
            (
                "conflict_theorem",
                "witness_hash",
                HashFieldClass::FieldSpecific,
            ),
            (
                "materialized_consequence_manifest",
                "candidate_digest",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "counterexample_suite",
                "forbidden_output_digests",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "wording_gate_record",
                "literal_part_digests",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "closure_bound_certificate",
                "collect_bounds",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "closure_bound_certificate",
                "generator_env_bounds",
                HashFieldClass::ArtifactRef,
            ),
            (
                "resolution_theorem",
                "applies_to",
                HashFieldClass::FieldSpecific,
            ),
            (
                "review_report",
                "report_items/minimal_theorem_set",
                HashFieldClass::ArtifactRef,
            ),
            (
                "admission_context",
                "admission_record_hash",
                HashFieldClass::ArtifactRef,
            ),
            (
                "proposal_record",
                "proposal_provenance_hashes",
                HashFieldClass::ArtifactRef,
            ),
            (
                "proposal_provenance_manifest",
                "output_bytes_hash",
                HashFieldClass::RawRecordedBytes,
            ),
            (
                "retrieval_proposal_trace",
                "score_record_hashes",
                HashFieldClass::RawRecordedBytes,
            ),
            (
                "witness_context",
                "right_clause_digest",
                HashFieldClass::NamedPayloadDigest,
            ),
            (
                "semantic_policy_set",
                "output_exclusions/left_value/reading_digest",
                HashFieldClass::NamedPayloadDigest,
            ),
        ] {
            assert_eq!(
                by_key.get(&(schema.to_string(), path.to_string())),
                Some(&class),
                "{schema}/{path}"
            );
        }
    }
}
