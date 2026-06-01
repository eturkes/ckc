//! Scaffold gate for task 0.8.2: the toy bundle loads every fixture into its
//! `ckc-core` type with the expected cardinalities, and `replay_command`
//! renders the whole-portfolio `ckc compile` invocation that downstream goldens
//! bake into each `CompiledTarget`.

use ckc_compile::{CompileBundle, replay_command};

#[test]
fn load_toy_parses_all_fixtures_with_expected_counts() {
    let bundle = CompileBundle::load_toy();
    assert_eq!(bundle.rules.len(), 3, "rules");
    assert_eq!(bundle.concepts.len(), 10, "concepts");
    assert_eq!(bundle.decision_tables.len(), 1, "decision_tables");
    assert_eq!(bundle.event_narratives.len(), 1, "event_narratives");
    assert_eq!(bundle.conflicts.len(), 3, "conflicts");
    assert_eq!(bundle.spans.len(), 16, "spans");
}

#[test]
fn replay_command_is_whole_portfolio_compile() {
    assert_eq!(replay_command(), "ckc compile examples/research_kernel");
}
