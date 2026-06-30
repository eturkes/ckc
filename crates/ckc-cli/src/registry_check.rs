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
    DiagnosticCode, DiagnosticRecord, Id, Outcome, ReferenceEntry, RegistryError, hash_bytes,
    is_safe_relative_path, parse_candidates, parse_corpora, parse_experiments, parse_prompts,
    parse_reference, parse_schemas, validate_model_registry, validate_registries,
};

use crate::shell::{Shell, static_id};

const CORPORA_FILE: &str = "registry/corpora.yaml";
const CANDIDATES_FILE: &str = "registry/candidates.yaml";
const EXPERIMENTS_FILE: &str = "registry/experiments.yaml";
const SCHEMAS_FILE: &str = "registry/schemas.yaml";
const PROMPTS_FILE: &str = "registry/prompts.yaml";

/// Run `registry check` rooted at `root` (the invocation working directory:
/// §3 anchors `registry/` and reference paths at the repository root). Evidence
/// and the outcome land entirely in the shell.
pub(crate) fn check(root: &Path, shell: &mut Shell) {
    check_core_registry(root, shell);
    check_model_registry(root, shell);
}

/// The §8.4 core surface: corpora + candidates + experiments + reference,
/// validated as one cross-referenced whole. A core-file load failure skips
/// semantic validation — the load diagnostics already decide the outcome.
fn check_core_registry(root: &Path, shell: &mut Shell) {
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

/// The §14 model surface (M2): `registry/schemas.yaml` + `registry/prompts.yaml`.
/// Both are optional additions — absent until a route configures them, so a
/// missing file is empty, not a diagnostic. Past [`validate_model_registry`]
/// (id uniqueness, schema paths, prompt source shape), each schema file and each
/// prompt body — the `path` file's bytes or the `inline` text — is hashed and
/// compared to the entry's pinned hash (`schema_hash` / `template_hash`): a
/// missing file or a mismatch lands one diagnostic. Independent of the §8.4 set.
fn check_model_registry(root: &Path, shell: &mut Shell) {
    let schemas = load_optional(root, SCHEMAS_FILE, parse_schemas, shell);
    let prompts = load_optional(root, PROMPTS_FILE, parse_prompts, shell);

    for finding in validate_model_registry(&schemas, &prompts) {
        shell.diagnostic(invalid_diagnostic(vec![(
            static_id("finding"),
            finding.to_string(),
        )]));
    }

    for schema in &schemas {
        if schema.path.is_empty() || !is_safe_relative_path(&schema.path) {
            continue; // Empty / unsafe path is a finding above; never read it.
        }
        let path = root.join(&schema.path);
        match std::fs::read(&path) {
            Ok(bytes) => {
                let actual = hash_bytes(&bytes);
                if actual != schema.schema_hash {
                    shell.diagnostic(invalid_diagnostic(vec![
                        (static_id("actual"), actual.to_string()),
                        (static_id("expected"), schema.schema_hash.to_string()),
                        (static_id("schema"), schema.id.to_string()),
                    ]));
                }
            }
            Err(e) => {
                shell.diagnostic(invalid_diagnostic(vec![
                    (static_id("reason"), format!("read {}: {e}", path.display())),
                    (static_id("schema"), schema.id.to_string()),
                ]));
            }
        }
    }

    for prompt in &prompts {
        let actual = match (&prompt.path, &prompt.inline) {
            (Some(path), None) if !path.is_empty() && is_safe_relative_path(path) => {
                let full = root.join(path);
                match std::fs::read(&full) {
                    Ok(bytes) => hash_bytes(&bytes),
                    Err(e) => {
                        shell.diagnostic(invalid_diagnostic(vec![
                            (static_id("prompt"), prompt.id.to_string()),
                            (static_id("reason"), format!("read {}: {e}", full.display())),
                        ]));
                        continue;
                    }
                }
            }
            (None, Some(inline)) if !inline.is_empty() => hash_bytes(inline.as_bytes()),
            // Empty / unsafe / both-set / neither-set: a finding above covers it.
            _ => continue,
        };
        if actual != prompt.template_hash {
            shell.diagnostic(invalid_diagnostic(vec![
                (static_id("actual"), actual.to_string()),
                (static_id("expected"), prompt.template_hash.to_string()),
                (static_id("prompt"), prompt.id.to_string()),
            ]));
        }
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

/// Like [`load`] but for an optional registry document: a file that does not
/// exist yields the default (empty) with no diagnostic — the §14 model files
/// are additive, absent until a route configures them. A file that exists but
/// fails to read or parse still lands one diagnostic.
fn load_optional<T: Default>(
    root: &Path,
    rel: &str,
    parse: fn(&str) -> Result<T, RegistryError>,
    shell: &mut Shell,
) -> T {
    let path = root.join(rel);
    match std::fs::read_to_string(&path) {
        Ok(text) => match parse(&text) {
            Ok(value) => value,
            Err(e) => {
                shell.diagnostic(invalid_diagnostic(vec![
                    (static_id("file"), rel.to_owned()),
                    (static_id("reason"), e.to_string()),
                ]));
                T::default()
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => T::default(),
        Err(e) => {
            shell.diagnostic(invalid_diagnostic(vec![
                (static_id("file"), rel.to_owned()),
                (static_id("reason"), format!("read {}: {e}", path.display())),
            ]));
            T::default()
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

    /// Run only the §14 model surface against `root`; same shell harness as
    /// [`checked`].
    fn model_checked(root: &Path) -> (TotalOperationResult, Vec<DiagnosticRecord>) {
        let mut shell = Shell::open(static_id("registry.check"), run_none(), None);
        check_model_registry(root, &mut shell);
        let finished = shell.finish().unwrap();
        let events: Vec<EventRecord> =
            read_jsonl(finished.streamed_events.as_deref().unwrap()).unwrap();
        (finished.result, events[0].diagnostics.clone())
    }

    // The committed `registry/schemas.yaml` loads and its files hash-match:
    // the model surface is clean at the repository root.
    #[test]
    fn committed_model_surface_checks_ok() {
        let (result, diagnostics) = model_checked(&repo_root());
        assert_eq!(result.outcome, Outcome::Ok);
        assert!(diagnostics.is_empty());
    }

    // The §14 files are optional: a root without them is clean, not an error.
    #[test]
    fn absent_model_files_are_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let (result, diagnostics) = model_checked(tmp.path());
        assert_eq!(result.outcome, Outcome::Ok);
        assert!(diagnostics.is_empty());
    }

    // A present-but-malformed schemas file lands one load diagnostic.
    #[test]
    fn malformed_schemas_file_reports() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, SCHEMAS_FILE, "][ not yaml");
        let (result, diagnostics) = model_checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_reported(&diagnostics, SCHEMAS_FILE);
    }

    // The gate: a schema whose file is missing and one whose bytes drift from
    // the pinned hash are each rejected; a matching entry stays silent.
    #[test]
    fn schema_file_missing_or_mismatched_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let good = hash_bytes(b"good schema bytes\n");
        write(
            root,
            SCHEMAS_FILE,
            &format!(
                "\
- id: schema.good
  path: schemas/good.json
  schema_hash: {good}
  target_kind: clinical_ir
- id: schema.missing
  path: schemas/missing.json
  schema_hash: {good}
  target_kind: clinical_ir
- id: schema.mismatch
  path: schemas/mismatch.json
  schema_hash: {good}
  target_kind: clinical_ir
"
            ),
        );
        write(root, "schemas/good.json", "good schema bytes\n");
        write(root, "schemas/mismatch.json", "different bytes\n");
        // schemas/missing.json is deliberately absent.

        let (result, diagnostics) = model_checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 2);
        assert_reported(&diagnostics, "schema.missing");
        assert_reported(&diagnostics, "schema.mismatch");
        assert!(
            !diagnostics
                .iter()
                .flat_map(|d| &d.payload)
                .any(|(_, value)| value.contains("schema.good")),
            "the matching entry should be silent: {diagnostics:?}"
        );
    }

    // Model-surface findings surface too: a duplicate schema id.
    #[test]
    fn duplicate_schema_id_reported() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let good = hash_bytes(b"x\n");
        write(
            root,
            SCHEMAS_FILE,
            &format!(
                "\
- id: schema.dup
  path: schemas/a.json
  schema_hash: {good}
  target_kind: clinical_ir
- id: schema.dup
  path: schemas/a.json
  schema_hash: {good}
  target_kind: clinical_ir
"
            ),
        );
        write(root, "schemas/a.json", "x\n");
        let (result, diagnostics) = model_checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_reported(&diagnostics, "duplicate schemas id schema.dup");
    }

    // An unsafe schema path (absolute or `..`-escaping) is reported as a
    // finding and never read: exactly one diagnostic, so no read of the
    // out-of-tree target was attempted (that would add a second).
    #[test]
    fn unsafe_schema_path_reported_not_read() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let good = hash_bytes(b"x\n");
        write(
            root,
            SCHEMAS_FILE,
            &format!(
                "\
- id: schema.escape
  path: ../escape.json
  schema_hash: {good}
  target_kind: clinical_ir
"
            ),
        );
        let (result, diagnostics) = model_checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_reported(&diagnostics, "schema.escape");
        assert_reported(&diagnostics, "not a safe repo-relative path");
    }

    // The optional prompts file is validated too: a prompt naming both a path
    // and inline body surfaces its PromptSource finding through the CLI.
    #[test]
    fn prompt_source_finding_reported() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            PROMPTS_FILE,
            "\
- id: prompt.both
  path: registry/prompts/x.txt
  inline: \"inline body\"
  template_hash: sha256:0000000000000000000000000000000000000000000000000000000000000000
  route: route.single_ir
",
        );
        let (result, diagnostics) = model_checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        // Both-set yields only the source-shape finding — the loop's `_` arm
        // never reads the (nonexistent) path, which would add a second.
        assert_eq!(
            diagnostics.len(),
            1,
            "both-set must not be read: {diagnostics:?}"
        );
        assert_reported(
            &diagnostics,
            "prompt prompt.both: set exactly one of path or inline",
        );
    }

    // The hash-mismatch diagnostic carries exactly the sorted-key payload
    // [actual, expected, schema] — the row order the shell folds on.
    #[test]
    fn mismatch_diagnostic_payload_is_sorted() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let pinned = hash_bytes(b"pinned\n");
        write(
            root,
            SCHEMAS_FILE,
            &format!(
                "\
- id: schema.x
  path: schemas/x.json
  schema_hash: {pinned}
  target_kind: clinical_ir
"
            ),
        );
        write(root, "schemas/x.json", "actual\n");
        let (_, diagnostics) = model_checked(root);
        let actual = hash_bytes(b"actual\n");
        assert_eq!(
            diagnostics
                .iter()
                .map(|d| d.payload.clone())
                .collect::<Vec<_>>(),
            vec![vec![
                (static_id("actual"), actual.to_string()),
                (static_id("expected"), pinned.to_string()),
                (static_id("schema"), "schema.x".to_string()),
            ]]
        );
    }

    // The prompt gate mirrors the schema gate: a path prompt whose file is
    // missing and one whose bytes drift from `template_hash` are each rejected;
    // a matching path prompt and a matching inline prompt stay silent.
    #[test]
    fn prompt_file_missing_or_mismatched_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let good = hash_bytes(b"good prompt bytes\n");
        let inline_hash = hash_bytes(b"inline body");
        write(
            root,
            PROMPTS_FILE,
            &format!(
                "\
- id: prompt.good
  path: registry/prompts/good.txt
  template_hash: {good}
  route: route.single_ir
- id: prompt.missing
  path: registry/prompts/missing.txt
  template_hash: {good}
  route: route.single_ir
- id: prompt.mismatch
  path: registry/prompts/mismatch.txt
  template_hash: {good}
  route: route.single_ir
- id: prompt.inline
  inline: \"inline body\"
  template_hash: {inline_hash}
  route: route.single_ir
"
            ),
        );
        write(root, "registry/prompts/good.txt", "good prompt bytes\n");
        write(root, "registry/prompts/mismatch.txt", "different bytes\n");
        // registry/prompts/missing.txt is deliberately absent.

        let (result, diagnostics) = model_checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 2);
        assert_reported(&diagnostics, "prompt.missing");
        assert_reported(&diagnostics, "prompt.mismatch");
        for silent in ["prompt.good", "prompt.inline"] {
            assert!(
                !diagnostics
                    .iter()
                    .flat_map(|d| &d.payload)
                    .any(|(_, value)| value.contains(silent)),
                "the matching entry {silent} should be silent: {diagnostics:?}"
            );
        }
    }

    // The prompt hash-mismatch diagnostic carries exactly the sorted-key payload
    // [actual, expected, prompt]; an inline body exercises the inline hash branch.
    #[test]
    fn prompt_mismatch_diagnostic_payload_is_sorted() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let pinned = hash_bytes(b"pinned template");
        write(
            root,
            PROMPTS_FILE,
            &format!(
                "\
- id: prompt.x
  inline: \"actual template\"
  template_hash: {pinned}
  route: route.single_ir
"
            ),
        );
        let (_, diagnostics) = model_checked(root);
        let actual = hash_bytes(b"actual template");
        assert_eq!(
            diagnostics
                .iter()
                .map(|d| d.payload.clone())
                .collect::<Vec<_>>(),
            vec![vec![
                (static_id("actual"), actual.to_string()),
                (static_id("expected"), pinned.to_string()),
                (static_id("prompt"), "prompt.x".to_string()),
            ]]
        );
    }

    // The prompt loop mirrors the schema guard `unsafe_schema_path_reported_not_read`:
    // an unsafe prompt path (absolute or `..`-escaping) is reported as a finding and
    // never read — exactly one diagnostic, so no read of the out-of-tree target ran.
    #[test]
    fn unsafe_prompt_path_reported_not_read() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let good = hash_bytes(b"x\n");
        write(
            root,
            PROMPTS_FILE,
            &format!(
                "\
- id: prompt.escape
  path: ../escape.txt
  template_hash: {good}
  route: route.single_ir
"
            ),
        );
        let (result, diagnostics) = model_checked(root);
        assert_eq!(result.outcome, Outcome::Invalid);
        assert_eq!(diagnostics.len(), 1);
        assert_reported(&diagnostics, "prompt.escape");
        assert_reported(&diagnostics, "not a safe repo-relative path");
    }
}
