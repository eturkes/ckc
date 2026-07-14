% ClinicalCNL ulex + registry gate (M3.ulex). Two blocks: `registry` accepts the live registry
% and rejects one isolated defect per validator rule (each a sole, exactly-pinned violation);
% `ulex` pins clinical_ulex's ulextext byte-identical to the frozen surface_cases oracle, proves
% the entry set equals the registry surface projection (no drift), and runs a one-golden APE parse
% smoke driving the file's own bytes to a clean v1 parse. Source-relative + cwd-independent.
%
%   Gate: swipl -q -g "consult('clinical/ulex_tests.pl'),(run_tests([registry,ulex])->halt(0);halt(1))" -t 'halt(1)'

:- module(ulex_tests, []).

:- use_module(library(plunit)).

% Load the registry + lexicon + frozen seeds + APE env, all source-relative (prolog/ape.pl asserts
% the absolute `ape` file_search_path when get_ape_results.pl loads — mirrors surface_goldens.pl).
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/registry.pl'], R), use_module(R),
   atomic_list_concat([D, '/clinical_ulex.pl'], U), use_module(U),
   atomic_list_concat([D, '/goldens/surface_cases.pl'], SC), use_module(SC),
   atomic_list_concat([D, '/../get_ape_results.pl'], Ape), ensure_loaded(Ape).

% n_solutions(+Goal, -N) — number of solutions of Goal (cardinality tripwires).
n_solutions(Goal, N) :- findall(t, Goal, L), length(L, N).

% expected_ulex_entry(-Entry) — the ulex entries the registry surfaces project to. The ulex block
% asserts this SET equals clinical_ulex:ulex_entry/1, binding the APE lexical layer to the registry.
expected_ulex_entry(noun_sg(W, W, human)) :- reg_population(_, W).
expected_ulex_entry(noun_sg(W, W, neutr)) :- reg_concept(_, W).
expected_ulex_entry(noun_sg(V, V, neutr)) :- reg_quantity(_, V, _, _, _).
expected_ulex_entry(noun_sg(S, L, neutr)) :- reg_quantity(_, _, S, _, L).
expected_ulex_entry(noun_pl(P, L, neutr)) :- reg_quantity(_, _, _, P, L).
expected_ulex_entry(pn_sg(PN, PN, neutr)) :- reg_drug(_, PN).
expected_ulex_entry(tv_finsg(F, L))       :- reg_action(_, F, _, L).
expected_ulex_entry(tv_infpl(P, L))       :- reg_action(_, _, P, L).
expected_ulex_entry(tv_finsg(F, L))       :- reg_guard_verb(F, _, L).
expected_ulex_entry(tv_infpl(P, L))       :- reg_guard_verb(_, P, L).

% ==========================================================================================
:- begin_tests(registry).

% ---- accept: the live registry is well-formed --------------------------------------------------
test(accept) :-
    registry_facts(Facts), registry_errors(Facts, Errors),
    ( Errors == [] -> true
    ; format(user_error, "registry: live registry has errors ~w~n", [Errors]), fail ).
test(valid_registry_agrees) :- valid_registry.

% ---- reject: one isolated defect per rule, each the exact sole violation -----------------------
% coverage — a kb_kernel lexeme id with no surface.
test(reject_uncovered_concept) :-
    registry_facts(Base), selectchk(reg_concept('cond.sepsis', sepsis), Base, Rest),
    registry_errors(Rest, Errors),
    assertion(Errors == [uncovered(concept, 'cond.sepsis')]).
% unknown — a surface fact's id outside the vocabulary (additive so coverage stays complete).
test(reject_unknown_concept) :-
    registry_facts(Base),
    registry_errors([reg_concept('cond.bogus', bogusword)|Base], Errors),
    assertion(Errors == [unknown_id(concept, 'cond.bogus')]).
% duplicate id — one id surfaced twice (forward ambiguity).
test(reject_duplicate_id) :-
    registry_facts(Base),
    registry_errors([reg_concept('cond.sepsis', fever)|Base], Errors),
    assertion(Errors == [duplicate(id, 'cond.sepsis')]).
% duplicate surface — one word shared across the lexicon (reverse ambiguity; APE "defined twice").
test(reject_duplicate_surface) :-
    registry_facts(Base), selectchk(reg_population('pop.patient', patient), Base, Rest),
    registry_errors([reg_population('pop.patient', sepsis)|Rest], Errors),
    assertion(Errors == [duplicate(surface, sepsis)]).
% malformed — a non-atom surface (gross shape error), id otherwise valid + covered.
test(reject_malformed) :-
    registry_facts(Base), selectchk(reg_concept('cond.sepsis', sepsis), Base, Rest),
    registry_errors([reg_concept('cond.sepsis', 123)|Rest], Errors),
    assertion(Errors == [malformed(reg_concept('cond.sepsis', 123))]).
% keyword-table field vocabularies (for is redundantly covered by suggest, so mutating recommend
% leaves only the intended violation).
test(reject_unknown_op) :-
    registry_facts(Base), selectchk(reg_keyword(recommend, should, for, strong), Base, Rest),
    registry_errors([reg_keyword(recommend, bogusop, for, strong)|Rest], Errors),
    assertion(Errors == [unknown_id(op, bogusop)]).
test(reject_unknown_direction) :-
    registry_facts(Base), selectchk(reg_keyword(recommend, should, for, strong), Base, Rest),
    registry_errors([reg_keyword(recommend, should, bogusdir, strong)|Rest], Errors),
    assertion(Errors == [unknown_id(direction, bogusdir)]).
test(reject_unknown_strength) :-
    registry_facts(Base), selectchk(reg_keyword(recommend, should, for, strong), Base, Rest),
    registry_errors([reg_keyword(recommend, should, for, bogusstr)|Rest], Errors),
    assertion(Errors == [unknown_id(strength, bogusstr)]).
% direction / frame coverage — permit has one keyword; should has one frame.
test(reject_uncovered_direction) :-
    registry_facts(Base), selectchk(reg_keyword('may-consider', may, permit, weak), Base, Rest),
    registry_errors(Rest, Errors),
    assertion(Errors == [uncovered(direction, permit)]).
test(reject_uncovered_frame) :-
    registry_facts(Base), selectchk(reg_frame(should, 'it is recommended that'), Base, Rest),
    registry_errors(Rest, Errors),
    assertion(Errors == [uncovered(frame, should)]).
% duplicate keyword / frame.
test(reject_duplicate_keyword) :-
    registry_facts(Base),
    registry_errors([reg_keyword(recommend, should, for, strong)|Base], Errors),
    assertion(Errors == [duplicate(keyword, recommend)]).
test(reject_duplicate_frame) :-
    registry_facts(Base),
    registry_errors([reg_frame(should, 'it is recommended that')|Base], Errors),
    assertion(Errors == [duplicate(frame, should)]).

% ---- direct lookups (bidirectional + the pn allowlist + the D1 decode) -------------------------
test(pn_allow_abx)     :- pn_allow('Abx-A').
test(pn_allow_only)    :- findall(P, pn_allow(P), Ps), assertion(Ps == ['Abx-A']).
test(concept_inverse)  :- reg_concept(Id, 'severe-renal-impairment'), assertion(Id == 'cond.renal_severe').
test(keyword_decode)   :-
    reg_keyword(contraindicate, Op, Dir, Str),
    assertion(Op == '-can'), assertion(Dir == contraindicate), assertion(Str == strong).
test(frame_lookup)     :- reg_frame('-should', P), assertion(P == 'it is not recommended that').

% ---- cardinality tripwire (guards vacuous passes + accidental deletion) ------------------------
test(registry_cardinalities) :-
    n_solutions(reg_concept(_, _),           NC), assertion(NC == 3),
    n_solutions(reg_drug(_, _),              ND), assertion(ND == 1),
    n_solutions(reg_action(_, _, _, _),      NA), assertion(NA == 1),
    n_solutions(reg_quantity(_, _, _, _, _), NQ), assertion(NQ == 1),
    n_solutions(reg_population(_, _),        NP), assertion(NP == 1),
    n_solutions(reg_guard_verb(_, _, _),     NG), assertion(NG == 1),
    n_solutions(reg_keyword(_, _, _, _),     NK), assertion(NK == 6),
    n_solutions(reg_frame(_, _),             NF), assertion(NF == 4),
    n_solutions(reg_op(_),                   NO), assertion(NO == 4),
    n_solutions(reg_v1_direction(_),         NV), assertion(NV == 4).

:- end_tests(registry).

% ==========================================================================================
:- begin_tests(ulex).

% ulex_text is byte-identical to the frozen surface_cases oracle.
test(ulex_text_matches_frozen) :-
    clinical_ulex:ulex_text(Text), surface_cases:surface_ulex(Frozen),
    assertion(Text == Frozen).

% The clinical_ulex entry SET equals the registry surface projection (the layers cannot drift).
test(registry_ulex_consistent) :-
    setof(E, expected_ulex_entry(E), Expected),
    setof(E, clinical_ulex:ulex_entry(E), Actual),
    assertion(Expected == Actual).

% Entry-count tripwire.
test(ulex_entry_count) :-
    n_solutions(clinical_ulex:ulex_entry(_), N), assertion(N == 12).

% One-golden APE parse smoke: the file's own ulextext drives the banked frame_recommend golden to a
% clean v1 parse (text/plain, serialized DRS, zero messages) — the fail-closed seam under this ulex.
test(ape_parse_smoke) :-
    surface_cases:surface_case(frame_recommend, v1, ACE),
    clinical_ulex:ulex_text(Ulex),
    ape:get_ape_results([ text=ACE, noclex=on, ulextext=Ulex, guess=off, solo=drs ], CT, Content),
    error_logger:get_messages(Msgs),
    assertion(CT == 'text/plain'),
    assertion(Msgs == []),
    assertion(Content \== '').

:- end_tests(ulex).
