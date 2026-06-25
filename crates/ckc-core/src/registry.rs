//! SPEC §8.4 registry entry types — the YAML-edited control surface a run
//! resolves against — plus the §8.2 reference expected-outcome entries.
//!
//! One type family per file: `registry/corpora.yaml` is a list of
//! [`CorpusEntry`] (acceptance fields per §8.2), `registry/candidates.yaml` is
//! one [`Candidates`] document holding [`PipelineEntry`] and [`ProcessingStageEntry`]
//! entries, `registry/experiments.yaml` is a list of [`ExperimentEntry`]
//! (what `ckc run --experiment` resolves), and `corpus/reference/*.yaml` is a list
//! of [`ReferenceEntry`] asserted by acceptance tests. Loading is strict: unknown
//! fields are rejected, [`Id`] fields are grammar-checked by `Id`'s serde,
//! and enum fields accept exactly their canonical spellings. Past loading,
//! [`validate_registries`] checks the set semantically — pool-level id
//! uniqueness, nonempty requirements, cross-file resolution, the §8.4
//! stage-chain rule — collecting every [`RegistryFinding`] so §3
//! `ckc registry check` reports the whole set.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::enums::{EvidenceStatus, Origin, fieldless_enum};
use crate::id::Id;
use crate::source_linkage::Provenance;

fieldless_enum! {
    /// SPEC §8.4 processing_stage determinism class. Every M1 processing_stage is
    /// `deterministic`; `nondeterministic` marks processing_stages whose reruns may
    /// diverge (M2's recorded weak-model routes), which replay handles
    /// through recorded I/O rather than re-execution.
    Determinism {
        Deterministic => "deterministic",
        Nondeterministic => "nondeterministic",
    }
}

/// One `registry/corpora.yaml` entry: a corpus document accepted with the
/// §8.2 fields — a working example of acceptance-over-proposer precedence
/// (`ai_generated` origin under `source_evidence_status`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEntry {
    /// Document id experiments and groups reference (e.g.
    /// `test_source.m1_guideline_a`).
    pub id: Id,
    /// Document path relative to the repository root.
    pub path: String,
    pub origin: Origin,
    /// EvidenceStatus granted on acceptance.
    pub evidence_status: EvidenceStatus,
    pub provenance: Provenance,
}

/// `registry/candidates.yaml`: the §8.4 candidates — pipelines and
/// the processing_stage entries they chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidates {
    pub pipelines: Vec<PipelineEntry>,
    pub processing_stages: Vec<ProcessingStageEntry>,
}

/// A pipeline candidate: an ordered chain of [`ProcessingStageEntry`] ids. The §8.4
/// chain rule (every processing_stage's declared input artifact kinds are produced by
/// its predecessors) is checked by registry validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineEntry {
    /// Pipeline id experiments reference (e.g. `pipe.layered_ckcir_to_smt`).
    pub id: Id,
    /// [`ProcessingStageEntry`] ids in execution order.
    pub processing_stages: Vec<Id>,
}

/// A processing_stage candidate: one pipeline step with its §8.4 fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessingStageEntry {
    /// [`ProcessingStageEntry`] id pipelines reference.
    pub id: Id,
    /// ProcessingStage role (§8.3 vocabulary: `extract`, `segment`, `normalize`,
    /// `assemble`, `compile`, `verify`, `trace`, `report`); open so later
    /// milestones add roles without reshaping entries.
    pub kind: Id,
    pub determinism: Determinism,
    /// Artifact kinds this processing_stage consumes; empty for a chain head whose
    /// input is the corpus document itself.
    pub input_artifact_kinds: Vec<Id>,
    /// Artifact kinds this processing_stage produces.
    pub output_artifact_kinds: Vec<Id>,
}

/// One `registry/experiments.yaml` entry: what `ckc run --experiment`
/// resolves into a §5 RunPlan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentEntry {
    /// Experiment id (e.g. `exp.m1_scaffold`).
    pub id: Id,
    /// Pipeline candidate this experiment executes.
    pub pipeline: Id,
    /// TestSource groups in evaluation order.
    pub test_source_groups: Vec<TestSourceGroup>,
    /// Deterministic seed for any seeded processing_stage.
    pub seed: u64,
    /// Budget caps: counter name → limit (the counters §4.6
    /// `resource_counters` consume against).
    pub budget: BTreeMap<Id, u64>,
    /// Expected-outcome ref: path of the reference file ([`ReferenceEntry`] list)
    /// asserted against this experiment's groups, relative to the
    /// repository root.
    pub expected_outcomes: String,
}

/// A §8.2 test_source group: the corpus documents one verdict is computed over.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestSourceGroup {
    /// Group id (e.g. `group.m1_conflict`).
    pub group_id: Id,
    /// Member [`CorpusEntry`] ids, in semantic order.
    pub test_sources: Vec<Id>,
}

/// One `corpus/reference/*.yaml` entry: the §8.2 expected outcome for a test_source
/// group, asserted by the acceptance tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceEntry {
    pub group_id: Id,
    /// Expected §6 verdict category (e.g. `semantic_contradiction`,
    /// `semantic_no_conflict`).
    pub expected_outcome: Id,
    /// Expected conflict kind for contradiction groups (e.g.
    /// `deontic_direction_conflict`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_conflict_kind: Option<Id>,
    /// Expected unsat-core assertion names, compared as a set.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub expected_unsat_core: BTreeSet<Id>,
    /// Whether the group's expected finding is a documented no-conflict result.
    #[serde(default, skip_serializing_if = "is_false")]
    pub expected_no_conflict_result: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// Error loading or serializing a registry document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// YAML engine failure: syntax, shape, or a field-level validation
    /// raised through serde (Id grammar, enum spelling, unknown field).
    Yaml(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::Yaml(message) => write!(f, "registry yaml: {message}"),
        }
    }
}

impl std::error::Error for RegistryError {}

fn from_yaml<T: DeserializeOwned>(yaml: &str) -> Result<T, RegistryError> {
    serde_saphyr::from_str(yaml).map_err(|e| RegistryError::Yaml(e.to_string()))
}

/// Serialize any registry value back to YAML (round-trip partner of the
/// `parse_*` loaders).
pub fn to_yaml<T: Serialize>(value: &T) -> Result<String, RegistryError> {
    serde_saphyr::to_string(value).map_err(|e| RegistryError::Yaml(e.to_string()))
}

/// Load a `registry/corpora.yaml` document.
pub fn parse_corpora(yaml: &str) -> Result<Vec<CorpusEntry>, RegistryError> {
    from_yaml(yaml)
}

/// Load a `registry/candidates.yaml` document.
pub fn parse_candidates(yaml: &str) -> Result<Candidates, RegistryError> {
    from_yaml(yaml)
}

/// Load a `registry/experiments.yaml` document.
pub fn parse_experiments(yaml: &str) -> Result<Vec<ExperimentEntry>, RegistryError> {
    from_yaml(yaml)
}

/// Load a `corpus/reference/*.yaml` expected-outcome document.
pub fn parse_reference(yaml: &str) -> Result<Vec<ReferenceEntry>, RegistryError> {
    from_yaml(yaml)
}

/// One finding from [`validate_registries`]. Validation collects every
/// finding rather than failing fast so `ckc registry check` reports the
/// whole set (findings map to §7.4 `schema_invalid` diagnostics at the CLI
/// layer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryFinding {
    /// Two entries in one registry pool share an id.
    Duplicate { pool: &'static str, id: Id },
    /// Two test_source groups of one experiment share a group id.
    DuplicateGroup { experiment: Id, group_id: Id },
    /// Two entries of one reference document assert the same group.
    DuplicateReferenceGroup { path: String, group_id: Id },
    /// A semantically required field of the named entry is empty.
    Empty { entry: Id, field: &'static str },
    /// A reference from `from` names an id its pool does not define.
    Dangling {
        from: Id,
        pool: &'static str,
        id: Id,
    },
    /// §8.4 chain rule: the processing_stage consumes an artifact kind no predecessor
    /// in the pipeline produces.
    ChainBreak {
        pipeline: Id,
        processing_stage: Id,
        kind: Id,
    },
    /// An experiment's expected-outcome ref matches no loaded reference document.
    ReferenceUnresolved { experiment: Id, path: String },
    /// A reference entry asserts a group the referencing experiment does not
    /// define.
    ReferenceGroupUnknown { experiment: Id, group_id: Id },
}

impl fmt::Display for RegistryFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryFinding::Duplicate { pool, id } => write!(f, "duplicate {pool} id {id}"),
            RegistryFinding::DuplicateGroup {
                experiment,
                group_id,
            } => {
                write!(
                    f,
                    "experiment {experiment}: duplicate test_source group {group_id}"
                )
            }
            RegistryFinding::DuplicateReferenceGroup { path, group_id } => {
                write!(f, "reference {path}: duplicate group {group_id}")
            }
            RegistryFinding::Empty { entry, field } => write!(f, "{entry}: {field} is empty"),
            RegistryFinding::Dangling { from, pool, id } => {
                write!(f, "{from}: reference to undefined {pool} {id}")
            }
            RegistryFinding::ChainBreak {
                pipeline,
                processing_stage,
                kind,
            } => write!(
                f,
                "pipeline {pipeline}: processing_stage {processing_stage} consumes {kind}, which no predecessor produces"
            ),
            RegistryFinding::ReferenceUnresolved { experiment, path } => write!(
                f,
                "experiment {experiment}: expected-outcome ref {path} matches no loaded reference document"
            ),
            RegistryFinding::ReferenceGroupUnknown {
                experiment,
                group_id,
            } => {
                write!(
                    f,
                    "experiment {experiment}: reference asserts undefined group {group_id}"
                )
            }
        }
    }
}

impl std::error::Error for RegistryFinding {}

/// Push a [`RegistryFinding::Duplicate`] for every repeated id.
fn note_duplicates<'a>(
    ids: impl Iterator<Item = &'a Id>,
    pool: &'static str,
    findings: &mut Vec<RegistryFinding>,
) {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            findings.push(RegistryFinding::Duplicate {
                pool,
                id: id.clone(),
            });
        }
    }
}

/// Validate the loaded §8.4 registry surface as one cross-referenced set.
///
/// Loading already enforces shape: required fields, [`Id`] grammar, enum
/// spellings. This pass checks what only the whole set can — id uniqueness
/// per pool, semantically required nonempty fields, cross-file resolution
/// (experiment → pipeline → processing_stage entries, test_source refs → corpora, each
/// experiment's expected-outcome ref → `reference`, whose entries must assert
/// groups that experiment defines), and the §8.4 stage-chain rule: every
/// processing_stage's declared input artifact kinds are produced by its predecessors
/// (a chain head declares none). `reference` maps expected-outcome paths exactly
/// as experiment entries write them to their loaded documents; reading
/// those files is the caller's concern. Findings come back in deterministic
/// order — corpora, candidates, experiments, reference internals — and an empty
/// vector means the set is valid (§3 `registry check`).
pub fn validate_registries(
    corpora: &[CorpusEntry],
    candidates: &Candidates,
    experiments: &[ExperimentEntry],
    reference: &BTreeMap<String, Vec<ReferenceEntry>>,
) -> Vec<RegistryFinding> {
    let mut findings = Vec::new();

    note_duplicates(corpora.iter().map(|c| &c.id), "corpora", &mut findings);
    for corpus in corpora {
        if corpus.path.is_empty() {
            findings.push(RegistryFinding::Empty {
                entry: corpus.id.clone(),
                field: "path",
            });
        }
    }

    note_duplicates(
        candidates.processing_stages.iter().map(|s| &s.id),
        "processing_stages",
        &mut findings,
    );
    note_duplicates(
        candidates.pipelines.iter().map(|p| &p.id),
        "pipelines",
        &mut findings,
    );
    let processing_stages_by_id: BTreeMap<&Id, &ProcessingStageEntry> = candidates
        .processing_stages
        .iter()
        .map(|s| (&s.id, s))
        .collect();
    for pipeline in &candidates.pipelines {
        if pipeline.processing_stages.is_empty() {
            findings.push(RegistryFinding::Empty {
                entry: pipeline.id.clone(),
                field: "processing_stages",
            });
        }
        let mut produced: BTreeSet<&Id> = BTreeSet::new();
        for processing_stage_id in &pipeline.processing_stages {
            let Some(processing_stage) = processing_stages_by_id.get(processing_stage_id) else {
                findings.push(RegistryFinding::Dangling {
                    from: pipeline.id.clone(),
                    pool: "processing_stages",
                    id: processing_stage_id.clone(),
                });
                continue;
            };
            for kind in &processing_stage.input_artifact_kinds {
                if !produced.contains(kind) {
                    findings.push(RegistryFinding::ChainBreak {
                        pipeline: pipeline.id.clone(),
                        processing_stage: processing_stage.id.clone(),
                        kind: kind.clone(),
                    });
                }
            }
            produced.extend(&processing_stage.output_artifact_kinds);
        }
    }

    let corpus_ids: BTreeSet<&Id> = corpora.iter().map(|c| &c.id).collect();
    let pipeline_ids: BTreeSet<&Id> = candidates.pipelines.iter().map(|p| &p.id).collect();
    note_duplicates(
        experiments.iter().map(|e| &e.id),
        "experiments",
        &mut findings,
    );
    for experiment in experiments {
        if !pipeline_ids.contains(&experiment.pipeline) {
            findings.push(RegistryFinding::Dangling {
                from: experiment.id.clone(),
                pool: "pipelines",
                id: experiment.pipeline.clone(),
            });
        }
        if experiment.test_source_groups.is_empty() {
            findings.push(RegistryFinding::Empty {
                entry: experiment.id.clone(),
                field: "test_source_groups",
            });
        }
        let mut group_ids = BTreeSet::new();
        for group in &experiment.test_source_groups {
            if !group_ids.insert(&group.group_id) {
                findings.push(RegistryFinding::DuplicateGroup {
                    experiment: experiment.id.clone(),
                    group_id: group.group_id.clone(),
                });
            }
            if group.test_sources.is_empty() {
                findings.push(RegistryFinding::Empty {
                    entry: group.group_id.clone(),
                    field: "test_sources",
                });
            }
            for test_source in &group.test_sources {
                if !corpus_ids.contains(test_source) {
                    findings.push(RegistryFinding::Dangling {
                        from: group.group_id.clone(),
                        pool: "corpora",
                        id: test_source.clone(),
                    });
                }
            }
        }
        if experiment.expected_outcomes.is_empty() {
            findings.push(RegistryFinding::Empty {
                entry: experiment.id.clone(),
                field: "expected_outcomes",
            });
        } else if let Some(entries) = reference.get(&experiment.expected_outcomes) {
            for entry in entries {
                if !group_ids.contains(&entry.group_id) {
                    findings.push(RegistryFinding::ReferenceGroupUnknown {
                        experiment: experiment.id.clone(),
                        group_id: entry.group_id.clone(),
                    });
                }
            }
        } else {
            findings.push(RegistryFinding::ReferenceUnresolved {
                experiment: experiment.id.clone(),
                path: experiment.expected_outcomes.clone(),
            });
        }
    }

    for (path, entries) in reference {
        let mut seen = BTreeSet::new();
        for entry in entries {
            if !seen.insert(&entry.group_id) {
                findings.push(RegistryFinding::DuplicateReferenceGroup {
                    path: path.clone(),
                    group_id: entry.group_id.clone(),
                });
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    /// Assert `value` survives a YAML write -> read round trip unchanged.
    fn round_trip<T>(value: &T)
    where
        T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let yaml = to_yaml(value).unwrap();
        let got: T = from_yaml(&yaml).unwrap();
        assert_eq!(&got, value, "yaml round trip changed the value:\n{yaml}");
    }

    const CORPORA: &str = "\
- id: test_source.m1_guideline_a
  path: corpus/test_sources/m1_guideline_a.html
  origin: ai_generated
  evidence_status: source_evidence_status
  provenance: synthetic
- id: test_source.m1_guideline_b
  path: corpus/test_sources/m1_guideline_b.html
  origin: ai_generated
  evidence_status: source_evidence_status
  provenance: synthetic
- id: test_source.m1_control
  path: corpus/test_sources/m1_control.html
  origin: ai_generated
  evidence_status: source_evidence_status
  provenance: synthetic
";

    const CANDIDATES: &str = "\
pipelines:
  - id: pipe.layered_ckcir_to_smt
    processing_stages: [processing_stage.m1.extract, processing_stage.m1.segment]
processing_stages:
  - id: processing_stage.m1.extract
    kind: extract
    determinism: deterministic
    input_artifact_kinds: []
    output_artifact_kinds: [source_document_graph]
  - id: processing_stage.m1.segment
    kind: segment
    determinism: deterministic
    input_artifact_kinds: [source_document_graph]
    output_artifact_kinds: [segments]
";

    const EXPERIMENTS: &str = "\
- id: exp.m1_scaffold
  pipeline: pipe.layered_ckcir_to_smt
  test_source_groups:
    - group_id: group.m1_conflict
      test_sources: [test_source.m1_guideline_a, test_source.m1_guideline_b]
    - group_id: group.m1_no_conflict
      test_sources: [test_source.m1_guideline_a, test_source.m1_control]
  seed: 42
  budget:
    solver_ms_per_query: 10000
  expected_outcomes: corpus/reference/m1_expected.yaml
";

    // Verbatim §8.2 reference block, comment included.
    const REFERENCE: &str = "\
- group_id: group.m1_conflict
  expected_outcome: semantic_contradiction
  expected_conflict_kind: deontic_direction_conflict
  expected_unsat_core: [a.test_source.m1_guideline_a.rule.0, a.test_source.m1_guideline_b.rule.0]   # compared as a set
- group_id: group.m1_no_conflict
  expected_outcome: semantic_no_conflict
  expected_no_conflict_result: true
";

    // §8.2 corpora acceptance fields load typed: ai_generated origin under
    // source_evidence_status, synthetic provenance.
    #[test]
    fn corpora_load_typed() {
        let corpora = parse_corpora(CORPORA).unwrap();
        assert_eq!(corpora.len(), 3);
        assert_eq!(corpora[0].id, id("test_source.m1_guideline_a"));
        assert_eq!(corpora[0].path, "corpus/test_sources/m1_guideline_a.html");
        for entry in &corpora {
            assert_eq!(entry.origin, Origin::AiGenerated);
            assert_eq!(entry.evidence_status, EvidenceStatus::SourceEvidenceStatus);
            assert_eq!(entry.provenance, Provenance::Synthetic);
        }
    }

    // §8.4 candidates: the pipeline chains processing_stage entries by id; processing_stages
    // declare role, determinism, and input/output artifact kinds.
    #[test]
    fn candidates_load_typed() {
        let candidates = parse_candidates(CANDIDATES).unwrap();
        assert_eq!(candidates.pipelines.len(), 1);
        let pipe = &candidates.pipelines[0];
        assert_eq!(pipe.id, id("pipe.layered_ckcir_to_smt"));
        assert_eq!(
            pipe.processing_stages,
            vec![
                id("processing_stage.m1.extract"),
                id("processing_stage.m1.segment")
            ]
        );
        let extract = &candidates.processing_stages[0];
        assert_eq!(extract.kind, id("extract"));
        assert_eq!(extract.determinism, Determinism::Deterministic);
        assert!(extract.input_artifact_kinds.is_empty());
        assert_eq!(
            extract.output_artifact_kinds,
            vec![id("source_document_graph")]
        );
        assert_eq!(
            candidates.processing_stages[1].input_artifact_kinds,
            vec![id("source_document_graph")]
        );
    }

    // §8.4 experiments: test_source groups, pipeline ref, seed, budget map, and
    // the expected-outcome ref.
    #[test]
    fn experiments_load_typed() {
        let experiments = parse_experiments(EXPERIMENTS).unwrap();
        assert_eq!(experiments.len(), 1);
        let exp = &experiments[0];
        assert_eq!(exp.id, id("exp.m1_scaffold"));
        assert_eq!(exp.pipeline, id("pipe.layered_ckcir_to_smt"));
        assert_eq!(exp.test_source_groups.len(), 2);
        assert_eq!(exp.test_source_groups[0].group_id, id("group.m1_conflict"));
        assert_eq!(
            exp.test_source_groups[1].test_sources,
            vec![
                id("test_source.m1_guideline_a"),
                id("test_source.m1_control")
            ]
        );
        assert_eq!(exp.seed, 42);
        assert_eq!(exp.budget[&id("solver_ms_per_query")], 10_000);
        assert_eq!(exp.expected_outcomes, "corpus/reference/m1_expected.yaml");
    }

    // The verbatim §8.2 reference block loads typed; absent optionals take their
    // defaults, and expected_unsat_core is order-insensitive (set comparison).
    #[test]
    fn reference_loads_spec_shape() {
        let reference = parse_reference(REFERENCE).unwrap();
        assert_eq!(reference.len(), 2);
        let conflict = &reference[0];
        assert_eq!(conflict.group_id, id("group.m1_conflict"));
        assert_eq!(conflict.expected_outcome, id("semantic_contradiction"));
        assert_eq!(
            conflict.expected_conflict_kind,
            Some(id("deontic_direction_conflict"))
        );
        assert!(!conflict.expected_no_conflict_result);
        let no_conflict = &reference[1];
        assert_eq!(no_conflict.expected_outcome, id("semantic_no_conflict"));
        assert!(no_conflict.expected_no_conflict_result);
        assert_eq!(no_conflict.expected_conflict_kind, None);
        assert!(no_conflict.expected_unsat_core.is_empty());

        let reordered = REFERENCE.replace(
            "[a.test_source.m1_guideline_a.rule.0, a.test_source.m1_guideline_b.rule.0]",
            "[a.test_source.m1_guideline_b.rule.0, a.test_source.m1_guideline_a.rule.0]",
        );
        assert_eq!(parse_reference(&reordered).unwrap()[0], *conflict);
        assert_eq!(
            conflict.expected_unsat_core,
            BTreeSet::from([
                id("a.test_source.m1_guideline_a.rule.0"),
                id("a.test_source.m1_guideline_b.rule.0"),
            ])
        );
    }

    #[test]
    fn all_documents_round_trip() {
        round_trip(&parse_corpora(CORPORA).unwrap());
        round_trip(&parse_candidates(CANDIDATES).unwrap());
        round_trip(&parse_experiments(EXPERIMENTS).unwrap());
        round_trip(&parse_reference(REFERENCE).unwrap());
    }

    // Strict loading: unknown fields, Id-grammar violations, and unknown
    // enum spellings are load-time errors.
    #[test]
    fn strict_loading_rejects_bad_documents() {
        let unknown_field = CORPORA.replace("  path:", "  surprise: 1\n  path:");
        assert!(parse_corpora(&unknown_field).is_err());
        let bad_id = REFERENCE.replace("group.m1_conflict", "Group.M1_Conflict");
        assert!(parse_reference(&bad_id).is_err());
        let bad_enum = CORPORA.replace("origin: ai_generated", "origin: vibes");
        assert!(parse_corpora(&bad_enum).is_err());
        let missing_field = EXPERIMENTS.replace("  seed: 42\n", "");
        assert!(parse_experiments(&missing_field).is_err());
    }

    const REFERENCE_PATH: &str = "corpus/reference/m1_expected.yaml";

    /// The inline §8.2/§8.4 documents loaded as one registry set, with the
    /// reference document supplied under the path EXPERIMENTS references.
    fn valid_set() -> (
        Vec<CorpusEntry>,
        Candidates,
        Vec<ExperimentEntry>,
        BTreeMap<String, Vec<ReferenceEntry>>,
    ) {
        (
            parse_corpora(CORPORA).unwrap(),
            parse_candidates(CANDIDATES).unwrap(),
            parse_experiments(EXPERIMENTS).unwrap(),
            BTreeMap::from([(
                REFERENCE_PATH.to_string(),
                parse_reference(REFERENCE).unwrap(),
            )]),
        )
    }

    // The M1 set cross-resolves cleanly: zero findings.
    #[test]
    fn validate_accepts_the_m1_set() {
        let (corpora, candidates, experiments, reference) = valid_set();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![]
        );
    }

    // §8.4 chain rule: inputs are satisfied only by predecessors — a
    // reversed chain and an unfed appended consumer both break.
    #[test]
    fn chain_rule_requires_predecessor_production() {
        let (corpora, mut candidates, experiments, reference) = valid_set();
        candidates.pipelines[0].processing_stages.reverse();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::ChainBreak {
                pipeline: id("pipe.layered_ckcir_to_smt"),
                processing_stage: id("processing_stage.m1.segment"),
                kind: id("source_document_graph"),
            }]
        );

        let (corpora, mut candidates, experiments, reference) = valid_set();
        candidates.processing_stages.push(ProcessingStageEntry {
            id: id("processing_stage.m1.compile"),
            kind: id("compile"),
            determinism: Determinism::Deterministic,
            input_artifact_kinds: vec![id("ir_bundle")],
            output_artifact_kinds: vec![id("compiled")],
        });
        candidates.pipelines[0]
            .processing_stages
            .push(id("processing_stage.m1.compile"));
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::ChainBreak {
                pipeline: id("pipe.layered_ckcir_to_smt"),
                processing_stage: id("processing_stage.m1.compile"),
                kind: id("ir_bundle"),
            }]
        );
    }

    // Dangling refs surface per edge: experiment → pipeline, pipeline →
    // processing_stage, test_source group → corpus.
    #[test]
    fn dangling_references_are_findings() {
        let (corpora, candidates, mut experiments, reference) = valid_set();
        experiments[0].pipeline = id("pipe.missing");
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::Dangling {
                from: id("exp.m1_scaffold"),
                pool: "pipelines",
                id: id("pipe.missing"),
            }]
        );

        let (corpora, mut candidates, experiments, reference) = valid_set();
        candidates.pipelines[0].processing_stages[1] = id("processing_stage.m1.missing");
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::Dangling {
                from: id("pipe.layered_ckcir_to_smt"),
                pool: "processing_stages",
                id: id("processing_stage.m1.missing"),
            }]
        );

        let (corpora, candidates, mut experiments, reference) = valid_set();
        experiments[0].test_source_groups[1].test_sources[1] = id("test_source.m1_missing");
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::Dangling {
                from: id("group.m1_no_conflict"),
                pool: "corpora",
                id: id("test_source.m1_missing"),
            }]
        );
    }

    // Id uniqueness: per pool, per experiment's groups, per reference document.
    #[test]
    fn duplicate_ids_are_findings() {
        let (mut corpora, candidates, experiments, reference) = valid_set();
        let dup = corpora[0].clone();
        corpora.push(dup);
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::Duplicate {
                pool: "corpora",
                id: id("test_source.m1_guideline_a"),
            }]
        );

        let (corpora, mut candidates, experiments, reference) = valid_set();
        let processing_stage = candidates.processing_stages[0].clone();
        candidates.processing_stages.push(processing_stage);
        let pipe = candidates.pipelines[0].clone();
        candidates.pipelines.push(pipe);
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![
                RegistryFinding::Duplicate {
                    pool: "processing_stages",
                    id: id("processing_stage.m1.extract"),
                },
                RegistryFinding::Duplicate {
                    pool: "pipelines",
                    id: id("pipe.layered_ckcir_to_smt"),
                },
            ]
        );

        let (corpora, candidates, mut experiments, reference) = valid_set();
        let exp = experiments[0].clone();
        experiments.push(exp);
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::Duplicate {
                pool: "experiments",
                id: id("exp.m1_scaffold"),
            }]
        );

        let (corpora, candidates, mut experiments, reference) = valid_set();
        let group = experiments[0].test_source_groups[0].clone();
        experiments[0].test_source_groups.push(group);
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::DuplicateGroup {
                experiment: id("exp.m1_scaffold"),
                group_id: id("group.m1_conflict"),
            }]
        );

        let (corpora, candidates, experiments, mut reference) = valid_set();
        let entries = reference.get_mut(REFERENCE_PATH).unwrap();
        let entry = entries[0].clone();
        entries.push(entry);
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::DuplicateReferenceGroup {
                path: REFERENCE_PATH.to_string(),
                group_id: id("group.m1_conflict"),
            }]
        );
    }

    // Semantically required nonempty fields; an experiment without groups
    // also orphans its reference assertions.
    #[test]
    fn empty_required_fields_are_findings() {
        let (mut corpora, candidates, experiments, reference) = valid_set();
        corpora[2].path.clear();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::Empty {
                entry: id("test_source.m1_control"),
                field: "path",
            }]
        );

        let (corpora, mut candidates, experiments, reference) = valid_set();
        candidates.pipelines[0].processing_stages.clear();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::Empty {
                entry: id("pipe.layered_ckcir_to_smt"),
                field: "processing_stages",
            }]
        );

        let (corpora, candidates, mut experiments, reference) = valid_set();
        experiments[0].test_source_groups.clear();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![
                RegistryFinding::Empty {
                    entry: id("exp.m1_scaffold"),
                    field: "test_source_groups",
                },
                RegistryFinding::ReferenceGroupUnknown {
                    experiment: id("exp.m1_scaffold"),
                    group_id: id("group.m1_conflict"),
                },
                RegistryFinding::ReferenceGroupUnknown {
                    experiment: id("exp.m1_scaffold"),
                    group_id: id("group.m1_no_conflict"),
                },
            ]
        );

        let (corpora, candidates, mut experiments, reference) = valid_set();
        experiments[0].test_source_groups[0].test_sources.clear();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::Empty {
                entry: id("group.m1_conflict"),
                field: "test_sources",
            }]
        );

        let (corpora, candidates, mut experiments, reference) = valid_set();
        experiments[0].expected_outcomes.clear();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::Empty {
                entry: id("exp.m1_scaffold"),
                field: "expected_outcomes",
            }]
        );
    }

    // Expected-outcome refs resolve against loaded reference documents whose
    // entries assert groups the experiment defines.
    #[test]
    fn reference_resolution_findings() {
        let (corpora, candidates, experiments, _) = valid_set();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &BTreeMap::new()),
            vec![RegistryFinding::ReferenceUnresolved {
                experiment: id("exp.m1_scaffold"),
                path: REFERENCE_PATH.to_string(),
            }]
        );

        let (corpora, candidates, experiments, mut reference) = valid_set();
        reference.get_mut(REFERENCE_PATH).unwrap()[1].group_id = id("group.m1_extra");
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &reference),
            vec![RegistryFinding::ReferenceGroupUnknown {
                experiment: id("exp.m1_scaffold"),
                group_id: id("group.m1_extra"),
            }]
        );
    }
}
