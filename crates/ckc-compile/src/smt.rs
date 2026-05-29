//! CKC → SMT-LIB emitters (SPEC 13.2, 14).
//!
//! Phase-0 task 0.8: the deontic norm-conflict encoding (`emit_norm_conflict`,
//! QF_UF, unsat witness) and the decision-table overlap/gap analysis
//! (`emit_decision_table`, QF_LRA, sat witness). Emit-only — the Z3/cvc5 solver
//! run, unsat-core extraction, and model recovery belong to task 0.9.

use ckc_core::artifact::DecisionTable;
use ckc_core::clinical::Rule;

use crate::{
    CompilationMap, CompileBundle, CompiledTarget, SymbolMapping, TargetLanguage, build_target,
    content_hash, sorted_lines,
};

/// Borrow a rule by its `rule_id`. Panics when the committed fixture stops
/// carrying it — a build-time bug, mirroring [`CompileBundle::load_toy`].
fn find_rule<'a>(bundle: &'a CompileBundle, rule_id: &str) -> &'a Rule {
    bundle
        .rules
        .iter()
        .find(|r| r.rule_id.as_str() == rule_id)
        .unwrap_or_else(|| panic!("toy bundle must contain rule {rule_id}"))
}

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
/// `(check-sat)` is `sat`: the model is the overlap point, and SAT also
/// certifies the gap point stays uncovered — a covered point would assert
/// `false` and flip the result to `unsat`. Declarations go through
/// [`sorted_lines`]; predicate definitions follow table row order; asserts
/// stay in fixed order.
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
