# A4 admission gates (revision 2) for all five routes via admit_route.
# Contract: poc/DESIGN.md. verdict.py supplies CANON_DECLS/run_z3 unchanged.
from m2 import routes
from m2.verdict import CANON_DECLS, run_z3

ACTIONS = ("drug_aspirin", "drug_warfarin", "drug_ibuprofen", "drug_methotrexate")
DIRECTIONS = ("require", "forbid")
AGE_OPS = (">=", "<=", ">", "<", "=")
STANCES = ("do", "do_not")
KINDS = ("recommendation", "contraindication")
MODALITIES = ("must", "must_not")


def _result(syntactic_valid, admitted, failure_code, failed_stage, rule):
    return {
        "syntactic_valid": syntactic_valid,
        "admitted": admitted,
        "failure_code": failure_code,
        "failed_stage": failed_stage,
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
    """Revision 1 direct gate; failed_stage 'main' on any failure."""
    rule, code = routes.parse_direct(text)
    if rule is None:
        return _result(False, False, code, "main", None)
    if _z3_sanity(rule["formula"])["status"] == "error":
        return _result(False, False, "target_parse_error", "main", None)
    if rule["action"] not in ACTIONS or rule["direction"] not in DIRECTIONS:
        return _result(True, False, "ai_schema_violation", "main", None)
    return _result(True, True, None, None, rule)


def _is_str_list(v):
    return isinstance(v, list) and all(isinstance(x, str) for x in v)


def _cond_ok(cond, var_types):
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
    return all(_cond_ok(c, var_types) for c in ir["conditions"])


def admit_ir(text, vocab):
    """Revision 1 single-IR gate; failed_stage 'main' on any failure."""
    ir, code = routes.parse_ir(text)
    if ir is None:
        return _result(False, False, code, "main", None)
    if not _ir_structural_ok(ir, vocab):
        return _result(True, False, "ai_schema_violation", "main", None)
    rule = routes.compile_ir(ir)
    if _z3_sanity(rule["formula"])["status"] == "error":
        return _result(True, False, "solver_execution_failure", "main", None)
    return _result(True, True, None, None, rule)


def _stage_checks(vocab):
    """-> {route_key: ((stage, structural_check), ...)} for multi-call routes.
    Checks mirror each stage's JSON schema shape (enums/types/ranges);
    cross-stage copy instructions stay unenforced."""
    actions = {a["id"] for a in vocab["actions"]}
    var_types = {v["id"]: v["smt_type"] for v in vocab["variables"]}

    def frame_ok(o):
        return (set(o) == {"population_ja", "intervention", "stance"}
                and isinstance(o["population_ja"], str)
                and o["intervention"] in actions
                and o["stance"] in STANCES)

    def rows_ok(o):
        return (set(o) == {"rows"} and isinstance(o["rows"], list)
                and all(_cond_ok(c, var_types) for c in o["rows"]))

    def surface_ok(o):
        return (set(o) == {"drug_ja", "polarity", "when_ja"}
                and isinstance(o["drug_ja"], str)
                and o["polarity"] in STANCES
                and _is_str_list(o["when_ja"]))

    def ground_ok(o):
        return (set(o) == {"action", "polarity", "when_ja"}
                and o["action"] in actions
                and o["polarity"] in STANCES
                and _is_str_list(o["when_ja"]))

    def segment_ok(o):
        return (set(o) == {"kind", "drug_ja"}
                and o["kind"] in KINDS
                and isinstance(o["drug_ja"], str))

    def statement_ok(o):
        return (set(o) == {"action", "modality", "condition_phrases_ja"}
                and o["action"] in actions
                and o["modality"] in MODALITIES
                and _is_str_list(o["condition_phrases_ja"]))

    def ir_ok(o):
        return _ir_structural_ok(o, vocab)

    return {
        "stacked": (("frame", frame_ok), ("rows", rows_ok)),
        "hop": (("surface", surface_ok), ("ground", ground_ok),
                ("typed", ir_ok)),
        "layered": (("segment", segment_ok), ("statement", statement_ok),
                    ("rule", ir_ok)),
    }


def _admit_multi(route_key, contents, vocab):
    """Uniform multi-stage walk: all-stage JSON parse, then per-stage
    structural checks in order, then final ir-shape -> compile + z3 sanity."""
    spec = _stage_checks(vocab)[route_key]
    texts = list(contents) + [""] * (len(spec) - len(contents))
    objs = []
    for (stage, _), text in zip(spec, texts):
        obj, code = routes.parse_ir(text)
        if obj is None:
            return _result(False, False, code, stage, None)
        objs.append(obj)
    for (stage, check), obj in zip(spec, objs):
        if not check(obj):
            return _result(True, False, "ai_schema_violation", stage, None)
    if route_key == "stacked":
        ir = routes.compile_stacked(objs[0], objs[1])
    else:
        ir = objs[-1]
    rule = routes.compile_ir(ir)
    if _z3_sanity(rule["formula"])["status"] == "error":
        return _result(True, False, "solver_execution_failure", spec[-1][0], None)
    return _result(True, True, None, None, rule)


def admit_dsl(text, vocab, parser):
    """rev-3 singular DSL gate (route.ckc_dsl, route.ckc_dsl_kw); parser is
    parse_dsl_terse or parse_dsl_kw. failed_stage 'main' on any failure."""
    ir, code = parser(text)
    if ir is None:
        return _result(False, False, code, "main", None)
    if not _ir_structural_ok(ir, vocab):
        return _result(True, False, "ai_schema_violation", "main", None)
    rule = routes.compile_ir(ir)
    if _z3_sanity(rule["formula"])["status"] == "error":
        return _result(True, False, "solver_execution_failure", "main", None)
    return _result(True, True, None, None, rule)


def _admit_dsl_hop(contents, vocab, typed_parser):
    """rev-3 hop DSL gate (route.ckc_dsl_hop, route.ckc_dsl_kw_hop): parse
    surface, ground, typed in order (all-parse, then check-all), mirroring
    _admit_multi. typed_parser is parse_dsl_terse or parse_dsl_kw."""
    actions = {a["id"] for a in vocab["actions"]}
    stages = ("surface", "ground", "typed")
    parsers = (routes.parse_dsl_surface, routes.parse_dsl_ground, typed_parser)
    texts = list(contents) + [""] * (len(stages) - len(contents))
    objs = []
    for stage, parser, text in zip(stages, parsers, texts):
        obj, code = parser(text)
        if obj is None:
            return _result(False, False, code, stage, None)
        objs.append(obj)
    if objs[1]["action"] not in actions:
        return _result(True, False, "ai_schema_violation", "ground", None)
    typed_ir = objs[2]
    if not _ir_structural_ok(typed_ir, vocab):
        return _result(True, False, "ai_schema_violation", "typed", None)
    rule = routes.compile_ir(typed_ir)
    if _z3_sanity(rule["formula"])["status"] == "error":
        return _result(True, False, "solver_execution_failure", "typed", None)
    return _result(True, True, None, None, rule)


def admit_route(route_key, contents, vocab):
    """contents = per-stage content strings from a record's calls, stage order.
    -> AdmissionResult {syntactic_valid, admitted, failure_code, failed_stage, rule}."""
    if route_key == "direct":
        return admit_direct(contents[0] if contents else "")
    if route_key == "ir":
        return admit_ir(contents[0] if contents else "", vocab)
    c0 = contents[0] if contents else ""
    if route_key == "dsl":
        return admit_dsl(c0, vocab, routes.parse_dsl_terse)
    if route_key == "dslk":
        return admit_dsl(c0, vocab, routes.parse_dsl_kw)
    if route_key == "dslh":
        return _admit_dsl_hop(contents, vocab, routes.parse_dsl_terse)
    if route_key == "dslkh":
        return _admit_dsl_hop(contents, vocab, routes.parse_dsl_kw)
    return _admit_multi(route_key, contents, vocab)
