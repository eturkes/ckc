---- MODULE Conflict ----
\* CKC -> TLA+ meta-spec stub: conflict_ec_allergy_persistence (SPEC 13.4, 15.1).
\* Generated stub reflecting the Event Calculus allergy-persistence conflict as a
\* bounded boolean state machine over the narrative's fluents: once allergy_known
\* holds, an active beta-lactam administration (drug_active) violates the
\* contraindication invariant AllergyContraindication (SPEC 15.1 unsafe persistence).
\* Stub only: no TLC/Apalache run (task 0.9).

VARIABLES allergy_known, drug_active

Init ==
  /\ allergy_known = FALSE
  /\ drug_active = FALSE

Next ==
  \/ /\ allergy_known' = TRUE
     /\ UNCHANGED drug_active
  \/ /\ drug_active' = TRUE
     /\ UNCHANGED allergy_known

AllergyContraindication == allergy_known => ~drug_active
====
