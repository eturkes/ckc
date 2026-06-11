# M2 PoC routes (A3): parsers/compiler for route.direct_smt and route.single_ir,
# IR JSON schema, prompt builders. Stdlib only. ASCII only; Japanese enters
# prompts solely via vocab ja fields and the {JA_TEXT} slot (.replace at call time).

import hashlib
import json
import re

CANON_VARS = ("age", "pregnant", "renal_impairment", "hepatic_impairment",
              "on_anticoagulant")
OPS = (">=", "<=", ">", "<", "=")

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
    """route.single_ir model output -> (dict|None, failure_code|None)."""
    try:
        obj = json.loads(text)
    except Exception:
        return None, "target_parse_error"
    if not isinstance(obj, dict):
        return None, "target_parse_error"
    return obj, None


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


def _vocab_block(vocab):
    lines = []
    for v in vocab["variables"]:
        rng = "Int 0..130" if v["smt_type"] == "Int" else "Bool"
        lines.append("{} = {} ({})".format(v["ja"], v["id"], rng))
    for a in vocab["actions"]:
        lines.append("{} = {}".format(a["ja"], a["id"]))
    return "\n".join(lines)


_DIRECTION_LINE = "Directions: require = must administer; forbid = must not administer."


def build_prompts(vocab):
    """-> {"direct": {"system", "user_template"}, "ir": {...}}.
    user_template carries exactly one {JA_TEXT} slot; consume with .replace."""
    head = ("Vocabulary (Japanese = symbol):\n" + _vocab_block(vocab)
            + "\n\n" + _DIRECTION_LINE + "\n")
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
    return {
        "direct": {
            "system": ("You translate one Japanese clinical rule into SMT-LIB. "
                       "Output only the required format."),
            "user_template": direct_user,
        },
        "ir": {
            "system": ("You translate one Japanese clinical rule into JSON. "
                       "Output only the required format."),
            "user_template": ir_user,
        },
    }


def prompt_sha(prompts):
    """-> {route: sha256 hex over canonical json of that route's prompts}."""
    return {route: hashlib.sha256(
                json.dumps(prompts[route], sort_keys=True,
                           ensure_ascii=True).encode()).hexdigest()
            for route in prompts}
