"""M2 PoC scoring (A5, revision 2): build the report dict from run records.

Design: poc/DESIGN.md. score_run(run_dir, dataset) is deterministic for a
fixed (run_dir, dataset, machine); no timestamps anywhere. Records are
trusted by their json keys (item/source/route/sample), never filenames.
Raw rows and group rows carry FULL source/route ids; internal indexing
uses short keys. write_reports emits report.json / report.md /
report.ja.md via report.canonical_bytes / report.render_md.
"""
import hashlib
import json
import subprocess
from pathlib import Path

from m2 import admit, grammars, routes, verdict
from m2 import report as report_mod

ROUTE_KEYS = ("direct", "ir", "stacked", "hop", "layered",
              "dsl", "dslh", "dslk", "dslkh")
ROUTE_IDS = {
    "direct": "route.direct_smt",
    "ir": "route.single_ir",
    "stacked": "route.stacked_ir",
    "hop": "route.ir_hop_chain",
    "layered": "route.ckc_layered",
    "dsl": "route.ckc_dsl",
    "dslh": "route.ckc_dsl_hop",
    "dslk": "route.ckc_dsl_kw",
    "dslkh": "route.ckc_dsl_kw_hop",
}
MODEL_NAME = "qwen2.5-1.5b-instruct-q4_k_m.gguf"
MODEL_SHA256 = "6a1a2eb6d15622bf3c96857206351ba97e1af16c30d7a74ee38970e434e9407e"
LLAMA_BUILD = "b9601 (4c6595503)"
EXPERIMENT_ID = "exp.m2poc_dsl"
PORT = 8077
MAX_TOKENS = 320


def _sha256(data):
    return hashlib.sha256(data).hexdigest()


def _rat(num, den):
    return {"num": int(num), "den": int(den),
            "value": round(num / den, 4) if den else 0.0}


def _r4(x):
    return round(x, 4) + 0.0  # + 0.0 folds -0.0 into 0.0


def _seed(n):
    return 4242 if n == 0 else 4251 + n


def _content(call):
    try:
        c = call["response"]["choices"][0]["message"]["content"]
    except (KeyError, IndexError, TypeError):
        return ""
    return c if isinstance(c, str) else ""


def score_run(run_dir, dataset):
    run_dir = Path(run_dir)
    vocab = dataset["vocab"]
    items = {it["id"]: it for it in dataset["items"]}
    groups = sorted(dataset["groups"], key=lambda g: g["id"])
    item_group = {m: g["id"] for g in groups for m in g["members"]}
    source_keys = [s["key"] for s in dataset["sources"]]
    source_ids = {s["key"]: s["id"] for s in dataset["sources"]}
    metric_names = report_mod.METRIC_ORDER

    # 1. gold gate: calibrate the instrument on gold IRs with real z3.
    for g in groups:
        a, b = g["members"]
        gv = verdict.group_verdict(
            routes.compile_ir(items[a]["gold_ir"]),
            routes.compile_ir(items[b]["gold_ir"]))
        assert gv["verdict"] == g["gold_verdict"], (
            "gold_gate", g["id"], gv["verdict"], g["gold_verdict"])
    gold_gate = {"pass": True, "groups": len(groups)}

    # 2. records -> admission results + raw rows (full ids).
    res = {}       # (item, source_key, route_key, sample) -> AdmissionResult
    meta = {}      # same key -> (seed, duration_ms_total, output_sha256)
    n_calls = 0
    for path in sorted((run_dir / "records").glob("*.json")):
        rec = json.loads(path.read_text(encoding="utf-8"))
        key = (rec["item"], rec["source"], rec["route"], rec["sample"])
        assert key not in res, ("duplicate_record", key)
        assert rec["route"] in ROUTE_IDS, ("unknown_route", rec["route"])
        assert rec["source"] in source_ids, ("unknown_source", rec["source"])
        assert rec["item"] in item_group, ("unknown_item", rec["item"])
        contents = [_content(c) for c in rec["calls"]]
        n_calls += len(rec["calls"])
        res[key] = admit.admit_route(rec["route"], contents, vocab)
        meta[key] = (
            rec["seed"],
            sum(c.get("duration_ms", 0) for c in rec["calls"]),
            _sha256(json.dumps(contents, ensure_ascii=True).encode()),
        )
    assert res, "no records under %s" % (run_dir / "records")
    k = max(key[3] for key in res) + 1
    s_ix = {sk: i for i, sk in enumerate(source_keys)}
    r_ix = {rk: i for i, rk in enumerate(ROUTE_KEYS)}

    raw_rows = []
    cell_raw = {}  # (source_key, route_key) -> [AdmissionResult, ...]
    for key in sorted(res, key=lambda t: (t[0], s_ix[t[1]], r_ix[t[2]], t[3])):
        iid, sk, rk, n = key
        ar = res[key]
        seed, dur, osha = meta[key]
        raw_rows.append({
            "item": iid,
            "group": item_group[iid],
            "source": source_ids[sk],
            "route": ROUTE_IDS[rk],
            "sample": n,
            "seed": seed,
            "syntactic_valid": ar["syntactic_valid"],
            "admitted": ar["admitted"],
            "failure_code": ar["failure_code"],
            "failed_stage": ar["failed_stage"],
            "duration_ms_total": dur,
            "output_sha256": osha,
        })
        cell_raw.setdefault((sk, rk), []).append(ar)

    # 3. group rows: solver verdict per group x source x route x sample.
    group_rows = []
    cell_groups = {}  # (source_key, route_key) -> {group_id: {sample: row}}
    for g in groups:
        a, b = g["members"]
        for sk in source_keys:
            for rk in ROUTE_KEYS:
                for n in range(k):
                    ra = res.get((a, sk, rk, n))
                    rb = res.get((b, sk, rk, n))
                    if ra is None or rb is None:  # cell absent from partial run
                        continue
                    gv = verdict.group_verdict(ra["rule"], rb["rule"])
                    row = {
                        "group": g["id"],
                        "source": source_ids[sk],
                        "route": ROUTE_IDS[rk],
                        "sample": n,
                        "verdict": gv["verdict"],
                        "reason": gv["reason"],
                        "gold": g["gold_verdict"],
                        "correct": gv["verdict"] == g["gold_verdict"],
                    }
                    group_rows.append(row)
                    cell_groups.setdefault((sk, rk), {}).setdefault(g["id"], {})[n] = row

    # 4. metrics: by_source cells from records actually read; pooled sums
    # the by_source numerators and denominators.
    def cell_nums(sk, rk):
        ars = cell_raw.get((sk, rk))
        if not ars:
            return None
        per_g = cell_groups.get((sk, rk), {})
        g0 = [rows[0] for rows in per_g.values() if 0 in rows]
        full = [rows for rows in per_g.values()
                if all(n in rows for n in range(k))]
        stable = sum(
            1 for rows in full
            if len({rows[n]["verdict"] for n in range(k)}) == 1
            and rows[0]["verdict"] != "incomputable")
        return {
            "syntactic_validity": (sum(a["syntactic_valid"] for a in ars), len(ars)),
            "admission_rate": (sum(a["admitted"] for a in ars), len(ars)),
            "verdict_accuracy_greedy": (sum(r["correct"] for r in g0), len(g0)),
            "verdict_stability": (stable, len(full)),
        }

    by_source = {}
    pooled_nums = {}
    for sk in source_keys:
        per_route = {}
        for rk in ROUTE_KEYS:
            nums = cell_nums(sk, rk)
            if nums is None:
                continue
            per_route[ROUTE_IDS[rk]] = {m: _rat(*nums[m]) for m in metric_names}
            pn = pooled_nums.setdefault(rk, {m: [0, 0] for m in metric_names})
            for m in metric_names:
                pn[m][0] += nums[m][0]
                pn[m][1] += nums[m][1]
        if per_route:
            by_source[source_ids[sk]] = per_route
    pooled = {ROUTE_IDS[rk]: {m: _rat(*pooled_nums[rk][m]) for m in metric_names}
              for rk in ROUTE_KEYS if rk in pooled_nums}
    metrics = {"pooled": pooled, "by_source": by_source}

    # 5. baseline delta per metric: value + delta vs direct (rev-2; direct delta
    # 0.0) + delta_ir vs single_ir (rev-3; single_ir delta_ir 0.0). delta_ir is
    # None only when single_ir is absent from a scope (full run: always present).
    def delta_scope(per_route):
        d_id = ROUTE_IDS["direct"]
        if d_id not in per_route:
            return None
        ir_id = ROUTE_IDS["ir"]
        ir_present = ir_id in per_route
        out = {}
        for m in metric_names:
            dv = per_route[d_id][m]["value"]
            iv = per_route[ir_id][m]["value"] if ir_present else None
            out[m] = {
                rid: {"value": cells[m]["value"],
                      "delta": 0.0 if rid == d_id else _r4(cells[m]["value"] - dv),
                      "delta_ir": (None if not ir_present else
                                   0.0 if rid == ir_id else
                                   _r4(cells[m]["value"] - iv))}
                for rid, cells in per_route.items()}
        return out

    baseline_delta = {}
    pooled_d = delta_scope(pooled)
    src_d = {sid: delta_scope(pr) for sid, pr in by_source.items()}
    for m in metric_names:
        baseline_delta[m] = {
            "pooled": pooled_d[m] if pooled_d else {},
            "by_source": {sid: d[m] for sid, d in src_d.items() if d},
        }

    # 6. taxonomy: failure codes over raw rows + group reasons over all group
    # rows + greedy fp/fn over s0 group rows; zero counts omitted.
    def bump(d, code):
        d[code] = d.get(code, 0) + 1

    tax_pooled = {}
    tax_by_source = {}
    for sk in source_keys:
        for rk in ROUTE_KEYS:
            if (sk, rk) not in cell_raw:
                continue
            t = {}
            for ar in cell_raw[(sk, rk)]:
                if ar["failure_code"]:
                    bump(t, ar["failure_code"])
            per_g = cell_groups.get((sk, rk), {})
            for rows in per_g.values():
                for row in rows.values():
                    if row["reason"] in ("member_inadmissible", "solver_error"):
                        bump(t, row["reason"])
                row0 = rows.get(0)
                if row0 is not None:
                    if row0["verdict"] == "conflict" and row0["gold"] == "no_conflict":
                        bump(t, "false_positive_conflict")
                    if row0["gold"] == "conflict" and row0["verdict"] != "conflict":
                        bump(t, "false_negative_conflict")
            tax_by_source.setdefault(source_ids[sk], {})[ROUTE_IDS[rk]] = t
            pt = tax_pooled.setdefault(ROUTE_IDS[rk], {})
            for code, n in t.items():
                pt[code] = pt.get(code, 0) + n
    taxonomy = {"pooled": tax_pooled, "by_source": tax_by_source}

    # 7. findings: one per source x group cell having at least one s0 group_row.
    findings = []
    for sk in source_keys:
        for g in groups:
            pairs = []  # (route_key, s0 verdict) in route key order
            for rk in ROUTE_KEYS:
                row = cell_groups.get((sk, rk), {}).get(g["id"], {}).get(0)
                if row is not None:
                    pairs.append((rk, row["verdict"]))
            if not pairs:
                continue
            a, b = g["members"]
            findings.append({
                "source": source_ids[sk],
                "group": g["id"],
                "kind": g["kind"],
                "gold": g["gold_verdict"],
                "greedy": {ROUTE_IDS[rk]: v for rk, v in pairs},
                "quote_a": items[a]["ja_texts"][sk],
                "quote_b": items[b]["ja_texts"][sk],
                "en": report_mod.finding_en(g["gold_verdict"], pairs),
                "ja": report_mod.finding_ja(g["gold_verdict"], pairs),
            })

    # 8. identities.
    prompts = routes.build_prompts(vocab)
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
        "prompt_sha": routes.prompt_sha(prompts),
        "schema_sha256": routes.schema_shas(vocab),
        "grammar_sha256": grammars.grammar_shas(vocab),
        "server_props": server_props,
    }

    # 9. echo blocks + assemble.
    covered_items = {key[0] for key in res}
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
        "sources": dataset["sources"],
        "routes": [{"key": rk, "id": ROUTE_IDS[rk]} for rk in ROUTE_KEYS],
        "config": {
            "k": k,
            "seeds": {"s%d" % n: _seed(n) for n in range(k)},
            "temperature": {"greedy": 0, "stability": 0.7},
            "top_p": 0.9,
            "max_tokens": MAX_TOKENS,
            "port": PORT,
            "route_keys": list(ROUTE_KEYS),
            "source_keys": source_keys,
        },
        "counts": {
            "records": len(raw_rows),
            "calls": n_calls,
            "items": len(covered_items),
            "groups": len({item_group[i] for i in covered_items}),
            "sources": len({key[1] for key in res}),
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
