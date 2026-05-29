//! Gate for task 0.8.13: `compile_all` runs every Phase-0 emitter in one fixed
//! order, grounds each emitted target, and is byte-deterministic per index
//! across independent runs.

use ckc_compile::{CompileBundle, TargetLanguage, compile_all, content_hash};

#[test]
fn compile_all_emits_nine_targets_in_fixed_order() {
    let targets = compile_all(&CompileBundle::load_toy());
    let langs: Vec<TargetLanguage> = targets.iter().map(|t| t.target_language).collect();
    assert_eq!(
        langs,
        vec![
            TargetLanguage::SmtLib,
            TargetLanguage::SmtLib,
            TargetLanguage::SmtLib,
            TargetLanguage::Asp,
            TargetLanguage::Asp,
            TargetLanguage::Datalog,
            TargetLanguage::Lean,
            TargetLanguage::TlaPlus,
            TargetLanguage::Alloy,
        ],
        "nine targets, in emitter order, with SmtLib and Asp repeating"
    );
}

#[test]
fn compile_all_targets_are_grounded() {
    let targets = compile_all(&CompileBundle::load_toy());
    for t in &targets {
        assert!(
            !t.compilation_map.0.is_empty(),
            "{:?} target carries a non-empty compilation map",
            t.target_language
        );
    }
}

#[test]
fn compile_all_is_deterministic_per_index() {
    let a = compile_all(&CompileBundle::load_toy());
    let b = compile_all(&CompileBundle::load_toy());
    let ha: Vec<_> = a.iter().map(content_hash).collect();
    let hb: Vec<_> = b.iter().map(content_hash).collect();
    assert_eq!(ha, hb, "per-index content_hash vector is stable across runs");
}
