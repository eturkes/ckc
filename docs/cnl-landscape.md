# CNL IR Research Workflow Results

Resumed by Codex on 2026-07-07. Source workflow: `cnl-ir-research`. Prior harvested
reports: `cnl-attempto.md`, `cnl-multilingual-ja.md` (this dir). This file completes the
interrupted survey as a compact second-opinion result set covering the branches those two
skip: FRET/EARS, Logical English/s(CASP), PENG, clinical guideline languages, broader CNL
landscape, LLM constrained-decoding evidence.

## Executive Verdict

No existing system satisfies CKC exactly:

```text
weak local model writes constrained surface
-> deterministic parse to CKC IR
-> deterministic compile to SMT + executable Prolog-family rules
-> deterministic verbalization back to clinician-readable EN/JA audit prose
```

Best answer: build a small CKC-owned DSL/CNL over existing CKC IR. Reimplement clean-room. Mine these systems:

1. ACE/APE/AceRules/AceWiki: deterministic interpretation rules, parser/verbalizer/paraphrase loop, deontic surface words, predictive editor.
2. Logical English + s(CASP): predicate templates, Prolog/s(CASP target, `#pred` proof verbalization, legal/regulatory examples.
3. PENGASP: single bidirectional grammar, writer/reader/planner, ASP round-trip, lookahead.
4. FRET/EARS/BRIDGE-Wiz: slot order, template semantics, simulator/counterexample feedback, clinical recommendation deontic slots.
5. GF/Japanese RGL + Japanese controlled-language practice: hand-built JA generation first; GF only when parsing or >2 languages justifies it.

## Shortlist

| rank | system | CKC use | reason |
|---:|---|---|---|
| 1 | CKC-owned DSL/CNL | adopt/build | Only path that gives parse determinism, license cleanliness, compact grammar, round-trip, Japanese templates, and exact CKC semantics. |
| 2 | s(CASP) | adapt as executable lane | Goal-directed ASP, constraints, defaults, abduction, explanations, SWI integration, `#pred` natural-language proof templates. |
| 3 | ACE/APE/AceRules | mine/adapt | Strongest deterministic CNL lineage; DRS->ACE and paraphrase prove canonical verbalization loop; AceRules proves rule traces can be verbalized. |
| 4 | Logical English | mine/adapt | CNL as Prolog-ish predicate templates; legal/regulatory examples; active Apache-2 repo; partial bidirectional story. |
| 5 | FRET/BRIDGE-Wiz | mine/adapt | Best slot-template and formalization-feedback patterns; BRIDGE-Wiz is closest clinical CNL precedent. |

## Attempto Ecosystem

Source: `../survey-attempto.md`.

| system | mechanism | target | bidirectional | license/status | verdict |
|---|---|---|---|---|---|
| ACE 6.7 | English CNL with fixed interpretation rules; PENS P4 E3 N4 S3 | DRS/FOL-ish; modals include `must/should/may/can` with domain semantics | via APE/DRACE for APE-shaped DRS | spec frozen 2013 | adapt design, not language wholesale |
| APE | SWI-Prolog DCG, deterministic ACE->DRS | DRS, OWL/SWRL, RuleML, FOL, TPTP, paraphrases | yes for DRS->ACE/Core ACE subset | LGPL-3.0, repo pushed 2024-04 | dev-time oracle / mine |
| AceRules | ACE subset -> rule semantics | courteous/stable ASP-like rules, answers/traces verbalized | partial: answers/traces to ACE | LGPL-3.0, pushed 2024-11 | adapt semantics and trace verbalization |
| AceWiki/Codeco | predictive editor grammar with lookahead/anaphora markers | OWL-compatible ACE subset | editor prevents invalid text | LGPL-3.0+, pushed 2026-02 | mine constrained-decoding grammar ideas |
| ACE-in-GF | GF abstract/concrete syntax for ACE subset | multilingual surface, no DRS semantics | parse/generate in GF; ambiguity remains | no license, dormant 2021 | mine only |
| Clex | large ACE lexicon | lexicon | n/a | GPL-3.0, 2018 | reject |
| RACE | FO reasoner over ACE/DRS | consistency/proof/query | n/a | closed hosted SOAP | reject dependency |

CKC lessons:

- Copy: fixed attachment/scoping rules, hyphen/identifier gluing for multiword clinical concepts, canonical paraphrase from semantics, deontic surface vocabulary, pluggable lexicon, predictive-editor/lookahead.
- Avoid: silent definite-NP fallback, unknown-capitalized-word guessing, anaphora-heavy grammar, full-English verbosity as model target, semantics-free multilingualism.
- ACE contributed clinical guideline evidence: Shiffman et al. rewrote guideline recommendations in ACE; `should/must/may` were added for clinical guidelines but lacked fixed logical semantics. CKC can provide that semantics in SMT/LP compilers.

URLs:
http://attempto.ifi.uzh.ch/site/docs/ace/6.7/ace_constructionrules.html,
http://attempto.ifi.uzh.ch/site/docs/ace/6.7/ace_interpretationrules.html,
https://github.com/Attempto/APE,
https://github.com/tkuhn/AceRules,
https://github.com/AceWiki/AceWiki,
https://github.com/Attempto/ACE-in-GF,
https://aclanthology.org/J14-1005/,
https://aclanthology.org/W17-6805/.

## FRET / EARS / Requirements Templates

| system | mechanism | target | status/license | verdict |
|---|---|---|---|---|
| FRET/FRETISH | six-slot schema: scope, condition, component, shall, timing, response; current docs add probabilistic mode | future/past metric temporal logic, CoCoSpec/Lustre, Copilot/R2U2 monitors, SMV/PRISM views | NASA repo active; v3.1.0 2026-03; README says Apache-2.0 | mine architecture, not surface |
| EARS | fixed clause patterns: ubiquitous, state, event, optional feature, unwanted behavior, complex combos | no formal semantics by itself | widely used, tool-light | mine readability only |
| Rupp/SOPHIST MASTeR | subject/modality/action/object/condition template + QA checks | requirements text | active guidance | mine slot QA |
| ISO/IEC/IEEE 29148 | requirement-quality standard and binding terms | standards/process | ISO 29148:2018 current | mine QA vocabulary |

FRET lessons for CKC:

- Template-key matrix: finite slot combinations -> cached formal semantics + natural-language explanation + diagrams.
- Formalization feedback matters: show formula/prose/counterexample back to user.
- Realizability analogue: no satisfiable patient context, shadowed exception, conflicting recommendations, unreachable decision-table row.
- Do not reuse FRETISH directly: it is reactive component behavior, not deontic clinical recommendations.

URLs:
https://github.com/NASA-SW-VnV/fret,
https://raw.githubusercontent.com/NASA-SW-VnV/fret/master/PUBLICATIONS.md,
https://alistairmavin.com/ears/,
https://www.iso.org/standard/72089.html.

## Logical English / s(CASP) / Prolog-Isomorphic CNL

| system | mechanism | target | bidirectional | status/license | verdict |
|---|---|---|---|---|---|
| Logical English | predicate templates, indented rules, scenarios, queries; variables as typed noun phrases | Prolog / TaxLog / s(CASP work | partial: templates support explanations/rendering, no proven total bijection | Apache-2.0, active 2026 | mine/adapt |
| LPS | reactive rules + logic-program beliefs + constraints/actions | LPS/SWISH/lps.js | not CKC round-trip | BSD-ish ecosystem | niche temporal ideas |
| s(CASP) | goal-directed top-down constraint ASP, no grounding | stable-model reasoning with constraints | proof trees rendered via `#pred` templates | Apache-2.0 repo; active through 2025 | adapt as executable target |
| ErgoAI/Flora-2 | Rulelog/F-logic with defeasibility, WFS, explanations | ErgoAI engine | explanatory but heavy | Apache-2.0 active 2026 | benchmark only |

s(CASP) fit:

- Pros: defaults/exceptions, abduction, unknowns, numeric/dense constraints, proof trees, natural-language templates, SWI-Prolog integration.
- Risks: stable-model semantics differs from SMT conflict semantics; top-down execution can non-terminate; file-level license notices need audit before redistribution.
- CKC rule: Prolog/s(CASP is compiler output. Weak model should not author Prolog directly.

URLs:
https://github.com/LogicalContracts/LogicalEnglish,
https://raw.githubusercontent.com/LogicalContracts/LogicalEnglish/main/le_syntax.md,
https://github.com/SWI-Prolog/sCASP,
https://raw.githubusercontent.com/SWI-Prolog/sCASP/master/README.md,
https://ceur-ws.org/Vol-3437/paper7GDE.pdf,
https://flora.sourceforge.net/,
https://github.com/ErgoAI/ErgoEngine.

## PENG / PENGASP

| item | fact | verdict |
|---|---|---|
| lineage | 1995 Fuchs/Schwitter CNL -> Prolog; PENG-Light -> DRS/FOL/TPTP; PENGASP -> ASP | strong design pattern |
| grammar | unification/DCG + chart parser + anaphora + lexicon + lookahead | copy architecture |
| round-trip | writer builds ASP; reader reads ASP; planner aggregates; same grammar verbalizes | copy reader/planner/writer |
| ASP coverage | facts/classes, rules, choice, strong negation, NAF, strong/weak constraints, questions, event calculus | useful target comparison |
| software | no official public PENGASP repo/license found | no dependency |
| public comparators | CNL2ASP Apache-2.0, ASP2CNL MIT, CNLWizard MIT | inspect only |

CKC lessons:

- Keep NAF data semantics explicit: `not_recorded`, `known_absent`, `contraindication_absent`, `exception_applies`.
- Avoid clinician-facing `not provably` unless the audit view is explicitly explaining a closed-world proof.
- LLM-constrained decoding can reuse lookahead machinery: parser state -> legal next terminals.

URLs:
https://arxiv.org/abs/cmp-lg/9507009,
https://aclanthology.org/U09-1011/,
https://www.ijcai.org/proceedings/2020/773,
https://aclanthology.org/2021.cnl-1.5.pdf,
https://github.com/dodaro/cnl2asp,
https://github.com/dodaro/asp2cnl.

## Clinical / Guideline Languages

| system | shape | CKC lesson | verdict |
|---|---|---|---|
| Arden Syntax | HL7 MLMs, procedural slots, temporal/event/action logic; v3.0 STU | single-decision module pattern, event/action slots | reference/export only |
| CQL/ELM | CQL DSL -> ELM JSON/XML canonical AST over FHIR/QDM | strongest clinical formal mapping; ELM not source round-trip | export/test oracle |
| FHIR Clinical Reasoning / PlanDefinition | workflow/action graph + Library/CQL logic | deployment packaging target | export target |
| CPG-on-FHIR | guideline recommendations/pathways/strategies, strength/quality/direction | export compatibility | export target |
| WHO SMART DAK L2/L3 | L2 authoring tables, L3 FHIR logical models/CQL | staged pipeline precedent | source/process |
| GLIF/PROforma/Asbru/SAGE/EON | historical CIGs, task/process/temporal/intention models | temporal/workflow lessons | mine only |
| GEM | XML guideline document model | extraction/audit completeness | mine schema |
| BRIDGE-Wiz | recommendation wizard: circumstances, actor, obligation, action, recipient, rationale | closest clinical CNL slot template | adapt |
| COGS/GLIA | guideline quality and implementability checks | lint rules | adopt validators |

No clinical system provides deterministic CNL parse <-> verbalize <-> executable formal target. CKC should stay smaller than CQL/FHIR internally and export outward when useful.

URLs:
https://cql.hl7.org/,
https://cql.hl7.org/06-translationsemantics.html,
https://build.fhir.org/clinicalreasoning-module.html,
https://build.fhir.org/plandefinition.html,
https://build.fhir.org/ig/HL7/cqf-recommendations/en/methodology.html,
https://smart.who.int/ig-starter-kit/l2_dak_authoring.html,
https://gem.med.yale.edu/BRIDGE-Wiz/BRIDGE-Wiz.pdf,
https://pubmed.ncbi.nlm.nih.gov/21846779/.

## Multilingual / Japanese

Source: `../survey-multilingual-ja.md`.

| item | fact | verdict |
|---|---|---|
| GF | typed abstract syntax + concrete syntaxes; linearization deterministic if variants avoided; RGL active 2026; runtime dual LGPL/BSD | adapt later |
| GF Japanese RGL | exists, maintained; enough for small-fragment generation; broad parsing risk | mine/adopt with GF |
| ACE-in-GF | ACE subset in GF; many languages, no Japanese, no semantics, no license | mine only |
| Technical Japanese / sangyo nihongo | active patent CNL/style guidance; no formal grammar | mine JA style rules |
| Miyata controlled Japanese | MT-oriented controlled authoring; no semantics | mine sentence patterns |
| Easy Japanese | style/vocab guidance; no grammar/semantics | reject for audit IR |
| PROLEG | Japanese law as Prolog with exceptions; non-public/unlicensed core | mine exception structure |
| Minds/MEDIS/JAMI | Minds = JP guideline source; MEDIS = terminology masters; no rule language | adopt as data/coding sources |

Cheapest credible Japanese path:

1. Hand-built template verbalizer first: AST -> JA strings with explicit particles, overt subjects, no ellipsis, interval markers `ijo/ika/miman/cho` in project romanization until final Japanese rendering.
2. Parse only CKC's own generated JA templates if needed; avoid broad Japanese guideline parsing as a deterministic CNL goal.
3. GF becomes attractive when JA parsing of CKC surface or >2 languages becomes a real requirement.

URLs:
https://www.grammaticalframework.org/,
https://github.com/GrammaticalFramework/gf-core,
https://github.com/GrammaticalFramework/gf-rgl,
https://github.com/Attempto/ACE-in-GF,
https://tech-jpn.jp/,
https://minds.jcqhc.or.jp/,
https://www.medis.or.jp/,
https://github.com/DaisukeBekki/lightblue,
https://github.com/jurisinformaticscenter/NL2Proleg.

## Broader CNL Landscape

| system | target | verdict |
|---|---|---|
| SBVR / RuleSpeak | typed FOL-ish business semantics with deontic/alethic modalities | mine deontic vocabulary, not parser |
| CLCE | Common Logic/FOL | mine variables/quantifiers, too broad/stale |
| CPL/CPL-Lite | Prolog-like KR via deterministic lite core | mine architecture |
| Rabbit / Sydney OWL / CLOnE / OWL SE | OWL verbalization/editing | terminology/ontology lessons only |
| ITA Controlled English | concepts/instances/rules/rationale | inspect open artifact, not core |
| Gellish | UID dictionary + semantic network | mine dictionary discipline |
| Naproche/ForTheL | math CNL -> proof tasks | proof-checking UX only |
| Gherkin | executable scenario AST | scenario/table harness idioms only |
| AMR | lossy semantic graph | extraction feature only |

Landscape conclusion: Kuhn's CNL survey/PENS remains the right vocabulary. CKC target should be `P5 E3 N4 S4 D`: fixed semantics, FOL/rule expressiveness with negation, readable natural-ish audit surface, <=10-page exact core spec, domain-specific.

URLs:
https://www.omg.org/spec/SBVR/1.5/About-SBVR,
https://www.jfsowa.com/clce/specs.htm,
https://ceur-ws.org/Vol-448/paper30.pdf,
https://github.com/ce-store/ce-store,
https://github.com/naproche/naproche,
https://cucumber.io/docs/gherkin/reference/,
https://aclanthology.org/J14-1005/.

## LLM / Constrained-Decoding Evidence

| evidence | result | CKC implication |
|---|---|---|
| Geng et al. 2023 grammar-constrained decoding | LLaMA-33B cIE F1 17.5 -> 36.0; ED avg 54.1 -> 80.3; constituency validity 64.2% -> 100% | constraints help syntax and often semantics |
| DOMINO 2024 | naive JSON constraints hurt GSM8K; DOMINO recovers accuracy and improves throughput | tokenizer-aware constraints matter |
| Tam et al. 2024 vs later work | strict JSON/XML can hurt reasoning; later causal study found most cases unaffected | format must match task, avoid unnatural wrappers |
| Grammar-Aligned Decoding / ASAp | ordinary masking distorts distribution | constraints are not semantically neutral |
| Logic-LM / SATLM / LINC | LLM as parser + solver as reasoner improves reliability | CKC architecture is right |
| CNL KGQA study | CNL targets beat SPARQL for several models | CNL-like targets can be easier than raw formal syntax |
| OpenAI structured-output docs | JSON mode guarantees valid JSON, not schema/semantic correctness; can whitespace-loop without JSON instruction | JSON is interchange, not weak-model surface |

Design rules:

- Model target: compact CKC DSL/CNL, not JSON/Prolog/SMT/Lean/unrestricted ACE.
- Grammar: tokenizer-tested, whitespace-flexible, whole-token keywords, input-dependent terminals for IDs/codes/units.
- Emit one recommendation/table row at a time.
- Validator stack beyond grammar: unit compatibility, interval closure, DNF normalization, exception precedence, temporal satisfiability, source-span support, duplicate/conflicting recommendations.

URLs:
https://aclanthology.org/2023.emnlp-main.674/,
https://proceedings.mlr.press/v235/beurer-kellner24a.html,
https://aclanthology.org/2024.emnlp-industry.91/,
https://proceedings.neurips.cc/paper_files/paper/2024/hash/2bdc2267c3d7d01523e2e17ac0a754f3-Abstract-Conference.html,
https://aclanthology.org/2023.findings-emnlp.248/,
https://arxiv.org/abs/2305.09656,
https://aclanthology.org/2023.emnlp-main.313/,
https://aclanthology.org/2025.acl-industry.34/,
https://developers.openai.com/api/docs/guides/structured-outputs.

