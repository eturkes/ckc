//! NormativeRule derivation (SPEC §8.3 normalize row, rule half): kept
//! [`ClinicalStatement`](ckc_core::ClinicalStatement)s lowered to conditioned
//! [`NormativeRule`]s, `rules[k]` deriving from `statements[k]` under the §8.6
//! id scheme (`rule_id = <document_id>.rule.<k>`, document ids the
//! corpora test_source ids).
//!
//! [`derive_norm_ir`] builds one DNF conjunct per statement:
//!
//! - population and condition atoms, interval-lowered: a positive
//!   [`ContextAtom::Concept`] whose lexicon entry carries interval
//!   semantics becomes that [`ContextAtom::Interval`] (成人 →
//!   `q.age_years >= 18`, 小児 → `< 18`); every other atom passes
//!   through verbatim;
//! - one [`ContextAtom::ConceptNegated`] per exception-clause `Concept`
//!   atom — §5: exceptions compile to negated context conjuncts — and
//!   the negation never interval-lowers; non-`Concept` clause atoms
//!   contribute nothing (unreachable through
//!   [`clinical_ir`](crate::normalize::clinical_ir), which attaches
//!   positive concepts only).
//!
//! The conjunct is stored pre-sorted in §4.3 set order (canonical sort
//! keys, byte-identical duplicates collapsed), so the in-memory value
//! equals its strict-read round trip. `source_region_ids` (§8.6 pins
//! `[r.2, r.3]`): the regions of the statement's recommendation-kind
//! source segments in reading order, then clause regions in clause
//! order. `exception_refs` cite the clause ids; direction, strength,
//! certainty, and action flow from the statement.

use std::collections::HashMap;

use ckc_core::{
    ClinicalIr, ContextAtom, ContextConjunct, ContextExpr, Id, NormIr, NormativeRule,
    QuantityInterval, SegmentIr, SegmentKind, canonical_sort_key,
};

use crate::normalize::Lexicon;

/// Derive the document's [`NormIr`] from its kept statements (module
/// doc; SPEC §5 NormativeRule row, §8.6 id scheme).
pub fn derive_norm_ir(
    document_id: &Id,
    clinical: &ClinicalIr,
    segments: &SegmentIr,
    lexicon: &Lexicon,
) -> NormIr {
    let intervals: HashMap<&Id, &QuantityInterval> = lexicon
        .concepts
        .iter()
        .filter_map(|c| c.interval.as_ref().map(|q| (&c.concept_id, q)))
        .collect();
    let rules = clinical
        .statements
        .iter()
        .enumerate()
        .map(|(k, statement)| {
            let rule_id = Id::new(format!("{document_id}.rule.{k}"))
                .expect("a valid document id keeps the Id grammar under a suffix");

            let mut all: Vec<ContextAtom> = statement
                .population
                .iter()
                .chain(&statement.condition)
                .map(|atom| lower(atom, &intervals))
                .collect();
            for clause in &statement.exceptions {
                for atom in &clause.atoms {
                    if let ContextAtom::Concept(concept) = atom {
                        all.push(ContextAtom::ConceptNegated(concept.clone()));
                    }
                }
            }
            all.sort_by_cached_key(|atom| {
                canonical_sort_key(atom).expect("context-atom canonical emission is infallible")
            });
            all.dedup();

            let mut source_region_ids: Vec<Id> = Vec::new();
            for segment in &segments.segments {
                if segment.kind == SegmentKind::Recommendation
                    && statement.source_segment_ids.contains(&segment.segment_id)
                {
                    source_region_ids.extend_from_slice(&segment.region_ids);
                }
            }
            for clause in &statement.exceptions {
                source_region_ids.extend_from_slice(&clause.region_ids);
            }

            NormativeRule {
                rule_id,
                context: ContextExpr {
                    any: vec![ContextConjunct { all }],
                },
                direction: statement.modality,
                action: statement.action.clone(),
                strength: statement.strength,
                source_region_ids,
                certainty: statement.certainty,
                exception_refs: statement
                    .exceptions
                    .iter()
                    .map(|clause| clause.exception_id.clone())
                    .collect(),
            }
        })
        .collect();
    NormIr { rules }
}

/// Interval-lower one population/condition atom (module doc).
fn lower(atom: &ContextAtom, intervals: &HashMap<&Id, &QuantityInterval>) -> ContextAtom {
    match atom {
        ContextAtom::Concept(concept) => match intervals.get(concept) {
            Some(interval) => ContextAtom::Interval((*interval).clone()),
            None => atom.clone(),
        },
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ckc_core::{
        Action, ArtifactWrapper, Certainty, ClinicalSegment, ClinicalStatement, DataClass,
        Direction, EvidenceStatus, ExceptionClause, Hash, Normalization, Origin, Producer,
        Provenance, SourceDocumentGraph, Strength, canonical_payload_bytes, content_hash,
        read_strict_canonical,
    };

    use crate::extract::{ExtractConfig, extract};
    use crate::normalize::{load_lexicon, normalize};
    use crate::segment::segment;

    fn id(s: &str) -> Id {
        Id::new(s).unwrap()
    }

    fn producer() -> Producer {
        Producer {
            pipeline_id: id("cand.m1"),
            pipeline_step_id: id("processing_stage.normalize"),
            toolchain_manifest_hash: Hash::new(format!("sha256:{}", "0".repeat(64))).unwrap(),
        }
    }

    fn test_source(name: &str) -> Vec<u8> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/test_sources/");
        std::fs::read(format!("{dir}{name}")).unwrap()
    }

    fn committed() -> Lexicon {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/lexicon/ja_core.yaml"
        );
        load_lexicon(&std::fs::read(path).unwrap()).unwrap()
    }

    fn extracted(name: &str, document_id: &str) -> ArtifactWrapper<SourceDocumentGraph> {
        let config = ExtractConfig {
            document_id: id(document_id),
            source_family: id("synthetic_test_source_html"),
            provenance: Provenance::Synthetic,
            data_class: DataClass::None,
            producer: producer(),
        };
        extract(&test_source(name), &config).unwrap()
    }

    /// Extract → segment → normalize a committed test_source under its
    /// corpora document id.
    fn pipeline(name: &str, document_id: &str) -> ArtifactWrapper<Normalization> {
        let source = extracted(name, document_id);
        let segments = segment(&source, &producer()).unwrap();
        normalize(&source, &segments, &committed(), &producer()).unwrap()
    }

    fn age(ge: Option<i64>, lt: Option<i64>) -> QuantityInterval {
        QuantityInterval {
            var: id("q.age_years"),
            ge,
            gt: None,
            le: None,
            lt,
        }
    }

    // THE oracle: the full pipeline reproduces the amended §8.6 NormativeRule
    // listing byte for byte, and the bytes strict-read back to the
    // derived value.
    #[test]
    fn pipeline_guideline_a_pins_spec86_bytes() {
        let wrapper = pipeline("m1_guideline_a.html", "test_source.m1_guideline_a");
        assert!(
            wrapper.diagnostics.is_empty(),
            "derives diagnostic-free, got {:?}",
            wrapper.diagnostics
        );
        let rules = &wrapper.payload.norm.rules;
        assert_eq!(rules.len(), 1, "one statement, one rule");
        let bytes = canonical_payload_bytes(&rules[0]).unwrap();
        assert_eq!(
            std::str::from_utf8(&bytes).unwrap(),
            concat!(
                r#"{"action":{"key":"act.administer:drug.abx_a","kind":"act.administer","#,
                r#""target":"drug.abx_a"},"context":{"any":[{"all":["#,
                r#"{"tag":"concept","value":"cond.sepsis"},"#,
                r#"{"tag":"concept_negated","value":"cond.renal_severe"},"#,
                r#"{"tag":"interval","value":{"ge":"18","var":"q.age_years"}}]}]},"#,
                r#""direction":"for","exception_refs":["exc.0"],"#,
                r#""rule_id":"test_source.m1_guideline_a.rule.0","#,
                r#""source_region_ids":["r.2","r.3"],"#,
                r#""strength":"strong"}"#
            )
        );
        let reread: NormativeRule = read_strict_canonical(&bytes).unwrap();
        assert_eq!(reread, rules[0], "strict read returns the derived value");
    }

    // guideline_b and control, full value pins (§8.6 docB paragraph):
    // populations lower to age intervals, exception-free
    // contraindicate/strong rules grounded in the one recommendation
    // region.
    #[test]
    fn pipeline_guideline_b_and_control_full_values() {
        let rule = |rule_id: &str, all: Vec<ContextAtom>| NormativeRule {
            rule_id: id(rule_id),
            context: ContextExpr {
                any: vec![ContextConjunct { all }],
            },
            direction: Direction::Contraindicate,
            action: Action::new(id("act.administer"), id("drug.abx_a")),
            strength: Strength::Strong,
            source_region_ids: vec![id("r.2")],
            certainty: None,
            exception_refs: vec![],
        };
        let cases = [
            (
                "m1_guideline_b.html",
                "test_source.m1_guideline_b",
                rule(
                    "test_source.m1_guideline_b.rule.0",
                    vec![
                        ContextAtom::Concept(id("cond.pregnancy")),
                        ContextAtom::Concept(id("cond.sepsis")),
                        ContextAtom::Interval(age(Some(18), None)),
                    ],
                ),
            ),
            (
                "m1_control.html",
                "test_source.m1_control",
                rule(
                    "test_source.m1_control.rule.0",
                    vec![
                        ContextAtom::Concept(id("cond.sepsis")),
                        ContextAtom::Interval(age(None, Some(18))),
                    ],
                ),
            ),
        ];
        for (name, document_id, want) in cases {
            let wrapper = pipeline(name, document_id);
            assert!(
                wrapper.diagnostics.is_empty(),
                "{name} derives diagnostic-free, got {:?}",
                wrapper.diagnostics
            );
            assert_eq!(wrapper.payload.norm, NormIr { rules: vec![want] }, "{name}");
        }
    }

    // Hand-built statements exercising the derivation rules the committed
    // test_sources cannot: recommendation regions follow reading order (seg.b
    // before seg.a) over the set order of source_segment_ids while the
    // exception-kind source segment contributes none, clause regions
    // follow clause order behind them, certainty flows, a clause Interval
    // atom contributes nothing, clause negation never interval-lowers
    // (pop.adult's entry carries an interval), and rules index as
    // rules[k] ↔ statements[k] with byte-equal atoms collapsing.
    #[test]
    fn derivation_semantics_hand_case() {
        let seg = |sid: &str, kind: SegmentKind, region: &str| ClinicalSegment {
            segment_id: id(sid),
            kind,
            region_ids: vec![id(region)],
        };
        let segments = SegmentIr {
            segments: vec![
                seg("seg.b", SegmentKind::Recommendation, "r.9"),
                seg("seg.a", SegmentKind::Recommendation, "r.1"),
                seg("seg.c", SegmentKind::Exception, "r.5"),
            ],
        };
        let action = Action::new(id("act.administer"), id("drug.abx_a"));
        let clinical = ClinicalIr {
            bindings: vec![],
            statements: vec![
                ClinicalStatement {
                    statement_id: id("stmt.0"),
                    population: vec![ContextAtom::Concept(id("pop.adult"))],
                    condition: vec![ContextAtom::Concept(id("cond.sepsis"))],
                    action: action.clone(),
                    modality: Direction::Require,
                    strength: Strength::Weak,
                    certainty: Some(Certainty::Moderate),
                    exceptions: vec![
                        ExceptionClause {
                            exception_id: id("exc.0"),
                            atoms: vec![
                                ContextAtom::Concept(id("pop.adult")),
                                ContextAtom::Interval(age(Some(65), None)),
                            ],
                            region_ids: vec![id("r.5")],
                        },
                        ExceptionClause {
                            exception_id: id("exc.1"),
                            atoms: vec![ContextAtom::Concept(id("cond.renal_severe"))],
                            region_ids: vec![id("r.4")],
                        },
                    ],
                    source_segment_ids: vec![id("seg.a"), id("seg.b"), id("seg.c")],
                },
                ClinicalStatement {
                    statement_id: id("stmt.1"),
                    population: vec![
                        ContextAtom::Concept(id("pop.child")),
                        ContextAtom::Concept(id("cond.sepsis")),
                    ],
                    condition: vec![ContextAtom::Concept(id("cond.sepsis"))],
                    action: action.clone(),
                    modality: Direction::For,
                    strength: Strength::Strong,
                    certainty: None,
                    exceptions: vec![],
                    source_segment_ids: vec![id("seg.a")],
                },
            ],
        };
        let norm = derive_norm_ir(&id("doc.hand"), &clinical, &segments, &committed());
        assert_eq!(
            norm,
            NormIr {
                rules: vec![
                    NormativeRule {
                        rule_id: id("doc.hand.rule.0"),
                        context: ContextExpr {
                            any: vec![ContextConjunct {
                                all: vec![
                                    ContextAtom::Concept(id("cond.sepsis")),
                                    ContextAtom::ConceptNegated(id("cond.renal_severe")),
                                    ContextAtom::ConceptNegated(id("pop.adult")),
                                    ContextAtom::Interval(age(Some(18), None)),
                                ],
                            }],
                        },
                        direction: Direction::Require,
                        action: action.clone(),
                        strength: Strength::Weak,
                        source_region_ids: vec![id("r.9"), id("r.1"), id("r.5"), id("r.4")],
                        certainty: Some(Certainty::Moderate),
                        exception_refs: vec![id("exc.0"), id("exc.1")],
                    },
                    NormativeRule {
                        rule_id: id("doc.hand.rule.1"),
                        context: ContextExpr {
                            any: vec![ContextConjunct {
                                all: vec![
                                    ContextAtom::Concept(id("cond.sepsis")),
                                    ContextAtom::Interval(age(None, Some(18))),
                                ],
                            }],
                        },
                        direction: Direction::For,
                        action,
                        strength: Strength::Strong,
                        source_region_ids: vec![id("r.1")],
                        certainty: None,
                        exception_refs: vec![],
                    },
                ],
            }
        );
    }

    // §4.4 wrapper shape over the processing_stage entry — ids, kind, producer,
    // [source, segments] input hashes in order, deterministic_compiler
    // under mechanical_evidence_status, empty sets, payload-hash agreement —
    // and determinism: double run byte-identical, the bytes surviving a
    // strict read → re-emit cycle.
    #[test]
    fn wrapper_requirements_and_double_run_determinism() {
        let lexicon = committed();
        let source = extracted("m1_guideline_a.html", "test_source.m1_guideline_a");
        let segments = segment(&source, &producer()).unwrap();
        let wrapper = normalize(&source, &segments, &lexicon, &producer()).unwrap();
        assert_eq!(wrapper.schema_id, id("schema.normalization"));
        assert_eq!(
            wrapper.artifact_id,
            id("test_source.m1_guideline_a.normalization")
        );
        assert_eq!(wrapper.artifact_kind, id("normalization"));
        assert_eq!(wrapper.producer, producer());
        assert_eq!(
            wrapper.input_hashes,
            vec![source.content_hash.clone(), segments.content_hash.clone()]
        );
        assert_eq!(wrapper.origin, Origin::DeterministicCompiler);
        assert_eq!(
            wrapper.evidence_status,
            EvidenceStatus::MechanicalEvidenceStatus
        );
        assert!(wrapper.external_effects.is_empty());
        assert!(wrapper.trace_refs.is_empty());
        assert!(wrapper.runtime_metadata.is_empty());
        assert_eq!(
            wrapper.content_hash,
            content_hash(&wrapper.payload).unwrap()
        );
        wrapper.validate().unwrap();

        let first = canonical_payload_bytes(&wrapper).unwrap();
        let second =
            canonical_payload_bytes(&normalize(&source, &segments, &lexicon, &producer()).unwrap())
                .unwrap();
        assert_eq!(first, second, "double normalize is byte-identical");
        let reread: ArtifactWrapper<Normalization> = read_strict_canonical(&first).unwrap();
        assert_eq!(
            canonical_payload_bytes(&reread).unwrap(),
            first,
            "strict read re-emits the same bytes"
        );
    }
}
