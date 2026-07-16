% ClinicalCNL naive-NAF vs AceRules courteous differential (M3.court-differential; SPEC §6 lane
% separation — "differential tests cover only shared fragments: context satisfaction over finite
% fixture contexts, exception expansion equivalence"). A purpose-built ISOMORPHIC pair puts the
% clinical KB's PROLEG naive-NAF derivability (kb_kernel:derivable/3, a synthetic-fixture reference,
% NOT G-RULE-EVAL shipped evaluation) side by side with the vendored AceRules courteous solver
% (court_interpreter/3) over the republican/pacifist priority structure, across the exhaustive
% fact-presence truth table, asserting agreement on the shared fragment and DOCUMENTING (both sides
% computed) where the two semantics diverge — the platypus divergence class, why courteous cannot
% serve as a 1:1 oracle, §L·acerules.
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
% solver directly on the vendored nixon rule terms keeps this gate offline and fast (no live APE, no
% build step — the sole load-time side effect is the `ape` search-path assertion court_interpreter's
% drs_to_ascii closure needs); the known-answer control grounds the court helpers against a
% hand-pinned expectation matching testcases/court/output/nixon (a light fixture-text anchor, not a
% full byte-pin of the fixture RULES/ANSWERSET — the rule terms are transcribed alpha-equivalent).
%
% Non-vacuity discipline: every court verdict is read from ONE required answerset whose Identity fact
% head must be present (a witness that the solver actually ran and returned a real answerset, never a
% silent []); divergence tests assert the positive present AND the negative absent, and each carries
% a counterfactual (drop the priorities / read the guards-only context) proving the conflict and its
% resolution are load-bearing rather than incidental.
%
%   Gate: swipl -q -g "consult('clinical/court_differential_tests.pl'),(run_tests(court_differential)->halt(0);halt(1))" -t 'halt(1)'

:- module(court_differential_tests, []).

:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/kb_kernel.pl'], K), use_module(K),
   % court_interpreter's load closure (utils/debug_output) needs the `ape` search path resolved to
   % the vendored APE prolog source — set it BEFORE loading the solver so the closure loads clean.
   % (This global assertion is the gate's one load-time side effect; behavior stays deterministic
   % under the sequential plunit run.)
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

% clinical_contraindicates(+KB, +Ctx) — some NON-positive-direction (against / avoid / contraindicate)
% statement is derivable for Ctx. The clinical_kb declares only a `for` statement — the
% contraindication lives ONLY as that statement's labeled NAF exception, never a positive conclusion —
% so this fails for every fixture context: the courteous -pacifist has no clinical counterpart.
clinical_contraindicates(KB, Ctx) :-
    member(rule(R, S), KB), member(direction(R, Dir), KB),
    \+ direction_group(Dir, positive),
    derivable(S, KB, Ctx).

% platypus_kb — the higher-arity clinical mirror of the courteous platypus fixture: two recommend
% statements for the SAME action, each guarded (cond.sepsis / cond.pregnancy), each defeated by a
% distinct labeled exception (concept cond.renal_severe / interval age >= 65). When both guards AND
% both exception atoms hold, naive-NAF blocks BOTH recommends — the clinical half of the platypus
% divergence at 2-support / 2-defeater arity.
platypus_kb([ rule('court_diff.p.rule.1', 'court_diff.p.stmt.1'),
              direction('court_diff.p.rule.1', for), strength('court_diff.p.rule.1', strong),
              population('court_diff.p.stmt.1', 'pop.patient'),
              condition('court_diff.p.bind.1', 'court_diff.p.stmt.1', concept('cond.sepsis')),
              action('court_diff.p.stmt.1', 'act.administer:drug.abx_a'),
              exception('court_diff.p.exc.1', 'court_diff.p.stmt.1', concept('cond.renal_severe')),
              rule('court_diff.p.rule.2', 'court_diff.p.stmt.2'),
              direction('court_diff.p.rule.2', for), strength('court_diff.p.rule.2', strong),
              population('court_diff.p.stmt.2', 'pop.patient'),
              condition('court_diff.p.bind.2', 'court_diff.p.stmt.2', concept('cond.pregnancy')),
              action('court_diff.p.stmt.2', 'act.administer:drug.abx_a'),
              exception('court_diff.p.exc.2', 'court_diff.p.stmt.2', interval('q.age_years', 65, closed, lower)) ]).

% ==========================================================================================
% Court side — the republican/pacifist rule terms (transcribed alpha-equivalent from
% testcases/court/output/nixon RULES; grounded by known_answer_control below).
% ==========================================================================================

quaker_fact(('', object(named('Nixon'), quaker,     countable, na, eq, 1), [])).
republican_fact(('', object(named('Nixon'), republican, countable, na, eq, 1), [])).

recommend_head(group([object(named('Nixon'), pacifist, countable, na, eq, 1)])).    % pacifist
contra_head(-group([object(named('Nixon'), pacifist, countable, na, eq, 1)])).      % -pacifist
identity_head(object(named('Nixon'), 'Nixon', named, na, eq, 1)).                    % the Identity fact's head

nixon_priority([overrides('Republican-Rule', 'Quaker-Rule')]).    % contra overrides recommend (AGAINST wins)
reversed_priority([overrides('Quaker-Rule', 'Republican-Rule')]). % recommend overrides contra (FOR wins — platypus class)
no_priority([]).                                                  % unresolved conflict (skeptical — both suppressed)

% court_program(+FactSubset, +Priorities, -Rules) — the fixed Quaker/Republican rules + a list of
% priority terms (0 or 1 overrides) over a subset of the two participating facts (fresh guard vars
% per rule); the Identity fact seeds a witness head into every answerset.
court_program(FactSubset, Priorities, Rules) :-
    Identity   = ('', object(named('Nixon'), 'Nixon', named, na, eq, 1), []),
    Quaker     = ('Quaker-Rule',     group([object(A, pacifist, countable, na, eq, 1)]),  [object(A, quaker, countable, na, eq, 1)]),
    Republican = ('Republican-Rule', -group([object(B, pacifist, countable, na, eq, 1)]), [object(B, republican, countable, na, eq, 1)]),
    append([Identity | FactSubset], [Quaker, Republican | Priorities], Rules).

% court_answerset(+FactSubset, +Priorities, -AS) — court's single answerset (once/1 keeps the gate
% choicepoint-clean).
court_answerset(FactSubset, Priorities, AS) :-
    court_program(FactSubset, Priorities, Rules),
    once(court_interpreter(Rules, AS, _)).

% court_verdict(+FactSubset, +Priorities, -Rec, -Contra) — read BOTH verdict bits from one REQUIRED
% answerset. The Identity fact head must be present: a witness proving the solver actually ran and
% returned a real answerset, so a silent [] (broken/vacuous execution) FAILS the verdict instead of
% masquerading as "recommend=no". Rec in {rec,no}, Contra in {contra,no}.
court_verdict(FactSubset, Priorities, Rec, Contra) :-
    court_answerset(FactSubset, Priorities, AS),
    identity_head(IdH), memberchk(IdH, AS),
    ( recommend_head(RH), memberchk(RH, AS) -> Rec = rec ; Rec = no ),
    ( contra_head(CH),    memberchk(CH, AS) -> Contra = contra ; Contra = no ).

% ==========================================================================================
% The exhaustive truth table. The isomorphism maps clinical cond_i <-> court fact_i, so each subset
% of the participating facts is a (clinical fixture context, court fact subset) pair.
% ==========================================================================================

row(none, [],       []).
row(c1,   [C1],     [Q])    :- cond1(C1), quaker_fact(Q).
row(c2,   [C2],     [R])    :- cond2(C2), republican_fact(R).
row(c1c2, [C1, C2], [Q, R]) :- cond1(C1), cond2(C2), quaker_fact(Q), republican_fact(R).

% fixture_says(+Name, +Substring) — Name's vendored courteous output file contains Substring (one
% published sentence). A light anchor tying the hand-pinned court expectations to
% testcases/court/output/* (not a full byte-pin of the fixture).
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

% Both clinical fixtures are real, valid ClinicalCNL KBs (not toy term sets).
test(clinical_kb_valid) :-
    clinical_kb(KB), valid_kb(KB).

test(platypus_kb_valid) :-
    platypus_kb(KB), valid_kb(KB).

% --- Core: coincidence on the shared fragment ("is the recommendation concluded?"). --------------

% Over the exhaustive truth table, naive-NAF derivability of the recommend COINCIDES with the
% courteous recommend verdict when the priority favours the AGAINST rule (nixon direction): the NAF
% exception blocks the recommend exactly where courteous priority resolution suppresses pacifist.
% court_verdict is REQUIRED per row (a failed solve fails the test, never a silent "no").
test(differential_coincides_nixon_direction) :-
    nixon_priority(P),
    forall( row(_Name, Ctx, Facts),
            ( ( clinical_recommends(Ctx) -> CR = rec ; CR = no ),
              court_verdict(Facts, P, KR, _),
              CR == KR ) ).

% Coincidence is not unique to the AGAINST-wins priority: with NO priority the courteous conflict is
% unresolved, so the courteous semantics (skeptical) suppresses the recommend on {c1c2} exactly as
% the NAF exception does — the recommend bit coincides here too. Divergence needs the FOR rule to WIN
% (see divergence_reversed_priority). This pins the scope of the "coincides" claim.
test(differential_coincides_no_priority) :-
    no_priority(P),
    forall( row(_Name, Ctx, Facts),
            ( ( clinical_recommends(Ctx) -> CR = rec ; CR = no ),
              court_verdict(Facts, P, KR, _),
              CR == KR ) ).

% Per-row verdicts pinned (anti-vacuity for the foralls + the explicit differential table). Columns:
% clinical-recommend, court-recommend, court-contraindicate. The recommend columns agree on every
% row; the contraindicate column shows the courteous-only conclusion (documented below).
test(differential_row_verdicts_nixon) :-
    nixon_priority(P),
    findall(Name-CR-KR-KC,
            ( row(Name, Ctx, Facts),
              ( clinical_recommends(Ctx) -> CR = rec ; CR = no ),
              court_verdict(Facts, P, KR, KC) ),
            Rows),
    Rows == [ none-no-no-no, c1-rec-rec-no, c2-no-no-contra, c1c2-no-no-contra ].

% --- Documented divergences (naive-NAF vs courteous), both sides computed. -----------------------

% Reverse the priority so the FOR rule wins (recommend overrides contra): on {c1,c2} the courteous
% verdict now CONCLUDES the recommendation (pacifist present AND -pacifist suppressed), but the
% clinical naive-NAF exception STILL blocks it — a NAF guard is direction-blind and cannot express
% "the recommendation overrides its own exception". The platypus divergence class in minimal form.
test(divergence_reversed_priority) :-
    reversed_priority(P),
    cond1(C1), cond2(C2), quaker_fact(Q), republican_fact(R),
    \+ clinical_recommends([C1, C2]),      % naive-NAF: the exception blocks the recommend
    court_verdict([Q, R], P, KR, KC),      % courteous: the recommend wins by priority ...
    KR == rec, KC == no.                   % ... pacifist concluded, -pacifist suppressed

% Court-side structural control grounded on the vendored platypus fixture: a 4-rule / 2-priority
% courteous program where two FOR rules each override a competing AGAINST rule concludes the positive
% though every AGAINST guard holds. The no-priority counterfactual makes the overrides LOAD-BEARING:
% the SAME four rules WITHOUT the priorities leave the conflict unresolved, so courteous suppresses
% BOTH heads — proving mammal comes from the priority resolution, not mere rule presence.
test(platypus_courteous_control) :-
    Base = [ ('', monotreme, []), ('', fur, []), ('', eggs, []), ('', bill, []),
             ('R1', mammal, [monotreme]), ('R2', mammal, [fur]),
             ('R3', -mammal, [eggs]),     ('R4', -mammal, [bill]) ],
    append(Base, [overrides('R1', 'R3'), overrides('R2', 'R4')], Prioritised),
    once(court_interpreter(Prioritised, AS, _)),
    memberchk(mammal, AS), \+ memberchk(-mammal, AS),        % priorities present: the FOR rules win
    once(court_interpreter(Base, AS0, _)),
    \+ memberchk(mammal, AS0), \+ memberchk(-mammal, AS0).   % counterfactual: no priorities -> both suppressed

% The platypus divergence with BOTH sides computed at 2-support / 2-defeater arity: the courteous
% analog concludes mammal, while the isomorphic clinical KB (two recommend statements, each guarded,
% each defeated by a distinct labeled exception) derives NEITHER recommend once both exception atoms
% hold. The guards-only witness proves the block is the EXCEPTIONS firing, not absent guards.
test(divergence_platypus_clinical) :-
    Prioritised = [ ('', monotreme, []), ('', fur, []), ('', eggs, []), ('', bill, []),
                    ('R1', mammal, [monotreme]), ('R2', mammal, [fur]),
                    ('R3', -mammal, [eggs]),     ('R4', -mammal, [bill]),
                    overrides('R1', 'R3'), overrides('R2', 'R4') ],
    once(court_interpreter(Prioritised, AS, _)),
    memberchk(mammal, AS),                                   % courteous: concludes the positive
    platypus_kb(KB),
    Guards = [ concept('cond.sepsis'), concept('cond.pregnancy') ],
    derivable('court_diff.p.stmt.1', KB, Guards),            % witness: recommend 1 derivable when its defeater is absent
    derivable('court_diff.p.stmt.2', KB, Guards),            % witness: recommend 2 derivable when its defeater is absent
    AllHold = [ concept('cond.sepsis'), concept('cond.pregnancy'),
                concept('cond.renal_severe'), quantity('q.age_years', 70) ],
    \+ derivable('court_diff.p.stmt.1', KB, AllHold),        % naive-NAF: exception (renal_severe) blocks recommend 1
    \+ derivable('court_diff.p.stmt.2', KB, AllHold).        % naive-NAF: exception (age >= 65) blocks recommend 2

% Anchors the analog's expected verdict to the vendored testcases/court/output/platypus fixture.
test(platypus_fixture_verdict) :-
    fixture_says(platypus, "John is a mammal.").

% Representational gap (outside the shared fragment): the courteous program derives an INDEPENDENT
% contraindication (-pacifist) from republican alone, whereas the clinical KB never concludes a
% contraindication positively — it declares no against/avoid/contraindicate statement at all; the
% contraindication is modeled ONLY as stmt.0's NAF exception guard. So "is a contraindication
% concluded?" is a courteous-only query, absent from the clinical lane.
test(contra_is_courteous_only) :-
    nixon_priority(P), cond1(C1), cond2(C2), republican_fact(R),
    court_verdict([R], P, _, KC), KC == contra,          % courteous: -pacifist concluded from republican alone
    clinical_kb(KB),
    \+ clinical_contraindicates(KB, [C1, C2]),           % clinical: no positive contraindication derivable (even with both guards)
    \+ ( member(direction(_, Dir), KB), \+ direction_group(Dir, positive) ),  % structurally: no against/avoid/contraindicate rule
    once(member(exception(_, 'court_diff.stmt.0', C2), KB)).                   % the contraindication IS present, as an exception

:- end_tests(court_differential).
