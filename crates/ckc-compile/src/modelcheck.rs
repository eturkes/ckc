//! CKC → model-checker meta-spec emitters (SPEC 13.4 workflow/temporal, 14).
//!
//! Phase-0 task 0.8: generated model-checker meta-spec stubs. The TLA+ stub
//! (`emit_tlaplus_stub`) reflects the Event Calculus allergy-persistence conflict
//! as a bounded boolean state machine whose safety invariant the persisted-allergy
//! trace violates. Stub-only — no TLC/Apalache run (the model-check execution is
//! task 0.9). The Alloy stub joins this module in task 0.8.12.

use crate::{
    CompilationMap, CompileBundle, CompiledTarget, SymbolMapping, TargetLanguage, build_target,
    content_hash,
};

/// Emit the generated TLA+ meta-spec stub for `conflict_ec_allergy_persistence`
/// (SPEC 13.4 protocol transition systems; SPEC 15.1 unsafe-persistence conflict).
///
/// The single `EventNarrative`'s boolean fluents (`allergy_known`, `drug_active`)
/// become TLA+ `VARIABLES`. `Init` starts both fluents `FALSE` (matching the
/// narrative's `initially`); the `Next` skeleton lets each fluent rise, so a trace
/// can set `allergy_known` and then `drug_active`. The safety invariant
/// `AllergyContraindication == allergy_known => ~drug_active` is the
/// unsafe-persistence conflict (SPEC 15.1): once the allergy fluent holds, an
/// active beta-lactam administration breaks it. A future TLC/Apalache run would
/// return that violating trace as a counterexample.
///
/// The `VARIABLES` declaration and the `Init` conjunction are data-driven from the
/// fluents, taken in sorted order so their bytes depend only on the fluent set,
/// not on fixture discovery order. The header comment, the `Next` skeleton, the
/// invariant, and the module footer stay in fixed order — the invariant encodes
/// the directional contraindication semantics specific to this conflict, mirroring
/// the fixed violation query of [`crate::asp::emit_event_calculus`]. The
/// TLC/Apalache run is task 0.9.
pub fn emit_tlaplus_stub(bundle: &CompileBundle) -> CompiledTarget {
    let narrative = bundle
        .event_narratives
        .first()
        .expect("toy bundle must contain an event narrative");

    // Fluents in sorted order: the declaration/Init bytes then depend only on the
    // fluent set. The toy set is {allergy_known, drug_active}, already sorted.
    let mut fluents: Vec<&str> = narrative.fluent_types.iter().map(String::as_str).collect();
    fluents.sort_unstable();

    let variables = fluents.join(", ");
    let init_block = fluents
        .iter()
        .map(|f| format!("  /\\ {f} = FALSE"))
        .collect::<Vec<_>>()
        .join("\n");

    let artifact_text = format!(
        "\
---- MODULE Conflict ----
\\* CKC -> TLA+ meta-spec stub: conflict_ec_allergy_persistence (SPEC 13.4, 15.1).
\\* Generated stub reflecting the Event Calculus allergy-persistence conflict as a
\\* bounded boolean state machine over the narrative's fluents: once allergy_known
\\* holds, an active beta-lactam administration (drug_active) violates the
\\* contraindication invariant AllergyContraindication (SPEC 15.1 unsafe persistence).
\\* Stub only: no TLC/Apalache run (task 0.9).

VARIABLES {variables}

Init ==
{init_block}

Next ==
  \\/ /\\ allergy_known' = TRUE
     /\\ UNCHANGED drug_active
  \\/ /\\ drug_active' = TRUE
     /\\ UNCHANGED allergy_known

AllergyContraindication == allergy_known => ~drug_active
====
"
    );

    // Each fluent maps to the TLA+ variable of the same name, grounded in the
    // narrative's shared source spans.
    let compilation_map = CompilationMap(
        fluents
            .iter()
            .map(|&f| SymbolMapping {
                ckc_node_id: f.to_string(),
                target_symbol: f.to_string(),
                source_span_ids: narrative.source_span_ids.clone(),
            })
            .collect(),
    );

    let source_artifact_hashes = vec![content_hash(narrative)];

    build_target(
        TargetLanguage::TlaPlus,
        artifact_text,
        compilation_map,
        source_artifact_hashes,
    )
}
