//! Gate for tasks 0.8.13/0.8.14: `compile_all` runs every Phase-0 emitter in
//! one fixed order, grounds each emitted target, is byte-deterministic per
//! index across independent runs, and round-trips byte-identically through the
//! committed `logic/*` + `lean/Ckc/*` artifact files.

use std::fs;
use std::path::PathBuf;

use ckc_compile::{ARTIFACT_PATHS, CompileBundle, TargetLanguage, compile_all, content_hash};

/// Repository root, two levels above this crate's manifest, so the committed
/// target-artifact files resolve under their `logic/`/`lean/` paths.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

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
    assert_eq!(
        ha, hb,
        "per-index content_hash vector is stable across runs"
    );
}

/// Gate for task 0.8.14: every committed target-artifact file holds exactly the
/// bytes its emitter produces, so regeneration is a byte-identical no-op. Drift
/// here means an emitter changed without rerunning the regenerator below.
#[test]
fn committed_artifact_files_match_emitted_text() {
    let targets = compile_all(&CompileBundle::load_toy());
    let root = workspace_root();
    assert_eq!(
        targets.len(),
        ARTIFACT_PATHS.len(),
        "every compile_all target has a committed artifact path"
    );
    for (target, rel) in targets.iter().zip(ARTIFACT_PATHS) {
        let committed = fs::read_to_string(root.join(rel))
            .unwrap_or_else(|e| panic!("read committed artifact {rel}: {e}"));
        assert_eq!(
            committed, target.artifact_text,
            "committed {rel} drifted from the {:?} emitter; rerun \
             `cargo test -p ckc-compile --test portfolio -- --ignored \
             regenerate_target_artifact_files`",
            target.target_language
        );
    }
}

/// Regenerator (ignored by default; run with `--ignored` after intentional
/// emitter changes). Writes each `compile_all` target's `artifact_text` to its
/// canonical `logic/`/`lean/` path, creating parent directories as needed.
#[test]
#[ignore = "regenerate-only; rewrites the committed logic/* and lean/Ckc/* target-artifact files"]
fn regenerate_target_artifact_files() {
    let targets = compile_all(&CompileBundle::load_toy());
    let root = workspace_root();
    for (target, rel) in targets.iter().zip(ARTIFACT_PATHS) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().expect("artifact path has a parent dir"))
            .unwrap_or_else(|e| panic!("create dir for {rel}: {e}"));
        fs::write(&path, &target.artifact_text).unwrap_or_else(|e| panic!("write {rel}: {e}"));
    }
}
