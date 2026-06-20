//! SPEC §8.5 item 3 workspace oracle: execute `exp.m1_spine` into a temp
//! dir, sweep the run directory (exact §8.3 file set — a later-processing_stage
//! artifact entering the layout must join the sweep), strict-read every
//! accepted artifact with §4.4 re-validation, and assert the experiment's
//! reference entries over the verifier results — the code oracle behind §8.5
//! items 5 and 6. The [`report`] module pins the landed report surface
//! — `report.json`, its `report_en.md` rendering, and the §5/§4.6 manifest
//! pair — over its own recorded runs: the finding/null partition, the
//! quoted spans resolving to test_source bytes (§8.5 item 9's code oracle),
//! and the run's provenance facts.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ckc_cli::trace::{ConflictKind, LineageIndex, TraceBundle, TraceNodeKind};
use ckc_core::{
    ArtifactWrapper, CanonRead, Canonical, DiagnosticRecord, EventRecord, ReferenceEntry, Hash, Id,
    IrBundle, Normalization, Outcome, SegmentIr, SourceDocumentGraph, TotalOperationResult,
    parse_experiments, parse_reference, read_strict_canonical, read_jsonl,
};
use ckc_smt::{CompiledArtifact, SolverVerdict, VerifierCategory, VerifierResults};

/// Repository root: two levels above the ckc-cli manifest, where the §3
/// `registry/` and `corpus/` trees live.
fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}

fn id(text: &str) -> Id {
    text.parse().unwrap()
}

/// §8.5 item 3's per-artifact bar: strict canonical read of the wrapper
/// bytes, then §4.4 re-validation (content and policy hashes recomputed
/// from the payload).
fn strict_read<P: Canonical + CanonRead>(path: &Path) -> ArtifactWrapper<P> {
    let bytes = std::fs::read(path).unwrap();
    let wrapper: ArtifactWrapper<P> =
        read_strict_canonical(&bytes).unwrap_or_else(|e| panic!("{}: strict read: {e:?}", path.display()));
    wrapper
        .validate()
        .unwrap_or_else(|e| panic!("{}: wrapper invariant: {e:?}", path.display()));
    wrapper
}

/// Every file under `root`, as sorted root-relative paths.
fn files_under(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, root, out);
            } else {
                out.push(path.strip_prefix(root).unwrap().to_owned());
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// One reference entry against its group's compiled plan and verifier results.
fn assert_group_matches_reference(
    entry: &ReferenceEntry,
    compiled: &CompiledArtifact,
    results: &VerifierResults,
) {
    let gid = &entry.group_id;
    let contradictions: Vec<_> = results
        .results
        .iter()
        .filter(|r| r.category == VerifierCategory::SemanticContradiction)
        .collect();
    if entry.expected_outcome == id("semantic_contradiction") {
        // §8.5 item 5 oracle: exactly one contradiction, riding a
        // deontic-consistency query — M1's deontic_direction_conflict
        // kind (§6) — with the unsat core matching reference as a set.
        assert_eq!(contradictions.len(), 1, "{gid}: exactly one contradiction");
        let hit = contradictions[0];
        assert!(
            compiled
                .solver_query_plan
                .iter()
                .any(|p| p.deontic_consistency_query_id == hit.query_id),
            "{gid}: the contradiction rides a deontic-consistency query"
        );
        assert_eq!(
            entry.expected_conflict_kind,
            Some(id("deontic_direction_conflict")),
            "{gid}: a deontic Q2 contradiction is the deontic_direction_conflict kind"
        );
        let core: BTreeSet<Id> = hit
            .unsat_core
            .clone()
            .expect("an unsat verdict carries its core")
            .into_iter()
            .collect();
        assert_eq!(core, entry.expected_unsat_core, "{gid}: unsat core as a set");
    } else if entry.expected_outcome == id("semantic_no_conflict") {
        // §8.5 item 6 oracle: every query closed without a contradiction;
        // the documented null is a §6 Q1-unsat closure — the overlap query
        // answered unsat and the pair's deontic query never ran.
        assert!(contradictions.is_empty(), "{gid}: no contradiction");
        assert!(
            results
                .results
                .iter()
                .all(|r| r.category == VerifierCategory::SemanticNoConflict),
            "{gid}: every query closed semantic_no_conflict"
        );
        if entry.expected_no_conflict_result {
            let closed: Vec<_> = compiled
                .solver_query_plan
                .iter()
                .filter(|p| {
                    results.results.iter().any(|r| {
                        r.query_id == p.context_overlap_query_id
                            && r.verdict == Some(SolverVerdict::Unsat)
                    })
                })
                .collect();
            assert!(
                !closed.is_empty(),
                "{gid}: a pair closed as documented null"
            );
            for pair in &closed {
                assert!(
                    results
                        .results
                        .iter()
                        .all(|r| r.query_id != pair.deontic_consistency_query_id),
                    "{gid}: closed pair {} skipped its deontic query",
                    pair.pair_id
                );
            }
        }
    } else {
        panic!(
            "{gid}: unhandled expected_outcome {}",
            entry.expected_outcome
        );
    }
}

#[test]
fn run_oracle_strict_reads_artifacts_and_matches_reference() {
    let root = repo_root();
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join("m1");
    let out = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args([
            "run",
            "--experiment",
            "exp.m1_spine",
            "--out",
            run_dir.to_str().unwrap(),
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let results: Vec<TotalOperationResult> = read_jsonl(&out.stdout).unwrap();
    assert_eq!(results.len(), 1, "stdout carries exactly one result line");
    assert_eq!(results[0].outcome, Outcome::Ok);

    // Expectations resolve through the registries: the experiment names its
    // groups, documents, and reference file.
    let experiments = parse_experiments(
        &std::fs::read_to_string(root.join("registry/experiments.yaml")).unwrap(),
    )
    .unwrap();
    let exp = experiments
        .iter()
        .find(|e| e.id == id("exp.m1_spine"))
        .unwrap();
    let reference: Vec<ReferenceEntry> =
        parse_reference(&std::fs::read_to_string(root.join(&exp.expected_outcomes)).unwrap()).unwrap();
    assert_eq!(
        reference.len(),
        exp.test_source_groups.len(),
        "one reference entry per test_source group"
    );

    let mut expected_files: Vec<PathBuf> = vec![
        "lineage_index.json".into(),
        "logs/diagnostics.jsonl".into(),
        "logs/events.jsonl".into(),
        "manifest.json".into(),
        "replay_manifest.json".into(),
        "report.json".into(),
        "report_en.md".into(),
        "trace_bundle.json".into(),
    ];

    // Document artifacts: the four §8.3 per-document layers strict-read.
    let documents: BTreeSet<&Id> = exp
        .test_source_groups
        .iter()
        .flat_map(|g| &g.test_sources)
        .collect();
    for doc in &documents {
        let dir = PathBuf::from("artifacts").join(doc.to_string());
        let _: ArtifactWrapper<SourceDocumentGraph> =
            strict_read(&run_dir.join(dir.join("source_document_graph.json")));
        let _: ArtifactWrapper<SegmentIr> = strict_read(&run_dir.join(dir.join("segments.json")));
        let _: ArtifactWrapper<Normalization> =
            strict_read(&run_dir.join(dir.join("normalization.json")));
        let _: ArtifactWrapper<IrBundle> = strict_read(&run_dir.join(dir.join("ir_bundle.json")));
        for name in [
            "ir_bundle.json",
            "normalization.json",
            "segments.json",
            "source_document_graph.json",
        ] {
            expected_files.push(dir.join(name));
        }
    }

    // Group artifacts: compiled + verifier results strict-read, every
    // materialized query byte-identical to its compiled body, reference asserted.
    for group in &exp.test_source_groups {
        let dir = PathBuf::from("groups").join(group.group_id.to_string());
        let compiled: ArtifactWrapper<CompiledArtifact> =
            strict_read(&run_dir.join(dir.join("compiled.json")));
        let verifier: ArtifactWrapper<VerifierResults> =
            strict_read(&run_dir.join(dir.join("verifier_results.json")));
        expected_files.push(dir.join("compiled.json"));
        expected_files.push(dir.join("verifier_results.json"));
        for body in &compiled.payload.query_bodies {
            let rel = dir.join("smt").join(format!("{}.smt2", body.query_id));
            let bytes = std::fs::read(run_dir.join(&rel)).unwrap();
            assert_eq!(
                bytes,
                body.body.as_bytes(),
                "{}: materialized query diverges from its compiled body",
                rel.display()
            );
            expected_files.push(rel);
        }
        let entry = reference
            .iter()
            .find(|e| e.group_id == group.group_id)
            .unwrap_or_else(|| panic!("{}: no reference entry", group.group_id));
        assert_group_matches_reference(entry, &compiled.payload, &verifier.payload);
    }

    // The report joins the §8.5 item 3 bar; its content pins live in the
    // `report` module over its own recorded run.
    let _: ArtifactWrapper<ckc_cli::report::Report> = strict_read(&run_dir.join("report.json"));

    // Trace artifacts: the §7.1 pair strict-read from the run root, both
    // wrapped by the trace component over the DAG's node content-hash
    // set; the §8.6 finding row and the hashless report node pin the
    // claim surface.
    let trace: ArtifactWrapper<TraceBundle> = strict_read(&run_dir.join("trace_bundle.json"));
    let lineage: ArtifactWrapper<LineageIndex> = strict_read(&run_dir.join("lineage_index.json"));
    trace.payload.validate().unwrap();
    lineage.payload.validate().unwrap();
    let mut node_hashes: Vec<Hash> = trace
        .payload
        .nodes
        .iter()
        .filter_map(|n| n.content_hash.clone())
        .collect();
    node_hashes.sort();
    node_hashes.dedup();
    assert_eq!(trace.schema_id, id("schema.trace_bundle"));
    assert_eq!(trace.artifact_id, id("trace_bundle"));
    assert_eq!(trace.artifact_kind, id("trace_bundle"));
    assert_eq!(trace.producer.pipeline_step_id, id("processing_stage.m1.trace"));
    assert_eq!(trace.input_hashes, node_hashes);
    assert_eq!(lineage.schema_id, id("schema.lineage_index"));
    assert_eq!(lineage.artifact_id, id("lineage_index"));
    assert_eq!(lineage.artifact_kind, id("lineage_index"));
    assert_eq!(lineage.producer.pipeline_step_id, id("processing_stage.m1.trace"));
    assert_eq!(lineage.input_hashes, node_hashes);

    // The DAG sink: exactly one report node, the only hashless one.
    let reports: Vec<_> = trace
        .payload
        .nodes
        .iter()
        .filter(|n| n.kind == TraceNodeKind::Report)
        .collect();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].node_id, id("report"));
    assert_eq!(reports[0].path, "report.json");
    assert_eq!(reports[0].content_hash, None);
    assert!(
        trace
            .payload
            .nodes
            .iter()
            .all(|n| n.content_hash.is_some() || n.kind == TraceNodeKind::Report)
    );

    // The §8.6 finding row: the conflict group's deontic claim cites the
    // cross-document core, both rules, their regions, and the report.
    let finding = trace
        .payload
        .claims
        .iter()
        .find(|c| c.finding_id == id("finding.group.m1_conflict.1"))
        .expect("the §8.6 finding row");
    assert_eq!(finding.group_id, id("group.m1_conflict"));
    assert_eq!(finding.query_id, id("q.m1_conflict.pair1.deontic"));
    assert_eq!(finding.category, VerifierCategory::SemanticContradiction);
    assert_eq!(finding.verdict, Some(SolverVerdict::Unsat));
    assert_eq!(
        finding.conflict_kind,
        Some(ConflictKind::DeonticDirectionConflict)
    );
    assert_eq!(
        finding.assertion_ids,
        vec![
            id("a.test_source.m1_guideline_a.rule.0"),
            id("a.test_source.m1_guideline_b.rule.0")
        ]
    );
    assert_eq!(
        finding.rule_ids,
        vec![
            id("test_source.m1_guideline_a.rule.0"),
            id("test_source.m1_guideline_b.rule.0")
        ]
    );
    assert_eq!(finding.region_ids, vec![id("r.2"), id("r.3")]);
    assert_eq!(finding.report_ref, id("report"));
    // Lineage covers the finding for both member documents.
    assert_eq!(
        lineage
            .payload
            .rows
            .iter()
            .filter(|r| r.finding_id == finding.finding_id)
            .count(),
        2
    );

    // §8.5 item 7 binary-level: `ckc trace` resolves the §8.6 finding to
    // the full chain in both directions, byte-pinned from the recorded
    // run; the result line follows the chain on stdout.
    let trace_out = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args([
            "trace",
            "--run",
            run_dir.to_str().unwrap(),
            "--finding",
            "finding.group.m1_conflict.1",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert_eq!(trace_out.status.code(), Some(0));
    let chain = "\
trace finding.group.m1_conflict.1 run m1
forward source spans -> segments -> statements -> rules -> named assertions -> solver verdict -> finding
document test_source.m1_guideline_a path corpus/test_sources/m1_guideline_a.html
  source spans: r.2 r.3
  segments: seg.2 seg.3
  statements: stmt.0
  rules: test_source.m1_guideline_a.rule.0
document test_source.m1_guideline_b path corpus/test_sources/m1_guideline_b.html
  source spans: r.2
  segments: seg.2
  statements: stmt.0
  rules: test_source.m1_guideline_b.rule.0
named assertions: a.test_source.m1_guideline_a.rule.0 a.test_source.m1_guideline_b.rule.0
solver verdict: unsat category semantic_contradiction conflict deontic_direction_conflict group group.m1_conflict pair q.m1_conflict.pair1 query q.m1_conflict.pair1.deontic
finding: finding.group.m1_conflict.1 report report
reverse finding -> solver verdict -> named assertions -> rules -> statements -> segments -> source spans
finding: finding.group.m1_conflict.1 report report
solver verdict: unsat category semantic_contradiction conflict deontic_direction_conflict group group.m1_conflict pair q.m1_conflict.pair1 query q.m1_conflict.pair1.deontic
named assertions: a.test_source.m1_guideline_a.rule.0 a.test_source.m1_guideline_b.rule.0
document test_source.m1_guideline_a path corpus/test_sources/m1_guideline_a.html
  rules: test_source.m1_guideline_a.rule.0
  statements: stmt.0
  segments: seg.2 seg.3
  source spans: r.2 r.3
document test_source.m1_guideline_b path corpus/test_sources/m1_guideline_b.html
  rules: test_source.m1_guideline_b.rule.0
  statements: stmt.0
  segments: seg.2
  source spans: r.2
";
    let stdout = String::from_utf8(trace_out.stdout).unwrap();
    let tail = stdout
        .strip_prefix(chain)
        .expect("stdout opens with the chain");
    let trace_results: Vec<TotalOperationResult> = read_jsonl(tail.as_bytes()).unwrap();
    assert_eq!(trace_results.len(), 1);
    assert_eq!(trace_results[0].outcome, Outcome::Ok);
    assert_eq!(trace_results[0].operation_id, id("trace"));

    // The null finding resolves through the same chain: the §8.5 item 6
    // Q1 unsat renders as a no-conflict verdict over context assertions.
    let no_conflict_out = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args([
            "trace",
            "--run",
            run_dir.to_str().unwrap(),
            "--finding",
            "finding.group.m1_no_conflict.0",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert_eq!(no_conflict_out.status.code(), Some(0));
    let no_conflict_stdout = String::from_utf8(no_conflict_out.stdout).unwrap();
    assert!(no_conflict_stdout.starts_with("trace finding.group.m1_no_conflict.0 run m1\n"));
    assert!(no_conflict_stdout.contains(
        "solver verdict: unsat category semantic_no_conflict \
         group group.m1_no_conflict pair q.m1_no_conflict.pair1 query q.m1_no_conflict.pair1.overlap\n"
    ));
    assert!(
        no_conflict_stdout.contains("document test_source.m1_control path corpus/test_sources/m1_control.html\n")
    );

    // Logs parse as their §4.6 record types; an ok total is a pure severity
    // fold, so nothing in the streams sits above ok.
    let events: Vec<EventRecord> =
        read_jsonl(&std::fs::read(run_dir.join("logs/events.jsonl")).unwrap()).unwrap();
    assert!(events.iter().all(|e| e.outcome == Outcome::Ok));
    let diagnostics: Vec<DiagnosticRecord> =
        read_jsonl(&std::fs::read(run_dir.join("logs/diagnostics.jsonl")).unwrap()).unwrap();
    assert!(diagnostics.iter().all(|d| d.outcome == Outcome::Ok));

    // The sweep is exhaustive: the run directory holds exactly the files
    // accounted for above.
    expected_files.sort();
    assert_eq!(files_under(&run_dir), expected_files);
}

/// The report processing_stage's live pins (`cargo test -p ckc-cli report::`): the
/// §7.2 partition over the recorded §8.6 world — one finding, one
/// documented null — with every quoted span resolving through its landed
/// source graph to the raw test_source bytes (§8.5 item 9), the corpus and
/// lexicon rows recomputed from the files in force, and the solver
/// identity matching the recorded verifier results. The trio pin extends
/// the surface: `report_en.md` as the landed payload's exact rendering and
/// the §5/§4.6 manifest pair attesting the run's provenance.
mod report {
    use super::*;

    use ckc_cli::report::{QuotedSpan, ReplayStatus, Report, Wording, render_markdown};
    use ckc_core::{ClaimTier, ReplayManifest, RunManifest, RunPlan, parse_corpora};

    /// Pin one row's evidence: the quoted spans carry exactly the
    /// `(document, region, span)` triples, each text equal to its span's
    /// `raw_text` in the document's landed source graph and present in
    /// the raw test_source file — quoted Japanese spans resolving to test_source
    /// bytes.
    fn assert_spans_ground(
        spans: &[QuotedSpan],
        triples: &[(&str, &str, &str)],
        graphs: &std::collections::BTreeMap<Id, ArtifactWrapper<SourceDocumentGraph>>,
        test_sources: &std::collections::BTreeMap<Id, String>,
    ) {
        let got: Vec<(Id, Id, Id)> = spans
            .iter()
            .map(|s| {
                (
                    s.document_id.clone(),
                    s.region_id.clone(),
                    s.span_id.clone(),
                )
            })
            .collect();
        let expected: Vec<(Id, Id, Id)> = triples
            .iter()
            .map(|(d, r, s)| (id(d), id(r), id(s)))
            .collect();
        assert_eq!(got, expected);
        for span in spans {
            let graph = &graphs[&span.document_id].payload;
            let raw = &graph
                .spans
                .iter()
                .find(|s| s.span_id == span.span_id)
                .expect("quoted span exists in its landed source graph")
                .raw_text;
            assert_eq!(&span.text, raw, "{}/{}", span.document_id, span.span_id);
            assert!(!span.text.is_empty());
            assert!(
                test_sources[&span.document_id].contains(&span.text),
                "{}/{}: quoted text missing from the raw test_source bytes",
                span.document_id,
                span.span_id
            );
        }
    }

    #[test]
    fn report_pins_the_partition_and_quoted_test_source_bytes() {
        let root = repo_root();
        let tmp = tempfile::tempdir().unwrap();
        let run_dir = tmp.path().join("m1");
        let out = Command::new(env!("CARGO_BIN_EXE_ckc"))
            .args([
                "run",
                "--experiment",
                "exp.m1_spine",
                "--out",
                run_dir.to_str().unwrap(),
            ])
            .current_dir(root)
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );

        // The test_source documents and their raw bytes, resolved through the
        // corpus registry like the run itself resolves them.
        let corpora =
            parse_corpora(&std::fs::read_to_string(root.join("registry/corpora.yaml")).unwrap())
                .unwrap();
        let docs = [
            "test_source.m1_control",
            "test_source.m1_guideline_a",
            "test_source.m1_guideline_b",
        ];
        let mut test_sources = std::collections::BTreeMap::new();
        let mut graphs = std::collections::BTreeMap::new();
        for doc in docs {
            let entry = corpora.iter().find(|c| c.id == id(doc)).unwrap();
            test_sources.insert(
                id(doc),
                std::fs::read_to_string(root.join(&entry.path)).unwrap(),
            );
            let graph: ArtifactWrapper<SourceDocumentGraph> =
                strict_read(&run_dir.join(format!("artifacts/{doc}/source_document_graph.json")));
            graphs.insert(id(doc), graph);
        }

        let report: ArtifactWrapper<Report> = strict_read(&run_dir.join("report.json"));
        report.payload.validate().unwrap();
        assert_eq!(report.schema_id, id("schema.report"));
        assert_eq!(report.artifact_id, id("report"));
        assert_eq!(report.artifact_kind, id("report"));
        assert_eq!(report.producer.pipeline_step_id, id("processing_stage.m1.report"));
        assert!(report.diagnostics.is_empty());

        // Input set: the trace pair, the three source graphs, the two
        // verifier results — every wrapper the assembly consumed.
        let trace: ArtifactWrapper<TraceBundle> = strict_read(&run_dir.join("trace_bundle.json"));
        let lineage: ArtifactWrapper<LineageIndex> =
            strict_read(&run_dir.join("lineage_index.json"));
        let mut inputs = vec![trace.content_hash.clone(), lineage.content_hash.clone()];
        inputs.extend(graphs.values().map(|g| g.content_hash.clone()));
        let mut identities = Vec::new();
        for gid in ["group.m1_conflict", "group.m1_no_conflict"] {
            let verifier: ArtifactWrapper<VerifierResults> =
                strict_read(&run_dir.join(format!("groups/{gid}/verifier_results.json")));
            inputs.push(verifier.content_hash.clone());
            identities.extend(
                verifier
                    .payload
                    .results
                    .iter()
                    .map(|r| r.solver_identity.clone()),
            );
        }
        inputs.sort();
        inputs.dedup();
        assert_eq!(report.input_hashes, inputs);

        // Corpus and lexicon rows recomputed from the files in force:
        // raw-byte hashes (§4.4 — files, not accepted artifacts), keys in
        // id order.
        let payload = &report.payload;
        let expected_corpus: Vec<(Id, Hash)> = docs
            .iter()
            .map(|doc| (id(doc), ckc_core::hash_bytes(test_sources[&id(doc)].as_bytes())))
            .collect();
        assert_eq!(payload.corpus_hashes, expected_corpus);
        assert_eq!(
            payload.lexicon_hash,
            ckc_core::hash_bytes(&std::fs::read(root.join("corpus/lexicon/ja_core.yaml")).unwrap())
        );

        // A clean run rolls up no diagnostics; the replay slot opens
        // unreplayed; the identity is the live z3 the verify processing_stage used.
        assert_eq!(payload.diagnostics_summary, Vec::new());
        assert_eq!(payload.replay_status, ReplayStatus::NotReplayed);
        assert_eq!(payload.solver_identity.solver_id, id("z3"));
        assert!(identities.iter().all(|i| *i == payload.solver_identity));
        assert_eq!(
            payload.wording,
            vec![
                Wording::DocumentedNoConflictResult,
                Wording::SyntheticTestSourceMeasurement
            ]
        );

        // §8.5 item 5's report surface: the §8.6 finding with the
        // cross-document core.
        assert_eq!(payload.findings.len(), 1);
        let finding = &payload.findings[0];
        assert_eq!(finding.finding_id, id("finding.group.m1_conflict.1"));
        assert_eq!(finding.query_id, id("q.m1_conflict.pair1.deontic"));
        assert_eq!(finding.verdict, SolverVerdict::Unsat);
        assert_eq!(
            finding.conflict_kind,
            Some(ConflictKind::DeonticDirectionConflict)
        );
        assert_eq!(finding.claim_tier, ClaimTier::S1Accepted);
        assert_eq!(finding.wording, Wording::SyntheticTestSourceMeasurement);
        let cross_core = vec![
            id("a.test_source.m1_guideline_a.rule.0"),
            id("a.test_source.m1_guideline_b.rule.0"),
        ];
        assert_eq!(finding.core, Some(cross_core.clone()));
        assert_eq!(finding.assertion_ids, cross_core);
        assert_eq!(
            finding.rule_ids,
            vec![
                id("test_source.m1_guideline_a.rule.0"),
                id("test_source.m1_guideline_b.rule.0")
            ]
        );
        assert_eq!(finding.region_ids, vec![id("r.2"), id("r.3")]);
        assert_spans_ground(
            &finding.quoted_spans,
            &[
                ("test_source.m1_guideline_a", "r.2", "s.2"),
                ("test_source.m1_guideline_a", "r.3", "s.3"),
                ("test_source.m1_guideline_b", "r.2", "s.2"),
            ],
            &graphs,
            &test_sources,
        );

        // §8.5 item 6's report surface: the disjoint-interval Q1 unsat as
        // the documented null — context assertions, no kind, no core.
        assert_eq!(payload.no_conflict_results.len(), 1);
        let null = &payload.no_conflict_results[0];
        assert_eq!(null.finding_id, id("finding.group.m1_no_conflict.0"));
        assert_eq!(null.query_id, id("q.m1_no_conflict.pair1.overlap"));
        assert_eq!(null.verdict, SolverVerdict::Unsat);
        assert_eq!(null.conflict_kind, None);
        assert_eq!(null.core, None);
        assert_eq!(null.claim_tier, ClaimTier::S1Accepted);
        assert_eq!(null.wording, Wording::DocumentedNoConflictResult);
        assert_eq!(
            null.assertion_ids,
            vec![
                id("ctx.test_source.m1_control.rule.0"),
                id("ctx.test_source.m1_guideline_a.rule.0")
            ]
        );
        assert_eq!(
            null.rule_ids,
            vec![
                id("test_source.m1_control.rule.0"),
                id("test_source.m1_guideline_a.rule.0")
            ]
        );
        assert_eq!(null.region_ids, vec![id("r.2"), id("r.3")]);
        assert_spans_ground(
            &null.quoted_spans,
            &[
                ("test_source.m1_control", "r.2", "s.2"),
                ("test_source.m1_guideline_a", "r.2", "s.2"),
                ("test_source.m1_guideline_a", "r.3", "s.3"),
            ],
            &graphs,
            &test_sources,
        );
    }

    /// The landed trio's live pins over its own recorded run: `report_en.md`
    /// is byte-for-byte the rendering of the strict-read `report.json`
    /// payload with every quoted span's grounded text present — closing
    /// the §8.5 item 9 surface on the derived view — and the §5/§4.6
    /// manifest pair attests the run's provenance: the registry-rebuilt
    /// plan hash, the build-baked commit, raw-byte hashes of the repo
    /// files in force, the `arch`/`os` profile, the recorded solver
    /// identity, the three test_source input hashes, and the nineteen
    /// accepted wrappers' output-hash set mirrored verbatim into the
    /// replay expectation.
    #[test]
    fn report_md_and_manifests_pin_run_provenance() {
        let root = repo_root();
        let tmp = tempfile::tempdir().unwrap();
        let run_dir = tmp.path().join("m1");
        let out = Command::new(env!("CARGO_BIN_EXE_ckc"))
            .args([
                "run",
                "--experiment",
                "exp.m1_spine",
                "--out",
                run_dir.to_str().unwrap(),
            ])
            .current_dir(root)
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );

        // §7.2 derived view: report_en.md carries exactly the landed
        // payload's rendering, quoted Japanese spans included.
        let report: ArtifactWrapper<Report> = strict_read(&run_dir.join("report.json"));
        let md = std::fs::read(run_dir.join("report_en.md")).unwrap();
        assert_eq!(md, render_markdown(&report.payload).into_bytes());
        let md = String::from_utf8(md).unwrap();
        for span in report
            .payload
            .findings
            .iter()
            .chain(&report.payload.no_conflict_results)
            .flat_map(|row| &row.quoted_spans)
        {
            assert!(
                md.contains(&span.text),
                "{}/{}: quoted text missing from report_en.md",
                span.document_id,
                span.span_id
            );
        }

        // The nineteen accepted wrappers — four layers per document, the
        // compile/verify pair per group, the trace pair, the report —
        // strict-read into the manifests' §4.3 output-hash set.
        let docs = [
            "test_source.m1_control",
            "test_source.m1_guideline_a",
            "test_source.m1_guideline_b",
        ];
        let mut output_hashes: Vec<Hash> = vec![report.content_hash.clone()];
        for doc in docs {
            let dir = run_dir.join("artifacts").join(doc);
            output_hashes.extend([
                strict_read::<SourceDocumentGraph>(&dir.join("source_document_graph.json")).content_hash,
                strict_read::<SegmentIr>(&dir.join("segments.json")).content_hash,
                strict_read::<Normalization>(&dir.join("normalization.json")).content_hash,
                strict_read::<IrBundle>(&dir.join("ir_bundle.json")).content_hash,
            ]);
        }
        for gid in ["group.m1_conflict", "group.m1_no_conflict"] {
            let dir = run_dir.join("groups").join(gid);
            output_hashes.extend([
                strict_read::<CompiledArtifact>(&dir.join("compiled.json")).content_hash,
                strict_read::<VerifierResults>(&dir.join("verifier_results.json")).content_hash,
            ]);
        }
        output_hashes.extend([
            strict_read::<TraceBundle>(&run_dir.join("trace_bundle.json")).content_hash,
            strict_read::<LineageIndex>(&run_dir.join("lineage_index.json")).content_hash,
        ]);
        assert_eq!(output_hashes.len(), 19, "nineteen accepted wrappers");
        output_hashes.sort();
        output_hashes.dedup();
        // Content addresses, not files: the control and guideline_b
        // segment payloads are byte-equal (structure-only layer, both
        // documents segment identically), so the §4.3 set collapses the
        // pair into eighteen addresses.
        assert_eq!(output_hashes.len(), 18, "one content-equal pair");

        // The §5/§4.6 records land bare — the manifests attest wrappers;
        // nothing wrappers them — so the bar is the canonical read.
        let manifest: RunManifest =
            read_strict_canonical(&std::fs::read(run_dir.join("manifest.json")).unwrap()).unwrap();
        let replay: ReplayManifest =
            read_strict_canonical(&std::fs::read(run_dir.join("replay_manifest.json")).unwrap()).unwrap();

        // §5 plan linkage: the plan rebuilt from the experiment registry
        // hashes to the recorded value.
        let experiments = parse_experiments(
            &std::fs::read_to_string(root.join("registry/experiments.yaml")).unwrap(),
        )
        .unwrap();
        let exp = experiments
            .iter()
            .find(|e| e.id == id("exp.m1_spine"))
            .unwrap();
        let plan = RunPlan {
            experiment_id: exp.id.clone(),
            test_source_groups: exp
                .test_source_groups
                .iter()
                .map(|g| g.group_id.clone())
                .collect(),
            pipelines: vec![exp.pipeline.clone()],
            seed: exp.seed,
            budget: exp.budget.iter().map(|(k, v)| (k.clone(), *v)).collect(),
        };
        assert_eq!(manifest.run_plan_hash, plan.plan_hash().unwrap());

        // Build-baked provenance: the commit this test and the binary
        // were built at (one build script serves both targets), 40 hex.
        assert_eq!(manifest.git_commit, env!("CKC_GIT_COMMIT"));
        assert_eq!(manifest.git_commit.len(), 40);
        assert!(manifest.git_commit.bytes().all(|b| b.is_ascii_hexdigit()));

        // Raw-byte hashes of the repo files in force (§4.4 `_hash` rule),
        // the environment profile, and the verify processing_stage's live identity.
        let file_hash = |rel: &str| ckc_core::hash_bytes(&std::fs::read(root.join(rel)).unwrap());
        assert_eq!(
            manifest.toolchain_manifest_hash,
            file_hash("rust-toolchain.toml")
        );
        assert_eq!(
            manifest.lockfile_hashes,
            vec![(id("cargo.lock"), file_hash("Cargo.lock"))]
        );
        assert_eq!(manifest.corpus_hash, file_hash("registry/corpora.yaml"));
        assert_eq!(
            manifest.lexicon_hash,
            file_hash("corpus/lexicon/ja_core.yaml")
        );
        assert_eq!(
            manifest.environment_profile,
            vec![
                (id("arch"), std::env::consts::ARCH.to_owned()),
                (id("os"), std::env::consts::OS.to_owned()),
            ]
        );
        assert_eq!(manifest.solver_identity, report.payload.solver_identity);
        assert_eq!(manifest.output_hashes, output_hashes);

        // §4.6 replay record: the re-execution argv, the three test_source
        // raw-byte input hashes, every shared fact mirrored from the §5
        // record, and the output expectation verbatim.
        assert_eq!(
            replay.command,
            [
                "ckc",
                "run",
                "--experiment",
                "exp.m1_spine",
                "--out",
                run_dir.to_str().unwrap(),
            ]
            .map(str::to_owned)
            .to_vec()
        );
        let corpora =
            parse_corpora(&std::fs::read_to_string(root.join("registry/corpora.yaml")).unwrap())
                .unwrap();
        let mut input_hashes = docs
            .map(|doc| {
                let entry = corpora.iter().find(|c| c.id == id(doc)).unwrap();
                file_hash(&entry.path)
            })
            .to_vec();
        input_hashes.sort();
        input_hashes.dedup();
        assert_eq!(replay.input_hashes, input_hashes);
        assert_eq!(replay.corpus_hash, manifest.corpus_hash);
        assert_eq!(replay.lexicon_hash, manifest.lexicon_hash);
        assert_eq!(
            replay.toolchain_manifest_hash,
            manifest.toolchain_manifest_hash
        );
        assert_eq!(replay.environment_profile, manifest.environment_profile);
        assert_eq!(replay.lockfile_hashes, manifest.lockfile_hashes);
        assert_eq!(replay.solver_identity, manifest.solver_identity);
        assert_eq!(replay.expected_output_hashes, manifest.output_hashes);
    }
}
