use std::path::PathBuf;

use ckc_core::canonical::{content_hash, to_canonical_bytes, ContentHash};
use ckc_core::envelope::ArtifactKind;
use ckc_core::profile::SemanticProfile;
use ckc_store::{ManifestEntry, StoreManifest};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

fn golden_dir() -> PathBuf {
    workspace_root().join("schemas").join("golden")
}

fn golden_store_manifest() -> StoreManifest {
    StoreManifest {
        entries: vec![
            ManifestEntry {
                hash: ContentHash(
                    "sha256:1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b"
                        .into(),
                ),
                kind: ArtifactKind::CorpusDocument,
                stage: "ingest".into(),
                profiles: vec![SemanticProfile::Text],
                stored_at_epoch: 1716854400,
            },
            ManifestEntry {
                hash: ContentHash(
                    "sha256:2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c"
                        .into(),
                ),
                kind: ArtifactKind::Rule,
                stage: "normalize".into(),
                profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
                stored_at_epoch: 1716854401,
            },
            ManifestEntry {
                hash: ContentHash(
                    "sha256:3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d"
                        .into(),
                ),
                kind: ArtifactKind::Conflict,
                stage: "conflicts".into(),
                profiles: vec![],
                stored_at_epoch: 1716854402,
            },
        ],
    }
}

#[test]
fn golden_canonical_bytes() {
    let manifest = golden_store_manifest();
    let bytes = to_canonical_bytes(&manifest);
    let path = golden_dir().join("store_manifest.json");
    let golden = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("read golden {}: {e}", path.display()));
    assert!(
        bytes == golden,
        "canonical bytes mismatch for store_manifest (got {} bytes, golden {} bytes)",
        bytes.len(),
        golden.len()
    );
}

#[test]
fn golden_roundtrip() {
    let manifest = golden_store_manifest();
    let bytes1 = to_canonical_bytes(&manifest);
    let h1 = content_hash(&manifest);
    let rt: StoreManifest = serde_json::from_slice(&bytes1).unwrap();
    let bytes2 = to_canonical_bytes(&rt);
    let h2 = content_hash(&rt);
    assert_eq!(bytes1, bytes2, "bytes differ after roundtrip");
    assert_eq!(h1, h2, "hash differs after roundtrip");
    assert_eq!(manifest, rt, "value differs after roundtrip");
}

#[test]
fn golden_schema() {
    let schema = schemars::schema_for!(StoreManifest);
    let json = serde_json::to_string_pretty(&schema).unwrap() + "\n";
    let path = workspace_root()
        .join("schemas")
        .join("store_manifest.schema.json");
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read schema {}: {e}", path.display()));
    assert!(json == golden, "schema mismatch for store_manifest");
}

#[test]
#[ignore]
fn regenerate() {
    let manifest = golden_store_manifest();
    let g = golden_dir();
    std::fs::create_dir_all(&g).unwrap();
    std::fs::write(
        g.join("store_manifest.json"),
        to_canonical_bytes(&manifest),
    )
    .unwrap();

    let schema = schemars::schema_for!(StoreManifest);
    std::fs::write(
        workspace_root()
            .join("schemas")
            .join("store_manifest.schema.json"),
        serde_json::to_string_pretty(&schema).unwrap() + "\n",
    )
    .unwrap();
}
