//! Portfolio agreement and backend-disagreement detection (SPEC 12.2
//! C5-Portfolio, 13 backend cross-checking, 14/15.1 first-class disagreement
//! reporting).
//!
//! Clusters the recorded verifier outcomes ([`crate::verdict`]) by the
//! underlying CKC claim — not by target artifact — so the several backends that
//! check one claim are compared against each other. When every backend in a
//! cluster reaches the same conclusion the claim earns a `C5-Portfolio`
//! [`AgreementRecord`]; when they split, a `backend_disagreement`
//! [`CompileDiagnostic`] surfaces the contradiction for review. Single-backend
//! targets are not portfolios and yield neither.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_core::compile::CompileDiagnostic;
use ckc_core::enums::CertificateClass;

use crate::certificate::wire_token;
use crate::verdict::{SolverId, VerdictStatus, VerifierOutcome};

/// One backend's contribution to a portfolio claim: the target artifact it
/// checks and the recorded status that establishes the claim for that backend.
/// Independent backends establish one claim with different status tokens (an SMT
/// `unsat`, a Lean `kernel_checked`, a Datalog `empty_relation`, an Alloy
/// `no_counterexample`), so the established status is recorded per member.
struct ClaimMember {
    artifact_path: &'static str,
    established_status: VerdictStatus,
}

/// A CKC claim cross-checked by ≥2 independent backends — the unit of portfolio
/// agreement. `claim_id` is the underlying CKC claim/conflict id (e.g. the real
/// `conflict_norm_bl_contradiction` from `conflicts.json`), so clustering keys on
/// the claim; each member's `artifact_path` is only the lookup key the recorded
/// oracle carries, since a [`VerifierOutcome`] names its target artifact rather
/// than its claim.
struct ClaimSpec {
    claim_id: &'static str,
    members: &'static [ClaimMember],
}

/// The Phase-0 multi-backend claims (SPEC 20 scenario 1 plus the toy
/// priority-graph acyclicity property). The norm-conflict claim is cross-checked
/// by Z3 and cvc5 (both over the one SMT target) and by Lean; the
/// priority-acyclicity claim by Soufflé and Alloy. The remaining toy targets
/// (decision-table, MaxSMT repair, defeasible/Event-Calculus ASP, TLA+, SHACL)
/// are each checked by a single backend, so they form no portfolio.
const PORTFOLIO_CLAIMS: &[ClaimSpec] = &[
    ClaimSpec {
        claim_id: "conflict_norm_bl_contradiction",
        members: &[
            ClaimMember {
                artifact_path: "logic/smt/norm_conflict.smt2",
                established_status: VerdictStatus::Unsat,
            },
            ClaimMember {
                artifact_path: "lean/Ckc/NormConflict.lean",
                established_status: VerdictStatus::KernelChecked,
            },
        ],
    },
    ClaimSpec {
        claim_id: "priority_acyclicity",
        members: &[
            ClaimMember {
                artifact_path: "logic/datalog/priority.dl",
                established_status: VerdictStatus::EmptyRelation,
            },
            ClaimMember {
                artifact_path: "logic/alloy/Priority.als",
                established_status: VerdictStatus::NoCounterexample,
            },
        ],
    },
];

/// One claim on which ≥2 independent backends agree (SPEC 12.2 C5-Portfolio).
/// `backends` is the sorted, deduplicated set of agreeing solvers; `agreed_result`
/// is the shared conclusion (`established` when every backend confirms the claim);
/// `certificate_class` is always `C5-Portfolio`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AgreementRecord {
    pub claim_id: String,
    pub backends: Vec<SolverId>,
    pub agreed_result: String,
    pub certificate_class: CertificateClass,
}

/// Cross-check the recorded verifier `outcomes` claim by claim (SPEC 13). For
/// each [`PORTFOLIO_CLAIMS`] entry with ≥2 matching outcomes, every backend
/// either establishes the claim (its recorded status equals the member's
/// expected status) or it does not; a shared sense yields one [`AgreementRecord`]
/// and a split yields one `backend_disagreement` [`CompileDiagnostic`]. Returns
/// `(agreements, disagreements)`; both stay empty for a claim with fewer than two
/// recorded backends.
pub fn portfolio_check(
    outcomes: &[VerifierOutcome],
) -> (Vec<AgreementRecord>, Vec<CompileDiagnostic>) {
    let mut agreements = Vec::new();
    let mut diagnostics = Vec::new();

    for claim in PORTFOLIO_CLAIMS {
        // Outcomes whose target artifact belongs to this claim, each paired with
        // the status that establishes the claim for that backend.
        let group: Vec<(&VerifierOutcome, VerdictStatus)> = outcomes
            .iter()
            .filter_map(|outcome| {
                claim
                    .members
                    .iter()
                    .find(|member| member.artifact_path == outcome.artifact_path)
                    .map(|member| (outcome, member.established_status))
            })
            .collect();

        // A portfolio needs ≥2 independent verdicts to cross-check.
        if group.len() < 2 {
            continue;
        }

        // Each backend establishes the claim or does not; the backends agree
        // when they share one sense, and a split is a backend disagreement.
        let established: Vec<bool> = group
            .iter()
            .map(|(outcome, expected)| outcome.status == *expected)
            .collect();
        let first = established[0];

        if established.iter().all(|&e| e == first) {
            let mut backends: Vec<SolverId> =
                group.iter().map(|(outcome, _)| outcome.solver).collect();
            backends.sort_by_key(|solver| wire_token(*solver));
            backends.dedup();
            let agreed_result = if first { "established" } else { "refuted" };
            agreements.push(AgreementRecord {
                claim_id: claim.claim_id.to_string(),
                backends,
                agreed_result: agreed_result.to_string(),
                certificate_class: CertificateClass::C5Portfolio,
            });
        } else {
            let conflicting: Vec<VerifierOutcome> = group
                .iter()
                .map(|(outcome, _)| (*outcome).clone())
                .collect();
            diagnostics.push(disagreement_diagnostic(claim.claim_id, &conflicting));
        }
    }

    (agreements, diagnostics)
}

/// Build the `backend_disagreement` diagnostic for a claim whose backends
/// returned contradictory verdicts (SPEC 14 first-class disagreement reporting,
/// 15.1 compiler-backend disagreement). The bilingual messages name the claim and
/// a deterministic `<solver>:<status>` summary of the conflicting verdicts.
/// `source_span_ids` is empty by design: the recorded oracle keys verdicts by
/// target artifact, so span attribution for this claim is recovered downstream
/// through the witnesses and certificate graph that share `claim_id`.
fn disagreement_diagnostic(claim_id: &str, conflicting: &[VerifierOutcome]) -> CompileDiagnostic {
    let mut verdicts: Vec<String> = conflicting
        .iter()
        .map(|outcome| {
            format!(
                "{}:{}",
                wire_token(outcome.solver),
                wire_token(outcome.status)
            )
        })
        .collect();
    verdicts.sort();
    verdicts.dedup();
    let summary = verdicts.join(", ");

    CompileDiagnostic {
        code: "backend_disagreement".to_string(),
        message_en: format!(
            "Independent backends returned contradictory verdicts for claim {claim_id}: {summary}. Review the formalization and solver encodings to reconcile them."
        ),
        message_ja: format!(
            "クレーム {claim_id} について独立したバックエンドが矛盾する判定を返しました：{summary}。整合のため形式化とソルバーのエンコードを確認してください。"
        ),
        source_span_ids: Vec::new(),
    }
}
