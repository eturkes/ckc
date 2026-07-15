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
% Exception blocks are map-exc's concern: map-emit emits only RULE facts and exposes rule_ordinals/2
% (the raw-label -> dense-ordinal map) so map-exc can resolve an exception's `rule RuleK` reference and
% bind its exc.<k> facts to the right rule's statements. The exc counter is separate from base/2
% (stmt/bind), so leaving exception items uncompiled keeps the rule-block counter threading exact.
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
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K).

%% document_bytes(+DocId, +Items, -Bytes) is det.
% The whole pipeline tail: map a document's rule blocks to KB facts, then serialise to canonical
% UTF-8/LF bytes (a string). Bytes is a function of the fact SET alone (kb_bytes/2 byte-sorts), so the
% emit order of map_document/3's facts is irrelevant.
document_bytes(DocId, Items, Bytes) :-
    map_document(DocId, Items, Facts),
    kb_bytes(Facts, Bytes).

%% map_document(+DocId, +Items, -Facts) is det.
% Map a whole document's RULE blocks to their KB fact terms, ids document-continuous. Groups the rule
% items by raw label, visits the blocks in first-appearance (physical) order under their dense
% ordinals, and threads base(StmtIdx, BindIdx) from base(0, 0) through map_rule/6. Facts EXCLUDES
% exceptions (map-exc). Emit order is unconstrained (kb_bytes/2 sorts).
map_document(DocId, Items, Facts) :-
    rule_ordinals(Items, OrdMap),
    map_blocks(OrdMap, DocId, Items, base(0, 0), Facts).

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

% map_blocks(+OrdMap, +DocId, +Items, +Base0, -Facts) — fold map_rule/6 across the rule blocks in
% ordinal order, threading the base/2 counters block to block.
map_blocks([], _DocId, _Items, _Base, []).
map_blocks([K-Ord|Rest], DocId, Items, Base0, Facts) :-
    block_header(K, Items, Keyword, Cert, Basis),
    block_disjuncts(K, Items, Disjuncts),
    map_rule(DocId, rule_header(Ord, Keyword, Cert, Basis), Disjuncts, Base0, F0, Base1),
    map_blocks(Rest, DocId, Items, Base1, F1),
    append(F0, F1, Facts).

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
