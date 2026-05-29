//! CKC → SMT-LIB emitters (SPEC 13.2, 14).
//!
//! Phase-0 task 0.8.4: the deontic norm-conflict encoding. Emit-only — the
//! Z3/cvc5 solver run and unsat-core extraction belong to task 0.9.

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
