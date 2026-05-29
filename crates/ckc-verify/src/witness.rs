//! ExecutionWitness builder (SPEC 10 ExecutionWitness, 11.1 witness semantics).
//!
//! Turns each compiled target plus its recorded oracle verdict ([`crate::verdict`])
//! into an [`ExecutionWitness`]: the source spans the target's symbols map back to,
//! the applicable/defeated rules and violated constraints the verdict exposes, the
//! model atoms or unsat core that witness it, and the certificate ([`certificate_for`])
//! it is checked by. Determinism is structural — the span union is sorted and
//! deduplicated, and [`normalize_all`] derives the stable `witness_id` (pass 12)
//! from the normalized content — so a witness's content hash is stable across runs
//! and machines, which the downstream certificate-graph and verification-manifest
//! goldens depend on.

use serde_json::Value;

use ckc_compile::{CompileBundle, CompiledTarget, compile_all};
use ckc_core::artifact::ExecutionWitness;
use ckc_core::enums::RuleKind;
use ckc_core::id::{BundleId, RuleId, SpanId, WitnessId};
use ckc_core::nf::normalize_all;

use crate::certificate::certificate_for;
use crate::verdict::{
    RecordedOutcomes, SolverId, VerdictStatus, VerifierOutcome, load_recorded_outcomes,
};

/// Repo-relative artifact path of the defeasible/argumentation ASP target
/// (`ARTIFACT_PATHS[3]`): the verdict whose stable model reports the applicable
/// and defeated rules.
const DEFEASIBLE_PATH: &str = "logic/asp/defeasible.lp";

/// Repo-relative artifact path of the Event Calculus ASP target
/// (`ARTIFACT_PATHS[4]`): the verdict whose stable model reports the temporal
/// violation.
const EVENT_CALCULUS_PATH: &str = "logic/asp/event_calculus.lp";

/// The norm-conflict unsat core in CKC terms: the two deontic projections that
/// jointly contradict under the shared sepsis + anaphylaxis context. Absent from
/// the SMT model (the run is unsat), so it is named here rather than read from a
/// model atom.
const NORM_CONFLICT_CONTRADICTION: &str =
    "recommend_administer_beta_lactam AND prohibit_administer_beta_lactam";

/// Single argument of every `<prefix>…)` atom in `atoms`. With prefix
/// `"defeated("`, `defeated(rule_x)` yields `rule_x` while the argumentation atom
/// `defeated_arg(arg_x)` is excluded (its prefix is `defeated_arg(`, not
/// `defeated(`). With prefix `"violation("`, `violation(conflict_y)` yields
/// `conflict_y`.
fn atom_args(prefix: &str, atoms: &[String]) -> Vec<String> {
    atoms
        .iter()
        .filter_map(|atom| {
            atom.strip_prefix(prefix)?
                .strip_suffix(')')
                .map(str::to_string)
        })
        .collect()
}

/// Each salient atom rendered as a JSON string value — the witness `models` /
/// `unsat_cores` element form.
fn atoms_as_json(atoms: &[String]) -> Vec<Value> {
    atoms.iter().map(|a| Value::String(a.clone())).collect()
}

/// Build the [`ExecutionWitness`] for one compiled `target` under its recorded
/// `outcome`. `source_span_ids` is the deduplicated union of the target's
/// compilation-map span ids (so every verifier output maps back to source spans);
/// `applicable_rules`/`defeated_rules` are filled for the defeasible ASP verdict;
/// `violated_constraints` records the EC temporal violation and the norm-conflict
/// contradiction; `models` carries the model atoms of a sat/satisfiable verdict and
/// `unsat_cores` the core of an unsat one; and `certificate_ids` links the matching
/// [`certificate_for`] certificate. [`normalize_all`] then derives the stable
/// `witness_id`.
pub fn witness_for(
    target: &CompiledTarget,
    outcome: &VerifierOutcome,
    bundle: &CompileBundle,
) -> ExecutionWitness {
    let mut source_span_ids: Vec<SpanId> = target
        .compilation_map
        .0
        .iter()
        .flat_map(|mapping| mapping.source_span_ids.iter().cloned())
        .collect();
    source_span_ids.sort();
    source_span_ids.dedup();

    // The defeasible ASP run is the sole verdict that reports rule defeat: its
    // applicable set is the bundle's defeasible-kind rules, and its defeated set is
    // read from the model's `defeated(<rule>)` atoms.
    let (applicable_rules, defeated_rules): (Vec<RuleId>, Vec<RuleId>) =
        if outcome.artifact_path == DEFEASIBLE_PATH {
            let applicable = bundle
                .rules
                .iter()
                .filter(|rule| rule.kind == RuleKind::Defeasible)
                .map(|rule| rule.rule_id.clone())
                .collect();
            let defeated = atom_args("defeated(", &outcome.salient_atoms)
                .into_iter()
                .map(RuleId::new)
                .collect();
            (applicable, defeated)
        } else {
            (Vec::new(), Vec::new())
        };

    // Violated constraints: the EC model's `violation(<conflict>)` atom, and the
    // norm-conflict contradiction whenever the SMT proof is unsat.
    let is_unsat = outcome.status == VerdictStatus::Unsat;
    let mut violated_constraints = Vec::new();
    if outcome.artifact_path == EVENT_CALCULUS_PATH {
        violated_constraints.extend(atom_args("violation(", &outcome.salient_atoms));
    }
    if is_unsat {
        violated_constraints.push(NORM_CONFLICT_CONTRADICTION.to_string());
    }

    // A sat/satisfiable verdict is witnessed by its model atoms; an unsat verdict by
    // its core (the contradictory constraints just collected).
    let models = match outcome.status {
        VerdictStatus::Sat | VerdictStatus::Satisfiable => atoms_as_json(&outcome.salient_atoms),
        _ => Vec::new(),
    };
    let unsat_cores = if is_unsat {
        violated_constraints
            .iter()
            .map(|c| Value::String(c.clone()))
            .collect()
    } else {
        Vec::new()
    };

    let certificate_ids = vec![certificate_for(target, outcome).certificate_id];

    let mut witness = ExecutionWitness {
        witness_id: WitnessId::new(String::new()),
        bundle_id: BundleId::new("bundle_research_kernel"),
        case_id: None,
        context_facts: Vec::new(),
        trace: Vec::new(),
        applicable_rules,
        defeated_rules,
        violated_constraints,
        models,
        unsat_cores,
        source_span_ids,
        certificate_ids,
    };
    normalize_all(&mut witness);
    witness
}

/// Build the Phase-0 witness set: one witness per `compile_all` target paired with
/// its recorded outcome (9, in `ARTIFACT_PATHS` order), plus the standalone cvc5
/// witness over the same norm-conflict SMT target — 10 in all, mirroring
/// [`certificates`](crate::certificate::certificates).
pub fn witnesses(bundle: &CompileBundle) -> Vec<ExecutionWitness> {
    let targets = compile_all(bundle);
    let RecordedOutcomes(outcomes) = load_recorded_outcomes();

    let mut ws: Vec<ExecutionWitness> = targets
        .iter()
        .zip(outcomes.iter())
        .map(|(target, outcome)| witness_for(target, outcome, bundle))
        .collect();

    let cvc5 = outcomes
        .iter()
        .find(|o| o.solver == SolverId::Cvc5)
        .expect("recorded oracle includes the cvc5 outcome");
    ws.push(witness_for(&targets[0], cvc5, bundle));

    ws
}
