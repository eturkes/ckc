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
- [ ] M0.0.3
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
