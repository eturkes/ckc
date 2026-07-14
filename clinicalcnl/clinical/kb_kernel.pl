% ClinicalCNL KB kernel — the rules-as-data term family + validators (M3.kb-contract; SPEC §6 LP
% profile, §5 domain IR). This module OWNS the KB contract: the fixed fact family a compiled
% ClinicalCNL document is (`clinical/KB.md`), the closed v1 vocabulary, the well-formedness /
% safety validators the plunit gate exercises, and the PROLEG negation-as-failure reference
% derivability the conflict layer builds on.
%
% A "KB" here is a LIST of ground fact terms (never asserted): kb-writer (kb_bytes/2, write_kb/2
% below) emits such a list to canonical bytes, map-* produce it, conflict-* consume it. Validation
% and kb_bytes/2 are pure; write_kb/2's only effect is its explicit stream write.
%
%   Gate: swipl -q -g "consult('clinical/kb_kernel_tests.pl'),(run_tests(kb_kernel)->halt(0);halt(1))" -t 'halt(1)'
%   Writer gate: swipl -q -g "consult('clinical/kb_writer_tests.pl'),(run_tests(kb_writer)->halt(0);halt(1))" -t 'halt(1)'
%
% Downstream (roadmap-pending, not yet in-tree): registry.pl (ulex) will mirror this vocabulary as
% surfaces + cross-check coverage against kb_concept/1 etc.; map-emit will byte-pin the emitter's
% output over whole documents; conflict-core will reuse the atom + exception structure for symbolic
% context overlap.

:- module(kb_kernel,
          [ valid_kb/1,            % +Facts            — the KB is well-formed (no violations)
            kb_errors/2,           % +Facts, -Errors   — sorted list of violation terms ([] = valid)
            derivable/3,           % +StmtId, +Facts, +Ctx — reference PROLEG derivability over a fixture context
            direction_group/2,     % ?Direction, ?Group — §L·conflict direction -> group relation
            valid_id/2,            % ?Id, ?Kind        — Id is a well-formed <doc>.<kind>.<k> id
            action_key/3,          % ?Key, ?Kind, ?Target — split/join a <kind>:<target> action key
            valid_atom/1,          % ?Atom             — a well-formed context atom
            kb_bytes/2,            % +Facts, -Bytes    — canonical byte serialization (byte-sorted)
            write_kb/2,            % +Stream, +Facts   — write canonical bytes to a stream
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
    scalar_text(Id),                     % no lone surrogate — would not re-read from canonical bytes
    atomic_list_concat(Parts, '.', Id),
    split_last_two(Parts, DocParts, KindAtom, CounterAtom),
    DocParts \== [],
    \+ member('', DocParts),             % every doc segment non-empty
    id_kind(KindAtom),
    ( var(Kind) -> Kind = KindAtom ; Kind == KindAtom ),
    canonical_counter(CounterAtom).

id_kind(K) :- memberchk(K, [rule, stmt, bind, exc]).

% canonical_counter(+Atom) — the counter is a canonical unsigned decimal: `0`, or a nonzero digit
% then digits. Rejects the sign / leading-zero / radix / underscore spellings (-0, +1, 01, 0x10,
% 1_0) that atom_number/2 silently accepts, so each logical id has exactly one textual form.
canonical_counter(A) :-
    atom_codes(A, Codes),
    canonical_digits(Codes).
canonical_digits([D])        :- !, digit(D).                               % any single digit
canonical_digits([D0,D1|Ds]) :- D0 >= 0'1, D0 =< 0'9, all_digits([D1|Ds]). % multi: no leading zero
digit(D) :- D >= 0'0, D =< 0'9.
all_digits([]).
all_digits([D|Ds]) :- digit(D), all_digits(Ds).

% scalar_text(+Atomic) — the atom/string carries only Unicode scalar values (no lone surrogate,
% U+D800..U+DFFF). write_term emits a surrogate as `\uXXXX`, which the reader rejects ("Illegal
% character code"), so a valid KB's free text (ids, source basis) must be scalar to round-trip from
% the canonical bytes. ACE-sourced text is ASCII, so this always holds in practice; the guard makes
% the round-trip guarantee total rather than merely typical.
scalar_text(X) :-
    ( atom(X) -> atom_codes(X, Cs) ; string(X) -> string_codes(X, Cs) ; Cs = [] ),
    \+ ( member(C, Cs), C >= 0xD800, C =< 0xDFFF ).

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

% ---- shape: the KB is a proper list ---------------------------------------------------------

% Fail closed: a non-list / improper-list "KB" has no proper members, so every member/2 check
% below would vacuously find nothing and certify it clean. This violation makes valid_kb/1 reject.
kb_violation(Facts, not_a_list) :-
    \+ is_list(Facts).

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

% Action key must be a ground ATOM `<kind>:<target>`. The atom/1 guards keep kb_errors total: a var
% key falls to nonground_fact, a compound / string / number key to malformed_action_key (never a raw
% atomic_list_concat/3 type/instantiation throw), and a string alias never validates as an atom key.
kb_violation(Facts, malformed_action_key(K)) :-
    member(action(_,K), Facts), ground(K), \+ ( atom(K), action_key(K, _, _) ).
kb_violation(Facts, unknown_action_kind(Kind)) :-
    member(action(_,K), Facts), atom(K), action_key(K, Kind, _), \+ kb_action_kind(Kind).
kb_violation(Facts, unknown_action_target(T)) :-
    member(action(_,K), Facts), atom(K), action_key(K, _, T), \+ kb_action_target(T).

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

% Structural integrity: no rule/2 disjunct-pair repeats, and no statement is owned by two distinct
% rules (each disjunct belongs to one rule; a stmt shared across docs is the very collision the
% doc-qualified ids exist to prevent).
kb_violation(Facts, duplicate_rule(R,S))     :- member(rule(R,S), Facts), count_matches(rule(R,S), Facts, N), N > 1.
kb_violation(Facts, multi_owned_stmt(S))     :- member(rule(R1,S), Facts), member(rule(R2,S), Facts), R1 @< R2.

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
ground_key(rule(R,S))        :- nonvar(R), nonvar(S).
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
    is_list(Regions), Regions = [_|_],
    forall(member(Reg, Regions), (integer(Reg), Reg >= 0)),
    strictly_ascending(Regions),
    ( Basis == none ; ( string(Basis), scalar_text(Basis) ) ).

% Region indices form a canonical set: at least one, strictly ascending (hence unique) — so one
% provenance has exactly one byte form for kb-writer to pin.
strictly_ascending([_]).
strictly_ascending([A,B|T]) :- A < B, strictly_ascending([B|T]).

% ==========================================================================================
% Reference execution semantics (PROLEG negation-as-failure), over a synthetic FIXTURE context.
%
% derivable/3 pins the advice-derivability the conflict layer's differential tests share (SPEC §6:
% "context satisfaction over finite fixture contexts, exception expansion equivalence"). It is a
% CONTRACT + test reference, NOT shipped patient evaluation — that stays behind §15 G-RULE-EVAL.
% The conflict layer builds SYMBOLIC context overlap (conflict-core) on this same atom + exception
% structure; it never patient-evaluates.
%
% A well-formed fixture context Ctx is a list of ground facts about one patient, at most one value
% per quantity (memberchk/2 reads the first, so the caller supplies exactly one):
%   concept(C)      — condition C holds (closed-world: absent = does not hold)
%   quantity(Q, V)  — quantity Q has the single exact rational value V
% ==========================================================================================

%% derivable(+StmtId, +Facts, +Ctx) is semidet.
% StmtId's advice holds for Ctx over the KB fact list Facts: StmtId is a declared disjunct, every
% condition atom holds, AND no exception fires. Facts is the same list valid_kb/1 validates (never
% asserted). The declared-disjunct guard stops derivability succeeding vacuously for an unknown id.
derivable(StmtId, Facts, Ctx) :-
    once(member(rule(_, StmtId), Facts)),
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

% ==========================================================================================
% Canonical emission (kb-writer). A KB fact list -> canonical bytes: one fact per line, each the
% fact written as a QUOTED, re-readable Prolog term terminated by `.`, the lines byte-sorted so input
% order and duplicate lines are irrelevant, a non-empty KB ending in one newline (the empty KB emits
% ""). Determinism is by construction — a dedicated write_term/3 with EVERY flag-sensitive knob pinned
% (see fact_line/2; never the bare write/print defaults) plus a total byte sort over the emitted line
% strings. SPEC §6: the LP program is "emitted deterministically"; the byte-order sort mirrors the SMT
% lane's "declarations sorted by symbol bytes" and the canonical-JSON byte convention. The wire form
% is UTF-8/LF/no-BOM (write_kb/2); the output is a loadable Prolog fact file that round-trips to the
% same fact SET under a standard reader (double_quotes=string, character_escapes=true — the SWI
% defaults). map-emit will byte-pin the emitter's output over whole documents.
% ==========================================================================================

%% kb_bytes(+Facts, -Bytes) is det.
% Bytes is the canonical serialization of the KB fact list Facts as a text string; its wire form is
% UTF-8 (write_kb/2). Pure and total over any list of ground fact terms — the caller validates
% (valid_kb/1), the writer does NOT (separation of concerns). Lines are byte-sorted and de-duplicated,
% so a KB, being a SET of facts, has exactly one byte form (repeated identical facts collapse to one
% line). rational_syntax is bound to compatibility across the render so exact bounds print `NrD`
% regardless of the ambient reader flag.
kb_bytes(Facts, Bytes) :-
    must_be(list, Facts),
    current_prolog_flag(rational_syntax, RS0),
    setup_call_cleanup(set_prolog_flag(rational_syntax, compatibility),
                       maplist(fact_line, Facts, Lines),
                       set_prolog_flag(rational_syntax, RS0)),
    sort(Lines, Sorted),                 % standard order over strings = code (byte) order
    with_output_to(string(Bytes), maplist(write, Sorted)).

% fact_line(+Fact, -Line): the fact as `<quoted-term>.\n`. Every write_term/3 knob that could drift
% with an ambient flag is pinned, so the bytes are a function of the fact alone: quoted(true) keeps
% atoms/strings re-readable; ignore_ops(true) forces functorial notation (a user `op/3` on a functor
% like `rule` can never turn `rule(a,b)` into `a rule b`, and lists still print `[..]`);
% character_escapes(true) + character_escapes_unicode(true) pin the escape spelling of control / quote
% / backslash chars (so a note's newline is escaped, never a raw line break); quote_non_ascii(false)
% emits printable non-ASCII literally onto the UTF-8 wire; numbervars(false) is inert here (ground).
% Every fact is a compound functor(...), so the line ends `).` — no float-vs-fullstop reparse hazard.
fact_line(Fact, Line) :-
    with_output_to(string(Body),
        write_term(Fact, [ quoted(true), numbervars(false), ignore_ops(true),
                           character_escapes(true), character_escapes_unicode(true),
                           quote_non_ascii(false) ])),
    string_concat(Body, ".\n", Line).

%% write_kb(+Stream, +Facts) is det.
% Write the canonical bytes to Stream (map-emit's file sink). The canonical wire form is UTF-8 with LF
% terminators and no BOM, so write_kb/2 pins the stream encoding to utf8 (restoring it after) — the
% octets are then identical however the caller opened the stream. kb_bytes/2 is pure; the only side
% effect here is the explicit stream write.
write_kb(Stream, Facts) :-
    kb_bytes(Facts, Bytes),
    stream_property(Stream, encoding(Enc0)),
    setup_call_cleanup(set_stream(Stream, encoding(utf8)),
                       write(Stream, Bytes),
                       set_stream(Stream, encoding(Enc0))).
