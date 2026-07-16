% ClinicalCNL conflict-core gate (M3.conflict-core). Hand-oracled over conflict.pl — a pure symbolic
% module (no live APE, like the sibling gates). The battery pairs two rules as a flat multi-document KB
% and asserts the verdict, spanning the two conflict halves: ELIGIBILITY (opposing directions across the
% §L·conflict groups; same action key) and CONTEXT OVERLAP (concept polarity, interval intersection over
% Q incl. the §L·thread age-disjoint control, exception expansion, and a bounded age range). The §8.6
% thread cases reuse the normative kb_examples (doc_a × doc_b, doc_a × control); the rest are built from a
% compact spec via a constructor whose output is proved kb_kernel-valid (anti-vacuity: real compiled KBs).
%
%   Gate: swipl -q -g "consult('clinical/conflict_tests.pl'),(run_tests(conflict)->halt(0);halt(1))" -t 'halt(1)'

:- module(conflict_tests, []).

:- use_module(library(plunit)).
:- use_module(library(lists)).

:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/conflict.pl'], C), use_module(C),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K),
   atomic_list_concat([D, '/goldens/kb_examples.pl'], KE), use_module(KE).

% ---- KB constructors (compact spec -> a kb_kernel-valid flat KB) -----------------------------
% disj_rule_facts(+Doc, +Dir, +Disjuncts, -Facts): a rule (id `Doc.rule.0`) with one statement per
% disjunct, direction Dir, strength strong, population pop.patient, action act.administer:drug.abx_a, and
% one rule source. Disjuncts = [d(Guard, Excs), ...]; Guard = a list of context atoms (concept(_) /
% interval(_,_,_,_)); Excs = a list of concept ids. stmt/bind/exc counters are document-continuous.
disj_rule_facts(Doc, Dir, Disjuncts, Facts) :-
    mkid(Doc, rule, 0, RuleId),
    mk_disjuncts(Disjuncts, Doc, RuleId, 0, 0, 0, DisjFacts, Regions0),
    sort(Regions0, Regions),
    append([ [ direction(RuleId, Dir), strength(RuleId, strong) ],
             DisjFacts,
             [ source(RuleId, Doc, Regions, none) ] ], Facts).

mk_disjuncts([], _, _, _, _, _, [], []).
mk_disjuncts([d(Guard, Excs)|Ds], Doc, RuleId, SC, BC, EC, Facts, [SC|Regions]) :-
    mkid(Doc, stmt, SC, StmtId),
    mk_conditions(Guard, Doc, StmtId, BC, BC1, CondFacts),
    mk_exceptions(Excs, Doc, StmtId, EC, EC1, ExcFacts),
    StmtHead = [ rule(RuleId, StmtId),
                 population(StmtId, 'pop.patient'),
                 action(StmtId, 'act.administer:drug.abx_a') ],
    append([StmtHead, CondFacts, ExcFacts], Emitted),
    SC1 is SC + 1,
    mk_disjuncts(Ds, Doc, RuleId, SC1, BC1, EC1, Rest, Regions),
    append(Emitted, Rest, Facts).

mk_conditions([], _, _, BC, BC, []).
mk_conditions([Atom|As], Doc, StmtId, BC, BCout, [condition(BindId, StmtId, Atom)|Rest]) :-
    mkid(Doc, bind, BC, BindId),
    BC1 is BC + 1,
    mk_conditions(As, Doc, StmtId, BC1, BCout, Rest).

mk_exceptions([], _, _, EC, EC, []).
mk_exceptions([Concept|Cs], Doc, StmtId, EC, ECout, [exception(ExcId, StmtId, concept(Concept))|Rest]) :-
    mkid(Doc, exc, EC, ExcId),
    EC1 is EC + 1,
    mk_exceptions(Cs, Doc, StmtId, EC1, ECout, Rest).

mkid(Doc, Kind, K, Id) :- format(atom(Id), '~w.~w.~w', [Doc, Kind, K]).

% rule_facts(+Doc, +Dir, +Guard, +Excs, -Facts): the single-statement special case.
rule_facts(Doc, Dir, Guard, Excs, Facts) :-
    disj_rule_facts(Doc, Dir, [d(Guard, Excs)], Facts).

% pair_kb(+SpecA, +SpecB, -KB, -RuleA, -RuleB): two rules in distinct documents `ta`/`tb`, flat-unioned.
% Spec = spec(Dir, Guard, Excs) (single statement) | disj(Dir, Disjuncts).
pair_kb(SpecA, SpecB, KB, 'ta.rule.0', 'tb.rule.0') :-
    spec_facts(ta, SpecA, FactsA),
    spec_facts(tb, SpecB, FactsB),
    append(FactsA, FactsB, KB).

spec_facts(Doc, spec(Dir, Guard, Excs), Facts) :- !, rule_facts(Doc, Dir, Guard, Excs, Facts).
spec_facts(Doc, disj(Dir, Disjuncts), Facts)   :- disj_rule_facts(Doc, Dir, Disjuncts, Facts).

% Context-atom shorthands (readability over the age quantity q.age_years).
sepsis(concept('cond.sepsis')).
pregnancy(concept('cond.pregnancy')).
renal(concept('cond.renal_severe')).
geq(N, interval('q.age_years', N, closed, lower)).   % age >= N
gt(N,  interval('q.age_years', N, open,   lower)).   % age >  N
leq(N, interval('q.age_years', N, closed, upper)).   % age =< N
lt(N,  interval('q.age_years', N, open,   upper)).   % age <  N

% All the pair-battery specs (for the anti-vacuity valid-KB meta-test). Each yields a kb_kernel-valid KB.
battery_kb(KB) :- battery_spec(SpecA, SpecB), pair_kb(SpecA, SpecB, KB, _, _).

battery_spec(spec(for, [S], []),            spec(contraindicate, [P], []))                :- sepsis(S), pregnancy(P).
battery_spec(spec(for, [G18], []),          spec(contraindicate, [L18], []))              :- geq(18, G18), lt(18, L18).
battery_spec(spec(for, [S, P], []),         spec(contraindicate, [S], [Pc]))             :- sepsis(S), pregnancy(P), Pc = 'cond.pregnancy'.
battery_spec(spec(for, [S], [Rc]),          spec(contraindicate, [S, P], []))            :- sepsis(S), pregnancy(P), Rc = 'cond.renal_severe'.
battery_spec(spec(for, [S, G18, L65], []),  spec(contraindicate, [S, G40], []))          :- sepsis(S), geq(18, G18), lt(65, L65), geq(40, G40).
battery_spec(spec(for, [S, G18, L40], []),  spec(contraindicate, [S, G50], []))          :- sepsis(S), geq(18, G18), lt(40, L40), geq(50, G50).
battery_spec(spec(contraindicate, [S, G18], []), spec(contraindicate, [S, G18b], []))    :- sepsis(S), geq(18, G18), geq(18, G18b).
battery_spec(disj(for, [d([S, G18], []), d([S, L10], [])]), spec(contraindicate, [S, G18b], [])) :- sepsis(S), geq(18, G18), lt(10, L10), geq(18, G18b).
battery_spec(disj(for, [d([S, L10], []), d([S, L5], [])]),  spec(contraindicate, [S, G18], []))  :- sepsis(S), lt(10, L10), lt(5, L5), geq(18, G18).

:- begin_tests(conflict).

% ---- §8.6 thread (the standing conformance cases, over the normative kb_examples) ------------

% docA (for, sepsis ∧ age>=18, renal exception) × docB (contraindicate, sepsis ∧ age>=18 ∧ pregnancy):
% same action, opposing directions, overlapping context (a sepsis+pregnant adult, docA's renal exception
% not required by docB) -> CONFLICT. The witness is the two rules' sole disjuncts.
test(thread_conflict) :-
    kb_example(doc_a, valid, A), kb_example(doc_b, valid, B), append(A, B, KB),
    RuleA = 'test_source.m1_guideline_a.rule.0', RuleB = 'test_source.m1_guideline_b.rule.0',
    rules_conflict(KB, RuleA, RuleB),
    rules_conflict(KB, RuleB, RuleA),                       % order-independent
    once(conflict_witness(KB, RuleA, RuleB, SA, SB)),
    assertion(SA == 'test_source.m1_guideline_a.stmt.0'),
    assertion(SB == 'test_source.m1_guideline_b.stmt.0'),
    conflict_pairs(KB, Pairs),
    assertion(Pairs == [RuleA-RuleB]).

% docA (for, age>=18) × control (contraindicate, age<18): opposing + same action, but ages DISJOINT
% (age>=18 ∩ age<18 = ∅ over Q) -> NO conflict (the control's documented no-conflict).
test(thread_no_conflict) :-
    kb_example(doc_a, valid, A), kb_example(control, valid, Ctrl), append(A, Ctrl, KB),
    RuleA = 'test_source.m1_guideline_a.rule.0', RuleC = 'test_source.m1_control.rule.0',
    \+ rules_conflict(KB, RuleA, RuleC),
    conflict_pairs(KB, []).

% ---- context overlap: concepts, intervals, exceptions ---------------------------------------

% Disjoint concept requirements (sepsis for vs pregnancy contra), no intervals: a patient can have both
% -> overlap -> CONFLICT (concepts are independent booleans).
test(overlap_concepts) :-
    pair_kb(spec(for, [concept('cond.sepsis')], []),
            spec(contraindicate, [concept('cond.pregnancy')], []), KB, RA, RB),
    rules_conflict(KB, RA, RB).

% Disjoint age intervals (age>=18 for vs age<18 contra) -> NO overlap -> no conflict (isolates the
% interval-disjointness half of the §L·thread control).
test(disjoint_intervals) :-
    pair_kb(spec(for, [interval('q.age_years', 18, closed, lower)], []),
            spec(contraindicate, [interval('q.age_years', 18, open, upper)], []), KB, RA, RB),
    \+ rules_conflict(KB, RA, RB).

% Concept polarity: rule A requires pregnancy (guard), rule B excludes pregnancy (exception) — the
% populations are disjoint (B's exception fires on exactly A's population) -> NO conflict.
test(polarity_block) :-
    pair_kb(spec(for, [concept('cond.sepsis'), concept('cond.pregnancy')], []),
            spec(contraindicate, [concept('cond.sepsis')], ['cond.pregnancy']), KB, RA, RB),
    \+ contexts_overlap(KB, 'ta.stmt.0', 'tb.stmt.0'),
    \+ rules_conflict(KB, RA, RB).

% Exception expansion that does NOT block: rule A's renal exception excludes a concept rule B never
% requires -> the exception is inert to the overlap -> CONFLICT still holds (the docA × docB pattern).
test(exception_survives) :-
    pair_kb(spec(for, [concept('cond.sepsis')], ['cond.renal_severe']),
            spec(contraindicate, [concept('cond.sepsis'), concept('cond.pregnancy')], []), KB, RA, RB),
    contexts_overlap(KB, 'ta.stmt.0', 'tb.stmt.0'),
    rules_conflict(KB, RA, RB).

% ---- bounded age range (v1 per profile-structure: >1 same-quantity interval atom in one guard) -----

% Bounded range [18, 65) for vs age>=40 contra: the guard's two age atoms fold to [18,65), which
% intersects [40,∞) at [40,65) -> overlap -> CONFLICT.
test(bounded_range_overlap) :-
    pair_kb(spec(for, [interval('q.age_years', 18, closed, lower),
                       interval('q.age_years', 65, open, upper)], []),
            spec(contraindicate, [interval('q.age_years', 40, closed, lower)], []), KB, RA, RB),
    rules_conflict(KB, RA, RB).

% Bounded range [18, 40) for vs age>=50 contra: [18,40) ∩ [50,∞) = ∅ -> NO conflict.
test(bounded_range_disjoint) :-
    pair_kb(spec(for, [interval('q.age_years', 18, closed, lower),
                       interval('q.age_years', 40, open, upper)], []),
            spec(contraindicate, [interval('q.age_years', 50, closed, lower)], []), KB, RA, RB),
    \+ rules_conflict(KB, RA, RB).

% ---- eligibility: direction opposition + self-pair --------------------------------------------

% Same direction (both contraindicate) over an overlapping context -> NOT eligible -> no conflict.
test(same_direction_no_conflict) :-
    pair_kb(spec(contraindicate, [concept('cond.sepsis'), interval('q.age_years', 18, closed, lower)], []),
            spec(contraindicate, [concept('cond.sepsis'), interval('q.age_years', 18, closed, lower)], []),
            KB, RA, RB),
    contexts_overlap(KB, 'ta.stmt.0', 'tb.stmt.0'),        % contexts DO overlap
    \+ rules_conflict(KB, RA, RB).                          % but the directions don't oppose

% A rule never conflicts with itself (a disjunction of alternatives, one shared direction).
test(no_self_conflict) :-
    kb_example(doc_a, valid, A), kb_example(doc_b, valid, B), append(A, B, KB),
    \+ rules_conflict(KB, 'test_source.m1_guideline_a.rule.0', 'test_source.m1_guideline_a.rule.0').

% ---- DNF disjunct-pair enumeration ----------------------------------------------------------

% Rule A = (sepsis ∧ age>=18) ∨ (sepsis ∧ age<10); rule B = sepsis ∧ age>=18 contra. The FIRST disjunct
% overlaps B (adult) though the second is disjoint -> ANY overlapping disjunct pair -> CONFLICT.
test(dnf_any_disjunct_overlaps) :-
    pair_kb(disj(for, [ d([concept('cond.sepsis'), interval('q.age_years', 18, closed, lower)], []),
                        d([concept('cond.sepsis'), interval('q.age_years', 10, open, upper)], []) ]),
            spec(contraindicate, [concept('cond.sepsis'), interval('q.age_years', 18, closed, lower)], []),
            KB, RA, RB),
    rules_conflict(KB, RA, RB),
    once(conflict_witness(KB, RA, RB, 'ta.stmt.0', 'tb.stmt.0')).   % the overlapping disjunct is stmt.0

% Rule A = (age<10) ∨ (age<5); rule B = age>=18 contra. NO disjunct pair overlaps -> no conflict.
test(dnf_no_disjunct_overlaps) :-
    pair_kb(disj(for, [ d([concept('cond.sepsis'), interval('q.age_years', 10, open, upper)], []),
                        d([concept('cond.sepsis'), interval('q.age_years', 5, open, upper)], []) ]),
            spec(contraindicate, [concept('cond.sepsis'), interval('q.age_years', 18, closed, lower)], []),
            KB, RA, RB),
    \+ rules_conflict(KB, RA, RB).

% ---- multi-document flat KB (the conflict operating mode) ------------------------------------

% Three documents flat-unioned: doc_a × doc_b conflict; the control conflicts with neither (age
% disjoint from doc_a, and doc_b is adult too — control is child) -> exactly ONE pair, canonical order.
test(conflict_pairs_multi) :-
    kb_example(doc_a, valid, A), kb_example(doc_b, valid, B), kb_example(control, valid, Ctrl),
    append([A, B, Ctrl], KB),
    conflict_pairs(KB, Pairs),
    assertion(Pairs == ['test_source.m1_guideline_a.rule.0'-'test_source.m1_guideline_b.rule.0']).

% ---- direct relations: opposing_directions, same_action -------------------------------------

% opposing_directions/2 over the §L·conflict groups: every (positive × non-positive) pair opposes (both
% orders), and no (positive × positive) or (non-positive × non-positive) pair does. `avoid` is
% non-positive. Differential over an independent restatement of the three groups.
test(opposing_directions_relation) :-
    Positive = [for, require, permit],
    NonPositive = [against, avoid, contraindicate],
    forall( ( member(P, Positive), member(N, NonPositive) ),
            ( opposing_directions(P, N), opposing_directions(N, P) ) ),
    forall( ( member(X, Positive), member(Y, Positive) ),
            \+ opposing_directions(X, Y) ),
    forall( ( member(X, NonPositive), member(Y, NonPositive) ),
            \+ opposing_directions(X, Y) ).

% same_action/3: identical action keys match, distinct keys do not (the eligibility action half; v1
% valid KBs only ever carry act.administer:drug.abx_a, so this exercises the relation directly).
test(same_action_relation) :-
    Facts = [ action(s1, 'act.administer:drug.abx_a'),
              action(s2, 'act.administer:drug.abx_a'),
              action(s3, 'other.kind:other.target') ],
    same_action(Facts, s1, s2),
    \+ same_action(Facts, s1, s3).

% ---- anti-vacuity: every constructed pair KB is kb_kernel-valid ------------------------------
% The battery's fixtures are REAL compiled KBs, so a construction bug (a bad id, a missing binder, a
% dangling ref) fails here rather than silently weakening a conflict assertion.
test(battery_fixtures_valid) :-
    forall( battery_kb(KB),
            ( kb_errors(KB, Errors),
              ( Errors == []
              -> true
              ;  format(user_error, "conflict: fixture invalid ~w~n", [Errors]), fail ) ) ),
    % the thread fixtures too
    forall( member(Name, [doc_a, doc_b, control]),
            ( kb_example(Name, valid, F), valid_kb(F) ) ).

:- end_tests(conflict).
