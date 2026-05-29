//! CKC → target-language compiler portfolio (SPEC 14).
//!
//! Each compiler consumes a normalized [`CompileBundle`] and emits a
//! [`CompiledTarget`]: target-language text, a CKC-node-id → target-symbol
//! map, deterministic diagnostics, source provenance, and a replay command.
//! This crate stays emit-only through Phase-0 task 0.8; solver/kernel
//! execution and certificate assignment happen in task 0.9.

use ckc_core::artifact::{DecisionTable, EventNarrative};
use ckc_core::clinical::Rule;
use ckc_core::source::{Concept, SourceSpan};
use ckc_core::verify::Conflict;

pub use ckc_core::canonical::{ContentHash, content_hash};
pub use ckc_core::compile::{CompilationMap, CompiledTarget, SymbolMapping};
pub use ckc_core::enums::TargetLanguage;

pub mod smt;

const RULES_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/rules.json");
const CONCEPTS_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/concepts.json");
const DECISION_TABLES_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/decision_tables.json");
const EVENT_NARRATIVES_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/event_narratives.json");
const CONFLICTS_JSON: &str =
    include_str!("../../../examples/research_kernel/fixtures/conflicts.json");
const SPANS_JSON: &str = include_str!("../../../examples/research_kernel/fixtures/spans.json");

/// In-memory aggregate of the normalized CKC artifacts a compiler reads.
///
/// Fixture ids are consumed verbatim: a `rule_id` such as
/// `rule_sepsis_bl_recommend` IS the CKC node id every `CompilationMap` entry
/// references, so emitters preserve them rather than re-running normal-form
/// stable-id assignment (which would desync the cross-references carried by
/// `conflicts`/`claims`).
#[derive(Clone, Debug)]
pub struct CompileBundle {
    pub rules: Vec<Rule>,
    pub concepts: Vec<Concept>,
    pub decision_tables: Vec<DecisionTable>,
    pub event_narratives: Vec<EventNarrative>,
    pub conflicts: Vec<Conflict>,
    pub spans: Vec<SourceSpan>,
}

impl CompileBundle {
    /// Load the Phase-0 research-kernel toy bundle from the committed fixtures
    /// embedded at compile time. Panics only when a committed fixture stops
    /// matching its `ckc-core` type, which is a build-time bug rather than a
    /// runtime condition.
    pub fn load_toy() -> Self {
        Self {
            rules: serde_json::from_str(RULES_JSON).expect("toy rules.json must deserialize"),
            concepts: serde_json::from_str(CONCEPTS_JSON)
                .expect("toy concepts.json must deserialize"),
            decision_tables: serde_json::from_str(DECISION_TABLES_JSON)
                .expect("toy decision_tables.json must deserialize"),
            event_narratives: serde_json::from_str(EVENT_NARRATIVES_JSON)
                .expect("toy event_narratives.json must deserialize"),
            conflicts: serde_json::from_str(CONFLICTS_JSON)
                .expect("toy conflicts.json must deserialize"),
            spans: serde_json::from_str(SPANS_JSON).expect("toy spans.json must deserialize"),
        }
    }
}

/// A CKC → target-language compiler (SPEC 14 compiler contract).
///
/// Implementors are emit-only: [`Compiler::compile`] produces target text and
/// its symbol map deterministically from `bundle` without invoking the target
/// solver/checker, which is task 0.9.
pub trait Compiler {
    /// The target language this compiler emits.
    fn target_language(&self) -> TargetLanguage;
    /// Emit the target artifact for `bundle`.
    fn compile(&self, bundle: &CompileBundle) -> CompiledTarget;
}

/// Canonical replay command that regenerates a target artifact through the
/// `ckc` CLI (SPEC 14 replay command, SPEC 18 `ckc compile`). The `--target`
/// token is the snake_case `TargetLanguage` wire form that downstream goldens
/// bake into each `CompiledTarget`.
pub fn replay_command(lang: TargetLanguage) -> String {
    let target = match lang {
        TargetLanguage::SmtLib => "smt_lib",
        TargetLanguage::Asp => "asp",
        TargetLanguage::Datalog => "datalog",
        TargetLanguage::Lean => "lean",
        TargetLanguage::TlaPlus => "tla_plus",
        TargetLanguage::Alloy => "alloy",
    };
    format!("ckc compile examples/research_kernel --target {target}")
}

/// Assemble a [`CompiledTarget`] from an emitter's text and symbol map, filling
/// the emit-only invariants every Phase-0 compiler shares: the canonical
/// [`replay_command`] for `target_language`, an empty diagnostics list, and an
/// absent `target_parse_ok` (the PATH-guarded target parse check belongs to
/// task 0.9). The four passed fields are stored verbatim.
pub fn build_target(
    target_language: TargetLanguage,
    artifact_text: String,
    compilation_map: CompilationMap,
    source_artifact_hashes: Vec<ContentHash>,
) -> CompiledTarget {
    CompiledTarget {
        target_language,
        artifact_text,
        compilation_map,
        diagnostics: Vec::new(),
        source_artifact_hashes,
        replay_command: replay_command(target_language),
        target_parse_ok: None,
    }
}

/// Build a deterministic text block: sort `lines` lexicographically, join them
/// with `'\n'`, and terminate with a single trailing `'\n'`. Every text emitter
/// reuses this for declaration/fact blocks so a block's bytes depend only on its
/// contents, not on the order an emitter discovers them.
pub fn sorted_lines(mut lines: Vec<String>) -> String {
    lines.sort();
    let mut block = lines.join("\n");
    block.push('\n');
    block
}
