% kb_kernel validator gate (M3.kb-contract). Accept/reject over the hand-written normative
% examples (goldens/kb_examples.pl) + direct tests of the id grammar, action-key split/join, the
% §L·conflict direction groups, and the PROLEG negation-as-failure reference derivability
% (including open-vs-closed interval boundaries). Source-relative + cwd-independent.
%
%   Gate: swipl -q -g "consult('clinical/kb_kernel_tests.pl'),(run_tests(kb_kernel)->halt(0);halt(1))" -t 'halt(1)'

:- module(kb_kernel_tests, []).

:- use_module(library(plunit)).

:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K),
   atomic_list_concat([D, '/goldens/kb_examples.pl'], E), use_module(E).

:- begin_tests(kb_kernel).

% ---- accept: every valid example validates clean --------------------------------------------
test(accept, [forall(kb_example(Name, valid, Facts))]) :-
    kb_errors(Facts, Errors),
    ( Errors == []
    -> true
    ;  format(user_error, "kb_kernel: valid example ~w has errors ~w~n", [Name, Errors]), fail ).

test(valid_kb_agrees, [forall(kb_example(_, valid, Facts))]) :-
    valid_kb(Facts).

% ---- reject: every invalid example fails, with the expected violation among the errors --------
test(reject, [forall(kb_example(Name, invalid(Functor), Facts))]) :-
    kb_errors(Facts, Errors),
    ( Errors = [E], functor(E, Functor, _)
    -> true
    ;  format(user_error, "kb_kernel: invalid example ~w expected sole ~w, got ~w~n",
              [Name, Functor, Errors]), fail ).

test(invalid_kb_agrees, [forall(kb_example(_, invalid(_), Facts))]) :-
    \+ valid_kb(Facts).

% Catalog tripwire: pins the example counts so a deleted/renamed generator category cannot make an
% accept/reject forall pass vacuously, and no name is reused across the valid/invalid sets.
test(example_catalog) :-
    findall(N, kb_example(N, valid, _),      Vs), sort(Vs, VS), length(VS, NV),
    findall(N, kb_example(N, invalid(_), _), Is), sort(Is, IS), length(IS, NI),
    assertion(NV =:= 4),
    assertion(NI =:= 40),
    \+ ( member(X, VS), member(X, IS) ).

% ---- id grammar -----------------------------------------------------------------------------
test(valid_id_ok)          :- valid_id('test_source.m1_guideline_a.rule.0', rule).
test(valid_id_stmt)        :- valid_id('test_source.m1_guideline_a.stmt.3', stmt).
test(valid_id_infers_kind) :- valid_id('d.exc.2', Kind), assertion(Kind == exc).
test(valid_id_bad_kind)    :- \+ valid_id('d.frob.0', _).
test(valid_id_bad_counter) :- \+ valid_id('d.rule.x', _).
test(valid_id_neg_counter) :- \+ valid_id('d.rule.-1', _).
test(valid_id_no_doc)      :- \+ valid_id('rule.0', _).
test(valid_id_empty_doc)   :- \+ valid_id('.rule.0', _).
test(valid_id_noncanon)    :- \+ valid_id('d.rule.01', _).
test(valid_id_signed)      :- \+ valid_id('d.rule.+1', _).

% ---- action key -----------------------------------------------------------------------------
test(action_key_split) :-
    action_key('act.administer:drug.abx_a', Kind, Target),
    assertion(Kind == 'act.administer'), assertion(Target == 'drug.abx_a').
test(action_key_join) :-
    action_key(Key, 'act.administer', 'drug.abx_a'),
    assertion(Key == 'act.administer:drug.abx_a').
test(action_key_nocolon_fails)   :- \+ action_key('act.administer', _, _).
test(action_key_twocolons_fails) :- \+ action_key('a:b:c', _, _).

% ---- context atoms (valid_atom/1; exact-rational bound vs float) -----------------------------
test(valid_atom_concept)         :- valid_atom(concept('cond.sepsis')).
test(valid_atom_interval)        :- valid_atom(interval('q.age_years', 18, closed, lower)).
test(valid_atom_rational_bound)  :- valid_atom(interval('q.age_years', 1r2, open, upper)).
test(valid_atom_rejects_float)   :- \+ valid_atom(interval('q.age_years', 1.5, closed, lower)).
test(valid_atom_rejects_concept) :- \+ valid_atom(concept('cond.bogus')).

% ---- direction groups (§L·conflict; avoid joins both non-positive groups) --------------------
test(direction_groups) :-
    findall(Dir-Grp, direction_group(Dir, Grp), Pairs),
    sort(Pairs, Sorted),
    assertion(Sorted == [ against-against, avoid-against, avoid-contraindicating,
                          contraindicate-contraindicating, for-positive,
                          permit-positive, require-positive ]).

% ---- reference derivability (PROLEG NAF; fixture contexts) -----------------------------------
% docA stmt.0: sepsis ∧ age>=18, exception = renal-impairment.
test(derivable_fires) :-
    kb_example(doc_a, valid, Facts),
    derivable('test_source.m1_guideline_a.stmt.0', Facts,
              [concept('cond.sepsis'), quantity('q.age_years', 30)]).
test(derivable_blocked_by_exception) :-
    kb_example(doc_a, valid, Facts),
    \+ derivable('test_source.m1_guideline_a.stmt.0', Facts,
                 [concept('cond.sepsis'), quantity('q.age_years', 30), concept('cond.renal_severe')]).
test(derivable_blocked_by_age) :-
    kb_example(doc_a, valid, Facts),
    \+ derivable('test_source.m1_guideline_a.stmt.0', Facts,
                 [concept('cond.sepsis'), quantity('q.age_years', 10)]).
test(derivable_needs_condition) :-
    kb_example(doc_a, valid, Facts),
    \+ derivable('test_source.m1_guideline_a.stmt.0', Facts,
                 [quantity('q.age_years', 30)]).
% Boundary: docA age>=18 (closed lower) admits 18; control age<18 (open upper) excludes 18.
test(derivable_boundary_closed_lower) :-
    kb_example(doc_a, valid, Facts),
    derivable('test_source.m1_guideline_a.stmt.0', Facts,
              [concept('cond.sepsis'), quantity('q.age_years', 18)]).
test(derivable_boundary_open_upper) :-
    kb_example(control, valid, Facts),
    \+ derivable('test_source.m1_control.stmt.0', Facts,
                 [concept('cond.sepsis'), quantity('q.age_years', 18)]).
test(derivable_child) :-
    kb_example(control, valid, Facts),
    derivable('test_source.m1_control.stmt.0', Facts,
              [concept('cond.sepsis'), quantity('q.age_years', 10)]).
% A well-formed but UNDECLARED statement id is not derivable (guards the vacuous-forall trap).
test(derivable_undeclared_fails) :-
    kb_example(doc_a, valid, Facts),
    \+ derivable('test_source.m1_guideline_a.stmt.9', Facts,
                 [concept('cond.sepsis'), quantity('q.age_years', 30)]).

:- end_tests(kb_kernel).
