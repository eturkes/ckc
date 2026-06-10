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
//! and enum fields admit exactly their canonical spellings. Cross-file
//! resolution and the §8.4 stage-chain rule land with registry validation
//! (core-registry.2).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::canon::{
    CanonError, CanonRead, CanonReadError, Canonical, Reader, emit_string, read_string,
};
use crate::enums::{Authority, Origin, fieldless_enum};
use crate::grounding::Provenance;
use crate::id::{Id, ValidationError};

fieldless_enum! {
    /// SPEC §8.4 stage-component determinism class. Every V1 component is
    /// `deterministic`; `nondeterministic` marks components whose reruns may
    /// diverge (V3's recorded weak-model routes), which replay handles
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
    /// `fixture.v1_guideline_a`).
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
    /// Experiment id (e.g. `exp.v1_spine`).
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
    /// Group id (e.g. `group.v1_conflict`).
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
- id: fixture.v1_guideline_a
  path: corpus/fixtures/v1_guideline_a.html
  origin: ai_generated
  authority: source_authority
  provenance: synthetic
- id: fixture.v1_guideline_b
  path: corpus/fixtures/v1_guideline_b.html
  origin: ai_generated
  authority: source_authority
  provenance: synthetic
- id: fixture.v1_control
  path: corpus/fixtures/v1_control.html
  origin: ai_generated
  authority: source_authority
  provenance: synthetic
";

    const CANDIDATES: &str = "\
pipelines:
  - id: pipe.layered_ckcir_to_smt
    stages: [stage.v1.extract, stage.v1.segment]
stages:
  - id: stage.v1.extract
    kind: extract
    determinism: deterministic
    input_artifact_kinds: []
    output_artifact_kinds: [source_graph]
  - id: stage.v1.segment
    kind: segment
    determinism: deterministic
    input_artifact_kinds: [source_graph]
    output_artifact_kinds: [segments]
";

    const EXPERIMENTS: &str = "\
- id: exp.v1_spine
  pipeline: pipe.layered_ckcir_to_smt
  fixture_groups:
    - group_id: group.v1_conflict
      fixtures: [fixture.v1_guideline_a, fixture.v1_guideline_b]
    - group_id: group.v1_null
      fixtures: [fixture.v1_guideline_a, fixture.v1_control]
  seed: 42
  budget:
    solver_ms_per_query: 10000
  expected_outcomes: corpus/gold/v1_expected.yaml
";

    // Verbatim §8.2 gold block, comment included.
    const GOLD: &str = "\
- group_id: group.v1_conflict
  expected_outcome: semantic_contradiction
  expected_conflict_kind: deontic_direction_conflict
  expected_core: [a.rule.a.cq1.r1, a.rule.b.contra1]   # compared as a set
- group_id: group.v1_null
  expected_outcome: semantic_no_conflict
  expected_null_result: true
";

    // §8.2 corpora admission fields load typed: ai_generated origin under
    // source_authority, synthetic provenance.
    #[test]
    fn corpora_load_typed() {
        let corpora = parse_corpora(CORPORA).unwrap();
        assert_eq!(corpora.len(), 3);
        assert_eq!(corpora[0].id, id("fixture.v1_guideline_a"));
        assert_eq!(corpora[0].path, "corpus/fixtures/v1_guideline_a.html");
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
            vec![id("stage.v1.extract"), id("stage.v1.segment")]
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
        assert_eq!(exp.id, id("exp.v1_spine"));
        assert_eq!(exp.pipeline, id("pipe.layered_ckcir_to_smt"));
        assert_eq!(exp.fixture_groups.len(), 2);
        assert_eq!(exp.fixture_groups[0].group_id, id("group.v1_conflict"));
        assert_eq!(
            exp.fixture_groups[1].fixtures,
            vec![id("fixture.v1_guideline_a"), id("fixture.v1_control")]
        );
        assert_eq!(exp.seed, 42);
        assert_eq!(exp.budget[&id("solver_ms_per_query")], 10_000);
        assert_eq!(exp.expected_outcomes, "corpus/gold/v1_expected.yaml");
    }

    // The verbatim §8.2 gold block loads typed; absent optionals take their
    // defaults, and expected_core is order-insensitive (set comparison).
    #[test]
    fn gold_loads_spec_shape() {
        let gold = parse_gold(GOLD).unwrap();
        assert_eq!(gold.len(), 2);
        let conflict = &gold[0];
        assert_eq!(conflict.group_id, id("group.v1_conflict"));
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
            "[a.rule.a.cq1.r1, a.rule.b.contra1]",
            "[a.rule.b.contra1, a.rule.a.cq1.r1]",
        );
        assert_eq!(parse_gold(&reordered).unwrap()[0], *conflict);
        assert_eq!(
            conflict.expected_core,
            BTreeSet::from([id("a.rule.a.cq1.r1"), id("a.rule.b.contra1")])
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
        let bad_id = GOLD.replace("group.v1_conflict", "Group.V1_Conflict");
        assert!(parse_gold(&bad_id).is_err());
        let bad_enum = CORPORA.replace("origin: ai_generated", "origin: vibes");
        assert!(parse_corpora(&bad_enum).is_err());
        let missing_field = EXPERIMENTS.replace("  seed: 42\n", "");
        assert!(parse_experiments(&missing_field).is_err());
    }
}
