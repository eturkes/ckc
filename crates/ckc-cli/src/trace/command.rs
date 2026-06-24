//! `ckc trace` (SPEC §8.5 item 7): resolve one finding through the run's
//! §7.1 pair — the claim row and source nodes from `trace_bundle.json`,
//! the per-document reference rows from `lineage_index.json` — into the
//! full chain source text spans → segments → statements → rules → named
//! assertions → solver verdict → finding, rendered in both directions as
//! the command's stdout body. Both artifacts cross the §8.5 item 3 bar
//! (strict canonical read, wrapper and payload re-validation) and the
//! pair must agree on the finding's cross-document evidence. Every
//! failure is one command-scope §7.4 diagnostic, outcome `invalid`:
//! `schema_invalid` while loading an artifact, `trace_incomplete` when
//! resolution finds a gap.

use std::path::Path;

use ckc_core::{
    ArtifactWrapper, CanonRead, Canonical, DiagnosticCode, DiagnosticRecord, Id, Outcome,
    read_strict_canonical,
};

use super::{
    ClaimEvidenceRow, LineageIndex, LineageRow, TraceBundle, TraceNodeKind, canonical_id_set,
};
use crate::shell::{Shell, static_id};

/// One finding resolved through the pair: its claim row joined with the
/// per-document lineage sections, documents in canonical row order.
struct Chain<'a> {
    claim: &'a ClaimEvidenceRow,
    documents: Vec<DocSection<'a>>,
}

/// One member document's chain layers plus its DAG source path.
struct DocSection<'a> {
    row: &'a LineageRow,
    source_path: &'a str,
}

/// `ckc trace --run <dir> --finding <id>`: strict-read the §7.1 pair from
/// the run root, resolve the finding, and return the rendered chain for
/// stdout — `None` after recording the failure diagnostic.
pub(crate) fn execute(run_dir: &Path, finding: &Id, shell: &mut Shell) -> Option<Vec<u8>> {
    let loaded = strict_payload::<TraceBundle>(run_dir, "trace_bundle.json").and_then(|bundle| {
        let lineage = strict_payload::<LineageIndex>(run_dir, "lineage_index.json")?;
        Ok((bundle, lineage))
    });
    let (bundle, lineage) = match loaded {
        Ok(pair) => pair,
        Err(diagnostic) => {
            shell.diagnostic(diagnostic);
            return None;
        }
    };
    match resolve(&bundle.payload, &lineage.payload, finding) {
        Ok(chain) => Some(render(&chain, shell.run_id())),
        Err(diagnostic) => {
            shell.diagnostic(diagnostic);
            None
        }
    }
}

/// Load one pair artifact at the §8.5 item 3 bar: strict canonical read,
/// §4.4 wrapper re-validation, payload structural validation. Failures
/// are `schema_invalid`.
fn strict_payload<P: Canonical + CanonRead + Validated>(
    run_dir: &Path,
    rel: &str,
) -> Result<ArtifactWrapper<P>, DiagnosticRecord> {
    let fail = |reason: String| DiagnosticRecord {
        code: DiagnosticCode::SchemaInvalid,
        outcome: Outcome::Invalid,
        payload: vec![
            (static_id("artifact"), rel.to_owned()),
            (static_id("reason"), reason),
        ],
        region_ids: vec![],
        artifact_hashes: vec![],
    };
    let bytes = std::fs::read(run_dir.join(rel)).map_err(|e| fail(e.to_string()))?;
    let wrapper: ArtifactWrapper<P> =
        read_strict_canonical(&bytes).map_err(|e| fail(format!("strict read: {e:?}")))?;
    wrapper
        .validate()
        .map_err(|e| fail(format!("wrapper invariant: {e}")))?;
    wrapper
        .payload
        .validate_payload()
        .map_err(|e| fail(format!("payload invariant: {e}")))?;
    Ok(wrapper)
}

/// The pair payloads' shared validation surface, so [`strict_payload`]
/// runs the §7.1 structural rules behind one bound.
trait Validated {
    fn validate_payload(&self) -> Result<(), super::TraceError>;
}

impl Validated for TraceBundle {
    fn validate_payload(&self) -> Result<(), super::TraceError> {
        self.validate()
    }
}

impl Validated for LineageIndex {
    fn validate_payload(&self) -> Result<(), super::TraceError> {
        self.validate()
    }
}

/// Resolve the finding: its claim row, its lineage rows (at least one),
/// pair agreement on the cross-document evidence (the claim's region and
/// rule sets equal the union of its rows'), and a DAG source node per
/// member document. Gaps are `trace_incomplete`.
fn resolve<'a>(
    bundle: &'a TraceBundle,
    lineage: &'a LineageIndex,
    finding: &Id,
) -> Result<Chain<'a>, DiagnosticRecord> {
    let fail = |reason: String| DiagnosticRecord {
        code: DiagnosticCode::TraceIncomplete,
        outcome: Outcome::Invalid,
        payload: vec![
            (static_id("finding"), finding.to_string()),
            (static_id("reason"), reason),
        ],
        region_ids: vec![],
        artifact_hashes: vec![],
    };
    let claim = bundle
        .claims
        .iter()
        .find(|c| c.finding_id == *finding)
        .ok_or_else(|| fail("no claim row in trace_bundle.json".to_owned()))?;
    let rows: Vec<&LineageRow> = lineage
        .rows
        .iter()
        .filter(|r| r.finding_id == *finding)
        .collect();
    if rows.is_empty() {
        return Err(fail("no lineage rows in lineage_index.json".to_owned()));
    }
    let region_union = canonical_id_set(rows.iter().flat_map(|r| &r.region_ids).cloned().collect());
    let rule_union = canonical_id_set(rows.iter().flat_map(|r| &r.rule_ids).cloned().collect());
    for (pool, claim_set, union) in [
        ("region_ids", &claim.region_ids, region_union),
        ("rule_ids", &claim.rule_ids, rule_union),
    ] {
        if *claim_set != union {
            return Err(fail(format!("the pair disagrees on {pool}")));
        }
    }
    let documents = rows
        .into_iter()
        .map(|row| {
            let source = bundle
                .nodes
                .iter()
                .find(|n| n.node_id == row.document_id && n.kind == TraceNodeKind::Source)
                .ok_or_else(|| fail(format!("document {} has no source node", row.document_id)))?;
            Ok(DocSection {
                row,
                source_path: &source.path,
            })
        })
        .collect::<Result<Vec<_>, DiagnosticRecord>>()?;
    Ok(Chain { claim, documents })
}

/// The §8.5 item 7 layer labels, derivation order.
const LAYERS: [&str; 4] = ["source text spans", "segments", "statements", "rules"];

/// Render the chain in both directions: forward walks derivation order
/// (per-document layers, then the claim's assertions → verdict → finding),
/// reverse walks the same lines back from the finding. Documents keep
/// canonical row order in both blocks; only the layer order turns.
fn render(chain: &Chain<'_>, run_id: &Id) -> Vec<u8> {
    let mut out = String::new();
    out.push_str(&format!("trace {} run {run_id}\n", chain.claim.finding_id));
    out.push_str(
        "forward source text spans -> segments -> statements -> rules -> named assertions \
         -> solver verdict -> finding\n",
    );
    for doc in &chain.documents {
        document_block(&mut out, doc, true);
    }
    claim_block(&mut out, chain.claim, true);
    out.push_str(
        "reverse finding -> solver verdict -> named assertions -> rules -> statements \
         -> segments -> source text spans\n",
    );
    claim_block(&mut out, chain.claim, false);
    for doc in &chain.documents {
        document_block(&mut out, doc, false);
    }
    out.into_bytes()
}

/// One member document's layer lines under its header.
fn document_block(out: &mut String, doc: &DocSection<'_>, forward: bool) {
    out.push_str(&format!(
        "document {} path {}\n",
        doc.row.document_id, doc.source_path
    ));
    let sets = [
        &doc.row.region_ids,
        &doc.row.segment_ids,
        &doc.row.statement_ids,
        &doc.row.rule_ids,
    ];
    let mut order: Vec<usize> = (0..LAYERS.len()).collect();
    if !forward {
        order.reverse();
    }
    for i in order {
        out.push_str(&format!("  {}: {}\n", LAYERS[i], join_ids(sets[i])));
    }
}

/// The claim row's cross-document lines: named assertions, solver verdict
/// (with its §6 category, conflict kind when present, and the §8.6 query
/// coordinates), finding → report. Forward order, or reversed.
fn claim_block(out: &mut String, claim: &ClaimEvidenceRow, forward: bool) {
    let assertions = format!("named assertions: {}\n", join_ids(&claim.assertion_ids));
    let verdict = match claim.verdict {
        Some(v) => v.as_str(),
        None => "none",
    };
    let conflict = match claim.conflict_kind {
        Some(kind) => format!(" conflict {}", kind.as_str()),
        None => String::new(),
    };
    let verdict = format!(
        "solver verdict: {verdict} category {}{conflict} group {} pair {} query {}\n",
        claim.category.as_str(),
        claim.group_id,
        claim.pair_id,
        claim.query_id
    );
    let finding = format!(
        "finding: {} report {}\n",
        claim.finding_id, claim.report_ref
    );
    let lines = if forward {
        [assertions, verdict, finding]
    } else {
        [finding, verdict, assertions]
    };
    for line in lines {
        out.push_str(&line);
    }
}

fn join_ids(ids: &[Id]) -> String {
    ids.iter().map(Id::as_str).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::super::{ClaimEvidenceRow, LineageIndex, LineageRow, TraceBundle, TraceNode};
    use super::*;
    use crate::shell::{FinishedCommand, run_none};
    use ckc_core::{
        EventRecord, EvidenceStatus, Hash, Origin, Producer, canonical_payload_bytes,
        canonicalization_policy_hash, content_hash, read_jsonl,
    };
    use ckc_smt::{SolverVerdict, VerifierCategory};

    use super::super::ConflictKind;

    fn id(text: &str) -> Id {
        text.parse().unwrap()
    }

    fn hash(seed: char) -> Hash {
        Hash::new(format!("sha256:{}", seed.to_string().repeat(64))).unwrap()
    }

    /// Two source nodes + the report node, one contradiction claim, in
    /// canonical-set order via the trace module's sorter.
    fn sample_bundle() -> TraceBundle {
        let mut nodes = vec![
            TraceNode {
                node_id: id("doc.a"),
                kind: TraceNodeKind::Source,
                path: "corpus/test_sources/doc.a.html".to_owned(),
                content_hash: Some(hash('a')),
            },
            TraceNode {
                node_id: id("doc.b"),
                kind: TraceNodeKind::Source,
                path: "corpus/test_sources/doc.b.html".to_owned(),
                content_hash: Some(hash('b')),
            },
            TraceNode {
                node_id: id("report"),
                kind: TraceNodeKind::Report,
                path: "report.json".to_owned(),
                content_hash: None,
            },
        ];
        super::super::sort_canonical(&mut nodes);
        let claims = vec![ClaimEvidenceRow {
            finding_id: id("finding.group.g1.0"),
            group_id: id("group.g1"),
            pair_id: id("pair.1"),
            query_id: id("pair.1.deontic"),
            category: VerifierCategory::SemanticContradiction,
            verdict: Some(SolverVerdict::Unsat),
            conflict_kind: Some(ConflictKind::DeonticDirectionConflict),
            assertion_ids: vec![id("a.doc.a.rule.0"), id("a.doc.b.rule.0")],
            rule_ids: vec![id("doc.a.rule.0"), id("doc.b.rule.0")],
            region_ids: vec![id("r.2"), id("r.3")],
            report_ref: id("report"),
        }];
        TraceBundle {
            nodes,
            edges: vec![],
            claims,
        }
    }

    fn sample_lineage() -> LineageIndex {
        LineageIndex {
            rows: vec![
                LineageRow {
                    finding_id: id("finding.group.g1.0"),
                    document_id: id("doc.a"),
                    region_ids: vec![id("r.2")],
                    rule_ids: vec![id("doc.a.rule.0")],
                    segment_ids: vec![id("seg.4")],
                    statement_ids: vec![id("doc.a.stmt.0")],
                },
                LineageRow {
                    finding_id: id("finding.group.g1.0"),
                    document_id: id("doc.b"),
                    region_ids: vec![id("r.3")],
                    rule_ids: vec![id("doc.b.rule.0")],
                    segment_ids: vec![id("seg.6")],
                    statement_ids: vec![id("doc.b.stmt.0")],
                },
            ],
        }
    }

    /// Wrapper one payload with real content/policy hashes and write its
    /// canonical bytes into `dir` — the run layout the command reads.
    fn write_artifact<P: Canonical>(dir: &Path, rel: &str, kind: &str, payload: P) {
        let wrapper = ArtifactWrapper {
            schema_id: id(&format!("schema.{kind}")),
            artifact_id: id(kind),
            artifact_kind: id(kind),
            producer: Producer {
                pipeline_id: id("cand.t"),
                pipeline_step_id: id("processing_stage.t.trace"),
                toolchain_manifest_hash: hash('f'),
            },
            input_hashes: vec![],
            content_hash: content_hash(&payload).unwrap(),
            canonicalization_policy_hash: canonicalization_policy_hash(),
            origin: Origin::DeterministicCompiler,
            evidence_status: EvidenceStatus::MechanicalEvidenceStatus,
            external_effects: vec![],
            trace_refs: vec![],
            diagnostics: vec![],
            runtime_metadata: vec![],
            payload,
        };
        wrapper.validate().unwrap();
        let bytes = canonical_payload_bytes(&wrapper).unwrap();
        std::fs::write(dir.join(rel), bytes).unwrap();
    }

    fn write_pair(dir: &Path, bundle: TraceBundle, lineage: LineageIndex) {
        bundle.validate().unwrap();
        lineage.validate().unwrap();
        write_artifact(dir, "trace_bundle.json", "trace_bundle", bundle);
        write_artifact(dir, "lineage_index.json", "lineage_index", lineage);
    }

    fn shell() -> Shell {
        Shell::open(static_id("trace"), run_none(), None)
    }

    /// Close the shell and surface (outcome, diagnostics) from the command
    /// event — the observable failure surface.
    fn finished(shell: Shell) -> (Outcome, Vec<DiagnosticRecord>) {
        let FinishedCommand {
            result,
            streamed_events,
            ..
        } = shell.finish().unwrap();
        let events: Vec<EventRecord> = read_jsonl(streamed_events.as_deref().unwrap()).unwrap();
        assert_eq!(events.len(), 1);
        (result.outcome, events[0].diagnostics.clone())
    }

    #[test]
    fn resolves_full_chain_both_directions() {
        let tmp = tempfile::tempdir().unwrap();
        write_pair(tmp.path(), sample_bundle(), sample_lineage());
        let mut shell = shell();
        let chain = execute(tmp.path(), &id("finding.group.g1.0"), &mut shell).unwrap();
        let expected = "\
trace finding.group.g1.0 run run.none
forward source text spans -> segments -> statements -> rules -> named assertions -> solver verdict -> finding
document doc.a path corpus/test_sources/doc.a.html
  source text spans: r.2
  segments: seg.4
  statements: doc.a.stmt.0
  rules: doc.a.rule.0
document doc.b path corpus/test_sources/doc.b.html
  source text spans: r.3
  segments: seg.6
  statements: doc.b.stmt.0
  rules: doc.b.rule.0
named assertions: a.doc.a.rule.0 a.doc.b.rule.0
solver verdict: unsat category semantic_contradiction conflict deontic_direction_conflict group group.g1 pair pair.1 query pair.1.deontic
finding: finding.group.g1.0 report report
reverse finding -> solver verdict -> named assertions -> rules -> statements -> segments -> source text spans
finding: finding.group.g1.0 report report
solver verdict: unsat category semantic_contradiction conflict deontic_direction_conflict group group.g1 pair pair.1 query pair.1.deontic
named assertions: a.doc.a.rule.0 a.doc.b.rule.0
document doc.a path corpus/test_sources/doc.a.html
  rules: doc.a.rule.0
  statements: doc.a.stmt.0
  segments: seg.4
  source text spans: r.2
document doc.b path corpus/test_sources/doc.b.html
  rules: doc.b.rule.0
  statements: doc.b.stmt.0
  segments: seg.6
  source text spans: r.3
";
        assert_eq!(String::from_utf8(chain).unwrap(), expected);
        let (outcome, diagnostics) = finished(shell);
        assert_eq!(outcome, Outcome::Ok);
        assert!(diagnostics.is_empty());
    }

    // A no-conflict claim renders its verdict line without the conflict
    // segment.
    #[test]
    fn no_conflict_claim_renders_without_conflict_kind() {
        let mut bundle = sample_bundle();
        bundle.claims[0].category = VerifierCategory::SemanticNoConflict;
        bundle.claims[0].conflict_kind = None;
        let tmp = tempfile::tempdir().unwrap();
        write_pair(tmp.path(), bundle, sample_lineage());
        let mut shell = shell();
        let chain = execute(tmp.path(), &id("finding.group.g1.0"), &mut shell).unwrap();
        let text = String::from_utf8(chain).unwrap();
        assert!(text.contains(
            "solver verdict: unsat category semantic_no_conflict group group.g1 \
             pair pair.1 query pair.1.deontic\n"
        ));
        assert!(!text.contains("conflict deontic_direction_conflict"));
    }

    #[test]
    fn missing_artifact_is_schema_invalid() {
        let tmp = tempfile::tempdir().unwrap();
        let mut shell = shell();
        assert_eq!(
            execute(tmp.path(), &id("finding.group.g1.0"), &mut shell),
            None
        );
        let (outcome, diagnostics) = finished(shell);
        assert_eq!(outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, DiagnosticCode::SchemaInvalid);
        assert_eq!(diagnostics[0].payload[0].1, "trace_bundle.json");
    }

    #[test]
    fn corrupt_artifact_fails_strict_read() {
        let tmp = tempfile::tempdir().unwrap();
        write_pair(tmp.path(), sample_bundle(), sample_lineage());
        std::fs::write(tmp.path().join("lineage_index.json"), b"{}").unwrap();
        let mut shell = shell();
        assert_eq!(
            execute(tmp.path(), &id("finding.group.g1.0"), &mut shell),
            None
        );
        let (outcome, diagnostics) = finished(shell);
        assert_eq!(outcome, Outcome::Invalid);
        assert_eq!(diagnostics[0].code, DiagnosticCode::SchemaInvalid);
        assert_eq!(diagnostics[0].payload[0].1, "lineage_index.json");
        assert!(diagnostics[0].payload[1].1.starts_with("strict read:"));
    }

    #[test]
    fn unknown_finding_is_trace_incomplete() {
        let tmp = tempfile::tempdir().unwrap();
        write_pair(tmp.path(), sample_bundle(), sample_lineage());
        let mut shell = shell();
        assert_eq!(
            execute(tmp.path(), &id("finding.group.g9.0"), &mut shell),
            None
        );
        let (outcome, diagnostics) = finished(shell);
        assert_eq!(outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, DiagnosticCode::TraceIncomplete);
        assert_eq!(diagnostics[0].payload[0].1, "finding.group.g9.0");
        assert_eq!(
            diagnostics[0].payload[1].1,
            "no claim row in trace_bundle.json"
        );
    }

    // The claim present but its lineage rows missing: the §7.1 pair is
    // incomplete for the finding.
    #[test]
    fn missing_lineage_rows_are_trace_incomplete() {
        let tmp = tempfile::tempdir().unwrap();
        write_pair(tmp.path(), sample_bundle(), LineageIndex { rows: vec![] });
        let mut shell = shell();
        assert_eq!(
            execute(tmp.path(), &id("finding.group.g1.0"), &mut shell),
            None
        );
        let (_, diagnostics) = finished(shell);
        assert_eq!(diagnostics[0].code, DiagnosticCode::TraceIncomplete);
        assert_eq!(
            diagnostics[0].payload[1].1,
            "no lineage rows in lineage_index.json"
        );
    }

    // The pair disagreeing on the finding's evidence union is a resolution
    // failure, never a partial print.
    #[test]
    fn pair_disagreement_is_trace_incomplete() {
        let mut lineage = sample_lineage();
        lineage.rows[1].region_ids = vec![id("r.9")];
        let tmp = tempfile::tempdir().unwrap();
        write_pair(tmp.path(), sample_bundle(), lineage);
        let mut shell = shell();
        assert_eq!(
            execute(tmp.path(), &id("finding.group.g1.0"), &mut shell),
            None
        );
        let (_, diagnostics) = finished(shell);
        assert_eq!(diagnostics[0].code, DiagnosticCode::TraceIncomplete);
        assert_eq!(
            diagnostics[0].payload[1].1,
            "the pair disagrees on region_ids"
        );
    }

    // A member document missing its DAG source node breaks the chain's
    // source end.
    #[test]
    fn missing_source_node_is_trace_incomplete() {
        let mut bundle = sample_bundle();
        bundle.nodes.retain(|n| n.node_id != id("doc.b"));
        let tmp = tempfile::tempdir().unwrap();
        write_pair(tmp.path(), bundle, sample_lineage());
        let mut shell = shell();
        assert_eq!(
            execute(tmp.path(), &id("finding.group.g1.0"), &mut shell),
            None
        );
        let (_, diagnostics) = finished(shell);
        assert_eq!(diagnostics[0].code, DiagnosticCode::TraceIncomplete);
        assert_eq!(
            diagnostics[0].payload[1].1,
            "document doc.b has no source node"
        );
    }
}
