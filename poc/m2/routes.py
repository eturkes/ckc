# M2 PoC routes (A3, revision 2-3): parsers/compilers, per-stage JSON schemas,
# route stage table, prompt builders for all nine routes. Stdlib only. ASCII
# only; Japanese enters prompts solely via vocab ja fields and the {JA_TEXT}
# slot. Slots ({JA_TEXT}, {PRIOR_JSON}, {FRAME_JSON}) consume with .replace,
# never .format. Contract: poc/DESIGN.md.

import hashlib
import json
import re

from m2 import grammars

CANON_VARS = ("age", "pregnant", "renal_impairment", "hepatic_impairment",
              "on_anticoagulant")
OPS = (">=", "<=", ">", "<", "=")

# rev-3 DSL parse regexes. terse: no spaces around =/ops; kw: spaces, RULE/GUARD,
# ' AND '. Both head shapes share _DSL_HEAD (surface/ground reuse it too).
_DSL_HEAD = re.compile(r"^(forbid|require) (\S+)$")
_DSL_T_AGE = re.compile(r"^age(>=|<=|>|<|=)(\d+)$")
_DSL_T_BOOL = re.compile(r"^([a-z_]+)=(true|false)$")
_DSL_K_RULE = re.compile(r"^RULE (forbid|require) (\S+)$")
_DSL_K_GUARD = re.compile(r"^GUARD (.+)$")
_DSL_K_AGE = re.compile(r"^age (>=|<=|>|<|=) (\d+)$")
_DSL_K_BOOL = re.compile(r"^([a-z_]+) = (true|false)$")

_DEFINE_RE = re.compile(r"\(define-fun\s+applies\s*\(\s*\)\s*Bool\b")
_META_RE = re.compile(r";\s*action=(\S+)\s+direction=(\S+)")
_TOKEN_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def _strip_fences(text):
    return "\n".join(l for l in text.split("\n")
                     if not l.lstrip().startswith("```"))


def _used_vars(formula):
    seen = []
    for tok in _TOKEN_RE.findall(formula):
        if tok in CANON_VARS and tok not in seen:
            seen.append(tok)
    return seen


def parse_direct(text):
    """route.direct_smt model output -> (ParsedRule|None, failure_code|None)."""
    cleaned = _strip_fences(text)
    m = _DEFINE_RE.search(cleaned)
    if m is None:
        return None, "target_parse_error"
    depth = 1  # net open parens inside the matched marker
    i = m.end()
    end = None
    while i < len(cleaned):
        c = cleaned[i]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                end = i
                break
        i += 1
    if end is None:  # unbalanced parens
        return None, "target_parse_error"
    formula = cleaned[m.end():end].strip()
    if not formula:
        return None, "target_parse_error"
    meta = _META_RE.search(cleaned)
    if meta is None:
        return None, "target_parse_error"
    return {"action": meta.group(1), "direction": meta.group(2),
            "formula": formula, "used_vars": _used_vars(formula)}, None


def parse_ir(text):
    """JSON stage model output -> (dict|None, failure_code|None)."""
    try:
        obj = json.loads(text)
    except Exception:
        return None, "target_parse_error"
    if not isinstance(obj, dict):
        return None, "target_parse_error"
    return obj, None


# --- rev-3 DSL parsers: deterministic, total on grammar output, lenient on
# surrounding whitespace. They STRUCTURE only; vocab validation stays in
# admission (_ir_structural_ok), mirroring the parse_ir/admit split.


def _dsl_type_conds(raw_conds, age_re, bool_re):
    """Typed condition dicts from DSL cond tokens; None on any non-match.
    [] input -> [] (empty guard), distinct from None (parse error)."""
    out = []
    for c in raw_conds:
        m = age_re.match(c)
        if m:
            out.append({"var": "age", "op": m.group(1), "value": int(m.group(2))})
            continue
        m = bool_re.match(c)
        if m:
            out.append({"var": m.group(1), "op": "=", "value": m.group(2) == "true"})
            continue
        return None
    return out


def parse_dsl_terse(text):
    """Terse line -> (ir_dict|None, code). Partition on the first ' when '."""
    s = text.strip()
    if " when " in s:
        head, _, guardpart = s.partition(" when ")
        raw_conds = guardpart.split(" & ")
    else:
        head, raw_conds = s, []
    m = _DSL_HEAD.match(head)
    if m is None:
        return None, "target_parse_error"
    conditions = _dsl_type_conds(raw_conds, _DSL_T_AGE, _DSL_T_BOOL)
    if conditions is None:
        return None, "target_parse_error"
    return {"action": m.group(2), "direction": m.group(1),
            "conditions": conditions}, None


def parse_dsl_kw(text):
    """Keyword block -> (ir_dict|None, code). Two lines: RULE then GUARD."""
    lines = text.strip().split("\n")
    if len(lines) != 2:
        return None, "target_parse_error"
    m0 = _DSL_K_RULE.match(lines[0])
    m1 = _DSL_K_GUARD.match(lines[1])
    if m0 is None or m1 is None:
        return None, "target_parse_error"
    body = m1.group(1)
    if body == "none":
        conditions = []
    else:
        conditions = _dsl_type_conds(body.split(" AND "), _DSL_K_AGE, _DSL_K_BOOL)
        if conditions is None:
            return None, "target_parse_error"
    return {"action": m0.group(2), "direction": m0.group(1),
            "conditions": conditions}, None


def _parse_dsl_phrase_line(text, slot_key):
    """surface/ground shared shape: '<dir> <token>' + free ' & ' phrases."""
    s = text.strip()
    if " when " in s:
        head, _, guardpart = s.partition(" when ")
        conds = guardpart.split(" & ")
    else:
        head, conds = s, []
    m = _DSL_HEAD.match(head)
    if m is None:
        return None, "target_parse_error"
    return {"direction": m.group(1), slot_key: m.group(2), "conds": conds}, None


def parse_dsl_surface(text):
    """Surface line -> ({direction, drug, conds}|None, code)."""
    return _parse_dsl_phrase_line(text, "drug")


def parse_dsl_ground(text):
    """Ground line -> ({direction, action, conds}|None, code); action enum-checked
    in admission."""
    return _parse_dsl_phrase_line(text, "action")


def _sval(v):
    if v is True:
        return "true"
    if v is False:
        return "false"
    return str(v)


def compile_ir(ir):
    """Validated IR dict -> ParsedRule. Deterministic."""
    terms = ["({} {} {})".format(c["op"], c["var"], _sval(c["value"]))
             for c in ir["conditions"]]
    if not terms:
        formula = "true"
    elif len(terms) == 1:
        formula = terms[0]
    else:
        formula = "(and {})".format(" ".join(terms))
    return {"action": ir["action"], "direction": ir["direction"],
            "formula": formula, "used_vars": _used_vars(formula)}


def compile_stacked(frame, rows_obj):
    """Validated stacked frame + rows objects -> ir-shaped dict."""
    return {"action": frame["intervention"],
            "direction": {"do": "require", "do_not": "forbid"}[frame["stance"]],
            "conditions": rows_obj["rows"]}


def ir_json_schema(vocab):
    """JSON Schema for IR; enums from dataset vocab. Server grammar + admit recheck."""
    return {
        "type": "object",
        "additionalProperties": False,
        "required": ["action", "direction", "conditions"],
        "properties": {
            "action": {"type": "string",
                       "enum": [a["id"] for a in vocab["actions"]]},
            "direction": {"type": "string",
                          "enum": list(vocab["directions"])},
            "conditions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": False,
                    "required": ["var", "op", "value"],
                    "properties": {
                        "var": {"type": "string",
                                "enum": [v["id"] for v in vocab["variables"]]},
                        "op": {"type": "string", "enum": list(OPS)},
                        "value": {"anyOf": [{"type": "integer"},
                                            {"type": "boolean"}]},
                    },
                },
            },
        },
    }


def _action_enum(vocab):
    return {"type": "string", "enum": [a["id"] for a in vocab["actions"]]}


def _stance_enum():
    return {"type": "string", "enum": ["do", "do_not"]}


def _str_array():
    return {"type": "array", "items": {"type": "string"}}


def stacked_frame_schema(vocab):
    return {
        "type": "object",
        "additionalProperties": False,
        "required": ["population_ja", "intervention", "stance"],
        "properties": {
            "population_ja": {"type": "string"},
            "intervention": _action_enum(vocab),
            "stance": _stance_enum(),
        },
    }


def stacked_rows_schema(vocab):
    return {
        "type": "object",
        "additionalProperties": False,
        "required": ["rows"],
        "properties": {
            "rows": {
                "type": "array",
                "items": ir_json_schema(vocab)["properties"]["conditions"]["items"],
            },
        },
    }


def hop_surface_schema():
    return {
        "type": "object",
        "additionalProperties": False,
        "required": ["drug_ja", "polarity", "when_ja"],
        "properties": {
            "drug_ja": {"type": "string"},
            "polarity": _stance_enum(),
            "when_ja": _str_array(),
        },
    }


def hop_ground_schema(vocab):
    return {
        "type": "object",
        "additionalProperties": False,
        "required": ["action", "polarity", "when_ja"],
        "properties": {
            "action": _action_enum(vocab),
            "polarity": _stance_enum(),
            "when_ja": _str_array(),
        },
    }


def layered_segment_schema():
    return {
        "type": "object",
        "additionalProperties": False,
        "required": ["kind", "drug_ja"],
        "properties": {
            "kind": {"type": "string",
                     "enum": ["recommendation", "contraindication"]},
            "drug_ja": {"type": "string"},
        },
    }


def layered_statement_schema(vocab):
    return {
        "type": "object",
        "additionalProperties": False,
        "required": ["action", "modality", "condition_phrases_ja"],
        "properties": {
            "action": _action_enum(vocab),
            "modality": {"type": "string", "enum": ["must", "must_not"]},
            "condition_phrases_ja": _str_array(),
        },
    }


def route_stages(vocab):
    """-> {route_key: [{"stage", "schema", "grammar", "slots"}, ...]} in call
    order. A JSON stage has grammar None and schema driving response_format; a
    DSL stage has schema None and grammar the GBNF string; direct/main has both
    None. Runner uses schema/grammar on the wire and slots to fill the
    user_template; admit walks the same stage order."""
    ir_schema = ir_json_schema(vocab)
    g = grammars.dsl_stage_grammars(vocab)
    return {
        "direct": [
            {"stage": "main", "schema": None, "grammar": None,
             "slots": ["JA_TEXT"]},
        ],
        "ir": [
            {"stage": "main", "schema": ir_schema, "grammar": None,
             "slots": ["JA_TEXT"]},
        ],
        "stacked": [
            {"stage": "frame", "schema": stacked_frame_schema(vocab),
             "grammar": None, "slots": ["JA_TEXT"]},
            {"stage": "rows", "schema": stacked_rows_schema(vocab),
             "grammar": None, "slots": ["JA_TEXT", "FRAME_JSON"]},
        ],
        "hop": [
            {"stage": "surface", "schema": hop_surface_schema(),
             "grammar": None, "slots": ["JA_TEXT"]},
            {"stage": "ground", "schema": hop_ground_schema(vocab),
             "grammar": None, "slots": ["PRIOR_JSON"]},
            {"stage": "typed", "schema": ir_schema,
             "grammar": None, "slots": ["PRIOR_JSON"]},
        ],
        "layered": [
            {"stage": "segment", "schema": layered_segment_schema(),
             "grammar": None, "slots": ["JA_TEXT"]},
            {"stage": "statement", "schema": layered_statement_schema(vocab),
             "grammar": None, "slots": ["JA_TEXT", "PRIOR_JSON"]},
            {"stage": "rule", "schema": ir_schema,
             "grammar": None, "slots": ["JA_TEXT", "PRIOR_JSON"]},
        ],
        "dsl": [
            {"stage": "main", "schema": None, "grammar": g["dsl"]["main"],
             "slots": ["JA_TEXT"]},
        ],
        "dslh": [
            {"stage": "surface", "schema": None,
             "grammar": g["dslh"]["surface"], "slots": ["JA_TEXT"]},
            {"stage": "ground", "schema": None,
             "grammar": g["dslh"]["ground"], "slots": ["PRIOR_JSON"]},
            {"stage": "typed", "schema": None,
             "grammar": g["dslh"]["typed"], "slots": ["PRIOR_JSON"]},
        ],
        "dslk": [
            {"stage": "main", "schema": None, "grammar": g["dslk"]["main"],
             "slots": ["JA_TEXT"]},
        ],
        "dslkh": [
            {"stage": "surface", "schema": None,
             "grammar": g["dslkh"]["surface"], "slots": ["JA_TEXT"]},
            {"stage": "ground", "schema": None,
             "grammar": g["dslkh"]["ground"], "slots": ["PRIOR_JSON"]},
            {"stage": "typed", "schema": None,
             "grammar": g["dslkh"]["typed"], "slots": ["PRIOR_JSON"]},
        ],
    }


def _vars_block(vocab):
    lines = []
    for v in vocab["variables"]:
        rng = "Int 0..130" if v["smt_type"] == "Int" else "Bool"
        lines.append("{} = {} ({})".format(v["ja"], v["id"], rng))
    return "\n".join(lines)


def _actions_block(vocab):
    return "\n".join("{} = {}".format(a["ja"], a["id"])
                     for a in vocab["actions"])


def _vocab_block(vocab):
    return _vars_block(vocab) + "\n" + _actions_block(vocab)


_DIRECTION_LINE = "Directions: require = must administer; forbid = must not administer."
_OPS_LINE = ("Ops: >= <= > < = for age; = for Bool variables with value true or "
             "false. All conditions AND together; use [] if there are none.")
_DSL_OPS_LINE = ("Ops: >= <= > < = for age; = for Bool variables with value "
                 "true or false.")
_POLARITY_LINE = "Polarity mapping: do -> require, do_not -> forbid."
_MODALITY_LINE = "Modality mapping: must -> require, must_not -> forbid."
_EX_RULE = "Example rule: patients aged 12 or older must receive ibuprofen."
_IR_FORMAT = ('{"action": "<action_id>", "direction": "<require|forbid>", '
              '"conditions": [{"var": "<var_id>", "op": "<op>", "value": <int|true|false>}]}')
_EX_IR = ('{"action": "drug_ibuprofen", "direction": "require", '
          '"conditions": [{"var": "age", "op": ">=", "value": 12}]}')


def build_prompts(vocab):
    """-> {route_key: {stage: {"system": str, "user_template": str}}} for all
    nine routes; single-call routes use stage "main". The five rev-2 routes are
    byte-identical to rev-2; dslh/dslkh share surface + ground prompts and
    diverge at typed. Every stage carries one worked example (require
    drug_ibuprofen if age>=12); multi-stage examples show the example prior
    line/JSON. Japanese in templates comes only from vocab ja fields; consume
    slots with .replace, never .format."""
    head = ("Vocabulary (Japanese = symbol):\n" + _vocab_block(vocab)
            + "\n\n" + _DIRECTION_LINE + "\n")
    actions_head = ("Drugs (Japanese = action id):\n" + _actions_block(vocab)
                    + "\n")
    vars_head = ("Variables (Japanese = variable id):\n" + _vars_block(vocab)
                 + "\n")
    ibu_ja = next(a["ja"] for a in vocab["actions"]
                  if a["id"] == "drug_ibuprofen")
    age_ja = next(v["ja"] for v in vocab["variables"] if v["id"] == "age")
    ex_cond = age_ja + " >= 12"
    ex_frame = ('{"population_ja": "' + ex_cond
                + '", "intervention": "drug_ibuprofen", "stance": "do"}')
    ex_rows = '{"rows": [{"var": "age", "op": ">=", "value": 12}]}'
    ex_surface = ('{"drug_ja": "' + ibu_ja
                  + '", "polarity": "do", "when_ja": ["' + ex_cond + '"]}')
    ex_ground = ('{"action": "drug_ibuprofen", "polarity": "do", '
                 '"when_ja": ["' + ex_cond + '"]}')
    ex_segment = '{"kind": "recommendation", "drug_ja": "' + ibu_ja + '"}'
    ex_statement = ('{"action": "drug_ibuprofen", "modality": "must", '
                    '"condition_phrases_ja": ["' + ex_cond + '"]}')

    direct_user = (
        head
        + "\nOutput exactly this format, nothing else:\n"
        + "(declare-const <var> <Int|Bool>)\n"
        + "(define-fun applies () Bool <formula>)\n"
        + "; action=<action_id> direction=<require|forbid>\n"
        + "\nFormula: conditions like (>= age 65) or (= pregnant true); "
        + "join several with (and ...); use true if there are none.\n"
        + "\nExample rule: patients aged 12 or older must receive ibuprofen.\n"
        + "(declare-const age Int)\n"
        + "(define-fun applies () Bool (>= age 12))\n"
        + "; action=drug_ibuprofen direction=require\n"
        + "\nRule: {JA_TEXT}")
    ir_user = (
        head
        + "\nOutput exactly this format, nothing else:\n"
        + '{"action": "<action_id>", "direction": "<require|forbid>", '
        + '"conditions": [{"var": "<var_id>", "op": "<op>", "value": <int|true|false>}]}\n'
        + "Ops: >= <= > < = for age; = for Bool variables with value true or false. "
        + "All conditions AND together; use [] if there are none.\n"
        + "\nExample rule: patients aged 12 or older must receive ibuprofen.\n"
        + '{"action": "drug_ibuprofen", "direction": "require", '
        + '"conditions": [{"var": "age", "op": ">=", "value": 12}]}\n'
        + "\nRule: {JA_TEXT}")
    frame_user = (
        actions_head
        + "\nStance: do = must administer; do_not = must not administer.\n"
        + "\nExtract the rule frame: population_ja = the patient-condition "
        + "phrase copied from the sentence; intervention = the action id of "
        + "the drug; stance = do or do_not.\n"
        + "\nOutput only the required JSON:\n"
        + '{"population_ja": "<phrase from the sentence>", '
        + '"intervention": "<action_id>", "stance": "<do|do_not>"}\n'
        + "\n" + _EX_RULE + "\n"
        + ex_frame + "\n"
        + "\nRule: {JA_TEXT}")
    rows_user = (
        vars_head
        + "\n" + _OPS_LINE + "\n"
        + "\nType every condition in the rule sentence as one row; the frame "
        + "JSON already carries the drug and stance.\n"
        + "\nOutput only the required JSON:\n"
        + '{"rows": [{"var": "<var_id>", "op": "<op>", "value": <int|true|false>}]}\n'
        + "\n" + _EX_RULE + "\n"
        + "Frame JSON: " + ex_frame + "\n"
        + ex_rows + "\n"
        + "\nRule: {JA_TEXT}\nFrame JSON: {FRAME_JSON}")
    surface_user = (
        "From the Japanese rule sentence extract: drug_ja = the drug name "
        "exactly as written; polarity = do if the drug must be administered, "
        "do_not if it must not; when_ja = every condition phrase copied from "
        "the sentence, [] if there are none.\n"
        + "\nOutput only the required JSON:\n"
        + '{"drug_ja": "<drug name>", "polarity": "<do|do_not>", '
        + '"when_ja": ["<condition phrase>"]}\n'
        + "\n" + _EX_RULE + "\n"
        + ex_surface + "\n"
        + "\nRule: {JA_TEXT}")
    ground_user = (
        actions_head
        + "\nRewrite the input JSON: replace drug_ja with its action id from "
        + "the table above; copy polarity and when_ja unchanged.\n"
        + "\nOutput only the required JSON:\n"
        + '{"action": "<action_id>", "polarity": "<do|do_not>", '
        + '"when_ja": ["<condition phrase>"]}\n'
        + "\n" + _EX_RULE + "\n"
        + "Input JSON: " + ex_surface + "\n"
        + ex_ground + "\n"
        + "\nInput JSON: {PRIOR_JSON}")
    typed_user = (
        vars_head
        + "\n" + _OPS_LINE + "\n"
        + _POLARITY_LINE + "\n"
        + "\nRewrite the input JSON into a typed rule: copy action; map "
        + "polarity to direction; type every when_ja phrase as one condition "
        + "object.\n"
        + "\nOutput only the required JSON:\n"
        + _IR_FORMAT + "\n"
        + "\n" + _EX_RULE + "\n"
        + "Input JSON: " + ex_ground + "\n"
        + _EX_IR + "\n"
        + "\nInput JSON: {PRIOR_JSON}")
    segment_user = (
        "Classify the Japanese rule sentence: kind = recommendation if the "
        "drug must be administered, contraindication if it must not; "
        "drug_ja = the drug name exactly as written.\n"
        + "\nOutput only the required JSON:\n"
        + '{"kind": "<recommendation|contraindication>", "drug_ja": "<drug name>"}\n'
        + "\n" + _EX_RULE + "\n"
        + ex_segment + "\n"
        + "\nRule: {JA_TEXT}")
    statement_user = (
        actions_head
        + "\nFrom the rule sentence and the segment JSON build a statement: "
        + "action = the action id for drug_ja; modality = must for a "
        + "recommendation, must_not for a contraindication; "
        + "condition_phrases_ja = every condition phrase copied from the "
        + "sentence, [] if there are none.\n"
        + "\nOutput only the required JSON:\n"
        + '{"action": "<action_id>", "modality": "<must|must_not>", '
        + '"condition_phrases_ja": ["<condition phrase>"]}\n'
        + "\n" + _EX_RULE + "\n"
        + "Segment JSON: " + ex_segment + "\n"
        + ex_statement + "\n"
        + "\nRule: {JA_TEXT}\nSegment JSON: {PRIOR_JSON}")
    rule_user = (
        head
        + _OPS_LINE + "\n"
        + _MODALITY_LINE + "\n"
        + "\nFrom the rule sentence and the statement JSON build the final "
        + "rule: copy action; map modality to direction; type every condition "
        + "phrase as one condition object.\n"
        + "\nOutput only the required JSON:\n"
        + _IR_FORMAT + "\n"
        + "\n" + _EX_RULE + "\n"
        + "Statement JSON: " + ex_statement + "\n"
        + _EX_IR + "\n"
        + "\nRule: {JA_TEXT}\nStatement JSON: {PRIOR_JSON}")
    dsl_main_user = (
        head
        + "\nOutput exactly one line in this format, nothing else:\n"
        + "<direction> <action_id> [when <cond> & <cond> ...]\n"
        + "Each cond is age<op><0-130> (e.g. age>=12) or <bool_var>=<true|false> "
        + "(e.g. pregnant=true); join conds with ' & '; omit 'when' if there are "
        + "none.\n"
        + _DSL_OPS_LINE + "\n"
        + "\n" + _EX_RULE + "\n"
        + "require drug_ibuprofen when age>=12\n"
        + "\nRule: {JA_TEXT}")
    dslk_main_user = (
        head
        + "\nOutput exactly two lines in this format, nothing else:\n"
        + "RULE <direction> <action_id>\n"
        + "GUARD <cond> AND <cond> ...\n"
        + "Each cond is 'age <op> <0-130>' (e.g. age >= 12) or "
        + "'<bool_var> = <true|false>' (e.g. pregnant = true); join conds with "
        + "' AND '; write GUARD none if there are no conditions.\n"
        + _DSL_OPS_LINE + "\n"
        + "\n" + _EX_RULE + "\n"
        + "RULE require drug_ibuprofen\nGUARD age >= 12\n"
        + "\nRule: {JA_TEXT}")
    dsl_surface_user = (
        "From the Japanese rule sentence write one line in this format:\n"
        + "<direction> <drug as written> [when <phrase> & <phrase> ...]\n"
        + "direction is forbid or require; copy the drug name and each condition "
        + "phrase exactly as written; join phrases with ' & '; omit 'when' if "
        + "there are none.\n"
        + "\n" + _EX_RULE + "\n"
        + "require " + ibu_ja + " when " + ex_cond + "\n"
        + "\nRule: {JA_TEXT}")
    dsl_ground_user = (
        actions_head
        + "\nRewrite the input line: replace the drug name with its action id "
        + "from the table above; keep the direction and every condition phrase "
        + "unchanged.\n"
        + "\n" + _EX_RULE + "\n"
        + "Input line: require " + ibu_ja + " when " + ex_cond + "\n"
        + "require drug_ibuprofen when " + ex_cond + "\n"
        + "\nInput line: {PRIOR_JSON}")
    dslh_typed_user = (
        vars_head
        + "\n" + _DSL_OPS_LINE + "\n"
        + "\nRewrite the input line into a typed rule: keep the direction and "
        + "action; type each condition phrase as age<op><0-130> or "
        + "<bool_var>=<true|false>.\n"
        + "\n" + _EX_RULE + "\n"
        + "Input line: require drug_ibuprofen when " + ex_cond + "\n"
        + "require drug_ibuprofen when age>=12\n"
        + "\nInput line: {PRIOR_JSON}")
    dslkh_typed_user = (
        vars_head
        + "\n" + _DSL_OPS_LINE + "\n"
        + "\nRewrite the input line into a typed rule on two lines:\n"
        + "RULE <direction> <action_id>\n"
        + "GUARD <cond> AND <cond> ...\n"
        + "keep the direction and action; type each condition phrase as "
        + "'age <op> <0-130>' or '<bool_var> = <true|false>'; write GUARD none "
        + "if there are no conditions.\n"
        + "\n" + _EX_RULE + "\n"
        + "Input line: require drug_ibuprofen when " + ex_cond + "\n"
        + "RULE require drug_ibuprofen\nGUARD age >= 12\n"
        + "\nInput line: {PRIOR_JSON}")

    return {
        "direct": {
            "main": {
                "system": ("You translate one Japanese clinical rule into SMT-LIB. "
                           "Output only the required format."),
                "user_template": direct_user,
            },
        },
        "ir": {
            "main": {
                "system": ("You translate one Japanese clinical rule into JSON. "
                           "Output only the required format."),
                "user_template": ir_user,
            },
        },
        "stacked": {
            "frame": {
                "system": ("You extract a clinical rule frame from one Japanese "
                           "rule sentence as JSON. Output only the required JSON."),
                "user_template": frame_user,
            },
            "rows": {
                "system": ("You extract typed condition rows from one Japanese "
                           "rule sentence as JSON. Output only the required JSON."),
                "user_template": rows_user,
            },
        },
        "hop": {
            "surface": {
                "system": ("You extract drug, polarity, and condition phrases "
                           "from one Japanese rule sentence as JSON. Output only "
                           "the required JSON."),
                "user_template": surface_user,
            },
            "ground": {
                "system": ("You ground the drug name in one JSON object to its "
                           "canonical action id. Output only the required JSON."),
                "user_template": ground_user,
            },
            "typed": {
                "system": ("You rewrite one JSON object into a fully typed rule. "
                           "Output only the required JSON."),
                "user_template": typed_user,
            },
        },
        "layered": {
            "segment": {
                "system": ("You classify one Japanese clinical rule sentence as "
                           "JSON. Output only the required JSON."),
                "user_template": segment_user,
            },
            "statement": {
                "system": ("You turn one Japanese clinical rule sentence into a "
                           "semi-typed statement as JSON. Output only the "
                           "required JSON."),
                "user_template": statement_user,
            },
            "rule": {
                "system": ("You turn one Japanese clinical rule sentence into a "
                           "fully typed rule as JSON. Output only the required "
                           "JSON."),
                "user_template": rule_user,
            },
        },
        "dsl": {
            "main": {
                "system": ("You translate one Japanese clinical rule into a "
                           "compact rule line. Output only the required format."),
                "user_template": dsl_main_user,
            },
        },
        "dslh": {
            "surface": {
                "system": ("You extract a direction, drug, and condition "
                           "phrases from one Japanese rule sentence as one line. "
                           "Output only the required format."),
                "user_template": dsl_surface_user,
            },
            "ground": {
                "system": ("You ground the drug name in one rule line to its "
                           "canonical action id. Output only the required format."),
                "user_template": dsl_ground_user,
            },
            "typed": {
                "system": ("You rewrite one rule line into a fully typed rule "
                           "line. Output only the required format."),
                "user_template": dslh_typed_user,
            },
        },
        "dslk": {
            "main": {
                "system": ("You translate one Japanese clinical rule into a "
                           "keyword rule block. Output only the required format."),
                "user_template": dslk_main_user,
            },
        },
        "dslkh": {
            "surface": {
                "system": ("You extract a direction, drug, and condition "
                           "phrases from one Japanese rule sentence as one line. "
                           "Output only the required format."),
                "user_template": dsl_surface_user,
            },
            "ground": {
                "system": ("You ground the drug name in one rule line to its "
                           "canonical action id. Output only the required format."),
                "user_template": dsl_ground_user,
            },
            "typed": {
                "system": ("You rewrite one rule line into a fully typed keyword "
                           "rule block. Output only the required format."),
                "user_template": dslkh_typed_user,
            },
        },
    }


def prompt_sha(prompts):
    """-> {route: sha256 hex over canonical json of that route's prompts}."""
    return {route: hashlib.sha256(
                json.dumps(prompts[route], sort_keys=True,
                           ensure_ascii=True).encode()).hexdigest()
            for route in prompts}


def schema_shas(vocab):
    """-> {route_key: {stage: sha256 hex}} over canonical dumps of each JSON
    stage schema; direct omitted (no JSON schema)."""
    out = {}
    for route, stages in route_stages(vocab).items():
        per = {}
        for st in stages:
            if st["schema"] is not None:
                per[st["stage"]] = hashlib.sha256(
                    json.dumps(st["schema"], sort_keys=True,
                               ensure_ascii=True).encode()).hexdigest()
        if per:
            out[route] = per
    return out
