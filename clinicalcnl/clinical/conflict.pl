% ClinicalCNL conflict core (M3.conflict-core; SPEC §6 conflict / LP profile, §L·conflict, D10). The
% symbolic conflict-detection stage over a compiled KB fact list (kb_kernel.pl / clinical/KB.md): it
% decides whether two rules issue directionally-opposed advice on the SAME action for an OVERLAPPING
% patient context — a semantic contradiction — WITHOUT patient-evaluating (derivable/3 stays a
% finite-fixture reference behind §15 G-RULE-EVAL). map-* produce the KB; conflict-verdict wraps this
% core's witness into the {category, kind, participating_rules, evidence, lane} record.
%
% A conflict has two independent halves (§L·conflict):
%   ELIGIBILITY     — the two rules share one normalized action key AND their directions OPPOSE: one in
%                     the `positive` group {for, require, permit}, the other in `against` {against,
%                     avoid} or `contraindicating` {contraindicate, avoid} (kb_kernel:direction_group/2).
%                     Same direction, or positive-vs-positive / against-vs-contraindicating, is no conflict.
%   CONTEXT OVERLAP — some disjunct (statement) of each rule is JOINTLY satisfiable by one patient once
%                     exceptions are expanded (SPEC §6: the SMT lane reads an exception as a negated
%                     context conjunct). Decided symbolically, no patient search:
%                       * concept polarity — no concept is required present by one guard yet excluded by
%                         either statement's labeled exception (exceptions join their statement as negated
%                         concepts). A concept both required and excluded ⇒ that disjunct pair is disjoint.
%                       * interval intersection — every constrained quantity's combined bounds hold a
%                         point over Q (intervals.pl; open/closed + dense order carry, D10). A guard may
%                         carry >1 same-quantity interval atom (a bounded age range, v1 per
%                         profile-structure); its atoms fold to an effective range first (bounds_range/2),
%                         then the two guards' ranges intersect (ranges_overlap/2). Disjoint ages ⇒ no
%                         overlap (the §L·thread control: adult age>=18 vs child age<18).
%
% DNF disjunct-pair enumeration: a rule is a disjunction of its statements, so every (statement of A,
% statement of B) pair is tried; the rules conflict if ANY eligible pair overlaps. Concepts are
% independent boolean features, orthogonal to the age quantity, so satisfiability of a disjunct pair is
% exactly (no concept polarity clash) ∧ (every constrained quantity's range non-empty) — a decision, not
% a search. conflict-core ASSUMES a kb_kernel-valid KB (raw gate → profile → map guarantee it) and reads
% it; it never re-validates. Pure over the fact list (never asserted, no live APE), like the sibling gates.
%
%   Gate: swipl -q -g "consult('clinical/conflict_tests.pl'),(run_tests(conflict)->halt(0);halt(1))" -t 'halt(1)'

:- module(conflict,
          [ rules_conflict/3,       % +Facts, ?RuleA, ?RuleB           — the two rules semantically contradict (check)
            conflict_witness/5,     % +Facts, ?RuleA, ?RuleB, -SA, -SB — an eligible overlapping disjunct pair (witness)
            conflict_pairs/2,       % +Facts, -Pairs                   — all conflicting rule pairs (RuleA @< RuleB), sorted
            contexts_overlap/3,     % +Facts, +StmtA, +StmtB           — guards + exceptions jointly satisfiable
            opposing_directions/2,  % ?DirA, ?DirB                     — one positive, the other against/contraindicating
            same_action/3           % +Facts, +StmtA, +StmtB           — identical normalized action key
          ]).

% kb_kernel = the §L·conflict direction-group relation (the eligibility half); intervals = the
% exact-rational bound algebra (the interval-overlap half). Loaded source-relative so the module is
% cwd-independent (mirrors drs_map.pl). registry is NOT needed — conflict-core reads KB ids, not surfaces.
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K),
   atomic_list_concat([D, '/intervals.pl'], I), use_module(I).

% ==========================================================================================
% Top-level: rule-pair conflict + witnesses.
% ==========================================================================================

%% rules_conflict(+Facts, ?RuleA, ?RuleB) is semidet.
% RuleA and RuleB are two DISTINCT rules in the KB that issue directionally-opposed advice on the same
% action for an overlapping patient context (a semantic contradiction, SPEC §6). Order-independent in
% RuleA/RuleB; semidet — one witness suffices to declare the conflict. With the rules unbound it binds
% the first conflicting pair; enumerate every pair with conflict_pairs/2.
rules_conflict(Facts, RuleA, RuleB) :-
    once(conflict_witness(Facts, RuleA, RuleB, _StmtA, _StmtB)).

%% conflict_witness(+Facts, ?RuleA, ?RuleB, -StmtA, -StmtB) is nondet.
% A witness for a conflict between RuleA and RuleB: distinct, opposing-directioned rules with an
% ELIGIBLE (same action key) disjunct pair — StmtA a disjunct of RuleA, StmtB a disjunct of RuleB —
% whose contexts OVERLAP. Enumerates every such disjunct pair (the verdict layer reads the witnesses for
% its evidence). Both rule orderings enumerate when RuleA/RuleB are unbound; conflict_pairs/2 canonicalizes.
conflict_witness(Facts, RuleA, RuleB, StmtA, StmtB) :-
    opposing_rule_directions(Facts, RuleA, RuleB),
    RuleA \== RuleB,
    member(rule(RuleA, StmtA), Facts),
    member(rule(RuleB, StmtB), Facts),
    same_action(Facts, StmtA, StmtB),
    contexts_overlap(Facts, StmtA, StmtB).

%% conflict_pairs(+Facts, -Pairs) is det.
% Every conflicting rule pair in a (possibly multi-document) flat KB, each unordered pair once as
% RuleA-RuleB with RuleA @< RuleB. Sorted and duplicate-free (the two witness orderings collapse).
conflict_pairs(Facts, Pairs) :-
    findall(A-B,
            ( conflict_witness(Facts, R1, R2, _, _), ordered_pair(R1, R2, A, B) ),
            Raw),
    sort(Raw, Pairs).

ordered_pair(R1, R2, A, B) :- ( R1 @< R2 -> A = R1, B = R2 ; A = R2, B = R1 ).

% ==========================================================================================
% Eligibility — same action key ∧ opposing directions (§L·conflict).
% ==========================================================================================

%% same_action(+Facts, +StmtA, +StmtB) is semidet.
% StmtA and StmtB carry the IDENTICAL action key (§5: action sameness = key identity). Each statement
% has exactly one action (kb_kernel cardinality), so this is a check, not a search.
same_action(Facts, StmtA, StmtB) :-
    once(member(action(StmtA, Key), Facts)),
    memberchk(action(StmtB, Key), Facts).

% opposing_rule_directions(+Facts, ?RuleA, ?RuleB): the rules' declared directions oppose. Binds the
% rules from the direction facts when they are unbound (drives conflict_witness enumeration).
opposing_rule_directions(Facts, RuleA, RuleB) :-
    member(direction(RuleA, DirA), Facts),
    member(direction(RuleB, DirB), Facts),
    opposing_directions(DirA, DirB).

%% opposing_directions(?DirA, ?DirB) is semidet.
% One direction is in the `positive` group and the other in `against` or `contraindicating`
% (kb_kernel:direction_group/2) — the deontic opposition that makes a same-action pair a conflict.
% `avoid` is non-positive (it joins both non-positive groups); two positives, or two non-positives, do
% not oppose. Semidet — a deterministic yes/no over the pair (no leftover choicepoint).
opposing_directions(DirA, DirB) :-
    ( positive_direction(DirA), nonpositive_direction(DirB) -> true
    ; nonpositive_direction(DirA), positive_direction(DirB) ).

positive_direction(Dir) :- direction_group(Dir, positive).

nonpositive_direction(Dir) :-
    ( direction_group(Dir, against) -> true
    ; direction_group(Dir, contraindicating) ).

% ==========================================================================================
% Context overlap — concept polarity ∧ interval intersection over Q (SPEC §6, D10).
% ==========================================================================================

%% contexts_overlap(+Facts, +StmtA, +StmtB) is semidet.
% Some patient satisfies both statements' guards at once, with neither statement's exceptions firing
% (SPEC §6 SMT-lane exception expansion). True iff no concept is required present by one guard yet
% excluded by either exception (concept polarity) AND every constrained quantity's combined interval
% range holds a rational point (interval intersection). Concepts are independent booleans, orthogonal to
% the age quantity, so the two checks together decide satisfiability with no search.
contexts_overlap(Facts, StmtA, StmtB) :-
    stmt_concepts(Facts, StmtA, PosA),
    stmt_concepts(Facts, StmtB, PosB),
    union_set(PosA, PosB, Required),
    stmt_exception_concepts(Facts, StmtA, NegA),
    stmt_exception_concepts(Facts, StmtB, NegB),
    union_set(NegA, NegB, Excluded),
    disjoint(Required, Excluded),
    quantities(Facts, StmtA, StmtB, Quantities),
    forall(member(Q, Quantities), quantity_overlap(Facts, StmtA, StmtB, Q)).

% stmt_concepts(+Facts, +Stmt, -Concepts): the sorted set of concept ids in Stmt's guard conditions.
stmt_concepts(Facts, Stmt, Concepts) :-
    findall(C, member(condition(_, Stmt, concept(C)), Facts), Cs),
    sort(Cs, Concepts).

% stmt_exception_concepts(+Facts, +Stmt, -Concepts): the sorted set of concept ids Stmt's labeled
% exceptions exclude (D6 exception bodies are single concepts — interval-/op-free, by profile).
stmt_exception_concepts(Facts, Stmt, Concepts) :-
    findall(C, member(exception(_, Stmt, concept(C)), Facts), Cs),
    sort(Cs, Concepts).

% quantities(+Facts, +StmtA, +StmtB, -Quantities): the sorted set of quantities either statement's
% guard constrains with an interval atom.
quantities(Facts, StmtA, StmtB, Quantities) :-
    findall(Q,
            ( member(Stmt, [StmtA, StmtB]),
              member(condition(_, Stmt, interval(Q, _, _, _)), Facts) ),
            Qs),
    sort(Qs, Quantities).

% quantity_overlap(+Facts, +StmtA, +StmtB, +Q): the two statements' effective ranges for quantity Q
% share a rational point. Each statement's same-quantity bounds fold to an effective range first (a
% bounded range is >1 atom); a quantity absent from one statement is unconstrained there
% (range(none, none)), which overlaps anything non-empty.
quantity_overlap(Facts, StmtA, StmtB, Q) :-
    stmt_range(Facts, StmtA, Q, RangeA),
    stmt_range(Facts, StmtB, Q, RangeB),
    ranges_overlap(RangeA, RangeB).

% stmt_range(+Facts, +Stmt, +Q, -Range): fold Stmt's same-quantity interval bounds into an effective
% range (bounds_range/2). No interval for Q -> range(none, none) (unconstrained, all-Q). interval_bound/3
% validates each bound and filters by quantity, so a non-interval atom is skipped.
stmt_range(Facts, Stmt, Q, Range) :-
    findall(Bound,
            ( member(condition(_, Stmt, Atom), Facts),
              interval_bound(Atom, Q, Bound) ),
            Bounds),
    bounds_range(Bounds, Range).

% ---- sorted-set helpers (findall/sort idiom, matching the sibling modules; no ordsets dep) ----------
union_set(SetA, SetB, Union) :-
    append(SetA, SetB, Both),
    sort(Both, Union).

disjoint(SetA, SetB) :-
    \+ ( member(X, SetA), memberchk(X, SetB) ).
