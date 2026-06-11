# A4 solver gate: z3 runner + group verdict ladder. Contract: poc/DESIGN.md.
import os
import subprocess
import tempfile

CANON_DECLS = (
    "(declare-const age Int)\n"
    "(declare-const pregnant Bool)\n"
    "(declare-const renal_impairment Bool)\n"
    "(declare-const hepatic_impairment Bool)\n"
    "(declare-const on_anticoagulant Bool)"
)


def run_z3(smt_text, timeout_s=5):
    fd, path = tempfile.mkstemp(suffix=".smt2")
    try:
        with os.fdopen(fd, "w") as f:
            f.write(smt_text)
        proc = subprocess.run(
            ["z3", "-T:%d" % int(timeout_s), path], capture_output=True, text=True
        )
    finally:
        os.unlink(path)
    lines = proc.stdout.splitlines()
    first = lines[0].strip() if lines else ""
    status = first if first in ("sat", "unsat", "unknown") else "error"
    if any(ln.lstrip().startswith("(error") for ln in lines):
        status = "error"
    return {"status": status, "stdout": proc.stdout, "stderr": proc.stderr}


def group_verdict(rule_a, rule_b):
    if rule_a is None or rule_b is None:
        return {"verdict": "incomputable", "reason": "member_inadmissible", "z3": None}
    if rule_a["action"] != rule_b["action"]:
        return {"verdict": "no_conflict", "reason": "different_action", "z3": None}
    if rule_a["direction"] == rule_b["direction"]:
        return {"verdict": "no_conflict", "reason": "same_direction", "z3": None}
    query = (
        CANON_DECLS
        + "\n(assert " + rule_a["formula"] + ")"
        + "\n(assert " + rule_b["formula"] + ")"
        + "\n(check-sat)\n"
    )
    res = run_z3(query)
    z3_info = dict(res, query=query)
    if res["status"] == "sat":
        return {"verdict": "conflict", "reason": "overlap_sat", "z3": z3_info}
    if res["status"] == "unsat":
        return {"verdict": "no_conflict", "reason": "overlap_unsat", "z3": z3_info}
    return {"verdict": "incomputable", "reason": "solver_error", "z3": z3_info}
