# ClinicalCNL v1 surface

Authoritative surface + framing spec for the ClinicalCNL EN v1 product line (SPEC §10.6). Fixes
the raw-document grammar, the per-sentence ACE surface, and the APE product seam. Every downstream
unit (`raw-gate`, `profile-drs`, `map-*`, `conflict-*`) builds on the DRS byte shapes pinned here.

Semantics = the M3 `D1`-`D10` decisions (`.agent/roadmap.md` §M3 replan, grounded in §L·probe).
Surfaces + DRS bytes + message lists = OBSERVED via the seam, never hand-written;
`clinical/surface_goldens.pl` replays every construct byte-exact over `clinical/goldens/`
(`surface_cases.pl` seeds, `surface_expected.pl` observed bytes). Each construct cites its golden id.

Every golden is one of three kinds under the seam and the v1 profile:

- `v1` — a v1-admissible surface: the seam returns `text/plain` + a serialized DRS AND zero
  messages (the fail-closed zero-message law holds).
- `nonv1` — APE accepts it (`text/plain` + DRS) but the surface is excluded from v1; a later lane
  (raw gate / `profile-drs`) rejects it. Its messages are pinned as evidence — empty for a
  shape-rule reject (single-bound law, in-guard negation), a warning for a message-driven reject
  (anaphor, the undefined-word `named` hole).
- `reject` — `text/xml` `<messages>`: APE itself fails closed on an error message.

## Product seam

Canonical APE invocation (`clinical/surface_goldens.pl` `run_seam/4`) — deterministic:

    get_ape_results([text=<ACE>, noclex=on, ulextext=<ulex>, guess=off, solo=drs], CT, Content)

- `noclex=on` — ulex-only; an unregistered lowercase content word is a hard error (`reject_oov_noun`,
  `reject_oov_verb`: `Use the prefix n:, v:, a: or p:.`).
- `guess=off` — no lowercase-word class guessing. It does NOT stop a capitalised OOV token from
  becoming a proper name: a capitalised unregistered word parses to `named(_)` with a warning
  regardless of `guess` (see the named-word hole below). Registration, not `guess`, is the gate.
- `solo=drs` — `Content` = the serialized DRS: `numbervars` (`A`,`B`,...), operators `-` `~` `=>`
  `v` `&` written in functional form (`-(...)`, `=>(...,...)`), `quoted(true)`. This is THE golden
  byte format (`prolog/utils/serialize_term.pl`).
- Discriminator: `CT=text/plain` + serialized DRS ⇒ APE parse; `CT=text/xml` + `<messages>` ⇒ APE
  reject. Downstream keys on `CT`, never on an exit code.

Fail-closure is NOT the seam alone. `get_solo_content` (`prolog/ape.pl`) keeps only ERROR messages
(comment: "we do not care about the warning messages"), so a parse that emits only WARNINGS still
returns `text/plain` + DRS with the warnings dropped. The v1 profile is fail-closed across three
layers: (1) the raw gate (a registry-driven whitelist BEFORE APE); (2) registry membership — a
registered `pn_sg` yields `named()` with zero warnings, an OOV capital yields `named()` WITH a
warning, so the registry is the authoritative `named(_)` discriminator; (3) the post-APE profile
zero-message law — an accepted document must be warning-free. The goldens capture the full message
list (`run_seam/4` reads the `error_logger` store the seam would drop) so `profile-drs` keys its
reject on observed warning bytes.

Per-sentence (`D2`): each rule/exception ACE sentence is parsed alone ⇒ SID is always `1`; each
atomic condition carries `-1/TID` provenance. This structurally prevents cross-sentence referent
merging. Only atomic conditions (`object`/`predicate`/`relation`) carry `-SID/TID`; the DRS
operator conditions (`should`/`may`/`can`/`=>`/`-`) do not.

## Framing grammar (raw document)

A ClinicalCNL v1 document is line-oriented; blank lines separate blocks. The raw-gate whitelists
this frame and emits one ACE sentence per rule-disjunct / exception to the seam. Byte-level lexing
(exact whitespace, newline, and non-emptiness laws) is finalised by `raw-gate`; this grammar fixes
the block structure and field vocabulary.

    document   ::= "document" WS doc-id NL blank { block }
    block      ::= (rule-block | exc-block) NL blank
    rule-block ::= "rule" WS k WS keyword { WS field } NL ace-rule { NL ace-rule }
    exc-block  ::= "exception" WS k WS "rule" WS k { WS "basis" WS quoted-string } NL ace-cond
    field      ::= "certainty" WS cert | "basis" WS quoted-string
    ace-rule   ::= a framed rule sentence (§Modality) — one line
    ace-cond   ::= a bare condition assertion (§Exceptions) — one line
    cert       ::= "high" | "moderate" | "low" | "very_low"

- `doc-id` — non-empty corpus document id (e.g. `test_source.m1_guideline_a`).
- `k` — non-negative integer; document-continuous per the `kb-contract` `stmt.k`/`exc.k` counters.
- `keyword` — one modality keyword (§Modality); carries direction + strength + required op.
- `certainty` — optional on a RULE block, at most once; a `D7` token `{high|moderate|low|very_low}`. A
  raw header field only, NEVER in the ACE sentence; carried to the KB rule-level `certainty` field. An
  exception block admits NO certainty (certainty keys on the rule id — there is no exception-certainty
  slot), so a certainty on an exception rejects fail-closed (`certainty_on_exception`).
- `basis` — optional, at most once; a double-quoted evidence string. Both rule and exception blocks
  carry their own basis (SPEC §10.6: each labeled exception has its own basis). Provenance only,
  never parsed as ACE.
- `ace-rule { NL ace-rule }` — one OR MORE rule sentences. Multiple sentences are the DNF disjuncts
  of one rule (§Disjunction): they share rule id `k` and map to `stmt.0`..`stmt.n`.

## Modality (`D1`)

Direction + strength are authored in the raw keyword (APE cannot express strength); the ACE frame
carries direction as a DRS operator. The raw-gate cross-checks that the frame's op equals the
keyword's required op (`op-mismatch` reject battery — `raw-gate-battery`).

A rule sentence is `If <guard> then <frame-clause>.`, where `<frame-clause>` applies one modality
frame to `<action>`. The whole sentence parses to `=>(drs(<guard-conds>), drs([], [<op>]))` — the
op lives in the CONSEQUENT, inside the implication (goldens `frame_*`, all over a uniform sepsis
guard so the op is the only variable):

| direction      | frame-clause (over `<action>`)        | consequent DRS op                | golden               |
| -------------- | ------------------------------------- | -------------------------------- | -------------------- |
| for            | `it is recommended that <action>`     | `should(<action>)`               | `frame_recommend`    |
| permit         | `it is admissible that <action>`      | `may(<action>)`                  | `frame_admissible`   |
| against        | `it is not recommended that <action>` | `-(drs([],[should(<action>)]))`  | `frame_not_recommend`|
| contraindicate | `it is not possible that <action>`    | `-(drs([],[can(<action>)]))`     | `frame_not_possible` |

`<action>` = `the patient takes <Drug>` ⇒ `should(drs([D],[predicate(D,take,PatientRef,named(<Drug>))]))`.

Keyword ⇒ (required op, direction group, strength) — total + injective (`1:1` decode):

| keyword          | required op | direction      | strength |
| ---------------- | ----------- | -------------- | -------- |
| `recommend`      | `should`    | for            | strong   |
| `suggest`        | `should`    | for            | weak     |
| `may-consider`   | `may`       | permit         | weak     |
| `not-recommend`  | `-should`   | against        | strong   |
| `not-suggest`    | `-should`   | against        | weak     |
| `contraindicate` | `-can`      | contraindicate | strong   |

Auxiliary modal surfaces (`the patient should take Abx-A`, `may`, `cannot`) and the `possible`⇒`can`
/ `necessary`⇒`must` frames parse at APE but are UNREGISTERED / non-v1 frames the raw-gate whitelist
excludes, keeping one canonical surface per direction. They carry no golden — they never reach the
seam.

## Lexicon and guards (`D2`/`D3`/`D5`)

- Population — introduced once as `a patient` in the first guard conjunct; every later mention is
  the definite `the patient` (within-sentence anaphora, warning-free).
- Condition — a registered countable noun surfaced as `<patient-ref> has a <cond>` ⇒
  `object(_,<cond>,countable,na,eq,1)` + `predicate(_,have,PatientRef,_)` in the guard (goldens
  `frame_*` pin `sepsis`). Conditions require the determiner `a`; bare (`has sepsis`) rejects. Mass
  nouns are excluded v1 (ACE demands a determiner on mass nouns).
- Negated condition (`D5`) — in-guard `<patient-ref> does not have a <cond>` PARSES clean (zero
  messages) to `-(drs([...],[object,predicate(have)]))` nested in the guard (golden `guard_neg`),
  but is DEFERRED from v1: it is a profile reject-battery member. All negative context enters v1
  through labeled exceptions (NAF), never an in-guard negation.
- Drug — a registered `pn_sg` proper name; action surface `<patient-ref> takes <Drug>` ⇒
  `predicate(_,take,PatientRef,named(<Drug>))`. The v1 action verb `takes` maps to `act.administer`.
- Condition verb `has`/`have` and action verb `takes`/`take` are the only v1 verbs.

## Anaphora (refines SPEC §10.6)

SPEC §10.6's pre-APE transplant reads "no pronouns, anaphora, ellipsis, or definite references".
The APE profile refines this to the two closed-class definite forms the ACE surface mechanically
requires, and forbids everything else:

- Sanctioned: the modality frame subject `It` (`It is recommended that ...`), and the
  antecedent-bound within-sentence `the patient` (bound to the guard's `a patient`, warning-free).
- Forbidden (raw-gate reject + zero-message law): cross-sentence anaphora (silently merges a prior
  referent, zero warning — killed structurally by per-sentence parsing, `D2`), free pronouns, and
  any unbound definite (`the age of a patient is ...` ⇒ an anaphor warning, golden `iv_anaphor`).

SPEC §10.6 already delegates surface authority to this file; this section is that refinement.

## Intervals (`D8`/`D9`)

Interval guard surface: `<patient-ref> has an age of <marker> <INT> <year-noun>`, where the quantity noun
agrees with the value — APE parses `1 year` and `18 years` but rejects `1 years` / `2 year`, so the raw gate
takes the singular noun for value 1 and the plural otherwise (a mismatch is a `malformed_sentence` reject). The
count operator and its DRS placement vary by marker; the guard walker (`map-core`) flattens the one-level nesting.

| marker         | CountOp   | placement            | v1  | golden        |
| -------------- | --------- | -------------------- | --- | ------------- |
| `at least`     | `geq`     | top-level `object`   | yes | `iv_at_least` |
| `more than`    | `greater` | top-level `object`   | yes | `iv_more_than`|
| `at most`      | `leq`     | nested sublist       | yes | `iv_at_most`  |
| `less than`    | `less`    | nested sublist       | yes | `iv_less_than`|
| `exactly`      | `exactly` | nested sublist       | no  | `iv_exactly`  |
| (bare `18`)    | `eq`      | top-level `object`   | no  | `iv_bare`     |

The guard conjunct is `object(_,age,...)`, `object(_,year,countable,na,<CountOp>,18)`,
`relation(Age,of,Year)`, `predicate(_,have,PatientRef,Age)`. Top-level markers put the bounded
`object(...,year,...)` directly among the guard conditions; nested markers wrap `relation(...)` +
the bounded `object(...)` in a sublist `[relation(...),object(...)]`.

The alternative `the age of <patient-ref> is <marker> <INT> years` surface is NON-v1: it parses to a
`predicate(be,...)` copula shape WITH an anaphor warning (`the age` has no antecedent), golden
`iv_anaphor`. `D9` selected the `has an age of` surface precisely because it is warning-free.

Single-bound law: v1 admits only the open/closed single-bound markers (`geq`/`greater`/`leq`/
`less`). `exactly` and bare `eq` PARSE clean at APE but are rejected at the raw/profile lane by the
shape rule, not by a message (goldens `iv_exactly`/`iv_bare` are pinned so `profile-drs` knows the
eq/exactly shape it must reject). Malformed number/unit phrasings ARE APE rejects: no unit noun
(`reject_iv_no_years`), a decimal (`reject_iv_decimal`), `years old` (`reject_iv_years_old`).
Conflict arithmetic (`D10`) treats open vs closed bounds distinctly over exact rationals.

## Disjunction (`D4`)

`or`-guards are rejected (they yield `v(drs,drs)` with a broken then-part anaphora). A disjunctive
rule is authored as one rule sentence per disjunct (`ace-rule { NL ace-rule }` in one rule-block):
the disjuncts share the block's rule id `k` and map to `stmt.0`..`stmt.n` (statement-major). Each
disjunct sentence is an ordinary single-guard rule. The `stmt.k`/counter semantics are fixed by
`kb-contract`; the raw-gate parses the grouping.

## Exceptions (`D6`)

An exception block carries a rule ref (and its own basis; §Framing grammar) and one bare condition
assertion (no modality frame; the direction/strength come from the parent rule). The body is
self-contained, single-concept, and interval-free — `A patient has a <cond>.` — and parses to a
bare `object`+`predicate(have)` DRS with no `=>` and no operator (golden `exception_body`, which is
docA's `exc.0`, `severe-renal-impairment`). `map-exc` compiles the exception condition into a NAF-
guarded PROLEG override on the referenced rule's statements.

## Named-word hole (`p6`)

A capitalised OOV token in the drug slot (`A patient takes Widget.`) parses to
`predicate(_,take,_,named('Widget'))` with a warning `Undefined word. Interpreted as a singular
proper name.` — under `guess=off`, and `solo=drs` drops the warning to `text/plain` (golden
`named_hole`, kind `nonv1`). This is THE hole; the raw gate (whitelist) + registry membership +
the profile zero-message law close it. Contrast `reject_oov_noun` (a LOWERCASE OOV is a hard error,
a true `reject`). An indefinite article before a proper name (`takes a Widget`) is a separate syntax
reject, unrelated to OOV-ness.

## Goldens

`clinical/goldens/` pins the seam over 24 cases; `clinical/surface_goldens.pl` asserts the
kind↔content-type invariant, the v1 zero-message law, and byte-exact `CT`/`Content`/`Messages` per
case, and regenerates `surface_expected.pl` via `capture/0`.

- `v1` (`text/plain`, zero messages): the 4 direction frames, 4 v1 interval markers, 3 §8.6 thread
  composites (`thread_doc_a`/`thread_doc_b`/`thread_control`), 1 exception body.
- `nonv1` (`text/plain`, pinned messages): in-guard negation (`guard_neg`), non-v1 intervals
  (`iv_exactly`/`iv_bare` shape-rule; `iv_anaphor` warning), the `named_hole` warning.
- `reject` (`text/xml`): malformed frames (`reject_frame_*`), malformed intervals (`reject_iv_*`),
  lowercase OOV classes (`reject_oov_*`).

The v1 ulex (`surface_cases.pl` `surface_ulex/1`) freezes the §L·ids registry surfaces (full
inflection families, some unexercised by the cases but frozen so the mirror is complete); the `ulex`
unit builds the canonical `clinical_ulex.pl` and must stay byte-consistent with this frozen set.

Gate: `swipl -q -g "consult('clinical/surface_goldens.pl'), (run_tests(surface_goldens)->halt(0);halt(1))" -t 'halt(1)'`
Regenerate: `swipl -q -g "consult('clinical/surface_goldens.pl'), surface_goldens:capture, halt" -t 'halt(1)'`
