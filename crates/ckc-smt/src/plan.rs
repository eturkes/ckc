//! SPEC §8.3 compile-stage planning — the eligibility scan and
//! contradiction-query plan over a fixture group's per-document FormalIRs.
//!
//! [`plan_queries`] returns [`ContradictionQueryPair`]s only: query text
//! derives from the paired constraints in
//! [`emit_overlap_query`](crate::emit_overlap_query) /
//! [`emit_deontic_query`](crate::emit_deontic_query), the pairs ride
//! [`CompiledArtifact::query_plan`](crate::CompiledArtifact), and each
//! document's `FormalIr::plan` slot stays empty (pairs may cross documents,
//! so `FormalIr::derive` leaves planning here).

use ckc_core::{ContradictionQueryPair, FormalConstraint, FormalIr, Id, directions_opposed};

/// SPEC §6 eligibility scan and §8.6 id minting over the per-document
/// FormalIRs of fixture group `group_id`.
///
/// A constraint pair — unordered, same- or cross-document — is
/// conflict-eligible when the two Action normalized keys are equal (§5
/// action sameness) and [`directions_opposed`] holds (one direction
/// positive, the other against or contraindicating). Contexts never gate
/// eligibility: disjoint-context pairs stay planned, and Q1 later closes
/// them as the documented null result (§6).
///
/// Each pair normalizes to `constraint_a_id < constraint_b_id` by id bytes
/// (unique across a group's documents — rule ids embed document ids, §8.6),
/// and ids follow the §8.6 forms: `pair_id = q.<gsuf>.pair<n>` with
/// `<gsuf>` the group id minus its `group.` prefix and `n` counting from 1
/// in `(a, b)` id-byte order; query ids `<pair_id>.overlap` (Q1
/// context_overlap) and `<pair_id>.deontic` (Q2 deontic_consistency). The
/// plan is input-order independent.
pub fn plan_queries<'a>(
    group_id: &Id,
    irs: impl IntoIterator<Item = &'a FormalIr>,
) -> Vec<ContradictionQueryPair> {
    let constraints: Vec<&FormalConstraint> =
        irs.into_iter().flat_map(|ir| &ir.constraints).collect();
    let mut eligible: Vec<(&FormalConstraint, &FormalConstraint)> = Vec::new();
    for (i, &a) in constraints.iter().enumerate() {
        for &b in &constraints[i + 1..] {
            if a.action.key == b.action.key && directions_opposed(a.direction, b.direction) {
                eligible.push(if a.constraint_id.as_str() < b.constraint_id.as_str() {
                    (a, b)
                } else {
                    (b, a)
                });
            }
        }
    }
    eligible.sort_by(|(a1, b1), (a2, b2)| {
        (a1.constraint_id.as_str(), b1.constraint_id.as_str())
            .cmp(&(a2.constraint_id.as_str(), b2.constraint_id.as_str()))
    });
    let group = group_id.as_str();
    let gsuf = group.strip_prefix("group.").unwrap_or(group);
    eligible
        .into_iter()
        .enumerate()
        .map(|(k, (a, b))| {
            let mint = |s: String| {
                Id::new(s).expect("'q.' + id bytes + pair/query suffix stays in the id grammar")
            };
            let pair = format!("q.{gsuf}.pair{}", k + 1);
            ContradictionQueryPair {
                action_key: a.action.key.clone(),
                constraint_a_id: a.constraint_id.clone(),
                constraint_b_id: b.constraint_id.clone(),
                context_overlap_query_id: mint(format!("{pair}.overlap")),
                deontic_consistency_query_id: mint(format!("{pair}.deontic")),
                pair_id: mint(pair),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::{
        Action, ContextAtom, ContextConjunct, ContextExpr, Direction, QuantityInterval, Strength,
    };

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    /// One-conjunct DNF over `atoms` (every §8.6/§8.2 context is this
    /// shape).
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

    /// A §8.6-shaped constraint: `fc.<rule_id>`, administer action over
    /// `target`, strong.
    fn fc(
        rule_id: &str,
        direction: Direction,
        target: &str,
        context: ContextExpr,
    ) -> FormalConstraint {
        FormalConstraint {
            constraint_id: id(&format!("fc.{rule_id}")),
            rule_id: id(rule_id),
            action: Action::new(id("act.administer"), id(target)),
            context,
            direction,
            strength: Strength::Strong,
            certainty: None,
        }
    }

    fn doc(constraints: Vec<FormalConstraint>) -> FormalIr {
        FormalIr {
            constraints,
            plan: Vec::new(),
        }
    }

    /// docA rule.0 (§8.6): for administer abx_a under
    /// sepsis ∧ ¬renal_severe ∧ age ≥ 18.
    fn doc_a() -> FormalIr {
        doc(vec![fc(
            "fixture.m1_guideline_a.rule.0",
            Direction::For,
            "drug.abx_a",
            dnf1(vec![
                concept("cond.sepsis"),
                ContextAtom::ConceptNegated(id("cond.renal_severe")),
                age(true),
            ]),
        )])
    }

    /// docB rule.0 (§8.6): contraindicate the same action under
    /// sepsis ∧ age ≥ 18 ∧ pregnancy.
    fn doc_b() -> FormalIr {
        doc(vec![fc(
            "fixture.m1_guideline_b.rule.0",
            Direction::Contraindicate,
            "drug.abx_a",
            dnf1(vec![
                concept("cond.sepsis"),
                age(true),
                concept("cond.pregnancy"),
            ]),
        )])
    }

    /// Control rule.0 (§8.2): contraindicate the same action under
    /// sepsis ∧ age < 18 — interval disjoint with docA's.
    fn control() -> FormalIr {
        doc(vec![fc(
            "fixture.m1_control.rule.0",
            Direction::Contraindicate,
            "drug.abx_a",
            dnf1(vec![concept("cond.sepsis"), age(false)]),
        )])
    }

    /// The §8.6 worked pair over group.m1_conflict, every id pinned; the
    /// plan is input-order independent.
    #[test]
    fn worked_pair_pins_pair_and_query_ids() {
        let (a, b) = (doc_a(), doc_b());
        let want = ContradictionQueryPair {
            pair_id: id("q.m1_conflict.pair1"),
            action_key: id("act.administer:drug.abx_a"),
            constraint_a_id: id("fc.fixture.m1_guideline_a.rule.0"),
            constraint_b_id: id("fc.fixture.m1_guideline_b.rule.0"),
            context_overlap_query_id: id("q.m1_conflict.pair1.overlap"),
            deontic_consistency_query_id: id("q.m1_conflict.pair1.deontic"),
        };
        let group = id("group.m1_conflict");
        assert_eq!(plan_queries(&group, [&a, &b]), std::slice::from_ref(&want));
        assert_eq!(plan_queries(&group, [&b, &a]), [want]);
    }

    /// group.m1_null: disjoint age intervals leave eligibility untouched —
    /// the pair plans, and Q1 later decides the null (§6 documented-null
    /// path). Normalization puts the control constraint first by id bytes.
    #[test]
    fn disjoint_interval_control_pair_stays_eligible() {
        let plan = plan_queries(&id("group.m1_null"), [&doc_a(), &control()]);
        assert_eq!(
            plan,
            [ContradictionQueryPair {
                pair_id: id("q.m1_null.pair1"),
                action_key: id("act.administer:drug.abx_a"),
                constraint_a_id: id("fc.fixture.m1_control.rule.0"),
                constraint_b_id: id("fc.fixture.m1_guideline_a.rule.0"),
                context_overlap_query_id: id("q.m1_null.pair1.overlap"),
                deontic_consistency_query_id: id("q.m1_null.pair1.deontic"),
            }]
        );
    }

    /// Same direction on the same action key: directions not opposed, no
    /// pair.
    #[test]
    fn same_direction_pair_excluded() {
        let x = doc(vec![fc(
            "fixture.m1_guideline_a.rule.0",
            Direction::For,
            "drug.abx_a",
            dnf1(vec![concept("cond.sepsis")]),
        )]);
        let y = doc(vec![fc(
            "fixture.m1_guideline_b.rule.0",
            Direction::For,
            "drug.abx_a",
            dnf1(vec![concept("cond.pregnancy")]),
        )]);
        assert_eq!(plan_queries(&id("group.m1_conflict"), [&x, &y]), []);
    }

    /// Opposed directions on different action keys: no action sameness, no
    /// pair (§5 normalized target keys).
    #[test]
    fn different_action_pair_excluded() {
        let x = doc(vec![fc(
            "fixture.m1_guideline_a.rule.0",
            Direction::For,
            "drug.abx_a",
            dnf1(vec![concept("cond.sepsis")]),
        )]);
        let z = doc(vec![fc(
            "fixture.m1_guideline_b.rule.0",
            Direction::Contraindicate,
            "drug.abx_b",
            dnf1(vec![concept("cond.sepsis")]),
        )]);
        assert_eq!(plan_queries(&id("group.m1_conflict"), [&x, &z]), []);
    }

    /// Same-document pairs are in scope, and ordinals count from 1 in
    /// `(a, b)` id-byte order.
    #[test]
    fn ordinals_follow_id_byte_order_within_one_document() {
        let d = doc(vec![
            fc(
                "fixture.m1_guideline_a.rule.0",
                Direction::For,
                "drug.abx_a",
                dnf1(vec![concept("cond.sepsis")]),
            ),
            fc(
                "fixture.m1_guideline_a.rule.1",
                Direction::Permit,
                "drug.abx_a",
                dnf1(vec![concept("cond.pregnancy")]),
            ),
            fc(
                "fixture.m1_guideline_a.rule.2",
                Direction::Against,
                "drug.abx_a",
                dnf1(vec![concept("cond.renal_severe")]),
            ),
        ]);
        let plan = plan_queries(&id("group.m1_conflict"), [&d]);
        let triples: Vec<(&str, &str, &str)> = plan
            .iter()
            .map(|p| {
                (
                    p.pair_id.as_str(),
                    p.constraint_a_id.as_str(),
                    p.constraint_b_id.as_str(),
                )
            })
            .collect();
        assert_eq!(
            triples,
            [
                (
                    "q.m1_conflict.pair1",
                    "fc.fixture.m1_guideline_a.rule.0",
                    "fc.fixture.m1_guideline_a.rule.2",
                ),
                (
                    "q.m1_conflict.pair2",
                    "fc.fixture.m1_guideline_a.rule.1",
                    "fc.fixture.m1_guideline_a.rule.2",
                ),
            ]
        );
    }
}
