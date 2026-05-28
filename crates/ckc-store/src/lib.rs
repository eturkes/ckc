use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ckc_core::canonical::{ContentHash, to_canonical_bytes};
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

    fn walk_objects(dir: &Path, entries: &mut Vec<ManifestEntry>) -> Result<(), StoreError> {
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
    use ckc_core::envelope::ArtifactMeta;
    use serde_json::json;
    use tempfile::TempDir;

    fn test_store() -> (ContentStore, TempDir) {
        let tmp = TempDir::new().unwrap();
        (ContentStore::new(tmp.path()), tmp)
    }

    fn hash_of(byte: char) -> ContentHash {
        ContentHash(format!("sha256:{}", byte.to_string().repeat(64)))
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
            content_hash: hash_of('0'),
            certificate_ids: vec![],
            replay_command: None,
        }
    }

    /// Three differently-shaped envelopes — used together to exercise CAS
    /// distinguishing distinct payloads. Each carries a distinct `stage`
    /// so the resulting envelope hashes are guaranteed to differ even if
    /// the inner payloads were trivially small.
    fn three_envelopes() -> [ArtifactEnvelope; 3] {
        let p1 = json!({"id": "rule_1", "kind": "strict"});
        let p2 = json!({"id": "doc_1", "title": "テスト"});
        let p3 = json!({"id": "conflict_1", "severity": "high"});
        [
            ArtifactEnvelope::wrap(ArtifactKind::Rule, &p1, meta("normalize")),
            ArtifactEnvelope::wrap(ArtifactKind::CorpusDocument, &p2, meta("ingest")),
            ArtifactEnvelope::wrap(ArtifactKind::Conflict, &p3, meta("conflicts")),
        ]
    }

    #[test]
    fn put_get_roundtrip() {
        let (store, _tmp) = test_store();
        let env = &three_envelopes()[0];
        let h = store.put(env).unwrap();
        assert_eq!(h, env.envelope_hash());
        assert_eq!(&store.get(&h).unwrap(), env);
        assert!(store.exists(&h));
        // Stored bytes match the canonical encoding of the envelope.
        let hex = extract_hex(&h).unwrap();
        let path = store
            .root()
            .join("objects")
            .join(&hex[..2])
            .join(&hex[2..4])
            .join(format!("{hex}.json"));
        assert_eq!(fs::read(&path).unwrap(), to_canonical_bytes(env));
    }

    #[test]
    fn put_is_idempotent() {
        let (store, _tmp) = test_store();
        let env = &three_envelopes()[0];
        assert_eq!(store.put(env).unwrap(), store.put(env).unwrap());
    }

    #[test]
    fn exists_false_for_absent() {
        let (store, _tmp) = test_store();
        assert!(!store.exists(&hash_of('a')));
    }

    #[test]
    fn verify_detects_corruption() {
        let (store, _tmp) = test_store();
        let env = &three_envelopes()[0];
        let h = store.put(env).unwrap();
        let hex = extract_hex(&h).unwrap();
        let path = store
            .root()
            .join("objects")
            .join(&hex[..2])
            .join(&hex[2..4])
            .join(format!("{hex}.json"));
        let mut bytes = fs::read(&path).unwrap();
        *bytes.last_mut().unwrap() ^= 0xFF;
        fs::write(&path, &bytes).unwrap();
        assert!(!store.verify(&h).unwrap());
    }

    #[test]
    fn invalid_hash_rejected() {
        let (store, _tmp) = test_store();
        let bad = ContentHash("garbage".into());
        let short = ContentHash("sha256:tooshort".into());
        assert!(matches!(store.get(&bad), Err(StoreError::InvalidHash(_))));
        assert!(matches!(
            store.verify(&short),
            Err(StoreError::InvalidHash(_))
        ));
        // exists() swallows hash-format errors and reports false.
        assert!(!store.exists(&bad));
    }

    #[test]
    fn get_missing_returns_io_error() {
        let (store, _tmp) = test_store();
        assert!(matches!(store.get(&hash_of('b')), Err(StoreError::Io(_))));
    }

    #[test]
    fn put_batch_round_trips_three_envelopes() {
        let (store, _tmp) = test_store();
        let envs = three_envelopes();
        let hashes = store.put_batch(&envs).unwrap();
        assert_eq!(hashes.len(), 3);
        for (env, h) in envs.iter().zip(&hashes) {
            assert_eq!(&store.get(h).unwrap(), env);
        }
        // The same hashes come back from individual puts.
        let again: Vec<_> = envs.iter().map(|e| store.put(e).unwrap()).collect();
        assert_eq!(hashes, again);
    }

    #[test]
    fn manifest_lists_all_stored_envelopes_with_metadata() {
        let (store, _tmp) = test_store();
        let envs = three_envelopes();
        let hashes = store.put_batch(&envs).unwrap();
        let m = store.generate_manifest().unwrap();
        assert_eq!(m.entries.len(), 3);
        // Entries sorted by hash for deterministic canonical bytes.
        let listed: Vec<_> = m.entries.iter().map(|e| &e.hash).collect();
        let mut sorted = listed.clone();
        sorted.sort();
        assert_eq!(listed, sorted);
        // Every put-hash is present and metadata propagates.
        for (env, h) in envs.iter().zip(&hashes) {
            let entry = m.entries.iter().find(|e| &e.hash == h).unwrap();
            assert_eq!(entry.kind, env.kind);
            assert_eq!(entry.stage, env.meta.stage);
            assert!(entry.stored_at_epoch > 0);
        }
    }

    #[test]
    fn manifest_is_storable_as_envelope() {
        let (store, _tmp) = test_store();
        store.put_batch(&three_envelopes()[..2]).unwrap();
        let manifest = store.generate_manifest().unwrap();
        let env = ArtifactEnvelope::wrap(ArtifactKind::StoreManifest, &manifest, meta("manifest"));
        let h = store.put(&env).unwrap();
        assert_eq!(
            store.get(&h).unwrap().extract::<StoreManifest>().unwrap(),
            manifest
        );
    }
}
