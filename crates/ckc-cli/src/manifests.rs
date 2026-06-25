//! SPEC §5/§4.6 manifest assembly — the run landing's core for
//! `manifest.json` ([`ckc_core::RunManifest`]) and `replay_manifest.json`
//! ([`ckc_core::ReplayManifest`]) of the §8.3 layout. [`assemble_manifests`]
//! builds both records over caller-supplied hash/identity values: no I/O,
//! no clock, no environment probing — the run landing (`run::manifest_inputs`)
//! gathers the values; this module owns the §5 plan-hash linkage, canonical
//! storage order for every set and map field, and the shared-provenance
//! guarantee: the two records agree on every fact they both attest, and the
//! §4.6 expected output hashes are exactly the §5 output hashes, so a
//! replay that matches the replay manifest matches the run manifest too.

use ckc_core::{CanonError, Hash, Id, ReplayManifest, RunManifest, RunPlan, SolverIdentity};

/// Caller-supplied run state, one value per fact the §5/§4.6 records
/// attest. Collection fields arrive in any order; assembly sorts them into
/// canonical storage (§4.3 set/map semantics). `command` is the §4.6 argv
/// `ckc replay` re-executes; `toolchain_manifest_hash` is the §4.4
/// producer value (the toolchain manifest file's raw-byte hash).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestInputs {
    /// The executed §5 plan; its canonical bytes hash into
    /// [`RunManifest::run_plan_hash`].
    pub plan: RunPlan,
    /// Argv tokens of the command to re-execute, in semantic order.
    pub command: Vec<String>,
    /// Repository commit the run was built at, raw hex text.
    pub git_commit: String,
    /// Hash of the toolchain manifest in force (§4.4 raw-byte rule).
    pub toolchain_manifest_hash: Hash,
    /// Lockfile name → raw-byte hash.
    pub lockfile_hashes: Vec<(Id, Hash)>,
    /// Content hash versioning the corpus in force.
    pub corpus_hash: Hash,
    /// Content hash versioning the lexicon in force.
    pub lexicon_hash: Hash,
    /// Recorded environment facts: identifier keys to raw text.
    pub environment_profile: Vec<(Id, String)>,
    pub solver_identity: SolverIdentity,
    /// Content hashes of the inputs the run consumed.
    pub input_hashes: Vec<Hash>,
    /// Content hashes of the run's accepted artifacts — the §5 output
    /// hashes and, verbatim, the §4.6 expected output hashes.
    pub output_hashes: Vec<Hash>,
}

/// Assemble the §5 run manifest and §4.6 replay manifest over one set of
/// caller-supplied values. Set fields sort and dedup into canonical
/// storage; map fields sort by key bytes with duplicate keys rejected;
/// `run_plan_hash` is computed from `plan`; the §4.6 record mirrors the
/// §5 record's shared fields, with `expected_output_hashes` the same
/// sorted set as `output_hashes`. Assembled values are canonical storage:
/// they round-trip their own canonical bytes unchanged.
pub fn assemble_manifests(
    inputs: &ManifestInputs,
) -> Result<(RunManifest, ReplayManifest), ManifestError> {
    for (field, is_empty) in [
        ("command", inputs.command.is_empty()),
        ("git_commit", inputs.git_commit.is_empty()),
        ("input_hashes", inputs.input_hashes.is_empty()),
        ("output_hashes", inputs.output_hashes.is_empty()),
    ] {
        if is_empty {
            return Err(ManifestError::Empty { field });
        }
    }
    let run_plan_hash = inputs.plan.plan_hash()?;
    let lockfile_hashes = sorted_map("lockfile_hashes", &inputs.lockfile_hashes)?;
    let environment_profile = sorted_map("environment_profile", &inputs.environment_profile)?;
    let input_hashes = sorted_hash_set(&inputs.input_hashes);
    let output_hashes = sorted_hash_set(&inputs.output_hashes);
    let manifest = RunManifest {
        run_plan_hash,
        git_commit: inputs.git_commit.clone(),
        toolchain_manifest_hash: inputs.toolchain_manifest_hash.clone(),
        lockfile_hashes: lockfile_hashes.clone(),
        corpus_hash: inputs.corpus_hash.clone(),
        lexicon_hash: inputs.lexicon_hash.clone(),
        environment_profile: environment_profile.clone(),
        solver_identity: inputs.solver_identity.clone(),
        output_hashes: output_hashes.clone(),
        // §9 M2 measurement record: omitted on deterministic runs, populated
        // by the model-route run path.
        model_identity: None,
        test_source_hash: None,
        reference_hash: None,
        schema_hash: None,
        prompt_template_hash: None,
        model_hash: None,
        runtime_hash: None,
    };
    let replay = ReplayManifest {
        command: inputs.command.clone(),
        input_hashes,
        lexicon_hash: inputs.lexicon_hash.clone(),
        corpus_hash: inputs.corpus_hash.clone(),
        toolchain_manifest_hash: inputs.toolchain_manifest_hash.clone(),
        environment_profile,
        lockfile_hashes,
        solver_identity: inputs.solver_identity.clone(),
        expected_output_hashes: output_hashes,
        // §9 M2 measurement record: omitted on deterministic runs.
        model_identity: None,
        test_source_hash: None,
        reference_hash: None,
        schema_hash: None,
        prompt_template_hash: None,
        model_hash: None,
        runtime_hash: None,
    };
    Ok((manifest, replay))
}

/// §4.3 map storage: entries sorted by key bytes, duplicate keys rejected
/// (`pool` names the field in errors). Id and Hash canonical bytes are
/// their raw text quoted without escapes, so raw-byte order is canonical
/// order.
fn sorted_map<V: Clone>(
    pool: &'static str,
    entries: &[(Id, V)],
) -> Result<Vec<(Id, V)>, ManifestError> {
    let mut out = entries.to_vec();
    out.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
    for pair in out.windows(2) {
        if pair[0].0 == pair[1].0 {
            return Err(ManifestError::DuplicateKey {
                pool,
                key: pair[0].0.clone(),
            });
        }
    }
    Ok(out)
}

/// §4.3 set storage for hashes: sorted by hash bytes, byte-identical
/// duplicates collapsed (matching `emit_set`, which dedups at emission).
fn sorted_hash_set(hashes: &[Hash]) -> Vec<Hash> {
    let mut out = hashes.to_vec();
    out.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    out.dedup();
    out
}

/// Assembly failure taxonomy: every variant names its offending field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// A required field arrived empty.
    Empty { field: &'static str },
    /// A map field carries one key twice.
    DuplicateKey { pool: &'static str, key: Id },
    /// Canonical emission failed while hashing the plan.
    Canon(CanonError),
}

impl From<CanonError> for ManifestError {
    fn from(e: CanonError) -> Self {
        ManifestError::Canon(e)
    }
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Empty { field } => write!(f, "{field} is empty"),
            ManifestError::DuplicateKey { pool, key } => {
                write!(f, "{pool} carries key {key} twice")
            }
            ManifestError::Canon(e) => write!(f, "canonical emission failed: {e:?}"),
        }
    }
}

impl std::error::Error for ManifestError {}

#[cfg(test)]
mod tests {
    use ckc_core::{canonical_payload_bytes, read_strict_canonical};

    use super::*;

    fn id(text: &str) -> Id {
        Id::new(text.to_owned()).unwrap()
    }

    fn hash(seed: char) -> Hash {
        Hash::new(format!("sha256:{}", seed.to_string().repeat(64))).unwrap()
    }

    /// Inputs with every collection deliberately unsorted and one output
    /// hash duplicated, so assembly's ordering work is observable.
    fn inputs() -> ManifestInputs {
        ManifestInputs {
            plan: RunPlan {
                experiment_id: id("exp.m1_scaffold"),
                test_source_groups: vec![id("group.m1_conflict"), id("group.m1_no_conflict")],
                pipelines: vec![id("pipe.layered_ckcir_to_smt")],
                seed: 42,
                budget: vec![(id("solver_ms_per_query"), 10_000)],
            },
            command: [
                "ckc",
                "run",
                "--experiment",
                "exp.m1_scaffold",
                "--out",
                "runs/m1",
            ]
            .map(str::to_owned)
            .to_vec(),
            git_commit: "79bc570fffffffffffffffffffffffffffffffff".to_owned(),
            toolchain_manifest_hash: hash('a'),
            lockfile_hashes: vec![
                (id("rust-toolchain.lock"), hash('b')),
                (id("cargo.lock"), hash('c')),
            ],
            corpus_hash: hash('d'),
            lexicon_hash: hash('e'),
            environment_profile: vec![
                (id("os"), "linux".to_owned()),
                (id("arch"), "x86_64".to_owned()),
            ],
            solver_identity: SolverIdentity {
                solver_id: id("z3"),
                version: "4.13.4".to_owned(),
            },
            input_hashes: vec![hash('9'), hash('1')],
            output_hashes: vec![hash('7'), hash('2'), hash('7')],
        }
    }

    #[test]
    fn assembly_sorts_links_and_mirrors() {
        let (manifest, replay) = assemble_manifests(&inputs()).unwrap();
        // §5 linkage: the manifest carries the plan's content hash.
        assert_eq!(manifest.run_plan_hash, inputs().plan.plan_hash().unwrap());
        // Canonical storage: maps key-sorted, sets sorted and deduped.
        assert_eq!(
            manifest.lockfile_hashes,
            vec![
                (id("cargo.lock"), hash('c')),
                (id("rust-toolchain.lock"), hash('b'))
            ]
        );
        assert_eq!(
            manifest.environment_profile,
            vec![
                (id("arch"), "x86_64".to_owned()),
                (id("os"), "linux".to_owned()),
            ]
        );
        assert_eq!(replay.input_hashes, vec![hash('1'), hash('9')]);
        assert_eq!(manifest.output_hashes, vec![hash('2'), hash('7')]);
        // Shared-provenance guarantee: every co-attested fact agrees, and
        // the §4.6 expectation is the §5 output set verbatim.
        assert_eq!(replay.corpus_hash, manifest.corpus_hash);
        assert_eq!(replay.lexicon_hash, manifest.lexicon_hash);
        assert_eq!(
            replay.toolchain_manifest_hash,
            manifest.toolchain_manifest_hash
        );
        assert_eq!(replay.environment_profile, manifest.environment_profile);
        assert_eq!(replay.lockfile_hashes, manifest.lockfile_hashes);
        assert_eq!(replay.solver_identity, manifest.solver_identity);
        assert_eq!(replay.expected_output_hashes, manifest.output_hashes);
        // Deterministic: same inputs, same records.
        assert_eq!(assemble_manifests(&inputs()).unwrap(), (manifest, replay));
    }

    #[test]
    fn assembled_values_are_canonical_storage() {
        let (manifest, replay) = assemble_manifests(&inputs()).unwrap();
        let manifest_bytes = canonical_payload_bytes(&manifest).unwrap();
        assert_eq!(
            read_strict_canonical::<RunManifest>(&manifest_bytes).unwrap(),
            manifest
        );
        let replay_bytes = canonical_payload_bytes(&replay).unwrap();
        assert_eq!(
            read_strict_canonical::<ReplayManifest>(&replay_bytes).unwrap(),
            replay
        );
    }

    #[test]
    fn duplicate_map_keys_are_rejected() {
        let mut dup = inputs();
        dup.lockfile_hashes.push((id("cargo.lock"), hash('f')));
        assert_eq!(
            assemble_manifests(&dup),
            Err(ManifestError::DuplicateKey {
                pool: "lockfile_hashes",
                key: id("cargo.lock"),
            })
        );
        let mut dup = inputs();
        dup.environment_profile.push((id("os"), "linux".to_owned()));
        assert_eq!(
            assemble_manifests(&dup),
            Err(ManifestError::DuplicateKey {
                pool: "environment_profile",
                key: id("os"),
            })
        );
    }

    #[test]
    fn empty_required_fields_are_rejected() {
        for (field, wipe) in [
            (
                "command",
                Box::new(|i: &mut ManifestInputs| i.command.clear())
                    as Box<dyn Fn(&mut ManifestInputs)>,
            ),
            (
                "git_commit",
                Box::new(|i: &mut ManifestInputs| i.git_commit.clear()),
            ),
            (
                "input_hashes",
                Box::new(|i: &mut ManifestInputs| i.input_hashes.clear()),
            ),
            (
                "output_hashes",
                Box::new(|i: &mut ManifestInputs| i.output_hashes.clear()),
            ),
        ] {
            let mut bad = inputs();
            wipe(&mut bad);
            assert_eq!(
                assemble_manifests(&bad),
                Err(ManifestError::Empty { field }),
                "{field} must be required"
            );
        }
    }
}
