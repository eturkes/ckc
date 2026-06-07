# CKC Roadmap

Flat ordered checklist over the SPEC.md §11.3 build-unit table, which is the
single canonical build plan: unit scopes, dependencies, and acceptance gates
live there, not here. `.agent/prompt.md` defines the session protocol that
consumes this list. A trailing `NN% NNNK/200K` on a completed unit records the
session's context usage at completion via `.agent/compaction.sh`; splitting a
unit replaces its line with `M0.x.y.z` sub-lines per the prompt's Splitting
rule.

## M0

- [x] M0.0.1 81% 161K/200K
- [x] M0.0.2.1 68% 136K/200K §1.5 writer core. New `crates/ckc-core/src/canon.rs` + lib.rs
  `pub mod canon;`: byte emitters for minimal-escape string (escape only
  U+0022, U+005C, controls as lowercase `\u00xx`), bool, array, object
  (members sorted by UTF-8 field-name bytes, duplicate-field rejection);
  trait `Canonical` (assoc const `TYPE_ID` — provisional symbol ids pending
  M0.0.3 registry — plus byte emission) and free fn `canonical_payload_bytes`;
  manual impls for bool, `Id`, `Hash`, `UInt`, `Int`, `Rational`, `Text<P>`
  reusing `to_decimal`/`as_str`; reconcile with existing serde Serialize
  impls so canonical bytes have one authority. Prior art:
  archive/phase0-research-kernel serializer. Read: §1.5 grammar.
  Test: `cargo test -p ckc-core canon`
- [x] M0.0.2.2 47% 93K/200K sort keys + collections in `canon.rs`:
  `canonical_sort_key(x) = (TYPE_ID, canonical_payload_bytes(x))`; `Set[T]`
  encodes as array sorted by it after dedup; `List[T]` keeps semantic order;
  `Map[K,V]` encodes as object iff K satisfies `is_identifier_ascii`, else
  sorted `{key,value}` pair array; duplicate-map-key rejection;
  accepted-reference sort helper (artifact_hash, then schema_id, then
  reference field name). Read: §1.5 sort-key/set/map paragraphs, §1.2
  hash-field conventions. Test: `cargo test -p ckc-core canon`
- [x] M0.0.2.3 86% 172K/200K unions, strict reading, gate. Tagged-union `{"tag","value"}`
  encoding (bare tag → `{}` value, duplicate-tag rejection); decide and
  document E-enum encoding (Id string vs tagged object); §1.7
  `OperationResult<T>` + §2 `Outcome` as first unions in new
  `crates/ckc-core/src/outcome.rs`; strict canonical reading rejecting JSON
  null, numeric tokens, duplicate object fields/map keys/union tags
  (re-serialize-compare, per `Text::from_canonical` /
  `Rational::from_canonical_parts`); injection proptests in
  `crates/ckc-core/tests/prop.rs`; new
  `crates/ckc-core/tests/t_canonical_bytes.rs` covering serializer injection,
  `canonical_sort_key` totality, union encoding, repeated `Hash::of_bytes`
  identity. Read: §1.1 schema-validation-rejects paragraph, §1.2, §1.5 union
  rule, §1.7, A.10.
  Gate: `cargo test -p ckc-core --test t_canonical_bytes`
- [x] M0.0.3.1 69% 138K/200K registry types. New crate `crates/ckc-schema` (add to workspace
  members; dep ckc-core): `src/registry.rs` SchemaRegistry, SchemaEntry,
  SchemaRole, StringPolicyBinding, SourceSupportAlias, SourceSupportAliasKind;
  `src/bounds.rs` SchemaBoundManifest, SchemaCollectionBound,
  BoundOverflowDisposition (types only; HandleBoundOverflow emission lands
  with its first §8.7 consumer). FeaturePath (§1.3 List[Id] newtype) in
  `crates/ckc-core/src/scalar.rs` + Canonical impl in canon.rs. E enums
  follow outcome.rs Outcome (id/from_id/ALL + string encoding); records
  follow UnicodePolicyManifest Canonical + strict Deserialize; one composed
  registry+bounds roundtrip via from_canonical_bytes + one
  optional-field-omission case. Read: §1.1 schema block, §1.2 alias table,
  §1.3 FeaturePath row. Test: `cargo test -p ckc-schema`
- [x] M0.0.3.2 72% 144K/200K spec extractor. `crates/ckc-schema/src/spec.rs`: line parser
  over SPEC.md bytes -> SpecDecls: S-decls (name, generic param, fields:
  TypeExpr), E-decls (alternative shapes: bare | name:Type | (sexp ArgTypes)
  | TypeRef | alias e.g. E RoleName = Id), T-tables by section (Rule,
  Command, Stage, §11.3 unit+obligation, §9.2 certificate), markdown-header
  section anchors, §6.2 builtin-definition names, §3.1 inventory block;
  §1.3 scalars predeclared as axioms. TypeExpr grammar: Base | Base? |
  Set[X] | List[X] | Map[K,V] | Text<p> | Name[T], nested. Tests: S/E
  counts equal an in-test independent line scan; spot-checks (SchemaRegistry
  fields, Premise sexps, OperationResult[T]); zero unparseable field types
  spec-wide. Test: `cargo test -p ckc-schema spec`
- [x] M0.0.3.3 82% 164K/200K symbol table + resolution (§1.1 steps 1-3).
  `crates/ckc-schema/src/symtab.rs` + `src/check.rs`: SymbolKind {schema,
  enum, enum_variant, union_alternative (both qualified Enum.name), builtin,
  proof_rule, certificate_class, gate, acceptance_gate, cli_operation, stage,
  section_anchor}; duplicate (kind,id)-divergent-anchor rejection; resolution
  of field types, enum refs, union alternatives + sexp arg types, builtin
  name<->§6.2 definition bijection, T Rule conclusions, T Command outputs +
  operations, stage-table operations, body-wide §-anchor refs
  (capitalized-token rule skips prose in table cells); §11.3 unit<->gate
  bijection. Checker-local sorted diagnostics struct,
  code=referential_integrity_error (§8.7 Diagnostic artifact defers). Expect
  spec-defect fallout; SPEC.md corrections in-scope. Tests: real SPEC
  resolves clean; synthetic duplicate + dangling-ref perturbations reject.
  Test: `cargo test -p ckc-schema check`
- [x] M0.0.3.4.1 70% 139K/200K type-graph walker. New `crates/ckc-schema/src/build.rs` +
  lib.rs `pub mod build;`: WalkedPath {schema_id, path: FeaturePath, leaf}
  rows over SpecDecls — transitive walk from each §3.1 inventory root,
  recurse TypeExpr::Name into nested SDecls (visited-set on cycles; Optional
  transparent; generic args walk through); leaf kinds = collection
  (Set/List/Map + enum-domain-key flag for Map keys resolving via e_decl),
  Text policy id, hash-named field (*_hash/*_hashes/*_digest); helper
  schema_has_source_support (§1.2 canonical field/alias-default names;
  shared by .4.2 authoring + .4.4 checker). Tests: synthetic decls (nested,
  cyclic, enum-domain map, generic), real-SPEC spot-checks + nonzero
  per-leaf-family counts. Read: §1.1 steps 3-4, §1.2 alias table, §3.1.
  Test: `cargo test -p ckc-schema build`
- [x] M0.0.3.4.2 87% 174K/200K SchemaEntry + alias rows. build.rs: authored
  schema_id->SchemaRole const table, one row per SpecDecls.inventory entry
  (roles from §2 Authority + §3.2 producer position +
  schema_has_source_support; non-obvious rows get one-line rationale);
  SchemaEntry per row — placeholder
  rust_type_hash/generated_json_schema_hash = sha256(S-decl line bytes)
  pending M0.0.4, tagged_union_alternatives_hash = None (every inventory row
  is an S-decl); SourceSupportAlias rows via §1.2 fixed-default field-name
  match over walked paths. Tests: entry count = inventory len, role + alias
  spot-checks, role/support consistency. Read: §1.2 role rule + alias table,
  §2 Authority rows, §3.2 producer table.
  Test: `cargo test -p ckc-schema build`
- [x] M0.0.3.4.3 57% 113K/200K binding/bound rows + v0 assembly. build.rs
  build_v0_registry(spec_bytes) -> (SchemaRegistry, SchemaBoundManifest):
  StringPolicyBinding per walked Text<p> path (dependent_policy_field = None
  for v0 unless a §1.4 algorithm names a sibling field);
  SchemaCollectionBound per walked collection path minus enum-domain-Map
  exemptions (authored DEFAULT_MAX_ITEMS + sparse per-path override consts;
  disposition reject_with_diagnostic); assembly — spec_contract_hash =
  sha256(SPEC.md bytes), schema_bound_manifest_hash over built-manifest
  canonical bytes, remaining *_hash fields placeholder = sha256 of named
  §-anchor line bytes pending M0.0.4/M0.0.5 (document per-field). Tests:
  per-family row counts vs independent line scan, binding spot-checks,
  composed from_canonical_bytes roundtrip. Read: §1.1 bound paragraphs, §1.2
  hash conventions, §1.4, §1.5. Test: `cargo test -p ckc-schema build`
- [x] M0.0.3.4.4 60% 119K/200K structural coverage checkers. check.rs
  check_registry(text, &SchemaRegistry, &SchemaBoundManifest) -> CheckReport
  (peer of check_spec; CheckIssue code=referential_integrity_error): bound
  coverage — exactly one SchemaCollectionBound per walk_inventory Collection
  leaf (enum_domain_key exempt) and zero bound rows off the walk; §1.2
  source-support/role rule — schema_has_source_support or registered alias
  vs SchemaEntry.schema_role non-semantic set. Tests: real SPEC +
  build_v0_registry clean; perturbations (dropped/extra bound row, wrong
  role) reject. Read: §1.1 step 4 + bound paragraphs, §1.2 role rule.
  Test: `cargo test -p ckc-schema check`
- [ ] M0.0.3.4.5.1 hash-field classifier + authored table. check.rs:
  HashFieldClass {ArtifactRef, NamedPayloadDigest, RawRecordedBytes,
  FieldSpecific, Unresolved} + classify_hash_fields over walk_inventory
  HashNamed rows (260 paths/171 terminal names at split time) — suffix
  defaults (*_hash/*_hashes ArtifactRef, *_digest NamedPayloadDigest)
  overridden by an authored terminal-name exception table, one-line
  rationale per row (spec_contract_hash, rust_type_hash,
  canonicalization_policy_hash, executable/fingerprint families, …);
  defect-suspect names (digest semantics under *_hash, conventionless
  inputs) get Unresolved rows — .5.2's burn-down list; SPEC.md stays
  untouched this unit. Tests (check_hash_* fns): real-SPEC totality (every
  HashNamed row classifies), per-class counts, default + exception
  spot-checks. Read: §1.2 hash conventions.
  Test: `cargo test -p ckc-schema check_hash`
- [ ] M0.0.3.4.5.2 hash-convention resolution + checker wiring. Burn down
  every Unresolved row: reclassify with rationale where §1.2 already covers
  the field, else correct SPEC.md per field (rename to convention suffix or
  define the computation beside the field; mind hash-cascade radius, memory
  2026-06-01) — expect spec-defect fallout; wire classify_hash_fields into
  check_registry: Unresolved or unclassified path emits
  referential_integrity_error. Tests: real SPEC + build_v0_registry clean
  with empty Unresolved class; synthetic unclassified-suffix +
  lingering-Unresolved perturbations reject. Read: §1.2 hash conventions +
  each Unresolved field's S-decl context.
  Test: `cargo test -p ckc-schema check`
- [ ] M0.0.3.4.6 producer-mapping checker. check.rs §3.2 reverse coverage:
  every SpecDecls.inventory payload named in a stage-producer TTable
  emitted-artifacts cell (reuse resolve_artifact_cell name handling) or in
  the authored control-emission allowlist (ProducerManifest,
  ValidationManifest, ToolchainManifest, EnvironmentProfile, ToolRecord,
  FiniteFixtureManifest + FrozenConstant/ParsedQuantity/DiagnosticTag rows);
  missing mapping emits producer_mapping_error. Expect spec-defect fallout;
  SPEC.md corrections in-scope. Tests: real SPEC clean; unmapped-payload
  perturbation rejects. Read: §3.2 producer table + control-emission rule.
  Test: `cargo test -p ckc-schema check`
- [ ] M0.0.3.4.7 local-bound dispatch + steps 1-2/5 + gate (completes
  T-Registry-Referential-Integrity). check.rs: local bound objects lacking
  BoundOverflowDisposition (e.g. CollectBound, dispatch defined beside §6.2
  collect) must name a consuming-algorithm dispatch — §6.2 reading = its
  bound lines only; registry-declared schema_ids feed symtab duplicate
  rejection (steps 1-2); step-5 ok/sorted diagnostics. Gate test
  `crates/ckc-schema/tests/t_registry_referential_integrity.rs`: clean over
  real SPEC + built registry; perturbations (dropped bound row, wrong role,
  duplicate entry, unmapped payload) reject.
  Gate: `cargo test -p ckc-schema --test t_registry_referential_integrity`
- [ ] M0.0.4
- [ ] M0.0.5
- [ ] M0.0.6
- [ ] review M0.0
- [ ] M0.1.1
- [ ] M0.1.2
- [ ] M0.1.3
- [ ] review M0.1
- [ ] M0.2.1
- [ ] M0.2.2
- [ ] review M0.2
- [ ] M0.3.1
- [ ] M0.3.2
- [ ] M0.3.3
- [ ] M0.3.4
- [ ] M0.3.5
- [ ] M0.3.6
- [ ] M0.3.7
- [ ] M0.3.8
- [ ] review M0.3
- [ ] M0.4.1
- [ ] M0.4.2
- [ ] M0.4.3
- [ ] M0.4.4
- [ ] M0.4.5
- [ ] M0.4.6
- [ ] M0.4.7
- [ ] M0.4.8
- [ ] review M0.4
- [ ] M0.5.1
- [ ] M0.5.2
- [ ] M0.5.3
- [ ] M0.5.4
- [ ] M0.5.5
- [ ] M0.5.6
- [ ] M0.5.7
- [ ] M0.5.8
- [ ] review M0.5
- [ ] M0.6.1
- [ ] M0.6.2
- [ ] M0.6.3
- [ ] M0.6.4
- [ ] M0.6.5
- [ ] review M0.6
- [ ] review M0
- [ ] GATED.1 (user-selected: confirm which deferred §3.3 gate before starting)
