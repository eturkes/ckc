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

    const RULES_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/research_kernel/fixtures/rules.json"
    );

    fn load_rules() -> Vec<Rule> {
        let json = std::fs::read_to_string(RULES_PATH).expect("rules.json fixture must exist");
        serde_json::from_str(&json).expect("rules must parse")
    }

    /// The toy `rule_incomplete_provenance` triggers exactly the two
    /// expected default-shape violations (missing source_span_ids and
    /// empty provenance), each at Violation severity. The other rules
    /// in the same fixture pass the shape clean.
    #[test]
    fn rule_shape_violations_localize_to_incomplete_rule() {
        let rules = load_rules();
        let report = validate_rules(&rules);
        assert!(!report.conforms);

        let incomplete: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.focus_node == "rule_incomplete_provenance")
            .collect();
        let paths: Vec<&str> = incomplete.iter().map(|v| v.path.as_str()).collect();
        assert_eq!(incomplete.len(), 2);
        assert!(paths.contains(&"source_span_ids"));
        assert!(paths.contains(&"provenance"));
        for v in &report.violations {
            assert_eq!(v.severity, ShaclSeverity::Violation);
        }

        let clean: Vec<_> = rules
            .into_iter()
            .filter(|r| r.rule_id.as_str() != "rule_incomplete_provenance")
            .collect();
        assert!(validate_rules(&clean).conforms);
        assert!(validate_rules(&[]).conforms);
    }
}
