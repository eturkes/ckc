use std::collections::HashSet;

use ckc_core::artifact::{
    DecisionTable, EventNarrative, ExecutionWitness, PatientCase, WorkflowFragment,
};
use ckc_core::canonical::{ContentHash, content_hash, to_canonical_bytes};
use ckc_core::clinical::{ClinicalClaim, Rule};
use ckc_core::envelope::{ArtifactEnvelope, ArtifactKind, ArtifactMeta};
use ckc_core::nf::{NfContext, Normalize};
use ckc_core::profile::SemanticProfile;
use ckc_core::source::{Concept, CorpusDocument, ExtractedTable, SourceSpan};
use ckc_core::verify::{ArgumentGraph, AssuranceNode, Conflict};
use ckc_store::ContentStore;
use serde_json::json;

// ---------------------------------------------------------------------------
// Fixture loading
// ---------------------------------------------------------------------------

const DOCUMENTS_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/documents.json");
const SPANS_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/spans.json");
const TABLES_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/tables.json");
const CONCEPTS_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/concepts.json");
const RULES_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/rules.json");
const CLAIMS_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/claims.json");
const DECISION_TABLES_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/decision_tables.json");
const EVENT_NARRATIVES_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/event_narratives.json");
const PATIENT_CASES_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/patient_cases.json");
const WORKFLOWS_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/workflows.json");
const CONFLICTS_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/conflicts.json");
const ARGUMENT_GRAPHS_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/argument_graphs.json");
const ASSURANCE_NODES_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/assurance_nodes.json");
const EXECUTION_WITNESSES_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/execution_witnesses.json");

struct Bundle {
    documents: Vec<CorpusDocument>,
    spans: Vec<SourceSpan>,
    tables: Vec<ExtractedTable>,
    concepts: Vec<Concept>,
    rules: Vec<Rule>,
    claims: Vec<ClinicalClaim>,
    decision_tables: Vec<DecisionTable>,
    event_narratives: Vec<EventNarrative>,
    patient_cases: Vec<PatientCase>,
    workflows: Vec<WorkflowFragment>,
    conflicts: Vec<Conflict>,
    argument_graphs: Vec<ArgumentGraph>,
    assurance_nodes: Vec<AssuranceNode>,
    execution_witnesses: Vec<ExecutionWitness>,
}

impl Bundle {
    fn load() -> Self {
        Self {
            documents: serde_json::from_str(DOCUMENTS_JSON).expect("documents.json"),
            spans: serde_json::from_str(SPANS_JSON).expect("spans.json"),
            tables: serde_json::from_str(TABLES_JSON).expect("tables.json"),
            concepts: serde_json::from_str(CONCEPTS_JSON).expect("concepts.json"),
            rules: serde_json::from_str(RULES_JSON).expect("rules.json"),
            claims: serde_json::from_str(CLAIMS_JSON).expect("claims.json"),
            decision_tables: serde_json::from_str(DECISION_TABLES_JSON)
                .expect("decision_tables.json"),
            event_narratives: serde_json::from_str(EVENT_NARRATIVES_JSON)
                .expect("event_narratives.json"),
            patient_cases: serde_json::from_str(PATIENT_CASES_JSON).expect("patient_cases.json"),
            workflows: serde_json::from_str(WORKFLOWS_JSON).expect("workflows.json"),
            conflicts: serde_json::from_str(CONFLICTS_JSON).expect("conflicts.json"),
            argument_graphs: serde_json::from_str(ARGUMENT_GRAPHS_JSON)
                .expect("argument_graphs.json"),
            assurance_nodes: serde_json::from_str(ASSURANCE_NODES_JSON)
                .expect("assurance_nodes.json"),
            execution_witnesses: serde_json::from_str(EXECUTION_WITNESSES_JSON)
                .expect("execution_witnesses.json"),
        }
    }

    fn doc_ids(&self) -> HashSet<&str> {
        self.documents.iter().map(|d| d.doc_id.as_str()).collect()
    }
    fn span_ids(&self) -> HashSet<&str> {
        self.spans.iter().map(|s| s.span_id.as_str()).collect()
    }
    fn table_ids(&self) -> HashSet<&str> {
        self.tables.iter().map(|t| t.table_id.as_str()).collect()
    }
    fn rule_ids(&self) -> HashSet<&str> {
        self.rules.iter().map(|r| r.rule_id.as_str()).collect()
    }
    fn dt_ids(&self) -> HashSet<&str> {
        self.decision_tables
            .iter()
            .map(|d| d.table_id.as_str())
            .collect()
    }
    fn workflow_ids(&self) -> HashSet<&str> {
        self.workflows
            .iter()
            .map(|w| w.workflow_id.as_str())
            .collect()
    }
    fn case_ids(&self) -> HashSet<&str> {
        self.patient_cases
            .iter()
            .map(|c| c.case_id.as_str())
            .collect()
    }
    fn witness_ids(&self) -> HashSet<&str> {
        self.execution_witnesses
            .iter()
            .map(|w| w.witness_id.as_str())
            .collect()
    }
    fn ag_ids(&self) -> HashSet<&str> {
        self.argument_graphs
            .iter()
            .map(|a| a.argument_graph_id.as_str())
            .collect()
    }
    fn assurance_node_ids(&self) -> HashSet<&str> {
        self.assurance_nodes
            .iter()
            .map(|n| n.node_id.as_str())
            .collect()
    }

    fn total_artifact_count(&self) -> usize {
        self.documents.len()
            + self.spans.len()
            + self.tables.len()
            + self.concepts.len()
            + self.rules.len()
            + self.claims.len()
            + self.decision_tables.len()
            + self.event_narratives.len()
            + self.patient_cases.len()
            + self.workflows.len()
            + self.conflicts.len()
            + self.argument_graphs.len()
            + self.assurance_nodes.len()
            + self.execution_witnesses.len()
    }
}

fn assert_subset(ids: &[&str], valid: &HashSet<&str>, context: &str) {
    for id in ids {
        assert!(valid.contains(id), "{context}: unknown ID {id:?}");
    }
}

fn envelope_meta(stage: &str, profiles: Vec<SemanticProfile>) -> ArtifactMeta {
    ArtifactMeta {
        schema_version: "0.0.0".into(),
        producer_version: "ckc-core/0.0.0".into(),
        command_manifest: json!({"command": "ckc", "args": ["demo", "research-kernel"]}),
        source_input_hashes: vec![],
        parent_hashes: vec![],
        stage: stage.into(),
        semantic_profiles: profiles,
        content_hash: ContentHash(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
        ),
        certificate_ids: vec![],
        replay_command: Some("ckc demo research-kernel --replay --out runs/research".into()),
    }
}

// ---------------------------------------------------------------------------
// Test: all fixture files deserialize correctly
// ---------------------------------------------------------------------------

#[test]
fn all_fixtures_deserialize() {
    let b = Bundle::load();
    assert_eq!(b.documents.len(), 3);
    assert_eq!(b.spans.len(), 16);
    assert_eq!(b.tables.len(), 1);
    assert_eq!(b.concepts.len(), 10);
    assert_eq!(b.rules.len(), 3);
    assert_eq!(b.claims.len(), 2);
    assert_eq!(b.decision_tables.len(), 1);
    assert_eq!(b.event_narratives.len(), 1);
    assert_eq!(b.patient_cases.len(), 2);
    assert_eq!(b.workflows.len(), 1);
    assert_eq!(b.conflicts.len(), 3);
    assert_eq!(b.argument_graphs.len(), 1);
    assert_eq!(b.assurance_nodes.len(), 5);
    assert_eq!(b.execution_witnesses.len(), 1);
    assert_eq!(b.total_artifact_count(), 50);
}

// ---------------------------------------------------------------------------
// Test: cross-referential consistency
// ---------------------------------------------------------------------------

#[test]
fn spans_reference_valid_documents() {
    let b = Bundle::load();
    let docs = b.doc_ids();
    for s in &b.spans {
        assert!(
            docs.contains(s.doc_id.as_str()),
            "span {} references unknown doc {}",
            s.span_id.as_str(),
            s.doc_id.as_str()
        );
    }
}

#[test]
fn span_chains_reference_valid_spans() {
    let b = Bundle::load();
    let sids = b.span_ids();
    for s in &b.spans {
        if let Some(ref prev) = s.previous_span_id {
            assert!(
                sids.contains(prev.as_str()),
                "span {} previous_span_id {} unknown",
                s.span_id.as_str(),
                prev.as_str()
            );
        }
        if let Some(ref next) = s.next_span_id {
            assert!(
                sids.contains(next.as_str()),
                "span {} next_span_id {} unknown",
                s.span_id.as_str(),
                next.as_str()
            );
        }
    }
}

#[test]
fn table_cell_spans_reference_valid_tables() {
    let b = Bundle::load();
    let tids = b.table_ids();
    for s in &b.spans {
        if let Some(ref tc) = s.table_cell {
            assert!(
                tids.contains(tc.table_id.as_str()),
                "span {} table_cell references unknown table {}",
                s.span_id.as_str(),
                tc.table_id.as_str()
            );
        }
    }
}

#[test]
fn tables_reference_valid_docs_and_spans() {
    let b = Bundle::load();
    let docs = b.doc_ids();
    let sids = b.span_ids();
    for t in &b.tables {
        assert!(
            docs.contains(t.doc_id.as_str()),
            "table {} references unknown doc",
            t.table_id.as_str()
        );
        if let Some(ref cap) = t.caption_span_id {
            assert!(
                sids.contains(cap.as_str()),
                "table {} caption_span_id unknown",
                t.table_id.as_str()
            );
        }
        let cell_ids: Vec<&str> = t.cell_span_ids.iter().map(|s| s.as_str()).collect();
        assert_subset(&cell_ids, &sids, &format!("table {}", t.table_id.as_str()));
    }
}

#[test]
fn concepts_reference_valid_spans() {
    let b = Bundle::load();
    let sids = b.span_ids();
    for c in &b.concepts {
        let refs: Vec<&str> = c.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_subset(&refs, &sids, &format!("concept {}", c.concept_id.as_str()));
    }
}

#[test]
fn rules_reference_valid_spans() {
    let b = Bundle::load();
    let sids = b.span_ids();
    for r in &b.rules {
        let refs: Vec<&str> = r.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_subset(&refs, &sids, &format!("rule {}", r.rule_id.as_str()));
    }
}

#[test]
fn claims_reference_valid_spans_rules_dts_workflows() {
    let b = Bundle::load();
    let sids = b.span_ids();
    let rids = b.rule_ids();
    let dtids = b.dt_ids();
    let wids = b.workflow_ids();
    for c in &b.claims {
        let ctx = format!("claim {}", c.claim_id.as_str());
        let span_refs: Vec<&str> = c.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_subset(&span_refs, &sids, &ctx);
        let rule_refs: Vec<&str> = c.rule_ids.iter().map(|r| r.as_str()).collect();
        assert_subset(&rule_refs, &rids, &ctx);
        let dt_refs: Vec<&str> = c.decision_table_ids.iter().map(|d| d.as_str()).collect();
        assert_subset(&dt_refs, &dtids, &ctx);
        let wf_refs: Vec<&str> = c.workflow_fragment_ids.iter().map(|w| w.as_str()).collect();
        assert_subset(&wf_refs, &wids, &ctx);
    }
}

#[test]
fn decision_tables_reference_valid_spans_and_tables() {
    let b = Bundle::load();
    let sids = b.span_ids();
    let tids = b.table_ids();
    for dt in &b.decision_tables {
        let ctx = format!("decision_table {}", dt.table_id.as_str());
        if let Some(ref stid) = dt.source_table_id {
            assert!(
                tids.contains(stid.as_str()),
                "{ctx}: unknown source_table_id"
            );
        }
        for row in &dt.rows {
            let refs: Vec<&str> = row.source_span_ids.iter().map(|s| s.as_str()).collect();
            assert_subset(&refs, &sids, &format!("{ctx} row {}", row.row_id.as_str()));
        }
    }
}

#[test]
fn event_narratives_reference_valid_spans() {
    let b = Bundle::load();
    let sids = b.span_ids();
    for en in &b.event_narratives {
        let refs: Vec<&str> = en.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_subset(&refs, &sids, "event_narrative");
    }
}

#[test]
fn patient_cases_reference_valid_spans() {
    let b = Bundle::load();
    let sids = b.span_ids();
    for pc in &b.patient_cases {
        let refs: Vec<&str> = pc.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_subset(
            &refs,
            &sids,
            &format!("patient_case {}", pc.case_id.as_str()),
        );
    }
}

#[test]
fn workflows_reference_valid_spans() {
    let b = Bundle::load();
    let sids = b.span_ids();
    for wf in &b.workflows {
        let refs: Vec<&str> = wf.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_subset(
            &refs,
            &sids,
            &format!("workflow {}", wf.workflow_id.as_str()),
        );
    }
}

#[test]
fn conflicts_reference_valid_spans_witnesses_graphs() {
    let b = Bundle::load();
    let sids = b.span_ids();
    let wids = b.witness_ids();
    let agids = b.ag_ids();
    for c in &b.conflicts {
        let ctx = format!("conflict {}", c.conflict_id.as_str());
        let refs: Vec<&str> = c.source_spans.iter().map(|s| s.as_str()).collect();
        assert_subset(&refs, &sids, &ctx);
        if let Some(ref w) = c.witness {
            assert!(
                wids.contains(w.as_str()),
                "{ctx}: unknown witness {}",
                w.as_str()
            );
        }
        if let Some(ref ag) = c.argument_graph_id {
            assert!(
                agids.contains(ag.as_str()),
                "{ctx}: unknown argument_graph_id {}",
                ag.as_str()
            );
        }
    }
}

#[test]
fn argument_graphs_reference_valid_spans() {
    let b = Bundle::load();
    let sids = b.span_ids();
    for ag in &b.argument_graphs {
        let refs: Vec<&str> = ag.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_subset(
            &refs,
            &sids,
            &format!("argument_graph {}", ag.argument_graph_id.as_str()),
        );
    }
}

#[test]
fn assurance_node_children_resolve() {
    let b = Bundle::load();
    let nids = b.assurance_node_ids();
    for n in &b.assurance_nodes {
        let refs: Vec<&str> = n.children.iter().map(|c| c.as_str()).collect();
        assert_subset(
            &refs,
            &nids,
            &format!("assurance_node {}", n.node_id.as_str()),
        );
    }
}

#[test]
fn execution_witnesses_reference_valid_cases_rules_spans() {
    let b = Bundle::load();
    let cids = b.case_ids();
    let rids = b.rule_ids();
    let sids = b.span_ids();
    for w in &b.execution_witnesses {
        let ctx = format!("witness {}", w.witness_id.as_str());
        if let Some(ref c) = w.case_id {
            assert!(
                cids.contains(c.as_str()),
                "{ctx}: unknown case_id {}",
                c.as_str()
            );
        }
        let app: Vec<&str> = w.applicable_rules.iter().map(|r| r.as_str()).collect();
        assert_subset(&app, &rids, &ctx);
        let def: Vec<&str> = w.defeated_rules.iter().map(|r| r.as_str()).collect();
        assert_subset(&def, &rids, &ctx);
        let spans: Vec<&str> = w.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_subset(&spans, &sids, &ctx);
    }
}

// ---------------------------------------------------------------------------
// Test: SPEC 20 Phase 0 scenario coverage
// ---------------------------------------------------------------------------

#[test]
fn all_eight_scenarios_covered() {
    let b = Bundle::load();

    // Scenario 1: sepsis recommends beta-lactam, anaphylaxis contraindicates, shared witness
    assert!(
        b.conflicts
            .iter()
            .any(|c| c.conflict_type == "norm_contradiction")
    );
    assert!(b.execution_witnesses.iter().any(|w| {
        w.case_id
            .as_ref()
            .is_some_and(|c| c.as_str() == "case_sepsis_allergy")
    }));

    // Scenario 2: βラクタム variants normalize through e-graph
    let bl_variants: Vec<_> = b
        .concepts
        .iter()
        .filter(|c| {
            c.egraph_class_id
                .as_ref()
                .is_some_and(|e| e.as_str() == "eclass_beta_lactam")
        })
        .collect();
    assert!(
        bl_variants.len() >= 5,
        "scenario 2: expected >= 5 beta-lactam e-graph variants, got {}",
        bl_variants.len()
    );

    // Scenario 3: decision table overlapping rows + gap witness
    assert!(
        b.conflicts
            .iter()
            .any(|c| c.conflict_type == "decision_table_overlap")
    );

    // Scenario 4: Event Calculus allergy persistence
    assert!(
        b.conflicts
            .iter()
            .any(|c| c.conflict_type == "temporal_violation")
    );
    assert!(!b.event_narratives.is_empty());

    // Scenario 5: repair candidates exist
    assert!(b.conflicts.iter().all(|c| !c.repair_candidates.is_empty()));

    // Scenario 6: missing provenance (for SHACL)
    assert!(
        b.rules
            .iter()
            .any(|r| r.source_span_ids.is_empty() && r.provenance.is_empty())
    );

    // Scenario 7: norm conflict as Lean proof obligation
    let norm_conflict = b
        .conflicts
        .iter()
        .find(|c| c.conflict_type == "norm_contradiction")
        .expect("norm conflict exists");
    assert!(norm_conflict.argument_graph_id.is_some());

    // Scenario 8: replay determinism (verified by cas_manifest_determinism test)
    assert!(b.total_artifact_count() >= 50);
}

// ---------------------------------------------------------------------------
// Test: CAS storage and manifest determinism
// ---------------------------------------------------------------------------

fn wrap_all(b: &Bundle) -> Vec<ArtifactEnvelope> {
    let mut envelopes = Vec::new();
    for d in &b.documents {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::CorpusDocument,
            d,
            envelope_meta("ingest", vec![SemanticProfile::Text]),
        ));
    }
    for s in &b.spans {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::SourceSpan,
            s,
            envelope_meta("extract", vec![SemanticProfile::Text]),
        ));
    }
    for t in &b.tables {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::ExtractedTable,
            t,
            envelope_meta("extract", vec![SemanticProfile::Text]),
        ));
    }
    for c in &b.concepts {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::Concept,
            c,
            envelope_meta("terminology", vec![SemanticProfile::Term]),
        ));
    }
    for r in &b.rules {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::Rule,
            r,
            envelope_meta(
                "formalize",
                vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
            ),
        ));
    }
    for c in &b.claims {
        let profiles: Vec<SemanticProfile> = c.profiles.clone();
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::ClinicalClaim,
            c,
            envelope_meta("formalize", profiles),
        ));
    }
    for dt in &b.decision_tables {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::DecisionTable,
            dt,
            envelope_meta("formalize", vec![SemanticProfile::Decision]),
        ));
    }
    for en in &b.event_narratives {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::EventNarrative,
            en,
            envelope_meta("formalize", vec![SemanticProfile::Event]),
        ));
    }
    for pc in &b.patient_cases {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::PatientCase,
            pc,
            envelope_meta("fixture", vec![]),
        ));
    }
    for wf in &b.workflows {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::WorkflowFragment,
            wf,
            envelope_meta("formalize", vec![SemanticProfile::Workflow]),
        ));
    }
    for c in &b.conflicts {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::Conflict,
            c,
            envelope_meta("conflicts", vec![]),
        ));
    }
    for ag in &b.argument_graphs {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::ArgumentGraph,
            ag,
            envelope_meta("conflicts", vec![SemanticProfile::Defeasible]),
        ));
    }
    for n in &b.assurance_nodes {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::AssuranceNode,
            n,
            envelope_meta("assurance", vec![SemanticProfile::Cert]),
        ));
    }
    for w in &b.execution_witnesses {
        envelopes.push(ArtifactEnvelope::wrap(
            ArtifactKind::ExecutionWitness,
            w,
            envelope_meta("verify", vec![SemanticProfile::Cert]),
        ));
    }
    envelopes
}

#[test]
fn cas_put_batch_stores_all_artifacts() {
    let b = Bundle::load();
    let envelopes = wrap_all(&b);
    assert_eq!(envelopes.len(), 50);

    let tmp = tempfile::TempDir::new().unwrap();
    let store = ContentStore::new(tmp.path());
    let hashes = store.put_batch(&envelopes).unwrap();
    assert_eq!(hashes.len(), 50);

    let unique: HashSet<_> = hashes.iter().collect();
    assert_eq!(
        unique.len(),
        50,
        "all 50 artifacts must have distinct envelope hashes"
    );

    for (env, hash) in envelopes.iter().zip(&hashes) {
        assert!(store.exists(hash));
        assert!(store.verify(hash).unwrap());
        let got = store.get(hash).unwrap();
        assert_eq!(env, &got);
    }
}

#[test]
fn cas_manifest_determinism() {
    let b = Bundle::load();
    let envelopes = wrap_all(&b);

    let tmp = tempfile::TempDir::new().unwrap();
    let store = ContentStore::new(tmp.path());
    store.put_batch(&envelopes).unwrap();

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
    assert_eq!(m1.entries.len(), 50);

    let hashes: Vec<_> = m1.entries.iter().map(|e| &e.hash).collect();
    let mut sorted = hashes.clone();
    sorted.sort();
    assert_eq!(hashes, sorted, "manifest entries must be sorted by hash");
}

#[test]
fn cas_manifest_hash_is_stable() {
    let b = Bundle::load();
    let envelopes = wrap_all(&b);

    let tmp1 = tempfile::TempDir::new().unwrap();
    let store1 = ContentStore::new(tmp1.path());
    store1.put_batch(&envelopes).unwrap();
    let m1 = store1.generate_manifest().unwrap();

    let tmp2 = tempfile::TempDir::new().unwrap();
    let store2 = ContentStore::new(tmp2.path());
    store2.put_batch(&envelopes).unwrap();
    let m2 = store2.generate_manifest().unwrap();

    let entries_sans_time_1: Vec<_> = m1
        .entries
        .iter()
        .map(|e| (&e.hash, &e.kind, &e.stage, &e.profiles))
        .collect();
    let entries_sans_time_2: Vec<_> = m2
        .entries
        .iter()
        .map(|e| (&e.hash, &e.kind, &e.stage, &e.profiles))
        .collect();
    assert_eq!(
        entries_sans_time_1, entries_sans_time_2,
        "manifest content (hashes, kinds, stages, profiles) must be identical across stores"
    );
}

// ---------------------------------------------------------------------------
// Test: NF idempotency for normalizable types
// ---------------------------------------------------------------------------

fn assert_nf_idempotent<T: Normalize + Clone + PartialEq + std::fmt::Debug + serde::Serialize>(
    items: &[T],
    type_name: &str,
) {
    for (i, item) in items.iter().enumerate() {
        let mut once = item.clone();
        once.normalize(&mut NfContext::new());
        let mut twice = once.clone();
        twice.normalize(&mut NfContext::new());
        assert_eq!(
            to_canonical_bytes(&once),
            to_canonical_bytes(&twice),
            "NF(NF(x)) must equal NF(x) for {type_name}[{i}]"
        );
    }
}

#[test]
fn nf_idempotency_rules() {
    let b = Bundle::load();
    assert_nf_idempotent(&b.rules, "Rule");
}

#[test]
fn nf_idempotency_claims() {
    let b = Bundle::load();
    assert_nf_idempotent(&b.claims, "ClinicalClaim");
}

#[test]
fn nf_idempotency_decision_tables() {
    let b = Bundle::load();
    assert_nf_idempotent(&b.decision_tables, "DecisionTable");
}

#[test]
fn nf_idempotency_argument_graphs() {
    let b = Bundle::load();
    assert_nf_idempotent(&b.argument_graphs, "ArgumentGraph");
}

#[test]
fn nf_idempotency_conflicts() {
    let b = Bundle::load();
    assert_nf_idempotent(&b.conflicts, "Conflict");
}

#[test]
fn nf_idempotency_execution_witnesses() {
    let b = Bundle::load();
    assert_nf_idempotent(&b.execution_witnesses, "ExecutionWitness");
}

#[test]
fn nf_idempotency_workflows() {
    let b = Bundle::load();
    assert_nf_idempotent(&b.workflows, "WorkflowFragment");
}

#[test]
fn nf_idempotency_patient_cases() {
    let b = Bundle::load();
    assert_nf_idempotent(&b.patient_cases, "PatientCase");
}

// ---------------------------------------------------------------------------
// Test: canonical hash determinism across all fixture types
// ---------------------------------------------------------------------------

#[test]
fn all_fixtures_have_deterministic_hashes() {
    let b1 = Bundle::load();
    let b2 = Bundle::load();
    let e1 = wrap_all(&b1);
    let e2 = wrap_all(&b2);
    assert_eq!(e1.len(), e2.len());
    for (a, b) in e1.iter().zip(e2.iter()) {
        assert_eq!(
            a.envelope_hash(),
            b.envelope_hash(),
            "envelope hashes must be deterministic across loads"
        );
        assert_eq!(
            a.meta.content_hash, b.meta.content_hash,
            "content hashes must be deterministic across loads"
        );
    }
}
