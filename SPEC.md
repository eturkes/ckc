# CKC — Staged Proof-Carrying Compiler for Japanese Clinical Text Conflict Review

Audience: autonomous coding, extraction, retrieval, formal-methods, compiler, verification, reporting, UI, evaluation, assurance agents.

CKC: ingests permission-tracked source editions, builds byte-stable `SourceGraph`, admits deterministic semantic generators + finite resources, derives exact finite licensed readings, projects to CKC Normal Form, checks M0 conflict + factual-inconsistency predicates, emits source-grounded review candidates + replayable certificates.

M0 authority: formalization-QA + clinical-text quality review only. Clinical/patient-care/regulated-CDS/SaMD/deployment/production-clinical-language authority requires `G-S3`.

Durable M0 assets: schemas, canonical bytes, source graphs + permissions, extraction manifests, mechanical observations, admitted generators, terminology resources, semantic policy sets, licensed reading sets, AIRCore records, CKC Normal Form objects, proof DAGs, M0 theorems, verifier witnesses, certificates, reports, replay manifests/checks. Code replaceable.

M0 semantic source of truth:

```text
SourceGraph
+ AcceptedGeneratorBase
+ TerminologyResourceSet
+ SemanticPolicySet
+ ProofDAG
+ CKCNormalForm
+ Certificate
```

NL rationales, prompts, retrieved passages, model outputs, UI explanations = annotations. Japanese/English glosses = deterministic views, `authority = view_only`.

Reusable set — `AcceptedGeneratorBase`, `TerminologyResourceSet`, `SemanticPolicySet` — = CKC optimization locus: describe corpus with fewest reusable components, each written/verified once then reused deterministically. M0 minimality descriptive only (§10); compression/reuse-payoff/MDL verdicts gated (`G-MDL`, `G-EMIN`); §13 frames optimization, IR design space, corpus scale, theoretical limits, open verification problem as research tracks not weakening the kernel.

Notation. `S Name(field:Type,field:Type?,fieldN:TypeN)` records; `E Name = a | b | c` enums/alias domains/tagged unions; `E Name = tag:PayloadType | tag(field:Type,fieldN:TypeN) | bare_tag` payload-bearing tagged unions, `tag:PayloadType?` marks tag payload optional; `T C1 | C2` then rows `v1 | v2` compact tables. Optional `?`. Containers `Set[T]`, `List[T]`, `Map[K,V]`. Type expr `A|B` only inside named `E` union or prose immediately naming that union schema. `Text<P>` uses statically declared `StringPolicy`; `Text<policy>` = dependent text field whose sibling `policy:StringPolicy` supplies normalization policy, dependency recorded in `SchemaRegistry`. Dense punctuation only in structural/enum declarations + compact tables. Prose, §6.2 grammar token spacing, canonical diagnostic-text templates, string-policy folds, CLI command vectors keep significant spaces.

## 0. Agent operating contract

Agents build CKC as sequence of small committed deliverables. Each session: load §11.4 reading slice for its build unit + earlier accepted artifacts, implement that one deliverable, run its acceptance gate, commit, end. §11.4 = authoritative loading rule; full-spec loading reserved for spec-maintenance. Next session relies on repo state + this spec.

Implementation invariants:

```text
one canonical representation per fact;
one canonical command per operation;
one schema-authority path;
one phase numbering scheme;
one M0 certificate vocabulary;
local change blast-radius;
byte-stable artifacts before convenience APIs;
deterministic internal checker before external backend breadth;
explicit residuals for unsupported constructions;
calibrated evidence before empirical thresholds.
```

Facts defined in a table referenced by row key; later citation treats that row as normative source.

## 1. Canonical data model

### 1.1 Schema authority

Executable schema authority for accepted artifacts = repo `SchemaRegistry`:

```text
S SchemaRegistry(registry_id:Id,registry_version:Id,spec_contract_hash:Hash,rust_type_manifest_hash:Hash,generated_json_schema_manifest_hash:Hash,canonicalization_policy_hash:Hash,schema_bound_manifest_hash:Hash,schema_entries:Set[SchemaEntry],string_policy_bindings:Set[StringPolicyBinding],source_support_aliases:Set[SourceSupportAlias])

S SchemaEntry(schema_id:Id,schema_version:Id,schema_role:SchemaRole,rust_type_hash:Hash,generated_json_schema_hash:Hash,tagged_union_alternatives_hash:Hash?)

E SchemaRole = semantic | source_only | schema_control | replay_control | environment_control | admission_control | evidence_discovery | view_only | proof_structure

S StringPolicyBinding(schema_id:Id,path:FeaturePath,policy:StringPolicy,dependent_policy_field:FeaturePath?)

S SourceSupportAlias(schema_id:Id,path:FeaturePath,alias_kind:SourceSupportAliasKind)

E SourceSupportAliasKind = singleton_region | region_set | inherited_subject | closed_region_members

S SchemaBoundManifest(manifest_id:Id,schema_collection_bounds:Set[SchemaCollectionBound])

S SchemaCollectionBound(schema_id:Id,path:FeaturePath,max_items:UInt,overflow_disposition:BoundOverflowDisposition)

E BoundOverflowDisposition = reject_with_diagnostic | emit_residual | emit_ambiguity | emit_incoherence
```

`canonicalization_policy_hash` stores accepted `UnicodePolicyManifest` envelope `artifact_hash` (§1.4). §1.5 canonical-bytes grammar fixed by `spec_contract_hash`.

`HandleBoundOverflow(bound, subject_hash, candidate_members, producer_id) -> invalid | residual | ambiguity | incoherence` = total dispatch for every `SchemaCollectionBound` overflow. `candidate_members` = canonical finite sequence considered for insertion into bounded collection. Checker retains first `bound.max_items + 1` members by `canonical_sort_key` to prove overflow; = `overflow_members`. `overflow_member_hash(x)` = `x.artifact_hash` for enveloped artifact, else `sha256(type_tagged_payload_bytes(x))` (§1.5). `overflow_source_regions` = canonical union of source-support projections of `subject_hash` + `overflow_members`; unresolved projections contribute `{}`. `overflow_proof_roots` = canonical union of proof roots from same inputs. Canonical diagnostic text = exactly `bound_overflow schema=<schema_id> path=<feature_path> max=<max_items> observed=<observed_count> producer=<producer_id>`, `<feature_path>` `/`-joined, `<observed_count> = |overflow_members|`.

```text
Bound overflow emission convention: every emitted Diagnostic uses subject_hash=subject_hash, source_regions=overflow_source_regions, text=canonical diagnostic text. reject_with_diagnostic emits only the diagnostic artifact and leaves the bounded-collection artifact absent.
T disposition | primary | exact non-diagnostic artifact | diagnostic code
reject_with_diagnostic | invalid | none | bound_overflow_reject
emit_residual | residual | Residual(class=unsupported_construction,subject_hash=subject_hash,source_regions=overflow_source_regions,diagnostic=canonical diagnostic text,proof_roots=overflow_proof_roots) | bound_overflow_residual
emit_ambiguity | ambiguity | Ambiguity(class=multiple_readings,alternatives={overflow_member_hash(m) | m in overflow_members},source_regions=overflow_source_regions,proof_roots=overflow_proof_roots) | bound_overflow_ambiguity
emit_incoherence | incoherence | Incoherence(class=incompatible_generator_outputs,subject_hashes={subject_hash} ∪ {overflow_member_hash(m) | m in overflow_members},source_regions=overflow_source_regions,proof_roots=overflow_proof_roots) | bound_overflow_incoherence
```

Schema-valid producer must call `HandleBoundOverflow` before accepting any collection exceeding declared `SchemaCollectionBound.max_items`. Local bound objects lacking `BoundOverflowDisposition` must define own dispatch at consuming algorithm; absent definition = registry error under `T-Registry-Referential-Integrity`.

Rust types = source for generated JSON Schema; this doc = design authority for creating/revising registry. M0 greenfield v0: accepted artifacts carry exactly v0 field set. Schema revision accepted when Rust type hash, generated JSON Schema hash, canonicalization policy hash, spec contract hash agree under `T-Schema-Equivalence`.

`T-Schema-Equivalence` (revision check): per `SchemaEntry`, canonicalize Rust type manifest, generated JSON Schema, union-alternative set, string-policy bindings, source-support aliases, collection bounds, canonicalization policy; accept iff all hashes equal `SchemaRegistry` fields and every changed field covered by new `schema_version`.

When prose notation, Rust types, generated JSON Schema differ during implementation, acceptance gate uses current `SchemaRegistry`, emits `schema_authority_mismatch`.

`T-Registry-Referential-Integrity` (total registry check):

```text
1 Build a symbol table from every schema_id, enum name, enum variant, tagged-union alternative,
  builtin name, proof rule, certificate class, gate, CLI operation, stage name, and section anchor
  declared by this specification and the generated SchemaRegistry.
2 Sort symbols by (symbol_kind, symbol_id, definition_anchor) and reject duplicate
  (symbol_kind,symbol_id) pairs with different definition anchors.
3 For every schema field type, enum reference, union alternative, FeaturePath root,
  StringPolicyBinding.path, SourceSupportAlias.path, builtin signature, ProofRule conclusion,
  Certificate subject, GateEvidenceRef.gate, and CLI output, require exactly one symbol-table target.
4 For every Set, List, and Map field in every accepted schema, require exactly one
  SchemaCollectionBound row unless the field is a scalar map whose maximum cardinality is fixed
  by an enum domain named in the field type.
5 Return ok when all references and collection bounds resolve; otherwise emit Diagnostic
  code=referential_integrity_error. A failing registry emits diagnostics only.
```

### 1.2 Artifact envelope

Every accepted payload stored in artifact envelope:

```text
S ArtifactEnvelope<T>(artifact_hash:Hash,schema_id:Id,schema_version:Id,schema_hash:Hash,canonicalization_policy_hash:Hash,producer_manifest_hash:Hash,replay_manifest_hash:Hash,accepted_effect_row:Set[Effect],payload:T)
```

`artifact_hash = sha256(canonical_payload_bytes(payload))`. Envelope `artifact_hash` field outside payload hash input. Store path derived from `artifact_hash`.

Hash-valued field convention: `*_hash`/`*_hashes` pointing at accepted artifact stores referenced envelope `artifact_hash`. `*_digest` stores `sha256(canonical_payload_bytes(named_payload))`, named payload defined beside field. Hash field whose input = raw source/executable bytes/external manifest/index fingerprint stores `sha256(exact_recorded_bytes)` + manifest supplying those bytes. Schema entry with hash-valued field and no applicable convention or field-specific computation = invalid under `T-Registry-Referential-Integrity`.

Every accepted envelope contains `accepted_effect_row`; accepted semantic envelopes use `{}`. Payload-level `accepted_effect_row`, when present, = envelope value.

Every semantically derived accepted payload exposes proof roots + source support via canonical field names below or one schema-defined alias. Alias proof-visible, = canonical source-support projection for that schema.

```text
proof_roots: Set[ProofId]
source_support: Set[RegionId]
```

Alias rows use these fixed field-name defaults unless row names narrower `FeaturePath`:

```text
source_region_id                  -> singleton_region
support_region_id                 -> singleton_region
source_regions                    -> region_set
exact_japanese_source_regions     -> region_set
source_region_ids                 -> region_set
subject_hash                      -> inherited_subject
closed_members                    -> closed_region_members
```

`singleton_region` projects one `RegionId`; `region_set` projects `Set[RegionId]`; `inherited_subject` resolves referenced artifact + uses its canonical support projection; `closed_region_members` resolves through owning `SourceRegion.region_id`.

Schema with neither canonical source-support field nor registered alias must have `SchemaEntry.schema_role` in `{source_only,schema_control,replay_control,environment_control,admission_control,evidence_discovery,view_only,proof_structure}`; else `T-Registry-Referential-Integrity` rejects. `proof_structure` marks kernel/proof payloads with proof-DAG-inherited source support (`ProofNode.support_digest`), not payload-projected.

### 1.3 Scalars

```text
Id          lowercase ASCII identifier matching [a-z][a-z0-9_:-]*
ProofId     Id naming a ProofNode
RegionId   Id naming a SourceRegion
FeaturePath List[Id] traversed over a schema-validated payload
Hash        "sha256:" followed by 64 lowercase hex digits
Bool        true | false
UInt        nonnegative integer, encoded as decimal string
Int         integer, encoded as optional "-" plus decimal digits
Rational    exact reduced rational, encoded as Rational object
Text<P>     UTF-8 string normalized by StringPolicy P
Set[T]      finite unordered collection encoded in canonical order
List[T]     finite ordered collection
Map[K,V]    finite map encoded by canonical key order
```

Accepted payloads use JSON objects, arrays, strings, booleans. Integers/rationals = strings or typed objects. Optional fields = omission. Union alternatives = tagged objects.

```text
S Rational(num:Int,den:UInt)
```

Rational invariants:

```text
den > 0;
gcd(abs(num), den) = 1;
zero is {num:"0", den:"1"};
decimal source forms are converted exactly by base-10 place value;
percent values are converted exactly by denominator multiplication by 100;
quantities, thresholds, units, metrics, and scores that affect accepted semantics use Rational.
```

JSON numeric tokens reserved for nonsemantic adapter-local files.

### 1.4 String policies

Every `Text` field declares exactly one `StringPolicyBinding` in `SchemaRegistry`.

```text
E StringPolicy = raw_source | source_nfkc | semantic_ja | semantic_en | identifier_ascii | template_literal | diagnostic_text | view_text
```

Policy algorithms:

```text
raw_source:
  store the UTF-8 scalar sequence emitted by the extractor or adapter exactly;
  preserve code point sequence, spacing, punctuation, and line breaks;
  record source byte hash and decoder manifest when available.

source_nfkc:
  apply Unicode NFKC using UnicodePolicyManifest tables;
  preserve resulting spacing and punctuation.

semantic_ja:
  apply Unicode NFKC;
  map whitespace code points HT, LF, VT, FF, CR, U+00A0, U+1680,
    U+2000..U+200A, U+2028, U+2029, U+202F, U+205F, U+3000 to U+0020;
  map punctuation:
    U+3001 and U+FF0C to ",";
    U+3002 and U+FF0E to ".";
    U+FF1A to ":";
    U+FF1B to ";";
    U+FF08 to "(";
    U+FF09 to ")";
    U+3010 and U+FF3B to "[";
    U+3011 and U+FF3D to "]";
    U+FF0D, U+2010, U+2011, U+2012, U+2013, U+2014, U+2015, U+2212 to "-";
    U+FF1C to "<";
    U+FF1E to ">";
    U+2264 and U+2266 to "<=";
    U+2265 and U+2267 to ">=";
  collapse each maximal run of U+0020 to one U+0020;
  trim leading and trailing U+0020.

semantic_en:
  apply Unicode NFKC;
  apply the same whitespace and punctuation fold as semantic_ja;
  lowercase ASCII letters only inside controlled-vocabulary identifier fields.

identifier_ascii:
  require nonempty ASCII bytes matching [a-z0-9_:./-]+;
  store bytes exactly.

template_literal:
  apply Unicode NFKC;
  preserve template-significant punctuation and slot markers;
  validate against the gloss-template grammar in §7.5.

diagnostic_text:
  apply Unicode NFKC;
  apply semantic_ja whitespace folding;
  preserve human-facing punctuation.

view_text:
  apply Unicode NFKC;
  preserve display-intended wording;
  record the renderer that produced the view when the text is generated.
```

```text
S UnicodePolicyManifest(manifest_id:Id,unicode_version:Text<identifier_ascii>,normalization_table_hash:Hash,punctuation_table_hash:Hash,policy_test_hash:Hash)
```

`T-Unicode-Idempotency` checks every string policy is idempotent + byte-stable.

### 1.5 Canonical JSON bytes

Canonical payload bytes serializer:

```text
object:
  "{" + members sorted by UTF-8 bytes of field name + "}";
  each member is string(field_name) + ":" + value;
  comma separates members;
  omitted optional fields are absent.

array:
  "[" + values in semantic order + "]";
  comma separates values.

set:
  encode as array sorted by canonical_sort_key(element).

map:
  encode as object when K is identifier_ascii;
  encode as sorted array of {key,value} pairs for other key types.

string:
  emit U+0022;
  emit code points as UTF-8 except U+0022, U+005C, and U+0000..U+001F;
  escape U+0022 as \";
  escape U+005C as \\;
  escape control characters as \u00xx using lowercase hex;
  emit U+0022.

bool:
  true or false.

tagged union:
  encode as an object with exactly two members, "tag" and "value";
  "tag" is the constructor tag string;
  "value" is the payload object, array, scalar, or {} for a bare tag or an absent optional payload;
  constructor tags are unique within the union schema.
```

Canonicalization type-guided. Schema validation rejects: duplicate object fields, unknown required fields, JSON nulls, JSON numeric tokens in accepted semantic payloads, duplicate map keys, duplicate union tags. `canonical_payload_bytes` = deterministic injection over schema-valid payloads of one declared type (field names, union tags, collection encodings, string policies, exact integer/rational encodings, optional-field omission all schema-fixed). Not injective across types: records carry no type tag, so field-isomorphic schemas (`PredAtom`/`NegPredAtom`, `MatchClassVar`/`VarTerm`, `RegionOfTerm`/`BoundAddress`/`TermValue`, `TableRowReading`/`NFDecisionRow`) share bytes for equal field values. Cross-type identity always via `canonical_sort_key`; `declared_type_id` prefix discriminates.

`canonical_sort_key(x) = (declared_type_id, canonical_payload_bytes(x))` inline; `declared_type_id` = `SchemaRegistry` `symbol_id` of declared type. Accepted object references sort by referenced `artifact_hash`, then `schema_id`, then declared reference field name. `type_tagged_payload_bytes(x)` = canonical encoding of `(declared_type_id, canonical_payload_bytes(x))`; any inline (non-enveloped) value hashed for cross-type identity or dedup uses `sha256(type_tagged_payload_bytes(x))` not `sha256(canonical_payload_bytes(x))`, so field-isomorphic payloads of different types never collide.

Tie-break priority order (deterministic operations):

```text
1 declared semantic key;
2 source_order_key;
3 canonical payload bytes;
4 artifact_hash;
5 schema_id.
```

`source_order_key`:

```text
(source_edition_hash, page_or_zero, reading_order, bbox_top, bbox_left,
 bbox_bottom, bbox_right, node_id, char_start, char_end, anchor_id)
```

Missing source-order fields use the type's canonical minimum and retain an extraction diagnostic.

### 1.6 Replay manifests

```text
S ReplayManifest(manifest_id:Id,command:List[Text<identifier_ascii>],input_hashes:Set[Hash],schema_registry_hash:Hash,toolchain_manifest_hash:Hash,environment_profile_hash:Hash,expected_output_hashes:Set[Hash],accepted_effect_row:Set[Effect])

S ReplayIdentityCheck(replay_manifest_hash:Hash,expected_output_hashes:Set[Hash],actual_output_hashes:Set[Hash],outcome:ReplayIdentityOutcome,diagnostic_hashes:Set[Hash])

S ProducerManifest(manifest_id:Id,operation_id:Id,command:List[Text<identifier_ascii>],input_hashes:Set[Hash],implementation_unit_hashes:Set[Hash],schema_registry_hash:Hash,toolchain_manifest_hash:Hash,accepted_effect_row:Set[Effect])

S ToolchainManifest(manifest_id:Id,tool_records:Set[ToolRecord],build_input_hashes:Set[Hash])

S ToolRecord(tool_id:Id,tool_family:Id,version:Text<identifier_ascii>,executable_hash:Hash?,config_hash:Hash?)

S EnvironmentProfile(profile_id:Id,os_family:Id,architecture:Id,locale_policy:Text<identifier_ascii>,timezone_policy:Text<identifier_ascii>,network_policy:Effect,clock_policy:Effect,environment_variable_hashes:Set[Hash])

S ValidationManifest(manifest_id:Id,validator_id:Id,validated_artifact_hashes:Set[Hash],check_ids:Set[Id],diagnostic_hashes:Set[Hash],replay_manifest_hash:Hash)
```

Replay identity compares a well-founded issuance stratum of: canonical payload hashes, envelope fields, proof roots, certificate hashes, report hashes, replay-check hashes. Wall-clock timestamps = evidence metadata, `Effect = Clock`; accepted replay uses logical time. `ReplayManifest.expected_output_hashes` names one closed prior stratum, excluding: the manifest payload; any artifact whose payload/envelope `replay_manifest_hash` = that manifest hash; the enclosing `ReplayIdentityCheck` payload; every `Certificate` whose `replay_identity_hashes` contains that check. `replay_manifest_hash` names lower-stratum producer manifests; audit manifests listing output hashes are referenced by the later `ReplayIdentityCheck`; a later stratum may certify the check. Demo: `RM-PRODUCER-BASE` = producer manifest for emitted artifacts; `RM-DEMO-CORE` audits outputs through `ReviewReport` and all certificates except `report_replay`; `RIC-DEMO-CORE` checks that set; `CERT-report_replay` references the report and `RIC-DEMO-CORE`. A.10 enumerates the outer fixture inventory, replay authority for `ckc demo m0`.

`ReplayIdentity` is total:

```text
1 Load every expected output hash from ReplayManifest.expected_output_hashes and verify that the set
  is a closed prior issuance stratum under the boundary rule above.
2 Re-run the command over the exact input_hashes, SchemaRegistry, ToolchainManifest, and
  EnvironmentProfile named by the manifest.
3 Canonicalize actual outputs from the same stratum, including envelopes, proof roots, certificates,
  reports, and any replay-check payloads that belong to earlier strata.
4 If the command cannot be replayed because a required toolchain or permissioned source byte is
  absent, emit ReplayIdentityCheck(outcome=replay_identity_unsupported) with diagnostics.
5 If expected and actual stratum hash sets are equal, emit replay_identity_pass.
6 Otherwise emit replay_identity_mismatch and include the symmetric-difference diagnostics.
```

### 1.7 Total-function convention

Every operation consuming accepted bytes = total deterministic function over schema-valid canonical payloads. Returns exactly one named outcome:

```text
success(value_hashes)
residual(residual_hashes, diagnostic_hashes)
ambiguity(ambiguity_hashes, diagnostic_hashes)
incoherence(incoherence_hashes, diagnostic_hashes)
unsupported(diagnostic_hashes)
invalid(diagnostic_hashes)
```

`OperationResult[T]` = typed implementation generic:

```text
E OperationResult[T] = success:List[T] | residual:Set[Hash] | ambiguity:Set[Hash] | incoherence:Set[Hash] | unsupported:Set[Hash] | invalid:Set[Hash]
```

`success` carries ≥1 canonical values or accepted payload hashes of type `T`; other variants carry hashes above. Persisted `Outcome` uses `ok` for `success`, same names otherwise. Any body that can call `HandleBoundOverflow` must return `OperationResult[T]` or map the overflow status to a declared variant before returning.

Primary-status selection order from multiple emitted facts:

```text
invalid > incoherence > unsupported > ambiguity > residual > success
```

All emitted residuals/ambiguities/incoherences/diagnostics are artifacts with canonical bytes. Algorithms may emit several non-success artifacts but pick primary status by the fixed order and sort emitted hashes by `canonical_sort_key`. Schema error = `invalid`; schema-valid but outside-M0 = `unsupported` or typed residual; semantic collision in accepted inputs = `incoherence`; multiple admissible interpretations = `ambiguity`; absent required license/policy/permission/evidence object/counterexample suite = typed `Residual`.

Every accepted algorithm depends only on canonical bytes, sorted enumerations, declared inputs, recorded manifests; map iteration order, thread interleaving, wall-clock time, random seeds, locale, platform float behavior, external service state enter only as proposal/gated evidence metadata. Accepted semantic replay excludes those metadata dependencies.

## 2. Canonical vocabulary

Canonical definition location for shared CKC vocabulary. Schema-local enums defined once at their consuming site.

```text
E Effect = Inference | Extract | Verify | Compile | IO | Network | Clock

E Authority = source_authority | mechanical_authority | admitted_authority | compiler_authority | verifier_authority | evidence_discovery_only | view_only

E SourceClass = guideline | package_insert | other

E SourceNodeKind = document | section | heading | paragraph | sentence | span | token | list | list_item | table | row | column | cell | caption | footnote | clinical_question | recommendation | pico_field | evidence_table | etd_field | cross_reference_anchor

E SourceEdgeKind = contains | precedes | table_coordinate | header_of | caption_of | footnote_of | continuation | crossref_targets

E GeneratorProfile = obs_pattern | term_resource | sem_rule | bridge | residual | gloss

E AirType = air.term | air.condition | air.action | air.temporal | air.cue | air.quantity | air.norm | air.factual

E BindingStatus = exact | synonym | unmapped | ambiguous

E TerminologyRelationKind = exact | synonym | unit_equivalent | section_equivalent | contraindication_target | mutually_exclusive | action_kind_equivalent

E Direction = for | against | contraindicate | require | permit | avoid

E ClaimTier = S0 | S1 | S2 | S3

E M0CertificateClass = source_graph | mech_observed | admitted_base | closed_nf | finite_checked | report_replay

E VerifierResult = valid | invalid | unsupported

E ReplayIdentityOutcome = replay_identity_pass | replay_identity_mismatch | replay_identity_unsupported

E Outcome = ok | residual | ambiguity | incoherence | unsupported | invalid

E ReviewClassification = candidate | residual | ambiguity | incoherence | replay_failure
```

M0 conflict kinds:

```text
E ConflictKind = contraindication_vs_recommendation | recommendation_for_vs_against | strict_consequents_jointly_contradictory | numeric_threshold_empty_intersection | terminology_mapping_incoherence
```

M0 factual-inconsistency kinds:

```text
E FactualInconsistencyKind = table_value_disagreement | package_insert_vs_guideline_unresolved_conflict | gloss_semantic_drift | source_metadata_disagreement | proof_or_certificate_replay_failure
```

M0 residual, ambiguity, and incoherence classes:

```text
E ResidualClass = no_license | unsupported_construction | unsupported_cross_reference | unsupported_table_structure | missing_terminology | missing_policy | missing_counterexample_suite | permission_limited | extraction_uncertain | verifier_unsupported | deferred_gate_required

E AmbiguityClass = multiple_readings | multiple_terms

E IncoherenceClass = functional_key_collision | mutually_exclusive_term_mapping | incompatible_generator_outputs
```

Gate names are canonical:

```text
E Gate = G-EXTRACTOR-ADAPTER | G-RET-PARITY | G-PORTFOLIO | G-AIR-FULL | G-REBIND | G-EMIN | G-MDL | G-RUNTIME-ORACLE | G-SELF-IMPROVE | G-PROB | G-WORLD-MODEL | G-LIVE-PATIENT | G-S3
```

### 2.1 Shared vocabulary consumers

Every shared enum variant reaches M0 via an emitting/consuming site. Variants outside this table are local to their defining schema section, carrying their consumer beside that schema.

```text
T Vocabulary | M0 consumer or emitter
Effect | Proposal/replay/admission records carry all declared effects; accepted semantic payloads discharge to {} per §6.4; certificates check discharge in §9.2.
Authority | Source, mechanical, admitted, compiler, verifier, evidence-discovery, view artifacts carry matching authority field at defining schema.
SourceClass | Source ingestion emits; §8.6 consumes guideline, package_insert; metadata/report paths preserve other.
SourceNodeKind and SourceEdgeKind | SourceGraph construction emits; §4.3 closure and §7.2 matching consume for regions, table headers, captions, footnotes, continuations, cross-references.
GeneratorProfile | §7.1 dispatches obs_pattern, term_resource, sem_rule, bridge, residual, gloss generators at declared stages.
AirType | §6.3 AIR keys type the eight reading kinds; §7.3 finite-set identity consumes every demanded key.
BindingStatus | §5.2 consumes exact, synonym, ambiguous, unmapped in terminology closure, ambiguity, residual.
TerminologyRelationKind | §5.2 consumes exact, synonym, unit_equivalent, section_equivalent, action_kind_equivalent, mutually_exclusive; §8.2 consumes contraindication_target via ActionTargetRelation rows.
Direction | §8.3 direction groups, §8.5 conflict predicates, §8.6 package-insert factual predicates, §7.5 gloss rendering consume all six.
ClaimTier | §3.4 computes S0-S3; §9.1, §9.2 check theorem and certificate claim records.
M0CertificateClass | §9.2 defines one certificate verification obligation per class.
VerifierResult | §9.1 emits valid, invalid, unsupported.
ReplayIdentityOutcome | §1.6 replay emits all; §8.6 consumes replay_identity_mismatch.
Outcome | Closure, AIRCore, admission, generator evaluation, diagnostics consume ok plus every non-success status.
ReviewClassification | §8.7 and §9.3 report construction consume all.
ConflictKind and FactualInconsistencyKind | §8.5 and §8.6 define the unique theorem builder per kind.
ResidualClass | §7.3 emits no_license; §6.2 and §8.1 emit unsupported_construction; §4.3 emits unsupported_cross_reference and unsupported_table_structure; §5.2 emits missing_terminology; §5.3, §8.2, §8.3, §8.6 emit missing_policy; §6.4 emits missing_counterexample_suite; §4.1 and §9.3 emit permission_limited; §4.2 emits extraction_uncertain; §9.1 emits verifier_unsupported; §3.3 and §12 emit deferred_gate_required.
AmbiguityClass | §7.3 emits multiple_readings; §5.2 emits multiple_terms.
IncoherenceClass | §5.2 emits functional_key_collision, mutually_exclusive_term_mapping, endpoint-form incompatible_generator_outputs; §5.3 and §8.2 emit incompatible_generator_outputs.
Gate | §3.3 defines the trigger and evidence object per gate; §12 consumes every gate through GateEvidenceRef.
```

## 3. M0 scope, stages, gates, and claim tiers

### 3.1 M0 pipeline

M0 = deterministic review candidates for the five M0 conflict kinds and five M0 factual-inconsistency kinds (§2).

```text
SourceEdition
-> SourcePermissionRecord
-> ExtractionManifest
-> SourceGraph
-> MechObsPayload
-> PatternObs
-> Match
-> MatchClass
-> admitted CKC-GEN semantic closure
-> LicensedReadingSet
-> AIRCoreRecord
-> CKCNormalForm
-> ConflictTheorem | FactualInconsistencyTheorem | Residual | Ambiguity | Incoherence
-> VerifierWitness
-> Certificate
-> ReviewReport
-> ReplayIdentityCheck
```

Inclusion criterion for required artifact payloads: every payload emitted by an M0 canonical command, consumed by a later M0 command, referenced from a certificate, or controlling schema/replay/admission/environment/validation. Evidence objects deferred behind §3.3 gates: see §12.

```text
SchemaRegistry
SchemaBoundManifest
UnicodePolicyManifest
ToolchainManifest
ToolRecord
EnvironmentProfile
ProducerManifest
ValidationManifest
SourceEdition
SourcePermissionRecord
CorpusDocument
ExtractionManifest
SourceGraph
SourceRegion
SourceSpan
SourceAnchor
RegionClosureCertificate
AnalyzerManifest
MechanicalLexicon
MechObsPayload
PatternObs
Match
MatchClass
ClassMember
CKCGen
GeneratorGrammarArtifact
FiniteFixtureManifest
FrozenConstant
ParsedQuantity
DiagnosticTag
AcceptedGeneratorBase
TerminologyResourceSet
TerminologyClosure
SemanticPolicySet
ResolutionTheorem
ProposalProvenanceManifest
ProposalRecord
RetrievalProposalTrace
AdmissionContext
ReviewerRecord
AdmissionRecord
EffectDischargeRecord
CounterexampleSuite
MaterializedConsequenceManifest
ClosureInput
ClosureOutput
ClosureBoundCertificate
License
LicensedReadingSet
AIRCoreRecord
CKCNormalForm
WitnessContext
GlossTemplate
GlossView
ConflictTheorem
FactualInconsistencyTheorem
Residual
Ambiguity
Incoherence
Diagnostic
VerifierWitness
SymbolSourceMap
ConstraintCoreWitness
RepairSetSearchTrace
ProofNode
ProofDAG
Certificate
ClaimRecord
ReportQuestionTemplate
ReportTraceIndex
ClaimTierSummary
WordingGateRecord
ReviewReport
ReplayManifest
ReplayIdentityCheck
```

M0 verification uses the internal kernel finite checker. External solver, proof-assistant, ontology, retrieval-quality, decision-workflow, probabilistic, world-model, live-patient, and regulated-clinical claims are gated by §3.3.

### 3.2 Stages

```text
Stage -40: schema, toolchain, replay-control, parser, proposal, admission, and fixture-control artifacts.
Stage -30: source editions, permissions, corpus documents, extraction manifests, SourceGraph, spans, anchors, and regions.
Stage -20: mechanical observations.
Stage -10: admitted pattern observations over mechanical observations.
Stage   0: matches, match classes, class members.
Stage  10: terminology-resource fragments and admitted terminology resource sets.
Stage  20: terminology closure.
Stage  30: term, cue, quantity, temporal, condition, and action licenses.
Stage  40: norm licenses, factual licenses, and resolution theorems.
Stage  50: licensed reading sets, AIRCore, and CKC Normal Form; kernel-produced by split AIR and NF builders.
Stage  60: conflict and factual-inconsistency theorem construction.
Stage  70: deterministic gloss views.
Stage  80: residuals, ambiguities, incoherences, and coverage diagnostics.
Stage  90: finite-checker witnesses, certificates, reports, replay identity, and demo orchestration.
```

Generator stages strictly stratified. A generator reads license, terminology-closure, resolution-theorem premises only from lower stages. Kernel stages 50, 90 = fixed compiler functions.

Stage producers fixed:

```text
T Stage | Producing operation | Generator profiles or builders | Emitted accepted artifacts
-40 | CheckSchemaRegistry | schema builders | SchemaRegistry, SchemaBoundManifest, UnicodePolicyManifest, schema diagnostics
-40 | ValidateRuntimeManifests | replay and environment-control builders | ToolchainManifest, EnvironmentProfile, runtime manifest diagnostics
-40 | LoadFiniteFixtureManifest | fixture-control builders | FiniteFixtureManifest, fixture manifest diagnostics
-40 | ParseCKCGen | parser builders | CKCGen, GeneratorGrammarArtifact, parse diagnostics
-40 | DischargeProposal | admission builders | ProposalProvenanceManifest, ProposalRecord, RetrievalProposalTrace, AdmissionContext, ReviewerRecord, AdmissionRecord, EffectDischargeRecord, CounterexampleSuite, MaterializedConsequenceManifest, admitted CKCGen, admitted TerminologyResourceSet, admitted SemanticPolicySet, admitted GlossTemplate, ReportQuestionTemplate, AcceptedGeneratorBase
-30 | IngestSourceEdition | source builders | SourceEdition, SourcePermissionRecord, CorpusDocument, ExtractionManifest
-30 | BuildSourceGraph | source builders | SourceGraph, SourceSpan, SourceAnchor, source diagnostics
-30 | source_region_closure | source builders | SourceRegion, RegionClosureCertificate
-20 | ObserveMech | analyzer builders | MechObsPayload, AnalyzerManifest, MechanicalLexicon
-10 | MaterializeGenerators | obs_pattern | PatternObs
0 | BuildMatches and BuildMatchClasses | class builders | Match, MatchClass, ClassMember
10 | MaterializeGenerators | term_resource | TerminologyResourceSet fragments
20 | BuildTerminologyClosure [internal:CloseM0] | terminology builders | TerminologyClosure
30 | MaterializeGenerators | sem_rule | term, cue, quantity, temporal, condition, and action License artifacts
40 | MaterializeGenerators | sem_rule | norm and factual License artifacts, ResolutionTheorem artifacts
50 | BuildAIRCore | kernel builder | LicensedReadingSet, AIRCoreRecord
50 | BuildNormalForm | kernel builder | CKCNormalForm
60 | BuildM0Theorems | conflict and factual-inconsistency builders, bridge diagnostics | WitnessContext, ConflictTheorem, FactualInconsistencyTheorem, bridge diagnostics
70 | BuildGloss | gloss helpers and renderer tables | GlossTemplate, GlossView
80 | BuildDiagnostics [internal:CloseM0] | residual generators and fixed coverage builders | Residual, Ambiguity, Incoherence, Diagnostic, RepairSetSearchTrace
90 | kernel_finite_checker | kernel checker | VerifierWitness, SymbolSourceMap, ConstraintCoreWitness, verifier residuals
90 | IssueCertificate | certificate builder | Certificate, certificate diagnostics
90 | BuildReviewReport | report builder | ClaimRecord, ReportTraceIndex, ClaimTierSummary, WordingGateRecord, ReviewReport, permission residuals
90 | ReplayIdentity | replay builder | ReplayManifest, ReplayIdentityCheck
90 | CloseM0 | closure orchestrator | ClosureInput, ClosureOutput, ClosureBoundCertificate, ProofNode, ProofDAG
90 | DemoM0 | fixture orchestrator | Appendix A accepted artifact inventory
```

`BuildTerminologyClosure`, `BuildDiagnostics` = CloseM0-internal suboperations: canonical command `ckc close`, plus direct test-harness entry points in owning build units. `SemanticPolicySet` accepted only via `DischargeProposal`; CloseM0 reads it through `ClosureInput.semantic_policy_set_hash`, produces no new `SemanticPolicySet`. `CloseM0`, `DemoM0` = stage-90 orchestration; semantic artifacts in their output retain producing suboperation's numeric stage. A stage reads only earlier-stage artifacts except fixed stage-50/90 kernel builders, which read completed finite snapshots by hash. Same-stage recursion invalid. Admission/proposal trace artifacts (`ProposalProvenanceManifest`, `ProposalRecord`, `RetrievalProposalTrace`, `CounterexampleSuite`, `MaterializedConsequenceManifest`, `AdmissionRecord`, `EffectDischargeRecord`) = accepted replay-control artifacts emitted by `DischargeProposal` before the consuming stage; included in replay output sets but not recursive semantic stage inputs unless an accepted artifact references their hash.

Cross-cutting control emissions fixed. `ProducerManifest`: emitted by each canonical command wrapper, same stage as wrapped op. `ValidationManifest`: emitted by each acceptance-gate runner, same stage as validated op; demo emits the schema, runtime-manifest, fixture-manifest, policy-admission, closure, verifier, report, replay validation manifests (Appendix A.10). `ToolchainManifest`, `EnvironmentProfile` = authored inputs accepted by `ValidateRuntimeManifests`; `ToolRecord` rows = accepted environment-control rows within `ToolchainManifest`. `FiniteFixtureManifest` = authored input accepted by `LoadFiniteFixtureManifest`; its `FrozenConstant`, `ParsedQuantity`, `DiagnosticTag` rows = accepted fixture-control rows within it. Semantic-policy duplicate-key quarantine validation runs when `DischargeProposal` accepts a `SemanticPolicySet` candidate; policy-admission `ValidationManifest.diagnostic_hashes` records quarantined keys and diagnostics.

Every valid §3.1 payload names ≥1 producing/accepting operation in this §3.2 stage-producer table or the cross-cutting control-emission rule. Missing mapping emits `producer_mapping_error` under `T-Registry-Referential-Integrity`.

### 3.3 Gates

Canonical gate definition. Each gated capability in M0 = trigger + named evidence object + claim boundary.

```text
T Gate | Trigger | Required evidence object | Claims enabled
G-EXTRACTOR-ADAPTER | Source extraction adapter changes SourceGraph-affecting output. | ExtractorAdapterRecord | extractor soundness for declared source profile
G-RET-PARITY | Retrieval quality, dense retrieval, late interaction, reranking, graph retrieval, or non-oracle retrieval claimed. | RetrievalParityReport | retrieval-quality claims
G-PORTFOLIO | Independent backend agreement claimed. | VerifierPortfolioReport | portfolio verification claims
G-AIR-FULL | Non-identity AIR abstraction domain accepted. | AIRDomainRecord | abstract-interpretation claims beyond finite-set identity
G-REBIND | Proof transport across source editions claimed. | RebindingEvidence | rebinding claims
G-EMIN | Coverage, compression, generator-reuse, or scientific efficacy claims made. | BenchmarkRelease and EMinReport | S2 research measurements
G-MDL | MDL, Pareto, or compression-payoff optimization claims made. | MDLEvidence | calibrated compression/payoff claims
G-RUNTIME-ORACLE | Layered IR pipeline, intermediate-IR layering, between-layer model call, or runtime (non-development-time) model pipeline benchmarked/claimed to improve natural-language-to-target fidelity. | RuntimeOracleReport | layered-pipeline / runtime-oracle fidelity claims
G-SELF-IMPROVE | Automated improvement modifies accepted generators, resources, passes, or policies. | SelfImprovementEvidence | proof-carrying self-improvement claims
G-PROB | Probabilistic facts, risks, weights, stochastic transitions, or rewards affect accepted outputs. | ProbabilisticProfileRecord | probabilistic claims
G-WORLD-MODEL | World-model, latent-state, image-derived, or multimodal observations affect accepted outputs. | WorldModelProfileRecord | world-model or multimodal claims
G-LIVE-PATIENT | Live or deidentified patient data enters CKC. | GovernedPatientDataProfile | patient-data handling claims
G-S3 | Clinical, regulatory, patient-care, CDS, SaMD, or deployment authority claimed. | S3AssuranceEvidence | clinical/regulatory authority
```

Gate diagnostics preserve all accepted S0/S1 artifacts whose replay and proof checks stay valid. §3.3 trigger present + required evidence object absent → gate checker emits `Residual(class=deferred_gate_required)` with a `GateEvidenceRef`-shaped diagnostic stub.

### 3.4 Claim tiers

```text
S0:
  Artifact bytes replay and proof-check from frozen source graphs, admitted generators,
  admitted resources, manifests, canonical encodings, and finite-checker witnesses.

S1:
  The artifact expresses the corpus theory endorsed by recorded admission decisions.

S2:
  The artifact or system output has gated research evidence for a stated population.

S3:
  The deployment profile has gated clinical/regulatory assurance evidence.
```

Allowed M0 wording: `candidate`, `review candidate`, `formalization-QA`, `text-quality analysis`, `source-grounded`, `proof-carrying`, `replayable`, `licensed by admitted generators`, `requires human adjudication`.

Each report phrase using this vocabulary emitted from an admitted template literal part (§9.3). Wording gate checks template literal IDs + renderer provenance, not free text. S2 wording requires `G-EMIN`, `G-MDL`, or `G-RUNTIME-ORACLE` evidence as applicable. S3 wording requires `G-S3` evidence.

```text
S ClaimRecord(claim_id:Id,subject_hash:Hash,tier:ClaimTier,evidence_hashes:Set[Hash],falsification_criterion:Text<diagnostic_text>,wording_gate_result:Outcome)
```

`T-Claim-Tiering` = strongest tier supported by present evidence:

```text
1 S0 holds when replay identity passes and all proof roots check.
2 S1 holds when S0 holds and every admitted generator, terminology resource,
  semantic policy, and resolution theorem used by the artifact has
  AdmissionRecord.decision = accept.
3 S2 holds when S1 holds and the relevant §3.3 research gate evidence object is present and valid.
4 S3 holds when S2 holds and valid G-S3 evidence is present.
5 Missing evidence stops at the highest satisfied earlier tier.
```

### 3.5 Source-corpus method disposition

Canonical map: each row = one corpus unit, one slot. `m=m0_core`=core realization by cited sections; `d=deferred_contract`=only via named §12 gate; `x=scope_excluded`=clinical/regulatory/deployment/live-patient/broad-ontology authority outside M0. Gate aliases: `Ex=ExtractorAdapterRecord@G-EXTRACTOR-ADAPTER`, `Ret=RetrievalParityReport@G-RET-PARITY`, `Port=VerifierPortfolioReport@G-PORTFOLIO`, `Air=AIRDomainRecord@G-AIR-FULL`, `Reb=RebindingEvidence@G-REBIND`, `Bench=BenchmarkRelease@G-EMIN`, `Emin=BenchmarkRelease|EMinReport@G-EMIN`, `EM=EMinReport@G-EMIN`, `MDL=MDLEvidence@G-MDL`, `SI=SelfImprovementEvidence@G-SELF-IMPROVE`, `Prob=ProbabilisticProfileRecord@G-PROB`, `WM=WorldModelProfileRecord@G-WORLD-MODEL`, `Pat=GovernedPatientDataProfile@G-LIVE-PATIENT`, `S3=S3AssuranceEvidence@G-S3`. Shorthands: `SrcGraph=SourceGraph`, `TermResourceSet=TerminologyResourceSet`, `RecMetadata=RecommendationMetadata`, `canon.=canonical`, `det.=deterministic`.

```text
E DispositionSlot = m0_core | deferred_contract | scope_excluded
```

```text
T Unit|S|Disposition
C1.1 CQL/ELM|m|CQL library/parameter/context/retrieve model,ANTLR4 surface,ELM XML/JSON AST,idempotency by canonical ELM tree-diff,null/3-valued/interval/terminology ops->§6.1,§6.2,§8.1,§8.3;executable CQL equivalence/Z3 encodings->Port(cql|smt);clinical execution->S3.
C1.2 FHIR Clinical Reasoning|m|Library,ActivityDefinition,PlanDefinition,Measure,relatedAction,selection/required/cardinality behavior->ActionReading,NFNorm,source support,report links;FHIR $apply/RequestOrchestration runtime,underspecified concurrency->Port(fhirpath|model_checker)/S3.
C1.3 CPG-on-FHIR/CQF packaging|m|L1 narrative→L4 executable enablement,CPGRecommendation/Strategy/Pathway/CaseFeature metadata,EBMonFHIR provenance->CKC stages/claim tiers/RecMetadata;L3/L4 executable package conformance->Bench/Port/S3.
C1.4 FHIRPath/StructureMap|m|FHIRPath collection navigation,exists/all/repeat,type ops,StructureMap unidirectional rule groups->FeaturePath,ClassPred,FieldConstraint,TemplateBinding;FHIR instance transformation/equivalence->Port(fhirpath);S3 clinical execution.
C1.5 FHIR Terminology Services|m|CodeSystem,ValueSet,ConceptMap,$lookup/$validate-code/$expand/$subsumes/$translate/$closure reduce in M0 to finite TermResourceSet,binding status,functional keys,closure;live service parity/version drift->Ret/Reb.
C1.6 CDS Hooks|x|Hook context,cards,suggestions,override links,EHR workflow invocation=CDS deployment behavior->S3;M0 emits static review reports only.
C1.7 SMART App Launch/Backend Services|x|OAuth2/OIDC,launch context,scopes,backend authorization,production data access=deployment/patient-data authority->Pat/S3.
C1.8 openEHR ADL/AQL/GDL2|m|ADL archetype constraints,AQL query shape,GDL2 rule form inform finite domains,premises,source-grounded action/condition licenses;openEHR runtime/AQL patient query->Pat.
C1.9 DMN/FEEL decision tables|m|Decision-table rows,FEEL scalar guards,null-sensitive residuals,hit-policy-style output exclusivity->TableReading,NFDecisionTable,table_outputs_compatible;engine conformance->Port.
C1.10 BPMN/BPM+ Health/ePath|m|Process ordering,branching,parallelism,pathway decomposition,ePath data-element thinking->stage stratification,temporal order preservation,workflow residuals;process execution/care-pathway deployment->S3.
C2.1 Minds/GRADE|m|Minds Manual/GRADE CQ/PICO/SR/EtD,recommendation direction/strength/certainty,COI/AGREE trace,Japanese modality phrases,no XML schema->SrcGraph recommendation nodes,RecMetadata,Direction,det. glosses.
C2.2 FHIR JP Core|d|Japan-realm FHIR profile validation,CodeableConcept binding,patient context enter only via Pat;profile terminology identifiers may be admitted as finite resources;Clinical Reasoning bindings non-M0.
C2.3 SS-MIX2|d|HL7-v2-like standardized-storage export parsing,orders/labs/patient context,message conversion,EHR-derived facts->Pat;M0 reviews source text without SS-MIX2 data.
C2.4 MEDIS standard masters|m|HOT,YJ,JAN,JLAC10/11,disease/procedure/standard drug masters=finite terminology resources with system/version/code functional keys,residuals for unmapped surfaces,version-pin drift->Reb.
C2.5 PMDA package inserts/reports/safety|m|Electronic package-insert XML sections,contraindication/dose/safety-information provenance,Japanese-only binding text,package-insert-vs-guideline predicates=core M0 objects;database-derived safety signals->Prob/Pat.
C2.6 ICD Japanese modification/ICD-11 mapping|m|ICD-10-JP code equality,ICD-11 post-coordination/OWL-aligned mappings enter as admitted terminology bindings/relations;dual-coding/split-merge transition impact->Reb.
C2.7 DPC/K/YJ/receipt crosswalks|m|DPC/PDPS,K codes,YJ/receipt/e-prescription crosswalks enter as finite terminology mappings with reimbursement-granularity residuals;claims analytics/billing validation->Pat/S3.
C2.8 MedDRA/J/JADER|d|MedDRA/J PT/LLT/SMQ adverse-event terminology may be admitted as finite bindings;JADER spontaneous-report risk/no denominator/duplicate-bias signal handling->Prob/Pat.
C2.9 OMOP CDM/OHDSI Japan|d|OMOP tables,Standard Concept IDs,ATLAS Cohort JSON,Japanese vocabulary mapping gaps,cohort diagnostics->Pat/Emin;no M0 patient-cohort authority.
C2.10 MID-NET/NDB/RWE|d|MID-NET/NDB/J-MID/RWE datasets=empirical validation/evaluation substrates,not M0 knowledge sources->Pat,Bench,S3 as applicable.
C3.1 OWL profiles|m|OWL EL/RL/QL/DL semantics dispositioned as finite M0 union-find/relation indexing;EL/RL classification,owl:Nothing justifications,open-world/UNA caveats,ELK/HermiT/Pellet/RDFox evidence->Air/Port.
C3.2 SHACL validation|m|SHACL Core closed-world node/property shapes map to SchemaRegistry,JSON Schema,policy keys,gate evidence validation;SHACL-SPARQL/Rules recursion/inference->Air/Port with portability residuals.
C3.3 RDF named graphs/TriG|m|RDF Dataset/TriG/N-Quads/source-scoped graph convention maps to immutable SourceEdition,SrcGraph,source regions,proof-rooted payloads;RDF-star/PROV-O annotations source-only unless admitted.
C3.4 SKOS/FHIR ValueSet/ConceptMap|m|SKOS labels/broader/exactMatch,FHIR ValueSet/ConceptMap/version/binding-strength governance->Concept,TerminologyBinding,TerminologyRelation,binding status,functional keys;broaderTransitive/subsumption->Air.
C3.5 OBO/ROBOT/ODK/DOSDP|d|ROBOT profile validation,ODK release workflows,DOSDP ontology-generation templates may propose resources;accepted ontology output/effects require Air/Port/Reb/SI,not M0 silent import.
C3.6 BFO/DOLCE upper ontology|x|Upper-ontology commitment outside M0 formalization-QA;only finite admitted policies/relations or an Air evidence object may carry ontology-derived commitments.
C3.7 MIREOT modular imports|d|MIREOT/locality-style ontology import,conservative module extraction,imported-axiom provenance->Air/Port/Reb before any accepted output effect.
C3.8 ontology alignment/repair|d|LogMap/AgreementMakerLight-style mappings,coherence repair,automated merge proposals may create candidate resources;accepted changes->admission/SI/Reb with incoherence reporting.
C3.9 Japanese entity linking/normalization|m|Japanese NEL over J-MeSH/MEDIS/ICD/YJ/HOT surfaces enters as proposal trace + finite terminology admission;ambiguous/unmapped/normalization-drift results emit Ambiguity/Residual/Reb.
C3.10 terminology diff/change impact|d|Versioned ontology/terminology structural+semantic diffs,added/deprecated codes,parent changes,proof-impact analysis->Reb;M0 records versioned resources,exact resource hashes.
C4.1 Lean 4|m|Lean 4/Mathlib/Aesop/grind proof-by-reflection discipline,small kernel,Decidable computation,CSLib reusable LTS/reduction-system patterns inform core checker obligations;external Lean proof claims->Port(lean).
C4.2 Rocq|d|Rocq/Coq Stdlib,MathComp,Iris,MetaCoq secondary proof ecosystem agreement,extracted checker evidence->Port(rocq).
C4.3 Isabelle/HOL|d|Isabelle/HOL,AFP,Sledgehammer,Nitpick/TLAPS audit,proof reconstruction->Port(isabelle) with independent TCB manifest.
C4.4 TLA+/PlusCal|d|TLA+/PlusCal,TLC explicit-state,Apalache SMT-bounded,TLAPS pipeline/convergence/idempotency model-checking->Port(tla)/EM;M0 preserves det. replay.
C4.5 Alloy/Forge|m|Alloy/Forge finite relational counterexamples,scope-bounded instance search,negative-control thinking->CounterexampleSuite,bounded domains,rejected/forbidden payload checks;solver claims->Port(alloy).
C4.6 Why3/WhyML|d|Why3/WhyML VC generation,SMT-backed executable specification agreement->Port(why3).
C4.7 F*|d|F*/Low* verified service or extraction claims->Port/S3;not an M0 semantic authority path.
C4.8 dependent/refinement-type IR schemas|m|Dependent/refinement-style obligations become SchemaRegistry types,FeaturePath constraints,finite enums,collection bounds,residuals;solver-backed refinement proofs->Port.
C4.9 proof by reflection|m|M0 theorem truth is proof-by-reflection: kernel finite checker re-evaluates executable predicates over canon. finite artifacts;external tactics supply optional witnesses only.
C4.10 proof certificates/traces|m|ProofDAG,VerifierWitness,Certificate,external LFSC/Alethe/DRAT/LRAT-style certificates=durable payload shapes;M0 accepts only internally replayed finite_checked certificates.
C4.11 CrossHair|d|CrossHair/Python contract symbolic execution for adapters/harnesses->Port(crosshair)/Ex,never core semantics.
C4.12 SAW/Crucible/Cryptol|d|SAW/Crucible/Cryptol binary,cryptographic,implementation-equivalence claims->Port(saw)/S3 supply-chain evidence.
C4.13 typed functional substrate|m|Rust-owned pure functions,algebraic enums,total pattern matches,no hidden effects,canon. serialization=implementation substrate;other typed FP substrates require Port.
C4.14 memory-safe systems languages|m|Rust=accepted production substrate;Ada/SPARK or other memory-safe backend evidence routes through VerifierPortfolioReport/S3,not parallel core semantics.
C5.1 SMT-LIB/Z3/cvc5/Bitwuzla|m|SMT-LIB scripts,logic selection,QF_LIA/BV/DT/string fragments,external models/unsat cores/proofs represented by ConstraintCoreWitness,Port(smt);internal finite checking authoritative.
C5.2 SAT/MaxSAT repair search|m|SAT/MaxSAT weighted/partial repair-set search,DRAT/LRAT-checkable unsat evidence,deterministic tie-breaks represented by RepairSetSearchTrace;accepted semantic edits still->admission.
C5.3 MUS/MCS/UNSAT core|m|MUS/MCS/group-MUS deletion-minimal cores,solver unsat cores localize contradictions;M0 emits internal deletion-minimal cores,optional external proofs->Port(sat|smt).
C5.4 Datalog/RDFox materialization|m|Pure Datalog semi-naive stratified materialization with stage snapshots/duplicate collapse=M0 closure engine;Datalog± existentials,aggregation variants,RDFox proof claims->Air/Port(datalog).
C5.5 OWL reasoners|d|ELK,HermiT,Pellet/RDFox EL/DL/RL classification,justifications,ontology consistency,axiom pinpointing->Air/Port(owl_reasoner).
C5.6 ASP defaults/exceptions|d|clingo stable-model semantics,negation-as-failure,strong negation,choice/aggregate/weak constraints,clinical exception encodings->Air/Port(asp);grounding blowup becomes residual/limitation evidence.
C5.7 CP-SAT/MiniZinc/OR-Tools|d|MiniZinc,OR-Tools CP-SAT,global constraints,LCG scheduling/optimization search->EM,MDL,S3 or Port(cp_sat|minizinc);non-unique optima require deterministic tie-break manifest.
C5.8 TLC/Apalache|d|TLC/Apalache bounded model-checking counterexample traces for pipeline properties outside replay tests->Port(model_checker|tla).
C5.9 e-graphs/equality saturation|d|egg/egglog e-class congruence,equality saturation,proof extraction beyond fixed NF rewrite system->Air/Port(egraph);bounded saturation required.
C5.10 PRISM/Storm|d|PRISM/Storm DTMC/CTMC/MDP,PCTL/CSL/reward model-checking for probabilistic policy/risk models->Prob backend_family={prism_mc|storm}/Port(prob_model_checker).
C5.11 Prolog/s(CASP)|m|Goal-directed justification trees,tabling-inspired proof parentage inform ProofDAG/VerifierWitness;SWI-Prolog/s(CASP) execution claims->Port(prolog).
C5.12 probabilistic logic programming|d|ProbLog/ProbLog2,cplint,PRISM(Sato),DeepProbLog,smProbLog,ProbEC use distribution semantics;exact inference as WMC compiled to SDD/d-DNNF/checkable circuits->Prob backend/circuit fields,never M0 weights.
C6.1 defeasible logic|m|Strict/defeasible/defeater rules,superiority,ambiguity variants,PROLEG-style Japanese exception reasoning reduce to finite admitted ResolutionTheorem{exception,priority,scope,supersession,reconciliation}.
C6.2 deontic logic|m|Obligation,prohibition,permission,avoidance,recommendation,contrary-to-duty/reparation chains reduce to Direction/normative groups + admitted policies;full modal proof theory->Air/Port.
C6.3 Dung argumentation|d|Abstract attack/defeat graph grounded/preferred/stable semantics beyond finite resolution membership->Air(argumentation_dung)/Emin.
C6.4 ASPIC+/Carneades|d|Structured clinical argument graphs,schemes,premise/exception attacks,argument-strength displays->Air(aspic|carneades),EM,S3;not core theorem truth.
C6.5 assumption-based argumentation|d|Assumption provenance,contrary mapping,dispute calculus beyond proof roots->Air(assumption_based).
C6.6 paraconsistent logic|m|Inconsistent guideline sets produce review candidates,residuals,incoherences without explosive inference;no paraconsistent consequence closure added to M0.
C6.7 event calculus|d|Event Calculus/ProbEC longitudinal Initiates/Terminates/HoldsAt patient-event reasoning->WM/Pat,Prob for uncertain events;not text-only M0.
C6.8 Allen interval algebra|m|M0 interval non-emptiness/STN-like finite numeric/temporal support closure implements deterministic subset;full 13-relation Allen IA,ORD-Horn/STNU->AIRDomainRecord/Port(smt).
C6.9 LTL/MTL/STL|d|LTL/MTL/STL patient-timeline monitors,STL robustness,CT-STL/TEL-style temporal specifications->WM,Pat,Port(model_checker|smt).
C6.10 MCDA|d|AHP,weighted-sum,ELECTRE/PROMETHEE/TOPSIS,GRADE EtD preference tradeoffs->MDL.preference_model_family,EM,S3;rankings never M0 proof objects.
C7.1 hybrid retrieval|m|BM25/BM25+/BM25F sparse baseline,Lucene/Pyserini/Anserini fingerprints,kuromoji/sudachi/mecab_unidic analyzer baselines,RRF(k=60)/weighted fusion,dense recall,reranking=RetrievalProposalTrace fields;retrieval quality->Ret with qrels/metrics.
C7.2 multilingual biomedical embeddings/rerankers|m|BGE-M3,Multilingual-E5,Jina,MedCPT,JMedRoBERTa,BioBERT/PubMedBERT outputs=evidence-discovery proposal traces with model manifests;quality/clinical use gated;licenses recorded in manifests.
C7.3 ColBERT/late interaction|m|ColBERT/JaColBERT/PLAID MaxSim token evidence,compression/centroid parameters enter late_interaction_family/manifests;acceptance depends on source-grounded discharge,Ret metrics.
C7.4 recommendation-level segmentation|m|CQ,recommendation,PICO,EtD,evidence-table,GRADE strength/certainty segmentation=SrcGraph node kinds,RecMetadata,retrieval segment granularity;extractor quality->Ex/Ret.
C7.5 layout-aware Japanese PDF/table extraction|m|Yomitoku,MinerU,Marker,LayoutLMv3,DocLayout-YOLO,table-transformer/OCR outputs become SrcGraph/MechObs facts only after byte-stable extraction;adapter quality,vertical text,round-trip checks->Ex.
C7.6 GraphRAG|m|MS GraphRAG,HippoRAG,LightRAG graph traversal/community summaries enter as proposal trace/source-region evidence;entity drift/hallucinated triples require fixed terminology,Ret.
C7.7 query decomposition/routing|m|LangGraph/LlamaIndex/Self-RAG-style decomposition hashes,routing decisions=ProposalProvenanceManifest/RetrievalProposalTrace fields;routing quality gated by Ret/EM.
C7.8 citation-grounded generation|m|Inline/post-hoc citation,Anthropic Citations-style span ranges,ALCE/LongBench-Cite citation precision map to source regions/proof roots;citations remain evidence until discharge.
C7.9 Japanese-English cross-lingual alignment|m|BGE-M3/multilingual-E5,mDPR,J-MeSH↔MeSH,MEDIS↔ICD,MedDRA/J↔EN mappings enter finite terminology/gloss/view-only resources with version pins;translation quality->Ret/Reb.
C7.10 RAG evaluation|d|RAGAS,TruLens,ARES,ALCE citation precision,faithfulness/context recall,MEMERAG Japanese calibration->Ret/Emin metric_family fields;judge outputs not M0 proof.
C8.1 closed frontier model ensembles|m|GPT-5.5/Claude Opus/Gemini-class closed LLM outputs=ProposalProvenanceManifest(generator_family=closed_frontier_llm) + structured-output/prompt/model manifests;non-determinism,PHI/API constraints->Pat/S3/EM.
C8.2 domain medical models|m|Med-Gemini,MedGemma,Meditron,GatorTron,LLaVA-Med,JMedLLM,UTH-BERT/JMedRoBERTa outputs=ProposalProvenanceManifest(domain_medical_model) for NER/normalization/embedding;no native verification.
C8.3 proof models/environments|m|DeepSeek-Prover,Kimina,LeanDojo/ReProver/Leanstral suggestions=ProposalProvenanceManifest(proof_model);checker acceptance uses internal finite checks or Port(lean).
C8.4 constrained decoding|m|xgrammar/Outlines/Guidance/JSONSchemaBench-style grammar-state artifacts/token masks=evidence-discovery proposal aids;GeneratorGrammarArtifact/T-GEN-Static remains acceptance authority;semantic dictionary constraints require post-check.
C8.5 tool-calling agents|m|MCP/function-calling/code-execution agents connected to Lean/SMT/terminology tools=ProposalProvenanceManifest(tool_calling_agent) with effect rows;DischargeProposal admits only effect-free accepted artifacts.
C8.6 self-consistency/convergence|d|k-run self-consistency,dominant canonical hash,ATP/embedding clusters,ASR/idempotency/convergence metrics->Emin;det. replay remains S0.
C8.7 retrieval-augmented autoformalization|m|LeanDojo/ReProver-style premise retrieval,clinical ontology exemplar retrieval=proposal trace;generated IR accepted only by discharge,proof checking.
C8.8 critique/adjudication|m|Critic-defender-judge,NLI contradiction checks,independent model-family critique feed ReviewerRecord/AdmissionRecord;judge/model bias metrics->EM/S3,not theorem truth.
C8.9 program-aided language models|m|PAL/Python/CQL/SQL/FHIRPath executable intermediates=program_aided_lm proposals;execution-output equivalence requires counterexample-suite discharge or Port,sandbox effects kept out.
C8.10 verifier-guided decoding/repair|m|Baldur/Goedel/HTPS/PRM-style bounded repair loops may propose artifacts;accepted changes require det. suite discharge/admission;loop metrics->EM/SI.
C8.11 LoRA/QLoRA adapters|d|LoRA/QLoRA/DoRA/full-finetune/prompt or retrieval-index updates->SelfImprovementEvidence.adapter_family + Bench/EM holdout,catastrophic-forgetting,safety-regression evidence.
C8.12 world models/patient trajectory|d|JEPA,latent-dynamics,ETHOS/Foresight/EHRWorld tokenized-EHR,multimodal trajectory predictors->WorldModelProfileRecord.world_model_family,Pat/S3;causal/counterfactual claims need causal_design evidence.
C9.1 gold guideline-to-IR corpus|d|Clinician/formalist adjudicated source_passage/CQ/recommendation corpus with Cohen/Fleiss/Krippendorff/γ agreement,split stratification,gold IR conformance->BenchmarkRelease.
C9.2 semantic equivalence/idempotency/convergence|m|Canonical bytes,NF idempotency,replay identity,fixture convergence=M0 acceptance gates;empirical AST isomorphism,logical equivalence,round-trip,convergence->Emin.
C9.3 contradiction/collision benchmark|m|Synthetic/real cross-guideline direct/action/temporal/threshold/epistemic cases seed Appendix A/T-Conflict-Fixtures/T-Factual-Fixtures;external clinician utility->Bench/EM.
C9.4 CQL/FHIR/DMN conformance suites|m|Inferno,FHIR $validate,CQL-to-ELM,DMN/FEEL,JP Core/JAMISDP conformance suites=CounterexampleSuite/admission inputs;external standard conformance->Ex/Port/S3.
C9.5 metamorphic/property-based tests|m|Paraphrase invariance,idempotency,merge commutativity,eligibility monotonicity,property generators extend acceptance gates/counterexample suites;empirical MR violation rates->EM.
C9.6 shadow-mode/silent trial|d|Shadow-mode production logs,AUROC/calibration/lead-time/alert-volume endpoints,stopping rules require patient data,deployment authority->Pat/S3.
C9.7 alert fatigue/tiered alerts|d|Hard-stop/soft-stop/informational tiering,override reason taxonomy,alert-governance metrics require deployment usability authority->S3;M0 may carry strength/actionability metadata only.
C9.8 CDS Five Rights|x|Right information/person/channel/format/time=CDS deployment configuration->S3;M0 produces static review artifacts,no channel/timing authority.
C9.9 human factors/ISO user-centered design|d|ISO 9241-210/62366 use specification,HFMEA,URRA,NASA-TLX/SUS/usability tests govern UI/deployment evidence->S3;M0 UI renders static proof artifacts.
C9.10 explanation quality|m|Traceability,citation precision,proof readability,controlled-NL faithfulness,clinical actionability,deterministic glosses map to report fields/falsification criteria;human/LLM judge metrics->EM/S3.
C9.11 equity/subgroup/calibration|d|Subgroup,external validation,calibration,Brier/ICI/fairness metrics,Japanese-population validation->EM/S3;no M0 demographic claim.
C9.12 implementation science|d|CFIR,NASSS,RE-AIM adoption/maintenance/reimbursement,workflow embedding->S3;not IR construction authority.
C10.1 GSN/SACM assurance cases|d|GSN,SACM,OntoGSN,Assurance 2.0,D-Case goal/strategy/solution/defeater graphs->S3AssuranceEvidence.assurance_case_family.
C10.2 ISO 14971/62304/62366|d|Risk management,software lifecycle,usability engineering,AI/ML hazard-taxonomy mappings->S3 risk/software/usability files.
C10.3 FDA/PMDA/IMDRF CDS/SaMD|d|FDA CDS,PMDA SaMD,IMDRF MLMD/GMLP/N81 classification,IDATEN/PCCP change protocols,transparency/independent-review claims->S3 jurisdiction/change fields.
C10.4 NIST AI RMF/ISO 42001|d|AI RMF Govern/Map/Measure/Manage,GenAI risk profile,ISO/IEC 42001 controls->S3 ai_management_system/control-map evidence.
C10.5 APPI/medical privacy|d|APPI special-care data,cross-border transfer,NGMIA anonymized/pseudonymized medical info,certified processors->Pat privacy_regime/deidentification fields,sometimes S3.
C10.6 STRIDE/LINDDUN/Zero Trust|d|STRIDE,LINDDUN,Zero Trust,OWASP LLM/MITRE ATLAS threat models->S3 threat_model_families/security evidence.
C10.7 de-identification/PPRL|d|k-anonymity,l-diversity,t-closeness,DP,PPRL/Bloom-filter linkage,secure on-site analysis->Pat deidentification_family/record_linkage fields.
C10.8 SBOM/AIBOM/reproducible supply chain|d|SPDX 3.0 AI/Dataset profiles,CycloneDX,in-toto/SLSA,Sigstore/Rekor provenance->S3 sbom/aibom/reproducible-build evidence;M0 keeps toolchain manifests/hashes.
C10.9 knowledge CI/CD|d|GitOps,FSH/SUSHI/IG Publisher,semver,canary/blue-green,rollback,IDATEN/PACMP deployment authority->S3;M0 keeps immutable source editions/replay.
C10.10 observability/audit/continuous verification|d|OpenTelemetry/Langfuse,FHIR AuditEvent/IHE BALP,hash-chain logs,continuous verification->S3 observability;M0 replay offline,static.
C10.11 drift monitoring|d|PSI/KS/Wasserstein/ADWIN/KSWIN + ontology/terminology structural/semantic diff for model,rule,ontology,terminology drift->Reb,SI,S3.
C10.12 incident response/post-market surveillance|d|Detect/triage/contain/CAPA/PMS,IMDRF AET,PMDA fuguai/JADER,AIID/ATLAS reports->S3 incident/post-market evidence.
```

Agent-language form DNA incorporated by form, not clinical semantics:

```text
T Form unit|S|Disposition
one canon. representation per fact|m|§0/§1 define one schema-authority path/canon. payload bytes.
one canon. command per operation|m|§11.1=the sole command vocabulary.
byte-stable artifacts before convenience APIs|m|§1.5,§1.6,§9,§11->canon. bytes/replay before UI views.
det. internal checker before backend breadth|m|§9.1=authoritative;Port records external agreement.
constrained syntax/valid-next-token masks|m|§6.2 emits evidence-discovery GeneratorGrammarArtifact/masks for proposal decoding;§6.2 T-GEN-Static accepts CKC-GEN artifacts.
explicit effects/residuals|m|§2,§6.4,§7.3,§8,§12 type every effect,residual,ambiguity,incoherence.
local change blast-radius|m|§0,§3.2,§7.1,§11.3->stage-stratified,session-sized changes.
proof-carrying artifacts/derivation traces|m|§7.2,§9.1,§9.2 define ProofDAG,VerifierWitness,Certificate.
proposal/runtime separation|m|§6.4 admits model,retrieval,tool outputs only after det. discharge.
explicit unsupported constructions|m|§2 residual classes/every builtin return typed unsupported outcomes.
```

## 4. Source grounding

### 4.1 Source and permission schemas

```text
S SourceEdition(edition_id:Id,source_hash:Hash,bibliographic_identity:Text<semantic_ja>,source_class:SourceClass,publisher:Text<semantic_ja>?,society:Text<semantic_ja>?,edition_label:Text<semantic_ja>?,publication_date:Text<semantic_ja>?,access_date:Text<semantic_ja>?,license_or_permission_ref:Id,extraction_manifest_hash:Hash)

S SourcePermissionRecord(source_edition_hash:Hash,rights_holder:Text<semantic_ja>,access_ref:Text<raw_source>?,license_label_or_contract_ref:Text<semantic_ja>,redistribution_status:RedistributionStatus,allowed_artifacts:Set[AllowedArtifact],permission_evidence_hash:Hash)

E RedistributionStatus = redistributable | reconstructable | restricted_internal_only

E AllowedArtifact = source_bytes | source_graph | quoted_snippets | offsets_only | hashes_only | derived_labels

S CorpusDocument(doc_id:Id,source_edition_hash:Hash,title_ja:Text<semantic_ja>,title_en:Text<semantic_en>?,content_hash:Hash,extraction_manifest_hash:Hash)
```

`SourceEdition` immutable; revisions create a new one. Cross-edition proof transport requires `G-REBIND`.

Permission semantics:

```text
redistributable: reports may include every artifact in allowed_artifacts.
reconstructable: reports carry offsets, hashes, source-region IDs, derived labels to
  reconstruct reviewed evidence under holder's access terms.
restricted_internal_only: accepted internal artifacts may be checked/replayed; exported
  reports carry only artifact hashes, source-region IDs, derived labels in allowed_artifacts.
permission_limited emission: report build emits Residual(class=permission_limited) when a
  requested view requires an artifact kind absent from SourcePermissionRecord.allowed_artifacts.
```

### 4.2 SourceGraph schemas

`SourceGraph` = finite directed graph over source structure, text anchors, table layout. Every accepted semantic fact lives in typed node/edge/span/anchor/region payloads.

```text
S SourceGraph(graph_id:Id,source_edition_hash:Hash,nodes:Set[SourceNode],edges:Set[SourceEdge],spans:Set[SourceSpan],anchors:Set[SourceAnchor],root_node_id:Id,extraction_manifest_hash:Hash)

S SourceNode(node_id:Id,kind:SourceNodeKind,attrs:SourceNodeAttrs)

S SourceNodeAttrs(label:Text<semantic_ja>?,table_id:Id?)

S SourceEdge(edge_id:Id,kind:SourceEdgeKind,from:Id,to:Id,attrs:SourceEdgeAttrs)

S SourceEdgeAttrs(role:Id?,table_id:Id?,row_index:UInt?,column_index:UInt?,reading_order:UInt?)

S SourceSpan(span_id:Id,source_node_id:Id,section_path:List[Text<semantic_ja>],page:UInt?,bbox:BBox?,table_cell_id:Id?,char_start:UInt,char_end:UInt,raw_text:Text<raw_source>,nfkc_text:Text<source_nfkc>,search_text:Text<semantic_ja>,display_text:Text<view_text>,language:Lang,reading_order:UInt)

S SourceAnchor(anchor_id:Id,span_id:Id,char_start:UInt,char_end:UInt,raw_text:Text<raw_source>,search_text:Text<semantic_ja>)

E Lang = ja | en | other

S BBox(top:Rational,left:Rational,bottom:Rational,right:Rational)
```

`SourceNodeAttrs.label` = structural extractor label (section/table role). Source text stored only in `SourceSpan`/`SourceAnchor`.

SourceGraph validation:

```text
P-SG-total-text: every accepted textual unit from a registered source has a SourceSpan and
  SourceAnchor or an extraction_uncertain residual.
P-SG-total-support: every accepted theorem support is a finite SourceRegion.
P-SG-canonical: same ExtractionManifest and source bytes produce identical SourceGraph
  canonical bytes.
P-SG-permission: accepted M0 reports store source-region IDs not quotations; any artifact
  storing raw source text is in SourcePermissionRecord.allowed_artifacts.
```

Textual units: running text, headings, tables, cells, footnotes, captions, appendices, recommendation statements, explicit cross-reference labels, and figure text only when extracted as text.

### 4.3 Source regions and closure

```text
E RegionMember = node:Id | span:Id | cell:Id | anchor:Id

S SourceRegion(region_id:RegionId,source_edition_hash:Hash,seed_members:Set[RegionMember],closed_members:Set[RegionMember],closure_certificate_hash:Hash)
```

`source_region_closure(SourceGraph S, Set[RegionMember] seed) -> OperationResult[SourceRegion]`: deterministic, total over schema-valid inputs:

```text
1 Validate every seed member exists in S. Missing seed → Residual(class=unsupported_construction),
  code=missing_region_member.
2 Compute finite universe U of region members addressable from S: nodes, spans, anchors,
  table-cell IDs recorded in spans, derived cell addresses.
3 R := seed.
4 Repeat until R unchanged; for each m in R by canonical_sort_key:
       add containing node, span, cell, anchor addresses;
       add containing heading, section, paragraph, list, list_item, table, row, column,
         document nodes via contains edges;
       add row and column header cells per table cell;
       add table caption per table member;
       add footnote body and target per footnote marker/body via footnote_of edges;
       add cross-reference target per crossref_targets edge;
       add continuation targets needed for a complete sentence, recommendation, table row, or caption;
       add adjacent span only when a continuation edge links it.
5 If any required table coordinate, header, caption, footnote, continuation, or cross-reference
  target absent, emit earliest residual by source_order_key: unsupported_table_structure for
  table/caption/footnote/continuation failure; unsupported_cross_reference for cross-reference failure.
6 Require R ⊆ U every iteration. Derived member outside U → unsupported_table_structure.
7 Before accepting, check `SourceRegion.seed_members` and `SourceRegion.closed_members`
  SchemaCollectionBound rows; on overflow call `HandleBoundOverflow`, return its exact status.
8 Return success(SourceRegion, closed_members sorted by canonical_sort_key).
```

Terminates: `U` finite; each successful iteration strictly grows `R`, at most `|U|-|seed|` times before fixed point.

```text
S RegionClosureCertificate(seed_members_digest:Hash,source_graph_hash:Hash,possible_member_count:UInt,iterations:UInt,added_member_batches:List[Set[RegionMember]],residual_hashes:Set[Hash])
```

`seed_members_digest = sha256(canonical_payload_bytes(seed_members))` over referencing `SourceRegion.seed_members`. Certificate accepted before the `SourceRegion` referencing it via `closure_certificate_hash` (acyclic). Interpretation region admissible when certificate valid: seed digest matches, replaying closure over certified `SourceGraph` reproduces `added_member_batches`, and `seed_members` ∪ batches = `closed_members`.

### 4.4 Extraction and mechanical observations

```text
S ExtractionManifest(manifest_id:Id,source_edition_hash:Hash,adapter_manifest_hash:Hash,input_bytes_hash:Hash,output_graph_hash:Hash,diagnostics_hashes:Set[Hash],replay_manifest_hash:Hash)

S AnalyzerManifest(analyzer_id:Id,analyzer_name:Id,analyzer_version:Text<identifier_ascii>,config_hash:Hash,replay_manifest_hash:Hash)

S MechanicalLexicon(lexicon_id:Id,entries:Set[LexiconEntry],authority:Authority)

S LexiconEntry(surface:Text<source_nfkc>,concept_candidate:Id)

E MechObsKind = text_node | anchor_span | token | table_cell | table_edge | caption_edge | footnote_surface | crossref_surface | lex_surface_hit | quantity_surface | temporal_surface | modality_marker | negation_marker

S MechObsPayload(obs_id:Id,kind:MechObsKind,source_region_id:RegionId,anchor_id:Id?,raw_text:Text<raw_source>?,nfkc_text:Text<source_nfkc>?,normalized_text:Text<semantic_ja>?,fields:Map[Id,Text<semantic_ja>],analyzer_manifest_hash:Hash?,authority:Authority)
```

`ExtractionManifest.input_bytes_hash` = `sha256(exact_recorded_bytes)` of extraction-adapter input; owning manifest supplies those bytes (§1.2 raw-recorded convention).

Authority invariant: `MechanicalLexicon.authority = MechObsPayload.authority = mechanical_authority`. `LexiconEntry` rows inline in owning lexicon; `lex_surface_hit` observation records matched entry's `concept_candidate`.

Emission and consumers:

```text
T MechObsKind | Emission site | M0 consumer
text_node | one per textual SourceSpan with node kind and normalized text fields | pattern generators for sentence, paragraph, heading, and source-metadata candidates
anchor_span | one per SourceAnchor | lexical, modality, negation, quantity, temporal, and action-pattern generators
token | deterministic tokenizer over each anchor/span | sequence premises and bounded-path fixtures
table_cell | one per table cell span with row and column fields | decision-table row generators and §4.3 table closure
table_edge | one per typed table adjacency/header relation | table-header closure, row assembly, and table_value_disagreement fixtures
caption_edge | one per caption/table link | source-region closure and table support construction
footnote_surface | one per extracted footnote marker/body pair | source-region closure and residual emission for unsupported footnote structure
crossref_surface | one per explicit cross-reference surface | source-region closure and unsupported_cross_reference residuals
lex_surface_hit | one per MechanicalLexicon surface hit | term-resource generators and multiple_terms ambiguity fixtures
quantity_surface | one per comparator-number-unit surface | quantity licenses, context intervals, and numeric-threshold predicates
temporal_surface | one per temporal cue surface | temporal licenses and gloss rendering
modality_marker | one per deontic or recommendation cue | norm-license generators that set Direction and original modality phrase
negation_marker | one per explicit negation cue | negative condition, contraindication, and avoid/against generators
```

Accepted `MechObsPayload` guarantee optional-field presence by kind; schema-visible, usable by `T-GEN-Static` when assigning optional source fields to required template targets:

```text
T MechObsKind | Optional fields guaranteed present
text_node | normalized_text
anchor_span | anchor_id
table_cell | anchor_id, normalized_text
lex_surface_hit | anchor_id, normalized_text
quantity_surface | anchor_id, normalized_text
temporal_surface | anchor_id, normalized_text
modality_marker | anchor_id, normalized_text
negation_marker | anchor_id, normalized_text
```

M0 uses deterministic fixture extraction for source bytes already in the test corpus. Additional extraction adapters require `G-EXTRACTOR-ADAPTER`.

`ObserveMech(SourceGraph, AnalyzerManifest, MechanicalLexicon*) -> OperationResult[Set[MechObsPayload]]`: pure over declared inputs. Enumerates spans/anchors by `source_order_key`, applies lexicon and surface recognizers in manifest order, canonicalizes, collapses duplicates by artifact hash. Textual unit required by `P-SG-total-text` with no stable anchor or conflicting byte offsets → `Residual(class=extraction_uncertain)`, preserving every observation whose support stays closed and permission-valid. At every MechObsPayload output bound, calls `HandleBoundOverflow`, returns exact overflow status. `T-Mech-Determinism` validates repeated-run byte identity.

## 5. Admitted terminology and semantic policies

### 5.1 Terminology resources

Admitted finite resources or consequences of `term_resource` generators.

```text
S TerminologyResourceSet(resource_set_id:Id,concepts:Set[Concept],bindings:Set[TerminologyBinding],relations:Set[TerminologyRelation],admission_record_hashes:Set[Hash],accepted_effect_row:Set[Effect])

S Concept(concept_id:Id,label_ja:Text<semantic_ja>,label_en:Text<semantic_en>?,semantic_type:Id,source_region_ids:Set[RegionId])

S TerminologyBinding(binding_id:Id,concept_id:Id?,surface:Text<semantic_ja>,system:Id,code:Text<identifier_ascii>?,version:Text<identifier_ascii>?,status:BindingStatus,source_region_ids:Set[RegionId])

S TerminologyRelation(relation_id:Id,kind:TerminologyRelationKind,from_concept_id:Id,to_concept_id:Id,source_region_ids:Set[RegionId])
```

### 5.2 Terminology closure and functional keys

Accepted finite index from a `TerminologyResourceSet`.

```text
S TerminologyClosure(closure_id:Id,terminology_resource_set_hash:Hash,representative_map:Map[Id,Id],equivalence_classes:Set[ConceptEquivalenceClass],normalized_relations:Set[TerminologyRelation],surface_index:Set[SurfaceIndexEntry],code_key_index:Set[FunctionalKeyIndexEntry],surface_key_index:Set[FunctionalKeyIndexEntry],incoherence_hashes:Set[Hash],proof_roots:Set[ProofId])

S ConceptEquivalenceClass(representative_concept_id:Id,member_concept_ids:Set[Id])

S SurfaceIndexEntry(surface:Text<semantic_ja>,representative_concept_ids:Set[Id],binding_statuses:Set[BindingStatus])

S FunctionalKeyIndexEntry(key_kind:FunctionalKeyKind,system:Id,version_or_empty:Text<identifier_ascii>,key_value:Text<semantic_ja>,representative_concept_ids:Set[Id])

E FunctionalKeyKind = code_key | surface_key
```

`BuildTerminologyClosure(TerminologyResourceSet T) -> OperationResult[TerminologyClosure]` total over schema-valid `T`:

```text
1 Validate every relation endpoint names an existing Concept. Failing endpoint emits
  Incoherence(class=incompatible_generator_outputs) with code=missing_concept_endpoint, omitted
  from normalized_relations.
2 Per binding:
     status in {exact,synonym,ambiguous} => require concept_id names existing Concept;
     status=unmapped => require concept_id absent;
     else emit Incoherence(class=incompatible_generator_outputs) with code=bad_binding_endpoint.
3 Build union-find classes over Concept IDs using TerminologyRelation.kind in
  {exact, synonym, unit_equivalent, section_equivalent, action_kind_equivalent}.
4 Representative of each class = min concept_id by canonical_sort_key.
5 Replace every relation endpoint by its representative, collapse duplicate relations.
6 surface_index[surface] = canonical set of representative concepts for bindings
  with status in {exact, synonym, ambiguous} and valid concept_id.
7 code_key for every binding with status in {exact, synonym} and code present:
     (system, version_or_empty, code).
8 surface_key for every binding with status = exact and code absent:
     (system, version_or_empty, surface).
9 functional_key_collision exists when one code_key or surface_key maps to >1
  representative concept.
10 mutually_exclusive_term_mapping exists when one surface maps to two representative
  concepts connected by mutually_exclusive in either direction after representative replacement.
11 Emit Incoherence per collision or mutually exclusive mapping, record hashes in
  TerminologyClosure.incoherence_hashes, retain the finite closure for residual reporting.
12 Check every TerminologyClosure collection bound; on overflow call `HandleBoundOverflow` and
  return its exact status, else return success(TerminologyClosure) with the §1.7 primary
  status raised to incoherence when step 11 emitted incoherence artifacts.
```

`status = ambiguous`: participate in `surface_index`, produce `Ambiguity(class=multiple_terms)` when a generator demands a single concept; ambiguity indexing, not functional-key satisfaction. `status = unmapped`: `concept_id` absent, produce `Residual(class=missing_terminology)` when a generator demands a concept. `status = unmapped` with present `concept_id` = `invalid_payload`; checker emits only the schema diagnostic for that binding.

M0 terminology reasoning = union-find plus finite relation indexing. OWL/SKOS classification, e-graph saturation, ontology alignment repair, terminology parity claims require §3.3 gates.

### 5.3 Semantic policy set

Every M0 semantic judgment outside the fixed §8 core = admitted finite input in `SemanticPolicySet`. `DischargeProposal` is the only operation accepting a `SemanticPolicySet`; CloseM0 reads it via `ClosureInput.semantic_policy_set_hash`, never produces a replacement. Absent required policy fact yields `Residual(class=missing_policy)` or `unsupported` exactly where the consuming algorithm states.

```text
S SemanticPolicySet(policy_set_id:Id,action_slot_specs:Set[ActionSlotSpec],action_target_relations:Set[ActionTargetRelation],output_exclusions:Set[OutputExclusion],metadata_singleton_keys:Set[MetadataKey],admission_record_hashes:Set[Hash],accepted_effect_row:Set[Effect])

S ActionSlotSpec(slot_id:Id,value_kind:Id,discriminates_action_identity:Bool,normalization:SlotNormalizationKind)

E SlotNormalizationKind = concept_representative | unit_quantity | literal_identity

S ActionTargetRelation(action_kind:Id,relation_kind:TerminologyRelationKind,left_concept_id:Id,right_concept_id:Id,symmetric:Bool)

S OutputExclusion(output_slot_id:Id,left_value:ReadingRef,right_value:ReadingRef,symmetric:Bool)

E MetadataKey = bibliographic_identity | publisher | society | edition_label | publication_date | access_date | license_or_permission_ref | source_class
```

Admission validation sorts rows by canonical payload bytes, rejects duplicate rows with different payload bytes under these semantic keys. Consumers canonicalize concept IDs through stage-20 `TerminologyClosure` before lookup; admission-time duplicate-key validation uses declared candidate concept IDs, records any quarantined lookup key in `ValidationManifest.diagnostic_hashes` of the policy-admission manifest for the accepted policy set.

```text
ActionSlotSpec key = slot_id.
ActionTargetRelation key =
  (action_kind, relation_kind, endpoint_a, endpoint_b, symmetric),
  where endpoint_a <= endpoint_b by canonical_sort_key when symmetric=true,
  otherwise endpoint_a=left_concept_id and endpoint_b=right_concept_id.
OutputExclusion key = (output_slot_id, value_a, value_b, symmetric),
  with the same endpoint sorting rule when symmetric=true.
Metadata singleton key = metadata key literal.
```

`ActionTargetRelation.relation_kind` proof-visible. M0 target-overlap policy uses `contraindication_target`; other terminology relation kinds keep their §5.2 consumers. Duplicate semantic keys with different payload bytes emit `Incoherence(class=incompatible_generator_outputs)` during `DischargeProposal`, retained as quarantined rows in the accepted `SemanticPolicySet`. Quarantined rows recorded via `ValidationManifest.diagnostic_hashes` and `ClosureOutput.incoherence_hashes`, unavailable for normal lookup. Non-quarantined rows remain usable in the same policy set. Consumer querying a quarantined key propagates the recorded incoherence; querying an absent non-quarantined key emits `Residual(class=missing_policy)`. Retention lets the single demo `SemanticPolicySet` contain the dose-collision fixture while the main theorem fixture uses the route and administration-speed rows.

## 6. CKC-GEN-core and admission

### 6.1 Generator form

`CKC-GEN-core` = accepted M0 generator language: finite, stratified, proof-producing relational transducer language.

```text
S CKCGen(generator_id:Id,profile:GeneratorProfile,stage:Int,sort:Id,scope:ClassPred,vars:List[TypedVar],premises:List[Premise],head:Head,admission_record_hash:Hash,replay_manifest_hash:Hash,accepted_effect_row:Set[Effect])

S TypedVar(var_id:Id,domain:FiniteVarDomain)

E FiniteVarDomain = SourceNode | SourceRegion | MechObsPayload | PatternObs | Match | MatchClass | ClassMember | Concept | TerminologyBinding | TerminologyRelation | License | ResolutionTheorem | AIRCoreRecord | CKCNormalForm | FrozenConstant | ParsedQuantity | DiagnosticTag
```

Generator profile dispatch:

```text
obs_pattern: stage -10, emits PatternObs.
term_resource: stage 10, emits TerminologyResourceSet fragments.
sem_rule: stages 30 and 40, emits semantic licenses and resolution theorems.
bridge: stage 60, emits theorem-supporting bridge diagnostics.
residual: stage 80, emits Residual, Ambiguity, Incoherence, and Diagnostic payloads.
gloss: stage 70, emits GlossTemplate or deterministic GlossView helpers.
```

Every `FiniteVarDomain` variant names one finite relation enumerated by §7.1. `FrozenConstant`, `ParsedQuantity`, `DiagnosticTag` = finite fixture relations loaded from the `ClosureInput` manifest, recorded in `ClosureBoundCertificate.finite_domain_cardinalities`.

```text
S FiniteFixtureManifest(manifest_id:Id,frozen_constants:Set[FrozenConstant],parsed_quantities:Set[ParsedQuantity],diagnostic_tags:Set[DiagnosticTag],replay_manifest_hash:Hash)

E FrozenConstantValue = literal:Literal | reading:Reading | context:ContextExpr | region:RegionExpr

S FrozenConstant(constant_id:Id,value:FrozenConstantValue,source_region_ids:Set[RegionId],proof_roots:Set[ProofId])

S ParsedQuantity(quantity_id:Id,comparator:Cmp,value:Rational,unit_id:Id,source_region_id:RegionId,raw_anchor_id:Id)

S DiagnosticTag(tag_id:Id,residual_class:ResidualClass?,ambiguity_class:AmbiguityClass?,incoherence_class:IncoherenceClass?,diagnostic_code:Id)
```

Generator constraints:

```text
C-GEN-single-head:
  One satisfying environment emits one head payload.

C-GEN-finite-vars:
  Every variable ranges over a finite domain declared in ClosureBoundCertificate.

C-GEN-head-safety:
  Every head variable is bound by a positive premise.

C-GEN-stratified:
  License and resolution-theorem premises read lower stages only.

C-GEN-effect-pure:
  Accepted generators use accepted_effect_row = {}.

C-GEN-grounded:
  Each emitted semantic slot is grounded in source support, inherited from an admitted theorem,
  or introduced by an admitted compiler helper with a proof node.

C-GEN-total-diagnostic:
  Static rejection, unsupported constructs, and bound excess emit typed diagnostics.
```

### 6.2 Predicate, premise, and head grammar

```text
E ClassPred = true | false | (has FeaturePath) | (eq FeaturePath Literal) | (neq FeaturePath Literal) | (in FeaturePath Set[Literal]) | (intersects FeaturePath Set[Literal]) | (count FeaturePath Cmp UInt) | (and List[ClassPred]) | (or List[ClassPred]) | (not ClassPred)

E Cmp = eq | ne | lt | le | gt | ge

E Premise = (mobs Var MechObsPattern) | (pobs Var PatternObsPattern) | (match Var MatchPattern) | (class Var MatchClassPattern) | (member Var MatchClassVar) | (capture MatchVar RoleName Var) | (rel RelationName List[Term]) | (license Var LicensePattern) | (resolution Var ResolutionTheoremPattern) | (eq Term Term) | (neq Term Term) | (in Term Set[Literal]) | (builtin BuiltinName List[Term] OutputVars) | (collect Var LicensePattern List[Premise] CollectBound) | (empty LicensePattern List[Premise]) | (seq RegionExpr List[SeqItem] List[RoleBinding]) | (bounded-path AxisRegex AnchorOrNode AnchorOrNode UInt)

E BuiltinName = support_of | support_union | within_support | source_region_closure | canonical_set | proof_visible_signature | unit_normalize | normalize_context | ctx_compatible | normalize_action | same_normalized_action | consequents_compatible | theorem_minimize | dependency_minimize

E Head = (pattern PatternObsTemplate) | (license LicenseTemplate) | (theorem ResolutionTheoremTemplate) | (residual ResidualTemplate) | (ambiguity AmbiguityTemplate) | (incoherence IncoherenceTemplate) | (gloss GlossTemplate) | (diagnostic DiagnosticTemplate)
```

S-expr grammar = display notation. Parser emits canonical JSON tagged objects (schemas below), applies §1 string policies, validates field paths vs target schema, serializes by §1.5.

`ckc gen check` also emits agent-facing grammar artifact for proposal decoders: `authority = evidence_discovery_only`, `accepted_effect_row = {}`. Supports constrained decoding + authoring diagnostics; accepted CKC-GEN semantics fixed by canonical tagged JSON, SchemaRegistry validation, `T-GEN-Static`.

```text
S GeneratorGrammarArtifact(grammar_id:Id,grammar_version:Id,nonterminal_schema_map:Set[NonterminalSchemaEntry],production_schema_map:Set[ProductionSchemaEntry],first_follow_sets:Set[FirstFollowSet],parser_state_machine:ParserStateMachine,valid_next_token_masks:Set[ValidNextTokenMask],authority:Authority,replay_manifest_hash:Hash,accepted_effect_row:Set[Effect])

S NonterminalSchemaEntry(nonterminal:Id,schema_id:Id,schema_version:Id)

S ProductionSchemaEntry(production_family_id:Id,nonterminal:Id,tagged_union_alternative:Id,schema_id:Id,constructor_tag:Id)

S FirstFollowSet(nonterminal:Id,first_token_classes:Set[TokenClass],follow_token_classes:Set[TokenClass],nullable:Bool)

S ParserStateMachine(machine_id:Id,start_state_id:Id,accepting_state_ids:Set[Id],states:Set[ParserState],transitions:Set[ParserTransition],reductions:Set[ParserReduction])

S ParserState(state_id:Id,lr_items_digest:Hash)

S LRItem(production_family_id:Id,tagged_union_alternative:Id,dot_position:UInt,lookahead:TokenClass)

S ParserTransition(from_state_id:Id,token_class:TokenClass,to_state_id:Id)

S ParserReduction(state_id:Id,lookahead:TokenClass,production_family_id:Id)

S TokenClass(token_class_id:Id,lexical_policy:StringPolicy?)

S ValidNextTokenMask(state_id:Id,token_classes:Set[TokenClass])
```

Artifact stores derived grammar structures inline; decoders + `T-GEN-Grammar-Evidence` read them directly. Display grammar, production families, finite token-class table (token-class ids, literal token bytes, lexical policies) = fixed §6.2 content committed by `spec_contract_hash`. `grammar_id`/`grammar_version` name that grammar; emitting run's `SchemaRegistry` binding rides `replay_manifest_hash`. `lr_items_digest = sha256(canonical_payload_bytes(items))` over state's LR(1) item set as `Set[LRItem]`; `LRItem` = `ProductionSchemaEntry` production key + dot position + one lookahead token class.

`FIRST(N)` = least fixed point of token classes beginning a derivation from `N`; `FOLLOW(N)` = least fixed point of token classes immediately following `N` in any derivation from CKCGen start symbol. Grammar finite → both terminate by monotone growth over finite token-class universe. Parser state machine = canonical LR(1) item automaton from display grammar, states sorted by canonical item-set bytes (`lr_items_digest` input). `ValidNextTokenMask(state)` = union of transition token classes leaving state + reduction lookahead token classes in state.

`ParseCKCGen(input_bytes) -> OperationResult[CKCGen]` total:

```text
1 Decode input UTF-8; decoder failure → Diagnostic(code=parse_utf8_error).
2 Tokenize by finite token-class table; unmatched byte span → Diagnostic(code=parse_token_error).
3 Traverse ParserStateMachine deterministically. Missing transition/reduction → emit
  Diagnostic(code=parse_unexpected_token) with ValidNextTokenMask for current state.
4 On acceptance, build tagged JSON via ProductionSchemaEntry.constructor_tag.
5 Validate tagged JSON vs SchemaRegistry; return success(canonical CKCGen bytes), else
  invalid with first schema diagnostic by canonical field-path order.
```

Display-grammar nonterminal coverage:

```text
T Nonterminal | Tagged JSON schema or enum | Production family
CKCGen | CKCGen | generator object
TypedVar | TypedVar | variable declaration
FiniteVarDomain | FiniteVarDomain | finite-domain enum
ClassPred | ClassPred | class-predicate union
Cmp | Cmp | comparator enum
Premise | Premise | premise union
BuiltinName | BuiltinName | builtin enum
Head | Head | head union
Var, MatchVar, MatchClassVar, OutputVars | Var schema; MatchVar alias of Var; MatchClassVar schema; OutputVars schema | variable-reference families
RoleName, RelationName | Id | identifier terminals
Term | Term | term union
Literal | Literal | literal union
FieldConstraint | FieldConstraint | field-constraint object
MechObsPattern | MechObsPattern | mechanical-observation pattern
PatternObsPattern | PatternObsPattern | pattern-observation pattern
MatchPattern | MatchPattern | match pattern
MatchClassPattern | MatchClassPattern | match-class pattern
LicensePattern | LicensePattern | license pattern
ResolutionTheoremPattern | ResolutionTheoremPattern | resolution pattern
SeqItem, RoleBinding | SeqItem schema; RoleBinding schema | sequence families
RegionExpr | RegionExpr | region-expression union
AnchorOrNode | AnchorOrNode | address union
AxisDirection, AxisStep, AxisRegex | AxisDirection enum; AxisStep schema; AxisRegex schema | path families
CollectBound | CollectBound | collect-bound object
TemplateValue, TemplateBinding, ReadingTemplate, AIRKeyTemplate | TemplateValue union; TemplateBinding schema; ReadingTemplate schema; AIRKeyTemplate schema | template families
PatternObsTemplate, LicenseTemplate, ResolutionTheoremTemplate, ResidualTemplate, AmbiguityTemplate, IncoherenceTemplate, DiagnosticTemplate | exactly the same-named template schema for each nonterminal | head-template families
GlossTemplate | GlossTemplate | gloss-template object
```

`T-GEN-Grammar-Evidence` validates evidence-discovery grammar artifact: every display-grammar nonterminal resolves to one tagged JSON schema, every schema alternative to one production family, every FIRST/FOLLOW set equals the least fixed point above, every parser-state token mask equals artifact's own deterministic valid-next-token relation, `authority = evidence_discovery_only`, `accepted_effect_row = {}`. Certifies constrained-decoding support only. `T-GEN-Static` = sole acceptance authority for CKC-GEN-core artifacts.

Generator leaf forms:

```text
S Var(var_id:Id)

E MatchVar = Var

S OutputVars(vars:List[Var])

E RoleName = Id
E RelationName = Id

S MatchClassVar(var:Var)

E Term = VarTerm | LiteralTerm | FieldTerm | TupleTerm | SetTerm

S VarTerm(var:Var)

S LiteralTerm(literal:Literal)

S FieldTerm(base:Var,path:FeaturePath)

S TupleTerm(items:List[Term])

S SetTerm(items:Set[Term])

E Literal = IdLiteral | TextLiteral | BoolLiteral | UIntLiteral | IntLiteral | RationalLiteral | HashLiteral | EnumLiteral | ReadingRefLiteral | RegionMemberLiteral

S IdLiteral(value:Id)
S TextLiteral(policy:StringPolicy,value:Text<policy>)
S BoolLiteral(value:Bool)
S UIntLiteral(value:UInt)
S IntLiteral(value:Int)
S RationalLiteral(value:Rational)
S HashLiteral(value:Hash)
S EnumLiteral(enum_name:Id,variant:Id)
S ReadingRefLiteral(value:ReadingRef)
S RegionMemberLiteral(value:RegionMember)
```

Pattern schemas:

```text
E PatternOp = has | eq | neq | in | intersects

S FieldConstraint(path:FeaturePath,op:PatternOp,value:Term?)

S MechObsPattern(kind:MechObsKind?,anchor:Term?,source_region:RegionExpr?,field_constraints:List[FieldConstraint],class_pred:ClassPred)

S PatternObsPattern(kind:Id?,relation:Id?,status:Outcome?,support_region:RegionExpr?,role_constraints:Map[RoleName,Term],field_constraints:List[FieldConstraint],class_pred:ClassPred)

S MatchPattern(observation_layer:Id?,match_shape:Id?,source_region:RegionExpr?,capture_constraints:Map[RoleName,Term],field_constraints:List[FieldConstraint],class_pred:ClassPred)

S MatchClassPattern(class_signature_hash:Term?,member_match:Term?,field_constraints:List[FieldConstraint],class_pred:ClassPred)

S LicensePattern(air_type:AirType?,slot_key:Id?,reading_kind:Id?,source_support:RegionExpr?,field_constraints:List[FieldConstraint],class_pred:ClassPred)

S ResolutionTheoremPattern(theorem_kind:ResolutionTheoremKind?,applies_to:Set[Term]?,context:Term?,field_constraints:List[FieldConstraint],class_pred:ClassPred)
```

Sequence, path, and bound schemas:

```text
E SeqPattern = mech:MechObsPattern | pobs:PatternObsPattern | match:MatchPattern

S SeqItem(item_id:Id,pattern:SeqPattern,role:RoleName?,min_gap:UInt,max_gap:UInt)

S RoleBinding(role:RoleName,value:Term)

E RegionExpr = RegionLiteral | RegionOfTerm | RegionClosure | RegionUnion

S RegionLiteral(region_id:RegionId)

S RegionOfTerm(term:Term)

S RegionClosure(seeds:Set[AnchorOrNode])

S RegionUnion(regions:List[RegionExpr])

E AnchorOrNode = AnchorRef | SpanRef | NodeRef | CellRef | BoundAddress

S AnchorRef(anchor_id:Id)
S SpanRef(span_id:Id)
S NodeRef(node_id:Id)
S CellRef(cell_id:Id)
S BoundAddress(term:Term)

E AxisDirection = forward | reverse

S AxisStep(edge_kind:SourceEdgeKind,direction:AxisDirection,min_repeat:UInt,max_repeat:UInt)

S AxisRegex(regex_id:Id,steps:List[AxisStep],max_total_steps:UInt)

S CollectBound(bound_id:Id,max_items:UInt)
```

Head template schemas:

```text
E TemplateValue = TermValue | RegionValue | ReadingTemplateValue | ListTemplateValue | SetTemplateValue | MapTemplateValue

S TermValue(term:Term)
S RegionValue(region:RegionExpr)
S ReadingTemplateValue(reading:ReadingTemplate)
S ListTemplateValue(values:List[TemplateValue])
S SetTemplateValue(values:Set[TemplateValue])
S MapTemplateValue(entries:Map[Id,TemplateValue])

S TemplateBinding(path:FeaturePath,value:TemplateValue)

S ReadingTemplate(reading_kind:Id,field_bindings:List[TemplateBinding])

S AIRKeyTemplate(air_type:AirType,support_region:RegionExpr,slot_key:Term)

S PatternObsTemplate(kind:Id,stage:Int,support_region:RegionExpr,roles:Map[RoleName,Term],relation:Id?,grounding:Set[Term],status:Outcome)

S LicenseTemplate(license_id:Term?,air_key:AIRKeyTemplate,reading:ReadingTemplate,source_support:Set[RegionExpr],proof_roots:Set[Term])

S ResolutionTheoremTemplate(theorem_id:Term?,theorem_kind:ResolutionTheoremKind,applies_to:Set[Term],context:Term,source_support:Set[RegionExpr],proof_roots:Set[Term])

S ResidualTemplate(residual_class:ResidualClass,subject_hash:Term?,source_regions:Set[RegionExpr],diagnostic:Term,proof_roots:Set[Term])

S AmbiguityTemplate(ambiguity_class:AmbiguityClass,alternatives:Set[Term],source_regions:Set[RegionExpr],proof_roots:Set[Term])

S IncoherenceTemplate(incoherence_class:IncoherenceClass,subject_hashes:Set[Term],source_regions:Set[RegionExpr],proof_roots:Set[Term])

S DiagnosticTemplate(code:Id,subject_hash:Term?,source_regions:Set[RegionExpr],text:Term)
```

`GlossTemplate` in `Head` = accepted artifact schema §7.5. M0 gloss generators instantiate literal templates; dynamic gloss rendering by §7.5.

Pattern constraints sort by `(path, op, canonical value bytes)`. Template bindings sort by `path` inside a head template, each path once. Sequence items preserve declared order; maps/sets use §1.5 canonical order.

Template type checking structural:

```text
TermValue valid iff evaluated term type assignable to target field type.
RegionValue valid only for RegionId, Set[RegionId], or source-support aliases.
ReadingTemplateValue valid iff its ReadingTemplate.reading_kind names target Reading schema or one member schema of a target Reading union.
ListTemplateValue valid iff every element valid for list element type and list cardinality bounds hold.
SetTemplateValue/MapTemplateValue use declared element/value type + §1.5 canonical collection rules.
Optional source `T?` → optional target `T?`: valid iff `T` assignable.
Optional source `T?` → required target `T`: valid iff static environment proves source path present.
Presence proved by: a premise field constraint resolving same path (`has`/`eq`/`in`/`intersects`/`count`), an accepted `MechObsKind` optional-field guarantee §4.4, or a schema-required field on source payload.
```

`ReadingTemplateValue` constructs reading payload before assignment. Micro-example: head field `NormReading.temporal:List[TemporalReading]` accepts `ListTemplateValue([ReadingTemplateValue(TemporalReading{temporal_kind=prompt,value="速やかに",raw_anchor_id=<anchor>} )])`, rejects raw `TermValue` whose term evaluates to `MechObsPayload`.

```text
E EvalScalar = id:Id | hash:Hash | uint:UInt | int:Int | rational:Rational | text:TextLiteral | enum:EnumLiteral

E EvalValue = scalar:EvalScalar | literal:Literal | artifact_ref:Hash | tuple:List[EvalValue] | set:Set[EvalValue] | region_ref:RegionId | bool:Bool

E TermEval = value:EvalValue | unsupported:DiagnosticRef | residual:Set[Hash] | ambiguity:Set[Hash] | incoherence:Set[Hash] | invalid:Set[Hash]
```

`eval_term(term, environment) -> TermEval` total:

```text
VarTerm: value(environment value) for declared var, else unsupported if unbound.
LiteralTerm: value(literal).
FieldTerm: resolve base var, traverse FeaturePath over schema-validated payload;
  unsupported on absent required fields or type mismatch.
TupleTerm: eval items declared order; return first non-value TermEval status by
  §1.7 primary-status order, else value(tuple).
SetTerm: eval items canonical term-byte order; return first non-value TermEval status by
  §1.7 primary-status order; else canonicalize as set, call `HandleBoundOverflow` when
  applicable SchemaBoundManifest row overflows and return that exact overflow status, else
  value(canonical set).
```

```text
E ClassPredEval = bool:Bool | unsupported:DiagnosticRef
```

`eval_class_pred(pred, payload) -> ClassPredEval` total:

```text
true: true.
false: false.
has path: true iff FeaturePath resolves to present field.
eq path literal: true iff path resolves and canonical value bytes = literal bytes.
neq path literal: true iff path absent or canonical value bytes differ.
in path set: true iff resolved value ∈ canonical set.
intersects path set: true iff resolved finite collection intersection nonempty.
count path Cmp UInt: true iff resolved finite collection cardinality satisfies comparator.
and list: empty → true; else eval declared order, return false at first false;
  unsupported propagates only when no earlier false decides.
or list: empty → false; else eval declared order, return true at first true;
  unsupported propagates only when no earlier true decides.
not p: boolean negation of p; unsupported propagates.
```

```text
S Environment(bindings:Map[Var,EvalValue])
```

```text
E PremiseEval = environments:Set[Environment] | unsupported:DiagnosticRef | residual:Set[Hash] | ambiguity:Set[Hash] | incoherence:Set[Hash] | invalid:Set[Hash]
```

`eval_premise(premise, environment, snapshot) -> PremiseEval` total. Boolean-true premise → `environments({environment})`; false → `environments({})`; unsupported → `unsupported`. Relation-valued premises: enumerate relevant finite snapshot relation by canonical order, extend `environment` with each candidate binding, filter by pattern schema + `eval_class_pred`, return canonical set of surviving environments. `member` reads `ClassMember`; `capture` reads `Match.captures`; `rel` resolves finite named relation from `TerminologyClosure`/`SemanticPolicySet`/`FiniteFixtureManifest`, unknown `RelationName` → `unsupported`. Equality/inequality/membership premises eval terms first; non-value `TermEval` maps to same `PremiseEval` status, value results compare canonical bytes per true/false mapping above. `builtin` dispatches by `BuiltinName`, binds `OutputVars` declared order; builtin unsupported/residual/ambiguity/incoherence/invalid map to same `PremiseEval` status. `collect`/`empty`/`seq`/`bounded-path` use bounded algorithms in this section, propagate their explicit residual/ambiguity/incoherence/invalid/unsupported result.

`T-GEN-Static` validates these schemas by total procedure:

```text
1 Parse display syntax to tagged JSON via `ParseCKCGen`, canonicalize.
2 Validate every grammar object vs its schema.
3 Resolve each FeaturePath vs declared domain or head target schema.
4 Check each variable reference declared and every head variable bound by a positive premise.
5 Check each RoleName, RelationName, enum variant, builtin name, production family, schema id resolves to exactly one definition.
6 Check each FieldConstraint value has type required by its path+operator, and each TemplateValue assignable to target head field type under structural rules above.
7 Check every RegionExpr, sequence, disjunction expansion, AxisRegex has finite static bound in SchemaBoundManifest or local bound object.
8 Check every CollectBound.max_items finite; on any SchemaCollectionBound overflow during static check, call `HandleBoundOverflow` and return its primary status.
9 Check stage stratification, builtin support, accepted-effect purity, source grounding.
10 When GeneratorGrammarArtifact emitted, run `T-GEN-Grammar-Evidence`, record result as evidence-discovery diagnostics; CKC-GEN acceptance depends on steps 1-9.
11 Emit canonical CKCGen bytes on success or typed Diagnostic on primary non-success outcome.
```

`empty LicensePattern Q`: enumerate every environment satisfying premise list `Q` over lower-stage snapshots, same left-to-right environment extension as ordinary premises. For each, instantiate `LicensePattern` with that environment, query against completed lower-stage license relation. True iff total canonical set of matching licenses over all such environments has cardinality zero; false when nonempty; propagates primary non-success outcome from any premise in `Q`. Makes correlated absence queries explicit. Example: `empty LicensePattern{air_type=air.norm,slot_key=renal_adjustment} [(member c class_renal)]` succeeds only when no lower-stage norm license for `renal_adjustment` exists in any environment that is member of `class_renal`.

`collect` sorts matches by canonical payload bytes. On `CollectBound.max_items` overflow, M0 local dispatch exact: emit `Residual(class=unsupported_construction)`, emit `Diagnostic(code=collect_bound_overflow)`, return `residual`; collected value stays unbound.

Region, sequence, and path evaluation:

```text
E RegionEval = region:SourceRegion | unsupported:DiagnosticRef | residual:Set[Hash] | ambiguity:Set[Hash] | incoherence:Set[Hash] | invalid:Set[Hash]

eval_region(region_expr, environment, snapshot) -> RegionEval is total.
eval_region(RegionLiteral r): region(r) when it resolves, else unsupported.
eval_region(RegionOfTerm t): resolve t to artifact with source_support, source_region_id, or source-support alias, return closed region, else unsupported. Non-value TermEval → corresponding RegionEval status.
eval_region(RegionClosure seeds): check `RegionClosure.seeds` SchemaBoundManifest row, call `HandleBoundOverflow` on overflow and return its status, else call source_region_closure on resolved seeds and map its residual to RegionEval.residual.
eval_region(RegionUnion regions): check `RegionUnion.regions` SchemaBoundManifest row, call `HandleBoundOverflow` on overflow and return its status, else call source_region_closure on union of evaluated region members and map its residual to RegionEval.residual.

seq(region, items, bindings):
  1 Require |items|,|bindings| satisfy SchemaBoundManifest; on overflow call `HandleBoundOverflow` for corresponding `SeqItem`/`RoleBinding` bound and return its primary status.
  2 Evaluate region to closed SourceRegion.
  3 For each SeqItem declared order, enumerate observations inside region matching item.pattern,
     sorted by source_order_key then canonical payload bytes.
  4 Enumerate finite Cartesian product of candidate observations in declared item order.
  5 Require source_order_key gap between consecutive selected observations within
     [min_gap,max_gap] for later SeqItem.
  6 Bind each item.role to selected observation when role present.
  7 Check every RoleBinding value vs bound role value.
  8 Return true for first canonical satisfying sequence; false when enumeration exhausted;
     unsupported only for unresolved regions, roles, or type errors.

bounded-path(axis_regex, start, end, max_len):
  1 Resolve start,end to SourceGraph node, span, cell, or anchor addresses.
  2 Require every AxisStep.max_repeat, AxisRegex.max_total_steps, max_len finite and
     max_total_steps <= max_len when both present; if AxisRegex.steps or any path-candidate collection overflows its SchemaBoundManifest row, call `HandleBoundOverflow` and return its primary status.
  3 Enumerate paths through SourceEdge values matching AxisStep.edge_kind and AxisStep.direction,
     sorted by edge source_order_key then edge_id.
  4 Enforce each AxisStep repeat bounds, AxisRegex.max_total_steps, max_len.
  5 Return true for first canonical path reaching end; false when finite search exhausted;
     unsupported only for unresolved addresses or invalid bounds.
```

Builtin definitions, total unsupported and overflow conditions:

```text
E BuiltinEval = outputs:List[EvalValue] | bool:Bool | unsupported:DiagnosticRef | residual:Set[Hash] | ambiguity:Set[Hash] | incoherence:Set[Hash] | invalid:Set[Hash]

support_of(x): canonical source-support projection for x; unsupported when x has no
  source-support field or alias.
support_union(S): eval every region in S, return source_region_closure over union of
  members; unsupported when any member not a region.
within_support(a,b): true iff a.closed_members ⊆ b.closed_members; unsupported for unresolved regions.
source_region_closure: §4.3.
canonical_set: §1.5 set canonicalization; when target set has SchemaBoundManifest row and candidate cardinality exceeds it, return exact `HandleBoundOverflow` outcome for that row.
proof_visible_signature: §7.2.
unit_normalize: replace unit concepts by terminology representatives linked by unit_equivalent;
  unequal unlinked units → unsupported.
normalize_context and ctx_compatible: §8.1.
normalize_action and same_normalized_action: §8.2.
consequents_compatible: §8.3.
theorem_minimize and dependency_minimize: §8.7.
```

Builtins pure, total over supported inputs, byte-stable, I/O-free. Unsupported inputs → `BuiltinEval.unsupported` with diagnostic payload code = builtin name + `_unsupported`. Any builtin body calling `HandleBoundOverflow` returns exact overflow status in `BuiltinEval`; no overflow outcome escapes declared codomain. Builtin output variables bound in order declared by `OutputVars`; arity mismatch = static diagnostic.

### 6.3 Readings and licenses

GRADE/Minds recommendation metadata = proof-visible annotation. M0 theorem predicates consume `direction`; reports/glosses consume recommendation metadata when present.

```text
E RecommendationStrength = strong | weak | conditional | good_practice | ungraded
E EvidenceCertainty = high | moderate | low | very_low | not_assessed

S RecommendationMetadata(clinical_question_id:Id?,recommendation_id:Id?,strength:RecommendationStrength?,evidence_certainty:EvidenceCertainty?,pico_region_ids:Set[RegionId],etd_region_ids:Set[RegionId],evidence_table_region_ids:Set[RegionId])
```

```text
E Reading = TermReading | ConditionReading | ActionReading | CueReading | QuantityReading | TemporalReading | NormReading | FactualReading | TableReading

S ReadingRef(reading_digest:Hash)

S TermReading(concept_id:Id,binding_status:BindingStatus,surface_anchor_id:Id)

S ConditionReading(predicate_id:Id,args:List[ReadingRef],polarity:Bool)

S ActionReading(action_kind:Id,target:ReadingRef,slots:Map[Id,ReadingRef],surface_anchor_id:Id)

S CueReading(cue_kind:Id,cue_value:Id,surface_anchor_id:Id)

S QuantityReading(comparator:Cmp,value:Rational,unit_id:Id,raw_anchor_id:Id)

S TemporalReading(temporal_kind:Id,value:Text<semantic_ja>,raw_anchor_id:Id)

S NormReading(context:ContextExpr,direction:Direction,action:ActionReading,temporal:List[TemporalReading],original_modality_phrase_ja:Text<semantic_ja>?,recommendation_metadata:RecommendationMetadata?)

S FactualReading(subject:ReadingRef?,consequent:FactualConsequent,context:ContextExpr,strict:Bool)

S TableReading(table_id:Id,input_variable_id:Id,unit_id:Id?,rows:List[TableRowReading])

S TableRowReading(row_id:Id,guard:ContextExpr,output_slot_id:Id,output_value:ReadingRef,source_region_id:RegionId)

S AIRKey(air_type:AirType,support_region_id:RegionId,slot_key:Id)

S License(license_id:Id,air_key:AIRKey,reading:Reading,generator_hash:Hash,source_support:Set[RegionId],proof_roots:Set[ProofId],accepted_effect_row:Set[Effect])
```

License = one candidate reading per AIR key. `slot_key` = generator-declared semantic key for alternatives of one fact; proof-visible. Distinct table rows live in one `TableReading`; one table → one AIR key.

`ReadingRef.reading_digest = sha256(canonical_payload_bytes(reading))` over the referenced `Reading`; readings inline in licenses/policies/witnesses, so a ref names reading bytes, not an envelope. Resolve a `ReadingRef` against the finite reading set in scope at the consuming algorithm.

`NormReading` modality = `(direction, original_modality_phrase_ja)`. Theorem predicates consume `direction`; gloss rendering consumes both when a template requests.

### 6.4 Proposal records and discharge

Proposal mechanisms (LLMs, retrieval, prompt programs, structured/grammar-constrained decoding, verifier-guided repair, human fixtures) → proposal records with trace authority. Accepted semantic authority begins only at `DischargeProposal` discharged artifacts.

```text
S ProposalProvenanceManifest(manifest_id:Id,generator_family:ProposalGeneratorFamily,model_or_tool_id:Id,model_version:Text<identifier_ascii>?,prompt_template_hash:Hash?,structured_output_schema_hash:Hash?,decoding_policy_hash:Hash?,tool_manifest_hashes:Set[Hash],input_context_hashes:Set[Hash],output_bytes_hash:Hash,authority:Authority,replay_manifest_hash:Hash)

E ProposalGeneratorFamily = closed_frontier_llm | domain_medical_model | proof_model | constrained_decoder | tool_calling_agent | self_consistency_sampler | rag_autoformalizer | critique_judge | program_aided_lm | verifier_guided_repair | adapter_finetune | world_model | human_fixture | other

S ProposalRecord(proposal_id:Id,proposed_subject_digest:Hash?,proposal_kind:Id,proposal_bytes_hash:Hash,proposal_provenance_hashes:Set[Hash],evidence_hashes:Set[Hash],proposal_effect_row:Set[Effect])
```

Proposal provenance authority invariant: `ProposalProvenanceManifest.authority = evidence_discovery_only`. Model/prompt/structured-output/function-calling/tool/verifier-feedback/self-consistency/adapter/world-model details affect accepted semantics only via deterministic discharge or a §3.3 gate.

Pre-acceptance digest convention: §6.4 `*_digest` = `sha256(canonical_payload_bytes(payload))` over the canonical candidate payload that `DischargeProposal` steps 1-4 produce from recorded candidate bytes. `ProposalRecord.proposed_subject_digest` (absent exactly when steps 1-4 reject bytes), `ReviewerRecord.reviewed_subject_digest`, `MaterializedConsequenceManifest.candidate_digest` digest that candidate payload. `emitted_payload_digests` digests each payload materialized at step 6. `CounterexampleSuite.required_output_digests`/`forbidden_output_digests` digest suite-authored expected payloads, compared vs `emitted_payload_digests` at step 9. Step-12 accepted artifact: envelope `artifact_hash` = `candidate_digest`.

Retrieval, reranking, graph traversal, query decomposition, citation-grounded generation enter M0 only via proposal traces with `authority = evidence_discovery_only`; consumed by `DischargeProposal` as evidence hashes and by §3.3 research gates for retrieval-quality claims.

```text
S RetrievalProposalTrace(trace_id:Id,query_hash:Hash,query_decomposition_hash:Hash?,segment_granularity:RetrievalSegmentGranularity,sparse_retriever_family:SparseRetrieverFamily?,sparse_retriever_manifest_hash:Hash?,dense_retriever_family:DenseRetrieverFamily?,dense_retriever_manifest_hash:Hash?,late_interaction_family:LateInteractionFamily?,late_interaction_manifest_hash:Hash?,graph_retrieval_manifest_hash:Hash?,fusion_policy_family:FusionPolicyFamily?,fusion_policy_hash:Hash?,reranker_family:RerankerFamily?,reranker_manifest_hash:Hash?,japanese_analyzer_family:JapaneseAnalyzerFamily?,candidate_region_ids:List[RegionId],cited_region_ids:Set[RegionId],score_record_hashes:Set[Hash],index_fingerprint_hashes:Set[Hash],authority:Authority,accepted_effect_row:Set[Effect])

E RetrievalSegmentGranularity = source_span | clinical_question | recommendation | pico_field | evidence_table_row | table_cell

E SparseRetrieverFamily = bm25 | bm25_plus | bm25f | lucene_bm25 | other

E DenseRetrieverFamily = dpr | ance | contriever | bge_m3 | multilingual_e5 | jina | medcpt | jmedroberta | generic_biencoder | other

E LateInteractionFamily = colbert | jacolbert | bge_m3_multivector | plaid | other

E FusionPolicyFamily = rrf_k60 | rrf_other | weighted_sum | relative_score_fusion | none | other

E RerankerFamily = cross_encoder | bge_reranker | cohere_rerank | medcpt_reranker | llm_judge | none | other

E JapaneseAnalyzerFamily = kuromoji | sudachi | mecab_ipadic | mecab_unidic | fugashi | sentencepiece | xlm_roberta_tokenizer | other
```

Retrieval trace authority invariant: `RetrievalProposalTrace.authority = evidence_discovery_only`. `score_record_hashes` = `sha256(exact_recorded_bytes)` of score records from named retriever/fusion/reranker stages; `*_manifest_hash` fields name the supplying manifests.

`T-Retrieval-Proposal-Trace`: every candidate/cited region resolves in SourceGraph; every named retrieval/analyzer/reranker family resolves to local enums above; every index fingerprint replayable; every fusion/reranking score is trace metadata, not accepted semantics; every retrieval-quality claim carries `G-RET-PARITY` evidence.

```text
E AdmissionDecision = accept | ambiguity | residual | escalate | reject

S AdmissionContext(context_id:Id,frozen_fixture_hashes:Set[Hash],schema_registry_hash:Hash,accepted_base_hash:Hash,counterexample_suite_hash:Hash?,admission_record_hash:Hash,reviewer_record_hashes:Set[Hash])

S ReviewerRecord(reviewer_record_id:Id,reviewer_role:ReviewerRole,reviewer_id:Id,reviewed_subject_digest:Hash,decision:AdmissionDecision,rationale:Text<diagnostic_text>,source_region_ids:Set[RegionId],logical_time:UInt,authority:Authority)

E ReviewerRole = formalist | clinician_reviewer | curator | automated_checker

E ProofParentPolicy = admitted_artifact_only

S CounterexampleSuite(suite_id:Id,subject_kind:Id,fixture_input_hashes:Set[Hash],required_output_digests:Set[Hash],forbidden_output_digests:Set[Hash],max_materialized_payloads:UInt,expected_residual_classes:Set[ResidualClass])

S MaterializedConsequenceManifest(candidate_digest:Hash,fixture_input_hashes:Set[Hash],emitted_payload_digests:Set[Hash],emitted_residual_classes:Set[ResidualClass],emitted_incoherence_classes:Set[IncoherenceClass],closure_bound_certificate_hash:Hash,proof_node_hashes:Set[Hash],status:Outcome)

S EffectDischargeRecord(proposal_hash:Hash,discharged_artifact_hash:Hash?,proposal_effect_row:Set[Effect],accepted_effect_row:Set[Effect],deterministic_check_hashes:Set[Hash],materialized_consequence_manifest_hash:Hash,admission_record_hash:Hash,proof_parent_policy:ProofParentPolicy)

S AdmissionRecord(subject_hash:Hash,subject_kind:Id,decision:AdmissionDecision,reviewer_record_hashes:Set[Hash],materialized_consequence_manifest_hash:Hash,counterexample_suite_hash:Hash?,admitted_effect_row:Set[Effect],admitted_at_logical_time:UInt)

S AcceptedGeneratorBase(base_id:Id,generator_hashes:Set[Hash],admission_record_hashes:Set[Hash],accepted_effect_row:Set[Effect])
```

`AcceptedGeneratorBase` + admitted `TerminologyResourceSet` + `SemanticPolicySet` = reusable-component set whose minimality vs corpus coverage is CKC's long-run optimization target (intro, §10, §13). M0: compactness reported descriptively only; reuse/compression/MDL verdicts gated by `G-MDL`, `G-EMIN`.

Admission-decision consumers:

```text
accept:
  permits accepted artifact construction when deterministic checks and effect discharge also pass.

ambiguity:
  records reviewer-recognized non-uniqueness and preserves proposal artifacts for trace; accepted
  semantic artifacts are built only from a later accepted proposal.

residual:
  records a reviewer-recognized unsupported or incomplete construction and preserves the emitted
  Residual payloads from materialization.

escalate:
  records that a gated or human-governance path is required before acceptance.

reject:
  records a negative admission decision and preserves deterministic diagnostics as the durable output.
```

Reviewer authority invariant: `ReviewerRecord.authority = admitted_authority`. `ReviewerRole` report-visible only; all variants consumed generically by admission trace rendering, accepted semantics unchanged. `reviewer_id` names reviewer/automated checker; `rationale` = justification inline under `diagnostic_text`.

`DischargeProposal(proposal, candidate_bytes, admission_context) -> OperationResult[EffectDischargeRecord]`:

```text
1 Decode candidate_bytes according to proposal.proposal_kind.
2 Apply declared string policies and canonical JSON serialization.
3 Macro-expand authoring syntax into CKC-GEN-core JSON when needed.
4 Validate against SchemaRegistry.
5 Run static checks: type, stage, finite-domain, head-safety, effect-row, grounding, and builtin-support.
6 Materialize deterministic consequences over admission_context.frozen_fixture_hashes.
7 Build MaterializedConsequenceManifest.
8 If proposal.proposal_kind is one of {CKCGen, TerminologyResourceSet, SemanticPolicySet, GlossTemplate}
  and admission_context.counterexample_suite_hash is absent, set status=residual and emit
  Residual(class=missing_counterexample_suite).
9 If a suite is present, compare emitted_payload_digests with required_output_digests and
  forbidden_output_digests, and require emitted_residual_classes = expected_residual_classes;
  required missing, forbidden present, or residual-class mismatch sets status=incoherence.
10 If emitted payload count exceeds CounterexampleSuite.max_materialized_payloads, use the local
  `counterexample_suite_bound_overflow` dispatch: set status=residual, emit
  Residual(class=unsupported_construction), emit Diagnostic(code=counterexample_suite_bound_overflow),
  and accept no candidate artifact from this proposal.
11 Read the recorded AdmissionDecision through admission_context.admission_record_hash.
12 Construct an accepted artifact exactly when static checks pass, materialization status is ok,
  suite comparison passes, AdmissionDecision=accept, and accepted_effect_row can be {}.
13 Store proposal artifacts only in trace and AdmissionRecord; accepted proof parents reference
  accepted artifacts.
14 Emit EffectDischargeRecord and return the §1.7 primary status over emitted artifacts; any SchemaCollectionBound overflow in the candidate, manifest, or record calls `HandleBoundOverflow` and returns the exact overflow status.
```

## 7. Closure, licensed readings, AIRCore, Normal Form, and glosses

### 7.1 Closure algorithm and bounds

```text
S ClosureInput(source_graph_hash:Hash,mech_obs_hashes:Set[Hash],accepted_generator_base_hash:Hash,terminology_resource_set_hash:Hash,semantic_policy_set_hash:Hash,finite_fixture_manifest_hash:Hash?,schema_registry_hash:Hash,schema_bound_manifest_hash:Hash)

S ClosureOutput(closure_input_hash:Hash,pattern_obs_hashes:Set[Hash],match_hashes:Set[Hash],match_class_hashes:Set[Hash],class_member_hashes:Set[Hash],terminology_closure_hash:Hash,license_hashes:Set[Hash],resolution_theorem_hashes:Set[Hash],licensed_reading_set_hashes:Set[Hash],air_core_hashes:Set[Hash],nf_hashes:Set[Hash],gloss_template_hashes:Set[Hash],gloss_view_hashes:Set[Hash],witness_context_hashes:Set[Hash],conflict_hashes:Set[Hash],factual_inconsistency_hashes:Set[Hash],residual_hashes:Set[Hash],ambiguity_hashes:Set[Hash],incoherence_hashes:Set[Hash],diagnostic_hashes:Set[Hash],verifier_witness_hashes:Set[Hash],symbol_source_map_hashes:Set[Hash],constraint_core_witness_hashes:Set[Hash],repair_set_search_trace_hashes:Set[Hash],certificate_hashes:Set[Hash],claim_record_hashes:Set[Hash],report_trace_index_hashes:Set[Hash],claim_tier_summary_hashes:Set[Hash],wording_gate_record_hashes:Set[Hash],review_report_hashes:Set[Hash],replay_manifest_hashes:Set[Hash],replay_identity_hashes:Set[Hash],proof_node_hashes:Set[Hash],proof_dag_hash:Hash,closure_bound_certificate_hash:Hash)
```

M0 closure = stratified finite materialization. `ClosureOutput` enumerates every CloseM0 accepted artifact class, stages -10..90, except `TerminologyResourceSet`; admitted set + stage-10 fragments excluded as a class, reachable via `ClosureInput`, `TerminologyClosure.terminology_resource_set_hash`, Appendix A.10 (demo stage-10 fragment set empty). Also excludes admitted stage -40 inputs (`ReportQuestionTemplate`, `AcceptedGeneratorBase`, `SemanticPolicySet`), runtime/schema/environment/fixture-control inputs, and manifest-stored row types. Other excluded artifacts reachable via `ClosureInput`, manifest fields, or A.10.

```text
1 Initialize R[-30] from SourceGraph payloads and R[-20] from MechObsPayload payloads.
2 Initialize ProofDAG with MECH_OBS proof nodes.
3 For each stage in §3.2 order:
     -10: evaluate obs_pattern generators over lower-stage relations.
       0: build Match, MatchClass, and ClassMember.
      10: evaluate term_resource generators for TerminologyResourceSet fragments.
      20: evaluate BuildTerminologyClosure over stage-10 TerminologyResourceSet artifacts; keep the admitted SemanticPolicySet from ClosureInput fixed and validate all policy lookups against its non-quarantined rows.
      30: evaluate sem_rule generators for term/cue/quantity/temporal/condition/action licenses.
      40: evaluate sem_rule generators for norm licenses, factual licenses, and resolution theorems.
      50: build demanded LicensedReadingSet, AIRCoreRecord, and CKCNormalForm.
      60: construct M0 theorem candidates by §8.
      70: render deterministic glosses.
      80: emit residuals, ambiguities, incoherences, and coverage diagnostics.
      90: run the kernel finite checker, issue certificates, reports, and replay checks.
4 For each generator, enumerate variable environments in declaration order over finite canonical domains.
5 Evaluate premises left-to-right over frozen lower-stage relation snapshots.
6 Sort satisfying environments by canonical environment key.
7 Instantiate one head per environment, canonicalize it, collapse duplicate payloads by artifact_hash,
  and retain all proof roots in canonical set order.
8 At every output collection bound recorded in SchemaBoundManifest, call `HandleBoundOverflow`;
  at every generator, stage, and kernel-builder bound recorded only in ClosureBoundCertificate,
  call the local `closure_bound_overflow` dispatch before accepting a payload whose bound is exceeded.
```

Same-stage recursion = add intermediate stage. Stratified negation: `empty` only over fully materialized lower-stage relations.

```text
S ClosureBoundCertificate(closure_input_hash:Hash,finite_domain_cardinalities:Map[Id,UInt],generator_env_bounds:Map[Hash,UInt],generator_materialized_counts:Map[Hash,UInt],collect_bounds:Map[Hash,UInt],sequence_bounds:Map[Hash,UInt],axis_path_bounds:Map[Hash,UInt],context_clause_bounds:Map[Hash,UInt],stage_bounds:Map[Int,UInt],kernel_builder_bounds:Map[Id,UInt],total_materialized_payloads:UInt)
```

Certificate restates only bounds recorded nowhere else; schema collection bounds ride `closure_input_hash` via `ClosureInput.schema_bound_manifest_hash`. Bound-map keys: `generator_env_bounds`, `generator_materialized_counts` keyed by accepted generator envelope `artifact_hash`; `collect_bounds`, `sequence_bounds`, `axis_path_bounds`, `context_clause_bounds` keyed by `sha256(canonical_payload_bytes(form))` over the bounded collect/seq/bounded-path/context-clause form.

`closure_bound_overflow` = local dispatch for `ClosureBoundCertificate` map bounds lacking `BoundOverflowDisposition`: emit `Residual(class=unsupported_construction)`, emit `Diagnostic(code=closure_bound_overflow)`, include overflowing bound key in canonical diagnostic text, reject overflowing materialized payload from accepted output.

Termination argument:

```text
1 SourceGraph, MechObsPayload, AcceptedGeneratorBase, TerminologyResourceSet, and SemanticPolicySet are finite accepted inputs.
2 Every generator variable ranges over a finite accepted domain.
3 Every path, sequence gap, and collect form has a finite bound.
4 Every license and resolution-theorem premise reads lower stages only.
5 Each generator emits at most one head per finite environment.
6 Kernel builders operate over finite relation sets and finite demand sets.
7 Duplicate payload collapse decreases or preserves cardinality.
8 The stage list is finite.
```

`T-Closure-Termination` recomputes all bounds and checks materialized counts.

### 7.2 Proof DAG, matches, and quotient classes

```text
E ProofRule = SOURCE | MECH_OBS | GEN_OBS | MATCH | CLASS | GEN_SEM | AIR_FSET | NF_PROJ | BRIDGE | GLOSS | RESIDUAL | FINITE_CHECK | REPORT | REPLAY | CERT

E JudgmentKind = SourceGraph | SourceRegion | MechObsPayload | PatternObs | Match | MatchClass | Member | License | Resolution | LicensedReadingSet | AIRCore | NF | WitnessContext | Conflict | FactualInconsistency | Residual | Ambiguity | Incoherence | Diagnostic | GlossTemplate | GlossView | VerifierWitness | SymbolSourceMap | ConstraintCoreWitness | RepairSetSearchTrace | Certificate | ClaimRecord | ReportTraceIndex | ClaimTierSummary | WordingGateRecord | ReviewReport | ReplayManifest | ReplayIdentityCheck

S ProofNode(proof_id:Id,rule:ProofRule,conclusion_kind:JudgmentKind,conclusion_hash:Hash,source_graph_hash:Hash,generator_hash:Hash?,stage:Int?,environment_digest:Hash?,support_digest:Hash?,payload_digest:Hash,premise_proof_ids:Set[ProofId],premise_artifact_hashes:Set[Hash])

S ProofDAG(proof_dag_id:Id,proof_nodes:Set[ProofNode],roots:Set[ProofId])

S PatternObs(pattern_id:Id,kind:Id,stage:Int,support_region_id:RegionId,roles:Map[Id,Id],relation:Id?,grounding:Set[Id],generator_hash:Hash,status:Outcome,proof_roots:Set[ProofId])

S Match(match_id:Id,source_region_id:RegionId,observation_layer:Id,match_shape:Id,captures:Map[Id,Id],observation_hashes:Set[Hash],pattern_proof_hashes:Set[Hash])

S MatchClass(match_class_id:Id,class_signature_hash:Hash,member_match_ids:Set[Id])

S ClassMember(match_id:Id,match_class_id:Id)
```

`ProofNode` digest fields (§1.2 convention): `payload_digest = sha256(canonical_payload_bytes(conclusion_payload))` over the node's concluded payload, where `conclusion_hash` = that payload's accepted-envelope `artifact_hash` when one exists, else = `payload_digest` for inline-only conclusion; `support_digest = sha256(canonical_payload_bytes(node_source_support))` over the proof-DAG-inherited `Set[RegionId]` the node carries (canonical source-support projection §1.2 routes through `proof_structure` payloads); `environment_digest = sha256(canonical_payload_bytes(satisfying_environment))` over the `Environment` (§6.2) that fired the generator. `environment_digest` present exactly for `GEN_OBS`, `GEN_SEM`; `support_digest` absent only for rules carrying no source support (`REPLAY`, `CERT`).

Proof-rule consumers and side conditions:

```text
T Rule | Emitted for | Checker side condition
SOURCE | SourceGraph, SourceRegion | Source edition, permissions, extraction manifest, and closure certificate resolve; source-region closure replays.
MECH_OBS | MechObsPayload | Observation is reproduced by ObserveMech from SourceGraph, AnalyzerManifest, and MechanicalLexicon.
GEN_OBS | PatternObs | Generator is admitted, stage=-10, environment satisfies premises, and support closes.
MATCH | Match | Captures resolve to observations and proof_visible_signature recomputes.
CLASS | MatchClass, ClassMember | Equivalence class is exactly the quotient of matches by proof-visible signature.
GEN_SEM | License, ResolutionTheorem | Generator is admitted, stage is valid, premises read lower-stage snapshots, head instantiation canonicalizes, and source grounding holds.
AIR_FSET | LicensedReadingSet, AIRCoreRecord | Omega set equals exactly the accepted licenses for the demanded AIR key and finite-set identity status is correct.
NF_PROJ | CKCNormalForm | NF payload is the deterministic projection of exactly one ok AIRCore reading or compiler metadata builder and NF(NF(x))=NF(x).
BRIDGE | WitnessContext, ConflictTheorem, FactualInconsistencyTheorem, RepairSetSearchTrace, bridge diagnostics | §8 predicate is re-evaluated, witnesses/minimal cores resolve, and missing policy/evidence becomes residual rather than theorem.
GLOSS | GlossTemplate, GlossView | Template is unique for (nf_kind, lang, renderer_id), every slot resolves, and rendering recomputes byte-identically.
RESIDUAL | Residual, Ambiguity, Incoherence, Diagnostic | The named class is emitted by its canonical producer and source/support/proof aliases resolve.
FINITE_CHECK | VerifierWitness, SymbolSourceMap, ConstraintCoreWitness | Kernel finite checker input resolves, symbol-source projections resolve, and the stored result equals recomputation; each constraint core is deletion-minimal under §8.1 order.
REPORT | ClaimRecord, ReportTraceIndex, ClaimTierSummary, WordingGateRecord, ReviewReport | Report items, trace rows, claim tiers, wording-gate decisions, and report payload are exactly the sorted projection of valid theorems, residuals, ambiguities, incoherences, replay failures, and permission diagnostics.
REPLAY | ReplayManifest, ReplayIdentityCheck | Replay manifest inputs resolve and ReplayIdentity in §1.6 recomputes the stored outcome.
CERT | Certificate | Certificate-class obligation in §9.2 verifies for the subject.
```

Each `JudgmentKind` variant = conclusion kind of the corresponding semantically derived / verifier / report / replay / certificate artifact schema named in §3.1, §3.2. Source-only, schema-control, replay-control, admission-control, environment-control, evidence-discovery artifacts have proof obligations at their named acceptance gates in §11.3, referenced from ProofNodes via `premise_artifact_hashes` when supporting a semantic conclusion.

Proof checker verifies rule side conditions by re-reading accepted inputs and proof-visible payload fields; independent of proposal traces. Node invalid when: any premise proof missing, OR a premise artifact hash ≠ its accepted envelope, OR conclusion hash ≠ recomputed canonical payload bytes.

Node identity = `proof_id` inside DAG payload; DAG identity = envelope `artifact_hash` (§1.2). Checker identity = envelope provenance via `producer_manifest_hash`; per-subject check outcomes = stage-90 `VerifierWitness` artifacts. Reverse dependency index = in-memory inversion of `premise_proof_ids`; `proof_nodes` = its single canonical representation.

`proof_visible_signature(Match m)` = canonical bytes of:

```text
match_shape;
observation_layer;
capture role names and captured object kinds;
normalized surfaces from captured anchors;
terminology candidate representative IDs;
quantity comparators, rational values, and unit representatives;
temporal surface kind IDs;
source section path IDs;
bounded path summaries.
```

`BuildMatches` sorts candidate matches by `source_order_key`, then `proof_visible_signature`, then canonical payload bytes. `MatchClass` = quotient of `Match` by proof-visible signature equality. `T-Quotient-Invariance` checks proof-invisible layout perturbations preserve class signatures.

### 7.3 Licensed reading sets and AIRCore

Demanded AIR keys are exactly:

```text
1 every AIRKey emitted by an accepted License;
2 every AIRKey named by a PatternObs with status=residual and relation=coverage_target.
```

For each demanded key `K`:

```text
Omega(K) = { l.reading | l is an accepted License, l.air_key = K, and l.proof_roots check in ProofDAG }
LicensedReadingSet(K) = canonical_set(Omega(K))
```

```text
S LicensedReadingSet(lrs_id:Id,air_key:AIRKey,readings:Set[Reading],license_hashes:Set[Hash],proof_roots:Set[ProofId])

E AIRDomainKind = finite_set_identity

S AIRCoreRecord(air_core_id:Id,air_key:AIRKey,readings:Set[Reading],domain_kind:AIRDomainKind,status:Outcome,residual_hash:Hash?,ambiguity_hash:Hash?,proof_roots:Set[ProofId])
```

AIRCore finite-set identity algorithm:

```text
1 Let L = LicensedReadingSet.readings.
2 If |L| = 0, emit AIRCoreRecord(status=residual) and Residual(class=no_license).
3 If |L| = 1, emit AIRCoreRecord(status=ok).
4 If |L| > 1, emit AIRCoreRecord(status=ambiguity) and Ambiguity(class=multiple_readings).
```

M0 AIR `domain_kind` = `finite_set_identity`. Non-identity AIR domains require `G-AIR-FULL`.

### 7.4 CKC Normal Form

```text
S CKCNormalForm(nf_id:Id,nf_kind:NFKind,nf_payload:NFPayload,source_support:Set[RegionId],parent_air_core_hashes:Set[Hash],diagnostics:Set[Hash],semantic_digest:Hash,proof_roots:Set[ProofId])

E NFKind = norm | factual_rule | decision_table | metadata_claim

E NFPayload = NFNorm | NFFactualRule | NFDecisionTable | NFMetadataClaim

S NFNorm(source_class:SourceClass,context:ContextExpr,direction:Direction,action:ActionReading,temporal:List[TemporalReading],original_modality_phrase_ja:Text<semantic_ja>?,recommendation_metadata:RecommendationMetadata?)

S NFFactualRule(source_class:SourceClass,context:ContextExpr,consequent:FactualConsequent,strict:Bool)

S NFDecisionTable(source_class:SourceClass,table_id:Id,input_variable_id:Id,unit_id:Id?,rows:List[NFDecisionRow])

S NFDecisionRow(row_id:Id,guard:ContextExpr,output_slot_id:Id,output_value:ReadingRef,source_region_id:RegionId)

S NFMetadataClaim(source_edition_hash:Hash,bibliographic_identity:Text<semantic_ja>,metadata_key:MetadataKey,metadata_value:Text<semantic_ja>)
```

Normal Form projection:

```text
NormReading -> NFNorm:
  copy source_class from the SourceEdition supporting the license; copy context, direction, action,
  temporal, original_modality_phrase_ja, and RecommendationMetadata.

FactualReading -> NFFactualRule:
  copy source_class; set strict from FactualReading.strict; copy context, adding subject as a
  context atom when subject present and not already in context; normalize FactualReading.consequent
  through §5.2 representatives and §8.1 atom normalization, copy as NFFactualRule.consequent (a
  FactualConsequent whose constraints = finite Set[ContextAtom] sorted by canonical bytes). One
  licensed strict fact may carry several QuantityConstraintAtom, TemporalConstraintAtom, or SlotEqAtom
  consequents: NF-Q1 carries systolic_bp < 90 and systolic_bp >= 90 as two QuantityConstraintAtom;
  NF-R1, NF-R2 carry one SlotEqAtom each.

TableReading -> NFDecisionTable:
  copy source_class, table_id, input_variable_id, unit_id, and rows in source row order.

SourceEdition metadata -> NFMetadataClaim:
  for every MetadataKey present in SourceEdition and demanded by SemanticPolicySet or fixture manifest,
  create exactly one metadata claim with semantic_ja-normalized value.
```

Normal Form algorithm:

```text
1 Preserve raw source support and proof roots.
2 Normalize semantic strings through declared StringPolicy.
3 Replace terminology references by representatives from §5.2.
4 Replace unit references by unit_equivalent representatives.
5 Normalize contexts by §8.1.
6 Normalize actions by §8.2 when an action affects theorem truth.
7 Sort commutative operands for conjunction, disjunction, unordered sets, unordered evidence sets, and unordered source sets.
8 Preserve order-sensitive structures: document order, temporal sequences, priority chains, and decision-table rows.
9 Preserve original modality surface and `RecommendationMetadata` when licensed; theorem predicates consume `direction`.
10 Generate deterministic IDs from normalized content and source anchors.
11 Sort diagnostics by source_order_key, diagnostic code, and artifact_hash.
12 Compute semantic_digest = sha256(canonical_payload_bytes(nf_payload)).
```

`semantic_digest` covers only `nf_payload`; `source_support`/`proof_roots` stay proof-visible on `CKCNormalForm` but outside `semantic_digest`, so equivalence ignores commutative-antecedent reordering and proof-root changes preserving NF payload bytes.

Worked digest example:

```text
payload_a = NFNorm(context = and(adult_population, suspected(sepsis)), direction=for, action=AB)
payload_b = NFNorm(context = and(suspected(sepsis), adult_population), direction=for, action=AB)
normalize_context sorts the commutative atoms before canonical_payload_bytes.
semantic_digest(payload_a) = sha256(canonical_payload_bytes(normalized payload_a)).
semantic_digest(payload_b) = sha256(canonical_payload_bytes(normalized payload_b)).
The two digests are equal even when their source_support or proof_roots differ.
```

Invariants:

```text
NF(NF(x)) = NF(x).
Reordered commutative antecedents produce identical semantic_digest by construction.
Order-sensitive edits produce a changed semantic_digest when the order changes meaning.
Every rewrite has a proof or compiler rule reference.
```

`T-NF-Idempotency` validates these invariants.

### 7.5 Deterministic glosses

Gloss = deterministic view over CKC Normal Form. Authority exactly `view_only`; theorem/checker semantics come from referenced NF and ProofDAG.

```text
S GlossTemplate(template_id:Id,nf_kind:NFKind,lang:Lang,literal_parts:List[Text<template_literal>],slots:List[GlossSlotSpec],renderer_id:Id)

S GlossSlotSpec(slot_name:Id,nf_path:FeaturePath,renderer_id:Id,required:Bool)

S GlossView(gloss_id:Id,nf_hash:Hash,lang:Lang,template_id:Id,slot_bindings:List[GlossSlotBinding],combined_slot_digest:Hash,rendered_text:Text<view_text>,authority:Authority,proof_roots:Set[ProofId])

S GlossSlotBinding(slot_name:Id,nf_path:FeaturePath,slot_digest:Hash,rendered_fragment:Text<view_text>)
```

Gloss authority invariant: `GlossView.authority = view_only`.

Rendering algorithm:

```text
1 Select the unique GlossTemplate matching (nf_kind, lang, renderer_id); zero or multiple matches yield unsupported.
2 Traverse slots in template order.
3 Resolve each nf_path against the NF payload; a missing required slot yields unsupported.
4 Render concepts from admitted terminology labels.
5 Render actions, directions, quantities, temporals, and contexts through fixed renderer tables keyed by renderer_id.
6 Compute slot_digest = sha256(canonical payload bytes of the resolved semantic slot).
7 Concatenate literal parts and rendered fragments.
8 Compute combined_slot_digest from ordered slot_digest values.
9 Store GlossView.
```

`gloss_semantic_drift(existing_gloss, current_nf, template)` holds when an existing claimed `GlossView` has the same `(nf_hash, lang, template_id)` and either:

```text
re-rendered canonical GlossView bytes differ; or
combined_slot_digest differs from the recomputed ordered slot digests.
```

## 8. Conflict and factual-inconsistency semantics

M0 mathematical core; kernel finite checker uses these predicates exactly. Unless an explicit parameter, predicates read accepted `TerminologyResourceSet T`, `SemanticPolicySet P`, `SourceGraph S`, `ProofDAG D` from `KernelFiniteCheckInput`.

### 8.1 Context satisfiability and compatibility

M0 contexts = finite disjunctions of finite conjunctions. `true` = one empty `ContextClause`; `false` = `clauses = {}`.

```text
S ContextExpr(clauses:Set[ContextClause])

S ContextClause(atoms:Set[ContextAtom])

E ContextAtom = PredAtom | NegPredAtom | SlotEqAtom | SlotInAtom | QuantityConstraintAtom | TemporalConstraintAtom | TemporalLiteralAtom

S PredAtom(predicate_id:Id,args:List[ReadingRef])

S NegPredAtom(predicate_id:Id,args:List[ReadingRef])

S SlotEqAtom(slot_id:Id,value:ReadingRef)

S SlotInAtom(slot_id:Id,values:Set[ReadingRef])

S QuantityConstraintAtom(variable_id:Id,comparator:Cmp,value:Rational,unit_id:Id)

S TemporalConstraintAtom(variable_id:Id,interval:TemporalInterval)

S TemporalLiteralAtom(variable_id:Id,value:Text<semantic_ja>)

S TemporalInterval(lower:Rational?,lower_closed:Bool,upper:Rational?,upper_closed:Bool,unit_id:Id)
```

Generator admission normalizes context syntax to this language. Disjunction expansion uses declared finite bounds. Bound excess calls `HandleBoundOverflow` for `ContextExpr.clauses` or the local disjunction-expansion bound; M0 context-generation local dispatch uses `emit_residual` → `Residual(class=unsupported_construction)` with `Diagnostic(code=context_bound_overflow)`.

```text
E ContextNormalizeResult = normalized:ContextExpr | unsupported:DiagnosticRef
```

`normalize_context(ContextExpr C, TerminologyResourceSet T) -> ContextNormalizeResult`:

```text
1 Replace term references by terminology representatives.
2 Replace units by unit_equivalent representatives.
3 Convert numeric quantity comparators to interval plus disequality constraints as required by
  `finite_constraint_check`.
4 Convert numeric temporal comparators to TemporalConstraintAtom and non-numeric temporal phrases
  to TemporalLiteralAtom.
5 Sort atoms and clauses by canonical payload bytes.
6 Remove duplicate atoms.
7 Preserve explicit negation as NegPredAtom.
8 Return unsupported when a referenced concept or unit lacks an admitted representative.
```

```text
E ContextCompatibility = compatible:WitnessContext | incompatible | unsupported:DiagnosticRef

S WitnessContext(left_clause_digest:Hash,right_clause_digest:Hash,atoms:Set[ContextAtom],assignments:Set[WitnessAssignment],source_support:Set[RegionId],proof_roots:Set[ProofId])

E WitnessAssignmentValue = reading_ref:ReadingRef | rational:Rational | text:Text<semantic_ja>

S WitnessAssignment(variable_id:Id,value:WitnessAssignmentValue)
```

`ctx_compatible(A,B) -> ContextCompatibility`:

```text
1 Normalize A and B. If normalization is unsupported, return unsupported.
2 If either normalized context has clauses = {}, return incompatible.
3 Enumerate clause pairs (a,b) by canonical_sort_key(a), then canonical_sort_key(b).
4 For each pair, C := union(a.atoms, b.atoms) and check C by finite_constraint_check(C).
5 Return compatible(WitnessContext) for the first satisfiable pair. Witness atoms = canonical duplicate-free union of the two normalized clauses; witness assignments = exactly the variables used by those atoms plus the deterministic rational/text/readings selected by `finite_constraint_check`; source_support and proof_roots = canonical unions of accepted operands contributing the selected clauses.
6 Record unsupported diagnostics but keep checking later pairs (a later satisfiable pair
  proves disjunction compatibility).
7 Return unsupported with the first diagnostic by canonical_sort_key when no pair is satisfiable
  and at least one pair is unsupported.
8 Return incompatible when every pair is checked and inconsistent.
```

`finite_constraint_check(C)` total over every M0 atom kind. First names atoms by sorting canonical atom bytes, assigning `a0`, `a1`, then increasing suffixes in order. Then branches:

```text
PredAtom and NegPredAtom:
  normalize arguments through terminology representatives when they are TermReading references;
  PredAtom(p,args) and NegPredAtom(p,args) are inconsistent exactly when p and normalized args match;
  open-world absence of an atom has no negative force.

SlotEqAtom and SlotInAtom:
  group by slot_id;
  normalize TermReading values through terminology representatives;
  all SlotEqAtom values for one slot must be identical canonical ReadingRef values;
  if two required SlotEqAtom values are distinct representatives connected by mutually_exclusive,
    the core reason is mutually_exclusive_slot_values; otherwise it is distinct_slot_values;
  intersect all SlotInAtom value sets after representative normalization;
  an empty intersection is inconsistent;
  a SlotEqAtom value must be a member of the remaining SlotInAtom intersection.

QuantityConstraintAtom:
  group by variable_id after unit normalization;
  comparator eq maps to [v,v]; lt maps to (-inf,v); le maps to (-inf,v]; gt maps to (v,inf);
  ge maps to [v,inf); ne records a finite disequality point v;
  intersect intervals and remove disequality points;
  the group is satisfiable iff the resulting rational interval contains at least one rational
  not removed by disequality constraints;
  unsupported is returned only when unit normalization is unsupported.

TemporalConstraintAtom:
  group by variable_id after unit normalization and apply the same interval-plus-disequality-free
  non-emptiness rule as quantity intervals.

TemporalLiteralAtom:
  group by variable_id;
  identical semantic_ja values are consistent;
  two different literal values for the same variable return unsupported with code=temporal_literal_mismatch,
  because M0 has no ordering or synonym theory for non-numeric temporal phrases.

Mixed temporal numeric/literal groups:
  a TemporalConstraintAtom and TemporalLiteralAtom with the same variable_id return unsupported
  with code=temporal_mixed_numeric_literal.
```

Rational interval witness selection with disequality points: first candidate from this deterministic list lying in the interval and not excluded: finite closed lower bound, finite open lower bound plus 1, finite closed upper bound, finite open upper bound minus 1, midpoint `(lower+upper)/2` when both bounds finite and unequal, 0, then `excluded_point+1` for excluded points in canonical order. One exists whenever interval-minus-finite-set is nonempty. Thus `x < 90 ∧ x <= 90` has canonical witness `89`.

`finite_constraint_check(C)` structured result: `result=satisfiable` carries the canonical witness assignment; `result=inconsistent` carries an inclusion-minimal named core under the fixed deletion order, plus an external solver core when the replayed solver proof is present. Internal core authoritative for M0.

```text
E FiniteConstraintResult = satisfiable | inconsistent | unsupported

S FiniteConstraintCheckResult(result:FiniteConstraintResult,witness_context:WitnessContext?,constraint_core_witness:ConstraintCoreWitness?,diagnostic_hashes:Set[Hash])

S NamedConstraintAtom(atom_id:Id,atom:ContextAtom,source_region_ids:Set[RegionId])

S ConstraintCoreWitness(named_atoms:Set[NamedConstraintAtom],core_atom_ids:Set[Id],internal_minimality_order_hash:Hash,external_backend_core_hash:Hash?,proof_roots:Set[ProofId])
```

`WitnessContext` canonical for the selected clause pair: atom set = duplicate-free union of the pair after normalization; deleting any atom changes the represented conjunction. `left_clause_digest`/`right_clause_digest` name the pair: each = `sha256(canonical_payload_bytes(clause))` over the corresponding normalized `ContextClause`, an in-memory value, never an accepted artifact. Kernel checker re-evaluates `ctx_compatible` directly. Inconsistent cores minimized by deterministic deletion: traverse atoms in canonical order, remove an atom when inconsistency is preserved, stop after one complete pass over the sorted list (inclusion-minimal for that traversal under monotone inconsistency). `ConstraintCoreWitness.internal_minimality_order_hash = sha256(canonical_payload_bytes(O))` over `List[Id]` `O` of `NamedConstraintAtom.atom_id` values in that fixed canonical deletion-traversal order; `external_backend_core_hash`, when present, = `sha256(exact_recorded_bytes)` of the replayed external solver core under the §1.2 raw-recorded convention.

### 8.2 Action normalization and sameness

```text
S NormalizedAction(action_kind:Id,target_key:Id,discriminating_slots:Map[Id,NormalizedSlotValue],support_hashes:Set[Hash])

E NormalizedSlotValue = ConceptSlotValue | QuantitySlotValue | LiteralSlotValue

S ConceptSlotValue(concept_id:Id)

S QuantitySlotValue(comparator:Cmp?,value:Rational,unit_id:Id)

S LiteralSlotValue(value_hash:Hash)

E ActionNormalizeResult = normalized:NormalizedAction | residual:Set[Hash] | incoherence:Set[Hash] | unsupported:DiagnosticRef

E ActionSameness = same:ActionSamenessWitness | distinct | residual:Set[Hash] | incoherence:Set[Hash] | unsupported:DiagnosticRef

E SlotNormalizeResult = normalized:NormalizedSlotValue | unsupported:DiagnosticRef

S ActionSamenessWitness(left_action_hash:Hash,right_action_hash:Hash,target_relation_hashes:Set[Hash],slot_spec_hashes:Set[Hash])
```

`normalize_slot_value(value, normalization, T) -> SlotNormalizeResult`:

```text
concept_representative:
  resolve value to TermReading and replace concept_id by its §5.2 representative.

unit_quantity:
  resolve value to QuantityReading, replace unit_id by its unit_equivalent representative,
  and keep exact comparator and Rational value.

literal_identity:
  compute value_hash from canonical payload bytes of the resolved reading or literal.
```

`normalize_action(ActionReading A, TerminologyResourceSet T, SemanticPolicySet P) -> ActionNormalizeResult`:

```text
1 Normalize A.action_kind through terminology representatives linked by action_kind_equivalent when
  action_kind names a Concept; otherwise keep the action_kind Id by literal identity.
2 Resolve A.target to a TermReading representative concept; non-term targets return unsupported.
3 For each slot in A.slots by slot_id:
     find exactly one ActionSlotSpec with that slot_id;
     zero matches returns residual with Residual(class=missing_policy);
     multiple non-identical matches returns incoherence with Incoherence(class=incompatible_generator_outputs);
     normalize the slot value by normalize_slot_value(value, ActionSlotSpec.normalization, T).
4 Retain exactly the slots whose ActionSlotSpec.discriminates_action_identity = true.
5 Sort retained slots by slot_id.
6 Return target_key as the representative target concept ID and support_hashes from the action and slot readings.
```

`targets_overlap(action_kind, left_target, right_target, P)`:

```text
1 If left_target = right_target, return same with empty relation witness.
2 Enumerate ActionTargetRelation rows by canonical_sort_key.
3 A row matches when row.action_kind = action_kind and its endpoints equal the target pair;
  if row.symmetric = true, either endpoint order matches.
4 Return same with the first matching row as witness.
5 Return distinct when no row matches.
```

`same_normalized_action(A,B,T,P)` returns:

```text
same(witness):
  both actions normalize;
  normalized action_kind values are equal;
  targets_overlap returns same;
  every slot_id in the union of discriminating_slots occurs on both sides and has equal normalized value.

distinct:
  both actions normalize and any equality condition above fails.

residual or incoherence:
  either action normalization emits the same primary non-success class.

unsupported:
  either action normalization returns unsupported.
```

### 8.3 Consequent and table compatibility

```text
E ConsequentExpr = NormConsequent | FactualConsequent | TableConsequent

S NormConsequent(direction:Direction,action:ActionReading)

S FactualConsequent(constraints:Set[ContextAtom])

S TableConsequent(output_slot_id:Id,output_value:ReadingRef)

E ConsequentCompatibility = compatible:WitnessContext? | incompatible:ConsequentIncompatibilityWitness | residual:Set[Hash] | incoherence:Set[Hash] | unsupported:DiagnosticRef

S ConsequentIncompatibilityWitness(left_hash:Hash,right_hash:Hash,reason_code:Id,action_witness_hash:Hash?,constraint_witness_hash:Hash?,output_exclusion_hash:Hash?)
```

Normative direction groups:

```text
normative_positive = {for, require, permit}
normative_against = {avoid, against}
normative_contraindicating = {contraindicate}
normative_negative = {contraindicate, avoid, against}
```

Normative consequents incompatible exactly when one direction ∈ `normative_positive`, the other ∈ `normative_negative`, and `same_normalized_action(left.action,right.action,T,P) = same`. Symmetric by unordered pair enumeration. Conflict predicates partition negative directions: `contraindication_vs_recommendation` uses `normative_contraindicating`, `recommendation_for_vs_against` uses `normative_against`, general consequent-compatibility and package-insert factual predicates use `normative_negative`.

Strict factual consequents incompatible when `finite_constraint_check(left.constraints ∪ right.constraints)` is inconsistent.

Table consequents evaluated by `table_outputs_compatible(left,right,SemanticPolicySet P)`:

```text
1 If output_slot_id differs, return compatible.
2 If output_value hashes are equal, return compatible.
3 Enumerate OutputExclusion rows by canonical_sort_key.
4 A row matches when row.output_slot_id equals the shared slot and its values equal the output pair;
  if row.symmetric = true, either order matches.
5 Return incompatible with the first matching OutputExclusion witness.
6 Return residual with Residual(class=missing_policy) when output values differ and no row matches.
```

`consequents_compatible(left,right,T,P) -> ConsequentCompatibility` = builtin named in §6.2:

```text
1 If both inputs are NormConsequent, evaluate same_normalized_action(left.action,right.action,T,P).
  If it returns residual, incoherence, or unsupported, propagate that result. If it returns same and
  the directions are incompatible under the normative direction-group rule, return incompatible with
  reason_code=norm_direction_conflict. Otherwise return compatible.
2 If both inputs are FactualConsequent, run finite_constraint_check(left.constraints ∪ right.constraints);
  inconsistent returns incompatible with reason_code=factual_constraint_conflict; satisfiable returns
  compatible with the witness context; unsupported returns unsupported.
3 If both inputs are TableConsequent, return table_outputs_compatible(left,right,P).
4 If the consequence tags differ, return compatible.
5 Propagate residual, incoherence, or unsupported outcomes by the §1.7 primary-status order when a called predicate emits a non-decision result.
```

Strict factual self-check:

```text
E StrictFactualSelfCheck = satisfiable:WitnessContext? | self_inconsistent:ConstraintCoreWitness | unsupported:DiagnosticRef

strict_factual_self_check(rule) -> StrictFactualSelfCheck:
  rule is an NFFactualRule with strict=true;
  run finite_constraint_check(rule.consequent.constraints);
  satisfiable returns satisfiable(witness_context);
  inconsistent returns self_inconsistent(constraint_core_witness);
  unsupported returns unsupported(first diagnostic by canonical_sort_key).
```

Strict factual rule with self-inconsistent consequent = single-subject inconsistency. Emits `numeric_threshold_empty_intersection` when the minimal core is numeric or temporal interval-empty; otherwise emits `Incoherence(class=incompatible_generator_outputs)` with the strict rule hash as subject. Not an operand of pairwise `strict_consequents_jointly_contradictory`.

### 8.4 Resolution set

```text
E ResolutionTheoremKind = exception | priority | scope_limitation | supersession | explicit_reconciliation

S ResolutionTheorem(theorem_id:Id,kind:ResolutionTheoremKind,applies_to:Set[Hash],context:ContextExpr,source_support:Set[RegionId],proof_roots:Set[ProofId])
```

`context_from` total/deterministic over `ContextCompatibility`:

```text
context_from(compatible(WitnessContext W)) = ContextExpr{clauses={ContextClause{atoms=W.atoms}}}
context_from(incompatible) = ContextExpr{clauses={}}
context_from(unsupported(_)) = ContextExpr{clauses={}}
```

Given payload `W`, `context_from(W)` = `context_from(compatible(W))`. Non-compatible cases return canonical false context. §8.5/§8.6 call `ResolutionSet` only after `ctx_compatible(input)=compatible(W)`.

`resolution_subject_ids(x)` = canonical hash set naming an operand:

```text
x.artifact_hash;
x.semantic_digest when x is CKCNormalForm;
every License hash contributing to x via parent_air_core_hashes -> AIRCoreRecord -> LicensedReadingSet.license_hashes.
```

`ResolutionSet(left,right,W)` = canonical set of admitted `ResolutionTheorem` s.t.:

```text
theorem.applies_to = {a,b} for some a in resolution_subject_ids(left) and b in resolution_subject_ids(right);
ctx_compatible(theorem.context, context_from(W)) = compatible(_).
```

Keying lets stage-40 theorems name license hashes while stage-60 builders consume NF operands. Pair unresolved when `ResolutionSet(left,right,W) = {}`.

All `ResolutionTheoremKind` variants (`exception`, `priority`, `scope_limitation`, `supersession`, `explicit_reconciliation`) have equal M0 resolving force: enter `ResolutionSet` when `applies_to`+`context` match. `kind` proof- and report-visible.

### 8.5 M0 conflict predicates

Conflict theorems reference parent NF objects, witness, proof roots. Pair predicates enumerate unordered distinct accepted-operand pairs by key `(min(hash), max(hash), conflict_kind)`. Single-subject predicates key `(subject_hash, conflict_kind)`. A pair predicate fires only when each operand passes its named self-consistency guard; self-inconsistent operands are reported by the detecting single-subject predicate and excluded from pairwise closure. For pair kinds, `ConflictTheorem.witness_hash` = accepted `WitnessContext` hash for the selected compatible pair `W`.

Single-subject keying rules:

```text
numeric_threshold_empty_intersection key = (subject_nf_hash, numeric_threshold_empty_intersection).
numeric_threshold_empty_intersection.witness_hash = the accepted ConstraintCoreWitness artifact hash for the selected deletion-minimal empty interval core.
numeric_threshold_empty_intersection.constraint_core_witness_hashes = {numeric_threshold_empty_intersection.witness_hash}.
terminology_mapping_incoherence key = (terminology_closure_hash, terminology_mapping_incoherence).
terminology_mapping_incoherence.left_artifact_hash = terminology_closure_hash.
terminology_mapping_incoherence.right_artifact_hash is omitted.
terminology_mapping_incoherence.witness_hash = sha256(canonical set of referenced terminology incoherence hashes).
```

Terminology predicate: one theorem per `TerminologyClosure` holding ≥1 `functional_key_collision` or `mutually_exclusive_term_mapping` incoherence; references all such incoherence hashes in witness + source-region union; individual incoherences stay independently reportable.

```text
contraindication_vs_recommendation:
  left and right are NFNorm;
  one direction is in `normative_contraindicating` and the other is in `normative_positive`;
  same_normalized_action(left.action,right.action,T,P) = same(action_witness);
  ctx_compatible(left.context,right.context) = compatible(W);
  ResolutionSet(left,right,W) = {};
  emit ConflictTheorem(kind=contraindication_vs_recommendation).

recommendation_for_vs_against:
  left and right are NFNorm;
  one of {left.direction,right.direction} is in `normative_positive`;
  the other is in `normative_against`;
  same_normalized_action(left.action,right.action,T,P) = same(action_witness);
  ctx_compatible(left.context,right.context) = compatible(W);
  ResolutionSet(left,right,W) = {};
  emit ConflictTheorem(kind=recommendation_for_vs_against).

strict_consequents_jointly_contradictory:
  left and right are NFFactualRule with strict = true;
  strict_factual_self_check(left) = satisfiable(_);
  strict_factual_self_check(right) = satisfiable(_);
  ctx_compatible(left.context,right.context) = compatible(W);
  consequents_compatible(left.consequent,right.consequent,T,P) = incompatible(witness);
  ResolutionSet(left,right,W) = {};
  emit ConflictTheorem(kind=strict_consequents_jointly_contradictory).

numeric_threshold_empty_intersection:
  a finite set of QuantityConstraintAtom or TemporalConstraintAtom values appears in one normalized
  context or in one strict factual consequent and shares variable_id and normalized unit_id;
  their interval intersection is empty under §8.1;
  emit one single-subject ConflictTheorem(kind=numeric_threshold_empty_intersection) with
  right_artifact_hash omitted for that normalized context or strict factual rule.

terminology_mapping_incoherence:
  §5.2 emits one or more functional_key_collision or mutually_exclusive_term_mapping incoherences
  inside one TerminologyClosure;
  emit one single-subject ConflictTheorem(kind=terminology_mapping_incoherence) for that closure and
  reference the complete canonical set of corresponding Incoherence diagnostics.
```

Unsupported action/context/consequent/policy coverage → typed residual, not theorem.

### 8.6 M0 factual-inconsistency predicates

Pair predicates use the §8.5 unordered distinct-pair enumeration. Single-subject predicates key `(subject_hash, factual_inconsistency_kind)`. A row/norm/metadata claim whose own guard/context/action-normalization/required-policy is unsupported or self-inconsistent emits its producer's typed residual or incoherence and is excluded from pairwise construction. Overlap of `package_insert_vs_guideline_unresolved_conflict` with kinds like `contraindication_vs_recommendation` is intended: one NF pair may yield one conflict + one factual-inconsistency theorem under two proof-visible lenses.

```text
table_value_disagreement:
  left and right are rows from the same NFDecisionTable or two NFDecisionTable objects with
  equal input_variable_id, normalized unit_id, and output_slot_id;
  ctx_compatible(left.guard,right.guard) = compatible(W);
  table_outputs_compatible(
    TableConsequent(left.output_slot_id,left.output_value),
    TableConsequent(right.output_slot_id,right.output_value),P) = incompatible(output_witness);
  emit FactualInconsistencyTheorem(kind=table_value_disagreement).

package_insert_vs_guideline_unresolved_conflict:
  left and right are NFNorm;
  one source_class is package_insert and the other is guideline;
  their directions are normatively incompatible under §8.3;
  same_normalized_action(left.action,right.action,T,P) = same;
  ctx_compatible = compatible(W);
  ResolutionSet = {};
  emit FactualInconsistencyTheorem(kind=package_insert_vs_guideline_unresolved_conflict).

gloss_semantic_drift:
  §7.5 drift predicate holds;
  emit FactualInconsistencyTheorem(kind=gloss_semantic_drift).

source_metadata_disagreement:
  left and right are NFMetadataClaim;
  metadata_key is equal and present in SemanticPolicySet.metadata_singleton_keys;
  bibliographic_identity is equal;
  metadata_value strings differ after semantic_ja normalization;
  emit FactualInconsistencyTheorem(kind=source_metadata_disagreement).

proof_or_certificate_replay_failure:
  ReplayIdentityCheck.outcome = replay_identity_mismatch or a Certificate.proof_root_ids member fails ProofDAG checking;
  emit FactualInconsistencyTheorem(kind=proof_or_certificate_replay_failure).
```

`metadata_key` absent from `metadata_singleton_keys` → `Residual(class=missing_policy)`.

`FactualInconsistencyTheorem.missing_evidence`:

```text
table_value_disagreement -> {}.
package_insert_vs_guideline_unresolved_conflict -> {admitted_resolution_theorem} when ResolutionSet is {}.
gloss_semantic_drift -> {}.
source_metadata_disagreement -> {} when metadata_singleton_keys contains the key; otherwise a missing_policy residual is emitted instead of a theorem.
proof_or_certificate_replay_failure -> {replay_identity_pass} for replay_identity_mismatch and {valid_certificate_proof_root} for a failed proof root.
```

Sorted by identifier bytes, hashed only through theorem payload; empty set encoded `[]`.

`FactualInconsistencyTheorem.witness_hash` by kind: `table_value_disagreement`, `package_insert_vs_guideline_unresolved_conflict` use accepted `WitnessContext` hash `W` from the compatible guard/context check; `gloss_semantic_drift`, `source_metadata_disagreement`, `proof_or_certificate_replay_failure` omit it (optional; evidence carried by referenced gloss/metadata/replay/certificate/diagnostic/verifier artifacts).

### 8.7 Theorem, diagnostic, and minimization schemas

```text
S ConflictTheorem(conflict_id:Id,conflict_kind:ConflictKind,left_artifact_hash:Hash,right_artifact_hash:Hash?,nf_hashes:Set[Hash],minimal_theorem_set:Set[Hash],minimal_generator_dependency_set:Set[Hash],source_regions:Set[RegionId],witness_hash:Hash,constraint_core_witness_hashes:Set[Hash],missing_resolution_class:Id?,verifier_witness_hashes:Set[Hash],review_question_ja:Text<view_text>,review_question_en:Text<view_text>?,classification:ReviewClassification,claim_tier:ClaimTier,falsification_criterion:Text<diagnostic_text>,proof_roots:Set[ProofId])

S FactualInconsistencyTheorem(inconsistency_id:Id,inconsistency_kind:FactualInconsistencyKind,left_artifact_hash:Hash,right_artifact_hash:Hash?,source_regions:Set[RegionId],witness_hash:Hash?,constraint_core_witness_hashes:Set[Hash],missing_evidence:Set[Id],verifier_witness_hashes:Set[Hash],review_question_ja:Text<view_text>,review_question_en:Text<view_text>?,classification:ReviewClassification,claim_tier:ClaimTier,falsification_criterion:Text<diagnostic_text>,proof_roots:Set[ProofId])

S Residual(residual_id:Id,class:ResidualClass,subject_hash:Hash?,source_regions:Set[RegionId],diagnostic:Text<diagnostic_text>,proof_roots:Set[ProofId])

S Ambiguity(ambiguity_id:Id,class:AmbiguityClass,alternatives:Set[Hash],source_regions:Set[RegionId],proof_roots:Set[ProofId])

S Incoherence(incoherence_id:Id,class:IncoherenceClass,subject_hashes:Set[Hash],source_regions:Set[RegionId],proof_roots:Set[ProofId])

S Diagnostic(diagnostic_id:Id,code:Id,subject_hash:Hash?,source_regions:Set[RegionId],text:Text<diagnostic_text>)

S DiagnosticRef(diagnostic_hash:Hash)
```

`Incoherence.subject_hashes` members are subject-identity hashes: envelope `artifact_hash` if enveloped, else `sha256(type_tagged_payload_bytes(subject))` (§1.5), whose `declared_type_id` prefix keeps field-isomorphic subjects of differing declared types distinct. §1.1 `overflow_member_hash` instantiates this.

Review-classification assignment total/fixed:

```text
ConflictTheorem.classification = candidate.
FactualInconsistencyTheorem.classification = candidate.
Residual report source classification = residual.
Ambiguity report source classification = ambiguity.
Incoherence report source classification = incoherence.
ReplayIdentityCheck with replay_identity_mismatch and failed certificate proof roots use replay_failure.
Coverage diagnostics point to an unmet demanded AIR key and use residual.
```

Theorem builders reject any payload whose stored `classification` differs.

Computed theorem fields:

```text
ConflictTheorem.missing_resolution_class:
  pair predicates that require ResolutionSet(left,right,W) = {} store unresolved_pair;
  pair predicates suppressed by a nonempty ResolutionSet emit no theorem;
  single-subject predicates omit the field.

FactualInconsistencyTheorem.missing_evidence:
  computed by §8.6 and stored as a sorted Set[Id].

review_question_ja and review_question_en:
  rendered only from admitted report-question templates under §9.3 wording-gate rules.
```

Micro-example: `N1`,`N3` satisfy recommendation-for-vs-against antecedents with `ResolutionSet(N1,N3,W)={RT-R1}` → pair key recorded in non-firing fixture list, no `ConflictTheorem` payload. Same antecedents with `ResolutionSet={}` → theorem stores `missing_resolution_class=unresolved_pair`.

`theorem_minimize` and `dependency_minimize` use deterministic deletion:

```text
1 Sort the input set by canonical_sort_key.
2 For each candidate in order, remove it when the bridge predicate still holds over the remaining set.
3 Return the final set.
```

Canonical for CKC M0: inclusion-minimal witness under fixed deletion order. Constraint contradictions also emit `ConstraintCoreWitness`; when an SMT unsat core is replayable, theorem stores both external core and internal deletion-minimal core. Repair-set search = proof-visible diagnostic trace; affects accepted semantics only via admitted edits.

```text
S RepairSetSearchTrace(trace_id:Id,conflict_or_inconsistency_hash:Hash,candidate_dependency_hashes:Set[Hash],objective:RepairObjective,returned_sets:Set[Set[Hash]],exactness:RepairSearchExactness,replay_manifest_hash:Hash,proof_roots:Set[ProofId])

E RepairObjective = minimal_conflict_core | minimal_correction_set | weighted_repair_set
E RepairSearchExactness = exact | bounded_exact | heuristic_diagnostic_only
```

## 9. Finite checking, certificates, reports, and replay

### 9.1 Kernel finite checker

Kernel finite checker validates M0 theorems by re-evaluating §8 over canonical NF, resource, policy, SourceGraph, ProofDAG inputs.

```text
S KernelFiniteCheckInput(subject_hash:Hash,subject_kind:KernelSubjectKind,proof_dag_hash:Hash,source_graph_hash:Hash,accepted_generator_base_hash:Hash,terminology_resource_set_hash:Hash,semantic_policy_set_hash:Hash,schema_registry_hash:Hash)

E KernelSubjectKind = conflict_theorem | factual_inconsistency_theorem | residual | ambiguity | incoherence | diagnostic | certificate | review_report | replay_identity_check

S VerifierWitness(witness_id:Id,subject_hash:Hash,result:VerifierResult,checked_predicates:Set[Id],diagnostic_hashes:Set[Hash],symbol_source_map_hash:Hash,replay_manifest_hash:Hash,proof_roots:Set[ProofId])

S SymbolSourceMap(rows:Set[SymbolSourceMapRow])

S SymbolSourceMapRow(symbol_id:Id,symbol_kind:Id,defining_section_anchor:Id,defining_artifact_hash:Hash?)
```

`SymbolSourceMap`: accepted schema-control artifact from `kernel_finite_checker`. `VerifierWitness.subject_hash` = checked subject's envelope `artifact_hash` (= `KernelFiniteCheckInput.subject_hash`); full check-input set pinned by `replay_manifest_hash`. Dispatch re-evaluation compares recomputed values against subject's stored witness artifacts in place; outcomes ride `result`+`diagnostic_hashes`, no separate persisted payload. `VerifierWitness.symbol_source_map_hash` = referenced `SymbolSourceMap` hash. `SymbolSourceMap.rows` = sorted set of `{symbol_id,symbol_kind,defining_section_anchor,defining_artifact_hash?}` for every predicate, enum variant, schema id, policy row key, terminology relation kind, proof rule, gate referenced while checking subject. Section anchors = stable headings, encoded as identifier strings e.g. `section-8-5`. Artifact-backed symbols include accepted artifact hash; specification-only symbols omit it.

`kernel_finite_checker` checks:

```text
1 schema validation and canonical bytes;
2 ProofDAG rule side conditions;
3 source-region closure certificates;
4 licensed-reading set exactness;
5 AIRCore finite-set outcome;
6 NF idempotency;
7 terminology closure and functional-key diagnostics;
8 semantic policy coverage required by §8;
9 context compatibility witnesses;
10 action sameness witnesses;
11 consequent incompatibility witnesses and constraint-core witnesses;
12 resolution-set emptiness;
13 conflict and factual-inconsistency predicate witnesses;
14 residual, ambiguity, incoherence, and diagnostic producer obligations;
15 claim-tier evidence;
16 certificate class obligations;
17 report projection exactness;
18 replay identity for referenced certificates and reports.
```

Dispatch by `KernelSubjectKind`, exhaustive:

```text
conflict_theorem:
  valid iff corresponding §8.5 predicate re-evaluates true and every witness/minimality claim checks.

factual_inconsistency_theorem:
  valid iff corresponding §8.6 predicate re-evaluates true and every witness/minimality claim checks.

residual, ambiguity, incoherence, diagnostic:
  valid iff named producer condition replays and payload class matches fixed §8.7 assignment.

certificate:
  valid, invalid, or unsupported exactly as `VerifyCertificate` (§9.2) returns over subject certificate.

review_report:
  valid iff `BuildReviewReport` (§9.3) recomputes same canonical report bytes and every item classification matches fixed report assignment; invalid on mismatch; unsupported only when report necessarily references a gated capability without valid gate evidence.

replay_identity_check:
  valid iff `ReplayIdentity` (§1.6) recomputes same `ReplayIdentityCheck` canonical payload and outcome; invalid on byte/outcome mismatch; unsupported iff replay unsupported for the same absent-toolchain or permissioned-source reason recorded in payload.
```

Failed checks → `invalid`. Unsupported predicate fragments → `unsupported` + `Residual(class=verifier_unsupported)`.

### 9.2 Certificates

```text
S Certificate(certificate_id:Id,certificate_class:M0CertificateClass,subject_hash:Hash,proof_root_ids:Set[ProofId],replay_identity_hashes:Set[Hash],verifier_witness_hashes:Set[Hash],claim_record_hashes:Set[Hash],issued_at_logical_time:UInt,accepted_effect_row:Set[Effect])
```

Logical time assigned by `CanonicalIssuanceOrder(run)`:

```text
1 Collect every accepted ReviewerRecord, AdmissionRecord, Certificate, and other accepted payload
  with a logical-time field in the run.
2 Build a finite citation DAG over those payloads. Add an edge from cited payload to citing
  payload whenever both endpoints have logical time and one endpoint references the other by
  artifact hash. Add the fixed admission edge ReviewerRecord -> AdmissionRecord for each
  AdmissionRecord.reviewer_record_hashes member. Add certificate edges from every referenced
  reviewer record, admission record, and certificate to the certificate that cites it through
  subject, replay, verifier, or claim-record evidence. Add the same cited -> citing edge for
  any future logical-time-bearing payload referenced by another logical-time-bearing payload.
3 Reject a nonempty cycle as invalid with diagnostic code=logical_time_cycle.
4 Assign values by Kahn topological order. At each step choose, among zero-indegree unassigned
  payloads, the minimum canonical tie key
  (stage, operation_id, payload schema_id, subject_hash_or_empty, artifact_hash) using §1.5
  canonical bytes, then assign UInt values "0", "1", then increasing decimal strings in the selected order.
5 Store the assigned value in ReviewerRecord.logical_time, AdmissionRecord.admitted_at_logical_time,
  Certificate.issued_at_logical_time, or the schema-declared logical-time field of the payload.
```

Replay-stable: edges = accepted artifact references, tie keys = canonical bytes or declared stage/operation ids. Reviewer records precede admission records citing them; each certificate follows logical-time-bearing records/certificates reachable through its cited evidence.

Certificate classes:

```text
source_graph:
  SourceGraph and permission records canonicalize and region closure checks.

mech_observed:
  MechObsPayload replay byte-identically from SourceGraph and AnalyzerManifest.

admitted_base:
  Accepted generators, terminology resources, and semantic policies discharge to accepted_effect_row = {}.

closed_nf:
  Closure terminates, licensed readings are exact, AIRCore outcomes check, and NF idempotency holds.

finite_checked:
  Kernel finite checker validates the M0 theorem witness.

report_replay:
  ReviewReport and ReplayIdentityCheck canonicalize and replay.
```

`IssueCertificate(subject_hash, certificate_class, proof_roots, replay_checks, verifier_witnesses, claim_records, run_manifest) -> OperationResult[Certificate]` total: validate all inputs, select branch obligation for `certificate_class`, call `VerifyCertificate` on candidate certificate bytes, emit certificate exactly when verification = `valid`, return `unsupported` for gated subject without evidence, return `invalid` for failed common/branch checks.

`VerifyCertificate(Certificate C) -> VerifierResult` total:

```text
1 Validate C schema, canonical bytes, logical-time monotonicity within the run manifest, and
  accepted_effect_row = {}.
2 Resolve subject_hash and every proof_root_id, replay_identity_hash, verifier_witness_hash, and
  claim_record_hash.
3 Check every proof root in ProofDAG.
4 Branch on certificate_class:
   source_graph: run P-SG-total-text, P-SG-total-support, P-SG-canonical, P-SG-permission, and every
     RegionClosureCertificate.
   mech_observed: rerun ObserveMech for the declared AnalyzerManifest and compare MechObsPayload hashes.
   admitted_base: verify every AdmissionRecord.decision=accept, every EffectDischargeRecord has
     accepted_effect_row={}, and required CounterexampleSuite obligations pass.
   closed_nf: run T-Closure-Termination, T-AIR-Finite-Set, T-NF-Idempotency, and terminology/policy checks.
   finite_checked: run kernel_finite_checker over the subject theorem and require valid.
   report_replay: rebuild ReviewReport, rerun ReplayIdentity, and require replay_identity_pass unless the
     certificate subject is an explicit replay-failure fixture.
5 Return valid iff the branch obligation and all common checks pass; return invalid for failed checks;
  return unsupported only when the subject names a gated capability without valid gate evidence.
```

### 9.3 Reports

```text
S ReviewReport(report_id:Id,source_edition_hashes:Set[Hash],report_items:List[ReportItem],replay_manifest_hash:Hash,proof_dag_hash:Hash,trace_hash:Hash,claim_tier_summary_hash:Hash,wording_gate_hash:Hash)

S ReportItem(item_id:Id,item_kind:ReportItemKind,exact_japanese_source_regions:Set[RegionId],deterministic_gloss_ja_hash:Hash?,deterministic_gloss_en_hash:Hash?,nf_hashes:Set[Hash],theorem_hashes:Set[Hash],minimal_theorem_set:Set[Hash],minimal_generator_dependency_set:Set[Hash],witness_hash:Hash?,verifier_witness_hashes:Set[Hash],certificate_hashes:Set[Hash],review_question_ja:Text<view_text>,review_question_en:Text<view_text>?,classification:ReviewClassification,certificate_depth:M0CertificateClass,claim_tier:ClaimTier,falsification_criterion:Text<diagnostic_text>)

E ReportItemKind = conflict_candidate | factual_inconsistency_candidate | ambiguity | residual | incoherence | replay_failure | coverage_diagnostic

S ReportQuestionTemplate(template_id:Id,item_kind:ReportItemKind,lang:Lang,tier:ClaimTier,literal_parts:List[Text<template_literal>],slots:List[GlossSlotSpec],allowed_wording_ids:Set[Id],renderer_id:Id,admission_record_hash:Hash)

S ReportTraceIndex(rows:Set[ReportTraceRow])

S ReportTraceRow(item_id:Id,item_kind:ReportItemKind,source_region_ids:Set[RegionId],nf_hashes:Set[Hash],theorem_hashes:Set[Hash],witness_hash:Hash?,verifier_witness_hashes:Set[Hash],certificate_hashes:Set[Hash],proof_root_ids:Set[ProofId])

S ClaimTierSummary(rows:Set[ClaimTierSummaryRow])

S ClaimTierSummaryRow(tier:ClaimTier,item_ids:Set[Id],claim_record_hashes:Set[Hash])

S WordingGateRecord(record_id:Id,template_hashes:Set[Hash],literal_part_digests:Set[Hash],max_claim_tier:ClaimTier,allowed_wording_ids:Set[Id],outcome:Outcome,diagnostic_hashes:Set[Hash])
```

Computed report fields:

```text
`ReportTraceIndex`, `ClaimTierSummary`, `WordingGateRecord`: accepted view-control artifacts from `BuildReviewReport`. `ReviewReport.trace_hash`/`claim_tier_summary_hash`/`wording_gate_hash` store the corresponding artifact hashes.

ReportTraceIndex rows sorted {item_id,item_kind,source_region_ids,nf_hashes,theorem_hashes,witness_hash,verifier_witness_hashes,certificate_hashes,proof_root_ids}.

ClaimTierSummary rows sorted {tier,item_ids,claim_record_hashes} for S0,S1,S2,S3.

WordingGateRecord names every accepted ReportQuestionTemplate and GlossTemplate whose literal parts appear in the report: `template_hashes` stores those envelope hashes; each `literal_part_digests` member = `sha256(canonical_payload_bytes(literal_parts))` over one named template's `literal_parts` list.

ReportItem.certificate_depth = greatest class in this order among valid certificate_hashes attached to the item:
source_graph < mech_observed < admitted_base < closed_nf < finite_checked < report_replay.
Item stores class, not count. Theorem item normally stores finite_checked. Report item stores report_replay only when that report_replay certificate belongs to a prior issuance stratum; the certificate certifying the current ReviewReport is outside the report payload, attached by the later §1.6 stratum.
```

Wording mechanically total: every `review_question_ja`, `review_question_en`, report-level explanatory sentence rendered from admitted `ReportQuestionTemplate`/`GlossTemplate` literal parts plus deterministic slot renderers. Wording gate validates template IDs, literal-part digests, renderer IDs, claim-tier vocabulary; arbitrary `view_text` without template provenance record → `invalid_payload`. Allowed M0 wording = §3.4 vocabulary plus source labels, artifact identifiers, section labels, deterministic slot renderings.

Report item kind consumers:

```text
conflict_candidate: emitted for each valid ConflictTheorem.
factual_inconsistency_candidate: emitted for each valid FactualInconsistencyTheorem.
ambiguity: emitted for each Ambiguity payload.
residual: emitted for each Residual payload, including permission_limited.
incoherence: emitted for each Incoherence payload.
replay_failure: emitted for replay_identity_mismatch or failed certificate proof roots.
coverage_diagnostic: emitted for declared coverage targets whose demanded AIR key remains represented by a coverage report item rather than a theorem, residual, ambiguity, or incoherence report item.
```

Report item classification fixed by `ReportItemKind`:

```text
conflict_candidate -> candidate.
factual_inconsistency_candidate -> candidate.
ambiguity -> ambiguity.
residual -> residual.
incoherence -> incoherence.
replay_failure -> replay_failure.
coverage_diagnostic -> residual when produced from an unmet demanded AIR key.
```

`BuildReviewReport` rejects any item whose stored `classification` differs from this table.

Report item ordering:

```text
1 source_order_key of first source region;
2 item_kind;
3 conflict or inconsistency kind;
4 theorem hash;
5 item_id.
```

Every report item links to exact Japanese source regions, deterministic glosses when available, NF payloads, proof roots, witness payloads, prior-stratum certificate payloads, claim tier, falsification criterion. The report_replay certificate certifying the current ReviewReport is excluded from this report payload, appears in the later §1.6 issuance stratum.

`BuildReviewReport(inputs, permission_records) -> OperationResult[ReviewReport]` total:

```text
1 Select valid ConflictTheorem and FactualInconsistencyTheorem artifacts by verifier witness.
2 Add every Residual, Ambiguity, and Incoherence not already represented by a theorem item.
3 Add replay_failure items for replay_identity_mismatch and failed certificate proof roots.
4 Add coverage_diagnostic items for demanded AIR keys not represented by theorem, residual,
  ambiguity, or incoherence items.
5 For each item, compute exact_japanese_source_regions from the theorem or diagnostic source-support
  projection and attach deterministic gloss hashes when a GlossView exists.
6 Apply SourcePermissionRecord. If a requested view needs quoted snippets or source bytes not allowed
  by allowed_artifacts, emit Residual(class=permission_limited) and produce a report item that contains
  only allowed region IDs, hashes, and derived labels.
7 Compute claim_tier by `T-Claim-Tiering`; build WordingGateRecord from admitted templates and reject any text whose template literal parts or allowed_wording_ids exceed the computed tier.
8 Sort items by the report ordering rule and canonicalize the report.
9 Check every ReviewReport collection bound; on overflow call `HandleBoundOverflow` and return its
  exact status, otherwise return the §1.7 primary status over the accepted report and emitted
  residuals.
```

### 9.4 UI

UI renders verified static report artifacts. Accepted CKC artifacts created only by canonical CLI/build commands and validated artifact writers.

Required M0 views:

```text
source and extraction QA;
terminology/resource/policy view;
conflict list;
conflict detail;
factual inconsistency detail;
residual and ambiguity list;
proof and certificate detail;
replay manifest;
derivation trace (source text to CKC Normal Form).
```

UI uses formalization-QA and text-quality language. Every claim links to source regions and artifact hashes.

Derivation-trace view: deterministic `view_only` rendering over already-proof-visible artifacts; for a selected source region walks `SourceGraph -> MechObsPayload -> PatternObs -> Match -> License -> AIRCoreRecord -> CKCNormalForm` along the `ProofDAG`, showing each intermediate representation and the rule producing it down to the most compact accepted form (the `semantic_digest` of the CKC Normal Form payload). Adds no semantics: every node shown is an accepted artifact carrying its own proof roots. When the §13.2 layered-IR track is exercised, the same view extends through the additional IR stages recorded in a `RuntimeOracleReport` down to the compiled target language.

## 10. Evaluation and calibration

M0 evaluation = replay + deterministic fixture checking through §11.3 acceptance gates. M0 reports may display deterministic counts derivable from accepted artifact sets: theorem counts by M0 kind, residual, ambiguity, incoherence, certificate, replay-outcome, demanded-AIR-key, no_license-residual, multiple_readings-ambiguity, accepted-generator counts, plus the unused-generator and per-generator marginal-coverage figures below. Report annotations derived from accepted semantic facts.

Component-economy accounting: two annotations instrumenting reusability deterministically. Unused-generator set = `AcceptedGeneratorBase.generator_hashes` minus union of every accepted theorem's `minimal_generator_dependency_set`; admitted generators no accepted result depends on. Per-generator marginal coverage, via deterministic license deletion under §8.1/§8.7: for each accepted generator, count of demanded AIR keys whose `ok` AIRCore reading is lost when that generator's licenses are removed. Both = exact functions of accepted artifact set, per-source-set accounting facts, not optimality/compression/coverage-of-clinical-knowledge claims; minimum-description-length, Pareto, reuse-payoff, corpus-coverage verdicts stay gated behind `G-MDL`/`G-EMIN` (§3.3), framed as research track in §13.1.

Empirical thresholds, precision, recall, retrieval quality, compression payoff, generalization, clinical-performance metrics require the relevant §3.3 evidence object.

Top M0 assurance claim:

```text
Accepted CKC artifacts are source-grounded, deterministic, finite-checkable,
replayable, and suitable for identifying clinical text requiring review.
```

Regulated profiles require `G-S3`.

## 11. CLI, repository, and build phases

### 11.1 Canonical CLI

CLI surface is namespaced. Sole canonical command names:

```text
ckc schema check
ckc runtime validate
ckc fixture load
ckc source ingest
ckc source graph
ckc source closure
ckc observe mech
ckc gen check
ckc gen materialize
ckc gen admit
ckc class build
ckc close
ckc air build
ckc nf build
ckc gloss build
ckc conflict build
ckc verify finite
ckc cert issue
ckc report build
ckc replay
ckc demo m0
```

Each command emits structured diagnostics and immutable artifacts.

Canonical command-to-operation map:

```text
T Command | Pipeline operation | Primary emitted artifacts
ckc schema check | CheckSchemaRegistry | SchemaRegistry, SchemaBoundManifest, UnicodePolicyManifest, schema diagnostics, schema equivalence checks
ckc runtime validate | ValidateRuntimeManifests | ToolchainManifest, EnvironmentProfile, runtime manifest diagnostics
ckc fixture load | LoadFiniteFixtureManifest | FiniteFixtureManifest, fixture manifest diagnostics
ckc source ingest | IngestSourceEdition | SourceEdition, SourcePermissionRecord, CorpusDocument, ExtractionManifest
ckc source graph | BuildSourceGraph | SourceGraph, SourceSpan, SourceAnchor, source diagnostics
ckc source closure | source_region_closure | SourceRegion, RegionClosureCertificate, closure residuals
ckc observe mech | ObserveMech | AnalyzerManifest, MechanicalLexicon, MechObsPayload, extraction residuals
ckc gen check | ParseCKCGen and T-GEN-Static | canonical CKCGen, GeneratorGrammarArtifact, diagnostics
ckc gen materialize | MaterializeGenerators | MaterializedConsequenceManifest, proposal diagnostics
ckc gen admit | DischargeProposal | AdmissionRecord, EffectDischargeRecord, ReportQuestionTemplate, accepted generator/resource/policy artifacts
ckc class build | BuildMatches and BuildMatchClasses | Match, MatchClass, ClassMember
ckc close | CloseM0 | ClosureInput, ClosureOutput, ClosureBoundCertificate, ProofNode, ProofDAG
ckc air build | BuildAIRCore | LicensedReadingSet, AIRCoreRecord, AIR residuals/ambiguities
ckc nf build | BuildNormalForm | CKCNormalForm, NF diagnostics
ckc gloss build | BuildGloss | GlossTemplate, GlossView, gloss diagnostics
ckc conflict build | BuildM0Theorems | WitnessContext, ConflictTheorem, FactualInconsistencyTheorem, theorem residuals
ckc verify finite | kernel_finite_checker | VerifierWitness, SymbolSourceMap, ConstraintCoreWitness, verifier residuals
ckc cert issue | IssueCertificate | Certificate, certificate diagnostics
ckc report build | BuildReviewReport | ClaimRecord, ReportTraceIndex, ClaimTierSummary, WordingGateRecord, ReviewReport, permission residuals
ckc replay | ReplayIdentity | ReplayManifest, ReplayIdentityCheck
ckc demo m0 | DemoM0 | every Appendix A accepted artifact and replay check
```

Command wrapper convention: every command is a total deterministic wrapper returning `OperationResult`. `CheckSchemaRegistry` runs `T-Registry-Referential-Integrity` and `T-Schema-Equivalence`; `ValidateRuntimeManifests` accepts authored `ToolchainManifest`/`EnvironmentProfile` payloads, validates embedded `ToolRecord` rows; `LoadFiniteFixtureManifest` accepts authored `FiniteFixtureManifest`, validates embedded `FrozenConstant`,`ParsedQuantity`,`DiagnosticTag` rows; `IngestSourceEdition` validates §4.1 schemas/permissions; `BuildSourceGraph` emits §4.2 graph + P-SG diagnostics; `BuildAIRCore`,`BuildNormalForm`,`BuildGloss`,`BuildM0Theorems`,`IssueCertificate`,`BuildReviewReport`,`ReplayIdentity` = algorithms named in §§7.3-9.3 and §1.6; `CloseM0` = §7.1 over one `ClosureInput`. `BuildTerminologyClosure`/`BuildDiagnostics` are explicit CloseM0-internal suboperations, use `ckc close` as canonical command surface. A wrapper that only orchestrates earlier operations returns §1.7 primary status over called operations' emitted artifacts. `ckc demo m0` invokes `DemoM0` (the §3.2 fixture orchestrator), calls `ckc runtime validate`,`ckc fixture load`, and the other canonical command wrappers needed to produce Appendix A.10.

### 11.2 Repository layout

```text
.
├── SPEC.md
├── crates/
│   ├── ckc-cli/
│   ├── ckc-core/
│   ├── ckc-schema/
│   ├── ckc-store/
│   ├── ckc-source/
│   ├── ckc-observe/
│   ├── ckc-gen/
│   ├── ckc-class/
│   ├── ckc-close/
│   ├── ckc-air/
│   ├── ckc-nf/
│   ├── ckc-conflict/
│   ├── ckc-verify/
│   ├── ckc-cert/
│   └── ckc-report/
├── schemas/
├── examples/
│   └── sepsis_beta_lactam/
├── eval/
├── ui/
└── runs/
```

Rust owns accepted semantics, canonical bytes, schema registry, closure, finite checking, certificates, reports, replay. External workers may produce proposal artifacts through explicit manifests; accepted artifacts materialized by Rust commands. A crate is created by the first build unit using it; listed layout = reserved namespace, not mandatory upfront scaffold.

### 11.3 Build-unit table and acceptance gates

Single canonical build plan and test-family list. Each row = one committable deliverable for one bounded agent session, carrying exactly one acceptance gate; each acceptance gate appears in exactly one row. Later units may rely on earlier units' accepted artifacts.

```text
T Unit|Deliverable|Depends on|Acceptance gate
M0.0.1|Scalar types,rational normalization,string policies,UnicodePolicyManifest fixtures|none|T-Unicode-Idempotency
M0.0.2|Canonical JSON serializer,canonical_sort_key,typed union encoding,hash identity tests|M0.0.1|T-Canonical-Bytes
M0.0.3|SchemaRegistry,collection-bound manifest,registry referential-integrity checker|M0.0.2|T-Registry-Referential-Integrity
M0.0.4|Rust type manifest/generated JSON Schema equivalence,string-policy/source-support binding equivalence|M0.0.3|T-Schema-Equivalence
M0.0.5|ArtifactEnvelope,content-addressed store paths,ToolchainManifest,EnvironmentProfile,ValidateRuntimeManifests,ProducerManifest,ReplayManifest,replay stratum boundary skeleton|M0.0.4|T-Replay-Manifest-Boundary
M0.0.6|CLI command parser,repository layout checks,structured diagnostic writer|M0.0.5|T-CLI-Contract
M0.1.1|SourceEdition,SourcePermissionRecord,CorpusDocument,permission residual projection|M0.0.6|T-Source-Permission
M0.1.2|Fixture SourceGraph,SourceSpan,SourceAnchor,SourceNode,SourceEdge,BBox,source ordering|M0.1.1|T-SourceGraph-Canonical
M0.1.3|source_region_closure,RegionClosureCertificate,table/caption/footnote/cross-reference residuals|M0.1.2|T-Region-Closure
M0.2.1|AnalyzerManifest,MechanicalLexicon,MechObsKind,MechObsPayload schemas,authority invariant,fixture recognizer manifests|M0.1.3|T-Mech-Manifest
M0.2.2|ObserveMech deterministic emission,duplicate collapse,extraction_uncertain residuals,MechObsPayload fixture output|M0.2.1|T-Mech-Determinism
M0.3.1|CKC-GEN canonical JSON schemas,display parser adapter,parse diagnostics|M0.2.2|T-GEN-Parse-Schema
M0.3.2|CKC-GEN static checker for FeaturePath,type,TemplateValue,variable,stage,grounding,effect,builtin support|M0.3.1|T-GEN-Static
M0.3.3|GeneratorGrammarArtifact,FIRST/FOLLOW,parser-state masks,constrained-decoder evidence-discovery contract|M0.3.2|T-GEN-Grammar-Evidence
M0.3.4|eval_term,eval_class_pred,eval_premise,seq,bounded-path,collect,empty,BuiltinEval totality|M0.3.2|T-GEN-Eval-Totality
M0.3.5|MaterializeGenerators over finite snapshots with one-head-per-environment/bound overflow dispatch|M0.3.4|T-GEN-Materialization
M0.3.6|ProposalProvenanceManifest,ProposalRecord,ReviewerRecord,AdmissionRecord,EffectDischargeRecord,accepted-effect discharge|M0.3.5|T-Effect-Discharge
M0.3.7|CounterexampleSuite/MaterializedConsequenceManifest acceptance comparison|M0.3.6|T-Counterexample-Suite
M0.3.8|RetrievalProposalTrace schema,retrieval/analyzer family enums,region/fingerprint checks,evidence-discovery authority invariant|M0.3.7|T-Retrieval-Proposal-Trace
M0.4.1|TerminologyResourceSet stage-10 resources and stage-20 BuildTerminologyClosure with representative maps,functional keys,ambiguity,incoherence emission|M0.3.8|T-Terminology-Closure
M0.4.2|SemanticPolicySet validation,duplicate-key quarantine,ActionSlotSpec keys,ActionTargetRelation keys,OutputExclusion keys,metadata singleton policy|M0.4.1|T-Policy-Coverage
M0.4.3|PatternObs,Match,MatchClass,ClassMember,proof_visible_signature,quotient invariance|M0.4.2|T-Quotient-Invariance
M0.4.4|ProofNode,ProofDAG construction,rule-side-condition checking,reverse dependency index|M0.4.3|T-ProofDAG-Check
M0.4.5|CloseM0 stratified materialization,CloseM0-internal BuildTerminologyClosure/BuildDiagnostics,ClosureOutput,ClosureBoundCertificate,termination recomputation|M0.4.4|T-Closure-Termination
M0.4.6|LicensedReadingSet demand discovery,AIRCore finite-set identity,no_license residual,multiple_readings ambiguity|M0.4.5|T-AIR-Finite-Set
M0.4.7|CKCNormalForm projection,semantic_digest over nf_payload bytes,metadata claims,NF idempotency|M0.4.6|T-NF-Idempotency
M0.4.8|GlossTemplate,GlossView,renderer tables,authority=view_only invariant,drift predicate|M0.4.7|T-Gloss-Drift
M0.5.1|ContextExpr normalization,finite_constraint_check,WitnessContext,ConstraintCoreWitness|M0.4.8|T-Context-Compatible
M0.5.2|normalize_action,targets_overlap,same_normalized_action,ActionSameness witnesses|M0.5.1|T-Action-Sameness
M0.5.3|NormConsequent,FactualConsequent,TableConsequent,table_outputs_compatible,strict factual self-check|M0.5.2|T-Consequent-Compatibility
M0.5.4|ResolutionTheorem schema,ResolutionSet matching,suppressed-conflict negative-control fixture|M0.5.3|T-Resolution-Suppression
M0.5.5|ConflictTheorem builders for the five ConflictKind variants/theorem minimization|M0.5.4|T-Conflict-Fixtures
M0.5.6|FactualInconsistencyTheorem builders for the five FactualInconsistencyKind variants|M0.5.5|T-Factual-Fixtures
M0.5.7|theorem_minimize,dependency_minimize,minimal-core validation,RepairSetSearchTrace schema|M0.5.6|T-Minimality-Check
M0.5.8|kernel_finite_checker dispatch/VerifierWitness production for every KernelSubjectKind|M0.5.7|T-Kernel-Finite-Checker
M0.6.1|Certificate schema,IssueCertificate,VerifyCertificate,all M0CertificateClass obligations|M0.5.8|T-Certificate-Issue
M0.6.2|ClaimRecord,T-Claim-Tiering implementation,template-based wording gate|M0.6.1|T-Claim-Tiering
M0.6.3|ReviewReport,ReportItem ordering,computed report hashes,permission_limited residuals,report projection exactness|M0.6.2|T-Report-Projection
M0.6.4|ReplayIdentity command integration/replay mismatch/unsupported diagnostics over stratum boundary|M0.6.3|T-Replay-Identity
M0.6.5|LoadFiniteFixtureManifest,FiniteFixtureManifest row validation,end-to-end sepsis beta-lactam fixture command,byte-identical repeated run|M0.6.4|T-Demo-M0-Replay
GATED.1|One deferred §3.3 gate evidence object/GateEvidenceRef validation with named backend/baseline enums,selected explicitly per implementation session|M0.6.5|T-Gate-Evidence-Contract
```

Acceptance-gate names resolve through this canonical obligation table. Each gate returns exactly one §1.7 outcome, emits sorted diagnostics on non-success.

```text
T Acceptance gate|Canonical obligation
T-Unicode-Idempotency|§1.4 string-policy idempotency/byte-stability over UnicodePolicyManifest fixtures.
T-Canonical-Bytes|§1.5 per-type serializer injection,declared_type_id-keyed inline identity (type_tagged_payload_bytes),canonical_sort_key totality,tagged-union encoding,repeated hash identity.
T-Registry-Referential-Integrity|§1.1 symbol,FeaturePath,enum,schema,bound,builtin,certificate,CLI,section-anchor resolution.
T-Schema-Equivalence|§1.1 schema revision hashes,string-policy bindings,source-support aliases,union alternatives,collection bounds agree.
T-Replay-Manifest-Boundary|§1.6 ReplayManifest canonical fields,stratum boundary exclusion,ProducerManifest/ToolchainManifest/EnvironmentProfile hashes validate.
T-CLI-Contract|§11.1 command names resolve to one operation and one primary emitted-artifact set or to an explicit CloseM0-internal suboperation.
T-Source-Permission|§4.1 source/permission schemas enforce allowed_artifacts/permission_limited projection.
T-SourceGraph-Canonical|§4.2 SourceGraph construction is byte-stable/satisfies P-SG-total-text,P-SG-total-support,P-SG-canonical,P-SG-permission.
T-Region-Closure|§4.3 source_region_closure reaches the exact finite closure/typed residual with a valid RegionClosureCertificate.
T-Mech-Manifest|§4.4 AnalyzerManifest,MechanicalLexicon,MechObsPayload authority,replay fields validate.
T-Mech-Determinism|§4.4 ObserveMech repeated runs produce identical canonical MechObsPayload sets/residuals.
T-GEN-Parse-Schema|§6.2 ParseCKCGen returns canonical CKCGen bytes/typed parse/schema diagnostics.
T-GEN-Static|§6.2 static checker validates type,FeaturePath,TemplateValue,variable,stage,finite-bound,builtin,effect,grounding conditions.
T-GEN-Grammar-Evidence|§6.2 GeneratorGrammarArtifact is evidence_discovery_only/its FIRST/FOLLOW/mask data self-checks.
T-GEN-Eval-Totality|§6.2 eval_term,eval_class_pred,eval_premise,seq,bounded-path,collect,empty,BuiltinEval return only declared outcomes over finite fixtures.
T-GEN-Materialization|§7.1 MaterializeGenerators enumerates finite environments deterministically/emits one canonical head per satisfying environment.
T-Effect-Discharge|§6.4 DischargeProposal admits accepted artifacts exactly when static,materialization,suite,admission,effect-discharge obligations pass.
T-Counterexample-Suite|§6.4 CounterexampleSuite comparison matches required,forbidden,residual-class,materialized-count obligations.
T-Retrieval-Proposal-Trace|§6.4 retrieval traces resolve retrieval/analyzer family enum values,regions,fingerprints while keeping scores evidence-only unless gated.
T-Terminology-Closure|§5.2 BuildTerminologyClosure computes representatives,indexes,ambiguities,incoherences exactly and rejects unmapped concept_id presence.
T-Policy-Coverage|§5.3 SemanticPolicySet validates semantic keys,retains duplicate-key quarantines,emits missing_policy/incompatible_generator_outputs at consuming sites.
T-Quotient-Invariance|§7.2 BuildMatches/BuildMatchClasses compute the proof-visible quotient/preserve class signatures under proof-invisible perturbations.
T-ProofDAG-Check|§7.2 ProofDAG rule side conditions re-read accepted inputs/match conclusion hashes.
T-Closure-Termination|§7.1 closure bounds/finite-domain cardinalities recompute/prove termination.
T-AIR-Finite-Set|§7.3 LicensedReadingSet/AIRCoreRecord statuses equal finite-set identity over demanded AIR keys.
T-NF-Idempotency|§7.4 normal form projection is deterministic/NF(NF(x))=NF(x)/semantic_digest is nf_payload-only.
T-Gloss-Drift|§7.5 gloss rendering is byte-identical/drift predicate fires exactly on slot-digest/rendering mismatch.
T-Context-Compatible|§8.1 context normalization,compatibility,witness assignment,unsupported diagnostics,minimal constraint cores check.
T-Action-Sameness|§8.2 action normalization,target overlap,discriminating slots,ActionSameness witnesses check.
T-Consequent-Compatibility|§8.3 norm,factual,table,strict factual self-check compatibility outcomes check.
T-Resolution-Suppression|§8.4 ResolutionSet selects admitted ResolutionTheorem fixtures and suppresses the dedicated conflict pair.
T-Conflict-Fixtures|§8.5/Appendix A.7 emit exactly the expected ConflictTheorem set,including single-subject keying and suppressed-pair absence.
T-Factual-Fixtures|§8.6/Appendix A.7 emit exactly the expected FactualInconsistencyTheorem set/no non-firing theorem.
T-Minimality-Check|§8.7 theorem,dependency,constraint-core minimization are inclusion-minimal under the fixed deletion order.
T-Kernel-Finite-Checker|§9.1 kernel_finite_checker dispatch is exhaustive/recomputes every subject outcome and symbol_source_map_hash.
T-Certificate-Issue|§9.2 Certificate issuance/VerifyCertificate obligations hold for every M0CertificateClass and logical_time order.
T-Claim-Tiering|§3.4 claim-tier computation/template wording gates match available evidence.
T-Report-Projection|§9.3 BuildReviewReport emits exactly the sorted projection of valid theorems,residuals,ambiguities,incoherences,replay failures,permission diagnostics and computed report hashes.
T-Replay-Identity|§1.6 ReplayIdentity deterministic recomputation for the unit's declared prior stratum.
T-Demo-M0-Replay|Appendix A/§11.1 ckc demo m0 emit exactly the listed artifact families in byte-identical repeated runs.
T-Gate-Evidence-Contract|§12 common gate evidence validity/the selected gate-specific evidence schema and named backend/baseline enum checks.
```

### 11.4 Required-reading map

Each build unit has one bounded, sufficient reading slice. An agent implementing a row loads the listed sections plus every earlier row's accepted repository artifacts. Appendix slices = fixture obligations.

```text
T Unit|Required sections|Required Appendix A slice
M0.0.1|Title,§0,§1.3,§1.4,§2|A.1 source strings for string-policy fixtures
M0.0.2|§1.1,§1.2,§1.5,§1.7|A.10 hash-identity expectations
M0.0.3|§1.1,§1.2,§2,§3.1|A.10 artifact inventory
M0.0.4|§1.1 schema equivalence paragraphs,§1.4 bindings,§1.5 canonical bytes|A.10 schema/control inventory
M0.0.5|§1.2,§1.6,§1.7,§9.2 logical-time references|A.9/A.10 replay boundary outputs
M0.0.6|§11.1,§11.2,§11.3|A.10 command target
M0.1.1|§4.1,§8.7 Residual,§9.3 permission rows|A.1/A.10 permission_limited expectation
M0.1.2|§1.5,§4.2,§4.4 source references|A.1 source units/A.2 source references
M0.1.3|§4.3,§4.2 edge kinds,§1.1 overflow convention,§8.7 Residual/Diagnostic|A.1 U14-U19/A.10 source-region expectations
M0.2.1|§4.4 schemas,§2 Authority,§1.6 manifests|A.2 observation vocabulary
M0.2.2|§4.4 ObserveMech,§1.5 sorting,§4.3 support closure,§8.7 Residual|A.2 all MO-* rows/A.10 MechObsPayload expectations
M0.3.1|§6.1,§6.2 through display-grammar coverage,§1.5,§1.7|A.4 CKCGen surface instance
M0.3.2|§6.1 constraints,§6.2 T-GEN-Static and TemplateValue type checking,§7.1 finite domains|A.4 generator instance/A.9 PR-PG1 static acceptance
M0.3.3|§6.2 grammar artifact schemas/T-GEN-Grammar-Evidence,§3.5 C8.4|A.10 GeneratorGrammarArtifact expectation
M0.3.4|§6.2 evaluator,sequence,path,collect,empty,builtin definitions,§8.7 diagnostics|A.4 generator instance/A.9 collect_overflow fixture
M0.3.5|§7.1 materialization,§6.3 readings,§6.4 materialized manifest|A.4 license table/A.5 license shorthands
M0.3.6|§6.4 proposal provenance,proposal,admission,reviewer,effect-discharge schemas,§8.7 Residual,§9.2 logical time|A.9 PR-PG1..PR-PG5
M0.3.7|§6.4 CounterexampleSuite/MaterializedConsequenceManifest|A.9 counterexample witnesses
M0.3.8|§6.4 RetrievalProposalTrace/T-Retrieval-Proposal-Trace and retrieval/analyzer family enums,§3.3 G-RET-PARITY boundary,§3.5 C7.1-C7.3|A.10 empty RetrievalProposalTrace expectation
M0.4.1|§5.1,§5.2,§2.1 terminology consumers,§8.7 residual/ambiguity/incoherence schemas|A.3 terminology rows/A.5 INC-* rows
M0.4.2|§5.3,§8.2 policy consumers,§8.3 output policy,§8.7 Incoherence|A.3 policy rows/dose_policy_collision and missing_policy notes
M0.4.3|§7.2 Match schemas/proof_visible_signature,§6.2 pattern schemas|A.4 PO-P1..PO-P17/match/class rows
M0.4.4|§7.2 ProofRule/JudgmentKind table,§9.1 proof checks|A.10 ProofNode/ProofDAG expectations
M0.4.5|§7.1 closure algorithm,§3.2 stages,§1.1 bounds|A.10 ClosureOutput/ClosureBoundCertificate expectations
M0.4.6|§7.3,§6.3 AIRKey/License,§2 AirType/Outcome,§8.7 Residual/Ambiguity|A.5 AIRCore residual/ambiguity rows
M0.4.7|§7.4,§8.1 normalization dependencies,§5.2 representatives|A.6 all NF rows
M0.4.8|§7.5,§4.1 permission note for views|A.8/A.10 gloss expectations
M0.5.1|§8.1,§6.2 normalize_context builtin,§7.2 ConstraintCoreWitness proof rule,§8.7 DiagnosticRef|A.7 CT-C3,CT-C4,NFC-NFALSE7
M0.5.2|§8.2,§5.3 ActionSlotSpec/ActionTargetRelation,§8.7 Residual/Incoherence|A.7 AW-AS1..AW-AS4
M0.5.3|§8.3,§5.3 OutputExclusion,§8.1 finite_constraint_check,§8.7 witnesses|A.7 FI-F1,CT-C3,NFC-NFALSE rows
M0.5.4|§8.4,§8.5 suppression micro-example,§8.7 computed theorem fields|A.7 RT-R1 and NFC-NFALSE8
M0.5.5|§8.4,§8.5,§8.7 minimization schemas|A.7 conflict rows plus N1-vs-N3 suppressed-pair control
M0.5.6|§8.4,§8.6,§8.7 factual theorem schema|A.7 FI-F1..FI-F5/NFC-NFALSE rows
M0.5.7|§8.7 theorem_minimize,dependency_minimize,RepairSetSearchTrace|A.7 minimal witness text/A.10 empty repair-set expectation
M0.5.8|§9.1,§7.2 proof-rule side conditions,§8 all predicates|A.9 verifier_unsupported fixture/A.10 VerifierWitness expectations
M0.6.1|§9.2,§3.4 ClaimRecord references,§1.6 replay references|A.10 certificate expectations
M0.6.2|§3.4,§9.2,§9.3 wording gate fields|A.10 claim-tier/report expectations
M0.6.3|§9.3,§4.1 permission semantics,§8.7 classification assignment|A.10 ReviewReport/permission_limited outputs
M0.6.4|§1.6,§9.3 replay references,§9.2 report_replay certificate|A.10 replay_identity_pass/mismatch/unsupported expectations
M0.6.5|§11.1,§11.3,all prior sections by artifact hash,Appendix A all|Appendix A complete
GATED.1|§3.3,§3.4,§12 including local evidence enums,the specific consuming section that triggered the gate|Appendix slice named by the selected gate fixture when one exists
```

First complete command target:

```text
ckc demo m0 --out runs/m0
```

Running the command twice produces identical accepted artifact hashes.

## 12. Deferred extension contracts

Deferred capability affects accepted M0 outputs only after its §3.3 evidence object validates, replays, and is referenced by a `ClaimRecord` or admitted resource.

```text
S GateEvidenceRef(gate:Gate,evidence_object_hash:Hash,subject_hash:Hash,replay_identity_hash:Hash,enabled_claims:Set[Id])

S GateEvidenceCommon(evidence_id:Id,gate:Gate,subject_hash:Hash,evidence_input_hashes:Set[Hash],schema_registry_hash:Hash,validation_manifest_hash:Hash,replay_manifest_hash:Hash,replay_identity_hash:Hash,enabled_claims:Set[Id],limitation_ids:Set[Id],falsification_criteria_hash:Hash,proof_roots:Set[ProofId],accepted_effect_row:Set[Effect])
```

Gate evidence object valid when:

```text
1 Schema validates under SchemaRegistry.
2 common.gate equals the triggered gate.
3 common.subject_hash names the enabled artifact, source profile, population, pipeline, or deployment profile.
4 common.replay_identity_hash resolves to replay_identity_pass.
5 Every enabled ClaimRecord.evidence_hashes includes this object.
6 Metrics, scores, adjudications, proofs, and external outputs are replayable or non-authoritative metadata.
7 Missing or invalid evidence emits Residual(class=deferred_gate_required).
```

`ValidateGateEvidenceRef(ref) -> OperationResult[GateEvidenceCommon]`: resolve `ref.gate`, load `ref.evidence_object_hash`, require §3.3 evidence type, `object.common.subject_hash = ref.subject_hash`, `object.common.replay_identity_hash = ref.replay_identity_hash`, `ref.enabled_claims ⊆ object.common.enabled_claims`; then apply common and gate-specific checks.

Gate interfaces:

```text
S ExtractorAdapterRecord(common:GateEvidenceCommon,source_profile_id:Id,extractor_families:Set[ExtractorFamily],adapter_manifest_hash:Hash,extraction_toolchain_hash:Hash,ocr_model_manifest_hash:Hash?,layout_model_manifest_hash:Hash?,table_structure_model_manifest_hash:Hash?,reading_order_policy_hash:Hash,golden_source_hashes:Set[Hash],expected_source_graph_hashes:Set[Hash],observed_source_graph_hashes:Set[Hash],region_closure_check_hashes:Set[Hash],text_totality_check_hashes:Set[Hash],table_layout_check_hashes:Set[Hash],footnote_crossref_check_hashes:Set[Hash],roundtrip_rerender_diff_hashes:Set[Hash],residual_policy_hash:Hash)

E ExtractorFamily = deterministic_fixture | yomitoku | mineru | marker | layoutlmv3 | doclayout_yolo | table_transformer | paddleocr | tesseract | other
```

`G-EXTRACTOR-ADAPTER`: source-profile extractor soundness. Interface `SourceEdition -> ExtractionManifest -> SourceGraph -> MechObsPayload*`; evidence proves byte-stable graph construction, text totality, reading order, layout/table support, caption/footnote/cross-reference closure, round-trip render comparison when source images used, residuals.

```text
S RetrievalParityReport(common:GateEvidenceCommon,corpus_hash:Hash,query_set_hash:Hash,qrels_hash:Hash,reference_implementation:RetrievalReferenceImplementation?,sparse_baseline_family:SparseRetrieverFamily?,sparse_index_fingerprint_hash:Hash?,dense_baseline_family:DenseRetrieverFamily?,dense_index_fingerprint_hash:Hash?,late_interaction_baseline_family:LateInteractionFamily?,late_interaction_index_fingerprint_hash:Hash?,graph_index_fingerprint_hash:Hash?,fusion_policy_family:FusionPolicyFamily?,fusion_policy_hash:Hash?,reranker_family:RerankerFamily?,reranker_manifest_hash:Hash?,japanese_analyzer_family:JapaneseAnalyzerFamily?,analyzer_manifest_hashes:Set[Hash],segmentation_policy_hash:Hash,metric_records:Set[RetrievalMetricRecord],citation_precision_record_hash:Hash?,failure_slice_hashes:Set[Hash])

S RetrievalMetricRecord(metric_id:Id,metric_name:Id,metric_family:EvaluationMetricFamily,query_slice_id:Id,value:Rational,confidence_interval_hash:Hash?,computation_manifest_hash:Hash)

E RetrievalReferenceImplementation = pyserini | anserini_lucene | opensearch | elasticsearch | vespa | qdrant | weaviate | other
```

`G-RET-PARITY`: retrieval-quality claims only. Interface `RetrievalProposalTrace* -> RetrievalParityReport -> ClaimRecord`; traces propose regions, accepted artifacts still require `DischargeProposal` and proof checking. Falsifiable sparse baseline names BM25/Lucene-family implementation, analyzer family, index fingerprint, qrels, metric manifest; hybrid/dense/late-interaction/graph/reranking claims add their named family fields, not replacing the sparse baseline.

```text
S VerifierPortfolioReport(common:GateEvidenceCommon,theorem_or_pipeline_hashes:Set[Hash],backend_records:Set[VerifierBackendRecord],agreement_matrix_hash:Hash,proof_translation_hashes:Set[Hash],unsupported_fragment_hashes:Set[Hash],divergence_diagnostic_hashes:Set[Hash])

S VerifierBackendRecord(backend_id:Id,backend_family:VerifierBackendFamily,backend_manifest_hash:Hash,input_translation_hash:Hash,result_hash:Hash,proof_certificate_hash:Hash?,trusted_base_manifest_hash:Hash)

E VerifierBackendFamily = lean | rocq | isabelle | why3 | smt | sat | datalog | asp | prolog | owl_reasoner | model_checker | tla | alloy | cp_sat | minizinc | prob_model_checker | egraph | cql | fhirpath | saw | crosshair | other
```

`G-PORTFOLIO`: independent-backend agreement. Interface `KernelFiniteCheckInput -> backend translations -> VerifierPortfolioReport -> ClaimRecord`; M0 validity stays the internal kernel result unless a later gated profile elevates portfolio agreement.

```text
S AIRDomainRecord(common:GateEvidenceCommon,domain_id:Id,domain_family:AIRDomainFamily,domain_kind:Id,carrier_schema_hash:Hash,order_definition_hash:Hash,join_definition_hash:Hash,meet_definition_hash:Hash?,bottom_hash:Hash?,top_hash:Hash?,alpha_definition_hash:Hash,gamma_definition_hash:Hash,soundness_theorem_hashes:Set[Hash],widening_policy_hash:Hash?,termination_bound_hash:Hash,counterexample_suite_hash:Hash)

E AIRDomainFamily = abstract_interpretation | owl_classification | shacl_rules | ontology_module | argumentation_dung | aspic | carneades | assumption_based | egraph | allen_relation_algebra | event_calculus | temporal_logic | mcda | other
```

`G-AIR-FULL`: non-identity abstract-interpretation, argumentation, ontology-derived, or equality-saturation domains. Interface `LicensedReadingSet -> AIRDomainRecord -> AIRCoreRecord`; absent evidence leaves finite-set identity.

```text
S RebindingEvidence(common:GateEvidenceCommon,old_source_edition_hash:Hash,new_source_edition_hash:Hash,diff_families:Set[VersionDiffFamily],region_alignment_hash:Hash,text_diff_hash:Hash,terminology_diff_hash:Hash,semantic_impact_hash:Hash,transported_proof_hashes:Set[Hash],rejected_transport_hashes:Set[Hash],residual_hashes:Set[Hash])

E VersionDiffFamily = source_text | terminology_version | ontology_axiom | fhir_artifact | code_crosswalk | guideline_supersession | other
```

`G-REBIND`: source-edition, terminology-version, ontology-version proof transport. Interface `old artifact + new SourceGraph + alignment -> RebindingEvidence -> transported or residual artifact`.

```text
S BenchmarkRelease(common:GateEvidenceCommon,release_id:Id,corpus_profile_id:Id,item_granularity:BenchmarkItemGranularity,source_permission_hashes:Set[Hash],item_hashes:Set[Hash],split_manifest_hash:Hash,stratification_manifest_hash:Hash,annotation_schema_hash:Hash,adjudication_protocol_hash:Hash,adjudication_metric_families:Set[AdjudicationMetricFamily],clinician_reviewer_hashes:Set[Hash],formalist_reviewer_hashes:Set[Hash],inter_annotator_agreement_hashes:Set[Hash],gold_ir_hashes:Set[Hash],gold_ir_conformance_hashes:Set[Hash],conformance_suite_hashes:Set[Hash],contradiction_fixture_hashes:Set[Hash],negative_control_hashes:Set[Hash])

S EMinReport(common:GateEvidenceCommon,benchmark_release_hash:Hash,system_under_test_hash:Hash,task_profile_id:Id,metric_records:Set[EvaluationMetricRecord],equivalence_backend_families:Set[VerifierBackendFamily],semantic_equivalence_hashes:Set[Hash],idempotency_record_hashes:Set[Hash],convergence_record_hashes:Set[Hash],metamorphic_test_hashes:Set[Hash],japanese_judge_calibration_hash:Hash?,error_taxonomy_hash:Hash,failure_case_hashes:Set[Hash])

S EvaluationMetricRecord(metric_id:Id,metric_name:Id,metric_family:EvaluationMetricFamily,value:Rational,denominator:UInt,slice_id:Id,computation_manifest_hash:Hash)

E BenchmarkItemGranularity = source_passage | clinical_question | recommendation | pico_field | evidence_table_row | synthetic_fixture | patient_scenario

E AdjudicationMetricFamily = cohens_kappa | fleiss_kappa | krippendorff_alpha | gamma_agreement | f1 | likert | other

E EvaluationMetricFamily = recall_at_k | mrr | ndcg | faithfulness | citation_precision | citation_recall | semantic_equivalence | idempotency | convergence | metamorphic_violation | auroc | calibration | subgroup_fairness | override_rate | other
```

`G-EMIN`: S2 research measurements, contradiction benchmarks, convergence, conformance, corpus evaluation. Interface `BenchmarkRelease + replayed run -> EMinReport -> ClaimRecord`.

```text
S MDLEvidence(common:GateEvidenceCommon,model_class_hash:Hash,corpus_hash:Hash,preference_model_family:PreferenceModelFamily,baseline_description_length:Rational,candidate_description_length:Rational,residual_description_length:Rational,scoring_rule_hash:Hash,pareto_front_hash:Hash,calibration_record_hash:Hash?,heldout_record_hash:Hash?)

E PreferenceModelFamily = none | weighted_sum | ahp | electre | promethee | topsis | grade_etd | other
```

`G-MDL`: calibrated compression, payoff, model-selection, preference tradeoffs. Interface `candidate artifact set -> MDLEvidence -> ClaimRecord`; M0 theorem truth ignores compression score.

```text
S SelfImprovementEvidence(common:GateEvidenceCommon,parent_artifact_hashes:Set[Hash],proposed_child_artifact_hashes:Set[Hash],change_set_hash:Hash,adapter_family:ModelAdapterFamily,counterexample_suite_hash:Hash,materialized_consequence_manifest_hash:Hash,regression_suite_hash:Hash,heldout_delta_hash:Hash?,catastrophic_forgetting_check_hash:Hash?,safety_regression_check_hash:Hash?,reviewer_record_hashes:Set[Hash],rollback_manifest_hash:Hash)

E ModelAdapterFamily = none | lora | qlora | dora | full_finetune | prompt_update | retrieval_index_update | rule_update | terminology_update | other
```

`G-SELF-IMPROVE`: automated generator/resource/policy improvement. Interface `proposal -> deterministic materialization -> regression discharge -> AdmissionRecord`; accepted artifacts remain effect-free.

```text
S ProbabilisticProfileRecord(common:GateEvidenceCommon,probabilistic_model_hash:Hash,random_variable_schema_hash:Hash,distribution_manifest_hash:Hash,backend_family:ProbabilisticBackendFamily,inference_backend_manifest_hash:Hash,seed_or_determinization_hash:Hash,checkable_explanation_kind:ProbabilisticEvidenceArtifactKind?,checkable_explanation_hash:Hash?,weighted_model_count_manifest_hash:Hash?,calibration_dataset_hash:Hash?,probability_claim_hashes:Set[Hash],decision_threshold_policy_hash:Hash?,uncertainty_report_hash:Hash)

E ProbabilisticBackendFamily = problog | cplint | prism_sato | deepproblog | smproblog | probec | prism_mc | storm | other

E ProbabilisticEvidenceArtifactKind = sdd | d_dnnf | bdd | mtbdd | weighted_model_count_circuit | markov_chain | mdp | proof_log | other
```

`G-PROB`: probabilistic facts, risks, stochastic transitions, weights, rewards. Interface `probabilistic evidence -> ProbabilisticProfileRecord -> ClaimRecord`; deterministic M0 predicates ignore probability fields. ProbLog/cplint/ProbEC-style exact-inference claims record distribution-semantics backend and a checkable SDD/d-DNNF/WMC artifact when claimed; PRISM/Storm model-checking claims record model-checker backend and transition/reward model artifact.

```text
S WorldModelProfileRecord(common:GateEvidenceCommon,observation_modality_set:Set[Id],world_model_family:WorldModelFamily,latent_state_schema_hash:Hash,transition_model_hash:Hash?,multimodal_encoder_manifest_hash:Hash?,trajectory_dataset_hash:Hash?,rollout_horizon_bound_hash:Hash?,causal_design_manifest_hash:Hash?,validation_protocol_hash:Hash,safety_boundary_hash:Hash,unsupported_projection_hashes:Set[Hash])

E WorldModelFamily = observation_generative | latent_dynamics | jepa_encoder | tokenized_ehr_transformer | multimodal_encoder | other
```

`G-WORLD-MODEL`: latent state, trajectory, image-derived, multimodal claims. Interface `world observation -> WorldModelProfileRecord -> gated feature artifact`; M0 source-text review stays independent.

```text
S GovernedPatientDataProfile(common:GateEvidenceCommon,data_source_profile_id:Id,data_model_family:PatientDataModelFamily,privacy_regime_family:PrivacyRegimeFamily,data_use_authority_hash:Hash,consent_or_optout_policy_hash:Hash?,cross_border_transfer_profile_hash:Hash?,privacy_law_assessment_hash:Hash,deidentification_family:DeidentificationFamily?,deidentification_profile_hash:Hash?,record_linkage_profile_hash:Hash?,dataset_bom_hash:Hash?,access_control_profile_hash:Hash,audit_log_profile_hash:Hash,data_minimization_hash:Hash,retention_policy_hash:Hash,breach_response_hash:Hash)

E PatientDataModelFamily = jp_core | ss_mix2 | omop | mid_net | ndb | local_ehr | claims | registry | other

E PrivacyRegimeFamily = appi | next_generation_medical_infrastructure_act | hipaa | gdpr_ehds | local_contract | other

E DeidentificationFamily = none | anonymized | pseudonymized | k_anonymity | differential_privacy | pprl | secure_on_site_analysis | other
```

`G-LIVE-PATIENT`: live, deidentified, linked, claims, registry, or real-world patient-data handling. Interface `patient-data source -> GovernedPatientDataProfile -> gated patient-context artifact`; absent evidence yields `deferred_gate_required`.

```text
S S3AssuranceEvidence(common:GateEvidenceCommon,assurance_case_hash:Hash,assurance_case_family:AssuranceCaseFamily,gsn_or_sacm_model_hash:Hash?,top_goal_hash:Hash,context_hashes:Set[Hash],assumption_hashes:Set[Hash],strategy_hashes:Set[Hash],solution_evidence_hashes:Set[Hash],defeater_hashes:Set[Hash],risk_management_file_hash:Hash,software_lifecycle_file_hash:Hash,usability_engineering_file_hash:Hash,regulatory_classification_hash:Hash,regulatory_jurisdiction_families:Set[RegulatoryJurisdictionFamily],ai_management_system_hash:Hash?,privacy_governance_hash:Hash?,threat_model_families:Set[ThreatModelFamily],threat_model_hash:Hash?,sbom_hash:Hash?,aibom_hash:Hash?,reproducible_build_hash:Hash?,pccp_or_idaten_change_protocol_hash:Hash?,deployment_policy_hash:Hash?,observability_profile_hash:Hash?,drift_monitoring_profile_hash:Hash?,incident_response_plan_hash:Hash?,post_market_surveillance_hash:Hash?,human_factors_validation_hash:Hash?,clinical_validation_hash:Hash?,residual_risk_acceptance_hash:Hash)

E AssuranceCaseFamily = gsn | sacm | ontogsn | assurance_2_0 | d_case | other

E ThreatModelFamily = stride | linddun | zero_trust | mitre_atlas | owasp_llm | other

E RegulatoryJurisdictionFamily = japan_pmda | us_fda | eu_mdr_ivdr_ai_act | imdrf | other
```

`G-S3`: clinical, patient-care, CDS, SaMD, deployment, regulatory, safety, cybersecurity, privacy, usability, alert-governance, implementation-science, post-market authority. Interface `deployment profile -> S3AssuranceEvidence -> S3 ClaimRecord`. Without `G-S3`, M0 outputs stay formalization-QA and text-quality review candidates.

```text
S RuntimeOracleReport(common:GateEvidenceCommon,pipeline_id:Id,target_language_family:RuntimeTargetFamily,deterministic_oracle_hash:Hash,stage_records:List[LayeredPipelineStageRecord],between_layer_model_families:Set[ProposalGeneratorFamily],direct_translation_baseline_hash:Hash?,weak_model_baseline_hash:Hash?,fidelity_metric_records:Set[EvaluationMetricRecord],idempotency_record_hashes:Set[Hash],convergence_record_hashes:Set[Hash],ablation_record_hashes:Set[Hash],divergence_case_hashes:Set[Hash])

S LayeredPipelineStageRecord(stage_index:UInt,input_ir_schema_hash:Hash,output_ir_schema_hash:Hash,transform_kind:LayeredTransformKind,model_family:ProposalGeneratorFamily?,stage_manifest_hash:Hash,stage_output_digest:Hash)

E RuntimeTargetFamily = ckc_normal_form | lean | rocq | isabelle | smtlib | datalog | asp | egraph | symbolic_math | bespoke_dsl | other

E LayeredTransformKind = deterministic_compile | model_call | retrieval_augmented | constrained_decode | verifier_repair | human_fixture | other
```

`G-RUNTIME-ORACLE`: layered-IR pipeline and runtime-oracle fidelity measurement. Interface `source text -> ordered IR stages (optionally between-layer model calls) -> target language -> RuntimeOracleReport -> ClaimRecord`. Candidate NL-to-target pipeline as first-class replayable object: `stage_records` = ordered IR layers, each naming input/output IR schema, `transform_kind` (`deterministic_compile`, `model_call`, retrieval, constrained decode, verifier repair) and `model_family` for a between-layer model call; `deterministic_oracle_hash` names the accepted CKC Normal Form / kernel result scored against; `direct_translation_baseline_hash`, `weak_model_baseline_hash` exhibit the hypothesis that a weak model fails direct clinical-text-to-target translation but succeeds through a specific staged setup; `ablation_record_hashes` = combinatorial search over stage compositions; `idempotency_record_hashes`, `convergence_record_hashes`, `fidelity_metric_records` quantify stability/accuracy. Never elevates a model-bearing pipeline to accepted M0 semantics: accepted artifacts stay deterministic kernel output; a layered/runtime-AI pipeline is admissible at runtime only as separately gated, replayed S2 evidence, per §6.4 proposal/runtime separation.

## 13. Roadmap, theoretical scope, and open problems

Non-normative for M0: no acceptance gate, no M0 obligation. Core §§1-12 is the first milestone. Research tracks produce evidence only through §3.3 gates; none weakens the kernel; accepted M0 semantics = kernel output.

### 13.1 Reusability and minimum description length as the optimization target

Aim: fewest reusable components — clinical-language-to-IR mapping verified once, reused deterministically. Reusable-component set = `AcceptedGeneratorBase + TerminologyResourceSet + SemanticPolicySet`; target = its size/overlap vs licensed corpus coverage. M0 instruments descriptively via §10 component-economy annotations → deterministic gradient. Optimization verdict (one base more compact/reusable/lower-description-length) is a calibrated S2 claim gated by `G-MDL` and `G-EMIN`. Automated base improvement additionally gated by `G-SELF-IMPROVE`; accepted improved artifacts stay effect-free, proof-carrying.

### 13.2 The intermediate-representation design space and the layered-IR / runtime-oracle track

M0 commits to a single accepted IR (CKC-GEN-core licenses to AIRCore to CKC Normal Form) checked by proof-by-reflection; external IR/proof-assistant/solver/agent languages dispositioned in §3.5, admitted only as gated portfolio evidence (`G-PORTFOLIO`, `G-AIR-FULL`). Open: whether the most reusable/accurate mapping uses a different IR, pre-existing IRs/proof-assistants/agent-languages, newly invented IRs, or several IR layers with model calls between them — a combinatorial search. `G-RUNTIME-ORACLE` (§12) is its replayable home: `RuntimeOracleReport` records an ordered IR-stage pipeline with optional between-layer model calls, scored against the kernel oracle, with direct-translation/weak-model baselines and ablations — mechanism for the hypothesis that a weak local model fails direct clinical-text-to-target (e.g. Lean) translation but succeeds through a specific staged setup. Promoted to runtime use only when such evidence proves it; until then proposal-only (§6.4), accepted M0 output stays deterministic.

### 13.3 Corpus scale

`ckc demo m0` runs over the single finite Appendix A fixture. First external deliverable: headless processing of a large corpus of publicly retrievable Japanese clinical guidelines into a review report. Staged bridge: layout-aware extraction of public guideline PDFs via `G-EXTRACTOR-ADAPTER`; autonomous retrieval/segmentation quality via `G-RET-PARITY`; cross-edition transport on revision via `G-REBIND`; corpus-scale coverage/contradiction measurements are S2 under `G-EMIN`. Immutable permission-tracked source editions + replayable artifacts scale the same deterministic pipeline from one fixture to a corpus without changing acceptance authority. Only publicly accessible, autonomously retrievable sources with no patient-data/security considerations are in scope before `G-LIVE-PATIENT`, `G-S3`.

### 13.4 Theoretical scope and limits

Reducing clinical knowledge to a small axiom set is a research direction, not assumed. M0 is finite and decidable: contexts finite disjunctions of finite conjunctions, AIR finite-set identity, `finite_constraint_check` decides linear rational and interval constraints, theorem truth proof-by-reflection over canonical finite artifacts. Finiteness escapes classical limits — Gödel incompleteness and undecidability of rich first-order/arithmetic theories apply to recursively axiomatized systems strong enough to encode arithmetic, not a fixed finite decidable fragment. As scope grows, expressive fragments (full temporal logics, first-order ontologies, quantified arithmetic, defeasible/argumentation closures) can become undecidable or incomplete; no complete consistent finite axiomatization assumed. Hence richer-reasoning, optimality, full-coverage claims are gated (`G-AIR-FULL`, `G-PORTFOLIO`, `G-EMIN`, `G-MDL`) not asserted; accepted core stays decidable and mechanically replayable.

### 13.5 The open verification problem

Open, resolved empirically: what counts as adequate evidence that a clinical-language-to-IR mapping is correct and reusable. M0 supplies operational verification of M0 artifacts — schema/canonical-byte validity, proof-by-reflection by the kernel finite checker, certificates, replay identity — plus generator discharge against counterexample suites; not claimed final. Candidate signals, each gated: replay identity + proof-checking (S0); admission + counterexample-suite discharge (S1); multi-backend portfolio agreement (`G-PORTFOLIO`); non-identity abstract-domain soundness (`G-AIR-FULL`); cross-edition rebinding stability (`G-REBIND`); benchmark fidelity, semantic equivalence, idempotency, convergence (`G-EMIN`); compression/description-length payoff (`G-MDL`); layered-pipeline runtime-oracle fidelity (`G-RUNTIME-ORACLE`). Which combination is adequate is left open, itself a benchmarkable S2 question; each signal kept replayable.

### 13.6 Visualization

§9.4 derivation-trace view renders, deterministically and `view_only`, the path from a source region through each IR (`SourceGraph -> MechObsPayload -> PatternObs -> Match -> License -> AIRCoreRecord -> CKCNormalForm`) along the `ProofDAG` to the most compact accepted form. When the §13.2 layered-IR track is exercised, the trace extends through the additional IR stages recorded in a `RuntimeOracleReport` to the compiled target. Presentation layer over already-proof-visible artifacts; adds no semantics.

## Appendix A. Worked M0 example: sepsis and beta-lactam

Canonical M0 fixture; finite, mechanically checkable. All IDs are fixture IDs. Shorthand expands to spec schemas: concept names→`TermReading` refs, quantity surfaces→exact `Rational`, support labels→closed `SourceRegion` hashes, Japanese surfaces→declared string policies. In Appendix A: `GDL=SRC-GDL`, `PI=SRC-PI`, `Ux=REG-Ux`, `Lx=LIC-Lx`, `Nx=NF-Nx`, `C=adult_population ∧ suspected(sepsis)`, `H=history(anaphylaxis_to(beta_lactam_antibacterial))`, `AB=administer(beta_lactam_antibacterial)`, `supp(Ux)=source_support=closure(REG-Ux)`.

### A.1 Fixture sources, permissions, and regions

```text
SRC-GDL class=guideline bid="敗血症抗菌薬ガイド fixture" publication_date="2026-01-01" redistribution_status=reconstructable allowed={source_graph,offsets_only,hashes_only,derived_labels}.
SRC-PI class=package_insert bid="敗血症抗菌薬ガイド fixture" publication_date="2025-12-01" redistribution_status=restricted_internal_only allowed={hashes_only,derived_labels}.
CDOC-GDL corpus document for SRC-GDL. CDOC-PI corpus document for SRC-PI.
ART-ST1 stale_internal_artifacts={one stale GlossView for NF-N1, one mutated certificate copy, one deliberately unsupported theorem witness}.
```

```text
U1 GDL "成人の敗血症が疑われる場合，βラクタム系抗菌薬を速やかに投与することを推奨する。"
U2 PI "βラクタム系抗菌薬に対するアナフィラキシーの既往がある患者には投与しない。"
U3 GDL "バイタル判定表: 収縮期血圧 < 90 なら緊急対応; 収縮期血圧 <= 90 なら通常対応。"
U4 GDL "成人の敗血症が疑われる場合，βラクタム系抗菌薬を避けることを推奨する。"
U5 GDL "βラクタム投与記録には出典ノードを必ず付与する。"
U6 GDL "βラクタム投与記録では出典ノードを空欄にする。"
U7 GDL "同一トリアージ条件は，収縮期血圧 < 90 かつ 収縮期血圧 >= 90 とする。"
U8 GDL "薬剤Xを投与する。"
U9 GDL "成人の敗血症が疑われる場合，βラクタム系抗菌薬を静脈内で速やかに投与することを推奨する。"
U10 GDL "成人の敗血症が疑われる場合，βラクタム系抗菌薬を経口で速やかに投与することを推奨する。"
U11 GDL "成人の敗血症が疑われる場合，βラクタム系抗菌薬を静脈内で通常速度で投与することを推奨する。"
U12 GDL "成人の敗血症が疑われる場合，セファゾリンを投与することを推奨する。"
U13 GDL "成人の敗血症が疑われる場合，βラクタム系抗菌薬を投与することを推奨する。"
U14 GDL "表1「バイタル判定表」を参照する。"
U15 GDL "表1 キャプション: バイタル判定表。"
U16 GDL "βラクタム系抗菌薬*。脚注*: 本例では fixture 用語である。"
U17 GDL "腎機能に応じてβラクタム系抗菌薬の用量を調整する。"
U18 GDL "表99「存在しない表」を参照する。"
U19 GDL "未整形表: 収縮期血圧 < 80; 出力列なし。"
U20 GDL "成人の敗血症が疑われる場合，未知薬Yを投与する。"
U21 GDL "成人の敗血症が疑われる場合，βラクタム系抗菌薬を試験経路で投与する。"
U22 GDL "画像由来の不鮮明な注記「投与量未確認」。"
U23 GDL "患者データから敗血症リスクを予測して抗菌薬を選択する。"
U24 GDL "成人の敗血症が疑われる場合，βラクタム系抗菌薬の投与を考慮し，状況により避ける。"
U25 GDL "表在感染を合併する場合は追加評価する。"
U26 GDL "βラクタム系抗菌薬の投与量を高用量とする。"
U27 GDL "βラクタム系抗菌薬の投与量を標準用量とする。"
```

`U18`: no `crossref_targets` edge. `U19`: table-like surface, no output column, no header relation. `U22`: conflicting byte offsets. Force typed residuals, not source drop.

### A.2 Mechanical observations

```text
MO-TEXT-U1 text_node kind=sentence normalized="成人の敗血症が疑われる場合,βラクタム系抗菌薬を速やかに投与することを推奨する." src=U1.
MO-ANCH-BLA-U1 anchor_span raw="βラクタム系抗菌薬" src=U1.
MO-TOK-BLA-U1 token raw="βラクタム" src=U1.
MO-LEX-BLA-U1 lex_surface_hit surface="βラクタム系抗菌薬" concept_candidate=beta_lactam_antibacterial src=U1.
MO-MOD-REC-U1 modality_marker raw="推奨する" kind=recommend src=U1.
MO-NEG-U2 negation_marker raw="しない" src=U2.
MO-TEMP-PROMPT-U1 temporal_surface raw="速やかに" shape=prompt src=U1.
MO-CELL-G1 table_cell row=DTR-T1-R1 column=guard raw="収縮期血圧 < 90" src=U3.
MO-CELL-O1 table_cell row=DTR-T1-R1 column=output raw="緊急対応" src=U3.
MO-CELL-G2 table_cell row=DTR-T1-R2 column=guard raw="収縮期血圧 <= 90" src=U3.
MO-CELL-O2 table_cell row=DTR-T1-R2 column=output raw="通常対応" src=U3.
MO-EDGE-HEADER-C1 table_edge kind=header_of from=header_bp to=MO-CELL-G1 src=U3.
MO-EDGE-CAPTION caption_edge kind=caption_of from=U15 to=U3 src=U15.
MO-XREF-T1 crossref_surface raw="表1" target=U3 src=U14.
MO-FOOTNOTE-F1 footnote_surface marker="*" body="本例では fixture 用語である" src=U16.
MO-Q-LT90 quantity_surface raw="< 90" cmp=lt number=90 unit="mmHg" src=U3.
MO-Q-LE90 quantity_surface raw="<= 90" cmp=le number=90 unit="mmHg" src=U3.
MO-MOD-REQ-U5 modality_marker raw="必ず" kind=require src=U5.
MO-Q-GE90 quantity_surface raw=">= 90" cmp=ge number=90 unit="mmHg" src=U7.
MO-ANCH-IV anchor_span raw="静脈内" src=U9.
MO-LEX-IV lex_surface_hit surface="静脈内" concept_candidate=intravenous_route src=U9.
MO-LEX-RAPID lex_surface_hit surface="速やかに" concept_candidate=rapid_administration src=U9.
MO-ANCH-ORAL anchor_span raw="経口" src=U10.
MO-LEX-ORAL lex_surface_hit surface="経口" concept_candidate=oral_route src=U10.
MO-ANCH-SPEED anchor_span raw="通常速度" src=U11.
MO-LEX-ROUTINE lex_surface_hit surface="通常速度" concept_candidate=routine_administration src=U11.
MO-ANCH-CFZ anchor_span raw="セファゾリン" src=U12.
MO-LEX-CFZ lex_surface_hit surface="セファゾリン" concept_candidate=cefazolin src=U12.
MO-ANCH-BLA-U13 anchor_span raw="βラクタム系抗菌薬" src=U13.
MO-LEX-BLA-U13 lex_surface_hit surface="βラクタム系抗菌薬" concept_candidate=beta_lactam_antibacterial src=U13.
MO-XREF-MISSING crossref_surface raw="表99" target=missing src=U18.
MO-CELL-MALFORMED-GUARD table_cell row=DTR-MALFORMED-R1 column=guard raw="収縮期血圧 < 80" src=U19.
MO-LEX-UNK lex_surface_hit surface="未知薬Y" concept_candidate=unmapped src=U20.
MO-LEX-TESTROUTE lex_surface_hit surface="試験経路" concept_candidate=test_route src=U21.
MO-TEXT-UNCERT text_node kind=sentence normalized="投与量未確認" src=U22 diagnostic=conflicting_offsets.
MO-TEXT-PATIENT text_node kind=sentence normalized="患者データから敗血症リスクを予測して抗菌薬を選択する." src=U23.
MO-MOD-PERMIT modality_marker raw="考慮し" kind=permit src=U24.
MO-MOD-AVOID modality_marker raw="避ける" kind=avoid src=U24.
MO-LEX-SUPINF-A lex_surface_hit surface="表在感染" concept_candidate=superficial_infection src=U25.
MO-LEX-SUPINF-B lex_surface_hit surface="表在感染" concept_candidate=device_infection src=U25.
MO-LEX-DOSE-HIGH lex_surface_hit surface="高用量" concept_candidate=high_dose src=U26.
MO-LEX-DOSE-STD lex_surface_hit surface="標準用量" concept_candidate=standard_dose src=U27.
```

MOs assert only surfaces and layout facts, not clinical truth.

### A.3 Admitted terminology and policy fixtures

```text
concepts={adult_population,sepsis,beta_lactam_antibacterial,cefazolin,anaphylaxis,systolic_bp,emergency_action,usual_action,source_node_present,source_node_absent,intravenous_route,oral_route,rapid_administration,routine_administration,drug_x_a,drug_x_b,test_route,superficial_infection,device_infection,high_dose,standard_dose}.
mutually_exclusive={(drug_x_a,drug_x_b),(superficial_infection,device_infection),(source_node_present,source_node_absent),(emergency_action,usual_action),(high_dose,standard_dose)}.
binding exact "βラクタム系抗菌薬" -> beta_lactam_antibacterial.
binding exact "セファゾリン" -> cefazolin.
binding exact "静脈内" -> intravenous_route.
binding exact "経口" -> oral_route.
binding exact "速やかに" -> rapid_administration.
binding exact "通常速度" -> routine_administration.
binding exact "薬剤X" system=yj code=drug_x_fixture -> drug_x_a.
binding exact "薬剤X" system=yj code=drug_x_fixture -> drug_x_b.
binding unmapped "未知薬Y" system=fixture.
binding ambiguous "表在感染" -> superficial_infection.
binding ambiguous "表在感染" -> device_infection.
binding exact "高用量" -> high_dose.
binding exact "標準用量" -> standard_dose.
```

Duplicate exact `薬剤X` functional key triggers `functional_key_collision`; its mutually exclusive targets trigger `mutually_exclusive_term_mapping`.

```text
ActionSlotSpec route: value_kind=route_concept, discriminates_action_identity=true, normalization=concept_representative.
ActionSlotSpec administration_speed: value_kind=speed_concept, discriminates_action_identity=false, normalization=concept_representative.
ActionSlotSpec dose: value_kind=dose_concept, discriminates_action_identity=true, normalization=concept_representative.
ActionSlotSpec dose: value_kind=dose_literal, discriminates_action_identity=true, normalization=literal_identity.
ActionTargetRelation administer contraindication_target cefazolin beta_lactam_antibacterial symmetric=true.
OutputExclusion triage_action emergency_action usual_action symmetric=true.
OutputExclusion source_node_state source_node_present source_node_absent symmetric=true.
metadata_singleton_keys={publication_date}.
```

Duplicate `dose` policy rows share a semantic key, differ in payload bytes → policy validation emits `Incoherence(class=incompatible_generator_outputs)` and quarantines the duplicate dose key. The single demo `SemanticPolicySet` still contains every row above. Main theorem fixtures query only non-quarantined route, administration-speed, target-relation, output-exclusion, metadata rows; dose-policy-collision fixture exercises quarantine within the same policy set.

### A.4 Pattern observations and generator origin

```text
PO-P1 suspected_if: support=closure(U1); condition="成人の敗血症が疑われる場合"; clause="βラクタム系抗菌薬を速やかに投与することを推奨する".
PO-P2 recommend_governance: cue="推奨する" governs action="投与する" target="βラクタム系抗菌薬".
PO-P3 anaphylaxis_negative_action: condition="βラクタム系抗菌薬に対するアナフィラキシーの既往"; cue="しない" scopes action="投与".
PO-P4 decision_rows: input="収縮期血圧"; DTR-T1-R1 <90 -> 緊急対応; DTR-T1-R2 <=90 -> 通常対応; headers in closure.
PO-P5 avoid_recommendation: cue="避けることを推奨する" governs avoid action for beta_lactam_antibacterial.
PO-P6 provenance_rules: U5 requires source_node_present; U6 states source_node_absent for the same record.
PO-P7 empty_numeric_interval: U7 contains systolic_bp < 90 ∧ systolic_bp >= 90 in one strict consequent.
PO-P8 route_slot_actions: U9 and U10 bind same action kind/target with different route slot values.
PO-P9 nondiscriminating_speed_actions: U9 and U11 bind same action kind/target/route with different administration_speed values.
PO-P10 target_relation_actions: U12 target=cefazolin; U13 target=beta_lactam_antibacterial.
PO-P11 renal_adjustment_coverage_target: U17 demands AIRKey(air.norm,slot_key=renal_adjustment); no accepted license supplies it.
PO-P12 malformed_crossref: U18 contains absent cross-reference target.
PO-P13 malformed_table: U19 contains table-like guard with no output column.
PO-P14 missing_term_action: U20 demands one concept for 未知薬Y and gets unmapped.
PO-P15 missing_policy_action: U21 binds slot_id=experimental_route with value test_route; the single SemanticPolicySet has no ActionSlotSpec for experimental_route.
PO-P16 ambiguous_modality_same_key: U24 yields two licenses for one AIRKey and different readings.
PO-P17 ambiguous_term_surface: U25 demands one concept for 表在感染 and gets two candidates.
MCH-PO1..MCH-PO17=BuildMatches(PO-P1..PO-P17). MCLS-PO1..MCLS-PO17 are quotient classes. CMEM-PO1..CMEM-PO17 link each match to its class. No two match fixtures share proof_visible_signature; every class has one member.
```

```text
T License | Generator | Required observations | Projection
LIC-L1 | gen_norm_recommend_beta_lactam | PO-P1, PO-P2, MO-LEX-BLA-U1, MO-MOD-REC-U1, MO-TEMP-PROMPT-U1 | NF-N1
LIC-L2 | gen_norm_contraindicate_anaphylaxis | PO-P3, MO-NEG-U2, beta-lactam and anaphylaxis surfaces | NF-N2
LIC-L3 | gen_table_vital_triage | PO-P4, MO-CELL-G1, MO-CELL-O1, MO-CELL-G2, MO-CELL-O2, MO-Q-LT90, MO-Q-LE90 | NF-T1
LIC-L4 | gen_norm_avoid_beta_lactam | PO-P5, avoid modality and beta-lactam surface | NF-N3
LIC-L5 | gen_strict_source_node_present | PO-P6, MO-MOD-REQ-U5, source-node-present surface | NF-R1
LIC-L6 | gen_strict_source_node_absent | PO-P6, source-node-absent surface | NF-R2
LIC-L7 | gen_strict_empty_numeric_interval | PO-P7, < 90, >= 90 | NF-Q1
LIC-L8 | gen_norm_route_iv | PO-P8, MO-LEX-IV, MO-LEX-RAPID | NF-N4
LIC-L9 | gen_norm_route_oral | PO-P8, MO-LEX-ORAL, MO-LEX-RAPID | NF-N5
LIC-L10 | gen_norm_speed_routine | PO-P9, MO-LEX-IV, MO-LEX-ROUTINE | NF-N6
LIC-L11 | gen_norm_cefazolin | PO-P10, MO-LEX-CFZ | NF-N7
LIC-L12 | gen_norm_beta_lactam_class | PO-P10, MO-LEX-BLA-U13 | NF-N8
LIC-L13a | gen_ambiguous_permit | PO-P16, MO-MOD-PERMIT | AIR ambiguity only
LIC-L13b | gen_ambiguous_avoid | PO-P16, MO-MOD-AVOID | AIR ambiguity only
```

Concrete CKC-GEN-core surface instance for `PO-P2 -> LIC-L1`:

```text
CKCGen id=gen_norm_recommend_beta_lactam profile=sem_rule stage=40 sort=norm_license vars=[m_drug:MechObsPayload,m_modality:MechObsPayload,m_temporal:MechObsPayload]
premises=[(mobs m_drug MechObsPattern{kind=lex_surface_hit,field_constraints=[concept_candidate eq beta_lactam_antibacterial, anchor_id has],class_pred=true}),(mobs m_modality MechObsPattern{kind=modality_marker,field_constraints=[kind eq recommend],class_pred=true}),(mobs m_temporal MechObsPattern{kind=temporal_surface,field_constraints=[shape eq prompt, normalized_text has, anchor_id has],class_pred=true}),(seq RegionClosure{seeds=[BoundAddress{term=FieldTerm{base=m_drug,path=[anchor_id]}}]} [SeqItem{item_id=s1,pattern=MechObsPattern{kind=lex_surface_hit,class_pred=true},role=drug,min_gap=0,max_gap=8}, SeqItem{item_id=s2,pattern=MechObsPattern{kind=modality_marker,class_pred=true},role=modality,min_gap=0,max_gap=16}] [RoleBinding{role=drug,value=VarTerm{var=m_drug}}, RoleBinding{role=modality,value=VarTerm{var=m_modality}}])]
head=(license LicenseTemplate{air_key=AIRKeyTemplate{air_type=air.norm,support_region=RegionOfTerm{term=FieldTerm{base=m_drug,path=[source_region_id]}},slot_key=LiteralTerm{literal=IdLiteral{value=n1}}},reading=ReadingTemplate{reading_kind=NormReading,field_bindings=[{path=[direction],value=TermValue{term=LiteralTerm{literal=EnumLiteral{enum_name=Direction,variant=for}}}},{path=[action,action_kind],value=TermValue{term=LiteralTerm{literal=IdLiteral{value=administer}}}},{path=[action,target],value=TermValue{term=LiteralTerm{literal=IdLiteral{value=beta_lactam_antibacterial}}}},{path=[temporal],value=ListTemplateValue{values=[ReadingTemplateValue{reading=ReadingTemplate{reading_kind=TemporalReading,field_bindings=[{path=[temporal_kind],value=TermValue{term=LiteralTerm{literal=IdLiteral{value=prompt}}}},{path=[value],value=TermValue{term=FieldTerm{base=m_temporal,path=[normalized_text]}}},{path=[raw_anchor_id],value=TermValue{term=FieldTerm{base=m_temporal,path=[anchor_id]}}}]}}]}}]},source_support={RegionOfTerm{term=FieldTerm{base=m_drug,path=[source_region_id]}}},proof_roots={}})
```

`T-GEN-Static` checks pattern schemas, role bindings, finite `seq` gaps, enum literals, head field paths before materialization.

### A.5 Semantic licenses, AIRCore, and residual/ambiguity materialization

```text
L1 norm n1: dir=for; action=AB; ctx=C; temporal=prompt; support=U1 -> N1.
L2 norm n2: dir=contraindicate; action=AB; ctx=H; support=U2 -> N2.
L3 factual t1: TableReading input=systolic_bp unit=mmHg rows DTR-T1-R1 guard <90 -> triage_action emergency_action; DTR-T1-R2 guard <=90 -> triage_action usual_action; support=U3 -> NF-T1.
L4 norm n3: dir=avoid; action=AB; ctx=C; support=U4 -> N3.
L5 factual r1: strict beta_lactam_administration_record -> source_node_state=source_node_present; support=U5 -> NF-R1.
L6 factual r2: strict beta_lactam_administration_record -> source_node_state=source_node_absent; support=U6 -> NF-R2.
L7 factual q1: strict consequent systolic_bp < 90 ∧ systolic_bp >= 90; support=U7 -> NF-Q1.
L8 norm a1: dir=for; action=AB{route=intravenous_route,administration_speed=rapid_administration}; ctx=C; support=U9 -> N4.
L9 norm a2: dir=for; action=AB{route=oral_route,administration_speed=rapid_administration}; ctx=C; support=U10 -> N5.
L10 norm a3: dir=for; action=AB{route=intravenous_route,administration_speed=routine_administration}; ctx=C; support=U11 -> N6.
L11 norm a4: dir=for; action=administer(cefazolin); ctx=C; support=U12 -> N7.
L12 norm a5: dir=for; action=AB; ctx=C; support=U13 -> N8.
L13a norm ambiguous_beta_lactam: dir=permit; action=AB; ctx=C; support=U24.
L13b norm ambiguous_beta_lactam: dir=avoid; action=AB; ctx=C; support=U24.
AIRCore(L1..L12)=ok with exactly one reading per key. AIRCore(renal_adjustment)=residual Residual(class=no_license). AIRCore(ambiguous_beta_lactam)=ambiguity Ambiguity(class=multiple_readings).
RT-R1 resolution explicit_reconciliation: applies_to={LIC-L1,LIC-L4}; context=C; support=U4; kind=explicit_reconciliation; resolves the dedicated N1 vs N3 fixture pair through §8.4 resolution_subject_ids.
```

```text
RES-no-license: PO-P11 AIRKey Omega(K)={} -> Residual(no_license).
RES-unsupported-construction: CollectBound.max_items=1 over two matching licenses -> Residual(unsupported_construction).
RES-unsupported-cross-reference: source_region_closure(U18) follows MO-XREF-MISSING to no target -> Residual(unsupported_cross_reference).
RES-unsupported-table-structure: source_region_closure(U19) needs output column and header relation -> Residual(unsupported_table_structure).
RES-missing-terminology: PO-P14 demands one concept for 未知薬Y -> Residual(missing_terminology).
RES-missing-policy: PO-P15 normalizes slot_id=experimental_route with no ActionSlotSpec row -> Residual(missing_policy).
RES-missing-counterexample-suite: PR-PG2 lacks CounterexampleSuite -> Residual(missing_counterexample_suite).
RES-permission-limited: report view requests quoted PI text but PI allows only hashes and derived labels -> Residual(permission_limited).
RES-extraction-uncertain: U22 has conflicting offsets -> Residual(extraction_uncertain).
RES-verifier-unsupported: stale theorem candidate VU1 depends on TemporalLiteralAtom values "速やかに" and "直ちに" for one variable -> Residual(verifier_unsupported).
RES-deferred-gate-required: U23 triggers patient-data and probabilistic claims without GateEvidenceRef -> Residual(deferred_gate_required).
AMB-multiple-readings: L13a and L13b share AIRKey with distinct readings -> Ambiguity(multiple_readings).
AMB-multiple-terms: PO-P17 demands one concept for 表在感染 and gets superficial_infection, device_infection -> Ambiguity(multiple_terms).
INC-functional-key-collision: system=yj code=drug_x_fixture maps to drug_x_a and drug_x_b -> Incoherence(functional_key_collision).
INC-mutually-exclusive-term-mapping: surface 薬剤X maps to mutually exclusive drug_x_a and drug_x_b -> Incoherence(mutually_exclusive_term_mapping).
INC-incompatible-generator-outputs: duplicate dose ActionSlotSpec rows differ -> Incoherence(incompatible_generator_outputs).
```

### A.6 Normal Form

Each NF object below = deterministic projection of the named `AIRCoreRecord` or metadata builder.

```text
N1=NFNorm(L1): source_class=guideline; dir=for; action=AB; ctx=C; temporal=prompt; original_modality_phrase_ja="推奨する"; supp(U1).
N2=NFNorm(L2): source_class=package_insert; dir=contraindicate; action=AB; ctx=H; original_modality_phrase_ja="投与しない"; supp(U2).
NF-T1=NFDecisionTable(L3): source_class=guideline; input=systolic_bp; unit=mmHg; rows as L3 preserving order; supp(U3).
N3=NFNorm(L4): source_class=guideline; dir=avoid; action=AB; ctx=C; supp(U4).
NF-R1=NFFactualRule(L5): strict=true; ctx=beta_lactam_administration_record; consequent source_node_state=source_node_present; supp(U5).
NF-R2=NFFactualRule(L6): strict=true; ctx=beta_lactam_administration_record; consequent source_node_state=source_node_absent; supp(U6).
NF-Q1=NFFactualRule(L7): strict=true; ctx=true; consequent includes systolic_bp < 90 and systolic_bp >= 90; supp(U7).
N4=NFNorm(L8): source_class=guideline; dir=for; action=AB{route=intravenous_route,administration_speed=rapid_administration}; ctx=C; supp(U9).
N5=NFNorm(L9): source_class=guideline; dir=for; action=AB{route=oral_route,administration_speed=rapid_administration}; ctx=C; supp(U10).
N6=NFNorm(L10): source_class=guideline; dir=for; action=AB{route=intravenous_route,administration_speed=routine_administration}; ctx=C; supp(U11).
N7=NFNorm(L11): source_class=guideline; dir=for; action=administer(cefazolin); ctx=C; supp(U12).
N8=NFNorm(L12): source_class=guideline; dir=for; action=AB; ctx=C; supp(U13).
NF-M1=NFMetadataClaim(SRC-GDL): bid="敗血症抗菌薬ガイド fixture"; key=publication_date; value="2026-01-01".
NF-M2=NFMetadataClaim(SRC-PI): bid="敗血症抗菌薬ガイド fixture"; key=publication_date; value="2025-12-01".
```

`NF(NF(x))=NF(x)` for all listed. Reordering conjunction atoms in N1,N3,N4,N5,N6,N7,N8 preserves `semantic_digest`; reordering NF-T1 rows changes it.

### A.7 Expected M0 theorem witnesses

Conflict candidates = exactly this closure. Pair entries have compatible context, empty `ResolutionSet`, stated action/consequent witness. Dedicated N1 vs N3 pair has nonempty `ResolutionSet={RT-R1}` → suppressed negative-control row, not a conflict theorem. NF-Q1 fails `strict_factual_self_check`, appears only in CT-C4; no other NF pair satisfies a §8.5 pair predicate.

```text
CT-C1a contraindication_vs_recommendation: N1 vs N2; same action AB; ctx witness C ∧ H; ResolutionSet={}.
CT-C1b contraindication_vs_recommendation: N2 vs N7; cefazolin overlaps beta_lactam_antibacterial by ActionTargetRelation(contraindication_target); ctx witness H ∧ C; ResolutionSet={}.
CT-C1c contraindication_vs_recommendation: N2 vs N8; same action AB; ctx witness H ∧ C; ResolutionSet={}.
CT-C2b recommendation_for_vs_against: N3 vs N7; directions avoid vs for; target overlap by ActionTargetRelation(contraindication_target); same context; ResolutionSet={}.
CT-C2c recommendation_for_vs_against: N3 vs N8; directions avoid vs for; same action and same context; ResolutionSet={}.
CT-C3 strict_consequents_jointly_contradictory: NF-R1 vs NF-R2; same ctx beta_lactam_administration_record; finite check sees SlotEqAtom(source_node_state,source_node_present) and SlotEqAtom(source_node_state,source_node_absent) with mutually exclusive values; ResolutionSet={}.
CT-C4 numeric_threshold_empty_intersection: NF-Q1 has systolic_bp < 90 and systolic_bp >= 90; (-inf,90) ∩ [90,inf)=empty; strict_factual_self_check(NF-Q1)=self_inconsistent; only theorem for NF-Q1.
CT-C5 terminology_mapping_incoherence: system=yj code=drug_x_fixture maps to drug_x_a and drug_x_b, and surface 薬剤X maps to mutually exclusive concepts; theorem references both terminology incoherence artifacts.
```

```text
RT-R1-SUPPRESSION: N1 vs N3 satisfies recommendation_for_vs_against antecedents, but ResolutionSet(N1,N3,C)={RT-R1}; no ConflictTheorem is emitted for that pair.
```

```text
AW-AS1: N4 vs N5 share action_kind=administer and target=beta_lactam_antibacterial; route intravenous_route vs oral_route differs; ActionSlotSpec(route).discriminates_action_identity=true; same_normalized_action=distinct.
AW-AS2: N4 vs N6 share action_kind,target,route; administration_speed rapid vs routine differs; ActionSlotSpec(administration_speed).discriminates_action_identity=false; same_normalized_action=same.
AW-AS3: N7 target=cefazolin and N8 target=beta_lactam_antibacterial are distinct; ActionTargetRelation(administer,contraindication_target,cefazolin,beta_lactam_antibacterial,symmetric=true) matches; targets_overlap=same; same_normalized_action=same.
AW-AS4: for each X in {N4,N5,N6} and Y in {N2,N3}, X has discriminating route and Y lacks route; discriminating slot IDs are not present on both sides; same_normalized_action=distinct.
```

```text
FI-F1 table_value_disagreement: NF-T1.DTR-T1-R1 and DTR-T1-R2 overlap at systolic_bp=89; emergency_action and usual_action are OutputExclusion for triage_action.
FI-F2a package_insert_vs_guideline_unresolved_conflict: N1 guideline vs N2 package_insert; normative incompatibility, same action, compatible context, ResolutionSet={}.
FI-F2b package_insert_vs_guideline_unresolved_conflict: N2 package_insert vs N7 guideline; normative incompatibility, target overlap through ActionTargetRelation(contraindication_target), compatible context, ResolutionSet={}.
FI-F2c package_insert_vs_guideline_unresolved_conflict: N2 package_insert vs N8 guideline; normative incompatibility, same action, compatible context, ResolutionSet={}.
FI-F3 gloss_semantic_drift: stale stored gloss for N1 renders different drug class with same nf_hash; re-rendered canonical GlossView bytes and combined_slot_digest differ.
FI-F4 source_metadata_disagreement: NF-M1 and NF-M2 share bid and publication_date key; normalized values "2026-01-01" and "2025-12-01" differ.
FI-F5 proof_or_certificate_replay_failure: mutated certificate copy has ReplayIdentityCheck.outcome=replay_identity_mismatch.
```

```text
NFC-NFALSE1 N4 vs N5: route discriminates action identity, so no same-action conflict.
NFC-NFALSE2 N4 vs N6: actions are same because administration_speed is nondiscriminating, but both directions are for.
NFC-NFALSE3 N7 vs N8: targets overlap by ActionTargetRelation, but both directions are for.
NFC-NFALSE4 for each X in {N4,N5,N6}, Y in {N2,N3}: route discriminates and Y lacks route, so no same-action conflict or package-insert factual predicate fires.
NFC-NFALSE5 NF-T1 rows under output_slot_id mismatch mutation: table_outputs_compatible returns compatible; FI-F1 does not fire.
NFC-NFALSE6 NF-M1 vs package_insert publication_date with bid or key mutation: source_metadata_disagreement does not fire.
NFC-NFALSE7 NF-Q1 vs each of {NF-R1,NF-R2}: NF-Q1 is self-inconsistent, excluded from pairwise strict-consequent checking; only CT-C4 fires.
NFC-NFALSE8 N1 vs N3: ResolutionSet={RT-R1}; recommendation_for_vs_against is suppressed and no ConflictTheorem fires.
```

### A.8 Deterministic gloss

For `N1`, the Japanese gloss template renders:

```text
成人かつ敗血症が疑われる場合には、βラクタム系抗菌薬の投与を推奨する。
```

Gloss stores ordered slot digests for `context adult_population; context suspected(sepsis); action administer; target beta_lactam_antibacterial; direction for.` Stale gloss fixture with same `(nf_hash, lang, template_id)` and different target rendering triggers FI-F3. Missing template unit fixture returns unsupported, emits a gloss diagnostic, not an accepted `GlossView`.

### A.9 Admission and discharge witnesses

Fixture ProposalRecord rows use proposal_provenance_hashes={} unless stated otherwise.

```text
PR-PG1: Proposal subject_kind=CKCGen proposal_kind=CKCGen candidate=gen_norm_recommend_beta_lactam; CES-PG1 requires LIC-L1 and forbids residual classes {unsupported_construction,missing_terminology}; DischargeProposal materializes LIC-L1, matches required_output_digests, emits zero forbidden payloads, AdmissionDecision=accept; EffectDischargeRecord.accepted_effect_row={}
PR-PG2: Proposal candidate=gen_missing_suite_fixture; AdmissionContext.counterexample_suite_hash absent; DischargeProposal emits Residual(missing_counterexample_suite) and leaves accepted generators unchanged.
PR-PG3 collect_overflow_fixture: CES-PG3 expects Residual(unsupported_construction); CollectBound.max_items=1 and two matching licenses; materialization emits that residual.
PR-PG4 gated_patient_probabilistic_fixture: CES-PG4 expects Residual(deferred_gate_required); proposal makes patient-data and probabilistic claim from U23; absent GovernedPatientDataProfile and ProbabilisticProfileRecord emit that residual.
PR-PG5 unsupported_verifier_fixture: CES-PG5 expects Residual(verifier_unsupported); theorem candidate depends on conflicting TemporalLiteralAtom values; kernel_finite_checker returns unsupported and emits that residual.
```

### A.10 Replay target

```text
ckc demo m0 --out runs/m0
```

Expected accepted outputs are exactly:

```text
SchemaRegistry hash; SchemaBoundManifest hash; UnicodePolicyManifest hash; ToolchainManifest hash with ToolRecord rows; EnvironmentProfile hash; ProducerManifest hashes for every canonical command invoked by the demo; ValidationManifest hashes for schema, runtime-manifest, fixture-manifest, policy-admission, closure, verifier, report, and replay gates.
FiniteFixtureManifest hash with FrozenConstant, ParsedQuantity, and DiagnosticTag rows used by Appendix A.
SourceEdition{SRC-GDL,SRC-PI}; SourcePermissionRecord{SRC-GDL,SRC-PI}; CorpusDocument{CDOC-GDL,CDOC-PI}; ExtractionManifest fixture hash; SourceGraph fixture hash.
SourceSpan and SourceAnchor hashes for each certain fixture source region.
SourceRegion and RegionClosureCertificate hashes for REG-U1..REG-U17, REG-U20, REG-U21, REG-U23..REG-U27.
Residual hashes for unsupported_cross_reference(REG-U18), unsupported_table_structure(REG-U19), extraction_uncertain(REG-U22), permission_limited(SRC-PI report view).
AnalyzerManifest and MechanicalLexicon hashes; MechObsPayload hashes for every certain A.2 MO-* observation.
PatternObs{PO-P1..PO-P17}; Match{MCH-PO1..MCH-PO17}; MatchClass{MCLS-PO1..MCLS-PO17}; ClassMember{CMEM-PO1..CMEM-PO17}.
ProposalRecord{PR-PG1..PR-PG5}; CounterexampleSuite{CES-PG1,CES-PG3..CES-PG5}; AdmissionRecord and EffectDischargeRecord{PR-PG1..PR-PG5}; MaterializedConsequenceManifest{PR-PG1..PR-PG5}.
AcceptedGeneratorBase hash; CKCGen hashes for every accepted generator named in A.4 and A.9; GeneratorGrammarArtifact hash with authority=evidence_discovery_only.
TerminologyResourceSet hash; TerminologyClosure hash; SemanticPolicySet hash naming the admitted policy input accepted by DischargeProposal; ClosureInput hash; ClosureOutput hash; ClosureBoundCertificate hash; ProofNode hashes for every demo ProofDAG node; ProofDAG hash.
License{LIC-L1..LIC-L13b}; ResolutionTheorem{RT-R1}; LicensedReadingSet hashes for all demanded AIR keys; AIRCore hashes for all demanded AIR keys including no_license and multiple_readings outcomes.
NF hashes for NF-N1,NF-N2,NF-N3,NF-N4,NF-N5,NF-N6,NF-N7,NF-N8,NF-T1,NF-R1,NF-R2,NF-Q1,NF-M1,NF-M2.
GlossTemplate hash for the N1 Japanese template and stale-gloss drift comparator; GlossView hash for N1; ReportQuestionTemplate hashes for every rendered report-question template.
ConflictTheorem{CT-C1a..CT-C1c,CT-C2b..CT-C2c,CT-C3..CT-C5}; FactualInconsistencyTheorem{FI-F1,FI-F2a..FI-F2c,FI-F3..FI-F5}.
Residual{RES-no-license,RES-unsupported-construction,RES-unsupported-cross-reference,RES-unsupported-table-structure,RES-missing-terminology,RES-missing-policy,RES-missing-counterexample-suite,RES-permission-limited,RES-extraction-uncertain,RES-verifier-unsupported,RES-deferred-gate-required}.
Ambiguity{AMB-multiple-readings,AMB-multiple-terms}; Incoherence{INC-functional-key-collision,INC-mutually-exclusive-term-mapping,INC-incompatible-generator-outputs}.
Diagnostic hashes for every residual, ambiguity, incoherence, stale gloss, source metadata disagreement, and replay-failure unit fixture.
WitnessContext hashes for every compatible context witness used by CT-C1a..CT-C1c, CT-C2b..CT-C2c, CT-C3, FI-F1, and FI-F2a..FI-F2c; ConstraintCoreWitness hashes for CT-C3 and CT-C4; VerifierWitness and SymbolSourceMap hashes for each valid theorem in CT-C1a..CT-C1c, CT-C2b..CT-C2c, CT-C3..CT-C5, FI-F1, FI-F2a..FI-F2c, FI-F3..FI-F5, plus the unsupported verifier fixture.
ActionSameness witness hashes AW-AS1..AW-AS4; ClaimRecord hashes for every theorem subject, residual/ambiguity/incoherence report item, certificate, and the ReviewReport; Certificate hashes for source_graph, mech_observed, admitted_base, closed_nf, finite_checked, report_replay.
ReportTraceIndex hash, ClaimTierSummary hash, WordingGateRecord hash, and ReviewReport hash sorted by §9.3 with trace_hash, claim_tier_summary_hash, and wording_gate_hash; ReplayManifest{RM-PRODUCER-BASE,RM-DEMO-CORE} where emitted artifacts point at RM-PRODUCER-BASE and RM-DEMO-CORE.expected_output_hashes exclude RM-DEMO-CORE, RIC-DEMO-CORE, and CERT-report_replay by §1.6; ReplayIdentityCheck outcomes: replay_identity_pass for non-mutated run, replay_identity_mismatch for deliberate replay-failure fixture, replay_identity_unsupported for permission-limited source-byte replay unit fixture.
```

Accepted artifact set emitted by `ckc demo m0` = exactly the set above. Emitted `RetrievalProposalTrace` set = `{}` (fixture uses authored proposals). Emitted stage-10 `TerminologyResourceSet` fragment set = `{}` (no `term_resource` generator admitted; single admitted `TerminologyResourceSet` enters via `ClosureInput`). Emitted `ResolutionTheorem` set = `{RT-R1}`; N1 vs N3 suppressed-pair fixture proves its consumer path. Emitted `RepairSetSearchTrace` set = `{}` (no repair-set search). Stale internal artifacts, permission-disallowed raw quotations, and unsupported gated claims appear only through the listed residuals, diagnostics, verifier witnesses, replay checks.
