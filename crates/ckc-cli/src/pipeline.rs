//! Run-pipeline stages (SPEC 18): load the bundle, then drive compile / verify /
//! conflicts / substrate emission and the `demo` orchestration. Task 0.11.1
//! lands the load entry point and the stage signatures; tasks 0.11.2–0.11.6
//! fill in each stage body.

use std::path::Path;

use anyhow::bail;

use crate::emit::write_artifact;
use crate::manifest::{RunManifest, RunManifestEntry};
use ckc_compile::{ARTIFACT_PATHS, CompileBundle, compile_all, portfolio_manifest};
use ckc_conflict::{conflict_manifest, detect_all};
use ckc_core::canonical::{content_hash, to_canonical_bytes};
use ckc_retrieve::{RetrievalQuery, RetrievalResult, SparseIndex};
use ckc_term::TerminologyGraph;
use ckc_term::rdf::export_skos_turtle;
use ckc_term::shacl::validate_rules;
use ckc_verify::{VerificationReport, verification_manifest, verify_all};

/// The recorded cvc5 proof artifact, embedded so [`run_verify`] can place it
/// beside the `cert_cvc5_norm_conflict` certificate that references its content
/// hash. It is recorded evidence (reached via the cert's `proof_artifact_hashes`),
/// not a `verify_all` artifact, so it carries no run-manifest entry of its own.
const CVC5_PROOF: &[u8] =
    include_bytes!("../../../examples/research_kernel/fixtures/cvc5_norm_conflict.proof");

/// The committed Phase-0 retrieval queries, embedded so [`run_substrate`] can
/// reproduce the `retrieval_results.json` snapshot without filesystem access.
const QUERIES_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/queries.json");

/// Top-k retrieval depth. Matches `ckc-retrieve/tests/persistence.rs` so the
/// emitted result set equals the committed snapshot.
const RETRIEVAL_TOP_K: usize = 5;

/// Load the [`CompileBundle`] for a run. Phase-0 serves only the committed
/// research-kernel toy bundle ([`CompileBundle::load_toy`]); any other path is a
/// not-yet-supported corpus and fails fast.
pub fn load_bundle(path: &str) -> anyhow::Result<CompileBundle> {
    match path {
        "examples/research_kernel" => Ok(CompileBundle::load_toy()),
        other => {
            bail!("unsupported bundle path {other:?}; Phase-0 serves examples/research_kernel")
        }
    }
}

/// Compile stage (task 0.11.2): emit the SPEC-14 target portfolio under `out_dir`.
///
/// Writes each [`compile_all`] target's `artifact_text` to its canonical
/// [`ARTIFACT_PATHS`] slot under `out_dir`, returning one `compile`-stage
/// [`RunManifestEntry`] per target. Each entry carries the `content_hash` that
/// [`portfolio_manifest`] byte-locks its target with, so the run manifest pins
/// every emitted artifact by hash.
pub fn run_compile(
    bundle: &CompileBundle,
    out_dir: &Path,
) -> anyhow::Result<Vec<RunManifestEntry>> {
    let targets = compile_all(bundle);
    let manifest = portfolio_manifest(bundle);
    let mut entries = Vec::with_capacity(targets.len());
    for ((target, rel), entry) in targets.iter().zip(ARTIFACT_PATHS).zip(manifest.0) {
        write_artifact(out_dir, rel, target.artifact_text.as_bytes())?;
        entries.push(RunManifestEntry {
            stage: "compile".to_string(),
            artifact_kind: "compiled_target".to_string(),
            artifact_path: rel.to_string(),
            content_hash: entry.content_hash,
        });
    }
    Ok(entries)
}

/// Verify stage (task 0.11.3): emit the SPEC-12/13 verification artifact set under
/// `out_dir`, returning the in-memory report alongside its run-manifest entries.
///
/// Writes each [`Certificate`](ckc_verify::Certificate) and
/// [`ExecutionWitness`](ckc_verify::ExecutionWitness) as `to_canonical_bytes` to
/// its `certs/{certificates,witnesses}/<id>.json` slot, the certificate graph and
/// assurance seed to their `certs/` files, and the recorded [`CVC5_PROOF`] beside
/// them — mirroring task 0.9.15's `report_artifacts` layout byte-for-byte. The
/// returned `verify`-stage entries come from [`verification_manifest`], so each
/// entry's `content_hash` byte-locks the file emitted at its path (24 entries: 11
/// certificates, 11 witnesses, the graph, the assurance seed); the proof copy is
/// recorded evidence and carries no entry.
pub fn run_verify(
    bundle: &CompileBundle,
    out_dir: &Path,
) -> anyhow::Result<(VerificationReport, Vec<RunManifestEntry>)> {
    let report = verify_all(bundle);
    for cert in &report.certificates {
        let rel = format!("certs/certificates/{}.json", cert.certificate_id.as_str());
        write_artifact(out_dir, &rel, &to_canonical_bytes(cert))?;
    }
    for witness in &report.witnesses {
        let rel = format!("certs/witnesses/{}.json", witness.witness_id.as_str());
        write_artifact(out_dir, &rel, &to_canonical_bytes(witness))?;
    }
    write_artifact(
        out_dir,
        "certs/certificate_graph.json",
        &to_canonical_bytes(&report.graph),
    )?;
    write_artifact(
        out_dir,
        "certs/assurance_seed.json",
        &to_canonical_bytes(&report.assurance),
    )?;
    write_artifact(out_dir, "certs/cvc5_norm_conflict.proof", CVC5_PROOF)?;

    let entries = verification_manifest(bundle)
        .0
        .into_iter()
        .map(|e| RunManifestEntry {
            stage: "verify".to_string(),
            artifact_kind: e.artifact_kind,
            artifact_path: e.artifact_path,
            content_hash: e.content_hash,
        })
        .collect();
    Ok((report, entries))
}

/// Conflicts stage (task 0.11.4): emit the SPEC-15 conflict artifact set under
/// `out_dir`, detected over the verification `report`.
///
/// Writes each [`Conflict`](ckc_conflict::Conflict) as `to_canonical_bytes` to its
/// `certs/conflicts/<conflict_id>.json` slot and each
/// [`ArgumentGraph`](ckc_conflict::ArgumentGraph) to
/// `certs/argument_graphs/<argument_graph_id>.json` — mirroring task 0.10.9's
/// `report_artifacts` layout byte-for-byte. The returned `conflicts`-stage entries
/// come from [`conflict_manifest`], so each entry's `content_hash` byte-locks the
/// file emitted at its path (4 entries: 3 conflicts, 1 argument graph).
pub fn run_conflicts(
    bundle: &CompileBundle,
    report: &VerificationReport,
    out_dir: &Path,
) -> anyhow::Result<Vec<RunManifestEntry>> {
    let detected = detect_all(bundle, report);
    for conflict in &detected.conflicts {
        let rel = format!("certs/conflicts/{}.json", conflict.conflict_id.as_str());
        write_artifact(out_dir, &rel, &to_canonical_bytes(conflict))?;
    }
    for graph in &detected.argument_graphs {
        let rel = format!(
            "certs/argument_graphs/{}.json",
            graph.argument_graph_id.as_str()
        );
        write_artifact(out_dir, &rel, &to_canonical_bytes(graph))?;
    }

    let entries = conflict_manifest(bundle, report)
        .0
        .into_iter()
        .map(|e| RunManifestEntry {
            stage: "conflicts".to_string(),
            artifact_kind: e.artifact_kind,
            artifact_path: e.artifact_path,
            content_hash: e.content_hash,
        })
        .collect();
    Ok(entries)
}

/// Substrate stage (task 0.11.5): emit the terminology, validation, and
/// retrieval substrate under `out_dir`, returning three `substrate`-stage
/// entries whose `content_hash` pins each emitted artifact in-memory.
///
/// - `terminology.ttl`: the SKOS/Turtle [`export_skos_turtle`] of the bundle's
///   concepts, written as raw Turtle bytes (byte-identical to the committed
///   `gen_fixtures` export — the export sorts by `concept_id`, so building the
///   graph from the in-memory bundle reproduces it exactly).
/// - `shacl_report.json`: the canonical-JSON [`validate_rules`] report over the
///   bundle's rules (an accepted CKC artifact, SPEC 5.2).
/// - `retrieval/retrieval_results.json`: the canonical-JSON `Vec<RetrievalResult>`
///   from running the committed `queries.json` over a [`SparseIndex`]. The span
///   set is sorted by `span_id` first so the corpus hash and index fingerprint
///   match the `ckc-retrieve/tests/persistence.rs` snapshot; BM25 ranking is
///   itself order-invariant (stable score/`span_id` sort).
pub fn run_substrate(
    bundle: &CompileBundle,
    out_dir: &Path,
) -> anyhow::Result<Vec<RunManifestEntry>> {
    // (a) RDF/SKOS terminology export.
    let mut graph = TerminologyGraph::new();
    for concept in &bundle.concepts {
        graph.insert(concept.clone());
    }
    let turtle = export_skos_turtle(&graph);
    write_artifact(out_dir, "terminology.ttl", turtle.as_bytes())?;

    // (b) SHACL rule-provenance report.
    let shacl = validate_rules(&bundle.rules);
    write_artifact(out_dir, "shacl_report.json", &to_canonical_bytes(&shacl))?;

    // (c) Sparse BM25 retrieval over the committed queries.
    let mut spans = bundle.spans.clone();
    spans.sort_by(|a, b| a.span_id.as_str().cmp(b.span_id.as_str()));
    let index = SparseIndex::build_from_spans(&spans)?;
    let index_fingerprint = index.fingerprint().clone();
    let corpus_hash = content_hash(&spans);
    let queries: Vec<RetrievalQuery> =
        serde_json::from_str(QUERIES_JSON).expect("toy queries.json must deserialize");
    let mut results = Vec::with_capacity(queries.len());
    for query in &queries {
        let hits = index.search(&query.query_text, RETRIEVAL_TOP_K)?;
        results.push(RetrievalResult {
            query: query.clone(),
            hits,
            index_fingerprint: index_fingerprint.clone(),
            corpus_hash: corpus_hash.clone(),
        });
    }
    write_artifact(
        out_dir,
        "retrieval/retrieval_results.json",
        &to_canonical_bytes(&results),
    )?;

    Ok(vec![
        RunManifestEntry {
            stage: "substrate".to_string(),
            artifact_kind: "rdf_export".to_string(),
            artifact_path: "terminology.ttl".to_string(),
            content_hash: content_hash(&turtle),
        },
        RunManifestEntry {
            stage: "substrate".to_string(),
            artifact_kind: "shacl_report".to_string(),
            artifact_path: "shacl_report.json".to_string(),
            content_hash: content_hash(&shacl),
        },
        RunManifestEntry {
            stage: "substrate".to_string(),
            artifact_kind: "retrieval_result".to_string(),
            artifact_path: "retrieval/retrieval_results.json".to_string(),
            content_hash: content_hash(&results),
        },
    ])
}

/// Canonical Phase-0 demo command recorded in every [`RunManifest`]. Held
/// independent of the concrete `out_dir` — it names the clap `--out` default and
/// the SPEC §25 invocation — so the manifest hash stays stable across output
/// locations, the cross-run determinism the demo gate and Phase-0 acceptance
/// require. (Standalone `ckc replay` against a committed `runs/research` manifest
/// is task 0.13.)
const DEMO_COMMAND: &str = "ckc demo research-kernel --out runs/research";

/// Run the full Phase-0 stage pipeline under `out_dir` and assemble the
/// [`RunManifest`], without writing the manifest file itself. [`run_demo`] calls
/// this once per pass so `--replay` can re-derive and hash-compare a second,
/// independent pass over the same stages.
///
/// Entries concatenate in pipeline order — compile (9) ++ verify (24) ++
/// conflicts (4) ++ substrate (3) — so the manifest pins every emitted artifact
/// by content hash, stage-ordered.
fn run_pipeline(scenario: &str, out_dir: &Path) -> anyhow::Result<RunManifest> {
    if scenario != "research-kernel" {
        bail!("unsupported demo scenario {scenario:?}; Phase-0 serves research-kernel");
    }
    let bundle = load_bundle("examples/research_kernel")?;
    let mut entries = run_compile(&bundle, out_dir)?;
    let (report, verify_entries) = run_verify(&bundle, out_dir)?;
    entries.extend(verify_entries);
    entries.extend(run_conflicts(&bundle, &report, out_dir)?);
    entries.extend(run_substrate(&bundle, out_dir)?);
    Ok(RunManifest {
        command: DEMO_COMMAND.to_string(),
        producer_version: env!("CARGO_PKG_VERSION").to_string(),
        entries,
    })
}

/// Demo orchestration (task 0.11.6): run every Phase-0 stage under `out_dir`,
/// assemble the deterministic [`RunManifest`], write it to
/// `out_dir/run_manifest.json`, and — when `replay` — re-run the whole pipeline
/// a second time and fail if the second pass hashes differently. The replay pass
/// re-emits into the same `out_dir` (idempotent, byte-identical writes), so a
/// clean replay proves the stages are reproducible end to end.
pub fn run_demo(scenario: &str, replay: bool, out_dir: &Path) -> anyhow::Result<RunManifest> {
    let manifest = run_pipeline(scenario, out_dir)?;
    write_artifact(out_dir, "run_manifest.json", &to_canonical_bytes(&manifest))?;
    if replay {
        let replayed = run_pipeline(scenario, out_dir)?;
        let (got, want) = (content_hash(&replayed), content_hash(&manifest));
        if got != want {
            bail!("replay hash mismatch: second pass {got} != first {want}");
        }
    }
    Ok(manifest)
}
