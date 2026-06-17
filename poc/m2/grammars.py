# M2 PoC rev-3 (A3): GBNF builders for the four M4 invented-DSL routes, the
# route->stage->grammar map that routes.route_stages embeds and the report
# consumes, and grammar_shas (mirrors routes.schema_shas: JSON stages there,
# grammar stages here). Stdlib only; ASCII source. action/boolvar enums derive
# from dataset vocab so they never drift; direction is the fixed 2-element set
# and ageval the fixed 0..130 range (no leading zeros). The \n in the kw root
# and the surface/ground char class is the two-char GBNF newline escape, so
# those bodies are raw strings. Contract: poc/DESIGN.md "Revision 3".

import hashlib

_DIRECTION = '"forbid" | "require"'
_BOOLVAL = '"true" | "false"'
_AGEOP = '">=" | "<=" | ">" | "<" | "="'
_AGEVAL = '"130" | "12" [0-9] | "1" [0-1] [0-9] | [1-9] [0-9] | [0-9]'
_CHAR = r"[^ &\n]"


def _alt(values):
    return " | ".join('"%s"' % v for v in values)


def _rules(pairs):
    """(name, body) pairs -> GBNF text, one rule per line, ::= aligned at col 10."""
    return "\n".join("%-9s ::= %s" % (name, body) for name, body in pairs)


def _actions(vocab):
    return _alt([a["id"] for a in vocab["actions"]])


def _boolvars(vocab):
    return _alt([v["id"] for v in vocab["variables"] if v["smt_type"] == "Bool"])


def dsl_terse_gbnf(vocab):
    """route.ckc_dsl main + route.ckc_dsl_hop typed: terse infix line.
    boolvar/boolval and age/ageop/ageval coupling makes pregnant>=65 unreachable."""
    return _rules([
        ("root", 'direction " " action guard?'),
        ("guard", '" when " cond (" & " cond)*'),
        ("direction", _DIRECTION),
        ("action", _actions(vocab)),
        ("cond", "boolcond | agecond"),
        ("boolcond", 'boolvar "=" boolval'),
        ("boolvar", _boolvars(vocab)),
        ("boolval", _BOOLVAL),
        ("agecond", '"age" ageop ageval'),
        ("ageop", _AGEOP),
        ("ageval", _AGEVAL),
    ])


def dsl_kw_gbnf(vocab):
    """route.ckc_dsl_kw main + route.ckc_dsl_kw_hop typed: keyword block.
    Same coupling, verbose surface (RULE/GUARD/AND, spaces, a newline)."""
    return _rules([
        ("root", r'"RULE " direction " " action "\n" "GUARD " guardbody'),
        ("guardbody", '"none" | cond (" AND " cond)*'),
        ("direction", _DIRECTION),
        ("action", _actions(vocab)),
        ("cond", "boolcond | agecond"),
        ("boolcond", 'boolvar " = " boolval'),
        ("boolvar", _boolvars(vocab)),
        ("boolval", _BOOLVAL),
        ("agecond", '"age " ageop " " ageval'),
        ("ageop", _AGEOP),
        ("ageval", _AGEVAL),
    ])


def dsl_surface_gbnf():
    """Shared surface stage of both hop routes: drug + phrases are free
    Japanese (raw = any run of non-space, non-&, non-newline characters)."""
    return _rules([
        ("root", 'direction " " raw guard?'),
        ("guard", '" when " raw (" & " raw)*'),
        ("direction", _DIRECTION),
        ("raw", "char+"),
        ("char", _CHAR),
    ])


def dsl_ground_gbnf(vocab):
    """Shared ground stage of both hop routes: the drug slot is the canonical
    action enum; condition phrases stay free Japanese."""
    return _rules([
        ("root", 'direction " " action guard?'),
        ("guard", '" when " raw (" & " raw)*'),
        ("direction", _DIRECTION),
        ("action", _actions(vocab)),
        ("raw", "char+"),
        ("char", _CHAR),
    ])


def dsl_stage_grammars(vocab):
    """{route_key: {stage: gbnf_text}} for the four DSL routes' grammar-bearing
    stages. Single source of truth: routes.route_stages embeds these strings,
    grammar_shas hashes them. dslh/dslkh share surface + ground, diverge at typed."""
    terse = dsl_terse_gbnf(vocab)
    kw = dsl_kw_gbnf(vocab)
    surface = dsl_surface_gbnf()
    ground = dsl_ground_gbnf(vocab)
    return {
        "dsl": {"main": terse},
        "dslh": {"surface": surface, "ground": ground, "typed": terse},
        "dslk": {"main": kw},
        "dslkh": {"surface": surface, "ground": ground, "typed": kw},
    }


def grammar_shas(vocab):
    """{route_key: {stage: sha256 hex}} over each DSL stage's GBNF text."""
    return {route: {stage: hashlib.sha256(text.encode()).hexdigest()
                    for stage, text in stages.items()}
            for route, stages in dsl_stage_grammars(vocab).items()}
