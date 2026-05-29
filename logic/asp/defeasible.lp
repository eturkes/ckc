% CKC -> ASP/Clingo defeasible + argumentation: conflict_norm_bl_contradiction
% Two defeasible norm rules and their superiority, as a Dung argumentation view:
%   rule_bl_anaphylaxis_contra > rule_sepsis_bl_recommend
% defeasible layer: applicable/1 + superior/2 facts, defeated/1 derivation.
% argumentation layer: argument/1 + attacks/2, grounded extension in/1 + defeated_arg/1.
% The derived attack graph is acyclic -> locally stratified -> unique stable model.
% Emit-only: the clingo run is task 0.9.
applicable(rule_bl_anaphylaxis_contra).
applicable(rule_sepsis_bl_recommend).
argument(arg_rule_bl_anaphylaxis_contra).
argument(arg_rule_sepsis_bl_recommend).
attacks(arg_rule_bl_anaphylaxis_contra,arg_rule_sepsis_bl_recommend).
superior(rule_bl_anaphylaxis_contra,rule_sepsis_bl_recommend).
attacked_by_undefeated(A) :- attacks(B,A), not defeated_arg(B).
defeated(R) :- superior(S,R), applicable(S), applicable(R).
defeated_arg(A) :- attacks(B,A), in(B).
in(A) :- argument(A), not attacked_by_undefeated(A).
#show in/1.
#show defeated/1.
#show defeated_arg/1.
