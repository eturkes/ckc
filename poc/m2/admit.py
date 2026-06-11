# A4 admission gates for both routes. Contract: poc/DESIGN.md.
from m2 import routes
from m2.verdict import CANON_DECLS, run_z3

ACTIONS = ("drug_aspirin", "drug_warfarin", "drug_ibuprofen", "drug_methotrexate")
DIRECTIONS = ("require", "forbid")
AGE_OPS = (">=", "<=", ">", "<", "=")


def _result(syntactic_valid, admitted, failure_code, rule):
    return {
        "syntactic_valid": syntactic_valid,
        "admitted": admitted,
        "failure_code": failure_code,
        "rule": rule,
    }


def _z3_sanity(formula):
    smt = (
        CANON_DECLS
        + "\n(define-fun applies () Bool " + formula + ")"
        + "\n(assert applies)\n(check-sat)\n"
    )
    return run_z3(smt)


def admit_direct(text):
    rule, code = routes.parse_direct(text)
    if rule is None:
        return _result(False, False, code, None)
    if _z3_sanity(rule["formula"])["status"] == "error":
        return _result(False, False, "target_parse_error", None)
    if rule["action"] not in ACTIONS or rule["direction"] not in DIRECTIONS:
        return _result(True, False, "ai_schema_violation", None)
    return _result(True, True, None, rule)


def _ir_structural_ok(ir, vocab):
    var_types = {v["id"]: v["smt_type"] for v in vocab["variables"]}
    actions = {a["id"] for a in vocab["actions"]}
    directions = set(vocab["directions"])
    if set(ir) != {"action", "direction", "conditions"}:
        return False
    if ir["action"] not in actions or ir["direction"] not in directions:
        return False
    if not isinstance(ir["conditions"], list):
        return False
    for cond in ir["conditions"]:
        if not isinstance(cond, dict) or set(cond) != {"var", "op", "value"}:
            return False
        var, op, value = cond["var"], cond["op"], cond["value"]
        if not isinstance(var, str) or var not in var_types:
            return False
        if var_types[var] == "Int":
            if op not in AGE_OPS:
                return False
            if type(value) is not int or not 0 <= value <= 130:
                return False
        else:
            if op != "=" or type(value) is not bool:
                return False
    return True


def admit_ir(text, vocab):
    ir, code = routes.parse_ir(text)
    if ir is None:
        return _result(False, False, code, None)
    if not _ir_structural_ok(ir, vocab):
        return _result(True, False, "ai_schema_violation", None)
    rule = routes.compile_ir(ir)
    if _z3_sanity(rule["formula"])["status"] == "error":
        return _result(True, False, "solver_execution_failure", None)
    return _result(True, True, None, rule)
