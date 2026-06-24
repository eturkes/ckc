//! SPEC §8.3 verify processing_stage, interpretation half — completes `verify` over
//! the adapter: parse solver replies into verdicts, satisfying_examples, and §6
//! categories, assemble validated [`VerifierResult`]s, and drive one
//! compiled group's two-query plan.
//!
//! §6 verifier-adapter requirements: parse the verdict token and result
//! s-expressions, normalize core tokens (strip `|…|`) to Ids, and record
//! cores as canonical sets sorted by canonical_sort_key — on that
//! normalized form, set-based comparison is plain equality. The satisfying_example
//! model on a sat Q1 rides byte-exact as the solver printed it.
//!
//! Observed z3 shape (4.13, live-probed): the §8.6 query texts end in a
//! static retrieval command, so on the verdicts that make its satisfying_example
//! unavailable — Q1 unsat (`get-model`), Q2 sat (`get-unsat-core`) — z3
//! prints the true verdict, then `(error "... is not available")`, and
//! exits 1. The leading verdict token therefore outranks the exit fate
//! wherever it parses, and the [`QueryRole`] decides which satisfying_example the
//! reply owes; the benign retrieval error on a satisfying_example-free path is
//! expected output, never a diagnostic. z3 also prints core symbols bare
//! when they lex as simple symbols (dots qualify), so pipe-stripping is
//! conditional per token.

use std::time::Duration;

use ckc_core::{DiagnosticCode, DiagnosticRecord, Id, Outcome, SolverIdentity, StringPolicy};

use crate::verify::leading_verdict;
use crate::{
    CompiledArtifact, RunOutcome, SolverRun, SolverVerdict, VerifierCategory, VerifierResult,
    Z3Adapter,
};

/// Cap on reply-stream excerpts quoted into diagnostic detail text.
const STREAM_HEAD_CHARS: usize = 160;

/// SPEC §6 role of one planned query inside its contradiction-query pair —
/// the role decides the category mapping and the satisfying_example the reply owes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryRole {
    /// Q1 `context_overlap`: sat records the overlap satisfying_example model; unsat
    /// closes the pair as the documented no-conflict result (§6).
    ContextOverlap,
    /// Q2 `deontic_consistency`: unsat is the semantic contradiction, its
    /// core naming the contributing assertions; sat documents no conflict.
    DeonticConsistency,
}

/// SPEC §8.3 verify-processing_stage core over one compiled group: invoke the adapter
/// per planned query, each under `budget` wall-clock time, and assemble
/// results in plan order — pair k's `context_overlap` first, its
/// `deontic_consistency` only when Q1 answered sat (§6: Q2 runs for pairs
/// with a sat Q1; every other Q1 fate leaves the pair closed as the
/// documented no-conflict result or undecided, with no Q2 result).
///
/// `artifact` must satisfy [`CompiledArtifact::validate`]; bodies out of
/// plan order are a caller bug and panic, like emitter slot order.
pub fn verify(
    adapter: &Z3Adapter,
    artifact: &CompiledArtifact,
    budget: Duration,
) -> Vec<VerifierResult> {
    assert_eq!(
        artifact.query_bodies.len(),
        2 * artifact.solver_query_plan.len(),
        "validated artifacts carry exactly two bodies per planned pair"
    );
    let identity = adapter.identity();
    let mut results = Vec::new();
    for (k, pair) in artifact.solver_query_plan.iter().enumerate() {
        let q1 = &artifact.query_bodies[2 * k];
        let q2 = &artifact.query_bodies[2 * k + 1];
        assert_eq!(
            q1.query_id, pair.context_overlap_query_id,
            "bodies follow plan order: pair k slots 2k/2k+1"
        );
        assert_eq!(
            q2.query_id, pair.deontic_consistency_query_id,
            "bodies follow plan order: pair k slots 2k/2k+1"
        );
        let run = adapter.invoke(&q1.body, budget);
        let r1 = assemble_result(&q1.query_id, QueryRole::ContextOverlap, &run, identity);
        let q1_sat = r1.verdict == Some(SolverVerdict::Sat);
        results.push(r1);
        if q1_sat {
            let run = adapter.invoke(&q2.body, budget);
            results.push(assemble_result(
                &q2.query_id,
                QueryRole::DeonticConsistency,
                &run,
                identity,
            ));
        }
    }
    results
}

/// One solver run interpreted into a [`VerifierResult`] that passes
/// [`VerifierResult::validate`] by construction.
///
/// §6 category mapping, role-aware: Q2 unsat → `semantic_contradiction`
/// with the normalized core; Q2 sat → `semantic_no_conflict`; Q1 sat →
/// `semantic_no_conflict` with the satisfying_example model recorded; Q1 unsat →
/// `semantic_no_conflict`, the documented-null path; unknown stays
/// `unknown` — raw `sat`/`unsat`/`unknown`/`timeout` tokens preserved
/// distinctly in `verdict`. Runs with no verdict token map to the failure
/// categories: a solver `(error …)` reply is `target_syntax_failure`,
/// anything else `solver_execution_failure`; a budget expiry is `unknown`
/// under the budget-minted `timeout` token. Every failure, unknown, or
/// missing-satisfying_example path mints one §7.4 diagnostic; an expected benign
/// retrieval error mints none.
pub fn assemble_result(
    query_id: &Id,
    role: QueryRole,
    run: &SolverRun,
    identity: &SolverIdentity,
) -> VerifierResult {
    let mut result = VerifierResult {
        query_id: query_id.clone(),
        category: VerifierCategory::SolverExecutionFailure,
        verdict: None,
        model: None,
        unsat_core: None,
        solver_identity: identity.clone(),
        diagnostics: vec![],
    };
    match &run.outcome {
        RunOutcome::SpawnFailure { error } => {
            result.diagnostics.push(diagnostic(
                DiagnosticCode::SolverExecutionFailure,
                Outcome::Invalid,
                query_id,
                &format!("solver failed to spawn: {error}"),
            ));
            return result;
        }
        RunOutcome::Timeout => {
            result.category = VerifierCategory::Unknown;
            result.verdict = Some(SolverVerdict::Timeout);
            result.diagnostics.push(diagnostic(
                DiagnosticCode::SolverTimeout,
                Outcome::Residual,
                query_id,
                "per-query wall-clock budget expired before a verdict",
            ));
            return result;
        }
        RunOutcome::Completed { .. } | RunOutcome::ExitFailure { .. } => {}
    }
    let (verdict, reply) = split_verdict(&run.stdout);
    match verdict {
        None => {
            // The reply decides the failure kind: a solver error
            // s-expression marks rejection of the query text, anything
            // else a run that ended without answering.
            if let Some(error_line) = error_line(run) {
                result.category = VerifierCategory::TargetSyntaxFailure;
                result.diagnostics.push(diagnostic(
                    DiagnosticCode::TargetParseError,
                    Outcome::Invalid,
                    query_id,
                    &format!("solver rejected the query text: {error_line}"),
                ));
            } else {
                result.diagnostics.push(diagnostic(
                    DiagnosticCode::SolverExecutionFailure,
                    Outcome::Invalid,
                    query_id,
                    &format!(
                        "no verdict in solver reply ({}): {:?}",
                        exit_fate(&run.outcome),
                        head(&run.stdout)
                    ),
                ));
            }
        }
        Some(SolverVerdict::Unknown) => {
            result.category = VerifierCategory::Unknown;
            result.verdict = Some(SolverVerdict::Unknown);
            result.diagnostics.push(diagnostic(
                DiagnosticCode::SolverUnknown,
                Outcome::Residual,
                query_id,
                "solver returned unknown",
            ));
        }
        Some(SolverVerdict::Sat) => {
            result.verdict = Some(SolverVerdict::Sat);
            result.category = VerifierCategory::SemanticNoConflict;
            if role == QueryRole::ContextOverlap {
                match sexpr_span(reply) {
                    Some((start, end)) => result.model = Some(reply[start..end].to_owned()),
                    None => result.diagnostics.push(diagnostic(
                        DiagnosticCode::SolverExecutionFailure,
                        Outcome::Invalid,
                        query_id,
                        "sat reply carries no parseable satisfying_example model",
                    )),
                }
            }
        }
        Some(SolverVerdict::Unsat) => {
            result.verdict = Some(SolverVerdict::Unsat);
            match role {
                QueryRole::ContextOverlap => {
                    result.category = VerifierCategory::SemanticNoConflict;
                }
                QueryRole::DeonticConsistency => {
                    result.category = VerifierCategory::SemanticContradiction;
                    match parse_core(reply) {
                        Some(core) if !core.is_empty() => result.unsat_core = Some(core),
                        _ => result.diagnostics.push(diagnostic(
                            DiagnosticCode::SolverExecutionFailure,
                            Outcome::Invalid,
                            query_id,
                            "unsat reply carries no parseable unsat core",
                        )),
                    }
                }
            }
        }
        Some(SolverVerdict::Timeout) => {
            unreachable!("leading_verdict never parses timeout (§6: budget-minted only)")
        }
    }
    result
}

/// Leading verdict token and the reply remainder after its line (§6: the
/// adapter parses the verdict token, then the result s-expressions).
fn split_verdict(stdout: &str) -> (Option<SolverVerdict>, &str) {
    let Some(first_line) = stdout.lines().next() else {
        return (None, "");
    };
    (leading_verdict(stdout), &stdout[first_line.len()..])
}

/// First line of either reply stream carrying a solver `(error` marker,
/// capped for diagnostic text; z3 prints errors to stdout, wrappers may
/// use stderr.
fn error_line(run: &SolverRun) -> Option<String> {
    [&run.stdout, &run.stderr]
        .into_iter()
        .flat_map(|stream| stream.lines())
        .find(|line| line.contains("(error"))
        .map(head)
}

/// Exit fate of a verdict-less run for diagnostic text.
fn exit_fate(outcome: &RunOutcome) -> String {
    match outcome {
        RunOutcome::Completed { .. } => "exit status 0".to_owned(),
        RunOutcome::ExitFailure { code: Some(code) } => format!("exit code {code}"),
        RunOutcome::ExitFailure { code: None } => "signal-terminated".to_owned(),
        RunOutcome::Timeout | RunOutcome::SpawnFailure { .. } => {
            unreachable!("handled before solver-result parsing")
        }
    }
}

/// First line of `text`, capped to [`STREAM_HEAD_CHARS`] characters.
fn head(text: &str) -> String {
    text.lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(STREAM_HEAD_CHARS)
        .collect()
}

/// One §7.4 verify-processing_stage diagnostic: `code` under `outcome`, detail
/// normalized by the diagnostic-text policy, payload naming the query.
fn diagnostic(
    code: DiagnosticCode,
    outcome: Outcome,
    query_id: &Id,
    detail: &str,
) -> DiagnosticRecord {
    let detail = StringPolicy::DiagnosticText
        .normalize(detail)
        .expect("diagnostic_text is infallible");
    DiagnosticRecord {
        code,
        outcome,
        payload: vec![
            (static_id("detail"), detail),
            (static_id("query"), query_id.to_string()),
        ],
        region_ids: vec![],
        artifact_hashes: vec![],
    }
}

/// An [`Id`] from a static, known-valid identifier.
fn static_id(s: &str) -> Id {
    Id::new(s).expect("static ids are valid")
}

/// Lexing state for the s-expression reader: pipe-quoted symbols and
/// string literals shield their bytes from paren counting (SMT-LIB 2
/// lexing; the `""` escape exits and re-enters [`Lex::Str`], shielding
/// correctly byte-wise).
enum Lex {
    Plain,
    Pipe,
    Str,
}

/// Byte span (start, end-exclusive) of the first balanced s-expression in
/// `text`, anything before its `(` skipped; `None` when no expression
/// opens or the text ends unbalanced. The span is the byte-exact satisfying_example
/// slice a sat Q1 records as its model (§6).
fn sexpr_span(text: &str) -> Option<(usize, usize)> {
    let start = text.find('(')?;
    let mut depth = 0usize;
    let mut state = Lex::Plain;
    for (i, b) in text.bytes().enumerate().skip(start) {
        match state {
            Lex::Pipe => {
                if b == b'|' {
                    state = Lex::Plain;
                }
            }
            Lex::Str => {
                if b == b'"' {
                    state = Lex::Plain;
                }
            }
            Lex::Plain => match b {
                b'|' => state = Lex::Pipe,
                b'"' => state = Lex::Str,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some((start, i + 1));
                    }
                }
                _ => {}
            },
        }
    }
    None
}

/// Atom tokens of the first balanced s-expression in `text` when it nests
/// nothing — the `get-unsat-core` reply shape `(tok tok ...)` — pipe
/// quoting kept per token; `None` when no expression parses, an element
/// nests, or a pipe quote never closes.
fn flat_list_tokens(text: &str) -> Option<Vec<&str>> {
    let (start, end) = sexpr_span(text)?;
    let inner = &text[start + 1..end - 1];
    let bytes = inner.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i] == b'(' {
            return None;
        }
        let tok_start = i;
        if bytes[i] == b'|' {
            i += 1;
            while i < bytes.len() && bytes[i] != b'|' {
                i += 1;
            }
            if i == bytes.len() {
                return None;
            }
            i += 1;
        } else {
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'(' {
                i += 1;
            }
        }
        tokens.push(&inner[tok_start..i]);
    }
    Some(tokens)
}

/// Normalize a `get-unsat-core` reply (§6): the flat token list, each
/// token pipe-stripped when quoted, to a canonical Id set sorted by
/// canonical_sort_key (Id byte order) and duplicate-free. `None` when no
/// flat list parses or a token is no Id; an empty list comes back
/// `Some(empty)` and the caller treats it as a missing satisfying_example — an unsat
/// over named assertions always cores at least one name.
fn parse_core(reply: &str) -> Option<Vec<Id>> {
    let tokens = flat_list_tokens(reply)?;
    let mut ids = Vec::with_capacity(tokens.len());
    for token in tokens {
        let bare = token
            .strip_prefix('|')
            .and_then(|t| t.strip_suffix('|'))
            .unwrap_or(token);
        ids.push(Id::new(bare).ok()?);
    }
    ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    ids.dedup();
    Some(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile;
    use ckc_core::{
        Action, ContextAtom, ContextConjunct, ContextExpr, Direction, FormalIr, NormIr,
        NormativeRule, QuantityInterval, Strength,
    };

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn z3_identity() -> SolverIdentity {
        SolverIdentity {
            solver_id: id("z3"),
            version: "4.13.4".to_owned(),
        }
    }

    /// A run that completed (exit 0) with `stdout`, its verdict slot filled
    /// the way the runner fills it.
    fn completed(stdout: &str) -> SolverRun {
        SolverRun {
            outcome: RunOutcome::Completed {
                verdict: leading_verdict(stdout),
            },
            stdout: stdout.to_owned(),
            stderr: String::new(),
        }
    }

    /// A run that exited 1 with `stdout` — the live-probed fate of a §8.6
    /// static retrieval command whose satisfying_example the verdict made unavailable.
    fn exit1(stdout: &str) -> SolverRun {
        SolverRun {
            outcome: RunOutcome::ExitFailure { code: Some(1) },
            stdout: stdout.to_owned(),
            stderr: String::new(),
        }
    }

    fn assemble(role: QueryRole, run: &SolverRun) -> VerifierResult {
        let result = assemble_result(
            &id("q.m1_conflict.pair1.overlap"),
            role,
            run,
            &z3_identity(),
        );
        assert_eq!(
            result.validate(),
            Ok(()),
            "assembly validates by construction"
        );
        result
    }

    // S-expression reader: spans over the observed model shape, quoting
    // shields, prefix skip, unbalanced and absent expressions.
    #[test]
    fn sexpr_reader_spans_and_tokens() {
        let model = "(\n  (define-fun q.age_years () Real\n    18.0)\n)";
        let reply = format!("\n{model}\n");
        let (start, end) = sexpr_span(&reply).unwrap();
        assert_eq!(&reply[start..end], model);
        assert_eq!(sexpr_span("(a |b)| c)"), Some((0, 10)));
        assert_eq!(sexpr_span("(\"a)b\" c)"), Some((0, 9)));
        assert_eq!(sexpr_span("noise (x)"), Some((6, 9)));
        assert_eq!(sexpr_span("(a (b)"), None);
        assert_eq!(sexpr_span("sat"), None);
        assert_eq!(
            flat_list_tokens("\n(a.x |a.y z| b)\n"),
            Some(vec!["a.x", "|a.y z|", "b"])
        );
        assert_eq!(flat_list_tokens("(a (b))"), None);
        assert_eq!(flat_list_tokens("none"), None);
    }

    // §6 core normalization: bare tokens (the observed z3 shape — dots lex
    // as simple symbols), pipe-stripping when quoted, canonical sort,
    // dedup; the normalized form makes set comparison plain equality.
    #[test]
    fn core_normalization_pins_worked_set() {
        let want = vec![
            id("a.test_source.m1_guideline_a.rule.0"),
            id("a.test_source.m1_guideline_b.rule.0"),
        ];
        assert_eq!(
            parse_core(
                "\n(a.test_source.m1_guideline_b.rule.0 a.test_source.m1_guideline_a.rule.0)\n"
            ),
            Some(want.clone())
        );
        assert_eq!(
            parse_core(
                "(|a.test_source.m1_guideline_a.rule.0| |a.test_source.m1_guideline_b.rule.0|)"
            ),
            Some(want)
        );
        assert_eq!(parse_core("(a.r1 a.r1)"), Some(vec![id("a.r1")]));
        assert_eq!(parse_core("()"), Some(vec![]));
        assert_eq!(parse_core("(a.r1 |a b|)"), None);
        assert_eq!(parse_core("no list"), None);
    }

    // Q1 sat records the satisfying_example model byte-exact: the balanced
    // s-expression span, multi-line indentation and all, trailing newline
    // outside.
    #[test]
    fn q1_sat_records_satisfying_example_model() {
        let stdout = concat!(
            "sat\n",
            "(\n",
            "  (define-fun cond.sepsis () Bool\n",
            "    true)\n",
            "  (define-fun q.age_years () Real\n",
            "    18.0)\n",
            ")\n",
        );
        let result = assemble(QueryRole::ContextOverlap, &completed(stdout));
        assert_eq!(result.category, VerifierCategory::SemanticNoConflict);
        assert_eq!(result.verdict, Some(SolverVerdict::Sat));
        assert_eq!(
            result.model.as_deref(),
            Some(concat!(
                "(\n",
                "  (define-fun cond.sepsis () Bool\n",
                "    true)\n",
                "  (define-fun q.age_years () Real\n",
                "    18.0)\n",
                ")",
            ))
        );
        assert_eq!(result.unsat_core, None);
        assert_eq!(result.diagnostics, vec![]);
    }

    // Q1 unsat closes the pair as the documented-null path of
    // semantic_no_conflict; the live-probed benign retrieval error (exit 1,
    // `(error "... model is not available")`) is expected output, never a
    // diagnostic.
    #[test]
    fn q1_unsat_closes_documented_no_conflict() {
        let result = assemble(
            QueryRole::ContextOverlap,
            &exit1("unsat\n(error \"line 7 column 10: model is not available\")\n"),
        );
        assert_eq!(result.category, VerifierCategory::SemanticNoConflict);
        assert_eq!(result.verdict, Some(SolverVerdict::Unsat));
        assert_eq!(result.model, None);
        assert_eq!(result.unsat_core, None);
        assert_eq!(result.diagnostics, vec![]);
    }

    // Q2 sat documents no conflict; its role owes no satisfying_example, so the
    // benign `unsat core is not available` error (exit 1) mints nothing.
    #[test]
    fn q2_sat_documents_no_conflict() {
        let result = assemble(
            QueryRole::DeonticConsistency,
            &exit1("sat\n(error \"line 8 column 15: unsat core is not available\")\n"),
        );
        assert_eq!(result.category, VerifierCategory::SemanticNoConflict);
        assert_eq!(result.verdict, Some(SolverVerdict::Sat));
        assert_eq!(result.model, None);
        assert_eq!(result.unsat_core, None);
        assert_eq!(result.diagnostics, vec![]);
    }

    // Q2 unsat is the semantic contradiction, its core normalized to the
    // §8.6 canonical set whatever order or quoting the solver printed.
    #[test]
    fn q2_unsat_records_canonical_core() {
        let want = [
            id("a.test_source.m1_guideline_a.rule.0"),
            id("a.test_source.m1_guideline_b.rule.0"),
        ];
        for stdout in [
            "unsat\n(a.test_source.m1_guideline_a.rule.0 a.test_source.m1_guideline_b.rule.0)\n",
            "unsat\n(|a.test_source.m1_guideline_b.rule.0| |a.test_source.m1_guideline_a.rule.0|)\n",
        ] {
            let result = assemble(QueryRole::DeonticConsistency, &completed(stdout));
            assert_eq!(result.category, VerifierCategory::SemanticContradiction);
            assert_eq!(result.verdict, Some(SolverVerdict::Unsat));
            assert_eq!(result.unsat_core.as_deref(), Some(&want[..]));
            assert_eq!(result.model, None);
            assert_eq!(result.diagnostics, vec![]);
        }
    }

    // Unknown stays unknown under both raw tokens, preserved distinctly:
    // a parsed `unknown` against the budget-minted `timeout`, each with
    // its own §7.4 diagnostic.
    #[test]
    fn unknown_and_timeout_stay_distinct() {
        let unknown = assemble(QueryRole::ContextOverlap, &completed("unknown\n"));
        assert_eq!(unknown.category, VerifierCategory::Unknown);
        assert_eq!(unknown.verdict, Some(SolverVerdict::Unknown));
        assert_eq!(unknown.diagnostics.len(), 1);
        assert_eq!(unknown.diagnostics[0].code, DiagnosticCode::SolverUnknown);
        assert_eq!(unknown.diagnostics[0].outcome, Outcome::Residual);

        let timeout = assemble(
            QueryRole::DeonticConsistency,
            &SolverRun {
                outcome: RunOutcome::Timeout,
                stdout: String::new(),
                stderr: String::new(),
            },
        );
        assert_eq!(timeout.category, VerifierCategory::Unknown);
        assert_eq!(timeout.verdict, Some(SolverVerdict::Timeout));
        assert_eq!(timeout.diagnostics.len(), 1);
        assert_eq!(timeout.diagnostics[0].code, DiagnosticCode::SolverTimeout);
        assert_eq!(timeout.diagnostics[0].outcome, Outcome::Residual);
    }

    // Verdict-less runs map to the failure categories: a solver `(error`
    // reply is the target rejecting the query text (the smt-verify.a
    // live-observed undeclared-constant shape), a spawn failure or a bare
    // nonzero/empty reply a failed execution.
    #[test]
    fn failure_paths_map_categories() {
        let syntax = assemble(
            QueryRole::ContextOverlap,
            &exit1("(error \"line 1 column 2: unknown constant p\")\n"),
        );
        assert_eq!(syntax.category, VerifierCategory::TargetSyntaxFailure);
        assert_eq!(syntax.verdict, None);
        assert_eq!(syntax.diagnostics.len(), 1);
        assert_eq!(syntax.diagnostics[0].code, DiagnosticCode::TargetParseError);
        assert_eq!(syntax.diagnostics[0].outcome, Outcome::Invalid);

        let spawn = assemble(
            QueryRole::ContextOverlap,
            &SolverRun {
                outcome: RunOutcome::SpawnFailure {
                    error: "No such file or directory (os error 2)".to_owned(),
                },
                stdout: String::new(),
                stderr: String::new(),
            },
        );
        assert_eq!(spawn.category, VerifierCategory::SolverExecutionFailure);
        assert_eq!(spawn.verdict, None);
        assert_eq!(
            spawn.diagnostics[0].code,
            DiagnosticCode::SolverExecutionFailure
        );

        let signal = assemble(
            QueryRole::DeonticConsistency,
            &SolverRun {
                outcome: RunOutcome::ExitFailure { code: None },
                stdout: String::new(),
                stderr: String::new(),
            },
        );
        assert_eq!(signal.category, VerifierCategory::SolverExecutionFailure);
        assert!(
            signal.diagnostics[0].payload[0]
                .1
                .contains("signal-terminated"),
            "detail {:?}",
            signal.diagnostics[0].payload[0].1
        );

        let unsupported = assemble(QueryRole::ContextOverlap, &completed("unsupported\n"));
        assert_eq!(
            unsupported.category,
            VerifierCategory::SolverExecutionFailure
        );
        assert!(
            unsupported.diagnostics[0].payload[0]
                .1
                .contains("exit status 0"),
            "detail {:?}",
            unsupported.diagnostics[0].payload[0].1
        );
    }

    // A verdict whose owed satisfying_example is missing or garbled keeps its §6
    // category — the verdict is the solver's answer — and mints one
    // invalid-outcome diagnostic for the broken evidence chain.
    #[test]
    fn degraded_satisfying_example_paths_diagnosed() {
        let no_model = assemble(QueryRole::ContextOverlap, &completed("sat\n"));
        assert_eq!(no_model.category, VerifierCategory::SemanticNoConflict);
        assert_eq!(no_model.model, None);
        assert_eq!(no_model.diagnostics.len(), 1);
        assert_eq!(no_model.diagnostics[0].outcome, Outcome::Invalid);

        for stdout in ["unsat\n(a.r1 (nested))\n", "unsat\n()\n", "unsat\n"] {
            let result = assemble(QueryRole::DeonticConsistency, &completed(stdout));
            assert_eq!(result.category, VerifierCategory::SemanticContradiction);
            assert_eq!(result.unsat_core, None);
            assert_eq!(result.diagnostics.len(), 1, "stdout {stdout:?}");
            assert_eq!(result.diagnostics[0].outcome, Outcome::Invalid);
        }
    }

    /// One-conjunct DNF over `atoms` in canonical stored order.
    fn dnf1(all: Vec<ContextAtom>) -> ContextExpr {
        ContextExpr {
            any: vec![ContextConjunct { all }],
        }
    }

    fn concept(c: &str) -> ContextAtom {
        ContextAtom::Concept(id(c))
    }

    /// `q.age_years` one-sided interval atom at 18: `ge` (成人) or `lt`
    /// (小児).
    fn age(adult: bool) -> ContextAtom {
        ContextAtom::Interval(QuantityInterval {
            var: id("q.age_years"),
            ge: adult.then_some(18),
            gt: None,
            le: None,
            lt: (!adult).then_some(18),
        })
    }

    /// A §8.6-shaped NormativeRule over the shared administer-abx_a action.
    fn nr(
        rule_id: &str,
        direction: Direction,
        context: ContextExpr,
        regions: &[&str],
    ) -> NormativeRule {
        NormativeRule {
            rule_id: id(rule_id),
            context,
            direction,
            action: Action::new(id("act.administer"), id("drug.abx_a")),
            strength: Strength::Strong,
            source_region_ids: regions.iter().map(|r| id(r)).collect(),
            certainty: None,
            exception_refs: vec![],
        }
    }

    /// One document's layer pair for [`compile`]: NormIR from `rules`,
    /// FormalIR derived per §5.
    fn layers(rules: Vec<NormativeRule>) -> (FormalIr, NormIr) {
        let norm = NormIr { rules };
        (FormalIr::derive(&norm), norm)
    }

    /// docA rule.0 (§8.6): for administer abx_a under
    /// sepsis ∧ ¬renal_severe ∧ age ≥ 18.
    fn rule_a() -> NormativeRule {
        nr(
            "test_source.m1_guideline_a.rule.0",
            Direction::For,
            dnf1(vec![
                concept("cond.sepsis"),
                ContextAtom::ConceptNegated(id("cond.renal_severe")),
                age(true),
            ]),
            &["r.2", "r.3"],
        )
    }

    /// docB rule.0 (§8.6): contraindicate the same action under
    /// pregnancy ∧ sepsis ∧ age ≥ 18.
    fn rule_b() -> NormativeRule {
        nr(
            "test_source.m1_guideline_b.rule.0",
            Direction::Contraindicate,
            dnf1(vec![
                concept("cond.pregnancy"),
                concept("cond.sepsis"),
                age(true),
            ]),
            &["r.2"],
        )
    }

    /// Control rule.0 (§8.2): contraindicate the same action under
    /// sepsis ∧ age < 18 — interval disjoint with docA's.
    fn rule_control() -> NormativeRule {
        nr(
            "test_source.m1_control.rule.0",
            Direction::Contraindicate,
            dnf1(vec![concept("cond.sepsis"), age(false)]),
            &["r.2"],
        )
    }

    // §8.6 worked thread end-to-end on live z3: plan + emit through
    // compile, verify over the conflict group — Q1 sat with the overlap
    // satisfying_example model recorded, Q2 unsat with the expected cross-document
    // core as the canonical set, both stamped with the probed identity.
    #[test]
    fn live_worked_pair_full_verify() {
        let (fa, na) = layers(vec![rule_a()]);
        let (fb, nb) = layers(vec![rule_b()]);
        let artifact = compile(&id("group.m1_conflict"), [(&fa, &na), (&fb, &nb)]);
        let adapter = Z3Adapter::new().unwrap();
        let results = verify(&adapter, &artifact, Duration::from_secs(30));

        assert_eq!(results.len(), 2, "sat Q1 runs Q2");
        let q1 = &results[0];
        assert_eq!(q1.query_id, id("q.m1_conflict.pair1.overlap"));
        assert_eq!(q1.category, VerifierCategory::SemanticNoConflict);
        assert_eq!(q1.verdict, Some(SolverVerdict::Sat));
        let model = q1
            .model
            .as_deref()
            .expect("Q1 sat records the satisfying_example model");
        assert!(model.contains("define-fun"), "model {model:?}");
        assert!(model.contains("q.age_years"), "model {model:?}");
        assert_eq!(q1.unsat_core, None);
        assert_eq!(q1.diagnostics, vec![]);
        assert_eq!(q1.validate(), Ok(()));
        assert_eq!(&q1.solver_identity, adapter.identity());

        let q2 = &results[1];
        assert_eq!(q2.query_id, id("q.m1_conflict.pair1.deontic"));
        assert_eq!(q2.category, VerifierCategory::SemanticContradiction);
        assert_eq!(q2.verdict, Some(SolverVerdict::Unsat));
        assert_eq!(
            q2.unsat_core.as_deref(),
            Some(
                &[
                    id("a.test_source.m1_guideline_a.rule.0"),
                    id("a.test_source.m1_guideline_b.rule.0"),
                ][..]
            )
        );
        assert_eq!(q2.model, None);
        assert_eq!(q2.diagnostics, vec![]);
        assert_eq!(q2.validate(), Ok(()));
        assert_eq!(&q2.solver_identity, adapter.identity());
    }

    // §8.6 control group on live z3: disjoint age intervals leave Q1
    // unsat, closing the pair as the documented no-conflict result — one clean
    // semantic_no_conflict, no Q2 result.
    #[test]
    fn live_control_pair_closes_no_conflict() {
        let (fa, na) = layers(vec![rule_a()]);
        let (fc, nc) = layers(vec![rule_control()]);
        let artifact = compile(&id("group.m1_no_conflict"), [(&fa, &na), (&fc, &nc)]);
        let adapter = Z3Adapter::new().unwrap();
        let results = verify(&adapter, &artifact, Duration::from_secs(30));

        assert_eq!(results.len(), 1, "unsat Q1 closes the pair without Q2");
        let q1 = &results[0];
        assert_eq!(q1.query_id, id("q.m1_no_conflict.pair1.overlap"));
        assert_eq!(q1.category, VerifierCategory::SemanticNoConflict);
        assert_eq!(q1.verdict, Some(SolverVerdict::Unsat));
        assert_eq!(q1.model, None);
        assert_eq!(q1.unsat_core, None);
        assert_eq!(q1.diagnostics, vec![]);
        assert_eq!(q1.validate(), Ok(()));
    }
}
