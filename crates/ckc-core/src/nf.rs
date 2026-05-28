use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::artifact::{
    DecisionRow, DecisionTable, EventNarrative, ExecutionWitness, PatientCase, WorkflowFragment,
};
use crate::canonical::to_canonical_bytes;
use crate::clinical::{
    Action, ClinicalClaim, ConfidenceInterval, EtDFrame, EvidenceAtom, Norm, PICOFrame, Rule,
};
use crate::enums::{DeonticProjection, HitPolicy};
use crate::id::{ConceptId, SpanId};
use crate::source::{
    BBox, Concept, CorpusDocument, ExtractedTable, ExtractorVote, SourceSpan, TableCellRef,
    TerminologyBinding,
};
use crate::verify::{ArgumentGraph, AssuranceNode, AuditTrace, Certificate, Conflict};

// ---------------------------------------------------------------------------
// NF context: rewrite log and diagnostics
// ---------------------------------------------------------------------------

/// Record of a single field rewrite during normalization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NfRewrite {
    pub pass: u8,
    pub field: String,
    pub before: String,
    pub after: String,
}

/// Structured diagnostic emitted during normalization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NfDiagnostic {
    pub stage: String,
    pub code: String,
    pub message: String,
}

/// Accumulated context for the CKC Normal Form rewrite pipeline.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NfContext {
    pub rewrites: Vec<NfRewrite>,
    pub diagnostics: Vec<NfDiagnostic>,
    /// Pre-computed concept_id → canonical concept_id mapping from e-graph
    /// equivalence. Populated by `with_term_map()` or `normalize_with_terms()`.
    /// When `None`, pass 8 is identity.
    #[serde(skip)]
    term_map: Option<BTreeMap<String, String>>,
}

impl NfContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a context with a pre-computed terminology map.
    /// Pass 8 resolves concept references through this map.
    pub fn with_term_map(term_map: BTreeMap<String, String>) -> Self {
        Self {
            rewrites: vec![],
            diagnostics: vec![],
            term_map: Some(term_map),
        }
    }

    /// Look up the canonical concept_id for a given concept_id.
    /// Returns `None` when no term_map is set or the concept_id is unknown.
    pub fn resolve_concept(&self, concept_id: &str) -> Option<&str> {
        self.term_map.as_ref()?.get(concept_id).map(|s| s.as_str())
    }

    /// Apply text normalization to a string field. Records a rewrite when
    /// the normalized result differs from the original.
    pub fn normalize_field(&mut self, pass: u8, field: &str, value: &mut String) {
        let normalized = normalize_text(value);
        if *value != normalized {
            self.rewrites.push(NfRewrite {
                pass,
                field: field.into(),
                before: std::mem::take(value),
                after: normalized.clone(),
            });
            *value = normalized;
        }
    }

    /// Apply text normalization to an optional string field.
    pub fn normalize_opt_field(&mut self, pass: u8, field: &str, value: &mut Option<String>) {
        if let Some(s) = value {
            self.normalize_field(pass, field, s);
        }
    }

    /// Apply text normalization to each element of a string vector.
    pub fn normalize_vec_field(&mut self, pass: u8, field: &str, values: &mut [String]) {
        for (i, v) in values.iter_mut().enumerate() {
            let indexed = format!("{field}[{i}]");
            self.normalize_field(pass, &indexed, v);
        }
    }

    /// Sort a `Vec<T: Ord>` in place. Record a rewrite when order changes.
    pub fn sort_ord<T: Ord>(&mut self, field: &str, values: &mut [T]) {
        if values.len() <= 1 {
            return;
        }
        if values.windows(2).all(|w| w[0] <= w[1]) {
            return;
        }
        values.sort();
        self.rewrites.push(NfRewrite {
            pass: 4,
            field: field.into(),
            before: format!("{} items", values.len()),
            after: "sorted".into(),
        });
    }

    fn sort_by_canonical_impl<T: Serialize>(
        &mut self,
        pass: u8,
        field: &str,
        desc: &str,
        values: &mut Vec<T>,
    ) {
        if values.len() <= 1 {
            return;
        }
        let mut keyed: Vec<(Vec<u8>, T)> = values
            .drain(..)
            .map(|v| {
                let key = to_canonical_bytes(&v);
                (key, v)
            })
            .collect();
        let already_sorted = keyed.windows(2).all(|w| w[0].0 <= w[1].0);
        if !already_sorted {
            keyed.sort_by(|a, b| a.0.cmp(&b.0));
            self.rewrites.push(NfRewrite {
                pass,
                field: field.into(),
                before: format!("{} items", keyed.len()),
                after: desc.into(),
            });
        }
        values.extend(keyed.into_iter().map(|(_, v)| v));
    }

    /// Sort a `Vec<T: Serialize>` by RFC 8785 canonical JSON byte comparison.
    /// Record a rewrite when order changes.
    pub fn sort_by_canonical<T: Serialize>(&mut self, field: &str, values: &mut Vec<T>) {
        self.sort_by_canonical_impl(4, field, "sorted by canonical bytes", values);
    }

    /// Pass 11: sort graph elements by canonical bytes for stable graph
    /// canonicalization. Used for ArgumentGraph and WorkflowFragment arrays.
    pub fn sort_graph<T: Serialize>(&mut self, field: &str, values: &mut Vec<T>) {
        self.sort_by_canonical_impl(11, field, "sorted for graph canonicalization", values);
    }

    /// Sort commutative AND/OR operands in a string expression.
    /// At schema v0, handles top-level operators in parenthesized expressions.
    pub fn sort_commutative(&mut self, field: &str, value: &mut String) {
        let sorted = sort_commutative_operands(value);
        if *value != sorted {
            self.rewrites.push(NfRewrite {
                pass: 4,
                field: field.into(),
                before: std::mem::take(value),
                after: sorted.clone(),
            });
            *value = sorted;
        }
    }

    /// Pass 6: lowercase an action_type string for canonical casing.
    pub fn normalize_action_type(&mut self, value: &mut String) {
        let lower = value.to_ascii_lowercase();
        if *value != lower {
            self.rewrites.push(NfRewrite {
                pass: 6,
                field: "action_type".into(),
                before: std::mem::take(value),
                after: lower.clone(),
            });
            *value = lower;
        }
    }

    /// Pass 7: normalize unit strings in a JSON `Value` field.
    /// Walks the value tree and replaces known unit aliases with
    /// UCUM canonical forms. Records a rewrite when any change occurs.
    pub fn normalize_units(&mut self, field: &str, value: &mut Value) {
        let before = to_canonical_bytes(value);
        if normalize_json_units(value) {
            self.rewrites.push(NfRewrite {
                pass: 7,
                field: field.into(),
                before: String::from_utf8_lossy(&before).into_owned(),
                after: String::from_utf8_lossy(&to_canonical_bytes(value)).into_owned(),
            });
        }
    }

    /// Pass 12: set a stable ID derived from normalized content (with ID
    /// blanked) and sorted source anchors. `content_with_blanked_id` must
    /// be the RFC 8785 canonical bytes of the object after replacing its
    /// primary ID field with an empty string.
    pub fn set_stable_id(
        &mut self,
        field: &str,
        id: &mut String,
        content_with_blanked_id: &[u8],
        source_span_ids: &[SpanId],
    ) {
        let stable = compute_stable_id(content_with_blanked_id, source_span_ids);
        if *id != stable {
            self.rewrites.push(NfRewrite {
                pass: 12,
                field: field.into(),
                before: std::mem::take(id),
                after: stable.clone(),
            });
            *id = stable;
        }
    }

    /// Pass 13: sort accumulated diagnostics by (stage, code, message).
    pub fn sort_diagnostics(&mut self) {
        self.diagnostics.sort_by(|a, b| {
            a.stage
                .cmp(&b.stage)
                .then(a.code.cmp(&b.code))
                .then(a.message.cmp(&b.message))
        });
    }
}

// ---------------------------------------------------------------------------
// Pass 12: stable ID computation
// ---------------------------------------------------------------------------

/// Compute a deterministic stable ID from normalized content bytes (with
/// the object's primary ID blanked) and sorted source anchor IDs.
/// Format: `nf-{32_lowercase_hex_chars}` (128-bit prefix of SHA-256).
fn compute_stable_id(content_with_blanked_id: &[u8], source_span_ids: &[SpanId]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content_with_blanked_id);
    for span_id in source_span_ids {
        hasher.update(b"\x00");
        hasher.update(span_id.as_str().as_bytes());
    }
    let digest = hasher.finalize();
    let prefix = u128::from_be_bytes(digest[..16].try_into().unwrap());
    format!("nf-{prefix:032x}")
}

// ---------------------------------------------------------------------------
// Text normalization (Pass 2)
// ---------------------------------------------------------------------------

/// Normalize a text string: Unicode NFKC, ideographic space (U+3000) to
/// ASCII space, whitespace collapse, trim.
///
/// NFKC handles fullwidth ASCII to halfwidth and halfwidth katakana to
/// fullwidth. The ideographic space replacement and whitespace collapse
/// handle remaining Japanese-specific spacing.
#[must_use]
pub fn normalize_text(s: &str) -> String {
    let nfkc: String = s.nfkc().collect();
    let mut result = String::with_capacity(nfkc.len());
    let mut prev_ws = true;
    for ch in nfkc.chars() {
        let ch = if ch == '\u{3000}' { ' ' } else { ch };
        if ch.is_whitespace() {
            if !prev_ws {
                result.push(' ');
            }
            prev_ws = true;
        } else {
            result.push(ch);
            prev_ws = false;
        }
    }
    if result.ends_with(' ') {
        result.pop();
    }
    result
}

/// Convenience: normalize a value in place and return the accumulated context.
/// Applies Pass 13 (diagnostic ordering) after all type-level passes.
/// Pass 8 (terminology) is identity; use `normalize_with_terms` for e-graph resolution.
pub fn normalize_all<T: Normalize>(value: &mut T) -> NfContext {
    let mut ctx = NfContext::new();
    value.normalize(&mut ctx);
    ctx.sort_diagnostics();
    ctx
}

/// Normalize with a pre-computed terminology map from e-graph equivalence.
/// Pass 8 resolves concept references (e.g., Action.target_concept) through
/// the canonical mapping, so variant concept IDs converge to one representative.
pub fn normalize_with_terms<T: Normalize>(
    value: &mut T,
    term_map: BTreeMap<String, String>,
) -> NfContext {
    let mut ctx = NfContext::with_term_map(term_map);
    value.normalize(&mut ctx);
    ctx.sort_diagnostics();
    ctx
}

// ---------------------------------------------------------------------------
// Commutative operand sorting (Pass 4 — string expressions)
// ---------------------------------------------------------------------------

/// Sort commutative AND/OR operands at depth 0 (outside parentheses).
/// Mixed AND/OR at depth 0 is left unchanged (ambiguous precedence).
/// At schema v0, Rule antecedent/consequent/context are strings; this
/// function provides text-level sorting until typed ASTs replace them.
fn sort_commutative_operands(s: &str) -> String {
    let and_parts = split_at_depth_0(s, " AND ");
    let or_parts = split_at_depth_0(s, " OR ");
    let and_splits = and_parts.len() > 1;
    let or_splits = or_parts.len() > 1;

    if and_splits && !or_splits {
        let mut parts = and_parts;
        parts.sort();
        return parts.join(" AND ");
    }
    if or_splits && !and_splits {
        let mut parts = or_parts;
        parts.sort();
        return parts.join(" OR ");
    }
    s.to_string()
}

/// Split `s` at depth-0 (outside parentheses) occurrences of `op`.
/// Each resulting part is trimmed.
fn split_at_depth_0<'a>(s: &'a str, op: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                i += 1;
            }
            _ => {
                if depth == 0 && s[i..].starts_with(op) {
                    parts.push(s[start..i].trim());
                    i += op.len();
                    start = i;
                } else {
                    i += 1;
                }
            }
        }
    }
    parts.push(s[start..].trim());
    parts
}

// ---------------------------------------------------------------------------
// Pass 7: unit string normalization (quantity constraints)
// ---------------------------------------------------------------------------

/// Map a string to its UCUM canonical form when it matches a known unit alias.
/// Returns `None` for strings that are already canonical or unrecognized.
fn canonical_unit(s: &str) -> Option<&'static str> {
    match s {
        "ml" | "ML" => Some("mL"),
        "l" => Some("L"),
        "\u{2103}" | "\u{00B0}C" | "degC" | "degree_celsius" => Some("Cel"),
        "mmHg" => Some("mm[Hg]"),
        "bpm" | "beats/min" => Some("/min"),
        "mcg" | "\u{03BC}g" => Some("ug"),
        _ => None,
    }
}

/// Walk a `serde_json::Value` tree and replace string values matching known
/// unit aliases with their UCUM canonical form. Returns true when any value
/// was changed.
fn normalize_json_units(v: &mut Value) -> bool {
    match v {
        Value::String(s) => {
            if let Some(canonical) = canonical_unit(s) {
                *s = canonical.into();
                true
            } else {
                false
            }
        }
        Value::Object(map) => {
            let mut changed = false;
            for val in map.values_mut() {
                changed |= normalize_json_units(val);
            }
            changed
        }
        Value::Array(arr) => {
            let mut changed = false;
            for val in arr.iter_mut() {
                changed |= normalize_json_units(val);
            }
            changed
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Pass 9: Japanese clinical modality lexicon (stub)
// ---------------------------------------------------------------------------

/// Look up a Japanese clinical modality phrase in the toy-scenario lexicon.
/// Returns the canonical `DeonticProjection` for recognized phrases.
/// Uses NFKC-normalized comparison for fullwidth/spacing tolerance.
///
/// This stub covers the minimal phrase set for Phase 0 toy scenarios.
/// Later phases expand coverage using corpus-derived phrase extraction.
fn modality_lexicon(phrase: &str) -> Option<DeonticProjection> {
    let normalized = normalize_text(phrase);
    match normalized.as_str() {
        "投与を推奨する" | "投与を強く推奨する" | "使用を推奨する" | "使用を提案する" => {
            Some(DeonticProjection::Recommended)
        }
        "投与すべきである" | "使用すべきである" => {
            Some(DeonticProjection::Obligatory)
        }
        "投与してはならない"
        | "使用してはならない"
        | "禁忌である"
        | "投与しないことを推奨する"
        | "使用しないことを推奨する" => Some(DeonticProjection::Prohibited),
        "投与を考慮してもよい" | "使用を考慮してもよい" => {
            Some(DeonticProjection::Permitted)
        }
        "投与は任意である" | "使用は任意である" => {
            Some(DeonticProjection::Optional)
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Normalize trait
// ---------------------------------------------------------------------------

/// Normalize a CKC object in place according to the NF pipeline.
///
/// Each type implements passes relevant to its fields. Types with a
/// `profiles` field use their profiles to determine which passes apply;
/// Pass 1-2 text normalization is universal across all profiles.
pub trait Normalize {
    fn normalize(&mut self, ctx: &mut NfContext);
}

impl<T: Normalize> Normalize for Vec<T> {
    fn normalize(&mut self, ctx: &mut NfContext) {
        for item in self.iter_mut() {
            item.normalize(ctx);
        }
    }
}

impl<T: Normalize> Normalize for Option<T> {
    fn normalize(&mut self, ctx: &mut NfContext) {
        if let Some(inner) = self {
            inner.normalize(ctx);
        }
    }
}

// ---------------------------------------------------------------------------
// Pass 1-2 implementations: types with text fields or delegating children
// ---------------------------------------------------------------------------

impl Normalize for SourceSpan {
    fn normalize(&mut self, ctx: &mut NfContext) {
        // Pass 1: raw_text preserved verbatim
        // Pass 2: normalize derived text fields
        ctx.normalize_field(2, "nfkc_text", &mut self.nfkc_text);
        ctx.normalize_field(2, "search_text", &mut self.search_text);
        ctx.normalize_field(2, "display_text", &mut self.display_text);
        self.extractor_votes.normalize(ctx);
        // Pass 4: sort unordered fields
        ctx.sort_by_canonical("extractor_votes", &mut self.extractor_votes);
        // Pass 5: section_path preserves document hierarchy order
    }
}

impl Normalize for CorpusDocument {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "title_ja", &mut self.title_ja);
        ctx.normalize_opt_field(2, "title_en", &mut self.title_en);
    }
}

impl Normalize for Concept {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "label_ja", &mut self.label_ja);
        ctx.normalize_opt_field(2, "label_en", &mut self.label_en);
        self.terminology_bindings.normalize(ctx);
        // Pass 4: sort unordered fields
        ctx.sort_by_canonical("terminology_bindings", &mut self.terminology_bindings);
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        // Pass 12: stable ID
        let saved = std::mem::take(&mut self.concept_id.0);
        let content = to_canonical_bytes(self);
        self.concept_id.0 = saved;
        ctx.set_stable_id(
            "concept_id",
            &mut self.concept_id.0,
            &content,
            &self.source_span_ids,
        );
    }
}

impl Normalize for TerminologyBinding {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "label", &mut self.label);
    }
}

impl Normalize for ExtractedTable {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_vec_field(2, "row_headers", &mut self.row_headers);
        ctx.normalize_vec_field(2, "column_headers", &mut self.column_headers);
        self.extraction_votes.normalize(ctx);
        // Pass 4: sort unordered fields
        ctx.sort_ord("cell_span_ids", &mut self.cell_span_ids);
        ctx.sort_by_canonical("extraction_votes", &mut self.extraction_votes);
        // Pass 5: row_headers, column_headers, reading_order preserve table structure
    }
}

impl Normalize for ClinicalClaim {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "gloss_ja", &mut self.gloss_ja);
        ctx.normalize_field(2, "gloss_en", &mut self.gloss_en);
        self.pico.normalize(ctx);
        self.etd.normalize(ctx);
        self.evidence_atoms.normalize(ctx);
        // Pass 4: sort unordered fields
        ctx.sort_ord("profiles", &mut self.profiles);
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        ctx.sort_by_canonical("evidence_atoms", &mut self.evidence_atoms);
        ctx.sort_ord("rule_ids", &mut self.rule_ids);
        ctx.sort_ord("decision_table_ids", &mut self.decision_table_ids);
        ctx.sort_ord("workflow_fragment_ids", &mut self.workflow_fragment_ids);
        // Pass 12: stable ID
        let saved = std::mem::take(&mut self.claim_id.0);
        let content = to_canonical_bytes(self);
        self.claim_id.0 = saved;
        ctx.set_stable_id(
            "claim_id",
            &mut self.claim_id.0,
            &content,
            &self.source_span_ids,
        );
    }
}

impl Normalize for Rule {
    fn normalize(&mut self, ctx: &mut NfContext) {
        self.norm.normalize(ctx);
        // Pass 4: sort commutative operands in string expressions
        ctx.sort_commutative("context", &mut self.context);
        ctx.sort_commutative("antecedent", &mut self.antecedent);
        ctx.sort_commutative("consequent", &mut self.consequent);
        // Pass 4: sort unordered fields
        ctx.sort_ord("profiles", &mut self.profiles);
        ctx.sort_ord("exceptions", &mut self.exceptions);
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        ctx.sort_ord("certificate_ids", &mut self.certificate_ids);
        // Pass 5: priority_over preserves priority chain order
        // Pass 12: stable ID from normalized content + source anchors
        let saved = std::mem::take(&mut self.rule_id.0);
        let content = to_canonical_bytes(self);
        self.rule_id.0 = saved;
        ctx.set_stable_id(
            "rule_id",
            &mut self.rule_id.0,
            &content,
            &self.source_span_ids,
        );
    }
}

impl Normalize for Norm {
    fn normalize(&mut self, ctx: &mut NfContext) {
        // Pass 1: original_modality_phrase_ja preserved verbatim
        self.action.normalize(ctx);
        // Pass 9: normalize deontic projection through modality lexicon
        if let Some(canonical) = modality_lexicon(&self.original_modality_phrase_ja)
            && self.deontic_projection != canonical
        {
            ctx.rewrites.push(NfRewrite {
                pass: 9,
                field: "deontic_projection".into(),
                before: format!("{:?}", self.deontic_projection).to_ascii_lowercase(),
                after: format!("{:?}", canonical).to_ascii_lowercase(),
            });
            self.deontic_projection = canonical;
        }
    }
}

impl Normalize for DecisionTable {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_vec_field(2, "input_columns", &mut self.input_columns);
        ctx.normalize_vec_field(2, "output_columns", &mut self.output_columns);
        self.rows.normalize(ctx);
        // Pass 4: sort rows for commutative hit policies only
        match self.hit_policy {
            HitPolicy::Unique | HitPolicy::Any | HitPolicy::Collect => {
                ctx.sort_by_canonical("rows", &mut self.rows);
            }
            // Pass 5: First, Priority, RuleOrder, OutputOrder preserve row order
            _ => {}
        }
        ctx.sort_ord("certificate_ids", &mut self.certificate_ids);
        // Pass 5: input_columns, output_columns preserve column order
        // Pass 12: stable ID (no direct source_span_ids on DecisionTable)
        let saved = std::mem::take(&mut self.table_id.0);
        let content = to_canonical_bytes(self);
        self.table_id.0 = saved;
        ctx.set_stable_id("table_id", &mut self.table_id.0, &content, &[]);
    }
}

impl Normalize for Conflict {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(
            2,
            "human_review_question_ja",
            &mut self.human_review_question_ja,
        );
        ctx.normalize_field(
            2,
            "human_review_question_en",
            &mut self.human_review_question_en,
        );
        // Pass 4: sort unordered fields
        ctx.sort_ord("minimal_artifact_set", &mut self.minimal_artifact_set);
        ctx.sort_ord("source_spans", &mut self.source_spans);
        ctx.sort_by_canonical("repair_candidates", &mut self.repair_candidates);
        ctx.sort_by_canonical("solver_evidence", &mut self.solver_evidence);
        // Pass 12: stable ID (Conflict uses source_spans, not source_span_ids)
        let saved = std::mem::take(&mut self.conflict_id.0);
        let content = to_canonical_bytes(self);
        self.conflict_id.0 = saved;
        ctx.set_stable_id(
            "conflict_id",
            &mut self.conflict_id.0,
            &content,
            &self.source_spans,
        );
    }
}

impl Normalize for AssuranceNode {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.normalize_field(2, "claim", &mut self.claim);
        // Pass 4: sort unordered fields
        ctx.sort_ord("evidence_artifact_ids", &mut self.evidence_artifact_ids);
        // Pass 5: children preserves assurance tree structure
    }
}

// ---------------------------------------------------------------------------
// Pass 4 implementations: types with unordered Vec fields to sort
// ---------------------------------------------------------------------------

impl Normalize for PICOFrame {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.sort_ord("exclusions", &mut self.exclusions);
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        // Pass 5: outcomes preserves clinical importance order
    }
}

impl Normalize for EtDFrame {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
    }
}

impl Normalize for EvidenceAtom {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        ctx.sort_ord("table_cell_refs", &mut self.table_cell_refs);
    }
}

impl Normalize for DecisionRow {
    fn normalize(&mut self, ctx: &mut NfContext) {
        // Pass 10: normalize cell Value trees (unit normalization)
        for (i, cond) in self.conditions.iter_mut().enumerate() {
            ctx.normalize_units(&format!("conditions[{i}]"), cond);
        }
        for (i, out) in self.outputs.iter_mut().enumerate() {
            ctx.normalize_units(&format!("outputs[{i}]"), out);
        }
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        ctx.sort_ord("cell_refs", &mut self.cell_refs);
        // Pass 5: conditions, outputs preserve column correspondence
    }
}

impl Normalize for WorkflowFragment {
    fn normalize(&mut self, ctx: &mut NfContext) {
        // Pass 11: stable graph canonicalization by canonical bytes
        ctx.sort_graph("states", &mut self.states);
        ctx.sort_graph("transitions", &mut self.transitions);
        ctx.sort_graph("outcomes", &mut self.outcomes);
        ctx.sort_graph("assessments", &mut self.assessments);
        ctx.sort_graph("tasks", &mut self.tasks);
        ctx.sort_graph("variance_rules", &mut self.variance_rules);
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        // Pass 12: stable ID
        let saved = std::mem::take(&mut self.workflow_id.0);
        let content = to_canonical_bytes(self);
        self.workflow_id.0 = saved;
        ctx.set_stable_id(
            "workflow_id",
            &mut self.workflow_id.0,
            &content,
            &self.source_span_ids,
        );
    }
}

impl Normalize for EventNarrative {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.sort_ord("event_types", &mut self.event_types);
        ctx.sort_ord("fluent_types", &mut self.fluent_types);
        ctx.sort_by_canonical("initially", &mut self.initially);
        ctx.sort_by_canonical("holds_queries", &mut self.holds_queries);
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        // Pass 5: happens, initiates, terminates preserve temporal order
    }
}

impl Normalize for PatientCase {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.sort_by_canonical("facts", &mut self.facts);
        ctx.sort_by_canonical("observations", &mut self.observations);
        ctx.sort_by_canonical("medications", &mut self.medications);
        ctx.sort_by_canonical("conditions", &mut self.conditions);
        ctx.sort_by_canonical("allergies", &mut self.allergies);
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        // Pass 5: events preserves temporal order
        // Pass 12: stable ID
        let saved = std::mem::take(&mut self.case_id.0);
        let content = to_canonical_bytes(self);
        self.case_id.0 = saved;
        ctx.set_stable_id(
            "case_id",
            &mut self.case_id.0,
            &content,
            &self.source_span_ids,
        );
    }
}

impl Normalize for ExecutionWitness {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.sort_by_canonical("context_facts", &mut self.context_facts);
        ctx.sort_ord("applicable_rules", &mut self.applicable_rules);
        ctx.sort_ord("defeated_rules", &mut self.defeated_rules);
        ctx.sort_ord("violated_constraints", &mut self.violated_constraints);
        ctx.sort_by_canonical("models", &mut self.models);
        ctx.sort_by_canonical("unsat_cores", &mut self.unsat_cores);
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        ctx.sort_ord("certificate_ids", &mut self.certificate_ids);
        // Pass 5: trace preserves temporal execution order
        // Pass 12: stable ID
        let saved = std::mem::take(&mut self.witness_id.0);
        let content = to_canonical_bytes(self);
        self.witness_id.0 = saved;
        ctx.set_stable_id(
            "witness_id",
            &mut self.witness_id.0,
            &content,
            &self.source_span_ids,
        );
    }
}

impl Normalize for ArgumentGraph {
    fn normalize(&mut self, ctx: &mut NfContext) {
        // Pass 11: stable graph canonicalization by canonical bytes
        ctx.sort_graph("arguments", &mut self.arguments);
        ctx.sort_graph("attack_edges", &mut self.attack_edges);
        ctx.sort_graph("support_edges", &mut self.support_edges);
        ctx.sort_graph("undercut_edges", &mut self.undercut_edges);
        ctx.sort_graph("defeat_edges", &mut self.defeat_edges);
        ctx.sort_graph("extension_summaries", &mut self.extension_summaries);
        ctx.sort_ord("source_span_ids", &mut self.source_span_ids);
        // Pass 12: stable ID
        let saved = std::mem::take(&mut self.argument_graph_id.0);
        let content = to_canonical_bytes(self);
        self.argument_graph_id.0 = saved;
        ctx.set_stable_id(
            "argument_graph_id",
            &mut self.argument_graph_id.0,
            &content,
            &self.source_span_ids,
        );
    }
}

impl Normalize for Certificate {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.sort_ord("input_artifact_hashes", &mut self.input_artifact_hashes);
        ctx.sort_ord("proof_artifact_hashes", &mut self.proof_artifact_hashes);
        // Pass 5: diagnostics preserves temporal/causal order
    }
}

impl Normalize for AuditTrace {
    fn normalize(&mut self, ctx: &mut NfContext) {
        ctx.sort_ord("artifact_hashes", &mut self.artifact_hashes);
        ctx.sort_ord("audit_export_refs", &mut self.audit_export_refs);
        // Pass 5: stage_spans, model_invocations, retrieval_events,
        //         verifier_events preserve temporal order
    }
}

// ---------------------------------------------------------------------------
// No-op implementations: types without normalizable fields in passes 1-5.
// Pass 3 (alpha-normalization) is identity at schema v0 for types using
// opaque String/Value fields; it activates when typed ASTs replace them.
// ---------------------------------------------------------------------------

macro_rules! normalize_noop {
    ($($ty:ty),+ $(,)?) => {
        $(impl Normalize for $ty {
            fn normalize(&mut self, _ctx: &mut NfContext) {}
        })+
    };
}

normalize_noop!(ExtractorVote, BBox, TableCellRef, ConfidenceInterval);

// ---------------------------------------------------------------------------
// Pass 6-8 implementation: Action domain normalization
// ---------------------------------------------------------------------------

impl Normalize for Action {
    fn normalize(&mut self, ctx: &mut NfContext) {
        // Pass 6: canonical action_type casing (lowercase ASCII)
        ctx.normalize_action_type(&mut self.action_type);
        // Pass 6: parameter keys are sorted by BTreeMap-backed serde_json::Map;
        //         canonical serializer handles RFC 8785 UTF-16 key ordering.
        // Pass 7: normalize unit strings in JSON value fields
        ctx.normalize_units("parameters", &mut self.parameters);
        ctx.normalize_units("temporal_constraints", &mut self.temporal_constraints);
        ctx.normalize_units("quantity_constraints", &mut self.quantity_constraints);
        // Pass 8: resolve target_concept through e-graph canonical representative
        let resolved = ctx
            .resolve_concept(self.target_concept.as_str())
            .map(str::to_owned);
        if let Some(canonical) = resolved
            && canonical != self.target_concept.as_str()
        {
            ctx.rewrites.push(NfRewrite {
                pass: 8,
                field: "target_concept".into(),
                before: std::mem::replace(&mut self.target_concept, ConceptId::new(&canonical)).0,
                after: canonical,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::ContentHash;
    use crate::enums::*;
    use crate::id::*;
    use crate::profile::SemanticProfile;

    // -- normalize_text unit tests --

    #[test]
    fn already_normal_unchanged() {
        assert_eq!(normalize_text("敗血症"), "敗血症");
        assert_eq!(normalize_text("hello world"), "hello world");
    }

    #[test]
    fn fullwidth_ascii_to_halfwidth() {
        assert_eq!(normalize_text("ＡＢＣＤ"), "ABCD");
        assert_eq!(normalize_text("１２３"), "123");
        assert_eq!(normalize_text("（ＩＶ）"), "(IV)");
    }

    #[test]
    fn halfwidth_katakana_to_fullwidth() {
        assert_eq!(normalize_text("ｶﾀｶﾅ"), "カタカナ");
    }

    #[test]
    fn ideographic_space_to_ascii() {
        assert_eq!(normalize_text("敗血症\u{3000}治療"), "敗血症 治療");
    }

    #[test]
    fn whitespace_collapse() {
        assert_eq!(normalize_text("a  b   c"), "a b c");
        assert_eq!(normalize_text("  leading"), "leading");
        assert_eq!(normalize_text("trailing  "), "trailing");
    }

    #[test]
    fn mixed_whitespace_types() {
        assert_eq!(normalize_text("a\t\n\u{3000}b"), "a b");
    }

    #[test]
    fn mixed_japanese_normalization() {
        assert_eq!(
            normalize_text("βラクタム系\u{3000}抗菌薬（ＩＶ投与）"),
            "βラクタム系 抗菌薬(IV投与)"
        );
    }

    #[test]
    fn fullwidth_digits_in_title() {
        assert_eq!(
            normalize_text("日本版敗血症診療ガイドライン\u{3000}２０２４"),
            "日本版敗血症診療ガイドライン 2024"
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(normalize_text(""), "");
    }

    #[test]
    fn only_whitespace() {
        assert_eq!(normalize_text("   \t  \u{3000}  "), "");
    }

    // -- SourceSpan: raw preserved, derived normalized --

    #[test]
    fn source_span_raw_preserved_derived_normalized() {
        let raw = "βラクタム系\u{3000}抗菌薬（ＩＶ）";
        let mut span = SourceSpan {
            span_id: SpanId::new("span_test"),
            doc_id: DocId::new("doc_test"),
            section_path: vec![],
            cq_id: None,
            page: None,
            bbox: None,
            table_cell: None,
            raw_text: raw.into(),
            nfkc_text: raw.into(),
            search_text: raw.into(),
            display_text: raw.into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![],
            confidence: 1.0,
        };

        let mut ctx = NfContext::new();
        span.normalize(&mut ctx);

        assert_eq!(span.raw_text, raw);
        let expected = "βラクタム系 抗菌薬(IV)";
        assert_eq!(span.nfkc_text, expected);
        assert_eq!(span.search_text, expected);
        assert_eq!(span.display_text, expected);
        assert_eq!(ctx.rewrites.len(), 3);
        assert!(ctx.rewrites.iter().all(|r| r.pass == 2));
    }

    #[test]
    fn source_span_no_rewrite_when_already_normal() {
        let text = "敗血症にはβラクタム系抗菌薬を投与する";
        let mut span = SourceSpan {
            span_id: SpanId::new("span_test"),
            doc_id: DocId::new("doc_test"),
            section_path: vec![],
            cq_id: None,
            page: None,
            bbox: None,
            table_cell: None,
            raw_text: text.into(),
            nfkc_text: text.into(),
            search_text: text.into(),
            display_text: text.into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![],
            confidence: 1.0,
        };

        let mut ctx = NfContext::new();
        span.normalize(&mut ctx);

        assert!(ctx.rewrites.is_empty());
    }

    // -- ExtractorVote: raw preserved --

    #[test]
    fn extractor_vote_raw_preserved() {
        let raw = "βラクタム系\u{3000}抗菌薬（ＩＶ）";
        let mut vote = ExtractorVote {
            extractor: "pymupdf".into(),
            raw_text: raw.into(),
            confidence: 0.99,
        };

        let mut ctx = NfContext::new();
        vote.normalize(&mut ctx);

        assert_eq!(vote.raw_text, raw);
        assert!(ctx.rewrites.is_empty());
    }

    // -- CorpusDocument: titles normalized --

    #[test]
    fn corpus_document_titles_normalized() {
        let mut doc = CorpusDocument {
            doc_id: DocId::new("doc_test"),
            title_ja: "日本版敗血症診療ガイドライン\u{3000}２０２４".into(),
            title_en: Some("Japanese  Clinical  Practice  Guidelines".into()),
            source_type: "guideline".into(),
            publisher: "test".into(),
            society: "test".into(),
            edition: "2024".into(),
            publication_date: None,
            access_date: None,
            license_status: "permitted".into(),
            content_hash: ContentHash(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
            ),
            extraction_manifest_id: ManifestId::new("manifest_test"),
            supersedes: None,
            superseded_by: None,
        };

        let mut ctx = NfContext::new();
        doc.normalize(&mut ctx);

        assert_eq!(doc.title_ja, "日本版敗血症診療ガイドライン 2024");
        assert_eq!(
            doc.title_en,
            Some("Japanese Clinical Practice Guidelines".into())
        );
        assert_eq!(ctx.rewrites.len(), 2);
    }

    #[test]
    fn corpus_document_none_title_en_skipped() {
        let mut doc = CorpusDocument {
            doc_id: DocId::new("doc_test"),
            title_ja: "テスト".into(),
            title_en: None,
            source_type: "guideline".into(),
            publisher: "test".into(),
            society: "test".into(),
            edition: "1".into(),
            publication_date: None,
            access_date: None,
            license_status: "permitted".into(),
            content_hash: ContentHash(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
            ),
            extraction_manifest_id: ManifestId::new("manifest_test"),
            supersedes: None,
            superseded_by: None,
        };

        let mut ctx = NfContext::new();
        doc.normalize(&mut ctx);

        assert!(ctx.rewrites.is_empty());
        assert!(doc.title_en.is_none());
    }

    // -- Concept: labels and child bindings normalized --

    #[test]
    fn concept_labels_and_bindings_normalized() {
        let mut concept = Concept {
            concept_id: ConceptId::new("concept_test"),
            label_ja: "βラクタム系\u{3000}抗菌薬".into(),
            label_en: Some("Beta-Lactam  Antibiotics".into()),
            semantic_type: "drug_class".into(),
            terminology_bindings: vec![TerminologyBinding {
                system: "MEDIS".into(),
                code: None,
                version: None,
                label: "βラクタム系\u{3000}抗菌薬".into(),
                status: BindingStatus::Exact,
                mapping_relation: "equivalent".into(),
                provenance: "test".into(),
                confidence: 1.0,
                license_status: "permitted".into(),
                valid_from: None,
                valid_to: None,
            }],
            egraph_class_id: None,
            source_span_ids: vec![],
        };

        let mut ctx = NfContext::new();
        concept.normalize(&mut ctx);

        assert_eq!(concept.label_ja, "βラクタム系 抗菌薬");
        assert_eq!(concept.label_en, Some("Beta-Lactam Antibiotics".into()));
        assert_eq!(concept.terminology_bindings[0].label, "βラクタム系 抗菌薬");
        assert_eq!(ctx.rewrites.len(), 4);
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 12 && r.field == "concept_id")
        );
    }

    // -- ClinicalClaim: glosses normalized --

    #[test]
    fn clinical_claim_glosses_normalized() {
        let mut claim = ClinicalClaim {
            claim_id: ClaimId::new("claim_test"),
            claim_type: "recommendation".into(),
            profiles: vec![SemanticProfile::Norm],
            source_span_ids: vec![],
            pico: None,
            etd: None,
            evidence_atoms: vec![],
            rule_ids: vec![],
            decision_table_ids: vec![],
            workflow_fragment_ids: vec![],
            gloss_ja: "βラクタム系\u{3000}抗菌薬の投与を強く推奨する".into(),
            gloss_en: "Beta-lactam  antibiotics  are  strongly  recommended".into(),
            status: "candidate".into(),
        };

        let mut ctx = NfContext::new();
        claim.normalize(&mut ctx);

        assert_eq!(claim.gloss_ja, "βラクタム系 抗菌薬の投与を強く推奨する");
        assert_eq!(
            claim.gloss_en,
            "Beta-lactam antibiotics are strongly recommended"
        );
        assert_eq!(ctx.rewrites.len(), 3);
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 12 && r.field == "claim_id")
        );
    }

    // -- Norm: original_modality_phrase_ja preserved --

    #[test]
    fn norm_preserves_original_modality() {
        let original = "投与を\u{3000}推奨する";
        let mut norm = Norm {
            context: "test".into(),
            direction: RecommendationDirection::For,
            action: Action {
                action_type: "administer".into(),
                target_concept: ConceptId::new("concept_test"),
                parameters: serde_json::Value::Null,
                temporal_constraints: serde_json::Value::Null,
                quantity_constraints: serde_json::Value::Null,
            },
            recommendation_strength: RecommendationStrength::Strong,
            evidence_certainty: EvidenceCertainty::Moderate,
            original_modality_phrase_ja: original.into(),
            deontic_projection: DeonticProjection::Recommended,
            exception_policy: "none".into(),
            prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
        };

        let mut ctx = NfContext::new();
        norm.normalize(&mut ctx);

        assert_eq!(norm.original_modality_phrase_ja, original);
        assert!(ctx.rewrites.is_empty());
    }

    // -- DecisionTable: column labels normalized --

    #[test]
    fn decision_table_columns_normalized() {
        let mut dt = DecisionTable {
            table_id: DecisionTableId::new("dt_test"),
            hit_policy: HitPolicy::Unique,
            input_columns: vec!["体温\u{3000}（℃）".into(), "心拍数".into()],
            output_columns: vec!["アラート\u{3000}レベル".into()],
            rows: vec![],
            source_table_id: None,
            dmn_export_id: None,
            certificate_ids: vec![],
        };

        let mut ctx = NfContext::new();
        dt.normalize(&mut ctx);

        assert_eq!(dt.input_columns[0], "体温 (\u{00B0}C)");
        assert_eq!(dt.input_columns[1], "心拍数");
        assert_eq!(dt.output_columns[0], "アラート レベル");
        assert_eq!(ctx.rewrites.len(), 3);
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 12 && r.field == "table_id")
        );
    }

    // -- ExtractedTable: headers normalized --

    #[test]
    fn extracted_table_headers_normalized() {
        let mut table = ExtractedTable {
            table_id: ExtractedTableId::new("tbl_test"),
            doc_id: DocId::new("doc_test"),
            caption_span_id: None,
            cell_span_ids: vec![],
            row_headers: vec!["体温\u{3000}".into(), "血圧".into()],
            column_headers: vec!["項目\u{3000}名".into()],
            reading_order: vec![],
            extraction_votes: vec![],
            normalized_table_hash: ContentHash(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
            ),
        };

        let mut ctx = NfContext::new();
        table.normalize(&mut ctx);

        assert_eq!(table.row_headers[0], "体温");
        assert_eq!(table.row_headers[1], "血圧");
        assert_eq!(table.column_headers[0], "項目 名");
        assert_eq!(ctx.rewrites.len(), 2);
    }

    // -- normalize_all convenience --

    #[test]
    fn normalize_all_returns_context() {
        let mut span = SourceSpan {
            span_id: SpanId::new("span_test"),
            doc_id: DocId::new("doc_test"),
            section_path: vec![],
            cq_id: None,
            page: None,
            bbox: None,
            table_cell: None,
            raw_text: "raw".into(),
            nfkc_text: "  extra  spaces  ".into(),
            search_text: "ok".into(),
            display_text: "ok".into(),
            language: Language::Ja,
            previous_span_id: None,
            next_span_id: None,
            extractor_votes: vec![],
            confidence: 1.0,
        };

        let ctx = normalize_all(&mut span);

        assert_eq!(span.nfkc_text, "extra spaces");
        assert_eq!(ctx.rewrites.len(), 1);
        assert_eq!(ctx.rewrites[0].field, "nfkc_text");
    }

    // -- Rewrite records before and after values --

    #[test]
    fn rewrite_records_before_and_after() {
        let mut ctx = NfContext::new();
        let mut value = "ＡＢＣ".to_string();
        ctx.normalize_field(2, "test_field", &mut value);

        assert_eq!(value, "ABC");
        assert_eq!(ctx.rewrites.len(), 1);
        assert_eq!(ctx.rewrites[0].before, "ＡＢＣ");
        assert_eq!(ctx.rewrites[0].after, "ABC");
        assert_eq!(ctx.rewrites[0].pass, 2);
        assert_eq!(ctx.rewrites[0].field, "test_field");
    }

    // -- Vec<String> field normalization --

    #[test]
    fn vec_field_indexed_rewrites() {
        let mut ctx = NfContext::new();
        let mut values = vec![
            "ＡＢＣ".to_string(),
            "normal".to_string(),
            "ＸＹＺ".to_string(),
        ];
        ctx.normalize_vec_field(2, "cols", &mut values);

        assert_eq!(values, vec!["ABC", "normal", "XYZ"]);
        assert_eq!(ctx.rewrites.len(), 2);
        assert_eq!(ctx.rewrites[0].field, "cols[0]");
        assert_eq!(ctx.rewrites[1].field, "cols[2]");
    }

    // ===================================================================
    // Pass 3-5 tests: structural normalization
    // ===================================================================

    // -- sort_commutative_operands unit tests --

    #[test]
    fn sort_and_operands() {
        assert_eq!(
            sort_commutative_operands("(dx sepsis) AND (adult patient)"),
            "(adult patient) AND (dx sepsis)"
        );
    }

    #[test]
    fn sort_or_operands() {
        assert_eq!(sort_commutative_operands("C OR B OR A"), "A OR B OR C");
    }

    #[test]
    fn sort_nested_only_top_level() {
        assert_eq!(
            sort_commutative_operands("(D OR C) AND (B OR A)"),
            "(B OR A) AND (D OR C)"
        );
    }

    #[test]
    fn sort_mixed_operators_unchanged() {
        let expr = "A AND B OR C";
        assert_eq!(sort_commutative_operands(expr), expr);
    }

    #[test]
    fn sort_single_operand_unchanged() {
        let expr = "(administer beta_lactam)";
        assert_eq!(sort_commutative_operands(expr), expr);
    }

    #[test]
    fn sort_already_sorted_unchanged() {
        assert_eq!(sort_commutative_operands("A AND B AND C"), "A AND B AND C");
    }

    #[test]
    fn sort_three_and_operands() {
        assert_eq!(sort_commutative_operands("Z AND A AND M"), "A AND M AND Z");
    }

    // -- Gate test: commutative antecedent order → identical NF --

    fn make_rule(antecedent: &str, context: &str, span_ids: Vec<SpanId>) -> Rule {
        Rule {
            rule_id: RuleId::new("rule_test"),
            profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
            kind: RuleKind::Defeasible,
            context: context.into(),
            antecedent: antecedent.into(),
            consequent: "(administer beta_lactam)".into(),
            norm: None,
            priority_over: vec![],
            exceptions: vec![],
            temporal_scope: None,
            population_scope: None,
            source_span_ids: span_ids,
            provenance: "test".into(),
            certificate_ids: vec![],
        }
    }

    #[test]
    fn gate_commutative_antecedent_order_identical_nf() {
        use crate::canonical::{content_hash, to_canonical_bytes};

        let mut rule_a = make_rule(
            "(adult patient) AND (dx sepsis)",
            "adult_patient AND sepsis",
            vec![SpanId::new("span_s1")],
        );
        let mut rule_b = make_rule(
            "(dx sepsis) AND (adult patient)",
            "sepsis AND adult_patient",
            vec![SpanId::new("span_s1")],
        );

        normalize_all(&mut rule_a);
        normalize_all(&mut rule_b);

        let bytes_a = to_canonical_bytes(&rule_a);
        let bytes_b = to_canonical_bytes(&rule_b);
        assert_eq!(
            bytes_a, bytes_b,
            "Rules with swapped commutative antecedent order must produce identical NF bytes"
        );

        assert_eq!(
            content_hash(&rule_a),
            content_hash(&rule_b),
            "Rules with swapped commutative antecedent order must produce identical NF digest"
        );
    }

    #[test]
    fn gate_swapped_span_ids_identical_nf() {
        use crate::canonical::content_hash;

        let mut rule_a = make_rule(
            "test",
            "test",
            vec![SpanId::new("span_z"), SpanId::new("span_a")],
        );
        let mut rule_b = make_rule(
            "test",
            "test",
            vec![SpanId::new("span_a"), SpanId::new("span_z")],
        );

        normalize_all(&mut rule_a);
        normalize_all(&mut rule_b);

        assert_eq!(
            content_hash(&rule_a),
            content_hash(&rule_b),
            "Rules with swapped source_span_ids must produce identical NF digest"
        );
    }

    // -- sort_ord tests --

    #[test]
    fn sort_ord_records_rewrite() {
        let mut ctx = NfContext::new();
        let mut ids = vec![
            SpanId::new("span_z"),
            SpanId::new("span_a"),
            SpanId::new("span_m"),
        ];
        ctx.sort_ord("source_span_ids", &mut ids);

        assert_eq!(
            ids,
            vec![
                SpanId::new("span_a"),
                SpanId::new("span_m"),
                SpanId::new("span_z"),
            ]
        );
        assert_eq!(ctx.rewrites.len(), 1);
        assert_eq!(ctx.rewrites[0].pass, 4);
        assert_eq!(ctx.rewrites[0].field, "source_span_ids");
    }

    #[test]
    fn sort_ord_already_sorted_no_rewrite() {
        let mut ctx = NfContext::new();
        let mut ids = vec![SpanId::new("a"), SpanId::new("b"), SpanId::new("c")];
        ctx.sort_ord("ids", &mut ids);
        assert!(ctx.rewrites.is_empty());
    }

    #[test]
    fn sort_ord_single_element_no_rewrite() {
        let mut ctx = NfContext::new();
        let mut ids = vec![SpanId::new("only")];
        ctx.sort_ord("ids", &mut ids);
        assert!(ctx.rewrites.is_empty());
    }

    // -- sort_by_canonical tests --

    #[test]
    fn sort_by_canonical_values() {
        let mut ctx = NfContext::new();
        let mut vals = vec![serde_json::json!({"z": 1}), serde_json::json!({"a": 2})];
        ctx.sort_by_canonical("vals", &mut vals);

        assert_eq!(vals[0], serde_json::json!({"a": 2}));
        assert_eq!(vals[1], serde_json::json!({"z": 1}));
        assert_eq!(ctx.rewrites.len(), 1);
        assert_eq!(ctx.rewrites[0].pass, 4);
    }

    #[test]
    fn sort_by_canonical_already_sorted_no_rewrite() {
        let mut ctx = NfContext::new();
        let mut vals = vec![serde_json::json!({"a": 1}), serde_json::json!({"b": 2})];
        ctx.sort_by_canonical("vals", &mut vals);
        assert!(ctx.rewrites.is_empty());
    }

    // -- sort_commutative on NfContext --

    #[test]
    fn sort_commutative_records_rewrite() {
        let mut ctx = NfContext::new();
        let mut expr = "(B) AND (A)".to_string();
        ctx.sort_commutative("antecedent", &mut expr);

        assert_eq!(expr, "(A) AND (B)");
        assert_eq!(ctx.rewrites.len(), 1);
        assert_eq!(ctx.rewrites[0].pass, 4);
        assert_eq!(ctx.rewrites[0].field, "antecedent");
        assert_eq!(ctx.rewrites[0].before, "(B) AND (A)");
        assert_eq!(ctx.rewrites[0].after, "(A) AND (B)");
    }

    #[test]
    fn sort_commutative_no_rewrite_when_sorted() {
        let mut ctx = NfContext::new();
        let mut expr = "(A) AND (B)".to_string();
        ctx.sort_commutative("antecedent", &mut expr);
        assert!(ctx.rewrites.is_empty());
    }

    // -- Rule: profiles sorted --

    #[test]
    fn rule_profiles_sorted() {
        let mut rule = make_rule("test", "test", vec![]);
        rule.profiles = vec![SemanticProfile::Defeasible, SemanticProfile::Norm];

        let ctx = normalize_all(&mut rule);

        assert_eq!(
            rule.profiles,
            vec![SemanticProfile::Norm, SemanticProfile::Defeasible]
        );
        assert!(ctx.rewrites.iter().any(|r| r.field == "profiles"));
    }

    // -- Rule: exceptions sorted --

    #[test]
    fn rule_exceptions_sorted() {
        let mut rule = make_rule("test", "test", vec![]);
        rule.exceptions = vec!["z_exception".into(), "a_exception".into()];

        let ctx = normalize_all(&mut rule);

        assert_eq!(rule.exceptions, vec!["a_exception", "z_exception"]);
        assert!(ctx.rewrites.iter().any(|r| r.field == "exceptions"));
    }

    // -- Rule: priority_over preserved (pass 5) --

    #[test]
    fn rule_priority_over_order_preserved() {
        let mut rule = make_rule("test", "test", vec![]);
        rule.priority_over = vec![RuleId::new("rule_z"), RuleId::new("rule_a")];

        normalize_all(&mut rule);

        assert_eq!(
            rule.priority_over,
            vec![RuleId::new("rule_z"), RuleId::new("rule_a")]
        );
    }

    // -- DecisionTable: rows sorted for Unique, preserved for Priority --

    fn make_decision_row(id: &str, cond_val: i64) -> DecisionRow {
        DecisionRow {
            row_id: DecisionRowId::new(id),
            conditions: vec![serde_json::json!({"value": cond_val})],
            outputs: vec![serde_json::json!({"action": id})],
            priority: None,
            source_span_ids: vec![],
            cell_refs: vec![],
        }
    }

    #[test]
    fn decision_table_rows_sorted_for_unique() {
        let mut dt = DecisionTable {
            table_id: DecisionTableId::new("dt_test"),
            hit_policy: HitPolicy::Unique,
            input_columns: vec!["temp".into()],
            output_columns: vec!["action".into()],
            rows: vec![
                make_decision_row("row_z", 39),
                make_decision_row("row_a", 37),
            ],
            source_table_id: None,
            dmn_export_id: None,
            certificate_ids: vec![],
        };

        let ctx = normalize_all(&mut dt);

        assert_eq!(dt.rows[0].row_id.as_str(), "row_a");
        assert_eq!(dt.rows[1].row_id.as_str(), "row_z");
        assert!(ctx.rewrites.iter().any(|r| r.field == "rows"));
    }

    #[test]
    fn decision_table_rows_preserved_for_priority() {
        let mut dt = DecisionTable {
            table_id: DecisionTableId::new("dt_test"),
            hit_policy: HitPolicy::Priority,
            input_columns: vec!["temp".into()],
            output_columns: vec!["action".into()],
            rows: vec![
                make_decision_row("row_z", 39),
                make_decision_row("row_a", 37),
            ],
            source_table_id: None,
            dmn_export_id: None,
            certificate_ids: vec![],
        };

        normalize_all(&mut dt);

        assert_eq!(dt.rows[0].row_id.as_str(), "row_z");
        assert_eq!(dt.rows[1].row_id.as_str(), "row_a");
    }

    #[test]
    fn decision_table_rows_preserved_for_first() {
        let mut dt = DecisionTable {
            table_id: DecisionTableId::new("dt_test"),
            hit_policy: HitPolicy::First,
            input_columns: vec!["temp".into()],
            output_columns: vec!["action".into()],
            rows: vec![
                make_decision_row("row_z", 39),
                make_decision_row("row_a", 37),
            ],
            source_table_id: None,
            dmn_export_id: None,
            certificate_ids: vec![],
        };

        normalize_all(&mut dt);

        assert_eq!(dt.rows[0].row_id.as_str(), "row_z");
        assert_eq!(dt.rows[1].row_id.as_str(), "row_a");
    }

    // -- ArgumentGraph: edges sorted --

    #[test]
    fn argument_graph_edges_sorted() {
        let mut ag = ArgumentGraph {
            argument_graph_id: ArgumentGraphId::new("ag_test"),
            arguments: vec![
                serde_json::json!({"id": "z_arg"}),
                serde_json::json!({"id": "a_arg"}),
            ],
            attack_edges: vec![
                serde_json::json!({"from": "z", "to": "a"}),
                serde_json::json!({"from": "a", "to": "z"}),
            ],
            support_edges: vec![],
            undercut_edges: vec![],
            defeat_edges: vec![],
            extension_summaries: vec![],
            source_span_ids: vec![SpanId::new("span_z"), SpanId::new("span_a")],
        };

        let ctx = normalize_all(&mut ag);

        assert_eq!(ag.arguments[0], serde_json::json!({"id": "a_arg"}));
        assert_eq!(ag.arguments[1], serde_json::json!({"id": "z_arg"}));
        assert_eq!(
            ag.source_span_ids,
            vec![SpanId::new("span_a"), SpanId::new("span_z")]
        );
        assert!(ctx.rewrites.iter().any(|r| r.field == "arguments"));
        assert!(ctx.rewrites.iter().any(|r| r.field == "source_span_ids"));
    }

    // -- EventNarrative: type sets sorted, temporal preserved --

    #[test]
    fn event_narrative_type_sets_sorted_temporal_preserved() {
        let mut en = EventNarrative {
            event_types: vec!["detect_allergy".into(), "administer_drug".into()],
            fluent_types: vec!["drug_active".into(), "allergy_known".into()],
            happens: vec![
                serde_json::json!({"event": "detect_allergy", "time": 0}),
                serde_json::json!({"event": "administer_drug", "time": 10}),
            ],
            initiates: vec![serde_json::json!({"time": 0})],
            terminates: vec![],
            initially: vec![
                serde_json::json!({"fluent": "z_fluent"}),
                serde_json::json!({"fluent": "a_fluent"}),
            ],
            holds_queries: vec![],
            source_span_ids: vec![],
        };

        let ctx = normalize_all(&mut en);

        assert_eq!(en.event_types, vec!["administer_drug", "detect_allergy"]);
        assert_eq!(en.fluent_types, vec!["allergy_known", "drug_active"]);
        // Temporal fields preserved
        assert_eq!(en.happens[0]["event"], "detect_allergy");
        assert_eq!(en.happens[1]["event"], "administer_drug");
        // initially sorted by canonical bytes
        assert_eq!(en.initially[0]["fluent"], "a_fluent");
        assert!(ctx.rewrites.iter().any(|r| r.field == "event_types"));
    }

    // -- WorkflowFragment: graph canonicalized (pass 11) --

    #[test]
    fn workflow_fragment_graph_canonicalized() {
        let mut wf = WorkflowFragment {
            workflow_id: WorkflowId::new("wf_test"),
            workflow_type: "epath".into(),
            states: vec![
                serde_json::json!({"id": "b"}),
                serde_json::json!({"id": "a"}),
            ],
            transitions: vec![
                serde_json::json!({"from": "b", "to": "a"}),
                serde_json::json!({"from": "a", "to": "b"}),
            ],
            outcomes: vec![
                serde_json::json!({"id": "z_outcome"}),
                serde_json::json!({"id": "a_outcome"}),
            ],
            assessments: vec![],
            tasks: vec![],
            variance_rules: vec![],
            source_span_ids: vec![SpanId::new("span_z"), SpanId::new("span_a")],
        };

        let ctx = normalize_all(&mut wf);

        // Pass 11: states sorted by canonical bytes
        assert_eq!(wf.states[0]["id"], "a");
        assert_eq!(wf.states[1]["id"], "b");
        // Pass 11: transitions sorted by canonical bytes
        assert_eq!(wf.transitions[0]["from"], "a");
        assert_eq!(wf.transitions[1]["from"], "b");
        // Pass 11: outcomes sorted
        assert_eq!(wf.outcomes[0]["id"], "a_outcome");
        assert_eq!(wf.outcomes[1]["id"], "z_outcome");
        // source_span_ids sorted (pass 4)
        assert_eq!(
            wf.source_span_ids,
            vec![SpanId::new("span_a"), SpanId::new("span_z")]
        );
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 11 && r.field == "states")
        );
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 11 && r.field == "transitions")
        );
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 11 && r.field == "outcomes")
        );
    }

    // -- ExecutionWitness: trace preserved, sets sorted --

    #[test]
    fn execution_witness_trace_preserved_sets_sorted() {
        let mut ew = ExecutionWitness {
            witness_id: WitnessId::new("w_test"),
            bundle_id: BundleId::new("b_test"),
            case_id: None,
            context_facts: vec![
                serde_json::json!({"fact": "z"}),
                serde_json::json!({"fact": "a"}),
            ],
            trace: vec![
                serde_json::json!({"step": 2}),
                serde_json::json!({"step": 1}),
            ],
            applicable_rules: vec![RuleId::new("rule_z"), RuleId::new("rule_a")],
            defeated_rules: vec![],
            violated_constraints: vec!["z_constraint".into(), "a_constraint".into()],
            models: vec![],
            unsat_cores: vec![],
            source_span_ids: vec![SpanId::new("span_z"), SpanId::new("span_a")],
            certificate_ids: vec![],
        };

        normalize_all(&mut ew);

        // Trace preserved (temporal)
        assert_eq!(ew.trace[0]["step"], 2);
        assert_eq!(ew.trace[1]["step"], 1);
        // Sets sorted
        assert_eq!(ew.context_facts[0]["fact"], "a");
        assert_eq!(
            ew.applicable_rules,
            vec![RuleId::new("rule_a"), RuleId::new("rule_z")]
        );
        assert_eq!(
            ew.violated_constraints,
            vec!["a_constraint", "z_constraint"]
        );
        assert_eq!(
            ew.source_span_ids,
            vec![SpanId::new("span_a"), SpanId::new("span_z")]
        );
    }

    // -- Certificate: hashes sorted, diagnostics preserved --

    #[test]
    fn certificate_hashes_sorted_diagnostics_preserved() {
        let mut cert = Certificate {
            certificate_id: CertificateId::new("cert_test"),
            certificate_class: CertificateClass::C4Executable,
            input_artifact_hashes: vec![
                ContentHash("sha256:zzzz".into()),
                ContentHash("sha256:aaaa".into()),
            ],
            compiler_hash: None,
            solver_or_checker: "z3".into(),
            command_manifest: serde_json::json!({}),
            result: "sat".into(),
            proof_artifact_hashes: vec![],
            replay_status: ReplayStatus::Passed,
            diagnostics: vec![
                serde_json::json!({"order": 2}),
                serde_json::json!({"order": 1}),
            ],
        };

        normalize_all(&mut cert);

        assert_eq!(cert.input_artifact_hashes[0].as_str(), "sha256:aaaa");
        assert_eq!(cert.input_artifact_hashes[1].as_str(), "sha256:zzzz");
        // Diagnostics preserve temporal order
        assert_eq!(cert.diagnostics[0]["order"], 2);
    }

    // -- EvidenceAtom: inner fields sorted --

    #[test]
    fn evidence_atom_inner_fields_sorted() {
        let mut atom = EvidenceAtom {
            evidence_type: "test".into(),
            source_span_ids: vec![SpanId::new("span_z"), SpanId::new("span_a")],
            pico_ref: None,
            effect_measure: None,
            value: None,
            unit: None,
            confidence_interval: None,
            certainty: EvidenceCertainty::Moderate,
            outcome_importance: None,
            table_cell_refs: vec![
                TableCellRef {
                    table_id: ExtractedTableId::new("tbl_b"),
                    row: 0,
                    col: 0,
                },
                TableCellRef {
                    table_id: ExtractedTableId::new("tbl_a"),
                    row: 0,
                    col: 0,
                },
            ],
        };

        normalize_all(&mut atom);

        assert_eq!(
            atom.source_span_ids,
            vec![SpanId::new("span_a"), SpanId::new("span_z")]
        );
        assert_eq!(atom.table_cell_refs[0].table_id.as_str(), "tbl_a");
        assert_eq!(atom.table_cell_refs[1].table_id.as_str(), "tbl_b");
    }

    // -- PatientCase: events preserved, sets sorted --

    #[test]
    fn patient_case_events_preserved_sets_sorted() {
        let mut pc = PatientCase {
            case_id: CaseId::new("case_test"),
            case_type: CaseType::Synthetic,
            facts: vec![
                serde_json::json!({"type": "z_fact"}),
                serde_json::json!({"type": "a_fact"}),
            ],
            events: vec![
                serde_json::json!({"time": 2}),
                serde_json::json!({"time": 1}),
            ],
            observations: vec![],
            medications: vec![],
            conditions: vec![],
            allergies: vec![],
            time_origin: None,
            source_span_ids: vec![SpanId::new("span_z"), SpanId::new("span_a")],
            privacy_status: "synthetic".into(),
        };

        normalize_all(&mut pc);

        // Events preserved (temporal)
        assert_eq!(pc.events[0]["time"], 2);
        // Facts sorted
        assert_eq!(pc.facts[0]["type"], "a_fact");
        assert_eq!(
            pc.source_span_ids,
            vec![SpanId::new("span_a"), SpanId::new("span_z")]
        );
    }

    // ===================================================================
    // Pass 6-8 tests: domain normalization
    // ===================================================================

    fn make_action(action_type: &str, params: serde_json::Value, qty: serde_json::Value) -> Action {
        Action {
            action_type: action_type.into(),
            target_concept: ConceptId::new("concept_test"),
            parameters: params,
            temporal_constraints: serde_json::json!({"onset": "immediate"}),
            quantity_constraints: qty,
        }
    }

    // -- Pass 6: action_type canonical casing --

    #[test]
    fn action_type_lowercased() {
        let mut action = make_action(
            "Administer",
            serde_json::json!({"route": "iv"}),
            serde_json::json!({"min_dose_mg": 1000}),
        );
        let ctx = normalize_all(&mut action);

        assert_eq!(action.action_type, "administer");
        assert!(ctx.rewrites.iter().any(|r| r.pass == 6
            && r.field == "action_type"
            && r.before == "Administer"
            && r.after == "administer"));
    }

    #[test]
    fn action_type_uppercase_lowercased() {
        let mut action = make_action(
            "CONTRAINDICATE",
            serde_json::Value::Null,
            serde_json::Value::Null,
        );
        let ctx = normalize_all(&mut action);

        assert_eq!(action.action_type, "contraindicate");
        assert_eq!(ctx.rewrites.iter().filter(|r| r.pass == 6).count(), 1,);
    }

    #[test]
    fn action_type_already_lowercase_no_rewrite() {
        let mut action = make_action(
            "administer",
            serde_json::json!({"route": "iv"}),
            serde_json::json!({"min_dose_mg": 1000}),
        );
        let ctx = normalize_all(&mut action);

        assert_eq!(action.action_type, "administer");
        assert!(!ctx.rewrites.iter().any(|r| r.pass == 6));
    }

    // -- Pass 7: unit normalization --

    #[test]
    fn canonical_unit_mappings() {
        assert_eq!(canonical_unit("ml"), Some("mL"));
        assert_eq!(canonical_unit("ML"), Some("mL"));
        assert_eq!(canonical_unit("l"), Some("L"));
        assert_eq!(canonical_unit("\u{2103}"), Some("Cel"));
        assert_eq!(canonical_unit("\u{00B0}C"), Some("Cel"));
        assert_eq!(canonical_unit("degC"), Some("Cel"));
        assert_eq!(canonical_unit("degree_celsius"), Some("Cel"));
        assert_eq!(canonical_unit("mmHg"), Some("mm[Hg]"));
        assert_eq!(canonical_unit("bpm"), Some("/min"));
        assert_eq!(canonical_unit("beats/min"), Some("/min"));
        assert_eq!(canonical_unit("mcg"), Some("ug"));
        assert_eq!(canonical_unit("\u{03BC}g"), Some("ug"));
    }

    #[test]
    fn canonical_unit_already_canonical() {
        assert_eq!(canonical_unit("mg"), None);
        assert_eq!(canonical_unit("mL"), None);
        assert_eq!(canonical_unit("L"), None);
        assert_eq!(canonical_unit("Cel"), None);
        assert_eq!(canonical_unit("kg"), None);
    }

    #[test]
    fn canonical_unit_unrecognized() {
        assert_eq!(canonical_unit("unknown_unit"), None);
        assert_eq!(canonical_unit("foobar"), None);
        assert_eq!(canonical_unit(""), None);
    }

    #[test]
    fn normalize_json_units_flat_string() {
        let mut v = serde_json::json!("ml");
        assert!(normalize_json_units(&mut v));
        assert_eq!(v, serde_json::json!("mL"));
    }

    #[test]
    fn normalize_json_units_nested_object() {
        let mut v = serde_json::json!({
            "dose": {"value": 500, "unit": "ml"},
            "temp": {"value": 38.5, "unit": "\u{2103}"}
        });
        assert!(normalize_json_units(&mut v));
        assert_eq!(v["dose"]["unit"], "mL");
        assert_eq!(v["temp"]["unit"], "Cel");
    }

    #[test]
    fn normalize_json_units_array() {
        let mut v = serde_json::json!([
            {"unit": "bpm", "value": 80},
            {"unit": "mmHg", "value": 120}
        ]);
        assert!(normalize_json_units(&mut v));
        assert_eq!(v[0]["unit"], "/min");
        assert_eq!(v[1]["unit"], "mm[Hg]");
    }

    #[test]
    fn normalize_json_units_no_change() {
        let mut v = serde_json::json!({"value": 1000, "unit": "mg"});
        assert!(!normalize_json_units(&mut v));
    }

    #[test]
    fn normalize_json_units_null_unchanged() {
        let mut v = serde_json::Value::Null;
        assert!(!normalize_json_units(&mut v));
    }

    #[test]
    fn action_quantity_unit_normalized() {
        let mut action = make_action(
            "administer",
            serde_json::json!({"route": "iv"}),
            serde_json::json!({"volume": {"value": 500, "unit": "ml"}}),
        );
        let ctx = normalize_all(&mut action);

        assert_eq!(action.quantity_constraints["volume"]["unit"], "mL");
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 7 && r.field == "quantity_constraints")
        );
    }

    #[test]
    fn action_quantity_already_canonical_no_rewrite() {
        let mut action = make_action(
            "administer",
            serde_json::json!({"route": "iv"}),
            serde_json::json!({"min_dose_mg": 1000}),
        );
        let ctx = normalize_all(&mut action);

        assert!(!ctx.rewrites.iter().any(|r| r.pass == 7));
    }

    #[test]
    fn action_temporal_unit_normalized() {
        let mut action = Action {
            action_type: "administer".into(),
            target_concept: ConceptId::new("concept_test"),
            parameters: serde_json::Value::Null,
            temporal_constraints: serde_json::json!({"monitor_temp_unit": "degC"}),
            quantity_constraints: serde_json::Value::Null,
        };
        let ctx = normalize_all(&mut action);

        assert_eq!(action.temporal_constraints["monitor_temp_unit"], "Cel");
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 7 && r.field == "temporal_constraints")
        );
    }

    // -- Gate tests: domain normalization produces identical NF --

    #[test]
    fn gate_action_case_variants_identical_nf() {
        use crate::canonical::{content_hash, to_canonical_bytes};

        let mut a = make_action(
            "administer",
            serde_json::json!({"route": "iv"}),
            serde_json::json!({"min_dose_mg": 1000}),
        );
        let mut b = make_action(
            "ADMINISTER",
            serde_json::json!({"route": "iv"}),
            serde_json::json!({"min_dose_mg": 1000}),
        );

        normalize_all(&mut a);
        normalize_all(&mut b);

        assert_eq!(
            to_canonical_bytes(&a),
            to_canonical_bytes(&b),
            "Actions differing only in action_type casing must produce identical NF bytes"
        );
        assert_eq!(
            content_hash(&a),
            content_hash(&b),
            "Actions differing only in action_type casing must produce identical NF digest"
        );
    }

    #[test]
    fn gate_action_unit_variants_identical_nf() {
        use crate::canonical::{content_hash, to_canonical_bytes};

        let mut a = make_action(
            "administer",
            serde_json::Value::Null,
            serde_json::json!({"volume": {"value": 500, "unit": "mL"}}),
        );
        let mut b = make_action(
            "administer",
            serde_json::Value::Null,
            serde_json::json!({"volume": {"value": 500, "unit": "ml"}}),
        );

        normalize_all(&mut a);
        normalize_all(&mut b);

        assert_eq!(
            to_canonical_bytes(&a),
            to_canonical_bytes(&b),
            "Actions differing only in unit representation must produce identical NF bytes"
        );
        assert_eq!(
            content_hash(&a),
            content_hash(&b),
            "Actions differing only in unit representation must produce identical NF digest"
        );
    }

    #[test]
    fn gate_action_mixed_variants_identical_nf() {
        use crate::canonical::content_hash;

        let mut a = make_action(
            "administer",
            serde_json::json!({"route": "iv"}),
            serde_json::json!({"temp_unit": "Cel", "bp_unit": "mm[Hg]"}),
        );
        let mut b = make_action(
            "Administer",
            serde_json::json!({"route": "iv"}),
            serde_json::json!({"temp_unit": "\u{2103}", "bp_unit": "mmHg"}),
        );

        normalize_all(&mut a);
        normalize_all(&mut b);

        assert_eq!(
            content_hash(&a),
            content_hash(&b),
            "Actions differing in casing and units must produce identical NF digest"
        );
    }

    // -- Pass 6-7 through Norm and Rule delegation --

    #[test]
    fn norm_delegates_action_normalization() {
        let mut norm = Norm {
            context: "test".into(),
            direction: RecommendationDirection::For,
            action: Action {
                action_type: "Administer".into(),
                target_concept: ConceptId::new("concept_test"),
                parameters: serde_json::Value::Null,
                temporal_constraints: serde_json::Value::Null,
                quantity_constraints: serde_json::json!({"unit": "ml"}),
            },
            recommendation_strength: RecommendationStrength::Strong,
            evidence_certainty: EvidenceCertainty::Moderate,
            original_modality_phrase_ja: "投与を推奨する".into(),
            deontic_projection: DeonticProjection::Recommended,
            exception_policy: "none".into(),
            prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
        };

        let ctx = normalize_all(&mut norm);

        assert_eq!(norm.action.action_type, "administer");
        assert_eq!(norm.action.quantity_constraints["unit"], "mL");
        assert!(ctx.rewrites.iter().any(|r| r.pass == 6));
        assert!(ctx.rewrites.iter().any(|r| r.pass == 7));
    }

    #[test]
    fn rule_delegates_action_normalization_through_norm() {
        let mut rule = Rule {
            rule_id: RuleId::new("rule_test"),
            profiles: vec![SemanticProfile::Norm],
            kind: RuleKind::Defeasible,
            context: "test".into(),
            antecedent: "test".into(),
            consequent: "test".into(),
            norm: Some(Norm {
                context: "test".into(),
                direction: RecommendationDirection::For,
                action: Action {
                    action_type: "ADMINISTER".into(),
                    target_concept: ConceptId::new("concept_test"),
                    parameters: serde_json::Value::Null,
                    temporal_constraints: serde_json::Value::Null,
                    quantity_constraints: serde_json::Value::Null,
                },
                recommendation_strength: RecommendationStrength::Strong,
                evidence_certainty: EvidenceCertainty::Moderate,
                original_modality_phrase_ja: "投与を推奨する".into(),
                deontic_projection: DeonticProjection::Recommended,
                exception_policy: "none".into(),
                prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
            }),
            priority_over: vec![],
            exceptions: vec![],
            temporal_scope: None,
            population_scope: None,
            source_span_ids: vec![],
            provenance: "test".into(),
            certificate_ids: vec![],
        };

        let ctx = normalize_all(&mut rule);

        assert_eq!(rule.norm.as_ref().unwrap().action.action_type, "administer");
        assert!(ctx.rewrites.iter().any(|r| r.pass == 6));
    }

    // -- Pass 8: terminology normalization --

    /// Build a toy term_map mapping beta-lactam variant concept IDs
    /// to the canonical representative "concept_beta_lactam".
    fn toy_term_map() -> BTreeMap<String, String> {
        let canonical = "concept_beta_lactam".to_string();
        BTreeMap::from([
            ("concept_beta_lactam".into(), canonical.clone()),
            ("concept_bl_variant_katakana".into(), canonical.clone()),
            ("concept_bl_variant_hyphenated".into(), canonical.clone()),
            ("concept_bl_variant_brand".into(), canonical.clone()),
            ("concept_bl_variant_english".into(), canonical),
        ])
    }

    #[test]
    fn pass8_identity_without_term_map() {
        let mut action = make_action(
            "administer",
            serde_json::json!({"route": "iv"}),
            serde_json::json!({"min_dose_mg": 1000}),
        );

        let before_hash = crate::canonical::content_hash(&action);
        normalize_all(&mut action);
        let after_hash = crate::canonical::content_hash(&action);

        assert_eq!(
            before_hash, after_hash,
            "pass 8 must be identity when no term_map is provided"
        );
        assert_eq!(action.target_concept.as_str(), "concept_test");
    }

    #[test]
    fn pass8_resolves_variant_to_canonical() {
        let mut action = Action {
            action_type: "administer".into(),
            target_concept: ConceptId::new("concept_bl_variant_katakana"),
            parameters: serde_json::json!({"route": "iv"}),
            temporal_constraints: serde_json::Value::Null,
            quantity_constraints: serde_json::Value::Null,
        };

        let ctx = normalize_with_terms(&mut action, toy_term_map());

        assert_eq!(action.target_concept.as_str(), "concept_beta_lactam");
        assert!(ctx.rewrites.iter().any(|r| r.pass == 8
            && r.field == "target_concept"
            && r.before == "concept_bl_variant_katakana"
            && r.after == "concept_beta_lactam"));
    }

    #[test]
    fn pass8_canonical_concept_unchanged() {
        let mut action = Action {
            action_type: "administer".into(),
            target_concept: ConceptId::new("concept_beta_lactam"),
            parameters: serde_json::Value::Null,
            temporal_constraints: serde_json::Value::Null,
            quantity_constraints: serde_json::Value::Null,
        };

        let ctx = normalize_with_terms(&mut action, toy_term_map());

        assert_eq!(action.target_concept.as_str(), "concept_beta_lactam");
        assert!(
            !ctx.rewrites.iter().any(|r| r.pass == 8),
            "canonical concept must record no pass 8 rewrite"
        );
    }

    #[test]
    fn pass8_unknown_concept_unchanged() {
        let mut action = Action {
            action_type: "administer".into(),
            target_concept: ConceptId::new("concept_unknown"),
            parameters: serde_json::Value::Null,
            temporal_constraints: serde_json::Value::Null,
            quantity_constraints: serde_json::Value::Null,
        };

        let ctx = normalize_with_terms(&mut action, toy_term_map());

        assert_eq!(action.target_concept.as_str(), "concept_unknown");
        assert!(!ctx.rewrites.iter().any(|r| r.pass == 8));
    }

    #[test]
    fn pass8_propagates_through_norm_and_rule() {
        let mut rule = Rule {
            rule_id: RuleId::new("rule_test"),
            profiles: vec![SemanticProfile::Norm],
            kind: RuleKind::Defeasible,
            context: "test".into(),
            antecedent: "test".into(),
            consequent: "test".into(),
            norm: Some(Norm {
                context: "test".into(),
                direction: RecommendationDirection::For,
                action: Action {
                    action_type: "administer".into(),
                    target_concept: ConceptId::new("concept_bl_variant_hyphenated"),
                    parameters: serde_json::Value::Null,
                    temporal_constraints: serde_json::Value::Null,
                    quantity_constraints: serde_json::Value::Null,
                },
                recommendation_strength: RecommendationStrength::Strong,
                evidence_certainty: EvidenceCertainty::Moderate,
                original_modality_phrase_ja: "投与を推奨する".into(),
                deontic_projection: DeonticProjection::Recommended,
                exception_policy: "none".into(),
                prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
            }),
            priority_over: vec![],
            exceptions: vec![],
            temporal_scope: None,
            population_scope: None,
            source_span_ids: vec![SpanId::new("span_s1")],
            provenance: "test".into(),
            certificate_ids: vec![],
        };

        let ctx = normalize_with_terms(&mut rule, toy_term_map());

        assert_eq!(
            rule.norm.as_ref().unwrap().action.target_concept.as_str(),
            "concept_beta_lactam"
        );
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 8 && r.field == "target_concept")
        );
    }

    #[test]
    fn gate_variant_concepts_produce_identical_nf() {
        use crate::canonical::{content_hash, to_canonical_bytes};

        let make = |concept_id: &str| -> Rule {
            Rule {
                rule_id: RuleId::new("rule_test"),
                profiles: vec![SemanticProfile::Norm, SemanticProfile::Defeasible],
                kind: RuleKind::Defeasible,
                context: "sepsis AND adult_patient".into(),
                antecedent: "(dx sepsis) AND (adult patient)".into(),
                consequent: "(administer beta_lactam)".into(),
                norm: Some(Norm {
                    context: "sepsis in adult patients".into(),
                    direction: RecommendationDirection::For,
                    action: Action {
                        action_type: "administer".into(),
                        target_concept: ConceptId::new(concept_id),
                        parameters: serde_json::json!({"route": "iv"}),
                        temporal_constraints: serde_json::Value::Null,
                        quantity_constraints: serde_json::Value::Null,
                    },
                    recommendation_strength: RecommendationStrength::Strong,
                    evidence_certainty: EvidenceCertainty::Moderate,
                    original_modality_phrase_ja: "投与を推奨する".into(),
                    deontic_projection: DeonticProjection::Recommended,
                    exception_policy: "none".into(),
                    prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
                }),
                priority_over: vec![],
                exceptions: vec![],
                temporal_scope: None,
                population_scope: None,
                source_span_ids: vec![SpanId::new("span_s1")],
                provenance: "test".into(),
                certificate_ids: vec![],
            }
        };

        let term_map = toy_term_map();

        let mut rule_katakana = make("concept_bl_variant_katakana");
        let mut rule_hyphenated = make("concept_bl_variant_hyphenated");
        let mut rule_canonical = make("concept_beta_lactam");
        let mut rule_brand = make("concept_bl_variant_brand");

        normalize_with_terms(&mut rule_katakana, term_map.clone());
        normalize_with_terms(&mut rule_hyphenated, term_map.clone());
        normalize_with_terms(&mut rule_canonical, term_map.clone());
        normalize_with_terms(&mut rule_brand, term_map);

        let bytes_k = to_canonical_bytes(&rule_katakana);
        let bytes_h = to_canonical_bytes(&rule_hyphenated);
        let bytes_c = to_canonical_bytes(&rule_canonical);
        let bytes_b = to_canonical_bytes(&rule_brand);

        assert_eq!(
            bytes_k, bytes_h,
            "katakana and hyphenated variants must produce identical NF bytes"
        );
        assert_eq!(
            bytes_k, bytes_c,
            "variant and canonical must produce identical NF bytes"
        );
        assert_eq!(
            bytes_k, bytes_b,
            "variant and brand must produce identical NF bytes"
        );
        assert_eq!(content_hash(&rule_katakana), content_hash(&rule_hyphenated));
        assert_eq!(content_hash(&rule_katakana), content_hash(&rule_canonical));
    }

    #[test]
    fn pass8_idempotent_with_term_map() {
        use crate::canonical::content_hash;

        let mut action = Action {
            action_type: "administer".into(),
            target_concept: ConceptId::new("concept_bl_variant_katakana"),
            parameters: serde_json::json!({"route": "iv"}),
            temporal_constraints: serde_json::Value::Null,
            quantity_constraints: serde_json::Value::Null,
        };

        let term_map = toy_term_map();
        normalize_with_terms(&mut action, term_map.clone());
        let hash1 = content_hash(&action);

        normalize_with_terms(&mut action, term_map);
        let hash2 = content_hash(&action);

        assert_eq!(
            hash1, hash2,
            "NF(NF(action)) must equal NF(action) with term_map"
        );
    }

    // ===================================================================
    // Pass 9 tests: Japanese clinical modality lexicon
    // ===================================================================

    #[test]
    fn modality_lexicon_recommended_phrases() {
        assert_eq!(
            modality_lexicon("投与を推奨する"),
            Some(DeonticProjection::Recommended)
        );
        assert_eq!(
            modality_lexicon("投与を強く推奨する"),
            Some(DeonticProjection::Recommended)
        );
        assert_eq!(
            modality_lexicon("使用を推奨する"),
            Some(DeonticProjection::Recommended)
        );
        assert_eq!(
            modality_lexicon("使用を提案する"),
            Some(DeonticProjection::Recommended)
        );
    }

    #[test]
    fn modality_lexicon_obligatory_phrases() {
        assert_eq!(
            modality_lexicon("投与すべきである"),
            Some(DeonticProjection::Obligatory)
        );
        assert_eq!(
            modality_lexicon("使用すべきである"),
            Some(DeonticProjection::Obligatory)
        );
    }

    #[test]
    fn modality_lexicon_prohibited_phrases() {
        assert_eq!(
            modality_lexicon("投与してはならない"),
            Some(DeonticProjection::Prohibited)
        );
        assert_eq!(
            modality_lexicon("使用してはならない"),
            Some(DeonticProjection::Prohibited)
        );
        assert_eq!(
            modality_lexicon("禁忌である"),
            Some(DeonticProjection::Prohibited)
        );
        assert_eq!(
            modality_lexicon("投与しないことを推奨する"),
            Some(DeonticProjection::Prohibited)
        );
        assert_eq!(
            modality_lexicon("使用しないことを推奨する"),
            Some(DeonticProjection::Prohibited)
        );
    }

    #[test]
    fn modality_lexicon_permitted_phrases() {
        assert_eq!(
            modality_lexicon("投与を考慮してもよい"),
            Some(DeonticProjection::Permitted)
        );
        assert_eq!(
            modality_lexicon("使用を考慮してもよい"),
            Some(DeonticProjection::Permitted)
        );
    }

    #[test]
    fn modality_lexicon_optional_phrases() {
        assert_eq!(
            modality_lexicon("投与は任意である"),
            Some(DeonticProjection::Optional)
        );
        assert_eq!(
            modality_lexicon("使用は任意である"),
            Some(DeonticProjection::Optional)
        );
    }

    #[test]
    fn modality_lexicon_unrecognized() {
        assert_eq!(modality_lexicon("何か別の表現"), None);
        assert_eq!(modality_lexicon(""), None);
    }

    #[test]
    fn modality_lexicon_fullwidth_tolerance() {
        // Leading/trailing ideographic spaces and fullwidth ASCII are
        // normalized before lookup
        assert_eq!(
            modality_lexicon("\u{3000}投与を推奨する\u{3000}"),
            Some(DeonticProjection::Recommended),
        );
        // Fullwidth katakana in a phrase is NFKC-normalized
        assert_eq!(
            modality_lexicon("禁忌である"),
            Some(DeonticProjection::Prohibited),
        );
    }

    #[test]
    fn norm_pass9_corrects_projection() {
        let mut norm = Norm {
            context: "test".into(),
            direction: RecommendationDirection::For,
            action: Action {
                action_type: "administer".into(),
                target_concept: ConceptId::new("concept_test"),
                parameters: serde_json::Value::Null,
                temporal_constraints: serde_json::Value::Null,
                quantity_constraints: serde_json::Value::Null,
            },
            recommendation_strength: RecommendationStrength::Strong,
            evidence_certainty: EvidenceCertainty::Moderate,
            original_modality_phrase_ja: "投与すべきである".into(),
            // Intentionally wrong: phrase says obligatory, field says recommended
            deontic_projection: DeonticProjection::Recommended,
            exception_policy: "none".into(),
            prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
        };

        let ctx = normalize_all(&mut norm);

        assert_eq!(norm.deontic_projection, DeonticProjection::Obligatory);
        assert_eq!(norm.original_modality_phrase_ja, "投与すべきである");
        assert!(ctx.rewrites.iter().any(|r| r.pass == 9
            && r.field == "deontic_projection"
            && r.before == "recommended"
            && r.after == "obligatory"));
    }

    #[test]
    fn norm_pass9_matching_projection_no_rewrite() {
        let mut norm = Norm {
            context: "test".into(),
            direction: RecommendationDirection::For,
            action: Action {
                action_type: "administer".into(),
                target_concept: ConceptId::new("concept_test"),
                parameters: serde_json::Value::Null,
                temporal_constraints: serde_json::Value::Null,
                quantity_constraints: serde_json::Value::Null,
            },
            recommendation_strength: RecommendationStrength::Strong,
            evidence_certainty: EvidenceCertainty::Moderate,
            original_modality_phrase_ja: "投与を推奨する".into(),
            deontic_projection: DeonticProjection::Recommended,
            exception_policy: "none".into(),
            prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
        };

        let ctx = normalize_all(&mut norm);

        assert_eq!(norm.deontic_projection, DeonticProjection::Recommended);
        assert!(!ctx.rewrites.iter().any(|r| r.pass == 9));
    }

    #[test]
    fn norm_pass9_unrecognized_phrase_no_rewrite() {
        let mut norm = Norm {
            context: "test".into(),
            direction: RecommendationDirection::For,
            action: Action {
                action_type: "administer".into(),
                target_concept: ConceptId::new("concept_test"),
                parameters: serde_json::Value::Null,
                temporal_constraints: serde_json::Value::Null,
                quantity_constraints: serde_json::Value::Null,
            },
            recommendation_strength: RecommendationStrength::Strong,
            evidence_certainty: EvidenceCertainty::Moderate,
            original_modality_phrase_ja: "未知の表現".into(),
            deontic_projection: DeonticProjection::Recommended,
            exception_policy: "none".into(),
            prima_facie_or_all_things_considered: NormCommitment::PrimaFacie,
        };

        let ctx = normalize_all(&mut norm);

        assert_eq!(norm.deontic_projection, DeonticProjection::Recommended);
        assert!(!ctx.rewrites.iter().any(|r| r.pass == 9));
    }

    #[test]
    fn norm_pass9_prohibited_contraindication() {
        let mut norm = Norm {
            context: "allergy_context".into(),
            direction: RecommendationDirection::Against,
            action: Action {
                action_type: "administer".into(),
                target_concept: ConceptId::new("concept_beta_lactam"),
                parameters: serde_json::Value::Null,
                temporal_constraints: serde_json::Value::Null,
                quantity_constraints: serde_json::Value::Null,
            },
            recommendation_strength: RecommendationStrength::Strong,
            evidence_certainty: EvidenceCertainty::High,
            original_modality_phrase_ja: "禁忌である".into(),
            deontic_projection: DeonticProjection::Recommended,
            exception_policy: "absolute".into(),
            prima_facie_or_all_things_considered: NormCommitment::AllThingsConsidered,
        };

        let ctx = normalize_all(&mut norm);

        assert_eq!(norm.deontic_projection, DeonticProjection::Prohibited);
        assert!(ctx.rewrites.iter().any(|r| r.pass == 9));
    }

    // ===================================================================
    // Pass 10 tests: decision table cell normalization
    // ===================================================================

    #[test]
    fn decision_row_cell_units_normalized() {
        let mut row = DecisionRow {
            row_id: DecisionRowId::new("row_test"),
            conditions: vec![
                serde_json::json!({"field": "temperature", "unit": "degC", "op": ">=", "value": 38.0}),
                serde_json::json!({"field": "heart_rate", "unit": "bpm", "op": ">=", "value": 90}),
            ],
            outputs: vec![serde_json::json!({"dose": {"value": 500, "unit": "ml"}})],
            priority: None,
            source_span_ids: vec![],
            cell_refs: vec![],
        };

        let ctx = normalize_all(&mut row);

        assert_eq!(row.conditions[0]["unit"], "Cel");
        assert_eq!(row.conditions[1]["unit"], "/min");
        assert_eq!(row.outputs[0]["dose"]["unit"], "mL");
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 7 && r.field == "conditions[0]")
        );
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 7 && r.field == "conditions[1]")
        );
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 7 && r.field == "outputs[0]")
        );
    }

    #[test]
    fn decision_row_canonical_units_no_rewrite() {
        let mut row = DecisionRow {
            row_id: DecisionRowId::new("row_test"),
            conditions: vec![
                serde_json::json!({"field": "temperature", "unit": "Cel", "value": 38.0}),
            ],
            outputs: vec![serde_json::json!({"dose": {"value": 500, "unit": "mL"}})],
            priority: None,
            source_span_ids: vec![],
            cell_refs: vec![],
        };

        let ctx = normalize_all(&mut row);

        assert!(!ctx.rewrites.iter().any(|r| r.pass == 7));
    }

    #[test]
    fn decision_table_cell_normalization_through_rows() {
        let mut dt = DecisionTable {
            table_id: DecisionTableId::new("dt_test"),
            hit_policy: HitPolicy::Unique,
            input_columns: vec!["temp".into()],
            output_columns: vec!["action".into()],
            rows: vec![DecisionRow {
                row_id: DecisionRowId::new("row_a"),
                conditions: vec![serde_json::json!({"temp_unit": "degC"})],
                outputs: vec![serde_json::json!({"dose_unit": "ml"})],
                priority: None,
                source_span_ids: vec![],
                cell_refs: vec![],
            }],
            source_table_id: None,
            dmn_export_id: None,
            certificate_ids: vec![],
        };

        let ctx = normalize_all(&mut dt);

        assert_eq!(dt.rows[0].conditions[0]["temp_unit"], "Cel");
        assert_eq!(dt.rows[0].outputs[0]["dose_unit"], "mL");
        assert!(ctx.rewrites.iter().any(|r| r.pass == 7));
    }

    #[test]
    fn gate_decision_table_unit_variants_identical_nf() {
        use crate::canonical::{content_hash, to_canonical_bytes};

        let mut dt_a = DecisionTable {
            table_id: DecisionTableId::new("dt_test"),
            hit_policy: HitPolicy::Unique,
            input_columns: vec!["temp".into()],
            output_columns: vec!["action".into()],
            rows: vec![DecisionRow {
                row_id: DecisionRowId::new("row_1"),
                conditions: vec![serde_json::json!({"unit": "Cel"})],
                outputs: vec![serde_json::json!({"unit": "mL"})],
                priority: None,
                source_span_ids: vec![],
                cell_refs: vec![],
            }],
            source_table_id: None,
            dmn_export_id: None,
            certificate_ids: vec![],
        };
        let mut dt_b = DecisionTable {
            table_id: DecisionTableId::new("dt_test"),
            hit_policy: HitPolicy::Unique,
            input_columns: vec!["temp".into()],
            output_columns: vec!["action".into()],
            rows: vec![DecisionRow {
                row_id: DecisionRowId::new("row_1"),
                conditions: vec![serde_json::json!({"unit": "degC"})],
                outputs: vec![serde_json::json!({"unit": "ml"})],
                priority: None,
                source_span_ids: vec![],
                cell_refs: vec![],
            }],
            source_table_id: None,
            dmn_export_id: None,
            certificate_ids: vec![],
        };

        normalize_all(&mut dt_a);
        normalize_all(&mut dt_b);

        assert_eq!(
            to_canonical_bytes(&dt_a),
            to_canonical_bytes(&dt_b),
            "DecisionTables with unit variants must produce identical NF bytes"
        );
        assert_eq!(
            content_hash(&dt_a),
            content_hash(&dt_b),
            "DecisionTables with unit variants must produce identical NF digest"
        );
    }

    // ===================================================================
    // Pass 11 tests: graph canonicalization
    // ===================================================================

    #[test]
    fn sort_graph_records_pass_11() {
        let mut ctx = NfContext::new();
        let mut vals = vec![
            serde_json::json!({"id": "z"}),
            serde_json::json!({"id": "a"}),
        ];
        ctx.sort_graph("nodes", &mut vals);

        assert_eq!(vals[0], serde_json::json!({"id": "a"}));
        assert_eq!(vals[1], serde_json::json!({"id": "z"}));
        assert_eq!(ctx.rewrites.len(), 1);
        assert_eq!(ctx.rewrites[0].pass, 11);
        assert_eq!(ctx.rewrites[0].field, "nodes");
    }

    #[test]
    fn sort_graph_already_sorted_no_rewrite() {
        let mut ctx = NfContext::new();
        let mut vals = vec![
            serde_json::json!({"id": "a"}),
            serde_json::json!({"id": "z"}),
        ];
        ctx.sort_graph("nodes", &mut vals);
        assert!(ctx.rewrites.is_empty());
    }

    #[test]
    fn argument_graph_uses_pass_11() {
        let mut ag = ArgumentGraph {
            argument_graph_id: ArgumentGraphId::new("ag_test"),
            arguments: vec![
                serde_json::json!({"id": "z_arg"}),
                serde_json::json!({"id": "a_arg"}),
            ],
            attack_edges: vec![
                serde_json::json!({"from": "z", "to": "a"}),
                serde_json::json!({"from": "a", "to": "z"}),
            ],
            support_edges: vec![],
            undercut_edges: vec![],
            defeat_edges: vec![],
            extension_summaries: vec![],
            source_span_ids: vec![],
        };

        let ctx = normalize_all(&mut ag);

        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 11 && r.field == "arguments")
        );
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 11 && r.field == "attack_edges")
        );
    }

    #[test]
    fn gate_argument_graph_shuffled_edges_identical_nf() {
        use crate::canonical::{content_hash, to_canonical_bytes};

        let mut ag_a = ArgumentGraph {
            argument_graph_id: ArgumentGraphId::new("ag_test"),
            arguments: vec![
                serde_json::json!({"id": "arg_1", "claim": "recommend"}),
                serde_json::json!({"id": "arg_2", "claim": "contraindicate"}),
            ],
            attack_edges: vec![
                serde_json::json!({"from": "arg_1", "to": "arg_2"}),
                serde_json::json!({"from": "arg_2", "to": "arg_1"}),
            ],
            support_edges: vec![serde_json::json!({"from": "ev_1", "to": "arg_1"})],
            undercut_edges: vec![],
            defeat_edges: vec![],
            extension_summaries: vec![serde_json::json!({"type": "grounded", "args": ["arg_1"]})],
            source_span_ids: vec![SpanId::new("span_s1"), SpanId::new("span_s2")],
        };
        let mut ag_b = ArgumentGraph {
            argument_graph_id: ArgumentGraphId::new("ag_test"),
            arguments: vec![
                serde_json::json!({"id": "arg_2", "claim": "contraindicate"}),
                serde_json::json!({"id": "arg_1", "claim": "recommend"}),
            ],
            attack_edges: vec![
                serde_json::json!({"from": "arg_2", "to": "arg_1"}),
                serde_json::json!({"from": "arg_1", "to": "arg_2"}),
            ],
            support_edges: vec![serde_json::json!({"from": "ev_1", "to": "arg_1"})],
            undercut_edges: vec![],
            defeat_edges: vec![],
            extension_summaries: vec![serde_json::json!({"type": "grounded", "args": ["arg_1"]})],
            source_span_ids: vec![SpanId::new("span_s2"), SpanId::new("span_s1")],
        };

        normalize_all(&mut ag_a);
        normalize_all(&mut ag_b);

        assert_eq!(
            to_canonical_bytes(&ag_a),
            to_canonical_bytes(&ag_b),
            "ArgumentGraphs with shuffled nodes/edges must produce identical NF bytes"
        );
        assert_eq!(
            content_hash(&ag_a),
            content_hash(&ag_b),
            "ArgumentGraphs with shuffled nodes/edges must produce identical NF digest"
        );
    }

    #[test]
    fn gate_workflow_shuffled_states_identical_nf() {
        use crate::canonical::{content_hash, to_canonical_bytes};

        let mut wf_a = WorkflowFragment {
            workflow_id: WorkflowId::new("wf_test"),
            workflow_type: "epath".into(),
            states: vec![
                serde_json::json!({"id": "triage", "label": "トリアージ"}),
                serde_json::json!({"id": "abx_admin", "label": "抗菌薬投与"}),
                serde_json::json!({"id": "monitoring", "label": "経過観察"}),
            ],
            transitions: vec![
                serde_json::json!({"from": "triage", "to": "abx_admin"}),
                serde_json::json!({"from": "abx_admin", "to": "monitoring"}),
            ],
            outcomes: vec![serde_json::json!({"id": "recovery"})],
            assessments: vec![],
            tasks: vec![serde_json::json!({"id": "blood_culture"})],
            variance_rules: vec![],
            source_span_ids: vec![SpanId::new("span_p1")],
        };
        let mut wf_b = WorkflowFragment {
            workflow_id: WorkflowId::new("wf_test"),
            workflow_type: "epath".into(),
            states: vec![
                serde_json::json!({"id": "monitoring", "label": "経過観察"}),
                serde_json::json!({"id": "triage", "label": "トリアージ"}),
                serde_json::json!({"id": "abx_admin", "label": "抗菌薬投与"}),
            ],
            transitions: vec![
                serde_json::json!({"from": "abx_admin", "to": "monitoring"}),
                serde_json::json!({"from": "triage", "to": "abx_admin"}),
            ],
            outcomes: vec![serde_json::json!({"id": "recovery"})],
            assessments: vec![],
            tasks: vec![serde_json::json!({"id": "blood_culture"})],
            variance_rules: vec![],
            source_span_ids: vec![SpanId::new("span_p1")],
        };

        normalize_all(&mut wf_a);
        normalize_all(&mut wf_b);

        assert_eq!(
            to_canonical_bytes(&wf_a),
            to_canonical_bytes(&wf_b),
            "WorkflowFragments with shuffled states/transitions must produce identical NF bytes"
        );
        assert_eq!(
            content_hash(&wf_a),
            content_hash(&wf_b),
            "WorkflowFragments with shuffled states/transitions must produce identical NF digest"
        );
    }

    // ===================================================================
    // Pass 12 tests: stable ID generation
    // ===================================================================

    #[test]
    fn pass12_rule_gets_stable_id() {
        let mut rule = make_rule(
            "(dx sepsis) AND (adult patient)",
            "sepsis AND adult_patient",
            vec![SpanId::new("span_s1")],
        );
        let original_id = rule.rule_id.as_str().to_string();
        let ctx = normalize_all(&mut rule);

        assert!(rule.rule_id.as_str().starts_with("nf-"));
        assert_ne!(rule.rule_id.as_str(), original_id);
        assert!(
            ctx.rewrites
                .iter()
                .any(|r| r.pass == 12 && r.field == "rule_id")
        );
    }

    #[test]
    fn pass12_stable_id_deterministic() {
        let mut rule_a = make_rule("test", "test", vec![SpanId::new("span_1")]);
        let mut rule_b = make_rule("test", "test", vec![SpanId::new("span_1")]);

        normalize_all(&mut rule_a);
        normalize_all(&mut rule_b);

        assert_eq!(
            rule_a.rule_id, rule_b.rule_id,
            "identical rules must produce identical stable IDs"
        );
    }

    #[test]
    fn pass12_different_content_different_id() {
        let mut rule_a = make_rule("A", "ctx_a", vec![SpanId::new("span_1")]);
        let mut rule_b = make_rule("B", "ctx_b", vec![SpanId::new("span_1")]);

        normalize_all(&mut rule_a);
        normalize_all(&mut rule_b);

        assert_ne!(
            rule_a.rule_id, rule_b.rule_id,
            "rules with different content must produce different stable IDs"
        );
    }

    #[test]
    fn pass12_different_spans_different_id() {
        let mut rule_a = make_rule("test", "test", vec![SpanId::new("span_1")]);
        let mut rule_b = make_rule("test", "test", vec![SpanId::new("span_2")]);

        normalize_all(&mut rule_a);
        normalize_all(&mut rule_b);

        assert_ne!(
            rule_a.rule_id, rule_b.rule_id,
            "rules with different source anchors must produce different stable IDs"
        );
    }

    #[test]
    fn pass12_already_stable_no_rewrite() {
        let mut rule = make_rule("test", "test", vec![SpanId::new("span_1")]);
        normalize_all(&mut rule);
        let stable_id = rule.rule_id.clone();

        let ctx = normalize_all(&mut rule);

        assert_eq!(
            rule.rule_id, stable_id,
            "stable ID must persist across normalizations"
        );
        assert!(
            !ctx.rewrites.iter().any(|r| r.pass == 12),
            "re-normalization with stable ID must record no pass 12 rewrite"
        );
    }

    #[test]
    fn pass12_commutative_variants_same_stable_id() {
        use crate::canonical::content_hash;

        let mut rule_a = make_rule(
            "(adult patient) AND (dx sepsis)",
            "adult_patient AND sepsis",
            vec![SpanId::new("span_s1")],
        );
        let mut rule_b = make_rule(
            "(dx sepsis) AND (adult patient)",
            "sepsis AND adult_patient",
            vec![SpanId::new("span_s1")],
        );

        normalize_all(&mut rule_a);
        normalize_all(&mut rule_b);

        assert_eq!(
            rule_a.rule_id, rule_b.rule_id,
            "commutative antecedent variants must produce identical stable IDs"
        );
        assert_eq!(content_hash(&rule_a), content_hash(&rule_b));
    }

    #[test]
    fn pass12_decision_table_stable_id() {
        let mut dt = DecisionTable {
            table_id: DecisionTableId::new("dt_original"),
            hit_policy: HitPolicy::Unique,
            input_columns: vec!["temp".into()],
            output_columns: vec!["action".into()],
            rows: vec![make_decision_row("row_a", 37)],
            source_table_id: None,
            dmn_export_id: None,
            certificate_ids: vec![],
        };

        normalize_all(&mut dt);

        assert!(dt.table_id.as_str().starts_with("nf-"));
    }

    #[test]
    fn pass12_argument_graph_stable_id() {
        let mut ag = ArgumentGraph {
            argument_graph_id: ArgumentGraphId::new("ag_original"),
            arguments: vec![serde_json::json!({"id": "arg_1"})],
            attack_edges: vec![],
            support_edges: vec![],
            undercut_edges: vec![],
            defeat_edges: vec![],
            extension_summaries: vec![],
            source_span_ids: vec![SpanId::new("span_s1")],
        };

        normalize_all(&mut ag);

        assert!(ag.argument_graph_id.as_str().starts_with("nf-"));
    }

    #[test]
    fn pass12_conflict_stable_id() {
        let mut conflict = Conflict {
            conflict_id: ConflictId::new("original"),
            conflict_type: "norm_contradiction".into(),
            severity: Severity::High,
            confidence: 0.95,
            minimal_artifact_set: vec![],
            source_spans: vec![SpanId::new("span_s1")],
            normalized_view: serde_json::json!({}),
            witness: None,
            repair_candidates: vec![],
            solver_evidence: vec![],
            argument_graph_id: None,
            human_review_question_ja: "テスト".into(),
            human_review_question_en: "test".into(),
            classification: ConflictClassification::TrueConflict,
        };

        normalize_all(&mut conflict);

        assert!(conflict.conflict_id.as_str().starts_with("nf-"));
    }

    #[test]
    fn pass12_stable_id_format_length() {
        let mut rule = make_rule("test", "test", vec![SpanId::new("span_1")]);
        normalize_all(&mut rule);

        let id = rule.rule_id.as_str();
        assert!(id.starts_with("nf-"), "stable ID must start with 'nf-'");
        assert_eq!(id.len(), 35, "stable ID must be 35 chars: 'nf-' + 32 hex");
        assert!(
            id[3..].chars().all(|c| c.is_ascii_hexdigit()),
            "stable ID suffix must be lowercase hex"
        );
    }

    // ===================================================================
    // Pass 13 tests: diagnostic ordering
    // ===================================================================

    #[test]
    fn pass13_diagnostics_sorted() {
        let mut ctx = NfContext::new();
        ctx.diagnostics.push(NfDiagnostic {
            stage: "z_stage".into(),
            code: "a_code".into(),
            message: "msg".into(),
        });
        ctx.diagnostics.push(NfDiagnostic {
            stage: "a_stage".into(),
            code: "z_code".into(),
            message: "msg".into(),
        });
        ctx.diagnostics.push(NfDiagnostic {
            stage: "a_stage".into(),
            code: "a_code".into(),
            message: "z_msg".into(),
        });
        ctx.diagnostics.push(NfDiagnostic {
            stage: "a_stage".into(),
            code: "a_code".into(),
            message: "a_msg".into(),
        });

        ctx.sort_diagnostics();

        assert_eq!(ctx.diagnostics[0].stage, "a_stage");
        assert_eq!(ctx.diagnostics[0].code, "a_code");
        assert_eq!(ctx.diagnostics[0].message, "a_msg");
        assert_eq!(ctx.diagnostics[1].message, "z_msg");
        assert_eq!(ctx.diagnostics[2].code, "z_code");
        assert_eq!(ctx.diagnostics[3].stage, "z_stage");
    }

    #[test]
    fn pass13_empty_diagnostics_no_error() {
        let mut ctx = NfContext::new();
        ctx.sort_diagnostics();
        assert!(ctx.diagnostics.is_empty());
    }

    // ===================================================================
    // NF idempotency tests: NF(NF(x)) == NF(x)
    // ===================================================================

    #[test]
    fn idempotent_rule() {
        use crate::canonical::content_hash;

        let mut rule = make_rule(
            "(dx sepsis) AND (adult patient)",
            "sepsis AND adult_patient",
            vec![SpanId::new("span_z"), SpanId::new("span_a")],
        );
        normalize_all(&mut rule);
        let hash_after_first = content_hash(&rule);
        let bytes_after_first = to_canonical_bytes(&rule);

        normalize_all(&mut rule);
        let hash_after_second = content_hash(&rule);
        let bytes_after_second = to_canonical_bytes(&rule);

        assert_eq!(
            hash_after_first, hash_after_second,
            "NF(NF(rule)) must equal NF(rule)"
        );
        assert_eq!(bytes_after_first, bytes_after_second);
    }

    #[test]
    fn idempotent_clinical_claim() {
        use crate::canonical::content_hash;

        let mut claim = ClinicalClaim {
            claim_id: ClaimId::new("claim_test"),
            claim_type: "recommendation".into(),
            profiles: vec![SemanticProfile::Norm, SemanticProfile::Evidence],
            source_span_ids: vec![SpanId::new("span_z"), SpanId::new("span_a")],
            pico: None,
            etd: None,
            evidence_atoms: vec![],
            rule_ids: vec![RuleId::new("rule_z"), RuleId::new("rule_a")],
            decision_table_ids: vec![],
            workflow_fragment_ids: vec![],
            gloss_ja: "βラクタム系\u{3000}抗菌薬".into(),
            gloss_en: "Beta-lactam  antibiotics".into(),
            status: "candidate".into(),
        };
        normalize_all(&mut claim);
        let hash1 = content_hash(&claim);

        normalize_all(&mut claim);
        let hash2 = content_hash(&claim);

        assert_eq!(hash1, hash2, "NF(NF(claim)) must equal NF(claim)");
    }

    #[test]
    fn idempotent_decision_table() {
        use crate::canonical::content_hash;

        let mut dt = DecisionTable {
            table_id: DecisionTableId::new("dt_test"),
            hit_policy: HitPolicy::Unique,
            input_columns: vec!["体温\u{3000}".into()],
            output_columns: vec!["action".into()],
            rows: vec![
                make_decision_row("row_z", 39),
                make_decision_row("row_a", 37),
            ],
            source_table_id: None,
            dmn_export_id: None,
            certificate_ids: vec![],
        };
        normalize_all(&mut dt);
        let hash1 = content_hash(&dt);

        normalize_all(&mut dt);
        let hash2 = content_hash(&dt);

        assert_eq!(
            hash1, hash2,
            "NF(NF(decision_table)) must equal NF(decision_table)"
        );
    }

    #[test]
    fn idempotent_argument_graph() {
        use crate::canonical::content_hash;

        let mut ag = ArgumentGraph {
            argument_graph_id: ArgumentGraphId::new("ag_test"),
            arguments: vec![
                serde_json::json!({"id": "z_arg"}),
                serde_json::json!({"id": "a_arg"}),
            ],
            attack_edges: vec![
                serde_json::json!({"from": "z", "to": "a"}),
                serde_json::json!({"from": "a", "to": "z"}),
            ],
            support_edges: vec![],
            undercut_edges: vec![],
            defeat_edges: vec![],
            extension_summaries: vec![],
            source_span_ids: vec![SpanId::new("span_z"), SpanId::new("span_a")],
        };
        normalize_all(&mut ag);
        let hash1 = content_hash(&ag);

        normalize_all(&mut ag);
        let hash2 = content_hash(&ag);

        assert_eq!(
            hash1, hash2,
            "NF(NF(argument_graph)) must equal NF(argument_graph)"
        );
    }

    #[test]
    fn idempotent_workflow_fragment() {
        use crate::canonical::content_hash;

        let mut wf = WorkflowFragment {
            workflow_id: WorkflowId::new("wf_test"),
            workflow_type: "epath".into(),
            states: vec![
                serde_json::json!({"id": "b"}),
                serde_json::json!({"id": "a"}),
            ],
            transitions: vec![],
            outcomes: vec![],
            assessments: vec![],
            tasks: vec![],
            variance_rules: vec![],
            source_span_ids: vec![SpanId::new("span_2"), SpanId::new("span_1")],
        };
        normalize_all(&mut wf);
        let hash1 = content_hash(&wf);

        normalize_all(&mut wf);
        let hash2 = content_hash(&wf);

        assert_eq!(hash1, hash2, "NF(NF(workflow)) must equal NF(workflow)");
    }

    #[test]
    fn idempotent_conflict() {
        use crate::canonical::content_hash;

        let mut conflict = Conflict {
            conflict_id: ConflictId::new("conflict_test"),
            conflict_type: "norm_contradiction".into(),
            severity: Severity::High,
            confidence: 0.9,
            minimal_artifact_set: vec![],
            source_spans: vec![SpanId::new("span_z"), SpanId::new("span_a")],
            normalized_view: serde_json::json!({}),
            witness: None,
            repair_candidates: vec![],
            solver_evidence: vec![],
            argument_graph_id: None,
            human_review_question_ja: "テスト\u{3000}質問".into(),
            human_review_question_en: "test  question".into(),
            classification: ConflictClassification::TrueConflict,
        };
        normalize_all(&mut conflict);
        let hash1 = content_hash(&conflict);

        normalize_all(&mut conflict);
        let hash2 = content_hash(&conflict);

        assert_eq!(hash1, hash2, "NF(NF(conflict)) must equal NF(conflict)");
    }

    #[test]
    fn idempotent_patient_case() {
        use crate::canonical::content_hash;

        let mut pc = PatientCase {
            case_id: CaseId::new("case_test"),
            case_type: CaseType::Synthetic,
            facts: vec![
                serde_json::json!({"type": "z_fact"}),
                serde_json::json!({"type": "a_fact"}),
            ],
            events: vec![serde_json::json!({"time": 1})],
            observations: vec![],
            medications: vec![],
            conditions: vec![],
            allergies: vec![],
            time_origin: None,
            source_span_ids: vec![SpanId::new("span_1")],
            privacy_status: "synthetic".into(),
        };
        normalize_all(&mut pc);
        let hash1 = content_hash(&pc);

        normalize_all(&mut pc);
        let hash2 = content_hash(&pc);

        assert_eq!(
            hash1, hash2,
            "NF(NF(patient_case)) must equal NF(patient_case)"
        );
    }

    #[test]
    fn idempotent_execution_witness() {
        use crate::canonical::content_hash;

        let mut ew = ExecutionWitness {
            witness_id: WitnessId::new("w_test"),
            bundle_id: BundleId::new("b_test"),
            case_id: None,
            context_facts: vec![
                serde_json::json!({"fact": "z"}),
                serde_json::json!({"fact": "a"}),
            ],
            trace: vec![serde_json::json!({"step": 1})],
            applicable_rules: vec![RuleId::new("rule_z"), RuleId::new("rule_a")],
            defeated_rules: vec![],
            violated_constraints: vec![],
            models: vec![],
            unsat_cores: vec![],
            source_span_ids: vec![SpanId::new("span_z"), SpanId::new("span_a")],
            certificate_ids: vec![],
        };
        normalize_all(&mut ew);
        let hash1 = content_hash(&ew);

        normalize_all(&mut ew);
        let hash2 = content_hash(&ew);

        assert_eq!(
            hash1, hash2,
            "NF(NF(execution_witness)) must equal NF(execution_witness)"
        );
    }

    #[test]
    fn idempotent_concept() {
        use crate::canonical::content_hash;

        let mut concept = Concept {
            concept_id: ConceptId::new("concept_test"),
            label_ja: "βラクタム系\u{3000}抗菌薬".into(),
            label_en: Some("Beta-Lactam  Antibiotics".into()),
            semantic_type: "drug_class".into(),
            terminology_bindings: vec![],
            egraph_class_id: None,
            source_span_ids: vec![SpanId::new("span_1")],
        };
        normalize_all(&mut concept);
        let hash1 = content_hash(&concept);

        normalize_all(&mut concept);
        let hash2 = content_hash(&concept);

        assert_eq!(hash1, hash2, "NF(NF(concept)) must equal NF(concept)");
    }
}
