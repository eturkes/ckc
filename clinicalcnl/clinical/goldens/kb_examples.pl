% Hand-written NORMATIVE KB examples (M3.kb-contract). NOT captured — authored to exercise every
% kb_kernel validator path. The valid examples are KERNEL-VALID fixtures: the §8.6 thread docs
% (docA/docB/control) plus one multi-disjunct synthetic, carrying DELIBERATELY VARYING provenance
% density (docA sources its statement + exception too, the others only their rule(s)) — map-emit
% provenance COMPLETENESS is a downstream golden's concern, not a kernel invariant. kb-writer
% byte-pins the valid set (it reads `kb_example(_, valid, Facts)`). Each invalid example isolates
% ONE defect — kb_errors returns exactly the single violation whose FUNCTOR the reject test expects.
%
% kb_example(Name, Validity, Facts): Validity = valid | invalid(ErrorFunctor); Facts = a KB (a
% list of ground rules-as-data fact terms, per clinical/KB.md).

:- module(kb_examples, [kb_example/3]).

% ==========================================================================================
% Valid — the §8.6 worked thread (docA × docB, + control). Ids doc-qualified + document-continuous.
% ==========================================================================================

% docA: 「成人(18歳以上)の敗血症患者には抗菌薬Aを投与することを推奨する。ただし重度腎機能障害の
% ある患者を除く」 — for/strong over sepsis ∧ age>=18, renal-impairment carve-out as exc.0 (the LP
% lane keeps the exception as a NAF guard; it is NOT an in-context negated conjunct).
kb_example(doc_a, valid,
  [ rule('test_source.m1_guideline_a.rule.0', 'test_source.m1_guideline_a.stmt.0'),
    direction('test_source.m1_guideline_a.rule.0', for),
    strength('test_source.m1_guideline_a.rule.0', strong),
    population('test_source.m1_guideline_a.stmt.0', 'pop.patient'),
    condition('test_source.m1_guideline_a.bind.0', 'test_source.m1_guideline_a.stmt.0', concept('cond.sepsis')),
    condition('test_source.m1_guideline_a.bind.1', 'test_source.m1_guideline_a.stmt.0', interval('q.age_years', 18, closed, lower)),
    action('test_source.m1_guideline_a.stmt.0', 'act.administer:drug.abx_a'),
    exception('test_source.m1_guideline_a.exc.0', 'test_source.m1_guideline_a.stmt.0', concept('cond.renal_severe')),
    source('test_source.m1_guideline_a.rule.0', 'test_source.m1_guideline_a', [0], "guideline A sepsis recommendation"),
    source('test_source.m1_guideline_a.stmt.0', 'test_source.m1_guideline_a', [0], none),
    source('test_source.m1_guideline_a.exc.0', 'test_source.m1_guideline_a', [1], "renal-impairment carve-out")
  ]).

% docB: 「成人の敗血症患者のうち、妊娠中の患者には抗菌薬Aを投与しないこと(禁忌)」 —
% contraindicate/strong over sepsis ∧ age>=18 ∧ pregnancy; same action key -> pair eligible w/ docA.
kb_example(doc_b, valid,
  [ rule('test_source.m1_guideline_b.rule.0', 'test_source.m1_guideline_b.stmt.0'),
    direction('test_source.m1_guideline_b.rule.0', contraindicate),
    strength('test_source.m1_guideline_b.rule.0', strong),
    population('test_source.m1_guideline_b.stmt.0', 'pop.patient'),
    condition('test_source.m1_guideline_b.bind.0', 'test_source.m1_guideline_b.stmt.0', concept('cond.sepsis')),
    condition('test_source.m1_guideline_b.bind.1', 'test_source.m1_guideline_b.stmt.0', interval('q.age_years', 18, closed, lower)),
    condition('test_source.m1_guideline_b.bind.2', 'test_source.m1_guideline_b.stmt.0', concept('cond.pregnancy')),
    action('test_source.m1_guideline_b.stmt.0', 'act.administer:drug.abx_a'),
    source('test_source.m1_guideline_b.rule.0', 'test_source.m1_guideline_b', [0], "guideline B pregnancy contraindication")
  ]).

% control: 「小児(18歳未満)の敗血症患者には抗菌薬Aは禁忌である」 — contraindicate/strong over
% sepsis ∧ age<18; no-conflict w/ docA (disjoint age intervals: age>=18 vs age<18).
kb_example(control, valid,
  [ rule('test_source.m1_control.rule.0', 'test_source.m1_control.stmt.0'),
    direction('test_source.m1_control.rule.0', contraindicate),
    strength('test_source.m1_control.rule.0', strong),
    population('test_source.m1_control.stmt.0', 'pop.patient'),
    condition('test_source.m1_control.bind.0', 'test_source.m1_control.stmt.0', concept('cond.sepsis')),
    condition('test_source.m1_control.bind.1', 'test_source.m1_control.stmt.0', interval('q.age_years', 18, open, upper)),
    action('test_source.m1_control.stmt.0', 'act.administer:drug.abx_a'),
    source('test_source.m1_control.rule.0', 'test_source.m1_control', [0], "control pediatric contraindication")
  ]).

% Synthetic multi-disjunct: rule.0 = a 2-disjunct rule (stmt.0 ∨ stmt.1) with certainty + two
% exceptions on stmt.0; rule.1 = a trailing 1-disjunct rule (stmt.2). Exercises document-continuous
% stmt/bind/exc counters across rules (NO per-rule reset), the optional certainty field, the two
% interval markers the thread omits (open-lower `>18`, closed-upper `<=18`), and permit direction.
kb_example(multi, valid,
  [ rule('test_source.kb_multi.rule.0', 'test_source.kb_multi.stmt.0'),
    rule('test_source.kb_multi.rule.0', 'test_source.kb_multi.stmt.1'),
    direction('test_source.kb_multi.rule.0', for),
    strength('test_source.kb_multi.rule.0', weak),
    certainty('test_source.kb_multi.rule.0', moderate),
    population('test_source.kb_multi.stmt.0', 'pop.patient'),
    condition('test_source.kb_multi.bind.0', 'test_source.kb_multi.stmt.0', concept('cond.sepsis')),
    condition('test_source.kb_multi.bind.1', 'test_source.kb_multi.stmt.0', interval('q.age_years', 18, open, lower)),
    action('test_source.kb_multi.stmt.0', 'act.administer:drug.abx_a'),
    exception('test_source.kb_multi.exc.0', 'test_source.kb_multi.stmt.0', concept('cond.renal_severe')),
    exception('test_source.kb_multi.exc.1', 'test_source.kb_multi.stmt.0', concept('cond.pregnancy')),
    population('test_source.kb_multi.stmt.1', 'pop.patient'),
    condition('test_source.kb_multi.bind.2', 'test_source.kb_multi.stmt.1', concept('cond.pregnancy')),
    condition('test_source.kb_multi.bind.3', 'test_source.kb_multi.stmt.1', interval('q.age_years', 18, closed, upper)),
    action('test_source.kb_multi.stmt.1', 'act.administer:drug.abx_a'),
    exception('test_source.kb_multi.exc.2', 'test_source.kb_multi.stmt.1', concept('cond.renal_severe')),
    rule('test_source.kb_multi.rule.1', 'test_source.kb_multi.stmt.2'),
    direction('test_source.kb_multi.rule.1', permit),
    strength('test_source.kb_multi.rule.1', weak),
    population('test_source.kb_multi.stmt.2', 'pop.patient'),
    condition('test_source.kb_multi.bind.4', 'test_source.kb_multi.stmt.2', concept('cond.sepsis')),
    action('test_source.kb_multi.stmt.2', 'act.administer:drug.abx_a'),
    source('test_source.kb_multi.rule.0', 'test_source.kb_multi', [0, 1], "multi-disjunct rule"),
    source('test_source.kb_multi.rule.1', 'test_source.kb_multi', [2], "trailing rule")
  ]).

% ==========================================================================================
% Invalid — each isolates ONE defect over a minimal otherwise-valid base (doc id `d`).
% ==========================================================================================

% ---- context-atom vocabulary + shape --------------------------------------------------------
kb_example(bad_concept, invalid(unknown_concept),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('d.bind.0','d.stmt.0', concept('cond.bogus')) ]).
kb_example(bad_quantity, invalid(unknown_quantity),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('d.bind.0','d.stmt.0', interval('q.bogus', 18, closed, lower)) ]).
kb_example(bad_openness, invalid(bad_interval_openness),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('d.bind.0','d.stmt.0', interval('q.age_years', 18, ajar, lower)) ]).
kb_example(bad_dir, invalid(bad_interval_dir),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('d.bind.0','d.stmt.0', interval('q.age_years', 18, closed, sideways)) ]).
kb_example(bad_bound_float, invalid(bad_interval_bound),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('d.bind.0','d.stmt.0', interval('q.age_years', 1.5, closed, lower)) ]).
kb_example(bad_atom_shape, invalid(malformed_atom),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('d.bind.0','d.stmt.0', frobnitz('cond.sepsis')) ]).
kb_example(bad_exception_atom, invalid(unknown_concept),  % exception payload validated like a condition
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    exception('d.exc.0','d.stmt.0', concept('cond.bogus')) ]).

% ---- action key -----------------------------------------------------------------------------
kb_example(bad_action_nocolon, invalid(malformed_action_key),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer') ]).
kb_example(bad_action_kind, invalid(unknown_action_kind),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.bogus:drug.abx_a') ]).
kb_example(bad_action_target, invalid(unknown_action_target),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.bogus') ]).
kb_example(bad_action_type, invalid(malformed_action_key),    % key a compound, not a `k:t` atom
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0', foo(bar)) ]).

% ---- rule-level vocabulary ------------------------------------------------------------------
kb_example(bad_direction, invalid(unknown_direction),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',sideways), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(bad_strength, invalid(unknown_strength),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',medium),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(bad_certainty, invalid(unknown_certainty),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    certainty('d.rule.0',sometimes),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(bad_population, invalid(unknown_population),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.bogus'), action('d.stmt.0','act.administer:drug.abx_a') ]).

% ---- id grammar -----------------------------------------------------------------------------
kb_example(bad_id, invalid(malformed_id),
  [ rule('d.rule.0','d.stmt.x'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.x','pop.patient'), action('d.stmt.x','act.administer:drug.abx_a'),
    condition('d.bind.0','d.stmt.x', concept('cond.sepsis')) ]).
kb_example(bad_id_empty_doc, invalid(malformed_id),      % empty doc segment (leading dot)
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('.bind.0','d.stmt.0', concept('cond.sepsis')) ]).
kb_example(bad_id_noncanon, invalid(malformed_id),       % non-canonical counter spelling `01`
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('d.bind.01','d.stmt.0', concept('cond.sepsis')) ]).

% ---- cardinality + safety -------------------------------------------------------------------
kb_example(no_population, invalid(missing_population),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(no_action, invalid(missing_action),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient') ]).
kb_example(no_direction, invalid(missing_direction),
  [ rule('d.rule.0','d.stmt.0'), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(dup_action, invalid(duplicate_action),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'),
    action('d.stmt.0','act.administer:drug.abx_a'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(dup_direction, invalid(duplicate_direction),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), direction('d.rule.0',against),
    strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(dup_certainty, invalid(duplicate_certainty),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    certainty('d.rule.0',high), certainty('d.rule.0',low),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(dup_bind, invalid(duplicate_bind),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('d.bind.0','d.stmt.0', concept('cond.sepsis')),
    condition('d.bind.0','d.stmt.0', concept('cond.pregnancy')) ]).
kb_example(dup_population, invalid(duplicate_population),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), population('d.stmt.0','pop.patient'),
    action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(no_strength, invalid(missing_strength),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(dup_strength, invalid(duplicate_strength),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for),
    strength('d.rule.0',strong), strength('d.rule.0',weak),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(dup_exception, invalid(duplicate_exception),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    exception('d.exc.0','d.stmt.0', concept('cond.renal_severe')),
    exception('d.exc.0','d.stmt.0', concept('cond.pregnancy')) ]).
kb_example(dup_rule, invalid(duplicate_rule),            % same rule/2 disjunct-pair repeated
  [ rule('d.rule.0','d.stmt.0'), rule('d.rule.0','d.stmt.0'),
    direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(multi_owned, invalid(multi_owned_stmt),       % one stmt owned by two distinct rules
  [ rule('d.rule.0','d.stmt.0'), rule('d.rule.1','d.stmt.0'),
    direction('d.rule.0',for), strength('d.rule.0',strong),
    direction('d.rule.1',against), strength('d.rule.1',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).

% ---- referential integrity ------------------------------------------------------------------
kb_example(dangling_stmt, invalid(dangling_stmt_ref),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('d.bind.1','d.stmt.5', concept('cond.sepsis')) ]).
kb_example(dangling_rule, invalid(dangling_rule_ref),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    direction('d.rule.9',against),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a') ]).
kb_example(dangling_source, invalid(dangling_source_ref),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    source('d.exc.7','d',[0],none) ]).
kb_example(bad_source, invalid(malformed_source),        % regions not a list
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    source('d.rule.0','d',notalist,none) ]).
kb_example(bad_source_order, invalid(malformed_source),  % regions not strictly ascending
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    source('d.rule.0','d',[1,0],none) ]).
kb_example(bad_source_basis, invalid(malformed_source),  % basis an atom, not a string or `none`
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    source('d.rule.0','d',[0],bogus_atom_basis) ]).

% ---- fact shape -----------------------------------------------------------------------------
kb_example(nonground, invalid(nonground_fact),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    condition('d.bind.0','d.stmt.0', concept(_)) ]).
kb_example(alien_fact, invalid(unknown_fact),
  [ rule('d.rule.0','d.stmt.0'), direction('d.rule.0',for), strength('d.rule.0',strong),
    population('d.stmt.0','pop.patient'), action('d.stmt.0','act.administer:drug.abx_a'),
    frobnicate('d.stmt.0', foo) ]).
% A "KB" that is not a proper list at all (fail-closed; every member/2 check would otherwise no-op).
kb_example(not_a_list, invalid(not_a_list), foo).
