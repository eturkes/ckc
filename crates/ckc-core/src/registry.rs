//! SPEC §8.4 registry entry types — the YAML-edited control surface a run
//! resolves against — plus the §8.2 gold expected-outcome entries.
//!
//! One type family per file: `registry/corpora.yaml` is a list of
//! [`CorpusEntry`] (admission fields per §8.2), `registry/candidates.yaml` is
//! one [`Candidates`] document holding [`PipelineEntry`] and [`StageEntry`]
//! components, `registry/experiments.yaml` is a list of [`ExperimentEntry`]
//! (what `ckc run --experiment` resolves), and `corpus/gold/*.yaml` is a list
//! of [`GoldEntry`] asserted by acceptance tests. Loading is strict: unknown
//! fields are rejected, [`Id`] fields are grammar-checked by `Id`'s serde,
//! and enum fields admit exactly their canonical spellings. Past loading,
//! [`validate_registries`] checks the set semantically — pool-level id
//! uniqueness, nonempty requirements, cross-file resolution, the §8.4
//! stage-chain rule — collecting every [`RegistryFinding`] so §3
//! `ckc registry check` reports the whole set.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::enums::{Authority, Origin, fieldless_enum};
use crate::grounding::Provenance;
use crate::id::Id;

fieldless_enum! {
    /// SPEC §8.4 stage-component determinism class. Every M1 component is
    /// `deterministic`; `nondeterministic` marks components whose reruns may
    /// diverge (M2's recorded weak-model routes), which replay handles
    /// through recorded I/O rather than re-execution.
    Determinism {
        Deterministic => "deterministic",
        Nondeterministic => "nondeterministic",
    }
}

/// One `registry/corpora.yaml` entry: a corpus document admitted with the
/// §8.2 fields — a working example of admission-over-proposer authority
/// (`ai_generated` origin under `source_authority`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEntry {
    /// Document id experiments and groups reference (e.g.
    /// `fixture.m1_guideline_a`).
    pub id: Id,
    /// Document path relative to the repository root.
    pub path: String,
    pub origin: Origin,
    /// Authority granted on admission.
    pub authority: Authority,
    pub provenance: Provenance,
}

/// `registry/candidates.yaml`: the §8.4 candidate components — pipelines and
/// the stage components they chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidates {
    pub pipelines: Vec<PipelineEntry>,
    pub stages: Vec<StageEntry>,
}

/// A pipeline candidate: an ordered chain of [`StageEntry`] ids. The §8.4
/// chain rule (every stage's declared input artifact kinds are produced by
/// its predecessors) is checked by registry validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineEntry {
    /// Pipeline id experiments reference (e.g. `pipe.layered_ckcir_to_smt`).
    pub id: Id,
    /// Stage-component ids in execution order.
    pub stages: Vec<Id>,
}

/// A stage component candidate: one pipeline step with its §8.4 fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageEntry {
    /// Stage-component id pipelines reference.
    pub id: Id,
    /// Stage role (§8.3 vocabulary: `extract`, `segment`, `normalize`,
    /// `assemble`, `compile`, `verify`, `trace`, `report`); open so later
    /// milestones add roles without reshaping entries.
    pub kind: Id,
    pub determinism: Determinism,
    /// Artifact kinds this stage consumes; empty for a chain head whose
    /// input is the corpus document itself.
    pub input_artifact_kinds: Vec<Id>,
    /// Artifact kinds this stage produces.
    pub output_artifact_kinds: Vec<Id>,
}

/// One `registry/experiments.yaml` entry: what `ckc run --experiment`
/// resolves into a §5 RunPlan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentEntry {
    /// Experiment id (e.g. `exp.m1_spine`).
    pub id: Id,
    /// Pipeline candidate this experiment executes.
    pub pipeline: Id,
    /// Fixture groups in evaluation order.
    pub fixture_groups: Vec<FixtureGroup>,
    /// Deterministic seed for any seeded component.
    pub seed: u64,
    /// Budget caps: counter name → limit (the counters §4.6
    /// `budget_counters` consume against).
    pub budget: BTreeMap<Id, u64>,
    /// Expected-outcome ref: path of the gold file ([`GoldEntry`] list)
    /// asserted against this experiment's groups, relative to the
    /// repository root.
    pub expected_outcomes: String,
}

/// A §8.2 fixture group: the corpus documents one verdict is computed over.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureGroup {
    /// Group id (e.g. `group.m1_conflict`).
    pub group_id: Id,
    /// Member [`CorpusEntry`] ids, in semantic order.
    pub fixtures: Vec<Id>,
}

/// One `corpus/gold/*.yaml` entry: the §8.2 expected outcome for a fixture
/// group, asserted by the acceptance tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldEntry {
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
    pub expected_core: BTreeSet<Id>,
    /// Whether the group's expected finding is a documented null result.
    #[serde(default, skip_serializing_if = "is_false")]
    pub expected_null_result: bool,
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

/// Load a `corpus/gold/*.yaml` expected-outcome document.
pub fn parse_gold(yaml: &str) -> Result<Vec<GoldEntry>, RegistryError> {
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
    /// Two fixture groups of one experiment share a group id.
    DuplicateGroup { experiment: Id, group_id: Id },
    /// Two entries of one gold document assert the same group.
    DuplicateGoldGroup { path: String, group_id: Id },
    /// A semantically required field of the named entry is empty.
    Empty { entry: Id, field: &'static str },
    /// A reference from `from` names an id its pool does not define.
    Dangling {
        from: Id,
        pool: &'static str,
        id: Id,
    },
    /// §8.4 chain rule: the stage consumes an artifact kind no predecessor
    /// in the pipeline produces.
    ChainBreak { pipeline: Id, stage: Id, kind: Id },
    /// An experiment's expected-outcome ref matches no loaded gold document.
    GoldUnresolved { experiment: Id, path: String },
    /// A gold entry asserts a group the referencing experiment does not
    /// define.
    GoldGroupUnknown { experiment: Id, group_id: Id },
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
                    "experiment {experiment}: duplicate fixture group {group_id}"
                )
            }
            RegistryFinding::DuplicateGoldGroup { path, group_id } => {
                write!(f, "gold {path}: duplicate group {group_id}")
            }
            RegistryFinding::Empty { entry, field } => write!(f, "{entry}: {field} is empty"),
            RegistryFinding::Dangling { from, pool, id } => {
                write!(f, "{from}: reference to undefined {pool} {id}")
            }
            RegistryFinding::ChainBreak {
                pipeline,
                stage,
                kind,
            } => write!(
                f,
                "pipeline {pipeline}: stage {stage} consumes {kind}, which no predecessor produces"
            ),
            RegistryFinding::GoldUnresolved { experiment, path } => write!(
                f,
                "experiment {experiment}: expected-outcome ref {path} matches no loaded gold document"
            ),
            RegistryFinding::GoldGroupUnknown {
                experiment,
                group_id,
            } => {
                write!(
                    f,
                    "experiment {experiment}: gold asserts undefined group {group_id}"
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
/// (experiment → pipeline → stage components, fixture refs → corpora, each
/// experiment's expected-outcome ref → `gold`, whose entries must assert
/// groups that experiment defines), and the §8.4 stage-chain rule: every
/// stage's declared input artifact kinds are produced by its predecessors
/// (a chain head declares none). `gold` maps expected-outcome paths exactly
/// as experiment entries write them to their loaded documents; reading
/// those files is the caller's concern. Findings come back in deterministic
/// order — corpora, candidates, experiments, gold internals — and an empty
/// vector means the set is valid (§3 `registry check`).
pub fn validate_registries(
    corpora: &[CorpusEntry],
    candidates: &Candidates,
    experiments: &[ExperimentEntry],
    gold: &BTreeMap<String, Vec<GoldEntry>>,
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
        candidates.stages.iter().map(|s| &s.id),
        "stages",
        &mut findings,
    );
    note_duplicates(
        candidates.pipelines.iter().map(|p| &p.id),
        "pipelines",
        &mut findings,
    );
    let stages_by_id: BTreeMap<&Id, &StageEntry> =
        candidates.stages.iter().map(|s| (&s.id, s)).collect();
    for pipeline in &candidates.pipelines {
        if pipeline.stages.is_empty() {
            findings.push(RegistryFinding::Empty {
                entry: pipeline.id.clone(),
                field: "stages",
            });
        }
        let mut produced: BTreeSet<&Id> = BTreeSet::new();
        for stage_id in &pipeline.stages {
            let Some(stage) = stages_by_id.get(stage_id) else {
                findings.push(RegistryFinding::Dangling {
                    from: pipeline.id.clone(),
                    pool: "stages",
                    id: stage_id.clone(),
                });
                continue;
            };
            for kind in &stage.input_artifact_kinds {
                if !produced.contains(kind) {
                    findings.push(RegistryFinding::ChainBreak {
                        pipeline: pipeline.id.clone(),
                        stage: stage.id.clone(),
                        kind: kind.clone(),
                    });
                }
            }
            produced.extend(&stage.output_artifact_kinds);
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
        if experiment.fixture_groups.is_empty() {
            findings.push(RegistryFinding::Empty {
                entry: experiment.id.clone(),
                field: "fixture_groups",
            });
        }
        let mut group_ids = BTreeSet::new();
        for group in &experiment.fixture_groups {
            if !group_ids.insert(&group.group_id) {
                findings.push(RegistryFinding::DuplicateGroup {
                    experiment: experiment.id.clone(),
                    group_id: group.group_id.clone(),
                });
            }
            if group.fixtures.is_empty() {
                findings.push(RegistryFinding::Empty {
                    entry: group.group_id.clone(),
                    field: "fixtures",
                });
            }
            for fixture in &group.fixtures {
                if !corpus_ids.contains(fixture) {
                    findings.push(RegistryFinding::Dangling {
                        from: group.group_id.clone(),
                        pool: "corpora",
                        id: fixture.clone(),
                    });
                }
            }
        }
        if experiment.expected_outcomes.is_empty() {
            findings.push(RegistryFinding::Empty {
                entry: experiment.id.clone(),
                field: "expected_outcomes",
            });
        } else if let Some(entries) = gold.get(&experiment.expected_outcomes) {
            for entry in entries {
                if !group_ids.contains(&entry.group_id) {
                    findings.push(RegistryFinding::GoldGroupUnknown {
                        experiment: experiment.id.clone(),
                        group_id: entry.group_id.clone(),
                    });
                }
            }
        } else {
            findings.push(RegistryFinding::GoldUnresolved {
                experiment: experiment.id.clone(),
                path: experiment.expected_outcomes.clone(),
            });
        }
    }

    for (path, entries) in gold {
        let mut seen = BTreeSet::new();
        for entry in entries {
            if !seen.insert(&entry.group_id) {
                findings.push(RegistryFinding::DuplicateGoldGroup {
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
- id: fixture.m1_guideline_a
  path: corpus/fixtures/m1_guideline_a.html
  origin: ai_generated
  authority: source_authority
  provenance: synthetic
- id: fixture.m1_guideline_b
  path: corpus/fixtures/m1_guideline_b.html
  origin: ai_generated
  authority: source_authority
  provenance: synthetic
- id: fixture.m1_control
  path: corpus/fixtures/m1_control.html
  origin: ai_generated
  authority: source_authority
  provenance: synthetic
";

    const CANDIDATES: &str = "\
pipelines:
  - id: pipe.layered_ckcir_to_smt
    stages: [stage.m1.extract, stage.m1.segment]
stages:
  - id: stage.m1.extract
    kind: extract
    determinism: deterministic
    input_artifact_kinds: []
    output_artifact_kinds: [source_graph]
  - id: stage.m1.segment
    kind: segment
    determinism: deterministic
    input_artifact_kinds: [source_graph]
    output_artifact_kinds: [segments]
";

    const EXPERIMENTS: &str = "\
- id: exp.m1_spine
  pipeline: pipe.layered_ckcir_to_smt
  fixture_groups:
    - group_id: group.m1_conflict
      fixtures: [fixture.m1_guideline_a, fixture.m1_guideline_b]
    - group_id: group.m1_null
      fixtures: [fixture.m1_guideline_a, fixture.m1_control]
  seed: 42
  budget:
    solver_ms_per_query: 10000
  expected_outcomes: corpus/gold/m1_expected.yaml
";

    // Verbatim §8.2 gold block, comment included.
    const GOLD: &str = "\
- group_id: group.m1_conflict
  expected_outcome: semantic_contradiction
  expected_conflict_kind: deontic_direction_conflict
  expected_core: [a.fixture.m1_guideline_a.rule.0, a.fixture.m1_guideline_b.rule.0]   # compared as a set
- group_id: group.m1_null
  expected_outcome: semantic_no_conflict
  expected_null_result: true
";

    // §8.2 corpora admission fields load typed: ai_generated origin under
    // source_authority, synthetic provenance.
    #[test]
    fn corpora_load_typed() {
        let corpora = parse_corpora(CORPORA).unwrap();
        assert_eq!(corpora.len(), 3);
        assert_eq!(corpora[0].id, id("fixture.m1_guideline_a"));
        assert_eq!(corpora[0].path, "corpus/fixtures/m1_guideline_a.html");
        for entry in &corpora {
            assert_eq!(entry.origin, Origin::AiGenerated);
            assert_eq!(entry.authority, Authority::SourceAuthority);
            assert_eq!(entry.provenance, Provenance::Synthetic);
        }
    }

    // §8.4 candidates: the pipeline chains stage components by id; stages
    // declare role, determinism, and input/output artifact kinds.
    #[test]
    fn candidates_load_typed() {
        let candidates = parse_candidates(CANDIDATES).unwrap();
        assert_eq!(candidates.pipelines.len(), 1);
        let pipe = &candidates.pipelines[0];
        assert_eq!(pipe.id, id("pipe.layered_ckcir_to_smt"));
        assert_eq!(
            pipe.stages,
            vec![id("stage.m1.extract"), id("stage.m1.segment")]
        );
        let extract = &candidates.stages[0];
        assert_eq!(extract.kind, id("extract"));
        assert_eq!(extract.determinism, Determinism::Deterministic);
        assert!(extract.input_artifact_kinds.is_empty());
        assert_eq!(extract.output_artifact_kinds, vec![id("source_graph")]);
        assert_eq!(
            candidates.stages[1].input_artifact_kinds,
            vec![id("source_graph")]
        );
    }

    // §8.4 experiments: fixture groups, pipeline ref, seed, budget map, and
    // the expected-outcome ref.
    #[test]
    fn experiments_load_typed() {
        let experiments = parse_experiments(EXPERIMENTS).unwrap();
        assert_eq!(experiments.len(), 1);
        let exp = &experiments[0];
        assert_eq!(exp.id, id("exp.m1_spine"));
        assert_eq!(exp.pipeline, id("pipe.layered_ckcir_to_smt"));
        assert_eq!(exp.fixture_groups.len(), 2);
        assert_eq!(exp.fixture_groups[0].group_id, id("group.m1_conflict"));
        assert_eq!(
            exp.fixture_groups[1].fixtures,
            vec![id("fixture.m1_guideline_a"), id("fixture.m1_control")]
        );
        assert_eq!(exp.seed, 42);
        assert_eq!(exp.budget[&id("solver_ms_per_query")], 10_000);
        assert_eq!(exp.expected_outcomes, "corpus/gold/m1_expected.yaml");
    }

    // The verbatim §8.2 gold block loads typed; absent optionals take their
    // defaults, and expected_core is order-insensitive (set comparison).
    #[test]
    fn gold_loads_spec_shape() {
        let gold = parse_gold(GOLD).unwrap();
        assert_eq!(gold.len(), 2);
        let conflict = &gold[0];
        assert_eq!(conflict.group_id, id("group.m1_conflict"));
        assert_eq!(conflict.expected_outcome, id("semantic_contradiction"));
        assert_eq!(
            conflict.expected_conflict_kind,
            Some(id("deontic_direction_conflict"))
        );
        assert!(!conflict.expected_null_result);
        let null = &gold[1];
        assert_eq!(null.expected_outcome, id("semantic_no_conflict"));
        assert!(null.expected_null_result);
        assert_eq!(null.expected_conflict_kind, None);
        assert!(null.expected_core.is_empty());

        let reordered = GOLD.replace(
            "[a.fixture.m1_guideline_a.rule.0, a.fixture.m1_guideline_b.rule.0]",
            "[a.fixture.m1_guideline_b.rule.0, a.fixture.m1_guideline_a.rule.0]",
        );
        assert_eq!(parse_gold(&reordered).unwrap()[0], *conflict);
        assert_eq!(
            conflict.expected_core,
            BTreeSet::from([
                id("a.fixture.m1_guideline_a.rule.0"),
                id("a.fixture.m1_guideline_b.rule.0"),
            ])
        );
    }

    #[test]
    fn all_documents_round_trip() {
        round_trip(&parse_corpora(CORPORA).unwrap());
        round_trip(&parse_candidates(CANDIDATES).unwrap());
        round_trip(&parse_experiments(EXPERIMENTS).unwrap());
        round_trip(&parse_gold(GOLD).unwrap());
    }

    // Strict loading: unknown fields, Id-grammar violations, and unknown
    // enum spellings are load-time errors.
    #[test]
    fn strict_loading_rejects_bad_documents() {
        let unknown_field = CORPORA.replace("  path:", "  surprise: 1\n  path:");
        assert!(parse_corpora(&unknown_field).is_err());
        let bad_id = GOLD.replace("group.m1_conflict", "Group.M1_Conflict");
        assert!(parse_gold(&bad_id).is_err());
        let bad_enum = CORPORA.replace("origin: ai_generated", "origin: vibes");
        assert!(parse_corpora(&bad_enum).is_err());
        let missing_field = EXPERIMENTS.replace("  seed: 42\n", "");
        assert!(parse_experiments(&missing_field).is_err());
    }

    const GOLD_PATH: &str = "corpus/gold/m1_expected.yaml";

    /// The inline §8.2/§8.4 documents loaded as one registry set, with the
    /// gold document supplied under the path EXPERIMENTS references.
    fn valid_set() -> (
        Vec<CorpusEntry>,
        Candidates,
        Vec<ExperimentEntry>,
        BTreeMap<String, Vec<GoldEntry>>,
    ) {
        (
            parse_corpora(CORPORA).unwrap(),
            parse_candidates(CANDIDATES).unwrap(),
            parse_experiments(EXPERIMENTS).unwrap(),
            BTreeMap::from([(GOLD_PATH.to_string(), parse_gold(GOLD).unwrap())]),
        )
    }

    // The M1 set cross-resolves cleanly: zero findings.
    #[test]
    fn validate_accepts_the_m1_set() {
        let (corpora, candidates, experiments, gold) = valid_set();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![]
        );
    }

    // §8.4 chain rule: inputs are satisfied only by predecessors — a
    // reversed chain and an unfed appended consumer both break.
    #[test]
    fn chain_rule_requires_predecessor_production() {
        let (corpora, mut candidates, experiments, gold) = valid_set();
        candidates.pipelines[0].stages.reverse();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::ChainBreak {
                pipeline: id("pipe.layered_ckcir_to_smt"),
                stage: id("stage.m1.segment"),
                kind: id("source_graph"),
            }]
        );

        let (corpora, mut candidates, experiments, gold) = valid_set();
        candidates.stages.push(StageEntry {
            id: id("stage.m1.compile"),
            kind: id("compile"),
            determinism: Determinism::Deterministic,
            input_artifact_kinds: vec![id("ir_bundle")],
            output_artifact_kinds: vec![id("compiled")],
        });
        candidates.pipelines[0].stages.push(id("stage.m1.compile"));
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::ChainBreak {
                pipeline: id("pipe.layered_ckcir_to_smt"),
                stage: id("stage.m1.compile"),
                kind: id("ir_bundle"),
            }]
        );
    }

    // Dangling refs surface per edge: experiment → pipeline, pipeline →
    // stage, fixture group → corpus.
    #[test]
    fn dangling_references_are_findings() {
        let (corpora, candidates, mut experiments, gold) = valid_set();
        experiments[0].pipeline = id("pipe.missing");
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::Dangling {
                from: id("exp.m1_spine"),
                pool: "pipelines",
                id: id("pipe.missing"),
            }]
        );

        let (corpora, mut candidates, experiments, gold) = valid_set();
        candidates.pipelines[0].stages[1] = id("stage.m1.missing");
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::Dangling {
                from: id("pipe.layered_ckcir_to_smt"),
                pool: "stages",
                id: id("stage.m1.missing"),
            }]
        );

        let (corpora, candidates, mut experiments, gold) = valid_set();
        experiments[0].fixture_groups[1].fixtures[1] = id("fixture.m1_missing");
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::Dangling {
                from: id("group.m1_null"),
                pool: "corpora",
                id: id("fixture.m1_missing"),
            }]
        );
    }

    // Id uniqueness: per pool, per experiment's groups, per gold document.
    #[test]
    fn duplicate_ids_are_findings() {
        let (mut corpora, candidates, experiments, gold) = valid_set();
        let dup = corpora[0].clone();
        corpora.push(dup);
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::Duplicate {
                pool: "corpora",
                id: id("fixture.m1_guideline_a"),
            }]
        );

        let (corpora, mut candidates, experiments, gold) = valid_set();
        let stage = candidates.stages[0].clone();
        candidates.stages.push(stage);
        let pipe = candidates.pipelines[0].clone();
        candidates.pipelines.push(pipe);
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![
                RegistryFinding::Duplicate {
                    pool: "stages",
                    id: id("stage.m1.extract"),
                },
                RegistryFinding::Duplicate {
                    pool: "pipelines",
                    id: id("pipe.layered_ckcir_to_smt"),
                },
            ]
        );

        let (corpora, candidates, mut experiments, gold) = valid_set();
        let exp = experiments[0].clone();
        experiments.push(exp);
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::Duplicate {
                pool: "experiments",
                id: id("exp.m1_spine"),
            }]
        );

        let (corpora, candidates, mut experiments, gold) = valid_set();
        let group = experiments[0].fixture_groups[0].clone();
        experiments[0].fixture_groups.push(group);
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::DuplicateGroup {
                experiment: id("exp.m1_spine"),
                group_id: id("group.m1_conflict"),
            }]
        );

        let (corpora, candidates, experiments, mut gold) = valid_set();
        let entries = gold.get_mut(GOLD_PATH).unwrap();
        let entry = entries[0].clone();
        entries.push(entry);
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::DuplicateGoldGroup {
                path: GOLD_PATH.to_string(),
                group_id: id("group.m1_conflict"),
            }]
        );
    }

    // Semantically required nonempty fields; an experiment without groups
    // also orphans its gold assertions.
    #[test]
    fn empty_required_fields_are_findings() {
        let (mut corpora, candidates, experiments, gold) = valid_set();
        corpora[2].path.clear();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::Empty {
                entry: id("fixture.m1_control"),
                field: "path",
            }]
        );

        let (corpora, mut candidates, experiments, gold) = valid_set();
        candidates.pipelines[0].stages.clear();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::Empty {
                entry: id("pipe.layered_ckcir_to_smt"),
                field: "stages",
            }]
        );

        let (corpora, candidates, mut experiments, gold) = valid_set();
        experiments[0].fixture_groups.clear();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![
                RegistryFinding::Empty {
                    entry: id("exp.m1_spine"),
                    field: "fixture_groups",
                },
                RegistryFinding::GoldGroupUnknown {
                    experiment: id("exp.m1_spine"),
                    group_id: id("group.m1_conflict"),
                },
                RegistryFinding::GoldGroupUnknown {
                    experiment: id("exp.m1_spine"),
                    group_id: id("group.m1_null"),
                },
            ]
        );

        let (corpora, candidates, mut experiments, gold) = valid_set();
        experiments[0].fixture_groups[0].fixtures.clear();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::Empty {
                entry: id("group.m1_conflict"),
                field: "fixtures",
            }]
        );

        let (corpora, candidates, mut experiments, gold) = valid_set();
        experiments[0].expected_outcomes.clear();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::Empty {
                entry: id("exp.m1_spine"),
                field: "expected_outcomes",
            }]
        );
    }

    // Expected-outcome refs resolve against loaded gold documents whose
    // entries assert groups the experiment defines.
    #[test]
    fn gold_resolution_findings() {
        let (corpora, candidates, experiments, _) = valid_set();
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &BTreeMap::new()),
            vec![RegistryFinding::GoldUnresolved {
                experiment: id("exp.m1_spine"),
                path: GOLD_PATH.to_string(),
            }]
        );

        let (corpora, candidates, experiments, mut gold) = valid_set();
        gold.get_mut(GOLD_PATH).unwrap()[1].group_id = id("group.m1_extra");
        assert_eq!(
            validate_registries(&corpora, &candidates, &experiments, &gold),
            vec![RegistryFinding::GoldGroupUnknown {
                experiment: id("exp.m1_spine"),
                group_id: id("group.m1_extra"),
            }]
        );
    }
}
