% ClinicalCNL interval algebra (M3.interval-algebra; SPEC §6 conflict, D10, clinical/KB.md §Context
% atoms). The exact-rational bound algebra the conflict layer's context-overlap check consumes — the
% SYMBOLIC counterpart to kb_kernel's holds_atom/2 (which decides ONE patient value against a bound).
% Here nothing is patient-evaluated: the algebra intersects bounds and decides whether the resulting
% range holds ANY point, over Q (the rationals). So the open-vs-closed and dense-order distinctions
% carry — `18 < X < 19` is EMPTY over the integers but NON-empty over Q (e.g. 37r2), so an integer/FD
% domain is UNSOUND for conflict overlap (D10). No clp(FD)/clp(Q) dependency: plain arithmetic over
% SWI native rationals, which stay exact under +, -, comparison.
%
% A bound(Value, Openness, Dir) is one half-line constraint on a quantity:
%   Openness ∈ {open, closed}   Dir ∈ {lower, upper}   Value an exact integer or `NrD` rational
%   (closed, lower) = X >= V     (open, lower) = X > V     (closed, upper) = X =< V     (open, upper) = X < V
% These are exactly the four v1 CountOp markers normalized (KB.md; geq at least / greater more than /
% leq at most / less less than) — this module is CountOp-agnostic and reads the normalized
% (Openness, Dir) the KB already stores. A v1 interval ATOM is a SINGLE bound (the single-bound law;
% valid_v1_interval/1, the 16-mask validity transplant). A guard may carry several same-quantity
% interval atoms (a bounded age range — v1 per profile-structure), whose bounds intersect to an
% effective range(Lower, Upper): Lower/Upper each `none` (unbounded) or a bound of that Dir.
% range_empty/1 decides satisfiability over Q; conflict-core intersects two guards' ranges and rejects
% an empty result (the §L·thread control: adult `age>=18` vs child `age<18` is age-disjoint → no
% conflict). Leaf module — no sibling dependency; the interval/4 atom shape is destructured, not read.
%
%   Gate: swipl -q -g "consult('clinical/intervals_tests.pl'),(run_tests(intervals)->halt(0);halt(1))" -t 'halt(1)'

:- module(intervals,
          [ valid_bound/1,          % ?bound(V,Openness,Dir)                  — a well-formed exact bound
            valid_v1_interval/1,    % +Bounds                                 — the single-bound law (16-mask)
            interval_bound/3,       % +interval(Q,V,O,D), -Q, -bound(V,O,D)   — KB context atom -> bound
            bounds_range/2,         % +Bounds, -range(Lower,Upper)            — intersect same-quantity bounds
            range_intersection/3,   % +R1, +R2, -R3                           — intersect two ranges
            range_empty/1,          % +range(Lower,Upper)                     — empty over Q (semidet)
            bounds_satisfiable/1,   % +Bounds                                 — bounds share a rational point
            ranges_overlap/2        % +R1, +R2                                — two ranges share a rational point
          ]).

% ---- bounds -----------------------------------------------------------------------------------

%% valid_bound(?Bound) is semidet.
% A well-formed exact bound: an integer or n/d rational value (never a float — D10 needs exact
% open/closed arithmetic), a closed-vocabulary Openness and Dir.
valid_bound(bound(V, Openness, Dir)) :-
    rational(V),
    openness(Openness),
    dir(Dir).

openness(open).
openness(closed).

dir(lower).
dir(upper).

%% interval_bound(+Atom, -Quantity, -Bound) is det.
% Split a KB interval/4 context atom (KB.md §Context atoms) into its quantity and its bound. The
% caller groups bounds by Quantity before intersecting — only same-quantity bounds intersect.
interval_bound(interval(Q, V, Openness, Dir), Q, bound(V, Openness, Dir)).

%% valid_v1_interval(+Bounds) is semidet.
% The v1 single-bound law — the interval validity battery transplanted from the CNL AST/parser side:
% a list of bounds is a legal v1 interval atom iff EXACTLY ONE bound is present and its value is >= 0
% (an age is non-negative). Over the four (Openness,Dir) slots {geq,greater,leq,less} this rejects the
% empty mask, every two-or-more-bound mask (a bare range is not a single atom), a malformed slot, and a
% negative value. The raw/profile lanes already enforce this upstream; the algebra re-checks its input.
valid_v1_interval([Bound]) :-
    valid_bound(Bound),
    Bound = bound(V, _, _),
    V >= 0.

% ---- ranges -----------------------------------------------------------------------------------
% A range(Lower, Upper) is the intersection of a set of same-quantity bounds: Lower is the tightest
% lower bound (or `none`), Upper the tightest upper bound (or `none`). An absent side is unbounded
% (-inf / +inf) and never causes emptiness on its own — only a two-sided range can be empty.

%% bounds_range(+Bounds, -Range) is det.
% Intersect a list of same-quantity bounds into range(Lower, Upper): the tightest lower and tightest
% upper among them. The empty list yields range(none, none) — an unconstrained (all-Q) quantity.
bounds_range(Bounds, Range) :-
    range_fold(Bounds, range(none, none), Range).

range_fold([], Range, Range).
range_fold([B|Bs], Acc0, Acc) :-
    tighten(B, Acc0, Acc1),
    range_fold(Bs, Acc1, Acc).

% tighten(+Bound, +RangeIn, -RangeOut): fold one bound into the accumulator range, combining it with
% the like-Dir side (the other side passes through unchanged).
tighten(bound(V, O, Dir), range(L0, U0), range(L, U)) :-
    ( Dir == lower
    -> combine_lower(bound(V, O, lower), L0, L), U = U0
    ;  combine_upper(bound(V, O, upper), U0, U), L = L0
    ).

%% range_intersection(+R1, +R2, -R3) is det.
% Intersect two ranges: the tighter of the two lowers and the tighter of the two uppers. Overlaps two
% guards' effective ranges (conflict-core) without re-deriving from raw bounds; equivalent to folding
% the union of the two guards' bounds.
range_intersection(range(L1, U1), range(L2, U2), range(L, U)) :-
    combine_lower(L1, L2, L),
    combine_upper(U1, U2, U).

% combine_lower(+A, +B, -Tighter) / combine_upper(+A, +B, -Tighter): the tighter of two same-Dir sides,
% each `none` (absent = unbounded) or a bound. `none` is always the looser side. Deterministic.
combine_lower(L1, L2, L) :-
    ( L1 == none -> L = L2
    ; L2 == none -> L = L1
    ; tighter_lower(L1, L2, L)
    ).

combine_upper(U1, U2, U) :-
    ( U1 == none -> U = U2
    ; U2 == none -> U = U1
    ; tighter_upper(U1, U2, U)
    ).

% tighter_lower(+A, +B, -T): the tighter of two lower bounds — the greater value excludes more; at an
% equal value the open bound (strict `>`) excludes the shared endpoint, so it beats the closed one.
tighter_lower(bound(Va, Oa, lower), bound(Vb, Ob, lower), Tighter) :-
    ( Va > Vb                   -> Tighter = bound(Va, Oa, lower)
    ; Vb > Va                   -> Tighter = bound(Vb, Ob, lower)
    ; stricter_openness(Oa, Ob) -> Tighter = bound(Va, Oa, lower)
    ;                              Tighter = bound(Vb, Ob, lower)
    ).

% tighter_upper(+A, +B, -T): the tighter of two upper bounds — the lesser value excludes more; at an
% equal value the open bound (strict `<`) beats the closed one.
tighter_upper(bound(Va, Oa, upper), bound(Vb, Ob, upper), Tighter) :-
    ( Va < Vb                   -> Tighter = bound(Va, Oa, upper)
    ; Vb < Va                   -> Tighter = bound(Vb, Ob, upper)
    ; stricter_openness(Oa, Ob) -> Tighter = bound(Va, Oa, upper)
    ;                              Tighter = bound(Vb, Ob, upper)
    ).

% stricter_openness(A, B): A strictly tighter than B at an equal value — open excludes the endpoint,
% closed includes it, so open is stricter than closed (and nothing is stricter than open).
stricter_openness(open, closed).

%% range_empty(+Range) is semidet.
% The range holds NO rational point. Only a two-sided range can be empty (an absent side is unbounded,
% so a single-sided or empty range always holds a point). Over Q (dense): a lower value ABOVE the upper
% value is empty; EQUAL values are empty unless BOTH bounds are closed (only then the shared point V is
% included); strictly between (Vl < Vu) always holds a point, whatever the openness.
range_empty(range(bound(Vl, Ol, lower), bound(Vu, Ou, upper))) :-
    ( Vl > Vu       -> true
    ; Vl =:= Vu     -> ( Ol == open -> true ; Ou == open )
    ;                  fail
    ).

%% bounds_satisfiable(+Bounds) is semidet.
% The same-quantity bounds have a common rational solution (their intersection range is non-empty).
bounds_satisfiable(Bounds) :-
    bounds_range(Bounds, Range),
    \+ range_empty(Range).

%% ranges_overlap(+R1, +R2) is semidet.
% Two ranges share a rational point — conflict-core's cross-guard age-overlap test.
ranges_overlap(R1, R2) :-
    range_intersection(R1, R2, R),
    \+ range_empty(R).
