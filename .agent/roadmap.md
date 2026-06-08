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
- [x] M0.0.3.4.5.1.1 85% 171K/200K hash classifier + a-l table half. check.rs:
  HashFieldClass {ArtifactRef, NamedPayloadDigest, RawRecordedBytes,
  FieldSpecific, Unresolved} + classify_hash_fields over walk_inventory
  HashNamed rows — suffix defaults (*_hash/*_hashes ArtifactRef, *_digest
  NamedPayloadDigest) overridden by an authored terminal-name exception
  table, one-line rationale per row; judge only terminal names
  lexicographically < "m" (85 names/123 paths at split:
  canonicalization_policy_hash, canonical_bytes_hash, config_hash,
  content_hash, environment_digest, executable_hash, grammar-payload
  family, index_fingerprint_hashes, …); defect-suspect names (digest
  semantics under *_hash, conventionless inputs) get Unresolved rows —
  .5.2's burn-down list; names >= "m" ride defaults unreviewed until
  .5.1.2; SPEC.md stays untouched. Tests (check_hash_* fns): real-SPEC
  totality (suffix defaults make every HashNamed row classify),
  provisional per-class counts, judged-half default + exception
  spot-checks. Read: §1.2 hash conventions + judged fields' S-decl
  context. Test: `cargo test -p ckc-schema check_hash`
- [x] M0.0.3.4.5.1.2 79% 158K/200K m-z table half. check.rs: judge remaining terminal
  names >= "m" (86 names/137 paths at split: payload/support/semantic
  digests, normalization/punctuation table hashes, query_hash,
  rationale_hash, reviewer_identity_hash, rust_type_hash, source_hash,
  spec_contract_hash, witness_payload_hash, …) — extend the exception +
  Unresolved table with rationale rows; finalize per-class count
  assertions; second-half default + exception spot-checks; SPEC.md stays
  untouched. Read: §1.2 hash conventions + judged fields' S-decl context.
  Test: `cargo test -p ckc-schema check_hash`
- [x] M0.0.3.4.5.2.1.1 69% 139K/200K hash-defect burn-down: §1.x cluster. Resolve 7 of the
  42 Unresolved terminal names: §1.1 bound-policy family
  (canonicalization_policy_hash, closure_bound_policy_hash,
  generator_static_bound_policy_hash, parser_bound_policy_hash — one shared
  fix), §1.6 locale_policy_hash + reproducibility_profile_hash,
  subject_hashes divergence (ValidationManifest §1.6 vs Incoherence §8.7).
  Per-name procedure (shared by all .2.x sub-units): reclassify with
  rationale where §1.2 already covers the field, else correct SPEC.md
  (rename to a convention suffix, define the computation beside the field,
  or drop the field; mind hash-cascade radius, memory 2026-06-01; a rename
  landing on plural *_digests extends the build.rs HashNamed suffix set in
  the same session); update HASH_FIELD_EXCEPTIONS (drop rows for removed
  fields); adjust per-class counts + spot-checks provisionally; reconcile
  spec.rs/build.rs test fallout. SPEC-edit design judgment is the
  deliverable — checker wiring waits for .2.3.2. Read: §1.2 hash
  conventions + each name's S-decl context.
  Test: `cargo test -p ckc-schema check_hash`
- [x] M0.0.3.4.5.2.1.2 74% 147K/200K hash-defect burn-down: §4.x cluster. Resolve 6 names:
  §4.3 family (closed_region_hash + closure_certificate_hash mutual-ref
  cycle, seed_region_hash, termination_argument_hash — second site
  ClosureBoundCertificate §7.1, one fix covers both), §4.4 entry_hashes
  (MechanicalLexicon), input_hash divergence (ExtractionManifest §4.4 vs
  VerifierWitness §9.1; the §1.2 alias row inherited_input is a cascade
  site). Procedure per .2.1.1. Read: §1.2 hash conventions + §4.3/§4.4 +
  each name's S-decl context. Test: `cargo test -p ckc-schema check_hash`
- [x] M0.0.3.4.5.2.2.1 66% 132K/200K hash-defect burn-down: §6.2 grammar family. Resolve 8
  names sharing one digest-semantics fix (payloads are not accepted
  artifacts; expect *_digest renames + payload-byte definitions beside the
  §6.2 grammar-artifact schemas): constrained_decoder_contract_hash,
  display_grammar_hash, first_follow_sets_hash, first_set_hash,
  follow_set_hash, parser_state_machine_hash, tagged_json_schema_hash,
  valid_next_token_masks_hash. Procedure per .2.1.1. Read: §1.2 hash
  conventions + §6.2 grammar-artifact schemas.
  Test: `cargo test -p ckc-schema check_hash`
- [x] M0.0.3.4.5.2.2.2 58% 115K/200K hash-defect burn-down: §6.3-§7.x cluster. Resolve 6
  names: reading_hash (§6.3), schema_collection_bounds_hash (§7.1),
  ProofNode/ProofDAG trio checker_hashes + reverse_dependency_index_hash +
  canonical_bytes_hash (§7.2, two shared S-decls; canonical_bytes_hash is
  self-referential), slot_value_hash (§7.5). Procedure per .2.1.1. Read:
  §1.2 hash conventions + each name's S-decl context.
  Test: `cargo test -p ckc-schema check_hash`
- [x] M0.0.3.4.5.2.2.3 51% 102K/200K hash-defect burn-down: §8.1/§9.1 cluster. Resolve 4
  names: left_clause_hash + right_clause_hash (one normalized-clause fix),
  minimality_proof_hash (§8.1 ctx_compatible constructs none),
  witness_payload_hash (§9.1 VerifierWitness, sole spec mention).
  Procedure per .2.1.1. Read: §1.2 hash conventions + §8.1 + §9.1 S-decl
  context. Test: `cargo test -p ckc-schema check_hash`
- [x] M0.0.3.4.5.2.3.1 78% 157K/200K hash-defect burn-down: §6.4 pre-acceptance cluster.
  Resolve the 11 remaining names (admission_decision_hash, candidate_hash,
  emitted_payload_hashes, forbidden_output_hashes,
  proposal_provenance_hashes, proposed_subject_hash, rationale_hash,
  required_output_hashes, reviewed_subject_hash, reviewer_identity_hash,
  score_record_hashes): author one shared pre-acceptance digest convention
  beside §1.2/§6.4 to cover most; proposal_provenance_hashes may instead
  add ProposalProvenanceManifest to the §3.1 inventory (build.rs
  role-table + count cascade; classify any newly walked fields
  in-session). Procedure per .2.1.1. Read: §1.2 hash conventions + §6.4.
  Test: `cargo test -p ckc-schema check_hash`
- [x] M0.0.3.4.5.2.3.2 84% 168K/200K literal_part_digests + checker wiring. Resolve
  WordingGateRecord.literal_part_digests (§9.3): extend the build.rs
  HashNamed suffix set (when no earlier sub-unit already did) or rename in
  SPEC. Wire classify_hash_fields into check_registry: Unresolved or
  unclassified path emits referential_integrity_error; finalize per-class
  count assertions with an empty Unresolved class. Tests: real SPEC +
  build_v0_registry clean; synthetic unclassified-suffix +
  lingering-Unresolved perturbations reject. Read: §1.2 hash conventions,
  §9.3 WordingGateRecord. Test: `cargo test -p ckc-schema check`
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
- [ ] M0.0.4.1 type-descriptor model + spec derivation. New
  `crates/ckc-core/src/descriptor.rs`: TypeExprRepr (mirrors spec.rs
  TypeExpr: name/optional/set/list/map + text {static StringPolicy id |
  dependent sibling field}), AltRepr (bare/typed/sexp/type_ref/alias),
  TypeDescriptor {type_id, record fields | enum alternatives}, Canonical
  impls via canon.rs macros. New `crates/ckc-schema/src/descriptor.rs`:
  derive_descriptors(&SpecDecls) — full table over S-decls + E-decls
  (229+116 at split; TypeExpr/EDecl conversion; Text param in
  StringPolicy ids → static, else dependent); per-inventory-entry
  union-transparent reachable closure (walker visited-set pattern but
  traversing E alternatives, unlike the leaf walk); rust_type_manifest =
  full sorted descriptor table, per-entry rust_type_hash = sha256 over
  the entry's sorted closure canonical bytes, manifest hash over table
  bytes. Tests: S/E count totality, schema_registry + text_literal
  closure spot-checks, determinism + descriptor roundtrip. Read: §1.1
  equivalence paragraphs, §1.3, §1.5.
  Test: `cargo test -p ckc-schema descriptor`
- [ ] M0.0.4.2 Rust-side descriptor emission + agreement. ckc-core
  descriptor.rs: trait CanonicalType { fn type_expr() -> TypeExprRepr }
  for scalars/composites (bool, Id, Hash, UInt, Int, Rational,
  FeaturePath, Text<P> via policy markers, Option/BTreeSet/Vec/BTreeMap);
  canonical_record!/bare_enum! additionally emit fn descriptor() per
  invocation (field names + CanonicalType::type_expr); explicit
  per-crate registries core_descriptors()/schema_descriptors() (later
  units append their types). Agreement test in ckc-schema: rust-emitted
  == spec-derived for every implemented type (UnicodePolicyManifest,
  Outcome, OperationResult, registry+bounds family); reconcile drift by
  fixing the Rust side — SPEC is authority. Read: §1.3, §1.5; canon.rs
  macro section. Test: `cargo test -p ckc-schema descriptor_agreement`
- [ ] M0.0.4.3 generated JSON Schema. New
  `crates/ckc-schema/src/json_schema.rs`: per-entry canonical JSON
  Schema document from descriptors (draft 2020-12; $defs over the entry
  closure): §1.3 string+pattern for Id/Hash/UInt/Int, Rational object
  schema, Text → string + x-ckc-string-policy annotation (dependent →
  sibling field name), Set/List → array, Map → object | pair-array per
  §1.5, Optional → omitted from required, non-bare E → oneOf
  {tag,value}/bare-tag forms, bare E → string enum; collection bounds
  stay out of the docs (T-Schema-Equivalence canonicalizes
  SchemaBoundManifest separately). Canonical document bytes via direct
  ObjectEmitter recursion (or a small canon.rs JSON-value emitter).
  generated_json_schema_hash per entry; manifest = sorted (schema_id,
  hash) pairs + manifest hash. Tests: every entry generates, full-doc
  byte spot-check on a small entry, determinism. Read: §1.1 equivalence
  paragraphs, §1.3, §1.4 policy ids, §1.5.
  Test: `cargo test -p ckc-schema json_schema`
- [ ] M0.0.4.4 registry rewiring + T-Schema-Equivalence gate. build.rs:
  replace placeholders — per-entry rust_type_hash/
  generated_json_schema_hash from the M0.0.4.1/.3 manifests;
  tagged_union_alternatives_hash = Some(sha256 over the entry's sorted
  reachable non-bare-alternative E descriptors), None when the closure
  reaches none; rust_type_manifest_hash/
  generated_json_schema_manifest_hash; canonicalization_policy_hash =
  sha256(canonical_payload_bytes(accepted §1.4 UnicodePolicyManifest
  fixture, ckc-core policy.rs)) per §1.2 ArtifactRef + envelope
  artifact_hash definition. Design fork (decide + one-line SPEC
  clarification): union-interior Text sites — FeaturePath-through-union
  addressing for StringPolicyBinding rows (TextLiteral.value,
  dependent_policy_field=policy) vs type-level dependent binding via
  descriptors with binding rows staying walk-scoped. check.rs
  check_schema_equivalence: recompute-and-compare every §1.1-listed
  input; any disagreement → schema_authority_mismatch (v0 has no
  version-bump path); sorted CheckReport. Fallout: build.rs placeholder
  tests + registry-pinning tests. Gate
  `crates/ckc-schema/tests/t_schema_equivalence.rs`: real SPEC +
  build_v0_registry clean; perturbations (mutated entry hash, dropped
  binding row, stale manifest/policy hash, mutated union set) reject.
  Read: §1.1, §1.2, §1.4 fixture, §6.2 TextLiteral.
  Gate: `cargo test -p ckc-schema --test t_schema_equivalence`
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
