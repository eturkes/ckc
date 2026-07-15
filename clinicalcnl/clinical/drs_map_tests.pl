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
   atomic_list_concat([D, '/registry.pl'], RG), use_module(RG),
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

% same_kb(+Facts, +Expected) — order-insensitive, duplicate-SENSITIVE multiset equality (msort keeps
% duplicates → stricter than set equality: a stray duplicate fact, e.g. a second source, fails; a KB
% is a set, so kb_bytes/2 likewise sorts the emitted facts).
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

% --- Bounded age range: more than one well-wired interval component in a single guard (the profile
%     admits it — user decision 2026-07-15). Pins the two referent-IDENTITY (==) companion matches,
%     each with its OWN failure mode under a unifying walk: same_ref_rel binds interval #2's of-relation
%     by AgeRef identity — unify instead and #2 binds through #1's relation, so both conditions read
%     18/closed/lower (only this two-interval test catches that); same_ref_year binds the bounded year
%     object by YearRef identity — unify instead and it aliases the first (population) object → eq →
%     countop_bound fails → extraction collapses (zero conditions, base(1,0)), already caught by every
%     single-interval marker/thread test. The DRS is a synthetic-but-shape-faithful
%     profile-valid guard (geq top-level + less nested, D9) mirroring the memory-verified warning-free
%     product-seam parse; a live-APE golden for the multi-interval case is surface-goldens' to make. ---

test(bounded_range) :-
    Drs = drs([], [ =>(drs([A, B, C, D, E, F, G],
                          [ object(A, patient, countable, na, eq, 1)-1/3,
                            object(B, age, countable, na, eq, 1)-1/6,
                            object(C, year, countable, na, geq, 18)-1/11,
                            relation(B, of, C)-1/7,
                            predicate(D, have, A, B)-1/4,
                            object(E, age, countable, na, eq, 1)-1/17,
                            [ relation(E, of, F)-1/18, object(F, year, countable, na, less, 65)-1/22 ],
                            predicate(G, have, A, E)-1/15 ]),
                       drs([], [ should(drs([H], [predicate(H, take, A, named('Abx-A'))-1/29])) ])) ]),
    map_rule('t.br', rule_header(0, recommend, none, none), [disj(0, Drs)], base(0, 0), Facts, Base),
    Base == base(1, 2),
    valid_kb(Facts),
    same_kb(Facts,
      [ rule('t.br.rule.0', 't.br.stmt.0'),
        direction('t.br.rule.0', for),
        strength('t.br.rule.0', strong),
        population('t.br.stmt.0', 'pop.patient'),
        condition('t.br.bind.0', 't.br.stmt.0', interval('q.age_years', 18, closed, lower)),
        condition('t.br.bind.1', 't.br.stmt.0', interval('q.age_years', 65, open, upper)),
        action('t.br.stmt.0', 'act.administer:drug.abx_a'),
        source('t.br.rule.0', 't.br', [0], none) ]).

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

% --- Cross-rule counter threading (the map-emit interface): map_rule threads base(StmtIdx, BindIdx)
%     and the raw rule ordinal across consecutive blocks. Map a first block from base(0,0), then a
%     SECOND block from the returned Base1 with a NONZERO rule ordinal + a nonzero raw sentence index,
%     pinning the second block's continued ids / region / basis. Guards a per-block counter reset. ---

test(cross_rule_threading) :-
    golden_drs(thread_doc_a, D1),
    golden_drs(frame_recommend, D2),
    map_rule('t.doc', rule_header(0, recommend, none, none), [disj(0, D1)], base(0, 0), F1, Base1),
    Base1 == base(1, 2),
    valid_kb(F1),
    map_rule('t.doc', rule_header(1, suggest, none, "second block"), [disj(3, D2)], Base1, F2, Base2),
    Base2 == base(2, 3),
    valid_kb(F2),
    same_kb(F2,
      [ rule('t.doc.rule.1', 't.doc.stmt.1'),
        direction('t.doc.rule.1', for),
        strength('t.doc.rule.1', weak),
        population('t.doc.stmt.1', 'pop.patient'),
        condition('t.doc.bind.2', 't.doc.stmt.1', concept('cond.sepsis')),
        action('t.doc.stmt.1', 'act.administer:drug.abx_a'),
        source('t.doc.rule.1', 't.doc', [3], "second block") ]).

% --- All six D1 modality keywords -> direction/strength, across all four consequent frames (should /
%     may / -should / -can via consequent_action). recommend + contraindicate are exercised above;
%     this closes the may / -should action-unwrap branches and the four remaining keyword mappings. ---

test(all_keywords) :-
    forall(
        member(kw(Keyword, Frame, Dir, Str),
               [ kw(recommend,       frame_recommend,     for,            strong),
                 kw(suggest,         frame_recommend,     for,            weak),
                 kw('may-consider',  frame_admissible,    permit,         weak),
                 kw('not-recommend', frame_not_recommend, against,        strong),
                 kw('not-suggest',   frame_not_recommend, against,        weak),
                 kw(contraindicate,  frame_not_possible,  contraindicate, strong) ]),
        ( map_one('t.kw', Keyword, none, none, [Frame], Facts, _),
          valid_kb(Facts),
          memberchk(direction('t.kw.rule.0', Dir), Facts),
          memberchk(strength('t.kw.rule.0', Str), Facts),
          memberchk(action('t.kw.stmt.0', 'act.administer:drug.abx_a'), Facts)
        )
    ).

% --- Action-lemma uniqueness: registry.pl delegates inverse-key uniqueness to map-core (the DRS-only
%     lemma slots sit outside its surface-duplicate check). map_action inverts reg_action on the verb
%     lemma, so a duplicate lemma would make it nondeterministic — gate the invariant here. ---

test(action_lemma_unique) :-
    findall(L, reg_action(_, _, _, L), Ls),
    sort(Ls, Unique),
    length(Ls, N),
    length(Unique, N).

:- end_tests(drs_map).
