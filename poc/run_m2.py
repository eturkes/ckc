"""M2 PoC CLI orchestrator. Run from repo root: python3 poc/run_m2.py <subcommand>."""
import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from m2 import llm

POC = Path(__file__).resolve().parent
RUNS = POC / "runs"
DATASET_PATH = POC / "dataset.json"
MODEL_PATH = POC / "vendor" / "qwen2.5-1.5b-instruct-q4_k_m.gguf"
MODEL_SHA256 = "6a1a2eb6d15622bf3c96857206351ba97e1af16c30d7a74ee38970e434e9407e"
SERVER_BIN = POC / "vendor" / "llama-b9601" / "llama-server"
PORT = 8077
MAX_TOKENS = 320


def jdump_bytes(obj):
    return (json.dumps(obj, ensure_ascii=True, sort_keys=True, indent=1) + "\n").encode()


def write_json(path, obj):
    Path(path).write_bytes(jdump_bytes(obj))


def load_dataset():
    return json.loads(DATASET_PATH.read_text(encoding="utf-8"))


def sample_params(n):
    if n == 0:
        return {"seed": 4242, "temperature": 0, "top_p": None}
    return {"seed": 4251 + n, "temperature": 0.7, "top_p": 0.9}


def refresh_latest(run_id):
    latest = RUNS / "latest"
    if latest.is_symlink() or latest.exists():
        latest.unlink()
    latest.symlink_to(run_id)


def write_report_files(run_dir, rep):
    from m2 import report
    (run_dir / "report.json").write_bytes(jdump_bytes(rep))
    (run_dir / "report.md").write_text(report.render_md(rep, "en"), encoding="utf-8")
    (run_dir / "report.ja.md").write_text(report.render_md(rep, "ja"), encoding="utf-8")


def do_score(run_id):
    from m2 import score
    run_dir = RUNS / run_id
    rep = score.score_run(run_dir, load_dataset())
    write_report_files(run_dir, rep)
    refresh_latest(run_id)
    print(f"score ok {run_id}")


def gold_gate(dataset, routes, verdict):
    items = {it["id"]: it for it in dataset["items"]}
    for g in dataset["groups"]:
        ia, ib = g["members"]
        v = verdict.group_verdict(routes.compile_ir(items[ia]["gold_ir"]),
                                  routes.compile_ir(items[ib]["gold_ir"]))
        assert v["verdict"] == g["gold_verdict"], \
            f"gold_gate fail {g['id']} got {v['verdict']} want {g['gold_verdict']}"
    print(f"gold_gate ok {len(dataset['groups'])} groups")


def cmd_setup_check(args):
    assert SERVER_BIN.is_file(), f"missing {SERVER_BIN}"
    print(f"ok server_bin {SERVER_BIN}")
    assert MODEL_PATH.is_file(), f"missing {MODEL_PATH}"
    h = hashlib.sha256()
    with open(MODEL_PATH, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    assert h.hexdigest() == MODEL_SHA256, f"model sha mismatch {h.hexdigest()}"
    print(f"ok model_sha256 {h.hexdigest()}")
    z = subprocess.run(["z3", "--version"], capture_output=True, text=True)
    assert z.returncode == 0, "z3 --version failed"
    print(f"ok z3 {z.stdout.strip()}")
    return 0


def cmd_run(args):
    from m2 import routes, verdict
    dataset = load_dataset()
    run_dir = RUNS / args.run_id
    rec_dir = run_dir / "records"
    rec_dir.mkdir(parents=True, exist_ok=True)

    gold_gate(dataset, routes, verdict)

    groups = dataset["groups"]
    if args.groups != "all":
        by_id = {g["id"]: g for g in groups}
        groups = [by_id[w] for w in args.groups.split(",")]
    items = {it["id"]: it for it in dataset["items"]}
    item_ids = [m for g in groups for m in g["members"]]

    route_list = args.routes.split(",")
    for rt in route_list:
        assert rt in ("direct", "ir"), f"unknown route {rt}"

    prompts = routes.build_prompts(dataset["vocab"])
    rf_ir = {"type": "json_schema",
             "json_schema": {"name": "rule_ir", "strict": True,
                             "schema": routes.ir_json_schema(dataset["vocab"])}}

    todo = [(i, rt, n) for i in item_ids for rt in route_list for n in range(args.k)]
    n_total = len(todo)
    n_done = 0

    server = llm.LlamaServer(SERVER_BIN, MODEL_PATH, port=PORT)
    try:
        if not args.no_server:
            server.start(run_dir / "server.log")
        write_json(run_dir / "server_props.json", server.props())
        for iid, rt, n in todo:
            rec_path = rec_dir / f"{iid}.{rt}.s{n}.json"
            if rec_path.exists():
                n_done += 1
                print(f"{iid} {rt} s{n} skip {n_done}/{n_total}", flush=True)
                continue
            p = sample_params(n)
            messages = [
                {"role": "system", "content": prompts[rt]["system"]},
                {"role": "user", "content":
                 prompts[rt]["user_template"].replace("{JA_TEXT}", items[iid]["ja_text"])},
            ]
            res = llm.chat(PORT, messages, seed=p["seed"], temperature=p["temperature"],
                           top_p=p["top_p"], max_tokens=MAX_TOKENS,
                           response_format=rf_ir if rt == "ir" else None)
            write_json(rec_path, {"item": iid, "route": rt, "sample": n,
                                  "request": res["request"], "response": res["response"],
                                  "duration_ms": res["duration_ms"],
                                  "seed": p["seed"], "temperature": p["temperature"]})
            ok = "ok" if res["response"].get("choices") else "err"
            n_done += 1
            print(f"{iid} {rt} s{n} {ok} {n_done}/{n_total}", flush=True)
    finally:
        server.stop()
    do_score(args.run_id)
    return 0


def cmd_score(args):
    do_score(args.run_id)
    return 0


def cmd_replay(args):
    from m2 import score
    run_dir = RUNS / args.run_id
    existing = json.loads((run_dir / "report.json").read_text(encoding="utf-8"))
    rep = score.score_run(run_dir, load_dataset())
    old_cmp = {k: v for k, v in existing.items() if k != "replay"}
    new_cmp = {k: v for k, v in rep.items() if k != "replay"}
    status = "match" if jdump_bytes(old_cmp) == jdump_bytes(new_cmp) else "mismatch"
    rep["replay"] = {"status": status}
    write_report_files(run_dir, rep)
    print(f"replay {status} {args.run_id}")
    return 0 if status == "match" else 1


def main():
    ap = argparse.ArgumentParser(prog="run_m2")
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("setup-check")
    rp = sub.add_parser("run")
    rp.add_argument("--run-id", required=True)
    rp.add_argument("--groups", default="all")
    rp.add_argument("--k", type=int, default=4)
    rp.add_argument("--routes", default="direct,ir")
    rp.add_argument("--no-server", action="store_true")
    for name in ("score", "replay"):
        p = sub.add_parser(name)
        p.add_argument("--run-id", required=True)
    args = ap.parse_args()
    fns = {"setup-check": cmd_setup_check, "run": cmd_run,
           "score": cmd_score, "replay": cmd_replay}
    sys.exit(fns[args.cmd](args))


if __name__ == "__main__":
    main()
