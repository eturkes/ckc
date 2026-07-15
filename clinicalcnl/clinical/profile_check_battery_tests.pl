% ClinicalCNL profile-checker DRS reject battery (M3.profile-battery; SPEC §10.6, SURFACE.md).
%
% The systematic hand-mutant matrix over profile_check/4 (the post-APE DRS whitelist). profile_check_tests.pl
% carries the accept battery (the v1 goldens) + a non-vacuity floor (the nonv1 goldens' 5 reject paths + 4
% crafted escapes); this unit adds full DRS-side reject DEPTH: one mutant CLASS per reject Construct the
% checker emits, each mutant a single-locus edit of a proven-accepted base asserting the exact reject.
% The hand-built DRS terms carry real referent vars (like the crafted escapes), so the checker runs pure and
% fast with no live APE — the p7 DRS hazards (v() disjunction, in-guard -drs, a fresh-referent then-part, a
% bare-then top, an unregistered named, a warning-bearing parse, an op/keyword mismatch per modality,
% malformed interval sublists) become concrete terms here.
%
% Anti-vacuity is PER-MUTANT, not sampled: every case carries its EXACT accepted base and bases_accept proves
% each base maps to ok, so a reject is the mutation's doing — a base that silently stopped accepting fails
% bases_accept rather than hiding behind a still-red mutant. Constructs are OBSERVED by running each mutant
% (never assumed; memory: never assert a reject Construct from a partial probe).
%
% HONEST COVERAGE. profile_construct/1 is the closed set of reject Constructs the checker emits, and
% constructs_match_source binds it to the set scanned from profile_check.pl's own reject(...) sites (parsed as
% terms — an independent authority, not the self-referential banked list) so a gate Construct nobody banked
% fails a self-check rather than staying invisible. every_construct_has_mutant then proves each has a mutant:
% the battery is exhaustive over the gate's emitted reject Constructs. There is NO dead defensive branch — bad_action_referent
% (Act =\= the predicate's event arg) is reachable when the used event arg is a guard referent (wellformed),
% so every emitted Construct is covered.
%
% Out of reject scope by design (profile_check is SHAPE-per-atom, NOT cardinality/placement — memory
% codex-930f954): the D9 canonical-guard-shape gaps codex flagged are ACCEPTS here, not rejects, so they carry
% no profile mutant and are map-core's (the canonical guard shape owner) — a top-level leq/less interval bound
% (v1_countop admits all four markers regardless of placement), a guard with >1 interval object, a non-interval
% atom nested inside the interval sublist, and the unconstrained of / have conjunct args (relation(_,of,_) /
% predicate(_,have,_,_) accept any arguments). profile-battery covers only what profile_check itself rejects.
%
%   Gate: swipl -q -g "consult('clinical/profile_check_battery_tests.pl'),(run_tests(profile_battery)->halt(0);halt(1))" -t 'halt(1)'

:- module(profile_check_battery_tests, []).

:- use_module(library(plunit)).

:- dynamic pc_source/1.

% Load the checker + the registry (the keyword × op authority for the op-mismatch Cartesian), source-relative
% + cwd-independent (mirrors profile_check_tests.pl). profile_check loads the registry too; use_module is
% idempotent. The checker's own file path is remembered for the source-scan self-check.
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/profile_check.pl'], PC), use_module(PC),
   atomic_list_concat([D, '/registry.pl'], Rg), use_module(Rg),
   assertz(pc_source(PC)).

% ==========================================================================================
% profile_construct(?Construct) — the closed set of reject Constructs profile_check/4 emits (its reject(...)
% functors). Enumerated from the checker source; constructs_match_source pins it to the source's actual sites.
% ==========================================================================================

profile_construct(nonempty_messages).       % a non-empty parse message list (the zero-message law)
profile_construct(not_wellformed).          % APE is_wellformed/1 fails (referent hygiene)
profile_construct(unregistered_named).      % a named() off the pn allowlist (or non-atom)
profile_construct(bad_top_shape).           % the top DRS is neither a rule implication nor an exception body
profile_construct(bad_consequent).          % the consequent op is not should / may / -should / -can
profile_construct(op_mismatch).             % D1 — the decoded op disagrees with the keyword's required op
profile_construct(interval_countop).        % a non-v1 interval marker (exactly / bare eq)
profile_construct(interval_bound).          % a negative interval value (v1 INTs are non-negative)
profile_construct(guard_shape).             % a guard conjunct outside the population / concept / interval set
profile_construct(nested_sublist).          % a list nested inside the one-level interval sublist
profile_construct(bad_population).          % a guard without exactly one population object
profile_construct(bad_action_referent).     % the action's declared event differs from its predicate event arg
profile_construct(bad_action_verb).         % the action verb is not the registered take lemma
profile_construct(action_subject_mismatch). % the action subject is not the guard's population referent
profile_construct(bad_action_target).       % the action target is not a ground named() drug
profile_construct(bad_action_shape).        % the consequent action is not a single take-predicate DRS
profile_construct(exception_shape).         % an exception body conjunct is not a bare concept-have

% ==========================================================================================
% Shared DRS fragments (fresh vars per call; var sharing within a fragment is explicit — the population
% referent P threads guard -> action). mk_rule assembles the canonical rule implication.
% ==========================================================================================

ctx_recommend(rule(0, recommend, 0, none, none)).
ctx_exception(exception(0, 0, none, none)).

% sepsis_guard(-P, -Domain, -Conds) — the sepsis concept guard, population referent P.
sepsis_guard(P, [P, S, H],
    [ object(P, patient, countable, na, eq, 1)-1/3,
      object(S, sepsis,  countable, na, eq, 1)-1/6,
      predicate(H, have, P, S)-1/4 ]).

% interval_guard(+Op, +N, -P, -Domain, -Conds) — the age-interval guard, unit object top-level (geq/greater
% canonical placement); Op/N vary for the interval mutants.
interval_guard(Op, N, P, [P, Ag, Yr, H],
    [ object(P,  patient, countable, na, eq, 1)-1/3,
      object(Ag, age,     countable, na, eq, 1)-1/6,
      object(Yr, year,    countable, na, Op, N)-1/11,
      relation(Ag, of, Yr)-1/7,
      predicate(H, have, P, Ag)-1/4 ]).

% v1_action(+Subj, -ActionDrs) / named_action(+Name, +Subj, -ActionDrs) — the take-Abx-A action; the drug
% name varies for the unregistered-named mutant.
v1_action(Subj, drs([Ev], [predicate(Ev, take, Subj, named('Abx-A'))-1/14])).
named_action(Name, Subj, drs([Ev], [predicate(Ev, take, Subj, named(Name))-1/14])).

% consequent(+Op, +ActionDrs, -OpCond) — the consequent op condition per token; mk_rule wraps a guard + a
% consequent into the rule implication drs([],[ =>(guard, drs([],[OpCond])) ]).
consequent(should,    A, should(A)).
consequent(may,       A, may(A)).
consequent('-should', A, -(drs([], [should(A)]))).
consequent('-can',    A, -(drs([], [can(A)]))).

mk_rule(GDom, GConds, OpCond, drs([], [ =>(drs(GDom, GConds), drs([], [OpCond])) ])).

% ==========================================================================================
% Accepted bases — each proven by bases_accept. base/2 keys a mutant to its counterfactual accept.
% ==========================================================================================

base(recommend_rule,       Drs) :- valid_rule(Drs).
base(patient_rule,         Drs) :- valid_patient_rule(Drs).
base(interval_rule,        Drs) :- valid_interval_rule(Drs).
base(nested_interval_rule, Drs) :- valid_nested_interval_rule(Drs).
base(exception,            Drs) :- valid_exception(Drs).

valid_rule(Drs) :-
    sepsis_guard(P, GDom, GConds), v1_action(P, A), mk_rule(GDom, GConds, should(A), Drs).

valid_patient_rule(Drs) :-
    v1_action(P, A),
    mk_rule([P], [object(P, patient, countable, na, eq, 1)-1/3], should(A), Drs).

valid_interval_rule(Drs) :-
    interval_guard(geq, 18, P, GDom, GConds), v1_action(P, A), mk_rule(GDom, GConds, should(A), Drs).

% the leq/less markers land in a one-level nested sublist (D9); the guard walker flattens one level.
valid_nested_interval_rule(Drs) :-
    v1_action(P, A),
    mk_rule([P, Ag, Yr, H],
        [ object(P,  patient, countable, na, eq,  1)-1/3,
          object(Ag, age,     countable, na, eq,  1)-1/6,
          [ relation(Ag, of, Yr)-1/7, object(Yr, year, countable, na, leq, 18)-1/11 ],
          predicate(H, have, P, Ag)-1/4 ], should(A), Drs).

valid_exception(drs([P, Cn, H],
    [ object(P,  patient,                    countable, na, eq, 1)-1/2,
      object(Cn, 'severe-renal-impairment',  countable, na, eq, 1)-1/5,
      predicate(H, have, P, Cn)-1/3 ])).

% ==========================================================================================
% DRS mutants: drs_mutant(?Label, ?Ctx, ?BaseId, ?Mutant, ?RejectArg) — a single-locus edit of BaseId's
% accepted DRS producing reject(RejectArg). The reject is pinned by functor + every ground arg; the arg's
% referent vars stay `_`. Messages are empty (the message-law mutant is a separate battery_case clause).
% ==========================================================================================

% -- pre-body checks (message / hygiene / named scan / top shape) ---------------------------------------
drs_mutant(undeclared_referent, R, recommend_rule, Mut, not_wellformed) :-
    ctx_recommend(R), mut_undeclared(Mut).
drs_mutant(off_allowlist_drug, R, recommend_rule, Mut, unregistered_named('Zzz')) :-
    ctx_recommend(R), sepsis_guard(P, GD, GC), named_action('Zzz', P, A), mk_rule(GD, GC, should(A), Mut).
drs_mutant(bare_then, R, recommend_rule, Mut, bad_top_shape) :-
    ctx_recommend(R), mut_bare_then(Mut).

% -- rule body: consequent + op (D1) -------------------------------------------------------------------
drs_mutant(must_consequent, R, recommend_rule, Mut, bad_consequent) :-
    ctx_recommend(R), sepsis_guard(P, GD, GC), v1_action(P, A), mk_rule(GD, GC, must(A), Mut).

% -- guard walker: intervals, shape, sublist nesting ---------------------------------------------------
drs_mutant(exactly_marker, R, interval_rule, Mut, interval_countop(exactly)) :-
    ctx_recommend(R), interval_guard(exactly, 18, P, GD, GC), v1_action(P, A), mk_rule(GD, GC, should(A), Mut).
drs_mutant(negative_bound, R, interval_rule, Mut, interval_bound(-1)) :-
    ctx_recommend(R), interval_guard(geq, -1, P, GD, GC), v1_action(P, A), mk_rule(GD, GC, should(A), Mut).
drs_mutant(alien_conjunct, R, recommend_rule, Mut, guard_shape(property(_, foo, pos)-_)) :-
    ctx_recommend(R), mut_guard_alien(Mut).
drs_mutant(disjunction_conjunct, R, patient_rule, Mut, guard_shape(v(_, _))) :-
    ctx_recommend(R), mut_guard_disjunction(Mut).
drs_mutant(in_guard_negation, R, patient_rule, Mut, guard_shape(-(_))) :-
    ctx_recommend(R), mut_guard_negation(Mut).
drs_mutant(deep_sublist, R, nested_interval_rule, Mut, nested_sublist([object(_, year, countable, na, leq, 18)-_])) :-
    ctx_recommend(R), mut_nested_sublist(Mut).

% -- population referent -------------------------------------------------------------------------------
drs_mutant(zero_population, R, recommend_rule, Mut, bad_population) :-
    ctx_recommend(R), mut_no_population(Mut).
drs_mutant(two_population, R, recommend_rule, Mut, bad_population) :-
    ctx_recommend(R), mut_two_population(Mut).

% -- action shape ---------------------------------------------------------------------------------------
drs_mutant(action_event_mismatch, R, recommend_rule, Mut, bad_action_referent) :-
    ctx_recommend(R), mut_action_referent(Mut).
drs_mutant(alien_verb, R, recommend_rule, Mut, bad_action_verb(give)) :-
    ctx_recommend(R), sepsis_guard(P, GD, GC),
    mk_rule(GD, GC, should(drs([Ev], [predicate(Ev, give, P, named('Abx-A'))-1/14])), Mut).
drs_mutant(subject_not_population, R, recommend_rule, Mut, action_subject_mismatch) :-
    ctx_recommend(R), mut_action_subject(Mut).
drs_mutant(nonnamed_target, R, recommend_rule, Mut, bad_action_target(int(5))) :-
    ctx_recommend(R), sepsis_guard(P, GD, GC),
    mk_rule(GD, GC, should(drs([Ev], [predicate(Ev, take, P, int(5))-1/14])), Mut).
drs_mutant(nonpredicate_action, R, recommend_rule, Mut, bad_action_shape) :-
    ctx_recommend(R), mut_action_shape(Mut).

% -- exception body (D6) --------------------------------------------------------------------------------
drs_mutant(exception_op, X, exception, Mut, exception_shape(should(_))) :-
    ctx_exception(X), mut_exception_op(Mut).
drs_mutant(exception_interval, X, exception, Mut, exception_shape(object(_, year, countable, na, geq, 18)-_)) :-
    ctx_exception(X), mut_exception_interval(Mut).

% -- mutant term builders (the single-locus edits, spelled out) -----------------------------------------

% not_wellformed: the population referent P is used in the guard + action but dropped from the guard domain.
mut_undeclared(Mut) :-
    v1_action(P, A),
    mk_rule([S, H],
        [ object(P, patient, countable, na, eq, 1)-1/3,
          object(S, sepsis,  countable, na, eq, 1)-1/6,
          predicate(H, have, P, S)-1/4 ], should(A), Mut).

% bad_top_shape: a bare modal at top-level (no If-then =>), so neither body_outcome head matches.
mut_bare_then(drs([P],
    [ object(P, patient, countable, na, eq, 1)-1/3,
      should(drs([Ev], [predicate(Ev, take, P, named('Abx-A'))-1/14])) ])).

% guard_shape: an alien (non-whitelisted) conjunct; the sepsis referent S stays used so hygiene passes.
mut_guard_alien(Mut) :-
    v1_action(P, A),
    mk_rule([P, S, H],
        [ object(P, patient, countable, na, eq, 1)-1/3,
          property(S, foo, pos)-1/6,
          predicate(H, have, P, S)-1/4 ], should(A), Mut).

% guard_shape: a disjunctive guard conjunct v(_,_) (the D4/p7 or-guard hazard).
mut_guard_disjunction(Mut) :-
    v1_action(P, A),
    mk_rule([P],
        [ object(P, patient, countable, na, eq, 1)-1/3,
          v( drs([X], [object(X, sepsis,     countable, na, eq, 1)-1/8]),
             drs([Y], [object(Y, pregnancy,  countable, na, eq, 1)-1/9]) ) ], should(A), Mut).

% guard_shape: an in-guard negation -(drs(...)) (the D5/p7 hazard; v1 negatives enter via exceptions).
mut_guard_negation(Mut) :-
    v1_action(P, A),
    mk_rule([P],
        [ object(P, patient, countable, na, eq, 1)-1/3,
          -( drs([S, H], [ object(S, sepsis, countable, na, eq, 1)-1/8,
                           predicate(H, have, P, S)-1/6 ]) ) ], should(A), Mut).

% nested_sublist: a list nested inside the one-level interval sublist.
mut_nested_sublist(Mut) :-
    v1_action(P, A),
    mk_rule([P, Ag, Yr, H],
        [ object(P,  patient, countable, na, eq, 1)-1/3,
          object(Ag, age,     countable, na, eq, 1)-1/6,
          [ relation(Ag, of, Yr)-1/7, [ object(Yr, year, countable, na, leq, 18)-1/11 ] ],
          predicate(H, have, P, Ag)-1/4 ], should(A), Mut).

% bad_population: no population object at all (subject = the sepsis referent so hygiene passes).
mut_no_population(Mut) :-
    named_action('Abx-A', S, A),
    mk_rule([S], [object(S, sepsis, countable, na, eq, 1)-1/6], should(A), Mut).

% bad_population: two population objects.
mut_two_population(Mut) :-
    v1_action(P1, A),
    mk_rule([P1, P2, H],
        [ object(P1, patient, countable, na, eq, 1)-1/3,
          object(P2, patient, countable, na, eq, 1)-1/5,
          predicate(H, have, P2, P1)-1/4 ], should(A), Mut).

% bad_action_referent: the action's declared event referent Ev is not the predicate's event arg (P, a guard
% referent used here — wellformed, so this Construct is reachable, not a dead defensive branch).
mut_action_referent(Mut) :-
    sepsis_guard(P, GD, GC),
    mk_rule(GD, GC, should(drs([Ev], [predicate(P, take, Ev, named('Abx-A'))-1/14])), Mut).

% action_subject_mismatch: the action subject is the sepsis referent S, not the population referent P.
mut_action_subject(Mut) :-
    mk_rule([P, S, H],
        [ object(P, patient, countable, na, eq, 1)-1/3,
          object(S, sepsis,  countable, na, eq, 1)-1/6,
          predicate(H, have, P, S)-1/4 ],
        should(drs([Ev], [predicate(Ev, take, S, named('Abx-A'))-1/14])), Mut).

% bad_action_shape: the consequent wraps an object, not the take-predicate action.
mut_action_shape(Mut) :-
    sepsis_guard(_Pop, GD, GC),
    mk_rule(GD, GC, should(drs([X], [object(X, sepsis, countable, na, eq, 1)-1/14])), Mut).

% exception_shape: a modal op inside an exception body.
mut_exception_op(drs([P, Cn, H],
    [ object(P,  patient,                   countable, na, eq, 1)-1/2,
      object(Cn, 'severe-renal-impairment', countable, na, eq, 1)-1/5,
      predicate(H, have, P, Cn)-1/3,
      should(drs([X], [object(X, sepsis, countable, na, eq, 1)-1/9])) ])).

% exception_shape: an interval object inside an exception body.
mut_exception_interval(drs([P, Cn, H, Yr],
    [ object(P,  patient,                   countable, na, eq, 1)-1/2,
      object(Cn, 'severe-renal-impairment', countable, na, eq, 1)-1/5,
      predicate(H, have, P, Cn)-1/3,
      object(Yr, year, countable, na, geq, 18)-1/9 ])).

% ==========================================================================================
% op-mismatch (D1): the decoded consequent op MUST equal the keyword's required op. The registry-derived
% keyword × op Cartesian — the matching op = the accepted base, every other op = an op_mismatch mutant —
% pins all 6 matches + all 18 mismatches (vs a per-keyword sample). op_rule builds a rule with a given op.
% ==========================================================================================

op_rule(Op, Drs) :-
    sepsis_guard(P, GDom, GConds), v1_action(P, A), consequent(Op, A, OpCond),
    mk_rule(GDom, GConds, OpCond, Drs).

% ==========================================================================================
% battery_case(?Class, ?Label, ?Ctx, ?Base, ?BaseMsgs, ?Mutant, ?MutMsgs, ?Reject) — the flat matrix: the
% DRS mutants (base via base/2, empty messages), the message-law mutant (the base DRS, a non-empty message
% list), and the op-mismatch Cartesian. Class = the reject Construct functor.
% ==========================================================================================

reject_construct(R, R) :- atom(R), !.
reject_construct(R, C) :- functor(R, C, _).

battery_case(Class, Label, Ctx, Base, [], Mut, [], reject(RA)) :-
    drs_mutant(Label, Ctx, BaseId, Mut, RA),
    reject_construct(RA, Class),
    base(BaseId, Base).
battery_case(nonempty_messages, message_law, Ctx, Drs, [], Drs,
             [message(warning, anaphor, 1-3, 'note', 'repair')], reject(nonempty_messages)) :-
    ctx_recommend(Ctx), valid_rule(Drs).
battery_case(op_mismatch, Label, rule(0, Kw, 0, none, none), Base, [], Mut, [],
             reject(op_mismatch(ExpOp, MutOp))) :-
    reg_keyword(Kw, ExpOp, _, _),
    reg_op(MutOp), MutOp \== ExpOp,
    atomic_list_concat([Kw, '_op_', MutOp], Label),
    op_rule(ExpOp, Base),
    op_rule(MutOp, Mut).

% ==========================================================================================
% source_reject_constructs(-Set) — the sorted reject(<Construct>) functors literally present in
% profile_check.pl, parsed as terms (comments and reject(Var) control-flow guards are excluded). The
% independent authority constructs_match_source binds profile_construct/1 to.
% ==========================================================================================

source_reject_constructs(Set) :-
    pc_source(File),
    setup_call_cleanup(open(File, read, S), read_reject_functors(S, Fs), close(S)),
    sort(Fs, Set).

read_reject_functors(S, Fs) :-
    read_term(S, T, []),
    (   T == end_of_file
    ->  Fs = []
    ;   findall(F, ( sub_reject_arg(T, Arg), nonvar(Arg), reject_construct(Arg, F) ), Here),
        read_reject_functors(S, More),
        append(Here, More, Fs) ).

sub_reject_arg(reject(Arg), Arg).
sub_reject_arg(Term, Arg) :- compound(Term), arg(_, Term, Sub), sub_reject_arg(Sub, Arg).

% ==========================================================================================
:- begin_tests(profile_battery).

% -- honest coverage: banked ≡ the checker's emitted Constructs (scanned from source), and every Construct
% has a mutant, so the battery is exhaustive over the whole gate; no mutant names an unbanked Construct. ---
test(constructs_match_source) :-
    findall(C, profile_construct(C), Hand0), sort(Hand0, Hand),
    source_reject_constructs(Source),
    assertion(Hand == Source).
test(every_construct_has_mutant) :-
    findall(C, ( profile_construct(C), \+ battery_case(C, _, _, _, _, _, _, _) ), Missing),
    assertion(Missing == []).
test(no_unbanked_construct) :-
    findall(C, ( battery_case(C, _, _, _, _, _, _, _), \+ profile_construct(C) ), Extra),
    assertion(Extra == []).

% -- anti-vacuity: every mutant's exact base accepts (empty-message accept), so each reject is the mutation's
% doing, not a base that silently stopped accepting. ---------------------------------------------------------
test(bases_accept, [forall(battery_case(Class, Label, Ctx, Base, BMsgs, _, _, _))]) :-
    profile_check(Ctx, Base, BMsgs, Result),
    (   Result == ok
    ->  true
    ;   format(user_error, "~N[profile_battery base] ~w/~w: base did not accept, got ~q~n",
               [Class, Label, Result]), fail ).

% -- the mutation matrix: every mutant rejects with exactly its pinned Construct (functor + ground args). -----
test(mutants_reject, [forall(battery_case(Class, Label, Ctx, _, _, Mut, MMsgs, Reject))]) :-
    profile_check(Ctx, Mut, MMsgs, Result),
    (   Result = Reject
    ->  true
    ;   format(user_error, "~N[profile_battery] ~w/~w: expected ~q, got ~q~n",
               [Class, Label, Reject, Result]), fail ).

:- end_tests(profile_battery).
