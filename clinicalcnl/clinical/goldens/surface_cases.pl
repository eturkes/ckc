% ClinicalCNL v1 surface goldens — case seeds (M3.surface-goldens; SPEC §10.6).
%
% Hand-authored INPUTS only (ids + ACE surface + kind); the EXPECTED bytes (content-type,
% serialized DRS, message list) are captured OBSERVED via the product seam into
% surface_expected.pl — never written here. surface_goldens.pl replays each case byte-exact
% and regenerates surface_expected.pl.
%
% Kind = the case's standing under the canonical seam (SURFACE.md §seam) and the v1 profile:
%   v1     -> get_ape_results returns text/plain + a serialized DRS AND zero messages
%             (a v1-admissible surface; the zero-message law holds).
%   nonv1  -> text/plain + a serialized DRS, but the surface is excluded from v1. APE accepts
%             it; a later lane (raw gate / profile-drs) rejects it. Its messages are pinned as
%             evidence (empty for a shape-rule reject e.g. single-bound law / in-guard negation;
%             a warning for a message-driven reject e.g. anaphor, undefined-word named hole).
%   reject -> text/xml <messages> (APE fail-closed on an error message).
% run_seam asserts: reject iff text/xml; v1/nonv1 both text/plain; v1 => messages == [].

:- module(surface_cases, [surface_ulex/1, surface_case/3, surface_note/2]).

%% surface_ulex(-UlexText:atom)
% Frozen v1 clinical ulex = the registry surfaces of §L·ids, as ulextext file bytes. Full
% inflection families (takes+take, has+have, year+years) mirror what the `ulex` unit's
% clinical_ulex.pl builds; that unit must stay byte-consistent with this frozen set. Entries
% are the registry, not a per-case minimum — some inflections are unexercised by the cases
% below yet frozen here so the registry mirror is complete.
surface_ulex('noun_sg(patient, patient, human).
noun_sg(sepsis, sepsis, neutr).
noun_sg(pregnancy, pregnancy, neutr).
noun_sg(''severe-renal-impairment'', ''severe-renal-impairment'', neutr).
noun_sg(age, age, neutr).
noun_sg(year, year, neutr).
noun_pl(years, year, neutr).
pn_sg(''Abx-A'', ''Abx-A'', neutr).
tv_finsg(takes, take).
tv_infpl(take, take).
tv_finsg(has, have).
tv_infpl(have, have).').

%% surface_case(?Id:atom, ?Kind:atom, ?ACE:atom)

% --- Frames (4 v1 directions): the modality frame op is the sole ACE-level modality carrier
%     (D1), and it lives in the CONSEQUENT of the conditional: `If <guard> then <frame> <action>`
%     -> =>(guard, op(action)). A uniform sepsis guard isolates the op as the only variable;
%     the concept-have guard atom (D3) is pinned here too. --------------------------------------
surface_case(frame_recommend,     v1, 'If a patient has a sepsis then it is recommended that the patient takes Abx-A.').
surface_case(frame_admissible,    v1, 'If a patient has a sepsis then it is admissible that the patient takes Abx-A.').
surface_case(frame_not_recommend, v1, 'If a patient has a sepsis then it is not recommended that the patient takes Abx-A.').
surface_case(frame_not_possible,  v1, 'If a patient has a sepsis then it is not possible that the patient takes Abx-A.').

% --- Guard negation (D5): in-guard `does not have` PARSES clean (zero messages) but is deferred
%     from v1 — all negative context enters via labeled exceptions (NAF), so this is a profile
%     reject-battery member, not a v1 surface. -------------------------------------------------
surface_case(guard_neg,           nonv1, 'If a patient does not have a sepsis then it is recommended that the patient takes Abx-A.').

% --- Interval markers (D8/D9), surface `the patient has an age of <marker> <INT> years`:
%     geq/greater land top-level; leq/less land in a nested sublist (guard walker flattens one).
surface_case(iv_at_least,         v1, 'If a patient has an age of at least 18 years then it is recommended that the patient takes Abx-A.').
surface_case(iv_more_than,        v1, 'If a patient has an age of more than 18 years then it is recommended that the patient takes Abx-A.').
surface_case(iv_at_most,          v1, 'If a patient has an age of at most 18 years then it is recommended that the patient takes Abx-A.').
surface_case(iv_less_than,        v1, 'If a patient has an age of less than 18 years then it is recommended that the patient takes Abx-A.').

% --- Interval non-v1 shapes: exactly/bare-eq PARSE clean but the single-bound law rejects them
%     at the raw/profile lane (pinned so profile-drs knows the eq/exactly shape it must reject);
%     the `the age of a patient is <marker> <INT> years` phrasing PARSES with an anaphor warning
%     (`the age` has no antecedent) — the warning-bearing surface D9 replaced. ------------------
surface_case(iv_exactly,          nonv1, 'If a patient has an age of exactly 18 years then it is recommended that the patient takes Abx-A.').
surface_case(iv_bare,             nonv1, 'If a patient has an age of 18 years then it is recommended that the patient takes Abx-A.').
surface_case(iv_anaphor,          nonv1, 'If the age of a patient is at least 18 years then it is recommended that the patient takes Abx-A.').

% --- Thread composites (§8.6 docA x docB + control): full multi-conjunct rule sentences. docA
%     = sepsis & age>=18 -> recommend (its renal EXCLUSION enters via exc.0, not an in-guard
%     negation — D5/D6); docB = sepsis & age>=18 & pregnancy -> contraindicate (-can); control =
%     sepsis & age<18 -> contraindicate (-can), age-disjoint from docA. -------------------------
surface_case(thread_doc_a,        v1, 'If a patient has a sepsis and the patient has an age of at least 18 years then it is recommended that the patient takes Abx-A.').
surface_case(thread_doc_b,        v1, 'If a patient has a sepsis and the patient has an age of at least 18 years and the patient has a pregnancy then it is not possible that the patient takes Abx-A.').
surface_case(thread_control,      v1, 'If a patient has a sepsis and the patient has an age of less than 18 years then it is not possible that the patient takes Abx-A.').

% --- Exception body (D6): a bare, self-contained, single-concept, interval-free condition
%     assertion (no modality frame; strength/direction come from the parent rule header). Parses
%     to a bare object+predicate(have) DRS. This body is docA's exc.0 (severe-renal-impairment).
surface_case(exception_body,      v1, 'A patient has a severe-renal-impairment.').

% --- Named-word hole (p6): a capitalised OOV token in the drug slot PARSES to named('Widget')
%     + an "Undefined word" warning even under guess=off (solo=drs drops the warning -> text/plain,
%     NOT a reject). THE hole; the raw gate + registry (registry membership = the discriminator)
%     and the profile zero-message law close it. Pinned as evidence, not a v1 surface. ----------
surface_case(named_hole,          nonv1, 'A patient takes Widget.').

% --- Rejected frames: malformed modality framing -> APE text/xml. ----------------------------
surface_case(reject_frame_no_that, reject, 'It is recommended a patient takes Abx-A.').
surface_case(reject_frame_no_is,   reject, 'It recommended that a patient takes Abx-A.').

% --- Rejected interval surfaces: malformed number/unit phrasing -> APE text/xml. --------------
surface_case(reject_iv_no_years,   reject, 'If a patient has an age of at least 18 then it is recommended that the patient takes Abx-A.').
surface_case(reject_iv_decimal,    reject, 'If a patient has an age of at least 18.5 years then it is recommended that the patient takes Abx-A.').
surface_case(reject_iv_years_old,  reject, 'If a patient has an age of at least 18 years old then it is recommended that the patient takes Abx-A.').

% --- Rejected OOV classes (noclex=on): an unregistered lowercase content word -> APE text/xml
%     hard error ("Use the prefix n:, v:, a: or p:."). Contrast named_hole above: a capitalised
%     OOV is NOT a reject, it is the named() hole. -----------------------------------------------
surface_case(reject_oov_noun,      reject, 'A patient takes a widget.').
surface_case(reject_oov_verb,      reject, 'A patient devours a sepsis.').

%% surface_note(?Id:atom, ?Note:atom) — why each nonv1 case is excluded from v1, and the lane
%  that rejects it (message-driven via the zero-message law, or shape-driven despite zero
%  messages). Downstream (profile-drs) keys its rejects on these pinned bytes.
surface_note(guard_neg,  'in-guard negation deferred (D5); shape-driven reject at raw/profile lane; zero messages').
surface_note(iv_exactly, 'exactly CountOp: single-bound-law reject at raw/profile lane; shape-driven, zero messages').
surface_note(iv_bare,    'eq CountOp: single-bound-law reject at raw/profile lane; shape-driven, zero messages').
surface_note(iv_anaphor, 'the age-of..is phrasing: anaphor warning (the age unbound); message-driven reject via the zero-message law').
surface_note(named_hole, 'capitalised OOV -> named() + undefined-word warning; message-driven reject via the zero-message law + registry check').
