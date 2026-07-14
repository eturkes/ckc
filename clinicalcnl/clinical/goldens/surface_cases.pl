% ClinicalCNL v1 surface goldens — case seeds (M3.surface-goldens; SPEC §10.6).
%
% Hand-authored INPUTS only (ids + ACE surface + kind); the EXPECTED bytes are
% captured OBSERVED via the product seam into surface_expected.pl — never written here.
% surface_goldens.pl replays each case byte-exact and regenerates surface_expected.pl.
%
% Kind = APE-parseability class under the canonical seam (SURFACE.md §seam):
%   parses   -> get_ape_results returns text/plain + a serialized DRS (v1-valid surface,
%               or a documented non-v1 surface that APE accepts but a later lane rejects)
%   rejected -> get_ape_results returns text/xml <messages> (APE fail-closed discriminator)
% run_seam checks kind<->content-type: parses iff text/plain, rejected iff text/xml.

:- module(surface_cases, [surface_ulex/1, surface_case/3, surface_note/2]).

%% surface_ulex(-UlexText:atom)
% Frozen v1 clinical ulex (registry surfaces of §L·ids), as ulextext file bytes. The `ulex`
% unit builds the canonical clinical_ulex.pl; it must stay consistent with this frozen set.
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

% --- Frames (4 v1 directions): frame op is the sole ACE-level modality carrier (D1). ------
surface_case(frame_recommend,      parses, 'It is recommended that a patient takes Abx-A.').
surface_case(frame_admissible,     parses, 'It is admissible that a patient takes Abx-A.').
surface_case(frame_not_recommend,  parses, 'It is not recommended that a patient takes Abx-A.').
surface_case(frame_not_possible,   parses, 'It is not possible that a patient takes Abx-A.').

% --- Guard atoms: concept-have and its NAF negation (D3/D5). --------------------------------
surface_case(guard_condition,      parses, 'It is recommended that if a patient has a sepsis then the patient takes Abx-A.').
surface_case(guard_neg_condition,  parses, 'It is recommended that if a patient does not have a sepsis then the patient takes Abx-A.').

% --- Interval markers (D8/D9): geq/greater land top-level; leq/less land in a nested sublist.
surface_case(iv_at_least,          parses, 'It is recommended that if the age of a patient is at least 18 years then the patient takes Abx-A.').
surface_case(iv_more_than,         parses, 'It is recommended that if the age of a patient is more than 18 years then the patient takes Abx-A.').
surface_case(iv_at_most,           parses, 'It is recommended that if the age of a patient is at most 18 years then the patient takes Abx-A.').
surface_case(iv_less_than,         parses, 'It is recommended that if the age of a patient is less than 18 years then the patient takes Abx-A.').

% --- Interval eq surfaces: APE accepts them (eq CountOp); the single-bound law rejects eq at
%     the raw/profile lane, not at APE. Pinned so profile-drs knows the shape it must reject.
surface_case(iv_exactly,           parses, 'It is recommended that if the age of a patient is exactly 18 years then the patient takes Abx-A.').
surface_case(iv_bare,              parses, 'It is recommended that if the age of a patient is 18 years then the patient takes Abx-A.').

% --- Thread composites (§8.6 docA x docB + control): full multi-conjunct rule sentences. ----
surface_case(thread_doc_a,         parses, 'It is recommended that if a patient has a sepsis and the patient does not have a severe-renal-impairment and the age of the patient is at least 18 years then the patient takes Abx-A.').
surface_case(thread_doc_b,         parses, 'It is not recommended that if a patient has a sepsis and the patient has a pregnancy and the age of the patient is at least 18 years then the patient takes Abx-A.').
surface_case(thread_control,       parses, 'It is recommended that if a patient has a sepsis and the age of the patient is less than 18 years then the patient takes Abx-A.').

% --- Exception body: bare conditional (no modality frame; strength/direction come from the
%     parent rule header). Parses to a bare =>/2 DRS. -----------------------------------------
surface_case(exception_body,       parses, 'If a patient has a pregnancy then the patient takes Abx-A.').

% --- Rejected frames (p1): malformed modality framing -> APE text/xml. ----------------------
surface_case(reject_frame_false,   rejected, 'It false recommended that a patient takes Abx-A.').
surface_case(reject_frame_no_that, rejected, 'It is recommended a patient takes Abx-A.').
surface_case(reject_frame_no_is,   rejected, 'It recommended that a patient takes Abx-A.').

% --- Rejected interval surfaces (p5): malformed number/unit phrasing -> APE text/xml. --------
surface_case(reject_iv_no_years,   rejected, 'It is recommended that if the age of a patient is at least 18 then the patient takes Abx-A.').
surface_case(reject_iv_years_old,  rejected, 'It is recommended that if a patient is at least 18 years old then the patient takes Abx-A.').
surface_case(reject_iv_decimal,    rejected, 'It is recommended that if the age of a patient is at least 18.5 years then the patient takes Abx-A.').

% --- Rejected OOV classes (p6): unregistered content words under noclex=on -> APE text/xml. --
surface_case(reject_oov_noun,      rejected, 'A patient takes a widget.').
surface_case(reject_oov_capital,   rejected, 'A patient takes a Widget.').
surface_case(reject_oov_verb,      rejected, 'A patient devours a sepsis.').

%% surface_note(?Id:atom, ?Note:atom) — downstream-lane provenance for non-v1 parses.
surface_note(iv_exactly, 'eq CountOp: single-bound-law reject at raw/profile lane (parses at APE)').
surface_note(iv_bare,    'eq CountOp: single-bound-law reject at raw/profile lane (parses at APE)').
