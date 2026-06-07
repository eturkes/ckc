//! Gate T-Unicode-Idempotency (§11.3): §1.4 string-policy idempotency and
//! byte-stability over UnicodePolicyManifest fixtures.
//!
//! Fixture regen: `CKC_BLESS=1 cargo test -p ckc-core --test
//! t_unicode_idempotency` rewrites `fixtures/policy_vectors.json` (expected
//! outputs recomputed from the seed inputs below) and
//! `fixtures/unicode_policy_manifest.json` (recomputed table fingerprints,
//! vectors-file hash, Unicode version), then verifies. Fixture files are
//! emitted by the §1.5 canonical serializer (`ckc_core::canon`); serde_json
//! is the validating reader.

use std::fs;
use std::path::PathBuf;

use ckc_core::canon::{
    CanonError, Canonical, ObjectEmitter, canonical_payload_bytes, emit_array, emit_string,
};
use ckc_core::policy::{
    StringPolicy, UnicodePolicyManifest, normalization_table_fingerprint,
    punctuation_table_fingerprint, unicode_version_string,
};
use ckc_core::scalar::Hash;
use serde::Deserialize;

/// Appendix A.1 source strings U1..U27, bytes extracted verbatim from SPEC.md.
const A1_SOURCES: [&str; 27] = [
    "成人の敗血症が疑われる場合，βラクタム系抗菌薬を速やかに投与することを推奨する。",
    "βラクタム系抗菌薬に対するアナフィラキシーの既往がある患者には投与しない。",
    "バイタル判定表: 収縮期血圧 < 90 なら緊急対応; 収縮期血圧 <= 90 なら通常対応。",
    "成人の敗血症が疑われる場合，βラクタム系抗菌薬を避けることを推奨する。",
    "βラクタム投与記録には出典ノードを必ず付与する。",
    "βラクタム投与記録では出典ノードを空欄にする。",
    "同一トリアージ条件は，収縮期血圧 < 90 かつ 収縮期血圧 >= 90 とする。",
    "薬剤Xを投与する。",
    "成人の敗血症が疑われる場合，βラクタム系抗菌薬を静脈内で速やかに投与することを推奨する。",
    "成人の敗血症が疑われる場合，βラクタム系抗菌薬を経口で速やかに投与することを推奨する。",
    "成人の敗血症が疑われる場合，βラクタム系抗菌薬を静脈内で通常速度で投与することを推奨する。",
    "成人の敗血症が疑われる場合，セファゾリンを投与することを推奨する。",
    "成人の敗血症が疑われる場合，βラクタム系抗菌薬を投与することを推奨する。",
    "表1「バイタル判定表」を参照する。",
    "表1 キャプション: バイタル判定表。",
    "βラクタム系抗菌薬*。脚注*: 本例では fixture 用語である。",
    "腎機能に応じてβラクタム系抗菌薬の用量を調整する。",
    "表99「存在しない表」を参照する。",
    "未整形表: 収縮期血圧 < 80; 出力列なし。",
    "成人の敗血症が疑われる場合，未知薬Yを投与する。",
    "成人の敗血症が疑われる場合，βラクタム系抗菌薬を試験経路で投与する。",
    "画像由来の不鮮明な注記「投与量未確認」。",
    "患者データから敗血症リスクを予測して抗菌薬を選択する。",
    "成人の敗血症が疑われる場合，βラクタム系抗菌薬の投与を考慮し，状況により避ける。",
    "表在感染を合併する場合は追加評価する。",
    "βラクタム系抗菌薬の投与量を高用量とする。",
    "βラクタム系抗菌薬の投与量を標準用量とする。",
];

/// Edge seeds: whitespace set, punctuation fold table, NFKC compatibility
/// forms, and per-policy contrast cases.
const EDGE_SEEDS: &[(&str, &str, &str)] = &[
    (
        "raw_source",
        "edge-raw-controls",
        "α\u{0009}β\u{3000}γ\r\n ，。",
    ),
    ("raw_source", "edge-raw-a1-u01", A1_SOURCES[0]),
    ("source_nfkc", "edge-halfwidth-kana", "ﾊﾞｲﾀﾙｻｲﾝ"),
    ("source_nfkc", "edge-compat-forms", "①㎎％㈱Ⅸﬁ²"),
    (
        "source_nfkc",
        "edge-combining-voiced",
        "カ\u{3099}キ\u{3099}",
    ),
    (
        "semantic_ja",
        "edge-ws-all",
        "\u{0009}a\u{000A}b\u{000B}c\u{000C}d\u{000D}e\u{00A0}f\u{1680}g\u{2000}h\u{2005}i\u{200A}j\u{2028}k\u{2029}l\u{202F}m\u{205F}n\u{3000}o \u{3000} ",
    ),
    (
        "semantic_ja",
        "edge-punct-all",
        "、。【】（）－‐‑‒–—―−≤≦≥≧＜＞：；，．",
    ),
    ("semantic_ja", "edge-empty", ""),
    ("semantic_ja", "edge-only-ws", "\u{3000} \u{0009}"),
    (
        "semantic_ja",
        "edge-fullwidth-ascii",
        "ＣＲＰ値＝１０ｍｇ／ｄＬ",
    ),
    ("semantic_en", "edge-en-fold", "Sepsis — β-Lactam ≤ 90 mmHg"),
    (
        "diagnostic_text",
        "edge-diag-preserve",
        "行３：\u{3000}値「未確認」。",
    ),
    (
        "template_literal",
        "edge-template-slots",
        "用量 {dose_value} を {route} で投与（固定）１",
    ),
    ("view_text", "edge-view", "βラクタム（注）⚠"),
    ("identifier_ascii", "edge-ident-cli", "runs/m0"),
    ("identifier_ascii", "edge-ident-version", "16.0.0"),
];

/// Canonical fixture rows in sorted vector order; emission via `Canonical`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    expected: String,
    input: String,
    policy: String,
    vector_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorFile {
    vectors: Vec<Vector>,
}

impl Canonical for Vector {
    const TYPE_ID: &'static str = "policy_vector";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        for (name, value) in [
            ("expected", &self.expected),
            ("input", &self.input),
            ("policy", &self.policy),
            ("vector_id", &self.vector_id),
        ] {
            obj.member(name, |b| {
                emit_string(b, value);
                Ok(())
            })?;
        }
        obj.finish(out)
    }
}

impl Canonical for VectorFile {
    const TYPE_ID: &'static str = "policy_vector_file";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("vectors", |b| {
            emit_array(b, &self.vectors, |b, v| v.emit_canonical(b))
        })?;
        obj.finish(out)
    }
}

fn seeds() -> Vec<(StringPolicy, String, String)> {
    let mut out = Vec::new();
    for (i, source) in A1_SOURCES.iter().enumerate() {
        let n = i + 1;
        out.push((
            StringPolicy::SemanticJa,
            format!("a1-u{n:02}-semantic-ja"),
            (*source).to_owned(),
        ));
        out.push((
            StringPolicy::SourceNfkc,
            format!("a1-u{n:02}-source-nfkc"),
            (*source).to_owned(),
        ));
    }
    for (policy_id, vector_id, input) in EDGE_SEEDS {
        let policy = StringPolicy::from_id(policy_id).expect("seed policy id resolves");
        out.push((policy, (*vector_id).to_owned(), (*input).to_owned()));
    }
    out.sort_by(|a, b| (a.0.id(), &a.1).cmp(&(b.0.id(), &b.1)));
    out
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn vectors_path() -> PathBuf {
    fixtures_dir().join("policy_vectors.json")
}

fn manifest_path() -> PathBuf {
    fixtures_dir().join("unicode_policy_manifest.json")
}

fn bless() {
    let vectors = seeds()
        .into_iter()
        .map(|(policy, vector_id, input)| {
            let expected = policy
                .normalize(&input)
                .unwrap_or_else(|e| panic!("seed {vector_id} must normalize: {e}"));
            Vector {
                expected,
                input,
                policy: policy.id().to_owned(),
                vector_id,
            }
        })
        .collect();
    let vector_bytes = canonical_payload_bytes(&VectorFile { vectors }).expect("vector file emits");
    fs::create_dir_all(fixtures_dir()).expect("fixtures dir");
    fs::write(vectors_path(), &vector_bytes).expect("write policy_vectors.json");

    let manifest = UnicodePolicyManifest {
        manifest_id: ckc_core::scalar::Id::new("upm-m0").expect("valid id"),
        normalization_table_hash: normalization_table_fingerprint(),
        policy_test_hash: Hash::of_bytes(&vector_bytes),
        punctuation_table_hash: punctuation_table_fingerprint(),
        unicode_version: ckc_core::policy::Text::from_canonical(&unicode_version_string())
            .expect("unicode version is identifier_ascii"),
    };
    let manifest_bytes = canonical_payload_bytes(&manifest).expect("manifest emits");
    fs::write(manifest_path(), &manifest_bytes).expect("write unicode_policy_manifest.json");
}

#[test]
fn t_unicode_idempotency() {
    if std::env::var_os("CKC_BLESS").is_some() {
        bless();
    }

    let vector_bytes = fs::read(vectors_path()).expect("read policy_vectors.json");
    let vector_file: VectorFile =
        serde_json::from_slice(&vector_bytes).expect("parse policy_vectors.json");

    // Committed file is the canonical serialization of its own parse, and
    // rows are sorted by (policy, vector_id) with unique ids.
    assert_eq!(
        canonical_payload_bytes(&vector_file).expect("re-emit"),
        vector_bytes,
        "policy_vectors.json bytes are non-canonical"
    );
    assert!(
        vector_file
            .vectors
            .windows(2)
            .all(|w| (&w[0].policy, &w[0].vector_id) < (&w[1].policy, &w[1].vector_id)),
        "vectors must be strictly sorted by (policy, vector_id)"
    );

    // Committed vectors cover exactly the seed list (stale-file detector).
    let file_rows: Vec<(String, String, String)> = vector_file
        .vectors
        .iter()
        .map(|v| (v.policy.clone(), v.vector_id.clone(), v.input.clone()))
        .collect();
    let seed_rows: Vec<(String, String, String)> = seeds()
        .into_iter()
        .map(|(p, id, input)| (p.id().to_owned(), id, input))
        .collect();
    assert_eq!(
        seed_rows, file_rows,
        "seed list and fixture rows diverge; re-bless"
    );

    // Gate core: expected output, idempotency, and byte-stability per vector.
    for v in &vector_file.vectors {
        let policy = StringPolicy::from_id(&v.policy)
            .unwrap_or_else(|| panic!("unknown policy {}", v.policy));
        let once = policy
            .normalize(&v.input)
            .unwrap_or_else(|e| panic!("{} normalize: {e}", v.vector_id));
        assert_eq!(once, v.expected, "{}: expected output", v.vector_id);
        let twice = policy
            .normalize(&once)
            .unwrap_or_else(|e| panic!("{} renormalize: {e}", v.vector_id));
        assert_eq!(twice, once, "{}: idempotency", v.vector_id);
        let again = policy
            .normalize(&v.input)
            .unwrap_or_else(|e| panic!("{} repeat: {e}", v.vector_id));
        assert_eq!(again, once, "{}: byte-stability", v.vector_id);
    }

    // Manifest fields recompute exactly.
    let manifest_bytes = fs::read(manifest_path()).expect("read unicode_policy_manifest.json");
    let manifest: UnicodePolicyManifest =
        serde_json::from_slice(&manifest_bytes).expect("parse unicode_policy_manifest.json");
    assert_eq!(manifest.manifest_id.as_str(), "upm-m0");
    assert_eq!(manifest.unicode_version.as_str(), unicode_version_string());
    assert_eq!(
        manifest.normalization_table_hash,
        normalization_table_fingerprint(),
        "NFKC table fingerprint drift"
    );
    assert_eq!(
        manifest.punctuation_table_hash,
        punctuation_table_fingerprint(),
        "fold table fingerprint drift"
    );
    assert_eq!(
        manifest.policy_test_hash,
        Hash::of_bytes(&vector_bytes),
        "policy_test_hash does not match policy_vectors.json bytes"
    );
    assert_eq!(
        canonical_payload_bytes(&manifest).expect("re-emit"),
        manifest_bytes,
        "unicode_policy_manifest.json bytes are non-canonical"
    );
}
