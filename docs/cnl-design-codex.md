# CKC CNL/IR Design Proposal

Provenance: Codex second-opinion design proposal (2026-07-07), preserved verbatim below the
line. Status: PARTIALLY SUPERSEDED — the adopted design (SPEC §10/§11, same date) diverges on
the central split: this doc keeps audit prose generation-only and targets a compact record DSL
for model emission; the adopted architecture makes ClinicalCNL itself the primary
emission target and locked knowledge surface (user directive), demoting the compact DSL to a
§11 ablation (`route.ckc_rec_dsl`). Its validation lists, LP program shape, PENS target, and
risk table remain live inputs.

## Position

CKC should not adopt ACE, Logical English, FRETISH, PENGASP, CQL, or SBVR as the primary IR. It should build a CKC-owned compact DSL/CNL over the existing CKC IR and keep prose as a generated audit view.

Core invariant:

```text
source text -> model emits CKC DSL under grammar constraints
          -> deterministic parse to CKC IR
          -> deterministic compile to SMT + executable LP
          -> deterministic EN/JA verbalization from accepted IR/LP
```

Reason:

- ACE/LE/PENG prove CNL<->logic is feasible, but each brings wrong scope, license/status risk, missing clinical fields, or excessive grammar complexity.
- FRET/BRIDGE-Wiz prove slot templates and formalization feedback are the tractable path.
- s(CASP) gives the best executable explanation lane, but it should be compiler output, not the weak model's target.
- Japanese is easiest as deterministic generation from a language-neutral AST; broad Japanese deterministic parsing is premature.

## Design Goals

Target PENS for CKC audit CNL: `P5 E3 N4 S4 D`.

```text
P5 fixed semantics: every accepted surface maps to one CKC AST.
E3 rule/FOL-ish expressiveness: quantification over patient context, rules, negation, intervals, exceptions.
N4 readable natural-ish text: valid controlled sentences, clumsy but clinician-auditable.
S4 small exact spec: <=10 pages core grammar/semantics for M4.
D domain-specific: guideline recommendations, not general English.
```

## Recommended Architecture

### 1. Authoritative Hub

Keep current CKC layer stack authoritative:

```text
ClinicalIR: statements, population, condition, action, direction, strength, certainty, exceptions
NormIR: normalized rules, contexts, actions, source regions
FormalIR: target-independent constraints and query plan
```

The DSL is a route surface, not a new semantic authority.

### 2. Three Surfaces

| surface | writer | reader | purpose |
|---|---|---|---|
| `ckc_rec_dsl` | weak model | deterministic parser | compact constrained-decoding target |
| `ckc_audit_en` | deterministic verbalizer | optional parser for own generated text | clinician audit |
| `ckc_audit_ja` | deterministic verbalizer | optional parser for own generated text | Japanese audit |

Rule: never use human-readable proof prose as source of truth. It can be parsed only if it is CKC-generated and grammar-versioned.

### 3. Route Candidates

M3/M4 should register at least these candidates:

| route | shape | expected role |
|---|---|---|
| `route.ckc_rec_dsl.v1` | one compact record per recommendation | first invented DSL |
| `route.ckc_slot_cnl.v1` | BRIDGE-Wiz/FRET-like slot sentence | readability ablation |
| `route.logic_template.v1` | Logical-English-like predicate templates, still parsed to CKC IR | Prolog-distance ablation |
| `route.audit_reparse.v1` | accepted IR -> audit EN -> model/grammar parse -> IR hash compare | optional stability metamorphic |

## `ckc_rec_dsl.v1`

### Concrete Example

```text
rec r.abx_a.adult_sepsis:
  source s.g1.rec3
  population all(concept pop.adult)
  condition all(concept cond.sepsis)
  action administer drug.abx_a
  direction for
  strength strong
  certainty moderate
  except e.renal:
    all(concept cond.renal_severe)
```

M3/M4 extension example:

```text
rec r.ics.asthma:
  source s.12
  population any(
    all(interval q.age_years ge 18),
    all(concept pop.adult)
  )
  condition any(
    all(concept cond.asthma, concept severity.moderate),
    all(concept cond.asthma, concept severity.severe)
  )
  action prescribe drug.inhaled_corticosteroid
  direction for
  strength strong
  certainty moderate
  timing within interval q.duration_days ge 0 le 7
  except e.contra_beta:
    all(concept cond.beta_blocker_contraindicated)
```

### Grammar Sketch

Keep grammar line-oriented. Prefer identifiers over free words. No pronouns. No implicit anaphora. No hidden defaults except explicitly versioned parser defaults.

```text
file        = rec+
rec         = "rec" id ":" nl field+
field       = source | population | condition | action | direction | strength | certainty | timing | exception
source      = indent "source" id_list nl
population  = indent "population" ctx nl
condition   = indent "condition" ctx nl
action      = indent "action" action_kind concept_id nl
direction   = indent "direction" ("for"|"against"|"contraindicate"|"require"|"permit"|"avoid") nl
strength    = indent "strength" ("strong"|"weak") nl
certainty   = indent "certainty" ("high"|"moderate"|"low"|"very_low") nl
timing      = indent "timing" timing_expr nl
exception   = indent "except" id ":" nl indent2 ctx nl
ctx         = "all(" atom_list ")" | "any(" ctx_list ")"
atom        = "concept" concept_id | "not" "concept" concept_id | interval_atom | slot_atom
interval_atom = "interval" quantity_id bound+
bound       = ("ge"|"gt"|"le"|"lt") int
```

Parser normalization:

- `population` and `condition` normalize to finite DNF.
- Single `all(...)` is one DNF conjunct.
- `any(all(...), all(...))` is canonical DNF.
- Sort atoms by canonical key after parse unless source-order preservation is needed for diagnostics.
- Bound pairs become exact interval objects.
- Exceptions remain separate rule-referencing payloads, not just negated context conjuncts, then SMT lane can compile them to negated conjuncts.

### Why This Beats JSON

- Smaller terminal set than JSON schema.
- No quoting/comma/bracket brittleness beyond a tiny expression grammar.
- IDs/codes are whole terminals supplied by grounding.
- Invalid partial outputs fail near the smallest field.
- The accepted object can still be serialized as canonical JSON after parse.

## `ckc_slot_cnl.v1`

Purpose: test a more readable model target inspired by BRIDGE-Wiz/EARS/FRET.

Example:

```text
recommendation r.abx_a.adult_sepsis:
  when the patient is an adult and has sepsis
  the clinician should administer antibiotic-a
  strength strong
  certainty moderate
  unless the patient has severe renal disease
  source s.g1.rec3
```

Rules:

- Lexicon maps controlled phrases to IDs; no open noun parsing.
- Multiword concepts are either lexicon phrases or hyphenated identifiers.
- `when` = context, `unless` = exception, deontic word maps to direction+default strength only when strength missing is allowed by route config.
- Better for audit, likely worse for weak-model emission. Run as an ablation, not primary.

## `logic_template.v1`

Purpose: test Logical-English/PENG proximity without depending on their parser.

Example:

```text
template applies to rule *a rule* and patient *a patient*.
template exception applies to rule *a rule* and patient *a patient*.
template patient *a patient* should receive *a drug* under rule *a rule*.

patient P should receive drug.abx_a under rule r.abx_a.adult_sepsis
if applies to rule r.abx_a.adult_sepsis and patient P
and exception applies to rule r.abx_a.adult_sepsis and patient P is false.
```

Likely verdict: useful for explanation/proof templates, too verbose as model target.

## Deterministic Compile Targets

### SMT Lane

Keep current SMT conflict checks as the classical oracle:

```text
Q1: conditioned context overlap
Q2: deontic direction conflict over shared action
```

Exception handling:

```text
conditioned_context(R) = context(R) AND NOT exception_applies(R,E1) AND ...
```

Do not let LP semantics change accepted SMT conflict results without an explicit §13.2 richer-rule-semantics adoption record.

### Logic-Program Lane

Recommended v1: rules-as-data plus small fixed kernel. Emit pure Prolog first; optionally emit s(CASP annotations for explanations.

Program shape:

```prolog
rule(r_abx_a_adult_sepsis).
source(r_abx_a_adult_sepsis, s_g1_rec3).
population(r_abx_a_adult_sepsis, concept(pop_adult)).
condition(r_abx_a_adult_sepsis, concept(cond_sepsis)).
action(r_abx_a_adult_sepsis, administer, drug_abx_a).
direction(r_abx_a_adult_sepsis, for).
strength(r_abx_a_adult_sepsis, strong).
certainty(r_abx_a_adult_sepsis, moderate).
exception(r_abx_a_adult_sepsis, e_renal).
exception_atom(e_renal, concept(cond_renal_severe)).

applies(R, P) :-
  rule(R),
  population_holds(R, P),
  condition_holds(R, P),
  not exception_applies(R, P).

advice(P, Action, Direction, Strength, Certainty, R) :-
  applies(R, P),
  action(R, Kind, Target),
  Action = action(Kind, Target),
  direction(R, Direction),
  strength(R, Strength),
  certainty(R, Certainty).
```

For s(CASP):

```prolog
#pred applies(R,P) :: 'rule @(R) applies to patient @(P)'.
#pred exception_applies(R,P) :: 'an exception to rule @(R) applies to patient @(P)'.
#pred advice(P,A,D,S,C,R) :: 'for patient @(P), rule @(R) gives @(S) @(D) advice: @(A), certainty @(C)'.
```

Use s(CASP for:

- patient-context expert-system queries,
- abduction/unknown exploration,
- proof trees,
- exception explanations.

Use SMT for:

- cross-rule classical conflict checks,
- interval/context satisfiability,
- deterministic no-conflict evidence.

### Conflict Between SMT and LP Semantics

Expected mismatch:

- SMT is closed over declared conflict atoms and classical satisfiability.
- s(CASP)/ASP is nonmonotonic and query-driven; `not` means failure to prove under program semantics.

Mitigation:

- Label lanes separately in reports.
- Add differential tests only for shared fragments:
  - context_holds equivalence on finite patient fixtures,
  - exception expansion equivalence,
  - action/direction extraction equivalence.
- Do not claim LP verdicts as replacements for SMT verdicts until a §13.2 trigger is accepted.

## Deterministic Verbalization

### EN Audit Template

Canonical English:

```text
Rule r.abx_a.adult_sepsis.
Source: s.g1.rec3.
For patients who are adults and have sepsis, clinicians should administer antibiotic A.
Strength: strong. Certainty: moderate.
Exception: this rule does not apply to patients with severe renal disease.
```

Rules:

- Always state rule id and source id.
- One rule per paragraph.
- Use controlled deontic mapping:
  - `require`: must
  - `for` + strong: should
  - `for` + weak: may consider
  - `permit`: may
  - `against` + strong: should not
  - `against` + weak: generally should not
  - `avoid`: should avoid
  - `contraindicate`: must not / is contraindicated
- Render DNF explicitly:
  - `any(all(A,B), all(C,D))` -> "either (A and B) or (C and D)".
- Render exceptions as "does not apply when ..." instead of burying them in the condition.

### JA Audit Template

Implement by hand first. Store Japanese surface strings in lexicon entries; keep this scratch text romanized to avoid accidental poor Japanese in design spec.

Shape:

```text
Rule <id>.
Source: <source>.
<population> de, <condition> no baai, <action> koto o <direction/strength> suru.
Evidence certainty: <certainty>.
Tadashi, <exception> no baai wa tekiyo shinai.
```

Natural Japanese final renderer should use:

- explicit patient subject; avoid zero pronouns,
- explicit particles,
- one proposition per sentence,
- interval markers that distinguish closed/open bounds,
- no broad relative-clause nesting,
- fixed register flag (`de aru` vs polite) if needed.

Path:

1. EN audit verbalizer + parse test.
2. JA audit verbalizer over same AST; no JA parsing.
3. Parser for own generated JA templates only if needed.
4. GF concrete syntax only when JA parse or >2 languages becomes valuable.

## Validation Plan

### Parser/Grammar

- `parse(dsl) -> IR` accepts only canonical grammar.
- `verbalize_dsl(parse(dsl))` produces canonical DSL.
- `parse(verbalize_dsl(IR)) == IR`.
- Enumerate ASTs to bounded depth, assert single parse tree per generated string.
- Fuzz malformed strings: unresolved ids, duplicate fields, missing required field, bad interval bounds, ambiguous DNF, unknown concept.
- Tokenizer audit against the actual constrained-decoding engine: every keyword and common delimiter should be stable; reject grammar terminals that split pathologically.

### Semantic Validators

Borrow GLIA/COGS/BRIDGE-Wiz checks:

- hidden actor,
- vague action,
- non-decidable condition,
- non-executable action,
- mixed AND/OR without explicit grouping,
- missing strength/certainty/source,
- interval with empty bounds,
- incompatible units,
- exception that duplicates full context,
- rule with unsatisfiable own context,
- action vocabulary mismatch,
- source-span support missing.

### Compiler Tests

- Deterministic byte-pinned SMT output unchanged for M1/M2 accepted fragments.
- LP output has canonical predicate order and id escaping.
- SMT vs LP fixture equivalence for shared finite patient contexts.
- s(CASP proof tree includes rule id, source id, exception id, and action id through `#pred` templates.

### Route Metrics

Add to M4 comparison:

```text
token_count_per_accepted_rule
grammar_valid_rate
semantic_valid_rate
repair_count
accepted_ir_hash_convergence
source_support_rate
round_trip_identity_rate
SMT_verdict_accuracy
LP_fixture_accuracy
clinician_audit_readability_proxy (lint only until human review gate)
```

## Milestone Placement

### M3

- Register method compendium entries:
  - `method.ace_attempto`
  - `method.logical_english`
  - `method.scasp`
  - `method.pengasp`
  - `method.fret`
  - `method.bridge_wiz`
  - `method.cql_elm`
  - `method.gf_japanese`
- Use BRIDGE-Wiz/GEM/COGS/GLIA as lint inspirations for M3 route/schema validators.
- Keep CQL/ELM/FHIR as export/backlog candidates.

### M4

Implement at least two invented candidates:

1. `route.ckc_rec_dsl.v1`: primary compact DSL.
2. `route.ckc_slot_cnl.v1`: readable slot-CNL ablation.

Optional:

- `route.logic_template.v1`: Prolog-isomorphic surface ablation.
- `informalization_round_trip`: accepted IR -> EN audit -> parse -> normalized hash.

### §13.2 Backlog

Add candidates behind triggers:

- `target.scasp`: executable expert-system lane with proof verbalization.
- `target.cql_elm`: clinical standards export.
- `verbalizer.gf_ja`: GF-based multilingual generation/parsing.
- `rule_semantics.defeasible`: priorities/superiority when exception-as-context underfits.

## Risks

| risk | mitigation |
|---|---|
| Grammar valid but clinically wrong | source-span support checks, reference fixtures, human audit later |
| JSON/schema false confidence | DSL parser + semantic validators + canonical JSON only after parse |
| LP nontermination | finite fragment first; s(CASP query budgets; pure Prolog fixtures |
| SMT/LP semantic drift | lane labels + shared-fragment differential tests |
| JA prose ambiguity | generation-only templates, no ellipsis, one proposition per sentence |
| License contamination | clean-room reimplementation; dependencies only after file-level license audit |
| Overlarge M4 unit | split grammar/parser, verbalizer, SMT bridge, LP bridge, route integration into separate units |

## Concrete Planning Recommendation

Plan M4 as small units:

1. `ckc-rec-dsl-grammar`: grammar spec + parser AST + parse error taxonomy.
2. `ckc-rec-dsl-bridge`: AST -> existing ClinicalIR/NormIR for M1/M2 fields.
3. `ckc-rec-dsl-verbalize`: IR -> canonical DSL + round-trip property tests.
4. `ckc-audit-en`: deterministic English audit renderer from accepted IR.
5. `ckc-rec-dsl-route`: registry schema/prompt + recorded route fixture.
6. `ckc-slot-cnl-ablation`: readable slot-CNL parser/bridge, limited to same semantic subset.
7. `lp-target-prototype` (if accepted into M4 or §13.2 prep): rules-as-data Prolog emitter + fixture queries + optional s(CASP `#pred` templates.

Keep JA verbalization as a separate later unit unless M4 explicitly needs bilingual audit output.

