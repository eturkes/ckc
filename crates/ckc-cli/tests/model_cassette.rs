//! SPEC §9 model cassette — committed test cassette + runtime-absent replay
//! (`model-cassette.2`, the live unit mirroring `model-adapter.2b`). The
//! `model-cassette.1` unit tests drive synthetic payloads through the store;
//! this pair closes the confirmation-vs-claim gap for the §9 cassette workflow:
//! an `#[ignore]`d bless RECORDS one cassette against the REAL
//! environment-supplied runtime resolved on PATH, and a normal test REPLAYS that
//! committed cassette with NO runtime present — the deterministic, runtime-absent
//! path CI and every other host take.
//!
//! Run the bless manually where the runtime is installed, commit the written
//! cassette, then re-pin `CASSETTE_HASH` from the recorded wrapper:
//!
//!   cargo test -p ckc-cli --test model_cassette record_cassette -- --ignored
//!   jq -r .content_hash \
//!     crates/ckc-cli/tests/fixtures/cassettes/route.fixture/test_source.fixture/seed-42.json
//!
//! The replay assertions are ENGINE-AGNOSTIC: the committed cassette's
//! payload-canonical `content_hash` is byte-pinned (drift guard), its recorded
//! output parses as JSON and schema-validates against the committed BOUNDED
//! fixture (closed enum + bool, `additionalProperties:false`), and its §4.4
//! provenance is the fixed AI-recording contract. Conformance is CONSISTENT WITH
//! `--constraint` honored end-to-end at record time — necessary, not alone
//! sufficient (the `.2b` framing) — never a model-specific output VALUE.
//!
//! IDENTITY: the committed cassette carries the runtime's REAL `model_identity`
//! (model/quant/engine), by deliberate decision — a recorded cassette is
//! machine-specific MEASUREMENT data whose honest provenance records what
//! produced it, so it is exempt from the engine-agnostic synthetic-token rule
//! that governs hand-authored contract/fixture artifacts (and from the identity
//! audit-grep). The replay test never asserts an identity VALUE, so the recorded
//! model is free to drift; only the output, provenance, and content hash pin.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ckc_cli::cassette::{CassetteKey, CassetteStore, RecordContext};
use ckc_cli::model::ModelAdapter;
use ckc_core::{Effect, EvidenceStatus, Origin, Producer, hash_bytes};
use serde_json::Value;

/// `crates/ckc-cli/tests/fixtures` — the cassette store roots here, so the
/// committed cassette lives beside the live-conformance schema fixture.
fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Committed bounded-schema constraint, reused as the record-time `--constraint`
/// and the replay-time conformance oracle (see `model_live.rs`).
fn constraint_path() -> PathBuf {
    fixtures_root().join("bounded_verdict.schema.json")
}

/// The single recorded call's key: a synthetic route/source and base seed 42.
fn cassette_key() -> CassetteKey {
    CassetteKey {
        route: "route.fixture".parse().unwrap(),
        source: "test_source.fixture".parse().unwrap(),
        seed: 42,
    }
}

/// Synthetic provenance stamped into the cassette wrapper's `producer`; the
/// recorded identity is the runtime's, the rest of the fixture is synthetic.
fn producer() -> Producer {
    Producer {
        pipeline_id: "pipe.fixture".parse().unwrap(),
        pipeline_step_id: "processing_stage.fixture.model_fill".parse().unwrap(),
        toolchain_manifest_hash: hash_bytes(b"fixture toolchain"),
    }
}

/// Inline prompt, mirroring `model_live.rs`: with constrained decoding it only
/// has to elicit a structured reply; its exact text never enters an assertion.
const PROMPT: &str = "Two medication orders may interact. Decide the verdict \
    and whether it is clinically actionable, then reply with the structured result.";

/// Synthetic route prompt template the prompt is notionally rendered from; its
/// raw-byte hash is the cassette's `prompt_template_hash`.
const PROMPT_TEMPLATE: &[u8] =
    b"route.fixture prompt template (synthetic): {source} -> bounded verdict";

/// Payload-canonical `content_hash` of the committed cassette (the §4.4 hash the
/// store stamps over the wrapper, NOT the file-byte sha256sum). Re-pin from the
/// recorded wrapper after a bless: `jq -r .content_hash <cassette>`.
const CASSETTE_HASH: &str =
    "sha256:8b465db593e000deadce69e22f87d62dcf543f908d2ce0174b9d93913dc130bd";

/// Parse `bytes` as JSON and assert it validates against the bounded constraint
/// fixture — the conformance oracle shared by the bless guard and the replay
/// assertion (the `.2b` framing: consistent with `--constraint` honored, not
/// alone sufficient).
fn assert_conforms(bytes: &[u8]) {
    let schema: Value = serde_json::from_slice(&std::fs::read(constraint_path()).unwrap())
        .expect("constraint fixture is valid JSON");
    let validator = jsonschema::validator_for(&schema).expect("fixture compiles as a JSON Schema");
    let instance: Value = serde_json::from_slice(bytes).expect("recorded output parses as JSON");
    assert!(
        validator.is_valid(&instance),
        "recorded output must validate against the bounded constraint; got {instance}"
    );
}

/// BLESS (live, `#[ignore]`d): record one cassette against the env-supplied
/// runtime resolved on PATH under the adapter's default bare name, writing it to
/// the committed fixtures path. Re-pin `CASSETTE_HASH` from the result, commit
/// the cassette, then the normal `replay_committed_cassette` test guards it.
#[test]
#[ignore = "live: records a cassette against the env-supplied model runtime; run manually with --ignored"]
fn record_cassette() {
    // A `CKC_MODEL_COMMAND` override would resolve a DIFFERENT command and void
    // the default bare-name PATH resolution this bless records against; require
    // it unset or empty.
    assert!(
        std::env::var("CKC_MODEL_COMMAND").map_or(true, |v| v.is_empty()),
        "unset CKC_MODEL_COMMAND to record against the adapter's default bare-name runtime"
    );
    let adapter =
        ModelAdapter::new().expect("env model runtime resolves on PATH and answers --identity");
    let store = CassetteStore::new(fixtures_root());
    let key = cassette_key();
    let constraint = constraint_path();
    let ctx = RecordContext {
        producer: producer(),
        prompt_template_hash: hash_bytes(PROMPT_TEMPLATE),
        budget: Duration::from_secs(120),
    };
    let wrapper = store
        .record(&adapter, &key, PROMPT, &constraint, &ctx)
        .expect("record writes a cassette from a clean live capture");
    // Guard the recording before it is committed: a degenerate (non-conforming)
    // capture must not be pinned.
    assert_conforms(&wrapper.payload.output_bytes().expect("output_hex decodes"));
    // Surface the hash to re-pin CASSETTE_HASH (or: `jq -r .content_hash <cassette>`).
    println!(
        "recorded cassette content_hash = {}",
        wrapper.content_hash.as_str()
    );
}

/// REPLAY (runtime-absent, normal `cargo test`): read the committed cassette
/// without touching the runtime and pin it — the payload-canonical content hash
/// (drift guard), the recorded output's JSON/schema conformance, and the fixed
/// §4.4 AI-recording provenance.
#[test]
fn replay_committed_cassette() {
    let store = CassetteStore::new(fixtures_root());
    let wrapper = store
        .replay(&cassette_key())
        .expect("committed cassette replays without the runtime");

    assert_eq!(
        wrapper.content_hash.as_str(),
        CASSETTE_HASH,
        "committed cassette content_hash drifted from the pin"
    );

    let output = wrapper.payload.output_bytes().expect("output_hex decodes");
    assert_conforms(&output);

    assert_eq!(wrapper.origin, Origin::AiGenerated);
    assert_eq!(
        wrapper.evidence_status,
        EvidenceStatus::EvidenceDiscoveryOnly
    );
    assert_eq!(wrapper.external_effects, vec![Effect::Ai]);
}
