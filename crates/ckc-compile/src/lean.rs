//! CKC → Lean 4 emitter (SPEC 13.1, 14).
//!
//! Phase-0 task 0.8: the deontic norm-conflict theorem
//! (`emit_norm_conflict_theorem`). The two defeasible norm rules project one
//! shared beta-lactam action to two distinct `Deontic` constructors; the emitted
//! theorem proves their conjunction is unsatisfiable absent a priority/exception
//! (SPEC 15.1). Emit-only — the `lake build` kernel check is task 0.9.

use ckc_core::clinical::Rule;
use ckc_core::enums::DeonticProjection;

use crate::{
    CompilationMap, CompileBundle, CompiledTarget, SymbolMapping, TargetLanguage, build_target,
    content_hash, find_rule,
};

/// Read the deontic projection a norm-conflict rule assigns to its action.
/// Panics when the committed fixture rule stops carrying a norm — a build-time
/// bug, mirroring [`crate::find_rule`].
fn norm_projection(rule: &Rule) -> DeonticProjection {
    rule.norm
        .as_ref()
        .unwrap_or_else(|| {
            panic!(
                "norm-conflict rule {} must carry a norm",
                rule.rule_id.as_str()
            )
        })
        .deontic_projection
}

/// Map a norm's deontic projection to the Lean `Deontic` constructor / norm
/// predicate identifier the emitter compiles it to. Panics on a projection
/// outside the toy norm-conflict's `recommended`/`prohibited` pair — a
/// build-time bug, mirroring [`crate::find_rule`].
fn lean_ident(projection: DeonticProjection) -> &'static str {
    match projection {
        DeonticProjection::Recommended => "recommend",
        DeonticProjection::Prohibited => "prohibit",
        other => panic!("toy Lean norm conflict expects recommended/prohibited, got {other:?}"),
    }
}

/// Emit the Lean 4 norm-conflict theorem for `conflict_norm_bl_contradiction`
/// (SPEC 13.1 proof kernel; SPEC 15.1 recommendation-for vs recommendation-against
/// on one normalized action under a shared satisfiable context).
///
/// `rule_sepsis_bl_recommend` (deontic `recommended`) and
/// `rule_bl_anaphylaxis_contra` (deontic `prohibited`) both fire over the shared
/// context. Their deontic projections compile to the two constructors of an
/// inductive `Deontic`; each norm becomes a `Deontic → Prop` predicate fixing the
/// shared action's projection to its constructor. The theorem
/// `unprioritized_norm_conflict` proves no projection satisfies both predicates,
/// because the constructors are distinct — the conflict witness, discharged by a
/// constructive `Deontic.noConfusion` term so the file carries no `sorry`/`admit`.
///
/// The Lean source is order-sensitive (declarations precede uses; the proof
/// structure is fixed), so unlike the fact-block emitters it builds no
/// [`crate::sorted_lines`] block. The constructor/predicate identifiers are
/// data-driven from each rule's `deontic_projection`. The `lake build` kernel
/// check is task 0.9.
pub fn emit_norm_conflict_theorem(bundle: &CompileBundle) -> CompiledTarget {
    const RULE_RECOMMEND: &str = "rule_sepsis_bl_recommend";
    const RULE_CONTRA: &str = "rule_bl_anaphylaxis_contra";

    let rule_recommend = find_rule(bundle, RULE_RECOMMEND);
    let rule_contra = find_rule(bundle, RULE_CONTRA);

    let recommend = lean_ident(norm_projection(rule_recommend));
    let prohibit = lean_ident(norm_projection(rule_contra));

    let artifact_text = format!(
        "\
/-
CKC -> Lean 4 norm-conflict theorem: conflict_norm_bl_contradiction (SPEC 13.1, 15.1)
  rule_sepsis_bl_recommend (for/recommended) vs rule_bl_anaphylaxis_contra (against/prohibited)
  shared satisfiable context: (dx sepsis) AND (allergy_history beta_lactam anaphylaxis)
One shared beta-lactam action cannot project to two distinct Deontic constructors,
so the two norms cannot both hold absent a priority/exception.
Emit-only: the `lake build` kernel check is task 0.9.
-/
namespace Ckc

/-- Deontic projection a norm assigns to a clinical action (SPEC 9 CKC-Norm). -/
inductive Deontic where
  | {recommend}
  | {prohibit}
  deriving DecidableEq, Repr

/-- rule_sepsis_bl_recommend: under the shared context the beta-lactam action is {recommend}ed. -/
def {recommend} (proj : Deontic) : Prop := proj = Deontic.{recommend}

/-- rule_bl_anaphylaxis_contra: under the same shared context the beta-lactam action is {prohibit}ed. -/
def {prohibit} (proj : Deontic) : Prop := proj = Deontic.{prohibit}

/-- Unprioritized norm conflict (SPEC 15.1): no deontic projection of the one shared
beta-lactam action satisfies both norms, since `{recommend}` and `{prohibit}` are
distinct `Deontic` constructors. -/
theorem unprioritized_norm_conflict (proj : Deontic) :
    ¬ ({recommend} proj ∧ {prohibit} proj) := by
  intro h
  have hr : proj = Deontic.{recommend} := h.1
  have hp : proj = Deontic.{prohibit} := h.2
  subst hr
  exact Deontic.noConfusion hp

end Ckc
"
    );

    // Each conflicting rule maps to its Lean norm-predicate identifier, grounded
    // in the rule's own source spans.
    let compilation_map = CompilationMap(vec![
        SymbolMapping {
            ckc_node_id: RULE_RECOMMEND.to_string(),
            target_symbol: recommend.to_string(),
            source_span_ids: rule_recommend.source_span_ids.clone(),
        },
        SymbolMapping {
            ckc_node_id: RULE_CONTRA.to_string(),
            target_symbol: prohibit.to_string(),
            source_span_ids: rule_contra.source_span_ids.clone(),
        },
    ]);

    let source_artifact_hashes = vec![content_hash(rule_recommend), content_hash(rule_contra)];

    build_target(
        TargetLanguage::Lean,
        artifact_text,
        compilation_map,
        source_artifact_hashes,
    )
}
