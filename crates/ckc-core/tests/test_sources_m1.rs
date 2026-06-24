//! fixtures-m1 gate: the committed M1 data layer — corpus test_sources, lexicon,
//! reference, registry seeds — loads through the SPEC §8.4 registry types,
//! validates as one set with zero findings, test_source and reference paths resolve
//! from the repository root, and test_source bytes carry the normative §8.2
//! sentences the reference core ids and §8.6 worked thread are derived from.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use ckc_core::{
    Candidates, Certainty, CorpusEntry, Direction, ExperimentEntry, Id, ReferenceEntry, Strength,
    parse_candidates, parse_corpora, parse_experiments, parse_reference, validate_registries,
};
use serde::Deserialize;

/// SPEC §8.2 normative test_source sentences, byte-exact with the spec quotes.
const A_RECOMMENDATION: &str =
    "成人(18歳以上)の敗血症患者には抗菌薬Aを投与することを推奨する(強い推奨)";
const A_EXCEPTION: &str = "ただし、重度腎機能障害のある患者を除く";
const B_CONTRAINDICATION: &str =
    "成人の敗血症患者のうち、妊娠中の患者には抗菌薬Aを投与しないこと(禁忌)";
const CONTROL_SENTENCE: &str = "小児(18歳未満)の敗血症患者には抗菌薬Aは禁忌である";

/// Repository root: two levels above the ckc-core manifest, where the §3
/// `corpus/` and `registry/` trees live and registry paths are anchored.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/ckc-core sits two levels under the repo root")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}

/// The committed registry surface loaded through the typed parsers, reference
/// keyed exactly by the path the experiment writes.
fn load_set() -> (
    Vec<CorpusEntry>,
    Candidates,
    Vec<ExperimentEntry>,
    BTreeMap<String, Vec<ReferenceEntry>>,
) {
    let corpora = parse_corpora(&read("registry/corpora.yaml")).unwrap();
    let candidates = parse_candidates(&read("registry/candidates.yaml")).unwrap();
    let experiments = parse_experiments(&read("registry/experiments.yaml")).unwrap();
    let mut reference = BTreeMap::new();
    for experiment in &experiments {
        reference.insert(
            experiment.expected_outcomes.clone(),
            parse_reference(&read(&experiment.expected_outcomes)).unwrap(),
        );
    }
    (corpora, candidates, experiments, reference)
}

// The committed set cross-resolves with zero findings, and exp.m1_scaffold
// carries the §8.2 groups over the §8.3 processing_stage chain.
#[test]
fn registry_set_loads_and_validates() {
    let (corpora, candidates, experiments, reference) = load_set();
    assert_eq!(
        validate_registries(&corpora, &candidates, &experiments, &reference),
        vec![]
    );

    let corpus_ids: Vec<&Id> = corpora.iter().map(|c| &c.id).collect();
    assert_eq!(
        corpus_ids,
        vec![
            &id("test_source.m1_guideline_a"),
            &id("test_source.m1_guideline_b"),
            &id("test_source.m1_control"),
        ]
    );

    assert_eq!(experiments.len(), 1);
    let exp = &experiments[0];
    assert_eq!(exp.id, id("exp.m1_scaffold"));
    assert_eq!(exp.pipeline, id("pipe.layered_ckcir_to_smt"));
    assert_eq!(exp.test_source_groups.len(), 2);
    assert_eq!(exp.test_source_groups[0].group_id, id("group.m1_conflict"));
    assert_eq!(
        exp.test_source_groups[0].test_sources,
        vec![
            id("test_source.m1_guideline_a"),
            id("test_source.m1_guideline_b")
        ]
    );
    assert_eq!(
        exp.test_source_groups[1].group_id,
        id("group.m1_no_conflict")
    );
    assert_eq!(
        exp.test_source_groups[1].test_sources,
        vec![
            id("test_source.m1_guideline_a"),
            id("test_source.m1_control")
        ]
    );
    assert!(exp.budget.contains_key(&id("solver_ms_per_query")));

    // The pipeline chains the eight §8.3 processing_stages in execution order.
    let processing_stage_kind: BTreeMap<&Id, &Id> = candidates
        .processing_stages
        .iter()
        .map(|s| (&s.id, &s.kind))
        .collect();
    let kinds: Vec<&str> = candidates.pipelines[0]
        .processing_stages
        .iter()
        .map(|sid| processing_stage_kind[sid].as_str())
        .collect();
    assert_eq!(
        kinds,
        [
            "extract",
            "segment",
            "normalize",
            "assemble",
            "compile",
            "verify",
            "trace",
            "report",
        ]
    );
}

// Every corpus path resolves from the repo root and the resolved bytes carry
// the §8.2 sentences; guideline_a also exercises the layout surface (CQ
// heading, definitions table, evidence list).
#[test]
fn test_source_paths_resolve_and_carry_spec_sentences() {
    let (corpora, ..) = load_set();
    let required: BTreeMap<&str, Vec<&str>> = BTreeMap::from([
        (
            "test_source.m1_guideline_a",
            vec![A_RECOMMENDATION, A_EXCEPTION, "CQ1", "<table>", "<ul>"],
        ),
        ("test_source.m1_guideline_b", vec![B_CONTRAINDICATION]),
        ("test_source.m1_control", vec![CONTROL_SENTENCE]),
    ]);
    assert_eq!(corpora.len(), required.len());
    for entry in &corpora {
        let body = read(&entry.path);
        assert!(
            body.contains(r#"<meta charset="utf-8">"#) && body.contains(r#"<html lang="ja">"#),
            "{}: test_source HTML preamble missing",
            entry.id
        );
        for needle in &required[entry.id.as_str()] {
            assert!(body.contains(needle), "{}: missing {needle:?}", entry.id);
        }
    }
}

// The reference file matches the §8.2 expectations entry for entry: the conflict
// group expects the cross-document unsat core as a set, the control group a
// documented no-conflict result.
#[test]
fn reference_matches_spec_expectations() {
    let (_, _, experiments, reference) = load_set();
    let entries = &reference[&experiments[0].expected_outcomes];
    assert_eq!(entries.len(), 2);

    let conflict = &entries[0];
    assert_eq!(conflict.group_id, id("group.m1_conflict"));
    assert_eq!(conflict.expected_outcome, id("semantic_contradiction"));
    assert_eq!(
        conflict.expected_conflict_kind,
        Some(id("deontic_direction_conflict"))
    );
    assert_eq!(
        conflict.expected_unsat_core,
        BTreeSet::from([
            id("a.test_source.m1_guideline_a.rule.0"),
            id("a.test_source.m1_guideline_b.rule.0"),
        ])
    );
    assert!(!conflict.expected_no_conflict_result);

    let null = &entries[1];
    assert_eq!(null.group_id, id("group.m1_no_conflict"));
    assert_eq!(null.expected_outcome, id("semantic_no_conflict"));
    assert_eq!(null.expected_conflict_kind, None);
    assert!(null.expected_unsat_core.is_empty());
    assert!(null.expected_no_conflict_result);
}

/// Test-local mirror of the `corpus/lexicon/ja_core.yaml` shape requirements
/// (the typed loader lands with stage-normalize.1); strict so field typos in
/// the committed file fail here.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Lexicon {
    system: Id,
    concepts: Vec<ConceptEntry>,
    actions: Vec<ActionEntry>,
    modality: Vec<ModalityEntry>,
    certainty: Vec<CertaintyEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConceptEntry {
    id: Id,
    surface: Vec<String>,
    #[serde(default)]
    interval: Option<IntervalSemantics>,
}

/// §5 interval semantics over a quantity variable, bounds mirroring the
/// QuantityInterval i64 sides.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IntervalSemantics {
    var: Id,
    #[serde(default)]
    ge: Option<i64>,
    #[serde(default)]
    gt: Option<i64>,
    #[serde(default)]
    le: Option<i64>,
    #[serde(default)]
    lt: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionEntry {
    id: Id,
    surface: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModalityEntry {
    surface: String,
    direction: Direction,
    strength: Strength,
    #[serde(default)]
    implies_action: Option<Id>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CertaintyEntry {
    surface: String,
    value: Certainty,
}

// The lexicon parses through the shape requirements and carries exactly the §8.6
// concept/action vocabulary, the §5 modality and certainty tables, and the
// adult/child age-interval semantics; every concept surface form is bound by
// at least one test_source.
#[test]
fn lexicon_parses_and_yields_spec_ids() {
    let lexicon: Lexicon = serde_saphyr::from_str(&read("corpus/lexicon/ja_core.yaml")).unwrap();
    assert_eq!(lexicon.system, id("ckc.lex"));

    let concept_ids: BTreeSet<&str> = lexicon.concepts.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        concept_ids,
        BTreeSet::from([
            "cond.sepsis",
            "cond.renal_severe",
            "cond.pregnancy",
            "drug.abx_a",
            "pop.adult",
            "pop.child",
        ])
    );

    let interval = |cid: &str| {
        lexicon
            .concepts
            .iter()
            .find(|c| c.id.as_str() == cid)
            .and_then(|c| c.interval.as_ref())
            .unwrap_or_else(|| panic!("{cid}: interval semantics expected"))
    };
    let adult = interval("pop.adult");
    assert_eq!(adult.var, id("q.age_years"));
    assert_eq!(
        (adult.ge, adult.gt, adult.le, adult.lt),
        (Some(18), None, None, None)
    );
    let child = interval("pop.child");
    assert_eq!(child.var, id("q.age_years"));
    assert_eq!(
        (child.ge, child.gt, child.le, child.lt),
        (None, None, None, Some(18))
    );
    assert!(
        lexicon
            .concepts
            .iter()
            .filter(|c| !c.id.as_str().starts_with("pop."))
            .all(|c| c.interval.is_none()),
        "only the population concepts carry interval semantics at M1"
    );

    assert_eq!(lexicon.actions.len(), 1);
    assert_eq!(lexicon.actions[0].id, id("act.administer"));
    assert!(
        ["投与する", "投与"]
            .iter()
            .all(|s| lexicon.actions[0].surface.iter().any(|f| f == s))
    );

    // §5 modality table: phrase → (direction, strength); 禁忌 implies the
    // administer action for verbless drug contraindications (§8.2 control).
    let modality: BTreeMap<&str, (Direction, Strength, Option<&Id>)> = lexicon
        .modality
        .iter()
        .map(|m| {
            (
                m.surface.as_str(),
                (m.direction, m.strength, m.implies_action.as_ref()),
            )
        })
        .collect();
    let administer = id("act.administer");
    let expected: BTreeMap<&str, (Direction, Strength, Option<&Id>)> = BTreeMap::from([
        ("推奨する", (Direction::For, Strength::Strong, None)),
        ("提案する", (Direction::For, Strength::Weak, None)),
        ("考慮してもよい", (Direction::Permit, Strength::Weak, None)),
        ("推奨しない", (Direction::Against, Strength::Strong, None)),
        ("提案しない", (Direction::Against, Strength::Weak, None)),
        (
            "投与しないこと",
            (Direction::Contraindicate, Strength::Strong, None),
        ),
        (
            "禁忌",
            (
                Direction::Contraindicate,
                Strength::Strong,
                Some(&administer),
            ),
        ),
    ]);
    assert_eq!(modality, expected);

    let certainty: BTreeMap<&str, Certainty> = lexicon
        .certainty
        .iter()
        .map(|c| (c.surface.as_str(), c.value))
        .collect();
    assert_eq!(
        certainty,
        BTreeMap::from([
            ("エビデンスの確実性:高", Certainty::High),
            ("エビデンスの確実性:中", Certainty::Moderate),
            ("エビデンスの確実性:低", Certainty::Low),
            ("エビデンスの確実性:非常に低", Certainty::VeryLow),
        ])
    );

    // SourceLinkage coverage: every concept surface form occurs in test_source text,
    // so each lexicon concept is bindable from the committed corpus.
    let (corpora, ..) = load_set();
    let all_test_source_text: String = corpora.iter().map(|c| read(&c.path)).collect();
    for concept in &lexicon.concepts {
        for surface in &concept.surface {
            assert!(
                all_test_source_text.contains(surface.as_str()),
                "{}: surface {surface:?} unbound by any test_source",
                concept.id
            );
        }
    }
}
