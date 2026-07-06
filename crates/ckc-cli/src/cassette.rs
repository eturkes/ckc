//! SPEC §9 model cassette store — record/replay of model I/O as committed test
//! source artifacts, keyed by (route, source, seed).
//!
//! Replay (default) reads a committed [`CassettePayload`] wrapper from disk and never
//! touches the model runtime → deterministic and runtime-absent. Record (gated behind
//! an explicit `--record`/experiment flag by the caller) invokes [`ModelAdapter`] once
//! and writes the recording. The recorded bytes ARE the determinism (greedy decode is
//! byte-stable on a fixed runtime but not across environments, §9), so CI and other
//! hosts replay the committed cassette rather than re-invoking the runtime.
//!
//! This is the store/IO layer; the §4.4 payload type lives in `ckc-core`. The cassette
//! root is caller-selected; committed test cassettes live under
//! `crates/ckc-cli/tests/fixtures/cassettes/<route>/<source>/seed-<seed>.json` (read via
//! the filesystem, not the run shell, which writes only run-output artifacts).

use std::path::{Path, PathBuf};
use std::time::Duration;

use ckc_core::{
    ArtifactWrapper, CanonError, CassettePayload, Effect, EvidenceStatus, Hash, Id, Origin,
    Producer, WrapperError, canonical_payload_bytes, canonicalization_policy_hash, content_hash,
    hash_bytes, read_strict_canonical,
};

use crate::model::{ModelAdapter, ModelOutcome};
use crate::shell::static_id;

/// Whether the store records live or replays committed cassettes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordMode {
    /// Invoke the runtime live and write the recording (gated; runtime required).
    Record,
    /// Read the committed recording; never invoke the runtime (default).
    Replay,
}

/// Identifies one recorded model call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CassetteKey {
    /// Route whose prompt template + constraint produced the call.
    pub route: Id,
    /// Test source the prompt was grounded on.
    pub source: Id,
    /// Decoding seed for the sample.
    pub seed: u64,
}

/// Provenance + budget for a live recording.
#[derive(Debug)]
pub struct RecordContext {
    /// Producer stamped into the cassette wrapper.
    pub producer: Producer,
    /// Raw-byte hash of the route prompt template the prompt was rendered from.
    pub prompt_template_hash: Hash,
    /// Wall-clock budget for the model invocation.
    pub budget: Duration,
}

/// Reads and writes cassettes under a caller-selected root (e.g. a test's `tests/fixtures`).
#[derive(Debug, Clone)]
pub struct CassetteStore {
    root: PathBuf,
}

impl CassetteStore {
    /// A store rooted at `root` (cassettes live under `root/cassettes/...`).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Path of the cassette for `key`, relative to the store root. Route and
    /// source are grammatical [`Id`]s — `[a-z][a-z0-9_.:-]*` excludes `/` and is
    /// never `.`/`..`, so each is one non-escaping path component (a `:` keeps
    /// this Unix-only, where the store runs).
    fn relative_path(key: &CassetteKey) -> PathBuf {
        PathBuf::from("cassettes")
            .join(key.route.as_str())
            .join(key.source.as_str())
            .join(format!("seed-{}.json", key.seed))
    }

    /// Absolute path of the cassette for `key`.
    pub fn path_for(&self, key: &CassetteKey) -> PathBuf {
        self.root.join(Self::relative_path(key))
    }

    /// Replay a committed cassette without invoking the runtime.
    pub fn replay(
        &self,
        key: &CassetteKey,
    ) -> Result<ArtifactWrapper<CassettePayload>, CassetteError> {
        self.load(key)
    }

    /// Record a live model call and write the cassette, returning the disk-read-back
    /// wrapper. Requires the runtime (via `adapter`).
    ///
    /// `constraint_hash` seals the exact constraint bytes this recorder reads; a
    /// re-read after the call confirms the file held across it (else
    /// [`CassetteError::ConstraintDrift`]), so the sealed hash attests the bytes the
    /// runtime actually decoded against.
    pub fn record(
        &self,
        adapter: &ModelAdapter,
        key: &CassetteKey,
        prompt: &str,
        constraint: &Path,
        ctx: &RecordContext,
    ) -> Result<ArtifactWrapper<CassettePayload>, CassetteError> {
        let constraint_bytes = std::fs::read(constraint).map_err(CassetteError::Io)?;
        let constraint_hash = hash_bytes(&constraint_bytes);

        let run = adapter.invoke(prompt, constraint, key.seed, ctx.budget);
        let output = match run.outcome {
            ModelOutcome::Completed { bytes } => bytes,
            other => return Err(CassetteError::Incomplete(format!("{other:?}"))),
        };
        // The runtime re-opened `constraint` by path; confirm the bytes held across
        // the call so `constraint_hash` attests exactly what the model decoded
        // against. Re-read rather than relocate the file — a snapshot copy would
        // break a constraint's relative `$ref`s.
        if std::fs::read(constraint).map_err(CassetteError::Io)? != constraint_bytes {
            return Err(CassetteError::ConstraintDrift);
        }

        let payload = CassettePayload::from_output(
            key.route.clone(),
            key.source.clone(),
            key.seed,
            prompt.to_owned(),
            constraint_hash,
            ctx.prompt_template_hash.clone(),
            adapter.identity().clone(),
            &output,
        );
        let wrapper = self.build_wrapper(key, payload, ctx.producer.clone())?;
        self.persist(key, wrapper)
    }

    /// The cassette's derived §4.4 `artifact_id` for `key`. The (route, source,
    /// seed) triple is the identity — realized collision-free by the on-disk path
    /// and the payload's own fields, which `load` anchors on. This id is a derived
    /// human label, not a uniqueness key: dotted route/source ids run together
    /// (`a.b`+`c` and `a`+`b.c` share a label), so downstream keys on the
    /// triple/path, never on this string.
    fn artifact_id(key: &CassetteKey) -> Result<Id, CassetteError> {
        format!(
            "model_cassette.{}.{}.seed-{}",
            key.route.as_str(),
            key.source.as_str(),
            key.seed
        )
        .parse::<Id>()
        .map_err(|_| CassetteError::DerivedId)
    }

    /// Wrap a payload as the §4.4 cassette artifact (origin `ai_generated`, evidence
    /// `evidence_discovery_only`, effect `ai`) and confirm it validates.
    pub(crate) fn build_wrapper(
        &self,
        key: &CassetteKey,
        payload: CassettePayload,
        producer: Producer,
    ) -> Result<ArtifactWrapper<CassettePayload>, CassetteError> {
        let artifact_id = Self::artifact_id(key)?;
        let content_hash = content_hash(&payload).map_err(CassetteError::Emit)?;
        let wrapper = ArtifactWrapper {
            schema_id: static_id("schema.model_cassette"),
            artifact_id,
            artifact_kind: static_id("model_cassette"),
            producer,
            input_hashes: vec![],
            content_hash,
            canonicalization_policy_hash: canonicalization_policy_hash(),
            origin: Origin::AiGenerated,
            evidence_status: EvidenceStatus::EvidenceDiscoveryOnly,
            external_effects: vec![Effect::Ai],
            trace_refs: vec![],
            diagnostics: vec![],
            runtime_metadata: vec![],
            payload,
        };
        wrapper.validate().map_err(CassetteError::Wrapper)?;
        Ok(wrapper)
    }

    /// Write the wrapper, then return the disk-read-back value (disk truth, mirroring
    /// the run-shell `land` discipline).
    pub(crate) fn persist(
        &self,
        key: &CassetteKey,
        wrapper: ArtifactWrapper<CassettePayload>,
    ) -> Result<ArtifactWrapper<CassettePayload>, CassetteError> {
        let bytes = canonical_payload_bytes(&wrapper).map_err(CassetteError::Emit)?;
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(CassetteError::Io)?;
        }
        std::fs::write(&path, &bytes).map_err(CassetteError::Io)?;
        self.load(key)
    }

    /// Enforce the fixed §4.4 cassette metadata `build_wrapper` stamps. `validate`
    /// covers only the two hash fields and the effect/evidence rule, so a committed
    /// cassette that lies about its schema/kind/id/provenance would otherwise load
    /// clean and mislead a downstream consumer — reject it here instead, naming the
    /// off-contract field.
    fn check_contract(
        key: &CassetteKey,
        wrapper: &ArtifactWrapper<CassettePayload>,
    ) -> Result<(), CassetteError> {
        let off = if wrapper.schema_id != static_id("schema.model_cassette") {
            Some("schema_id")
        } else if wrapper.artifact_kind != static_id("model_cassette") {
            Some("artifact_kind")
        } else if wrapper.artifact_id != Self::artifact_id(key)? {
            Some("artifact_id")
        } else if wrapper.origin != Origin::AiGenerated {
            Some("origin")
        } else if wrapper.evidence_status != EvidenceStatus::EvidenceDiscoveryOnly {
            Some("evidence_status")
        } else if wrapper.external_effects.len() != 1 || wrapper.external_effects[0] != Effect::Ai {
            Some("external_effects")
        } else if !wrapper.input_hashes.is_empty() {
            Some("input_hashes")
        } else {
            None
        };
        match off {
            Some(field) => Err(CassetteError::Contract(field)),
            None => Ok(()),
        }
    }

    /// Read, parse, validate, key-check, contract-check (the fixed §4.4 cassette
    /// provenance fields), and confirm the payload's `output_hex` decodes — a
    /// committed cassette that fails any check is rejected here, not deferred to
    /// the point of use.
    fn load(&self, key: &CassetteKey) -> Result<ArtifactWrapper<CassettePayload>, CassetteError> {
        let bytes = std::fs::read(self.path_for(key)).map_err(CassetteError::Io)?;
        let wrapper: ArtifactWrapper<CassettePayload> =
            read_strict_canonical(&bytes).map_err(|e| CassetteError::Canon(format!("{e}")))?;
        wrapper.validate().map_err(CassetteError::Wrapper)?;
        if wrapper.payload.route != key.route
            || wrapper.payload.source != key.source
            || wrapper.payload.seed != key.seed
        {
            return Err(CassetteError::KeyMismatch);
        }
        Self::check_contract(key, &wrapper)?;
        wrapper
            .payload
            .output_bytes()
            .map_err(|_| CassetteError::InvalidHex)?;
        Ok(wrapper)
    }
}

/// Failure recording or replaying a cassette.
#[derive(Debug)]
pub enum CassetteError {
    /// Filesystem read/write failed (a missing cassette surfaces here on replay).
    Io(std::io::Error),
    /// Committed cassette bytes are not strict canonical JSON for the wrapper.
    Canon(String),
    /// Canonical emission of the wrapper failed.
    Emit(CanonError),
    /// The wrapper failed §4.4 validation.
    Wrapper(WrapperError),
    /// The live recording did not return a clean `Completed` output.
    Incomplete(String),
    /// The constraint file changed between the pre-read and the runtime call, so the
    /// sealed `constraint_hash` would not attest the bytes the model read.
    ConstraintDrift,
    /// The cassette at the keyed path records a different (route, source, seed).
    KeyMismatch,
    /// A committed cassette violates the fixed §4.4 cassette contract; the field
    /// names the off-contract wrapper member (`origin`, `schema_id`, …).
    Contract(&'static str),
    /// The cassette payload's `output_hex` is not decodable lowercase hex.
    InvalidHex,
    /// The derived cassette artifact id is not a grammatical [`Id`] (defensive).
    DerivedId,
}

impl std::fmt::Display for CassetteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CassetteError::Io(e) => write!(f, "cassette io: {e}"),
            CassetteError::Canon(e) => write!(f, "cassette parse: {e}"),
            CassetteError::Emit(e) => write!(f, "cassette emit: {e}"),
            CassetteError::Wrapper(e) => write!(f, "cassette wrapper invalid: {e}"),
            CassetteError::Incomplete(o) => write!(f, "recording incomplete: {o}"),
            CassetteError::ConstraintDrift => {
                f.write_str("constraint file changed during recording")
            }
            CassetteError::KeyMismatch => f.write_str("cassette key mismatch at path"),
            CassetteError::Contract(field) => write!(f, "cassette off-contract: {field}"),
            CassetteError::InvalidHex => f.write_str("cassette output_hex is not decodable hex"),
            CassetteError::DerivedId => f.write_str("derived cassette id is not grammatical"),
        }
    }
}

impl std::error::Error for CassetteError {}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::ModelIdentity;

    fn producer() -> Producer {
        Producer {
            pipeline_id: static_id("pipe.test"),
            pipeline_step_id: static_id("processing_stage.test.model_fill"),
            toolchain_manifest_hash: hash_bytes(b"toolchain"),
        }
    }

    fn key() -> CassetteKey {
        CassetteKey {
            route: "route.fixture".parse().unwrap(),
            source: "test_source.fixture".parse().unwrap(),
            seed: 42,
        }
    }

    fn payload_for(k: &CassetteKey, output: &[u8]) -> CassettePayload {
        CassettePayload::from_output(
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
        )
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ckc-cassette-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    // Runtime-absent: persist a synthetic cassette, then replay it byte-stably.
    #[test]
    fn replay_round_trips_without_runtime() {
        let dir = temp_dir("rt");
        let store = CassetteStore::new(&dir);
        let k = key();
        let wrapper = store
            .build_wrapper(&k, payload_for(&k, &[0x00, 0xff, 0x7b, 0x0a]), producer())
            .unwrap();
        let written = store.persist(&k, wrapper.clone()).unwrap();
        let replayed = store.replay(&k).unwrap();
        assert_eq!(written, replayed);
        assert_eq!(replayed.content_hash, wrapper.content_hash);
        assert_eq!(
            replayed.payload.output_bytes().unwrap(),
            [0x00, 0xff, 0x7b, 0x0a]
        );
        assert_eq!(replayed.origin, Origin::AiGenerated);
        assert_eq!(
            replayed.evidence_status,
            EvidenceStatus::EvidenceDiscoveryOnly
        );
        assert_eq!(replayed.external_effects, vec![Effect::Ai]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_cassette_is_io_error() {
        let store = CassetteStore::new(temp_dir("missing"));
        assert!(matches!(store.replay(&key()), Err(CassetteError::Io(_))));
    }

    // A cassette filed at the wrong key path is rejected. (`persist` itself
    // key-checks on read-back, so the bytes are written directly to bypass it.)
    #[test]
    fn key_mismatch_rejected() {
        let dir = temp_dir("mismatch");
        let store = CassetteStore::new(&dir);
        let filed = key();
        let other = CassetteKey {
            source: "test_source.other".parse().unwrap(),
            ..filed.clone()
        };
        let wrapper = store
            .build_wrapper(&other, payload_for(&other, b"x"), producer())
            .unwrap();
        let bytes = canonical_payload_bytes(&wrapper).unwrap();
        let path = store.path_for(&filed);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            store.replay(&filed),
            Err(CassetteError::KeyMismatch)
        ));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // A cassette whose `output_hex` is not decodable hex is rejected at load,
    // not silently deferred to `output_bytes()` at the point of use. (Bytes are
    // written directly so the malformed payload bypasses `persist`'s read-back.)
    #[test]
    fn malformed_hex_rejected() {
        let dir = temp_dir("malformed");
        let store = CassetteStore::new(&dir);
        let k = key();
        let mut payload = payload_for(&k, b"x");
        payload.output_hex = "zz".to_owned();
        let wrapper = store.build_wrapper(&k, payload, producer()).unwrap();
        let bytes = canonical_payload_bytes(&wrapper).unwrap();
        let path = store.path_for(&k);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(matches!(store.replay(&k), Err(CassetteError::InvalidHex)));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // A committed cassette that lies about its §4.4 provenance (here `origin`,
    // though it stays internally consistent → `validate` passes) is rejected at
    // load by the contract check, not trusted downstream.
    #[test]
    fn off_contract_rejected() {
        let dir = temp_dir("contract");
        let store = CassetteStore::new(&dir);
        let k = key();
        let mut wrapper = store
            .build_wrapper(&k, payload_for(&k, b"x"), producer())
            .unwrap();
        wrapper.origin = Origin::DeterministicCompiler; // not a recording; content_hash unaffected
        wrapper.validate().unwrap(); // still internally consistent — the gap validate misses
        let bytes = canonical_payload_bytes(&wrapper).unwrap();
        let path = store.path_for(&k);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            store.replay(&k),
            Err(CassetteError::Contract("origin"))
        ));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
