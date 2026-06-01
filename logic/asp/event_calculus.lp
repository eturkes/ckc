% CKC -> ASP/Clingo Event Calculus: conflict_ec_allergy_persistence
% Allergy detected at t=0, beta-lactam administered at t=10.
% Discrete Event Calculus persistence/clipping: holdsAt derives from an initiating
% event when no terminating event clips the fluent in the open interval.
% allergy_known is initiated at t=0 and the terminates set is empty, so holdsAt(allergy_known,10)
% derives; the violation fires as administer_drug at t=10 meets the persisted allergy.
% Emit-only: the clingo run is task 0.9.
happens(administer_drug,10).
happens(detect_allergy,0).
initiates(administer_drug,drug_active,10).
initiates(detect_allergy,allergy_known,0).
#defined terminates/3.
time(T) :- happens(_,T).
holdsAt(F,T2) :- happens(E,T1), initiates(E,F,T1), time(T2), T1 < T2, not stoppedIn(T1,F,T2).
stoppedIn(T1,F,T2) :- happens(E,T), terminates(E,F,T), time(T1), time(T2), T1 < T, T < T2.
violation(conflict_ec_allergy_persistence) :- happens(administer_drug,10), holdsAt(allergy_known,10).
#show holdsAt/2.
#show violation/1.
