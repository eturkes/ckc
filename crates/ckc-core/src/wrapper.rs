//! SPEC §4.4 artifact wrapper and SPEC §4.6 event records, plus the JSONL
//! stream form behind `events.jsonl` and `diagnostics.jsonl`.
//!
//! [`ArtifactWrapper`] is the one canonical JSON wrapper every accepted
//! artifact ships in; [`validate`](ArtifactWrapper::validate) enforces the
//! §4.4 invariants that are mechanical at this layer. [`EventRecord`] is one
//! `events.jsonl` line of runtime evidence. [`write_jsonl`]/[`read_jsonl`]
//! frame any canonical type as newline-delimited canonical JSON, strictly in
//! both directions.

use std::fmt;

use crate::canon::{
    CanonError, CanonRead, CanonReadError, Canonical, ObjectEmitter, ObjectReader, Reader,
    canonical_payload_bytes, emit_raw_map, emit_set, emit_string, emit_u64, emit_u64_map,
    read_raw_map, read_set, read_strict_canonical, read_string, read_u64, read_u64_map,
};
use crate::enums::{DiagnosticRecord, EvidenceStatus, Origin, Outcome, fieldless_enum};
use crate::hash::{canonicalization_policy_hash, content_hash};
use crate::id::{Hash, Id, ValidationError};

/// SPEC §4.4 `schema_version`: bumped on breaking schema change. The wrapper
/// emits this constant and reading requires it, so a foreign version never
/// constructs a value.
pub const SCHEMA_VERSION: &str = "ckc.1";

fieldless_enum! {
    /// SPEC §4.4 `external_effects` value: an effect channel an
    /// evidence-discovery artifact may record. Accepted semantic artifacts
    /// carry the empty set ([`ArtifactWrapper::validate`]).
    Effect {
        Network => "network",
        Clock => "clock",
        Ai => "ai",
        Tool => "tool",
    }
}

/// SPEC §4.4 `producer`: the registered pipeline step that emitted the artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Producer {
    pub pipeline_id: Id,
    pub pipeline_step_id: Id,
    /// Hash of the toolchain manifest in force; this schema declares raw-byte
    /// hashing (§4.4 `_hash` rule) — the manifest is a file, not an accepted
    /// artifact.
    pub toolchain_manifest_hash: Hash,
}

impl Canonical for Producer {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("pipeline_id", |b| self.pipeline_id.emit_canonical(b))?;
        obj.member("pipeline_step_id", |b| {
            self.pipeline_step_id.emit_canonical(b)
        })?;
        obj.member("toolchain_manifest_hash", |b| {
            self.toolchain_manifest_hash.emit_canonical(b)
        })?;
        obj.finish(out)
    }
}

impl CanonRead for Producer {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let pipeline_id = obj.member("pipeline_id", Id::read)?;
        let pipeline_step_id = obj.member("pipeline_step_id", Id::read)?;
        let toolchain_manifest_hash = obj.member("toolchain_manifest_hash", Hash::read)?;
        obj.close()?;
        Ok(Producer {
            pipeline_id,
            pipeline_step_id,
            toolchain_manifest_hash,
        })
    }
}

/// A SPEC §4.4 wrapper invariant failed ([`ArtifactWrapper::validate`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrapperError {
    /// `external_effects` was nonempty under an evidence_status other than
    /// `evidence_discovery_only` (carried here).
    EffectsForbidden(EvidenceStatus),
    /// The `content_hash` field is not the hash of the payload's canonical
    /// bytes.
    ContentHash { declared: Hash, computed: Hash },
    /// The `canonicalization_policy_hash` field is not the in-force §4.3
    /// policy descriptor's hash.
    PolicyHash { declared: Hash, computed: Hash },
    /// The payload's own canonical emission failed while recomputing hashes.
    Canon(CanonError),
}

impl fmt::Display for WrapperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WrapperError::EffectsForbidden(evidence_status) => write!(
                f,
                "external_effects requires evidence_discovery_only evidence_status, found {}",
                evidence_status.as_str()
            ),
            WrapperError::ContentHash { declared, computed } => write!(
                f,
                "content_hash {} does not match payload bytes ({})",
                declared.as_str(),
                computed.as_str()
            ),
            WrapperError::PolicyHash { declared, computed } => write!(
                f,
                "canonicalization_policy_hash {} is not the in-force descriptor {}",
                declared.as_str(),
                computed.as_str()
            ),
            WrapperError::Canon(e) => write!(f, "payload emission: {e}"),
        }
    }
}

impl std::error::Error for WrapperError {}

/// SPEC §4.4 wrapper: the one canonical JSON wrapper every accepted artifact
/// ships in, generic over the typed payload `P`. Builders fill the struct,
/// computing the two derived fields with [`content_hash`] and
/// [`canonicalization_policy_hash`], then confirm with
/// [`validate`](Self::validate); `schema_version` is the [`SCHEMA_VERSION`]
/// constant, emitted and required on read rather than stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactWrapper<P> {
    /// Schema identifier, e.g. `schema.ir_bundle`.
    pub schema_id: Id,
    pub artifact_id: Id,
    pub artifact_kind: Id,
    pub producer: Producer,
    /// Content hashes of the accepted artifacts this one consumed.
    pub input_hashes: Vec<Hash>,
    /// Hash of the canonical payload bytes — and nothing else: wrapper
    /// metadata never shifts content identity.
    pub content_hash: Hash,
    /// Hash of the §4.3 canonicalization-policy descriptor in force.
    pub canonicalization_policy_hash: Hash,
    pub origin: Origin,
    /// `compiler_evidence_status` is reserved for compiled artifacts and
    /// `verifier_evidence_status` for verifier results; those schemas bind the
    /// reservation where they land (M1 compile/verify processing_stages).
    pub evidence_status: EvidenceStatus,
    /// `[]` for accepted semantic artifacts; evidence-discovery artifacts may
    /// record the channels they used. Set semantics.
    pub external_effects: Vec<Effect>,
    /// Trace links (§7.1 entities). Set semantics.
    pub trace_refs: Vec<Id>,
    /// Structured §7.4 diagnostics. Set semantics.
    pub diagnostics: Vec<DiagnosticRecord>,
    /// Runtime evidence (run ids, timings): identifier keys to raw text,
    /// excluded from `content_hash` by construction (§4.4, §4.6).
    pub runtime_metadata: Vec<(Id, String)>,
    /// The typed content.
    pub payload: P,
}

impl<P: Canonical> ArtifactWrapper<P> {
    /// Enforce the §4.4 invariants that are mechanical at the wrapper layer:
    /// both derived hash fields match recomputation, and a nonempty
    /// `external_effects` is confined to `evidence_discovery_only` evidence_status.
    pub fn validate(&self) -> Result<(), WrapperError> {
        if !self.external_effects.is_empty()
            && self.evidence_status != EvidenceStatus::EvidenceDiscoveryOnly
        {
            return Err(WrapperError::EffectsForbidden(self.evidence_status));
        }
        let computed = content_hash(&self.payload).map_err(WrapperError::Canon)?;
        if self.content_hash != computed {
            return Err(WrapperError::ContentHash {
                declared: self.content_hash.clone(),
                computed,
            });
        }
        let policy = canonicalization_policy_hash();
        if self.canonicalization_policy_hash != policy {
            return Err(WrapperError::PolicyHash {
                declared: self.canonicalization_policy_hash.clone(),
                computed: policy,
            });
        }
        Ok(())
    }
}

impl<P: Canonical> Canonical for ArtifactWrapper<P> {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("artifact_id", |b| self.artifact_id.emit_canonical(b))?;
        obj.member("artifact_kind", |b| self.artifact_kind.emit_canonical(b))?;
        obj.member("canonicalization_policy_hash", |b| {
            self.canonicalization_policy_hash.emit_canonical(b)
        })?;
        obj.member("content_hash", |b| self.content_hash.emit_canonical(b))?;
        obj.member("diagnostics", |b| emit_set(b, &self.diagnostics))?;
        obj.member("evidence_status", |b| {
            self.evidence_status.emit_canonical(b)
        })?;
        obj.member("external_effects", |b| emit_set(b, &self.external_effects))?;
        obj.member("input_hashes", |b| emit_set(b, &self.input_hashes))?;
        obj.member("origin", |b| self.origin.emit_canonical(b))?;
        obj.member("payload", |b| self.payload.emit_canonical(b))?;
        obj.member("producer", |b| self.producer.emit_canonical(b))?;
        obj.member("runtime_metadata", |b| {
            emit_raw_map(b, &self.runtime_metadata)
        })?;
        obj.member("schema_id", |b| self.schema_id.emit_canonical(b))?;
        obj.member("schema_version", |b| {
            emit_string(b, SCHEMA_VERSION);
            Ok(())
        })?;
        obj.member("trace_refs", |b| emit_set(b, &self.trace_refs))?;
        obj.finish(out)
    }
}

impl<P: CanonRead> CanonRead for ArtifactWrapper<P> {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let artifact_id = obj.member("artifact_id", Id::read)?;
        let artifact_kind = obj.member("artifact_kind", Id::read)?;
        let canonicalization_policy_hash =
            obj.member("canonicalization_policy_hash", Hash::read)?;
        let content_hash = obj.member("content_hash", Hash::read)?;
        let diagnostics = obj.member("diagnostics", read_set::<DiagnosticRecord>)?;
        let evidence_status = obj.member("evidence_status", EvidenceStatus::read)?;
        let external_effects = obj.member("external_effects", read_set::<Effect>)?;
        let input_hashes = obj.member("input_hashes", read_set::<Hash>)?;
        let origin = obj.member("origin", Origin::read)?;
        let payload = obj.member("payload", P::read)?;
        let producer = obj.member("producer", Producer::read)?;
        let runtime_metadata = obj.member("runtime_metadata", read_raw_map)?;
        let schema_id = obj.member("schema_id", Id::read)?;
        obj.member("schema_version", |r| {
            let version = read_string(r)?;
            if version != SCHEMA_VERSION {
                return Err(CanonReadError::Policy(ValidationError::Enum(format!(
                    "schema_version must be {SCHEMA_VERSION:?}, got {version:?}"
                ))));
            }
            Ok(())
        })?;
        let trace_refs = obj.member("trace_refs", read_set::<Id>)?;
        obj.close()?;
        Ok(ArtifactWrapper {
            schema_id,
            artifact_id,
            artifact_kind,
            producer,
            input_hashes,
            content_hash,
            canonicalization_policy_hash,
            origin,
            evidence_status,
            external_effects,
            trace_refs,
            diagnostics,
            runtime_metadata,
            payload,
        })
    }
}

/// SPEC §4.6 event: one `events.jsonl` line. Events are runtime evidence —
/// accepted semantics live only in validated artifacts — so run ids and
/// wall-clock fields ride here and stay out of content hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    pub event_id: Id,
    pub run_id: Id,
    pub pipeline_id: Id,
    pub pipeline_step_id: Id,
    /// Pipeline processing_stage the event belongs to (e.g. `extract`).
    pub processing_stage: Id,
    /// Log-level (log_level) token (e.g. `info`); §4.6 leaves the value set open at M1.
    pub log_level: Id,
    /// Deterministic event ordering within a run.
    pub event_sequence_number: u64,
    /// Wall-clock bounds as raw runtime text (e.g. RFC 3339).
    pub started_at: String,
    pub ended_at: String,
    pub duration_ms: u64,
    pub input_hashes: Vec<Hash>,
    pub output_hashes: Vec<Hash>,
    pub outcome: Outcome,
    /// §7.4 diagnostics raised during the event. Set semantics.
    pub diagnostics: Vec<DiagnosticRecord>,
    /// Counter name to consumed amount (tokens, calls, milliseconds).
    pub resource_counters: Vec<(Id, u64)>,
}

impl Canonical for EventRecord {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("diagnostics", |b| emit_set(b, &self.diagnostics))?;
        obj.member("duration_ms", |b| {
            emit_u64(b, self.duration_ms);
            Ok(())
        })?;
        obj.member("ended_at", |b| {
            emit_string(b, &self.ended_at);
            Ok(())
        })?;
        obj.member("event_id", |b| self.event_id.emit_canonical(b))?;
        obj.member("event_sequence_number", |b| {
            emit_u64(b, self.event_sequence_number);
            Ok(())
        })?;
        obj.member("input_hashes", |b| emit_set(b, &self.input_hashes))?;
        obj.member("log_level", |b| self.log_level.emit_canonical(b))?;
        obj.member("outcome", |b| self.outcome.emit_canonical(b))?;
        obj.member("output_hashes", |b| emit_set(b, &self.output_hashes))?;
        obj.member("pipeline_id", |b| self.pipeline_id.emit_canonical(b))?;
        obj.member("pipeline_step_id", |b| {
            self.pipeline_step_id.emit_canonical(b)
        })?;
        obj.member("processing_stage", |b| {
            self.processing_stage.emit_canonical(b)
        })?;
        obj.member("resource_counters", |b| {
            emit_u64_map(b, &self.resource_counters)
        })?;
        obj.member("run_id", |b| self.run_id.emit_canonical(b))?;
        obj.member("started_at", |b| {
            emit_string(b, &self.started_at);
            Ok(())
        })?;
        obj.finish(out)
    }
}

impl CanonRead for EventRecord {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let diagnostics = obj.member("diagnostics", read_set::<DiagnosticRecord>)?;
        let duration_ms = obj.member("duration_ms", read_u64)?;
        let ended_at = obj.member("ended_at", read_string)?;
        let event_id = obj.member("event_id", Id::read)?;
        let event_sequence_number = obj.member("event_sequence_number", read_u64)?;
        let input_hashes = obj.member("input_hashes", read_set::<Hash>)?;
        let log_level = obj.member("log_level", Id::read)?;
        let outcome = obj.member("outcome", Outcome::read)?;
        let output_hashes = obj.member("output_hashes", read_set::<Hash>)?;
        let pipeline_id = obj.member("pipeline_id", Id::read)?;
        let pipeline_step_id = obj.member("pipeline_step_id", Id::read)?;
        let processing_stage = obj.member("processing_stage", Id::read)?;
        let resource_counters = obj.member("resource_counters", read_u64_map)?;
        let run_id = obj.member("run_id", Id::read)?;
        let started_at = obj.member("started_at", read_string)?;
        obj.close()?;
        Ok(EventRecord {
            event_id,
            run_id,
            pipeline_id,
            pipeline_step_id,
            processing_stage,
            log_level,
            event_sequence_number,
            started_at,
            ended_at,
            duration_ms,
            input_hashes,
            output_hashes,
            outcome,
            diagnostics,
            resource_counters,
        })
    }
}

/// One canonical JSONL line: the value's §4.3 canonical bytes plus the
/// terminating newline. Canonical strings escape every control byte, so the
/// payload bytes never contain a raw newline and line framing is unambiguous.
pub fn jsonl_line<T: Canonical>(value: &T) -> Result<Vec<u8>, CanonError> {
    let mut line = canonical_payload_bytes(value)?;
    line.push(b'\n');
    Ok(line)
}

/// Serialize a record stream (`events.jsonl`, `diagnostics.jsonl`) as
/// canonical JSONL: one [`jsonl_line`] per record, order preserved (streams
/// are append-ordered runtime evidence, not sets).
pub fn write_jsonl<'a, T: Canonical + 'a>(
    records: impl IntoIterator<Item = &'a T>,
) -> Result<Vec<u8>, CanonError> {
    let mut out = Vec::new();
    for record in records {
        out.extend_from_slice(&jsonl_line(record)?);
    }
    Ok(out)
}

/// Strict inverse of [`write_jsonl`]: every line is exactly one canonical
/// record and every line ends in `\n` — an unterminated tail reads as a
/// truncated stream ([`CanonReadError::Eof`]). Empty input is the empty
/// stream.
pub fn read_jsonl<T: CanonRead>(bytes: &[u8]) -> Result<Vec<T>, CanonReadError> {
    let Some(body) = bytes.strip_suffix(b"\n") else {
        return if bytes.is_empty() {
            Ok(Vec::new())
        } else {
            Err(CanonReadError::Eof)
        };
    };
    body.split(|&b| b == b'\n')
        .map(read_strict_canonical)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::DiagnosticCode;

    /// Canonical bytes of `value` as a UTF-8 string, for exact-match assertions.
    fn canon<T: Canonical>(value: &T) -> String {
        String::from_utf8(canonical_payload_bytes(value).unwrap()).unwrap()
    }

    /// Assert `value` survives a canonical write -> read round trip unchanged.
    fn round_trip<T: Canonical + CanonRead + std::fmt::Debug + PartialEq>(value: T) {
        let bytes = canonical_payload_bytes(&value).unwrap();
        let got: T = read_strict_canonical(&bytes).unwrap();
        assert_eq!(got, value, "round trip changed the value");
    }

    /// A valid [`Hash`] built from one repeated hex digit.
    fn h(digit: char) -> Hash {
        Hash::new(format!("sha256:{}", digit.to_string().repeat(64))).unwrap()
    }

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn sample_diagnostic() -> DiagnosticRecord {
        DiagnosticRecord {
            code: DiagnosticCode::ExtractionUncertain,
            outcome: Outcome::Residual,
            payload: vec![(id("node"), "p1".to_owned())],
            region_ids: vec![id("region.a")],
            artifact_hashes: vec![],
        }
    }

    fn sample_wrapper() -> ArtifactWrapper<Id> {
        let payload = id("payload.v");
        ArtifactWrapper {
            schema_id: id("schema.test"),
            artifact_id: id("artifact.a"),
            artifact_kind: id("test_kind"),
            producer: Producer {
                pipeline_id: id("pipe.base"),
                pipeline_step_id: id("processing_stage.base.unit"),
                toolchain_manifest_hash: h('b'),
            },
            input_hashes: vec![h('a')],
            content_hash: content_hash(&payload).unwrap(),
            canonicalization_policy_hash: canonicalization_policy_hash(),
            origin: Origin::DeterministicCompiler,
            evidence_status: EvidenceStatus::MechanicalEvidenceStatus,
            external_effects: vec![],
            trace_refs: vec![id("trace.t1")],
            diagnostics: vec![],
            runtime_metadata: vec![(id("run_id"), "run.1".to_owned())],
            payload,
        }
    }

    // Pins the §4.4 external_effects value set: spelling, round-trips, and
    // canonical set order (byte-lexicographic, independent of declaration).
    #[test]
    fn effect_spellings_and_set_order() {
        let spelled: Vec<&str> = Effect::ALL.iter().map(|&v| v.as_str()).collect();
        assert_eq!(spelled, ["network", "clock", "ai", "tool"]);
        for v in Effect::ALL {
            round_trip(*v);
        }
        let mut out = Vec::new();
        emit_set(&mut out, Effect::ALL).unwrap();
        assert_eq!(out, br#"["ai","clock","network","tool"]"#);
    }

    // Pins the wrapper's canonical shape: every §4.4 field, byte-sorted, with
    // the schema_version constant and object-form runtime_metadata.
    #[test]
    fn wrapper_canonical_bytes() {
        let env = sample_wrapper();
        let want = format!(
            concat!(
                r#"{{"artifact_id":"artifact.a","artifact_kind":"test_kind","#,
                r#""canonicalization_policy_hash":"{}","content_hash":"{}","#,
                r#""diagnostics":[],"evidence_status":"mechanical_evidence_status","#,
                r#""external_effects":[],"input_hashes":["{}"],"#,
                r#""origin":"deterministic_compiler","payload":"payload.v","#,
                r#""producer":{{"pipeline_id":"pipe.base","pipeline_step_id":"processing_stage.base.unit","#,
                r#""toolchain_manifest_hash":"{}"}},"runtime_metadata":{{"run_id":"run.1"}},"#,
                r#""schema_id":"schema.test","schema_version":"ckc.1","#,
                r#""trace_refs":["trace.t1"]}}"#
            ),
            canonicalization_policy_hash().as_str(),
            env.content_hash.as_str(),
            h('a').as_str(),
            h('b').as_str(),
        );
        assert_eq!(canon(&env), want);
    }

    #[test]
    fn wrapper_round_trips_fully_populated() {
        let mut env = sample_wrapper();
        env.evidence_status = EvidenceStatus::EvidenceDiscoveryOnly;
        env.external_effects = vec![Effect::Ai, Effect::Tool];
        env.diagnostics = vec![sample_diagnostic()];
        round_trip(env);
    }

    #[test]
    fn validate_accepts_consistent_wrappers() {
        sample_wrapper().validate().unwrap();
        // evidence-discovery artifacts may record effects
        let mut env = sample_wrapper();
        env.evidence_status = EvidenceStatus::EvidenceDiscoveryOnly;
        env.external_effects = vec![Effect::Network, Effect::Clock];
        env.validate().unwrap();
    }

    // §4.4: external_effects is [] for accepted semantic artifacts; only
    // evidence-discovery artifacts record effects.
    #[test]
    fn validate_rejects_effects_outside_evidence_discovery() {
        let mut env = sample_wrapper();
        env.external_effects = vec![Effect::Network];
        assert_eq!(
            env.validate(),
            Err(WrapperError::EffectsForbidden(
                EvidenceStatus::MechanicalEvidenceStatus
            ))
        );
    }

    #[test]
    fn validate_rejects_derived_hash_drift() {
        let mut env = sample_wrapper();
        env.content_hash = h('c');
        assert!(matches!(
            env.validate(),
            Err(WrapperError::ContentHash { .. })
        ));
        let mut env = sample_wrapper();
        env.canonicalization_policy_hash = h('d');
        assert!(matches!(
            env.validate(),
            Err(WrapperError::PolicyHash { .. })
        ));
    }

    // §4.4: runtime_metadata is excluded from content_hash — the wrapper
    // bytes change, the content identity does not.
    #[test]
    fn runtime_metadata_never_shifts_content_hash() {
        let a = sample_wrapper();
        let mut b = sample_wrapper();
        b.runtime_metadata = vec![(id("host"), "other".to_owned())];
        assert_ne!(
            canonical_payload_bytes(&a).unwrap(),
            canonical_payload_bytes(&b).unwrap()
        );
        assert_eq!(a.content_hash, b.content_hash);
        b.validate().unwrap();
    }

    #[test]
    fn reading_rejects_foreign_schema_version() {
        let text = canon(&sample_wrapper())
            .replace(r#""schema_version":"ckc.1""#, r#""schema_version":"ckc.2""#);
        assert!(matches!(
            read_strict_canonical::<ArtifactWrapper<Id>>(text.as_bytes()),
            Err(CanonReadError::Policy(ValidationError::Enum(_)))
        ));
    }

    fn sample_event() -> EventRecord {
        EventRecord {
            event_id: id("event.1"),
            run_id: id("run.20260610"),
            pipeline_id: id("pipe.base"),
            pipeline_step_id: id("processing_stage.base.extract"),
            processing_stage: id("extract"),
            log_level: id("info"),
            event_sequence_number: 3,
            started_at: "2026-06-10T06:30:00Z".to_owned(),
            ended_at: "2026-06-10T06:30:01Z".to_owned(),
            duration_ms: 1000,
            input_hashes: vec![h('a')],
            output_hashes: vec![h('b')],
            outcome: Outcome::Ok,
            diagnostics: vec![],
            resource_counters: vec![(id("tokens"), 42)],
        }
    }

    // Pins the §4.6 event field list in canonical order, with decimal-string
    // integers and raw wall-clock text.
    #[test]
    fn event_record_canonical_bytes() {
        let want = format!(
            concat!(
                r#"{{"diagnostics":[],"duration_ms":"1000","#,
                r#""ended_at":"2026-06-10T06:30:01Z","event_id":"event.1","#,
                r#""event_sequence_number":"3","input_hashes":["{}"],"log_level":"info","#,
                r#""outcome":"ok","output_hashes":["{}"],"pipeline_id":"pipe.base","#,
                r#""pipeline_step_id":"processing_stage.base.extract","processing_stage":"extract","#,
                r#""resource_counters":{{"tokens":"42"}},"run_id":"run.20260610","#,
                r#""started_at":"2026-06-10T06:30:00Z"}}"#
            ),
            h('a').as_str(),
            h('b').as_str(),
        );
        assert_eq!(canon(&sample_event()), want);
    }

    #[test]
    fn event_record_round_trips_and_bounds_counters() {
        round_trip(sample_event());
        let mut populated = sample_event();
        populated.diagnostics = vec![sample_diagnostic()];
        populated.resource_counters = vec![(id("calls"), 1), (id("tokens"), u64::MAX)];
        round_trip(populated);
        // negative integers are not u64 runtime counters
        let neg =
            canon(&sample_event()).replace(r#""duration_ms":"1000""#, r#""duration_ms":"-1""#);
        assert!(matches!(
            read_strict_canonical::<EventRecord>(neg.as_bytes()),
            Err(CanonReadError::Integer(_))
        ));
    }

    // JSONL covers both streams: events.jsonl and diagnostics.jsonl.
    #[test]
    fn jsonl_round_trips_event_and_diagnostic_streams() {
        let mut second = sample_event();
        second.event_id = id("event.2");
        second.event_sequence_number = 4;
        let events = vec![sample_event(), second];
        let bytes = write_jsonl(&events).unwrap();
        assert_eq!(bytes.iter().filter(|&&b| b == b'\n').count(), 2);
        assert!(bytes.ends_with(b"\n"));
        assert_eq!(read_jsonl::<EventRecord>(&bytes).unwrap(), events);

        let diagnostics = vec![sample_diagnostic()];
        let bytes = write_jsonl(&diagnostics).unwrap();
        assert_eq!(read_jsonl::<DiagnosticRecord>(&bytes).unwrap(), diagnostics);

        assert_eq!(read_jsonl::<EventRecord>(b"").unwrap(), Vec::new());
    }

    #[test]
    fn jsonl_rejects_unterminated_and_blank_lines() {
        let mut bytes = write_jsonl(&[sample_event()]).unwrap();
        bytes.pop(); // drop the line terminator
        assert_eq!(read_jsonl::<EventRecord>(&bytes), Err(CanonReadError::Eof));
        // a blank line is no record
        assert!(read_jsonl::<EventRecord>(b"\n").is_err());
    }
}
