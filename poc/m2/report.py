"""M2 PoC report rendering (A5, revision 2): markdown + canonical bytes.

Design: poc/DESIGN.md. Pure functions of the report dict; no timestamps.
JA literals below are unicode escape sequences; source bytes are ASCII.
Quoted Japanese spans from the report dict render verbatim in both langs.
"""
import json

METRIC_ORDER = ("syntactic_validity", "admission_rate",
                "verdict_accuracy_greedy", "verdict_stability")
FINDING_LABELS = {"direct": "direct", "ir": "single-IR", "stacked": "stacked",
                  "hop": "hop-chain", "layered": "layered"}

L_EN = {
    "title": "M2 PoC 5x5x5 report",
    "identity": "Identity",
    "gold_gate": "Gold gate",
    "pass": "PASS",
    "fail": "FAIL",
    "groups_word": "groups",
    "metrics": "Metrics",
    "metric": "metric",
    "pooled": "pooled",
    "baseline_delta": "Baseline delta",
    "matrix": "Greedy verdict matrix",
    "group": "group",
    "kind": "kind",
    "gold": "gold",
    "taxonomy": "Failure taxonomy",
    "code": "code",
    "findings": "Findings",
    "quote_a": "quote A",
    "quote_b": "quote B",
    "omitted": "%d all-correct source x group rows omitted",
    "replay": "Replay",
}

# --- JA literals (ASCII unicode escapes) ---
VERDICT_JA = {'conflict': '\u77db\u76fe', 'no_conflict': '\u77db\u76fe\u306a\u3057', 'incomputable': '\u8a08\u7b97\u4e0d\u80fd'}
METRIC_JA = {'syntactic_validity': '\u69cb\u6587\u59a5\u5f53\u7387', 'admission_rate': '\u53d7\u7406\u7387', 'verdict_accuracy_greedy': '\u5224\u5b9a\u7cbe\u5ea6(\u30b0\u30ea\u30fc\u30c7\u30a3)', 'verdict_stability': '\u5224\u5b9a\u5b89\u5b9a\u6027'}
JA_FINDING_HEAD = '\u6b63\u89e3\u306f%s\u3002'
JA_FINDING_SEG = '%s\u7d4c\u8def\u306f%s\u3068\u5224\u5b9a'
JA_JOIN = '\u3001'
JA_END = '\u3002'
L_JA = {'title': 'M2 PoC 5x5x5 \u30ec\u30dd\u30fc\u30c8', 'identity': '\u8b58\u5225\u60c5\u5831', 'gold_gate': '\u30b4\u30fc\u30eb\u30c9\u30b2\u30fc\u30c8', 'pass': '\u5408\u683c', 'fail': '\u4e0d\u5408\u683c', 'groups_word': '\u30b0\u30eb\u30fc\u30d7', 'metrics': '\u6307\u6a19', 'metric': '\u6307\u6a19', 'pooled': '\u5168\u4f53', 'baseline_delta': '\u30d9\u30fc\u30b9\u30e9\u30a4\u30f3\u5dee\u5206', 'matrix': '\u30b0\u30ea\u30fc\u30c7\u30a3\u5224\u5b9a\u30de\u30c8\u30ea\u30af\u30b9', 'group': '\u30b0\u30eb\u30fc\u30d7', 'kind': '\u7a2e\u5225', 'gold': '\u6b63\u89e3', 'taxonomy': '\u5931\u6557\u5206\u985e', 'code': '\u30b3\u30fc\u30c9', 'findings': '\u6240\u898b', 'quote_a': '\u5f15\u7528A', 'quote_b': '\u5f15\u7528B', 'omitted': '\u5168\u30eb\u30fc\u30c8\u6b63\u89e3\u306e\u30bd\u30fc\u30b9\u00d7\u30b0\u30eb\u30fc\u30d7\u884c\u3092%d\u4ef6\u7701\u7565\u3002', 'replay': '\u30ea\u30d7\u30ec\u30a4'}

L = {"en": L_EN, "ja": L_JA}


def canonical_bytes(report):
    """Exact bytes of report.json per the design canonical-bytes rule."""
    return (json.dumps(report, ensure_ascii=True, sort_keys=True, indent=1) + "\n").encode()


def finding_en(gold, pairs):
    """pairs = [(route_key, s0 verdict), ...] in route key order."""
    segs = ["gold %s" % gold]
    segs += ["%s %s" % (FINDING_LABELS.get(rk, rk), v) for rk, v in pairs]
    return "; ".join(segs)


def finding_ja(gold, pairs):
    segs = [JA_FINDING_SEG % (FINDING_LABELS.get(rk, rk), VERDICT_JA.get(v, v))
            for rk, v in pairs]
    return (JA_FINDING_HEAD % VERDICT_JA.get(gold, gold)
            + JA_JOIN.join(segs) + JA_END)


def _vd(v, lang):
    return VERDICT_JA.get(v, v) if lang == "ja" else v


def _mname(name, lang):
    return METRIC_JA.get(name, name) if lang == "ja" else name


def _table(header_cells, rows):
    out = ["| " + " | ".join(header_cells) + " |",
           "| " + " | ".join(["---"] * len(header_cells)) + " |"]
    out += ["| " + " | ".join(r) + " |" for r in rows]
    return out


def _metrics_rows(per_route, route_ids, lang):
    rows = []
    for name in METRIC_ORDER:
        cells = [_mname(name, lang)]
        for rid in route_ids:
            c = per_route.get(rid, {}).get(name)
            cells.append("-" if c is None
                         else "%d/%d = %s" % (c["num"], c["den"], c["value"]))
        rows.append(cells)
    return rows


def render_md(report, lang):
    assert lang in L, lang
    t = L[lang]
    idn = report["identities"]
    route_ids = [r["id"] for r in report["routes"]]
    route_keys = [r["key"] for r in report["routes"]]
    out = []
    # title + identity
    out.append("# %s - %s" % (t["title"], report["run_id"]))
    out.append("")
    out.append("## " + t["identity"])
    out.append("")
    out.append("- model_name: %s" % idn["model_name"])
    out.append("- model_sha256: %s" % idn["model_sha256"])
    out.append("- llama_cpp_build: %s" % idn["llama_cpp_build"])
    out.append("- z3_version: %s" % idn["z3_version"])
    out.append("- dataset_sha256: %s" % idn["dataset_sha256"])
    for rk in route_keys:
        if rk in idn["prompt_sha"]:
            out.append("- prompt_sha.%s: %s" % (rk, idn["prompt_sha"][rk]))
    for rk in route_keys:
        for stage, sha in idn["schema_sha256"].get(rk, {}).items():
            out.append("- schema_sha256.%s.%s: %s" % (rk, stage, sha))
    sp = idn["server_props"]
    if sp is None:
        out.append("- server_props: -")
    else:
        out.append("- server_props: model_path=%s n_ctx=%s" % (sp["model_path"], sp["n_ctx"]))
    out.append("- experiment: %s" % report["experiment"])
    out.append("")
    # gold gate
    gg = report["gold_gate"]
    out.append("%s: %s (%s %s)" % (
        t["gold_gate"], t["pass"] if gg["pass"] else t["fail"],
        gg["groups"], t["groups_word"]))
    out.append("")
    # pooled metrics
    out.append("## %s (%s)" % (t["metrics"], t["pooled"]))
    out.append("")
    out += _table([t["metric"]] + route_ids,
                  _metrics_rows(report["metrics"]["pooled"], route_ids, lang))
    out.append("")
    # pooled baseline delta, non-direct cols
    nd_ids = [rid for rid in route_ids if rid != "route.direct_smt"]
    out.append("## %s (%s)" % (t["baseline_delta"], t["pooled"]))
    out.append("")
    rows = []
    for name in METRIC_ORDER:
        scope = report["baseline_delta"][name]["pooled"]
        cells = [_mname(name, lang)]
        for rid in nd_ids:
            d = scope.get(rid)
            cells.append("-" if d is None
                         else "%s (%+.4f)" % (d["value"], d["delta"]))
        rows.append(cells)
    out += _table([t["metric"]] + nd_ids, rows)
    out.append("")
    # per-source blocks: metrics + greedy verdict matrix
    for s in report["sources"]:
        sid = s["id"]
        per_route = report["metrics"]["by_source"].get(sid)
        if per_route is None:
            continue
        out.append("## %s (%s)" % (sid, s["ja"] if lang == "ja" else s["en"]))
        out.append("")
        out.append("### " + t["metrics"])
        out.append("")
        out += _table([t["metric"]] + route_ids,
                      _metrics_rows(per_route, route_ids, lang))
        out.append("")
        out.append("### " + t["matrix"])
        out.append("")
        rows = []
        for f in report["findings"]:
            if f["source"] != sid:
                continue
            cells = [f["group"], f["kind"], _vd(f["gold"], lang)]
            for rid in route_ids:
                v = f["greedy"].get(rid)
                if v is None:
                    cells.append("-")
                else:
                    cells.append(_vd(v, lang) + ("*" if v != f["gold"] else ""))
            rows.append(cells)
        out += _table([t["group"], t["kind"], t["gold"]] + route_ids, rows)
        out.append("")
    # pooled taxonomy
    out.append("## %s (%s)" % (t["taxonomy"], t["pooled"]))
    out.append("")
    tax = report["taxonomy"]["pooled"]
    codes = sorted(set().union(*[set(c) for c in tax.values()])) if tax else []
    out += _table([t["code"]] + route_ids,
                  [[code] + [str(tax.get(rid, {}).get(code, 0)) for rid in route_ids]
                   for code in codes])
    out.append("")
    # findings: only rows with a wrong greedy verdict, then the omitted count
    out.append("## " + t["findings"])
    out.append("")
    shown = [f for f in report["findings"]
             if any(v != f["gold"] for v in f["greedy"].values())]
    for f in shown:
        out.append("- %s %s [%s]: %s" % (f["source"], f["group"], f["kind"], f[lang]))
        out.append("  - %s: %s" % (t["quote_a"], f["quote_a"]))
        out.append("  - %s: %s" % (t["quote_b"], f["quote_b"]))
    if shown:
        out.append("")
    out.append(t["omitted"] % (len(report["findings"]) - len(shown)))
    out.append("")
    # replay line
    out.append("%s: %s" % (t["replay"], report["replay"]["status"]))
    return "\n".join(out) + "\n"
