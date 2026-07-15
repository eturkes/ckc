% ClinicalCNL whole-document DRS -> KB driver + canonical emission (M3.map-emit; SPEC §10.6, KB.md).
% The whole-document stage over map-core: it drives drs_map:map_rule/6 across a profile-validated
% document's rule blocks — assigning each block its DENSE 0-based rule ordinal (raw_gate guarantees a
% block label is UNIQUE but not dense/ordered, so map-emit owns rule.<Ord> canonicalisation, KB.md),
% threading the document-continuous base(StmtIdx, BindIdx) counters across the blocks, and collecting
% the KB facts — then serialises them to canonical bytes via kb_kernel:kb_bytes/2.
%
% Input = a list of item(SentIdx, Ctx, Drs): one profile-validated sentence — SentIdx its raw ordinal
% (provenance), Ctx the raw-gate block context (rule(K, Keyword, DisjIdx, Cert, Basis) |
% exception(K, RuleK, Cert, Basis)), Drs its APE DRS. map-core ASSUMES profile-validity and EXTRACTS,
% so map-emit likewise trusts its input; the fact output is kb_kernel-valid by construction (the caller
% / gate runs valid_kb/1 — kb_bytes/2 does not validate, KB.md).
%
% Exception blocks compile to NAF-guarded PROLEG overrides (map-exc). A labeled exception is a bare
% condition body (D6) whose single concept guards its referenced rule's advice. exception(ExcId, StmtId,
% Atom) keys on the STATEMENT, so an exception is cloned across every statement of the rule it references
% -- one exc.<k> fact per (statement, exception) pair, emitted stmt-major (stmt.0's exceptions in
% appearance order, then stmt.1's, ...), the exc counter document-continuous and NEVER reset per rule.
% Each clone's source duplicates the exception block's own raw sentence index + basis. rule_ordinals/2
% stays exposed (the raw-label -> dense-ordinal map).
%
% The gate (map_emit_tests.pl) builds each document's items from the byte-pinned surface goldens
% (read-back DRS) + hand-built raw contexts, so map-emit runs pure and fast with no live APE.
%
%   Gate: swipl -q -g "consult('clinical/map_emit_tests.pl'),(run_tests(map_emit)->halt(0);halt(1))" -t 'halt(1)'

:- module(map_emit, [ map_document/3, document_bytes/3, rule_ordinals/2 ]).

% map_rule/6 = the per-rule-block mapper (map-core); kb_bytes/2 = the canonical emitter (kb-writer).
% Loaded source-relative so the module is cwd-independent (mirrors drs_map.pl / profile_check.pl).
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/drs_map.pl'], M), use_module(M),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K),
   atomic_list_concat([D, '/registry.pl'], R), use_module(R).

%% document_bytes(+DocId, +Items, -Bytes) is det.
% The whole pipeline tail: map a document's rule blocks to KB facts, then serialise to canonical
% UTF-8/LF bytes (a string). Bytes is a function of the fact SET alone (kb_bytes/2 byte-sorts), so the
% emit order of map_document/3's facts is irrelevant.
document_bytes(DocId, Items, Bytes) :-
    map_document(DocId, Items, Facts),
    kb_bytes(Facts, Bytes).

%% map_document(+DocId, +Items, -Facts) is det.
% Map a whole document's rule + exception blocks to their KB fact terms, ids document-continuous.
% Groups the rule items by raw label, visits the blocks in first-appearance (physical) order under
% their dense ordinals, threads base(StmtIdx, BindIdx) from base(0, 0) through map_rule/6, and after
% each rule block clones its exceptions across the block's statements (a separate document-continuous
% exc counter). Emit order is unconstrained (kb_bytes/2 sorts).
map_document(DocId, Items, Facts) :-
    rule_ordinals(Items, OrdMap),
    map_blocks(OrdMap, DocId, Items, base(0, 0), 0, Facts).

%% rule_ordinals(+Items, -OrdMap) is det.
% OrdMap = [K-Ord, ...]: each raw rule label K paired with its DENSE 0-based ordinal, in
% first-appearance (physical document) order. raw_gate checks label uniqueness only (not density or
% order), so map-emit assigns the canonical rule.<Ord>; map-exc resolves an exception's RuleK through
% this map.
rule_ordinals(Items, OrdMap) :-
    findall(K, member(item(_, rule(K, _, _, _, _), _), Items), Ks),
    list_to_set(Ks, Labels),
    enum(Labels, 0, OrdMap).

enum([], _, []).
enum([L|Ls], N, [L-N|Rest]) :- N1 is N + 1, enum(Ls, N1, Rest).

% map_blocks(+OrdMap, +DocId, +Items, +Base0, +Exc0, -Facts) — fold across the rule blocks in ordinal
% order. Each block maps its rule (map_rule/6, advancing base/2) then clones its exceptions across the
% statements that block minted (advancing the separate exc counter). Both counters thread block to block.
map_blocks([], _DocId, _Items, _Base, _Exc, []).
map_blocks([K-Ord|Rest], DocId, Items, Base0, Exc0, Facts) :-
    block_header(K, Items, Keyword, Cert, Basis),
    block_disjuncts(K, Items, Disjuncts),
    map_rule(DocId, rule_header(Ord, Keyword, Cert, Basis), Disjuncts, Base0, RuleFacts, Base1),
    stmt_ids(DocId, Base0, Base1, StmtIds),
    block_exceptions(K, Items, Excs),
    clone_exceptions(StmtIds, Excs, DocId, Exc0, ExcFacts, Exc1),
    map_blocks(Rest, DocId, Items, Base1, Exc1, RestFacts),
    append([RuleFacts, ExcFacts, RestFacts], Facts).

% block_header(+K, +Items, -Keyword, -Cert, -Basis) — the rule block's shared header fields. raw_gate
% parses one header line per block, so every disjunct of label K carries the same Keyword/Cert/Basis;
% take the first item's.
block_header(K, Items, Keyword, Cert, Basis) :-
    memberchk(item(_, rule(K, Keyword, _, Cert, Basis), _), Items).

% block_disjuncts(+K, +Items, -Disjuncts) — the block's disjuncts as [disj(SentIdx, Drs), ...] ordered
% by DisjIdx (D4 disjunct numbering; keysort is stable and DisjIdx is unique within a block), each
% carrying its raw sentence index for map_rule/6's provenance regions.
block_disjuncts(K, Items, Disjuncts) :-
    findall(DisjIdx-disj(SentIdx, Drs),
            member(item(SentIdx, rule(K, _, DisjIdx, _, _), Drs), Items),
            Pairs),
    keysort(Pairs, Sorted),
    pairs_values(Sorted, Disjuncts).

% ==========================================================================================
% Exception blocks -> NAF-guarded PROLEG overrides (map-exc).
% ==========================================================================================

% stmt_ids(+DocId, +Base0, +Base1, -StmtIds) — the statement ids map_rule/6 minted for one rule block,
% ascending: the stmt counter ran from Base0's index up to (not including) Base1's, one per disjunct.
% These are the statements the block's exceptions clone across.
stmt_ids(DocId, base(S0, _), base(S1, _), StmtIds) :-
    ( S0 < S1 -> Hi is S1 - 1, numlist(S0, Hi, Ords) ; Ords = [] ),
    findall(Id, ( member(N, Ords), mk_id(DocId, stmt, N, Id) ), StmtIds).

% block_exceptions(+K, +Items, -Excs) — the exception blocks referencing rule label K, ordered by raw
% sentence index (first appearance). Each is exc(SentIdx, Basis, Drs): its raw ordinal (provenance),
% its evidence basis (a string | none), and its bare-body DRS. raw_gate rejects a dangling reference,
% so every exception item's RuleK resolves to some rule label visited here.
block_exceptions(K, Items, Excs) :-
    findall(SentIdx-exc(SentIdx, Basis, Drs),
            member(item(SentIdx, exception(_, K, _, Basis), Drs), Items),
            Pairs),
    keysort(Pairs, Sorted),
    pairs_values(Sorted, Excs).

% clone_exceptions(+StmtIds, +Excs, +DocId, +Exc0, -Facts, -Exc1) — clone the block's exceptions across
% its statements, stmt-major: for each statement (ascending), one exc.<k> per exception (appearance
% order), the exc counter running Exc0 .. Exc1 without resetting per statement. A 2-statement rule with
% exceptions [ea, eb] thus mints exc.0/exc.1 on stmt.0 then exc.2/exc.3 on stmt.1.
clone_exceptions([], _Excs, _DocId, Exc, [], Exc).
clone_exceptions([StmtId|Ss], Excs, DocId, Exc0, Facts, Exc) :-
    stmt_exceptions(Excs, StmtId, DocId, Exc0, F0, Exc1),
    clone_exceptions(Ss, Excs, DocId, Exc1, F1, Exc),
    append(F0, F1, Facts).

% stmt_exceptions(+Excs, +StmtId, +DocId, +Exc0, -Facts, -Exc1) — one statement's exception facts: per
% exception, an exception(exc.<k>, StmtId, Atom) NAF guard + its source (the exception's own raw region
% + basis, duplicated onto this clone), advancing the counter. The list is built before the recursive
% call (the tail call stays last, opaque to head-arg folding — the SWI clause-compile quirk).
stmt_exceptions([], _StmtId, _DocId, Exc, [], Exc).
stmt_exceptions([exc(SentIdx, Basis, Drs)|Es], StmtId, DocId, Exc0, Facts, Exc) :-
    mk_id(DocId, exc, Exc0, ExcId),
    exception_atom(Drs, Atom),
    Exc1 is Exc0 + 1,
    Facts = [ exception(ExcId, StmtId, Atom),
              source(ExcId, DocId, [SentIdx], Basis) | Rest ],
    stmt_exceptions(Es, StmtId, DocId, Exc1, Rest, Exc).

% exception_atom(+Drs, -Atom) — the exception body's context atom. profile_check proved the body is a
% single concept object wired to the population by have (D6, interval-/op-free), so the concept object
% (its noun a registered concept, distinct from the population noun) inverts to concept(ConceptId).
exception_atom(drs(_, Conds), concept(CId)) :-
    member(object(_, N, countable, na, eq, 1)-_, Conds),
    reg_concept(CId, N), !.

% mk_id(+Doc, +Kind, +N, -Id) — a document-qualified id <Doc>.<Kind>.<N> (kb_kernel:valid_id/2); mirrors
% drs_map's constructor for the stmt / exc ids assigned here.
mk_id(Doc, Kind, N, Id) :- atomic_list_concat([Doc, '.', Kind, '.', N], Id).
