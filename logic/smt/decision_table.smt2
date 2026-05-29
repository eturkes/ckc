; CKC -> SMT-LIB decision table: dt_vitals_triage (hit_policy unique)
; Row-match predicates over one vital-sign input point.
; Overlap witness: temperature >= 38.5 fires row_temp_high AND row_temp_very_high
;   (outputs administer_antipyretic vs initiate_cooling differ) -> unique-policy violation.
; Gap witness: (temperature=37.5, heart_rate=85, systolic_bp=95) fires no row.
; check-sat is sat: the model is the overlap point; the gap point stays uncovered.
(set-logic QF_LRA)
(declare-const heart_rate Real)
(declare-const systolic_bp Real)
(declare-const temperature Real)
(define-fun row_temp_high ((t Real)) Bool (>= t 38.0))
(define-fun row_temp_very_high ((t Real)) Bool (>= t 38.5))
(define-fun row_hr_high ((hr Real)) Bool (> hr 90.0))
(define-fun row_bp_low ((bp Real)) Bool (< bp 90.0))
(assert (and (row_temp_high temperature) (row_temp_very_high temperature) (not (row_hr_high heart_rate)) (not (row_bp_low systolic_bp))))
(assert (not (or (row_temp_high 37.5) (row_temp_very_high 37.5) (row_hr_high 85.0) (row_bp_low 95.0))))
(check-sat)
