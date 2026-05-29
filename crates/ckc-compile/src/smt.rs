//! CKC → SMT-LIB emitters (SPEC 13.2, 14).
//!
//! Phase-0 task 0.8: the deontic norm-conflict encoding (`emit_norm_conflict`,
//! QF_UF, unsat witness), the decision-table overlap/gap analysis
//! (`emit_decision_table`, QF_LRA, sat witness), and the MaxSMT minimal-repair
//! search (`emit_repair_maxsmt`, Z3 Optimize). Emit-only — the Z3/cvc5 solver
//! run, unsat-core extraction, model recovery, and optimization belong to
//! task 0.9.

use ckc_core::artifact::DecisionTable;
use ckc_core::verify::Conflict;

use crate::{
    CompilationMap, CompileBundle, CompiledTarget, SymbolMapping, TargetLanguage, build_target,
    content_hash, find_rule, sorted_lines,
};

/// Emit the SMT-LIB deontic norm-conflict encoding for
/// `conflict_norm_bl_contradiction` (SPEC 15.1: recommendation-for vs
/// recommendation-against on one normalized action under a shared satisfiable
/// context).
///
/// `rule_sepsis_bl_recommend` (direction `for`, deontic `recommended`) and
/// `rule_bl_anaphylaxis_contra` (direction `against`, deontic `prohibited`)
/// both fire over the shared context `(dx sepsis) AND (allergy_history
/// beta_lactam anaphylaxis)`. The deontic-exclusion assert makes the
/// recommend/prohibit conjunction impossible, so `(check-sat)` is `unsat`
/// absent a priority rule — the conflict witness. Declarations go through
/// [`sorted_lines`]; asserts stay in fixed clinical order.
pub fn emit_norm_conflict(bundle: &CompileBundle) -> CompiledTarget {
    const RULE_RECOMMEND: &str = "rule_sepsis_bl_recommend";
    const RULE_CONTRA: &str = "rule_bl_anaphylaxis_contra";
    const SYM_RECOMMEND: &str = "recommend_administer_beta_lactam";
    const SYM_PROHIBIT: &str = "prohibit_administer_beta_lactam";
    const ATOM_SEPSIS: &str = "dx_sepsis";
    const ATOM_ALLERGY: &str = "allergy_history_beta_lactam_anaphylaxis";

    const HEADER: &str = "\
; CKC -> SMT-LIB norm conflict: conflict_norm_bl_contradiction
; rule_sepsis_bl_recommend (for/recommended) vs rule_bl_anaphylaxis_contra (against/prohibited)
; shared context: (dx sepsis) AND (allergy_history beta_lactam anaphylaxis)
; check-sat is unsat: the norms cannot coexist absent a priority rule
(set-logic QF_UF)
";

    let rule_recommend = find_rule(bundle, RULE_RECOMMEND);
    let rule_contra = find_rule(bundle, RULE_CONTRA);

    // One Bool per context/deontic atom; sorted so the block's bytes depend
    // only on its contents.
    let declarations = sorted_lines(vec![
        format!("(declare-const {ATOM_SEPSIS} Bool)"),
        format!("(declare-const {ATOM_ALLERGY} Bool)"),
        format!("(declare-const {SYM_RECOMMEND} Bool)"),
        format!("(declare-const {SYM_PROHIBIT} Bool)"),
    ]);

    // Fixed clinical order: shared context holds, each norm fires under its
    // context, then the deontic incompatibility that yields unsat.
    let asserts = [
        format!("(assert {ATOM_SEPSIS})"),
        format!("(assert {ATOM_ALLERGY})"),
        format!("(assert (=> {ATOM_SEPSIS} {SYM_RECOMMEND}))"),
        format!("(assert (=> {ATOM_ALLERGY} {SYM_PROHIBIT}))"),
        format!("(assert (not (and {SYM_RECOMMEND} {SYM_PROHIBIT})))"),
    ]
    .join("\n");

    let artifact_text = format!("{HEADER}{declarations}{asserts}\n(check-sat)\n");

    // Each conflicting rule maps to its deontic symbol, grounded in the rule's
    // own source spans.
    let compilation_map = CompilationMap(vec![
        SymbolMapping {
            ckc_node_id: RULE_RECOMMEND.to_string(),
            target_symbol: SYM_RECOMMEND.to_string(),
            source_span_ids: rule_recommend.source_span_ids.clone(),
        },
        SymbolMapping {
            ckc_node_id: RULE_CONTRA.to_string(),
            target_symbol: SYM_PROHIBIT.to_string(),
            source_span_ids: rule_contra.source_span_ids.clone(),
        },
    ]);

    let source_artifact_hashes = vec![content_hash(rule_recommend), content_hash(rule_contra)];

    build_target(
        TargetLanguage::SmtLib,
        artifact_text,
        compilation_map,
        source_artifact_hashes,
    )
}

/// Borrow the decision table by its `table_id`. Panics when the committed
/// fixture stops carrying it — a build-time bug, mirroring [`find_rule`].
fn find_decision_table<'a>(bundle: &'a CompileBundle, table_id: &str) -> &'a DecisionTable {
    bundle
        .decision_tables
        .iter()
        .find(|t| t.table_id.as_str() == table_id)
        .unwrap_or_else(|| panic!("toy bundle must contain decision table {table_id}"))
}

/// Emit the SMT-LIB decision-table analysis for `dt_vitals_triage`
/// (`hit_policy = unique`; SPEC 15.1 decision-table overlaps and gaps).
///
/// The four rows compile to linear-real-arithmetic match predicates over one
/// vital-sign input point `(temperature, heart_rate, systolic_bp)`:
/// `row_temp_high` (≥ 38.0), `row_temp_very_high` (≥ 38.5), `row_hr_high`
/// (> 90), `row_bp_low` (< 90). Two witnesses share the program:
///
/// * overlap — an input with `temperature ≥ 38.5` fires both `row_temp_high`
///   and `row_temp_very_high`, whose outputs (`administer_antipyretic` vs
///   `initiate_cooling`) differ, so the `unique` policy is violated;
/// * gap — the concrete point `(37.5, 85, 95)` fires no row.
///
/// `(check-sat)` is `sat`: the solver finds a real overlap point (the symbolic
/// vital signs), and the gap assert — a closed formula over the literal point
/// `(37.5, 85, 95)`, decided at emit time — holds because that point is
/// uncovered; a covered point would reduce it to `false` and flip the result to
/// `unsat`. Declarations go through [`sorted_lines`]; predicate definitions
/// follow table row order; asserts stay in fixed order.
pub fn emit_decision_table(bundle: &CompileBundle) -> CompiledTarget {
    const TABLE_ID: &str = "dt_vitals_triage";

    const HEADER: &str = "\
; CKC -> SMT-LIB decision table: dt_vitals_triage (hit_policy unique)
; Row-match predicates over one vital-sign input point.
; Overlap witness: temperature >= 38.5 fires row_temp_high AND row_temp_very_high
;   (outputs administer_antipyretic vs initiate_cooling differ) -> unique-policy violation.
; Gap witness: (temperature=37.5, heart_rate=85, systolic_bp=95) fires no row.
; check-sat is sat: the model is the overlap point; the gap point stays uncovered.
(set-logic QF_LRA)
";

    let table = find_decision_table(bundle, TABLE_ID);

    // One Real per input field (the symbolic overlap point); sorted so the
    // block's bytes depend only on its contents.
    let declarations = sorted_lines(vec![
        "(declare-const temperature Real)".to_string(),
        "(declare-const heart_rate Real)".to_string(),
        "(declare-const systolic_bp Real)".to_string(),
    ]);

    // Row-match predicates in table row (document) order.
    let predicates = [
        "(define-fun row_temp_high ((t Real)) Bool (>= t 38.0))",
        "(define-fun row_temp_very_high ((t Real)) Bool (>= t 38.5))",
        "(define-fun row_hr_high ((hr Real)) Bool (> hr 90.0))",
        "(define-fun row_bp_low ((bp Real)) Bool (< bp 90.0))",
    ]
    .join("\n");

    // Fixed order: overlap witness (exactly the two temperature rows fire),
    // then the concrete gap point that matches no row.
    let asserts = [
        "(assert (and (row_temp_high temperature) (row_temp_very_high temperature) (not (row_hr_high heart_rate)) (not (row_bp_low systolic_bp))))",
        "(assert (not (or (row_temp_high 37.5) (row_temp_very_high 37.5) (row_hr_high 85.0) (row_bp_low 95.0))))",
    ]
    .join("\n");

    let artifact_text = format!("{HEADER}{declarations}{predicates}\n{asserts}\n(check-sat)\n");

    // Each row maps to its match-predicate symbol (the row_id reused verbatim),
    // grounded in that row's own table-cell source spans.
    let compilation_map = CompilationMap(
        table
            .rows
            .iter()
            .map(|row| SymbolMapping {
                ckc_node_id: row.row_id.as_str().to_string(),
                target_symbol: row.row_id.as_str().to_string(),
                source_span_ids: row.source_span_ids.clone(),
            })
            .collect(),
    );

    let source_artifact_hashes = vec![content_hash(table)];

    build_target(
        TargetLanguage::SmtLib,
        artifact_text,
        compilation_map,
        source_artifact_hashes,
    )
}

/// Borrow the conflict by its `conflict_id`. Panics when the committed fixture
/// stops carrying it — a build-time bug, mirroring [`find_rule`].
fn find_conflict<'a>(bundle: &'a CompileBundle, conflict_id: &str) -> &'a Conflict {
    bundle
        .conflicts
        .iter()
        .find(|c| c.conflict_id.as_str() == conflict_id)
        .unwrap_or_else(|| panic!("toy bundle must contain conflict {conflict_id}"))
}

/// Emit the SMT-LIB MaxSMT minimal-repair search for
/// `conflict_norm_bl_contradiction` (SPEC 13.2 MaxSMT repair; Z3 Optimize
/// dialect).
///
/// The encoding reuses the [`emit_norm_conflict`] hard constraints, then gates
/// the recommendation on the conflict's two `repair_candidates` (read from the
/// `Conflict`, in fixture order). The recommendation fires from sepsis *unless*
/// a repair is applied; either repair withdraws it under the allergy context:
///
/// * `repair_add_priority` — `rule_bl_anaphylaxis_contra` ≻
///   `rule_sepsis_bl_recommend`, so the contraindication defeats the
///   recommendation;
/// * `repair_add_exception` — `rule_sepsis_bl_recommend` excludes
///   `beta_lactam_anaphylaxis` from its scope.
///
/// The contraindication and the deontic exclusion stay absolute, so while the
/// allergy context holds the recommendation must be false — which needs at
/// least one repair applied. Each repair carries an `(assert-soft (not …)
/// :weight 1)` penalty for being applied, so the minimum-cost model applies
/// exactly one. `(check-sat)` then `(get-objectives)` report the optimum.
/// Declarations go through [`sorted_lines`]; hard asserts then soft asserts
/// stay in fixed order. The Z3 Optimize run / model recovery is task 0.9.
pub fn emit_repair_maxsmt(bundle: &CompileBundle) -> CompiledTarget {
    const CONFLICT_ID: &str = "conflict_norm_bl_contradiction";
    const SYM_RECOMMEND: &str = "recommend_administer_beta_lactam";
    const SYM_PROHIBIT: &str = "prohibit_administer_beta_lactam";
    const ATOM_SEPSIS: &str = "dx_sepsis";
    const ATOM_ALLERGY: &str = "allergy_history_beta_lactam_anaphylaxis";

    const HEADER: &str = "\
; CKC -> SMT-LIB MaxSMT repair: conflict_norm_bl_contradiction (Z3 Optimize)
; Reuses the norm-conflict hard constraints; the recommendation fires from sepsis
; unless a repair toggle withdraws it under the allergy context:
;   repair_add_priority   - rule_bl_anaphylaxis_contra > rule_sepsis_bl_recommend
;   repair_add_exception  - rule_sepsis_bl_recommend excludes beta_lactam_anaphylaxis
; assert-soft penalizes applying a repair (weight 1 each); the minimum-cost model
; applies exactly one, restoring satisfiability. Optimize run is task 0.9.
(set-logic QF_UF)
";

    let conflict = find_conflict(bundle, CONFLICT_ID);

    // Repair toggles, in fixture (document) order: each candidate's `type`
    // becomes the soft-constraint symbol `repair_<type>`.
    let repair_types: Vec<String> = conflict
        .repair_candidates
        .iter()
        .map(|rc| {
            rc.get("type")
                .and_then(|v| v.as_str())
                .expect("repair candidate must carry a string `type`")
                .to_string()
        })
        .collect();
    let repair_symbols: Vec<String> = repair_types.iter().map(|t| format!("repair_{t}")).collect();

    // One Bool per context/deontic atom plus one per repair toggle; sorted so
    // the block's bytes depend only on its contents.
    let mut decl_lines = vec![
        format!("(declare-const {ATOM_SEPSIS} Bool)"),
        format!("(declare-const {ATOM_ALLERGY} Bool)"),
        format!("(declare-const {SYM_RECOMMEND} Bool)"),
        format!("(declare-const {SYM_PROHIBIT} Bool)"),
    ];
    decl_lines.extend(
        repair_symbols
            .iter()
            .map(|s| format!("(declare-const {s} Bool)")),
    );
    let declarations = sorted_lines(decl_lines);

    // "no repair applied" guard, e.g. `(not repair_add_priority) (not
    // repair_add_exception)`, in fixture order.
    let no_repair = repair_symbols
        .iter()
        .map(|s| format!("(not {s})"))
        .collect::<Vec<_>>()
        .join(" ");

    // Hard constraints, fixed clinical order: context holds; the recommendation
    // fires from sepsis unless a repair is applied; the contraindication fires
    // from allergy; the deontic exclusion is absolute.
    let hard = [
        format!("(assert {ATOM_SEPSIS})"),
        format!("(assert {ATOM_ALLERGY})"),
        format!("(assert (=> (and {ATOM_SEPSIS} {no_repair}) {SYM_RECOMMEND}))"),
        format!("(assert (=> {ATOM_ALLERGY} {SYM_PROHIBIT}))"),
        format!("(assert (not (and {SYM_RECOMMEND} {SYM_PROHIBIT})))"),
    ]
    .join("\n");

    // Soft constraints in fixture order: each applied repair costs 1, so the
    // minimum-cost model applies exactly one.
    let soft = repair_symbols
        .iter()
        .map(|s| format!("(assert-soft (not {s}) :weight 1)"))
        .collect::<Vec<_>>()
        .join("\n");

    let artifact_text =
        format!("{HEADER}{declarations}{hard}\n{soft}\n(check-sat)\n(get-objectives)\n");

    // Each repair candidate maps the owning conflict node to its soft-constraint
    // symbol, grounded in the conflict's own source spans. A candidate `type` is
    // a repair kind, not a CKC node id, so the resolvable node is the conflict
    // that carries the repair_candidates; the symbol (repair_<type>) records
    // which repair.
    let compilation_map = CompilationMap(
        repair_symbols
            .iter()
            .map(|sym| SymbolMapping {
                ckc_node_id: CONFLICT_ID.to_string(),
                target_symbol: sym.clone(),
                source_span_ids: conflict.source_spans.clone(),
            })
            .collect(),
    );

    let source_artifact_hashes = vec![content_hash(conflict)];

    build_target(
        TargetLanguage::SmtLib,
        artifact_text,
        compilation_map,
        source_artifact_hashes,
    )
}
