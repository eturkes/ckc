//! SPEC §9 model-runtime adapter — LIVE end-to-end confirmation
//! (`model-adapter.2b`). The `model-adapter.1`/`.2a` unit tests drive a
//! committed in-source stub; this test drives the REAL environment-supplied
//! runtime resolved on PATH under the adapter's default bare name, closing
//! the confirmation-vs-claim gap for the §9 runtime properties the cassette
//! design assumes. It is `#[ignore]`d so the normal suite (no runtime
//! present) skips it; run it manually where the runtime is installed:
//!
//!   cargo test -p ckc-cli --test model_live -- --ignored
//!
//! Every assertion is ENGINE-AGNOSTIC — byte-stability, k-sample
//! reproducibility, identity-parse, derived-seed values, and
//! constraint-conformance — never a model-specific output VALUE (that stays
//! model-dependent). Conformance is proven against a committed BOUNDED
//! schema fixture (closed enum + bool, `additionalProperties:false`): a
//! runtime that ignored `--constraint` would emit non-conforming bytes and
//! FAIL here. The full `schemas/clinical_ir.schema.json` is deliberately not
//! used — its free inter-token whitespace lets a weak greedy model
//! degenerate into a truncated, invalid instance (the expected weak-baseline
//! failure mode, recorded machine-locally), which would mask the mechanism
//! check.

use std::path::Path;
use std::time::Duration;

use ckc_cli::model::{ModelAdapter, ModelOutcome};
use serde_json::Value;

/// Committed bounded-schema constraint: a closed `verdict` enum plus a bool,
/// `additionalProperties:false`. Bounded so a weak greedy model emits a
/// complete, valid, terminating instance rather than looping on free
/// whitespace.
const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/bounded_verdict.schema.json"
);

/// One inline prompt for the whole run. With constrained decoding the prompt
/// only has to elicit a structured reply; its exact text never enters an
/// assertion (the output value is model-dependent).
const PROMPT: &str = "Two medication orders may interact. Decide the verdict \
    and whether it is clinically actionable, then reply with the structured result.";

/// Per-call wall-clock budget: a cold subprocess loads the model before
/// generating, so this is generous relative to the few-token bounded output.
const BUDGET: Duration = Duration::from_secs(120);

/// Pinned per-sample seeds for `base_seed = 42` (splitmix64 `derive_seed`,
/// also pinned in `model.rs` and memory `## Runtime`). Proves
/// `invoke_samples` seeds each draw through the replay-load-bearing
/// derivation end-to-end.
const DERIVED_SEEDS_42: [u64; 3] = [
    12_058_926_934_050_108_962,
    13_679_457_532_755_275_413,
    2_949_826_092_126_892_291,
];

/// Drive the real runtime through the full adapter codepath and assert the
/// §9 properties live: PATH-resolved construction with a parsed identity,
/// byte-stable greedy generation across processes, a complete EOF-gated
/// capture, schema-conformance of the constrained output, and a
/// reproducible, correctly-seeded k-sample draw.
#[test]
#[ignore = "live: drives the env-supplied model runtime; run manually with --ignored"]
fn live_adapter_end_to_end_through_env_runtime() {
    // Construct over the DEFAULT bare command name resolved on PATH — the
    // live PATH-resolution path `.1` could only prove for an ABSENT command.
    // Construction succeeds only if the `--identity` probe parsed a complete
    // `ModelIdentity`, so a held adapter already proves identity-parse.
    let adapter = ModelAdapter::new()
        .expect("env model runtime resolves on PATH and answers --identity with a parsed identity");
    let identity = adapter.identity();
    assert!(
        !identity.quant.is_empty() && !identity.runtime_version.is_empty(),
        "live identity fields must be non-empty: {identity:?}"
    );

    let constraint = Path::new(FIXTURE);

    // Byte-stability: the same (prompt, constraint, seed) yields a complete,
    // byte-identical capture across two separate processes — the greedy
    // determinism the recorded-bytes cassette replays. A complete capture
    // mints `Completed` (stdout reached EOF); a truncation or held pipe
    // would surface as `CaptureIncomplete` and fail here.
    let run1 = adapter.invoke(PROMPT, constraint, 42, BUDGET);
    let run2 = adapter.invoke(PROMPT, constraint, 42, BUDGET);
    let ModelOutcome::Completed { bytes } = run1.outcome.clone() else {
        panic!(
            "expected a complete capture, got {:?}; stderr: {}",
            run1.outcome, run1.stderr
        );
    };
    assert_eq!(
        run2.stdout_bytes, run1.stdout_bytes,
        "greedy output must be byte-stable across processes"
    );

    // Constraint-conformance: the constrained bytes parse as JSON and
    // validate against the very schema fed as `--constraint`. This is the
    // engine-side proof that the runtime honored the constraint end-to-end —
    // NOT an assertion on which verdict it picked.
    let schema: Value =
        serde_json::from_slice(&std::fs::read(constraint).expect("fixture readable"))
            .expect("fixture is valid JSON");
    let validator = jsonschema::validator_for(&schema).expect("fixture compiles as a JSON Schema");
    let instance: Value =
        serde_json::from_slice(&bytes).expect("constrained output parses as JSON");
    assert!(
        validator.is_valid(&instance),
        "constrained output must validate against the constraint schema; got {instance}"
    );

    // k-sample draw: reproducible from (base_seed, k), each sample seeded by
    // `derive_seed` and a complete capture. Under greedy decoding the seed is
    // inert, so every draw coincides with the single greedy output above —
    // the strongest live proof of end-to-end determinism (a sampling runtime
    // would diverge here, signalling the cassette assumption broke).
    let draw_a = adapter.invoke_samples(PROMPT, constraint, 42, 3, BUDGET);
    let draw_b = adapter.invoke_samples(PROMPT, constraint, 42, 3, BUDGET);
    assert_eq!(
        draw_a, draw_b,
        "k-sample draw must reproduce from the same (base_seed, k)"
    );
    assert_eq!(draw_a.len(), 3, "k == 3 yields three recorded samples");
    for (i, sample) in draw_a.iter().enumerate() {
        assert_eq!(
            sample.seed, DERIVED_SEEDS_42[i],
            "sample {i} must carry its derive_seed(42, {i}) seed"
        );
        assert!(
            matches!(sample.run.outcome, ModelOutcome::Completed { .. }),
            "sample {i} must be a complete capture, got {:?}",
            sample.run.outcome
        );
        assert_eq!(
            sample.run.stdout_bytes, bytes,
            "greedy is seed-inert: sample {i} must equal the single greedy output"
        );
    }
}
