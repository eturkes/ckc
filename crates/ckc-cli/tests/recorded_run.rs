//! run-m2.2c — recorded-run pin battery + replay coverage (deterministic,
//! runtime-absent). Executes `exp.m2_multihop` in a temp invocation root
//! built from the committed registry + corpus + schemas + prompts plus the
//! repo's live-recorded `/cassettes/**` (run-m2.2b) — gated by `registry
//! check` over that root (§8.5 item 2), the drift pin tying the committed
//! schema/prompt payload bytes to their declared registry hashes — then
//! pins the §9 measurement surface from the observed run: the honest weak-baseline
//! failure census (report.json failure_taxonomy + metrics emission order),
//! both manifests' §9 seven-tuple, the rendered bodies re-rendered from the
//! landed report.json (no const body pin — the solver version is
//! env-dependent and the recorded model identity stays out of test code:
//! identity asserts are EQUALITY against the committed cassettes' recorded
//! identity, never a literal), and `replay::execute` over the finished run
//! (`matched()` — the model-artifact replay coverage).

use std::path::Path;
use std::process::Command;

use ckc_cli::metrics::{K_SAMPLE_CONVERGENCE, MetricValue, MetricsSection};
use ckc_cli::replay;
use ckc_cli::report::{Report, render_markdown, render_markdown_ja};
use ckc_cli::trace::{LineageIndex, TraceBundle};
use ckc_core::{
    ArtifactWrapper, CanonRead, Canonical, CassettePayload, DiagnosticRecord, Hash, Outcome,
    ReplayManifest, RunManifest, SegmentIr, SourceDocumentGraph, TotalOperationResult, hash_bytes,
    read_jsonl, read_strict_canonical,
};

/// Repository root: two levels above the ckc-cli manifest, where the §3
/// `registry/`, `corpus/`, and recorded `cassettes/` trees live.
fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}

/// §8.5 per-artifact bar (run_oracle pattern): strict canonical read of the
/// wrapper bytes, then §4.4 re-validation.
fn strict_read<P: Canonical + CanonRead>(path: &Path) -> ArtifactWrapper<P> {
    let bytes = std::fs::read(path).unwrap();
    let wrapper: ArtifactWrapper<P> = read_strict_canonical(&bytes)
        .unwrap_or_else(|e| panic!("{}: strict read: {e:?}", path.display()));
    wrapper
        .validate()
        .unwrap_or_else(|e| panic!("{}: wrapper invariant: {e:?}", path.display()));
    wrapper
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Temp invocation root = the committed registry (index files + the
/// schema/prompt payload files they declare) + locked corpus inputs
/// (`write_m2_root` pattern) + the repo's recorded experiment cassettes.
fn write_recorded_root(root: &Path) {
    let repo = repo_root();
    for rel in [
        "registry/corpora.yaml",
        "registry/candidates.yaml",
        "registry/experiments.yaml",
        "registry/schemas.yaml",
        "registry/prompts.yaml",
        "registry/prompts/direct_smt.txt",
        "registry/prompts/single_ir.txt",
        "schemas/clinical_ir.schema.json",
        "schemas/smt_query.grammar",
        "rust-toolchain.toml",
        "Cargo.lock",
        "corpus/lexicon/ja_core.yaml",
        "corpus/test_sources/m1_guideline_a.html",
        "corpus/test_sources/m1_guideline_b.html",
        "corpus/test_sources/m1_control.html",
        "corpus/reference/m1_expected.yaml",
    ] {
        let target = root.join(rel);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(repo.join(rel), target).unwrap();
    }
    copy_tree(&repo.join("cassettes"), &root.join("cassettes"));
}

/// Sorted entry names of one directory.
fn listing(path: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

const DOC_IDS: [&str; 3] = [
    "test_source.m1_control",
    "test_source.m1_guideline_a",
    "test_source.m1_guideline_b",
];

// The recorded weak-baseline run is all-terminal on every fill point
// (run-m2.2b census): every attempt rejects `ai_schema_violation`, every
// fill point exhausts its repair budget, and single_ir's two groups fail
// compile over member-short groups. All values below are blessed from the
// observed run.
#[test]
fn recorded_run_pins_m2_sections_manifests_renders_and_replays() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    write_recorded_root(&root);

    // §8.5 item 2 over the committed registry: `registry check` re-hashes
    // each schema payload file and prompt template against its declared
    // registry hash — the standing drift pin for the payload files the
    // replay-mode run below never opens (only their DECLARED hashes ride
    // the §9 manifest slots).
    let registry_out = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args(["registry", "check"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(
        registry_out.status.code(),
        Some(0),
        "committed registry + payload bytes self-consistent; stderr: {}",
        String::from_utf8_lossy(&registry_out.stderr)
    );

    let run_dir = tmp.path().join("run");
    let out = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args([
            "run",
            "--experiment",
            "exp.m2_multihop",
            "--out",
            run_dir.to_str().unwrap(),
        ])
        .current_dir(&root)
        .output()
        .unwrap();

    // §4.4: the all-terminal census folds to an outcome-mapped Invalid
    // (exit 2), not an abort — the run completes and lands its whole tail.
    assert_eq!(
        out.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let results: Vec<TotalOperationResult> = read_jsonl(&out.stdout).unwrap();
    assert_eq!(results.len(), 1, "stdout carries exactly one result line");
    assert_eq!(results[0].outcome, Outcome::Invalid);

    // Landing census: the full run-level tail lands beside the two route
    // subtrees; terminal fills land no accepted target artifact, so each
    // route holds exactly its three deterministic document heads — no
    // ir_bundle, no groups/ dir on either route.
    assert_eq!(
        listing(&run_dir),
        [
            "lineage_index.json",
            "logs",
            "manifest.json",
            "replay_manifest.json",
            "report.json",
            "report_en.md",
            "report_ja.md",
            "routes",
            "trace_bundle.json"
        ]
    );
    assert_eq!(
        listing(&run_dir.join("routes")),
        ["pipe.m2_direct_smt", "pipe.m2_single_ir"]
    );
    for route in ["pipe.m2_direct_smt", "pipe.m2_single_ir"] {
        let rdir = run_dir.join("routes").join(route);
        assert_eq!(listing(&rdir), ["artifacts"], "{route}: groups/ absent");
        for doc in DOC_IDS {
            assert_eq!(
                listing(&rdir.join("artifacts").join(doc)),
                ["segments.json", "source_document_graph.json"],
                "{route} {doc}: head only, no accepted fill artifact"
            );
        }
    }

    // The heads are route-INDEPENDENT (deterministic extract + segment over
    // the same locked sources): each document's segments and graph carry the
    // same payload content hash on both routes, so the §4.3 attestation set
    // dedups the 12 landed head wrappers — to FIVE unique hashes, not six:
    // m1_control and m1_guideline_b share one segments payload (SegmentIr is
    // a structural classification; the text distinctions ride the graph,
    // whose three hashes are distinct). Consumed by the replay-census pin.
    let mut head_hashes: std::collections::BTreeSet<Hash> = std::collections::BTreeSet::new();
    let mut seg_hashes = std::collections::BTreeMap::new();
    let mut graph_hashes = std::collections::BTreeMap::new();
    for doc in DOC_IDS {
        let head = |route: &str, file: &str| {
            run_dir.join(format!("routes/{route}/artifacts/{doc}/{file}"))
        };
        let segments: [ArtifactWrapper<SegmentIr>; 2] = ["pipe.m2_direct_smt", "pipe.m2_single_ir"]
            .map(|route| strict_read(&head(route, "segments.json")));
        assert_eq!(
            segments[0].content_hash, segments[1].content_hash,
            "{doc}: cross-route segments parity"
        );
        head_hashes.insert(segments[0].content_hash.clone());
        seg_hashes.insert(doc, segments[0].content_hash.clone());
        let graphs: [ArtifactWrapper<SourceDocumentGraph>; 2] =
            ["pipe.m2_direct_smt", "pipe.m2_single_ir"]
                .map(|route| strict_read(&head(route, "source_document_graph.json")));
        assert_eq!(
            graphs[0].content_hash, graphs[1].content_hash,
            "{doc}: cross-route graph parity"
        );
        head_hashes.insert(graphs[0].content_hash.clone());
        graph_hashes.insert(doc, graphs[0].content_hash.clone());
    }
    // The collision pinned PAIRWISE: control ≡ guideline_b segments is the
    // one duplicate; the remaining five payloads (both segment classes, all
    // three graphs) are pairwise distinct, so the six inserts dedup to
    // exactly that five-set.
    assert_eq!(
        seg_hashes["test_source.m1_control"], seg_hashes["test_source.m1_guideline_b"],
        "control ≡ guideline_b segments payload"
    );
    let distinct: std::collections::BTreeSet<Hash> = [
        &seg_hashes["test_source.m1_control"],
        &seg_hashes["test_source.m1_guideline_a"],
        &graph_hashes["test_source.m1_control"],
        &graph_hashes["test_source.m1_guideline_a"],
        &graph_hashes["test_source.m1_guideline_b"],
    ]
    .into_iter()
    .cloned()
    .collect();
    assert_eq!(
        distinct.len(),
        5,
        "remaining head payloads pairwise distinct"
    );
    assert_eq!(head_hashes, distinct, "head census = exactly those five");

    // §7.4 ledger census: 10 ai_schema_violation (5 fill points × base +
    // repair attempt) → 5 repair_limit_exceeded + 2 schema_invalid
    // (single_ir's member-short group compiles).
    let diagnostics: Vec<DiagnosticRecord> =
        read_jsonl(&std::fs::read(run_dir.join("logs/diagnostics.jsonl")).unwrap()).unwrap();
    let mut code_census: std::collections::BTreeMap<&str, usize> =
        std::collections::BTreeMap::new();
    for d in &diagnostics {
        *code_census.entry(d.code.as_str()).or_default() += 1;
    }
    assert_eq!(
        code_census,
        std::collections::BTreeMap::from([
            ("ai_schema_violation", 10),
            ("repair_limit_exceeded", 5),
            ("schema_invalid", 2),
        ]),
        "recorded-run ledger code census (17 records, no other code)"
    );

    // The run trace still spans BOTH routes (their landed heads chain into
    // the one run bundle) while minting no claim — nothing compiled.
    let bundle = strict_read::<TraceBundle>(&run_dir.join("trace_bundle.json"));
    assert_eq!(
        bundle.payload.claims.len(),
        0,
        "no compiled group, no claim"
    );
    let node_paths: Vec<&str> = bundle
        .payload
        .nodes
        .iter()
        .map(|n| n.path.as_str())
        .collect();
    for route in ["pipe.m2_single_ir", "pipe.m2_direct_smt"] {
        assert!(
            node_paths
                .iter()
                .any(|p| p.starts_with(&format!("routes/{route}/"))),
            "run trace chains the {route} route"
        );
    }

    // Identity anchor: the committed cassettes' recorded evaluator identity,
    // read back through the copied root (EQUALITY anchor — the recorded
    // identity is machine-specific measurement data and stays out of test
    // code; the run's identity-agreement gate makes any one cassette
    // representative).
    let cassette = strict_read::<CassettePayload>(
        &root.join("cassettes/route.single_ir/test_source.m1_control/seed-42.json"),
    );
    let recorded_identity = cassette.payload.model_identity.clone();

    // report.json M2 sections (typed strict read; values blessed from the
    // observed run). Taxonomy: the full per-route §7.4 code maps.
    let report = strict_read::<Report>(&run_dir.join("report.json"));
    let payload = &report.payload;
    let taxonomy: Vec<(&str, Vec<(&str, u64)>)> = payload
        .failure_taxonomy
        .as_ref()
        .expect("model run populates failure_taxonomy")
        .routes
        .iter()
        .map(|(pid, codes)| {
            (
                pid.as_str(),
                codes
                    .iter()
                    .map(|(c, n)| (c.as_str(), *n))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        taxonomy,
        vec![
            (
                "pipe.m2_direct_smt",
                vec![("ai_schema_violation", 4), ("repair_limit_exceeded", 2)]
            ),
            (
                "pipe.m2_single_ir",
                vec![
                    ("ai_schema_violation", 6),
                    ("repair_limit_exceeded", 3),
                    ("schema_invalid", 2)
                ]
            ),
        ],
        "weak-baseline failure census, per route"
    );

    // Metrics: raw rows for both routes precede the lone baseline-delta
    // table (§9 raw-before-ranking), baselined on the direct route; a
    // single k=1 recorded draw reads k_sample_convergence not_applicable.
    let metrics = payload
        .metrics
        .as_ref()
        .expect("model run populates metrics");
    assert_eq!(
        metrics.baseline_pipeline_id.as_str(),
        "pipe.m2_direct_smt",
        "direct route is the §7.3 delta baseline"
    );
    let order: Vec<(&str, bool)> = metrics
        .emission_order()
        .iter()
        .map(|section| match section {
            MetricsSection::RawRows(r) => (r.pipeline_id.as_str(), false),
            MetricsSection::DeltaTable(d) => (d.pipeline_id.as_str(), true),
        })
        .collect();
    assert_eq!(
        order,
        vec![
            ("pipe.m2_direct_smt", false),
            ("pipe.m2_single_ir", false),
            ("pipe.m2_single_ir", true),
        ],
        "raw rows for both routes precede the lone baseline-delta table"
    );
    for route in &metrics.routes {
        let k = route
            .rows
            .iter()
            .find(|row| row.metric.as_str() == K_SAMPLE_CONVERGENCE)
            .expect("k_sample_convergence row present");
        assert_eq!(
            k.value,
            MetricValue::NotApplicable,
            "single k=1 draw: k_sample_convergence not_applicable on {}",
            route.pipeline_id.as_str()
        );
    }
    // §9 evaluator identity: the report attributes the run to the recorded
    // identity — equality against the cassette anchor.
    assert_eq!(
        payload.model_identity.as_ref(),
        Some(&recorded_identity),
        "report §9 identity equals the cassettes' recorded identity"
    );

    // Rendered bodies re-render byte-identically from the landed report.json
    // (run_oracle pattern — no const body pin: the solver version is
    // env-dependent and the recorded identity stays out of test code).
    let report_en = std::fs::read(run_dir.join("report_en.md")).unwrap();
    let report_ja = std::fs::read(run_dir.join("report_ja.md")).unwrap();
    assert_eq!(
        report_en,
        render_markdown(payload).into_bytes(),
        "report_en.md re-renders from the landed report.json"
    );
    assert_eq!(
        report_ja,
        render_markdown_ja(payload).into_bytes(),
        "report_ja.md re-renders from the landed report.json"
    );

    // §9 measurement record: both manifests carry the recorded identity
    // (equality), the four input hashes over THIS run's committed inputs —
    // blessed from the observed run and equal to the fixture-run literals in
    // `m2_route_loop_lands_both_routes_namespaced` (same committed inputs,
    // the cross-consistency check) — and model_hash/runtime_hash None (env
    // bare-name runtime commits no bytes). reference_hash is independently
    // recomputed from the copied reference bytes, anchoring that slot.
    let manifest: RunManifest =
        read_strict_canonical(&std::fs::read(run_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest.model_identity.as_ref(),
        Some(&recorded_identity),
        "run manifest §9 identity equals the cassettes' recorded identity"
    );
    assert!(
        manifest.model_hash.is_none() && manifest.runtime_hash.is_none(),
        "the env bare-name runtime commits no model/runtime bytes"
    );
    assert_eq!(
        [
            manifest.test_source_hash.as_ref().map(|h| h.as_str()),
            manifest.reference_hash.as_ref().map(|h| h.as_str()),
            manifest.schema_hash.as_ref().map(|h| h.as_str()),
            manifest.prompt_template_hash.as_ref().map(|h| h.as_str()),
        ],
        [
            Some("sha256:52023a235277950b672288f1c196550139e2c1a8c32a1c559509ad35aba0d7f8"),
            Some("sha256:7192125b87593d1731795a1757c8f37f417061dc15603595f4a9178aeefee82f"),
            Some("sha256:c814fdbdef45361a17fba8e924190b57eda8a4bab91f58e61a215d9a201497c6"),
            Some("sha256:98148a65544a88fac3b013e16fd42a29bbcbb7e361f62657bd5ef9d8c30b54af"),
        ],
        "run manifest §9 measurement hashes (test_source / reference / schema / prompt_template)"
    );
    let reference_bytes = std::fs::read(root.join("corpus/reference/m1_expected.yaml")).unwrap();
    assert_eq!(
        manifest.reference_hash.as_ref(),
        Some(&hash_bytes(&reference_bytes)),
        "reference_hash is the raw sha256 of the experiment's expected_outcomes"
    );
    let replay_manifest: ReplayManifest =
        read_strict_canonical(&std::fs::read(run_dir.join("replay_manifest.json")).unwrap())
            .unwrap();
    assert_eq!(
        (
            &replay_manifest.model_identity,
            &replay_manifest.test_source_hash,
            &replay_manifest.reference_hash,
            &replay_manifest.schema_hash,
            &replay_manifest.prompt_template_hash,
            &replay_manifest.model_hash,
            &replay_manifest.runtime_hash,
        ),
        (
            &manifest.model_identity,
            &manifest.test_source_hash,
            &manifest.reference_hash,
            &manifest.schema_hash,
            &manifest.prompt_template_hash,
            &manifest.model_hash,
            &manifest.runtime_hash,
        ),
        "replay manifest §9 record matches the run manifest"
    );

    // Replay coverage: re-execute the recorded run from the same root into a
    // fresh scratch — every accepted artifact reproduces byte-identically
    // from the committed cassettes (runtime-absent), and the re-execution
    // reaches the same §4.4 outcome. The attested set is pinned EXACTLY: the
    // 5 deduped head hashes + the 3 run-level tail wrappers (trace bundle,
    // lineage index, report) = the 8 artifacts of the run-m2.2b census.
    let scratch = tmp.path().join("replay-scratch");
    let check = replay::execute(&root, &run_dir, &scratch).unwrap();
    assert!(
        check.matched(),
        "replay reproduces every accepted artifact: missing {:?} unexpected {:?}",
        check.missing,
        check.unexpected
    );
    let lineage = strict_read::<LineageIndex>(&run_dir.join("lineage_index.json"));
    let mut attested = head_hashes;
    attested.insert(bundle.content_hash.clone());
    attested.insert(lineage.content_hash.clone());
    attested.insert(report.content_hash.clone());
    assert_eq!(attested.len(), 8, "attested accepted-artifact census");
    // All four carriers hold the same sorted set: the prior run's manifest,
    // the replay manifest it mirrors, and the replay check's expected +
    // re-run actual sides.
    let sorted: Vec<Hash> = attested.iter().cloned().collect();
    for (carrier, hashes) in [
        ("run manifest output_hashes", &manifest.output_hashes),
        (
            "replay manifest expected_output_hashes",
            &replay_manifest.expected_output_hashes,
        ),
        ("replay expected", &check.expected),
        ("replay re-run actual", &check.actual),
    ] {
        assert_eq!(
            hashes, &sorted,
            "{carrier} = deduped heads + run-level tail wrappers, exactly"
        );
    }
    assert_eq!(check.rerun_outcome, Outcome::Invalid, "re-run §4.4 outcome");
}
