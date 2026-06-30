//! SPEC §7.4/§9 model-fill processing stage core — the nondeterministic stage
//! that fills a route's target artifact (ClinicalIR JSON, SMT text, …) from a
//! recorded model call, generic over the target.
//!
//! [`model_fill`] obtains the model output through the [`CassetteStore`] —
//! replay (default, runtime-absent) reads the committed cassette, record (gated)
//! invokes the [`ModelAdapter`] once and writes the recording — then parses the
//! decoded output into the route's target via a caller-supplied parser. The
//! parser is the §4 acceptance check, so the target choice (ClinicalIR / SMT)
//! and its acceptance (JSON-Schema + structural / solver parse) stay with the
//! route; a parse-or-acceptance failure becomes a §7.4 `ai_schema_violation`
//! diagnostic in place of a target.
//!
//! The stage counts the model calls it recorded — one per fill here; the
//! stage-model-fill.2 repair loop raises it — for the §7.3 recorded-call metric.
//! The count rides the stage's §4.6 event as the [`RECORDED_CALLS_COUNTER`]
//! resource counter; the route/run wiring (run-m2.1) builds that event from the
//! returned count. No repair loop or grounding check yet (→ stage-model-fill.2).

use std::path::Path;

use ckc_core::{DiagnosticCode, DiagnosticRecord, Outcome};

use crate::cassette::{CassetteError, CassetteKey, CassetteStore, RecordContext};
use crate::model::ModelAdapter;
use crate::shell::static_id;

/// §4.6 event resource-counter key for the §7.3 recorded-call count.
pub const RECORDED_CALLS_COUNTER: &str = "recorded_calls";

/// How [`model_fill`] obtains its model output.
pub enum FillSource<'a> {
    /// Replay the committed cassette — never touches the runtime (default).
    Replay,
    /// Invoke the runtime once and record it (gated; requires the adapter,
    /// prompt, constraint path, and recording context).
    Record {
        /// Live model-runtime adapter.
        adapter: &'a ModelAdapter,
        /// Prompt sent to the runtime over stdin.
        prompt: &'a str,
        /// Constraint (schema/grammar) path passed to the runtime.
        constraint: &'a Path,
        /// Provenance + budget for the recording.
        ctx: &'a RecordContext,
    },
}

/// One model-fill attempt.
#[derive(Debug)]
pub struct ModelFill<T> {
    /// The accepted target, or `None` when the model output failed the §4
    /// acceptance check (an `ai_schema_violation` then sits in `diagnostics`).
    pub target: Option<T>,
    /// Stage diagnostics — an `ai_schema_violation` on parse/acceptance failure.
    pub diagnostics: Vec<DiagnosticRecord>,
    /// Model calls recorded for this fill: the §7.3 recorded-call count, carried
    /// on the stage's §4.6 event under [`RECORDED_CALLS_COUNTER`]. One per fill
    /// here (the stage-model-fill.2 repair loop raises it).
    pub recorded_calls: u64,
}

/// Run one model-fill: obtain the cassette (replay/record) through `store`,
/// decode the recorded output, and parse it into the route target with `parse`.
///
/// `parse` is the route's §4 acceptance check over the raw model output; its
/// `Err(reason)` becomes an `ai_schema_violation` and yields no target. Generic
/// over the target, so the route selects ClinicalIR / SMT by supplying the
/// parser. A cassette IO/contract failure returns `Err` — the recording is
/// missing or malformed, distinct from a model output that fails acceptance.
pub fn model_fill<T>(
    store: &CassetteStore,
    key: &CassetteKey,
    source: FillSource<'_>,
    parse: impl FnOnce(&[u8]) -> Result<T, String>,
) -> Result<ModelFill<T>, CassetteError> {
    let cassette = match source {
        FillSource::Replay => store.replay(key)?,
        FillSource::Record {
            adapter,
            prompt,
            constraint,
            ctx,
        } => store.record(adapter, key, prompt, constraint, ctx)?,
    };
    // The store validated `output_hex` decodability on load, so this holds.
    let output = cassette
        .payload
        .output_bytes()
        .map_err(|_| CassetteError::InvalidHex)?;
    // One recorded model call per fill (the stage-model-fill.2 repair loop raises this).
    let recorded_calls: u64 = 1;
    Ok(match parse(&output) {
        Ok(target) => ModelFill {
            target: Some(target),
            diagnostics: Vec::new(),
            recorded_calls,
        },
        Err(reason) => ModelFill {
            target: None,
            diagnostics: vec![ai_schema_violation(reason)],
            recorded_calls,
        },
    })
}

/// The §7.4 `ai_schema_violation` diagnostic: model output failed the stage's §4
/// acceptance check (parse or schema). Outcome `invalid`, mirroring the
/// deterministic `schema_invalid`; the reason rides the structured payload.
fn ai_schema_violation(reason: String) -> DiagnosticRecord {
    DiagnosticRecord {
        code: DiagnosticCode::AiSchemaViolation,
        outcome: Outcome::Invalid,
        payload: vec![(static_id("reason"), reason)],
        region_ids: Vec::new(),
        artifact_hashes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::{CassettePayload, ModelIdentity, Producer, hash_bytes};

    fn key() -> CassetteKey {
        CassetteKey {
            route: "route.fixture".parse().unwrap(),
            source: "test_source.fixture".parse().unwrap(),
            seed: 42,
        }
    }

    fn producer() -> Producer {
        Producer {
            pipeline_id: static_id("pipe.test"),
            pipeline_step_id: static_id("processing_stage.test.model_fill"),
            toolchain_manifest_hash: hash_bytes(b"toolchain"),
        }
    }

    fn temp_store(tag: &str) -> (CassetteStore, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("ckc-model-fill-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        (CassetteStore::new(&dir), dir)
    }

    // Persist a synthetic cassette carrying `output` for runtime-absent replay,
    // using the store's own contract-valid wrapper builder.
    fn seed_cassette(store: &CassetteStore, k: &CassetteKey, output: &[u8]) {
        let payload = CassettePayload::from_output(
            k.route.clone(),
            k.source.clone(),
            k.seed,
            "prompt body".to_owned(),
            hash_bytes(b"constraint"),
            hash_bytes(b"prompt template"),
            ModelIdentity {
                model_id: "model.fixture".parse().unwrap(),
                quant: "fixture_quant".to_owned(),
                runtime_version: "1.0.0".to_owned(),
            },
            output,
        );
        let wrapper = store.build_wrapper(k, payload, producer()).unwrap();
        store.persist(k, wrapper).unwrap();
    }

    // A route parser: accept JSON objects carrying an `ok` field (a stand-in for
    // the route's §4 acceptance check); anything else fails acceptance.
    fn parse_json(bytes: &[u8]) -> Result<serde_json::Value, String> {
        let value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|e| format!("parse: {e}"))?;
        if value.get("ok").is_some() {
            Ok(value)
        } else {
            Err("missing required field `ok`".to_owned())
        }
    }

    // A valid recording fills the target and accounts exactly one recorded call.
    #[test]
    fn valid_fill_yields_target_and_one_recorded_call() {
        let (store, dir) = temp_store("valid");
        let k = key();
        seed_cassette(&store, &k, br#"{"ok":true}"#);
        let fill = model_fill(&store, &k, FillSource::Replay, parse_json).unwrap();
        assert!(fill.target.is_some());
        assert!(fill.diagnostics.is_empty());
        assert_eq!(fill.recorded_calls, 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // Output that does not parse fails acceptance: no target, one
    // `ai_schema_violation`, and the recorded call is still accounted.
    #[test]
    fn unparsable_output_emits_ai_schema_violation() {
        let (store, dir) = temp_store("unparsable");
        let k = key();
        seed_cassette(&store, &k, b"not json");
        let fill = model_fill(&store, &k, FillSource::Replay, parse_json).unwrap();
        assert!(fill.target.is_none());
        assert_eq!(fill.diagnostics.len(), 1);
        assert_eq!(fill.diagnostics[0].code, DiagnosticCode::AiSchemaViolation);
        assert_eq!(fill.diagnostics[0].outcome, Outcome::Invalid);
        assert_eq!(fill.recorded_calls, 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // Output that parses but misses the structural check also violates the
    // schema (acceptance is the parser's whole job, not just well-formedness).
    #[test]
    fn schema_miss_emits_ai_schema_violation() {
        let (store, dir) = temp_store("schemamiss");
        let k = key();
        seed_cassette(&store, &k, br#"{"other":1}"#);
        let fill = model_fill(&store, &k, FillSource::Replay, parse_json).unwrap();
        assert!(fill.target.is_none());
        assert_eq!(fill.diagnostics[0].code, DiagnosticCode::AiSchemaViolation);
        assert_eq!(fill.recorded_calls, 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // A missing recording is a store error, not a schema violation — the two
    // failure kinds stay distinct (the route can tell a broken cassette from a
    // bad model output).
    #[test]
    fn missing_cassette_is_error_not_violation() {
        let (store, dir) = temp_store("missing");
        let err = model_fill(&store, &key(), FillSource::Replay, parse_json).unwrap_err();
        assert!(matches!(err, CassetteError::Io(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // The recorded-call count plus RECORDED_CALLS_COUNTER form the §4.6 event
    // resource counter the route/run wiring emits.
    #[test]
    fn recorded_call_count_keys_the_event_counter() {
        let (store, dir) = temp_store("counter");
        let k = key();
        seed_cassette(&store, &k, br#"{"ok":true}"#);
        let fill = model_fill(&store, &k, FillSource::Replay, parse_json).unwrap();
        let counter = (static_id(RECORDED_CALLS_COUNTER), fill.recorded_calls);
        assert_eq!(counter, (static_id("recorded_calls"), 1));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
