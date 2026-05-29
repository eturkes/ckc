//! Gate for task 0.8.5: the SMT-LIB decision-table emitter is byte-deterministic
//! and grounds every row to its match-predicate symbol and table-cell spans.

use ckc_compile::smt::emit_decision_table;
use ckc_compile::{CompileBundle, content_hash};

#[test]
fn emit_decision_table_is_byte_identical_across_runs() {
    let bundle = CompileBundle::load_toy();
    let a = emit_decision_table(&bundle);
    let b = emit_decision_table(&bundle);
    assert_eq!(a.artifact_text, b.artifact_text, "artifact text bytes");
    assert_eq!(content_hash(&a), content_hash(&b), "whole-artifact hash");
}

#[test]
fn emit_decision_table_artifact_has_smt_skeleton() {
    let target = emit_decision_table(&CompileBundle::load_toy());
    assert!(target.artifact_text.contains("(set-logic QF_LRA)"));
    assert!(target.artifact_text.contains("(check-sat)"));
}

#[test]
fn emit_decision_table_grounds_all_rows() {
    let bundle = CompileBundle::load_toy();
    let target = emit_decision_table(&bundle);
    let map = &target.compilation_map.0;

    // One mapping per row, each carrying the row_id as its predicate symbol and
    // the row's own table-cell spans.
    let expected = [
        ("row_temp_high", ["span_cell_r0c0", "span_cell_r0c1"]),
        ("row_temp_very_high", ["span_cell_r1c0", "span_cell_r1c1"]),
        ("row_hr_high", ["span_cell_r2c0", "span_cell_r2c1"]),
        ("row_bp_low", ["span_cell_r3c0", "span_cell_r3c1"]),
    ];
    assert_eq!(map.len(), expected.len(), "one mapping per row");
    for (row_id, spans) in expected {
        let entry = map
            .iter()
            .find(|m| m.ckc_node_id == row_id)
            .unwrap_or_else(|| panic!("mapping for {row_id}"));
        assert_eq!(entry.target_symbol, row_id, "row predicate symbol");
        let got: Vec<&str> = entry.source_span_ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(got, spans, "table-cell spans for {row_id}");
    }

    // source_artifact_hashes is the decision table's content hash — confirms the
    // emitter hashed the table, not its rows.
    let table = bundle
        .decision_tables
        .iter()
        .find(|t| t.table_id.as_str() == "dt_vitals_triage")
        .unwrap();
    assert_eq!(target.source_artifact_hashes, vec![content_hash(table)]);
}
