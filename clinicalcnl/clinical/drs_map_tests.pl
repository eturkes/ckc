% ClinicalCNL DRS -> KB mapper gate (M3.map-core). Hand-oracles map_rule/6's KB terms over the §8.6
% thread rules (docA exception-free, docB, control), the four v1 interval markers (each CountOp ->
% its openness/direction), and a synthetic two-disjunct rule (D4 statement-major grouping +
% document-continuous stmt/bind counter threading). Each rule DRS is reconstructed from the
% byte-pinned surface goldens via read_term_from_atom (the profile-drs read-back pattern), so the
% gate runs pure and fast with no live APE. Every mapped set is also asserted kb_kernel-valid.
%
%   Gate: swipl -q -g "consult('clinical/drs_map_tests.pl'),(run_tests(drs_map)->halt(0);halt(1))" -t 'halt(1)'

:- module(drs_map_tests, []).

:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/drs_map.pl'], M), use_module(M),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K),
   atomic_list_concat([D, '/goldens/surface_expected.pl'], G), use_module(G).

% golden_drs(+Id, -Drs) — the surface golden's serialized DRS read back to a term (fresh referent
% vars, with in-term sharing preserved by same-named uppercase variables).
golden_drs(Id, Drs) :-
    surface_expected(Id, 'text/plain', Atom, _),
    read_term_from_atom(Atom, Drs, []).

% map_one(+DocId, +Keyword, +Cert, +Basis, +GoldenIds, -Facts, -Base1) — map one rule whose
% disjuncts are the listed goldens (list position = raw sentence index), from base(0,0).
map_one(DocId, Keyword, Cert, Basis, GoldenIds, Facts, Base1) :-
    findall(disj(Idx, Drs), ( nth0(Idx, GoldenIds, Gid), golden_drs(Gid, Drs) ), Disjuncts),
    map_rule(DocId, rule_header(0, Keyword, Cert, Basis), Disjuncts, base(0, 0), Facts, Base1).

% same_kb(+Facts, +Expected) — order-free fact-set equality (a KB is a set; kb_bytes/2 sorts).
same_kb(Facts, Expected) :- msort(Facts, M), msort(Expected, M).

% conditions(+Facts, -Atoms) — the context atoms, in emit order.
conditions(Facts, Atoms) :- findall(A, member(condition(_, _, A), Facts), Atoms).

:- begin_tests(drs_map).

% --- Interval markers: each CountOp -> its KB (openness, direction), one condition, valid KB. -----

test(marker_at_least) :-
    map_one('t.iv', recommend, none, none, [iv_at_least], Facts, Base),
    Base == base(1, 1),
    valid_kb(Facts),
    same_kb(Facts,
      [ rule('t.iv.rule.0', 't.iv.stmt.0'),
        direction('t.iv.rule.0', for),
        strength('t.iv.rule.0', strong),
        population('t.iv.stmt.0', 'pop.patient'),
        condition('t.iv.bind.0', 't.iv.stmt.0', interval('q.age_years', 18, closed, lower)),
        action('t.iv.stmt.0', 'act.administer:drug.abx_a'),
        source('t.iv.rule.0', 't.iv', [0], none) ]).

test(marker_more_than) :-
    map_one('t.iv', recommend, none, none, [iv_more_than], Facts, _),
    valid_kb(Facts),
    conditions(Facts, [interval('q.age_years', 18, open, lower)]).

test(marker_at_most) :-
    map_one('t.iv', recommend, none, none, [iv_at_most], Facts, _),
    valid_kb(Facts),
    conditions(Facts, [interval('q.age_years', 18, closed, upper)]).

test(marker_less_than) :-
    map_one('t.iv', recommend, none, none, [iv_less_than], Facts, _),
    valid_kb(Facts),
    conditions(Facts, [interval('q.age_years', 18, open, upper)]).

% --- §8.6 thread rules (exception-free): concepts + intervals + direction/strength + provenance. ---

test(thread_doc_a) :-
    map_one('test_source.m1_guideline_a', recommend, none, "guideline A sepsis recommendation",
            [thread_doc_a], Facts, Base),
    Base == base(1, 2),
    valid_kb(Facts),
    same_kb(Facts,
      [ rule('test_source.m1_guideline_a.rule.0', 'test_source.m1_guideline_a.stmt.0'),
        direction('test_source.m1_guideline_a.rule.0', for),
        strength('test_source.m1_guideline_a.rule.0', strong),
        population('test_source.m1_guideline_a.stmt.0', 'pop.patient'),
        condition('test_source.m1_guideline_a.bind.0', 'test_source.m1_guideline_a.stmt.0', concept('cond.sepsis')),
        condition('test_source.m1_guideline_a.bind.1', 'test_source.m1_guideline_a.stmt.0', interval('q.age_years', 18, closed, lower)),
        action('test_source.m1_guideline_a.stmt.0', 'act.administer:drug.abx_a'),
        source('test_source.m1_guideline_a.rule.0', 'test_source.m1_guideline_a', [0], "guideline A sepsis recommendation") ]).

test(thread_doc_b) :-
    map_one('test_source.m1_guideline_b', contraindicate, none, "guideline B pregnancy contraindication",
            [thread_doc_b], Facts, Base),
    Base == base(1, 3),
    valid_kb(Facts),
    same_kb(Facts,
      [ rule('test_source.m1_guideline_b.rule.0', 'test_source.m1_guideline_b.stmt.0'),
        direction('test_source.m1_guideline_b.rule.0', contraindicate),
        strength('test_source.m1_guideline_b.rule.0', strong),
        population('test_source.m1_guideline_b.stmt.0', 'pop.patient'),
        condition('test_source.m1_guideline_b.bind.0', 'test_source.m1_guideline_b.stmt.0', concept('cond.sepsis')),
        condition('test_source.m1_guideline_b.bind.1', 'test_source.m1_guideline_b.stmt.0', interval('q.age_years', 18, closed, lower)),
        condition('test_source.m1_guideline_b.bind.2', 'test_source.m1_guideline_b.stmt.0', concept('cond.pregnancy')),
        action('test_source.m1_guideline_b.stmt.0', 'act.administer:drug.abx_a'),
        source('test_source.m1_guideline_b.rule.0', 'test_source.m1_guideline_b', [0], "guideline B pregnancy contraindication") ]).

test(thread_control) :-
    map_one('test_source.m1_control', contraindicate, none, "control pediatric contraindication",
            [thread_control], Facts, Base),
    Base == base(1, 2),
    valid_kb(Facts),
    same_kb(Facts,
      [ rule('test_source.m1_control.rule.0', 'test_source.m1_control.stmt.0'),
        direction('test_source.m1_control.rule.0', contraindicate),
        strength('test_source.m1_control.rule.0', strong),
        population('test_source.m1_control.stmt.0', 'pop.patient'),
        condition('test_source.m1_control.bind.0', 'test_source.m1_control.stmt.0', concept('cond.sepsis')),
        condition('test_source.m1_control.bind.1', 'test_source.m1_control.stmt.0', interval('q.age_years', 18, open, upper)),
        action('test_source.m1_control.stmt.0', 'act.administer:drug.abx_a'),
        source('test_source.m1_control.rule.0', 'test_source.m1_control', [0], "control pediatric contraindication") ]).

% --- Two-disjunct rule (D4): stmt-major grouping under one rule id, document-continuous bind counter
%     across statements, optional certainty, and regions spanning both disjunct sentences. ---------

test(two_disjunct) :-
    map_one('test_source.map_two', recommend, moderate, "two-disjunct rule",
            [thread_doc_a, frame_recommend], Facts, Base),
    Base == base(2, 3),
    valid_kb(Facts),
    same_kb(Facts,
      [ rule('test_source.map_two.rule.0', 'test_source.map_two.stmt.0'),
        rule('test_source.map_two.rule.0', 'test_source.map_two.stmt.1'),
        direction('test_source.map_two.rule.0', for),
        strength('test_source.map_two.rule.0', strong),
        certainty('test_source.map_two.rule.0', moderate),
        population('test_source.map_two.stmt.0', 'pop.patient'),
        condition('test_source.map_two.bind.0', 'test_source.map_two.stmt.0', concept('cond.sepsis')),
        condition('test_source.map_two.bind.1', 'test_source.map_two.stmt.0', interval('q.age_years', 18, closed, lower)),
        action('test_source.map_two.stmt.0', 'act.administer:drug.abx_a'),
        population('test_source.map_two.stmt.1', 'pop.patient'),
        condition('test_source.map_two.bind.2', 'test_source.map_two.stmt.1', concept('cond.sepsis')),
        action('test_source.map_two.stmt.1', 'act.administer:drug.abx_a'),
        source('test_source.map_two.rule.0', 'test_source.map_two', [0, 1], "two-disjunct rule") ]).

:- end_tests(drs_map).
