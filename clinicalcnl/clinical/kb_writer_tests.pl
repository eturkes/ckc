% kb-writer canonical-emission gate (M3.kb-writer). Byte-pins the deterministic KB emitter
% (kb_kernel:kb_bytes/2) over the kb-contract valid examples: hand-authored normative bytes for the
% §8.6 thread docs (docA/docB/control) + the multi-disjunct synthetic, plus the properties the
% canonical form guarantees — byte-sorted lines, faithful round-trip, input-order invariance, single
% trailing newline. Source-relative + cwd-independent.
%
%   Gate: swipl -q -g "consult('clinical/kb_writer_tests.pl'),(run_tests(kb_writer)->halt(0);halt(1))" -t 'halt(1)'

:- module(kb_writer_tests, []).

:- use_module(library(plunit)).
:- use_module(library(lists)).

:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K),
   atomic_list_concat([D, '/goldens/kb_examples.pl'], E), use_module(E).

% ---- hand-authored normative bytes ----------------------------------------------------------
% writer_golden(Name, Lines): Lines = the canonical output split into per-fact lines (each the
% quoted term + '.'), in BYTE-SORTED order (hand-derived: functor bytes, then argument bytes — so
% action < certainty < condition < direction < exception < population < rule < source < strength,
% and within a family by the first argument). golden_bytes/2 rebuilds the byte string by joining
% with '\n' and a single trailing '\n', so the golden pins the sort order AND the per-line framing
% independently of the writer: a writer that sorted by term (arity-first) rather than by line bytes,
% dropped the trailing newline, or misplaced the '.' would diverge here.

writer_golden(doc_a,
[ "action('test_source.m1_guideline_a.stmt.0','act.administer:drug.abx_a')."
, "condition('test_source.m1_guideline_a.bind.0','test_source.m1_guideline_a.stmt.0',concept('cond.sepsis'))."
, "condition('test_source.m1_guideline_a.bind.1','test_source.m1_guideline_a.stmt.0',interval('q.age_years',18,closed,lower))."
, "direction('test_source.m1_guideline_a.rule.0',for)."
, "exception('test_source.m1_guideline_a.exc.0','test_source.m1_guideline_a.stmt.0',concept('cond.renal_severe'))."
, "population('test_source.m1_guideline_a.stmt.0','pop.patient')."
, "rule('test_source.m1_guideline_a.rule.0','test_source.m1_guideline_a.stmt.0')."
, "source('test_source.m1_guideline_a.exc.0','test_source.m1_guideline_a',[1],\"renal-impairment carve-out\")."
, "source('test_source.m1_guideline_a.rule.0','test_source.m1_guideline_a',[0],\"guideline A sepsis recommendation\")."
, "source('test_source.m1_guideline_a.stmt.0','test_source.m1_guideline_a',[0],none)."
, "strength('test_source.m1_guideline_a.rule.0',strong)."
]).

writer_golden(doc_b,
[ "action('test_source.m1_guideline_b.stmt.0','act.administer:drug.abx_a')."
, "condition('test_source.m1_guideline_b.bind.0','test_source.m1_guideline_b.stmt.0',concept('cond.sepsis'))."
, "condition('test_source.m1_guideline_b.bind.1','test_source.m1_guideline_b.stmt.0',interval('q.age_years',18,closed,lower))."
, "condition('test_source.m1_guideline_b.bind.2','test_source.m1_guideline_b.stmt.0',concept('cond.pregnancy'))."
, "direction('test_source.m1_guideline_b.rule.0',contraindicate)."
, "population('test_source.m1_guideline_b.stmt.0','pop.patient')."
, "rule('test_source.m1_guideline_b.rule.0','test_source.m1_guideline_b.stmt.0')."
, "source('test_source.m1_guideline_b.rule.0','test_source.m1_guideline_b',[0],\"guideline B pregnancy contraindication\")."
, "strength('test_source.m1_guideline_b.rule.0',strong)."
]).

writer_golden(control,
[ "action('test_source.m1_control.stmt.0','act.administer:drug.abx_a')."
, "condition('test_source.m1_control.bind.0','test_source.m1_control.stmt.0',concept('cond.sepsis'))."
, "condition('test_source.m1_control.bind.1','test_source.m1_control.stmt.0',interval('q.age_years',18,open,upper))."
, "direction('test_source.m1_control.rule.0',contraindicate)."
, "population('test_source.m1_control.stmt.0','pop.patient')."
, "rule('test_source.m1_control.rule.0','test_source.m1_control.stmt.0')."
, "source('test_source.m1_control.rule.0','test_source.m1_control',[0],\"control pediatric contraindication\")."
, "strength('test_source.m1_control.rule.0',strong)."
]).

writer_golden(multi,
[ "action('test_source.kb_multi.stmt.0','act.administer:drug.abx_a')."
, "action('test_source.kb_multi.stmt.1','act.administer:drug.abx_a')."
, "action('test_source.kb_multi.stmt.2','act.administer:drug.abx_a')."
, "certainty('test_source.kb_multi.rule.0',moderate)."
, "condition('test_source.kb_multi.bind.0','test_source.kb_multi.stmt.0',concept('cond.sepsis'))."
, "condition('test_source.kb_multi.bind.1','test_source.kb_multi.stmt.0',interval('q.age_years',18,open,lower))."
, "condition('test_source.kb_multi.bind.2','test_source.kb_multi.stmt.1',concept('cond.pregnancy'))."
, "condition('test_source.kb_multi.bind.3','test_source.kb_multi.stmt.1',interval('q.age_years',18,closed,upper))."
, "condition('test_source.kb_multi.bind.4','test_source.kb_multi.stmt.2',concept('cond.sepsis'))."
, "direction('test_source.kb_multi.rule.0',for)."
, "direction('test_source.kb_multi.rule.1',permit)."
, "exception('test_source.kb_multi.exc.0','test_source.kb_multi.stmt.0',concept('cond.renal_severe'))."
, "exception('test_source.kb_multi.exc.1','test_source.kb_multi.stmt.0',concept('cond.pregnancy'))."
, "exception('test_source.kb_multi.exc.2','test_source.kb_multi.stmt.1',concept('cond.renal_severe'))."
, "population('test_source.kb_multi.stmt.0','pop.patient')."
, "population('test_source.kb_multi.stmt.1','pop.patient')."
, "population('test_source.kb_multi.stmt.2','pop.patient')."
, "rule('test_source.kb_multi.rule.0','test_source.kb_multi.stmt.0')."
, "rule('test_source.kb_multi.rule.0','test_source.kb_multi.stmt.1')."
, "rule('test_source.kb_multi.rule.1','test_source.kb_multi.stmt.2')."
, "source('test_source.kb_multi.rule.0','test_source.kb_multi',[0,1],\"multi-disjunct rule\")."
, "source('test_source.kb_multi.rule.1','test_source.kb_multi',[2],\"trailing rule\")."
, "strength('test_source.kb_multi.rule.0',weak)."
, "strength('test_source.kb_multi.rule.1',weak)."
]).

% golden_bytes(+Lines, -Bytes): the writer's framing — each golden line then a newline, in this exact
% order, the whole ending in a single newline. atomic_list_concat/3's '\n' separator handles the
% between-line newlines; atom_concat/3 adds the trailer.
golden_bytes(Lines, Bytes) :-
    atomic_list_concat(Lines, '\n', Joined),
    atom_concat(Joined, '\n', WithTrailer),
    atom_string(WithTrailer, Bytes).

:- begin_tests(kb_writer).

% ---- byte-pin: emitter output == hand-authored normative bytes ------------------------------
test(bytes_pinned, [forall(writer_golden(Name, Lines))]) :-
    kb_example(Name, valid, Facts),
    kb_bytes(Facts, Bytes),
    golden_bytes(Lines, Expected),
    ( Bytes == Expected
    -> true
    ;  format(user_error, "kb_writer: ~w bytes mismatch~n got:~n~w~nexpected:~n~w~n",
              [Name, Bytes, Expected]), fail ).

% Every valid example carries a golden (no example silently unpinned; no golden without an example).
test(golden_coverage) :-
    findall(N, kb_example(N, valid, _), Es), sort(Es, ES),
    findall(N, writer_golden(N, _),     Gs), sort(Gs, GS),
    assertion(ES == GS).

% ---- property: output lines are byte-sorted (independent of the hand goldens) ----------------
test(byte_sorted, [forall(kb_example(Name, valid, Facts))]) :-
    kb_bytes(Facts, Bytes),
    output_lines(Bytes, Lines),
    ( strictly_ascending_strings(Lines)
    -> true
    ;  format(user_error, "kb_writer: ~w lines not strictly byte-ascending: ~w~n", [Name, Lines]), fail ).

% ---- property: round-trip — the bytes reparse to exactly the fact set -----------------------
test(round_trip, [forall(kb_example(Name, valid, Facts))]) :-
    kb_bytes(Facts, Bytes),
    read_all_terms(Bytes, Terms),
    sort(Facts, FS), sort(Terms, TS),
    ( FS == TS
    -> true
    ;  format(user_error, "kb_writer: ~w round-trip differs (facts vs reparsed)~n", [Name]), fail ).

% ---- property: input-order invariance — a shuffled fact list emits identical bytes -----------
test(order_invariant, [forall(kb_example(_, valid, Facts))]) :-
    kb_bytes(Facts, B0),
    reverse(Facts, Rev), kb_bytes(Rev, B1),
    rotate1(Facts, Rot), kb_bytes(Rot, B2),
    assertion(B0 == B1),
    assertion(B0 == B2).

% ---- framing: ends in exactly one newline, no blank lines -----------------------------------
test(single_trailing_newline, [forall(kb_example(_, valid, Facts))]) :-
    kb_bytes(Facts, Bytes),
    string_concat(_, "\n", Bytes),              % ends with a newline
    \+ sub_string(Bytes, _, _, _, "\n\n").      % contains no blank line

% ---- focused: rational bound renders as NrD (never a float); empty + singleton framing --------
test(rational_render) :-
    kb_bytes([condition('d.bind.0','d.stmt.0', interval('q.age_years', 1r2, open, upper))], Bytes),
    assertion(Bytes == "condition('d.bind.0','d.stmt.0',interval('q.age_years',1r2,open,upper)).\n").
test(empty_kb) :-
    kb_bytes([], Bytes),
    assertion(Bytes == "").
test(singleton_framing) :-
    kb_bytes([direction('d.rule.0', for)], Bytes),
    assertion(Bytes == "direction('d.rule.0',for).\n").
% A non-list is a caller error (not a silent empty emit).
test(non_list_errors) :-
    catch(kb_bytes(not_a_list, _), _, Caught = yes),
    assertion(Caught == yes).

:- end_tests(kb_writer).

% ---- helpers --------------------------------------------------------------------------------
% output_lines(+Bytes, -Lines): the byte string split into its per-fact lines, dropping the empty
% tail after the final newline.
output_lines(Bytes, Lines) :-
    split_string(Bytes, "\n", "", Parts),
    exclude(==(""), Parts, Lines).

strictly_ascending_strings([]).
strictly_ascending_strings([_]).
strictly_ascending_strings([A, B | T]) :- A @< B, strictly_ascending_strings([B | T]).

rotate1([], []).
rotate1([H | T], R) :- append(T, [H], R).

% read_all_terms(+String, -Terms): reparse the emitted bytes (a loadable fact file) back to terms.
read_all_terms(String, Terms) :-
    setup_call_cleanup(open_string(String, S),
                       read_stream_terms(S, Terms),
                       close(S)).
read_stream_terms(S, Terms) :-
    read_term(S, T, []),
    ( T == end_of_file
    -> Terms = []
    ;  Terms = [T | Rest], read_stream_terms(S, Rest) ).
