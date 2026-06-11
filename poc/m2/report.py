"""M2 PoC report rendering: markdown + canonical bytes from the report dict.

Design: poc/DESIGN.md (A5). Pure functions of the report dict; no timestamps.
Generated file: JA literals below are unicode escape sequences; bytes are ASCII.
"""
import json

METRIC_ORDER = ("syntactic_validity", "admission_rate", "verdict_accuracy_greedy", "verdict_stability")
ROUTE_ID_ORDER = ("route.direct_smt", "route.single_ir")

L_EN = {
    "title": "M2 short-hop PoC report",
    "identity": "Identity",
    "gold_gate": "Gold gate",
    "pass": "PASS",
    "fail": "FAIL",
    "groups_word": "groups",
    "metrics": "Metrics",
    "metric": "metric",
    "baseline_delta": "Baseline delta",
    "delta": "delta",
    "matrix": "Group verdict matrix",
    "group": "group",
    "kind": "kind",
    "gold": "gold",
    "taxonomy": "Failure taxonomy",
    "code": "code",
    "findings": "Findings",
    "quote_a": "quote A",
    "quote_b": "quote B",
    "replay": "Replay",
}

# --- generated JA literals (ASCII unicode escapes) ---
VERDICT_JA = {'conflict': '\u77db\u76fe', 'no_conflict': '\u77db\u76fe\u306a\u3057', 'incomputable': '\u8a08\u7b97\u4e0d\u80fd'}
METRIC_JA = {'syntactic_validity': '\u69cb\u6587\u59a5\u5f53\u7387', 'admission_rate': '\u53d7\u7406\u7387', 'verdict_accuracy_greedy': '\u5224\u5b9a\u7cbe\u5ea6(\u30b0\u30ea\u30fc\u30c7\u30a3)', 'verdict_stability': '\u5224\u5b9a\u5b89\u5b9a\u6027'}
JA_FINDING = '\u6b63\u89e3\u306f%s\u3002direct\u7d4c\u8def\u306f%s\u3068\u5224\u5b9a\u3001IR\u7d4c\u8def\u306f%s\u3068\u5224\u5b9a\u3002'
L_JA = {'title': 'M2 \u30b7\u30e7\u30fc\u30c8\u30db\u30c3\u30d7 PoC \u30ec\u30dd\u30fc\u30c8', 'identity': '\u8b58\u5225\u60c5\u5831', 'gold_gate': '\u30b4\u30fc\u30eb\u30c9\u30b2\u30fc\u30c8', 'pass': '\u5408\u683c', 'fail': '\u4e0d\u5408\u683c', 'groups_word': '\u30b0\u30eb\u30fc\u30d7', 'metrics': '\u6307\u6a19', 'metric': '\u6307\u6a19', 'baseline_delta': '\u30d9\u30fc\u30b9\u30e9\u30a4\u30f3\u5dee\u5206', 'delta': '\u5dee\u5206', 'matrix': '\u30b0\u30eb\u30fc\u30d7\u5224\u5b9a\u30de\u30c8\u30ea\u30af\u30b9', 'group': '\u30b0\u30eb\u30fc\u30d7', 'kind': '\u7a2e\u5225', 'gold': '\u6b63\u89e3', 'taxonomy': '\u5931\u6557\u5206\u985e', 'code': '\u30b3\u30fc\u30c9', 'findings': '\u6240\u898b', 'quote_a': '\u5f15\u7528A', 'quote_b': '\u5f15\u7528B', 'replay': '\u30ea\u30d7\u30ec\u30a4'}

L = {"en": L_EN, "ja": L_JA}


def canonical_bytes(report):
    """Exact bytes of report.json per the design canonical-bytes rule."""
    return (json.dumps(report, ensure_ascii=True, sort_keys=True, indent=1) + "\n").encode()


def finding_en(gold, direct_v, ir_v):
    return "gold %s; direct route said %s; IR route said %s" % (gold, direct_v, ir_v)


def finding_ja(gold, direct_v, ir_v):
    return JA_FINDING % (
        VERDICT_JA.get(gold, gold),
        VERDICT_JA.get(direct_v, direct_v),
        VERDICT_JA.get(ir_v, ir_v))


def _vd(v, lang):
    return VERDICT_JA.get(v, v) if lang == "ja" else v


def _mark(v, gold, lang):
    s = _vd(v, lang)
    return s + "*" if v != gold else s


def _mname(name, lang):
    return METRIC_JA.get(name, name) if lang == "ja" else name


def render_md(report, lang):
    assert lang in L, lang
    t = L[lang]
    idn = report["identities"]
    out = []
    out.append("# %s - %s" % (t["title"], report["run_id"]))
    out.append("")
    out.append("## " + t["identity"])
    out.append("")
    out.append("- model_name: %s" % idn["model_name"])
    out.append("- model_sha256: %s" % idn["model_sha256"])
    out.append("- llama_cpp_build: %s" % idn["llama_cpp_build"])
    out.append("- z3_version: %s" % idn["z3_version"])
    out.append("- dataset_sha256: %s" % idn["dataset_sha256"])
    out.append("- ir_schema_sha256: %s" % idn["ir_schema_sha256"])
    out.append("- prompt_sha.direct: %s" % idn["prompt_sha"]["direct"])
    out.append("- prompt_sha.ir: %s" % idn["prompt_sha"]["ir"])
    sp = idn["server_props"]
    if sp is None:
        out.append("- server_props: -")
    else:
        out.append("- server_props: model_path=%s n_ctx=%s" % (sp["model_path"], sp["n_ctx"]))
    out.append("- experiment: %s" % report["experiment"])
    out.append("")
    gg = report["gold_gate"]
    out.append("%s: %s (%s %s)" % (
        t["gold_gate"], t["pass"] if gg["pass"] else t["fail"], gg["groups"], t["groups_word"]))
    out.append("")
    out.append("## " + t["metrics"])
    out.append("")
    out.append("| %s | %s | %s |" % (t["metric"], ROUTE_ID_ORDER[0], ROUTE_ID_ORDER[1]))
    out.append("| --- | --- | --- |")
    met = report["metrics"]
    for name in METRIC_ORDER:
        cells = ["%d/%d = %s" % (met[rid][name]["num"], met[rid][name]["den"], met[rid][name]["value"])
                 for rid in ROUTE_ID_ORDER]
        out.append("| %s | %s | %s |" % (_mname(name, lang), cells[0], cells[1]))
    out.append("")
    out.append("## " + t["baseline_delta"])
    out.append("")
    out.append("| %s | %s | %s | %s |" % (t["metric"], ROUTE_ID_ORDER[0], ROUTE_ID_ORDER[1], t["delta"]))
    out.append("| --- | --- | --- | --- |")
    bd = report["baseline_delta"]
    for name in METRIC_ORDER:
        row = bd[name]
        out.append("| %s | %s | %s | %s |" % (
            _mname(name, lang), row[ROUTE_ID_ORDER[0]], row[ROUTE_ID_ORDER[1]], row["delta"]))
    out.append("")
    out.append("## " + t["matrix"])
    out.append("")
    out.append("| %s | %s | %s | direct | ir |" % (t["group"], t["kind"], t["gold"]))
    out.append("| --- | --- | --- | --- | --- |")
    for f in report["findings"]:
        out.append("| %s | %s | %s | %s | %s |" % (
            f["group"], f["kind"], _vd(f["gold"], lang),
            _mark(f["direct_greedy"], f["gold"], lang),
            _mark(f["ir_greedy"], f["gold"], lang)))
    out.append("")
    out.append("## " + t["taxonomy"])
    out.append("")
    out.append("| %s | %s | %s |" % (t["code"], ROUTE_ID_ORDER[0], ROUTE_ID_ORDER[1]))
    out.append("| --- | --- | --- |")
    tax = report["taxonomy"]
    for code in sorted(set(tax[ROUTE_ID_ORDER[0]]) | set(tax[ROUTE_ID_ORDER[1]])):
        out.append("| %s | %s | %s |" % (
            code, tax[ROUTE_ID_ORDER[0]].get(code, 0), tax[ROUTE_ID_ORDER[1]].get(code, 0)))
    out.append("")
    out.append("## " + t["findings"])
    out.append("")
    for f in report["findings"]:
        out.append("- %s [%s]: %s" % (f["group"], f["kind"], f[lang]))
        out.append("  - %s: %s" % (t["quote_a"], f["quote_a"]))
        out.append("  - %s: %s" % (t["quote_b"], f["quote_b"]))
    out.append("")
    out.append("%s: %s" % (t["replay"], report["replay"]["status"]))
    return "\n".join(out) + "\n"
