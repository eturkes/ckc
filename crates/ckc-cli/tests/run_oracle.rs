//! SPEC §8.5 item 3 workspace oracle: execute `exp.m1_spine` into a temp
//! dir, sweep the run directory (exact §8.3 file set — a later-stage
//! artifact entering the layout must join the sweep), strict-read every
//! accepted artifact with §4.4 re-validation, and assert the experiment's
//! gold entries over the verifier results — the code oracle behind §8.5
//! items 5 and 6.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ckc_core::{
    ArtifactEnvelope, CanonRead, Canonical, DiagnosticRecord, EventRecord, GoldEntry, Id, IrBundle,
    Normalization, Outcome, SegmentIr, SourceGraph, TotalOperationResult, parse_experiments,
    parse_gold, read_canonical, read_jsonl,
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

    let mut expected_files: Vec<PathBuf> =
        vec!["logs/diagnostics.jsonl".into(), "logs/events.jsonl".into()];

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
