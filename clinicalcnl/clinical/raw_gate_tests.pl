% ClinicalCNL raw-gate accept + core-reject gate (M3.raw-gate). Two concerns:
%
%   accept — thread docs (§8.6 docA/docB/control) + the 4 direction frames + the 4 v1 interval
%     markers + the strength-sharing keywords + a 2-disjunct rule + certainty/basis variants each
%     parse to the expected sentence(Idx, Ace, Ctx) list. Each accepted ACE that maps to a v1 golden
%     is cross-checked BYTE-IDENTICAL against the frozen surface_cases oracle (itself APE-validated in
%     surface-goldens), so every distinct v1 surface the gate emits here is a proven APE-parseable
%     surface WITHOUT running APE. (The finite goldens pin the corpus surfaces; they do not claim the
%     whole grammar's language is APE-parseable — the number-agreement rejects below pin one seam
%     where the two accept sets must coincide.)
%   core rejects — one document per whitelist / framing / document-integrity hazard (unregistered
%     lexeme, the p6 capitalised OOV, a `n:` prefix, or-guard, every, decimal, number-agreement
%     mismatch, op-mismatch, bad keyword/certainty, duplicate field, bad input / doc-id, structural,
%     CRLF, multi-line exception, duplicate rule/exception id, dangling exception ref). The EXHAUSTIVE
%     per-hazard mutation matrix is the separate raw-gate-battery unit; here each reject is the sole
%     diagnosis term.
%
%   Gate: swipl -q -g "consult('clinical/raw_gate_tests.pl'),(run_tests(raw_gate)->halt(0);halt(1))" -t 'halt(1)'

:- module(raw_gate_tests, []).

:- use_module(library(plunit)).

% Load the gate + the frozen ACE oracle, source-relative + cwd-independent (mirrors ulex_tests.pl).
:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/raw_gate.pl'], G), use_module(G),
   atomic_list_concat([D, '/goldens/surface_cases.pl'], SC), use_module(SC).

% oracle(+Id, -Ace) — the frozen v1 surface_cases ACE for a golden id (the accept cross-check target).
oracle(Id, Ace) :- surface_cases:surface_case(Id, v1, Ace).

% mini(+HeaderRest, +AceLine, -Doc) — a minimal one-rule document around a single sentence, for the
% reject probes (`document d` + one block); HeaderRest e.g. 'rule 0 recommend'.
mini(Header, Ace, Doc) :- atomic_list_concat(['document d\n\n', Header, '\n', Ace, '\n'], Doc).

% ==========================================================================================
% Raw documents (hand-authored). Every ACE line is typed verbatim; the accept tests cross-check each
% oracle-backed ACE against the frozen oracle, so a transcription drift fails loud.
% ==========================================================================================

doc_raw(doc_a,
'document test_source.m1_guideline_a

rule 0 recommend certainty high basis "guideline A RCT"
If a patient has a sepsis and the patient has an age of at least 18 years then it is recommended that the patient takes Abx-A.

exception 0 rule 0 basis "renal safety exclusion"
A patient has a severe-renal-impairment.
').

doc_raw(doc_b,
'document test_source.m1_guideline_b

rule 0 contraindicate certainty moderate basis "guideline B safety"
If a patient has a sepsis and the patient has an age of at least 18 years and the patient has a pregnancy then it is not possible that the patient takes Abx-A.
').

doc_raw(doc_control,
'document test_source.m1_control

rule 0 contraindicate basis "control doc"
If a patient has a sepsis and the patient has an age of less than 18 years then it is not possible that the patient takes Abx-A.
').

doc_raw(doc_frames,
'document test_frames

rule 0 recommend
If a patient has a sepsis then it is recommended that the patient takes Abx-A.

rule 1 may-consider
If a patient has a sepsis then it is admissible that the patient takes Abx-A.

rule 2 not-recommend
If a patient has a sepsis then it is not recommended that the patient takes Abx-A.

rule 3 contraindicate
If a patient has a sepsis then it is not possible that the patient takes Abx-A.
').

doc_raw(doc_shared,
'document test_shared

rule 0 suggest
If a patient has a sepsis then it is recommended that the patient takes Abx-A.

rule 1 not-suggest
If a patient has a sepsis then it is not recommended that the patient takes Abx-A.
').

doc_raw(doc_intervals,
'document test_intervals

rule 0 recommend
If a patient has an age of at least 18 years then it is recommended that the patient takes Abx-A.

rule 1 recommend
If a patient has an age of more than 18 years then it is recommended that the patient takes Abx-A.

rule 2 recommend
If a patient has an age of at most 18 years then it is recommended that the patient takes Abx-A.

rule 3 recommend
If a patient has an age of less than 18 years then it is recommended that the patient takes Abx-A.
').

% A 2-disjunct rule: the two disjuncts share rule id 0 (→ stmt.0 / stmt.1, D4). The second is the
% interval surface (an oracle-backed golden) so both disjuncts cross-check against the oracle.
doc_raw(doc_disj,
'document test_disj

rule 0 recommend
If a patient has a sepsis then it is recommended that the patient takes Abx-A.
If a patient has an age of at least 18 years then it is recommended that the patient takes Abx-A.
').

doc_raw(doc_cert,
'document test_cert

rule 0 recommend certainty very_low
If a patient has a sepsis then it is recommended that the patient takes Abx-A.

rule 1 recommend certainty low basis "graded low"
If a patient has a sepsis then it is recommended that the patient takes Abx-A.
').

% ==========================================================================================
:- begin_tests(raw_gate).

% ---- accept: §8.6 thread docs (docA rule + exception, docB, control) --------------------------
test(accept_doc_a) :-
    doc_raw(doc_a, Doc), raw_gate:gate_document(Doc, R),
    R = ok(doc(Id, [sentence(0, A0, C0), sentence(1, A1, C1)])),
    assertion(Id == 'test_source.m1_guideline_a'),
    oracle(thread_doc_a, E0),   assertion(A0 == E0),
    assertion(C0 == rule(0, recommend, 0, high, "guideline A RCT")),
    oracle(exception_body, E1), assertion(A1 == E1),
    assertion(C1 == exception(0, 0, none, "renal safety exclusion")).

test(accept_doc_b) :-
    doc_raw(doc_b, Doc), raw_gate:gate_document(Doc, R),
    R = ok(doc('test_source.m1_guideline_b', [sentence(0, A, C)])),
    oracle(thread_doc_b, E), assertion(A == E),
    assertion(C == rule(0, contraindicate, 0, moderate, "guideline B safety")).

test(accept_doc_control) :-
    doc_raw(doc_control, Doc), raw_gate:gate_document(Doc, R),
    R = ok(doc('test_source.m1_control', [sentence(0, A, C)])),
    oracle(thread_control, E), assertion(A == E),
    assertion(C == rule(0, contraindicate, 0, none, "control doc")).

% ---- accept: the 4 direction frames, each op agreeing with its keyword (D1) --------------------
test(accept_frames) :-
    doc_raw(doc_frames, Doc), raw_gate:gate_document(Doc, R),
    R = ok(doc(_, Sents)),
    Sents = [ sentence(0, A0, rule(0, recommend,       0, none, none)),
              sentence(1, A1, rule(1, 'may-consider',  0, none, none)),
              sentence(2, A2, rule(2, 'not-recommend', 0, none, none)),
              sentence(3, A3, rule(3, contraindicate,  0, none, none)) ],
    oracle(frame_recommend, A0), oracle(frame_admissible, A1),
    oracle(frame_not_recommend, A2), oracle(frame_not_possible, A3).

% ---- accept: strength-sharing keywords reuse the recommend / not-recommend phrases -------------
test(accept_shared_phrases) :-
    doc_raw(doc_shared, Doc), raw_gate:gate_document(Doc, R),
    R = ok(doc(_, [sentence(0, A0, rule(0, suggest, 0, none, none)),
                   sentence(1, A1, rule(1, 'not-suggest', 0, none, none))])),
    oracle(frame_recommend, A0), oracle(frame_not_recommend, A1).

% ---- accept: the 4 v1 interval markers (context bound: 4 single-disjunct rules) ----------------
test(accept_intervals) :-
    doc_raw(doc_intervals, Doc), raw_gate:gate_document(Doc, R),
    R = ok(doc(_, [ sentence(0, A0, rule(0, recommend, 0, none, none)),
                    sentence(1, A1, rule(1, recommend, 0, none, none)),
                    sentence(2, A2, rule(2, recommend, 0, none, none)),
                    sentence(3, A3, rule(3, recommend, 0, none, none)) ])),
    oracle(iv_at_least, A0), oracle(iv_more_than, A1),
    oracle(iv_at_most, A2),  oracle(iv_less_than, A3).

% ---- accept: a 2-disjunct rule shares its id, mapping to stmt.0 / stmt.1 (D4) ------------------
test(accept_disjunction) :-
    doc_raw(doc_disj, Doc), raw_gate:gate_document(Doc, R),
    R = ok(doc(_, [ sentence(0, A0, rule(0, recommend, 0, none, none)),
                    sentence(1, A1, rule(0, recommend, 1, none, none)) ])),
    oracle(frame_recommend, A0), oracle(iv_at_least, A1).

% ---- accept: certainty (D7) + basis field carry into the context (basis is a string, KB.md) ----
test(accept_certainty_fields) :-
    doc_raw(doc_cert, Doc), raw_gate:gate_document(Doc, R),
    R = ok(doc(_, [ sentence(0, A0, rule(0, recommend, 0, very_low, none)),
                    sentence(1, A1, rule(1, recommend, 0, low, "graded low")) ])),
    oracle(frame_recommend, A0), oracle(frame_recommend, A1).

% ---- reject: whitelist token hazards (each the sole diagnosis) ---------------------------------
test(reject_unregistered_noun) :-
    mini('rule 0 recommend', 'If a patient has a widget then it is recommended that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, widget, unregistered_token)])).

test(reject_capitalised_oov) :-        % the p6 named() hole — a capital OOV in the drug slot
    mini('rule 0 recommend', 'If a patient has a sepsis then it is recommended that the patient takes Widget.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, 'Widget', unregistered_capital)])).

test(reject_prefix_token) :-
    mini('rule 0 recommend', 'If a patient has a n:sepsis then it is recommended that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, 'n:sepsis', unregistered_token)])).

test(reject_or_guard) :-
    mini('rule 0 recommend', 'If a patient has a sepsis or the patient has a pregnancy then it is recommended that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, or, unregistered_token)])).

test(reject_every_quantifier) :-
    mini('rule 0 recommend', 'If every patient has a sepsis then it is recommended that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, every, unregistered_token)])).

test(reject_decimal_interval) :-
    mini('rule 0 recommend', 'If a patient has an age of at least 18.5 years then it is recommended that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, '18.5', bad_number)])).

% number agreement: APE parses `1 year` / `18 years` but rejects `1 years` / `2 year` — the gate's
% accept set must coincide, so a value-noun mismatch is a structural (malformed) reject.
test(reject_number_agreement) :-
    mini('rule 0 recommend', 'If a patient has an age of at least 1 years then it is recommended that the patient takes Abx-A.', D1),
    raw_gate:gate_document(D1, R1),
    assertion(R1 == reject([reject(0, '', malformed_sentence)])),
    mini('rule 0 recommend', 'If a patient has an age of at least 2 year then it is recommended that the patient takes Abx-A.', D2),
    raw_gate:gate_document(D2, R2),
    assertion(R2 == reject([reject(0, '', malformed_sentence)])).

test(reject_op_mismatch) :-            % keyword recommend (should) vs the -can frame (D1 cross-check)
    mini('rule 0 recommend', 'If a patient has a sepsis then it is not possible that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, recommend, op_mismatch)])).

% ---- reject: header + framing hazards ----------------------------------------------------------
test(reject_bad_keyword) :-
    mini('rule 0 mandate', 'If a patient has a sepsis then it is recommended that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, mandate, bad_keyword)])).

test(reject_bad_certainty) :-
    mini('rule 0 recommend certainty superhigh', 'If a patient has a sepsis then it is recommended that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, superhigh, bad_certainty)])).

test(reject_duplicate_field) :-
    mini('rule 0 recommend certainty high certainty low', 'If a patient has a sepsis then it is recommended that the patient takes Abx-A.', Doc),
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, certainty, duplicate_field)])).

test(reject_no_period) :-
    Doc = 'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A\n',
    raw_gate:gate_document(Doc, R),
    R = reject([reject(0, _, no_period)]).

test(reject_no_document_header) :-
    Doc = 'rule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(-1, '', bad_document_header)])).

test(reject_empty_block) :-
    Doc = 'document d\n\nrule 0 recommend\n',
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(0, '', empty_block)])).

test(reject_carriage_return) :-
    Doc = 'document d\r\n\r\nrule 0 recommend\r\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\r\n',
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(-1, '', carriage_return)])).

test(reject_multi_line_exception) :-   % rule 0 present so the exc ref resolves — isolates the shape
    Doc = 'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 0\nA patient has a sepsis.\nA patient has a pregnancy.\n',
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(1, '', multi_line_exception)])).

% ---- reject: document-level id integrity (unique rule/exc ids, resolvable exception refs) -------
test(reject_duplicate_rule_id) :-
    Doc = 'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nrule 0 recommend\nIf a patient has a pregnancy then it is recommended that the patient takes Abx-A.\n',
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(1, 0, duplicate_rule_id)])).

test(reject_duplicate_exception_id) :-
    Doc = 'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 0\nA patient has a pregnancy.\n\nexception 0 rule 0\nA patient has a severe-renal-impairment.\n',
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(2, 0, duplicate_exception_id)])).

test(reject_dangling_exception) :-
    Doc = 'document d\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n\nexception 0 rule 9\nA patient has a pregnancy.\n',
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(1, 9, dangling_exception)])).

% ---- reject: input shape + doc-id well-formedness (the gate is total over any term) -------------
test(reject_bad_input_number) :-
    raw_gate:gate_document(123, R), assertion(R == reject([reject(-1, '', bad_input)])).
test(reject_bad_input_compound) :-
    raw_gate:gate_document(foo(bar), R), assertion(R == reject([reject(-1, '', bad_input)])).
test(reject_bad_input_nonground_list) :-
    raw_gate:gate_document([_], R), assertion(R == reject([reject(-1, '', bad_input)])).
test(reject_bad_input_atom_list) :-
    raw_gate:gate_document([foo], R), assertion(R == reject([reject(-1, '', bad_input)])).
test(reject_bad_input_out_of_range) :-
    raw_gate:gate_document([1114112], R), assertion(R == reject([reject(-1, '', bad_input)])).
test(reject_bad_doc_id) :-
    Doc = 'document a..b\n\nrule 0 recommend\nIf a patient has a sepsis then it is recommended that the patient takes Abx-A.\n',
    raw_gate:gate_document(Doc, R),
    assertion(R == reject([reject(-1, '', bad_document_header)])).

% ---- totality: the gate never throws + always yields ok/reject (fail-closed) -------------------
test(total_on_empty) :-
    raw_gate:gate_document('', R), assertion(R = reject(_)).
test(total_on_garbage) :-
    raw_gate:gate_document('!@#$ not a document at all', R), assertion(R = reject(_)).

:- end_tests(raw_gate).
