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
//! Every assertion is ENGINE-AGNOSTIC — identity-parse, cross-process
//! byte-stability, complete (EOF-gated) capture, SAME-seed reproducibility,
//! pinned derived-seed values, and constraint-conformance — never a
//! model-specific output VALUE (that stays model-dependent) and never a
//! cross-seed coincidence: whether DISTINCT seeds diverge is the runtime's
//! decoding mode (greedy converges, sampling diverges), out of the adapter's
//! scope, so the test asserts only that the SAME seed reproduces. Conformance
//! is checked against a committed BOUNDED schema fixture (closed enum + bool,
//! `additionalProperties:false`): a free-running runtime that ignored
//! `--constraint` would almost surely emit non-conforming bytes and fail
//! here, so a pass is CONSISTENT WITH the constraint being honored
//! end-to-end — necessary, not alone sufficient (a runtime emitting a fixed
//! conforming object would also pass; the bounded schema makes accidental
//! conformance unlikely for a free-running weak model). The full
//! `schemas/clinical_ir.schema.json` is deliberately not used — its free
//! inter-token whitespace lets a weak greedy model degenerate into a
//! truncated, invalid instance (the expected weak-baseline failure mode,
//! recorded machine-locally), which would mask the conformance check.

use std::path::Path;
use std::time::Duration;

use ckc_cli::model::{ModelAdapter, ModelOutcome, ModelRun};
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

/// Assert a run is a COMPLETE capture and return its bytes. A clean exit
/// mints [`ModelOutcome::Completed`] only once stdout reached EOF, so a
/// timeout, an unproven-complete capture, or a failure fails here rather than
/// slipping a partial capture through a later bytes-only comparison. The
/// adapter guarantees the `Completed { bytes }` payload equals the raw
/// [`ModelRun::stdout_bytes`] on a clean EOF-gated exit (the cassette records
/// those bytes) — assert that field-invariant too.
fn completed_bytes(run: &ModelRun) -> &[u8] {
    let ModelOutcome::Completed { bytes } = &run.outcome else {
        panic!(
            "expected a complete capture, got {:?}; stderr: {}",
            run.outcome, run.stderr
        );
    };
    assert_eq!(
        bytes, &run.stdout_bytes,
        "Completed payload must equal the raw stdout_bytes"
    );
    bytes
}

/// Drive the real runtime through the full adapter codepath and assert the
/// §9 properties live: PATH-resolved construction with a parsed identity,
/// byte-stable greedy generation across processes, a complete EOF-gated
/// capture, schema-conformance of the constrained output, and a
/// reproducible, correctly-seeded k-sample draw.
#[test]
#[ignore = "live: drives the env-supplied model runtime; run manually with --ignored"]
fn live_adapter_end_to_end_through_env_runtime() {
    // `ModelAdapter::new()` honors the `CKC_MODEL_COMMAND` override, which
    // would resolve a DIFFERENT command and silently void the default
    // bare-name PATH-resolution this test exists to cover. Require it unset
    // or empty so the construction below truly exercises the default name.
    assert!(
        std::env::var("CKC_MODEL_COMMAND").map_or(true, |v| v.is_empty()),
        "unset CKC_MODEL_COMMAND to exercise the adapter's default bare-name PATH resolution"
    );

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
    let schema: Value =
        serde_json::from_slice(&std::fs::read(constraint).expect("fixture readable"))
            .expect("fixture is valid JSON");
    let validator = jsonschema::validator_for(&schema).expect("fixture compiles as a JSON Schema");
    // Conformance check reused for the single run and every sample. Parses
    // the constrained bytes and validates them against the very schema fed as
    // `--constraint`: a pass is CONSISTENT WITH the runtime honoring the
    // constraint (see the module doc) — NOT an assertion on which verdict it
    // picked.
    let conforms = |bytes: &[u8]| {
        let instance: Value =
            serde_json::from_slice(bytes).expect("constrained output parses as JSON");
        assert!(
            validator.is_valid(&instance),
            "constrained output must validate against the constraint schema; got {instance}"
        );
    };

    // Byte-stability: the SAME (prompt, constraint, seed) yields a complete,
    // byte-identical capture across two separate processes — the determinism
    // the recorded-bytes cassette replays (greedy, or any runtime that is
    // deterministic given its seed). Each run is proven `Completed`, so a
    // truncation or held pipe surfaces as a non-`Completed` outcome and fails
    // rather than passing a bytes-only comparison on a partial capture.
    let run1 = adapter.invoke(PROMPT, constraint, 42, BUDGET);
    let run2 = adapter.invoke(PROMPT, constraint, 42, BUDGET);
    let bytes1 = completed_bytes(&run1);
    let bytes2 = completed_bytes(&run2);
    assert_eq!(
        bytes2, bytes1,
        "the same seed must yield byte-identical output across processes"
    );
    conforms(bytes1);

    // k-sample draw: reproducible from `(base_seed, k)`, each sample seeded by
    // `derive_seed`, a complete capture, and conformant. Two draws are
    // compared SAMPLE-WISE on (seed, completed bytes) — not by whole-`Vec`
    // equality, which would also pin the diagnostic `stderr` and flake on any
    // nondeterministic runtime logging. The reproduced equality is per-seed;
    // the test does NOT require distinct seeds to coincide (greedy seed-
    // inertness is environment-specific, recorded machine-locally).
    let draw_a = adapter.invoke_samples(PROMPT, constraint, 42, 3, BUDGET);
    let draw_b = adapter.invoke_samples(PROMPT, constraint, 42, 3, BUDGET);
    assert_eq!(draw_a.len(), 3, "k == 3 yields three recorded samples");
    assert_eq!(draw_b.len(), 3, "k == 3 yields three recorded samples");
    for (i, (sa, sb)) in draw_a.iter().zip(draw_b.iter()).enumerate() {
        assert_eq!(
            sa.seed, DERIVED_SEEDS_42[i],
            "sample {i} must carry its derive_seed(42, {i}) seed"
        );
        assert_eq!(
            sb.seed, sa.seed,
            "sample {i} seed must reproduce across draws"
        );
        let sa_bytes = completed_bytes(&sa.run);
        let sb_bytes = completed_bytes(&sb.run);
        assert_eq!(
            sb_bytes, sa_bytes,
            "sample {i} must reproduce byte-identically from the same (base_seed, k)"
        );
        conforms(sa_bytes);
    }
}
