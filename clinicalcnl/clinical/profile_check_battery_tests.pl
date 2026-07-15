% ClinicalCNL profile-checker DRS reject battery (M3.profile-battery + M3.profile-structure;
% SPEC §10.6, SURFACE.md).
%
% The systematic hand-mutant matrix over profile_check/4 (the post-APE STRUCTURAL whitelist).
% profile_check_tests.pl carries the accept battery (the v1 goldens) + a non-vacuity floor (the nonv1
% goldens' reject paths + crafted escapes); this unit adds full DRS-side reject DEPTH: one mutant
% CLASS per reject Construct the checker emits, each mutant a single-locus edit of a proven-accepted
% base asserting the exact reject. The hand-built DRS terms carry real referent vars (like the crafted
% escapes), so the checker runs pure and fast with no live APE — the p7 DRS hazards (v() disjunction,
% in-guard -drs, a fresh-referent then-part, a bare-then top, an unregistered named, a warning-bearing
% parse, an op/keyword mismatch per modality) plus the M3.profile-structure hazards (a patient-only
% guard, a mis-placed / mis-wired interval, a non-interval atom in the interval sublist, a malformed
% exception body) become concrete terms here.
%
% Anti-vacuity is PER-MUTANT, not sampled: every case carries its EXACT accepted base and bases_accept
% proves each base maps to ok, so a reject is the mutation's doing, not a base that silently stopped
% accepting. Where wellformedness forces a mutant to touch more than one field (e.g. dropping a
% component's have while keeping the DRS wellformed), the edit is one construct-locus with the
% coordination wellformedness demands, not a literal single-field diff. Constructs are OBSERVED by
% running each mutant (never assumed; memory: never assert a reject Construct from a partial probe).
%
% F3: mutants_reject asserts Result =@= Reject against a FULLY-SPELLED reject term (not a wildcard
% unification) — the echo-payload rejects (guard_shape, interval_sublist, guard_wiring, exception_shape,
% bad_action_target) pin the exact offending subterm, referent-var identity staying free under =@=.
%
% HONEST COVERAGE. profile_construct/1 is the closed set of reject Constructs the checker emits, and
% constructs_match_source binds it to the set of reject(...) FUNCTORS scanned from profile_check.pl
% (parsed as terms — an independent authority, not the self-referential banked list) so a gate Construct
% nobody banked fails a self-check rather than staying invisible. every_construct_has_mutant then proves
% each functor has a mutant. ground_rejects_exercised sharpens this past functor granularity: every
% GROUND reject reason in source (a control atom or discriminant, e.g. bad_exception(population)) must be
% pinned by a mutant that observably returns it — so a per-functor discriminant cannot go untested. The
% M3.profile-structure rejects (interval_placement / interval_sublist / no_guard_component / guard_wiring
% / bad_exception, + the codex-review adds shared_object_referent / bad_provenance / bad_exception(aliased))
% — the accept-gaps codex flagged, now profile_check's to reject — are covered here, not deferred.
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
profile_construct(bad_provenance).           % an atomic provenance sentence id != 1 (D2 fixes SID = 1)
profile_construct(unregistered_named).      % a named() off the pn allowlist (or non-atom)
profile_construct(bad_top_shape).           % the top DRS is neither a rule implication nor an exception body
profile_construct(bad_consequent).          % the consequent op is not should / may / -should / -can
profile_construct(op_mismatch).             % D1 — the decoded op disagrees with the keyword's required op
profile_construct(interval_countop).        % a non-v1 interval marker (exactly / bare eq)
profile_construct(interval_bound).          % a negative interval value (v1 INTs are non-negative)
profile_construct(interval_placement).      % D9 — a leq/less bound at top level, or a geq/greater bound nested
profile_construct(interval_sublist).        % a non-interval element (incl. a deeper list) in the interval sublist
profile_construct(guard_shape).             % a guard conjunct outside the component set (v / -drs / an alien atom)
profile_construct(no_guard_component).      % a guard with a population but no well-wired concept / interval component
profile_construct(guard_wiring).            % a component object with a missing / mis-wired have / of, or an orphan piece
profile_construct(bad_population).          % a guard without exactly one population object
profile_construct(shared_object_referent).  % two guard object atoms sharing one referent (role-aliased)
profile_construct(bad_action_referent).     % the action's declared event differs from its predicate event arg
profile_construct(bad_action_verb).         % the action verb is not the registered take lemma
profile_construct(action_subject_mismatch). % the action subject is not the guard's population referent
profile_construct(bad_action_target).       % the action target is not a ground named() drug
profile_construct(bad_action_shape).        % the consequent action is not a single take-predicate DRS
profile_construct(exception_shape).         % an exception body conjunct is an op / interval / alien atom
profile_construct(bad_exception).           % an exception body without exactly one population + one concept (or mis-wired)

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

% mk_exc(+Dom, +Conds, -Drs) — a bare exception-body DRS. It is a PREDICATE CALL (not a direct
% Mut = drs(...) unification in a mutant builder): SWI folds a last-goal head-argument construction
% into the clause head and DROPS a preceding `Sub = Term` goal (aliasing Sub to a fresh head var,
% so an echo subterm comes back unbound). Routing construction through a call keeps the subterm bound.
mk_exc(Dom, Conds, drs(Dom, Conds)).

% ==========================================================================================
% Accepted bases — each proven by bases_accept. base/2 keys a mutant to its counterfactual accept.
% ==========================================================================================

base(recommend_rule,       Drs) :- valid_rule(Drs).
base(interval_rule,        Drs) :- valid_interval_rule(Drs).
base(nested_interval_rule, Drs) :- valid_nested_interval_rule(Drs).
base(exception,            Drs) :- valid_exception(Drs).

valid_rule(Drs) :-
    sepsis_guard(P, GDom, GConds), v1_action(P, A), mk_rule(GDom, GConds, should(A), Drs).

valid_interval_rule(Drs) :-
    interval_guard(geq, 18, P, GDom, GConds), v1_action(P, A), mk_rule(GDom, GConds, should(A), Drs).

% the leq/less markers land in a one-level nested sublist (D9); normalize/3 flattens one level.
valid_nested_interval_rule(Drs) :-
    v1_action(P, A),
    mk_rule([P, Ag, Yr, H],
        [ object(P,  patient, countable, na, eq,  1)-1/3,
          object(Ag, age,     countable, na, eq,  1)-1/6,
          [ relation(Ag, of, Yr)-1/7, object(Yr, year, countable, na, leq, 18)-1/11 ],
          predicate(H, have, P, Ag)-1/4 ], should(A), Drs).

% valid_range_rule — a bounded age range (age >= 18 AND age < 65): two well-wired interval components,
% geq top-level + less nested (D9). The exact warning-free product-seam parse (probed 2026-07-15); the
% user decision (accept >=1 well-wired interval, any count) is realized + verified here, not a reject.
valid_range_rule(drs([], [ =>(drs([A, B, C, D, E, F, G],
    [ object(A, patient, countable, na, eq, 1)-1/3,
      object(B, age,     countable, na, eq, 1)-1/6,
      object(C, year,    countable, na, geq, 18)-1/11,
      relation(B, of, C)-1/7,
      predicate(D, have, A, B)-1/4,
      object(E, age,     countable, na, eq, 1)-1/17,
      [ relation(E, of, F)-1/18, object(F, year, countable, na, less, 65)-1/22 ],
      predicate(G, have, A, E)-1/15 ]),
    drs([], [should(drs([H], [predicate(H, take, A, named('Abx-A'))-1/30]))])) ])).

valid_exception(drs([P, Cn, H],
    [ object(P,  patient,                    countable, na, eq, 1)-1/2,
      object(Cn, 'severe-renal-impairment',  countable, na, eq, 1)-1/5,
      predicate(H, have, P, Cn)-1/3 ])).

% ==========================================================================================
% DRS mutants: drs_mutant(?Label, ?Ctx, ?BaseId, ?Mutant, ?RejectArg) — a single-construct-locus edit of
% BaseId's accepted DRS producing reject(RejectArg). The echo-payload rejects pass the exact offending
% subterm (shared with Mutant) so mutants_reject's =@= pins it whole; the referent vars stay free.
% Messages are empty (the message-law mutant is a separate battery_case clause).
% ==========================================================================================

% -- pre-body checks (message / hygiene / named scan / top shape) ---------------------------------------
drs_mutant(undeclared_referent, R, recommend_rule, Mut, not_wellformed) :-
    ctx_recommend(R), mut_undeclared(Mut).
drs_mutant(off_allowlist_drug, R, recommend_rule, Mut, unregistered_named('Zzz')) :-
    ctx_recommend(R), sepsis_guard(P, GD, GC), named_action('Zzz', P, A), mk_rule(GD, GC, should(A), Mut).
drs_mutant(bare_then, R, recommend_rule, Mut, bad_top_shape) :-
    ctx_recommend(R), mut_bare_then(Mut).
drs_mutant(bad_sid, R, recommend_rule, Mut, bad_provenance(2/3)) :-
    ctx_recommend(R), mut_sid2(Mut).

% -- rule body: consequent + op (D1) -------------------------------------------------------------------
drs_mutant(must_consequent, R, recommend_rule, Mut, bad_consequent) :-
    ctx_recommend(R), sepsis_guard(P, GD, GC), v1_action(P, A), mk_rule(GD, GC, must(A), Mut).

% -- guard: interval marker / bound / D9 placement / sublist -------------------------------------------
drs_mutant(exactly_marker, R, interval_rule, Mut, interval_countop(exactly)) :-
    ctx_recommend(R), interval_guard(exactly, 18, P, GD, GC), v1_action(P, A), mk_rule(GD, GC, should(A), Mut).
drs_mutant(negative_bound, R, interval_rule, Mut, interval_bound(-1)) :-
    ctx_recommend(R), interval_guard(geq, -1, P, GD, GC), v1_action(P, A), mk_rule(GD, GC, should(A), Mut).
drs_mutant(leq_top_level, R, interval_rule, Mut, interval_placement(leq)) :-
    ctx_recommend(R), interval_guard(leq, 18, P, GD, GC), v1_action(P, A), mk_rule(GD, GC, should(A), Mut).
drs_mutant(geq_nested, R, nested_interval_rule, Mut, interval_placement(geq)) :-
    ctx_recommend(R), mut_geq_nested(Mut).
drs_mutant(sublist_alien, R, nested_interval_rule, Mut, interval_sublist(Sub)) :-
    ctx_recommend(R), mut_sublist_alien(Mut, Sub).
drs_mutant(deep_sublist, R, nested_interval_rule, Mut, interval_sublist(Sub)) :-
    ctx_recommend(R), mut_deep_sublist(Mut, Sub).
drs_mutant(reversed_sublist, R, nested_interval_rule, Mut, interval_sublist(Sub)) :-
    ctx_recommend(R), mut_reversed_sublist(Mut, Sub).

% -- guard: shape (an alien / disjunctive / in-guard-negated conjunct is a leftover) -------------------
drs_mutant(alien_conjunct, R, recommend_rule, Mut, guard_shape(Conj)) :-
    ctx_recommend(R), mut_guard_alien(Mut, Conj).
drs_mutant(disjunction_conjunct, R, recommend_rule, Mut, guard_shape(Conj)) :-
    ctx_recommend(R), mut_guard_disjunction(Mut, Conj).
drs_mutant(in_guard_negation, R, recommend_rule, Mut, guard_shape(Conj)) :-
    ctx_recommend(R), mut_guard_negation(Mut, Conj).

% -- guard: population + component cardinality ----------------------------------------------------------
drs_mutant(zero_population, R, recommend_rule, Mut, bad_population) :-
    ctx_recommend(R), mut_no_population(Mut).
drs_mutant(two_population, R, recommend_rule, Mut, bad_population) :-
    ctx_recommend(R), mut_two_population(Mut).
drs_mutant(patient_only, R, recommend_rule, Mut, no_guard_component) :-
    ctx_recommend(R), mut_patient_only(Mut).
drs_mutant(aliased_guard_ref, R, recommend_rule, Mut, shared_object_referent) :-
    ctx_recommend(R), mut_shared_guard_ref(Mut).

% -- guard: component wiring (a missing / mis-wired have or of, naming the unwireable object) ----------
drs_mutant(concept_missing_have, R, recommend_rule, Mut, guard_wiring(Obj)) :-
    ctx_recommend(R), mut_concept_no_have(Mut, Obj).
drs_mutant(age_missing_bound, R, interval_rule, Mut, guard_wiring(Obj)) :-
    ctx_recommend(R), mut_age_no_bound(Mut, Obj).
drs_mutant(of_miswired, R, interval_rule, Mut, guard_wiring(Obj)) :-
    ctx_recommend(R), mut_of_miswired(Mut, Obj).
drs_mutant(year_bad_unit, R, interval_rule, Mut, guard_wiring(Obj)) :-
    ctx_recommend(R), mut_year_bad_unit(Mut, Obj).

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
drs_mutant(exception_op, X, exception, Mut, exception_shape(Op)) :-
    ctx_exception(X), mut_exception_op(Mut, Op).
drs_mutant(exception_interval, X, exception, Mut, exception_shape(YObj)) :-
    ctx_exception(X), mut_exception_interval(Mut, YObj).
drs_mutant(exception_no_concept, X, exception, Mut, bad_exception(concept_count)) :-
    ctx_exception(X), mut_exc_no_concept(Mut).
drs_mutant(exception_multi_concept, X, exception, Mut, bad_exception(concept_count)) :-
    ctx_exception(X), mut_exc_multi_concept(Mut).
drs_mutant(exception_two_patient, X, exception, Mut, bad_exception(population)) :-
    ctx_exception(X), mut_exc_two_patient(Mut).
drs_mutant(exception_miswired, X, exception, Mut, bad_exception(wiring)) :-
    ctx_exception(X), mut_exc_miswired(Mut).
drs_mutant(exception_aliased, X, exception, Mut, bad_exception(aliased)) :-
    ctx_exception(X), mut_exc_aliased(Mut).
drs_mutant(exception_extra_have, X, exception, Mut, bad_exception(wiring)) :-
    ctx_exception(X), mut_exc_extra_have(Mut).

% -- mutant term builders (the single-construct-locus edits, spelled out) -------------------------------

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

% bad_provenance: an atomic provenance sentence id != 1 (D2 parses one sentence at a time, so SID = 1).
mut_sid2(Mut) :-
    mk_rule([P, S, H],
        [ object(P, patient, countable, na, eq, 1)-2/3,
          object(S, sepsis,  countable, na, eq, 1)-2/6,
          predicate(H, have, P, S)-2/4 ],
        should(drs([Ev], [predicate(Ev, take, P, named('Abx-A'))-2/14])), Mut).

% interval_placement: a geq bound placed in the leq/less nested sublist (must be top-level, D9).
mut_geq_nested(Mut) :-
    v1_action(P, A),
    mk_rule([P, Ag, Yr, H],
        [ object(P,  patient, countable, na, eq, 1)-1/3,
          object(Ag, age,     countable, na, eq, 1)-1/6,
          [ relation(Ag, of, Yr)-1/7, object(Yr, year, countable, na, geq, 18)-1/11 ],
          predicate(H, have, P, Ag)-1/4 ], should(A), Mut).

% interval_sublist: a non-interval (concept) atom appended to the interval sublist; Sub is the whole
% offending sublist (a valid nested bound is EXACTLY [of-relation, bounded year object]).
mut_sublist_alien(Mut, Sub) :-
    v1_action(P, A),
    Sub = [ relation(Ag, of, Yr)-1/7, object(Yr, year, countable, na, leq, 18)-1/11,
            object(S, sepsis, countable, na, eq, 1)-1/12 ],
    mk_rule([P, Ag, Yr, H, S],
        [ object(P,  patient, countable, na, eq, 1)-1/3,
          object(Ag, age,     countable, na, eq, 1)-1/6,
          Sub,
          predicate(H, have, P, Ag)-1/4 ], should(A), Mut).

% interval_sublist: a deeper list where the bounded year object belongs; Sub is the whole offending sublist.
mut_deep_sublist(Mut, Sub) :-
    v1_action(P, A),
    Sub = [ relation(Ag, of, Yr)-1/7, [ object(Yr, year, countable, na, leq, 18)-1/11 ] ],
    mk_rule([P, Ag, Yr, H],
        [ object(P,  patient, countable, na, eq, 1)-1/3,
          object(Ag, age,     countable, na, eq, 1)-1/6,
          Sub,
          predicate(H, have, P, Ag)-1/4 ], should(A), Mut).

% interval_sublist: the nested bound pair in reversed order (year object before its of-relation); only
% [of-relation, bounded year object] in that order is a valid nested bound. Sub is the whole sublist.
mut_reversed_sublist(Mut, Sub) :-
    v1_action(P, A),
    Sub = [ object(Yr, year, countable, na, leq, 18)-1/11, relation(Ag, of, Yr)-1/7 ],
    mk_rule([P, Ag, Yr, H],
        [ object(P,  patient, countable, na, eq, 1)-1/3,
          object(Ag, age,     countable, na, eq, 1)-1/6,
          Sub,
          predicate(H, have, P, Ag)-1/4 ], should(A), Mut).

% guard_wiring: the year object carries a non-canonical quantisation/unit (not countable/na), so it is not
% recognised as the interval's bounded year and the age component fails to wire; Obj is the age object.
mut_year_bad_unit(Mut, Obj) :-
    v1_action(P, A),
    Obj = object(Ag, age, countable, na, eq, 1)-1/6,
    mk_rule([P, Ag, Yr, H],
        [ object(P, patient, countable, na, eq, 1)-1/3, Obj,
          object(Yr, year, mass, kg, geq, 18)-1/11,
          relation(Ag, of, Yr)-1/7,
          predicate(H, have, P, Ag)-1/4 ], should(A), Mut).

% guard_shape: an alien (non-whitelisted) property conjunct beside a valid sepsis component, so the alien
% is the sole leftover — a single, order-independent guard-shape defect. Conj is the alien conjunct.
mut_guard_alien(Mut, Conj) :-
    v1_action(P, A),
    Conj = property(X, foo, pos)-1/8,
    mk_rule([P, S, H, X],
        [ object(P, patient, countable, na, eq, 1)-1/3,
          object(S, sepsis,  countable, na, eq, 1)-1/6,
          predicate(H, have, P, S)-1/4,
          Conj ], should(A), Mut).

% guard_shape: a disjunctive conjunct v(_,_) (the D4/p7 or-guard hazard) beside a valid sepsis component.
mut_guard_disjunction(Mut, Conj) :-
    v1_action(P, A),
    Conj = v( drs([X], [object(X, pregnancy,                   countable, na, eq, 1)-1/8]),
              drs([Y], [object(Y, 'severe-renal-impairment',   countable, na, eq, 1)-1/9]) ),
    mk_rule([P, S, H],
        [ object(P, patient, countable, na, eq, 1)-1/3,
          object(S, sepsis,  countable, na, eq, 1)-1/6,
          predicate(H, have, P, S)-1/4,
          Conj ], should(A), Mut).

% guard_shape: an in-guard negation -(drs(...)) (the D5/p7 hazard) beside a valid sepsis component.
mut_guard_negation(Mut, Conj) :-
    v1_action(P, A),
    Conj = -( drs([S2, H2], [ object(S2, pregnancy, countable, na, eq, 1)-1/8,
                             predicate(H2, have, P, S2)-1/9 ]) ),
    mk_rule([P, S, H],
        [ object(P, patient, countable, na, eq, 1)-1/3,
          object(S, sepsis,  countable, na, eq, 1)-1/6,
          predicate(H, have, P, S)-1/4,
          Conj ], should(A), Mut).

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

% no_guard_component: a population but no concept / interval component (the reduced patient-only guard).
mut_patient_only(Mut) :-
    v1_action(P, A),
    mk_rule([P], [object(P, patient, countable, na, eq, 1)-1/3], should(A), Mut).

% shared_object_referent: the population and concept collapse to one referent (two object atoms, one
% referent) — is_wellformed proves declarations unique, not object roles, so the guard passes hygiene.
mut_shared_guard_ref(Mut) :-
    v1_action(P, A),
    mk_rule([P, H],
        [ object(P, patient, countable, na, eq, 1)-1/3,
          object(P, sepsis,  countable, na, eq, 1)-1/6,
          predicate(H, have, P, P)-1/4 ], should(A), Mut).

% guard_wiring: a concept object with no have; Obj is the unwireable object.
mut_concept_no_have(Mut, Obj) :-
    v1_action(P, A),
    Obj = object(S, sepsis, countable, na, eq, 1)-1/6,
    mk_rule([P, S], [ object(P, patient, countable, na, eq, 1)-1/3, Obj ], should(A), Mut).

% guard_wiring: an age object with a have but no of-relation / year object bound; Obj is the age object.
mut_age_no_bound(Mut, Obj) :-
    v1_action(P, A),
    Obj = object(Ag, age, countable, na, eq, 1)-1/6,
    mk_rule([P, Ag, H],
        [ object(P, patient, countable, na, eq, 1)-1/3, Obj,
          predicate(H, have, P, Ag)-1/4 ], should(A), Mut).

% guard_wiring: an of-relation linking the age to itself, not to the year object; Obj is the age object.
mut_of_miswired(Mut, Obj) :-
    v1_action(P, A),
    Obj = object(Ag, age, countable, na, eq, 1)-1/6,
    mk_rule([P, Ag, Yr, H],
        [ object(P, patient, countable, na, eq, 1)-1/3, Obj,
          object(Yr, year, countable, na, geq, 18)-1/11,
          relation(Ag, of, Ag)-1/7,
          predicate(H, have, P, Ag)-1/4 ], should(A), Mut).

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

% exception_shape: a modal op inside an exception body; Op is that op condition.
mut_exception_op(Mut, Op) :-
    Op = should(drs([Z], [object(Z, sepsis, countable, na, eq, 1)-1/9])),
    mk_exc([P, Cn, H],
        [ object(P,  patient,                   countable, na, eq, 1)-1/2,
          object(Cn, 'severe-renal-impairment', countable, na, eq, 1)-1/5,
          predicate(H, have, P, Cn)-1/3,
          Op ], Mut).

% exception_shape: an interval object inside an exception body; YObj is that year object.
mut_exception_interval(Mut, YObj) :-
    YObj = object(Yr, year, countable, na, geq, 18)-1/9,
    mk_exc([P, Cn, H, Yr],
        [ object(P,  patient,                   countable, na, eq, 1)-1/2,
          object(Cn, 'severe-renal-impairment', countable, na, eq, 1)-1/5,
          predicate(H, have, P, Cn)-1/3,
          YObj ], Mut).

% bad_exception(concept_count): a patient-only exception body (no concept).
mut_exc_no_concept(drs([P], [ object(P, patient, countable, na, eq, 1)-1/2 ])).

% bad_exception(concept_count): a multi-concept exception body (two concepts).
mut_exc_multi_concept(drs([P, S, G, H1, H2],
    [ object(P, patient,   countable, na, eq, 1)-1/2,
      object(S, sepsis,    countable, na, eq, 1)-1/5,
      object(G, pregnancy, countable, na, eq, 1)-1/6,
      predicate(H1, have, P, S)-1/3,
      predicate(H2, have, P, G)-1/4 ])).

% bad_exception(population): two population objects in an exception body.
mut_exc_two_patient(drs([P1, P2, C, H],
    [ object(P1, patient, countable, na, eq, 1)-1/2,
      object(P2, patient, countable, na, eq, 1)-1/3,
      object(C,  sepsis,  countable, na, eq, 1)-1/5,
      predicate(H, have, P1, C)-1/4 ])).

% bad_exception(wiring): the have links the concept to itself, not the population to the concept.
mut_exc_miswired(drs([P, C, H],
    [ object(P, patient, countable, na, eq, 1)-1/2,
      object(C, sepsis,  countable, na, eq, 1)-1/5,
      predicate(H, have, C, C)-1/4 ])).

% bad_exception(aliased): the population and concept share one referent (two object atoms, one referent).
mut_exc_aliased(drs([P, H],
    [ object(P, patient, countable, na, eq, 1)-1/2,
      object(P, sepsis,  countable, na, eq, 1)-1/5,
      predicate(H, have, P, P)-1/3 ])).

% bad_exception(wiring): a correct have plus an extra mis-wired have (the body must be exactly one
% population object + one concept object + one have).
mut_exc_extra_have(drs([P, C, H1, H2],
    [ object(P,  patient, countable, na, eq, 1)-1/2,
      object(C,  sepsis,  countable, na, eq, 1)-1/5,
      predicate(H1, have, P, C)-1/3,
      predicate(H2, have, P, P)-1/6 ])).

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

% source_reject_ground(-Set) — the sorted GROUND reject(<Term>) arguments in the checker source: the
% closed control + discriminant reasons whose payload carries no referent var (e.g. no_guard_component,
% bad_exception(population)). ground_rejects_exercised pins each to a mutant that observably returns it.
source_reject_ground(Set) :-
    pc_source(File),
    setup_call_cleanup(open(File, read, S), read_ground_rejects(S, Gs), close(S)),
    sort(Gs, Set).

read_ground_rejects(S, Gs) :-
    read_term(S, T, []),
    (   T == end_of_file
    ->  Gs = []
    ;   findall(Arg, ( sub_reject_arg(T, Arg), ground(Arg) ), Here),
        read_ground_rejects(S, More),
        append(Here, More, Gs) ).

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

% -- ground-reason coverage: every GROUND reject reason the source emits (a control atom or discriminant,
% e.g. bad_exception(population)) is pinned by a mutant that observably returns it, closing the gap that
% functor-level banking alone leaves for a per-functor discriminant. --------------------------------------
test(ground_rejects_exercised) :-
    source_reject_ground(Ground),
    findall(RA, battery_case(_, _, _, _, _, _, _, reject(RA)), Pinned),
    findall(G, ( member(G, Ground), \+ ( member(P, Pinned), P == G ) ), Uncovered),
    assertion(Uncovered == []).

% -- anti-vacuity: every mutant's exact base accepts (empty-message accept), so each reject is the mutation's
% doing, not a base that silently stopped accepting. ---------------------------------------------------------
test(bases_accept, [forall(battery_case(Class, Label, Ctx, Base, BMsgs, _, _, _))]) :-
    profile_check(Ctx, Base, BMsgs, Result),
    (   Result == ok
    ->  true
    ;   format(user_error, "~N[profile_battery base] ~w/~w: base did not accept, got ~q~n",
               [Class, Label, Result]), fail ).

% -- the mutation matrix: every mutant rejects with exactly its pinned Construct (functor + ground args + the
% echo-payload subterm, referent identity free), asserted by =@= against a fully-spelled reject term. --------
test(mutants_reject, [forall(battery_case(Class, Label, Ctx, _, _, Mut, MMsgs, Reject))]) :-
    profile_check(Ctx, Mut, MMsgs, Result),
    (   Result =@= Reject
    ->  true
    ;   format(user_error, "~N[profile_battery] ~w/~w: expected ~q, got ~q~n",
               [Class, Label, Reject, Result]), fail ).

% -- the bounded-range accept (user decision 2026-07-15): a >=1 well-wired interval guard of any count is
% v1-admissible; the exact warning-free probed DRS is accepted, not rejected. --------------------------------
test(accept_bounded_range) :-
    ctx_recommend(Ctx), valid_range_rule(Drs),
    profile_check(Ctx, Drs, [], Result),
    assertion(Result == ok).

:- end_tests(profile_battery).
