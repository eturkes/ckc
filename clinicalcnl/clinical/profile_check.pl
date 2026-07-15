% ClinicalCNL post-APE DRS profile checker — the second fail-closed layer (M3.profile-drs;
% SPEC §10.6, SURFACE.md). The raw gate (raw_gate.pl) whitelists surfaces BEFORE APE; APE then
% erases surface facts (prefix tokens, numeral spellings, comments, silent anaphora merges), so an
% independent layer re-validates the DRS AFTER APE. profile_check/4 is that layer: a pure
% structural whitelist over the APE DRS term — it reads a term, never runs a parser — fail-closed
% on anything outside the v1 profile.
%
% What it enforces (SURFACE.md + the D-decisions, roadmap §M3):
%   - the zero-message law: an accepted v1 parse is warning-free. APE's solo seam drops warnings,
%     so the anaphor and undefined-word `named` holes surface only here and at the raw gate.
%   - canonical referent hygiene via APE's own is_wellformed/1 (no duplicate / undeclared
%     referents) — the first-parse-wins canonical backstop.
%   - a recursive named(_) scan against the pn allowlist (the p6 `named` hole discriminator).
%   - the rule shape drs([],[=>(guard, drs([],[op]))]), its consequent op decoded and matched to
%     the raw-header keyword (D1) — defense-in-depth over the raw gate's surface cross-check.
%   - a guard whitelist: population / concept / interval objects, the of-relation and have-predicate,
%     with one level of interval sublist (D8/D9 nest leq/less bounds). It rejects in-guard negation
%     (-drs), disjunction (v), and bare-eq / exactly interval bounds (the single-bound law).
%   - the action shape predicate(_, take, <subject>, named(<registered drug>)), whose subject is the
%     guard's population referent.
%   - the exception body: a bare, interval-free, op-free concept-have condition (D6).
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

% body_outcome(+Ctx, +Drs, -Outcome) — the shape + content whitelist per Ctx kind. The head pins
% the top-level DRS shape (a rule's implication / an exception's bare body); a DRS that fails to
% match the head leaves body_outcome to FAIL, so profile_check reports bad_top_shape. Outcome is
% ok | reject(Reason).
body_outcome(rule(_, Keyword, _, _, _),
             drs([], [ =>(drs(_, GConds), drs([], [OpCond])) ]),
             Outcome) :- !,
    % guard_outcome + population_ref run unconditionally (both total, deterministic) so PatRef stays
    % bound through to the action check; a failing if-condition would otherwise undo the binding.
    guard_outcome(GConds, GO),
    population_ref(GConds, PatRef, PO),
    (   GO = reject(_)                 -> Outcome = GO
    ;   PO = reject(_)                 -> Outcome = PO
    ;   \+ consequent_op(OpCond, _, _) -> Outcome = reject(bad_consequent)
    ;   consequent_op(OpCond, Op, ActionDrs),
        reg_keyword(Keyword, ExpOp, _, _),
        (   Op \== ExpOp -> Outcome = reject(op_mismatch(ExpOp, Op))
        ;   action_outcome(ActionDrs, PatRef, Outcome)
        )
    ).
body_outcome(exception(_, _, _, _), drs(_, Conds), Outcome) :- !,
    exception_outcome(Conds, Outcome).

% --- guard walker: every conjunct a whitelisted population / concept / interval atom, with a
% single level of sublist nesting (D8/D9 place leq/less interval atoms in a nested list). ---------
guard_outcome([], ok).
guard_outcome([C|Cs], Outcome) :-
    guard_item(C, IO),
    ( IO = reject(_) -> Outcome = IO ; guard_outcome(Cs, Outcome) ).

% guard_item(+Conjunct, -Outcome). A list conjunct is a one-level interval sublist; a bare atom is a
% population / concept / quantity object, the of-relation, or the have-predicate. A `year` unit
% object carries the interval bound (v1 markers only); every other object is an eq-1 head noun.
guard_item(List, Outcome) :- is_list(List), !, sublist_outcome(List, Outcome).
guard_item(object(_, N, countable, na, eq, 1)-_, ok) :- eq1_noun(N), !.
guard_item(object(_, N, countable, na, CO, INT)-_, Outcome) :- unit_noun(N), !,
    ( v1_countop(CO), integer(INT) -> Outcome = ok ; Outcome = reject(interval_countop(CO)) ).
guard_item(relation(_, of, _)-_, ok) :- !.
guard_item(predicate(_, have, _, _)-_, ok) :- !.
guard_item(Other, reject(guard_shape(Other))).

% sublist_outcome(+Items, -Outcome) — a one-level interval sublist: interval atoms only, no further
% nesting.
sublist_outcome([], ok).
sublist_outcome([C|Cs], Outcome) :-
    (   is_list(C) -> Outcome = reject(nested_sublist(C))
    ;   guard_item(C, IO), ( IO = reject(_) -> Outcome = IO ; sublist_outcome(Cs, Outcome) )
    ).

% --- consequent op: normalize the then-part op condition to its token + action DRS (D1). --------
consequent_op(should(A), should, A) :- !.
consequent_op(may(A), may, A) :- !.
consequent_op(-(drs([], [should(A)])), '-should', A) :- !.
consequent_op(-(drs([], [can(A)])), '-can', A) :- !.

% --- action shape: drs([Act],[predicate(Act, take, <subject>, named(<drug>))]); subject == the
% guard's population referent. The drug's REGISTRATION is the recursive named scan's sole
% responsibility (that scan already rejected any unregistered named before body_outcome runs), so
% here the target is only checked to be a proper name — the two together give named(RegisteredDrug).
action_outcome(ActionDrs, PatRef, Outcome) :-
    (   ActionDrs = drs([Act], [predicate(Act2, Verb, SubjRef, Target)-_])
    ->  (   Act \== Act2                             -> Outcome = reject(bad_action_referent)
        ;   reg_action(_, _, _, Lemma), Verb \== Lemma -> Outcome = reject(bad_action_verb(Verb))
        ;   SubjRef \== PatRef                       -> Outcome = reject(action_subject_mismatch)
        ;   Target = named(_)                        -> Outcome = ok
        ;   Outcome = reject(bad_action_target(Target))
        )
    ;   Outcome = reject(bad_action_shape)
    ).

% --- exception body: a bare, interval-free, op-free concept-have condition (D6). ----------------
exception_outcome([], ok).
exception_outcome([C|Cs], Outcome) :-
    exception_item(C, IO),
    ( IO = reject(_) -> Outcome = IO ; exception_outcome(Cs, Outcome) ).

exception_item(object(_, N, countable, na, eq, 1)-_, ok) :- concept_or_population(N), !.
exception_item(predicate(_, have, _, _)-_, ok) :- !.
exception_item(Other, reject(exception_shape(Other))).

% --- referents + vocabulary ---------------------------------------------------------------------

% population_ref(+GConds, -PatRef, -Outcome) — bind PatRef to THE population referent (the real var,
% so the action-subject `==` check is sound), rejecting a guard without exactly one population object.
population_ref(GConds, PatRef, Outcome) :-
    reg_population(_, PatNoun),
    population_refs(GConds, PatNoun, Refs),
    ( Refs = [PatRef] -> Outcome = ok ; Outcome = reject(bad_population) ).

population_refs([], _, []).
population_refs([object(R, N, countable, na, eq, 1)-_|Cs], N, [R|Rs]) :- !, population_refs(Cs, N, Rs).
population_refs([_|Cs], N, Rs) :- population_refs(Cs, N, Rs).

% unregistered_named(+Term, -Name) — some named(Name) anywhere in Term is off the pn allowlist.
unregistered_named(Term, Name) :-
    named_args(Term, Names),
    member(Name, Names),
    \+ pn_allow(Name).

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

% eq-1 head nouns (population subject, condition concepts, the age var noun) vs the interval unit
% noun (year); the v1 interval markers (SURFACE.md §Intervals; exactly / bare eq are non-v1).
eq1_noun(N) :- reg_population(_, N).
eq1_noun(N) :- reg_concept(_, N).
eq1_noun(N) :- reg_quantity(_, N, _, _, _).
unit_noun(N) :- reg_quantity(_, _, _, _, N).
concept_or_population(N) :- reg_population(_, N).
concept_or_population(N) :- reg_concept(_, N).
v1_countop(geq).
v1_countop(greater).
v1_countop(leq).
v1_countop(less).
