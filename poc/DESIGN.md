# M2-M4 PoC design contract (throwaway branch poc-m2-3-4)

Scope: this PoC spans SPEC M2-M4 on one throwaway branch. M2 (sec.9 short-hop
pair) and M3 (sec.10 route trio) are specified and measured at revision 2 below
(5-route field, run matrix5); M4 (sec.11 invented-DSL routes: grammar-masked
concrete syntax, deterministic parse -> IR -> compile) is specified at revision 3
in the closing "Revision 3 -- M4 invented DSLs" section, widening the field from
5 to 9 routes (two project-born DSL candidates, each singular + hop-layered).

Revision 2 widens the SPEC sec.9 short-hop PoC from a 2-route minimal pair to a
5 x 5 x 5 matrix: 5 routes (the sec.9 pair plus the three sec.10 route shapes adapted to
PoC scale), 5 surface-style source families re-expressing the same 20 rules and 10
gold groups, k=5 samples per item x source x route. One weak local model translates
synthetic Japanese clinical rules into an executable formal target; a real solver
scores every route against shared gold. Stack deviates from spec deliberately:
Python stdlib + llama-server HTTP + z3 subprocess. Speed over robustness;
evaluation real, never mocked. Revision 1 (the 2-route contract and its recorded
run `m2`) lives in git history; code targets the current revision (2-3).

Authoritative for all build agents. Interfaces here are exact; deviations break peers.

## Rules for every agent

- Read this file fully before writing anything.
- ASCII-only in: final messages, Python source, HTML/JS source, this design, printed
  output. Japanese exists ONLY inside dataset.json string values (file itself is
  ASCII via JSON \uXXXX escapes) and in model I/O record values (same escaping).
  This design pins required Japanese substrings as \uXXXX escape sequences.
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
  DESIGN.md  README.md (post-run summary)  .gitignore
  dataset.json            A1 (version m2poc-2)
  run_m2.py               A2 (CLI orchestrator)
  m2/__init__.py          A2 (empty)
  m2/llm.py  m2/redact.py A2
  m2/routes.py            A3
  m2/grammars.py          (rev-3 DSL GBNF builders)
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
- Routes (short key = code/records/filenames; full id = report metrics tables):

| key | full id | calls/sample | stages |
| --- | --- | --- | --- |
| direct | route.direct_smt | 1 | main |
| ir | route.single_ir | 1 | main |
| stacked | route.stacked_ir | 2 | frame, rows |
| hop | route.ir_hop_chain | 3 | surface, ground, typed |
| layered | route.ckc_layered | 3 | segment, statement, rule |
| dsl | route.ckc_dsl | 1 | main |
| dslh | route.ckc_dsl_hop | 3 | surface, ground, typed |
| dslk | route.ckc_dsl_kw | 1 | main |
| dslkh | route.ckc_dsl_kw_hop | 3 | surface, ground, typed |

The last four (dsl, dslh, dslk, dslkh) are the rev-3 M4 DSL routes; full
interfaces in the "Revision 3" section. Route key order everywhere: direct, ir,
stacked, hop, layered, dsl, dslh, dslk, dslkh.
- Sources: 5 families (see dataset section); key order: directive, insert, table,
  prose, morph.
- Experiment id: rev-2 5-route field `exp.m2poc_5x5x5` (run matrix5); rev-3
  9-route DSL field `exp.m2poc_dsl`. K_SAMPLES = 5.
- Samples per item x source x route: s0 greedy `{"temperature": 0, "seed": 4242}`;
  s1..s4 stability `{"temperature": 0.7, "top_p": 0.9, "seed": 4251+n}` (n=1..4,
  so 4252..4255). `"max_tokens": 320` on every call. Every call within one sample
  uses that sample's params.
- Scale check (rev-2): 20 items x 5 sources x 5 routes x 5 samples = 2500
  records; 20 x 5 x 5 x (1+1+2+3+3) = 5000 calls. Rev-3 9-route field: 20 x 5 x 9
  x 5 = 4500 records; 20 x 5 x 5 x (1+1+2+3+3+1+3+1+3) = 9000 calls.

## Vocabulary (canonical symbols)

Variables (SMT sort): age Int; pregnant Bool; renal_impairment Bool;
hepatic_impairment Bool; on_anticoagulant Bool.
Actions: drug_aspirin, drug_warfarin, drug_ibuprofen, drug_methotrexate.
Directions: require (must administer), forbid (must not administer).
Condition ops: age: `>=  <=  >  <  =` with integer value 0..130;
bool vars: `=` with value true|false. Conditions in a rule are a conjunction (AND).
Empty conditions = rule always applies.

## dataset.json (A1, version m2poc-2)

```json
{"version": "m2poc-2",
 "vocab": {... byte-identical to m2poc-1 vocab ...},
 "sources": [{"key": "directive", "id": "src.directive",
              "en": "concise directive guideline sentence", "ja": "<short label>"},
             {"key": "insert", "id": "src.package_insert", "en": "...", "ja": "..."},
             {"key": "table", "id": "src.table_row", "en": "...", "ja": "..."},
             {"key": "prose", "id": "src.verbose_prose", "en": "...", "ja": "..."},
             {"key": "morph", "id": "src.metamorphic", "en": "...", "ja": "..."}],
 "items": [{"id": "r01a", "group": "g01",
            "ja_texts": {"directive": "...", "insert": "...", "table": "...",
                         "prose": "...", "morph": "..."},
            "gold_ir": {... unchanged from m2poc-1 ...}}, ...],
 "groups": [... unchanged from m2poc-1 ...]}
```

20 items, 10 groups, gold_ir and group table unchanged from m2poc-1 (semantic
content is held fixed; only the surface family varies). Group table (exact):

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

Item ids r01a/r01b .. r10a/r10b matching group order. File pure ASCII: compose raw
Japanese in memory, write via `json.dump(obj, f, ensure_ascii=True, sort_keys=True,
indent=1)`.

Every family text must be mechanically faithful to its item's gold_ir: every gold
condition expressed exactly once, zero conditions or clinical facts beyond gold_ir,
drug name as the vocab katakana surface, direction unambiguous. Register varies;
content does not.

### Family pins

- `directive`: the 20 m2poc-1 `ja_text` strings copied byte-identical (continuity
  with recorded run m2).
- `insert`: package-insert register. Line starts with header
  `"\u3010\u7981\u5fcc\u3011"` (forbid items) or `"\u3010\u6295\u4e0e\u3011"`
  (require items); ends with `"\u6295\u4e0e\u3057\u306a\u3044\u3053\u3068\u3002"`
  (forbid) or `"\u6295\u4e0e\u3059\u308b\u3053\u3068\u3002"` (require). Patient
  class as a noun phrase + `"\u306e\u60a3\u8005\u306b\u306f"`; condition phrasing
  otherwise free.
- `table`: terse fragment row, exactly three fields joined by ASCII `/`, ASCII `:`
  after each field name: `"\u6761\u4ef6"` (conditions; 2+ joined with
  `"\u304b\u3064"`), `"\u85ac\u5264"` (drug katakana), `"\u6307\u793a"` (value
  `"\u6295\u4e0e"` for require, `"\u6295\u4e0e\u4e0d\u53ef"` for forbid). No
  sentence-final period, no polite endings.
- `prose`: verbose formal register, one sentence, subordinate clauses allowed;
  contains `"\u60a3\u8005\u306b\u304a\u3044\u3066\u306f"`; ends
  `"\u3082\u306e\u3068\u3059\u308b\u3002"`; require items contain
  `"\u6295\u4e0e\u3092\u884c\u3046"`, forbid items
  `"\u6295\u4e0e\u3092\u884c\u308f\u306a\u3044"`. Extra words are discourse
  function only, never added content.
- `morph`: deterministic transform of the item's `directive` string, applied as
  whole-string replacements in this order: (1) each ASCII digit 0-9 to its
  full-width form U+FF10..U+FF19; (2) `"\u3001"` -> `"\uff0c"` and `"\u3002"` ->
  `"\uff0e"`; (3) `"\u6b73"` -> `"\u624d"`; (4) `"\u3053\u3068"` -> `"\u4e8b"`.
  Compute via code, never freehand; the audit recomputes and byte-compares.

## Routes and formal shapes

ParsedRule (dict, the inter-module currency):
`{"action": str, "direction": str, "formula": str, "used_vars": [str]}`
formula is an SMT-LIB Bool term over canonical variables, e.g.
`(and (= on_anticoagulant true) (<= age 70))`, single condition `(>= age 65)`,
empty conditions `true`.

IR shape (the shared final JSON form; single_ir output, hop stage `typed` output,
layered stage `rule` output, and stacked after frame+rows merge):
`{"action": <enum>, "direction": <enum>, "conditions": [{"var": <enum>, "op": <enum>, "value": int|bool}, ...]}`
`compile_ir(ir) -> ParsedRule`: deterministic; conditions -> formula
(`(= var true)` form for bools; `(op var int)` for age; 2+ conds nest in one `(and ...)`
in given order; 0 conds -> `true`). Unchanged from revision 1.

### route.direct_smt (key `direct`, 1 call, stage `main`)

Model must output exactly:

```
(declare-const <var> <Int|Bool>)        [0+ lines, vars it uses]
(define-fun applies () Bool <formula>)
; action=<action_id> direction=<require|forbid>
```

`parse_direct(text) -> (ParsedRule|None, failure_code|None)`: unchanged from
revision 1 (fence strip, balanced-paren body extraction, metadata comment regex;
failures -> `(None, "target_parse_error")`).

### route.single_ir (key `ir`, 1 call, stage `main`)

`parse_ir(text) -> (dict|None, "target_parse_error"|None)`: json.loads, must be
dict. `ir_json_schema(vocab) -> dict`: unchanged from revision 1 (enums from
vocab, additionalProperties false everywhere, value via anyOf int|bool, required
all three keys). Server enforces as grammar; admit re-checks structurally.

### route.stacked_ir (key `stacked`, 2 calls)

Existing-IR stack: a PICO-style frame, then typed condition rows.

- Stage `frame` schema `stacked_frame_schema(vocab)`:
  `{"population_ja": <string>, "intervention": <enum action ids>, "stance": <enum ["do","do_not"]>}`
  object, additionalProperties false, required all three. population_ja = the
  patient-condition phrase copied from the sentence (string content unchecked).
- Stage `rows` schema `stacked_rows_schema(vocab)`:
  `{"rows": [<condition item identical to ir_json_schema conditions items>]}`
  object, additionalProperties false, required ["rows"].
- `compile_stacked(frame, rows_obj) -> ir-shaped dict`:
  `{"action": frame["intervention"], "direction": {"do": "require", "do_not": "forbid"}[frame["stance"]], "conditions": rows_obj["rows"]}`.

### route.ir_hop_chain (key `hop`, 3 calls)

Chain of adjacent dialects, each hop a minimal delta; the sentence is visible to
hop 1 ONLY -- later hops see only the previous hop's output. Information must
survive the chain.

- Stage `surface` schema `hop_surface_schema()` (no vocab enums):
  `{"drug_ja": <string>, "polarity": <enum ["do","do_not"]>, "when_ja": [<string>, ...]}`
  required all three. Prompt sees the rule sentence only (no vocabulary block).
- Stage `ground` schema `hop_ground_schema(vocab)`:
  `{"action": <enum action ids>, "polarity": <enum ["do","do_not"]>, "when_ja": [<string>, ...]}`
  required all three. Prompt sees: actions vocab lines + `{PRIOR_JSON}`. Delta:
  ground drug_ja to its canonical id; instruction says copy polarity/when_ja
  unchanged (not enforced by admission).
- Stage `typed` schema = `ir_json_schema(vocab)`. Prompt sees: variables vocab
  lines + ops line + direction mapping line + `{PRIOR_JSON}`. Delta: type the
  when_ja phrases into condition objects; polarity do->require, do_not->forbid.

### route.ckc_layered (key `layered`, 3 calls)

CKC layers stage by stage; the sentence is visible at EVERY stage; each stage
deepens typing (classify -> semi-typed statement -> fully typed rule).

- Stage `segment` schema `layered_segment_schema()` (no vocab enums):
  `{"kind": <enum ["recommendation","contraindication"]>, "drug_ja": <string>}`
  required both. Prompt sees the rule sentence only.
- Stage `statement` schema `layered_statement_schema(vocab)`:
  `{"action": <enum action ids>, "modality": <enum ["must","must_not"]>, "condition_phrases_ja": [<string>, ...]}`
  required all three. Prompt sees: actions vocab lines + `{PRIOR_JSON}` + sentence.
- Stage `rule` schema = `ir_json_schema(vocab)`. Prompt sees: full vocab block +
  direction line + ops line + `{PRIOR_JSON}` + sentence.

### Prompts (A3)

`build_prompts(vocab) -> {route_key: {stage_name: {"system": str, "user_template": str}}}`
for all five routes; single-call routes use stage name `main`. The `direct` and
`ir` `main` system/user_template strings stay byte-identical to the revision 1
builders (behavior continuity). user_templates carry slots `{JA_TEXT}`,
`{PRIOR_JSON}`, `{FRAME_JSON}` (consume with .replace, never .format -- formulas
and JSON carry braces). Slot table (exact):

| route | stage | slots | response_format schema |
| --- | --- | --- | --- |
| direct | main | JA_TEXT | none |
| ir | main | JA_TEXT | ir_json_schema |
| stacked | frame | JA_TEXT | stacked_frame_schema |
| stacked | rows | JA_TEXT, FRAME_JSON | stacked_rows_schema |
| hop | surface | JA_TEXT | hop_surface_schema |
| hop | ground | PRIOR_JSON | hop_ground_schema |
| hop | typed | PRIOR_JSON | ir_json_schema |
| layered | segment | JA_TEXT | layered_segment_schema |
| layered | statement | JA_TEXT, PRIOR_JSON | layered_statement_schema |
| layered | rule | JA_TEXT, PRIOR_JSON | ir_json_schema |

routes.py exposes this as data:
`route_stages(vocab) -> {route_key: [{"stage": str, "schema": dict|None, "slots": [str, ...]}, ...]}`
(stage order = call order; schema None only for direct/main).

Vocabulary blocks built from loaded dataset vocab, never literals: `_vocab_block`
(all lines, as revision 1), `_actions_block` (action lines only), `_vars_block`
(variable lines only). Direction semantics one-liner as revision 1; ops line as
revision 1 ir prompt; hop `typed` and layered `rule` prompts also carry the
mapping line `polarity/modality: do|must -> require, do_not|must_not -> forbid`
(adapted per route). Every stage prompt carries ONE worked example (same example
rule everywhere: require drug_ibuprofen if age>=12, not in dataset), expressed in
that stage's input/output shape; multi-stage examples show the example prior JSON
too. English instruction text; output-format block exact per stage ("Output
exactly this format, nothing else" for direct; JSON stages may say "Output only
the required JSON"). System prompts one line each. Japanese enters prompts ONLY
via vocab ja fields and the {JA_TEXT} slot.

`prompt_sha(prompts) -> {route_key: sha256}` over
`json.dumps(prompts[route_key], sort_keys=True, ensure_ascii=True)` (route
subtree, so multi-stage routes hash their stage dict).
`schema_shas(vocab) -> {route_key: {stage: sha256}}` over the same canonical dump
of each JSON stage schema (direct omitted).

## m2/llm.py + run_m2.py (A2)

m2/llm.py unchanged from revision 1:

```python
class LlamaServer:
    def __init__(self, server_bin, model_path, port=8077, ctx=4096, threads=8): ...
    def start(self, log_path): ...   # Popen, poll GET /health until {"status":"ok"} (timeout 180s)
    def stop(self): ...              # terminate, wait
    def props(self) -> dict: ...     # GET /props json
def chat(port, messages, seed, temperature, top_p=None, max_tokens=320,
         response_format=None, timeout_s=300) -> dict:
    # returns {"request": <body dict>, "response": <full response json>, "duration_ms": int}
```

response_format wrap for a JSON stage (OpenAI shape, verified live in revision 1):
`{"type": "json_schema", "json_schema": {"name": "<route>_<stage>", "strict": true, "schema": <schema>}}`
Content extraction: `response["choices"][0]["message"]["content"]`.

m2/redact.py unchanged: `redact(s) -> s` if all chars < U+0080 else
`<ja {len(s)} {sha256(s.encode())[:8].hex()}>`.

run_m2.py subcommands (argparse):
- `setup-check`: unchanged (binaries exist, model sha, z3 --version, ok lines).
- `run --run-id ID [--groups all|gNN,..] [--k 5] [--routes all|direct,ir,..] [--sources all|directive,table,..] [--no-server]`:
  load dataset; validate route keys against the route table and source keys
  against dataset sources; gold gate first (unchanged -- gold is shared across
  sources, gate runs once); start server (log runs/ID/server.log); save
  server_props.json. todo order: for item in group-order item ids, for source in
  source key order, for route in route key order, for n in 0..k-1. Resume: skip
  when the record file exists. Per todo entry, call stages sequentially per
  `route_stages`: build messages `[{"role": "system", ...}, {"role": "user", ...}]`
  from `prompts[route][stage]`; fill `{JA_TEXT}` with
  `items[iid]["ja_texts"][source_key]`; fill `{FRAME_JSON}`/`{PRIOR_JSON}` with
  the RAW content string of the immediately preceding stage call ("" when that
  call failed to produce content) -- the runner never validates stage output, it
  only threads strings; response_format from the stage schema (None for direct).
  Write record
  `runs/ID/records/<item>.<source_key>.<route_key>.s<n>.json` =
  `{"item": id, "source": source_key, "route": route_key, "sample": n,
    "seed": ..., "temperature": ...,
    "calls": [{"stage": str, "request": ..., "response": ..., "duration_ms": int}, ...]}`
  (calls in stage order; single-call routes have one entry, stage "main").
  Progress line `<item> <source> <route> s<n> <ok> <n_done>/<n_total>` where ok
  requires every call's response to carry choices; stop server; then score.
- `score --run-id ID`: m2.score.score_run(run_dir, dataset) -> writes report.json,
  report.md, report.ja.md, refreshes `runs/latest` symlink (relative, replace).
- `replay --run-id ID`: unchanged mechanism (recompute report dict from records;
  byte-compare canonical dumps excluding key "replay"; rewrite with replay.status;
  exit 1 on mismatch).

K default 5. Smoke recipe (post-build gate):
`run --run-id smoke2 --groups g01 --sources directive,table --routes all --k 1`
= 2 items x 2 sources x 5 routes = 20 records, 40 calls; then `score` + `replay`
on smoke2 must pass.

## m2/admit.py + m2/verdict.py (A4)

```python
CANON_DECLS = "(declare-const age Int)\n..."  # all five, unchanged
def run_z3(smt_text, timeout_s=5) -> {"status": "sat"|"unsat"|"unknown"|"error", "stdout": str, "stderr": str}
def admit_route(route_key, contents, vocab) -> AdmissionResult
# contents: list of per-stage content strings from a record's calls, stage order.
# AdmissionResult: {"syntactic_valid": bool, "admitted": bool,
#   "failure_code": None|"target_parse_error"|"ai_schema_violation"|"solver_execution_failure",
#   "failed_stage": None|str,   # stage name of the first failing stage
#   "rule": ParsedRule|None}
```

Dispatch per route_key:
- `direct`: revision 1 admit_direct semantics on contents[0] (parse_direct fail ->
  syntactic_valid False, target_parse_error; else z3 sanity check of
  `CANON_DECLS + (define-fun applies () Bool <formula>) + (assert applies) + (check-sat)`,
  z3 error -> syntactic_valid False, target_parse_error -- off-vocabulary symbols
  land here deliberately; else syntactic_valid True, then action/direction in
  enums else ai_schema_violation). failed_stage "main" on any failure.
- `ir`: revision 1 admit_ir semantics on contents[0] (json parse fail ->
  target_parse_error, syntactic_valid False; structural check against vocab
  enums/ops/types/ranges -- int 0..130 for age with any of the 5 ops, bool vars op
  `=` value bool -- violation -> ai_schema_violation with syntactic_valid True;
  pass -> compile_ir + z3 sanity parse which must never fail, if it does ->
  solver_execution_failure). failed_stage "main" on any failure.
- `stacked` / `hop` / `layered` (uniform pattern): walk stages in order. JSON
  parse of stage content must yield a dict -> else syntactic_valid False,
  target_parse_error, failed_stage = that stage. After ALL stages parse,
  syntactic_valid True; then structural check each stage against its OWN schema
  shape (enums/types/ranges; string-array fields must be lists of str; condition
  items checked exactly like ir) in stage order -- first violation ->
  ai_schema_violation, failed_stage = that stage. Cross-stage copy instructions
  are NOT enforced. Pass -> final ir-shaped dict (stacked: compile_stacked merge;
  hop/typed and layered/rule: the final stage object) -> compile_ir + z3 sanity
  parse (must never fail; if it does -> solver_execution_failure).
- admitted = every gate above passes (including z3 sanity). rule = ParsedRule on
  admission, else None.

```python
def group_verdict(rule_a, rule_b) -> {"verdict": "conflict"|"no_conflict"|"incomputable",
                                      "reason": str, "z3": dict|None}
```
Unchanged from revision 1 (member None -> incomputable/member_inadmissible;
different action -> no_conflict/different_action; same direction ->
no_conflict/same_direction; else overlap query sat -> conflict/overlap_sat,
unsat -> no_conflict/overlap_unsat, unknown|error -> incomputable/solver_error).

## m2/score.py + m2/report.py (A5)

`score_run(run_dir: Path, dataset: dict) -> report: dict` then writers. Constants:
`ROUTE_IDS = {"direct": "route.direct_smt", "ir": "route.single_ir", "stacked": "route.stacked_ir", "hop": "route.ir_hop_chain", "layered": "route.ckc_layered"}`;
source key -> id from dataset sources. Steps:

1. Gold gate: unchanged -- for each group, group_verdict over compile_ir of the two
   gold_irs must equal gold_verdict (assert). Field `gold_gate: {"pass": true, "groups": 10}`.
2. Read all records (trust the json keys item/source/route/sample, not filenames).
   contents = per call `response["choices"][0]["message"]["content"]`, "" when
   absent. admit_route -> raw_row
   `{"item", "group", "source": <source id>, "route": <route id>, "sample",
     "seed", "syntactic_valid", "admitted", "failure_code", "failed_stage",
     "duration_ms_total": <sum over calls>,
     "output_sha256": sha256 of json.dumps(contents, ensure_ascii=True).encode()}`.
3. group_rows: per group x source x sample: verdict via group_verdict on the two
   members' rules from that source+sample ->
   `{"group", "source": <id>, "route": <id>, "sample", "verdict", "reason",
     "gold", "correct": verdict==gold}`.
4. metrics (rational dicts `{"num": int, "den": int, "value": float}` per cell;
   4 metric keys: syntactic_validity, admission_rate, verdict_accuracy_greedy,
   verdict_stability -- stability = groups whose k sample verdicts are all
   computable AND identical):
   `"metrics": {"pooled": {<route id>: {<metric>: cell}},`
   `            "by_source": {<source id>: {<route id>: {<metric>: cell}}}}`.
   Denominators derive from records actually read (partial runs score their
   covered cells): by_source validity/admission = samples read for that source x
   route, greedy = groups with an s0 group_row, stability = groups with all k
   samples present; pooled sums the by_source numerators and denominators.
   Full-run values: 100/10/10 per source cell, 500/50/50 pooled.
5. baseline_delta (floats round 4; direct's delta = 0.0):
   `{<metric>: {"pooled": {<route id>: {"value": v, "delta": v - v_direct}},`
   `            "by_source": {<source id>: {<route id>: {"value": v, "delta": ...}}}}}`.
6. taxonomy: `{"pooled": {<route id>: {<code>: n}}, "by_source": {<source id>: {<route id>: {<code>: n}}}}`.
   Codes: failure_code counts over all raw rows + group reasons
   member_inadmissible/solver_error over all group rows + greedy verdict errors as
   false_positive_conflict (verdict conflict, gold no_conflict) /
   false_negative_conflict (gold conflict, verdict no_conflict or incomputable)
   over s0 group rows. Zero counts omitted.
7. findings: one per source x group cell having at least one s0 group_row
   (full run: 50):
   `{"source": <id>, "group", "kind", "gold",
     "greedy": {<route id>: <s0 verdict>},
     "quote_a": <that source family's ja_texts for member a>, "quote_b": ...,
     "en": "gold X; direct A; single-IR B; stacked C; hop-chain D; layered E",
     "ja": same sentence in Japanese}`.
   JA template strings live in report.py as \uXXXX escape literals.
8. identities: model sha + name, llama_cpp_build "b9601 (4c6595503)", z3_version
   (live `z3 --version` output), dataset_sha256, prompt_sha per route key,
   schema_sha256 = schema_shas(vocab), server_props subset (model path, n_ctx)
   from runs/ID/server_props.json.
9. echo blocks: `"sources": <dataset sources array verbatim>`,
   `"routes": [{"key": k, "id": ROUTE_IDS[k]} in route order]`,
   config (k, seeds map s0..s4, temperature, top_p, max_tokens, port, route keys,
   source keys), counts (records read, calls read, items/groups/sources covered),
   `"replay": {"status": "not_yet_replayed"}`, experiment id, run_id.

Canonical bytes: `(json.dumps(report, ensure_ascii=True, sort_keys=True, indent=1) + "\n").encode()`.
Deterministic: no timestamps anywhere (durations only inside raw rows).

report.md / report.ja.md: `render_md(report, lang)` purely from the report dict:
title + identity block; gold gate line; pooled metrics table (rows = 4 metrics,
cols = 5 routes); pooled baseline-delta table (rows = metrics, cols = the 4
non-direct routes, delta vs direct); per source (5 blocks): metrics table + greedy
verdict matrix (rows g01..g10: kind, gold, then one col per route's s0 verdict,
wrong cells marked `*`); pooled taxonomy table (rows = codes, cols = routes);
findings: ONLY rows where at least one route's greedy verdict is wrong, plus one
line "N all-correct source x group rows omitted"; replay line. Quoted Japanese
spans stay verbatim in both renderings (they are quotes; the md files are UTF-8
renderings of escaped JSON). All JA literal templates in source as \uXXXX escapes.

## ui/index.html (A6)

Single file, vanilla JS/CSS, no CDN. Loads `../runs/latest/report.json` via fetch
with `?run=<id>` override; `?lang=` override; EN | JA toggle persisted in
localStorage (default EN, set `<html lang>`); i18n dict in source, ja values as
\uXXXX escapes; elements carry data-i18n keys. Source labels (en/ja) from the
report's sources echo; route columns in route order everywhere. Sections:
1. Verdict banner: pooled verdict_accuracy_greedy for all 5 routes on one line,
   best non-direct delta vs direct called out.
2. Scope tabs: Pooled | one tab per source (label from sources echo). Tabs switch
   sections 3-5 between pooled and that source's slice.
3. Metrics table: rows = 4 metrics, cols = 5 routes; per non-direct cell show the
   delta vs direct in the same scope, green when above direct, red when below.
4. Group matrix: rows g01..g10 (kind, gold) x route s0 verdict cols; wrong red,
   correct green, incomputable gray. Pooled tab: per-cell "n/5 sources correct"
   coloring (green 5/5, red 0/5, amber between) -- tooltip lists the per-source
   verdicts.
5. Taxonomy: code x route count grid for the scope.
6. Findings: filter = scope (tab) + default "only rows with a wrong greedy
   verdict" toggle; per row the lang-dependent sentence (en/ja field) + the two
   quoted Japanese spans verbatim.
7. Identity footer: model, build, z3, dataset/prompt/schema hashes (truncate 12),
   replay status, gold gate, experiment id, counts.
Layout: clean, compact, system-ui font, max-width 1200px, the five route columns
visually paired. No external requests beyond the report fetch.

## Audit stage

B1 (after A1): per family x item: recompute morph from directive and
byte-compare; check every family pin substring (headers, endings, field skeleton);
faithfulness cross-check of each ja_texts value against gold_ir (each condition
exactly once, no extra content, drug katakana matches vocab, direction matches
gold); ids/groups/vocab byte-identical to m2poc-1; sources array exact; file
ASCII purity. Output ASCII table family x check: ok/issue counts; fix issues in
place and re-run.

B2 (after all authors): py_compile all (incl. m2/grammars.py), `python3 -c`
import chain, signature cross-check vs this design, dataset structural check
(ids/refs/enums/ASCII purity), the smoke recipe, fix mechanical seams in place,
report what changed. Rev-3 DSL gates live in the "Revision 3" section.

## Metrics intent (for report wording)

Locked synthetic fixture measurement, instrument = the deterministic gate chain
(grammar/admission/z3) shared by all routes; the varying factors are the
translation route and the surface family of the same underlying rules. Findings
wording: measured rates on synthetic fixtures, no clinical claims. The headline
artifacts are the pooled baseline_delta table and the per-source metric matrix.

## Revision 3 -- M4 invented DSLs (route field 5 -> 9)

Revision 3 adds SPEC sec.11's invented IR/DSLs: two project-born DSL candidates
under a GBNF grammar mask, each in a singular and a hop-layered config (4 new
routes), parsed deterministically into the SAME IR dict the five rev-2 routes
share, then `compile_ir` (unchanged) -> z3. Dataset, sources, gold, seeds, and
the five rev-2 routes stay byte-identical to rev-2; the DSL run is a 9-route
superset.

Thesis (sec.11; the bar is route.single_ir, the rev-2 winner, and
route.direct_smt, the M2 baseline): rev-2 pinned the discriminating gate at
admission-time type coherence -- the var/op/value coupling JSON-Schema -> GBNF
cannot express, so the IR routes saturate syntactic validity yet still leak
incoherent conditions. A hand-written GBNF DOES couple var/op/value, so a
grammar-masked DSL drives BOTH syntactic_validity and admission_rate toward
~100% and pushes the signal onto verdict_accuracy and stability (semantic
mistranslation the grammar cannot prevent). The two candidates hold grammar
strength fixed (both tight) and vary token compactness -- the sec.11 design
dimension, a sec.12 seed coordinate: candidate T is a terse infix line,
candidate K a verbose keyword block. A documented null result (no DSL beats
single_ir) is first-class.

Unit sequencing: dsl-impl is one unit, built in two
internally-gated passes in a single commit. Pass 1 (SINGULAR): m2/grammars.py
(terse + keyword grammars, grammar_shas), the two canonical parsers, the chat
grammar field, run threading, score's ROUTE_KEYS/ROUTE_IDS + dual baseline, and
routes dsl + dslk -- smoked green. Pass 2 (HOP): the shared surface/ground DSL
dialects and routes dslh + dslkh -- smoked green. dsl-run then runs all nine and
adds the report/UI widening + README rows; poc-accept ranks the DSL field
against single_ir and direct.

### Candidate syntaxes

Both candidates parse to the rev-2 IR dict
`{"action": <enum>, "direction": <enum>, "conditions": [{"var","op","value"}, ...]}`
and reuse `compile_ir`. Canonical examples (g01a forbid-aspirin-if-pregnant;
g05b require-aspirin-if-anticoag-and-age<=70; an empty-guard require):

Candidate T -- terse infix (route.ckc_dsl singular, route.ckc_dsl_hop layered):
```
forbid drug_aspirin when pregnant=true
require drug_aspirin when on_anticoagulant=true & age<=70
require drug_ibuprofen
```
Candidate K -- keyword block (route.ckc_dsl_kw singular, route.ckc_dsl_kw_hop):
```
RULE forbid drug_aspirin
GUARD pregnant = true
```
```
RULE require drug_aspirin
GUARD on_anticoagulant = true AND age <= 70
```
```
RULE require drug_ibuprofen
GUARD none
```
T omits `when` for an empty guard; K writes `GUARD none`. K is token-heavier for
the identical rule -- the compactness axis. The grammar pins spacing exactly
(T: no spaces around `=`/ops; K: spaces, ` AND `, a newline), so the parsers are
total on grammar output.

### GBNF grammars (m2/grammars.py)

New asset module; builders read the loaded dataset vocab so action/var/direction
enums never drift (action = `vocab.actions` ids; boolvar = the Bool-typed
`vocab.variables` ids; age is the lone Int var; direction = `vocab.directions`).
The age value is pinned to the exact 0..130 range in the grammar (admission
re-checks as a safety net). `grammar_shas(vocab) -> {route_key: {stage: sha256}}`
over the GBNF text of each grammar-bearing stage, mirroring `schema_shas`. GBNF
strings are ASCII; the `\n` in a char class is the two-character newline escape.

`dsl_terse_gbnf(vocab)` (route.ckc_dsl main, route.ckc_dsl_hop typed):
```
root      ::= direction " " action guard?
guard     ::= " when " cond (" & " cond)*
direction ::= "forbid" | "require"
action    ::= "drug_aspirin" | "drug_warfarin" | "drug_ibuprofen" | "drug_methotrexate"
cond      ::= boolcond | agecond
boolcond  ::= boolvar "=" boolval
boolvar   ::= "pregnant" | "renal_impairment" | "hepatic_impairment" | "on_anticoagulant"
boolval   ::= "true" | "false"
agecond   ::= "age" ageop ageval
ageop     ::= ">=" | "<=" | ">" | "<" | "="
ageval    ::= "130" | "12" [0-9] | "1" [0-1] [0-9] | [1-9] [0-9] | [0-9]
```
The boolvar/boolval and age/ageop/ageval coupling is the grammar's whole point:
`pregnant>=65` is unreachable. `ageval` is exactly 0..130 (no leading zeros).

`dsl_kw_gbnf(vocab)` (route.ckc_dsl_kw main, route.ckc_dsl_kw_hop typed):
```
root      ::= "RULE " direction " " action "\n" "GUARD " guardbody
guardbody ::= "none" | cond (" AND " cond)*
direction ::= "forbid" | "require"
action    ::= <as terse>
cond      ::= boolcond | agecond
boolcond  ::= boolvar " = " boolval
boolvar   ::= <as terse>
boolval   ::= "true" | "false"
agecond   ::= "age " ageop " " ageval
ageop     ::= ">=" | "<=" | ">" | "<" | "="
ageval    ::= <as terse>
```
Same coupling, verbose surface (RULE/GUARD/AND keywords, spaces around `=`/ops,
a newline).

`dsl_surface_gbnf()` (shared; both hop routes' surface stage; no vocab, drug +
phrases are free Japanese, like hop_surface_schema):
```
root      ::= direction " " raw guard?
guard     ::= " when " raw (" & " raw)*
direction ::= "forbid" | "require"
raw       ::= char+
char      ::= [^ &\n]
```
`raw` is any run of non-space, non-`&`, non-newline characters, so Japanese drug
names and condition phrases pass while ` when ` and ` & ` stay delimiters.

`dsl_ground_gbnf(vocab)` (shared; both hop routes' ground stage):
```
root      ::= direction " " action guard?
guard     ::= " when " raw (" & " raw)*
direction ::= "forbid" | "require"
action    ::= <vocab action ids>
raw       ::= char+
char      ::= [^ &\n]
```
Delta from surface: the drug slot becomes the canonical action enum; phrases stay
free Japanese.

### DSL parsers (m2/routes.py)

Deterministic, total on grammar output, lenient on surrounding whitespace; they
STRUCTURE only -- vocab validation stays in admission (`_ir_structural_ok`),
mirroring the rev-2 parse_ir/admit split. All return `(obj|None, code|None)`,
code `"target_parse_error"` on shape mismatch.

`parse_dsl_terse(text) -> (ir_dict|None, code)`: strip; partition on the first
` when ` into head + guardpart; `head` matches `^(forbid|require) (\S+)$` ->
direction, action; conditions = [] when no guard else `guardpart.split(" & ")`,
each matched as age `^age(>=|<=|>|<|=)(\d+)$` (var "age", int value) or bool
`^([a-z_]+)=(true|false)$` (op "=", bool value); any non-match -> parse error.
Returns `{"action","direction","conditions":[{"var","op","value"}, ...]}`.

`parse_dsl_kw(text) -> (ir_dict|None, code)`: strip; split lines; line 0
`^RULE (forbid|require) (\S+)$`; line 1 `^GUARD (.+)$` -> body; conditions = []
when body == "none" else `body.split(" AND ")`, each matched as age
`^age (>=|<=|>|<|=) (\d+)$` or bool `^([a-z_]+) = (true|false)$` (spaces); any
non-match -> parse error. Same ir_dict shape.

`parse_dsl_surface(text) -> ({"direction","drug","conds":[str]}|None, code)`:
strip; partition ` when `; head `^(forbid|require) (\S+)$` -> direction, drug;
conds = [] or `guardpart.split(" & ")`.
`parse_dsl_ground(text) -> ({"direction","action","conds":[str]}|None, code)`:
identical, the second head token named `action` (enum-checked in admission).

### route_stages, chat grammar field, run threading

The stage descriptor gains a `grammar` slot:
`route_stages(vocab) -> {route_key: [{"stage", "schema": dict|None, "grammar": str|None, "slots": [...]}, ...]}`.
A JSON stage has `grammar` None and `schema` driving `response_format` (rev-2
unchanged); a DSL stage has `schema` None and `grammar` the GBNF string;
direct/main keeps both None. New entries (stage order = call order):

| route | stage | slots | grammar |
| --- | --- | --- | --- |
| dsl | main | JA_TEXT | dsl_terse |
| dslk | main | JA_TEXT | dsl_kw |
| dslh | surface | JA_TEXT | dsl_surface |
| dslh | ground | PRIOR_JSON | dsl_ground |
| dslh | typed | PRIOR_JSON | dsl_terse |
| dslkh | surface | JA_TEXT | dsl_surface |
| dslkh | ground | PRIOR_JSON | dsl_ground |
| dslkh | typed | PRIOR_JSON | dsl_kw |

dslh/dslkh share surface + ground (grammar AND prompt) and diverge only at
`typed` (terse vs keyword), mirroring route.ir_hop_chain stage for stage except
the wire dialect -- the controlled "DSL landing vs JSON-IR landing" contrast. The
`{PRIOR_JSON}` slot carries the prior stage's raw string verbatim (the runner
already threads it); for DSL that string is the previous DSL line. Later hops see
only the prior line (no sentence), as in route.ir_hop_chain.

`m2/llm.py`: `chat(..., grammar=None)` adds `body["grammar"] = grammar` when set
(verified live on b9601: `/v1/chat/completions` honors a GBNF `grammar` body
field). A stage carries EITHER `response_format` (JSON) OR `grammar` (DSL), never
both.

`m2/run_m2.py`: per stage pass `grammar=st["grammar"]` to `chat` (None for JSON
stages); validate `--routes` keys against `route_stages(vocab)` keys (a route
whose stages are not yet wired is rejected, not silently skipped); the DSL run's
experiment id is `exp.m2poc_dsl`.

### Prompts (DSL stages, m2/routes.py build_prompts)

Same recipe as rev-2: English instructions, ONE worked example per stage built
from vocab ja fields (the example rule require drug_ibuprofen if age>=12, reusing
`ibu_ja`/`age_ja`/`ex_cond`), an exact output-format block,
`_vocab_block`/`_actions_block`/`_vars_block` reused, Japanese only via vocab ja
fields and the {JA_TEXT} slot. Per stage:

- dsl/main: full vocab head + direction line + the terse format
  (`<direction> <action_id> [when <cond> & <cond> ...]`, cond = `age<op><0-130>`
  or `<bool_var>=<true|false>`, omit `when` for none) + ops line + example
  `require drug_ibuprofen when age>=12`. Slots [JA_TEXT].
- dslk/main: full vocab head + the keyword format (two lines
  `RULE <direction> <action_id>` / `GUARD <cond> AND <cond> ...`, `GUARD none`
  for none) + ops line + the two-line example. Slots [JA_TEXT].
- {dslh,dslkh}/surface (shared prompt): "write one line
  `<direction> <drug as written> [when <phrase> & ...]`; direction forbid or
  require" + example `require <ibu_ja> when <ex_cond>`. Slots [JA_TEXT].
- {dslh,dslkh}/ground (shared): actions head + "replace the drug name with its
  action id; keep direction and phrases unchanged" + example
  `require <ibu_ja> when <ex_cond>` -> `require drug_ibuprofen when <ex_cond>`.
  Slots [PRIOR_JSON].
- dslh/typed: vars head + ops + "type each condition phrase as `age<op><0-130>`
  or `<bool_var>=<true|false>`; keep direction and action" + example
  `require drug_ibuprofen when <ex_cond>` -> `require drug_ibuprofen when age>=12`.
  Slots [PRIOR_JSON]. dslkh/typed: same instruction, the keyword output shape.

`prompt_sha` (rev-2, iterates the prompts subtree) extends automatically;
`schema_shas` is unchanged (DSL stages carry no schema); `grammar_shas` is new.

### Admission (m2/admit.py)

`admit_route` dispatch gains: dsl -> `admit_dsl(c0, vocab, parse_dsl_terse)`;
dslk -> `admit_dsl(c0, vocab, parse_dsl_kw)`; dslh ->
`_admit_dsl_hop(contents, vocab, parse_dsl_terse)`; dslkh ->
`_admit_dsl_hop(contents, vocab, parse_dsl_kw)` (c0 = contents[0] if contents
else "").

`admit_dsl(text, vocab, parser)`: parser -> ir; ir None -> (False, False, code,
"main", None); not `_ir_structural_ok(ir, vocab)` -> (True, False,
"ai_schema_violation", "main", None); else `compile_ir` + z3 sanity, z3 error ->
(True, False, "solver_execution_failure", "main", None); else admitted.

`_admit_dsl_hop(contents, vocab, typed_parser)`: pad contents to 3; parse
surface, ground, typed in order (parse_dsl_surface, parse_dsl_ground,
typed_parser) -- first parse failure -> syntactic_valid False,
target_parse_error, that stage. After all parse, syntactic_valid True; structural
pass in order: ground `action` in vocab action ids else ai_schema_violation at
"ground"; typed `_ir_structural_ok` else ai_schema_violation at "typed" (surface
needs no extra check -- its regex pins direction and a non-empty drug). Then
`compile_ir(typed_ir)` + z3 sanity (error -> solver_execution_failure at
"typed"). This mirrors rev-2 `_admit_multi`'s parse-all-then-check-all shape,
keeping per-stage failure attribution comparable across routes.

### Scoring + report (m2/score.py, m2/report.py)

- `ROUTE_KEYS = ("direct","ir","stacked","hop","layered","dsl","dslh","dslk","dslkh")`;
  `ROUTE_IDS` adds dsl->route.ckc_dsl, dslh->route.ckc_dsl_hop,
  dslk->route.ckc_dsl_kw, dslkh->route.ckc_dsl_kw_hop. (Authored whole in
  dsl-impl-core; hop routes simply have no records until dsl-impl-hop, scored as
  empty cells per the rev-2 partial-run rule.)
- baseline_delta gains a SECOND baseline: each cell becomes
  `{"value", "delta", "delta_ir"}` where `delta` is vs route.direct_smt (rev-2,
  unchanged -- direct's delta 0.0) and `delta_ir` is vs route.single_ir
  (single_ir's delta_ir 0.0). sec.11 measures deltas against direct_smt AND the
  rev-2 field winner; the roadmap names single_ir as that bar. delta_ir is null
  only when single_ir is absent from a scope (full run: always present).
  render_md keeps the rev-2 vs-direct table from `delta`; dsl-run adds a
  vs-single_ir table from `delta_ir`.
- `FINDING_LABELS` adds dsl->"dsl", dslh->"dsl-hop", dslk->"dsl-kw",
  dslkh->"dsl-kw-hop" (ASCII, no \uXXXX needed); finding_en/finding_ja already
  iterate route order, so the per-finding sentences extend automatically
  (.get fallback keeps them safe before the labels land).
- report identities (step 8) add `grammar_sha256 = grammar_shas(vocab)` beside
  schema_sha256. All metric/taxonomy/per-source tables widen to 9 route columns
  by iterating ROUTE_KEYS (no per-route literals to touch).

### UI (ui/index.html, dsl-run)

Route columns extend to 9 from the routes echo (already route-driven). The
verdict banner adds the best DSL route's pooled verdict_accuracy_greedy and its
delta vs single_ir (the headline M4 claim). The metrics table shows, per DSL
cell, the delta vs single_ir alongside the existing delta vs direct. No new
external requests.

### Run, audit, gates

The DSL run (`exp.m2poc_dsl`, a new run id) is a 9-route superset over the rev-2
inputs; because dataset/sources/gold/seeds and the five rev-2 routes are
byte-identical to matrix5, dsl-run MAY seed the run dir with matrix5's
five-route records (resume skips existing files) and run only the four DSL
routes, or run all nine fresh -- either scores identically.

B2 also py_compiles m2/grammars.py, import-chains it, and cross-checks the DSL
parser/grammar/admit/score signatures against this section. dsl-impl smoke checkpoints (one
per pass; k=1, one group, one source; include direct + ir so both baseline-delta
tables carry their baselines -- render_md is robust to absent routes, rendering
"-"):
- pass 1 singular: `run --run-id smoke_dslc --groups g01 --sources directive --routes direct,ir,dsl,dslk --k 1`
  then `score` + `replay`.
- pass 2 hop: `run --run-id smoke_dslh --groups g01 --sources directive --routes direct,ir,dslh,dslkh --k 1`
  then `score` + `replay`.
ASCII discipline holds: GBNF strings, parsers, and prompts are ASCII; Japanese
enters only via vocab ja fields, the {JA_TEXT} slot, and recorded model I/O
(the surface/ground DSL outputs carry Japanese phrases, redacted in stdout).

### Metrics intent (M4)

A tight grammar collapses syntactic_validity and admission_rate toward ~100% for
the DSL routes, so the discriminating signal moves to verdict_accuracy and
stability (the grammar cannot prevent semantic mistranslation). T-vs-K isolates
token compactness; the hop configs mirror route.ir_hop_chain except the landing
dialect, isolating whether an invented DSL landing beats a JSON-IR landing on the
final typing hop. Baselines: route.direct_smt (the weak baseline) and
route.single_ir (the rev-2 field winner, the bar M4 must clear). A documented
null result -- no DSL form beats single_ir -- is a first-class outcome (sec.11).
