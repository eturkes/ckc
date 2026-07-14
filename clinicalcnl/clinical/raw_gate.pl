% ClinicalCNL raw gate — the WHITELIST layer BEFORE APE (M3.raw-gate; SPEC §10.6, SURFACE.md).
%
% The first of the three fail-closed layers (SURFACE.md §seam): a registry-driven whitelist over
% the raw ClinicalCNL v1 document bytes. Two DCG stages:
%
%   (1) framing — a byte DCG splits the document into blank-separated paragraphs, then a header DCG
%       parses each block header (rule/exception id, modality keyword, certainty + basis fields).
%   (2) per-sentence templates — each rule-disjunct / exception ACE line is tokenised and matched
%       against the registered-pattern sentence grammar. The grammar admits ONLY registered surfaces
%       (reg_concept / pn_allow / reg_frame / the 4 interval markers) + the closed frame/function
%       words, so any unregistered token, prefix (`n:`), capitalised OOV (the p6 named() hole),
%       `or`/`every`/quantifier surface, decimal, or malformed frame fails the whitelist.
%
% Output (accept) = ok(doc(DocId, Sentences)), Sentences an ordered list of sentence(Idx, Ace, Ctx):
%   Idx  — 0-based raw sentence ordinal (per-sentence APE dispatch index / provenance sentence_index).
%   Ace  — the verbatim ACE sentence atom dispatched to APE (byte-identical to a surface_cases oracle).
%   Ctx  — rule(K, Keyword, DisjIdx, Cert, Basis) | exception(K, RuleK, Cert, Basis) (block context).
% Output (reject) = reject(Rejects), each reject(Idx, Token, Construct) naming the offending locus.
% Fail-closed + total: any reject rejects the whole document; the predicate never throws.
%
% Per-sentence (D2): each ACE sentence is emitted alone for a solo APE parse, structurally killing
% cross-sentence referent merging. The raw gate does not run APE (that is profile-drs / map-core);
% it only whitelists the surface + cross-checks the frame op against the header keyword's op (D1).
%
%   Gate: swipl -q -g "consult('clinical/raw_gate_tests.pl'),(run_tests(raw_gate)->halt(0);halt(1))" -t 'halt(1)'

:- module(raw_gate,
          [ gate_document/2        % +Doc(atom|string|codes), -Result(ok(doc(Id,Sents)) | reject(Rejects))
          ]).

% The registry is the whitelist authority; loaded source-relative so the module is cwd-independent.
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/registry.pl'], R), use_module(R).

% ==========================================================================================
% Top level. Reject forbidden whitespace up front (strict LF discipline), else split into
% paragraphs and classify. A structural parse failure is itself a fail-closed reject.
% ==========================================================================================

gate_document(In, Result) :-
    to_codes(In, Codes),
    (   forbidden_char(Codes, Con)
    ->  Result = reject([reject(-1, '', Con)])
    ;   phrase(paragraphs(Paras), Codes)
    ->  classify(Paras, Result)
    ;   Result = reject([reject(-1, '', bad_structure)])
    ).

to_codes(Codes, Codes) :- is_list(Codes), !.
to_codes(In, Codes)    :- string(In), !, string_codes(In, Codes).
to_codes(In, Codes)    :- atom(In), atom_codes(In, Codes).

% CR (CRLF) and TAB are never valid v1 whitespace — fail closed rather than lex around them.
forbidden_char(Codes, carriage_return) :- memberchk(0'\r, Codes), !.
forbidden_char(Codes, tab)             :- memberchk(0'\t, Codes).

% ==========================================================================================
% Stage 1a — framing byte DCG. A document is line-oriented; blank lines separate paragraphs. The
% first paragraph is the single `document <doc-id>` line, each later paragraph a block:
% [header-line | content-line...]. Content lines are captured verbatim (validated in stage 2).
% ==========================================================================================

% paragraphs(-Paras) — Paras is the list of paragraphs (each a non-empty list of line atoms);
% leading / trailing / separating blank lines are dropped.
paragraphs(Ps) --> blanks, paras(Ps).

paras([P|Ps]) --> paragraph(P), !, blanks, paras(Ps).
paras([])     --> [].

paragraph([L|Ls]) --> nonblank_line(L), para_rest(Ls).
para_rest([L|Ls]) --> nonblank_line(L), !, para_rest(Ls).
para_rest([])     --> [].

nonblank_line(A) --> line(Cs), { Cs \== [], atom_codes(A, Cs) }.

blanks --> blank_line, !, blanks.
blanks --> [].
blank_line --> line(Cs), { Cs == [] }.

% line(-Cs) — one line's codes (sans newline). LF-terminated, or the final non-empty line at EOS.
line(Cs) --> line_codes(Cs), "\n", !.
line(Cs) --> line_codes(Cs), at_eos, { Cs \== [] }.
line_codes([C|Cs]) --> [C], { C =\= 0'\n }, !, line_codes(Cs).
line_codes([])     --> [].
at_eos([], []).

% ==========================================================================================
% Stage 1b — document + header classification. Extract the doc id, then walk the blocks.
% ==========================================================================================

classify([], reject([reject(-1, '', empty_document)])) :- !.
classify([DocPara|BlockParas], Result) :-
    (   DocPara = [DocLine],
        atom_concat('document ', DocId, DocLine),
        DocId \== '',
        \+ sub_atom(DocId, _, _, _, ' ')
    ->  walk_blocks(BlockParas, 0, Sents, Rejs),
        ( Rejs == [] -> Result = ok(doc(DocId, Sents)) ; Result = reject(Rejs) )
    ;   Result = reject([reject(-1, '', bad_document_header)])
    ).

% walk_blocks(+Blocks, +Idx0, -Sentences, -Rejects) — thread the running raw-sentence index; a
% block contributes its sentences (accepts) and rejects (any reject fails the document downstream).
walk_blocks([], _, [], []).
walk_blocks([Block|Bs], Idx0, Sents, Rejs) :-
    validate_block(Block, Idx0, Idx1, S, R),
    walk_blocks(Bs, Idx1, Ss, Rs),
    append(S, Ss, Sents),
    append(R, Rs, Rejs).

% validate_block(+[Header|Content], +Idx0, -Idx1, -Sentences, -Rejects).
validate_block([HeaderLine|Content], Idx0, Idx1, Sentences, Rejects) :-
    atom_codes(HeaderLine, HCs),
    (   phrase(header(H), HCs)
    ->  block_kind(H, Content, Idx0, Idx1, Sentences, Rejects)
    ;   Idx1 = Idx0, Sentences = [], Rejects = [reject(Idx0, HeaderLine, bad_header)]
    ).

block_kind(rule(K, Kw, Fields), Content, Idx0, Idx1, Sentences, Rejects) :- !,
    header_fields(rule(K, Kw, Fields), Cert, Basis, HErrs),
    (   HErrs \== []
    ->  Idx1 = Idx0, Sentences = [], herrs_rejects(HErrs, Idx0, Rejects)
    ;   Content == []
    ->  Idx1 = Idx0, Sentences = [], Rejects = [reject(Idx0, '', empty_block)]
    ;   disjuncts(Content, K, Kw, Cert, Basis, 0, Idx0, Idx1, Sentences, Rejects)
    ).
block_kind(exc(K, RuleK, Fields), Content, Idx0, Idx1, Sentences, Rejects) :- !,
    header_fields(exc(K, RuleK, Fields), Cert, Basis, HErrs),
    (   HErrs \== []
    ->  Idx1 = Idx0, Sentences = [], herrs_rejects(HErrs, Idx0, Rejects)
    ;   Content = [Line]
    ->  Idx1 is Idx0 + 1,
        ( classify_line(exc_sentence, Line, Idx0, sent)
        ->  Sentences = [sentence(Idx0, Line, exception(K, RuleK, Cert, Basis))], Rejects = []
        ;   classify_line(exc_sentence, Line, Idx0, rej(Rej)),
            Sentences = [], Rejects = [Rej]
        )
    ;   Content == []
    ->  Idx1 = Idx0, Sentences = [], Rejects = [reject(Idx0, '', empty_block)]
    ;   Idx1 = Idx0, Sentences = [], Rejects = [reject(Idx0, '', multi_line_exception)]
    ).

% disjuncts(+Lines, +K, +Kw, +Cert, +Basis, +DisjIdx, +Idx0, -IdxN, -Sentences, -Rejects) — each
% content line of a rule block is a DNF disjunct sharing rule id K, mapped to stmt.DisjIdx (D4). The
% raw sentence index advances per line (even on a rejected line, so an index names a stable locus).
disjuncts([], _, _, _, _, _, Idx, Idx, [], []).
disjuncts([Line|Ls], K, Kw, Cert, Basis, Dj, Idx0, IdxN, Sentences, Rejects) :-
    Idx1 is Idx0 + 1, Dj1 is Dj + 1,
    (   classify_line(rule_sentence(Op), Line, Idx0, sent)
    ->  ( reg_keyword(Kw, Op, _, _)
        ->  Sentences = [sentence(Idx0, Line, rule(K, Kw, Dj, Cert, Basis))|Ss], Rejs = Rs
        ;   Sentences = Ss, Rejs = [reject(Idx0, Kw, op_mismatch)|Rs]
        )
    ;   classify_line(rule_sentence(_), Line, Idx0, rej(Rej)),
        Sentences = Ss, Rejs = [Rej|Rs]
    ),
    disjuncts(Ls, K, Kw, Cert, Basis, Dj1, Idx1, IdxN, Ss, Rs),
    Rejects = Rejs.

% ==========================================================================================
% Header DCG + field validation. Header line = `rule K keyword {field}` | `exception K rule K {field}`;
% field = ` certainty CERT` | ` basis "STRING"`. K is a non-negative int (no leading zero).
% ==========================================================================================

header(rule(K, Kw, Fields)) -->
    "rule ", int_dcg(K), " ", word_dcg(KwCs), { atom_codes(Kw, KwCs) }, fields(Fields).
header(exc(K, RuleK, Fields)) -->
    "exception ", int_dcg(K), " rule ", int_dcg(RuleK), fields(Fields).

fields([f(certainty, C)|Fs]) --> " certainty ", word_dcg(Ccs), { atom_codes(C, Ccs) }, fields(Fs).
fields([f(basis, B)|Fs])     --> " basis ", qstring(B), fields(Fs).
fields([])                   --> [].

int_dcg(N) --> digits(Ds), { Ds \== [], no_leading_zero(Ds), number_codes(N, Ds) }.
digits([D|Ds]) --> [D], { code_type(D, digit) }, !, digits(Ds).
digits([]) --> [].

word_dcg([C|Cs]) --> [C], { C =\= 0'\s }, !, word_dcg(Cs).
word_dcg([]) --> [].

qstring(Atom) --> [0'"], qchars(Cs), [0'"], { atom_codes(Atom, Cs) }.
qchars([C|Cs]) --> [C], { C =\= 0'" }, !, qchars(Cs).
qchars([]) --> [].

% header_fields(+Header, -Cert, -Basis, -Errs) — keyword membership (rule blocks) + at-most-once
% certainty (∈ D7 set) / basis. Errs a list of Key-Value pairs, [] iff the header is well-formed.
header_fields(rule(_, Kw, Fields), Cert, Basis, Errs) :-
    ( reg_keyword(Kw, _, _, _) -> KwE = [] ; KwE = [bad_keyword-Kw] ),
    field_values(Fields, Cert, Basis, FE),
    append(KwE, FE, Errs).
header_fields(exc(_, _, Fields), Cert, Basis, Errs) :-
    field_values(Fields, Cert, Basis, Errs).

field_values(Fields, Cert, Basis, Errs) :-
    findall(C, member(f(certainty, C), Fields), Cs),
    findall(B, member(f(basis, B), Fields), Bs),
    cert_field(Cs, Cert, CE),
    basis_field(Bs, Basis, BE),
    append(CE, BE, Errs).

cert_field([], none, []) :- !.
cert_field([C], C, []) :- valid_cert(C), !.
cert_field([C], none, [bad_certainty-C]) :- !.
cert_field([_,_|_], none, [duplicate_field-certainty]).

basis_field([], none, []) :- !.
basis_field([B], B, []) :- !.
basis_field([_,_|_], none, [duplicate_field-basis]).

valid_cert(high). valid_cert(moderate). valid_cert(low). valid_cert(very_low).

herrs_rejects(HErrs, Idx, Rejects) :-
    findall(reject(Idx, V, K), member(K-V, HErrs), Rejects).

% ==========================================================================================
% Stage 2 — per-sentence whitelist. Tokenise the ACE line, match the registered-pattern grammar;
% on failure, diagnose the first offending token. classify_line/4 returns sent | rej(Reject).
% ==========================================================================================

classify_line(Grammar, Line, Idx, Result) :-
    (   line_tokens(Line, Tokens)
    ->  ( phrase(Grammar, Tokens)
        ->  Result = sent
        ;   diagnose(Tokens, Tok, Con), Result = rej(reject(Idx, Tok, Con))
        )
    ;   line_reject(Line, Idx, Result)
    ).

% line_tokens(+Line, -Tokens) — Line ends with `.`; tokens are its single-space-separated words
% (hyphens stay inside a token). Fails on a missing period or non-single spacing (line_reject names it).
line_tokens(Line, Tokens) :-
    atom_concat(Body, '.', Line),
    split_string(Body, " ", "", Parts),
    \+ memberchk("", Parts),
    maplist(str_atom, Parts, Tokens).
str_atom(S, A) :- atom_string(A, S).

line_reject(Line, Idx, rej(reject(Idx, Line, no_period))) :- \+ atom_concat(_, '.', Line), !.
line_reject(_,    Idx, rej(reject(Idx, '', whitespace))).

% diagnose(+Tokens, -Token, -Construct) — the first token outside the whitelist lexicon names the
% reject; if every token is a legal lexeme the failure is structural (order / arity).
diagnose(Tokens, Tok, Con) :-
    ( first_illegal(Tokens, Bad)
    ->  Tok = Bad, illegal_kind(Bad, Con)
    ;   Tok = '', Con = malformed_sentence
    ).
first_illegal([T|_], T)  :- \+ legal_lexeme(T), !.
first_illegal([_|Ts], T) :- first_illegal(Ts, T).

illegal_kind(Bad, unregistered_capital) :- capitalized(Bad), \+ pn_allow(Bad), !.
illegal_kind(Bad, bad_number)           :- starts_digit(Bad), \+ valid_int_atom(Bad), !.
illegal_kind(_,   unregistered_token).

capitalized(A)  :- atom_codes(A, [C|_]), code_type(C, upper).
starts_digit(A) :- atom_codes(A, [C|_]), code_type(C, digit).

% ==========================================================================================
% The registered-pattern sentence grammar (token-level DCG = the whitelist). Frames + surfaces are
% derived from the registry (no duplicated tables); the closed frame/function/marker words are the
% only literals. A rule sentence: `If <guard> then <frame> <action>`; an exception: `A patient has a <cond>`.
% ==========================================================================================

rule_sentence(Op) --> ['If'], guard(first), [then], frame_clause(Op).

guard(Pos) --> conjunct(Pos), guard_rest.
guard_rest --> [and], !, conjunct(later), guard_rest.
guard_rest --> [].

conjunct(Pos) --> subject(Pos), [has], guard_body.
guard_body --> [a], concept_noun.
guard_body --> [an, age, of], interval_marker, int_token, [years].

% Subject anaphora (D3): the population is introduced once as `a patient`; every later conjunct is
% the definite `the patient` (the frame action re-uses `the patient` too, inside frame_clause).
subject(first) --> [a, patient].
subject(later) --> [the, patient].

concept_noun --> [W], { reg_concept(_, W) }.

% The 4 v1 interval markers (D9); `exactly` / bare-eq are non-v1 and are absent by construction.
interval_marker --> [at, least].
interval_marker --> [more, than].
interval_marker --> [at, most].
interval_marker --> [less, than].

int_token --> [N], { valid_int_atom(N) }.

frame_clause(Op) --> frame_phrase(Op), [the, patient, takes], drug.
frame_phrase(Op) --> { reg_frame(Op, Phrase), atomic_list_concat(Ws, ' ', Phrase) }, seq(Ws).
drug --> [D], { pn_allow(D) }.

exc_sentence --> ['A', patient, has, a], concept_noun.

% seq(+Terminals) — match a list of token terminals in order.
seq([]) --> [].
seq([T|Ts]) --> [T], seq(Ts).

% ==========================================================================================
% Whitelist lexicon (diagnostics only — the grammar above is the real gate). A token is legal iff a
% registry surface, the drug allowlist, a valid integer, or a closed frame/function/marker word.
% ==========================================================================================

legal_lexeme(W) :- reg_concept(_, W).
legal_lexeme(W) :- reg_population(_, W).
legal_lexeme(W) :- reg_guard_verb(W, _, _).
legal_lexeme(W) :- reg_action(_, W, _, _).
legal_lexeme(W) :- reg_quantity(_, W, _, _, _).
legal_lexeme(W) :- reg_quantity(_, _, W, _, _).
legal_lexeme(W) :- reg_quantity(_, _, _, W, _).
legal_lexeme(D) :- pn_allow(D).
legal_lexeme(N) :- valid_int_atom(N).
legal_lexeme(W) :- fixed_word(W).

fixed_word('If').  fixed_word('A').   fixed_word(then).        fixed_word(and).
fixed_word(a).     fixed_word(an).    fixed_word(the).         fixed_word(it).
fixed_word(is).    fixed_word(that).  fixed_word(of).          fixed_word(not).
fixed_word(recommended). fixed_word(admissible). fixed_word(possible).
fixed_word(at).    fixed_word(least). fixed_word(most).
fixed_word(more).  fixed_word(less).  fixed_word(than).

% valid_int_atom(+A) — a non-negative integer atom with no leading zero (`0` alone is legal).
valid_int_atom(A) :- atom(A), atom_codes(A, Cs), Cs \== [], all_digits(Cs), no_leading_zero(Cs).
all_digits([C|Cs]) :- code_type(C, digit), all_digits(Cs).
all_digits([]).
no_leading_zero([0'0]) :- !.
no_leading_zero([C|_]) :- C =\= 0'0.
