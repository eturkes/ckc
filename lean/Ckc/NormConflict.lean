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
  | recommend
  | prohibit
  deriving DecidableEq, Repr

/-- rule_sepsis_bl_recommend: under the shared context the beta-lactam action is recommended. -/
def recommend (proj : Deontic) : Prop := proj = Deontic.recommend

/-- rule_bl_anaphylaxis_contra: under the same shared context the beta-lactam action is prohibited. -/
def prohibit (proj : Deontic) : Prop := proj = Deontic.prohibit

/-- Unprioritized norm conflict (SPEC 15.1): no deontic projection of the one shared
beta-lactam action satisfies both norms, since `recommend` and `prohibit` are
distinct `Deontic` constructors. -/
theorem unprioritized_norm_conflict (proj : Deontic) :
    ¬ (recommend proj ∧ prohibit proj) := by
  intro h
  have hr : proj = Deontic.recommend := h.1
  have hp : proj = Deontic.prohibit := h.2
  subst hr
  exact Deontic.noConfusion hp

end Ckc
