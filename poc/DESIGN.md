# M2 PoC design contract (throwaway branch poc-m2-oneshot)

One-shot PoC of SPEC §9 short-hop translation: claim 1's minimal pair on this laptop.
A weak local model translates synthetic Japanese clinical rules into an executable
formal target via two routes; a real solver scores both routes against gold. Stack
deviates from spec deliberately: Python stdlib + llama-server HTTP + z3 subprocess.
Speed over robustness; evaluation real, never mocked.

Authoritative for all build agents. Interfaces here are exact; deviations break peers.

## Rules for every agent

- Read this file fully before writing anything.
- ASCII-only in: final messages, Python source, HTML/JS source, this design, printed
  output. Japanese exists ONLY inside dataset.json string values (file itself is
  ASCII via JSON \uXXXX escapes) and in model I/O record values (same escaping).
- Every JSON file written by any code: `json.dump(..., ensure_ascii=True, sort_keys=True)`.
- Stdout printing of model/dataset text is forbidden; print ids, codes, counts,
  hashes. Error paths route strings through `redact.redact(s)`.
- Python: 3.13 stdlib only (urllib.request, json, subprocess, hashlib, pathlib,
  argparse, time, os, re, sys). Zero pip/uv deps. Type hints optional. Asserts fine.
- Self-check before finishing: `python3 -m py_compile <your files>` plus the checks
  named in your task. UI agent: no check beyond well-formedness by inspection.
- If a write is blocked by the vocabulary gate citing file:line, reword those lines
  with plain operational words and retry.

## Layout

```
poc/
  DESIGN.md  README.md (post-build summary)  .gitignore
  dataset.json            A1
  run_m2.py               A2 (CLI orchestrator)
  m2/__init__.py          A2 (empty)
  m2/llm.py  m2/redact.py A2
  m2/routes.py            A3
  m2/admit.py m2/verdict.py A4
  m2/score.py m2/report.py  A5
  ui/index.html           A6
  vendor/                 (gitignored) llama-b9601/llama-server, qwen gguf
  runs/<run_id>/          (gitignored) records/, report.json, report.md, report.ja.md
  runs/latest             symlink -> <run_id>
```

Run from repo root: `python3 poc/run_m2.py ...`. Modules import as `from m2 import llm`
with `sys.path.insert(0, str(Path(__file__).parent))` in run_m2.py.

## Fixed identities

- Model: `poc/vendor/qwen2.5-1.5b-instruct-q4_k_m.gguf`
  sha256 6a1a2eb6d15622bf3c96857206351ba97e1af16c30d7a74ee38970e434e9407e
- llama-server: `poc/vendor/llama-b9601/llama-server`, build 9601 (4c6595503),
  port 8077, args: `-m <gguf> --host 127.0.0.1 --port 8077 -c 4096 -t 8`
- z3 4.13.3 on PATH; invoke `z3 -T:5 <file.smt2>`.
- Routes: `route.direct_smt`, `route.single_ir`. Experiment id: `exp.m2_shorthop_poc`.
- Samples per item per route: s0 greedy `{"temperature": 0, "seed": 4242}`;
  s1..s3 stability `{"temperature": 0.7, "top_p": 0.9, "seed": 4251+k}` (k=1..3).
  `"max_tokens": 320` all samples. K_SAMPLES = 4.

## Vocabulary (canonical symbols)

Variables (SMT sort): age Int; pregnant Bool; renal_impairment Bool;
hepatic_impairment Bool; on_anticoagulant Bool.
Actions: drug_aspirin, drug_warfarin, drug_ibuprofen, drug_methotrexate.
Directions: require (must administer), forbid (must not administer).
Condition ops: age: `>=  <=  >  <  =` with integer value 0..130;
bool vars: `=` with value true|false. Conditions in a rule are a conjunction (AND).
Empty conditions = rule always applies.

## dataset.json (A1)

```json
{"version": "m2poc-1",
 "vocab": {
   "variables": [{"id": "age", "smt_type": "Int", "ja": "<surface>", "en": "patient age in years"}, ...],
   "actions":   [{"id": "drug_aspirin", "ja": "<katakana>", "en": "aspirin"}, ...],
   "directions": ["require", "forbid"]},
 "items": [{"id": "r01a", "group": "g01", "ja_text": "<one Japanese sentence>",
            "gold_ir": {"action": "drug_aspirin", "direction": "forbid",
                        "conditions": [{"var": "pregnant", "op": "=", "value": true}]}}, ...],
 "groups": [{"id": "g01", "kind": "conflict", "members": ["r01a", "r01b"],
             "gold_verdict": "conflict", "en": "<one-line rationale>"}, ...]}
```

20 items, 10 groups. gold_verdict: `conflict` | `no_conflict`. Group table (exact;
A1 composes one natural Japanese guideline sentence per item — concise directive
style, e.g. patient-condition phrase + drug + administer/do-not-administer ending;
drug names in katakana; condition phrasing unambiguous and mechanically faithful
to gold_ir; sentence must contain nothing beyond its gold_ir content):

| group | kind | member a | member b | gold |
| --- | --- | --- | --- | --- |
| g01 | conflict | forbid drug_aspirin if pregnant=true | require drug_aspirin if age>=65 | conflict |
| g02 | conflict | require drug_warfarin if age>=75 | forbid drug_warfarin if hepatic_impairment=true | conflict |
| g03 | conflict | forbid drug_ibuprofen if renal_impairment=true | require drug_ibuprofen if age>=18 | conflict |
| g04 | conflict | forbid drug_methotrexate if pregnant=true | require drug_methotrexate if renal_impairment=false | conflict |
| g05 | conflict | forbid drug_aspirin if age<18 | require drug_aspirin if on_anticoagulant=true AND age<=70 | conflict |
| g06 | null_disjoint | require drug_ibuprofen if age>=18 | forbid drug_ibuprofen if age<18 | no_conflict |
| g07 | null_disjoint | forbid drug_warfarin if pregnant=true | require drug_warfarin if pregnant=false | no_conflict |
| g08 | null_diff_action | forbid drug_aspirin if renal_impairment=true | require drug_ibuprofen if renal_impairment=true | no_conflict |
| g09 | null_diff_action | require drug_methotrexate if age>=18 | forbid drug_warfarin if age>=18 | no_conflict |
| g10 | null_same_direction | forbid drug_aspirin if hepatic_impairment=true | forbid drug_aspirin if renal_impairment=true | no_conflict |

Item ids: r01a/r01b .. r10a/r10b matching group order. File must be pure ASCII:
compose with raw Japanese in memory, then write via
`json.dump(obj, f, ensure_ascii=True, sort_keys=True, indent=1)`.

## Routes and formal shapes

ParsedRule (dict, the inter-module currency):
`{"action": str, "direction": str, "formula": str, "used_vars": [str]}`
formula is an SMT-LIB Bool term over canonical variables, e.g.
`(and (= on_anticoagulant true) (<= age 70))`, single condition `(>= age 65)`,
empty conditions `true`.

### route.direct_smt (A3 parse)

Model must output exactly:

```
(declare-const <var> <Int|Bool>)        [0+ lines, vars it uses]
(define-fun applies () Bool <formula>)
; action=<action_id> direction=<require|forbid>
```

`parse_direct(text) -> (ParsedRule|None, failure_code|None)`:
strip code fences if present; find `(define-fun applies () Bool` and extract the
balanced-paren body -> formula; regex the metadata comment
`;\s*action=(\S+)\s+direction=(\S+)`. Missing either, unbalanced parens, or empty
formula -> `(None, "target_parse_error")`. Model declare-const lines are discarded
(canonical decls are supplied downstream); used_vars = canonical var ids appearing
as tokens in formula. Vocabulary/type errors surface later via z3 (admit).

### route.single_ir (A3 parse + compile)

IR JSON: `{"action": <enum>, "direction": <enum>, "conditions": [{"var": <enum>, "op": <enum>, "value": int|bool}, ...]}`
`ir_json_schema(vocab) -> dict`: JSON Schema with enums from dataset vocab,
additionalProperties false everywhere, value via `"anyOf": [{"type":"integer"},{"type":"boolean"}]`,
required all three keys. Server enforces it as grammar; admit re-checks structurally.
`parse_ir(text) -> (dict|None, "target_parse_error"|None)`: json.loads, must be dict.
`compile_ir(ir) -> ParsedRule`: deterministic; conditions -> formula
(`(= var true)` form for bools; `(op var int)` for age; 2+ conds nest in one `(and ...)`
in given order; 0 conds -> `true`).

### Prompts (A3)

`build_prompts(vocab) -> {"direct": {"system": str, "user_template": str}, "ir": {...}}`
user_template has one `{JA_TEXT}` slot (use .replace, not .format — formulas carry braces).
Both prompts carry an identical vocabulary block built from dataset.json vocab:
lines `<ja> = <id> (Int 0..130 | Bool)` for variables, `<ja> = <id>` for actions —
the only place Japanese enters prompts besides the rule sentence; build it from
loaded data, never literals. Both carry direction semantics one-liner
(require = must administer; forbid = must not administer) and ONE worked example
(same example rule both routes, expressed in that route's output format):
example rule = require drug_ibuprofen if age>=12 (not in dataset). English
instruction text; output-format block exact per route ("Output exactly this format,
nothing else"). System prompts one line each, e.g. direct: "You translate one
Japanese clinical rule into SMT-LIB. Output only the required format." ir: same
with JSON.

`prompt_sha(prompts) -> {"direct": sha256-of-canonical-json, "ir": ...}` over
`json.dumps(prompts[route], sort_keys=True, ensure_ascii=True)`.

## m2/llm.py + run_m2.py (A2)

```python
class LlamaServer:
    def __init__(self, server_bin, model_path, port=8077, ctx=4096, threads=8): ...
    def start(self, log_path): ...   # Popen, poll GET /health until {"status":"ok"} (timeout 180s)
    def stop(self): ...              # terminate, wait
    def props(self) -> dict: ...     # GET /props json
def chat(port, messages, seed, temperature, top_p=None, max_tokens=320,
         response_format=None, timeout_s=300) -> dict:
    # POST /v1/chat/completions {"messages":..., "seed":..., "temperature":...,
    #  "max_tokens":..., (+top_p), (+response_format)}; urllib; returns
    # {"request": <body dict>, "response": <full response json>, "duration_ms": int}
```

response_format for single_ir (OpenAI shape; A2 verifies live in smoke):
`{"type": "json_schema", "json_schema": {"name": "rule_ir", "strict": true, "schema": <ir_json_schema>}}`
Content extraction: `response["choices"][0]["message"]["content"]`.

m2/redact.py: `redact(s) -> s` if all chars < U+0080 else `<ja {len(s)} {sha256(s.encode())[:8].hex()}>`.

run_m2.py subcommands (argparse):
- `setup-check`: binaries exist, model sha matches design, z3 --version, print ok lines.
- `run --run-id ID [--groups all|gNN,..] [--k 4] [--routes direct,ir] [--no-server]`:
  load dataset; gold gate first (see score); start server (log runs/ID/server.log);
  for item x route x sample s0..s(k-1): skip if record exists (resume); build
  messages from prompts; chat(); write record
  `runs/ID/records/<item>.<route>.s<n>.json` =
  `{"item": id, "route": route, "sample": n, "request": ..., "response": ...,
    "duration_ms": ..., "seed": ..., "temperature": ...}`; print progress line
  `<item> <route> s<n> <http_ok> <n_done>/<n_total>`; stop server; then score.
- `score --run-id ID`: m2.score.score_run(run_dir, dataset) -> writes report.json,
  report.md, report.ja.md, refreshes `runs/latest` symlink (relative, replace).
- `replay --run-id ID`: recompute report dict from records; byte-compare canonical
  dump EXCLUDING key "replay" vs existing file's dump excluding "replay"; rewrite
  report.json with replay.status = "match"|"mismatch" (+ report.md/.ja.md re-render);
  print status. Exit 1 on mismatch.

Route name constants: "direct" and "ir" in code/records/filenames; report.json uses
full ids route.direct_smt / route.single_ir in metrics tables (map at score time).

## m2/admit.py + m2/verdict.py 8(A4)

```python
CANON_DECLS = "(declare-const age Int)\n(declare-const pregnant Bool)\n..."  # all five
def run_z3(smt_text, timeout_s=5) -> {"status": "sat"|"unsat"|"unknown"|"error", "stdout": str, "stderr": str}
    # tmp file under the run dir, subprocess z3 -T:5; status from first stdout line; (error ...) anywhere -> error
def admit_direct(text) -> AdmissionResult
def admit_ir(text, vocab) -> AdmissionResult
# AdmissionResult dict: {"syntactic_valid": bool, "admitted": bool,
#   "failure_code": None|"target_parse_error"|"ai_schema_violation"|"solver_execution_failure",
#   "rule": ParsedRule|None}
```

admit_direct: parse_direct -> fail => syntactic_valid False, code target_parse_error.
Else z3-check `CANON_DECLS + (define-fun applies () Bool <formula>) + (assert applies) + (check-sat)`:
z3 error => syntactic_valid False, target_parse_error (off-vocabulary symbols land
here deliberately). Else syntactic_valid True; then action/direction in enums else
ai_schema_violation; admitted = both gates pass.
admit_ir: parse_ir fail => target_parse_error (grammar should prevent; truncation
can still cause it). Else structural check against vocab enums/ops/types/ranges
(int 0..130 for age, bool for bool vars) => violation = ai_schema_violation,
syntactic_valid True (it parsed as JSON). Pass => compile_ir, plus same z3 sanity
parse as direct (must never fail; if it does, solver_execution_failure). admitted
= structural pass.

```python
def group_verdict(rule_a, rule_b) -> {"verdict": "conflict"|"no_conflict"|"incomputable",
                                      "reason": str, "z3": dict|None}
```
Either rule None -> incomputable/member_inadmissible. action differs ->
no_conflict/different_action. direction same -> no_conflict/same_direction. Else
overlap query: CANON_DECLS + `(assert <fa>)` + `(assert <fb>)` + `(check-sat)`:
sat -> conflict/overlap_sat; unsat -> no_conflict/overlap_unsat; unknown|error ->
incomputable/solver_error. z3 key = run_z3 result + the query text.

## m2/score.py + m2/report.py (A5)

`score_run(run_dir: Path, dataset: dict) -> report: dict` then writers. Steps:
1. Gold gate: for each group, group_verdict(compile_ir(gold_a), compile_ir(gold_b))
   must equal gold_verdict (assert; this calibrates the instrument with real z3).
   Report field `gold_gate: {"pass": true, "groups": 10}`.
2. Read all records. For each: admit via route's admit fn on content string ->
   raw_row `{"item", "group", "route", "sample", "seed", "syntactic_valid",
   "admitted", "failure_code", "duration_ms", "output_sha256": sha256 of content}`.
3. group_rows: per group x route x sample index: verdict via group_verdict on the
   two members' AdmissionResult rules; `{"group", "route", "sample", "verdict",
   "reason", "gold", "correct": verdict==gold}`.
4. metrics per route (rational dicts `{"num": int, "den": int, "value": float}`):
   syntactic_validity (valid samples/all samples), admission_rate (admitted/all),
   verdict_accuracy_greedy (correct s0 groups/10), verdict_stability (groups whose
   4 sample verdicts are computable AND identical / 10).
5. baseline_delta: per metric `{"route.direct_smt": v, "route.single_ir": v,
   "delta": ir-direct}` (floats, round 4).
6. taxonomy per route: counts of failure_code over raw rows + group reasons
   member_inadmissible/solver_error + greedy verdict errors as
   false_positive_conflict (verdict conflict, gold no_conflict) /
   false_negative_conflict (gold conflict, verdict no_conflict or incomputable).
7. findings: per group `{"group", "kind", "gold", "direct_greedy": verdict,
   "ir_greedy": verdict, "quote_a": items ja_text, "quote_b": ..., "en": one
   sentence "gold X; direct route said Y; IR route said Z", "ja": same sentence
   in Japanese}`. JA template strings live in report.py as \uXXXX escape literals.
8. identities: model sha + name, llama_cpp_build "b9601 (4c6595503)", z3_version
   (live `z3 --version` output), dataset_sha256, ir_schema_sha256, prompt_sha
   per route, server_props subset (model path, n_ctx) read from
   runs/ID/server_props.json if A2 saved it (A2: save props() there during run).
9. config echo (seeds, k, max_tokens, port) + `"replay": {"status": "not_yet_replayed"}`
   + experiment id + run_id + counts.

Canonical bytes: `(json.dumps(report, ensure_ascii=True, sort_keys=True, indent=1) + "\n").encode()`.
report.md / report.ja.md: `render_md(report, lang) -> str` purely from the report
dict: title + identity block, gold gate line, metrics table per route, baseline
delta table, group verdict matrix (group/kind/gold/direct/ir, mark wrong cells
`*`), taxonomy table, findings list (quotes included; in report.md JA quotes stay
as-is — file is UTF-8 markdown rendered from escaped JSON so writing decoded JA
into report.ja.md/report.md is acceptable there and only there), replay line.
All JA literal templates in source as \uXXXX escapes. Deterministic: no
timestamps anywhere in report.json/md (duration_ms lives in raw rows only).

## ui/index.html (A6)

Single file, vanilla JS/CSS, no CDN. Loads `../runs/latest/report.json` via
fetch with `?run=<id>` override (`../runs/<id>/report.json`). Serve:
`python3 -m http.server 8076 -d poc` then http://127.0.0.1:8076/ui/.
Header: title + run id + EN | JA toggle buttons (persist choice in localStorage,
default EN, set `<html lang>`). i18n dict in source: `{en: {...}, ja: {...}}`,
ja values as \uXXXX escapes; elements carry data-i18n keys. Sections:
1. Verdict banner: per-route verdict_accuracy_greedy + the delta, one big line.
2. Metrics table: rows = 4 metrics, cols = direct | single_ir | delta (delta cell
   green when ir > direct, red when lower).
3. Group matrix: rows g01..g10: kind, gold, direct s0 verdict, ir s0 verdict;
   wrong cells red, correct green, incomputable gray.
4. Taxonomy: two columns of code:count.
5. Findings: per group, lang-dependent sentence (en/ja field) + the two quoted
   Japanese spans (always shown verbatim, both languages — they are quotes).
6. Identity footer: model, build, z3, hashes (truncate 12 chars), replay status,
   gold gate. Layout: clean, compact, system-ui font, max-width 960px, the two
   route columns visually paired. No external requests beyond the report fetch.
```

## Audit stage

B1 (after A1): re-derive every group's verdict from gold_irs by hand-reasoning,
cross-check ja_text wording against gold_ir and the design table semantics; output
ASCII table group:ok/mismatch + issues. B2 (after all authors): py_compile all,
`python3 -c` import chain, signature cross-check vs this design, dataset
structural check (ids/refs/enums/ASCII purity), fix mechanical seams in place,
report what changed.

## Metrics intent (for report wording)

Locked synthetic fixture measurement, instrument = the deterministic gate chain
(grammar/admission/z3) shared by both routes; the only varying factor is the
translation route. Findings wording: measured rates on synthetic fixtures, no
clinical claims. The headline artifact is the baseline_delta table.
