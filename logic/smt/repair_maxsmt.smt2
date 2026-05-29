; CKC -> SMT-LIB MaxSMT repair: conflict_norm_bl_contradiction (Z3 Optimize)
; Reuses the norm-conflict hard constraints; the recommendation fires from sepsis
; unless a repair toggle withdraws it under the allergy context:
;   repair_add_priority   - rule_bl_anaphylaxis_contra > rule_sepsis_bl_recommend
;   repair_add_exception  - rule_sepsis_bl_recommend excludes beta_lactam_anaphylaxis
; assert-soft penalizes applying a repair (weight 1 each); the minimum-cost model
; applies exactly one, restoring satisfiability. Optimize run is task 0.9.
(set-logic QF_UF)
(declare-const allergy_history_beta_lactam_anaphylaxis Bool)
(declare-const dx_sepsis Bool)
(declare-const prohibit_administer_beta_lactam Bool)
(declare-const recommend_administer_beta_lactam Bool)
(declare-const repair_add_exception Bool)
(declare-const repair_add_priority Bool)
(assert dx_sepsis)
(assert allergy_history_beta_lactam_anaphylaxis)
(assert (=> (and dx_sepsis (not repair_add_priority) (not repair_add_exception)) recommend_administer_beta_lactam))
(assert (=> allergy_history_beta_lactam_anaphylaxis prohibit_administer_beta_lactam))
(assert (not (and recommend_administer_beta_lactam prohibit_administer_beta_lactam)))
(assert-soft (not repair_add_priority) :weight 1)
(assert-soft (not repair_add_exception) :weight 1)
(check-sat)
(get-objectives)
