# Multilingual CNL family + Japanese story — research report (verified 2026-07-07)

Provenance: `cnl-ir-research` workflow harvest (Claude subagents, 2026-07-07). Companions: `cnl-attempto.md`, `cnl-landscape.md`. Consumer: SPEC §10 (ClinicalCNL — the JA-primary design rests on §5 verdict here).

## 1. System inventory

### Grammatical Framework (GF)
- **What**: typed functional grammar formalism. One **abstract syntax** (algebraic datatypes = language-neutral AST) + N **concrete syntaxes** (linearization rules AST→string per language). Compiles to PGF binary; runtime parses + linearizes.
- **Grammar/parser mechanism + determinism**: concrete syntax = PMCFG; parsing = polynomial PMCFG parsing, returns ALL parse trees (ambiguity possible, grammar-dependent). **Linearization = total function AST→string, deterministic** unless grammar uses `variants` (avoid them → strict determinism). Generation-only use is the safe deterministic direction.
- **Semantic targets**: none built-in; AST is the semantics. Ecosystem mappings exist (OWL↔GF in MOLTO, ACE subset).
- **Bidirectional**: yes by construction — same grammar parses and verbalizes. Parse determinism NOT guaranteed per concrete language (see ACE-in-GF evidence below).
- **Expressiveness vs CKC domain**: GF is a surface-syntax tool — deontic/defeasible/temporal semantics live in YOUR abstract syntax; GF handles rendering (incl. numerals; RGL has Numeral/Symbol modules). No native intervals/deontics — you define constructors `recommend : Population -> Action -> Direction -> Strength -> Rule` and GF renders them.
- **Implementation language**: compiler + main runtime Haskell; C runtime; bindings Python (`pgf` 1.1 on PyPI), Java.
- **License** (verified from repo LICENSE files): gf-core compiler + web service **GPL**; runtime **dual LGPL/BSD**; **RGL dual LGPL/BSD with explicit grant: application grammars derived from RGL via its API may take ANY license** → license-clean for CKC (GPL compiler = dev tool only, like GCC).
- **Maintenance** (verified via GitHub API): gf-core release **3.12 = 2025-08-03**, pushed 2026-05; gf-rgl pushed **2026-06-10**, not archived. Active.
- **Key papers**: Ranta, *Grammatical Framework: Programming with Multilingual Grammars* (CSLI 2011); Angelov & Ranta, "Implementing Controlled Languages in GF" (CNL 2009, LNCS 5972 — ACE-like grammar ported to 7 langs: Eng/Fre/Ger/Ita/Swe/Fin/Urd, cited in MOLTO D11.1).
- URLs: https://www.grammaticalframework.org/ · https://github.com/GrammaticalFramework/gf-core · https://github.com/GrammaticalFramework/gf-rgl · https://pypi.org/project/pgf/

### GF Resource Grammar Library — Japanese RGL
- **EXISTS.** Author: **Liza Zimina**; initial import commit 2012-04-18 by Aarne Ranta, message "Japanese RGL by Liza Zimina - almost complete!" (verified via commit history). Status page marks her `*` = still-active author.
- **Completeness** (official status page row `Jpn ++ + + + - - - - + +`): Paradigms complete incl. smart paradigms (++), lexicon nearly complete (+), syntax nearly complete (+), API compiles (+), tested-in-apps (+), publications (+); NO irregular-verb module, NO large morphological dictionary. Status page shows Symb "-" but repo now HAS `SymbolJpn.gf` (+ `NumeralJpn.gf`, full ~28-module set) → status page stale, repo ahead.
- **Recent activity**: `src/japanese` touched **2026-04-01** (Inari Listenmaa: "add GN, LN, SN + constructors"), 43 commits total on the dir; `gf-wordnet` has `WordNetJpn.gf` + `ParseJpn.gf` (large lexicon exists there despite status page "-").
- **Quality**: mid-tier RGL member; UNCERTAIN in depth — politeness/register params exist (`Style = Plain | Resp` in ResJpn.gf), but no published coverage eval found. Small-fragment generation (CKC's need) is the low-risk use; broad parsing would be the risky use.
- URLs: https://www.grammaticalframework.org/lib/doc/status.html · https://github.com/GrammaticalFramework/gf-rgl/tree/master/src/japanese · https://github.com/GrammaticalFramework/gf-wordnet

### ACE-in-GF (Attempto Controlled English ported multilingually)
- **What**: AceWiki subset of ACE 6.6/6.7 **syntax** (explicitly NOT the DRS semantics mapping) reimplemented as a GF abstract syntax + concrete syntaxes. MOLTO WP11 product.
- **Languages**: grammar files for ~25 concretes (Ace/Ape = English variants, Bul Cat Chi Dan Dut Eng Est Fin Fre Ger Gre Hin Ita Lav Mlt Nor Pol Ron Rus Spa Swe Tha Urd); Makefile "maintained" list = 17. **NO JAPANESE** (no RGL Jpn dependency ever added).
- **Mechanism**: shared GF **functor** with ~100 linearization rules + per-language instantiation (RGL resource + lexicon module) → per-language cost = lexicon + small overrides. D11.1: "adding support for new languages is straight-forward and requires little extra work" (qualitative; no person-day figures published in D11.1 or the ESWC paper — I checked both).
- **Determinism evidence** (Kaljurand & Kuhn, ESWC 2013, arXiv:1303.4293): goal = 1 abstract tree per sentence; achieved **ambiguity 3.3%** on the 19,422-sentence Codeco test set for ACE itself (residual = semantically harmless 2-tree cases); **other languages NOT ambiguity-free** — semantically severe cases: subject/object relative-clause ambiguity (Dutch/German), double-negation readings (Romance). Mitigations in AceWiki-GF: store the **set** of trees, cross-language disambiguation dialogs, proposed per-language "disambiguation syntax" (ugly-but-unambiguous alternate concrete syntax).
- **License**: **no license file, no license statement in README** (verified) → legally unusable as vendored code; fine to imitate (clean-room).
- **Maintenance**: last push **2021-09-05**; effectively dormant. Attempto ACE itself: attempto.ifi.uzh.ch (UZH); ACE 6.7; APE parser is SWI-Prolog, LGPL.
- **Key papers**: Kaljurand & Kuhn, "A Multilingual Semantic Wiki Based on ACE and GF" (ESWC 2013, DOI 10.1007/978-3-642-38288-8_29); Camilleri/Fuchs/Kaljurand, MOLTO D11.1 "ACE Grammar Library" (2012); Kuhn, "A Survey and Classification of Controlled Natural Languages" (Comput. Linguistics 40(1), 2014).
- URLs: https://github.com/Attempto/ACE-in-GF · https://arxiv.org/abs/1303.4293 · http://attempto.ifi.uzh.ch/ · D11.1 via Wayback: http://web.archive.org/web/2015/http://www.molto-project.eu/sites/default/files/d11_1.pdf

### MOLTO (Multilingual On-Line Translation, FP7 247914)
- 2010-03→2013-05, €2.975M EU, coordinator U. Gothenburg (Ranta); partners incl. UZH (Attempto), Ontotext, UPC, U. Helsinki, Be Informed.
- Results: GF-based domain translation with ontology interlinguas; OWL↔GF two-way interop; case studies: **math exercises (15 langs), patent data (≥3 langs: en/fr/de), museum descriptions (15 langs)**; GF+SMT hybridization for robustness.
- **No Japanese** anywhere in MOLTO. Project domain molto-project.eu is DEAD (TLS cert now = cloud.grammaticalframework.org) → deliverables only via Wayback.
- URL: https://cordis.europa.eu/project/id/247914

### 産業日本語 / Technical (Industrial) Japanese — Japio/JPO patent CNL
- **What**: "Japanese written so technical info is easy for humans to understand AND machines to process" — MT-friendliness explicit. Run by 産業日本語研究会 (est. FY2009, Japio-backed; JPO-adjacent).
- **Rules**: 特許ライティングマニュアル (Patent Writing Manual): 1st ed 2013-06 = 8 categories/31 rewriting rules; **2nd ed 2018-03 = 7 categories/27 rules**; free download; companion learning text on patent-document quality. Rule style = human rewriting guidance (sentence splitting, explicit particles, resolve modifier attachment), positioned as "foundation for computer-based writing support"; **no public parser/checker tool named on the site** (older Japio checker prototypes existed — UNCERTAIN, not verifiable on current site).
- **No formal grammar, no semantics, no bidirectionality** — style-rule CNL (Kuhn-survey type: human-oriented reliability CNL).
- **Active**: 17th symposium Feb 2026, FY2025 annual report Apr 2026 (verified on site).
- URLs: https://tech-jpn.jp/ · https://tech-jpn.jp/tokkyo-writing-manual/

### Controlled Japanese for MT — academic line (verified core: Miyata et al.)
- **Rei Miyata (Nagoya U) + Anthony Hartley + Kyo Kageura**: controlled authoring of Japanese municipal documents for MT. **MuTUAL** authoring support system (COLING 2016 demo); usability eval (PBML 2017); consolidated in Miyata's book *Controlled Document Authoring in a Machine Translation Age* (2021; LRE review 2023). Controlled-Japanese rule sets (sentence patterns per document function, terminology control, pre-editing rules) empirically validated for ja→en MT quality. **One-directional (authoring), no formal semantics.**
- 1990s NTT/Fuji Xerox controlled-Japanese-for-MT efforts (Ogura et al.): UNCERTAIN — could not verify primary sources this session; treat as historical only.
- URL: https://dblp.org/search?q=miyata+controlled+authoring

### やさしい日本語 (Easy Japanese)
- Disaster-communication register born of 1995 Kobe quake (Sato Kazuyuki, Hirosaki U); anchored ~JLPT N4 vocab/grammar. Official 「在留支援のためのやさしい日本語ガイドライン」 Immigration Services Agency + Agency for Cultural Affairs, 2020-08 (+ spoken ed. 2022). NHK NEWS WEB EASY since 2012.
- **Style guidance + vocab lists, NOT machine-processable formal language** — multiple agency guidelines, no grammar, no parser, no semantics. NLP artifacts exist as **corpora**: SNOW T15 "Japanese Simplified Corpus with Core Vocabulary" — 50k sentence pairs, manual rewrites (Maruyama & Yamamoto, LREC 2018, Nagaoka UT).
- Relevance to CKC: patient-facing lexical-choice policy at most; useless as a compile target.
- URLs: https://www3.nhk.or.jp/news/easy/ · https://www.jnlp.org/GengoHouse/snow/t15 · guideline: moj.go.jp (page exists, bot-403s; see https://ja.wikipedia.org/wiki/やさしい日本語)

### Japanese with formal semantics (nearest thing to "controlled Japanese with semantics")
- **lightblue** (Daisuke Bekki, Ochanomizu U): Japanese CCG parser producing Dependent Type Semantics representations. Haskell, **BSD-3**, pushed **2026-06-27** — ACTIVE. Broad-coverage stochastic parsing of REAL Japanese, not a CNL → no determinism guarantee.
- **ccg2lambda** (Mineshima/Martínez-Gómez et al., NII/Tokyo): syntax-semantics interface EN+JA → higher-order logic; Apache-2.0; last push 2023-12 (dormant).
- No published "controlled Japanese with formal semantics" CNL found (i.e., no Japanese ACE equivalent). UNCERTAIN as a universal negative, but consistent with Kuhn's 2014 survey (CNLs overwhelmingly English-based) and with everything above.
- URLs: https://github.com/DaisukeBekki/lightblue · https://github.com/mynlp/ccg2lambda

### PROLEG (Ken Satoh, NII / Center for Juris-Informatics)
- **What**: Japanese Civil Code (later criminal law, GDPR, private international law) as Prolog implementing 要件事実論 (**Presupposed Ultimate Fact theory**): each legal conclusion = general rule + separately-asserted **`exception(Rule, Exception)`** facts; defeasibility mirrors burden-of-proof allocation between parties. Rule base ~2500 rules + ~800 exceptions per Satoh's papers (UNCERTAIN — figures vary by version).
- **Surface/verbalization**: rules ARE Prolog (predicates = romanized/translated legal terms); GUI renders reasoning as block diagrams. **NL→PROLEG** is active research: "Interactive Natural Language Interface for PROLEG" (JURISIN 2022), NL2Proleg repo (pushed 2025-10), "Can Legislation Be Made Machine-Readable in PROLEG?" (2025/26), LLM translation of contracts→predicates (2026). **No published deterministic PROLEG→Japanese verbalizer** (UNCERTAIN — nothing public; direction is LLM-based and parse-side).
- **Availability/license**: NO public core-PROLEG repo; sites.google.com/view/proleg is login-walled; jurisinformaticscenter GitHub org (active 2024-2026) hosts GDPR-PROLEG examples/data **without licenses** → dev-oracle use requires contacting NII; imitating the rule STRUCTURE is unencumbered.
- **Key papers**: Satoh et al., "PROLEG: An Implementation of the Presupposed Ultimate Fact Theory of Japanese Civil Code by PROLOG Technology" (JURISIN 2010/LNCS 2011); "PROLEG: Practical Legal Reasoning System" (in *Prolog: The Next 50 Years*, LNCS 13900, 2023).
- URLs: https://research.nii.ac.jp/~ksatoh/ · https://jurisinformaticscenter.github.io/ · https://github.com/jurisinformaticscenter/NL2Proleg

### Japanese clinical decision support / rule representation
- **Minds** (医療情報サービス, JCQHC): national clinical-practice-guideline library, GRADE-based CQ→recommendation+strength+certainty format — **exactly CKC's source-side domain shape, in Japanese prose**; no computable form. https://minds.jcqhc.or.jp/
- **MEDIS-DC**: standard terminology masters (disease names/ICD-10, drugs, lab codes) — vocabulary, not rules. https://www.medis.or.jp/ — the natural code system for CKC concept atoms in Japanese deployments.
- **JAMI**: medical-informatics society; standards = messaging/storage (SS-MIX2), not rule logic. https://www.jami.jp/
- **No maintained public Japanese computable-guideline rule language found** (CiNii probe surfaced only guideline-methodology papers; early-2000s GLIF-Japanese electronic-guideline pilots existed — UNCERTAIN, apparently abandoned). Gap = CKC's opportunity.

## 2. Evidence of use for rules/regulations/clinical
- GF: MOLTO patents (en/fr/de patent claims), math, museum; AceWiki-GF multilingual OWL wikis; Kuhn survey documents ACE-family use for regulations/specs. Clinical: no GF clinical deployment found (UNCERTAIN as negative).
- ACE proper: AceWiki ontologies, ACE→OWL/SWRL/TPTP; used in EU protein/biomed ontology pilots (Attempto site). Multilingual variant (ACE-in-GF) = research prototype only, never production.
- 産業日本語: patent-office ecosystem, 17 annual symposiums, manual in wide distribution (6,600+ downloads of 2nd ed) — real but human-process-only.
- PROLEG: the flagship "Japanese statutes as executable defeasible logic program" line, 15+ years, active LLM-integration research 2024-2026, judge-education use claimed by Satoh (UNCERTAIN).
- Miyata et al.: deployed controlled-Japanese authoring for municipal disaster/administrative texts (regulation-adjacent).

## 3. Design lessons for CKC
**Copy:**
- **GF's architecture is CKC's architecture**: language-neutral typed AST + per-language deterministic linearizers. Even without adopting GF, structure the verbalizer as linearization RULES over the IR AST, one function per constructor, no free text.
- **ACE-in-GF functor trick**: put sentence-pattern skeletons in a shared functor (~100 rules sufficed for the whole AceWiki subset); per-language cost collapses to a lexicon + particle/word-order overrides. For CKC: EN and JA verbalizers share the rule-shape iterator; only surface templates differ.
- **Generation-only sidesteps the hard problem**: ACE-in-GF's measured failure = parse ambiguity in NON-English concretes (Dutch/German relative clauses, Romance double negation); linearization stayed deterministic. CKC's model-writes/human-reads flow needs exactly the easy direction. Japanese-side determinism risks are attachment ambiguity of relative clauses (連体修飾) and quantifier/negation scope — invisible in generation if templates keep one proposition per sentence.
- **PROLEG's exception structure**: represent exceptions as separate, labeled, rule-referencing assertions (`exception(rule_id, condition)`) rather than compiling ¬-atoms into the condition DNF — preserves auditability (clinician sees "rule R unless E" as written in the guideline), maps cleanly to defeasible LP (negation-as-failure on exception predicates) AND to SMT (R ∧ ¬E expansion at compile time). Deontic direction + burden-of-proof analogy: PROLEG proves "who must establish what" — mirrors for/against/contraindicate audit.
- **Codeco-style regression testing** (Kuhn): exhaustive enumeration of all sentences up to token depth N against the grammar; count trees per sentence; assert =1. Directly automatable for CKC's grammar-constrained IR: enumerate ASTs to depth k, round-trip verbalize→parse, assert identity. ACE-in-GF's 19,422-sentence set + 3.3% residual shows even mature CNLs need this loop.
- **Ambiguity budget honesty**: if residual ambiguity exists, store the SET of parses and refuse silent choice (AceWiki-GF) — for CKC: verbalizer output must re-parse to exactly one AST or the build fails.
- **産業日本語/Miyata sentence patterns for the JA templates**: one proposition per sentence; explicit particles everywhere; no anaphora/ellipsis (zero pronouns are default in natural Japanese — templates must force overt subjects); fixed functional patterns e.g. 「【対象集団】が【条件】の場合、【行為】を行うことを【強く/弱く】推奨する。ただし【例外】の場合を除く。」; numeric intervals as 「18歳以上」「7日以上14日以内」 (postpositional bound markers 以上/以下/未満/超 are exact interval-endpoint vocabulary — Japanese is BETTER than English here: closed/open bounds are lexically explicit).
**Avoid:**
- Vendoring ACE-in-GF (no license) or depending on Attempto APE (SWI-Prolog LGPL, dormant ecosystem).
- Trusting the GF RGL status page (stale vs repo) or assuming Jpn RGL large-lexicon coverage — clinical vocabulary must be a CKC-owned lexicon table keyed to MEDIS/ICD codes regardless of surface tech.
- Full-Japanese CNL parsing as an early goal: no prior art exists (no Japanese ACE); morphology-free segmentation, particle scope, and zero anaphora make deterministic Japanese PARSING research-grade — while deterministic Japanese GENERATION is trivial.
- MOLTO-style OWL interlingua sprawl: keep IR small and domain-shaped; MOLTO's generality cost a 3-year EU project.

## 4. Fit verdicts
| System | Verdict | Reason vs CKC facts |
|---|---|---|
| GF (framework + PGF runtime) | **adapt now / adopt later** | Architecture = exactly CKC's IR+verbalizer split; license-clean (runtime BSD, RGL grammar-output free); active 2025-2026. But full GF adds a Haskell toolchain + grammar-engineering skill for what (at 1 IR × 2 langs, generation-only, finite DNF shapes) a few hundred lines of template code do deterministically. Adopt if/when Japanese PARSING or >2 languages needed. |
| Japanese RGL | mine-ideas (adopt only with GF) | Exists, maintained (2026 commits), covers CKC's needed syntax; but only pays off inside a GF adoption. |
| ACE-in-GF | **mine-ideas** | Best empirical record of multilingual-CNL determinism limits + functor pattern; unlicensed, dormant 2021, no Japanese → nothing to run, everything to learn. |
| MOLTO | mine-ideas (historical) | Proves GF-CNL scales to patents/15 langs; dead project, dead domain. |
| 産業日本語 manual | **adapt (imitate rules)** | 27 free rewriting rules = ready-made spec for CKC's JA template style (MT/machine-processability was its design goal); no tooling to adopt. |
| Miyata controlled Japanese | mine-ideas | Validated JA sentence-pattern methodology for controlled authoring; system (MuTUAL) not distributed. |
| やさしい日本語 | reject (except lexical policy) | Style guidance, no grammar/semantics/tooling; irrelevant to clinician-audit surface. |
| lightblue / ccg2lambda | mine-ideas (future JA parsing) | Only live Japanese→formal-semantics line (BSD-3, active 2026); stochastic broad-coverage ≠ CNL determinism → candidate oracle for future JA parse direction, not a component. |
| PROLEG | **mine-ideas (imitate structure), reject as dependency** | Rule+exception defeasible pattern = the proven Japanese-normative-domain encoding, direct fit for CKC's exceptions + Prolog target; but code non-public/unlicensed → clean-room reimplementation of the PATTERN only. |
| Minds/MEDIS/JAMI | adopt as data/coding sources | Source guidelines (Minds, GRADE fields match IR) + concept codes (MEDIS); no rule language exists to adopt. |

## 5. Key question — cheapest credible path to Japanese rendering (+ eventual parsing)
**Verdict: hand-built template verbalizer first; GF concrete syntax as the planned upgrade; parsing much later via re-parse of your own CNL, never open Japanese.**
Evidence chain:
1. CKC ASTs are finite/flat (DNF atoms, 6 directions, 2 strengths, interval + duration literals) → JA linearization = string templates + a counter/unit table (歳, 日, 回, mg/dL) + 以上/以下/未満/超 for bounds. Japanese needs NO agreement, NO article/number morphology → template verbalizer is deterministically correct by construction; ~days of work; zero deps; license-clean; renders in both です/ます and である register via one flag (RGL's own `Plain|Resp` param confirms this is the only register axis).
2. GF path costs more but is real: Jpn RGL exists, authored 2012 (Zimina), touched 2026 (Listenmaa), LGPL/BSD with free application-grammar licensing; PGF Python runtime on PyPI; ACE-in-GF proves the shared-functor pattern keeps per-language cost at "lexicon + overrides" ("little extra work", D11.1). Choose it when: (a) JA parsing of the audit surface becomes a requirement (GF gives parse+generate from ONE grammar → round-trip theorem nearly free), or (b) language count grows (zh/ko/de guidelines).
3. Precedent confirms the ordering: nobody has built a deterministic Japanese-parsing CNL (no Japanese ACE exists; ACE-in-GF stopped short of Japanese; PROLEG — the flagship Japanese normative formalization — skips a CNL entirely and now throws LLMs at the NL→logic direction). Meanwhile all Japanese CNL successes (産業日本語, Miyata, やさしい日本語) are generation/authoring-side. Cheapest credible JA parsing for CKC = parse only your OWN generated CNL (templates make it a regular language → trivially deterministic), exactly the AceWiki "disambiguation syntax" idea inverted.
4. Round-trip guarantee (CNL→IR→CNL): with templates, prove verbalize∘parse = id by depth-bounded AST enumeration (Codeco method); with GF, the same PGF gives both directions but you must still test ambiguity per concrete (ACE-in-GF: 3.3% residual even for English after tuning — budget for a fix-loop).

Sources: [GF](https://www.grammaticalframework.org/) · [RGL status](https://www.grammaticalframework.org/lib/doc/status.html) · [gf-rgl](https://github.com/GrammaticalFramework/gf-rgl) · [gf-core](https://github.com/GrammaticalFramework/gf-core) · [gf-wordnet](https://github.com/GrammaticalFramework/gf-wordnet) · [pgf/PyPI](https://pypi.org/project/pgf/) · [ACE-in-GF](https://github.com/Attempto/ACE-in-GF) · [Kaljurand & Kuhn 2013](https://arxiv.org/abs/1303.4293) · [MOLTO/CORDIS](https://cordis.europa.eu/project/id/247914) · [MOLTO D11.1 (Wayback)](http://web.archive.org/web/2015/http://www.molto-project.eu/sites/default/files/d11_1.pdf) · [Attempto](http://attempto.ifi.uzh.ch/) · [産業日本語](https://tech-jpn.jp/) · [特許ライティングマニュアル](https://tech-jpn.jp/tokkyo-writing-manual/) · [やさしい日本語 (ja.wp)](https://ja.wikipedia.org/wiki/やさしい日本語) · [NHK Easy](https://www3.nhk.or.jp/news/easy/) · [SNOW T15](https://www.jnlp.org/GengoHouse/snow/t15) · [Miyata/dblp](https://dblp.org/search?q=miyata+controlled+authoring) · [Satoh/NII](https://research.nii.ac.jp/~ksatoh/) · [Juris-Informatics Center](https://jurisinformaticscenter.github.io/) · [NL2Proleg](https://github.com/jurisinformaticscenter/NL2Proleg) · [lightblue](https://github.com/DaisukeBekki/lightblue) · [ccg2lambda](https://github.com/mynlp/ccg2lambda) · [Minds](https://minds.jcqhc.or.jp/) · [MEDIS](https://www.medis.or.jp/) · [JAMI](https://www.jami.jp/)