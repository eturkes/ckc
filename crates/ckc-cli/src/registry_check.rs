//! SPEC §3 `ckc registry check` (cli-runner.1.2; closes §8.5 item 2).
//!
//! Loads the §8.4 registry surface from the invocation root —
//! `registry/{corpora,candidates,experiments}.yaml` plus every reference document
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
    DiagnosticCode, DiagnosticRecord, Id, Outcome, ReferenceEntry, RegistryError, parse_candidates,
    parse_corpora, parse_experiments, parse_reference, validate_registries,
};

use crate::shell::{Shell, static_id};

const CORPORA_FILE: &str = "registry/corpora.yaml";
const CANDIDATES_FILE: &str = "registry/candidates.yaml";
const EXPERIMENTS_FILE: &str = "registry/experiments.yaml";

/// Run `registry check` rooted at `root` (the invocation working directory:
/// §3 anchors `registry/` and reference paths at the repository root). Evidence
/// and the outcome land entirely in the shell.
pub(crate) fn check(root: &Path, shell: &mut Shell) {
    let corpora = load(root, CORPORA_FILE, parse_corpora, shell);
    let candidates = load(root, CANDIDATES_FILE, parse_candidates, shell);
    let experiments = load(root, EXPERIMENTS_FILE, parse_experiments, shell);
    let (Some(corpora), Some(candidates), Some(experiments)) = (corpora, candidates, experiments)
    else {
        return;
    };

    // Reference documents load per unique experiment ref, keyed exactly by the
    // path text the experiment writes (the resolution key validation uses).
    // Empty refs are an Empty finding below, not a read; a ref whose file
    // fails to load keeps its load diagnostic and surfaces again as
    // ReferenceUnresolved — one cause, both layers reported.
    let mut reference: BTreeMap<String, Vec<ReferenceEntry>> = BTreeMap::new();
    for experiment in &experiments {
        let rel = &experiment.expected_outcomes;
        if rel.is_empty() || reference.contains_key(rel) {
            continue;
        }
        if let Some(entries) = load(root, rel, parse_reference, shell) {
            reference.insert(rel.clone(), entries);
        }
    }

    for finding in validate_registries(&corpora, &candidates, &experiments, &reference) {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("finding"),
            finding.to_string(),
        )]));
    }
}

/// Read and strictly parse one registry document; a failure lands as one
/// `schema_invalid` diagnostic naming the file and returns `None`. Shared
/// with the run command's resolution step (`crate::run`).
pub(crate) fn load<T>(
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

/// `schema_invalid`/`invalid` diagnostic from sorted-key payload rows; the
/// shared §4.4 "validation fails" shape (also `crate::run` resolution).
pub(crate) fn invalid_diagnostic(payload: Vec<(Id, String)>) -> DiagnosticRecord {
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
    processing_stages: [processing_stage.head]
processing_stages:
  - id: processing_stage.head
    kind: extract
    determinism: deterministic
    input_artifact_kinds: []
    output_artifact_kinds: [source_document_graph]
",
        );
        write(
            root,
            EXPERIMENTS_FILE,
            "\
- id: exp.e
  pipeline: pipe.p
  test_source_groups:
    - group_id: group.g
      test_sources: [test_source.x]
  seed: 1
  budget:
    solver_ms_per_query: 1000
  expected_outcomes: corpus/reference/g.yaml
",
        );
        let (result, diagnostics) = checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_reported(&diagnostics, CORPORA_FILE);
    }

    // A loadable but semantically broken set is reported whole: every
    // finding and the reference load failure land as their own diagnostics, no
    // fail-fast.
    #[test]
    fn broken_set_reports_every_finding() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            CORPORA_FILE,
            "\
- id: test_source.x
  path: corpus/test_sources/x.html
  origin: ai_generated
  evidence_status: source_evidence_status
  provenance: synthetic
",
        );
        // processing_stage.consume's input has no producing predecessor: ChainBreak.
        write(
            root,
            CANDIDATES_FILE,
            "\
pipelines:
  - id: pipe.broken
    processing_stages: [processing_stage.consume]
processing_stages:
  - id: processing_stage.consume
    kind: segment
    determinism: deterministic
    input_artifact_kinds: [source_document_graph]
    output_artifact_kinds: [segments]
",
        );
        // exp.broken: dangling pipeline, dangling test_source, reference ref whose
        // file exists but fails to parse; exp.holey: empty reference ref.
        write(
            root,
            EXPERIMENTS_FILE,
            "\
- id: exp.broken
  pipeline: pipe.missing
  test_source_groups:
    - group_id: group.g1
      test_sources: [test_source.x, test_source.missing]
  seed: 1
  budget:
    solver_ms_per_query: 1000
  expected_outcomes: corpus/reference/broken.yaml
- id: exp.holey
  pipeline: pipe.broken
  test_source_groups:
    - group_id: group.g2
      test_sources: [test_source.x]
  seed: 1
  budget:
    solver_ms_per_query: 1000
  expected_outcomes: \"\"
",
        );
        write(root, "corpus/reference/broken.yaml", "][ not yaml");

        let (result, diagnostics) = checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        // reference load failure + ChainBreak + 2 danglings + ReferenceUnresolved +
        // Empty expected_outcomes.
        assert_eq!(diagnostics.len(), 6);
        assert_reported(&diagnostics, "corpus/reference/broken.yaml");
        assert_reported(&diagnostics, "no predecessor produces");
        assert_reported(&diagnostics, "undefined pipelines pipe.missing");
        assert_reported(&diagnostics, "undefined corpora test_source.missing");
        assert_reported(&diagnostics, "matches no loaded reference document");
        assert_reported(&diagnostics, "expected_outcomes is empty");
    }
}
