//! SPEC §5 run plan and run manifest records, plus the SPEC §4.6 replay
//! manifest.
//!
//! [`RunPlan`] fixes what a run executes (experiment, test_source groups,
//! pipelines, seed, budget); its canonical bytes hash into
//! [`RunManifest::run_plan_hash`] via [`RunPlan::plan_hash`]. [`RunManifest`]
//! attests what a run was built from and produced. [`ReplayManifest`] is the
//! §4.6 provenance/attestation record over content hashes that `ckc replay`
//! re-executes and compares against (runtime metadata excluded). The CLI
//! behavior behind these records lands with the run/replay commands; this
//! module owns their durable shapes.

use crate::canon::{
    CanonError, CanonRead, CanonReadError, Canonical, ObjectEmitter, ObjectReader, RawText, Reader,
    emit_array, emit_map, emit_raw_map, emit_set, emit_u64, emit_u64_map, read_array, read_map,
    read_raw_map, read_set, read_u64, read_u64_map,
};
use crate::hash::content_hash;
use crate::id::{Hash, Id};

/// SPEC §5/§4.6 solver identity: the solver a run's verdicts depend on,
/// recorded in manifests (and later verifier results) per the §1 rule that
/// tool versions live in lockfiles and manifests, never prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolverIdentity {
    /// Solver name as an identifier (e.g. `z3`).
    pub solver_id: Id,
    /// Version token as reported by the tool, raw text.
    pub version: String,
}

impl Canonical for SolverIdentity {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("solver_id", |b| self.solver_id.emit_canonical(b))?;
        obj.member("version", |b| {
            RawText(self.version.clone()).emit_canonical(b)
        })?;
        obj.finish(out)
    }
}

impl CanonRead for SolverIdentity {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let solver_id = obj.member("solver_id", Id::read)?;
        let version = obj.member("version", |r| Ok(RawText::read(r)?.0))?;
        obj.close()?;
        Ok(SolverIdentity { solver_id, version })
    }
}

/// SPEC §5 run plan: experiment id, test_source groups, pipeline(s), seed,
/// budget — everything that fixes what a run executes, before it runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPlan {
    /// Experiment registry entry this run executes (e.g. `exp.m1_scaffold`).
    pub experiment_id: Id,
    /// TestSource groups in scope (§8.2 `group.*`). Set semantics.
    pub test_source_groups: Vec<Id>,
    /// Pipeline candidates the run executes — a singleton at M1; M2 onward
    /// runs several over the same test_sources. Set semantics.
    pub pipelines: Vec<Id>,
    /// Deterministic seed for any seeded processing_stage.
    pub seed: u64,
    /// Budget caps: counter name → limit, the counters §4.6 event
    /// `resource_counters` consume against. Map semantics.
    pub budget: Vec<(Id, u64)>,
}

impl RunPlan {
    /// The §5 requirements "canonical bytes hashed into the manifest": this
    /// plan's [`content_hash`], stored as [`RunManifest::run_plan_hash`].
    pub fn plan_hash(&self) -> Result<Hash, CanonError> {
        content_hash(self)
    }
}

impl Canonical for RunPlan {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("budget", |b| emit_u64_map(b, &self.budget))?;
        obj.member("experiment_id", |b| self.experiment_id.emit_canonical(b))?;
        obj.member("pipelines", |b| emit_set(b, &self.pipelines))?;
        obj.member("seed", |b| {
            emit_u64(b, self.seed);
            Ok(())
        })?;
        obj.member("test_source_groups", |b| {
            emit_set(b, &self.test_source_groups)
        })?;
        obj.finish(out)
    }
}

impl CanonRead for RunPlan {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let budget = obj.member("budget", read_u64_map)?;
        let experiment_id = obj.member("experiment_id", Id::read)?;
        let pipelines = obj.member("pipelines", read_set::<Id>)?;
        let seed = obj.member("seed", read_u64)?;
        let test_source_groups = obj.member("test_source_groups", read_set::<Id>)?;
        obj.close()?;
        Ok(RunPlan {
            experiment_id,
            test_source_groups,
            pipelines,
            seed,
            budget,
        })
    }
}

/// SPEC §5 run manifest: run plan hash, git commit, toolchain/lockfile/
/// corpus/lexicon hashes, environment profile, solver identity, output
/// hashes — what a run was built from and what it produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunManifest {
    /// [`RunPlan::plan_hash`] of the executed plan.
    pub run_plan_hash: Hash,
    /// Repository commit the run was built at, raw hex text.
    pub git_commit: String,
    /// Hash of the toolchain manifest in force (§4.4 `_hash` raw-byte rule —
    /// the manifest is a file, not an accepted artifact).
    pub toolchain_manifest_hash: Hash,
    /// Lockfile name → raw-byte hash (e.g. `cargo.lock`). Map semantics.
    pub lockfile_hashes: Vec<(Id, Hash)>,
    /// Content hash versioning the corpus in force.
    pub corpus_hash: Hash,
    /// Content hash versioning the lexicon in force (§5 lexicon requirements).
    pub lexicon_hash: Hash,
    /// Recorded environment facts: identifier keys to raw text. Map
    /// semantics.
    pub environment_profile: Vec<(Id, String)>,
    pub solver_identity: SolverIdentity,
    /// Content hashes of the run's accepted artifacts. Set semantics.
    pub output_hashes: Vec<Hash>,
}

impl Canonical for RunManifest {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("corpus_hash", |b| self.corpus_hash.emit_canonical(b))?;
        obj.member("environment_profile", |b| {
            emit_raw_map(b, &self.environment_profile)
        })?;
        obj.member("git_commit", |b| {
            RawText(self.git_commit.clone()).emit_canonical(b)
        })?;
        obj.member("lexicon_hash", |b| self.lexicon_hash.emit_canonical(b))?;
        obj.member("lockfile_hashes", |b| {
            emit_map(b, self.lockfile_hashes.iter().map(|(k, v)| (k, v)))
        })?;
        obj.member("output_hashes", |b| emit_set(b, &self.output_hashes))?;
        obj.member("run_plan_hash", |b| self.run_plan_hash.emit_canonical(b))?;
        obj.member("solver_identity", |b| {
            self.solver_identity.emit_canonical(b)
        })?;
        obj.member("toolchain_manifest_hash", |b| {
            self.toolchain_manifest_hash.emit_canonical(b)
        })?;
        obj.finish(out)
    }
}

impl CanonRead for RunManifest {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let corpus_hash = obj.member("corpus_hash", Hash::read)?;
        let environment_profile = obj.member("environment_profile", read_raw_map)?;
        let git_commit = obj.member("git_commit", |r| Ok(RawText::read(r)?.0))?;
        let lexicon_hash = obj.member("lexicon_hash", Hash::read)?;
        let lockfile_hashes = obj.member("lockfile_hashes", read_map::<Id, Hash>)?;
        let output_hashes = obj.member("output_hashes", read_set::<Hash>)?;
        let run_plan_hash = obj.member("run_plan_hash", Hash::read)?;
        let solver_identity = obj.member("solver_identity", SolverIdentity::read)?;
        let toolchain_manifest_hash = obj.member("toolchain_manifest_hash", Hash::read)?;
        obj.close()?;
        Ok(RunManifest {
            run_plan_hash,
            git_commit,
            toolchain_manifest_hash,
            lockfile_hashes,
            corpus_hash,
            lexicon_hash,
            environment_profile,
            solver_identity,
            output_hashes,
        })
    }
}

/// SPEC §4.6 `replay_manifest.json`: command, input hashes, lexicon/corpus
/// hashes, toolchain manifest hash, environment profile, lockfile hashes,
/// solver identity, expected output hashes — a provenance/attestation record
/// over content hashes. `ckc replay` re-executes `command` over the same
/// inputs and compares canonical content hashes; mismatches emit
/// mismatch diagnostics and missing external tools emit
/// `replay_identity_unsupported`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayManifest {
    /// Argv tokens of the command to re-execute, in semantic order.
    pub command: Vec<String>,
    /// Content hashes of the inputs the run consumed. Set semantics.
    pub input_hashes: Vec<Hash>,
    /// Content hash versioning the lexicon in force.
    pub lexicon_hash: Hash,
    /// Content hash versioning the corpus in force.
    pub corpus_hash: Hash,
    /// Hash of the toolchain manifest in force (§4.4 `_hash` raw-byte rule).
    pub toolchain_manifest_hash: Hash,
    /// Recorded environment facts: identifier keys to raw text. Map
    /// semantics.
    pub environment_profile: Vec<(Id, String)>,
    /// Lockfile name → raw-byte hash. Map semantics.
    pub lockfile_hashes: Vec<(Id, Hash)>,
    pub solver_identity: SolverIdentity,
    /// Content hashes the re-execution must reproduce. Set semantics.
    pub expected_output_hashes: Vec<Hash>,
}

impl Canonical for ReplayManifest {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("command", |b| {
            let argv: Vec<RawText> = self.command.iter().map(|s| RawText(s.clone())).collect();
            emit_array(b, &argv)
        })?;
        obj.member("corpus_hash", |b| self.corpus_hash.emit_canonical(b))?;
        obj.member("environment_profile", |b| {
            emit_raw_map(b, &self.environment_profile)
        })?;
        obj.member("expected_output_hashes", |b| {
            emit_set(b, &self.expected_output_hashes)
        })?;
        obj.member("input_hashes", |b| emit_set(b, &self.input_hashes))?;
        obj.member("lexicon_hash", |b| self.lexicon_hash.emit_canonical(b))?;
        obj.member("lockfile_hashes", |b| {
            emit_map(b, self.lockfile_hashes.iter().map(|(k, v)| (k, v)))
        })?;
        obj.member("solver_identity", |b| {
            self.solver_identity.emit_canonical(b)
        })?;
        obj.member("toolchain_manifest_hash", |b| {
            self.toolchain_manifest_hash.emit_canonical(b)
        })?;
        obj.finish(out)
    }
}

impl CanonRead for ReplayManifest {
    fn read(r: &mut Reader<'_>) -> Result<Self, CanonReadError> {
        let mut obj = ObjectReader::open(r)?;
        let command = obj.member("command", |r| {
            Ok(read_array::<RawText>(r)?
                .into_iter()
                .map(|t| t.0)
                .collect::<Vec<String>>())
        })?;
        let corpus_hash = obj.member("corpus_hash", Hash::read)?;
        let environment_profile = obj.member("environment_profile", read_raw_map)?;
        let expected_output_hashes = obj.member("expected_output_hashes", read_set::<Hash>)?;
        let input_hashes = obj.member("input_hashes", read_set::<Hash>)?;
        let lexicon_hash = obj.member("lexicon_hash", Hash::read)?;
        let lockfile_hashes = obj.member("lockfile_hashes", read_map::<Id, Hash>)?;
        let solver_identity = obj.member("solver_identity", SolverIdentity::read)?;
        let toolchain_manifest_hash = obj.member("toolchain_manifest_hash", Hash::read)?;
        obj.close()?;
        Ok(ReplayManifest {
            command,
            input_hashes,
            lexicon_hash,
            corpus_hash,
            toolchain_manifest_hash,
            environment_profile,
            lockfile_hashes,
            solver_identity,
            expected_output_hashes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon::{canonical_payload_bytes, read_strict_canonical};

    /// Canonical bytes of `value` as a UTF-8 string, for exact-match assertions.
    fn canon<T: Canonical>(value: &T) -> String {
        String::from_utf8(canonical_payload_bytes(value).unwrap()).unwrap()
    }

    /// Assert `value` survives a canonical write -> read round trip unchanged.
    fn round_trip<T: Canonical + CanonRead + std::fmt::Debug + PartialEq>(value: T) {
        let bytes = canonical_payload_bytes(&value).unwrap();
        let got: T = read_strict_canonical(&bytes).unwrap();
        assert_eq!(got, value, "round trip changed the value");
    }

    /// A valid [`Hash`] built from one repeated hex digit.
    fn h(digit: char) -> Hash {
        Hash::new(format!("sha256:{}", digit.to_string().repeat(64))).unwrap()
    }

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn z3() -> SolverIdentity {
        SolverIdentity {
            solver_id: id("z3"),
            version: "4.13.4".to_owned(),
        }
    }

    fn sample_plan() -> RunPlan {
        RunPlan {
            experiment_id: id("exp.m1_scaffold"),
            test_source_groups: vec![id("group.m1_conflict"), id("group.m1_no_conflict")],
            pipelines: vec![id("pipe.layered_ckcir_to_smt")],
            seed: 42,
            budget: vec![(id("solver_ms_per_query"), 10_000)],
        }
    }

    fn sample_manifest() -> RunManifest {
        RunManifest {
            run_plan_hash: sample_plan().plan_hash().unwrap(),
            git_commit: "0d424a397281bc8a276f4dd666c433a89d6b1228".to_owned(),
            toolchain_manifest_hash: h('a'),
            lockfile_hashes: vec![(id("cargo.lock"), h('b'))],
            corpus_hash: h('c'),
            lexicon_hash: h('d'),
            environment_profile: vec![
                (id("arch"), "x86_64".to_owned()),
                (id("os"), "linux".to_owned()),
            ],
            solver_identity: z3(),
            output_hashes: vec![h('e')],
        }
    }

    fn sample_replay() -> ReplayManifest {
        ReplayManifest {
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
            input_hashes: vec![h('f')],
            lexicon_hash: h('d'),
            corpus_hash: h('c'),
            toolchain_manifest_hash: h('a'),
            environment_profile: vec![(id("os"), "linux".to_owned())],
            lockfile_hashes: vec![(id("cargo.lock"), h('b'))],
            solver_identity: z3(),
            expected_output_hashes: vec![h('e')],
        }
    }

    // Pins the §5 RunPlan canonical shape: every field, byte-sorted members,
    // set-ordered groups, decimal-string integers.
    #[test]
    fn run_plan_canonical_bytes() {
        assert_eq!(
            canon(&sample_plan()),
            concat!(
                r#"{"budget":{"solver_ms_per_query":"10000"},"#,
                r#""experiment_id":"exp.m1_scaffold","#,
                r#""pipelines":["pipe.layered_ckcir_to_smt"],"seed":"42","#,
                r#""test_source_groups":["group.m1_conflict","group.m1_no_conflict"]}"#
            )
        );
        // Empty collections keep their type-guided forms ({} map, [] sets).
        let empty = RunPlan {
            experiment_id: id("exp.m1_scaffold"),
            test_source_groups: vec![],
            pipelines: vec![],
            seed: 0,
            budget: vec![],
        };
        assert_eq!(
            canon(&empty),
            concat!(
                r#"{"budget":{},"experiment_id":"exp.m1_scaffold","#,
                r#""pipelines":[],"seed":"0","test_source_groups":[]}"#
            )
        );
    }

    // §5: the plan's canonical bytes hash into the manifest — deterministic,
    // value-sensitive, and stable across a round trip.
    #[test]
    fn plan_hash_is_deterministic_and_value_sensitive() {
        let plan = sample_plan();
        assert_eq!(
            plan.plan_hash().unwrap(),
            sample_plan().plan_hash().unwrap()
        );
        let bytes = canonical_payload_bytes(&plan).unwrap();
        let reread: RunPlan = read_strict_canonical(&bytes).unwrap();
        assert_eq!(reread.plan_hash().unwrap(), plan.plan_hash().unwrap());
        let mut reseeded = sample_plan();
        reseeded.seed = 43;
        assert_ne!(reseeded.plan_hash().unwrap(), plan.plan_hash().unwrap());
    }

    // Pins the §5 RunManifest canonical shape, with the plan hash linked in.
    #[test]
    fn run_manifest_canonical_bytes() {
        let manifest = sample_manifest();
        let want = format!(
            concat!(
                r#"{{"corpus_hash":"{c}","#,
                r#""environment_profile":{{"arch":"x86_64","os":"linux"}},"#,
                r#""git_commit":"0d424a397281bc8a276f4dd666c433a89d6b1228","#,
                r#""lexicon_hash":"{d}","lockfile_hashes":{{"cargo.lock":"{b}"}},"#,
                r#""output_hashes":["{e}"],"run_plan_hash":"{plan}","#,
                r#""solver_identity":{{"solver_id":"z3","version":"4.13.4"}},"#,
                r#""toolchain_manifest_hash":"{a}"}}"#
            ),
            a = h('a').as_str(),
            b = h('b').as_str(),
            c = h('c').as_str(),
            d = h('d').as_str(),
            e = h('e').as_str(),
            plan = sample_plan().plan_hash().unwrap().as_str(),
        );
        assert_eq!(canon(&manifest), want);
    }

    // Pins the §4.6 replay-manifest field list in canonical order, with the
    // command as an ordered argv array.
    #[test]
    fn replay_manifest_canonical_bytes() {
        let want = format!(
            concat!(
                r#"{{"command":["ckc","run","--experiment","exp.m1_scaffold","--out","runs/m1"],"#,
                r#""corpus_hash":"{c}","environment_profile":{{"os":"linux"}},"#,
                r#""expected_output_hashes":["{e}"],"input_hashes":["{f}"],"#,
                r#""lexicon_hash":"{d}","lockfile_hashes":{{"cargo.lock":"{b}"}},"#,
                r#""solver_identity":{{"solver_id":"z3","version":"4.13.4"}},"#,
                r#""toolchain_manifest_hash":"{a}"}}"#
            ),
            a = h('a').as_str(),
            b = h('b').as_str(),
            c = h('c').as_str(),
            d = h('d').as_str(),
            e = h('e').as_str(),
            f = h('f').as_str(),
        );
        assert_eq!(canon(&sample_replay()), want);
    }

    #[test]
    fn plan_and_manifests_round_trip() {
        round_trip(sample_plan());
        round_trip(sample_manifest());
        round_trip(sample_replay());
        round_trip(z3());
    }
}
