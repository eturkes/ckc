//! SPEC §8.5 item 3 workspace oracle: execute `exp.m1_spine` into a temp
//! dir, sweep the run directory (exact §8.3 file set — a later-stage
//! artifact entering the layout must join the sweep), strict-read every
//! accepted artifact with §4.4 re-validation, and assert the experiment's
//! gold entries over the verifier results — the code oracle behind §8.5
//! items 5 and 6. The [`report`] module pins the landed `report.json`
//! over its own recorded run: the finding/null partition and the quoted
//! spans resolving to fixture bytes (§8.5 item 9's code oracle).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ckc_cli::trace::{ConflictKind, LineageIndex, TraceBundle, TraceNodeKind};
use ckc_core::{
    ArtifactEnvelope, CanonRead, Canonical, DiagnosticRecord, EventRecord, GoldEntry, Hash, Id,
    IrBundle, Normalization, Outcome, SegmentIr, SourceGraph, TotalOperationResult,
    parse_experiments, parse_gold, read_canonical, read_jsonl,
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

/// §8.5 item 3's per-artifact bar: strict canonical read of the envelope
/// bytes, then §4.4 re-validation (content and policy hashes recomputed
/// from the payload).
fn strict_read<P: Canonical + CanonRead>(path: &Path) -> ArtifactEnvelope<P> {
    let bytes = std::fs::read(path).unwrap();
    let envelope: ArtifactEnvelope<P> =
        read_canonical(&bytes).unwrap_or_else(|e| panic!("{}: strict read: {e:?}", path.display()));
    envelope
        .validate()
        .unwrap_or_else(|e| panic!("{}: envelope invariant: {e:?}", path.display()));
    envelope
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

/// One gold entry against its group's compiled plan and verifier results.
fn assert_group_matches_gold(
    entry: &GoldEntry,
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
        // kind (§6) — with the unsat core matching gold as a set.
        assert_eq!(contradictions.len(), 1, "{gid}: exactly one contradiction");
        let hit = contradictions[0];
        assert!(
            compiled
                .query_plan
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
        assert_eq!(core, entry.expected_core, "{gid}: unsat core as a set");
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
        if entry.expected_null_result {
            let closed: Vec<_> = compiled
                .query_plan
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
fn run_oracle_strict_reads_artifacts_and_matches_gold() {
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
    // groups, documents, and gold file.
    let experiments = parse_experiments(
        &std::fs::read_to_string(root.join("registry/experiments.yaml")).unwrap(),
    )
    .unwrap();
    let exp = experiments
        .iter()
        .find(|e| e.id == id("exp.m1_spine"))
        .unwrap();
    let gold: Vec<GoldEntry> =
        parse_gold(&std::fs::read_to_string(root.join(&exp.expected_outcomes)).unwrap()).unwrap();
    assert_eq!(
        gold.len(),
        exp.fixture_groups.len(),
        "one gold entry per fixture group"
    );

    let mut expected_files: Vec<PathBuf> = vec![
        "lineage_index.json".into(),
        "logs/diagnostics.jsonl".into(),
        "logs/events.jsonl".into(),
        "report.json".into(),
        "trace_bundle.json".into(),
    ];

    // Document artifacts: the four §8.3 per-document layers strict-read.
    let documents: BTreeSet<&Id> = exp
        .fixture_groups
        .iter()
        .flat_map(|g| &g.fixtures)
        .collect();
    for doc in &documents {
        let dir = PathBuf::from("artifacts").join(doc.to_string());
        let _: ArtifactEnvelope<SourceGraph> =
            strict_read(&run_dir.join(dir.join("source_graph.json")));
        let _: ArtifactEnvelope<SegmentIr> = strict_read(&run_dir.join(dir.join("segments.json")));
        let _: ArtifactEnvelope<Normalization> =
            strict_read(&run_dir.join(dir.join("normalization.json")));
        let _: ArtifactEnvelope<IrBundle> = strict_read(&run_dir.join(dir.join("ir_bundle.json")));
        for name in [
            "ir_bundle.json",
            "normalization.json",
            "segments.json",
            "source_graph.json",
        ] {
            expected_files.push(dir.join(name));
        }
    }

    // Group artifacts: compiled + verifier results strict-read, every
    // materialized query byte-identical to its compiled body, gold asserted.
    for group in &exp.fixture_groups {
        let dir = PathBuf::from("groups").join(group.group_id.to_string());
        let compiled: ArtifactEnvelope<CompiledArtifact> =
            strict_read(&run_dir.join(dir.join("compiled.json")));
        let verifier: ArtifactEnvelope<VerifierResults> =
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
        let entry = gold
            .iter()
            .find(|e| e.group_id == group.group_id)
            .unwrap_or_else(|| panic!("{}: no gold entry", group.group_id));
        assert_group_matches_gold(entry, &compiled.payload, &verifier.payload);
    }

    // The report joins the §8.5 item 3 bar; its content pins live in the
    // `report` module over its own recorded run.
    let _: ArtifactEnvelope<ckc_cli::report::Report> = strict_read(&run_dir.join("report.json"));

    // Trace artifacts: the §7.1 pair strict-read from the run root, both
    // enveloped by the trace component over the DAG's node content-hash
    // set; the §8.6 finding row and the hashless report node pin the
    // claim surface.
    let trace: ArtifactEnvelope<TraceBundle> = strict_read(&run_dir.join("trace_bundle.json"));
    let lineage: ArtifactEnvelope<LineageIndex> = strict_read(&run_dir.join("lineage_index.json"));
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
    assert_eq!(trace.producer.component_id, id("stage.m1.trace"));
    assert_eq!(trace.input_hashes, node_hashes);
    assert_eq!(lineage.schema_id, id("schema.lineage_index"));
    assert_eq!(lineage.artifact_id, id("lineage_index"));
    assert_eq!(lineage.artifact_kind, id("lineage_index"));
    assert_eq!(lineage.producer.component_id, id("stage.m1.trace"));
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
            id("a.fixture.m1_guideline_a.rule.0"),
            id("a.fixture.m1_guideline_b.rule.0")
        ]
    );
    assert_eq!(
        finding.rule_ids,
        vec![
            id("fixture.m1_guideline_a.rule.0"),
            id("fixture.m1_guideline_b.rule.0")
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
document fixture.m1_guideline_a path corpus/fixtures/m1_guideline_a.html
  source spans: r.2 r.3
  segments: seg.2 seg.3
  statements: stmt.0
  rules: fixture.m1_guideline_a.rule.0
document fixture.m1_guideline_b path corpus/fixtures/m1_guideline_b.html
  source spans: r.2
  segments: seg.2
  statements: stmt.0
  rules: fixture.m1_guideline_b.rule.0
named assertions: a.fixture.m1_guideline_a.rule.0 a.fixture.m1_guideline_b.rule.0
solver verdict: unsat category semantic_contradiction conflict deontic_direction_conflict group group.m1_conflict pair q.m1_conflict.pair1 query q.m1_conflict.pair1.deontic
finding: finding.group.m1_conflict.1 report report
reverse finding -> solver verdict -> named assertions -> rules -> statements -> segments -> source spans
finding: finding.group.m1_conflict.1 report report
solver verdict: unsat category semantic_contradiction conflict deontic_direction_conflict group group.m1_conflict pair q.m1_conflict.pair1 query q.m1_conflict.pair1.deontic
named assertions: a.fixture.m1_guideline_a.rule.0 a.fixture.m1_guideline_b.rule.0
document fixture.m1_guideline_a path corpus/fixtures/m1_guideline_a.html
  rules: fixture.m1_guideline_a.rule.0
  statements: stmt.0
  segments: seg.2 seg.3
  source spans: r.2 r.3
document fixture.m1_guideline_b path corpus/fixtures/m1_guideline_b.html
  rules: fixture.m1_guideline_b.rule.0
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
    let null_out = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args([
            "trace",
            "--run",
            run_dir.to_str().unwrap(),
            "--finding",
            "finding.group.m1_null.0",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert_eq!(null_out.status.code(), Some(0));
    let null_stdout = String::from_utf8(null_out.stdout).unwrap();
    assert!(null_stdout.starts_with("trace finding.group.m1_null.0 run m1\n"));
    assert!(null_stdout.contains(
        "solver verdict: unsat category semantic_no_conflict \
         group group.m1_null pair q.m1_null.pair1 query q.m1_null.pair1.overlap\n"
    ));
    assert!(
        null_stdout.contains("document fixture.m1_control path corpus/fixtures/m1_control.html\n")
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

/// The report stage's live pins (`cargo test -p ckc-cli report::`): the
/// §7.2 partition over the recorded §8.6 world — one finding, one
/// documented null — with every quoted span resolving through its landed
/// source graph to the raw fixture bytes (§8.5 item 9), the corpus and
/// lexicon rows recomputed from the files in force, and the solver
/// identity matching the recorded verifier results.
mod report {
    use super::*;

    use ckc_cli::report::{QuotedSpan, ReplayStatus, Report, Wording};
    use ckc_core::{ClaimTier, parse_corpora};

    /// Pin one row's evidence: the quoted spans carry exactly the
    /// `(document, region, span)` triples, each text equal to its span's
    /// `raw_text` in the document's landed source graph and present in
    /// the raw fixture file — quoted Japanese spans resolving to fixture
    /// bytes.
    fn assert_spans_ground(
        spans: &[QuotedSpan],
        triples: &[(&str, &str, &str)],
        graphs: &std::collections::BTreeMap<Id, ArtifactEnvelope<SourceGraph>>,
        fixtures: &std::collections::BTreeMap<Id, String>,
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
                fixtures[&span.document_id].contains(&span.text),
                "{}/{}: quoted text missing from the raw fixture bytes",
                span.document_id,
                span.span_id
            );
        }
    }

    #[test]
    fn report_pins_the_partition_and_quoted_fixture_bytes() {
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

        // The fixture documents and their raw bytes, resolved through the
        // corpus registry like the run itself resolves them.
        let corpora =
            parse_corpora(&std::fs::read_to_string(root.join("registry/corpora.yaml")).unwrap())
                .unwrap();
        let docs = [
            "fixture.m1_control",
            "fixture.m1_guideline_a",
            "fixture.m1_guideline_b",
        ];
        let mut fixtures = std::collections::BTreeMap::new();
        let mut graphs = std::collections::BTreeMap::new();
        for doc in docs {
            let entry = corpora.iter().find(|c| c.id == id(doc)).unwrap();
            fixtures.insert(
                id(doc),
                std::fs::read_to_string(root.join(&entry.path)).unwrap(),
            );
            let graph: ArtifactEnvelope<SourceGraph> =
                strict_read(&run_dir.join(format!("artifacts/{doc}/source_graph.json")));
            graphs.insert(id(doc), graph);
        }

        let report: ArtifactEnvelope<Report> = strict_read(&run_dir.join("report.json"));
        report.payload.validate().unwrap();
        assert_eq!(report.schema_id, id("schema.report"));
        assert_eq!(report.artifact_id, id("report"));
        assert_eq!(report.artifact_kind, id("report"));
        assert_eq!(report.producer.component_id, id("stage.m1.report"));
        assert!(report.diagnostics.is_empty());

        // Input set: the trace pair, the three source graphs, the two
        // verifier results — every envelope the assembly consumed.
        let trace: ArtifactEnvelope<TraceBundle> = strict_read(&run_dir.join("trace_bundle.json"));
        let lineage: ArtifactEnvelope<LineageIndex> =
            strict_read(&run_dir.join("lineage_index.json"));
        let mut inputs = vec![trace.content_hash.clone(), lineage.content_hash.clone()];
        inputs.extend(graphs.values().map(|g| g.content_hash.clone()));
        let mut identities = Vec::new();
        for gid in ["group.m1_conflict", "group.m1_null"] {
            let verifier: ArtifactEnvelope<VerifierResults> =
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
            .map(|doc| (id(doc), ckc_core::hash_bytes(fixtures[&id(doc)].as_bytes())))
            .collect();
        assert_eq!(payload.corpus_hashes, expected_corpus);
        assert_eq!(
            payload.lexicon_hash,
            ckc_core::hash_bytes(&std::fs::read(root.join("corpus/lexicon/ja_core.yaml")).unwrap())
        );

        // A clean run rolls up no diagnostics; the replay slot opens
        // unreplayed; the identity is the live z3 the verify stage used.
        assert_eq!(payload.diagnostics_summary, Vec::new());
        assert_eq!(payload.replay_status, ReplayStatus::NotReplayed);
        assert_eq!(payload.solver_identity.solver_id, id("z3"));
        assert!(identities.iter().all(|i| *i == payload.solver_identity));
        assert_eq!(
            payload.wording,
            vec![
                Wording::DocumentedNullResult,
                Wording::SyntheticFixtureMeasurement
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
        assert_eq!(finding.claim_tier, ClaimTier::S1Admitted);
        assert_eq!(finding.wording, Wording::SyntheticFixtureMeasurement);
        let cross_core = vec![
            id("a.fixture.m1_guideline_a.rule.0"),
            id("a.fixture.m1_guideline_b.rule.0"),
        ];
        assert_eq!(finding.core, Some(cross_core.clone()));
        assert_eq!(finding.assertion_ids, cross_core);
        assert_eq!(
            finding.rule_ids,
            vec![
                id("fixture.m1_guideline_a.rule.0"),
                id("fixture.m1_guideline_b.rule.0")
            ]
        );
        assert_eq!(finding.region_ids, vec![id("r.2"), id("r.3")]);
        assert_spans_ground(
            &finding.quoted_spans,
            &[
                ("fixture.m1_guideline_a", "r.2", "s.2"),
                ("fixture.m1_guideline_a", "r.3", "s.3"),
                ("fixture.m1_guideline_b", "r.2", "s.2"),
            ],
            &graphs,
            &fixtures,
        );

        // §8.5 item 6's report surface: the disjoint-interval Q1 unsat as
        // the documented null — context assertions, no kind, no core.
        assert_eq!(payload.null_results.len(), 1);
        let null = &payload.null_results[0];
        assert_eq!(null.finding_id, id("finding.group.m1_null.0"));
        assert_eq!(null.query_id, id("q.m1_null.pair1.overlap"));
        assert_eq!(null.verdict, SolverVerdict::Unsat);
        assert_eq!(null.conflict_kind, None);
        assert_eq!(null.core, None);
        assert_eq!(null.claim_tier, ClaimTier::S1Admitted);
        assert_eq!(null.wording, Wording::DocumentedNullResult);
        assert_eq!(
            null.assertion_ids,
            vec![
                id("ctx.fixture.m1_control.rule.0"),
                id("ctx.fixture.m1_guideline_a.rule.0")
            ]
        );
        assert_eq!(
            null.rule_ids,
            vec![
                id("fixture.m1_control.rule.0"),
                id("fixture.m1_guideline_a.rule.0")
            ]
        );
        assert_eq!(null.region_ids, vec![id("r.2"), id("r.3")]);
        assert_spans_ground(
            &null.quoted_spans,
            &[
                ("fixture.m1_control", "r.2", "s.2"),
                ("fixture.m1_guideline_a", "r.2", "s.2"),
                ("fixture.m1_guideline_a", "r.3", "s.3"),
            ],
            &graphs,
            &fixtures,
        );
    }
}
