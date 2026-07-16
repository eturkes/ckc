% ClinicalCNL naive-NAF vs AceRules courteous differential (M3.court-differential; SPEC §6 lane
% separation — "differential tests cover only shared fragments: context satisfaction over finite
% fixture contexts, exception expansion equivalence"). A purpose-built ISOMORPHIC pair puts the
% clinical KB's PROLEG naive-NAF derivability (kb_kernel:derivable/3, a synthetic-fixture reference,
% NOT G-RULE-EVAL shipped evaluation) side by side with the vendored AceRules courteous solver
% (court_interpreter/3) over the republican/pacifist priority structure, across the exhaustive
% fact-presence truth table, asserting agreement on the shared fragment and DOCUMENTING where the
% two semantics diverge (the platypus divergence class — why courteous cannot serve as a 1:1 oracle,
% §L·acerules).
%
% The isomorphism (clinical recommend rule + contraindicate exception  <->  nixon priority pair):
%   quaker(Nixon)        <->  cond1 = the recommend statement's guard        (concept cond.sepsis)
%   pacifist(Nixon)      <->  the recommendation is concluded                (derivable stmt.0)
%   republican(Nixon)    <->  cond2 = the labeled contraindicate exception   (concept cond.renal_severe)
%   -pacifist(Nixon)     <->  the contraindication (has no positive clinical conclusion — see below)
%   Republican overrides Quaker  <->  the exception defeats the recommend (the AGAINST rule wins)
%
% court_interpreter/3 consumes (Label, Head, Body) tuples (answerset_generator.pl); a fact is a rule
% with empty body (_, Head, []); the returned answerset is a list of bare heads; a conflict between
% Head and its strong negation -Head resolves via the asserted overrides/2 priority. Driving the
% solver directly on the vendored nixon rule terms keeps this gate pure and fast (no live APE — only
% the `ape` search path, for court_interpreter's drs_to_ascii dependency); the known-answer control
% grounds the whole harness against testcases/court/output/nixon.
%
%   Gate: swipl -q -g "consult('clinical/court_differential_tests.pl'),(run_tests(court_differential)->halt(0);halt(1))" -t 'halt(1)'

:- module(court_differential_tests, []).

:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K),
   % court_interpreter's load closure (utils/debug_output) needs the `ape` search path resolved to
   % the vendored APE prolog source — set it BEFORE loading the solver so the closure loads clean.
   atomic_list_concat([D, '/../prolog'], ApeRel), absolute_file_name(ApeRel, ApeDir),
   asserta(user:file_search_path(ape, ApeDir)),
   atomic_list_concat([D, '/../vendor/acerules/engine/court_interpreter/court_interpreter.pl'], CI0),
   absolute_file_name(CI0, CI), use_module(CI),
   atomic_list_concat([D, '/../vendor/acerules/engine/testcases/court'], CC0),
   absolute_file_name(CC0, CC), asserta(court_cases_dir(CC)).

% ==========================================================================================
% Clinical side — a purpose-built, valid ClinicalCNL KB: one recommend rule (for/strong) guarded by
% cond1, carrying a labeled contraindicate exception cond2 (the nixon-isomorph).
% ==========================================================================================

clinical_kb([ rule('court_diff.rule.0', 'court_diff.stmt.0'),
              direction('court_diff.rule.0', for),
              strength('court_diff.rule.0', strong),
              population('court_diff.stmt.0', 'pop.patient'),
              condition('court_diff.bind.0', 'court_diff.stmt.0', concept('cond.sepsis')),
              action('court_diff.stmt.0', 'act.administer:drug.abx_a'),
              exception('court_diff.exc.0', 'court_diff.stmt.0', concept('cond.renal_severe')) ]).

cond1(concept('cond.sepsis')).          % the recommend guard   <-> quaker
cond2(concept('cond.renal_severe')).    % the contraindicate exc <-> republican

% clinical_recommends(+Ctx) — the recommend statement is derivable for fixture context Ctx (naive-NAF:
% every guard holds AND no labeled exception fires).
clinical_recommends(Ctx) :- clinical_kb(KB), derivable('court_diff.stmt.0', KB, Ctx).

% ==========================================================================================
% Court side — the republican/pacifist rule terms (verbatim from testcases/court/output/nixon RULES).
% ==========================================================================================

quaker_fact(('', object(named('Nixon'), quaker,     countable, na, eq, 1), [])).
republican_fact(('', object(named('Nixon'), republican, countable, na, eq, 1), [])).

recommend_head(group([object(named('Nixon'), pacifist, countable, na, eq, 1)])).    % pacifist
contra_head(-group([object(named('Nixon'), pacifist, countable, na, eq, 1)])).      % -pacifist

nixon_priority(overrides('Republican-Rule', 'Quaker-Rule')).    % contra overrides recommend (AGAINST wins)
reversed_priority(overrides('Quaker-Rule', 'Republican-Rule')). % recommend overrides contra (FOR wins — platypus class)

% court_program(+FactSubset, +Overrides, -Rules) — the fixed Quaker/Republican rules + priority over a
% subset of the two participating facts (fresh guard vars per rule).
court_program(FactSubset, Overrides, Rules) :-
    Identity   = ('', object(named('Nixon'), 'Nixon', named, na, eq, 1), []),
    Quaker     = ('Quaker-Rule',     group([object(A, pacifist, countable, na, eq, 1)]),  [object(A, quaker, countable, na, eq, 1)]),
    Republican = ('Republican-Rule', -group([object(B, pacifist, countable, na, eq, 1)]), [object(B, republican, countable, na, eq, 1)]),
    append([Identity | FactSubset], [Quaker, Republican, Overrides], Rules).

% court_answerset/court_recommends/court_contraindicates — court gives one answerset (once/1 keeps the
% gate choicepoint-clean).
court_answerset(FactSubset, Overrides, AS) :-
    court_program(FactSubset, Overrides, Rules),
    once(court_interpreter(Rules, AS, _)).
court_recommends(FactSubset, Overrides) :-
    court_answerset(FactSubset, Overrides, AS), recommend_head(H), memberchk(H, AS).
court_contraindicates(FactSubset, Overrides) :-
    court_answerset(FactSubset, Overrides, AS), contra_head(H), memberchk(H, AS).

% ==========================================================================================
% The exhaustive truth table. The isomorphism maps clinical cond_i <-> court fact_i, so each subset
% of the participating facts is a (clinical fixture context, court fact subset) pair.
% ==========================================================================================

row(none, [],       []).
row(c1,   [C1],     [Q])    :- cond1(C1), quaker_fact(Q).
row(c2,   [C2],     [R])    :- cond2(C2), republican_fact(R).
row(c1c2, [C1, C2], [Q, R]) :- cond1(C1), cond2(C2), quaker_fact(Q), republican_fact(R).

% fixture_says(+Name, +Substring) — Name's vendored courteous output file contains Substring (the
% published verdict). Anchors the hand-pinned court expectations to testcases/court/output/*.
fixture_says(Name, Substring) :-
    court_cases_dir(CC),
    atomic_list_concat([CC, '/output/', Name], File),
    read_file_to_string(File, S, []),
    once(sub_string(S, _, _, _, Substring)).

:- begin_tests(court_differential).

% --- Grounding: the court harness reproduces the vendored fixture. --------------------------------

% court_interpreter on the full nixon program (both facts, contra-overrides-recommend) reproduces the
% published testcases/court/output/nixon answerset exactly — grounds every court_* helper below.
test(known_answer_control) :-
    quaker_fact(Q), republican_fact(R), nixon_priority(P),
    court_answerset([Q, R], P, AS),
    msort(AS, Got),
    msort([ -group([object(named('Nixon'), pacifist, countable, na, eq, 1)]),
            object(named('Nixon'), 'Nixon', named, na, eq, 1),
            object(named('Nixon'), quaker, countable, na, eq, 1),
            object(named('Nixon'), republican, countable, na, eq, 1) ], Want),
    Got == Want.

% The published nixon verdict (the AGAINST rule wins) — the source of the hand-pinned expectation.
test(nixon_fixture_verdict) :-
    fixture_says(nixon, "It is false that Nixon is a pacifist.").

% The clinical fixture is a real, valid ClinicalCNL KB (not a toy term set).
test(clinical_kb_valid) :-
    clinical_kb(KB), valid_kb(KB).

% --- Core: coincidence on the shared fragment ("is the recommendation concluded?"). --------------

% Over the exhaustive truth table, naive-NAF derivability of the recommend COINCIDES with the
% courteous recommend verdict when the priority favours the AGAINST rule (nixon direction): the NAF
% exception blocks the recommend exactly where courteous priority resolution suppresses pacifist.
test(differential_coincides_nixon_direction) :-
    nixon_priority(P),
    forall( row(_Name, Ctx, Facts),
            ( ( clinical_recommends(Ctx)  -> CR = rec ; CR = no ),
              ( court_recommends(Facts, P) -> KR = rec ; KR = no ),
              CR == KR ) ).

% Per-row verdicts pinned (anti-vacuity for the forall + the explicit differential table). Columns:
% clinical-recommend, court-recommend, court-contraindicate. The recommend columns agree on every
% row; the contraindicate column shows the courteous-only conclusion (documented below).
test(differential_row_verdicts_nixon) :-
    nixon_priority(P),
    findall(Name-CR-KR-KC,
            ( row(Name, Ctx, Facts),
              ( clinical_recommends(Ctx)     -> CR = rec    ; CR = no ),
              ( court_recommends(Facts, P)    -> KR = rec    ; KR = no ),
              ( court_contraindicates(Facts, P) -> KC = contra ; KC = no ) ),
            Rows),
    Rows == [ none-no-no-no, c1-rec-rec-no, c2-no-no-contra, c1c2-no-no-contra ].

% --- Documented divergences (naive-NAF vs courteous). --------------------------------------------

% Reverse the priority so the FOR rule wins (recommend overrides contra): on {c1,c2} the courteous
% verdict now CONCLUDES the recommendation, but the clinical naive-NAF exception STILL blocks it — a
% NAF guard is direction-blind and cannot express "the recommendation overrides its own exception".
% This is the platypus divergence class in minimal form.
test(divergence_reversed_priority) :-
    reversed_priority(P),
    cond1(C1), cond2(C2), quaker_fact(Q), republican_fact(R),
    \+ clinical_recommends([C1, C2]),      % naive-NAF: the exception blocks the recommend
    court_recommends([Q, R], P).           % courteous: the recommend wins by priority

% The named platypus case, structurally reproduced: a 4-rule / 2-priority courteous program where two
% FOR rules each override a competing AGAINST rule concludes the positive even though every AGAINST
% guard holds. A naive-NAF model (recommended-unless-eggs / recommended-unless-bill) would block it.
test(divergence_platypus_analog) :-
    Rules = [ ('', monotreme, []), ('', fur, []), ('', eggs, []), ('', bill, []),
              ('R1', mammal, [monotreme]), ('R2', mammal, [fur]),
              ('R3', -mammal, [eggs]),     ('R4', -mammal, [bill]),
              overrides('R1', 'R3'), overrides('R2', 'R4') ],
    once(court_interpreter(Rules, AS, _)),
    memberchk(mammal, AS),       % courteous: the FOR rules win
    \+ memberchk(-mammal, AS).   % the AGAINST conclusion is suppressed

% Anchors the analog's expected verdict to the vendored testcases/court/output/platypus fixture.
test(platypus_fixture_verdict) :-
    fixture_says(platypus, "John is a mammal.").

% Representational gap (outside the shared fragment): the courteous program derives an INDEPENDENT
% contraindication (-pacifist) whenever the against-guard holds — including when the recommend guard
% is absent ({c2}) — whereas the clinical KB models the contraindication ONLY as a NAF exception
% guard, never a positive conclusion. So "is a contraindication concluded?" is not a shared query;
% the differential compares the recommend bit.
test(contra_is_courteous_only) :-
    nixon_priority(P),
    cond2(C2), republican_fact(R),
    court_contraindicates([R], P),         % courteous concludes -pacifist from republican alone
    \+ clinical_recommends([C2]).          % clinical: nothing derivable (the recommend guard is absent)

:- end_tests(court_differential).
