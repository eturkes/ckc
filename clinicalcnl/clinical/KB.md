# ClinicalCNL KB kernel contract

Authoritative contract for the clinical Prolog knowledge base (SPEC §6 LP profile, §5 domain IR).
Fixes the rules-as-data term family a compiled ClinicalCNL document IS, its ground-term grammar, the
closed v1 vocabulary, the safety invariants, and the negation-as-failure execution semantics the
conflict layer builds on. `clinical/kb_kernel.pl` implements the validators + reference derivability;
`clinical/goldens/kb_examples.pl` holds the hand-written normative examples; `clinical/kb_kernel_tests.pl`
is the plunit gate.

A "KB" is a LIST of ground fact terms — never asserted. `map-core`/`map-emit` produce it, `kb-writer`
emits it to canonical bytes, `conflict-*` consume it. Validation (`valid_kb/1`, `kb_errors/2`) is
side-effect-free over the list.

## Fact family (rules-as-data)

Nine fixed families (§L·lp / SPEC §6 `rule population condition action direction strength certainty
exception source`). Rule-level fields (`direction`/`strength`/`certainty`) key on the rule id (one
modality keyword per rule-block, shared by its disjuncts); statement-level fields key on the statement
id (one DNF disjunct = one guard + action).

| fact                              | keys on   | role                                                              |
| --------------------------------- | --------- | ----------------------------------------------------------------- |
| `rule(RuleId, StmtId)`            | rule+stmt | `StmtId` is a DNF disjunct of `RuleId` (1+ facts per rule)         |
| `direction(RuleId, Direction)`    | rule      | deontic direction (from the keyword)                              |
| `strength(RuleId, Strength)`      | rule      | `strong`/`weak` (proof-visible annotation)                        |
| `certainty(RuleId, Certainty)`    | rule      | `high`/`moderate`/`low`/`very_low` — OPTIONAL (0 or 1 per rule)    |
| `population(StmtId, PopKey)`      | stmt      | the subject (v1: `pop.patient`); binds the statement's subject     |
| `condition(BindId, StmtId, Atom)` | bind+stmt | a context atom (the guard); each is a `bind.k` binding            |
| `action(StmtId, ActionKey)`       | stmt      | the `<kind>:<target>` action key                                  |
| `exception(ExcId, StmtId, Atom)`  | exc+stmt  | a labeled exception atom, NAF-guarded (PROLEG)                    |
| `source(Id, DocId, Regions, Basis)` | rule/stmt/exc | provenance: doc, ascending raw-sentence indices, evidence basis |

## Ids

Every id is a doc-qualified dotted atom `<doc>.<kind>.<k>`:

- `<doc>` — the corpus test-source id (itself dotted, e.g. `test_source.m1_guideline_a`), non-empty.
- `<kind>` — one of `rule`, `stmt`, `bind`, `exc`.
- `<k>` — a canonical non-negative decimal (`0`, or a nonzero digit then digits; no sign, leading zero,
  radix, or underscore), so each logical id has exactly one textual form. It is DOCUMENT-CONTINUOUS:
  `map-emit` runs one counter per kind across the whole document and NEVER resets it per rule (a
  2-disjunct rule then a trailing rule yields `stmt.0`/`stmt.1` then `stmt.2`, not a reset to `stmt.0`).

`<doc>.rule.<k>` matches SPEC §8.6's `rule_id` exactly (so `participating_rules` = `a.<doc>.rule.<k>`
composes directly); `stmt`/`bind`/`exc` extend the same qualification so a flat KB holding several
documents (the conflict operating mode) never collides. `valid_id(Id, Kind)` validates the shape
(non-empty doc segments, canonical counter) and infers/checks `Kind`. Counter DENSITY (0-based,
gap-free) is a `map-emit` property, not a kernel reject — like source completeness below.

## Context atoms

A `condition` or `exception` payload `Atom` is one of:

- `concept(ConceptId)` — a clinical condition concept holds.
- `interval(Quantity, Bound, Openness, Dir)` — a bounded quantity constraint. `Openness` ∈
  `{open, closed}`, `Dir` ∈ `{lower, upper}`. `Bound` is EXACT — an `integer` or a SWI rational (`NrD`
  syntax, e.g. `1r2`), never a float; conflict arithmetic (`D10`) needs open-vs-closed distinctions over
  exact rationals.

The interval normalizes the DRS `CountOp` (`SURFACE.md` §Intervals) into `(Openness, Dir)`, which
`map-core` computes; the KB never carries a raw `CountOp`:

| DRS `CountOp` | surface marker | KB `interval` `(Openness, Dir)` |
| ------------- | -------------- | ------------------------------- |
| `geq`         | at least       | `(closed, lower)` (`X >= B`)     |
| `greater`     | more than      | `(open, lower)` (`X > B`)        |
| `leq`         | at most        | `(closed, upper)` (`X =< B`)     |
| `less`        | less than      | `(open, upper)` (`X < B`)        |

`exactly`/bare `eq` are single-bound-law rejects at the raw/profile lane (`SURFACE.md`); the KB never
sees them. Population demographics carry no concept atom: adult/child = an age `interval` over
`q.age_years` (`age >= 18` / `age < 18`), not a `pop.adult`/`pop.child` concept.

## Action key

`action(StmtId, ActionKey)` where `ActionKey = <kind>:<target>` — exactly one colon.
`action_key(Key, Kind, Target)` splits/joins. Action sameness (conflict eligibility) = key identity
(§5: same kind + normalized target). v1: `act.administer:drug.abx_a`.

## Vocabulary (closed, v1)

`kb_kernel.pl` OWNS the closed vocabulary (the semantic id set); the `ulex` registry later covers each
id with a surface and its integrity checker cross-checks coverage against these predicates.

- `kb_concept/1` — `cond.sepsis`, `cond.renal_severe`, `cond.pregnancy`.
- `kb_action_kind/1` — `act.administer`. `kb_action_target/1` — `drug.abx_a`.
- `kb_quantity/1` — `q.age_years`. `kb_population/1` — `pop.patient`.
- `kb_direction/1` — `for require permit against avoid contraindicate`. v1 keywords emit only
  `{for, permit, against, contraindicate}`; `require`/`avoid` are contract-admitted (they appear in the
  §L·conflict direction groups) but not v1-emitted.
- `kb_strength/1` — `strong weak`. `kb_certainty/1` — `high moderate low very_low`.

### Direction groups (→ conflict)

`direction_group(Direction, Group)` is the §L·conflict direction → group relation the conflict layer's
eligibility check consumes (`avoid` joins BOTH non-positive groups):

- positive: `for`, `require`, `permit`
- against: `against`, `avoid`
- contraindicating: `contraindicate`, `avoid`

## Safety invariants

`valid_kb/1` succeeds iff `kb_errors/2` returns `[]`. Enforced (each a distinct violation term):

- Shape — the KB is a proper list (`not_a_list`); every fact is one of the nine families
  (`unknown_fact`) and fully ground (`nonground_fact`).
- Ids — every id is a well-formed `<doc>.<kind>.<k>` of the right kind (`malformed_id`).
- Vocabulary — direction/strength/certainty/population/concept/quantity/action-kind/action-target all
  in the closed set; action key is `<kind>:<target>` (`unknown_*`, `malformed_action_key`,
  `bad_interval_*`, `malformed_atom`).
- Cardinality — every declared statement has EXACTLY ONE population (the subject binder) and one action;
  every declared rule has exactly one direction, one strength, at most one certainty; `bind`/`exc` ids
  are unique; each `rule/2` disjunct-pair is unique and each statement is owned by exactly one rule
  (`missing_*`, `duplicate_*`, `multi_owned_stmt`).
- Referential integrity — no rule-level field for an undeclared rule, no statement-level fact for an
  undeclared statement, no `source` for an undeclared subject (`dangling_*_ref`).
- Source shape — `source/4`'s subject id is a `rule`/`stmt`/`exc` id, its atom doc matches that id's
  prefix, its regions are a NON-EMPTY STRICTLY-ASCENDING list of non-negative sentence indices, and its
  basis is a string or `none`; any breach (a malformed subject id included) is `malformed_source`.

"Every statement variable is bound by population/condition atoms" is realized structurally: the
population binds the subject, condition atoms bind the guard's concepts/quantities, and the whole KB is
ground — no free referent survives. Per-element provenance COMPLETENESS (every rule + exception carries
a `source`) is a `map-emit` obligation, not a kernel reject (the kernel validates `source` shape +
reference when present).

## Execution semantics (PROLEG negation-as-failure)

`derivable(StmtId, Facts, Ctx)` is the reference derivability: `StmtId`'s advice holds for a fixture
context `Ctx` when `StmtId` is a declared disjunct, every condition atom holds, AND no exception fires —
`\+ (exception(_, StmtId, Atom), holds_atom(Atom, Ctx))`, the PROLEG NAF guard. Exceptions stay LABELED
NAF guards here (the SMT lane instead expands them to negated context conjuncts — SPEC §6 lane
separation). `Ctx` is a synthetic list of `concept(C)` / `quantity(Q, V)` facts about one patient
(closed-world: an absent concept does not hold); `holds_atom` checks interval bounds open-vs-closed.

`derivable/3` is a CONTRACT + differential-test reference — the finite-fixture "context satisfaction /
exception-expansion equivalence" SPEC §6 sanctions. It is NOT shipped patient evaluation, which stays
behind §15 `G-RULE-EVAL`. The conflict layer (`conflict-core`) builds SYMBOLIC context overlap over this
same atom + exception structure (DNF disjunct-pair enumeration × concept polarity × interval
intersection × exception expansion); it never patient-evaluates.

## Validators, examples, gate

- `clinical/kb_kernel.pl` — the vocabulary, `valid_id/2`, `action_key/3`, `valid_atom/1`, `kb_errors/2`,
  `valid_kb/1`, `direction_group/2`, `derivable/3`. Loads warning-free; will be imported by `kb-writer`/`map-*`.
- `clinical/goldens/kb_examples.pl` — `kb_example(Name, Validity, Facts)`: the §8.6 thread
  (`doc_a`/`doc_b`/`control`) + a multi-disjunct synthetic (`multi`, all four interval markers, optional
  certainty, document-continuous counters, two exceptions on one statement) as `valid`; one
  isolated-defect example per validator rule as `invalid(Functor)`. `kb-writer` byte-pins the `valid`
  set.
- `clinical/kb_kernel_tests.pl` — accept (valid → no errors) / reject (invalid → EXACTLY the one
  expected violation functor) over every example, a catalog tripwire pinning the example counts, plus
  direct tests of the id grammar, action-key split/join, context atoms, direction groups, and the
  derivability open/closed boundaries + declared-disjunct guard.

Gate: `swipl -q -g "consult('clinical/kb_kernel_tests.pl'),(run_tests(kb_kernel)->halt(0);halt(1))" -t 'halt(1)'`
