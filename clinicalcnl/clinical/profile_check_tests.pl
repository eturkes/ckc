% ClinicalCNL profile checker gate (M3.profile-drs). Reconstructs each byte-pinned surface golden
% DRS (surface_expected.pl) to its real-variable term — a serialized DRS reads back to the same
% term, and every reconstruction re-serializes byte-identically (test golden_roundtrip proves it),
% so the checker runs against provably-faithful APE output with no live APE dependency. The v1
% goldens must all pass (accept battery); the nonv1 goldens must all reject on their characteristic
% path (non-vacuity). The exhaustive hand-mutant DRS reject coverage is the profile-battery unit.
%
%   Gate: swipl -q -g "consult('clinical/profile_check_tests.pl'),(run_tests(profile_check)->halt(0);halt(1))" -t 'halt(1)'

:- module(profile_check_tests, []).
:- use_module(library(plunit)).

:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/profile_check.pl'], PC), use_module(PC),
   atomic_list_concat([D, '/../prolog/utils/serialize_term.pl'], ST), use_module(ST),
   atomic_list_concat([D, '/goldens/surface_cases.pl'], SC), use_module(SC),
   atomic_list_concat([D, '/goldens/surface_expected.pl'], SE), use_module(SE).

%% golden_drs(+Id, -Drs, -Messages) — the reconstructed DRS term + pinned messages for a text/plain
% golden. read_term_from_atom rebuilds the real referent vars (equal uppercase names share one fresh
% var); the reconstruction's fidelity is proven separately (golden_roundtrip).
golden_drs(Id, Drs, Msgs) :-
    surface_expected(Id, 'text/plain', Content, Msgs),
    read_term_from_atom(Content, Drs, []).

%% golden_ctx(?Id, ?Ctx) — each golden's raw-gate context: a v1 rule surface's modality keyword
% (whose decoded op must match the frame), or the exception body. The nonv1 rows carry a rule Ctx
% too; their shape / message reject fires regardless (named_hole and iv_anaphor on the zero-message
% law, before any Ctx-sensitive check).
golden_ctx(frame_recommend,     rule(0, recommend,       0, none, none)).
golden_ctx(frame_admissible,    rule(0, 'may-consider',  0, none, none)).
golden_ctx(frame_not_recommend, rule(0, 'not-recommend', 0, none, none)).
golden_ctx(frame_not_possible,  rule(0, contraindicate,  0, none, none)).
golden_ctx(iv_at_least,         rule(0, recommend,       0, none, none)).
golden_ctx(iv_more_than,        rule(0, recommend,       0, none, none)).
golden_ctx(iv_at_most,          rule(0, recommend,       0, none, none)).
golden_ctx(iv_less_than,        rule(0, recommend,       0, none, none)).
golden_ctx(thread_doc_a,        rule(0, recommend,       0, none, none)).
golden_ctx(thread_doc_b,        rule(0, contraindicate,  0, none, none)).
golden_ctx(thread_control,      rule(0, contraindicate,  0, none, none)).
golden_ctx(exception_body,      exception(0, 0, none, none)).
golden_ctx(guard_neg,           rule(0, recommend,       0, none, none)).
golden_ctx(iv_exactly,          rule(0, recommend,       0, none, none)).
golden_ctx(iv_bare,             rule(0, recommend,       0, none, none)).
golden_ctx(iv_anaphor,          rule(0, recommend,       0, none, none)).
golden_ctx(named_hole,          rule(0, recommend,       0, none, none)).

%% nonv1_reject(?Id, ?Reason) — each nonv1 golden's characteristic reject, one per rejection path:
% in-guard negation (guard shape), a non-v1 interval bound (exactly / bare eq), and the zero-message
% law (anaphor / undefined-word warnings). This is the accept battery's non-vacuity floor; the
% exhaustive DRS-side reject coverage lands in profile-battery.
nonv1_reject(guard_neg,  reject(guard_shape(_))).
nonv1_reject(iv_exactly, reject(interval_countop(exactly))).
nonv1_reject(iv_bare,    reject(interval_countop(eq))).
nonv1_reject(iv_anaphor, reject(nonempty_messages)).
nonv1_reject(named_hole, reject(nonempty_messages)).

%% crafted_reject(?Label, ?Ctx, ?Drs, ?Reason) — hand-built non-golden DRS terms pinning the two
% acceptance escapes closed after codex-review: an action target that is not a ground drug name
% (F1 — `Target = named(_)` unified a bare referent var to ok, and an unbound / non-atom name bound
% through pn_allow/1), and a negative interval bound (F4 — only integer/1 was checked, not the raw
% gate's non-negativity). Targeted regression guards; the exhaustive DRS-side reject matrix stays
% profile-battery's scope.
crafted_reject('action target = action referent var', rule(0, recommend, 0, none, none),
  drs([],[=>(drs([P,C,H],[object(P,patient,countable,na,eq,1)-1/3,object(C,sepsis,countable,na,eq,1)-1/6,predicate(H,have,P,C)-1/4]),
             drs([],[should(drs([A],[predicate(A,take,P,A)-1/14]))]))]),
  reject(bad_action_target(_))).
crafted_reject('action target = patient referent var', rule(0, recommend, 0, none, none),
  drs([],[=>(drs([P,C,H],[object(P,patient,countable,na,eq,1)-1/3,object(C,sepsis,countable,na,eq,1)-1/6,predicate(H,have,P,C)-1/4]),
             drs([],[should(drs([A],[predicate(A,take,P,P)-1/14]))]))]),
  reject(bad_action_target(_))).
crafted_reject('action target = non-atom drug name', rule(0, recommend, 0, none, none),
  drs([],[=>(drs([P,C,H],[object(P,patient,countable,na,eq,1)-1/3,object(C,sepsis,countable,na,eq,1)-1/6,predicate(H,have,P,C)-1/4]),
             drs([],[should(drs([A],[predicate(A,take,P,named(foo(bar)))-1/14]))]))]),
  reject(unregistered_named(foo(bar)))).
crafted_reject('negative interval bound', rule(0, recommend, 0, none, none),
  drs([],[=>(drs([P,Q,U,H],[object(P,patient,countable,na,eq,1)-1/3,object(Q,age,countable,na,eq,1)-1/6,object(U,year,countable,na,geq,-18)-1/11,relation(Q,of,U)-1/7,predicate(H,have,P,Q)-1/4]),
             drs([],[should(drs([A],[predicate(A,take,P,named('Abx-A'))-1/19]))]))]),
  reject(interval_bound(_))).

:- begin_tests(profile_check).

% Accept battery: every v1 surface's canonical DRS passes under its raw-gate Ctx.
test(accept_v1, [forall(( surface_case(Id, v1, _), golden_ctx(Id, Ctx) ))]) :-
    golden_drs(Id, Drs, Msgs),
    profile_check(Ctx, Drs, Msgs, Result),
    assertion(Result == ok).

% Every v1 surface case is covered by the accept battery (no case silently skipped for want of a Ctx).
test(accept_covers_all_v1) :-
    findall(Id, surface_case(Id, v1, _), V1s),
    findall(Id, (surface_case(Id, v1, _), golden_ctx(Id, _)), Covered),
    assertion(V1s == Covered).

% Non-vacuity: every nonv1 golden (real APE output, excluded from v1) rejects on its path.
test(reject_nonv1, [forall(nonv1_reject(Id, Reason))]) :-
    golden_ctx(Id, Ctx),
    golden_drs(Id, Drs, Msgs),
    profile_check(Ctx, Drs, Msgs, Result),
    assertion(Result = Reason).

% Every nonv1 surface case is covered by the reject battery.
test(reject_covers_all_nonv1) :-
    findall(Id, surface_case(Id, nonv1, _), NV1s), msort(NV1s, S1),
    findall(Id, nonv1_reject(Id, _), Rejected), msort(Rejected, S2),
    assertion(S1 == S2).

% Regression: each crafted non-v1 DRS rejects on its pinned path (the two post-review fixes).
test(reject_crafted_escapes, [forall(crafted_reject(_Label, Ctx, Drs, Reason))]) :-
    profile_check(Ctx, Drs, [], Result),
    assertion(Result = Reason).

% Reconstruction fidelity: every text/plain golden re-serializes byte-identically, so the term the
% checker sees is APE's canonical parse, not an artefact of read-back. Runs last: serialize_term
% permanently narrows the `- ~ => v &` operators, and functional read-back above is unaffected.
test(golden_roundtrip, [forall(surface_expected(_, 'text/plain', Content, _))]) :-
    read_term_from_atom(Content, Drs, []),
    serialize_term_into_atom(Drs, Re),
    assertion(Re == Content).

:- end_tests(profile_check).
