% ClinicalCNL DRS -> KB term mapper (M3.map-core; SPEC §10.6, §5 domain IR, clinical/KB.md). The
% third pipeline stage after the raw gate (pre-APE whitelist) and profile_check (post-APE structural
% whitelist): a validated v1 rule DRS -> the rules-as-data KB fact terms. Exception-free — a labeled
% exception block is map-exc's concern; map-core maps a rule and its disjuncts only.
%
% profile_check has already proven the DRS is a canonical v1 shape, so the mapper ASSUMES validity
% and EXTRACTS (no re-validation, no reject path): it inverts the registry surfaces back to ids
% (registry.pl is bidirectional), reads the D1 direction/strength off the raw-header keyword, the
% D9/D10 interval bound off the year object's CountOp (openness + direction, open/closed distinct),
% and groups a rule's disjunct sentences (D4) statement-major under one rule id.
%
% Ids are document-continuous (KB.md): the stmt / bind counters thread through a base/2 term the
% caller (map-emit) advances across rule + exception blocks; the rule counter is the raw block
% ordinal. map_rule/6 maps one rule block; map-emit is the whole-document driver (counter threading,
% referent canonicalization, kb-writer bytes, determinism gate). Output = kb_kernel-valid TERMS; the
% bytes are kb-writer's (map-emit) — a KB is a fact SET, so emit order is free (kb_bytes/2 sorts).
%
% The gate (drs_map_tests.pl) reconstructs the DRS from the byte-pinned surface goldens via
% read_term_from_atom (the profile-drs read-back pattern) and hand-oracles the KB terms, so map-core
% runs pure and fast with no live APE.
%
%   Gate: swipl -q -g "consult('clinical/drs_map_tests.pl'),(run_tests(drs_map)->halt(0);halt(1))" -t 'halt(1)'

:- module(drs_map, [ map_rule/6 ]).

% kb_kernel = the KB contract (action_key/3 to join a key); registry = the id<->surface bridge
% (inverted here). Loaded source-relative so the module is cwd-independent (mirrors profile_check.pl).
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K),
   atomic_list_concat([D, '/registry.pl'], R), use_module(R).

%% map_rule(+DocId, +Header, +Disjuncts, +Base0, -Facts, -Base1) is det.
% Map one rule block to its KB facts. DocId = the document id atom; Header =
% rule_header(RuleOrd, Keyword, Cert, Basis) — RuleOrd the raw block ordinal (-> rule.RuleOrd),
% Keyword the D1 modality keyword, Cert a certainty token or the atom `none` (absent), Basis the
% evidence string or the atom `none`. Disjuncts = [disj(SentIdx, Drs), ...] in disjunct order (D4);
% each maps to one statement. Base0 / Base1 = base(StmtIdx, BindIdx), the document-continuous
% counters in / out. Facts = the rule's KB terms (rule/direction/strength/[certainty]/population/
% condition/action + one rule-level source), EXCLUDING exceptions. Emit order is unconstrained.
map_rule(DocId, rule_header(RuleOrd, Keyword, Cert, Basis), Disjuncts, Base0, Facts, Base1) :-
    mk_id(DocId, rule, RuleOrd, RuleId),
    reg_keyword(Keyword, _Op, Direction, Strength),
    header_facts(RuleId, Direction, Strength, Cert, HeaderFacts),
    map_disjuncts(Disjuncts, DocId, RuleId, Base0, StmtFacts, Base1, SentIdxs),
    sort(SentIdxs, Regions),
    append([HeaderFacts, StmtFacts, [source(RuleId, DocId, Regions, Basis)]], Facts).

% header_facts(+RuleId, +Dir, +Str, +Cert, -Facts) — the rule-level deontic facts; certainty is a
% D7 header field, optional (the atom `none` = absent).
header_facts(RuleId, Dir, Str, none, [direction(RuleId, Dir), strength(RuleId, Str)]) :- !.
header_facts(RuleId, Dir, Str, Cert, [direction(RuleId, Dir), strength(RuleId, Str), certainty(RuleId, Cert)]).

% map_disjuncts(+Disjuncts, +DocId, +RuleId, +Base0, -Facts, -Base1, -SentIdxs) — fold the base/2
% counters through the disjuncts statement-major (D4): each disjunct is one statement stmt.k under
% RuleId, its guard conditions taking the continuing bind counter. SentIdxs = each disjunct's raw
% sentence index (map_rule sorts them into the rule's provenance regions).
map_disjuncts([], _DocId, _RuleId, Base, [], Base, []).
map_disjuncts([disj(SentIdx, Drs)|Ds], DocId, RuleId, base(S0, B0), Facts, Base, [SentIdx|Idxs]) :-
    map_disjunct(Drs, DocId, RuleId, S0, B0, F0, B1),
    S1 is S0 + 1,
    map_disjuncts(Ds, DocId, RuleId, base(S1, B1), F1, Base, Idxs),
    append(F0, F1, Facts).

% map_disjunct(+Drs, +DocId, +RuleId, +StmtIdx, +BindIdx0, -Facts, -BindIdx1) — one rule disjunct ->
% one statement: rule(RuleId, StmtId), population, the guard conditions (bind counter BindIdx0 ..
% BindIdx1), and the action. The DRS is profile-validated, so the head pins the canonical rule shape.
map_disjunct(drs([], [ =>(drs(_, GConds), drs([], [OpCond])) ]), DocId, RuleId, StmtIdx, B0, Facts, B1) :-
    mk_id(DocId, stmt, StmtIdx, StmtId),
    flatten_intervals(GConds, Flat),
    population_id(Flat, PopId),
    map_conditions(Flat, Flat, DocId, StmtId, B0, CondFacts, B1),
    map_action(OpCond, StmtId, ActionFact),
    append([ [rule(RuleId, StmtId), population(StmtId, PopId)], CondFacts, [ActionFact] ], Facts).

% ==========================================================================================
% Guard -> population + context atoms.
% ==========================================================================================

% flatten_intervals(+Conds, -Flat) — lift each one-level interval sublist ([of-relation, bounded
% year object] for a nested leq/less marker, D9) inline, so the anchor walk sees a flat conjunct list
% whatever the marker's placement. A non-list conjunct passes through.
flatten_intervals([], []).
flatten_intervals([C|Cs], Flat) :-
    ( is_list(C) -> append(C, Rest, Flat) ; Flat = [C|Rest] ),
    flatten_intervals(Cs, Rest).

% population_id(+Conds, -PopId) — the sole population object's noun, inverted to its population id.
% profile_check proved exactly one population object, so the first match is it.
population_id([object(_, N, countable, na, eq, 1)-_|_], PopId) :- reg_population(PopId, N), !.
population_id([_|Cs], PopId) :- population_id(Cs, PopId).

% map_conditions(+Walk, +All, +DocId, +StmtId, +BindIdx0, -Conds, -BindIdx1) — walk the flat guard
% conjuncts in surface order; each concept / age anchor object emits one condition(bind.k, StmtId,
% Atom) taking the next bind id. All is the full flat list (companion look-up for an interval's
% of-relation + year object). Non-anchor conjuncts (the population object, a year object, of-relations,
% have predicates) pass through — their content is carried by the anchor they belong to.
map_conditions([], _All, _DocId, _StmtId, B, [], B).
map_conditions([C|Cs], All, DocId, StmtId, B0, Conds, B) :-
    (   guard_atom(C, All, Atom)
    ->  mk_id(DocId, bind, B0, BindId),
        B1 is B0 + 1,
        Conds = [condition(BindId, StmtId, Atom)|Rest],
        map_conditions(Cs, All, DocId, StmtId, B1, Rest, B)
    ;   map_conditions(Cs, All, DocId, StmtId, B0, Conds, B)
    ).

% guard_atom(+Conjunct, +All, -Atom) — an anchor object's KB context atom, or fail (a non-anchor).
% A concept object inverts its noun to concept(Id); an age object gathers its of-related year
% object's CountOp into a bounded interval(Q, N, Openness, Dir) (the CountOp fixes the bound; D9
% placement is irrelevant post-flatten). Companions matched by referent identity (==).
guard_atom(object(_, N, countable, na, eq, 1)-_, _All, concept(CId)) :-
    reg_concept(CId, N), !.
guard_atom(object(AgeRef, N, countable, na, eq, 1)-_, All, interval(QId, Num, Openness, Dir)) :-
    reg_quantity(QId, N, _, _, _), !,
    same_ref_rel(AgeRef, YearRef, All),
    same_ref_year(YearRef, CountOp, Num, All),
    countop_bound(CountOp, Openness, Dir).

% same_ref_rel(+AgeRef, -YearRef, +Conds) — the of-relation whose subject is AgeRef (by identity),
% binding YearRef. same_ref_year(+YearRef, -CountOp, -Num, +Conds) — the bounded year object with
% referent YearRef. Identity (==) matching, not unification, so a second interval's companions
% (a bounded age range) never bind through (the Prolog term-walk false-positive lesson).
same_ref_rel(AgeRef, YearRef, [relation(A, of, Y)-_|_]) :- A == AgeRef, !, YearRef = Y.
same_ref_rel(AgeRef, YearRef, [_|Cs]) :- same_ref_rel(AgeRef, YearRef, Cs).

same_ref_year(YearRef, CountOp, Num, [object(Y, _, countable, na, CountOp, Num)-_|_]) :- Y == YearRef, !.
same_ref_year(YearRef, CountOp, Num, [_|Cs]) :- same_ref_year(YearRef, CountOp, Num, Cs).

% countop_bound(?CountOp, ?Openness, ?Dir) — the DRS interval CountOp -> KB (openness, direction)
% (KB.md; D10 open/closed distinction). The four v1 single-bound markers only (profile_check already
% rejected exactly / bare eq).
countop_bound(geq,     closed, lower).
countop_bound(greater, open,   lower).
countop_bound(leq,     closed, upper).
countop_bound(less,    open,   upper).

% ==========================================================================================
% Consequent -> action.
% ==========================================================================================

% map_action(+OpCond, +StmtId, -ActionFact) — the consequent's action DRS -> action(StmtId, Key).
% The op token (should/may/-should/-can) is direction, already carried by the keyword, so map-core
% needs only the inner action DRS: the verb lemma -> action kind, the named() proper name -> drug
% target, joined into the <kind>:<target> key (kb_kernel:action_key/3).
map_action(OpCond, StmtId, action(StmtId, Key)) :-
    consequent_action(OpCond, ActionDrs),
    ActionDrs = drs([Act], [predicate(Act, Verb, _Subj, named(Drug))-_]),
    reg_action(KindId, _, _, Verb),
    reg_drug(TargetId, Drug),
    action_key(Key, KindId, TargetId).

% consequent_action(+OpCond, -ActionDrs) — unwrap the modality op to its action DRS (D1 frames).
consequent_action(should(A), A).
consequent_action(may(A), A).
consequent_action(-(drs([], [should(A)])), A).
consequent_action(-(drs([], [can(A)])), A).

% mk_id(+Doc, +Kind, +N, -Id) — a document-qualified id <Doc>.<Kind>.<N> (kb_kernel:valid_id/2).
mk_id(Doc, Kind, N, Id) :- atomic_list_concat([Doc, '.', Kind, '.', N], Id).
