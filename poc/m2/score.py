"""M2 PoC scoring: build the report dict from run records, plus report writers.

Design: poc/DESIGN.md (A5). score_run is deterministic for a fixed
(run_dir, dataset, machine); replay re-runs it and byte-compares canonical
dumps excluding the "replay" key. write_reports emits report.json /
report.md / report.ja.md into the run dir.
"""
import hashlib
import json
import subprocess
from pathlib import Path

from m2 import admit, routes, verdict
from m2 import report as report_mod

ROUTES = ("direct", "ir")
ROUTE_IDS = {"direct": "route.direct_smt", "ir": "route.single_ir"}
MODEL_NAME = "qwen2.5-1.5b-instruct-q4_k_m.gguf"
MODEL_SHA256 = "6a1a2eb6d15622bf3c96857206351ba97e1af16c30d7a74ee38970e434e9407e"
LLAMA_BUILD = "b9601 (4c6595503)"
EXPERIMENT_ID = "exp.m2_shorthop_poc"
TAX_CODES = (
    "target_parse_error",
    "ai_schema_violation",
    "solver_execution_failure",
    "member_inadmissible",
    "solver_error",
    "false_positive_conflict",
    "false_negative_conflict",
)


def _sha256(data):
    return hashlib.sha256(data).hexdigest()


def _rat(num, den):
    assert den > 0, "empty denominator"
    return {"num": int(num), "den": int(den), "value": round(num / den, 4)}


def score_run(run_dir, dataset):
    run_dir = Path(run_dir)
    items = {it["id"]: it for it in dataset["items"]}
    groups = sorted(dataset["groups"], key=lambda g: g["id"])
    item_group = {m: g["id"] for g in groups for m in g["members"]}

    # 1. gold gate: calibrate the instrument on gold IRs with real z3.
    for g in groups:
        a, b = g["members"]
        gv = verdict.group_verdict(
            routes.compile_ir(items[a]["gold_ir"]),
            routes.compile_ir(items[b]["gold_ir"]))
        assert gv["verdict"] == g["gold_verdict"], (
            "gold_gate", g["id"], gv["verdict"], g["gold_verdict"])
    gold_gate = {"pass": True, "groups": len(groups)}

    # 2. records -> admission results + raw rows.
    res = {}
    raw_rows = []
    for path in sorted((run_dir / "records").glob("*.json")):
        rec = json.loads(path.read_text(encoding="utf-8"))
        content = rec["response"]["choices"][0]["message"]["content"]
        route = rec["route"]
        assert route in ROUTES, route
        if route == "direct":
            ar = admit.admit_direct(content)
        else:
            ar = admit.admit_ir(content, dataset["vocab"])
        res[(rec["item"], route, rec["sample"])] = ar
        raw_rows.append({
            "item": rec["item"],
            "group": item_group[rec["item"]],
            "route": route,
            "sample": rec["sample"],
            "seed": rec["seed"],
            "syntactic_valid": ar["syntactic_valid"],
            "admitted": ar["admitted"],
            "failure_code": ar["failure_code"],
            "duration_ms": rec["duration_ms"],
            "output_sha256": _sha256(content.encode("utf-8")),
        })
    assert raw_rows, "no records under %s" % (run_dir / "records")
    raw_rows.sort(key=lambda r: (r["item"], r["route"], r["sample"]))
    k = max(r["sample"] for r in raw_rows) + 1

    # 3. group rows: solver verdict per group x route x sample index.
    group_rows = []
    for g in groups:
        a, b = g["members"]
        if (a, "direct", 0) not in res:  # group absent from a partial run
            continue
        for route in ROUTES:
            for s in range(k):
                gv = verdict.group_verdict(
                    res[(a, route, s)]["rule"], res[(b, route, s)]["rule"])
                group_rows.append({
                    "group": g["id"],
                    "route": route,
                    "sample": s,
                    "verdict": gv["verdict"],
                    "reason": gv["reason"],
                    "gold": g["gold_verdict"],
                    "correct": gv["verdict"] == g["gold_verdict"],
                })
    greedy = {(r["group"], r["route"]): r for r in group_rows if r["sample"] == 0}
    groups = [g for g in groups if (g["id"], "direct") in greedy]  # partial runs

    # 4. per-route metrics as num/den/value rationals.
    metrics = {}
    for route in ROUTES:
        rr = [r for r in raw_rows if r["route"] == route]
        per_group = {}
        for r in group_rows:
            if r["route"] == route:
                per_group.setdefault(r["group"], []).append(r["verdict"])
        stable = sum(
            1 for vs in per_group.values()
            if len(set(vs)) == 1 and vs[0] != "incomputable")
        metrics[ROUTE_IDS[route]] = {
            "syntactic_validity": _rat(sum(r["syntactic_valid"] for r in rr), len(rr)),
            "admission_rate": _rat(sum(r["admitted"] for r in rr), len(rr)),
            "verdict_accuracy_greedy": _rat(
                sum(greedy[(g["id"], route)]["correct"] for g in groups), len(groups)),
            "verdict_stability": _rat(stable, len(groups)),
        }

    # 5. baseline delta (ir minus direct), floats rounded to 4.
    baseline_delta = {}
    for name in report_mod.METRIC_ORDER:
        dv = metrics["route.direct_smt"][name]["value"]
        iv = metrics["route.single_ir"][name]["value"]
        baseline_delta[name] = {
            "route.direct_smt": dv,
            "route.single_ir": iv,
            "delta": round(iv - dv, 4),
        }

    # 6. failure taxonomy per route.
    taxonomy = {}
    for route in ROUTES:
        t = {c: 0 for c in TAX_CODES}
        for r in raw_rows:
            if r["route"] == route and r["failure_code"]:
                t[r["failure_code"]] += 1
        for r in group_rows:
            if r["route"] == route and r["reason"] in ("member_inadmissible", "solver_error"):
                t[r["reason"]] += 1
        for g in groups:
            row = greedy[(g["id"], route)]
            if row["verdict"] == "conflict" and row["gold"] == "no_conflict":
                t["false_positive_conflict"] += 1
            if row["gold"] == "conflict" and row["verdict"] != "conflict":
                t["false_negative_conflict"] += 1
        taxonomy[ROUTE_IDS[route]] = t

    # 7. findings: greedy verdicts + quoted JA spans + EN/JA sentences.
    findings = []
    for g in groups:
        a, b = g["members"]
        dv = greedy[(g["id"], "direct")]["verdict"]
        iv = greedy[(g["id"], "ir")]["verdict"]
        findings.append({
            "group": g["id"],
            "kind": g["kind"],
            "gold": g["gold_verdict"],
            "direct_greedy": dv,
            "ir_greedy": iv,
            "quote_a": items[a]["ja_text"],
            "quote_b": items[b]["ja_text"],
            "en": report_mod.finding_en(g["gold_verdict"], dv, iv),
            "ja": report_mod.finding_ja(g["gold_verdict"], dv, iv),
        })

    # 8. identities.
    schema = routes.ir_json_schema(dataset["vocab"])
    prompts = routes.build_prompts(dataset["vocab"])
    z3v = subprocess.run(
        ["z3", "--version"], capture_output=True, text=True, check=True).stdout.strip()
    server_props = None
    props_path = run_dir / "server_props.json"
    if props_path.exists():
        props = json.loads(props_path.read_text(encoding="utf-8"))
        n_ctx = props.get("n_ctx")
        if n_ctx is None:
            n_ctx = props.get("default_generation_settings", {}).get("n_ctx")
        server_props = {"model_path": props.get("model_path"), "n_ctx": n_ctx}
    identities = {
        "model_name": MODEL_NAME,
        "model_sha256": MODEL_SHA256,
        "llama_cpp_build": LLAMA_BUILD,
        "z3_version": z3v,
        "dataset_sha256": _sha256(
            json.dumps(dataset, ensure_ascii=True, sort_keys=True, indent=1).encode()),
        "ir_schema_sha256": _sha256(
            json.dumps(schema, ensure_ascii=True, sort_keys=True).encode()),
        "prompt_sha": routes.prompt_sha(prompts),
        "server_props": server_props,
    }

    # 9. assemble.
    return {
        "experiment": EXPERIMENT_ID,
        "run_id": run_dir.name,
        "gold_gate": gold_gate,
        "raw_rows": raw_rows,
        "group_rows": group_rows,
        "metrics": metrics,
        "baseline_delta": baseline_delta,
        "taxonomy": taxonomy,
        "findings": findings,
        "identities": identities,
        "config": {
            "port": 8077,
            "max_tokens": 320,
            "k_samples": k,
            "greedy": {"seed": 4242, "temperature": 0},
            "stability": {"seeds": [4252, 4253, 4254], "temperature": 0.7, "top_p": 0.9},
        },
        "counts": {
            "items": len(items),
            "groups": len(groups),
            "routes": len(ROUTES),
            "records": len(raw_rows),
        },
        "replay": {"status": "not_yet_replayed"},
    }


def write_reports(run_dir, report):
    run_dir = Path(run_dir)
    (run_dir / "report.json").write_bytes(report_mod.canonical_bytes(report))
    (run_dir / "report.md").write_text(
        report_mod.render_md(report, "en"), encoding="utf-8")
    (run_dir / "report.ja.md").write_text(
        report_mod.render_md(report, "ja"), encoding="utf-8")
