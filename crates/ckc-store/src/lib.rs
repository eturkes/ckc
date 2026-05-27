use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ckc_core::canonical::{to_canonical_bytes, ContentHash};
use ckc_core::envelope::{ArtifactEnvelope, ArtifactKind};
use ckc_core::profile::SemanticProfile;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Deserialize(serde_json::Error),
    /// Hash string failed `sha256:<64 hex chars>` validation.
    InvalidHash(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "store I/O: {e}"),
            Self::Deserialize(e) => write!(f, "store deserialize: {e}"),
            Self::InvalidHash(h) => write!(f, "invalid content hash: {h}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Deserialize(e) => Some(e),
            Self::InvalidHash(_) => None,
        }
    }
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Deserialize(e)
    }
}

/// One entry in a `StoreManifest`, summarizing a stored artifact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ManifestEntry {
    pub hash: ContentHash,
    pub kind: ArtifactKind,
    pub stage: String,
    pub profiles: Vec<SemanticProfile>,
    /// File modification time as seconds since Unix epoch.
    pub stored_at_epoch: u64,
}

/// Deterministic inventory of all artifacts in a `ContentStore`.
///
/// Entries are sorted by `hash` for deterministic canonical JSON bytes.
/// The manifest is itself content-hashable and storable as an
/// `ArtifactKind::StoreManifest` envelope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StoreManifest {
    pub entries: Vec<ManifestEntry>,
}

/// Filesystem content-addressed store for CKC artifact envelopes.
///
/// Objects live at `<root>/objects/<hex[0:2]>/<hex[2:4]>/<hex>.json`
/// where `<hex>` is the SHA-256 of the envelope's canonical JSON bytes.
pub struct ContentStore {
    root: PathBuf,
}

impl ContentStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Serialize envelope to canonical JSON, compute SHA-256, write to
    /// the content-addressed path. Idempotent: skips write when the
    /// object already exists. Returns the envelope's content hash
    /// (the store key).
    pub fn put(&self, envelope: &ArtifactEnvelope) -> Result<ContentHash, StoreError> {
        let bytes = to_canonical_bytes(envelope);
        let hash = sha256_of_bytes(&bytes);
        let path = self.object_path(&hash)?;
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let tmp = path.with_extension("json.tmp");
            fs::write(&tmp, &bytes)?;
            fs::rename(&tmp, &path)?;
        }
        Ok(hash)
    }

    /// Store multiple envelopes. Returns their content hashes in the
    /// same order as the input slice.
    pub fn put_batch(
        &self,
        envelopes: &[ArtifactEnvelope],
    ) -> Result<Vec<ContentHash>, StoreError> {
        envelopes.iter().map(|e| self.put(e)).collect()
    }

    /// Read an envelope by its content hash.
    pub fn get(&self, hash: &ContentHash) -> Result<ArtifactEnvelope, StoreError> {
        let path = self.object_path(hash)?;
        let bytes = fs::read(&path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Check whether an object with the given hash exists.
    pub fn exists(&self, hash: &ContentHash) -> bool {
        self.object_path(hash).map(|p| p.exists()).unwrap_or(false)
    }

    /// Re-hash the stored bytes and confirm they match the expected hash.
    pub fn verify(&self, hash: &ContentHash) -> Result<bool, StoreError> {
        let path = self.object_path(hash)?;
        let bytes = fs::read(&path)?;
        Ok(sha256_of_bytes(&bytes) == *hash)
    }

    /// Walk the store and produce a deterministic manifest of all stored
    /// artifacts, sorted by content hash.
    pub fn generate_manifest(&self) -> Result<StoreManifest, StoreError> {
        let objects_dir = self.root.join("objects");
        let mut entries = Vec::new();

        if objects_dir.is_dir() {
            Self::walk_objects(&objects_dir, &mut entries)?;
        }

        entries.sort_by(|a, b| a.hash.cmp(&b.hash));
        Ok(StoreManifest { entries })
    }

    fn walk_objects(
        dir: &Path,
        entries: &mut Vec<ManifestEntry>,
    ) -> Result<(), StoreError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::walk_objects(&path, entries)?;
            } else if path.extension().is_some_and(|e| e == "json") {
                let bytes = fs::read(&path)?;
                let envelope: ArtifactEnvelope = serde_json::from_slice(&bytes)?;
                let hash = sha256_of_bytes(&bytes);
                let mtime_epoch = fs::metadata(&path)?
                    .modified()?
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                entries.push(ManifestEntry {
                    hash,
                    kind: envelope.kind,
                    stage: envelope.meta.stage.clone(),
                    profiles: envelope.meta.semantic_profiles.clone(),
                    stored_at_epoch: mtime_epoch,
                });
            }
        }
        Ok(())
    }

    fn object_path(&self, hash: &ContentHash) -> Result<PathBuf, StoreError> {
        let hex = extract_hex(hash)?;
        Ok(self
            .root
            .join("objects")
            .join(&hex[..2])
            .join(&hex[2..4])
            .join(format!("{hex}.json")))
    }
}

/// Extract the 64-char hex portion from `sha256:<hex>`.
fn extract_hex(hash: &ContentHash) -> Result<&str, StoreError> {
    hash.as_str()
        .strip_prefix("sha256:")
        .filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| StoreError::InvalidHash(hash.as_str().to_owned()))
}

/// Compute `sha256:<hex>` from raw bytes.
fn sha256_of_bytes(bytes: &[u8]) -> ContentHash {
    use std::fmt::Write;
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(7 + 64);
    s.push_str("sha256:");
    for b in digest.iter() {
        write!(s, "{b:02x}").unwrap();
    }
    ContentHash(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::canonical::content_hash;
    use ckc_core::clinical::{Action, Norm, Rule};
    use ckc_core::enums::*;
    use ckc_core::envelope::ArtifactMeta;
    use ckc_core::id::*;
    use ckc_core::source::CorpusDocument;
    use ckc_core::verify::Conflict;
    use serde_json::json;
    use tempfile::TempDir;

    fn test_store() -> (ContentStore, TempDir) {
        let tmp = TempDir::new().unwrap();
        (ContentStore::new(tmp.path()), tmp)
    }

    fn meta(stage: &str) -> ArtifactMeta {
        ArtifactMeta {
            schema_version: "0.0.0".into(),
            producer_version: "ckc-store/test".into(),
            command_manifest: json!({"test": true}),
            source_input_hashes: vec![],
            parent_hashes: vec![],
            stage: stage.into(),
            semantic_profiles: vec![],
            content_hash: ContentHash(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .into(),
            ),
            certificate_ids: vec![],
            replay_command: None,
        }
    }

    fn meta_with_profiles(stage: &str, profiles: Vec<SemanticProfile>) -> ArtifactMeta {
        ArtifactMeta {
            semantic_profiles: profiles,
            ..meta(stage)
        }
    }

    fn rule_envelope() -> ArtifactEnvelope {
        let rule = Rule {
            rule_id: RuleId::new("rule_test_001"),
            profiles: vec![SemanticProfile::Norm],
            kind: RuleKind::Strict,
            context: "test_context".into(),
            antecedent: "(dx test)".into(),
            consequent: "(action test)".into(),
            norm: Some(Norm {
                context: "test".into(),
                direction: RecommendationDirection::For,
                action: Action {
                    action_type: "test".into(),
                    target_concept: ConceptId::new("concept_test"),
                    parameters: json!({}),
                    temporal_constraints: json!({}),
                    quantity_constraints: json!({}),
                },
                recommendation_strength: RecommendationStrength::Strong,
                evidence_certainty: EvidenceCertainty::High,
                original_modality_phrase_ja: "推奨する".into(),
                deontic_projection: DeonticProjection::Recommended,
                exception_policy: "none".into(),
                prima_facie_or_all_things_considered: NormCommitment::AllThingsConsidered,
            }),
            priority_over: vec![],
            exceptions: vec![],
            temporal_scope: None,
            population_scope: None,
            source_span_ids: vec![SpanId::new("span_1")],
            provenance: "test".into(),
            certificate_ids: vec![],
        };
        ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            &rule,
            meta_with_profiles("normalize", vec![SemanticProfile::Norm]),
        )
    }

    fn document_envelope() -> ArtifactEnvelope {
        let doc = CorpusDocument {
            doc_id: DocId::new("doc_test_001"),
            title_ja: "テスト文書".into(),
            title_en: Some("Test Document".into()),
            source_type: "guideline".into(),
            publisher: "test_publisher".into(),
            society: "test_society".into(),
            edition: "2024".into(),
            publication_date: Some("2024-01-01".into()),
            access_date: None,
            license_status: "permitted".into(),
            content_hash: ContentHash(
                "sha256:bb00000000000000000000000000000000000000000000000000000000000001"
                    .into(),
            ),
            extraction_manifest_id: ManifestId::new("manifest_test_001"),
            supersedes: None,
            superseded_by: None,
        };
        ArtifactEnvelope::wrap(
            ArtifactKind::CorpusDocument,
            &doc,
            meta_with_profiles("ingest", vec![SemanticProfile::Text]),
        )
    }

    fn conflict_envelope() -> ArtifactEnvelope {
        let conflict = Conflict {
            conflict_id: ConflictId::new("conflict_test_001"),
            conflict_type: "norm_contradiction".into(),
            severity: Severity::High,
            confidence: 0.9,
            minimal_artifact_set: vec![],
            source_spans: vec![SpanId::new("span_1")],
            normalized_view: json!({"type": "test"}),
            witness: None,
            repair_candidates: vec![],
            solver_evidence: vec![],
            argument_graph_id: None,
            human_review_question_ja: "テスト質問".into(),
            human_review_question_en: "Test question".into(),
            classification: ConflictClassification::TrueConflict,
        };
        ArtifactEnvelope::wrap(ArtifactKind::Conflict, &conflict, meta("conflicts"))
    }

    // -- put / get round-trip on multiple artifact types --

    #[test]
    fn put_get_roundtrip_rule() {
        let (store, _tmp) = test_store();
        let envelope = rule_envelope();
        let hash = store.put(&envelope).unwrap();
        let got = store.get(&hash).unwrap();
        assert_eq!(envelope, got);
    }

    #[test]
    fn put_get_roundtrip_document() {
        let (store, _tmp) = test_store();
        let envelope = document_envelope();
        let hash = store.put(&envelope).unwrap();
        let got = store.get(&hash).unwrap();
        assert_eq!(envelope, got);
    }

    #[test]
    fn put_get_roundtrip_conflict() {
        let (store, _tmp) = test_store();
        let envelope = conflict_envelope();
        let hash = store.put(&envelope).unwrap();
        let got = store.get(&hash).unwrap();
        assert_eq!(envelope, got);
    }

    // -- hash consistency --

    #[test]
    fn put_returns_envelope_hash() {
        let (store, _tmp) = test_store();
        let envelope = rule_envelope();
        let hash = store.put(&envelope).unwrap();
        assert_eq!(hash, envelope.envelope_hash());
    }

    #[test]
    fn put_hash_matches_content_hash_fn() {
        let (store, _tmp) = test_store();
        let envelope = rule_envelope();
        let hash = store.put(&envelope).unwrap();
        assert_eq!(hash, content_hash(&envelope));
    }

    // -- idempotency --

    #[test]
    fn put_is_idempotent() {
        let (store, _tmp) = test_store();
        let envelope = rule_envelope();
        let h1 = store.put(&envelope).unwrap();
        let h2 = store.put(&envelope).unwrap();
        assert_eq!(h1, h2);
        let got = store.get(&h1).unwrap();
        assert_eq!(envelope, got);
    }

    // -- exists --

    #[test]
    fn exists_true_after_put() {
        let (store, _tmp) = test_store();
        let envelope = rule_envelope();
        let hash = store.put(&envelope).unwrap();
        assert!(store.exists(&hash));
    }

    #[test]
    fn exists_false_for_absent() {
        let (store, _tmp) = test_store();
        let hash = ContentHash(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .into(),
        );
        assert!(!store.exists(&hash));
    }

    // -- verify --

    #[test]
    fn verify_passes_for_valid() {
        let (store, _tmp) = test_store();
        let envelope = rule_envelope();
        let hash = store.put(&envelope).unwrap();
        assert!(store.verify(&hash).unwrap());
    }

    #[test]
    fn verify_detects_corruption() {
        let (store, _tmp) = test_store();
        let envelope = rule_envelope();
        let hash = store.put(&envelope).unwrap();
        let hex = extract_hex(&hash).unwrap();
        let path = store
            .root()
            .join("objects")
            .join(&hex[..2])
            .join(&hex[2..4])
            .join(format!("{hex}.json"));
        let mut bytes = fs::read(&path).unwrap();
        if let Some(b) = bytes.last_mut() {
            *b ^= 0xFF;
        }
        fs::write(&path, &bytes).unwrap();
        assert!(!store.verify(&hash).unwrap());
    }

    // -- error cases --

    #[test]
    fn get_nonexistent_returns_io_error() {
        let (store, _tmp) = test_store();
        let hash = ContentHash(
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .into(),
        );
        assert!(matches!(store.get(&hash), Err(StoreError::Io(_))));
    }

    #[test]
    fn invalid_hash_rejected_by_get() {
        let (store, _tmp) = test_store();
        let bad = ContentHash("bad_hash".into());
        assert!(matches!(
            store.get(&bad),
            Err(StoreError::InvalidHash(_))
        ));
    }

    #[test]
    fn invalid_hash_rejected_by_verify() {
        let (store, _tmp) = test_store();
        let bad = ContentHash("sha256:tooshort".into());
        assert!(matches!(
            store.verify(&bad),
            Err(StoreError::InvalidHash(_))
        ));
    }

    #[test]
    fn invalid_hash_exists_returns_false() {
        let (store, _tmp) = test_store();
        let bad = ContentHash("garbage".into());
        assert!(!store.exists(&bad));
    }

    // -- directory structure --

    #[test]
    fn correct_directory_structure() {
        let (store, _tmp) = test_store();
        let envelope = rule_envelope();
        let hash = store.put(&envelope).unwrap();
        let hex = extract_hex(&hash).unwrap();
        let expected = store
            .root()
            .join("objects")
            .join(&hex[..2])
            .join(&hex[2..4])
            .join(format!("{hex}.json"));
        assert!(expected.exists());
    }

    // -- multiple artifacts --

    #[test]
    fn multiple_artifacts_coexist() {
        let (store, _tmp) = test_store();
        let e1 = rule_envelope();
        let e2 = document_envelope();
        let e3 = conflict_envelope();
        let h1 = store.put(&e1).unwrap();
        let h2 = store.put(&e2).unwrap();
        let h3 = store.put(&e3).unwrap();
        assert_ne!(h1, h2);
        assert_ne!(h2, h3);
        assert_ne!(h1, h3);
        assert_eq!(store.get(&h1).unwrap(), e1);
        assert_eq!(store.get(&h2).unwrap(), e2);
        assert_eq!(store.get(&h3).unwrap(), e3);
        assert!(store.verify(&h1).unwrap());
        assert!(store.verify(&h2).unwrap());
        assert!(store.verify(&h3).unwrap());
    }

    // -- stored bytes are canonical JSON --

    #[test]
    fn stored_bytes_are_canonical() {
        let (store, _tmp) = test_store();
        let envelope = rule_envelope();
        let hash = store.put(&envelope).unwrap();
        let hex = extract_hex(&hash).unwrap();
        let path = store
            .root()
            .join("objects")
            .join(&hex[..2])
            .join(&hex[2..4])
            .join(format!("{hex}.json"));
        let stored = fs::read(&path).unwrap();
        let canonical = to_canonical_bytes(&envelope);
        assert_eq!(stored, canonical);
    }

    // -- put_batch --

    #[test]
    fn put_batch_stores_all() {
        let (store, _tmp) = test_store();
        let envelopes = [rule_envelope(), document_envelope(), conflict_envelope()];
        let hashes = store.put_batch(&envelopes).unwrap();
        assert_eq!(hashes.len(), 3);
        for (env, hash) in envelopes.iter().zip(&hashes) {
            assert!(store.exists(hash));
            assert_eq!(&store.get(hash).unwrap(), env);
        }
    }

    #[test]
    fn put_batch_returns_hashes_in_input_order() {
        let (store, _tmp) = test_store();
        let envelopes = [rule_envelope(), document_envelope(), conflict_envelope()];
        let batch_hashes = store.put_batch(&envelopes).unwrap();
        let individual_hashes: Vec<_> = envelopes.iter().map(|e| store.put(e).unwrap()).collect();
        assert_eq!(batch_hashes, individual_hashes);
    }

    #[test]
    fn put_batch_empty() {
        let (store, _tmp) = test_store();
        let hashes = store.put_batch(&[]).unwrap();
        assert!(hashes.is_empty());
    }

    // -- generate_manifest --

    #[test]
    fn manifest_empty_store() {
        let (store, _tmp) = test_store();
        let manifest = store.generate_manifest().unwrap();
        assert!(manifest.entries.is_empty());
    }

    #[test]
    fn manifest_contains_all_stored_artifacts() {
        let (store, _tmp) = test_store();
        let envelopes = [rule_envelope(), document_envelope(), conflict_envelope()];
        let hashes = store.put_batch(&envelopes).unwrap();
        let manifest = store.generate_manifest().unwrap();
        assert_eq!(manifest.entries.len(), 3);
        let manifest_hashes: Vec<_> = manifest.entries.iter().map(|e| &e.hash).collect();
        for h in &hashes {
            assert!(manifest_hashes.contains(&h));
        }
    }

    #[test]
    fn manifest_entries_sorted_by_hash() {
        let (store, _tmp) = test_store();
        store
            .put_batch(&[rule_envelope(), document_envelope(), conflict_envelope()])
            .unwrap();
        let manifest = store.generate_manifest().unwrap();
        let hashes: Vec<_> = manifest.entries.iter().map(|e| &e.hash).collect();
        let mut sorted = hashes.clone();
        sorted.sort();
        assert_eq!(hashes, sorted);
    }

    #[test]
    fn manifest_entry_metadata_correct() {
        let (store, _tmp) = test_store();
        let envelope = rule_envelope();
        let hash = store.put(&envelope).unwrap();
        let manifest = store.generate_manifest().unwrap();
        assert_eq!(manifest.entries.len(), 1);
        let entry = &manifest.entries[0];
        assert_eq!(entry.hash, hash);
        assert_eq!(entry.kind, ArtifactKind::Rule);
        assert_eq!(entry.stage, "normalize");
        assert_eq!(entry.profiles, vec![SemanticProfile::Norm]);
        assert!(entry.stored_at_epoch > 0);
    }

    #[test]
    fn manifest_deterministic_across_regeneration() {
        let (store, _tmp) = test_store();
        store
            .put_batch(&[rule_envelope(), document_envelope(), conflict_envelope()])
            .unwrap();
        let m1 = store.generate_manifest().unwrap();
        let m2 = store.generate_manifest().unwrap();
        assert_eq!(
            to_canonical_bytes(&m1),
            to_canonical_bytes(&m2),
            "consecutive manifest generations must produce identical canonical bytes"
        );
        assert_eq!(
            content_hash(&m1),
            content_hash(&m2),
            "consecutive manifest generations must produce identical content hashes"
        );
    }

    #[test]
    fn manifest_hash_changes_after_new_artifact() {
        let (store, _tmp) = test_store();
        store.put(&rule_envelope()).unwrap();
        let m1 = store.generate_manifest().unwrap();
        let h1 = content_hash(&m1);
        store.put(&document_envelope()).unwrap();
        let m2 = store.generate_manifest().unwrap();
        let h2 = content_hash(&m2);
        assert_ne!(h1, h2);
    }

    #[test]
    fn manifest_is_storable_as_envelope() {
        let (store, _tmp) = test_store();
        store
            .put_batch(&[rule_envelope(), document_envelope()])
            .unwrap();
        let manifest = store.generate_manifest().unwrap();
        let envelope = ArtifactEnvelope::wrap(
            ArtifactKind::StoreManifest,
            &manifest,
            meta("manifest"),
        );
        let hash = store.put(&envelope).unwrap();
        let got = store.get(&hash).unwrap();
        assert_eq!(envelope, got);
        let extracted: StoreManifest = got.extract().unwrap();
        assert_eq!(manifest, extracted);
    }

    #[test]
    fn manifest_serde_roundtrip() {
        let manifest = StoreManifest {
            entries: vec![
                ManifestEntry {
                    hash: ContentHash(
                        "sha256:aa00000000000000000000000000000000000000000000000000000000000001"
                            .into(),
                    ),
                    kind: ArtifactKind::Rule,
                    stage: "normalize".into(),
                    profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
                    stored_at_epoch: 1716854400,
                },
                ManifestEntry {
                    hash: ContentHash(
                        "sha256:bb00000000000000000000000000000000000000000000000000000000000002"
                            .into(),
                    ),
                    kind: ArtifactKind::CorpusDocument,
                    stage: "ingest".into(),
                    profiles: vec![SemanticProfile::Text],
                    stored_at_epoch: 1716854401,
                },
            ],
        };
        let bytes = to_canonical_bytes(&manifest);
        let rt: StoreManifest = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(manifest, rt);
        let bytes2 = to_canonical_bytes(&rt);
        assert_eq!(bytes, bytes2);
    }
}
