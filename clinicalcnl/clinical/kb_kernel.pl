% ClinicalCNL KB kernel — the rules-as-data term family + validators (M3.kb-contract; SPEC §6 LP
% profile, §5 domain IR). This module OWNS the KB contract: the fixed fact family a compiled
% ClinicalCNL document is (`clinical/KB.md`), the closed v1 vocabulary, the well-formedness /
% safety validators the plunit gate exercises, and the PROLEG negation-as-failure reference
% derivability the conflict layer builds on.
%
% A "KB" here is a LIST of ground fact terms (never asserted): kb-writer emits such a list to
% canonical bytes, map-* produce it, conflict-* consume it. Validation is side-effect-free.
%
%   Gate: swipl -q -g "consult('clinical/kb_kernel_tests.pl'),(run_tests(kb_kernel)->halt(0);halt(1))" -t 'halt(1)'
%
% Downstream: registry.pl (ulex) mirrors this vocabulary as surfaces and its integrity checker
% cross-checks coverage against kb_concept/1 etc.; kb-writer byte-pins goldens/kb_examples.pl;
% conflict-core reuses the atom + exception structure for symbolic context overlap.

:- module(kb_kernel,
          [ valid_kb/1,            % +Facts            — the KB is well-formed (no violations)
            kb_errors/2,           % +Facts, -Errors   — sorted list of violation terms ([] = valid)
            derivable/3,           % +StmtId, +Facts, +Ctx — reference PROLEG derivability over a fixture context
            direction_group/2,     % ?Direction, ?Group — §L·conflict direction -> group relation
            valid_id/2,            % ?Id, ?Kind        — Id is a well-formed <doc>.<kind>.<k> id
            action_key/3,          % ?Key, ?Kind, ?Target — split/join a <kind>:<target> action key
            valid_atom/1,          % ?Atom             — a well-formed context atom
            kb_concept/1, kb_action_kind/1, kb_action_target/1, kb_quantity/1,
            kb_population/1, kb_direction/1, kb_strength/1, kb_certainty/1
          ]).

% ==========================================================================================
% Vocabulary (closed, v1). Owned here; the ulex registry covers each id with a surface.
% ==========================================================================================

%% kb_concept(?Concept) — a clinical condition concept (a context atom's `concept(_)` payload).
kb_concept('cond.sepsis').
kb_concept('cond.renal_severe').
kb_concept('cond.pregnancy').

%% kb_action_kind(?Kind) / kb_action_target(?Target) — the two halves of a `<kind>:<target>` key.
kb_action_kind('act.administer').
kb_action_target('drug.abx_a').

%% kb_quantity(?Q) — a proof-visible quantity variable (an `interval(_,...)` atom's var).
kb_quantity('q.age_years').

%% kb_population(?Pop) — a population subject. v1 rules are all about the generic patient; the
% adult/child demographic is carried as an age `interval` condition, never a population id.
kb_population('pop.patient').

%% kb_direction(?Direction) — the deontic direction vocabulary (§L·conflict groups). v1 keywords
% emit {for, permit, against, contraindicate}; require/avoid are contract-admitted, not v1-emitted.
kb_direction(for).
kb_direction(require).
kb_direction(permit).
kb_direction(against).
kb_direction(avoid).
kb_direction(contraindicate).

%% kb_strength(?Strength) / kb_certainty(?Certainty) — proof-visible annotations (§5); conflict
% logic ignores them (it consumes direction + normalized action/context).
kb_strength(strong).
kb_strength(weak).

kb_certainty(high).
kb_certainty(moderate).
kb_certainty(low).
kb_certainty(very_low).

%% direction_group(?Direction, ?Group) — the §L·conflict direction -> group relation (the conflict
% layer's eligibility check consumes it). Relational: `avoid` joins both non-positive groups.
direction_group(for,            positive).
direction_group(require,        positive).
direction_group(permit,         positive).
direction_group(against,        against).
direction_group(avoid,          against).
direction_group(avoid,          contraindicating).
direction_group(contraindicate, contraindicating).

% ==========================================================================================
% Grammar helpers.
% ==========================================================================================

%% valid_id(?Id, ?Kind) is semidet.
% Id is a doc-qualified dotted atom `<doc>.<kind>.<k>`: a non-empty doc prefix (itself dotted,
% e.g. test_source.m1_guideline_a), one of the four Kinds, then a non-negative integer counter.
% The counter is document-continuous (map-emit never resets it per rule).
valid_id(Id, Kind) :-
    atom(Id),
    atomic_list_concat(Parts, '.', Id),
    split_last_two(Parts, DocParts, KindAtom, CounterAtom),
    DocParts \== [],
    id_kind(KindAtom),
    ( var(Kind) -> Kind = KindAtom ; Kind == KindAtom ),
    atom_number(CounterAtom, N),
    integer(N), N >= 0.

id_kind(K) :- memberchk(K, [rule, stmt, bind, exc]).

% split_last_two(+List, -Init, -Penult, -Last) — deterministic last-two split (List has >= 2).
split_last_two(List, Init, Penult, Last) :-
    reverse(List, [Last, Penult | RevInit]),
    reverse(RevInit, Init).

%% id_doc(+Id, -Doc) — the document-id prefix of a well-formed id.
id_doc(Id, Doc) :-
    atomic_list_concat(Parts, '.', Id),
    split_last_two(Parts, DocParts, _Kind, _Counter),
    atomic_list_concat(DocParts, '.', Doc).

%% action_key(?Key, ?Kind, ?Target) is semidet.
% A ClinicalCNL action key is exactly `<kind>:<target>` — one colon. atomic_list_concat/3 is
% bidirectional and fails unless the key splits into exactly two parts (0 or 2+ colons reject).
action_key(Key, Kind, Target) :-
    atomic_list_concat([Kind, Target], ':', Key).

%% valid_atom(?Atom) is semidet.
% A context atom (in a condition or an exception): a concept, or a bounded interval over a
% quantity. Bounds are EXACT (integer or n/d rational), never float (D10 arithmetic).
valid_atom(concept(C)) :-
    kb_concept(C).
valid_atom(interval(Q, Bound, Openness, Dir)) :-
    kb_quantity(Q),
    rational(Bound),
    interval_openness(Openness),
    interval_dir(Dir).

interval_openness(open).
interval_openness(closed).

interval_dir(lower).
interval_dir(upper).

% ==========================================================================================
% Validator. kb_errors/2 collects every violation; valid_kb/1 = no violations.
% Each violation term names a defect precisely so the reject tests pinpoint the invariant.
% ==========================================================================================

%% valid_kb(+Facts) is semidet.
valid_kb(Facts) :-
    kb_errors(Facts, []).

%% kb_errors(+Facts, -Errors) is det.
kb_errors(Facts, Errors) :-
    findall(E, kb_violation(Facts, E), Es),
    sort(Es, Errors).

% kb_violation/2 clauses are grouped by concern below (interspersed with their local helpers),
% not contiguously.
:- discontiguous kb_violation/2.

% ---- shape: family membership + groundness --------------------------------------------------

kb_violation(Facts, unknown_fact(F)) :-
    member(F, Facts),
    \+ kb_fact(F).

kb_violation(Facts, nonground_fact(F)) :-
    member(F, Facts),
    \+ ground(F).

%% kb_fact(?Fact) — the nine rules-as-data families (functor + arity).
kb_fact(rule(_,_)).
kb_fact(direction(_,_)).
kb_fact(strength(_,_)).
kb_fact(certainty(_,_)).
kb_fact(population(_,_)).
kb_fact(condition(_,_,_)).
kb_fact(action(_,_)).
kb_fact(exception(_,_,_)).
kb_fact(source(_,_,_,_)).

% ---- shape: ids well-formed per family ------------------------------------------------------

kb_violation(Facts, malformed_id(R)) :- member(rule(R,_),        Facts), \+ valid_id(R, rule).
kb_violation(Facts, malformed_id(S)) :- member(rule(_,S),        Facts), \+ valid_id(S, stmt).
kb_violation(Facts, malformed_id(R)) :- member(direction(R,_),   Facts), \+ valid_id(R, rule).
kb_violation(Facts, malformed_id(R)) :- member(strength(R,_),    Facts), \+ valid_id(R, rule).
kb_violation(Facts, malformed_id(R)) :- member(certainty(R,_),   Facts), \+ valid_id(R, rule).
kb_violation(Facts, malformed_id(S)) :- member(population(S,_),  Facts), \+ valid_id(S, stmt).
kb_violation(Facts, malformed_id(B)) :- member(condition(B,_,_), Facts), \+ valid_id(B, bind).
kb_violation(Facts, malformed_id(S)) :- member(condition(_,S,_), Facts), \+ valid_id(S, stmt).
kb_violation(Facts, malformed_id(S)) :- member(action(S,_),      Facts), \+ valid_id(S, stmt).
kb_violation(Facts, malformed_id(X)) :- member(exception(X,_,_), Facts), \+ valid_id(X, exc).
kb_violation(Facts, malformed_id(S)) :- member(exception(_,S,_), Facts), \+ valid_id(S, stmt).

% ---- vocabulary ------------------------------------------------------------------------------

kb_violation(Facts, unknown_direction(D))  :- member(direction(_,D),  Facts), \+ kb_direction(D).
kb_violation(Facts, unknown_strength(St))  :- member(strength(_,St),  Facts), \+ kb_strength(St).
kb_violation(Facts, unknown_certainty(C))  :- member(certainty(_,C),  Facts), \+ kb_certainty(C).
kb_violation(Facts, unknown_population(P)) :- member(population(_,P), Facts), \+ kb_population(P).

kb_violation(Facts, malformed_action_key(K)) :-
    member(action(_,K), Facts), \+ action_key(K, _, _).
kb_violation(Facts, unknown_action_kind(Kind)) :-
    member(action(_,K), Facts), action_key(K, Kind, _), \+ kb_action_kind(Kind).
kb_violation(Facts, unknown_action_target(T)) :-
    member(action(_,K), Facts), action_key(K, _, T), \+ kb_action_target(T).

% ---- context-atom well-formedness (conditions + exceptions) --------------------------------

kb_violation(Facts, E) :- member(condition(_,_,A), Facts), ground(A), atom_violation(A, E).
kb_violation(Facts, E) :- member(exception(_,_,A), Facts), ground(A), atom_violation(A, E).

atom_violation(concept(C), unknown_concept(C)) :- \+ kb_concept(C).
atom_violation(interval(Q,_,_,_), unknown_quantity(Q))      :- \+ kb_quantity(Q).
atom_violation(interval(_,B,_,_), bad_interval_bound(B))    :- \+ rational(B).
atom_violation(interval(_,_,O,_), bad_interval_openness(O)) :- \+ interval_openness(O).
atom_violation(interval(_,_,_,D), bad_interval_dir(D))      :- \+ interval_dir(D).
atom_violation(A, malformed_atom(A)) :- A \= concept(_), A \= interval(_,_,_,_).

% ---- referential integrity ------------------------------------------------------------------

% A rule-level field (direction/strength/certainty) for a rule id that no rule/2 declares.
kb_violation(Facts, dangling_rule_ref(R)) :-
    member(F, Facts), rule_field(F, R),
    \+ member(rule(R,_), Facts).

% A statement-level fact (population/condition/action/exception) for a stmt no rule/2 declares.
kb_violation(Facts, dangling_stmt_ref(S)) :-
    member(F, Facts), stmt_ref(F, S),
    \+ member(rule(_,S), Facts).

% A source/4 whose subject id is neither a declared rule, statement, nor exception.
kb_violation(Facts, dangling_source_ref(I)) :-
    member(source(I,_,_,_), Facts),
    \+ member(rule(I,_), Facts),
    \+ member(rule(_,I), Facts),
    \+ member(exception(I,_,_), Facts).

rule_field(direction(R,_), R).
rule_field(strength(R,_),  R).
rule_field(certainty(R,_), R).

stmt_ref(population(S,_),  S).
stmt_ref(condition(_,S,_), S).
stmt_ref(action(S,_),      S).
stmt_ref(exception(_,S,_), S).

% ---- cardinality + safety -------------------------------------------------------------------

% Every declared statement has exactly one population (the subject binder) and one action.
kb_violation(Facts, missing_population(S))   :- declared_stmt(Facts, S), \+ member(population(S,_), Facts).
kb_violation(Facts, missing_action(S))       :- declared_stmt(Facts, S), \+ member(action(S,_), Facts).
kb_violation(Facts, duplicate_population(S)) :- member(population(S,_), Facts), count_matches(population(S,_), Facts, N), N > 1.
kb_violation(Facts, duplicate_action(S))     :- member(action(S,_),     Facts), count_matches(action(S,_),     Facts, N), N > 1.

% Every declared rule has exactly one direction, one strength, at most one certainty.
kb_violation(Facts, missing_direction(R))    :- declared_rule(Facts, R), \+ member(direction(R,_), Facts).
kb_violation(Facts, missing_strength(R))     :- declared_rule(Facts, R), \+ member(strength(R,_), Facts).
kb_violation(Facts, duplicate_direction(R))  :- member(direction(R,_), Facts), count_matches(direction(R,_), Facts, N), N > 1.
kb_violation(Facts, duplicate_strength(R))   :- member(strength(R,_),  Facts), count_matches(strength(R,_),  Facts, N), N > 1.
kb_violation(Facts, duplicate_certainty(R))  :- member(certainty(R,_), Facts), count_matches(certainty(R,_), Facts, N), N > 1.

% Binding + exception ids are unique across the KB (document-continuous, no reuse).
kb_violation(Facts, duplicate_bind(B))       :- member(condition(B,_,_), Facts), count_matches(condition(B,_,_), Facts, N), N > 1.
kb_violation(Facts, duplicate_exception(X))  :- member(exception(X,_,_), Facts), count_matches(exception(X,_,_), Facts, N), N > 1.

declared_rule(Facts, R) :- member(rule(R,_), Facts).
declared_stmt(Facts, S) :- member(rule(_,S), Facts).

% count_matches(+Template, +Facts, -N): N raw occurrences unifying Template (Template ground-keyed).
% Raw (not distinct), so an identically-duplicated fact is caught too — a canonical KB has exactly
% one fact per key.
count_matches(Template, Facts, N) :-
    ground_key(Template),
    findall(x, ( member(F, Facts), \+ \+ F = Template ), Xs),
    length(Xs, N).

% A template is ground-keyed when its identifying arg is bound (guards against enumerating vars).
ground_key(population(S,_))  :- nonvar(S).
ground_key(action(S,_))      :- nonvar(S).
ground_key(direction(R,_))   :- nonvar(R).
ground_key(strength(R,_))    :- nonvar(R).
ground_key(certainty(R,_))   :- nonvar(R).
ground_key(condition(B,_,_)) :- nonvar(B).
ground_key(exception(X,_,_)) :- nonvar(X).

% ---- source shape ---------------------------------------------------------------------------

kb_violation(Facts, malformed_source(source(I,D,R,B))) :-
    member(source(I,D,R,B), Facts),
    \+ valid_source(I, D, R, B).

% Provenance: subject id (rule/stmt/exc), an atom doc id matching the subject's prefix, an ordered
% list of non-negative raw-sentence indices, and a basis string (or `none`). Kernel validates shape
% + referential integrity; per-element provenance COMPLETENESS is a map-emit obligation (KB.md).
valid_source(I, D, Regions, Basis) :-
    ( valid_id(I, rule) ; valid_id(I, stmt) ; valid_id(I, exc) ),
    atom(D),
    id_doc(I, D),
    is_list(Regions),
    forall(member(Reg, Regions), (integer(Reg), Reg >= 0)),
    ( Basis == none ; atom(Basis) ; string(Basis) ).

% ==========================================================================================
% Reference execution semantics (PROLEG negation-as-failure), over a synthetic FIXTURE context.
%
% derivable/2 pins the advice-derivability the conflict layer's differential tests share (SPEC §6:
% "context satisfaction over finite fixture contexts, exception expansion equivalence"). It is a
% CONTRACT + test reference, NOT shipped patient evaluation — that stays behind §15 G-RULE-EVAL.
% The conflict layer builds SYMBOLIC context overlap (conflict-core) on this same atom + exception
% structure; it never patient-evaluates.
%
% A fixture context Ctx is a list of ground facts about one patient:
%   concept(C)      — condition C holds (closed-world: absent = does not hold)
%   quantity(Q, V)  — quantity Q has exact value V
% ==========================================================================================

%% derivable(+StmtId, +Facts, +Ctx) is semidet.
% StmtId's advice holds for Ctx over the KB fact list Facts: every condition atom holds AND no
% exception fires. Facts is the same list valid_kb/1 validates (never asserted).
derivable(StmtId, Facts, Ctx) :-
    forall(member(condition(_, StmtId, Atom), Facts), holds_atom(Atom, Ctx)),
    \+ exception_fires(StmtId, Facts, Ctx).

% An exception FIRES when any of the statement's labeled exception atoms holds — the NAF guard.
exception_fires(StmtId, Facts, Ctx) :-
    member(exception(_, StmtId, Atom), Facts),
    holds_atom(Atom, Ctx).

holds_atom(concept(C), Ctx) :-
    memberchk(concept(C), Ctx).
holds_atom(interval(Q, Bound, Openness, Dir), Ctx) :-
    memberchk(quantity(Q, V), Ctx),
    satisfies_bound(V, Bound, Openness, Dir).

satisfies_bound(V, B, closed, lower) :- V >= B.
satisfies_bound(V, B, open,   lower) :- V >  B.
satisfies_bound(V, B, closed, upper) :- V =< B.
satisfies_bound(V, B, open,   upper) :- V <  B.
