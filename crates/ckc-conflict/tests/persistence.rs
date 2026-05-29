//! Subtask 0.10.11: CAS persistence + cross-run determinism for the Phase-0
//! conflict-detection artifacts.
//!
//! Pipeline: load the toy bundle, run `verify_all`, run `detect_all`, then wrap
//! each `Conflict` (`ArtifactKind::Conflict`, profile `CKC-Para`) and each
//! `ArgumentGraph` (`ArtifactKind::ArgumentGraph`, profile `CKC-Defeasible`) as a
//! `stage = "conflict"` envelope, and store the batch in a fresh
//! content-addressed store. The conflict report and manifest carry no dedicated
//! `ArtifactKind` and are byte-locked elsewhere (committed `certs/conflicts/*` +
//! `certs/argument_graphs/*`, 0.10.9; manifest golden, 0.10.10), so they are out
//! of scope here.
//!
//! Determinism strategy mirrors `crates/ckc-verify/tests/persistence.rs` and
//! `crates/ckc-compile/tests/persistence.rs`: raw `StoreManifest` bytes carry
//! per-file `stored_at_epoch` and drift across runs by design, so the cross-run
//! gate compares per-artifact `envelope_hash` / `content_hash` and `put_batch`
//! store keys, never raw manifest bytes. No committed fixture is added here — the
//! conflict set is already byte-locked by the committed `certs/` files (0.10.9)
//! and the conflict-manifest golden (0.10.10).

use ckc_conflict::{CompileBundle, ContentHash, content_hash, detect_all, verify_all};
use ckc_core::envelope::{ArtifactEnvelope, ArtifactKind, ArtifactMeta};
use ckc_core::profile::SemanticProfile;
use ckc_store::ContentStore;
use serde_json::json;
use tempfile::TempDir;

/// Phase-0 conflict artifact counts: `detect_all` emits 3 conflicts
/// (norm-contradiction, decision-table-defect, temporal-violation) and the single
/// Dung argument graph backing the defeasible conflicts.
const CONFLICT_COUNT: usize = 3;
const ARGUMENT_GRAPH_COUNT: usize = 1;
const TOTAL: usize = CONFLICT_COUNT + ARGUMENT_GRAPH_COUNT;

// ---------------------------------------------------------------------------
// Envelope wrapping (stage = "conflict") + store helper
// ---------------------------------------------------------------------------

/// Template matches `meta_verify` in the ckc-verify persistence test: identical
/// pipeline-metadata fields, distinguished by `stage = "conflict"` and a
/// per-kind `semantic_profile` — conflicts occupy SPEC §9 `CKC-Para` (the
/// inconsistency-tolerant conflict-local review profile), argument graphs occupy
/// `CKC-Defeasible` (structured defeasible argumentation). The placeholder
/// `content_hash` is overwritten by `ArtifactEnvelope::wrap` with the payload's
/// true canonical digest.
fn meta_conflict(profile: SemanticProfile) -> ArtifactMeta {
    ArtifactMeta {
        schema_version: "0.0.0".into(),
        producer_version: "ckc-conflict/0.0.0".into(),
        command_manifest: json!({"command": "ckc", "args": ["demo", "research-kernel"]}),
        source_input_hashes: vec![],
        parent_hashes: vec![],
        stage: "conflict".into(),
        semantic_profiles: vec![profile],
        content_hash: ContentHash(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
        ),
        certificate_ids: vec![],
        replay_command: Some("ckc demo research-kernel --replay --out runs/research".into()),
    }
}

/// Run the load→verify_all→detect_all→wrap pipeline and wrap its full artifact
/// set — every Conflict (`CKC-Para`), then every ArgumentGraph
/// (`CKC-Defeasible`) — returning the envelopes alongside each artifact's typed
/// canonical `content_hash` (parallel-indexed), so the storage gate can compare
/// envelope hashes against the source artifacts.
fn wrap_conflicts(bundle: &CompileBundle) -> (Vec<ArtifactEnvelope>, Vec<ContentHash>) {
    let report = detect_all(bundle, &verify_all(bundle));
    let mut envelopes = Vec::new();
    let mut inner_hashes = Vec::new();
    for conflict in &report.conflicts {
        inner_hashes.push(content_hash(conflict));
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::Conflict,
            conflict,
            meta_conflict(SemanticProfile::Para),
        ));
    }
    for graph in &report.argument_graphs {
        inner_hashes.push(content_hash(graph));
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::ArgumentGraph,
            graph,
            meta_conflict(SemanticProfile::Defeasible),
        ));
    }
    (envelopes, inner_hashes)
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
// Gate 1: every conflict artifact is stored, verified, and content-hashed
// ---------------------------------------------------------------------------

#[test]
fn each_conflict_artifact_is_stored_verified_and_content_hashed() {
    let (envelopes, inner_hashes) = wrap_conflicts(&CompileBundle::load_toy());
    assert_eq!(
        envelopes.len(),
        TOTAL,
        "detect_all emits 3 conflicts + 1 argument graph",
    );

    let (_tmp, _hashes) = build_store_and_verify(&envelopes);

    // `wrap` overwrote the placeholder hash with each artifact's true canonical
    // digest (Conflict / ArgumentGraph respectively).
    for (env, expected) in envelopes.iter().zip(&inner_hashes) {
        assert_eq!(
            &env.meta.content_hash, expected,
            "envelope content_hash must equal the {:?} artifact canonical hash",
            env.kind,
        );
    }
}

// ---------------------------------------------------------------------------
// Gate 2: per-artifact hashes match across two fully independent runs/stores
// ---------------------------------------------------------------------------

#[test]
fn per_artifact_hashes_match_across_two_independent_runs() {
    let (envelopes1, _inner1) = wrap_conflicts(&CompileBundle::load_toy());
    let (_t1, hashes1) = build_store_and_verify(&envelopes1);

    let (envelopes2, _inner2) = wrap_conflicts(&CompileBundle::load_toy());
    let (_t2, hashes2) = build_store_and_verify(&envelopes2);

    assert_eq!(
        envelopes1.len(),
        envelopes2.len(),
        "detect_all → wrap must emit the same artifact count across runs",
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
            "inner content_hash must be stable across runs for {:?}",
            e1.kind,
        );
    }
    assert_eq!(
        hashes1, hashes2,
        "per-artifact store keys must be identical across independent stores",
    );
}
