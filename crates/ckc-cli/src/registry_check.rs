//! SPEC §3 `ckc registry check` (cli-runner.1.2; closes §8.5 item 2).
//!
//! Loads the §8.4 registry surface from the invocation root —
//! `registry/{corpora,candidates,experiments}.yaml` plus every gold document
//! an experiment's expected-outcome ref names — through the strict
//! core-registry loaders, then runs [`validate_registries`] over the set as
//! one cross-referenced whole (resolution, uniqueness, the stage-chain
//! rule). Every load failure and every [`RegistryFinding`] lands as a §7.4
//! `schema_invalid` diagnostic carrying outcome `invalid` (§4.4 "registry
//! ... validation fails"), severity-folded by the shell; a set that loads
//! and validates clean leaves the total result `ok`. Semantic validation
//! needs all three registry documents, so any core-file load failure skips
//! it — the load diagnostics already decide the outcome.

use std::collections::BTreeMap;
use std::path::Path;

use ckc_core::{
    DiagnosticCode, DiagnosticRecord, GoldEntry, Id, Outcome, RegistryError, parse_candidates,
    parse_corpora, parse_experiments, parse_gold, validate_registries,
};

use crate::shell::{Shell, static_id};

const CORPORA_FILE: &str = "registry/corpora.yaml";
const CANDIDATES_FILE: &str = "registry/candidates.yaml";
const EXPERIMENTS_FILE: &str = "registry/experiments.yaml";

/// Run `registry check` rooted at `root` (the invocation working directory:
/// §3 anchors `registry/` and gold paths at the repository root). Evidence
/// and the outcome land entirely in the shell.
pub(crate) fn check(root: &Path, shell: &mut Shell) {
    let corpora = load(root, CORPORA_FILE, parse_corpora, shell);
    let candidates = load(root, CANDIDATES_FILE, parse_candidates, shell);
    let experiments = load(root, EXPERIMENTS_FILE, parse_experiments, shell);
    let (Some(corpora), Some(candidates), Some(experiments)) = (corpora, candidates, experiments)
    else {
        return;
    };

    // Gold documents load per unique experiment ref, keyed exactly by the
    // path text the experiment writes (the resolution key validation uses).
    // Empty refs are an Empty finding below, not a read; a ref whose file
    // fails to load keeps its load diagnostic and surfaces again as
    // GoldUnresolved — one cause, both layers reported.
    let mut gold: BTreeMap<String, Vec<GoldEntry>> = BTreeMap::new();
    for experiment in &experiments {
        let rel = &experiment.expected_outcomes;
        if rel.is_empty() || gold.contains_key(rel) {
            continue;
        }
        if let Some(entries) = load(root, rel, parse_gold, shell) {
            gold.insert(rel.clone(), entries);
        }
    }

    for finding in validate_registries(&corpora, &candidates, &experiments, &gold) {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("finding"),
            finding.to_string(),
        )]));
    }
}

/// Read and strictly parse one registry document; a failure lands as one
/// `schema_invalid` diagnostic naming the file and returns `None`.
fn load<T>(
    root: &Path,
    rel: &str,
    parse: fn(&str) -> Result<T, RegistryError>,
    shell: &mut Shell,
) -> Option<T> {
    let path = root.join(rel);
    let outcome = match std::fs::read_to_string(&path) {
        Ok(text) => parse(&text).map_err(|e| e.to_string()),
        Err(e) => Err(format!("read {}: {e}", path.display())),
    };
    match outcome {
        Ok(value) => Some(value),
        Err(reason) => {
            shell.diagnostic(invalid_diagnostic(vec![
                (static_id("file"), rel.to_owned()),
                (static_id("reason"), reason),
            ]));
            None
        }
    }
}

fn invalid_diagnostic(payload: Vec<(Id, String)>) -> DiagnosticRecord {
    DiagnosticRecord {
        code: DiagnosticCode::SchemaInvalid,
        outcome: Outcome::Invalid,
        payload,
        region_ids: Vec::new(),
        artifact_hashes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use ckc_core::{EventRecord, TotalOperationResult, read_jsonl};

    use crate::shell::run_none;

    /// Repository root: two levels above the ckc-cli manifest, where the §3
    /// `registry/` and `corpus/` trees live.
    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("crates/ckc-cli sits two levels under the repo root")
            .to_path_buf()
    }

    /// Run the checker against `root` in a fresh shell and return the §4.4
    /// result with the canonically sorted diagnostics off the event stream.
    fn checked(root: &Path) -> (TotalOperationResult, Vec<DiagnosticRecord>) {
        let mut shell = Shell::open(static_id("registry.check"), run_none(), None);
        check(root, &mut shell);
        let finished = shell.finish().unwrap();
        let events: Vec<EventRecord> =
            read_jsonl(finished.streamed_events.as_deref().unwrap()).unwrap();
        (finished.result, events[0].diagnostics.clone())
    }

    fn write(root: &Path, rel: &str, text: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    /// Every diagnostic is schema_invalid/invalid; some payload value
    /// contains the needle.
    fn assert_reported(diagnostics: &[DiagnosticRecord], needle: &str) {
        for diagnostic in diagnostics {
            assert_eq!(diagnostic.code, DiagnosticCode::SchemaInvalid);
            assert_eq!(diagnostic.outcome, Outcome::Invalid);
        }
        assert!(
            diagnostics
                .iter()
                .flat_map(|d| &d.payload)
                .any(|(_, value)| value.contains(needle)),
            "no diagnostic payload contains {needle:?}: {diagnostics:?}"
        );
    }

    // §8.5 item 2 in-process: the committed registry set loads and
    // validates clean from the repository root.
    #[test]
    fn committed_set_checks_ok() {
        let (result, diagnostics) = checked(&repo_root());
        assert_eq!(result.outcome, Outcome::Ok);
        assert!(result.diagnostic_hashes.is_empty());
        assert!(diagnostics.is_empty());
    }

    // A root without registry files: one read diagnostic per core file,
    // total invalid.
    #[test]
    fn missing_files_each_get_a_diagnostic() {
        let tmp = tempfile::tempdir().unwrap();
        let (result, diagnostics) = checked(tmp.path());
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(result.diagnostic_hashes.len(), 3);
        assert_eq!(diagnostics.len(), 3);
        for file in [CORPORA_FILE, CANDIDATES_FILE, EXPERIMENTS_FILE] {
            assert_reported(&diagnostics, file);
        }
    }

    // A core file that fails to parse blocks semantic validation: the load
    // diagnostic stands alone even though the other files reference ids the
    // broken file would have to define.
    #[test]
    fn malformed_core_file_skips_validation() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, CORPORA_FILE, "][ not yaml");
        write(
            root,
            CANDIDATES_FILE,
            "\
pipelines:
  - id: pipe.p
    stages: [stage.head]
stages:
  - id: stage.head
    kind: extract
    determinism: deterministic
    input_artifact_kinds: []
    output_artifact_kinds: [source_graph]
",
        );
        write(
            root,
            EXPERIMENTS_FILE,
            "\
- id: exp.e
  pipeline: pipe.p
  fixture_groups:
    - group_id: group.g
      fixtures: [fixture.x]
  seed: 1
  budget:
    solver_ms_per_query: 1000
  expected_outcomes: corpus/gold/g.yaml
",
        );
        let (result, diagnostics) = checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_reported(&diagnostics, CORPORA_FILE);
    }

    // A loadable but semantically broken set is reported whole: every
    // finding and the gold load failure land as their own diagnostics, no
    // fail-fast.
    #[test]
    fn broken_set_reports_every_finding() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            CORPORA_FILE,
            "\
- id: fixture.x
  path: corpus/fixtures/x.html
  origin: ai_generated
  authority: source_authority
  provenance: synthetic
",
        );
        // stage.consume's input has no producing predecessor: ChainBreak.
        write(
            root,
            CANDIDATES_FILE,
            "\
pipelines:
  - id: pipe.broken
    stages: [stage.consume]
stages:
  - id: stage.consume
    kind: segment
    determinism: deterministic
    input_artifact_kinds: [source_graph]
    output_artifact_kinds: [segments]
",
        );
        // exp.broken: dangling pipeline, dangling fixture, gold ref whose
        // file exists but fails to parse; exp.holey: empty gold ref.
        write(
            root,
            EXPERIMENTS_FILE,
            "\
- id: exp.broken
  pipeline: pipe.missing
  fixture_groups:
    - group_id: group.g1
      fixtures: [fixture.x, fixture.missing]
  seed: 1
  budget:
    solver_ms_per_query: 1000
  expected_outcomes: corpus/gold/broken.yaml
- id: exp.holey
  pipeline: pipe.broken
  fixture_groups:
    - group_id: group.g2
      fixtures: [fixture.x]
  seed: 1
  budget:
    solver_ms_per_query: 1000
  expected_outcomes: \"\"
",
        );
        write(root, "corpus/gold/broken.yaml", "][ not yaml");

        let (result, diagnostics) = checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        // gold load failure + ChainBreak + 2 danglings + GoldUnresolved +
        // Empty expected_outcomes.
        assert_eq!(diagnostics.len(), 6);
        assert_reported(&diagnostics, "corpus/gold/broken.yaml");
        assert_reported(&diagnostics, "no predecessor produces");
        assert_reported(&diagnostics, "undefined pipelines pipe.missing");
        assert_reported(&diagnostics, "undefined corpora fixture.missing");
        assert_reported(&diagnostics, "matches no loaded gold document");
        assert_reported(&diagnostics, "expected_outcomes is empty");
    }
}
