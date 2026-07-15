% ClinicalCNL raw-gate mutation reject battery (M3.raw-gate-battery; SPEC §10.6, SURFACE.md).
%
% The systematic per-hazard mutation matrix over raw_gate:gate_document/2. raw_gate_tests.pl carries the
% core suite (accept cross-checks + one sole-diagnosis reject per reachable gate Construct); this unit
% adds mutation DEPTH: one mutant CLASS per banked hazard (the roadmap §M3 raw-gate-battery unit line +
% SURFACE.md §Modality's op-mismatch battery), each mutant a single-locus edit of a proven-valid base
% asserting the exact reject(Idx, Token, Construct). The banked set is the whitelist / sentence / id-
% integrity hazard surface; the framing/field Constructs (carriage_return, tab, whitespace, empty_document,
% empty_block, no_period, bad_certainty, duplicate_field, multi_line_exception) are the core suite's sole-
% diagnosis rejects — together the two files exercise every reachable reject Construct. bad_structure is
% the one unreachable defensive branch: paragraphs//1 is total over any scalar code list, so its reject is
% dead by construction (not a coverage gap).
%
% Anti-vacuity is PER-MUTANT, not sampled: every case carries its EXACT accepted base and bases_accept
% proves each base maps to ok(_), so a reject is the mutation's doing — a base that silently stopped
% accepting (e.g. a year_noun(1) regression breaking valid `1 year` alongside the `1 years` mutant, or a
% code_list/1 regression breaking valid code-list input) fails bases_accept rather than hiding behind a
% still-red mutant. Five accept controls pin the canonical accept shapes to their exact sentence lists and
% cover all three input encodings (atom / string / code list). Two self-checks bind the matrix to the
% banked-hazard set (every hazard covered, no unbanked class typo).
%
% The op-mismatch battery is the FULL registry-derived keyword × frame-op Cartesian (every one of the 18
% mismatched pairs rejects op_mismatch, each of the 6 matching pairs is its own accepted base) rather than
% a hand-picked sample. Constructs are OBSERVED by running each mutant (never assumed; memory: never assert
% "every X rejects" from a partial probe). Pure Prolog — raw_gate whitelists BEFORE APE and never runs it.
%
%   Gate: swipl -q -g "consult('clinical/raw_gate_battery_tests.pl'),(run_tests(raw_gate_battery)->halt(0);halt(1))" -t 'halt(1)'

:- module(raw_gate_battery_tests, []).

:- use_module(library(plunit)).

% Load the gate + the registry (the op × frame-op authority for the op-mismatch Cartesian), source-
% relative + cwd-independent (mirrors raw_gate_tests.pl). raw_gate loads the registry too; use_module
% is idempotent, so both import the one registry module.
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/raw_gate.pl'], G),  use_module(G),
   atomic_list_concat([D, '/registry.pl'], Rg), use_module(Rg).

% mono(+HeaderRest, +Ace, -Doc) — a minimal one-block document (`document d` + one rule/exception block
% of a single ACE line) around a single mutated (or base) sentence. The multi-block / document-level /
% term-input hazards carry their own base+mutant documents in raw_case/5 instead.
mono(Hdr, Ace, Doc) :- atomic_list_concat(['document d\n\n', Hdr, '\n', Ace, '\n'], Doc).

% canondoc(-Doc) — the canonical accepted one-rule document; the shared accept base + input-shape control.
canondoc('document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n').

% expand(+Spec, -Input) — resolve a base/mutant Spec to the raw gate input. `canon` / `canoncodes` are the
% canonical accepted document as an atom / a code list (bad_input's valid counterfactuals); any other term
% (a full document atom, or a non-document term for the bad_input mutants) passes through verbatim.
expand(canon, Doc)       :- !, canondoc(Doc).
expand(canoncodes, Codes):- !, canondoc(A), atom_codes(A, Codes).
expand(Term, Term).

% ==========================================================================================
% banked_hazard(?Class) — the closed set of hazard classes the battery MUST cover: the roadmap §M3
% raw-gate-battery unit line's enumeration + op_mismatch (SURFACE.md §Modality assigns the op-mismatch
% reject battery to this unit). covers_every_banked_hazard / no_unbanked_class pin battery_case to it.
% ==========================================================================================

banked_hazard(capitalized_oov).          % p6 named() hole — a capital OOV in a lexical slot
banked_hazard(prefix_token).             % an APE word-class prefix n: / v: / a: / p:
banked_hazard(pronoun).                  % a free pronoun beyond the frame `It`
banked_hazard(or_guard).                 % a disjunctive `or`-guard (D4 — one sentence per disjunct)
banked_hazard(every_surface).            % a universal quantifier surface
banked_hazard(bare_then).                % a then-clause with no modality frame
banked_hazard(does_not).                 % D5 in-guard negation (v1 negatives enter via exceptions)
banked_hazard(cross_sentence_definite).  % a later disjunct opening on `the patient` (D2 kills merge)
banked_hazard(no_antecedent_definite).   % a lone sentence opening on the definite `the patient`
banked_hazard(decimal).                  % a non-integer interval value
banked_hazard(leading_zero).             % a leading-zero integer (interval value or block id)
banked_hazard(spaced_multiword).         % a hyphenated term written space-separated
banked_hazard(unregistered_keyword).     % a modality keyword outside the 6 registered
banked_hazard(duplicate_rule_id).        % two rule blocks sharing an id
banked_hazard(duplicate_exception_id).   % two exception blocks sharing an id
banked_hazard(dangling_exception).       % an exception referencing an undeclared rule
banked_hazard(number_agreement).         % a value/unit-noun disagreement (`1 years` / `2 year`)
banked_hazard(bad_input).                % a non-atom/string/scalar-code-list input term
banked_hazard(bad_doc_id).               % a malformed document id
banked_hazard(exactly_eq_marker).        % the non-v1 `exactly` / bare-eq interval markers
banked_hazard(missing_header).           % a block whose first line is not a rule/exception header
banked_hazard(ace_comment).              % a comment token in the surface
banked_hazard(quotation).                % a quoted term in the surface
banked_hazard(op_mismatch).              % D1 — a frame op disagreeing with its keyword's required op
banked_hazard(certainty_on_exception).   % a certainty field on an exception (rule-level; no KB slot)

% ==========================================================================================
% b_sent(?BaseId, ?HeaderRest, ?Ace) — the accepted one-block sentence bases a mono_case mutates. Each is
% proven valid by bases_accept (via battery_case), so a mono mutant's reject is attributable to its edit.
% ==========================================================================================

b_sent(rule, 'rule 0 recommend',
    'If a patient has a sepsis then it is recommended that the patient takes Abx-A.').
b_sent(iv, 'rule 0 recommend',
    'If a patient has an age of at least 18 years then it is recommended that the patient takes Abx-A.').
b_sent(age1, 'rule 0 recommend',
    'If a patient has an age of at least 1 year then it is recommended that the patient takes Abx-A.').
b_sent(age2, 'rule 0 recommend',
    'If a patient has an age of at least 2 years then it is recommended that the patient takes Abx-A.').

% ==========================================================================================
% mono_case(?Class, ?Label, ?BaseId, ?MutAce, ?Reject) — single-block sentence mutations: BaseId names the
% accepted base sentence (b_sent/3), MutAce is its single-locus mutation, Reject the exact reject/3 tuple.
% ==========================================================================================

% ---- whitelist token hazards → unregistered_capital / unregistered_token / bad_number --------------
mono_case(capitalized_oov, drug_slot, rule,
    'If a patient has a sepsis then it is recommended that the patient takes Widget.',
    reject(0, 'Widget', unregistered_capital)).
mono_case(capitalized_oov, concept_slot, rule,
    'If a patient has a Sepsis then it is recommended that the patient takes Abx-A.',
    reject(0, 'Sepsis', unregistered_capital)).

mono_case(prefix_token, n_noun, rule,
    'If a patient has a n:sepsis then it is recommended that the patient takes Abx-A.',
    reject(0, 'n:sepsis', unregistered_token)).
mono_case(prefix_token, v_verb, rule,
    'If a patient has a sepsis then it is recommended that the patient v:takes Abx-A.',
    reject(0, 'v:takes', unregistered_token)).
mono_case(prefix_token, a_adj, rule,
    'If a patient has a a:sepsis then it is recommended that the patient takes Abx-A.',
    reject(0, 'a:sepsis', unregistered_token)).
mono_case(prefix_token, p_proper, rule,
    'If a p:patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject(0, 'p:patient', unregistered_token)).

mono_case(pronoun, he_consequent, rule,
    'If a patient has a sepsis then it is recommended that he takes Abx-A.',
    reject(0, he, unregistered_token)).

mono_case(or_guard, or_conjunction, rule,
    'If a patient has a sepsis or the patient has a pregnancy then it is recommended that the patient takes Abx-A.',
    reject(0, or, unregistered_token)).

mono_case(every_surface, every_subject, rule,
    'If every patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject(0, every, unregistered_token)).

mono_case(does_not, in_guard_negation, rule,
    'If a patient does not have a sepsis then it is recommended that the patient takes Abx-A.',
    reject(0, does, unregistered_token)).

mono_case(spaced_multiword, spaced_concept, rule,
    'If a patient has a severe renal impairment then it is recommended that the patient takes Abx-A.',
    reject(0, severe, unregistered_token)).

mono_case(ace_comment, hash_line, rule,
    '# a comment.',
    reject(0, '#', unregistered_token)).

mono_case(quotation, quoted_drug, rule,
    'If a patient has a sepsis then it is recommended that the patient takes "Abx-A".',
    reject(0, '"Abx-A"', unregistered_token)).

mono_case(decimal, interval_value, iv,
    'If a patient has an age of at least 18.5 years then it is recommended that the patient takes Abx-A.',
    reject(0, '18.5', bad_number)).

mono_case(leading_zero, interval_value, iv,
    'If a patient has an age of at least 018 years then it is recommended that the patient takes Abx-A.',
    reject(0, '018', bad_number)).

mono_case(exactly_eq_marker, exactly, iv,
    'If a patient has an age of exactly 18 years then it is recommended that the patient takes Abx-A.',
    reject(0, exactly, unregistered_token)).

% ---- structural hazards (every token legal, the shape is wrong) → malformed_sentence --------------
mono_case(exactly_eq_marker, bare_eq, iv,
    'If a patient has an age of 18 years then it is recommended that the patient takes Abx-A.',
    reject(0, '', malformed_sentence)).
mono_case(bare_then, no_frame, rule,
    'If a patient has a sepsis then the patient takes Abx-A.',
    reject(0, '', malformed_sentence)).
mono_case(no_antecedent_definite, definite_first_subject, rule,
    'If the patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject(0, '', malformed_sentence)).
mono_case(number_agreement, plural_for_one, age1,
    'If a patient has an age of at least 1 years then it is recommended that the patient takes Abx-A.',
    reject(0, '', malformed_sentence)).
mono_case(number_agreement, singular_for_two, age2,
    'If a patient has an age of at least 2 year then it is recommended that the patient takes Abx-A.',
    reject(0, '', malformed_sentence)).

% ==========================================================================================
% raw_case(?Class, ?Label, ?BaseSpec, ?MutantSpec, ?Reject) — header, multi-block, document-level, and
% term-input mutations; BaseSpec / MutantSpec each resolve through expand/2 (a document atom, `canon` /
% `canoncodes`, or a raw non-document term for bad_input). Base + mutant differ by one locus.
% ==========================================================================================

% ---- an unregistered modality keyword in the header (the sentence stays valid) → bad_keyword --------
raw_case(unregistered_keyword, obligatory, canon,
    'document d\n\nrule 0 obligatory\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    reject(0, obligatory, bad_keyword)).
raw_case(unregistered_keyword, prohibited, canon,
    'document d\n\nrule 0 prohibited\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    reject(0, prohibited, bad_keyword)).

% ---- a leading-zero block id → the header DCG rejects the id (int_dcg no_leading_zero) → bad_header ---
raw_case(leading_zero, block_id,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    'document d\n\nrule 00 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    reject(0, 'rule 00 recommend', bad_header)).

% ---- a later disjunct opening on the definite `the patient` (index 1; the first disjunct accepts) ----
raw_case(cross_sentence_definite, second_disjunct,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\nIf a patient has a pregnancy then it is recommended that the patient takes Abx-A.\n',
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\nIf the patient has a pregnancy then it is recommended that the patient takes Abx-A.\n',
    reject(1, '', malformed_sentence)).

% ---- document-level id integrity (both sentences valid → the sole reject is the id defect) -----------
raw_case(duplicate_rule_id, two_rule_zero,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nrule 1 recommend\nIf a patient has a pregnancy then it is recommended that the patient takes Abx-A.\n',
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nrule 0 recommend\nIf a patient has a pregnancy then it is recommended that the patient takes Abx-A.\n',
    reject(1, 0, duplicate_rule_id)).
raw_case(duplicate_exception_id, two_exc_zero,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 0\nA patient has a pregnancy.\n\nexception 1 rule 0\nA patient has a severe-renal-impairment.\n',
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 0\nA patient has a pregnancy.\n\nexception 0 rule 0\nA patient has a severe-renal-impairment.\n',
    reject(2, 0, duplicate_exception_id)).
raw_case(dangling_exception, ref_rule_nine,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 0\nA patient has a pregnancy.\n',
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 9\nA patient has a pregnancy.\n',
    reject(1, 9, dangling_exception)).

% ---- a certainty field on an exception header → certainty_on_exception. Certainty is a rule-level
% surface (KB.md `certainty(RuleId, _)`; no exception-certainty slot), so a certainty on an exception
% has no KB home and rejects fail-closed (SURFACE.md exc-block = basis only). -------------------------
raw_case(certainty_on_exception, exc_certainty,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 0\nA patient has a pregnancy.\n',
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 0 certainty moderate\nA patient has a pregnancy.\n',
    reject(1, moderate, certainty_on_exception)).

% ---- a block with no rule/exception header (its single line fails the header DCG) → bad_header --------
raw_case(missing_header, headerless_block,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    'document d\n\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    reject(0, 'If a patient has a sepsis then it is recommended that the patient takes Abx-A.', bad_header)).

% ---- malformed document id (empty dotted segment / an embedded space) → bad_document_header ----------
raw_case(bad_doc_id, empty_segment,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    'document a..b\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    reject(-1, '', bad_document_header)).
raw_case(bad_doc_id, embedded_space,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    'document a b\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    reject(-1, '', bad_document_header)).

% ---- input shape: the gate is total over any term (never throws) → bad_input. The bases prove a valid
% atom (canon) and a valid scalar code list (canoncodes) both accept, so a code_list/1 regression fails
% bases_accept rather than hiding behind these non-list / non-scalar-list mutants. -----------------
raw_case(bad_input, number,        canon,      123,          reject(-1, '', bad_input)).
raw_case(bad_input, compound,      canon,      foo(bar),     reject(-1, '', bad_input)).
raw_case(bad_input, nonground,     canoncodes, [_],          reject(-1, '', bad_input)).
raw_case(bad_input, atom_list,     canoncodes, [foo],        reject(-1, '', bad_input)).
raw_case(bad_input, out_of_range,  canoncodes, [1114112],    reject(-1, '', bad_input)).   % > 0x10FFFF
raw_case(bad_input, surrogate,     canoncodes, [55296],      reject(-1, '', bad_input)).   % 0xD800

% ==========================================================================================
% op-mismatch (D1): a rule keyword's frame op MUST equal the keyword's op. The registry-derived Cartesian
% over keyword × frame-op — matching op = the accepted base, every mismatched op = a reject(0, Kw,
% op_mismatch) mutant — pins all 6 matches + all 18 mismatches (vs a per-keyword sample).
% ==========================================================================================

op_mismatch_pair(Kw, Label, BasePhrase, MutPhrase) :-
    reg_keyword(Kw, OpK, _, _),
    reg_frame(OpK, BasePhrase),                 % the matching frame → the accepted base
    reg_frame(OpF, MutPhrase), OpF \== OpK,      % every other frame → an op-mismatch mutant
    atomic_list_concat([Kw, '_vs_', OpF], Label).

frame_sentence(Phrase, Ace) :-
    atomic_list_concat(['If a patient has a sepsis then ', Phrase, ' the patient takes Abx-A.'], Ace).

% ==========================================================================================
% battery_case(?Class, ?Label, -Base, -Mutant, ?Reject) — the flat matrix: the mono_case base+mutant
% sentences (built into a document), the raw_case base+mutant inputs (via expand/2), and the op-mismatch
% Cartesian. Base is the accepted counterfactual, Mutant its single-locus mutation, Reject the reject/3.
% ==========================================================================================

battery_case(C, L, Base, Mut, R) :-
    mono_case(C, L, BaseId, MutAce, R),
    b_sent(BaseId, Hdr, BaseAce),
    mono(Hdr, BaseAce, Base),
    mono(Hdr, MutAce, Mut).
battery_case(C, L, Base, Mut, R) :-
    raw_case(C, L, BaseSpec, MutSpec, R),
    expand(BaseSpec, Base),
    expand(MutSpec, Mut).
battery_case(op_mismatch, Label, Base, Mut, reject(0, Kw, op_mismatch)) :-
    op_mismatch_pair(Kw, Label, BasePhrase, MutPhrase),
    atom_concat('rule 0 ', Kw, Hdr),
    frame_sentence(BasePhrase, BaseAce), mono(Hdr, BaseAce, Base),
    frame_sentence(MutPhrase, MutAce),   mono(Hdr, MutAce, Mut).

% ==========================================================================================
:- begin_tests(raw_gate_battery).

% ---- accept controls: the canonical accept shapes (exact sentence lists) + all three input encodings --
test(control_base_rule) :-
    mono('rule 0 recommend', 'If a patient has a sepsis then it is recommended that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R = ok(doc('d', [sentence(0, _, rule(0, recommend, 0, none, none))]))).
test(control_base_interval) :-
    mono('rule 0 recommend', 'If a patient has an age of at least 18 years then it is recommended that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R = ok(doc('d', [sentence(0, _, rule(0, recommend, 0, none, none))]))).
test(control_base_exception) :-
    Doc = 'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 0\nA patient has a severe-renal-impairment.\n',
    raw_gate:gate_document(Doc, R),
    assertion(R = ok(doc('d', [ sentence(0, _, rule(0, recommend, 0, none, none)),
                                sentence(1, _, exception(0, 0, none, none)) ]))).
test(control_base_string) :-        % the same document as a STRING input accepts (to_codes string path)
    canondoc(Atom), atom_string(Atom, Str),
    raw_gate:gate_document(Str, R),
    assertion(R = ok(doc('d', [sentence(0, _, rule(0, recommend, 0, none, none))]))).
test(control_base_codes) :-         % the same document as a CODE LIST accepts (to_codes code_list path)
    canondoc(Atom), atom_codes(Atom, Codes),
    raw_gate:gate_document(Codes, R),
    assertion(R = ok(doc('d', [sentence(0, _, rule(0, recommend, 0, none, none))]))).

% ---- anti-vacuity: every mutant's exact base accepts, so each reject is the mutation's doing -----------
test(bases_accept, [forall(battery_case(Class, Label, Base, _, _))]) :-
    raw_gate:gate_document(Base, Result),
    (   Result = ok(_)
    ->  true
    ;   format(user_error, "~N[raw_gate_battery base] ~w/~w: base did not accept, got ~q~n",
               [Class, Label, Result]),
        fail
    ).

% ---- the mutation matrix: every mutant rejects with exactly its (Idx, Token, Construct). The single
% reject/3 tuple is wrapped here, so a case can only ever express a rejection (never an accept). ---------
test(mutants_reject, [forall(battery_case(Class, Label, _, Mut, Reject))]) :-
    raw_gate:gate_document(Mut, Result),
    (   Result == reject([Reject])
    ->  true
    ;   format(user_error, "~N[raw_gate_battery] ~w/~w: expected ~q, got ~q~n",
               [Class, Label, reject([Reject]), Result]),
        fail
    ).

% ---- self-checks: the matrix covers exactly the banked-hazard set (a gap / a typo'd class fails) -------
test(covers_every_banked_hazard) :-
    findall(H, ( banked_hazard(H), \+ battery_case(H, _, _, _, _) ), Uncovered),
    assertion(Uncovered == []).
test(no_unbanked_class) :-
    findall(C, ( battery_case(C, _, _, _, _), \+ banked_hazard(C) ), Unbanked),
    assertion(Unbanked == []).

:- end_tests(raw_gate_battery).
