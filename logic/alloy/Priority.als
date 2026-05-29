// CKC -> Alloy meta-spec stub: conflict_norm_bl_contradiction priority graph (SPEC 14, 15.1).
// Each Rule atom carries priority_over superiority edges; the sole toy edge is
//   rule_bl_anaphylaxis_contra -> rule_sepsis_bl_recommend.
// check NoPriorityCycle searches for a rule reachable from itself through the
// transitive closure ^priority_over. The toy superiority is a single edge, so no
// counterexample exists -> the priority graph is acyclic (SPEC 15.1
// cyclic/contradictory priority relations). Stub only: no Alloy Analyzer run (task 0.9).

abstract sig Rule {
  priority_over: set Rule
}

one sig rule_bl_anaphylaxis_contra extends Rule {}
one sig rule_sepsis_bl_recommend extends Rule {}

fact Superiority {
  priority_over = rule_bl_anaphylaxis_contra -> rule_sepsis_bl_recommend
}

check NoPriorityCycle { no r: Rule | r in r.^priority_over }
