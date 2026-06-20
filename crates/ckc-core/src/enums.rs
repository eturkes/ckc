//! SPEC §4.4 enum family and total operation result, SPEC §7.4 diagnostics.
//!
//! Every enum here is fieldless: its canonical form is the value's
//! identifier_ascii spelling as a bare JSON string (SPEC §4.3), and reading
//! accepts exactly the spelled value set. [`Outcome`] additionally orders its
//! variants by §4.4 severity so `max` aggregates processing_stage outcomes.
//! [`DiagnosticRecord`] is the §7.4 diagnostic (stable code, structured
//! payload, region/artifact refs, exactly one outcome); [`TotalOperationResult`]
//! is the §4.4 record every processing_stage and command returns exactly once.

use crate::canon::{
    CanonError, CanonRead, CanonReadError, Canonical, ObjectEmitter, ObjectReader, Reader,
    emit_map, emit_set, emit_string_policy, read_map, read_set, read_string_policy,
};
use crate::id::{Hash, Id};
use crate::strings::StringPolicy;

/// Define a fieldless SPEC §4.4 enum: variants in declaration order, an
/// [`ALL`](Outcome::ALL) table, the `as_str`/`parse` spelling pair, and
/// [`Canonical`]/[`CanonRead`] impls over the bare identifier_ascii string.
/// Serde impls reuse the same spelling, so YAML surfaces (the §8.4
/// registries) and canonical bytes agree on every value. `#[macro_export]` +
/// `$crate::` paths let sibling crates (ckc-smt) define their §4.4-style
/// value sets through the same generator.
#[macro_export]
macro_rules! fieldless_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $($(#[$vmeta:meta])* $variant:ident => $text:literal),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($(#[$vmeta])* $variant,)+
        }

        impl $name {
            /// Every value, in declaration order.
            pub const ALL: &[$name] = &[$($name::$variant),+];

            /// The SPEC §4.4 identifier_ascii spelling.
            pub fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $text,)+
                }
            }

            /// Inverse of [`as_str`](Self::as_str); any other token fails with
            /// [`ValidationError::Enum`].
            pub fn parse(token: &str) -> Result<Self, $crate::ValidationError> {
                match token {
                    $($text => Ok($name::$variant),)+
                    other => Err($crate::ValidationError::Enum(format!(
                        concat!(stringify!($name), " has no value {:?}"),
                        other
                    ))),
                }
            }
        }

        impl $crate::Canonical for $name {
            fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), $crate::CanonError> {
                $crate::emit_string(out, self.as_str());
                Ok(())
            }
        }

        impl $crate::CanonRead for $name {
            fn read(r: &mut $crate::Reader<'_>) -> Result<Self, $crate::CanonReadError> {
                Ok($name::parse(&$crate::read_string(r)?)?)
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let token: String = ::serde::Deserialize::deserialize(deserializer)?;
                $name::parse(&token).map_err(::serde::de::Error::custom)
            }
        }
    };
}

/// Crate-visible so sibling modules (wrapper) define their own §4.4-style
/// value sets through the same generator.
pub(crate) use fieldless_enum;

fieldless_enum! {
    /// SPEC §4.4 operation outcome. Variants are declared in ascending
    /// severity — `ok < residual < ambiguity < unsupported < incoherence <
    /// invalid` — so the derived order IS the §4.4 severity order and `max`
    /// aggregates per-item outcomes into a total one.
    #[derive(PartialOrd, Ord)]
    Outcome {
        /// Output valid for the declared processing_stage.
        Ok => "ok",
        /// Schema-valid but incomplete: permission-limited, missing evidence,
        /// missing policy, partial extraction.
        Residual => "residual",
        /// Multiple acceptable readings, bindings, spans, or normalizations
        /// remain.
        Ambiguity => "ambiguity",
        /// Schema-valid construction outside implemented semantics.
        Unsupported => "unsupported",
        /// Accepted harness inputs collide (e.g. incompatible policy rows);
        /// source-level conflicts between guideline rules are findings,
        /// delivered as values under `ok`.
        Incoherence => "incoherence",
        /// Schema, hash, canonicalization, registry, or command validation
        /// fails.
        Invalid => "invalid",
    }
}

fieldless_enum! {
    /// SPEC §4.4 artifact origin.
    Origin {
        HumanAuthored => "human_authored",
        AiAssisted => "ai_assisted",
        AiGenerated => "ai_generated",
        ExternalAdapterGenerated => "external_adapter_generated",
        DeterministicCompiler => "deterministic_compiler",
    }
}

fieldless_enum! {
    /// SPEC §4.4 artifact evidence_status. `compiler_evidence_status` is reserved for
    /// compiled artifacts and `verifier_evidence_status` for verifier results.
    EvidenceStatus {
        SourceEvidenceStatus => "source_evidence_status",
        MechanicalEvidenceStatus => "mechanical_evidence_status",
        EvidenceDiscoveryOnly => "evidence_discovery_only",
        AcceptedEvidenceStatus => "accepted_evidence_status",
        CompilerEvidenceStatus => "compiler_evidence_status",
        VerifierEvidenceStatus => "verifier_evidence_status",
        ViewOnly => "view_only",
    }
}

fieldless_enum! {
    /// SPEC §4.4 terminology binding status (SPEC §5 binding requirements).
    BindingStatus {
        Exact => "exact",
        Synonym => "synonym",
        Ambiguous => "ambiguous",
        Unmapped => "unmapped",
    }
}

fieldless_enum! {
    /// SPEC §4.4 rule direction (SPEC §6 groups `for|require|permit` against
    /// `against|contraindicate|avoid` for conflict eligibility).
    Direction {
        For => "for",
        Against => "against",
        Contraindicate => "contraindicate",
        Require => "require",
        Permit => "permit",
        Avoid => "avoid",
    }
}

fieldless_enum! {
    /// SPEC §4.4 claim evidence tier.
    ClaimTier {
        S0Replayable => "s0_replayable",
        S1Accepted => "s1_accepted",
        S2ResearchEvidence => "s2_research_evidence",
        S3ClinicalRegulatory => "s3_clinical_regulatory",
    }
}

fieldless_enum! {
    /// SPEC §4.4 review classification of a run finding.
    ReviewClassification {
        Candidate => "candidate",
        Residual => "residual",
        Ambiguity => "ambiguity",
        Incoherence => "incoherence",
        ReplayFailure => "replay_failure",
        DocumentedNoConflictResult => "documented_no_conflict_result",
    }
}

fieldless_enum! {
    /// SPEC §4.4 attempt classification (first used by the M5 loop).
    AttemptOutcome {
        Improved => "improved",
        Equivalent => "equivalent",
        Dominated => "dominated",
        Regression => "regression",
        Invalid => "invalid",
        Unsupported => "unsupported",
        Timeout => "timeout",
        Crash => "crash",
        NoConflictResult => "no_conflict_result",
        NearMiss => "near_miss",
        Unreproducible => "unreproducible",
        Unauthorized => "unauthorized",
        GateRequired => "gate_required",
    }
}

fieldless_enum! {
    /// SPEC §4.4 promotion decision (first used by the M5 loop).
    PromotionDecision {
        Promote => "promote",
        Reject => "reject",
        Quarantine => "quarantine",
        DeferGate => "defer_gate",
        RequestReplay => "request_replay",
    }
}

fieldless_enum! {
    /// SPEC §4.4 promotion scope (first used by the M5 loop).
    PromotionScope {
        RunLocal => "run_local",
        RegistryStatus => "registry_status",
    }
}

fieldless_enum! {
    /// SPEC §7.4 stable diagnostic codes, base set. Later milestones extend
    /// this set at elaboration time (M2 model-route, M4 invented-DSL,
    /// M5 loop/budget/surface, M6 source/permission/drift codes).
    DiagnosticCode {
        ExtractionUncertain => "extraction_uncertain",
        TableStructureUncertain => "table_structure_uncertain",
        SpanSourceLinkageMissing => "span_source_linkage_missing",
        SegmentationBoundaryError => "segmentation_boundary_error",
        TerminologyUnmapped => "terminology_unmapped",
        TerminologyAmbiguous => "terminology_ambiguous",
        TerminologyIncoherent => "terminology_incoherent",
        SemanticSlotMissing => "semantic_slot_missing",
        MissingPolicy => "missing_policy",
        IncompatiblePolicyRows => "incompatible_policy_rows",
        UnsupportedIrFragment => "unsupported_ir_fragment",
        SchemaInvalid => "schema_invalid",
        CompilerError => "compiler_error",
        TargetParseError => "target_parse_error",
        SolverTimeout => "solver_timeout",
        SolverUnknown => "solver_unknown",
        SolverExecutionFailure => "solver_execution_failure",
        ProcessCrash => "process_crash",
        TraceIncomplete => "trace_incomplete",
        ReplayMismatch => "replay_mismatch",
        ReplayIdentityUnsupported => "replay_identity_unsupported",
        DeferredGateRequired => "deferred_gate_required",
        FalsePositiveConflict => "false_positive_conflict",
        FalseNegativeConflict => "false_negative_conflict",
        MetamorphicInstability => "metamorphic_instability",
    }
}

/// A diagnostic-payload value: a string under the `diagnostic_text` policy
/// (SPEC §4.2), so payload text is normalized on write and verified on read.
struct DiagText(String);

impl Canonical for DiagText {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string_policy(out, StringPolicy::DiagnosticText, &self.0)
    }
}

impl CanonRead for DiagText {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        Ok(DiagText(read_string_policy(
            r,
            StringPolicy::DiagnosticText,
        )?))
    }
}

/// Emit the §7.4 structured payload as a SPEC §4.3 map: [`Id`] keys in object
/// form, values normalized under `diagnostic_text`. Numeric payload entries
/// ride as canonical decimal strings, matching the integer byte form.
/// Crate-visible so the ir module's structural emission reuses it verbatim.
pub(crate) fn emit_payload(out: &mut Vec<u8>, entries: &[(Id, String)]) -> Result<(), CanonError> {
    let texts: Vec<DiagText> = entries.iter().map(|(_, v)| DiagText(v.clone())).collect();
    emit_map(out, entries.iter().map(|(k, _)| k).zip(&texts))
}

/// Read a §7.4 structured payload, the inverse of [`emit_payload`].
/// Crate-visible so sibling payload carriers (bundle assumptions) reuse it.
pub(crate) fn read_payload(r: &mut Reader<'_>) -> Result<Vec<(Id, String)>, CanonReadError> {
    let entries = read_map::<Id, DiagText>(r)?;
    Ok(entries.into_iter().map(|(k, v)| (k, v.0)).collect())
}

/// SPEC §7.4 diagnostic: a stable code from the base set, a structured
/// payload, region/artifact refs, and exactly one [`Outcome`]. The ref lists
/// are sets in canonical form (sorted, duplicate-free); the producing processing_stage
/// chooses the outcome its code maps to (e.g. `terminology_ambiguous` ⇒
/// `ambiguity`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticRecord {
    pub code: DiagnosticCode,
    pub outcome: Outcome,
    /// Code-specific structured payload: identifier keys to `diagnostic_text`
    /// values.
    pub payload: Vec<(Id, String)>,
    /// SPEC §4.5 source-region ids the diagnostic grounds in.
    pub region_ids: Vec<Id>,
    /// Content hashes of implicated accepted artifacts (`_hash` rule, §4.4).
    pub artifact_hashes: Vec<Hash>,
}

impl Canonical for DiagnosticRecord {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("artifact_hashes", |b| emit_set(b, &self.artifact_hashes))?;
        obj.member("code", |b| self.code.emit_canonical(b))?;
        obj.member("outcome", |b| self.outcome.emit_canonical(b))?;
        obj.member("payload", |b| emit_payload(b, &self.payload))?;
        obj.member("region_ids", |b| emit_set(b, &self.region_ids))?;
        obj.finish(out)
    }
}

impl CanonRead for DiagnosticRecord {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let artifact_hashes = obj.member("artifact_hashes", read_set::<Hash>)?;
        let code = obj.member("code", DiagnosticCode::read)?;
        let outcome = obj.member("outcome", Outcome::read)?;
        let payload = obj.member("payload", read_payload)?;
        let region_ids = obj.member("region_ids", read_set::<Id>)?;
        obj.close()?;
        Ok(DiagnosticRecord {
            code,
            outcome,
            payload,
            region_ids,
            artifact_hashes,
        })
    }
}

/// SPEC §4.4 total operation result: every processing_stage and command returns exactly
/// one. `outcome` is the severity-aggregated total; the five buckets hold
/// content hashes of the produced value, diagnostic, residual, ambiguity, and
/// incoherence artifacts (sets in canonical form). Aggregation itself is wired
/// where processing_stages run (the CLI shell).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotalOperationResult {
    pub operation_id: Id,
    pub outcome: Outcome,
    pub value_hashes: Vec<Hash>,
    pub diagnostic_hashes: Vec<Hash>,
    pub residual_hashes: Vec<Hash>,
    pub ambiguity_hashes: Vec<Hash>,
    pub incoherence_hashes: Vec<Hash>,
}

impl Canonical for TotalOperationResult {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("ambiguity_hashes", |b| emit_set(b, &self.ambiguity_hashes))?;
        obj.member("diagnostic_hashes", |b| {
            emit_set(b, &self.diagnostic_hashes)
        })?;
        obj.member("incoherence_hashes", |b| {
            emit_set(b, &self.incoherence_hashes)
        })?;
        obj.member("operation_id", |b| self.operation_id.emit_canonical(b))?;
        obj.member("outcome", |b| self.outcome.emit_canonical(b))?;
        obj.member("residual_hashes", |b| emit_set(b, &self.residual_hashes))?;
        obj.member("value_hashes", |b| emit_set(b, &self.value_hashes))?;
        obj.finish(out)
    }
}

impl CanonRead for TotalOperationResult {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let ambiguity_hashes = obj.member("ambiguity_hashes", read_set::<Hash>)?;
        let diagnostic_hashes = obj.member("diagnostic_hashes", read_set::<Hash>)?;
        let incoherence_hashes = obj.member("incoherence_hashes", read_set::<Hash>)?;
        let operation_id = obj.member("operation_id", Id::read)?;
        let outcome = obj.member("outcome", Outcome::read)?;
        let residual_hashes = obj.member("residual_hashes", read_set::<Hash>)?;
        let value_hashes = obj.member("value_hashes", read_set::<Hash>)?;
        obj.close()?;
        Ok(TotalOperationResult {
            operation_id,
            outcome,
            value_hashes,
            diagnostic_hashes,
            residual_hashes,
            ambiguity_hashes,
            incoherence_hashes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon::{canonical_payload_bytes, read_strict_canonical};
    use crate::id::ValidationError;

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

    fn spellings<T: Copy>(all: &[T], as_str: fn(T) -> &'static str) -> Vec<&'static str> {
        all.iter().map(|&v| as_str(v)).collect()
    }

    // Pins every enum's value set and spelling against the §4.4/§7.4 tables.
    #[test]
    fn enum_spellings_match_spec_tables() {
        assert_eq!(
            spellings(Outcome::ALL, Outcome::as_str),
            [
                "ok",
                "residual",
                "ambiguity",
                "unsupported",
                "incoherence",
                "invalid"
            ]
        );
        assert_eq!(
            spellings(Origin::ALL, Origin::as_str),
            [
                "human_authored",
                "ai_assisted",
                "ai_generated",
                "external_adapter_generated",
                "deterministic_compiler"
            ]
        );
        assert_eq!(
            spellings(EvidenceStatus::ALL, EvidenceStatus::as_str),
            [
                "source_evidence_status",
                "mechanical_evidence_status",
                "evidence_discovery_only",
                "accepted_evidence_status",
                "compiler_evidence_status",
                "verifier_evidence_status",
                "view_only"
            ]
        );
        assert_eq!(
            spellings(BindingStatus::ALL, BindingStatus::as_str),
            ["exact", "synonym", "ambiguous", "unmapped"]
        );
        assert_eq!(
            spellings(Direction::ALL, Direction::as_str),
            [
                "for",
                "against",
                "contraindicate",
                "require",
                "permit",
                "avoid"
            ]
        );
        assert_eq!(
            spellings(ClaimTier::ALL, ClaimTier::as_str),
            [
                "s0_replayable",
                "s1_accepted",
                "s2_research_evidence",
                "s3_clinical_regulatory"
            ]
        );
        assert_eq!(
            spellings(ReviewClassification::ALL, ReviewClassification::as_str),
            [
                "candidate",
                "residual",
                "ambiguity",
                "incoherence",
                "replay_failure",
                "documented_no_conflict_result"
            ]
        );
        assert_eq!(
            spellings(AttemptOutcome::ALL, AttemptOutcome::as_str),
            [
                "improved",
                "equivalent",
                "dominated",
                "regression",
                "invalid",
                "unsupported",
                "timeout",
                "crash",
                "no_conflict_result",
                "near_miss",
                "unreproducible",
                "unauthorized",
                "gate_required"
            ]
        );
        assert_eq!(
            spellings(PromotionDecision::ALL, PromotionDecision::as_str),
            [
                "promote",
                "reject",
                "quarantine",
                "defer_gate",
                "request_replay"
            ]
        );
        assert_eq!(
            spellings(PromotionScope::ALL, PromotionScope::as_str),
            ["run_local", "registry_status"]
        );
        assert_eq!(
            spellings(DiagnosticCode::ALL, DiagnosticCode::as_str),
            [
                "extraction_uncertain",
                "table_structure_uncertain",
                "span_source_linkage_missing",
                "segmentation_boundary_error",
                "terminology_unmapped",
                "terminology_ambiguous",
                "terminology_incoherent",
                "semantic_slot_missing",
                "missing_policy",
                "incompatible_policy_rows",
                "unsupported_ir_fragment",
                "schema_invalid",
                "compiler_error",
                "target_parse_error",
                "solver_timeout",
                "solver_unknown",
                "solver_execution_failure",
                "process_crash",
                "trace_incomplete",
                "replay_mismatch",
                "replay_identity_unsupported",
                "deferred_gate_required",
                "false_positive_conflict",
                "false_negative_conflict",
                "metamorphic_instability"
            ]
        );
    }

    // §4.4 severity order: invalid > incoherence > unsupported > ambiguity >
    // residual > ok; `max` is severity aggregation.
    #[test]
    fn outcome_order_is_severity() {
        assert!(Outcome::ALL.windows(2).all(|w| w[0] < w[1]));
        assert!(Outcome::Invalid > Outcome::Incoherence);
        assert!(Outcome::Incoherence > Outcome::Unsupported);
        assert!(Outcome::Unsupported > Outcome::Ambiguity);
        assert!(Outcome::Ambiguity > Outcome::Residual);
        assert!(Outcome::Residual > Outcome::Ok);
        let aggregated = [Outcome::Ok, Outcome::Ambiguity, Outcome::Residual]
            .into_iter()
            .max();
        assert_eq!(aggregated, Some(Outcome::Ambiguity));
    }

    #[test]
    fn every_enum_value_round_trips() {
        for v in Outcome::ALL {
            round_trip(*v);
        }
        for v in Origin::ALL {
            round_trip(*v);
        }
        for v in EvidenceStatus::ALL {
            round_trip(*v);
        }
        for v in BindingStatus::ALL {
            round_trip(*v);
        }
        for v in Direction::ALL {
            round_trip(*v);
        }
        for v in ClaimTier::ALL {
            round_trip(*v);
        }
        for v in ReviewClassification::ALL {
            round_trip(*v);
        }
        for v in AttemptOutcome::ALL {
            round_trip(*v);
        }
        for v in PromotionDecision::ALL {
            round_trip(*v);
        }
        for v in PromotionScope::ALL {
            round_trip(*v);
        }
        for v in DiagnosticCode::ALL {
            round_trip(*v);
        }
    }

    #[test]
    fn enums_emit_bare_identifier_strings() {
        assert_eq!(canon(&Outcome::Ok), "\"ok\"");
        assert_eq!(canon(&Direction::For), "\"for\"");
        assert_eq!(canon(&ClaimTier::S0Replayable), "\"s0_replayable\"");
        assert_eq!(canon(&PromotionDecision::DeferGate), "\"defer_gate\"");
    }

    #[test]
    fn enum_reading_rejects_unknown_tokens_and_non_strings() {
        // a token outside the value set (including case variants)
        assert!(matches!(
            read_strict_canonical::<Outcome>(br#""weird""#),
            Err(CanonReadError::Policy(ValidationError::Enum(_)))
        ));
        assert!(matches!(
            read_strict_canonical::<Direction>(br#""FOR""#),
            Err(CanonReadError::Policy(ValidationError::Enum(_)))
        ));
        // null is never canonical
        assert_eq!(
            read_strict_canonical::<Outcome>(b"null"),
            Err(CanonReadError::Token)
        );
    }

    fn sample_diagnostic() -> DiagnosticRecord {
        DiagnosticRecord {
            code: DiagnosticCode::TerminologyAmbiguous,
            outcome: Outcome::Ambiguity,
            payload: vec![
                (Id::new("candidates").unwrap(), "2".to_owned()),
                (Id::new("mention").unwrap(), "アスピリン 錠".to_owned()),
            ],
            region_ids: vec![Id::new("region.a").unwrap(), Id::new("region.b").unwrap()],
            artifact_hashes: vec![h('a')],
        }
    }

    // Pins the record's canonical shape: byte-sorted fields, object-form
    // payload map, set-form refs.
    #[test]
    fn diagnostic_record_canonical_bytes() {
        let mut record = sample_diagnostic();
        // unsorted inputs: sets sort by canonical bytes on emission
        record.region_ids.swap(0, 1);
        // diagnostic_text folds the doubled space on emission
        record.payload[1].1 = "アスピリン  錠".to_owned();
        let want = format!(
            concat!(
                r#"{{"artifact_hashes":["{}"],"code":"terminology_ambiguous","#,
                r#""outcome":"ambiguity","payload":{{"candidates":"2","mention":"アスピリン 錠"}},"#,
                r#""region_ids":["region.a","region.b"]}}"#
            ),
            h('a').as_str()
        );
        assert_eq!(canon(&record), want);
    }

    #[test]
    fn diagnostic_record_round_trips() {
        round_trip(sample_diagnostic());
    }

    #[test]
    fn diagnostic_payload_rejects_duplicates_unsorted_and_unnormalized() {
        // duplicate payload keys are ambiguous on emission
        let mut record = sample_diagnostic();
        record.payload = vec![
            (Id::new("k").unwrap(), "1".to_owned()),
            (Id::new("k").unwrap(), "2".to_owned()),
        ];
        let mut out = Vec::new();
        assert!(matches!(
            record.emit_canonical(&mut out),
            Err(CanonError::DuplicateMapKey(_))
        ));
        // misordered payload keys are non-canonical on reading
        let unsorted = br#"{"artifact_hashes":[],"code":"missing_policy","outcome":"residual","payload":{"b":"1","a":"2"},"region_ids":[]}"#;
        assert_eq!(
            read_strict_canonical::<DiagnosticRecord>(unsorted),
            Err(CanonReadError::Unsorted)
        );
        // a payload value that is not diagnostic_text-normalized is rejected
        let unnormalized = br#"{"artifact_hashes":[],"code":"missing_policy","outcome":"residual","payload":{"k":"a  b"},"region_ids":[]}"#;
        assert_eq!(
            read_strict_canonical::<DiagnosticRecord>(unnormalized),
            Err(CanonReadError::Unnormalized(StringPolicy::DiagnosticText))
        );
    }

    fn sample_result() -> TotalOperationResult {
        TotalOperationResult {
            operation_id: Id::new("compile").unwrap(),
            outcome: Outcome::Ok,
            value_hashes: vec![h('a')],
            diagnostic_hashes: vec![],
            residual_hashes: vec![],
            ambiguity_hashes: vec![],
            incoherence_hashes: vec![],
        }
    }

    // Pins the §4.4 total-result shape (the spec example, in canonical field
    // order) — exactly the five hash buckets plus operation_id and outcome.
    #[test]
    fn total_result_canonical_bytes() {
        let want = format!(
            concat!(
                r#"{{"ambiguity_hashes":[],"diagnostic_hashes":[],"incoherence_hashes":[],"#,
                r#""operation_id":"compile","outcome":"ok","residual_hashes":[],"#,
                r#""value_hashes":["{}"]}}"#
            ),
            h('a').as_str()
        );
        assert_eq!(canon(&sample_result()), want);
    }

    // Serde (the §8.4 YAML registry surface) speaks the same identifier_ascii
    // spelling as the canonical bytes, and accepts nothing else.
    #[test]
    fn fieldless_enum_serde_mirrors_canonical_spelling() {
        assert_eq!(serde_json::to_string(&Outcome::Ok).unwrap(), r#""ok""#);
        let parsed: Origin = serde_json::from_str(r#""ai_generated""#).unwrap();
        assert_eq!(parsed, Origin::AiGenerated);
        assert!(serde_json::from_str::<Origin>(r#""AI_GENERATED""#).is_err());
    }

    #[test]
    fn total_result_round_trips_and_rejects_unknown_fields() {
        round_trip(sample_result());
        let bytes = canonical_payload_bytes(&sample_result()).unwrap();
        let mut text = String::from_utf8(bytes).unwrap();
        // splice an extra trailing field the schema never declared
        text.truncate(text.len() - 1);
        text.push_str(r#","z":"1"}"#);
        assert_eq!(
            read_strict_canonical::<TotalOperationResult>(text.as_bytes()),
            Err(CanonReadError::UnknownField("z".to_owned()))
        );
    }
}
