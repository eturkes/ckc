% ClinicalCNL surface registry — the id<->surface authority (M3.ulex; SPEC §10.6, SURFACE.md).
%
% The single semantic bridge between the closed kb_kernel vocabulary (a compiled KB's ids) and the
% ClinicalCNL EN v1 surface. Each clinical id — condition concept, drug target, action kind, age
% quantity, population subject — gets exactly one surface; plus the D1 modality keyword table
% (keyword -> op + direction + strength), its ACE frame phrases, and the proper-name allowlist.
% The raw gate consumes it as a WHITELIST (only registered surfaces parse; pn_allow/1 is the
% named() discriminator that closes the p6 hole); map-core inverts DRS lemmas back to ids through
% it. clinical_ulex.pl projects these surfaces to APE ulex entries (cross-checked in ulex_tests).
%
% Bidirectional by construction: each reg_* fact is a plain relation (reg_concept(Id, Word) yields
% the word from the id and the id from the word). The integrity checker (registry_errors/2, pure
% over a fact list) proves the registry covers the whole kb_kernel vocabulary, references no unknown
% id, is duplicate-free, and is well-formed; valid_registry/0 runs it over the live facts.
%
%   Gate: swipl -q -g "consult('clinical/ulex_tests.pl'),(run_tests([registry,ulex])->halt(0);halt(1))" -t 'halt(1)'

:- module(registry,
          [ reg_concept/2,        % ?ConceptId, ?Noun
            reg_drug/2,           % ?TargetId, ?ProperName
            reg_action/4,         % ?KindId, ?FinSg, ?InfPl, ?Lemma
            reg_quantity/5,       % ?QuantityId, ?VarNoun, ?UnitSg, ?UnitPl, ?UnitLemma
            reg_population/2,      % ?PopId, ?Noun
            reg_guard_verb/3,     % ?FinSg, ?InfPl, ?Lemma
            reg_keyword/4,        % ?Keyword, ?Op, ?Direction, ?Strength
            reg_frame/2,          % ?Op, ?Phrase
            reg_op/1,             % ?Op          — the closed consequent-op-token vocabulary
            reg_v1_direction/1,   % ?Direction   — the four directions v1 keywords emit
            pn_allow/1,           % ?ProperName  — the proper-name allowlist (drug names)
            registry_facts/1,     % -Facts       — the live registry as a flat reg_* fact list
            registry_errors/2,    % +Facts, -Errors — sorted violation terms ([] = well-formed)
            valid_registry/0      % the live registry is well-formed
          ]).

% kb_kernel OWNS the closed semantic vocabulary; the checker covers it with surfaces. Loaded
% source-relative so the module is cwd-independent (mirrors kb_kernel_tests.pl).
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K).

% ==========================================================================================
% Surfaces (§L·ids). Each clinical id -> its EN v1 surface. Word == DRS lemma unless noted.
% ==========================================================================================

%% reg_concept(?ConceptId, ?Noun) — a condition concept's countable-noun surface (the DRS object
% lemma). Surfaced as `<patient> has a <Noun>` in a guard; the noun is its own lemma.
reg_concept('cond.sepsis',       sepsis).
reg_concept('cond.renal_severe', 'severe-renal-impairment').
reg_concept('cond.pregnancy',    pregnancy).

%% reg_drug(?TargetId, ?ProperName) — a drug target's proper-name surface (the DRS named() value).
% The set of ProperNames IS the pn allowlist (pn_allow/1): the only capitalised tokens the raw gate
% admits, so any other capital is the p6 named() hole and rejects.
reg_drug('drug.abx_a', 'Abx-A').

%% reg_action(?KindId, ?FinSg, ?InfPl, ?Lemma) — an action kind's transitive-verb surface: the
% finite-singular form (`takes`), the infinitive/plural form (`take`), and the DRS predicate lemma.
reg_action('act.administer', takes, take, take).

%% reg_quantity(?QuantityId, ?VarNoun, ?UnitSg, ?UnitPl, ?UnitLemma) — the age-interval nouns.
% Surfaced as `<patient> has an <VarNoun> of <marker> <INT> <UnitPl>`; the DRS carries object(VarNoun)
% + object(UnitLemma) + relation(of). q.age_years is a compound shape map-core walks structurally.
reg_quantity('q.age_years', age, year, years, year).

%% reg_population(?PopId, ?Noun) — the population subject's human noun. Introduced as `a patient`,
% later `the patient` (within-sentence anaphora); adult/child are age intervals, not populations.
reg_population('pop.patient', patient).

%% reg_guard_verb(?FinSg, ?InfPl, ?Lemma) — the structural guard predicate `has`/`have` (DRS lemma
% `have`) that attaches a condition to the subject. Not a kb id; a fixed lexeme of the guard grammar.
reg_guard_verb(has, have, have).

%% pn_allow(?ProperName) — the proper-name allowlist (the drug names). The raw gate's capital-token
% whitelist; membership is the authoritative named() discriminator (SURFACE.md §Named-word hole).
pn_allow(PN) :- reg_drug(_, PN).

% ==========================================================================================
% Modality (D1). The keyword table + ACE frame phrases (SURFACE.md §Modality is authoritative).
% ==========================================================================================

%% reg_keyword(?Keyword, ?Op, ?Direction, ?Strength) — the raw-header modality keyword table:
% keyword -> (consequent DRS op token, deontic direction, strength). Strength lives ONLY here (APE
% cannot express it); the ACE frame carries the op + direction. Total over the 6 v1 keywords.
reg_keyword(recommend,       should,    for,            strong).
reg_keyword(suggest,         should,    for,            weak).
reg_keyword('may-consider',  may,       permit,         weak).
reg_keyword('not-recommend', '-should', against,        strong).
reg_keyword('not-suggest',   '-should', against,        weak).
reg_keyword(contraindicate,  '-can',    contraindicate, strong).

%% reg_frame(?Op, ?Phrase) — a consequent op token -> its ACE modality frame phrase. Applied as
% `<Phrase> <action>`; the raw gate whitelists the phrase and op-mismatch-checks it against the
% keyword's op. recommend + suggest share `should`/the recommend phrase (strength aside).
reg_frame(should,    'it is recommended that').
reg_frame(may,       'it is admissible that').
reg_frame('-should', 'it is not recommended that').
reg_frame('-can',    'it is not possible that').

%% reg_op(?Op) — the closed consequent-op-token vocabulary (the reg_frame / reg_keyword op domain).
reg_op(should).
reg_op(may).
reg_op('-should').
reg_op('-can').

%% reg_v1_direction(?Direction) — the four directions v1 keywords emit. require/avoid are kb_direction
% members (they appear in the conflict direction groups) but no v1 keyword surfaces them.
reg_v1_direction(for).
reg_v1_direction(permit).
reg_v1_direction(against).
reg_v1_direction(contraindicate).

% ==========================================================================================
% Integrity checker. Pure over a registry-fact list; valid_registry/0 folds the live facts through
% it. Four violation classes (each a distinct functor so a reject test localizes to one rule):
%
%   uncovered(Kind, Id)     — a kb_kernel vocabulary id (or v1 direction / op frame) has no surface.
%   unknown_id(Kind, Id)    — a surface fact's id / keyword field is outside the closed vocabulary.
%   duplicate(Kind, Key)    — an id surfaced twice, a surface word shared, or a keyword/frame repeated.
%   malformed(Term)         — a surface / keyword fact whose shape or args are ill-formed.
% ==========================================================================================

%% registry_facts(-Facts) — the live registry as a flat list of reg_* terms (order irrelevant to
% the checker). Tests mutate this base to trip one rule at a time.
registry_facts(Facts) :- findall(F, live_reg_fact(F), Facts).

live_reg_fact(reg_concept(I, W))          :- reg_concept(I, W).
live_reg_fact(reg_drug(I, W))             :- reg_drug(I, W).
live_reg_fact(reg_action(I, F, P, L))     :- reg_action(I, F, P, L).
live_reg_fact(reg_quantity(I, V, S, P, L)):- reg_quantity(I, V, S, P, L).
live_reg_fact(reg_population(I, W))        :- reg_population(I, W).
live_reg_fact(reg_guard_verb(F, P, L))     :- reg_guard_verb(F, P, L).
live_reg_fact(reg_keyword(K, O, D, S))     :- reg_keyword(K, O, D, S).
live_reg_fact(reg_frame(O, P))             :- reg_frame(O, P).

%% valid_registry — the live registry is well-formed (no violations).
valid_registry :- registry_facts(Facts), registry_errors(Facts, []).

%% registry_errors(+Facts, -Errors) — the sorted, de-duplicated violation terms; [] iff well-formed.
registry_errors(Facts, Errors) :-
    findall(E, registry_violation(Facts, E), Es),
    sort(Es, Errors).

% registry_violation(+Facts, -Violation) — one violation per solution, grouped by class (malformed
% / unknown_id / uncovered / duplicate). The helpers follow, after all the clauses.
%
% malformed — a reg fact whose args are not all atoms (a gross shape error). Atom-guarded elsewhere
% so a non-atom id surfaces here only, never doubly as unknown.
registry_violation(Facts, malformed(F)) :-
    member(F, Facts),
    \+ well_formed_reg(F).
% unknown_id / uncovered over the lexeme families (data-driven via lexeme_family/4).
registry_violation(Facts, unknown_id(Kind, Id)) :-
    lexeme_family(Kind, Id, Skel, Vocab),
    member(Skel, Facts),
    atom(Id),
    \+ call(Vocab).

registry_violation(Facts, uncovered(Kind, Id)) :-
    lexeme_family(Kind, Id, Skel, Vocab),
    call(Vocab),
    \+ member(Skel, Facts).

% unknown_id / uncovered over the keyword table + frame phrases.
registry_violation(Facts, unknown_id(op, O)) :-
    member(reg_keyword(_, O, _, _), Facts), atom(O), \+ reg_op(O).
registry_violation(Facts, unknown_id(direction, Dir)) :-
    member(reg_keyword(_, _, Dir, _), Facts), atom(Dir), \+ reg_v1_direction(Dir).
registry_violation(Facts, unknown_id(strength, S)) :-
    member(reg_keyword(_, _, _, S), Facts), atom(S), \+ kb_strength(S).
registry_violation(Facts, unknown_id(frame_op, O)) :-
    member(reg_frame(O, _), Facts), atom(O), \+ reg_op(O).

registry_violation(Facts, uncovered(direction, Dir)) :-
    reg_v1_direction(Dir), \+ member(reg_keyword(_, _, Dir, _), Facts).
registry_violation(Facts, uncovered(frame, O)) :-
    reg_op(O), \+ member(reg_frame(O, _), Facts).

% --- duplicate: an id surfaced by two lexeme facts, a surface word shared across the lexicon (the
% mapper could not invert it, and APE warns "defined twice"), or a repeated keyword / frame op. ---
registry_violation(Facts, duplicate(id, Id)) :-
    findall(I, reg_id(Facts, I), Ids), duplicated(Ids, Id).
registry_violation(Facts, duplicate(surface, W)) :-
    findall(X, reg_surface_word(Facts, X), Words), duplicated(Words, W).
registry_violation(Facts, duplicate(keyword, K)) :-
    findall(X, member(reg_keyword(X, _, _, _), Facts), Ks), duplicated(Ks, K).
registry_violation(Facts, duplicate(frame, O)) :-
    findall(X, member(reg_frame(X, _), Facts), Os), duplicated(Os, O).

% --- helpers ------------------------------------------------------------------------------------

well_formed_reg(reg_concept(I, W))           :- atom(I), atom(W).
well_formed_reg(reg_drug(I, W))              :- atom(I), atom(W).
well_formed_reg(reg_action(I, F, P, L))      :- atom(I), atom(F), atom(P), atom(L).
well_formed_reg(reg_quantity(I, V, S, P, L)) :- atom(I), atom(V), atom(S), atom(P), atom(L).
well_formed_reg(reg_population(I, W))         :- atom(I), atom(W).
well_formed_reg(reg_guard_verb(F, P, L))      :- atom(F), atom(P), atom(L).
well_formed_reg(reg_keyword(K, O, D, S))      :- atom(K), atom(O), atom(D), atom(S).
well_formed_reg(reg_frame(O, P))              :- atom(O), atom(P).

% lexeme_family(?Kind, ?Id, ?Skeleton, ?VocabGoal) — drives coverage + unknown-id uniformly over
% the surface-bearing kb_kernel families (Skeleton shares Id; VocabGoal is the kb membership check).
lexeme_family(concept,       Id, reg_concept(Id, _),           kb_concept(Id)).
lexeme_family(action_kind,   Id, reg_action(Id, _, _, _),      kb_action_kind(Id)).
lexeme_family(action_target, Id, reg_drug(Id, _),              kb_action_target(Id)).
lexeme_family(quantity,      Id, reg_quantity(Id, _, _, _, _), kb_quantity(Id)).
lexeme_family(population,    Id, reg_population(Id, _),         kb_population(Id)).

reg_id(Facts, I) :- member(reg_concept(I, _), Facts).
reg_id(Facts, I) :- member(reg_drug(I, _), Facts).
reg_id(Facts, I) :- member(reg_action(I, _, _, _), Facts).
reg_id(Facts, I) :- member(reg_quantity(I, _, _, _, _), Facts).
reg_id(Facts, I) :- member(reg_population(I, _), Facts).

reg_surface_word(Facts, W) :- member(reg_concept(_, W), Facts).
reg_surface_word(Facts, W) :- member(reg_drug(_, W), Facts).
reg_surface_word(Facts, W) :- member(reg_action(_, W, _, _), Facts).
reg_surface_word(Facts, W) :- member(reg_action(_, _, W, _), Facts).
reg_surface_word(Facts, W) :- member(reg_quantity(_, W, _, _, _), Facts).
reg_surface_word(Facts, W) :- member(reg_quantity(_, _, W, _, _), Facts).
reg_surface_word(Facts, W) :- member(reg_quantity(_, _, _, W, _), Facts).
reg_surface_word(Facts, W) :- member(reg_population(_, W), Facts).
reg_surface_word(Facts, W) :- member(reg_guard_verb(W, _, _), Facts).
reg_surface_word(Facts, W) :- member(reg_guard_verb(_, W, _), Facts).

%% duplicated(+List, -X) — X occurs at least twice in List (each occurrence backtracks; sort/2 in
% registry_errors/2 collapses them to one violation).
duplicated(List, X) :- select(X, List, Rest), memberchk(X, Rest).
