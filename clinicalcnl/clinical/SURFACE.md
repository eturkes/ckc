# ClinicalCNL v1 surface

Authoritative surface + framing spec for the ClinicalCNL EN v1 product line (SPEC §10.6). Fixes
the raw-document grammar, the per-sentence ACE surface, and the APE product seam. Every downstream
unit (`raw-gate`, `profile-drs`, `map-*`, `conflict-*`) builds on the DRS byte shapes pinned here.

Semantics = the M3 `D1`-`D10` decisions (`.agent/roadmap.md` §M3 replan). Surfaces + DRS bytes =
OBSERVED via the seam, never hand-written; `clinical/surface_goldens.pl` replays every construct
byte-exact over `clinical/goldens/` (`surface_cases.pl` seeds, `surface_expected.pl` observed bytes).
Each construct below cites its golden id.

## Product seam

Canonical APE invocation (`clinical/surface_goldens.pl` `run_seam/3`) — fail-closed, deterministic:

    get_ape_results([text=<ACE>, noclex=on, ulextext=<ulex>, guess=off, solo=drs], CT, Content)

- `noclex=on` — ulex-only; every content word must be registered, so unregistered words fail-close.
- `guess=off` — no proper-name guessing; a capitalised OOV token rejects instead of silently
  becoming `named(_)` (that hole is a `guess=on`/raw-layer behaviour, closed at the seam).
- `solo=drs` — `Content` = the serialized DRS: `numbervars` (`A`,`B`,...), operators `-` `~` `=>`
  `v` `&` written in functional form (`-(...)`, `=>(...,...)`), `quoted(true)`. This is THE golden
  byte format (`prolog/utils/serialize_term.pl`).
- Discriminator: `CT=text/plain` + serialized DRS ⇒ parse; `CT=text/xml` + `<messages>` ⇒ APE
  reject. Downstream keys on `CT`, never on an exit code.
- Per-sentence (`D2`): each rule/exception ACE sentence is parsed alone ⇒ SID is always `1`; every
  condition carries `Cond-1/TID` provenance. This structurally prevents cross-sentence referent
  merging.

## Framing grammar (raw document)

A ClinicalCNL v1 document is line-oriented; blank lines separate blocks. The raw-gate whitelists
this frame and emits one ACE sentence per rule/exception block to the seam.

    document   ::= "document" WS doc-id NL blank { block }
    block      ::= (rule-block | exception-block) NL blank
    rule-block ::= "rule" WS k WS keyword { WS field } NL ace-sentence
    exc-block  ::= "exception" WS k WS "rule" WS k NL ace-sentence
    field      ::= "certainty" WS cert | "basis" WS quoted-string

- `doc-id` — corpus document id (e.g. `test_source.m1_guideline_a`).
- `k` — non-negative integer; document-continuous per the `kb-contract` `stmt.k`/`exc.k` counters.
- `keyword` — one modality keyword (§Modality).
- `certainty` — optional; a token from the certainty registry (enumeration fixed by `registry`/
  `kb-contract`). Carried through to the KB `certainty` field.
- `basis` — optional; a double-quoted evidence string. Carried as provenance, never parsed as ACE.
- `ace-sentence` — exactly one line: a framed rule sentence (rule blocks) or a bare conditional
  (exception blocks). Both go verbatim to the seam.

## Modality (`D1`)

Direction lives in the ACE frame (APE-visible); strength lives in the raw keyword (APE cannot
express it). A rule block authors BOTH; the raw-gate cross-checks that the frame op equals the
keyword's required op (`op-mismatch` reject battery — `raw-gate-battery`).

Frame surface ⇒ DRS op (goldens `frame_*`):

| direction      | frame surface (`S` = `if <guard> then <action>`) | DRS op                    | golden               |
| -------------- | ------------------------------------------------ | ------------------------- | -------------------- |
| for            | `It is recommended that S.`                      | `should(...)`             | `frame_recommend`    |
| permit         | `It is admissible that S.`                       | `may(...)`                | `frame_admissible`   |
| against        | `It is not recommended that S.`                  | `-(drs([],[should(...)]))`| `frame_not_recommend`|
| contraindicate | `It is not possible that S.`                     | `-(drs([],[can(...)]))`   | `frame_not_possible` |

Keyword ⇒ (required op, direction group, strength) — total + injective (`1:1` decode):

| keyword          | required op | direction      | strength |
| ---------------- | ----------- | -------------- | -------- |
| `recommend`      | `should`    | for            | strong   |
| `suggest`        | `should`    | for            | weak     |
| `may-consider`   | `may`       | permit         | weak     |
| `not-recommend`  | `-should`   | against        | strong   |
| `not-suggest`    | `-should`   | against        | weak     |
| `contraindicate` | `-can`      | contraindicate | strong   |

Auxiliary modal surfaces (`A patient should take Abx-A.`, `may`, `cannot`) parse at APE but hoist
to the same `should()`/`can()` DRS — they are UNREGISTERED and raw-gate-rejected, keeping one
canonical surface per direction. `possible`⇒`can` and `necessary`⇒`must` also parse but are non-v1
frames the raw-gate whitelist excludes, so they never reach the seam.

## Lexicon and guards (`D2`/`D3`/`D5`)

- Population — introduced once as `a patient` in the first guard conjunct; every later mention is
  the definite `the patient` (within-sentence anaphora, warning-free).
- Condition — a registered countable noun surfaced as `<patient-ref> has a <cond>` (golden
  `guard_condition`). Negation: `<patient-ref> does not have a <cond>` ⇒ a clean `-(drs(...))`
  nested inside the guard (golden `guard_neg_condition`). Conditions require the determiner `a`;
  bare (`has sepsis`) rejects. Mass nouns are excluded v1 (ACE demands a determiner on mass nouns).
- Drug — a registered `pn_sg` proper name; action surface `<patient-ref> takes <Drug>` ⇒
  `predicate(_,take,PatientRef,named(<Drug>))`. The v1 action verb `takes` maps to `act.administer`.
- Condition verb `has`/`have` and action verb `takes`/`take` are the only v1 verbs.

## Intervals (`D8`/`D9`)

Interval guard surface: `the age of <patient-ref> is <marker> 18 years`. The count operator and its
DRS placement vary by marker; the guard walker (`map-core`) flattens the one-level nesting.

| marker         | CountOp   | placement            | v1  | golden        |
| -------------- | --------- | -------------------- | --- | ------------- |
| `at least`     | `geq`     | top-level `object`   | yes | `iv_at_least` |
| `more than`    | `greater` | top-level `object`   | yes | `iv_more_than`|
| `at most`      | `leq`     | nested sublist       | yes | `iv_at_most`  |
| `less than`    | `less`    | nested sublist       | yes | `iv_less_than`|
| `exactly`      | `exactly` | nested sublist       | no  | `iv_exactly`  |
| (bare `18`)    | `eq`      | top-level `object`   | no  | `iv_bare`     |

Nested placement means the `predicate(be,...)` and the bounded `object(...)` sit in a sublist
`[predicate(be,...),object(...)]` inside the guard conditions; top-level markers put the bounded
`object(...)` directly among the guard conditions.

Single-bound law: v1 admits only the open/closed single-bound markers (`geq`/`greater`/`leq`/
`less`). `exactly` and bare `eq` PARSE at APE but are rejected at the raw/profile lane, not at APE
(goldens `iv_exactly`/`iv_bare` are pinned so `profile-drs` knows the eq/exactly shape it must
reject). Conflict arithmetic (`D10`) treats open vs closed bounds distinctly over exact rationals.

## Disjunction (`D4`)

`or`-guards are rejected (they yield `v(drs,drs)` with a broken then-part anaphora). A disjunctive
rule is authored as one rule sentence per disjunct, all grouped under one rule id as `stmt.k`
(statement-major). Each disjunct sentence is an ordinary single-guard rule.

## Exceptions (`D6`)

An exception block carries a rule ref and one bare conditional ACE sentence (no modality frame; the
direction/strength come from the parent rule). It parses to a bare `=>/2` DRS (golden
`exception_body`): `If <exception-guard> then <parent-action>.`. Bodies are self-contained (a fresh
`a patient`). `map-exc` compiles the exception guard into a NAF-guarded PROLEG override on the
referenced rule's statements.

## Goldens

`clinical/goldens/` pins two APE-parseability classes; `clinical/surface_goldens.pl` asserts the
class↔content-type invariant plus byte-exact `CT`/`Content` per case, and regenerates
`surface_expected.pl` via `capture/0`.

- parses (`text/plain`): the 4 frames, 2 guard atoms, 4 v1 interval markers + 2 non-v1 eq surfaces,
  3 §8.6 thread composites (`thread_doc_a`/`thread_doc_b`/`thread_control`), 1 exception body.
- rejected (`text/xml`): malformed frames (`reject_frame_*`), malformed interval phrasing
  (`reject_iv_*`: missing unit, `years old`, decimal), OOV classes (`reject_oov_*`: lowercase noun,
  capitalised token, unregistered verb).

The v1 ulex (`surface_cases.pl` `surface_ulex/1`) freezes the §L·ids registry surfaces; the `ulex`
unit builds the canonical `clinical_ulex.pl` and must stay consistent with this frozen set.

Gate: `swipl -q -g "consult('clinical/surface_goldens.pl'), (run_tests(surface_goldens)->halt(0);halt(1))" -t 'halt(1)'`
Regenerate: `swipl -q -g "consult('clinical/surface_goldens.pl'), surface_goldens:capture, halt" -t 'halt(1)'`
