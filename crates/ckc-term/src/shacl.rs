use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ckc_core::clinical::Rule;

/// SHACL severity levels.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ShaclSeverity {
    Info,
    Warning,
    Violation,
}

/// A single SHACL constraint violation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ShaclViolation {
    pub focus_node: String,
    pub path: String,
    pub message: String,
    pub severity: ShaclSeverity,
}

/// SHACL validation report (serializable, content-hashable via CAS).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ShaclReport {
    pub conforms: bool,
    pub violations: Vec<ShaclViolation>,
}

/// Constraint kind for a property shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstraintKind {
    /// Array/set field must have at least `n` elements.
    MinCount(usize),
    /// String field must have length >= `n` characters.
    MinLength(usize),
}

/// Property constraint within a node shape.
#[derive(Clone, Debug)]
pub struct PropertyConstraint {
    pub path: String,
    pub kind: ConstraintKind,
}

/// Node shape targeting a class of CKC objects.
#[derive(Clone, Debug)]
pub struct NodeShape {
    pub name: String,
    pub constraints: Vec<PropertyConstraint>,
}

/// Phase 0 rule validation shapes.
///
/// Requires every Rule to have:
/// - `source_span_ids` with at least 1 element
/// - `provenance` with at least 1 character
pub fn default_rule_shapes() -> Vec<NodeShape> {
    vec![NodeShape {
        name: "RuleShape".into(),
        constraints: vec![
            PropertyConstraint {
                path: "source_span_ids".into(),
                kind: ConstraintKind::MinCount(1),
            },
            PropertyConstraint {
                path: "provenance".into(),
                kind: ConstraintKind::MinLength(1),
            },
        ],
    }]
}

fn evaluate_rule_constraint(
    rule: &Rule,
    constraint: &PropertyConstraint,
) -> Option<ShaclViolation> {
    let (actual, required) = match (constraint.path.as_str(), &constraint.kind) {
        ("source_span_ids", ConstraintKind::MinCount(n)) => (rule.source_span_ids.len(), *n),
        ("provenance", ConstraintKind::MinLength(n)) => (rule.provenance.len(), *n),
        _ => return None,
    };

    if actual >= required {
        return None;
    }

    Some(ShaclViolation {
        focus_node: rule.rule_id.as_str().to_owned(),
        path: constraint.path.clone(),
        message: format!(
            "{} requires minimum {required} (found {actual})",
            constraint.path
        ),
        severity: ShaclSeverity::Violation,
    })
}

/// Validate rules against Phase 0 SHACL shapes.
///
/// Applies `default_rule_shapes()` to each rule. `conforms` is true when
/// no violations exist.
pub fn validate_rules(rules: &[Rule]) -> ShaclReport {
    let shapes = default_rule_shapes();
    let mut violations = Vec::new();

    for rule in rules {
        for shape in &shapes {
            for constraint in &shape.constraints {
                if let Some(v) = evaluate_rule_constraint(rule, constraint) {
                    violations.push(v);
                }
            }
        }
    }

    violations.sort_by(|a, b| a.focus_node.cmp(&b.focus_node).then(a.path.cmp(&b.path)));

    ShaclReport {
        conforms: violations.is_empty(),
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckc_core::canonical::content_hash;

    const RULES_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/toy_research_kernel/fixtures/rules.json"
    );

    fn load_rules() -> Vec<Rule> {
        let json = std::fs::read_to_string(RULES_PATH).expect("rules.json fixture must exist");
        serde_json::from_str(&json).expect("rules must parse")
    }

    #[test]
    fn catches_provenance_incomplete_rule() {
        let rules = load_rules();
        let report = validate_rules(&rules);

        assert!(!report.conforms, "report must flag non-conformance");

        let incomplete_violations: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.focus_node == "rule_incomplete_provenance")
            .collect();

        assert_eq!(
            incomplete_violations.len(),
            2,
            "provenance-incomplete rule must trigger exactly 2 violations \
             (source_span_ids + provenance)"
        );

        let paths: Vec<&str> = incomplete_violations
            .iter()
            .map(|v| v.path.as_str())
            .collect();
        assert!(paths.contains(&"source_span_ids"));
        assert!(paths.contains(&"provenance"));
    }

    #[test]
    fn valid_rules_pass_clean() {
        let rules = load_rules();
        let valid_rules: Vec<_> = rules
            .into_iter()
            .filter(|r| r.rule_id.as_str() != "rule_incomplete_provenance")
            .collect();

        let report = validate_rules(&valid_rules);
        assert!(
            report.conforms,
            "valid rules must pass: {:?}",
            report.violations
        );
        assert!(report.violations.is_empty());
    }

    #[test]
    fn empty_rules_pass() {
        let report = validate_rules(&[]);
        assert!(report.conforms);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn violation_severity_is_violation() {
        let rules = load_rules();
        let report = validate_rules(&rules);
        for v in &report.violations {
            assert_eq!(v.severity, ShaclSeverity::Violation);
        }
    }

    #[test]
    fn report_serializable_roundtrip() {
        let rules = load_rules();
        let report = validate_rules(&rules);

        let json = serde_json::to_string(&report).expect("report must serialize");
        let rt: ShaclReport = serde_json::from_str(&json).expect("report must deserialize");
        assert_eq!(report, rt);
    }

    #[test]
    fn report_content_hashable() {
        let rules = load_rules();
        let report = validate_rules(&rules);

        let h1 = content_hash(&report);
        let h2 = content_hash(&report);
        assert_eq!(h1, h2, "content hash must be deterministic");
        assert!(
            h1.0.starts_with("sha256:"),
            "content hash must use sha256 prefix"
        );
    }

    #[test]
    fn report_is_deterministic() {
        let rules = load_rules();
        let r1 = validate_rules(&rules);
        let r2 = validate_rules(&rules);
        assert_eq!(r1, r2, "repeated validation must produce identical reports");
        assert_eq!(
            content_hash(&r1),
            content_hash(&r2),
            "content hashes must match"
        );
    }

    #[test]
    fn default_shapes_have_expected_constraints() {
        let shapes = default_rule_shapes();
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].name, "RuleShape");
        assert_eq!(shapes[0].constraints.len(), 2);
        assert_eq!(shapes[0].constraints[0].path, "source_span_ids");
        assert_eq!(shapes[0].constraints[0].kind, ConstraintKind::MinCount(1));
        assert_eq!(shapes[0].constraints[1].path, "provenance");
        assert_eq!(shapes[0].constraints[1].kind, ConstraintKind::MinLength(1));
    }
}
