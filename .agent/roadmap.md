# CKC Roadmap

Flat ordered checklist over the SPEC.md §11.3 build-unit table, which is the
single canonical build plan: unit scopes, dependencies, and acceptance gates
live there, not here. The `/session-prompt` command (`.claude/commands/session-prompt.md`)
defines the session protocol that consumes this list. A trailing
`NN% NNNK/200K` on a completed unit records the session's context usage at
completion via `.agent/compaction.sh`; splitting a unit replaces its line with
`M0.x.y.z` sub-lines per the command's Splitting rule.

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
- [x] M0.0.3.4.6 50% 100K/200K producer-mapping checker. check.rs §3.2 reverse coverage:
  every SpecDecls.inventory payload named in a stage-producer TTable
  emitted-artifacts cell (reuse resolve_artifact_cell name handling) or in
  the authored control-emission allowlist (ProducerManifest,
  ValidationManifest, ToolchainManifest, EnvironmentProfile, ToolRecord,
  FiniteFixtureManifest + FrozenConstant/ParsedQuantity/DiagnosticTag rows);
  missing mapping emits producer_mapping_error. Expect spec-defect fallout;
  SPEC.md corrections in-scope. Tests: real SPEC clean; unmapped-payload
  perturbation rejects. Read: §3.2 producer table + control-emission rule.
  Test: `cargo test -p ckc-schema check`
- [x] M0.0.3.4.7 84% 169K/200K local-bound dispatch + steps 1-2/5 + gate (completes
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
- [ ] M0.0.5.1 envelope + store foundation. New crate `crates/ckc-store` (add
  to workspace members; dep ckc-core): `src/envelope.rs` ArtifactEnvelope<T:
  Canonical> — §1.2 9-field generic envelope (artifact_hash, schema_id,
  schema_version, schema_hash, canonicalization_policy_hash,
  producer_manifest_hash, replay_manifest_hash, accepted_effect_row:Set[Effect],
  payload:T); manual Canonical via ObjectEmitter (canonical_record! is
  non-generic) + strict Deserialize; constructor sets artifact_hash =
  Hash::of_bytes(canonical_payload_bytes(payload)) (field outside its own hash
  input). `src/store.rs` content-addressed path from artifact_hash (archive CAS
  layout). Effect enum (§2) via bare_enum! in ckc-core beside outcome.rs. Defer
  §1.2 proof_roots/source_support projection to its first semantic consumer
  (control payloads role-exempt). Register new descriptors; keep
  descriptor_agreement/T-Schema-Equivalence green. Tests: envelope roundtrip
  (from_canonical_bytes over UnicodePolicyManifest), artifact_hash = payload
  hash independent of envelope fields, store-path determinism, Effect
  id/from_id. Read: §1.2, §2 Effect, §1.5; canon.rs ObjectEmitter.
  Test: `cargo test -p ckc-store`
- [ ] M0.0.5.2 runtime manifests + ValidateRuntimeManifests. ckc-store
  `src/manifest.rs`: ToolchainManifest, ToolRecord (executable_hash/config_hash
  optional), EnvironmentProfile (network_policy/clock_policy:Effect) via
  canonical_record! (sets for tool_records/*_hashes). ValidateRuntimeManifests
  (`ckc runtime validate`, §11.1: accept authored ToolchainManifest +
  EnvironmentProfile, validate embedded ToolRecord rows) -> OperationResult,
  sorted §1.7 diagnostics. Register descriptors; keep agreement green. Tests:
  per-record roundtrip + optional-omission; validate accepts a well-formed
  fixture, rejects a malformed ToolRecord (invalid). Read: §1.6 toolchain/
  environment rows, §11.1 runtime-validate wrapper, §1.7.
  Test: `cargo test -p ckc-store manifest`
- [ ] M0.0.5.3 replay/producer manifests. ckc-store `src/manifest.rs`:
  ProducerManifest, ReplayManifest, ReplayIdentityCheck, ValidationManifest via
  canonical_record! (sets for *_hashes/accepted_effect_row); ReplayIdentity-
  Outcome enum (§2) via bare_enum! at the ReplayIdentityCheck site
  (schema-local). Register descriptors; keep agreement green. Tests: per-record
  roundtrip; ReplayIdentityOutcome id/from_id; §1.6 canonical-field presence.
  Read: §1.6 producer/replay/validation rows, §2 ReplayIdentityOutcome.
  Test: `cargo test -p ckc-store replay`
- [ ] M0.0.5.4 replay stratum boundary skeleton + gate. ckc-store
  `src/replay.rs`: over a generic stratum-member view (artifact_hash,
  replay_manifest_hash?, replay_identity_hashes, is_replay_check — no §9.2
  Certificate dep; full ReplayIdentity recompute is M0.6.4), decide whether
  ReplayManifest.expected_output_hashes is a closed prior issuance stratum under
  the §1.6 boundary rule (exclude the manifest payload, members whose
  replay_manifest_hash equals that manifest hash, the enclosing
  ReplayIdentityCheck, and replay-checks/certificates citing it); validate
  referenced ProducerManifest/ToolchainManifest/EnvironmentProfile hashes
  resolve. Sorted §1.7 diagnostics. Gate
  `crates/ckc-store/tests/t_replay_manifest_boundary.rs`
  (T-Replay-Manifest-Boundary): synthetic RM-PRODUCER-BASE/RM-DEMO-CORE-shaped
  stratum (A.10 boundary note) clean; perturbations (included excluded member,
  dangling producer/toolchain/environment hash, manifest self-inclusion)
  reject. Read: §1.6 boundary rule + ReplayIdentity steps, A.10 RM-*/RIC-*.
  Gate: `cargo test -p ckc-store --test t_replay_manifest_boundary`
- [ ] M0.0.6.1 ckc-cli crate + argv parser + CanonicalCommand model. New
  crate `crates/ckc-cli` (add to workspace members; deps ckc-core,
  ckc-schema): `src/command.rs` CanonicalCommand — one variant per §11.1
  command-table row[0] (21 names `ckc schema check`…`ckc demo m0`), parsed
  from `ckc <namespace> <verb>` argv (2-token `ckc close`/`ckc replay`,
  3-token `ckc demo m0`); CloseM0-internal BuildTerminologyClosure/
  BuildDiagnostics are not variants (they route through `ckc close`). The
  command→operation→artifact authority stays the SPEC §11.1 table (.6.3
  proves the enum bijects with ckc-schema `command_table` over parsed
  SpecDecls), never a re-authored const. CLI model is CLI-internal (absent
  from §3.1 inventory) → no registry/descriptor registration,
  descriptor_agreement + T-Schema-Equivalence untouched; plain enum
  (bare_enum! only if canonical bytes wanted). Tests: parse round-trip for
  all 21 names, unknown/misspelled + CloseM0-internal-name rejection. Read:
  §11.1 command surface + command-to-operation map.
  Test: `cargo test -p ckc-cli command`
- [ ] M0.0.6.2 structured diagnostic writer + repository layout check.
  `crates/ckc-cli/src/diagnostic.rs`: CLI-boundary writer serializing a
  command wrapper's §1.7 OperationResult + a sorted ckc-schema CheckReport
  (CheckIssue rows) to canonical bytes via ckc-core canon.rs emitters —
  reuse, never redefine, outcome.rs OperationResult/Outcome. `src/layout.rs`:
  §11.2 reserved-namespace check — present `crates/*` directory names ⊆ the
  15 reserved crate names (ckc-cli…ckc-report), SPEC.md + crates/ present;
  crate-on-first-use convention → membership only (absent reserved crates and
  extra non-crate top-level entries like Cargo.toml/.agent are not errors).
  Tests: writer determinism + sorted-output spot-check; layout accepts the
  real repo, rejects a synthetic `crates/ckc-bogus`. Read: §11.2 layout
  (reuse existing OperationResult/CheckReport).
  Test: `cargo test -p ckc-cli diagnostic layout`
- [ ] M0.0.6.3 T-CLI-Contract checker + gate (completes T-CLI-Contract).
  `crates/ckc-cli/src/check.rs`: check_cli_contract(parser surface,
  &SpecDecls) — the ckc-cli CanonicalCommand surface bijects with §11.1
  `command_table` row[0]; each command names exactly one wrapper row with a
  non-empty primary emitted-artifact set (operation cell may be compound,
  e.g. `BuildMatches and BuildMatchClasses`); CloseM0-internal suboperations
  BuildTerminologyClosure/BuildDiagnostics resolve via `ckc close`, never as
  top-level commands; emit ckc-schema CheckReport/CheckIssue,
  code=cli_contract_error. Distinct from ckc-schema check_command_table
  (operations→§3.2 stage, T-Registry-Referential-Integrity); this checks the
  runtime surface vs §11.1. Expect spec-defect fallout; SPEC.md corrections
  in-scope. Gate `crates/ckc-cli/tests/t_cli_contract.rs`: clean over the
  real §11.1 table + parser; perturbations (spec command absent from parser,
  parser name absent from §11.1, empty artifact set, CloseM0-internal
  suboperation promoted to a top-level command) reject. Read: §11.1 wrapper
  convention + command table, §11.3 T-CLI-Contract.
  Gate: `cargo test -p ckc-cli --test t_cli_contract`
- [ ] review M0.0
- [ ] M0.1.1.1 ckc-source crate + source/permission/corpus schemas. New crate
  `crates/ckc-source` (add to workspace members; dep ckc-core): §4.1 records via
  canonical_record! — SourceEdition, SourcePermissionRecord
  (allowed_artifacts:Set[AllowedArtifact]), CorpusDocument; §4.1 enums via
  bare_enum! — RedistributionStatus, AllowedArtifact; SourceClass (§2 vocabulary,
  first consumer) via bare_enum! in ckc-core beside outcome.rs. Add ckc-core Text
  policy markers for the §1.4 policies these fields first use (semantic_ja,
  semantic_en, raw_source) so descriptor emission resolves. Append
  source_descriptors() and wire into ckc-schema descriptor_agreement; reconcile
  rust-emitted vs spec-derived (SPEC authority). These §3.1 inventory rows are
  already covered by the spec-derived registry/bounds/hash-class/producer-map
  (M0.0.3/.4) — no build.rs/checker edit. Tests: per-record roundtrip +
  optional-field omission, enum id/from_id, descriptor_agreement green. Read: §4.1
  schemas, §2 SourceClass, §1.4 policy ids. Test: `cargo test -p ckc-source`
- [ ] M0.1.1.2 Residual schema + permission residual projection + gate (completes
  T-Source-Permission). ckc-core scalar.rs: RegionId, ProofId §1.3 Id-newtypes
  (first consumer — §1.2 envelope deferred proof_roots/source_support) +
  CanonicalType. ckc-core shared diagnostics: ResidualClass via bare_enum! (11
  variants), Residual via canonical_record! (source_regions:Set[RegionId],
  proof_roots:Set[ProofId], diagnostic:Text<diagnostic_text> — add that policy
  marker); append descriptors + reconcile agreement. ckc-source
  `src/permission.rs`: permission residual projection — over a
  SourcePermissionRecord and a requested AllowedArtifact view set, emit
  Residual(class=permission_limited) per kind absent from allowed_artifacts (§4.1
  rule; full §9.3 report-build integration defers to M0.6.3). Gate
  `crates/ckc-source/tests/t_source_permission.rs` (T-Source-Permission): A.1
  SRC-GDL (reconstructable) / SRC-PI (restricted_internal_only) fixtures — allowed
  views project clean, a source_bytes view over SRC-PI yields permission_limited
  (A.10 expectation); perturbations (disallowed kind admitted, missing residual)
  reject. Read: §4.1 permission semantics, §8.7 Residual, §9.3 step 6.
  Gate: `cargo test -p ckc-source --test t_source_permission`
- [ ] M0.1.2.1 node/edge skeleton schemas + kind enums. New ckc-source
  `src/graph.rs` (lib.rs `pub mod graph;`): §4.2 records via canonical_record! —
  SourceNode, SourceNodeAttrs (label:Text<semantic_ja>?, table_id:Id?), SourceEdge
  (from/to:Id), SourceEdgeAttrs (role:Id?, table_id:Id?,
  row_index/column_index/reading_order:UInt?); §2-vocabulary enums via bare_enum!
  in ckc-core beside outcome.rs — SourceNodeKind (21 variants), SourceEdgeKind (8
  variants), first consumer SourceNode.kind/SourceEdge.kind (SourceClass
  precedent). Append enum descriptors to core_descriptors(), records to
  source_descriptors(); keep ckc-schema descriptor_agreement green (rust-emitted ==
  spec-derived, SPEC authority). §3.1 inventory rows already covered by the
  spec-derived registry/bounds/hash-class/producer-map (M0.0.3/.4) — no
  build.rs/checker edit. Tests: per-record roundtrip + optional-field omission,
  enum id/from_id, descriptor_agreement green. Read: §4.2 node/edge schemas, §2
  SourceNodeKind/SourceEdgeKind + §2.1 consumers row.
  Test: `cargo test -p ckc-source graph`
- [ ] M0.1.2.2 span/anchor/geometry schemas + source ordering. ckc-source
  graph.rs: §4.2 records via canonical_record! — SourceSpan (16 fields:
  section_path:List[Text<semantic_ja>], page/bbox/table_cell_id optional,
  char_start/char_end:UInt, raw_text:Text<raw_source>, nfkc_text:Text<source_nfkc>,
  search_text:Text<semantic_ja>, display_text:Text<view_text>, language:Lang,
  reading_order:UInt), SourceAnchor, BBox (top/left/bottom/right:Rational); Lang
  via bare_enum! in graph.rs (§4.2-local). Add ckc-core Text policy markers
  source_nfkc, view_text (§1.4; raw_source/semantic_ja from M0.1.1.1) so descriptor
  emission resolves. ckc-core canon.rs beside canonical_sort_key: SourceOrderView
  (optional source-order fields) + source_order_key -> the §1.5 11-tuple
  (source_edition_hash, page_or_zero, reading_order, bbox top/left/bottom/right,
  node_id, char_start, char_end, anchor_id) bytes, missing field = type canonical
  minimum; ckc-source impls the view per SourceSpan/SourceAnchor (ckc-core cannot
  dep ckc-source). Append descriptors to source_descriptors(); keep
  descriptor_agreement green. Tests: per-record roundtrip + optional-omission, Lang
  id/from_id, source_order_key tuple + missing-field-minimum spot-checks, agreement
  green. Read: §4.2 SourceSpan/SourceAnchor/BBox/Lang, §1.4 source_nfkc/view_text,
  §1.5 source_order_key. Test: `cargo test -p ckc-source graph`
- [ ] M0.1.2.3 fixture leaf content (A.1 corpus). New ckc-source `src/fixture.rs`
  (lib.rs `pub mod fixture;`): author SourceSpan + SourceAnchor + leaf SourceNode
  (sentence + table-cell kinds) for A.1 units U1-U27 — raw/nfkc/search/display text
  per unit, char offsets, reading_order, language; SRC-PI unit U2 flagged for the
  .5 permission check; U22 conflicting-offsets unit gets no stable span (drives
  extraction_uncertain in .5). Extract A.1 strings programmatically from SPEC.md
  (memory 2026-06-07 fullwidth/ASCII trap). Constructors
  fixture_spans()/fixture_anchors()/fixture_leaf_nodes() for .4 assembly. Tests:
  leaf-content roundtrip, source_order_key ascending over fixture_spans(), expected
  per-unit counts. Read: §4.2, A.1 U1-U27, A.2 src= references.
  Test: `cargo test -p ckc-source fixture`
- [ ] M0.1.2.4 SourceGraph container + structure wiring + assembly. ckc-source
  graph.rs: SourceGraph via canonical_record! (graph_id:Id,
  source_edition_hash:Hash, nodes/edges/spans/anchors:Set, root_node_id:Id,
  extraction_manifest_hash:Hash) + descriptor + agreement. fixture.rs: container
  SourceNode values (document/section/heading/table/row/column/cell/caption/
  footnote/cross_reference_anchor) + every SourceEdge across the 8 SourceEdgeKind
  (contains, precedes, table_coordinate, header_of, caption_of, footnote_of,
  continuation, crossref_targets) wiring U1-U27 incl. table U3 (rows/columns/cells/
  header), caption U15->U3, footnote U16, crossref U14->U3, dangling U18; assemble
  fixture_source_graph() (root_node_id, four Sets, source_edition_hash = A.1
  SRC-GDL edition hash, extraction_manifest_hash = fixture-literal Hash —
  ExtractionManifest schema defers to M0.2.1). Append descriptor; keep agreement
  green. Tests: assembled-graph roundtrip, P-SG-canonical byte-stability (re-derive
  identical canonical bytes). Read: §4.2 SourceGraph/SourceEdgeKind, §4.4
  extraction_manifest_hash note, A.1 U3/U14-U16/U18 structure.
  Test: `cargo test -p ckc-source fixture`
- [ ] M0.1.2.5 P-SG predicates + gate (completes T-SourceGraph-Canonical). New
  ckc-source `src/check.rs`: validate_source_graph(&SourceGraph,
  &Set[SourcePermissionRecord]) -> OperationResult emitting sorted §8.7
  Residual/diagnostics — P-SG-total-text (every A.1 textual unit has a SourceSpan +
  SourceAnchor or a Residual(class=extraction_uncertain), U22), P-SG-total-support
  (vacuous pre-theorem; full SourceRegion check lands with §4.3 in M0.1.3),
  P-SG-canonical (re-derived SourceGraph canonical bytes identical), P-SG-permission
  (any raw_source-bearing artifact's source allowed by
  SourcePermissionRecord.allowed_artifacts — SRC-PI source_graph disallowed). Reuse
  ckc-core Residual/ResidualClass + ckc-source permission projection (M0.1.1.2).
  Gate `crates/ckc-source/tests/t_sourcegraph_canonical.rs` (T-SourceGraph-
  Canonical): the .4 fixture over SRC-GDL/SRC-PI permissions validates clean +
  byte-stable; perturbations (textual unit missing span without residual, mutated
  raw_text, SRC-PI raw-text artifact disallowed, reordered Set breaking canonical
  bytes) reject. Read: §4.2 P-SG predicates, §4.1 allowed_artifacts, §8.7 Residual.
  Gate: `cargo test -p ckc-source --test t_sourcegraph_canonical`
- [ ] M0.1.3.1 §8.7 diagnostic family schemas. Extend ckc-core's shared-diagnostics
  module (beside Residual/ResidualClass, M0.1.1.2; `diagnostic.rs`): Diagnostic
  (subject_hash:Hash? optional), DiagnosticRef, Ambiguity (alternatives:Set[Hash]),
  Incoherence (subject_hashes:Set[Hash]) via canonical_record! (all source_regions:
  Set[RegionId], proof_roots:Set[ProofId]); AmbiguityClass (multiple_readings|
  multiple_terms), IncoherenceClass (functional_key_collision|
  mutually_exclusive_term_mapping|incompatible_generator_outputs) via bare_enum!.
  Append descriptors to core_descriptors(); keep ckc-schema descriptor_agreement green
  (rust-emitted == spec-derived, SPEC authority). §3.1 inventory rows already covered by
  the spec-derived registry/bounds/hash-class/producer-map (M0.0.3/.4) — no
  build.rs/checker edit. Tests: per-record roundtrip + Diagnostic optional-omission, enum
  id/from_id, agreement green. Read: §8.7 schemas, §2 AmbiguityClass/IncoherenceClass.
  Test: cargo test -p ckc-core diagnostic
- [ ] M0.1.3.2 §4.3 region schemas. New ckc-source `src/region.rs` (lib.rs
  `pub mod region;`): RegionMember tagged union (node:Id|span:Id|cell:Id|anchor:Id) —
  first source-domain union, hand-written {tag,value} Canonical + strict Deserialize +
  descriptor (AltRepr typed alternatives), following outcome.rs Outcome; SourceRegion
  (region_id:RegionId from M0.1.1.2, seed_members/closed_members:Set[RegionMember],
  closure_certificate_hash:Hash), RegionClosureCertificate (added_member_batches:
  List[Set[RegionMember]], residual_hashes:Set[Hash]) via canonical_record!. Append
  descriptors to source_descriptors(); keep descriptor_agreement green. §3.1 inventory
  rows already covered (M0.0.3/.4) — no build.rs/checker edit. Tests: per-record
  roundtrip, RegionMember tag-encoding spot-check, agreement green. Read: §4.3 schemas,
  §1.5 union rule. Test: cargo test -p ckc-source region
- [ ] M0.1.3.3 HandleBoundOverflow dispatch (first §8.7 overflow consumer). New ckc-schema
  `src/overflow.rs` (lib.rs `pub mod overflow;`): handle_bound_overflow(
  &SchemaCollectionBound, subject_hash:Hash, candidate_members, producer_id:Id) ->
  internal primary (invalid|residual|ambiguity|incoherence) + the disposition's exact
  emitted artifacts (M0.1.3.1 Diagnostic + Residual/Ambiguity/Incoherence per the §1.1
  table); overflow_members = first max_items+1 by canonical_sort_key, overflow_member_hash
  = artifact_hash else sha256(canonical_payload_bytes), canonical text `bound_overflow
  schema=<schema_id> path=<feature_path> max=<max_items> observed=<count>
  producer=<producer_id>` (`/`-joined path); overflow_source_regions/overflow_proof_roots
  = {} (§1.2 projection deferred to its first semantic consumer; "unresolved projections
  contribute {}"). Algorithm-internal (absent from §3.1 inventory) — no
  descriptor/registry edit. Tests: each BoundOverflowDisposition arm emits the table's
  primary+artifacts+code over a synthetic overflowing bound, diagnostic-text byte
  spot-check, overflow_members retention count. Read: §1.1 HandleBoundOverflow + emission
  table, §8.7 disposition artifacts. Test: cargo test -p ckc-schema overflow
- [ ] M0.1.3.4 source_region_closure engine + contains-closure + certificate construction.
  ckc-source dep ckc-schema (add); new `src/closure.rs` (lib.rs `pub mod closure;`):
  source_region_closure(&SourceGraph, seed:Set[RegionMember], &SchemaBoundManifest) ->
  OperationResult[SourceRegion] over §4.3 steps 1 (missing seed -> Residual(class=
  unsupported_construction, code=missing_region_member)), 2 (finite universe U), 3-4 own
  node/span/cell/anchor + contains-edge heading/section/paragraph/list/list_item/table/
  row/column/document ancestors, 6 (R⊆U), 7 (SourceRegion seed_members/closed_members
  bounds via handle_bound_overflow), 8 (closed_members sorted by canonical_sort_key);
  build RegionClosureCertificate (possible_member_count=|U|, iterations,
  added_member_batches, seed_members_digest=sha256(canonical_payload_bytes(seed)),
  residual_hashes). Tests over fixture_source_graph() (M0.1.2.4): clean containment
  closure from U1/U17 span seeds reaches exact closed_members + finite-U termination,
  missing-seed residual, certificate batches ∪ seed = closed_members. Read: §4.3 steps
  1-8, §4.2 contains edge, §1.1 overflow convention. Test: cargo test -p ckc-source closure
- [ ] M0.1.3.5 relational closure + table/caption/footnote/cross-reference residuals.
  closure.rs step-4 relational expansion — table_coordinate/header_of row+column header
  cells per table cell, caption_of table caption, footnote_of body+target,
  crossref_targets target, continuation targets + continuation-linked adjacent span — and
  step-5 earliest-by-source_order_key residual: unsupported_table_structure (table/
  caption/footnote/continuation failure), unsupported_cross_reference (crossref failure);
  step-6 out-of-U contribution. Tests over fixture_source_graph(): U3 table closure (cells
  -> row/column headers, caption U15 -> U3), U14 crossref -> U3, U16 footnote, U18 dangling
  crossref -> unsupported_cross_reference, U19 unformatted table ->
  unsupported_table_structure. Read: §4.3 steps 4-6, §4.2 edge kinds, §8.7 Residual, A.1
  U14-U19. Test: cargo test -p ckc-source closure
- [ ] M0.1.3.6 certificate admissibility + region fixture + gate (completes
  T-Region-Closure). ckc-source `src/check.rs`: validate_region_certificate(
  &RegionClosureCertificate, &SourceRegion, &SourceGraph) -> admissible iff
  seed_members_digest matches, replaying source_region_closure over the certified graph
  reproduces added_member_batches, and seed_members ∪ batches = closed_members. fixture.rs:
  fixture_source_regions() — seeds + closed SourceRegion + RegionClosureCertificate for
  the gate units (clean U1/U17 + table U3/caption U15/crossref U14/footnote U16 + residual
  U18/U19); exhaustive REG-U1..U27 demo enumeration rides M0.6.5. Gate
  `crates/ckc-source/tests/t_region_closure.rs` (T-Region-Closure): clean closures reach
  exact closed_members with admissible certificates, U18 -> unsupported_cross_reference,
  U19 -> unsupported_table_structure; perturbations (missing seed member, tampered
  added_member_batches breaking admissibility, dropped required header/caption/footnote/
  crossref target, unsorted closed_members) reject. Read: §4.3 certificate admissibility,
  §8.7 Residual, A.1 U14-U19, A.10 source-region expectations.
  Gate: cargo test -p ckc-source --test t_region_closure
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
