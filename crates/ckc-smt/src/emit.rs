//! SPEC §6/§8.6 deterministic SMT-LIB emission — query text for the two §6
//! contradiction queries of one planned pair
//! ([`plan_queries`](crate::plan_queries) mints the pair and its query ids;
//! smt-emit.3b assembles [`CompiledArtifact`](crate::CompiledArtifact)s
//! from these bodies).
//!
//! Byte rules (§6 SMT profile, pinned by the §8.6 listings): one
//! s-expression command per line, no comments, the file ending in a
//! newline; command order set-logic, :print-success false, the per-query
//! produce option, declarations sorted by symbol bytes one per line, named
//! assertions in pair order, check-sat, then the result command. Guard
//! conjuncts render in stored ContextExpr order (validated artifacts store
//! canonical-set order; emission never re-sorts), a single-disjunct `any`
//! collapses to a bare `and`, and degenerate forms keep SMT-LIB validity:
//! a single-atom conjunct renders bare, an empty conjunct or expression
//! renders `true`, an interval atom contributes one comparison per present
//! bound (ge gt le lt order), and a negative bound renders `(- n)`.

use std::collections::BTreeMap;

use ckc_core::{
    ContextAtom, ContextConjunct, ContextExpr, ContradictionQueryPair, Direction, FormalConstraint,
};

use crate::{QueryBody, SmtLogic};

/// SPEC §6 Q1 context_overlap (QF_LRA): both guarded contexts as
/// `ctx.<rule_id>` named assertions; sat yields the recorded overlap
/// witness, unsat closes the pair as the documented null result. `a`/`b`
/// are the pair's constraints in slot order.
pub fn emit_overlap_query(
    pair: &ContradictionQueryPair,
    a: &FormalConstraint,
    b: &FormalConstraint,
) -> QueryBody {
    check_slots(pair, a, b);
    let mut symbols = BTreeMap::new();
    collect_symbols(&a.context, &mut symbols);
    collect_symbols(&b.context, &mut symbols);
    let mut body = preamble(SmtLogic::QfLra, ":produce-models");
    for (symbol, sort) in symbols {
        body.push_str(&format!("(declare-const |{symbol}| {sort})\n"));
    }
    for constraint in [a, b] {
        body.push_str(&format!(
            "(assert (! {} :named |ctx.{}|))\n",
            context_term(&constraint.context),
            constraint.rule_id
        ));
    }
    body.push_str("(check-sat)\n(get-model)\n");
    QueryBody {
        query_id: pair.context_overlap_query_id.clone(),
        logic: SmtLogic::QfLra,
        body,
    }
}

/// SPEC §6 Q2 deontic_consistency (QF_UF): each constraint's direction as a
/// polarity literal on the shared action key — positive directions bare,
/// against/contraindicating negated — under `a.<rule_id>` names; unsat is
/// the semantic contradiction, its core naming the contributing assertions.
/// `a`/`b` are the pair's constraints in slot order.
pub fn emit_deontic_query(
    pair: &ContradictionQueryPair,
    a: &FormalConstraint,
    b: &FormalConstraint,
) -> QueryBody {
    check_slots(pair, a, b);
    let literal = format!("|pos:{}|", pair.action_key);
    let mut body = preamble(SmtLogic::QfUf, ":produce-unsat-cores");
    body.push_str(&format!("(declare-const {literal} Bool)\n"));
    for constraint in [a, b] {
        // Exhaustive over Direction so a new variant forces a §6
        // polarity-group decision here.
        let polarity = match constraint.direction {
            Direction::For | Direction::Require | Direction::Permit => literal.clone(),
            Direction::Against | Direction::Avoid | Direction::Contraindicate => {
                format!("(not {literal})")
            }
        };
        body.push_str(&format!(
            "(assert (! {polarity} :named |a.{}|))\n",
            constraint.rule_id
        ));
    }
    body.push_str("(check-sat)\n(get-unsat-core)\n");
    QueryBody {
        query_id: pair.deontic_consistency_query_id.clone(),
        logic: SmtLogic::QfUf,
        body,
    }
}

/// The slot contract both emitters lean on: `a`/`b` arrive in pair order
/// (`constraint_a_id`, `constraint_b_id`). A mismatch is a caller bug,
/// never data.
fn check_slots(pair: &ContradictionQueryPair, a: &FormalConstraint, b: &FormalConstraint) {
    assert_eq!(
        (a.constraint_id.as_str(), b.constraint_id.as_str()),
        (pair.constraint_a_id.as_str(), pair.constraint_b_id.as_str()),
        "constraints out of pair slot order for {}",
        pair.pair_id
    );
}

/// The three-command §6 file head shared by both queries.
fn preamble(logic: SmtLogic, produce_option: &str) -> String {
    format!(
        "(set-logic {})\n(set-option :print-success false)\n(set-option {produce_option} true)\n",
        logic.smt_token()
    )
}

/// Q1 declaration scan: concepts declare Bool, interval variables Real
/// (the two lexicon pools are disjoint), and BTreeMap keys give the
/// sorted-by-symbol-bytes declaration order.
fn collect_symbols<'c>(context: &'c ContextExpr, symbols: &mut BTreeMap<&'c str, &'static str>) {
    for conjunct in &context.any {
        for atom in &conjunct.all {
            match atom {
                ContextAtom::Concept(c) | ContextAtom::ConceptNegated(c) => {
                    symbols.insert(c.as_str(), "Bool");
                }
                ContextAtom::Interval(q) => {
                    symbols.insert(q.var.as_str(), "Real");
                }
            }
        }
    }
}

/// Render a stored guard: DNF disjuncts in stored order under `or`,
/// collapsing per the module byte rules.
fn context_term(context: &ContextExpr) -> String {
    let mut terms: Vec<String> = context.any.iter().map(conjunct_term).collect();
    match terms.len() {
        0 => "true".to_owned(),
        1 => terms.pop().expect("one term"),
        _ => format!("(or {})", terms.join(" ")),
    }
}

/// Render one conjunct: atom terms in stored order under `and`, collapsing
/// per the module byte rules.
fn conjunct_term(conjunct: &ContextConjunct) -> String {
    let mut args = Vec::new();
    for atom in &conjunct.all {
        atom_args(atom, &mut args);
    }
    match args.len() {
        0 => "true".to_owned(),
        1 => args.pop().expect("one arg"),
        _ => format!("(and {})", args.join(" ")),
    }
}

/// Splice one atom's terms into the enclosing conjunction: concepts as bare
/// or negated literals, an interval as one comparison per present bound in
/// ge gt le lt order.
fn atom_args(atom: &ContextAtom, args: &mut Vec<String>) {
    match atom {
        ContextAtom::Concept(c) => args.push(format!("|{c}|")),
        ContextAtom::ConceptNegated(c) => args.push(format!("(not |{c}|)")),
        ContextAtom::Interval(q) => {
            for (op, bound) in [(">=", q.ge), (">", q.gt), ("<=", q.le), ("<", q.lt)] {
                if let Some(n) = bound {
                    args.push(format!("({op} |{}| {})", q.var, numeral(n)));
                }
            }
        }
    }
}

/// SMT-LIB numeral term for a bound: plain decimal, negatives via unary
/// minus (a bare `-n` token would read as a symbol).
fn numeral(n: i64) -> String {
    if n < 0 {
        format!("(- {})", n.unsigned_abs())
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan_queries;
    use ckc_core::{Action, FormalIr, Id, QuantityInterval, Strength};

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    /// One-conjunct DNF over `atoms` in the given stored order (validated
    /// artifacts store canonical-set order; tests construct it directly).
    fn dnf1(all: Vec<ContextAtom>) -> ContextExpr {
        ContextExpr {
            any: vec![ContextConjunct { all }],
        }
    }

    fn concept(c: &str) -> ContextAtom {
        ContextAtom::Concept(id(c))
    }

    fn interval(
        var: &str,
        ge: Option<i64>,
        gt: Option<i64>,
        le: Option<i64>,
        lt: Option<i64>,
    ) -> ContextAtom {
        ContextAtom::Interval(QuantityInterval {
            var: id(var),
            ge,
            gt,
            le,
            lt,
        })
    }

    /// A §8.6-shaped constraint over the shared administer-abx_a action.
    fn fc(rule_id: &str, direction: Direction, context: ContextExpr) -> FormalConstraint {
        FormalConstraint {
            constraint_id: id(&format!("fc.{rule_id}")),
            rule_id: id(rule_id),
            action: Action::new(id("act.administer"), id("drug.abx_a")),
            context,
            direction,
            strength: Strength::Strong,
            certainty: None,
        }
    }

    /// Plan the one eligible pair the two constraints form, through the
    /// real planner (the smt-emit.3b flow).
    fn plan_pair(
        group: &str,
        a: &FormalConstraint,
        b: &FormalConstraint,
    ) -> ContradictionQueryPair {
        let ir = |c: &FormalConstraint| FormalIr {
            constraints: vec![c.clone()],
            plan: Vec::new(),
        };
        let irs = [ir(a), ir(b)];
        let mut plan = plan_queries(&id(group), &irs);
        assert_eq!(plan.len(), 1, "helpers build one eligible pair");
        plan.remove(0)
    }

    /// docA rule.0 (§8.6): for administer abx_a under
    /// sepsis ∧ ¬renal_severe ∧ age ≥ 18, atoms in canonical stored order.
    fn doc_a() -> FormalConstraint {
        fc(
            "fixture.m1_guideline_a.rule.0",
            Direction::For,
            dnf1(vec![
                concept("cond.sepsis"),
                ContextAtom::ConceptNegated(id("cond.renal_severe")),
                interval("q.age_years", Some(18), None, None, None),
            ]),
        )
    }

    /// docB rule.0 (§8.6): contraindicate the same action under
    /// pregnancy ∧ sepsis ∧ age ≥ 18 (canonical stored order).
    fn doc_b() -> FormalConstraint {
        fc(
            "fixture.m1_guideline_b.rule.0",
            Direction::Contraindicate,
            dnf1(vec![
                concept("cond.pregnancy"),
                concept("cond.sepsis"),
                interval("q.age_years", Some(18), None, None, None),
            ]),
        )
    }

    /// Control rule.0 (§8.2): contraindicate the same action under
    /// sepsis ∧ age < 18 — interval disjoint with docA's.
    fn control() -> FormalConstraint {
        fc(
            "fixture.m1_control.rule.0",
            Direction::Contraindicate,
            dnf1(vec![
                concept("cond.sepsis"),
                interval("q.age_years", None, None, None, Some(18)),
            ]),
        )
    }

    /// §8.6 Q1 listing, byte-exact, under the pair's minted query id and
    /// recorded logic.
    #[test]
    fn worked_pair_overlap_pins_spec_listing() {
        let (a, b) = (doc_a(), doc_b());
        let q = emit_overlap_query(&plan_pair("group.m1_conflict", &a, &b), &a, &b);
        assert_eq!(q.query_id, id("q.m1_conflict.pair1.overlap"));
        assert_eq!(q.logic, SmtLogic::QfLra);
        assert_eq!(
            q.body,
            concat!(
                "(set-logic QF_LRA)\n",
                "(set-option :print-success false)\n",
                "(set-option :produce-models true)\n",
                "(declare-const |cond.pregnancy| Bool)\n",
                "(declare-const |cond.renal_severe| Bool)\n",
                "(declare-const |cond.sepsis| Bool)\n",
                "(declare-const |q.age_years| Real)\n",
                "(assert (! (and |cond.sepsis| (not |cond.renal_severe|) (>= |q.age_years| 18)) :named |ctx.fixture.m1_guideline_a.rule.0|))\n",
                "(assert (! (and |cond.pregnancy| |cond.sepsis| (>= |q.age_years| 18)) :named |ctx.fixture.m1_guideline_b.rule.0|))\n",
                "(check-sat)\n",
                "(get-model)\n",
            )
        );
    }

    /// §8.6 Q2 listing, byte-exact: docA's `for` asserts the polarity
    /// literal bare, docB's `contraindicate` negates it.
    #[test]
    fn worked_pair_deontic_pins_spec_listing() {
        let (a, b) = (doc_a(), doc_b());
        let q = emit_deontic_query(&plan_pair("group.m1_conflict", &a, &b), &a, &b);
        assert_eq!(q.query_id, id("q.m1_conflict.pair1.deontic"));
        assert_eq!(q.logic, SmtLogic::QfUf);
        assert_eq!(
            q.body,
            concat!(
                "(set-logic QF_UF)\n",
                "(set-option :print-success false)\n",
                "(set-option :produce-unsat-cores true)\n",
                "(declare-const |pos:act.administer:drug.abx_a| Bool)\n",
                "(assert (! |pos:act.administer:drug.abx_a| :named |a.fixture.m1_guideline_a.rule.0|))\n",
                "(assert (! (not |pos:act.administer:drug.abx_a|) :named |a.fixture.m1_guideline_b.rule.0|))\n",
                "(check-sat)\n",
                "(get-unsat-core)\n",
            )
        );
    }

    /// group.m1_null control pair (§8.2): the disjoint-interval Q1 the
    /// verify stage closes as the documented null, and a Q2 whose negated
    /// polarity lands in slot a (the control sorts first by id bytes).
    #[test]
    fn control_pair_pins_observed_bytes() {
        let (a, b) = (control(), doc_a());
        let pair = plan_pair("group.m1_null", &a, &b);
        assert_eq!(
            emit_overlap_query(&pair, &a, &b).body,
            concat!(
                "(set-logic QF_LRA)\n",
                "(set-option :print-success false)\n",
                "(set-option :produce-models true)\n",
                "(declare-const |cond.renal_severe| Bool)\n",
                "(declare-const |cond.sepsis| Bool)\n",
                "(declare-const |q.age_years| Real)\n",
                "(assert (! (and |cond.sepsis| (< |q.age_years| 18)) :named |ctx.fixture.m1_control.rule.0|))\n",
                "(assert (! (and |cond.sepsis| (not |cond.renal_severe|) (>= |q.age_years| 18)) :named |ctx.fixture.m1_guideline_a.rule.0|))\n",
                "(check-sat)\n",
                "(get-model)\n",
            )
        );
        assert_eq!(
            emit_deontic_query(&pair, &a, &b).body,
            concat!(
                "(set-logic QF_UF)\n",
                "(set-option :print-success false)\n",
                "(set-option :produce-unsat-cores true)\n",
                "(declare-const |pos:act.administer:drug.abx_a| Bool)\n",
                "(assert (! (not |pos:act.administer:drug.abx_a|) :named |a.fixture.m1_control.rule.0|))\n",
                "(assert (! |pos:act.administer:drug.abx_a| :named |a.fixture.m1_guideline_a.rule.0|))\n",
                "(check-sat)\n",
                "(get-unsat-core)\n",
            )
        );
    }

    /// Degenerate and numeral forms (module byte rules): a two-bound atom
    /// splices two comparisons in ge gt le lt order, a negative bound
    /// renders `(- n)`, a multi-disjunct `any` renders `or` with
    /// single-atom conjuncts bare, and Q2 maps `against` to the negated
    /// polarity.
    #[test]
    fn degenerate_and_numeral_forms() {
        let x = fc(
            "t.rule.x",
            Direction::For,
            dnf1(vec![interval("q.temp_c", Some(-5), None, None, Some(40))]),
        );
        let y = fc(
            "t.rule.y",
            Direction::Against,
            ContextExpr {
                any: vec![
                    ContextConjunct {
                        all: vec![concept("cond.a")],
                    },
                    ContextConjunct {
                        all: vec![
                            ContextAtom::ConceptNegated(id("cond.b")),
                            interval("q.n", None, Some(0), Some(2), None),
                        ],
                    },
                ],
            },
        );
        let pair = plan_pair("group.t", &x, &y);
        assert_eq!(
            emit_overlap_query(&pair, &x, &y).body,
            concat!(
                "(set-logic QF_LRA)\n",
                "(set-option :print-success false)\n",
                "(set-option :produce-models true)\n",
                "(declare-const |cond.a| Bool)\n",
                "(declare-const |cond.b| Bool)\n",
                "(declare-const |q.n| Real)\n",
                "(declare-const |q.temp_c| Real)\n",
                "(assert (! (and (>= |q.temp_c| (- 5)) (< |q.temp_c| 40)) :named |ctx.t.rule.x|))\n",
                "(assert (! (or |cond.a| (and (not |cond.b|) (> |q.n| 0) (<= |q.n| 2))) :named |ctx.t.rule.y|))\n",
                "(check-sat)\n",
                "(get-model)\n",
            )
        );
        assert_eq!(
            emit_deontic_query(&pair, &x, &y).body,
            concat!(
                "(set-logic QF_UF)\n",
                "(set-option :print-success false)\n",
                "(set-option :produce-unsat-cores true)\n",
                "(declare-const |pos:act.administer:drug.abx_a| Bool)\n",
                "(assert (! |pos:act.administer:drug.abx_a| :named |a.t.rule.x|))\n",
                "(assert (! (not |pos:act.administer:drug.abx_a|) :named |a.t.rule.y|))\n",
                "(check-sat)\n",
                "(get-unsat-core)\n",
            )
        );
    }

    /// Emission never re-sorts: a guard stored out of canonical-set order
    /// emits in stored order, and a single-atom single-conjunct guard
    /// renders as the bare atom.
    #[test]
    fn emission_follows_stored_conjunct_order() {
        let x = fc(
            "t.rule.x",
            Direction::For,
            dnf1(vec![
                interval("q.age_years", Some(18), None, None, None),
                concept("cond.sepsis"),
            ]),
        );
        let y = fc(
            "t.rule.y",
            Direction::Against,
            dnf1(vec![concept("cond.x")]),
        );
        let pair = plan_pair("group.t", &x, &y);
        assert_eq!(
            emit_overlap_query(&pair, &x, &y).body,
            concat!(
                "(set-logic QF_LRA)\n",
                "(set-option :print-success false)\n",
                "(set-option :produce-models true)\n",
                "(declare-const |cond.sepsis| Bool)\n",
                "(declare-const |cond.x| Bool)\n",
                "(declare-const |q.age_years| Real)\n",
                "(assert (! (and (>= |q.age_years| 18) |cond.sepsis|) :named |ctx.t.rule.x|))\n",
                "(assert (! |cond.x| :named |ctx.t.rule.y|))\n",
                "(check-sat)\n",
                "(get-model)\n",
            )
        );
    }

    /// Totality floors: an empty conjunct and an empty expression both
    /// render `true`.
    #[test]
    fn empty_contexts_render_true() {
        assert_eq!(context_term(&ContextExpr { any: vec![] }), "true");
        assert_eq!(context_term(&dnf1(vec![])), "true");
    }

    /// Slot order is the caller's contract; a swap is a bug, not data.
    #[test]
    #[should_panic(expected = "out of pair slot order")]
    fn swapped_slots_panic() {
        let (a, b) = (doc_a(), doc_b());
        let pair = plan_pair("group.m1_conflict", &a, &b);
        emit_overlap_query(&pair, &b, &a);
    }
}
