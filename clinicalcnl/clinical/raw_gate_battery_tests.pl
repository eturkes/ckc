% ClinicalCNL raw-gate mutation reject battery (M3.raw-gate-battery; SPEC §10.6, SURFACE.md).
%
% The EXHAUSTIVE per-hazard mutation matrix over raw_gate:gate_document/2. raw_gate_tests.pl carries
% ONE sole-diagnosis reject per hazard as part of the raw-gate acceptance; this unit is the systematic
% matrix: one mutant CLASS per banked hazard (the roadmap §M3 raw-gate-battery unit line + SURFACE.md
% §Modality's op-mismatch reject battery), every case a single-locus mutation of a proven-valid base,
% asserting the exact reject(Idx, Token, Construct). All-reject; three accept CONTROLS prove each base
% is valid, so every reject is attributable to its mutation, not a broken base (anti-vacuity). Two
% self-checks bind the matrix to the banked-hazard set (every hazard covered, no unbanked class typo).
%
% Pure Prolog — raw_gate is a whitelist BEFORE APE and never runs it, so the battery is fast like the
% kb_kernel / ulex / registry gates (no ape.exe). Constructs pinned here are OBSERVED from the gate,
% never assumed (memory: never assert "every X rejects" from a partial probe — each case was run).
%
%   Gate: swipl -q -g "consult('clinical/raw_gate_battery_tests.pl'),(run_tests(raw_gate_battery)->halt(0);halt(1))" -t 'halt(1)'

:- module(raw_gate_battery_tests, []).

:- use_module(library(plunit)).

% Load the gate source-relative + cwd-independent (mirrors raw_gate_tests.pl).
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/raw_gate.pl'], G), use_module(G).

% mono(+HeaderRest, +Ace, -Doc) — a minimal one-block document (`document d` + one rule/exception
% block of a single ACE line) around a single mutated sentence. The multi-block / document-level /
% term-input hazards carry their own raw_case/4 document instead.
mono(Hdr, Ace, Doc) :- atomic_list_concat(['document d\n\n', Hdr, '\n', Ace, '\n'], Doc).

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

% ==========================================================================================
% mono_case(?Class, ?Label, ?HeaderRest, ?Ace, ?Expected) — single-block sentence mutations. Base
% rule sentence = `If a patient has a sepsis then it is recommended that the patient takes Abx-A.`;
% base interval = `... has an age of at least 18 years ...` (both proven valid by the controls below).
% ==========================================================================================

% ---- whitelist token hazards → unregistered_token / unregistered_capital / bad_number -------------
mono_case(capitalized_oov, drug_slot, 'rule 0 recommend',
    'If a patient has a sepsis then it is recommended that the patient takes Widget.',
    reject([reject(0, 'Widget', unregistered_capital)])).
mono_case(capitalized_oov, concept_slot, 'rule 0 recommend',
    'If a patient has a Sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, 'Sepsis', unregistered_capital)])).

mono_case(prefix_token, n_noun, 'rule 0 recommend',
    'If a patient has a n:sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, 'n:sepsis', unregistered_token)])).
mono_case(prefix_token, v_verb, 'rule 0 recommend',
    'If a patient has a sepsis then it is recommended that the patient v:takes Abx-A.',
    reject([reject(0, 'v:takes', unregistered_token)])).
mono_case(prefix_token, a_adj, 'rule 0 recommend',
    'If a patient has a a:sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, 'a:sepsis', unregistered_token)])).
mono_case(prefix_token, p_proper, 'rule 0 recommend',
    'If a p:patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, 'p:patient', unregistered_token)])).

mono_case(pronoun, he_consequent, 'rule 0 recommend',
    'If a patient has a sepsis then it is recommended that he takes Abx-A.',
    reject([reject(0, he, unregistered_token)])).

mono_case(or_guard, or_conjunction, 'rule 0 recommend',
    'If a patient has a sepsis or the patient has a pregnancy then it is recommended that the patient takes Abx-A.',
    reject([reject(0, or, unregistered_token)])).

mono_case(every_surface, every_subject, 'rule 0 recommend',
    'If every patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, every, unregistered_token)])).

mono_case(does_not, in_guard_negation, 'rule 0 recommend',
    'If a patient does not have a sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, does, unregistered_token)])).

mono_case(spaced_multiword, spaced_concept, 'rule 0 recommend',
    'If a patient has a severe renal impairment then it is recommended that the patient takes Abx-A.',
    reject([reject(0, severe, unregistered_token)])).

mono_case(exactly_eq_marker, exactly, 'rule 0 recommend',
    'If a patient has an age of exactly 18 years then it is recommended that the patient takes Abx-A.',
    reject([reject(0, exactly, unregistered_token)])).

mono_case(ace_comment, hash_line, 'rule 0 recommend',
    '# a comment.',
    reject([reject(0, '#', unregistered_token)])).

mono_case(quotation, quoted_drug, 'rule 0 recommend',
    'If a patient has a sepsis then it is recommended that the patient takes "Abx-A".',
    reject([reject(0, '"Abx-A"', unregistered_token)])).

mono_case(decimal, interval_value, 'rule 0 recommend',
    'If a patient has an age of at least 18.5 years then it is recommended that the patient takes Abx-A.',
    reject([reject(0, '18.5', bad_number)])).

mono_case(leading_zero, interval_value, 'rule 0 recommend',
    'If a patient has an age of at least 018 years then it is recommended that the patient takes Abx-A.',
    reject([reject(0, '018', bad_number)])).

% ---- structural hazards (every token legal, the shape is wrong) → malformed_sentence --------------
mono_case(bare_then, no_frame, 'rule 0 recommend',
    'If a patient has a sepsis then the patient takes Abx-A.',
    reject([reject(0, '', malformed_sentence)])).
mono_case(no_antecedent_definite, definite_first_subject, 'rule 0 recommend',
    'If the patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, '', malformed_sentence)])).
mono_case(number_agreement, plural_for_one, 'rule 0 recommend',
    'If a patient has an age of at least 1 years then it is recommended that the patient takes Abx-A.',
    reject([reject(0, '', malformed_sentence)])).
mono_case(number_agreement, singular_for_two, 'rule 0 recommend',
    'If a patient has an age of at least 2 year then it is recommended that the patient takes Abx-A.',
    reject([reject(0, '', malformed_sentence)])).
mono_case(exactly_eq_marker, bare_eq, 'rule 0 recommend',
    'If a patient has an age of 18 years then it is recommended that the patient takes Abx-A.',
    reject([reject(0, '', malformed_sentence)])).

% ---- header keyword: an unregistered modality keyword → bad_keyword (short-circuits the sentence) --
mono_case(unregistered_keyword, obligatory, 'rule 0 obligatory',
    'If a patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, obligatory, bad_keyword)])).
mono_case(unregistered_keyword, prohibited, 'rule 0 prohibited',
    'If a patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, prohibited, bad_keyword)])).

% ---- op-mismatch (D1): a valid keyword paired with a frame whose op differs, one per keyword -------
mono_case(op_mismatch, recommend_vs_can, 'rule 0 recommend',
    'If a patient has a sepsis then it is not possible that the patient takes Abx-A.',
    reject([reject(0, recommend, op_mismatch)])).
mono_case(op_mismatch, suggest_vs_may, 'rule 0 suggest',
    'If a patient has a sepsis then it is admissible that the patient takes Abx-A.',
    reject([reject(0, suggest, op_mismatch)])).
mono_case(op_mismatch, may_consider_vs_should, 'rule 0 may-consider',
    'If a patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, 'may-consider', op_mismatch)])).
mono_case(op_mismatch, not_recommend_vs_can, 'rule 0 not-recommend',
    'If a patient has a sepsis then it is not possible that the patient takes Abx-A.',
    reject([reject(0, 'not-recommend', op_mismatch)])).
mono_case(op_mismatch, not_suggest_vs_should, 'rule 0 not-suggest',
    'If a patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, 'not-suggest', op_mismatch)])).
mono_case(op_mismatch, contraindicate_vs_should, 'rule 0 contraindicate',
    'If a patient has a sepsis then it is recommended that the patient takes Abx-A.',
    reject([reject(0, contraindicate, op_mismatch)])).

% ==========================================================================================
% raw_case(?Class, ?Label, ?DocOrTerm, ?Expected) — multi-block, document-level, and term-input
% hazards, each carrying its full document (or the raw input term for the bad_input class).
% ==========================================================================================

% ---- leading zero in a block id → the header DCG rejects the id (int_dcg no_leading_zero) ---------
raw_case(leading_zero, block_id,
    'document d\n\nrule 00 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    reject([reject(0, 'rule 00 recommend', bad_header)])).

% ---- a later disjunct opening on the definite `the patient` (index 1; the first disjunct accepts) --
raw_case(cross_sentence_definite, second_disjunct,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\nIf the patient has a pregnancy then it is recommended that the patient takes Abx-A.\n',
    reject([reject(1, '', malformed_sentence)])).

% ---- document-level id integrity (both sentences valid → the sole reject is the id defect) ---------
raw_case(duplicate_rule_id, two_rule_zero,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nrule 0 recommend\nIf a patient has a pregnancy then it is recommended that the patient takes Abx-A.\n',
    reject([reject(1, 0, duplicate_rule_id)])).
raw_case(duplicate_exception_id, two_exc_zero,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 0\nA patient has a pregnancy.\n\nexception 0 rule 0\nA patient has a severe-renal-impairment.\n',
    reject([reject(2, 0, duplicate_exception_id)])).
raw_case(dangling_exception, ref_rule_nine,
    'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 9\nA patient has a pregnancy.\n',
    reject([reject(1, 9, dangling_exception)])).

% ---- a block with no rule/exception header (its single line fails the header DCG) → bad_header -----
raw_case(missing_header, headerless_block,
    'document d\n\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    reject([reject(0, 'If a patient has a sepsis then it is recommended that the patient takes Abx-A.', bad_header)])).

% ---- malformed document id (empty dotted segment / an embedded space) → bad_document_header --------
raw_case(bad_doc_id, empty_segment,
    'document a..b\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    reject([reject(-1, '', bad_document_header)])).
raw_case(bad_doc_id, embedded_space,
    'document a b\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    reject([reject(-1, '', bad_document_header)])).

% ---- input shape: the gate is total over any term (never throws) → bad_input ----------------------
raw_case(bad_input, number,        123,          reject([reject(-1, '', bad_input)])).
raw_case(bad_input, compound,      foo(bar),     reject([reject(-1, '', bad_input)])).
raw_case(bad_input, nonground,     [_],          reject([reject(-1, '', bad_input)])).
raw_case(bad_input, atom_list,     [foo],        reject([reject(-1, '', bad_input)])).
raw_case(bad_input, out_of_range,  [1114112],    reject([reject(-1, '', bad_input)])).   % > 0x10FFFF
raw_case(bad_input, surrogate,     [55296],      reject([reject(-1, '', bad_input)])).   % 0xD800

% battery_case(?Class, ?Label, ?DocOrTerm, ?Expected) — the mono_case single-block sentence mutations
% (built into a full document) plus the raw_case documents / input terms, one flat matrix.
battery_case(C, L, Doc, E) :- mono_case(C, L, Hdr, Ace, E), mono(Hdr, Ace, Doc).
battery_case(C, L, Doc, E) :- raw_case(C, L, Doc, E).

% ==========================================================================================
:- begin_tests(raw_gate_battery).

% ---- accept controls: each mutation base is itself valid, so a reject is the mutation's doing ------
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

% ---- the mutation matrix: every case rejects with its exact sentence index + construct ------------
test(all_reject, [forall(battery_case(Class, Label, Doc, Expected))]) :-
    raw_gate:gate_document(Doc, Result),
    (   Result == Expected
    ->  true
    ;   format(user_error, "~N[raw_gate_battery] ~w/~w: expected ~q, got ~q~n",
               [Class, Label, Expected, Result]),
        fail
    ).

% ---- self-checks: the matrix covers exactly the banked-hazard set (a gap / a typo'd class fails) ---
test(covers_every_banked_hazard) :-
    findall(H, ( banked_hazard(H), \+ battery_case(H, _, _, _) ), Uncovered),
    assertion(Uncovered == []).
test(no_unbanked_class) :-
    findall(C, ( battery_case(C, _, _, _), \+ banked_hazard(C) ), Unbanked),
    assertion(Unbanked == []).

:- end_tests(raw_gate_battery).
