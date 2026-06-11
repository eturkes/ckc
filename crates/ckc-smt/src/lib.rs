//! ckc-smt — FormalIR → SMT-LIB compilation and verification for the
//! Clinical Knowledge Compiler (SPEC §3 crate table: emission, query
//! planning, assertion maps, solver invocation, verdict parsing). This
//! crate owns the two §5 durable payloads plus planning and emission; the
//! Z3 adapter (verify module) lands with its roadmap unit. Surface by
//! module:
//!
//! - `artifact` — compile-stage payload: [`CompiledArtifact`] (target id, §5
//!   query-plan slots, §6 query bodies under [`SmtLogic`], the
//!   named-assertion map of [`AssertionRecord`]s, target metadata,
//!   diagnostics) with structural validation ([`ArtifactError`]).
//! - `emit` — compile-stage emission: [`emit_overlap_query`] /
//!   [`emit_deontic_query`], the §6 byte-pinned (§8.6) query texts of one
//!   planned pair as [`QueryBody`]s.
//! - `plan` — compile-stage planning: [`plan_queries`], the §6 eligibility
//!   scan over a fixture group's per-document FormalIRs, minting the §8.6
//!   pair/query ids into §5 ContradictionQueryPair slots.
//! - `result` — verify-stage payload: [`VerifierResult`] (per-query §6
//!   [`VerifierCategory`], raw [`SolverVerdict`] token kept distinct,
//!   witness model or canonical unsat core, §5 solver identity, diagnostics)
//!   with coherence validation ([`VerifierError`]).
#![forbid(unsafe_code)]

mod artifact;
mod emit;
mod plan;
mod result;

pub use artifact::{ArtifactError, AssertionRecord, CompiledArtifact, QueryBody, SmtLogic};
pub use emit::{emit_deontic_query, emit_overlap_query};
pub use plan::plan_queries;
pub use result::{SolverVerdict, VerifierCategory, VerifierError, VerifierResult};

use ckc_core::Id;

/// How a stored id sequence broke canonical-set order.
pub(crate) enum SetBreak {
    Duplicate,
    Unsorted,
}

/// First canonical-set storage break among `ids`, with the offending
/// element. Stored sets and map keys must be sorted by Id bytes — identical
/// to canonical_sort_key order, since every byte the Id grammar admits sorts
/// above the JSON quote — and duplicate-free, so stored values equal their
/// strict-read round trip (the canonical writer sorts on emission either
/// way).
pub(crate) fn first_set_break<'a>(
    ids: impl IntoIterator<Item = &'a Id>,
) -> Option<(SetBreak, &'a Id)> {
    let mut prev: Option<&Id> = None;
    for id in ids {
        if let Some(p) = prev {
            match p.as_str().cmp(id.as_str()) {
                std::cmp::Ordering::Equal => return Some((SetBreak::Duplicate, id)),
                std::cmp::Ordering::Greater => return Some((SetBreak::Unsorted, id)),
                std::cmp::Ordering::Less => {}
            }
        }
        prev = Some(id);
    }
    None
}
