% ClinicalCNL interval-algebra gate (M3.interval-algebra). Hand-oracled over intervals.pl — a pure
% arithmetic module, so the whole battery runs with no live APE. Two legs: the 16-mask single-bound
% validity battery (256 slot-assignment combos vs an INDEPENDENT re-statement of the single-bound law)
% and the open/closed boundary properties over Q (geq/greater/less adjacency, the dense-order case, the
% §L·thread age-disjoint control), every expectation hand-written.
%
%   Gate: swipl -q -g "consult('clinical/intervals_tests.pl'),(run_tests(intervals)->halt(0);halt(1))" -t 'halt(1)'

:- module(intervals_tests, []).

:- use_module(library(plunit)).
:- use_module(library(lists)).

:- prolog_load_context(directory, D),
   atomic_list_concat([D, '/intervals.pl'], I), use_module(I).

% ---- 16-mask validity battery generator + independent oracle --------------------------------
% The four (Openness,Dir) slots = the four v1 markers {geq,greater,leq,less}. Each slot is assigned a
% choice ∈ {absent, present with value -1|0|1}: 4 slots × 4 choices = 256 combos, spanning all 16
% presence masks with the present bounds ranging over {-1,0,1}. The single-bound law (valid iff exactly
% one bound present ∧ its value >= 0) is re-stated INDEPENDENTLY here (list length + the value), so the
% battery differentially checks valid_v1_interval/1 across the whole space rather than echoing it.

slots([od(closed,lower), od(open,lower), od(closed,upper), od(open,upper)]).

choice(absent).
choice(present(-1)).
choice(present(0)).
choice(present(1)).

% mask_combo(-Bounds): one of the 256 slot-assignment combinations, as its present-bound list.
mask_combo(Bounds) :-
    slots(Slots),
    assign(Slots, Bounds).

assign([], []).
assign([od(O,D)|Slots], Bounds) :-
    choice(C),
    ( C = absent     -> Bounds = Rest
    ; C = present(V) -> Bounds = [bound(V,O,D)|Rest]
    ),
    assign(Slots, Rest).

% oracle_valid(+Bounds, -Valid): the hand-stated single-bound law, independent of the module.
oracle_valid(Bounds, Valid) :-
    length(Bounds, N),
    ( N =:= 1, Bounds = [bound(V,_,_)], V >= 0 -> Valid = true ; Valid = false ).

:- begin_tests(intervals).

% ---- 16-mask validity ------------------------------------------------------------------------

% The generator spans exactly 256 combos and both verdicts occur (8 valid = 4 slots × values {0,1};
% 248 invalid), so the differential battery below cannot pass vacuously.
test(mask_space_shape) :-
    findall(B, mask_combo(B), All),                        length(All, 256),
    findall(B, (mask_combo(B), oracle_valid(B, true)),  V), length(V, 8),
    findall(B, (mask_combo(B), oracle_valid(B, false)), I), length(I, 248).

% valid_v1_interval/1 agrees with the independent single-bound oracle on every mask combo.
test(mask_16_validity) :-
    forall( mask_combo(Bounds),
            ( oracle_valid(Bounds, Expected),
              ( valid_v1_interval(Bounds) -> Got = true ; Got = false ),
              ( Got == Expected
              -> true
              ;  format(user_error, "intervals: mask ~w expected ~w got ~w~n", [Bounds, Expected, Got]),
                 fail ) ) ).

% Explicit hand-written mask pins — literal expectations immune to an oracle bug (anti-vacuity anchor).
test(mask_pins) :-
    \+ valid_v1_interval([]),                                                   % empty mask
    valid_v1_interval([bound(0, closed, lower)]),                              % single geq, value 0
    valid_v1_interval([bound(18, open, upper)]),                              % single less, value 18
    \+ valid_v1_interval([bound(-1, closed, lower)]),                          % single, negative value
    \+ valid_v1_interval([bound(18, closed, lower), bound(18, open, upper)]), % two bounds (a range)
    \+ valid_v1_interval([bound(18, closed, lower), bound(20, closed, lower)]),% two lowers
    \+ valid_v1_interval([bound(18.0, closed, lower)]).                        % float value (not exact)

% ---- bounds -----------------------------------------------------------------------------------

test(valid_bound_ok) :-
    valid_bound(bound(18, closed, lower)),
    valid_bound(bound(0, open, upper)),
    valid_bound(bound(1r2, open, lower)),        % exact rational
    valid_bound(bound(-3, closed, upper)).       % a negative bound is well-formed (value law is v1-only)

test(valid_bound_rejects) :-
    \+ valid_bound(bound(18.0, closed, lower)),  % float — D10 exactness
    \+ valid_bound(bound(18, ajar, lower)),      % bad openness
    \+ valid_bound(bound(18, closed, sideways)), % bad dir
    \+ valid_bound(bound(foo, closed, lower)).   % non-numeric value

test(interval_bound_bridge) :-
    interval_bound(interval('q.age_years', 18, closed, lower), Q, B),
    assertion(Q == 'q.age_years'),
    assertion(B == bound(18, closed, lower)).

% ---- range folding ---------------------------------------------------------------------------

% Empty / single-sided folds — never empty (unbounded on the missing side).
test(range_unbounded) :-
    bounds_range([], range(none, none)),
    bounds_range([bound(18, closed, lower)], range(bound(18, closed, lower), none)),
    bounds_range([bound(18, open, upper)],   range(none, bound(18, open, upper))).

% Tightening a lower: the greater value wins; at an equal value the open bound (strict >) wins.
test(range_tighten_lower) :-
    bounds_range([bound(18, closed, lower), bound(20, closed, lower)], R1),
    assertion(R1 == range(bound(20, closed, lower), none)),                % higher value
    bounds_range([bound(18, closed, lower), bound(18, open, lower)],    R2),
    assertion(R2 == range(bound(18, open, lower), none)).                  % open beats closed

% Tightening an upper: the lesser value wins; at an equal value the open bound (strict <) wins.
test(range_tighten_upper) :-
    bounds_range([bound(19, closed, upper), bound(18, closed, upper)], R1),
    assertion(R1 == range(none, bound(18, closed, upper))),                % lower value
    bounds_range([bound(19, closed, upper), bound(19, open, upper)],    R2),
    assertion(R2 == range(none, bound(19, open, upper))).                  % open beats closed

% ---- open/closed boundary properties (D10 dense-order, geq/greater/less adjacency) ----------

% The load-bearing dense-order case: `18 < X < 19` is EMPTY over the integers but NON-empty over Q.
test(dense_open_open_nonempty) :-
    bounds_satisfiable([bound(18, open, lower), bound(19, open, upper)]),          % 18 <  X <  19 (37r2)
    \+ range_empty(range(bound(18, open, lower), bound(19, open, upper))).

% Half-open across adjacent integers stays non-empty.
test(dense_half_open_nonempty) :-
    bounds_satisfiable([bound(18, closed, lower), bound(19, open, upper)]),        % 18 =< X <  19 (18)
    bounds_satisfiable([bound(18, open, lower),   bound(19, closed, upper)]).      % 18 <  X =< 19 (19)

% Equal endpoints: the shared point is included iff BOTH bounds are closed.
test(equal_endpoint_openness) :-
    bounds_satisfiable([bound(18, closed, lower), bound(18, closed, upper)]),      % {18}
    range_empty(range(bound(18, open, lower),   bound(18, open, upper))),          % 18 <  X <  18 = ∅
    range_empty(range(bound(18, closed, lower), bound(18, open, upper))),          % 18 =< X <  18 = ∅
    range_empty(range(bound(18, open, lower),   bound(18, closed, upper))).        % 18 <  X =< 18 = ∅

% A lower value strictly above the upper value is empty.
test(reversed_empty) :-
    range_empty(range(bound(20, closed, lower), bound(18, open, upper))),          % 20 =< X <  18 = ∅
    \+ bounds_satisfiable([bound(20, closed, lower), bound(18, open, upper)]).

% Single-sided and unconstrained ranges are never empty (unbounded on the missing side).
test(single_sided_nonempty) :-
    \+ range_empty(range(bound(18, closed, lower), none)),
    \+ range_empty(range(none, bound(18, open, upper))),
    \+ range_empty(range(none, none)).

% ---- §L·thread control (the standing conformance thread) ------------------------------------
% Adult `age >= 18` (docA/docB) vs child `age < 18` (control) is age-DISJOINT → no overlap → the
% control's documented no-conflict. docA vs docB share the adult half-line → age overlaps, so conflict
% stays eligible on the age dimension.
test(thread_age_disjoint) :-
    Adult = range(bound(18, closed, lower), none),   % age >= 18
    Child = range(none, bound(18, open, upper)),     % age <  18
    \+ ranges_overlap(Adult, Child),
    range_intersection(Adult, Child, R),
    assertion(range_empty(R)),
    assertion(R == range(bound(18, closed, lower), bound(18, open, upper))).

test(thread_age_overlap) :-
    Adult = range(bound(18, closed, lower), none),
    ranges_overlap(Adult, Adult),                    % both adult — overlaps
    range_intersection(Adult, Adult, R),
    assertion(R == range(bound(18, closed, lower), none)).

% ---- range_intersection composition ---------------------------------------------------------

% Intersecting two guards' ranges == folding the union of their raw bounds (bounds_range) — the two
% entry points agree; the tighter upper (19) survives over the looser (65).
test(intersection_equals_union_fold) :-
    B1 = [bound(18, closed, lower)],
    B2 = [bound(19, open, upper), bound(65, open, upper)],
    bounds_range(B1, R1), bounds_range(B2, R2),
    range_intersection(R1, R2, RI),
    append(B1, B2, All), bounds_range(All, RU),
    assertion(RI == RU),
    assertion(RI == range(bound(18, closed, lower), bound(19, open, upper))).

% ---- rational exactness + bounded-range guard -----------------------------------------------

% Exact rational arithmetic: a half-open point at a rational endpoint, and a strictly-between case.
test(rational_exact) :-
    range_empty(range(bound(1r2, open, lower), bound(1r2, open, upper))),          % 1/2 < X < 1/2 = ∅
    bounds_satisfiable([bound(1r2, closed, lower), bound(1r2, closed, upper)]),     % {1/2}
    bounds_satisfiable([bound(1r3, open, lower), bound(1r2, open, upper)]).         % 1/3 < X < 1/2 (2r5)

% A bounded age range guard (v1 per profile-structure): its own interval atoms intersect to an
% effective range; a self-contradictory guard is unsatisfiable (conflict-core reads it as no-conflict).
test(bounded_range_guard) :-
    bounds_range([bound(18, closed, lower), bound(65, open, upper)], R),
    assertion(R == range(bound(18, closed, lower), bound(65, open, upper))),
    bounds_satisfiable([bound(18, closed, lower), bound(65, open, upper)]),         % 18 =< X < 65
    \+ bounds_satisfiable([bound(65, closed, lower), bound(18, open, upper)]).       % 65 =< X < 18 = ∅

:- end_tests(intervals).
