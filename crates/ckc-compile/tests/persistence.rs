//! Subtask 0.8.16: CAS persistence + cross-run determinism for the Phase-0
//! `CompiledTarget` portfolio.
//!
//! Pipeline: load the toy bundle, emit the nine-target portfolio with
//! `compile_all`, wrap each `CompiledTarget` as an
//! `ArtifactKind::CompiledTarget` envelope (stage = "compile"), and store the
//! batch in a fresh content-addressed store.
//!
//! Determinism strategy mirrors `cas_manifest_hash_is_stable`
//! (crates/ckc-store/tests/research_fixture_bundle.rs) and
//! crates/ckc-retrieve/tests/persistence.rs: raw `StoreManifest` bytes carry
//! per-file `stored_at_epoch` and drift across runs by design, so the cross-run
//! gate compares per-artifact `envelope_hash` / `content_hash`, never raw
//! manifest bytes. No committed fixture is added here — the portfolio is
//! already byte-locked by the committed `logic/*`+`lean/Ckc/*` files (0.8.14)
//! and the manifest golden (0.8.15).

use std::collections::HashSet;

use ckc_compile::{CompileBundle, CompiledTarget, compile_all};
use ckc_core::canonical::{ContentHash, content_hash};
use ckc_core::envelope::{ArtifactEnvelope, ArtifactKind, ArtifactMeta};
use ckc_core::profile::SemanticProfile;
use ckc_store::ContentStore;
use serde_json::json;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Per-target source profiles
// ---------------------------------------------------------------------------

/// SPEC §9 semantic profile each portfolio target's source content occupies,
/// aligned element-for-element with `compile_all` / `ARTIFACT_PATHS` order.
/// Each target reflects the profile of the CKC content it compiles: the
/// deontic norm clash (`CKC-Norm`), the decision table (`CKC-Decision`), the
/// MaxSMT repair over the norm conflict (`CKC-Norm`+`CKC-Defeasible`), the
/// defeasible/argumentation and priority/acyclicity views (`CKC-Defeasible`),
/// the Event Calculus narrative and its TLA+ persistence invariant
/// (`CKC-Event`), and the Lean norm-conflict theorem (`CKC-Norm`). Mirrors the
/// per-kind profile convention in
/// `crates/ckc-store/tests/research_fixture_bundle.rs::wrap_all`.
const TARGET_PROFILES: [&[SemanticProfile]; 9] = [
    &[SemanticProfile::Norm],                              // smt norm_conflict
    &[SemanticProfile::Decision],                          // smt decision_table
    &[SemanticProfile::Norm, SemanticProfile::Defeasible], // smt repair_maxsmt
    &[SemanticProfile::Defeasible],                        // asp defeasible
    &[SemanticProfile::Event],                             // asp event_calculus
    &[SemanticProfile::Defeasible],                        // datalog priority
    &[SemanticProfile::Norm],                              // lean theorem
    &[SemanticProfile::Event],                             // tla+ stub
    &[SemanticProfile::Defeasible],                        // alloy stub
];

// ---------------------------------------------------------------------------
// Envelope wrapping (stage = "compile") + store helper
// ---------------------------------------------------------------------------

/// Template matches `envelope_meta` in the ckc-store and ckc-retrieve
/// persistence tests: identical pipeline-metadata fields, distinguished by
/// `stage = "compile"`. The placeholder `content_hash` is overwritten by
/// `ArtifactEnvelope::wrap` with the payload's true canonical digest.
fn meta_compile(profiles: &[SemanticProfile]) -> ArtifactMeta {
    ArtifactMeta {
        schema_version: "0.0.0".into(),
        producer_version: "ckc-compile/0.0.0".into(),
        command_manifest: json!({"command": "ckc", "args": ["demo", "research-kernel"]}),
        source_input_hashes: vec![],
        parent_hashes: vec![],
        stage: "compile".into(),
        semantic_profiles: profiles.to_vec(),
        content_hash: ContentHash(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
        ),
        certificate_ids: vec![],
        replay_command: Some("ckc demo research-kernel --replay --out runs/research".into()),
    }
}

fn wrap_portfolio(targets: &[CompiledTarget]) -> Vec<ArtifactEnvelope> {
    assert_eq!(
        targets.len(),
        TARGET_PROFILES.len(),
        "compile_all target count must match the per-target profile table",
    );
    targets
        .iter()
        .zip(TARGET_PROFILES)
        .map(|(t, profiles)| {
            ArtifactEnvelope::wrap(ArtifactKind::CompiledTarget, t, meta_compile(profiles))
        })
        .collect()
}

fn build_store_and_verify(envelopes: &[ArtifactEnvelope]) -> (TempDir, Vec<ContentHash>) {
    let tmp = TempDir::new().expect("create tempdir");
    let store = ContentStore::new(tmp.path());
    let hashes = store.put_batch(envelopes).expect("put_batch");
    for h in &hashes {
        assert!(store.exists(h), "envelope {h:?} must exist after put_batch");
        assert!(
            store.verify(h).expect("verify call"),
            "envelope {h:?} must verify",
        );
    }
    (tmp, hashes)
}

// ---------------------------------------------------------------------------
// Gate 1: every CompiledTarget is stored, verified, and content-hashed
// ---------------------------------------------------------------------------

#[test]
fn each_target_is_stored_verified_and_content_hashed() {
    let targets = compile_all(&CompileBundle::load_toy());
    assert_eq!(targets.len(), 9, "Phase-0 portfolio emits nine targets");
    let envelopes = wrap_portfolio(&targets);

    let (_tmp, hashes) = build_store_and_verify(&envelopes);

    // `wrap` overwrote the placeholder hash with the CompiledTarget's true
    // canonical digest.
    for (env, t) in envelopes.iter().zip(&targets) {
        assert_eq!(
            env.meta.content_hash,
            content_hash(t),
            "envelope content_hash must equal the {:?} CompiledTarget canonical hash",
            t.target_language,
        );
    }

    // Distinct targets yield distinct envelope store keys; catches accidental
    // payload aliasing even where target_language or profiles repeat.
    let unique: HashSet<&ContentHash> = hashes.iter().collect();
    assert_eq!(
        unique.len(),
        hashes.len(),
        "every CompiledTarget envelope must have a distinct store key",
    );
}

// ---------------------------------------------------------------------------
// Gate 2: per-artifact hashes match across two fully independent runs/stores
// ---------------------------------------------------------------------------

#[test]
fn per_target_hashes_match_across_two_independent_runs() {
    let targets1 = compile_all(&CompileBundle::load_toy());
    let envelopes1 = wrap_portfolio(&targets1);
    let (_t1, hashes1) = build_store_and_verify(&envelopes1);

    let targets2 = compile_all(&CompileBundle::load_toy());
    let envelopes2 = wrap_portfolio(&targets2);
    let (_t2, hashes2) = build_store_and_verify(&envelopes2);

    assert_eq!(
        targets1, targets2,
        "compile_all output must be deterministic across runs",
    );
    for (e1, e2) in envelopes1.iter().zip(&envelopes2) {
        assert_eq!(
            e1.envelope_hash(),
            e2.envelope_hash(),
            "envelope_hash must be stable across runs for {:?}",
            e1.kind,
        );
        assert_eq!(
            e1.meta.content_hash, e2.meta.content_hash,
            "inner content_hash must be stable across runs",
        );
    }
    assert_eq!(
        hashes1, hashes2,
        "per-target store keys must be identical across independent stores",
    );
}
