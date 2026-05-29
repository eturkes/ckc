//! Subtask 0.9.17: CAS persistence + cross-run determinism for the Phase-0
//! verification artifacts.
//!
//! Pipeline: load the toy bundle, run `verify_all`, wrap each Certificate
//! (`ArtifactKind::Certificate`), ExecutionWitness
//! (`ArtifactKind::ExecutionWitness`), and AssuranceNode
//! (`ArtifactKind::AssuranceNode`) as a `CKC-Cert` envelope (stage = "verify"),
//! and store the batch in a fresh content-addressed store. The certificate graph
//! and portfolio-agreement records carry no dedicated `ArtifactKind` and are
//! byte-locked elsewhere (committed `certs/*`, 0.9.15; manifest golden, 0.9.16),
//! so they are out of scope here.
//!
//! Determinism strategy mirrors `cas_manifest_hash_is_stable`
//! (crates/ckc-store/tests/research_fixture_bundle.rs) and
//! crates/ckc-compile/tests/persistence.rs: raw `StoreManifest` bytes carry
//! per-file `stored_at_epoch` and drift across runs by design, so the cross-run
//! gate compares per-artifact `envelope_hash` / `content_hash` and `put_batch`
//! store keys, never raw manifest bytes. No committed fixture is added here — the
//! verification set is already byte-locked by the committed `certs/*` files
//! (0.9.15) and the verification-manifest golden (0.9.16).

use ckc_compile::CompileBundle;
use ckc_core::canonical::{ContentHash, content_hash};
use ckc_core::envelope::{ArtifactEnvelope, ArtifactKind, ArtifactMeta};
use ckc_core::profile::SemanticProfile;
use ckc_store::ContentStore;
use ckc_verify::verify_all;
use serde_json::json;
use tempfile::TempDir;

/// Phase-0 verification artifact counts: `verify_all` emits 11 certificates and
/// 11 execution witnesses (the 10 solver/cvc5 entries plus the in-process SHACL
/// grounding cert/witness), and the assurance seed is the GSN root goal plus its
/// three strategy nodes.
const CERT_COUNT: usize = 11;
const WITNESS_COUNT: usize = 11;
const ASSURANCE_NODE_COUNT: usize = 4;
const TOTAL: usize = CERT_COUNT + WITNESS_COUNT + ASSURANCE_NODE_COUNT;

// ---------------------------------------------------------------------------
// Envelope wrapping (stage = "verify") + store helper
// ---------------------------------------------------------------------------

/// Template matches `envelope_meta` in the ckc-store, ckc-retrieve, and
/// ckc-compile persistence tests: identical pipeline-metadata fields,
/// distinguished by `stage = "verify"`. Every verification artifact occupies
/// SPEC §9 `CKC-Cert` (certificates, solver/proof outputs, replay status,
/// assurance links). The placeholder `content_hash` is overwritten by
/// `ArtifactEnvelope::wrap` with the payload's true canonical digest.
fn meta_verify() -> ArtifactMeta {
    ArtifactMeta {
        schema_version: "0.0.0".into(),
        producer_version: "ckc-verify/0.0.0".into(),
        command_manifest: json!({"command": "ckc", "args": ["demo", "research-kernel"]}),
        source_input_hashes: vec![],
        parent_hashes: vec![],
        stage: "verify".into(),
        semantic_profiles: vec![SemanticProfile::Cert],
        content_hash: ContentHash(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
        ),
        certificate_ids: vec![],
        replay_command: Some("ckc demo research-kernel --replay --out runs/research".into()),
    }
}

/// Run `verify_all` and wrap its full artifact set — every Certificate, then
/// every ExecutionWitness, then every AssuranceNode — returning the envelopes
/// alongside each artifact's typed canonical `content_hash` (parallel-indexed),
/// so the storage gate can compare envelope hashes against the source artifacts.
fn wrap_verification(bundle: &CompileBundle) -> (Vec<ArtifactEnvelope>, Vec<ContentHash>) {
    let report = verify_all(bundle);
    let mut envelopes = Vec::new();
    let mut inner_hashes = Vec::new();
    for cert in &report.certificates {
        inner_hashes.push(content_hash(cert));
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::Certificate,
            cert,
            meta_verify(),
        ));
    }
    for witness in &report.witnesses {
        inner_hashes.push(content_hash(witness));
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::ExecutionWitness,
            witness,
            meta_verify(),
        ));
    }
    for node in &report.assurance.0 {
        inner_hashes.push(content_hash(node));
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::AssuranceNode,
            node,
            meta_verify(),
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
// Gate 1: every verification artifact is stored, verified, and content-hashed
// ---------------------------------------------------------------------------

#[test]
fn each_verification_artifact_is_stored_verified_and_content_hashed() {
    let (envelopes, inner_hashes) = wrap_verification(&CompileBundle::load_toy());
    assert_eq!(
        envelopes.len(),
        TOTAL,
        "verify_all emits 11 certificates + 11 witnesses + 4 assurance nodes",
    );

    let (_tmp, _hashes) = build_store_and_verify(&envelopes);

    // `wrap` overwrote the placeholder hash with each artifact's true canonical
    // digest (Certificate / ExecutionWitness / AssuranceNode respectively).
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
    let (envelopes1, _inner1) = wrap_verification(&CompileBundle::load_toy());
    let (_t1, hashes1) = build_store_and_verify(&envelopes1);

    let (envelopes2, _inner2) = wrap_verification(&CompileBundle::load_toy());
    let (_t2, hashes2) = build_store_and_verify(&envelopes2);

    assert_eq!(
        envelopes1.len(),
        envelopes2.len(),
        "verify_all → wrap must emit the same artifact count across runs",
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
