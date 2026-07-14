% ClinicalCNL v1 user lexicon — the APE ulex entries + their ulextext projection (M3.ulex; SPEC
% §10.6, SURFACE.md). Under noclex=on these are the registered COMMON content words the seam admits
% (the §L·ids surfaces, as APE lexicon templates — prolog/lexicon/ulex.pl lexicon_template/1). They
% are not the whole fail-closed boundary: a capitalised OOV token still parses as named() with an
% "Undefined word" warning (surface_case named_hole) — that p6 hole is closed downstream by the raw
% gate's pn_allow whitelist and the profile's zero-message law, not by this lexicon.
%
% ulex_text/1 emits them as the canonical ulextext atom, identical to the frozen oracle
% surface_cases:surface_ulex/1 (pinned in ulex_tests; codepoint-exact, and v1 surfaces are ASCII so
% byte-exact). The entry SET is cross-checked against the registry surfaces there too, so this
% APE-lexical layer and the semantic registry cannot drift.
% Entry order = the frozen role order (population, conditions, age nouns, drug pn, action verb,
% guard verb); ulex_text preserves it. APE asserts entries order-independently, but the fixed order
% keeps the emitted bytes deterministic.

:- module(clinical_ulex, [ulex_entry/1, ulex_text/1]).

%% ulex_entry(?Entry) — a v1 APE lexicon entry (an ulex.pl lexicon_template/1 term). The closed v1
% lexicon: patient (human), the three condition nouns, the age + year nouns, the Abx-A proper name,
% and the take/have transitive verbs (finite-singular `takes`/`has` + infinitive/plural `take`/`have`).
ulex_entry(noun_sg(patient, patient, human)).
ulex_entry(noun_sg(sepsis, sepsis, neutr)).
ulex_entry(noun_sg(pregnancy, pregnancy, neutr)).
ulex_entry(noun_sg('severe-renal-impairment', 'severe-renal-impairment', neutr)).
ulex_entry(noun_sg(age, age, neutr)).
ulex_entry(noun_sg(year, year, neutr)).
ulex_entry(noun_pl(years, year, neutr)).
ulex_entry(pn_sg('Abx-A', 'Abx-A', neutr)).
ulex_entry(tv_finsg(takes, take)).
ulex_entry(tv_infpl(take, take)).
ulex_entry(tv_finsg(has, have)).
ulex_entry(tv_infpl(have, have)).

%% ulex_text(-Text:atom) is det.
% The canonical ulextext: one `functor(arg, ...).` per ulex_entry/1, in declaration order, LF-joined
% with no trailing LF. Identical to the frozen surface_cases:surface_ulex/1 oracle (codepoint-exact;
% v1 surfaces are ASCII, so byte-exact).
ulex_text(Text) :-
    findall(Line, (ulex_entry(E), entry_line(E, Line)), Lines),
    atomic_list_concat(Lines, '\n', Text).

entry_line(Entry, Line) :-
    Entry =.. [Functor | Args],
    maplist(quoted_atom, Args, ArgTexts),
    atomic_list_concat(ArgTexts, ', ', ArgList),
    atomic_list_concat([Functor, '(', ArgList, ').'], Line).

%% quoted_atom(+Atom, -Text) — Atom as a re-readable quoted token, write_term flags pinned so the
% bytes never move under an ambient flag (the kb-writer serializer lesson): quote when the token
% needs it, escape control chars, printable non-ASCII literal. All v1 surfaces are ASCII; the pin
% keeps the projection deterministic regardless.
quoted_atom(Atom, Text) :-
    with_output_to(string(Text),
        write_term(Atom, [ quoted(true), character_escapes(true),
                           character_escapes_unicode(true), quote_non_ascii(false) ])).
