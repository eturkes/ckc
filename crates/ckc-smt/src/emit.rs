//! SPEC §6/§8.6 deterministic SMT-LIB emission and §8.3 compile-processing_stage
//! assembly — [`emit_overlap_query`] / [`emit_deontic_query`] render the two
//! §6 contradiction queries of one planned pair
//! ([`plan_queries`](crate::plan_queries) mints the pair and its query ids),
//! and [`compile`] assembles one test_source group's
//! [`CompiledArtifact`](crate::CompiledArtifact) from them: planned pairs
//! gated by the §6 M1 atom profile, bodies in plan order, the §8.5-item-4
//! assertion map bound through NormIR rules, target metadata.
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

use std::collections::{BTreeMap, HashMap};

use ckc_core::{
    ContextAtom, ContextConjunct, ContextExpr, ContradictionQueryPair, DiagnosticCode,
    DiagnosticRecord, Direction, FormalConstraint, FormalIr, Id, NormIr, NormativeRule, Outcome,
    StringPolicy,
};

use crate::{AssertionRecord, CompiledArtifact, QueryBody, SmtLogic, plan_queries};

/// SPEC §6 Q1 context_overlap (QF_LRA): both conditioned contexts as
/// `ctx.<rule_id>` named assertions; sat yields the recorded overlap
/// satisfying_example, unsat closes the pair as the documented no-conflict result. `a`/`b`
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

/// SPEC §8.3 compile-processing_stage core — the [`CompiledArtifact`] of one test_source
/// group. [`plan_queries`] plans over the documents' FormalIRs, each
/// surviving pair emits its two §6 bodies in plan order, and the §6
/// assertion map binds every `ctx.<rule_id>` and `a.<rule_id>` name to its
/// rule id and the rule's source region ids as a canonical set — looked up
/// from NormIR by `rule_id`, since FormalConstraint carries no regions
/// (§8.5 item 4 audits exactly this binding).
///
/// An out-of-profile construct — any context atom beyond the §6 M1
/// concept/concept_negated/interval set expressible in QF_LRA — drops its
/// pair from the plan with an `unsupported_ir_fragment` diagnostic (no plan
/// slot, bodies, or assertion entries; later pairs keep their minted
/// sequence_numbers) while the artifact still validates.
///
/// Documents arrive as the (FormalIR, NormIR) layer pairs of one IrBundle
/// each, validated per §5 before compilation; a planned rule the layers
/// cannot resolve is a caller bug, like emitter slot order. Target facts
/// ride the pinned target row: `target.smtlib2` with metadata
/// `profile m1`, `smtlib_version 2.6`.
pub fn compile<'a>(
    group_id: &Id,
    docs: impl IntoIterator<Item = (&'a FormalIr, &'a NormIr)>,
) -> CompiledArtifact {
    compile_with(group_id, docs, in_m1_profile)
}

/// SPEC §6 M1 declared profile over context atoms: concept and negated-
/// concept Bool constants plus QF_LRA quantity intervals — every atom kind
/// the M1 type accepts, exhaustively matched so an M3 variant (slot
/// equality, temporal interval) forces a profile decision here.
fn in_m1_profile(atom: &ContextAtom) -> bool {
    match atom {
        ContextAtom::Concept(_) | ContextAtom::ConceptNegated(_) | ContextAtom::Interval(_) => true,
    }
}

/// [`compile`] over an explicit atom profile — the seam §6 names "declared
/// target profiles gate anything richer", and the test path for the drop
/// machinery (the M1 profile accepts every constructible atom).
fn compile_with<'a>(
    group_id: &Id,
    docs: impl IntoIterator<Item = (&'a FormalIr, &'a NormIr)>,
    in_profile: impl Fn(&ContextAtom) -> bool,
) -> CompiledArtifact {
    let docs: Vec<(&FormalIr, &NormIr)> = docs.into_iter().collect();
    let constraints: HashMap<&str, &FormalConstraint> = docs
        .iter()
        .flat_map(|&(formal, _)| &formal.constraints)
        .map(|c| (c.constraint_id.as_str(), c))
        .collect();
    let rules: HashMap<&str, &NormativeRule> = docs
        .iter()
        .flat_map(|&(_, norm)| &norm.rules)
        .map(|r| (r.rule_id.as_str(), r))
        .collect();
    let mut solver_query_plan = Vec::new();
    let mut query_bodies = Vec::new();
    let mut assertion_to_source_map: BTreeMap<Id, AssertionRecord> = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for pair in plan_queries(group_id, docs.iter().map(|&(formal, _)| formal)) {
        let constraint = |id: &Id| {
            *constraints
                .get(id.as_str())
                .expect("planner pairs derive from these constraints")
        };
        let (a, b) = (
            constraint(&pair.constraint_a_id),
            constraint(&pair.constraint_b_id),
        );
        if let Some(offender) = [a, b]
            .into_iter()
            .find(|c| atoms(&c.context).any(|atom| !in_profile(atom)))
        {
            diagnostics.push(dropped_pair(
                &pair,
                normative_rule(&rules, &offender.rule_id),
            ));
            continue;
        }
        query_bodies.push(emit_overlap_query(&pair, a, b));
        query_bodies.push(emit_deontic_query(&pair, a, b));
        for constraint in [a, b] {
            let rule = normative_rule(&rules, &constraint.rule_id);
            let record = AssertionRecord {
                rule_ids: vec![rule.rule_id.clone()],
                region_ids: region_set(&rule.source_region_ids),
            };
            for prefix in ["ctx.", "a."] {
                let name = Id::new(format!("{prefix}{}", rule.rule_id))
                    .expect("a §6 assertion prefix before a valid id forms a valid id");
                assertion_to_source_map.insert(name, record.clone());
            }
        }
        solver_query_plan.push(pair);
    }
    CompiledArtifact {
        target_id: static_id("target.smtlib2"),
        solver_query_plan,
        query_bodies,
        assertion_to_source_map: assertion_to_source_map.into_iter().collect(),
        target_metadata: vec![
            (static_id("profile"), "m1".to_owned()),
            (static_id("smtlib_version"), "2.6".to_owned()),
        ],
        diagnostics,
    }
}

/// Every atom of a stored guard, in stored order.
fn atoms(context: &ContextExpr) -> impl Iterator<Item = &ContextAtom> {
    context.any.iter().flat_map(|conjunct| &conjunct.all)
}

/// Resolve a planned constraint's rule through the group's NormIRs (§5:
/// bundles validate layer coherence before compilation).
fn normative_rule<'r>(rules: &HashMap<&str, &'r NormativeRule>, rule_id: &Id) -> &'r NormativeRule {
    rules
        .get(rule_id.as_str())
        .unwrap_or_else(|| panic!("rule {rule_id} absent from NormIR"))
}

/// A rule's ordered source regions as the canonical set an
/// [`AssertionRecord`] or diagnostic stores (sorted by Id bytes,
/// duplicates collapsed).
fn region_set(ordered: &[Id]) -> Vec<Id> {
    let mut set = ordered.to_vec();
    set.sort();
    set.dedup();
    set
}

/// The §7.4 record of one dropped pair: `unsupported_ir_fragment` under
/// outcome `unsupported`, grounded in the offending rule's regions, naming
/// the pair whose plan slot it cost.
fn dropped_pair(pair: &ContradictionQueryPair, rule: &NormativeRule) -> DiagnosticRecord {
    let detail = StringPolicy::DiagnosticText
        .normalize(&format!(
            "pair {} dropped: rule {} context atom outside the target profile",
            pair.pair_id, rule.rule_id
        ))
        .expect("diagnostic_text is infallible");
    DiagnosticRecord {
        code: DiagnosticCode::UnsupportedIrFragment,
        outcome: Outcome::Unsupported,
        payload: vec![
            (static_id("detail"), detail),
            (static_id("pair"), pair.pair_id.to_string()),
        ],
        region_ids: region_set(&rule.source_region_ids),
        artifact_hashes: vec![],
    }
}

/// An [`Id`] from a static, known-valid identifier.
fn static_id(s: &str) -> Id {
    Id::new(s).expect("static ids are valid")
}

/// The slot requirements both emitters lean on: `a`/`b` arrive in pair order
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

    /// A NormativeRule behind an [`fc`]-shaped constraint: the §5 fields the
    /// projection copies plus regions and exception refs.
    fn nr(
        rule_id: &str,
        direction: Direction,
        target: &str,
        context: ContextExpr,
        regions: &[&str],
        exceptions: &[&str],
    ) -> NormativeRule {
        NormativeRule {
            rule_id: id(rule_id),
            context,
            direction,
            action: Action::new(id("act.administer"), id(target)),
            strength: Strength::Strong,
            source_region_ids: regions.iter().map(|r| id(r)).collect(),
            certainty: None,
            exception_refs: exceptions.iter().map(|e| id(e)).collect(),
        }
    }

    /// One document's layer pair for [`compile`]: NormIR from `rules`,
    /// FormalIR derived per §5.
    fn layers(rules: Vec<NormativeRule>) -> (FormalIr, NormIr) {
        let norm = NormIr { rules };
        (FormalIr::derive(&norm), norm)
    }

    /// docA rule.0 (§8.6 NormativeRule listing): for administer abx_a under
    /// sepsis ∧ ¬renal_severe ∧ age ≥ 18 (atoms in canonical stored order),
    /// grounded in r.2 + r.3 with exc.0 folded in.
    fn rule_a() -> NormativeRule {
        nr(
            "test_source.m1_guideline_a.rule.0",
            Direction::For,
            "drug.abx_a",
            dnf1(vec![
                concept("cond.sepsis"),
                ContextAtom::ConceptNegated(id("cond.renal_severe")),
                interval("q.age_years", Some(18), None, None, None),
            ]),
            &["r.2", "r.3"],
            &["exc.0"],
        )
    }

    /// docB rule.0 (§8.6): contraindicate the same action under
    /// pregnancy ∧ sepsis ∧ age ≥ 18 (canonical stored order), grounded in
    /// r.2.
    fn rule_b() -> NormativeRule {
        nr(
            "test_source.m1_guideline_b.rule.0",
            Direction::Contraindicate,
            "drug.abx_a",
            dnf1(vec![
                concept("cond.pregnancy"),
                concept("cond.sepsis"),
                interval("q.age_years", Some(18), None, None, None),
            ]),
            &["r.2"],
            &[],
        )
    }

    /// Control rule.0 (§8.2): contraindicate the same action under
    /// sepsis ∧ age < 18 — interval disjoint with docA's — grounded in r.2.
    fn rule_control() -> NormativeRule {
        nr(
            "test_source.m1_control.rule.0",
            Direction::Contraindicate,
            "drug.abx_a",
            dnf1(vec![
                concept("cond.sepsis"),
                interval("q.age_years", None, None, None, Some(18)),
            ]),
            &["r.2"],
            &[],
        )
    }

    /// docA rule.0 as FormalIR projects it.
    fn doc_a() -> FormalConstraint {
        FormalConstraint::from_rule(&rule_a())
    }

    /// docB rule.0 as FormalIR projects it.
    fn doc_b() -> FormalConstraint {
        FormalConstraint::from_rule(&rule_b())
    }

    /// Control rule.0 as FormalIR projects it.
    fn control() -> FormalConstraint {
        FormalConstraint::from_rule(&rule_control())
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
                "(assert (! (and |cond.sepsis| (not |cond.renal_severe|) (>= |q.age_years| 18)) :named |ctx.test_source.m1_guideline_a.rule.0|))\n",
                "(assert (! (and |cond.pregnancy| |cond.sepsis| (>= |q.age_years| 18)) :named |ctx.test_source.m1_guideline_b.rule.0|))\n",
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
                "(assert (! |pos:act.administer:drug.abx_a| :named |a.test_source.m1_guideline_a.rule.0|))\n",
                "(assert (! (not |pos:act.administer:drug.abx_a|) :named |a.test_source.m1_guideline_b.rule.0|))\n",
                "(check-sat)\n",
                "(get-unsat-core)\n",
            )
        );
    }

    /// group.m1_no_conflict control pair (§8.2): the disjoint-interval Q1 the
    /// verify processing_stage closes as the documented no-conflict result, and a
    /// Q2 whose negated polarity lands in slot a (the control sorts first by id
    /// bytes).
    #[test]
    fn control_pair_pins_observed_bytes() {
        let (a, b) = (control(), doc_a());
        let pair = plan_pair("group.m1_no_conflict", &a, &b);
        assert_eq!(
            emit_overlap_query(&pair, &a, &b).body,
            concat!(
                "(set-logic QF_LRA)\n",
                "(set-option :print-success false)\n",
                "(set-option :produce-models true)\n",
                "(declare-const |cond.renal_severe| Bool)\n",
                "(declare-const |cond.sepsis| Bool)\n",
                "(declare-const |q.age_years| Real)\n",
                "(assert (! (and |cond.sepsis| (< |q.age_years| 18)) :named |ctx.test_source.m1_control.rule.0|))\n",
                "(assert (! (and |cond.sepsis| (not |cond.renal_severe|) (>= |q.age_years| 18)) :named |ctx.test_source.m1_guideline_a.rule.0|))\n",
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
                "(assert (! (not |pos:act.administer:drug.abx_a|) :named |a.test_source.m1_control.rule.0|))\n",
                "(assert (! |pos:act.administer:drug.abx_a| :named |a.test_source.m1_guideline_a.rule.0|))\n",
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

    /// Slot order is the caller's requirements; a swap is a bug, not data.
    #[test]
    #[should_panic(expected = "out of pair slot order")]
    fn swapped_slots_panic() {
        let (a, b) = (doc_a(), doc_b());
        let pair = plan_pair("group.m1_conflict", &a, &b);
        emit_overlap_query(&pair, &b, &a);
    }

    /// compile assembles the §8.6 pair's artifact: pinned plan, both
    /// emitter bodies in plan order, the §8.5-item-4 assertion map with
    /// regions looked up from NormIR (docA r.2 + r.3, docB r.2), the
    /// pinned target row, no diagnostics; the §5 payload validates.
    #[test]
    fn compile_assembles_worked_pair_artifact() {
        let (fa, na) = layers(vec![rule_a()]);
        let (fb, nb) = layers(vec![rule_b()]);
        let artifact = compile(&id("group.m1_conflict"), [(&fa, &na), (&fb, &nb)]);
        assert_eq!(artifact.validate(), Ok(()));
        assert_eq!(artifact.target_id, id("target.smtlib2"));
        assert_eq!(artifact.solver_query_plan.len(), 1);
        let pair = &artifact.solver_query_plan[0];
        assert_eq!(pair.pair_id, id("q.m1_conflict.pair1"));
        let (a, b) = (doc_a(), doc_b());
        assert_eq!(
            artifact.query_bodies,
            [
                emit_overlap_query(pair, &a, &b),
                emit_deontic_query(pair, &a, &b),
            ]
        );
        let record_a = AssertionRecord {
            rule_ids: vec![id("test_source.m1_guideline_a.rule.0")],
            region_ids: vec![id("r.2"), id("r.3")],
        };
        let record_b = AssertionRecord {
            rule_ids: vec![id("test_source.m1_guideline_b.rule.0")],
            region_ids: vec![id("r.2")],
        };
        assert_eq!(
            artifact.assertion_to_source_map,
            [
                (id("a.test_source.m1_guideline_a.rule.0"), record_a.clone()),
                (id("a.test_source.m1_guideline_b.rule.0"), record_b.clone()),
                (id("ctx.test_source.m1_guideline_a.rule.0"), record_a),
                (id("ctx.test_source.m1_guideline_b.rule.0"), record_b),
            ]
        );
        assert_eq!(
            artifact.target_metadata,
            [
                (id("profile"), "m1".to_owned()),
                (id("smtlib_version"), "2.6".to_owned()),
            ]
        );
        assert_eq!(artifact.diagnostics, []);
    }

    /// The §8.2 control pair (group.m1_no_conflict) compiles the same way: the
    /// disjoint age intervals stay in profile (Q1 decides the no-conflict result at
    /// verify), the control rule in slot a by id bytes.
    #[test]
    fn compile_assembles_control_pair_artifact() {
        let (fa, na) = layers(vec![rule_a()]);
        let (fc_, nc) = layers(vec![rule_control()]);
        let artifact = compile(&id("group.m1_no_conflict"), [(&fa, &na), (&fc_, &nc)]);
        assert_eq!(artifact.validate(), Ok(()));
        assert_eq!(artifact.solver_query_plan.len(), 1);
        let pair = &artifact.solver_query_plan[0];
        assert_eq!(pair.pair_id, id("q.m1_no_conflict.pair1"));
        assert_eq!(pair.constraint_a_id, id("fc.test_source.m1_control.rule.0"));
        let (a, b) = (control(), doc_a());
        assert_eq!(
            artifact.query_bodies,
            [
                emit_overlap_query(pair, &a, &b),
                emit_deontic_query(pair, &a, &b),
            ]
        );
        let record_control = AssertionRecord {
            rule_ids: vec![id("test_source.m1_control.rule.0")],
            region_ids: vec![id("r.2")],
        };
        let record_a = AssertionRecord {
            rule_ids: vec![id("test_source.m1_guideline_a.rule.0")],
            region_ids: vec![id("r.2"), id("r.3")],
        };
        assert_eq!(
            artifact.assertion_to_source_map,
            [
                (
                    id("a.test_source.m1_control.rule.0"),
                    record_control.clone()
                ),
                (id("a.test_source.m1_guideline_a.rule.0"), record_a.clone()),
                (id("ctx.test_source.m1_control.rule.0"), record_control),
                (id("ctx.test_source.m1_guideline_a.rule.0"), record_a),
            ]
        );
        assert_eq!(artifact.diagnostics, []);
    }

    /// Multi-pair assembly: a rule shared by two pairs keeps one ctx./a.
    /// entry pair (identical records re-inserted), bodies stack in plan
    /// order, and ordered source regions store as canonical sets (r.9
    /// before r.10 in source order, r.10 first by id bytes).
    #[test]
    fn shared_rule_assertions_merge_and_regions_sort() {
        let (f, n) = layers(vec![
            nr(
                "t.rule.0",
                Direction::For,
                "drug.abx_a",
                dnf1(vec![concept("cond.sepsis")]),
                &["r.0"],
                &[],
            ),
            nr(
                "t.rule.1",
                Direction::Permit,
                "drug.abx_a",
                dnf1(vec![concept("cond.pregnancy")]),
                &["r.1"],
                &[],
            ),
            nr(
                "t.rule.2",
                Direction::Against,
                "drug.abx_a",
                dnf1(vec![concept("cond.renal_severe")]),
                &["r.9", "r.10"],
                &[],
            ),
        ]);
        let artifact = compile(&id("group.t"), [(&f, &n)]);
        assert_eq!(artifact.validate(), Ok(()));
        let pair_ids: Vec<&str> = artifact
            .solver_query_plan
            .iter()
            .map(|p| p.pair_id.as_str())
            .collect();
        assert_eq!(pair_ids, ["q.t.pair1", "q.t.pair2"]);
        let body_ids: Vec<&str> = artifact
            .query_bodies
            .iter()
            .map(|q| q.query_id.as_str())
            .collect();
        assert_eq!(
            body_ids,
            [
                "q.t.pair1.overlap",
                "q.t.pair1.deontic",
                "q.t.pair2.overlap",
                "q.t.pair2.deontic",
            ]
        );
        let keys: Vec<&str> = artifact
            .assertion_to_source_map
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(
            keys,
            [
                "a.t.rule.0",
                "a.t.rule.1",
                "a.t.rule.2",
                "ctx.t.rule.0",
                "ctx.t.rule.1",
                "ctx.t.rule.2",
            ]
        );
        assert_eq!(
            artifact.assertion_to_source_map[2].1.region_ids,
            [id("r.10"), id("r.9")]
        );
    }

    /// The §6 profile gate through the compile_with seam (the M1 profile
    /// accepts every constructible atom): a target without interval support
    /// drops the interval-bearing pair — no plan slot, bodies, or assertion
    /// entries; the survivor keeps its minted sequence_number — minting the §7.4
    /// unsupported_ir_fragment record grounded in the offending rule's
    /// regions, and the artifact still validates.
    #[test]
    fn out_of_profile_pair_drops_with_diagnostic() {
        let (fx, nx) = layers(vec![
            nr(
                "t.x.rule.0",
                Direction::For,
                "drug.abx_a",
                dnf1(vec![
                    concept("cond.sepsis"),
                    interval("q.age_years", Some(18), None, None, None),
                ]),
                &["r.0", "r.1"],
                &[],
            ),
            nr(
                "t.x.rule.1",
                Direction::For,
                "drug.abx_b",
                dnf1(vec![concept("cond.pregnancy")]),
                &["r.2"],
                &[],
            ),
        ]);
        let (fy, ny) = layers(vec![
            nr(
                "t.y.rule.0",
                Direction::Contraindicate,
                "drug.abx_a",
                dnf1(vec![concept("cond.sepsis")]),
                &["r.0"],
                &[],
            ),
            nr(
                "t.y.rule.1",
                Direction::Against,
                "drug.abx_b",
                dnf1(vec![concept("cond.pregnancy")]),
                &["r.1"],
                &[],
            ),
        ]);
        let docs = [(&fx, &nx), (&fy, &ny)];
        let no_intervals = |atom: &ContextAtom| !matches!(atom, ContextAtom::Interval(_));
        let artifact = compile_with(&id("group.t"), docs, no_intervals);
        assert_eq!(artifact.validate(), Ok(()));
        let pair_ids: Vec<&str> = artifact
            .solver_query_plan
            .iter()
            .map(|p| p.pair_id.as_str())
            .collect();
        assert_eq!(pair_ids, ["q.t.pair2"]);
        assert_eq!(artifact.query_bodies.len(), 2);
        let keys: Vec<&str> = artifact
            .assertion_to_source_map
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(
            keys,
            [
                "a.t.x.rule.1",
                "a.t.y.rule.1",
                "ctx.t.x.rule.1",
                "ctx.t.y.rule.1"
            ]
        );
        assert_eq!(
            artifact.diagnostics,
            [DiagnosticRecord {
                code: DiagnosticCode::UnsupportedIrFragment,
                outcome: Outcome::Unsupported,
                payload: vec![
                    (
                        id("detail"),
                        "pair q.t.pair1 dropped: rule t.x.rule.0 context atom outside the \
                         target profile"
                            .to_owned(),
                    ),
                    (id("pair"), "q.t.pair1".to_owned()),
                ],
                region_ids: vec![id("r.0"), id("r.1")],
                artifact_hashes: vec![],
            }]
        );
        // The M1 profile keeps the same group whole.
        let m1 = compile(&id("group.t"), docs);
        assert_eq!(m1.solver_query_plan.len(), 2);
        assert_eq!(m1.diagnostics, []);
    }

    /// A group with no eligible pairs compiles to the empty artifact —
    /// pinned target row and metadata, nothing else — and validates.
    #[test]
    fn pairless_group_compiles_empty() {
        let (f, n) = layers(vec![nr(
            "t.rule.0",
            Direction::For,
            "drug.abx_a",
            dnf1(vec![concept("cond.sepsis")]),
            &["r.0"],
            &[],
        )]);
        let artifact = compile(&id("group.t"), [(&f, &n)]);
        assert_eq!(artifact.validate(), Ok(()));
        assert_eq!(artifact.target_id, id("target.smtlib2"));
        assert_eq!(artifact.solver_query_plan, []);
        assert_eq!(artifact.query_bodies, []);
        assert_eq!(artifact.assertion_to_source_map, []);
        assert_eq!(
            artifact.target_metadata,
            [
                (id("profile"), "m1".to_owned()),
                (id("smtlib_version"), "2.6".to_owned()),
            ]
        );
        assert_eq!(artifact.diagnostics, []);
    }

    /// Layer coherence is the caller's requirements (§5: bundles validate
    /// before compilation); a planned rule the NormIRs cannot resolve is a
    /// bug, not data.
    #[test]
    #[should_panic(expected = "absent from NormIR")]
    fn unresolvable_rule_panics() {
        let (fa, _) = layers(vec![rule_a()]);
        let (fb, nb) = layers(vec![rule_b()]);
        let empty = NormIr { rules: vec![] };
        compile(&id("group.m1_conflict"), [(&fa, &empty), (&fb, &nb)]);
    }
}
