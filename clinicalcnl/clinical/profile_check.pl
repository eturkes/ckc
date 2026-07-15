% ClinicalCNL post-APE DRS profile checker — the second fail-closed layer (M3.profile-drs +
% M3.profile-structure; SPEC §10.6, SURFACE.md). The raw gate (raw_gate.pl) whitelists surfaces
% BEFORE APE; APE then erases surface facts (prefix tokens, numeral spellings, comments, silent
% anaphora merges), so an independent layer re-validates the DRS AFTER APE. profile_check/4 is that
% layer: a pure STRUCTURAL whitelist over the APE DRS term — it reads a term, never runs a parser —
% fail-closed on anything outside the v1 profile.
%
% What it enforces (SURFACE.md + the D-decisions, roadmap §M3):
%   - the zero-message law: an accepted v1 parse is warning-free. APE's solo seam drops warnings,
%     so the anaphor and undefined-word `named` holes surface only here and at the raw gate.
%   - canonical referent hygiene via APE's own is_wellformed/1 (no duplicate / undeclared
%     referents) — the first-parse-wins canonical backstop. It runs FIRST, so the structural walkers
%     below may assume distinct, declared referents and match companion atoms by referent identity (==).
%   - a recursive named(_) scan against the pn allowlist (the p6 `named` hole discriminator).
%   - the rule shape drs([],[=>(guard, drs([],[op]))]), its consequent op decoded and matched to
%     the raw-header keyword (D1) — defense-in-depth over the raw gate's surface cross-check.
%   - a STRUCTURAL guard whitelist (M3.profile-structure): exactly one population object plus one or
%     more WELL-WIRED components. A component is a concept {object(C,eq,1), predicate(have,Pop,C)}
%     or an interval {object(age,eq,1), object(year,CountOp,N), relation(age,of,year),
%     predicate(have,Pop,age)} with D9 placement (geq/greater top-level, leq/less in a one-level
%     sublist) and correct of/have wiring. A bounded age range (>=1 well-wired interval, any count)
%     is admitted (user decision 2026-07-15). It rejects a patient-only guard, a mis-placed /
%     mis-wired interval, a non-interval atom in the interval sublist, in-guard negation (-drs),
%     disjunction (v), a non-v1 interval marker (exactly / bare eq), a negative interval bound, and
%     any alien conjunct.
%   - the action shape predicate(_, take, <subject>, named(<registered drug>)), subject == the
%     guard's population referent.
%   - the exception body (D6): exactly one population + one concept wired by have (single-concept,
%     interval-free, op-free). It rejects a patient-only / multi-concept / empty body, a mis-wired
%     have, and any alien (op / interval) atom.
%
% The DRS is acetext_to_drs/5's term (real referent vars + `-SID/TID` provenance); the gate
% (profile_check_tests.pl) reconstructs it from the byte-pinned surface goldens — a serialized DRS
% reads back to the same term (round-trip proven) — so the checker runs pure and fast, no live APE.
%
%   Gate: swipl -q -g "consult('clinical/profile_check_tests.pl'),(run_tests(profile_check)->halt(0);halt(1))" -t 'halt(1)'

:- module(profile_check, [ profile_check/4 ]).

% registry = the id<->surface + modality authority; is_wellformed/1 = APE's DRS hygiene check.
% Both loaded source-relative so the module is cwd-independent (mirrors registry.pl loading kb_kernel).
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/registry.pl'], R), use_module(R),
   atomic_list_concat([D, '/../prolog/utils/is_wellformed.pl'], W), use_module(W).

%% profile_check(+Ctx, +Drs, +Messages, -Result) is det.
% Ctx  = the raw gate's per-sentence context: rule(_,Keyword,_,_,_) | exception(_,_,_,_) — only the
%        kind (rule / exception) and, for a rule, the modality Keyword are read.
% Drs  = the APE DRS term for the sentence (acetext_to_drs/5).
% Messages = the parse's message list (the zero-message law's substrate).
% Result = ok | reject(Reason) — Reason a precise, single-cause term (profile-battery pins each).
% Checks run earliest-most-general first (messages, then hygiene, then the named scan, then shape),
% so the reject Reason names the sole failing rule.
profile_check(Ctx, Drs, Messages, Result) :-
    (   Messages \== []                 -> Result = reject(nonempty_messages)
    ;   \+ is_wellformed(Drs)           -> Result = reject(not_wellformed)
    ;   unregistered_named(Drs, Name)   -> Result = reject(unregistered_named(Name))
    ;   body_outcome(Ctx, Drs, Outcome) -> Result = Outcome
    ;   Result = reject(bad_top_shape)
    ).

% body_outcome(+Ctx, +Drs, -Outcome) — the shape + structural whitelist per Ctx kind. The head pins
% the top-level DRS shape (a rule's implication / an exception's bare body); a DRS that fails to
% match the head leaves body_outcome to FAIL, so profile_check reports bad_top_shape. Outcome is
% ok | reject(Reason).
body_outcome(rule(_, Keyword, _, _, _),
             drs([], [ =>(drs(_, GConds), drs([], [OpCond])) ]),
             Outcome) :- !,
    % guard_check runs unconditionally (total, deterministic) so PatRef stays bound through to the
    % action check; a failing if-condition would otherwise undo the binding.
    guard_check(GConds, PatRef, GO),
    (   GO = reject(_)                 -> Outcome = GO
    ;   \+ consequent_op(OpCond, _, _) -> Outcome = reject(bad_consequent)
    ;   consequent_op(OpCond, Op, ActionDrs),
        reg_keyword(Keyword, ExpOp, _, _),
        (   Op \== ExpOp -> Outcome = reject(op_mismatch(ExpOp, Op))
        ;   action_outcome(ActionDrs, PatRef, Outcome)
        )
    ).
body_outcome(exception(_, _, _, _), drs(_, Conds), Outcome) :- !,
    exception_outcome(Conds, Outcome).

% ==========================================================================================
% Structural guard whitelist. Two passes over the guard's condition list.
%   normalize/3  — validate each interval marker + its D9 placement and flatten each one-level
%                  interval sublist, so the wiring pass sees a flat conjunct list.
%   wire_guard/3 — bind the sole population referent, then consume the flat conjuncts
%                  component-by-component (each anchored by a concept / age object + its wiring),
%                  requiring >=1 component and no leftover atom.
% Companion atoms are matched by referent IDENTITY (==), sound because is_wellformed/1 already
% proved the referents distinct and declared.
% ==========================================================================================

guard_check(GConds, PatRef, Outcome) :-
    normalize(GConds, Flat, NO),
    ( NO = reject(_) -> Outcome = NO ; wire_guard(Flat, PatRef, Outcome) ).

% --- pass 1: interval marker + D9 placement, flattening interval sublists --------------------------

% normalize(+Conds, -Flat, -Outcome) — Flat (meaningful only when Outcome=ok) is Conds with each
% valid interval sublist replaced by its atoms; Outcome names the FIRST interval / sublist defect.
normalize([], [], ok).
normalize([C|Cs], Flat, Outcome) :-
    norm_item(C, Atoms, IO),
    (   IO = reject(_)
    ->  Flat = [], Outcome = IO
    ;   normalize(Cs, Rest, Outcome), append(Atoms, Rest, Flat)
    ).

% norm_item(+Conjunct, -Atoms, -Outcome) — a list conjunct is a one-level interval sublist (nested
% leq/less bound), flattened to its atoms; a top-level year object carries a top-placed bound
% (geq/greater); every other conjunct passes through for the wiring pass.
norm_item(List, Atoms, Outcome) :- is_list(List), !, norm_sublist(List, Atoms, Outcome).
norm_item(object(R, N, Ct, U, CO, Num)-P, [object(R, N, Ct, U, CO, Num)-P], Outcome) :-
    unit_noun(N), !, check_year(CO, Num, top, Outcome).
norm_item(C, [C], ok).

% norm_sublist(+List, -Atoms, -Outcome) — a nested interval bound: an of-relation + a bounded year
% object (nested placement), flattened; any other element (incl. a deeper list) rejects.
norm_sublist([], [], ok).
norm_sublist([E|Es], Atoms, Outcome) :-
    (   sublist_element(E, EAtoms, IO)
    ->  (   IO = reject(_)
        ->  Atoms = [], Outcome = IO
        ;   norm_sublist(Es, Rest, Outcome), append(EAtoms, Rest, Atoms)
        )
    ;   Atoms = [], Outcome = reject(interval_sublist(E))
    ).

sublist_element(relation(A, of, Y)-P, [relation(A, of, Y)-P], ok) :- !.
sublist_element(object(R, N, Ct, U, CO, Num)-P, [object(R, N, Ct, U, CO, Num)-P], Outcome) :-
    unit_noun(N), !, check_year(CO, Num, nested, Outcome).

% check_year(+CountOp, +Num, +Placement, -Outcome) — a v1 interval bound: a single-bound marker
% (exactly / bare eq are non-v1), a non-negative integer value, and D9 placement (geq/greater must
% be top-level, leq/less nested).
check_year(CO, Num, Place, Outcome) :-
    (   \+ v1_countop(CO)             -> Outcome = reject(interval_countop(CO))
    ;   \+ ( integer(Num), Num >= 0 ) -> Outcome = reject(interval_bound(Num))
    ;   \+ placement_ok(CO, Place)    -> Outcome = reject(interval_placement(CO))
    ;   Outcome = ok
    ).

placement_ok(geq,     top).
placement_ok(greater, top).
placement_ok(leq,     nested).
placement_ok(less,    nested).

% --- pass 2: population + component wiring ---------------------------------------------------------

% wire_guard(+Flat, -PatRef, -Outcome) — bind PatRef to the sole population referent, then consume
% the remaining conjuncts as well-wired concept / interval components.
wire_guard(Flat, PatRef, Outcome) :-
    population_refs(Flat, Pops),
    (   Pops = [PatRef]
    ->  exclude_pop(Flat, PatRef, Pool),
        consume_components(Pool, PatRef, 0, Outcome)
    ;   Outcome = reject(bad_population)
    ).

% population_refs(+Conds, -Refs) — the referents of every top-level population object.
population_refs([], []).
population_refs([object(R, N, countable, na, eq, 1)-_|Cs], [R|Rs]) :- reg_population(_, N), !,
    population_refs(Cs, Rs).
population_refs([_|Cs], Rs) :- population_refs(Cs, Rs).

% exclude_pop(+Conds, +PatRef, -Pool) — drop the sole population object (by referent identity).
exclude_pop([], _, []).
exclude_pop([object(R, _, _, _, _, _)-_|Cs], PatRef, Cs) :- R == PatRef, !.
exclude_pop([C|Cs], PatRef, [C|Pool]) :- exclude_pop(Cs, PatRef, Pool).

% consume_components(+Pool, +PatRef, +N0, -Outcome) — pull a complete component out of Pool
% (anchored by a concept / age object + its wiring) and recurse. When no anchor remains, a leftover
% conjunct is a wiring / shape violation; otherwise at least one component is required.
consume_components(Pool, PatRef, N0, Outcome) :-
    (   select_anchor(Pool, Obj, Pool1)
    ->  anchor(Obj, PatRef, Pool1, Pool2, R),
        (   R = reject(_) -> Outcome = R
        ;   N1 is N0 + 1, consume_components(Pool2, PatRef, N1, Outcome)
        )
    ;   (   Pool = [L|_] -> leftover_reject(L, Outcome)
        ;   N0 =:= 0     -> Outcome = reject(no_guard_component)
        ;   Outcome = ok
        )
    ).

% select_anchor(+Pool, -Obj, -Rest) — remove the first concept object or age object.
select_anchor([Obj|T], Obj, T) :- anchor_object(Obj), !.
select_anchor([H|T], Obj, [H|Rest]) :- select_anchor(T, Obj, Rest).

anchor_object(object(_, N, countable, na, eq, 1)-_) :- component_noun(N).
component_noun(N) :- reg_concept(_, N), !.
component_noun(N) :- reg_quantity(_, N, _, _, _), !.

% anchor(+Object, +PatRef, +Pool, -Rest, -Outcome) — consume Object's component from Pool. A concept
% needs its have (PatRef -> concept); an interval needs its have plus an of-relation + bounded year
% object (age -> year). A missing / mis-wired companion is a guard_wiring violation naming Object.
anchor(Obj, PatRef, Pool, Rest, Outcome) :- Obj = object(C, N, _, _, _, _)-_, reg_concept(_, N), !,
    (   select_have(C, PatRef, Pool, Rest) -> Outcome = ok
    ;   Rest = Pool, Outcome = reject(guard_wiring(Obj))
    ).
anchor(Obj, PatRef, Pool, Rest, Outcome) :- Obj = object(A, N, _, _, _, _)-_, reg_quantity(_, N, _, _, _), !,
    (   select_have(A, PatRef, Pool, Pool1),
        select_rel(A, Y, Pool1, Pool2),
        select_year(Y, Pool2, Rest)
    ->  Outcome = ok
    ;   Rest = Pool, Outcome = reject(guard_wiring(Obj))
    ).

% select_have(+ObjRef, +PatRef, +Pool, -Rest) — remove predicate(_,have,PatRef,ObjRef) (subject and
% object matched by referent identity). Fails if absent or the subject is not PatRef.
select_have(ObjRef, PatRef, [predicate(_, have, S, O)-_|T], T) :- S == PatRef, O == ObjRef, !.
select_have(ObjRef, PatRef, [X|T], [X|Rest]) :- select_have(ObjRef, PatRef, T, Rest).

% select_rel(+AgeRef, -YearRef, +Pool, -Rest) — remove relation(AgeRef, of, YearRef) (age matched by
% identity), binding YearRef.
select_rel(A, Y, [relation(A2, of, Y2)-_|T], T) :- A2 == A, !, Y = Y2.
select_rel(A, Y, [X|T], [X|Rest]) :- select_rel(A, Y, T, Rest).

% select_year(+YearRef, +Pool, -Rest) — remove the bounded year object with referent YearRef.
select_year(Y, [object(Y2, N, _, _, _, _)-_|T], T) :- Y2 == Y, unit_noun(N), !.
select_year(Y, [X|T], [X|Rest]) :- select_year(Y, T, Rest).

% leftover_reject(+Conjunct, -Outcome) — a guard conjunct left after component matching: an orphan
% wiring piece (a have / of-relation / bounded year object with no owning component) is a wiring
% error; anything else (disjunction, in-guard negation, an alien atom) is a guard-shape violation.
leftover_reject(C, reject(guard_wiring(C))) :- orphan_piece(C), !.
leftover_reject(C, reject(guard_shape(C))).

orphan_piece(predicate(_, have, _, _)-_).
orphan_piece(relation(_, of, _)-_).
orphan_piece(object(_, N, countable, na, _, _)-_) :- unit_noun(N).

% --- consequent op: normalize the then-part op condition to its token + action DRS (D1). --------
consequent_op(should(A), should, A) :- !.
consequent_op(may(A), may, A) :- !.
consequent_op(-(drs([], [should(A)])), '-should', A) :- !.
consequent_op(-(drs([], [can(A)])), '-can', A) :- !.

% --- action shape: drs([Act],[predicate(Act, take, <subject>, named(<drug>))]); subject == the
% guard's population referent. The drug's REGISTRATION is the recursive named scan's sole
% responsibility (that scan already rejected any unregistered named before body_outcome runs), so
% here the target is only checked to be a GROUND proper name (named/1 of an atom) — the two together
% give named(RegisteredDrug); an unbound or non-atom target fails the whitelist.
action_outcome(ActionDrs, PatRef, Outcome) :-
    (   ActionDrs = drs([Act], [predicate(Act2, Verb, SubjRef, Target)-_])
    ->  (   Act \== Act2                             -> Outcome = reject(bad_action_referent)
        ;   reg_action(_, _, _, Lemma), Verb \== Lemma -> Outcome = reject(bad_action_verb(Verb))
        ;   SubjRef \== PatRef                       -> Outcome = reject(action_subject_mismatch)
        ;   nonvar(Target), Target = named(N), atom(N) -> Outcome = ok
        ;   Outcome = reject(bad_action_target(Target))
        )
    ;   Outcome = reject(bad_action_shape)
    ).

% ==========================================================================================
% Exception body (D6): exactly one population + one concept, wired by have; interval-free, op-free.
% An alien (op / interval / relation / any non {population, concept, have}) atom is caught first, so
% the population / concept cardinality reasons name a genuine cardinality defect. population_refs +
% concept_refs are hoisted before the conditional so the last branch's referents stay bound.
% ==========================================================================================
exception_outcome(Conds, Outcome) :-
    population_refs(Conds, Pops),
    concept_refs(Conds, Cs),
    (   first_exc_alien(Conds, Bad) -> Outcome = reject(exception_shape(Bad))
    ;   Pops \= [_]                 -> Outcome = reject(bad_exception(population))
    ;   Cs \= [_]                   -> Outcome = reject(bad_exception(concept_count))
    ;   Pops = [PatRef], Cs = [CRef],
        (   has_have(Conds, PatRef, CRef) -> Outcome = ok
        ;   Outcome = reject(bad_exception(wiring))
        )
    ).

% first_exc_alien(+Conds, -Bad) — the first conjunct that is not a population object, a concept
% object, or a have predicate (an op / interval / relation etc.).
first_exc_alien([C|_], C) :- \+ exc_allowed(C), !.
first_exc_alien([_|Cs], Bad) :- first_exc_alien(Cs, Bad).

exc_allowed(object(_, N, countable, na, eq, 1)-_) :- reg_population(_, N), !.
exc_allowed(object(_, N, countable, na, eq, 1)-_) :- reg_concept(_, N), !.
exc_allowed(predicate(_, have, _, _)-_).

% concept_refs(+Conds, -Refs) — the referents of every concept object.
concept_refs([], []).
concept_refs([object(R, N, countable, na, eq, 1)-_|Cs], [R|Rs]) :- reg_concept(_, N), !, concept_refs(Cs, Rs).
concept_refs([_|Cs], Rs) :- concept_refs(Cs, Rs).

% has_have(+Conds, +PatRef, +ConceptRef) — a have predicate wiring PatRef -> ConceptRef (by identity).
has_have([predicate(_, have, S, O)-_|_], PatRef, CRef) :- S == PatRef, O == CRef, !.
has_have([_|Cs], PatRef, CRef) :- has_have(Cs, PatRef, CRef).

% ==========================================================================================
% The recursive named scan + vocabulary.
% ==========================================================================================

% unregistered_named(+Term, -Name) — some named(Name) anywhere in Term is off the pn allowlist, or
% is not a ground atom (an unbound name would otherwise BIND through pn_allow/1 and slip the scan).
unregistered_named(Term, Name) :-
    named_args(Term, Names),
    member(Name, Names),
    ( atom(Name) -> \+ pn_allow(Name) ; true ).

% named_args(+Term, -Names) — every named/1 argument in Term, via a subterm worklist (referent vars
% are skipped; order is irrelevant — the caller only asks whether an unregistered one exists).
named_args(Term, Names) :- named_walk([Term], [], Names).
named_walk([], A, A).
named_walk([T|Ts], A0, A) :-
    (   var(T)      -> named_walk(Ts, A0, A)
    ;   compound(T) -> ( T = named(N) -> A1 = [N|A0] ; A1 = A0 ),
                       T =.. [_|Args], append(Args, Ts, W), named_walk(W, A1, A)
    ;   named_walk(Ts, A0, A)
    ).

% unit_noun = the interval unit noun's DRS lemma (year); the v1 interval markers (SURFACE.md
% §Intervals; exactly / bare eq are non-v1).
unit_noun(N) :- reg_quantity(_, _, _, _, N).
v1_countop(geq).
v1_countop(greater).
v1_countop(leq).
v1_countop(less).
