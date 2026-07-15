% ClinicalCNL whole-document mapper + emission gate (M3.map-emit + M3.map-exc). Drives
% map_emit:document_bytes/3 over whole profile-validated documents built from the byte-pinned surface
% goldens (read-back DRS) + hand-built raw contexts — so the gate runs pure and fast with no live APE.
% Byte-pins the emitter's OBSERVED canonical output over the §8.6 thread docs (docB, control, and docA
% with its exception block compiled to its NAF guard) plus a synthetic two-rule / two-disjunct /
% non-dense out-of-order-label document and the §L·pins worked exception oracle (bridge). Each mapped
% KB is asserted kb_kernel-valid; docB / control are cross-checked against the normative kb_examples
% (rule-only match). Determinism is gated two ways: each export yields exactly ONE solution
% (single_solution/1 — findall-based, robust under plunit's body control-wrapping, where a
% deterministic/1 probe misreports an ancestor choicepoint) and re-emits byte-identically, the fact
% set emit-order-free. Grouping / counters are exercised past the easy cases: disjuncts group across
% NON-adjacent items and sort by DisjIdx (interleaved + reversed fixture); an exception is stmt-major
% cloned across its rule's statements with a document-continuous exc counter (no per-rule reset), and
% the RULE layer stays base/2-transparent (rule -> exception -> rule == the exception-removed items
% plus only the exception + its own source); the empty document is total. NOT input-permutation
% invariant (KB.md / map-core): rule-block order is surface-positional, so per appearance order the
% same raw label owns a different rule.<Ord> (block_order_positional pins the ordinals + directions,
% not just a byte inequality).
%
%   Gate: swipl -q -g "consult('clinical/map_emit_tests.pl'),(run_tests(map_emit)->halt(0);halt(1))" -t 'halt(1)'

:- module(map_emit_tests, []).

:- use_module(library(lists)).

:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/map_emit.pl'], M), use_module(M),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K),
   atomic_list_concat([D, '/goldens/surface_expected.pl'], G), use_module(G),
   atomic_list_concat([D, '/goldens/kb_examples.pl'], E), use_module(E).

% golden_drs(+Id, -Drs) — the surface golden's serialized DRS read back to a term (the profile-drs /
% map-core read-back pattern; fresh referent vars, in-term sharing preserved by same-named vars).
golden_drs(Id, Drs) :-
    surface_expected(Id, 'text/plain', Atom, _),
    read_term_from_atom(Atom, Drs, []).

% doc(+Name, -DocId, -Items) — a whole-document item list (raw sentence order): each item's DRS a
% read-back surface golden, its Ctx the exact raw-gate block context (rule/5 | exception/4) raw_gate
% would emit. docA carries its exception block (skipped by map-emit); multi's rule labels 5 then 2 are
% non-dense and out of order (dense ordinals follow first appearance: 5 -> rule.0, 2 -> rule.1).
doc(doc_b, 'test_source.m1_guideline_b',
    [item(0, rule(0, contraindicate, 0, none, "guideline B pregnancy contraindication"), Db)]) :-
    golden_drs(thread_doc_b, Db).
doc(control, 'test_source.m1_control',
    [item(0, rule(0, contraindicate, 0, none, "control pediatric contraindication"), Dc)]) :-
    golden_drs(thread_control, Dc).
doc(doc_a, 'test_source.m1_guideline_a',
    [item(0, rule(0, recommend, 0, none, "guideline A sepsis recommendation"), Da),
     item(1, exception(0, 0, none, "renal-impairment carve-out"), De)]) :-
    golden_drs(thread_doc_a, Da), golden_drs(exception_body, De).
doc(multi, 'test_source.map_multi',
    [item(0, rule(5, recommend, 0, moderate, "multi first rule"), D1),
     item(1, rule(5, recommend, 1, moderate, "multi first rule"), D2),
     item(2, rule(2, 'may-consider', 0, none, "multi trailing"), D3)]) :-
    golden_drs(thread_doc_a, D1), golden_drs(frame_recommend, D2), golden_drs(frame_admissible, D3).
% bridge — the §L·pins worked exception oracle: a 2-disjunct rule (rule.0: stmt.0 sepsis+age, stmt.1
% sepsis) carrying TWO exception blocks (renal @sent 2, pregnancy @sent 3), then a trailing 1-disjunct
% rule (rule.1: stmt.2) carrying ONE exception (renal @sent 5). Catches the stmt-major clone + the
% document-continuous exc counter (no per-rule reset): stmt.0{exc.0,exc.1}, stmt.1{exc.2,exc.3},
% stmt.2{exc.4}. (The roadmap's "rule.2" is a Rust-model carryover — one rule id per disjunct; the
% Prolog KB gives one rule id per BLOCK, so the trailing rule is the dense rule.1.)
doc(bridge, 'test_source.map_bridge',
    [item(0, rule(0, recommend, 0, none, "rule A"), A0),
     item(1, rule(0, recommend, 1, none, "rule A"), A1),
     item(2, exception(0, 0, none, "renal carve-out"), Ea),
     item(3, exception(1, 0, none, "pregnancy carve-out"), Eb),
     item(4, rule(1, recommend, 0, none, "rule B"), B0),
     item(5, exception(2, 1, none, "renal carve-out B"), Ec)]) :-
    golden_drs(thread_doc_a, A0), golden_drs(frame_recommend, A1),
    golden_drs(exception_body, Ea), exc_body(pregnancy, Eb),
    golden_drs(frame_recommend, B0), golden_drs(exception_body, Ec).

% exc_body(+ConceptNoun, -Drs) — a bare-condition exception body DRS (D6) for a given registered concept
% noun, matching the OBSERVED exception_body golden shape (population object + concept object wired by
% have). Built via a clause so its referent vars stay fresh; varies the oracle's second exception.
exc_body(Noun, drs([A, B, C],
    [ object(A, patient, countable, na, eq, 1)-1/2,
      object(B, Noun, countable, na, eq, 1)-1/5,
      predicate(C, have, A, B)-1/3 ])).

% doc_golden(+Name, -Lines) — the emitter's OBSERVED canonical bytes for doc(Name, ...), split into
% per-fact lines in byte-sorted order (captured from a run, corroborated for docB / control by
% kb_writer_tests's independent goldens over the same rule-only fact sets). golden_bytes/2 reframes
% them into the wire string, so the pin locks the sort order and the per-line framing together.
doc_golden(doc_b,
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
doc_golden(control,
[ "action('test_source.m1_control.stmt.0','act.administer:drug.abx_a')."
, "condition('test_source.m1_control.bind.0','test_source.m1_control.stmt.0',concept('cond.sepsis'))."
, "condition('test_source.m1_control.bind.1','test_source.m1_control.stmt.0',interval('q.age_years',18,open,upper))."
, "direction('test_source.m1_control.rule.0',contraindicate)."
, "population('test_source.m1_control.stmt.0','pop.patient')."
, "rule('test_source.m1_control.rule.0','test_source.m1_control.stmt.0')."
, "source('test_source.m1_control.rule.0','test_source.m1_control',[0],\"control pediatric contraindication\")."
, "strength('test_source.m1_control.rule.0',strong)."
]).
doc_golden(doc_a,
[ "action('test_source.m1_guideline_a.stmt.0','act.administer:drug.abx_a')."
, "condition('test_source.m1_guideline_a.bind.0','test_source.m1_guideline_a.stmt.0',concept('cond.sepsis'))."
, "condition('test_source.m1_guideline_a.bind.1','test_source.m1_guideline_a.stmt.0',interval('q.age_years',18,closed,lower))."
, "direction('test_source.m1_guideline_a.rule.0',for)."
, "exception('test_source.m1_guideline_a.exc.0','test_source.m1_guideline_a.stmt.0',concept('cond.renal_severe'))."
, "population('test_source.m1_guideline_a.stmt.0','pop.patient')."
, "rule('test_source.m1_guideline_a.rule.0','test_source.m1_guideline_a.stmt.0')."
, "source('test_source.m1_guideline_a.exc.0','test_source.m1_guideline_a',[1],\"renal-impairment carve-out\")."
, "source('test_source.m1_guideline_a.rule.0','test_source.m1_guideline_a',[0],\"guideline A sepsis recommendation\")."
, "strength('test_source.m1_guideline_a.rule.0',strong)."
]).
doc_golden(multi,
[ "action('test_source.map_multi.stmt.0','act.administer:drug.abx_a')."
, "action('test_source.map_multi.stmt.1','act.administer:drug.abx_a')."
, "action('test_source.map_multi.stmt.2','act.administer:drug.abx_a')."
, "certainty('test_source.map_multi.rule.0',moderate)."
, "condition('test_source.map_multi.bind.0','test_source.map_multi.stmt.0',concept('cond.sepsis'))."
, "condition('test_source.map_multi.bind.1','test_source.map_multi.stmt.0',interval('q.age_years',18,closed,lower))."
, "condition('test_source.map_multi.bind.2','test_source.map_multi.stmt.1',concept('cond.sepsis'))."
, "condition('test_source.map_multi.bind.3','test_source.map_multi.stmt.2',concept('cond.sepsis'))."
, "direction('test_source.map_multi.rule.0',for)."
, "direction('test_source.map_multi.rule.1',permit)."
, "population('test_source.map_multi.stmt.0','pop.patient')."
, "population('test_source.map_multi.stmt.1','pop.patient')."
, "population('test_source.map_multi.stmt.2','pop.patient')."
, "rule('test_source.map_multi.rule.0','test_source.map_multi.stmt.0')."
, "rule('test_source.map_multi.rule.0','test_source.map_multi.stmt.1')."
, "rule('test_source.map_multi.rule.1','test_source.map_multi.stmt.2')."
, "source('test_source.map_multi.rule.0','test_source.map_multi',[0,1],\"multi first rule\")."
, "source('test_source.map_multi.rule.1','test_source.map_multi',[2],\"multi trailing\")."
, "strength('test_source.map_multi.rule.0',strong)."
, "strength('test_source.map_multi.rule.1',weak)."
]).
doc_golden(bridge,
[ "action('test_source.map_bridge.stmt.0','act.administer:drug.abx_a')."
, "action('test_source.map_bridge.stmt.1','act.administer:drug.abx_a')."
, "action('test_source.map_bridge.stmt.2','act.administer:drug.abx_a')."
, "condition('test_source.map_bridge.bind.0','test_source.map_bridge.stmt.0',concept('cond.sepsis'))."
, "condition('test_source.map_bridge.bind.1','test_source.map_bridge.stmt.0',interval('q.age_years',18,closed,lower))."
, "condition('test_source.map_bridge.bind.2','test_source.map_bridge.stmt.1',concept('cond.sepsis'))."
, "condition('test_source.map_bridge.bind.3','test_source.map_bridge.stmt.2',concept('cond.sepsis'))."
, "direction('test_source.map_bridge.rule.0',for)."
, "direction('test_source.map_bridge.rule.1',for)."
, "exception('test_source.map_bridge.exc.0','test_source.map_bridge.stmt.0',concept('cond.renal_severe'))."
, "exception('test_source.map_bridge.exc.1','test_source.map_bridge.stmt.0',concept('cond.pregnancy'))."
, "exception('test_source.map_bridge.exc.2','test_source.map_bridge.stmt.1',concept('cond.renal_severe'))."
, "exception('test_source.map_bridge.exc.3','test_source.map_bridge.stmt.1',concept('cond.pregnancy'))."
, "exception('test_source.map_bridge.exc.4','test_source.map_bridge.stmt.2',concept('cond.renal_severe'))."
, "population('test_source.map_bridge.stmt.0','pop.patient')."
, "population('test_source.map_bridge.stmt.1','pop.patient')."
, "population('test_source.map_bridge.stmt.2','pop.patient')."
, "rule('test_source.map_bridge.rule.0','test_source.map_bridge.stmt.0')."
, "rule('test_source.map_bridge.rule.0','test_source.map_bridge.stmt.1')."
, "rule('test_source.map_bridge.rule.1','test_source.map_bridge.stmt.2')."
, "source('test_source.map_bridge.exc.0','test_source.map_bridge',[2],\"renal carve-out\")."
, "source('test_source.map_bridge.exc.1','test_source.map_bridge',[3],\"pregnancy carve-out\")."
, "source('test_source.map_bridge.exc.2','test_source.map_bridge',[2],\"renal carve-out\")."
, "source('test_source.map_bridge.exc.3','test_source.map_bridge',[3],\"pregnancy carve-out\")."
, "source('test_source.map_bridge.exc.4','test_source.map_bridge',[5],\"renal carve-out B\")."
, "source('test_source.map_bridge.rule.0','test_source.map_bridge',[0,1],\"rule A\")."
, "source('test_source.map_bridge.rule.1','test_source.map_bridge',[4],\"rule B\")."
, "strength('test_source.map_bridge.rule.0',strong)."
, "strength('test_source.map_bridge.rule.1',strong)."
]).

% golden_bytes(+Lines, -Bytes) — the writer framing (kb_writer_tests's helper): lines joined by '\n',
% one trailing '\n', as a string.
golden_bytes(Lines, Bytes) :-
    atomic_list_concat(Lines, '\n', Joined),
    atom_concat(Joined, '\n', WithTrailer),
    atom_string(WithTrailer, Bytes).

% same_kb(+Facts, +Expected) — order-insensitive, duplicate-SENSITIVE multiset equality (msort keeps
% duplicates, so a stray duplicate fact fails; matches map-core's same_kb).
same_kb(Facts, Expected) :- msort(Facts, M), msort(Expected, M).

rotate1([H|T], R) :- append(T, [H], R).

% single_solution(:Goal) — Goal has exactly one solution. findall collects every solution isolated
% from any enclosing choicepoint, so this gates the solution-count half of `is det` robustly where a
% deterministic/1 probe would misreport (plunit wraps a test body in catch/->, whose choicepoint the
% probe observes). A residual choicepoint on the unique solution is a separate purity property the
% code is free of but plunit cannot robustly gate, so it is verified out-of-band, not claimed here.
single_solution(Goal) :- findall(t, Goal, Sols), Sols == [t].

% multi_oracle(-Facts) — the synthetic multi document's KB, hand-authored logically grouped (the
% byte-pin is byte-sorted, this reads as rule.0's two disjuncts then rule.1) — a reviewable,
% independent oracle for the grouping / dense-ordinal / document-continuous-counter logic.
multi_oracle(
  [ rule('test_source.map_multi.rule.0','test_source.map_multi.stmt.0'),
    rule('test_source.map_multi.rule.0','test_source.map_multi.stmt.1'),
    direction('test_source.map_multi.rule.0',for),
    strength('test_source.map_multi.rule.0',strong),
    certainty('test_source.map_multi.rule.0',moderate),
    population('test_source.map_multi.stmt.0','pop.patient'),
    condition('test_source.map_multi.bind.0','test_source.map_multi.stmt.0',concept('cond.sepsis')),
    condition('test_source.map_multi.bind.1','test_source.map_multi.stmt.0',interval('q.age_years',18,closed,lower)),
    action('test_source.map_multi.stmt.0','act.administer:drug.abx_a'),
    population('test_source.map_multi.stmt.1','pop.patient'),
    condition('test_source.map_multi.bind.2','test_source.map_multi.stmt.1',concept('cond.sepsis')),
    action('test_source.map_multi.stmt.1','act.administer:drug.abx_a'),
    source('test_source.map_multi.rule.0','test_source.map_multi',[0,1],"multi first rule"),
    rule('test_source.map_multi.rule.1','test_source.map_multi.stmt.2'),
    direction('test_source.map_multi.rule.1',permit),
    strength('test_source.map_multi.rule.1',weak),
    population('test_source.map_multi.stmt.2','pop.patient'),
    condition('test_source.map_multi.bind.3','test_source.map_multi.stmt.2',concept('cond.sepsis')),
    action('test_source.map_multi.stmt.2','act.administer:drug.abx_a'),
    source('test_source.map_multi.rule.1','test_source.map_multi',[2],"multi trailing") ]).

:- begin_tests(map_emit).

% ---- byte-pin: document_bytes == the OBSERVED canonical emitter output, per document --------------
test(bytes_pinned, [forall(doc_golden(Name, Lines))]) :-
    doc(Name, DocId, Items),
    document_bytes(DocId, Items, Bytes),
    golden_bytes(Lines, Expected),
    ( Bytes == Expected
    -> true
    ;  format(user_error, "map_emit: ~w bytes mismatch~n got:~n~w~nexpected:~n~w~n",
              [Name, Bytes, Expected]), fail ).

% Every doc carries a golden and every golden a doc, with no duplicate doc name (a duplicate would
% sort-merge and leave a variant silently unpinned) — nothing silently unpinned.
test(golden_coverage) :-
    findall(N, doc(N, _, _),        Ds), sort(Ds, DS),
    findall(N, doc_golden(N, _),    Gs), sort(Gs, GS),
    assertion(DS == GS),
    length(Ds, ND), length(DS, NDS), assertion(ND == NDS).

% ---- each mapped whole-document KB is kb_kernel-valid by construction ----------------------------
test(all_valid, [forall(doc(_, DocId, Items))]) :-
    map_document(DocId, Items, Facts),
    valid_kb(Facts).

% ---- docB / control reproduce the normative kb_examples fact sets (rule-only; independent oracle) -
test(matches_kb_example) :-
    doc(doc_b, Db, Ib),  map_document(Db, Ib, Fb),  kb_example(doc_b, valid, Eb),
    assertion(same_kb(Fb, Eb)),
    doc(control, Dc, Ic), map_document(Dc, Ic, Fc), kb_example(control, valid, Ec),
    assertion(same_kb(Fc, Ec)).

% ---- multi: grouping + dense ordinals + document-continuous counters == the grouped oracle --------
test(multi_facts_oracle) :-
    doc(multi, D, I),
    map_document(D, I, Facts),
    multi_oracle(Expected),
    assertion(same_kb(Facts, Expected)).

% ---- rule ordinals are DENSE and follow first appearance, not label magnitude (5 -> 0, 2 -> 1);
%      a lone non-zero raw label densifies to rule.0 (raw_gate checks uniqueness, not zero-basing) ---
test(rule_ordinals_appearance) :-
    doc(multi, _, I),
    rule_ordinals(I, Ord),
    assertion(Ord == [5-0, 2-1]),
    assertion(rule_ordinals([item(0, rule(7, recommend, 0, none, none), _)], [7-0])).

% ---- docA's exception block compiles to its NAF guard on stmt.0 + a source duplicating the block's
%      own raw region ([1]) and basis (map-exc); the mapped KB stays kb_kernel-valid ------------------
test(exception_compiled) :-
    doc(doc_a, D, I),
    assertion(memberchk(item(_, exception(_, _, _, _), _), I)),
    map_document(D, I, Facts),
    assertion(member(exception('test_source.m1_guideline_a.exc.0',
                               'test_source.m1_guideline_a.stmt.0',
                               concept('cond.renal_severe')), Facts)),
    assertion(member(source('test_source.m1_guideline_a.exc.0',
                            'test_source.m1_guideline_a', [1],
                            "renal-impairment carve-out"), Facts)),
    valid_kb(Facts).

% ---- determinism: the same document re-emits byte-identically -------------------------------------
test(rerun_deterministic, [forall(doc(_, DocId, Items))]) :-
    document_bytes(DocId, Items, B1),
    document_bytes(DocId, Items, B2),
    assertion(B1 == B2).

% ---- the emitted fact SET is emit-order-free (kb_bytes byte-sorts) — reverse + rotate agree -------
test(emit_order_invariant, [forall(doc(_, DocId, Items))]) :-
    map_document(DocId, Items, Facts),
    kb_bytes(Facts, B0),
    reverse(Facts, Rev), kb_bytes(Rev, B1),
    rotate1(Facts, Rot), kb_bytes(Rot, B2),
    assertion(B0 == B1),
    assertion(B0 == B2).

% ---- surface-positional (NOT input-permutation invariant): swapping the two rule blocks' appearance
%      order reassigns rule.<Ord>, so the bytes DIFFER (KB.md / map-core determinism semantics) ------
test(block_order_positional) :-
    golden_drs(frame_recommend, Fr1),   golden_drs(frame_not_possible, Fp1),
    golden_drs(frame_recommend, Fr2),   golden_drs(frame_not_possible, Fp2),
    AB = [item(0, rule(0, recommend, 0, none, none), Fr1),
          item(1, rule(1, contraindicate, 0, none, none), Fp1)],
    BA = [item(0, rule(1, contraindicate, 0, none, none), Fp2),
          item(1, rule(0, recommend, 0, none, none), Fr2)],
    % dense ordinals follow appearance: the same raw label owns a DIFFERENT rule.<Ord> per order
    assertion(rule_ordinals(AB, [0-0, 1-1])),
    assertion(rule_ordinals(BA, [1-0, 0-1])),
    map_document('t.ord', AB, Fab), assertion(valid_kb(Fab)), kb_bytes(Fab, Bab),
    map_document('t.ord', BA, Fba), assertion(valid_kb(Fba)), kb_bytes(Fba, Bba),
    % rule.0 = the first-appearing block: recommend (-> for) in AB, contraindicate in BA — the real
    % signature of positional assignment (the byte inequality alone also follows from moved regions)
    assertion(member(direction('t.ord.rule.0', for), Fab)),
    assertion(member(direction('t.ord.rule.0', contraindicate), Fba)),
    assertion(Bab \== Bba).

% ---- each export yields exactly one solution on every fixture (the solution-count half of the
%      doc-comment `is det`; see single_solution/1 for why findall, not deterministic/1) -----------
test(exports_single_solution, [forall(doc(_, DocId, Items))]) :-
    single_solution(rule_ordinals(Items, _)),
    single_solution(map_document(DocId, Items, _)),
    single_solution(document_bytes(DocId, Items, _)).

% ---- the empty document (SURFACE `{ block }` / raw_gate admit ok(doc(Id, []))) is total + det: no
%      blocks, no facts, empty canonical bytes --------------------------------------------------
test(empty_document) :-
    assertion(rule_ordinals([], [])),
    map_document('t.empty', [], Facts),   assertion(Facts == []),
    document_bytes('t.empty', [], Bytes), assertion(Bytes == ""),
    single_solution(rule_ordinals([], _)),
    single_solution(map_document('t.empty', [], _)),
    single_solution(document_bytes('t.empty', [], _)).

% ---- rule -> exception -> rule: the middle exception block consumes NO base/2 counter (the exc
%      counter is separate), so the RULE layer (rule / population / condition / action / rule-source)
%      is byte-for-byte as if the exception were absent — only the exception + its own source are added,
%      and the trailing rule keeps its stmt / bind ids ---------------------------------------------
test(exception_counter_transparent) :-
    golden_drs(thread_doc_a, R0a), golden_drs(exception_body, Ex), golden_drs(frame_recommend, R1a),
    WithExc = [item(0, rule(0, recommend, 0, none, "r0"), R0a),
               item(1, exception(0, 0, none, "carve-out"), Ex),
               item(2, rule(1, recommend, 0, none, "r1"), R1a)],
    golden_drs(thread_doc_a, R0b), golden_drs(frame_recommend, R1b),
    NoExc =   [item(0, rule(0, recommend, 0, none, "r0"), R0b),
               item(2, rule(1, recommend, 0, none, "r1"), R1b)],
    map_document('t.xc', WithExc, FWith),
    map_document('t.xc', NoExc,   FNo),
    assertion(valid_kb(FWith)),
    exclude(exc_layer, FWith, FWithRules),   % drop the exception + its (exc-id) source
    assertion(same_kb(FWithRules, FNo)),     % the rule layer is untouched by the exception
    assertion(member(exception('t.xc.exc.0', 't.xc.stmt.0', concept('cond.renal_severe')), FWith)),
    assertion(member(source('t.xc.exc.0', 't.xc', [1], "carve-out"), FWith)),
    assertion(member(rule('t.xc.rule.1', 't.xc.stmt.1'), FWith)).   % trailing rule keeps stmt.1

% exc_layer(+Fact) — a fact contributed by exception compilation: an exception/3 or a source/4 whose id
% is an exception id (valid_id/2 distinguishes it from rule / stmt sources).
exc_layer(exception(_, _, _)).
exc_layer(source(Id, _, _, _)) :- valid_id(Id, exc).

% ---- the §L·pins worked exception oracle (doc bridge): the exception FACTS are stmt-major cloned with
%      a document-continuous exc counter (no per-rule / per-statement reset), and each clone's source
%      duplicates its exception block's own raw region + basis (distinct exc ids, shared region+basis) -
test(bridge_exception_facts) :-
    doc(bridge, D, I),
    map_document(D, I, Facts),
    assertion(valid_kb(Facts)),
    ExcOracle = [ exception('test_source.map_bridge.exc.0', 'test_source.map_bridge.stmt.0', concept('cond.renal_severe')),
                  exception('test_source.map_bridge.exc.1', 'test_source.map_bridge.stmt.0', concept('cond.pregnancy')),
                  exception('test_source.map_bridge.exc.2', 'test_source.map_bridge.stmt.1', concept('cond.renal_severe')),
                  exception('test_source.map_bridge.exc.3', 'test_source.map_bridge.stmt.1', concept('cond.pregnancy')),
                  exception('test_source.map_bridge.exc.4', 'test_source.map_bridge.stmt.2', concept('cond.renal_severe')) ],
    findall(E, (member(E, Facts), E = exception(_, _, _)), GotExc),
    assertion(same_kb(GotExc, ExcOracle)),   % exactly these five, so a per-rule reset (a repeat exc.0) fails
    SrcOracle = [ source('test_source.map_bridge.exc.0', 'test_source.map_bridge', [2], "renal carve-out"),
                  source('test_source.map_bridge.exc.1', 'test_source.map_bridge', [3], "pregnancy carve-out"),
                  source('test_source.map_bridge.exc.2', 'test_source.map_bridge', [2], "renal carve-out"),
                  source('test_source.map_bridge.exc.3', 'test_source.map_bridge', [3], "pregnancy carve-out"),
                  source('test_source.map_bridge.exc.4', 'test_source.map_bridge', [5], "renal carve-out B") ],
    forall(member(S, SrcOracle), assertion(member(S, Facts))).

% ---- disjuncts group across NON-adjacent items and sort by DisjIdx, not appearance: rule 5's disj1
%      (@sent 0, sepsis only) and disj0 (@sent 2, sepsis + age interval) are interleaved by rule 2 and
%      given in reversed DisjIdx order — keysort maps stmt.0 <- disj0 (interval) and stmt.1 <- disj1 -
test(disjunct_grouping_keysort) :-
    golden_drs(frame_recommend, Disj1),
    golden_drs(frame_admissible, R2),
    golden_drs(thread_doc_a, Disj0),
    Items = [item(0, rule(5, recommend, 1, moderate, "ks"), Disj1),
             item(1, rule(2, recommend, 0, none, "ks2"),    R2),
             item(2, rule(5, recommend, 0, moderate, "ks"), Disj0)],
    map_document('t.ks', Items, Facts),
    assertion(valid_kb(Facts)),
    assertion(member(condition(_, 't.ks.stmt.0', interval(_, _, _, _)), Facts)),
    assertion(\+ member(condition(_, 't.ks.stmt.1', interval(_, _, _, _)), Facts)),
    assertion(member(rule('t.ks.rule.0', 't.ks.stmt.0'), Facts)),
    assertion(member(rule('t.ks.rule.0', 't.ks.stmt.1'), Facts)),
    assertion(member(rule('t.ks.rule.1', 't.ks.stmt.2'), Facts)),
    assertion(member(source('t.ks.rule.0', _, [0, 2], _), Facts)).

:- end_tests(map_emit).
