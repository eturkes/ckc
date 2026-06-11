# agentlanguages.dev — full catalogue

> Machine-readable full text of the agentlanguages.dev catalogue: 32 languages designed for AI agents to author code, in five buckets — three philosophical camps (syntactic, verification, orchestration) plus adjacent and unclassified. Full-text companion to the short index at https://agentlanguages.dev/llms.txt, so an agent can fetch the whole catalogue in one HTTP round-trip rather than 33 (homepage + one per entry). Each entry: `## Name` heading; metadata block (camp, author, implementation language, compilation target, licence, first seen, maturity, site, repo, paper, agent tooling); editorial prose; where present, design-DNA cross-references and timeline events. Originally catalogued in the post "Three camps alike in dignity" (https://negroniventurestudios.com/2026/05/20/three-camps-alike-in-dignity/), Negroni Venture Studios blog. Maintained by Alasdair Allan. Editorial principles: descriptive, not promotional; no ranking; inclusion requires designers explicitly targeting LLMs/agents as authors; tools that *use* an LLM at runtime are out of scope.

---

# Syntactic camp (12)

> If models trip on syntax, strip ambiguity from the syntax itself. The camp treats the problem as representation — models choke on tokens that mean different things in different positions, operators needing disambiguation, whitespace that might or might not be load-bearing. Answer: a syntax where every token has one job.

## Axis

Camp: Syntactic (also spans Verification) | Author: Vladimir Melnic | Impl: Rust | Target: native artefacts (SQL DDL, Rust/axum server, TypeScript and Rust client SDKs, OpenAPI, GraphQL); also interpreted directly from the AST via axum + sqlx | Licence: Unknown | First seen: May 2026 | Maturity: working compiler | Site/Repo: https://github.com/vmelnic/axis
Agent tooling: CLAUDE.md; `axis --constrain` (grammar state machine as JSON); `axis --logit-masks` (per-state binary masks over the Axis token vocabulary); `axis --completions` (parser state and valid next tokens at cursor); `axis --lsp` (stdio language server).

### Key idea

Bet: a backend authored in twelve constructs with an LL(1) grammar and prefix-only expressions fails less than free-form framework code, for small-LLM authors; the toolchain ships grammar-aware logit masks for constrained decoding.

## The thesis.

Most entries target "LLMs" generically; Axis targets the smallest practical agents — v0.2 spec: "target authors: 1B, 3B, 7B parameter models with 2K–32K context windows" — and constrains the surface accordingly. Twelve top-level keywords (`SHAPE`, `SOURCE`, `REALM`, `FLOW`, `SAGA`, `SURFACE`, `POLICY`, `SERVICE`, `MIGRATE`, `STREAM`, `FUNC`, `STORAGE`) cover a production backend; grammar is LL(1), one token of lookahead, no backtracking; expressions are prefix-only (`FILTER id EQ path.id`, never `id == path.id`); "one operation per binding" forbids compound expressions. Pullquote: "Axis is not a better Python. It is the negation of human-centric programming."

Distinctive: constrained decoding as toolchain output. `axis --constrain` exports the parser as a JSON state machine; `axis --logit-masks` a per-state binary mask over the Axis token vocabulary. `src/editor/constrain.rs` enumerates 41 parse states (`TopLevel`, `ShapeBody`, `FlowBodyField`, `FilterClause`, …) with allowed token sets; a host harness maps the masks onto an LLM tokeniser at decode time, so the model can only emit syntactically valid Axis. Where most agent-targeted languages assume free generation then verification, Axis makes syntactic invalidity unreachable at generation time.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">SHAPE</span> <span class="ty">Todo</span>
  id <span class="ty">UUID</span> <span class="ct">PK</span> <span class="ct">AUTO</span>
  title <span class="ty">STRING</span> <span class="num">200</span> <span class="ct">REQUIRED</span>
  completed <span class="ty">BOOL</span> <span class="ct">DEFAULT</span> <span class="kw">false</span>

<span class="kw">SOURCE</span> todos <span class="kw">POSTGRES</span>
  <span class="kw">SHAPE</span> <span class="ty">Todo</span>
  <span class="ct">INDEX</span> completed

<span class="kw">REALM</span> api
  <span class="ct">CAPABILITY</span> read todos
  <span class="ct">CAPABILITY</span> write todos

<span class="kw">FLOW</span> get_todo <span class="kw">get</span> /todos/:id
  <span class="kw">REALM</span> api
  <span class="kw">LET</span> todo
    <span class="kw">FETCH</span> todos
      <span class="ct">FILTER</span> id <span class="op">EQ</span> path.id
    <span class="ct">OR</span> <span class="num">404</span>
  <span class="kw">RETURN</span> <span class="num">200</span> todo</pre>
  </div>
  <p class="caption">Indentation is significant (2 spaces, tabs illegal). Operators are prefix tokens: <code>EQ</code>, not <code>==</code>. The <code>OR 404</code> clause makes the missing-row case a parse-level concern rather than a runtime <code>if</code>.</p>
</div>

## Distinctive moves.

- 1B/3B/7B target named in spec. Design principles: "context locality: a single endpoint must be fully generatable within 512 Axis tokens (~2K LLM tokens)"; "failure is syntax: unhandled errors, missing auth, unindexed queries are parse or compile errors, not runtime surprises."
- Twelve constructs cover a production backend: `SHAPE` (data model), `SOURCE` (database binding), `REALM` (permission scope and tenancy), `FLOW` (HTTP endpoint), `SAGA` (distributed transaction with compensating steps), `SURFACE` (API routing and versioning), `POLICY` (compile-time invariants across flows), `SERVICE` (external integration), `MIGRATE` (schema migration), `STREAM` (WebSocket/SSE), `FUNC` (pure function), `STORAGE` (file backend).
- Logit masks as JSON: masks in `src/editor/constrain.rs`; CLI plumbing in `src/main.rs` writes `{vocabulary, masks}` JSON.
- Two execution modes, one source. Compile pipeline (`lex → parse → link → verify → plan → codegen`) emits SQL DDL (`--sql`), axum server (`--rust`), TypeScript and Rust client SDKs, OpenAPI 3.0, GraphQL, Kubernetes manifests, migrations, and a generated test suite, each behind its own flag. `axis --serve` runs `lex → parse → verify → serve`, interpreting the AST via axum + sqlx against Postgres, MySQL, or SQLite (auto-detected from `DATABASE_URL`); no codegen on the serve path.
- Prefix operators; every `LET` binds exactly one operation; compound expressions like `a + b * c` forbidden — precedence ambiguity removed at syntax level.
- POLICY: tenant isolation, capability checks, rate limits, auth requirements declared once and verified across every `FLOW` at compile time rather than re-asserted per endpoint.

## Maturity.

Single initial commit 24 May 2026 dropped a working Rust workspace: compiler front-end (`lexer`, `parser`, `link`, `verify`, `plan`), codegen for SQL, Rust/axum, WASM stubs, client SDKs; generators for OpenAPI, TypeScript, GraphQL, Kubernetes manifests, migrations, test suites; stdio LSP; VS Code extension; 73 KB spec (`docs/axis-spec.md` v0.2). Cargo: `version = "0.1.0"`, Rust 1.85 (edition 2024). Three working projects: `todolist/` (CRUD proof, 20/20 integration tests), `advanced/` (auth, tenants, guards, rules, funcs, MATCH, TRY/RECOVER, rate limiting, caching, surfaces — 32/32 tests), `helperbook/` (client-provider marketplace; Postgres, Redis, Meilisearch, Prometheus, Grafana via Docker Compose; "zero application source code" outside `.axis` files). 3 stars, 0 forks, 0 open issues at cataloguing; no GitHub releases. No `LICENSE` file; GitHub API licence field null; `Cargo.toml` declares `license = "MIT"` — open until the author resolves it. Bet (spec framing "the negation of human-centric programming"): a domain surface tight enough for a 1B model plus grammar-aware decoding outperforms a general-purpose language plus a frontier model. No prior catalogue entry shipped grammar-aware logit masks; next milestone to watch: integration with a downstream decoder.

## Agent tooling.

`CLAUDE.md` (7.5 KB) targets agents writing the Axis compiler, not Axis authors: build commands, module-by-module repo map, canonical AST field names to head off guessing ("`TypeExpr` — not `FieldType`, not `ScalarType`"), both pipelines stated. No `AGENTS.md`, `SKILL.md`, MCP server, or `llms.txt`. Authoring surface = the constrained-decoding pipeline: `--constrain`, `--logit-masks`, `--completions`, `--lsp`. Compile-mode outputs (`--plan`, `--openapi`, `--testgen`, …) emit structured JSON for downstream agents.

### Design DNA

- Mog (Syntactic) — closest sibling on the 'small constrained surface' bet: Mog fits its full spec in 3,200 tokens for a general-purpose embedded language (spec direction); Axis fits a production backend in twelve constructs (grammar direction) and adds the masks. Same wager: bounding the surface beats scaling the model.
- NERD (Syntactic) — different lever: NERD strips operators to English keywords betting BPE tokenisers prefer words; Axis strips a backend stack to twelve construct keywords and forbids compound expressions (one operation per LET).
- X07 (Syntactic) — both treat the AST as the executable artefact: X07 stores canonical JSON ASTs edited via RFC 6902 JSON Patch; Axis keeps a textual `.axis` surface but `--serve` runs the parsed AST via axum + sqlx, no codegen.
- Codong (Syntactic) — same diagnosis (choice paralysis), different canonicality scope: Codong ships one canonical function per task across a nine-module general-purpose stdlib; Axis one canonical construct per backend concern — auth is REALM, distributed transactions SAGA, cross-cutting rules POLICY.

*https://agentlanguages.dev/languages/axis/ · https://agentlanguages.dev/languages/axis.md*

## B-IR

Camp: Syntactic | Author: Jason Hall (Chainguard) | Impl: Python (bootstrap) | Target: Arm64 assembly (Mach-O via clang) | Licence: Unknown | First seen: January 2026 | Maturity: thought experiment | Site: https://articles.imjasonh.com/llm-programming-language.md | Repo: https://github.com/imjasonh/loom

### Key idea

Written narrative of three iterations at an LLM-optimised language. Gemini produced B-IR with multi-byte unicode opcodes, too cumbersome for the model to bootstrap. Claude Opus replaced it with TBIR using single-byte control characters in the 0x80-0x8B range, then on its own initiative decided the unreadable characters were in the way and substituted short English keywords (init, fetch, emit, print, loop, exit). The final iteration, Loom, keeps token density but adds unambiguous scope, mandatory pre/postconditions, and stable error codes the model is expected to look up rather than re-read in prose.

## What it is.

Not a project: a blog post by Jason Hall, principal engineer at Chainguard, published on his personal articles site 11 January 2026 — a Sunday prompted by the Oxide and Friends "Predictions for 2026" episode prediction that 2026 would be the year LLMs got a programming language not intelligible to humans. Hall asked first Gemini, then Claude Opus, to design such a language and candidly recorded what each produced. Artefacts — manual.md, an l1-compiler.tbir just under 700 lines, the loom.md specification — live in the companion repo `imjasonh/loom` (originally published as `imjasonh/b-ir`; that URL still redirects).

## Why it's here.

Marker of a meta-question: what happens when a working engineer asks two frontier models to design for their own consumption and reports honestly. Hall's observation: the third iteration resembles existing languages with cleaner error codes and unambiguous scope — read by the catalogue as the design space converging on concerns the verification camp arrived at independently. Not rated against working compilers; marked as a candid record of what the model gravitates toward given a blank page.

### Design DNA

- Sever (Syntactic) — both catalogue-meta companions: what falls out when a frontier model designs a language for itself; both authors keep the result at arm's length from production claims.
- Laze (Syntactic) — both small single-author, one-model weekend explorations; Laze ships a compiler, B-IR an article.
- Vera (Verification) — Loom's conclusions (unambiguous scope, mandatory pre/postconditions, stable error codes) converge on the diagnosis Vera reached independently in the verification camp.

*https://agentlanguages.dev/languages/b-ir/ · https://agentlanguages.dev/languages/b-ir.md*

## Codong

Camp: Syntactic | Author: Brett (brettinhere) | Impl: Go | Target: native binary via Go IR + `go build` | Licence: MIT | First seen: March 2026 | Maturity: working compiler | Site: https://codong.org | Repo: https://github.com/brettinhere/Codong | Codong Arena: https://codong.org/arena/
Agent tooling: SPEC_FOR_AI.md (system-prompt injection — Markdown spec with paired CORRECT/WRONG examples for every rule); structured JSON errors with `fix` and `retry` repair fields; compact error format (project-reported ~39% token reduction).

### Key idea

"Designed for AI to write, humans to review, and machines to execute" (README): collapse choice paralysis to one canonical function per task; compile through Go.

## The thesis.

Diagnosis: choice paralysis. Python has five ways to make an HTTP request; JavaScript has four state-management libraries; every choice costs tokens and produces unpredictable output. Move: collapse language and stdlib to one canonical form per task — `http.get(url)`, `web.serve(port: N)`, `db.connect(url)`, `json.parse(s)`. Nine modules bundled, zero external dependencies, no package manager. Pullquote: "Codong has exactly one way to do everything." Which choice gets eliminated is the distinctive part: NERD strips operators to English keywords, Magpie makes SSA the user-facing surface; Codong leaves operators and surface alone and collapses the standard library — one HTTP function, one JSON parser, one error shape. Compilation: `.cod` → lexer, parser, AST, Go IR, then `go build` static native binary — essentially a frontend for Go's toolchain, as TypeScript is for JavaScript or Kotlin for JVM bytecode.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">fn</span> load_config(path) {
    content = fs.read(path)<span class="op">?</span>
    config = json.parse(content)<span class="op">?</span>
    host = config.get(<span class="str">"host"</span>, <span class="str">"localhost"</span>)
    port = config.get(<span class="str">"port"</span>, <span class="num">8080</span>)
    <span class="kw">return</span> {host: host, port: port}
}

<span class="kw">try</span> {
    config = load_config(<span class="str">"config.json"</span>)<span class="op">?</span>
    print(<span class="str">"Server: <span class="sl">{config.host}</span>:<span class="sl">{config.port}</span>"</span>)
} <span class="kw">catch</span> err {
    print(<span class="str">"Failed: <span class="sl">{err.code}</span> - <span class="sl">{err.fix}</span>"</span>)
}</pre>
  </div>
  <p class="caption">Two bundled stdlib calls, <code>?</code> on three of them propagating structured errors up the stack, and the <code>err.fix</code> field doing repair-loop work in the catch branch.</p>
</div>

## Distinctive moves.

- One canonical function per task: `http.get(url)` is the only HTTP request, `db.connect(url)` the only database open; the bundled stdlib *is* the ecosystem.
- Nine bundled modules, zero external dependencies: `web`, `db`, `http`, `llm`, `fs`, `json`, `env`, `time`, `error`. No package manager or version-resolution tax. (README headline counts eight; the table lists nine — `error` ships alongside the rest.)
- Structured JSON errors: stable code, message, human-readable fix suggestion, retry boolean — agents match on code, apply the fix, decide retry without parsing prose.
- `?` error propagation: unary postfix at highest precedence alongside `()` and `.`; `content = fs.read(path)?` binds the value or returns the error up the stack; no nested `if err != nil` chains.
- Three execution modes from one source: `codong eval` (AST interpreter, sub-second startup), `codong run` (Go IR → `go run`, 0.3–2s startup), `codong build` (Go IR → static native binary).
- Self-reported token savings: Arena benchmark (codong.org/arena) reports a 955-token Codong solution vs 1,867 Python, 1,710 JavaScript, 4,367 Java on a Posts-CRUD task with Claude Sonnet 4 — single workload measured by the project itself, not an independent study.

## Maturity.

Working compiler, MIT, v0.1.3 (28 March 2026), four tagged releases since v0.1.0 first shipped 24 March 2026. 92 commits, 67 stars, 7 forks. 1,427 tests across three suites (1,425 passing, 2 skipped for unconfigured MySQL/PostgreSQL environments). Go is 95.9% of source; binaries for Linux and macOS on amd64 and arm64; no Windows binary yet. v0.1.3 added a compilation cache, project-reported "~170× speedup" on repeat runs. Single author (Brett, `brettinhere`); contributors list also shows a `claude` bot account, consistent with the "AI to write, humans to review" framing.

## Agent tooling.

`SPEC_FOR_AI.md` at repo root: ~6,000 words, ~1,600 lines, 20+ sections, paired `// CORRECT` and `// WRONG` examples for every rule, designed for paste-in to any LLM system prompt. Structured JSON errors with `fix`/`retry` handle the repair loop. `set_format("compact")` produces single-line errors (`err_code:E_MATH|src:divide|fix:check divisor|retry:false`) for token-constrained contexts — project-reported ~39% token reduction in error output. MCP server for Claude Desktop: Stage 7 in the v0.1.3 Roadmap — planned, not yet shipped.

### Design DNA

- NERD (Syntactic) — same diagnosis (choice paralysis burns tokens), opposite lever: operators-to-English-keywords vs stdlib collapse with conventional operators. Both self-report token-savings benchmarks from single-author runs.
- Zero (Verification) — cross-camp foil sharing the 'one X way' slogan: Zero buys obviousness with capability-typed effects, `raises` markers, and a typed `zero fix --plan --json` API inside a verification project; Codong with a single canonical stdlib inside a pure syntactic project. Industrial-backing contrast: Vercel Labs vs single author.
- Magpie (Syntactic) — opposite mechanism: Magpie surfaces SSA (every value `%`-prefixed, ~2.3× more tokens per operation) so the model has nowhere to be wrong; Codong keeps a conventional surface with one canonical function per task so the model never has to choose. Magpie pays in tokens for unambiguity; Codong pays in stdlib scope.

*https://agentlanguages.dev/languages/codong/ · https://agentlanguages.dev/languages/codong.md*

## Laze

Camp: Syntactic | Author: kerv | Impl: Python (bootstrap) | Target: C (via gcc/clang) | Licence: Unknown | First seen: April 2026 | Maturity: early implementation | Site/Repo: https://github.com/kerv/laze

### Key idea

Indentation-based language with infix operators and no punctuation. Compiler is a single Python script (laze/lazec.py): parses `.laze` files to an AST, generates C in memory without writing it to disk, pipes the result to a C compiler. Bet: LLMs are most accurate emitting text-shaped, readable input; ergonomic syntax for the model outranks expressive power or efficiency at the language layer.

## What it is.

Weekend experiment, not a production tool. README opens, verbatim: "This was just an experiment in which I asked Claude Opus 4.7 to create a programming language in the most efficient way it could." Generated C is piped to `cc -O2` for a native macOS binary. Repo (linked from the author's LinkedIn handle millerkev): four commits, two example files, one demonstration — `nes.laze`, a 2,000-plus-line NES emulator covering a 6502 CPU, PPU sprites and scrolling, an APU, and mappers 0, 1, and 4. Author reports Super Mario Bros. fully playable and Legend of Zelda playable with minor glitches.

## Why it's here.

Marks a syntactic-camp position: optimise the surface for what an LLM finds easiest to produce, not what a compiler can analyse. README thesis: an LLM specialises in text-shaped input because that is its training, so the right target is whichever syntax it generates most correctly and fastest. Not rated against compilers shipped by larger teams; a single contributor's snapshot of what falls out when an LLM designs its own language and a human supplies only the prompt.

### Design DNA

- Magpie (Syntactic) — opposite end of the same axis: Magpie chooses an explicit SSA surface to remove ambiguity; Laze strips punctuation and indentation rules to maximise the model's generation speed.
- B-IR (Syntactic) — both small individual weekend explorations of letting an LLM design its own language and recording the result.

*https://agentlanguages.dev/languages/laze/ · https://agentlanguages.dev/languages/laze.md*

## LLMLang

Camp: Syntactic (also spans Verification) | Author: Paul Williams (paulprogrammer) | Impl: Rust | Target: LLVM IR (then native via clang); OpenCL JIT for GPU map kernels at runtime | Licence: GPL-3.0 with Runtime Exception | First seen: May 2026 | Maturity: working compiler | Site/Repo: https://github.com/paulprogrammer/llmlang
Agent tooling: MCP server (llm-mcp binary, stdio transport) — tools analyze_codebase, search_symbols, get_definition, get_diagnostics, find_callers, structural_search, patch_symbol; resources llm://spec (LLM_SPEC.md), llm://agent-workflow (MCP_GUIDE.md). GEMINI.md (Gemini CLI orientation). Stable diagnostic codes E000-E018, W001 catalogued in DIAGNOSTICS.md. `.llmi` signature files for cross-module imports.

### Key idea

Token-density extreme: prefix-arity AST in single-character ASCII operators, De Bruijn variables, linear/affine ownership enforced at compile time, compiler-injected OpenTelemetry via metadata marker, OpenCL JIT translating pure `map` bodies to GPU kernels at runtime with CPU-vectorisation fallback if OpenCL is absent.

## The thesis.

`LLM_SPEC.md` header: `[TOKEN_OPTIMIZED: HIGH_DENSITY]`; design guide: "Target Audience: Large Language Models (LLMs). Non-Goal: Human readability." Source is a prefix-arity AST in single-character ASCII: `+ 10 20` is addition, `> ^0` consumes the most-recent binding, `$ ^1` borrows the next-most-recent, `? cond t f` branches, `# Point x y` declares a struct-of-arrays shape, `: name args body` defines a function, `. e1 e2` sequences. No parentheses, no semicolons, no infix precedence. Variables are De Bruijn indices (`^0`, `^1`, `^2`); the parser accepts named identifiers but resolves them to indices before the AST stores anything.

Two levers at once. Density: where NERD bets on English keywords because BPE tokenisers fragment punctuation, LLMLang bets the opposite — single ASCII characters cost one token each in the right tokeniser, and the win is biggest with no punctuation to fragment. Enforcement: affine ownership (`>` move, `$` borrow, `~` mut-borrow) verified at compile time in `src/compiler/analysis/verify.rs` via a `VariableState` stack — `E004` use-after-move, `E005` double-move, `E009` branch-state mismatch, `E016` moving a borrowed variable. A Rust-style borrow checker on a syntactic-camp surface is why the entry spans into verification: safety enforced, not advisory.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="cm">// Factorial. ^0 refers to the most-recent binding (the parameter n).</span>
<span class="kw">:</span> fact n <span class="op">?</span> <span class="sl">^0</span> <span class="op">*</span> <span class="op">$</span> <span class="sl">^0</span> <span class="kw">@</span> fact <span class="op">-</span> <span class="op">></span> <span class="sl">^0</span> <span class="num">1</span> <span class="op">></span> <span class="sl">^0</span>

<span class="cm">// Auto-instrumented function. The M marker triggers compiler-injected</span>
<span class="cm">// span entry/exit and timing around handle_request.</span>
<span class="kw">M</span> <span class="str">"otel"</span> <span class="str">"handle_request"</span> <span class="kw">:</span> handle_request req
    <span class="op">+</span> <span class="op">$</span> req <span class="num">1</span></pre>
  </div>
  <p class="caption">Every form is prefix-arity; <code>^0</code> is De Bruijn for "most-recent binding"; <code>&gt;</code> consumes, <code>$</code> borrows. The <code>M</code> metadata marker is read by the compiler in <code>src/main.rs</code> and routes the following definition through a code path that wraps the body in <code>llm_otel_enter_span</code> / <code>llm_get_time_ns</code> / <code>llm_otel_emit_span</code> / <code>llm_otel_exit_span</code> calls.</p>
</div>

## Distinctive moves.

- Prefix-arity single-character ASCII: `+ - * / > $ ~ ? . # : ^ ( ) & | =` cover binary math, ownership, branching, sequencing, shape declarations, definitions, De Bruijn lookup, and read/write handles; short keywords (`sl`, `sc`, `ss`, `sf`, `sr`, `sp`, `jp`, `ju`, `map`, `flt`, `oe`) handle the rest. Compound expressions parse without parentheses because arity is fixed per operator.
- De Bruijn references: AST stores only `Expr::DeBruijn(usize)`; sample programs in `examples/` use the bare-index form directly (`: fact n ? ^0 * $ ^0 @ fact - > ^0 1 > ^0`).
- Compile-time affine checking: `verify.rs` walks every function body with a `VariableState` stack tracking `Available`, `Borrowed`, `Moved` per binding; E004/E005/E009/E016 as above; unconsumed bindings emit `W001` and auto-drop at scope exit.
- Compiler-injected OpenTelemetry: `M "otel" "span_name" : func` is recognised in `src/main.rs` and routed to a `gen_function` overload wrapping the body in `llm_otel_enter_span`, `llm_get_time_ns`, `llm_otel_emit_span`, `llm_otel_exit_span`. Nested tagged functions propagate trace context via thread-local storage in the C runtime; `OTEL_EXPORTER_OTLP_ENDPOINT` toggles stdout JSON lines vs HTTP POST. No other catalogue entry ships compiler-injected OpenTelemetry.
- OpenCL JIT for `map` over SoA columns: a `map` over a struct-of-arrays column with a pure function triggers `translate_to_opencl` in `src/compiler/codegen/expr.rs`, synthesising an OpenCL `__kernel void map_kernel(...)`; `src/runtime/driver_src/opencl_driver.c` `dlopen`s `libOpenCL.so` at runtime to compile and dispatch. Absent OpenCL: LLVM loop and SLP vectorisers, plus implicit parallelism hoisting pure subtrees above a complexity threshold into `parallel_task_N` functions on a work-stealing thread pool (`llm_fork`).
- `.llmi` signature files: compiling a module with `-o` generates a high-density header of exported symbols and shape definitions; the `I` operator reads it for downstream type and arity resolution. Same "production backend primitives in the language" framing: the `Money` primitive (`%+`, `%-`, `%*`, `%/` over 4-decimal fixed-point integers, `%str` formatting) and Kubernetes Service Bindings in `src/runtime/db.c` (reads `SERVICE_BINDING_ROOT` to assemble database connection strings from projected files).

## Maturity.

`v0.4.0` at cataloguing; sixteen tagged releases (`v0.1.0` to `v0.4.0`) cut 18–24 May 2026 against a repo created 18 May 2026 — one feature wave per day for roughly a week, then consolidation commits through 27 May. ~13,300 lines of Rust and C across 46 source files: `src/compiler/{lexer,parser,ast,analysis,codegen}` plus a C runtime covering HTTP client and server with `picohttpparser`, TLS via `mbedtls`, `cJSON`, SQLite/Redis/MongoDB drivers, OpenCL dispatcher, MPSC emission queue, and a libtai-baseline temporal module; 31 self-hosted test programs under `tests/lang/`, 47 Rust unit tests in `tests/compiler_tests.rs`. GPLv3 with the `llmlang` Runtime Exception — a GCC-style carve-out keeping the compiler copyleft while generated binaries may link the runtime libraries into proprietary code without the licence propagating. Single author Paul Williams (`paulprogrammer`, Denver, Colorado, GitHub bio "Barefoot Coders"); 0 stars, 0 forks at cataloguing.

README discloses: "This entire repository has been largely vibecoded with humans acting as the product owners, and the LLM acting as the developer" — same factual family as AILANG's "written autonomously by AI agents" framing and Codong's "designed for AI to write, humans to review" position; what is shipped is real engineering with real automated tests, authorship noted as context rather than judgement. `MAYBE.md` separates roadmap from shipped: first-class AST manipulation beyond the existing `patch_symbol`, formal intent-and-contract metadata nodes, and TDD/BDD scenario nodes are not yet in the compiler; OpenTelemetry already crossed off. Bet: the syntactic camp's bet intensified — a surface compressed to single-character prefix operators with indexed variables, plus an MCP server exposing the same AST the compiler sees, produces more correct output per token than a conventional language plus a smarter model.
## Agent tooling.

`llm-mcp` is the primary agent surface, a second cargo target alongside the compiler; seven stdio tools:
- `analyze_codebase`: walks a directory, parses every `.llm` file into the compiler's own AST
- `search_symbols`: functions/shapes by name
- `get_definition`: realised AST + file location
- `get_diagnostics`: parser front-end, `E00x`/`W00x` codes
- `find_callers`: call-graph traversal
- `structural_search`: SHA-256 of a body's operator-and-control-flow shape (literals/names omitted), returns functions sharing the fingerprint — "what else does the same thing?" without name similarity
- `patch_symbol`: takes a JSON AST for a new function body, swaps the matching `Define` node's body, rewrites via the compiler's own pretty-printer (`PrettyExpr` in `src/compiler/ast/display.rs`) — edits syntactically valid by construction

MCP resources: `llm://spec` embeds `LLM_SPEC.md` (token-density grammar reference); `llm://agent-workflow` embeds `MCP_GUIDE.md` (analyse→locate→extract→patch). Stable codes `E000`–`E018`, `W001` catalogued in `DIAGNOSTICS.md`; same identifiers in compiler output, MCP responses, and `llm://spec` text.

### Design DNA

- **NERD** *(Syntactic)* — Closest sibling on token efficiency, opposite lever: English keywords (`plus`, `minus`, `eq`) betting BPE tokenisers fragment punctuation, vs LLMLang's single ASCII characters (`+`, `>`, `$`, `~`) betting the right tokeniser maps each symbol to one token.
- **Magpie** *(Syntactic)* — Same camp, more extreme densification: SSA with `%`-prefixed typed values, ~2.3× more tokens/operation for unambiguity, vs LLMLang's prefix-arity, single-character operators, indexed variables. Both ship structured diagnostics with stable codes.
- **Vera** *(Verification)* — Cross-camp foil on De Bruijn indices: Vera's typed slot references `@T.n` are a verification move (LLMs make naming errors faster than logic errors); LLMLang's `^0`, `^1` a syntactic move (names cost tokens). Same mechanism, different camp.
- **Lumen** *(Orchestration)* — Also ships MCP, differently positioned: `lumen-provider-mcp` is one provider crate among several (HTTP, Gemini, custom-model) inside a human-facing orchestration language; `llm-mcp` is the primary agent surface with structural-fingerprint search and `patch_symbol`.

*Detail page: https://agentlanguages.dev/languages/llmlang/  ·  Markdown companion: https://agentlanguages.dev/languages/llmlang.md*

## Lume

> AI-first backend language, immutable by default, with one canonical way to express each operation. Ships a built-in token-budgeted retrieval tool (lume kb) that packs local language docs, examples, and structured diagnostics under a caller-set token cap.

**Camp:** Syntactic
**Author:** Marcelo Augusto Vilas Boas
**Implementation language:** Go
**Compilation target:** Native binary via Go transpilation + `go build`
**Licence:** MIT
**First seen:** May 2026
**Maturity:** early implementation
**Site:** https://github.com/mavboas/lume
**Repo:** https://github.com/mavboas/lume
**Agent tooling:**
- lume kb build / pack / lint / stats (token-budgeted local context packer)
- Structured semantic diagnostics with stable codes and fix_hint metadata (catalogued in JSON)
- VS Code syntax highlighting and snippets for .lm files

### Key idea

Standard syntactic moves (immutable by default, inferred types, errors as values, one canonical way); the distinctive move is the toolchain: `lume kb` builds a local Markdown knowledge base from the project's own docs, examples, and diagnostic catalog; `lume kb pack "<question>" --ai --max-tokens N` assembles a query-scoped, token-budgeted context pack to paste straight into an LLM prompt.

## The thesis.

Diagnosis: ambient choice in a conventional backend language wastes tokens and yields unpredictable LLM output. README: "Immutable by default, designed for concise LLM-generated code, and currently implemented as an experimental compiler that transpiles `.lm` files to Go before invoking `go build`." Shipped subset: type annotations optional/inferred; functions return their final expression; `switch`/`match` give literal dispatch and pattern matching with exhaustiveness checks (rebinding/`.with()` semantics per caption below). `docs/design.md`: "Tokens are a real cost. Syntax should be concise without becoming cryptic. Immutability is the default. Errors should become values, not hidden control flow. The language should prefer one canonical way to express common backend tasks."

Pullquote: "The goal is to avoid sending the same language reference, examples, diagnostics, and compiler notes to an LLM on every interaction." Contrast: Codong's `SPEC_FOR_AI.md` = whole-spec injection; Mog's `docs/context.md` = hand-curated compact reference; Lume = query-scoped extractor with caller-set ceiling, knowledge base as build output rather than maintained prose.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">cl</span> <span class="ty">Account</span>
{
    id: <span class="ty">str</span>
    balance: <span class="ty">int</span>
}

<span class="kw">fn</span> debit(acc: <span class="ty">Account</span>, amount: <span class="ty">int</span>) -&gt; <span class="ty">Account</span>
<span class="str">"Returns a new Account with balance reduced by amount."</span>
{
    acc.with(balance<span class="op">=</span> acc.balance <span class="op">-</span> amount)
}

<span class="kw">fn</span> main()
{
    acc <span class="op">=</span> <span class="ty">Account</span>(id<span class="op">=</span> <span class="str">"acc-1"</span>, balance<span class="op">=</span> <span class="num">1000</span>)
    acc2 <span class="op">=</span> debit(acc, <span class="num">300</span>)
    print(<span class="str">"After debit: <span class="sl">${acc2.balance}</span>"</span>)
}</pre>
  </div>
  <p class="caption">Classes declared with <code>cl</code>, named constructors, optional doc strings on function signatures, and <code>.with()</code> deriving a new value instead of mutating a field. <code>=</code> introduces a binding on first use; a second <code>=</code> on the same name in the same scope rebinds rather than mutates.</p>
</div>

## Distinctive moves.

- **`lume kb pack` as the agent-facing surface.** `lume kb pack "implement pipe" --ai --max-tokens 1200` tokenises the query, scores pages by path-match and body-match weight, prints header (`# Lume AI Context Pack`, `query:`, `budget:`, listed concepts, examples, error codes, source refs) then highest-scoring chunked page bodies; `internal/kb/kb.go` (~830 lines) tracks the running token estimate, stopping before the budget is breached.
- **Diagnostics as a structured catalog.** `internal/sema/diagnostics.go`: 37 semantic diagnostics, each `{code, feature, message, fix_hint}` (`E2805` = "match expression is not exhaustive", `fix_hint: "add case(_) or cover true/false for bool"`); `pack` can pull error context by code.
- **Immutable bindings, no field mutation.** Class/object fields cannot be reassigned in the current subset; reference: "Same-scope rebinding as a new value, not mutation."
- **Go transpilation as the v0 strategy.** `internal/codegen/golang.go` emits Go; `internal/driver/driver.go` invokes `go build`; `lume gen <file.lm>` prints generated Go. `docs/design.md`, deliberately interim: "This keeps the runtime, scheduler, garbage collector, and native binary story simple while the language surface is still changing."
- **Sequential `let` expressions.** `let(base = price * qty, fee = 2){ base + fee }` — ordered local bindings in a scoped block usable as the function's final expression; one construct for same-scope and nested binding.

## Maturity.

`v0.1.0-experimental`. Seven commits, all 16 May 2026, dropping the whole project: `cmd/lume/main.go`; lexer, parser, AST, semantic checker, Go codegen, driver; `internal/kb/kb.go`; 10 examples; four docs (language reference, compiler architecture, design notes, roadmap); `vscode/` extension scaffold; CHANGELOG/CONTRIBUTING/CODE_OF_CONDUCT/SECURITY/LICENSE. ~6,000 lines Go; 69 test functions across `internal/parser`, `internal/sema`, `internal/driver`, `internal/kb`. MIT. Sole author's bio: "Tech Lead | Itaú Unibanco | 3x AWS | 1x Azure | MBA." 1 star, 0 forks, no tagged releases.

"Planned Language Ideas" (not in compiler): Hindley-Milner inference, ADTs + union pattern matching, `Result`/`Option` with `?` propagation, pipe operator, modules, lambdas, effect annotations, refinement types, spec blocks, backend stdlib. GitHub description claims syntax "strict enough for the compiler to prove correctness"; what ships is conventional soundness — name resolution, type compatibility, `match` exhaustiveness, branch-type agreement on `if`/`switch`, list-element homogeneity, class-field validation — not refinement types or contract discharge. Verdict: syntactic camp, no verification span.

Name collision: another "Lume" was announced as a manifesto 25 May 2026 by David Brown (LinkedIn `dbrown01`, ex-TechnologyOne principal architect) — no public code, GitHub, or company entity. Mavboas's predates it by nine days and ships code; catalogue convention (first to ship code under a name) makes "Lume" = mavboas/lume, pending disambiguation if Brown's ships.

## Agent tooling.

`lume kb build` reads `docs/language.md`, `docs/compiler.md`, every `.lm` example, and the diagnostic catalog; writes one wikilinked (`[[...]]`) page per concept (`kb/language/let.md`), example (`kb/examples/let.lm.md`), and error code (`kb/errors/E2805.md`), plus `kb/index.md`. `lume kb lint` flags broken wikilinks and undocumented examples; `lume kb stats` reports raw vs packed token estimates. No `SKILL.md`, `AGENTS.md`, `CLAUDE.md`, `llms.txt`, or MCP server at root — `lume kb` is the equivalent surface; the diagnostic catalog is the repair-loop substrate.

### Design DNA

- **Codong** *(Syntactic)* — Closest sibling on the 'one canonical way' bet: nine-module general-purpose stdlib, one canonical function per task, same Go-transpile path; Lume applies the diagnosis to a smaller backend surface.
- **Axis** *(Syntactic)* — Same wave (May 2026), same backend-DSL framing, opposite lever end: Axis bounds the surface to twelve top-level constructs with an LL(1) grammar plus per-state logit masks so 1B/3B/7B models can't emit invalid syntax; Lume keeps a conventional curly-brace surface and bounds the context instead.
- **NERD** *(Syntactic)* — Same camp, different lever: English keywords betting BPE prefers words, vs conventional operators betting the bigger win is in what context the model receives.
- **Mog** *(Syntactic)* — Same context-budget concern, opposite directions: Mog designs the language under budget (3,200-token spec); Lume ships a larger surface plus call-time extraction of a query-relevant subset (generated `kb/` tree rebuilt from sources).

*Detail page: https://agentlanguages.dev/languages/lume/  ·  Markdown companion: https://agentlanguages.dev/languages/lume.md*

## Magpie

> SSA as the surface syntax. Every value %-prefixed and typed at definition; one canonical way to express each operation; compiles to native via LLVM.

**Camp:** Syntactic
**Author:** Magpie Language Developers
**Implementation language:** Rust
**Compilation target:** LLVM IR / native, also WebAssembly
**Licence:** MIT
**First seen:** April 2026
**Maturity:** early implementation
**Site:** https://magpie-lang.com
**Repo:** https://github.com/magpie-lang/magpie
**Agent tooling:**
- SKILL.md
- AGENTS.md

### Key idea

The textual program is already in SSA form. Bet: removing surface ambiguity reduces LLM error rates more than added verification does.

## The thesis.

The syntactic camp's premise at its logical end: don't add verification, remove ambiguity. Site: "Magpie eliminates ambiguity so LLMs can write perfect code on the first try." The text is identical to the compiler's IR: each value typed inline, assigned exactly once; basic blocks explicit (`bb0:`). Premise: the hidden semantics of conventional syntax — operator overloading, implicit conversions, invisible lifetime rules — are exactly where LLMs hallucinate. Published trade: "~2.3× more tokens per operation, but eliminates the hidden rules that cause AI retries and borrow checker failures."

Vera contrast: verification atop conventional syntax (mandatory contracts, Z3 discharge, the `<Inference>` effect) — "let the compiler catch what the model gets wrong" — vs Magpie stripping the surface — "don't give the model anywhere to be wrong in the first place."

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">module</span> demo.main
<span class="kw">exports</span> { <span class="sl">@main</span> }
<span class="kw">imports</span> { }
<span class="kw">digest</span> <span class="str">"0000000000000000"</span>

<span class="kw">fn</span> <span class="sl">@add_two</span>(a: <span class="ty">i64</span>, b: <span class="ty">i64</span>) -&gt; <span class="ty">i64</span> {
<span class="sl">bb0</span>:
  <span class="sl">%sum</span>: <span class="ty">i64</span> = i.add { lhs=<span class="sl">%a</span>, rhs=<span class="sl">%b</span> }
  <span class="kw">ret</span> <span class="sl">%sum</span>
}</pre>
  </div>
  <p class="caption">Every value is %-prefixed, typed at definition, and computed by a named operation with named operands.</p>
</div>

## Distinctive moves.

- **SSA as the surface.** The parser doesn't construct an IR — the source is the IR.
- **Operations as records.** `i.add { lhs=%a, rhs=%b }` not `a + b`; overflow behaviour, type coercion, operand order explicit.
- **Explicit ownership operations.** `borrow.shared`, `mutborrow`, `share` are statements, not inferences; the borrow checker has nowhere to hide.
- **One way per concept.** Branching is `cbr` and `br`, full stop. Vocabulary-complexity ratio 0.107 vs Rust 0.225, TypeScript 0.231.
- **Token cost made explicit.** ~2.3× more tokens per operation vs fewer retry loops and borrow-checker failures.

## Maturity.

Rust workspace at v0.1: 44 commits, 3 stars, 1 fork, no releases, MIT, footer "© 2026 Magpie Language Developers." Crates: lexer, parser, semantic analysis, type checking, ownership checking, MPIR lowering with verifier, ARC insertion, LLVM-text and WASM codegen. Site benchmarks: compile 155 ms vs Rust 234 ms / TypeScript 268 ms; execution matches Rust at 32 ms; peak memory 1.6 MB vs Rust 1.4 MB / TypeScript 69.2 MB. Stable diagnostic codes (`magpie explain MPT2014`), JSON output (`--output json`/`jsonl`). Stdlib small; no LSP, registry, or IDE plug-ins yet. Bet: small machine-shaped surface + structured diagnostics beats conventional surface + verification for first-try generation.

## Agent tooling.

`SKILL.md` (coding-and-diagnostic guide written for agents) and `AGENTS.md` alongside `DOCUMENTATION.md` and `DOCUMENTATION_QUICKSTART.md`. CLI: `magpie mcp serve`, `magpie memory build`/`query`, `magpie ctx pack`. Token-efficiency claims in `BENCHMARK.md`.

### Design DNA

- **Vera** *(Verification)* — Cross-camp foil: verification layer atop conventional syntax vs stripping the surface; mechanical checks vs no place to be wrong.
- **NERD** *(Syntactic)* — Same camp, opposite tactic: minimal English-like tokens vs machine-style explicit SSA. Both bet one canonical shape beats many.
- **X07** *(Syntactic)* — Adjacent move: canonical JSON x07AST + JSON Patch quickfixes vs textual SSA. Different bets on the canonical form.

*Detail page: https://agentlanguages.dev/languages/magpie/  ·  Markdown companion: https://agentlanguages.dev/languages/magpie.md*

## Mog

> Statically typed embedded language with flat operators (no precedence) and host-granted capabilities. Full spec fits in 3,200 tokens. Compiles to native via a safe-Rust port of QBE.

**Camp:** Syntactic
**Also spans:** Verification
**Author:** Voltropy
**Implementation language:** Rust
**Compilation target:** Native (via rqbe, an in-process safe-Rust port of QBE)
**Licence:** MIT
**First seen:** March 2026
**Maturity:** working compiler
**Site:** https://moglang.org
**Repo:** https://github.com/voltropy/mog
**Agent tooling:**
- docs/context.md ("LLM Context — Compact reference designed to fit in an LLM's context window")
- lang_spec.md
- showcase.mog (755-line single file demonstrating every language feature)
- .mogdecl capability declarations (host-side typing)

### Key idea

Statically typed, embedded-only, explicitly designed for LLMs to write.

## The thesis.

Two camp moves intersect: syntactically, ambiguity is the enemy; at the verification level, ambient authority is. A `requires http, log;` line at the top declares what the host must provide; everything else is unreachable. Site framing: "statically typed Lua, designed to be written by LLMs."

Compilation is in-process — no JIT, interpreter overhead, or process startup cost; the frontend compiles `.mog` via rqbe to a `.dylib`/`.so` the host can `dlopen`. The first version of Mog was authored by Voltropy's Volt coding agent in a single three-week continuous session, using Claude Opus 4.6, Kimi k2.5, and GLM-4.7, with Voltropy's lossless context management preserving working memory across compactions — the same agent-authored cluster as AILANG.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">import</span> agent;
<span class="kw">optional</span> log;

<span class="cm">// post-compaction hook: re-inject key context</span>
<span class="kw">pub</span> <span class="kw">fn</span> on_post_compaction(session: agent.<span class="ty">Session</span>) {
  log.info(<span class="str">"post-compaction hook: injecting reminder"</span>);

  session.messages.push(agent.<span class="ty">Message</span> {
    role: agent.<span class="ty">Role</span>.SYSTEM,
    content: <span class="str">"IMPORTANT: Always run tests before committing."</span>,
  });
}</pre>
  </div>
  <p class="caption">A Mog script declaring an <code>agent</code> import and an <code>optional</code> log capability. The host decides whether to provide <code>log</code>; the script runs either way. The <code>agent</code> namespace is host-provided typed data.</p>
</div>

## Distinctive moves.

- **Flat operators, no precedence.** Mixing different operators requires parentheses (no precedence table to memorise); `a + b * c` is a compile error; non-associative operators (`-`, `/`, `%`, comparisons) cannot chain even with themselves — `a - b - c` rejected, `(a - b) - c` fine.
- **Capability-based I/O.** `requires fs, http, log;` (or `optional` for graceful degradation); host registers what it provides via `.mogdecl`; runtime refuses calls to anything unregistered. Authority is the host's to grant, not the script's to assume.
- **Embedded only.** README: "Not standalone. Mog is always embedded in a host application. There is no standard library for file I/O or networking — the host provides everything." The orthogonality is the point.
- **Spec fits in 3,200 tokens.** A full spec deliberately bounded by token budget rather than feature count.
- **rqbe.** Quentin Carbonneaux's QBE backend (2016, ~10% the code of advanced compilers for ~70% the performance) ported to safe Rust, ~15,000 lines; shells out only to the system assembler and linker.
- **Agent-authored at origin** (per above: Volt, one three-week session, lossless context management).

## Maturity.

128 commits on main, no tagged release, 1,146+ compiler tests plus 186 rqbe tests passing. 17-chapter site guide: basics through embedding APIs, capabilities, tensors. Security candidly unaudited: "Mog has not been audited, and it is presented without security guarantees. It should be possible to secure it, but that work has not yet been done." Zero public stars at cataloguing — like Boruna's initial state, understating the shipped surface: working compiler, safe-Rust QBE port, 17-chapter spec, capability system, async/await via LLVM-style coroutine lowering.

## Agent tooling.

`docs/context.md` is the headline surface; `lang_spec.md` and the 755-line `showcase.mog` accompany it. No SKILL.md, AGENTS.md, CLAUDE.md, MCP server, or llms.txt — the bet is that a spec small enough to fit in the model's context is the right level of agent tooling. Ships less than Vera or Boruna and gets away with it because the language itself is small.

### Design DNA

- **AILANG** *(Verification)* — Closest relative on capabilities: AILANG carves IO/FS/Net/Clock/AI as row-polymorphic effects in the type system; Mog grants per-capability at the host via .mogdecl. Both make authority explicit; both agent-authored at origin.
- **Zero** *(Verification)* — Sister project on the 'small, one canonical way' diagnosis: Zero pairs it with verification machinery and a structured-JSON CLI; Mog with capabilities and an in-process QBE backend. Both compile to native.
- **NanoLang** *(Verification)* — Cross-camp cousin in the syntactic+verification spanning region: Coq proofs and mandatory tests vs capabilities and a 3,200-token spec. Different bets on what to make load-bearing in a small language.

*Detail page: https://agentlanguages.dev/languages/mog/  ·  Markdown companion: https://agentlanguages.dev/languages/mog.md*

## NERD

> No Effort Required, Done. Replaces every operator with an English keyword on the bet that LLM tokenisers spend fewer tokens on words than on symbols. Built-in MCP client and LLM call primitives.

**Camp:** Syntactic
**Author:** Guru Sattanathan
**Implementation language:** C
**Compilation target:** LLVM IR (then native via clang)
**Licence:** Apache-2.0
**First seen:** January 2026
**Maturity:** working compiler
**Site:** https://www.nerd-lang.org
**Repo:** https://github.com/Nerd-Lang/nerd-lang-core
**Agent tooling:**
- llms.txt
- First-class MCP client primitives (mcp tools / mcp use / mcp resources / mcp read / mcp prompts / mcp init / mcp log)
- First-class LLM call primitive (llm claude "prompt")

### Key idea

Every symbolic operator becomes an English keyword (full list under moves): the claim is BPE tokenisers fragment punctuation but treat common English words as single tokens, so the same logic is cheaper to generate. No type system, braces, or semicolons; functions and side-effects (`http get`, `mcp use`, `llm claude`) are first-class statements.

## The thesis.

Token economics. Site: "40% of code is LLM-written. That number is growing." Control flow ends with `done` rather than a brace. README: "Machines write it. Machines read it. Humans observe it." Pullquote: "The irony: cryptic symbols don't save tokens. Plain words win."

Distinctive is what NERD does *not* ship: no type system, error union, contracts, or checker beyond the parser — the syntactic camp at its purest; smoothing the generation surface buys more than verification would, with the difference in the inference bill. NERD picks the lower-effort lever, accepting "audit" rather than "verify" as the only safety net.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre>fn fizzbuzz n
repeat n times as i
  if i mod 15 eq zero out "FizzBuzz" else if i mod three eq zero out "Fizz" else if i mod five eq zero out "Buzz" else out i
done

fn main
call fizzbuzz 15</pre>
  </div>
  <p class="caption">Every operator is a word: <code>mod</code>, <code>eq</code>, <code>repeat</code>, <code>done</code>. No braces, no semicolons, no <code>+</code>/<code>==</code>.</p>
</div>

## Distinctive moves.

- **Operators as keywords.** `plus minus times over mod`, `eq ne gt lt ge le`, `inc x`, `dec x`, `neg x`. No `+`, `==`, `++`, or `!`.
- **Agent primitives in the grammar.** `llm claude "prompt"`, `mcp tools "url"`, `mcp use "url" "tool" "args"`, `http get "url" auth bearer "token"` — MCP and HTTP are statements, not library calls.
- **`llms.txt` published at the project root.** Teaching corpus on the site for one-fetch syntax ingestion.
- **Self-reported token savings.** Author's four-function math benchmark: 32 NERD tokens vs 70 JavaScript (54% saving), 96 TypeScript (67%), 273 Java (80%) — single workload, single tokeniser, not an independent study.
- **C bootstrap to LLVM IR.** Lexer/parser in C, codegen to LLVM IR, `clang` to native; no runtime; releases are single binaries for macOS Apple Silicon and static Linux x86_64.

## Maturity.

Working compiler, Apache-2.0, 135 stars, two contributors, 30 commits, five tagged releases (latest v0.1.4, Jan 2026). README labels itself "🚧 Early days" and warns the implementation might change completely. Native binaries (macOS-arm64, static Linux) checked into the repo alongside source.

## Agent tooling.

`llms.txt` primary, served from the site root for direct ingestion. Capabilities table: fifteen MCP and HTTP operations shipping today, plus single-line `llm claude "..."` auto-loading `ANTHROPIC_API_KEY` from `.env`. README: scaffolding to experiment with, not a production agent stack — OAuth 2.1 and SSE streaming "coming next."

### Design DNA

- **Magpie** *(Syntactic)* — Same diagnosis (strip ambiguity at the surface), opposite mechanism: SSA with %-prefixed names and one canonical operation per concept, vs stripped operators and a larger surface for shorter tokens.
- **Zero** *(Verification)* — Cross-camp foil: Zero also leans on keywords and 'one obvious way' but pairs them with a type checker and structured-JSON CLI; NERD ships neither.
- **X07** *(Syntactic)* — Same camp, most extreme contrast: X07 walks past textual syntax to JSON ASTs; NERD keeps the text and economises the tokens inside it.

*Detail page: https://agentlanguages.dev/languages/nerd/  ·  Markdown companion: https://agentlanguages.dev/languages/nerd.md*

## Sever

> Single-character opcodes for extreme density. The author's README disclaims the entire repository as Claude-generated and explicitly frames the project as a thought experiment or art piece.

**Camp:** Syntactic
**Author:** Avital Tamir
**Implementation language:** Zig (Claude-generated)
**Compilation target:** Native (via Zig backend, claimed)
**Licence:** Unknown
**First seen:** February 2026
**Maturity:** thought experiment
**Site:** https://github.com/AvitalTamir/sever
**Repo:** https://github.com/AvitalTamir/sever
**Agent tooling:**
- MCP server exposing 29 tools (claimed) across compilation, AST manipulation, dependency analysis, and probabilistic distributions
- Bidirectional conversion between the dense SEV format and a human-readable SIRS JSON form

### Key idea

Two surface forms front a single AST: dense SEV encodes programs as single-character opcodes (P, D, L, R, C) and type tags (I, F, B, S); SIRS JSON mirrors the same AST in human-readable form. The README claims everything below the author's disclaimer is Claude-generated, including the 29-tool MCP server that integrates the model into the compilation loop.
## What it is.

Sever is not a working compiler project. README opens with a disclaimer from GitHub owner Avital Tamir (software engineer at groundcover; creator of the Cyphernetes query language): everything below it was generated by Claude; the author claims no accuracy for any line of code, design decision, or assertion in the repo, README included. Registers as Zig per GitHub language stats. Artefacts: dense SEV opcode format (single-character opcodes P/D/L/R/C, type tags I/F/B/S); SIRS JSON mirror of the same AST; MCP server reporting 29 tools (compilation, AST manipulation, dependency analysis, probabilistic distributions). Author explicitly declines to vouch it compiles/runs as claimed.

## Why it's here.

Marker of a recurring syntactic-camp move: token-density taken to its conclusion. Conceptual art adjacent to engineering — what a frontier model produces given unlimited resources and a brief to design a language for itself. Not rated against working compilers; a snapshot of the design space with model as author, human as curator.

### Design DNA

- **X07** *(Syntactic)* — Both push density to an extreme: X07 replaces text with JSON ASTs edited via RFC 6902 patches; Sever collapses keywords to single-character opcodes.
- **B-IR** *(Syntactic)* — Catalogue-meta companions: artefacts of "what would an LLM-optimised language be", kept by their authors at arm's length from claims of seriousness.

*Detail page: https://agentlanguages.dev/languages/sever/  ·  Markdown companion: https://agentlanguages.dev/languages/sever.md*

## Tacit

> AST as the source of truth. Canonical byte-exact text, BLAKE3-addressed definitions, DeBruijn indices, typed Hole nodes for malformed code, and explicit effects in function signatures.

**Camp:** Syntactic
**Also spans:** Verification
**Author:** weetster
**Implementation language:** Rust
**Compilation target:** LLVM IR / native (x86_64 Linux)
**Licence:** Apache-2.0 OR MIT
**First seen:** April 2026
**Maturity:** working compiler
**Site:** https://github.com/weetster/tacit
**Repo:** https://github.com/weetster/tacit
**Agent tooling:** AGENTS.md, CLAUDE.md

### Key idea

Surface syntax is a lossy intermediate; the AST is authoritative. Each valid AST has exactly one canonical text serialisation; definitions are content-addressed by the BLAKE3 hash of that form; variable references are DeBruijn indices; parse errors yield typed `Hole` nodes with structured diagnostics, not failures. Effects explicit in signatures.

## The thesis.

Text source forces models to maintain style/names/whitespace the compiler discards. README: "The AST is the source of truth. Tacit does not treat a human-oriented surface syntax as the authoritative program representation." Pullquote: "If human readability is not the primary constraint, a language can optimise for three things at once." Two views of the same tree — dense **authoring view** for token budgets, layered **inspection view** for debugging/review — round-trip losslessly via JSON sidecar (`.tacd`). One serialisation per AST kills formatter debates, makes hashing meaningful. Display names: sidecar only, no semantic weight. Cross-camp (verification-adjacent): Tacit-Lite effect lattice `IO`/`Alloc`/`Mut`/`Div` in signatures; unit boundaries must declare type and effect rows.

## What it looks like.

```tacit
unit Math {
  import inc : Int -> Int
    from blake3:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef;

  private double : Int -> Int =
    lambda x. @add x x;

  export public add_two : Int -> Int =
    lambda x. inc (inc x);
}
```

Imports name exact `blake3:<64-hex>` hashes, not paths/version ranges. Visibility (`public`/`package`/`private`) is part of the artefact. Names `x`, `inc`, `double` are sidecar metadata; canonical form uses DeBruijn indices.

## Distinctive moves.

- **AST authoritative.** `.tac` is the byte-exact canonical projection; comments and free-form formatting are not in the language; names, field order, type/effect hints live in the `.tacd` sidecar.
- **Content-addressed definitions.** Imports resolve to BLAKE3 hashes. Renaming leaves a hash unchanged; changing signature, body, or referenced hashes creates a new identity. Package manifests pin a hash-indexed local cache.
- **Typed `Hole` nodes.** Malformed code becomes a `Hole` with structured diagnostic and type slot so tools operate on partial programs (ADR 0040, landed Phase 2).
- **Explicit effect rows.** Mandatory at unit exports, inferred locally. Tacit-Full (refinement types, capability-based security, handlers) is roadmap, not shipped.
- **Toolchain pin.** `tacit init` writes `tacit-toolchain.toml` pinning toolchain, primer, and bundled stdlib hashes; package-aware commands refuse a mismatched pin with a `toolchain-pin-*` diagnostic.

## Maturity.

Rust workspace, five crates (`tacit-canonical`, `tacit-views`, `tacit-typecheck`, `tacit-codegen`, `tacit-cli`); v0.7.7, 19 May 2026; Apache-2.0/MIT; 237 commits, 3 stars, 2 forks at cataloguing. ~90 ADRs. Phase 6 frozen by [ADR 0089](https://github.com/weetster/tacit/blob/main/decisions/0089-phase-6-frozen.md) on 2026-05-17, closing: modules/units; package manifests with hash-pinned lockfiles; package tests with stable `tacit-test-v1` JSON; fixed-width integers (wrapping/checked/saturating); typed mutable-memory handles; source-level stdlib packages (`tacit.core`, `.bytes`, `.array`, `.text`, `.collections`, `.io`); constrained host-interface ABI with generated C headers and Rust bindings. Rust embedding demo links a Tacit kernel as a static library. Phase 7 next; debugger, diff/blame, IDE, public registry, arbitrary FFI out of scope until a later ADR. LLVM 19 pinned via `inkwell`; release artefacts: Linux x86_64, glibc 2.35 floor.

## Agent tooling.

`AGENTS.md` (1.7 KB): Codex-facing sealed-corpus guardrail + pointer to `CLAUDE.md` (~20 KB), a full dev guide: frozen artefacts, ground rules, file-extension contract (`.tac`/`.tacd`/`.taca`), per-phase delivered surface. `tacit primer` prints the byte-pinned Tacit-Lite primer; `--search`/`--list-sections`/`--section <id>` give selective disclosure sized to a context window. Diagnostics, package tests, `tacit version` emit stable JSON.

### Design DNA

- **Magpie** *(Syntactic)* — Closest neighbour: Magpie surfaces SSA as textual source; Tacit declares text non-authoritative. Both pay token cost to strip ambiguity.
- **X07** *(Syntactic)* — Same "text is lossy" axis: X07 stores canonical JSON ASTs edited via JSON Patch; Tacit stores canonical text projected from the AST, BLAKE3-addressed.
- **Vera** *(Verification)* — Vera abolishes parameter names for typed DeBruijn slots (`@Int.0`); Tacit keeps display names in sidecar, DeBruijn canonical. Both treat names as model-error source.
- **Mog** *(Syntactic)* — Mog: small embedded language, capability system, sub-3,200-token spec; Tacit ships a constrained ABI for a Rust host, defers capabilities to Tacit-Full. Different bets on capabilities in v1.

*Detail page: https://agentlanguages.dev/languages/tacit/  ·  Markdown companion: https://agentlanguages.dev/languages/tacit.md*

## X07

> Eliminates text syntax. Programs are canonical JSON ASTs (x07AST); edits are RFC 6902 JSON Patch operations; diagnostics ship as stable JSON with quickfixes the toolchain applies deterministically.

**Camp:** Syntactic
**Author:** Author unknown
**Implementation language:** Rust
**Compilation target:** Native (via C codegen); WebAssembly
**Licence:** Apache-2.0 OR MIT
**First seen:** April 2026
**Maturity:** working compiler
**Site:** https://x07lang.org
**Repo:** https://github.com/x07lang/x07
**Agent tooling:** AGENT.md; agent portal at /agent (versioned JSON entrypoints: manifest, schemas, skills, examples, stdlib, packages); x07-mcp (build MCP servers in X07); x07lang-mcp bridge (typed toolchain access over MCP); per-release skills pack; stable error codes, quickfixes as JSON Patch.

### Key idea

A program is an x07AST JSON document (`*.x07.json`) with versioned schema; edits are RFC 6902 JSON Patch applied mechanically. Stable error codes as structured JSON, paired with quickfixes applied via `x07 fix --write` or `x07 ast apply-patch`. Side effects live in explicit capability worlds; sandboxing is policy-driven.

## The thesis.

Text source is exactly where agents lose: whitespace load-bearing, identical ASTs serialise differently, patches collide on formatting noise, diagnostics target humans. X07 deletes the text layer. Pullquote: "One canonical approach. No \"should I use a for loop or map?\" decisions." `x07lang.org/agent` publishes versioned JSON entrypoints (thesis listing names `examples/catalog.json`) for direct agent consumption, not HTML scraping. Effects gated by named capability worlds (`run-os`, `run-os-sandboxed`). Closest direction-of-travel: Magpie surfaces SSA at source level; X07 deletes the source level.

## What it looks like.

```
{
  "schema_version": "x07.x07ast@0.4.0",
  "kind": "entry",
  "module_id": "main",
  "imports": ["std.bytes"],
  "decls": [],
  "solve": ["std.bytes.reverse", "input"]
}
```

A quickfix is an array of JSON Patch ops applied to this document; the agent never edits a source string, only the tree.

## Distinctive moves.

- **AST as canonical source.** On-disk artefact is the parsed tree; schemas versioned (`x07.x07ast@0.4.0`).
- **JSON Patch as edit primitive.** Structural diffs apply mechanically; no whitespace exists to break a patch.
- **Capability worlds.** Deterministic solve worlds or named OS worlds (`run-os`, `run-os-sandboxed`); no ambient access by default.
- **Stable codes, deterministic quickfixes.** `x07 lint` emits `x07diag` JSON with stable code + optional JSON Patch fix; `x07 fix --write` applies.
- **Versioned agent portal.** Each release ships `/agent/v<version>/` machine entrypoints alongside human docs.
- **Performance claims, published comparison repo.** README points at `x07lang/x07-perf-compare` (v0.0.3 snapshot): native parity with C/Rust on included workloads, faster compiles; methodology lives in that repo.

## Maturity.

Multi-crate Rust workspace, Apache-2.0/MIT; 471 commits, 108 tagged releases (latest GitHub tag v0.1.49, Mar 2026); docs site is source of truth for later point releases (latest `0.2.10`). Portal advertises 14 skills, 258 schemas, 17 examples, 410 packages, 19 stdlib modules — bigger doc surface than most entries. Community signal lags sharply: 7 stars, 0 forks, 0 open issues at cataloguing; README does not name the author. The gap between toolchain depth and visible user base is the entry's defining quality.

## Agent tooling.

`AGENT.md` at repo root for human orientation; everything else structured. `/agent` portal: per-version stable JSON entrypoints (`entrypoints.json`, `manifest.json`, `schemas/index.json`, `skills/index.json`, `examples/index.json`, `stdlib/index.json`, `packages/index.json`). `x07-mcp` kit authors MCP servers in X07; `x07lang-mcp` bridges agent runtimes to the toolchain. Canonical loop: `x07 init → x07 lint → x07 fix → x07 run → x07 test`, all stages structured JSON.

### Design DNA

- **Magpie** *(Syntactic)* — Magpie surfaces SSA but keeps text; X07 makes the AST canonical. Same direction, further along.
- **Sever** *(Syntactic)* — Both push far from human authoring: Sever crams meaning into single characters; X07 abandons characters as the unit.
- **NERD** *(Syntactic)* — Opposite end of camp: NERD keeps text, economises tokens inside it; X07 edits the tree directly.

*Detail page: https://agentlanguages.dev/languages/x07/  ·  Markdown companion: https://agentlanguages.dev/languages/x07.md*

---

# Verification camp (11)

> The model doesn't need to be right. It needs to be checkable. LLMs will keep making semantic errors; can the compiler catch them? Answer: mandatory contracts, refinement types, effect systems, SMT-backed proofs — formal methods repurposed as guardrail for generative code.

## AILANG

> Row-polymorphic Hindley-Milner with capability-based effects (IO, FS, Net, Clock, AI). No loops, lambda calculus only. Written autonomously by AI agents.

**Camp:** Verification
**Author:** Mark Edmondson / Sunholo
**Implementation language:** Go
**Compilation target:** Native binaries, WebAssembly
**Licence:** Apache-2.0
**First seen:** September 2025
**Maturity:** working compiler
**Site:** https://ailang.sunholo.com
**Repo:** https://github.com/sunholo-data/ailang
**AILANG Benchmark Dashboard:** https://ailang.sunholo.com/docs/benchmarks/performance
**Agent tooling:** SKILL.md, AGENTS.md, CLAUDE.md, MCP server, llms.txt, Claude Code plugin, Gemini CLI extension, slash commands

### Key idea

Purely functional, effect-typed substrate for AI-generated code. Hindley-Milner + row polymorphism; effects carved into capability categories (IO, FS, Net, Clock, AI) granted at the CLI with --caps. No loops: lambda calculus, pattern matching, ADTs are the only control. Compiler written autonomously by AI agents via a coordinator.

## The thesis.

LLMs hallucinate side effects — network calls in pure functions, FS writes in read-looking helpers, unapproved model calls. Every effect is visible row-polymorphically in the signature; an undeclared effect can't be performed; a run not granted a capability at the CLI can't grant it from inside. Pullquote: "For humans, a language is a tool for expression. For AIs, it's a substrate for reasoning." Distinctive move: no `for`, `while`, or mutable accumulator. Vera tracks model calls as one `<Inference>` effect; AILANG splits authority into five categories. Bet: determinism, replay, and structured per-effect traces are worth giving up the loop.

## What it looks like.

```
module examples/hello

import std/io (println)

export func main() -> () ! {IO} {
  println("Hello from AILANG!")
}
```

`! {IO}` after the return type is the effect row; a caller without `IO` granted via `ailang run --caps IO` cannot invoke. Rows compose: calling `IO`- and `FS`-effecting helpers requires declaring `{IO, FS}`.

## Distinctive moves.

- **Capability carving, not tracking.** `IO`, `FS`, `Net`, `Clock`, `AI` each granted/refused separately via `--caps`; the model can't widen authority from inside.
- **No loops.** Lambda calculus, recursion, pattern matching only; dedicated "Why No Loops?" reference page; design axioms treat absence of mutable iteration as load-bearing for replay.
- **Row-polymorphic Hindley-Milner.** Effect rows are first-class type-level objects, inferred/unified like row-typed records; a network-free helper has a smaller row than its caller.
- **Written by agents, end-to-end.** README: "written autonomously by AI agents via its own coordinator"; Sonar, OpenSSF Scorecard, OpenSSF Best Practices badges cited as third-party verification.
- **MCP-first surface.** Hosted MCP server at `mcp.ailang.sunholo.com`: typed tools over stdlib, examples, benchmarks. Claude Code plugin + Gemini CLI extension install compiler, prompt, and MCP server in one command.

## Maturity.

v0.20.1; 110 GitHub releases; Apache-2.0; 2,958 commits, 26 stars. Go (85.5% of source); native binaries for macOS (Intel + Apple Silicon) and Linux; WebAssembly target used by nine in-browser demos. Stdlib: `std/io`, `std/fs`, `std/json`, `std/zip`, `std/xml`, `std/crypto`, `std/http`, `std/net`. Dashboard: 33 tasks × 8 frontier models × 3 modes (zero-shot, self-repair, full agentic) on every release. Bet: others design a language a human reads and an AI writes; AILANG is written and maintained by AI. Next test: whether agent-authored development yields a stdlib competing with MoonBit's ~2-year head start (MoonBit launched 18 August 2023).

## Agent tooling.

`SKILL.md`, `AGENTS.md`, `CLAUDE.md` in repo; `llms.txt` + `llms-full.txt` on docs site. Remote MCP server: typed tools for stdlib lookup, examples, design docs, benchmark dashboard. `ailang_bootstrap` plugin installs slash commands (`/ailang:prompt`, `/ailang:new`, `/ailang:run`, `/ailang:challenge`) into Claude Code; equivalent Gemini CLI extension; both download a platform-matched compiler binary on install. CLI emits structured per-effect traces for the next iteration.

### Design DNA

- **Vera** *(Verification)* — Vera: one &lt;Inference&gt; effect; AILANG: IO, FS, Net, Clock, AI as separate per-run capability categories.
- **Boruna** *(Orchestration)* — Both capability-based effect systems; Boruna enforces at the VM, AILANG at the type system + CLI capability flag.
- **MoonBit** *(Verification)* — Both effect typing on a functional core; MoonBit's conventional/general-purpose, AILANG's row-polymorphic, carved for agent-relevant authority.

### Timeline

- **Sep 2025** — First public GitHub release, Apache-2.0.
- **Jan 2026** — v0.6.2; Mark Edmondson publishes the language framing on dev.to.
- **May 2026** — v0.20.1; 110 releases, 2,958 commits; 33-benchmark dashboard × 8 frontier models per release; MCP server, Claude Code plugin, Gemini CLI extension in production.

*Detail page: https://agentlanguages.dev/languages/ailang/  ·  Markdown companion: https://agentlanguages.dev/languages/ailang.md*

## Aver

> Every function carries intent, declared effects, and a colocated verify block. Pure verify blocks export to Lean 4 theorems and Dafny lemmas; effectful ones lift through the Oracle proof export.

**Camp:** Verification
**Author:** jasisz
**Implementation language:** Rust
**Compilation target:** bytecode VM, Rust, WebAssembly GC + wasip2 (Lean 4 / Dafny via proof export)
**Licence:** MIT
**First seen:** February 2026
**Maturity:** working compiler
**Site:** https://averlang.dev
**Repo:** https://github.com/jasisz/aver
**Agent tooling:** CLAUDE.md, llms.txt

### Key idea

Intent, effects, and verification colocated with the body: prose intent (?), effect declaration (!), verify block. Pure verify blocks export as Lean 4 theorems / Dafny lemmas; effectful ones lift through Oracle, which quantifies over bounded effect parameters in the exported theorem.

## The thesis.

Target audience named: the reviewer, not the generator. Every function carries `? "..."` intent, effect declaration (`! [Console.print]`), and a `verify` block of `expression => expected` cases. Export via `aver proof --backend lean|dafny`; Oracle lifts classified effects (`Random`, `Http`, `Disk`, `Time`, `Console.readLine`, ...) into proof artefacts as explicit function parameters typed with bounded subtypes (`RandomFloatInUnit`); the theorem quantifies over every possible such function, not just the test stub. Architectural choices are syntax: `decision UseResultNotExceptions { chosen = "Result", rejected = ["Exceptions"], ... }`. Pullquote: "Code is cheap to generate. Expensive to trust." Vs Vera: shared DNA — mandatory verification artefacts, explicit effects, no `if`/`else`, no closures, no exceptions, no nulls, no loops — but Vera drops names via De Bruijn slots (`@Int.0`); Aver keeps names, makes metadata mandatory. Vera's bet: names are the failure mode; Aver's: absence of intent is.

## What it looks like.

```
fn safeDivide(a: Int, b: Int) -> Result<Int, String>
    ? "Safe integer division. Returns Err on zero."
    match b
        0 -> Result.Err("Division by zero")
        _ -> Result.Ok(a / b)

verify safeDivide
    safeDivide(10, 2) => Result.Ok(5)
    safeDivide(7, 0)  => Result.Err("Division by zero")
```

Prose intent (?), no if/else, colocated verify block: the function ships its specification.

## Distinctive moves.

- **Mandatory intent.** `?` prose after every signature; effects without description warn.
- **Effects as type signatures.** `! [Http.get]` = specific capability; `! [Http]` = namespace. Violations are type errors; `aver.toml` constrains reachable hosts/paths.
- **Verify, then prove.** One `verify` block runs as sampled cases (`aver verify`), adversarial-profile checks (`aver verify --hostile`), or Lean 4 / Dafny export (`aver proof`). The four readings can disagree on identical source — Oracle page walks one through.
- **Oracle for effectful code.** Effects lifted via `BranchPath` + per-branch counters; theorem quantifies universally.
- **`aver context` for agents.** Token-budgeted export of types, effects, intents (`--budget 10kb`) sized to an LLM window.
- **Decisions as syntax.** `decision` blocks make ADRs queryable from the codebase, not a wiki.

## Maturity.

v0.21 on crates.io (`cargo install aver-lang`), MIT, Rust, primary author `jasisz`. Three backends: bytecode VM, native Rust codegen, standalone WASM-GC (also lowered to wasip2 / WASI 0.2 Component Model for server-side). Site demos seven games on WebAssembly GC — Snake 4.3 KiB, roguelike 25.6 KiB — on Chrome 119+/Firefox 120+/Safari 18.2+. Proof export: Lean 4 (via `lake build`) and Dafny. Wide, functional toolchain. Bet: one source as implementation and reviewable spec, proof export as upper-bound check.

## Agent tooling.

`llms.txt` at averlang.dev/llms.txt: long-form crib sheet — syntax, `=>` separator (vs `=`), constructor qualification (`Result.Ok`, never bare `Ok`), numbered list of common LLM mistakes. `CLAUDE.md` + `.claude/skills/` in repo. `aver context` exports a token-budgeted codebase slice. Diagnostics carry structured hints (`Hint: add ! [Console.print]`); playground renders them live.

### Design DNA

- **Vera** *(Verification)* — Closest relative (shared DNA per thesis). Aver became the first non-Python/TS baseline in VeraBench; bench README: 'a Haskell-inspired language with zero LLM training data.'
- **Prove** *(Verification)* — Same camp, opposite politics: both contracts + explicit effects; Aver ships llms.txt and welcomes AI authoring; Prove ships an anti-training licence prohibiting training use of source.
- **Pact** *(Verification)* — Both per-function intent + effects declarations (Aver's ? and ![Effect]; Pact's intent and needs db). Aver pushes proof export; Pact pushes built-in SQLite and HTTP.

### Timeline

- **Apr 2026** — First external language in VeraBench as baseline alongside Python and TypeScript; bench README: 'a Haskell-inspired language with zero LLM training data, providing a second data point alongside Vera for the zero-training-data thesis.'
- **May 2026** — v0.21 on crates.io. Oracle proof export to Lean 4 + Dafny stabilises. Seven games to WASM-GC (Snake 4.3 KiB, Doom 20.4 KiB, roguelike 25.6 KiB), Chrome 119+/Firefox 120+/Safari 18.2+.

*Detail page: https://agentlanguages.dev/languages/aver/  ·  Markdown companion: https://agentlanguages.dev/languages/aver.md*

## BHC/hx

> Not a new language. The bet is that Haskell's purity and semantic density already make AI-written, verifiable compute feel natural — once toolchain friction is removed. hx wraps cabal/stack/ghcup/HLS in Rust; BHC is a clean-slate Haskell 2026 compiler with per-profile runtimes.

**Camp:** Verification
**Author:** Raffael Schneider
**Implementation language:** Rust
**Compilation target:** GHC/Cabal/HLS (hx, wrapping); LLVM, WebAssembly, GPU (BHC, in development)
**Licence:** MIT (hx); BSD-3-Clause (BHC)
**First seen:** April 2026
**Maturity:** early implementation
**Site:** https://raskell.io
**Repo:** https://github.com/arcanist-sh/hx

### Key idea

One editorial position more than two projects. Schneider's public blog trilogy at raskell.io: Haskell's types-as-proofs and pure-by-default already give LLMs the formal scaffolding; Haskell loses agent benchmarks because of the surrounding toolchain — fragmented build tools, slow compiles, one-size-fits-all runtime. Engineering response under the arcanist.sh org: hx (Rust binary wrapping GHC, Cabal, GHCup, HLS behind one interface) and BHC (in-development Haskell 2026 compiler, multiple runtime profiles).

## The thesis.

The verification camp has the right diagnosis, wrong locus. Declarative + strongly typed fits LLM strengths: generating expressions satisfying formal constraints rather than simulating execution across mutable steps. Type-checked Haskell looks like a proof; the compiler is the proof checker; aligned types eliminate large error classes by construction. The blocker is friction outside the language. Pullquote: "The language was right. The surrounding infrastructure was not." Distinctive move: refusal to design a new language. AILANG, Vera, Aver ship fresh syntax with effect typing; BHC/hx extend Haskell — BHC targets the Haskell 2026 specification with profile-specific runtimes selected at compile time. The typed substrate is correct; the missing layer is operational, not linguistic.

## Distinctive moves.

- **The position, stated.** raskell.io trilogy: "The Last Programming Language Might Not Be for Humans", "What Comes After the Last Programming Language", "Source Code Is the New Assembly" — medium-term winner is declarative-plus-typed, not procedural-plus-checked.
- **hx wraps before it replaces.** Cargo-workspace Rust binary, ~14 crates (`hx-cli`, `hx-core`, `hx-toolchain`, `hx-cabal`, `hx-solver`, `hx-lsp`, `hx-plugins`, …); commands `build`, `run`, `test`, `lock`, `sync`, `watch`, `fmt`, `lint`, `docs`, `publish`; lockfile `hx.lock`. Stated strategy: wrap GHC/Cabal/GHCup/HLS first, replace last.
- **BHC: multiple runtime profiles.** README lists four — default, server, numeric, edge — selected at compile time; essays + arcanist-sh org profile describe a planned six (adding `realtime`, `embedded`). Catalogue treats the four shipped as ground truth, the two extra as stated intent.
- **Conservative scope per release.** Both pre-1.0. arcanist-sh org profile advertises a 5.6× cold-build speedup over Cabal; no methodology, benchmark suite, GHC/Cabal versions, or hardware published anywhere reachable — treat as marketing claim, not verified measurement.
- **No agent-specific surface yet.** No SKILL.md, AGENTS.md, MCP server, or `llms.txt` in either repo; argument: well-typed Haskell already gives an agent what it needs, agent tooling is downstream.

## Maturity.

Early. hx: MIT Rust, v0.6.0 (Feb 2026), 12 tagged releases, 129 commits, 23 stars; orchestrates GHC/Cabal/GHCup/HLS rather than replacing them. BHC: BSD-3-Clause, v0.2.1 (Jan 2026), 389 commits, 3 releases, 11 stars, single contributor; README roadmap: parser, type checker, Core IR, one codegen path substantially complete; WASM/GPU lowerings in progress; no conformance suite or benchmark numbers in repo. Multi-year arc: essays and infrastructure now, language-level claims later.
## Agent tooling.

None shipped. Schneider's position: the right intervention is upstream of agent-specific files — a faster, more coherent build, a compiler whose error messages and runtime profile match the deployment target, and a type system the agent can already use as a proof obligation. The bet hinges on declarative-plus-typed beating procedural-plus-checked once tooling friction is gone, before agent-native languages with built-in MCP surfaces lock in a different shape.

### Design DNA

- **AILANG** *(Verification)* — Closest design relative. Both bet on purely functional, effect-typed code as the right shape for agents to author; AILANG designed a new language, BHC/hx argues the language is already fine and rebuilds the tooling around Haskell.
- **MoonBit** *(Verification)* — Industrial-backing foil: sampler-level verification, three years of training data, Shenzhen-funded team vs BHC/hx's one-person Swiss effort betting that better tooling around an established language beats a new language with a new ecosystem.
- **Vera** *(Verification)* — Adjacent, not competing. Vera is the bespoke agent-native route; BHC/hx bets Haskell's purity and semantic density make AI-written, verifiable compute natural once toolchain friction is removed. Schneider's essays cite Vera by name as the canonical language-design route, framing BHC/hx as the complementary route through tooling.

*Detail page: https://agentlanguages.dev/languages/bhc-hx/  ·  Markdown companion: https://agentlanguages.dev/languages/bhc-hx.md*

## Intent

**Camp:** Verification
**Author:** lhaig
**Implementation language:** Go
**Compilation target:** Native binaries (via Rust), JavaScript, WebAssembly (direct binary)
**Licence:** Apache-2.0
**First seen:** February 2026
**Maturity:** working compiler
**Site/Repo:** https://github.com/lhaig/intent
**Agent tooling:** AGENTS.md, CLAUDE.md, INTENT.md

### Key idea

Every function carries `requires`/`ensures`; every entity an `invariant`; loops `invariant` and `decreases`. An `intent` block links natural-language goals to specific contracts via `verified_by`; the compiler refuses references with no matching contract. `intentc verify` discharges what it can to Z3; the rest runs as enforced runtime checks in Rust (panic), JavaScript (throw), or WebAssembly (trap), all from the same source file.

## The thesis.

Premise: humans audit contracts, not implementations. Repo framing: "Humans audit contracts, not implementations. When you generate Intent code, the human reads your `requires`, `ensures`, `invariant`, and `intent` blocks to verify correctness." Pullquote: "The contract system is the product. The implementation is secondary."

The intent block pairs a named natural-language goal with `verified_by` references to specific contracts (`BankAccount.invariant`, `BankAccount.deposit.requires`, `BankAccount.withdraw.ensures`); each must resolve to an actual `requires`, `ensures`, or `invariant` clause or compilation fails, so published intent blocks cannot drift from cited contracts. Runtime enforcement of undischarged contracts is identical across all three targets.

## What it looks like.

```intent
entity BankAccount {
    field balance: Int;

    invariant self.balance >= 0;

    method deposit(amount: Int) returns Void
        requires amount > 0
        ensures self.balance == old(self.balance) + amount
    {
        self.balance = self.balance + amount;
    }
}

intent "Safe withdrawal preserves non-negative balance" {
    goal "BankAccount.withdraw never results in balance < 0";
    guarantee "if withdraw returns false then balance is unchanged";
    verified_by BankAccount.invariant;
    verified_by BankAccount.withdraw.requires;
}
```

`old(...)` captures pre-mutation state for `ensures`. `forall i in 0..n: p` and `exists i in 0..n: p` quantify over integer ranges in contracts. Loops carry `invariant` and `decreases` for inductive reasoning at verification time.

## Distinctive moves.

- **Mandatory contracts at three levels.** Functions `requires`/`ensures`; entities `invariant`; loops `invariant`/`decreases`. The grammar reserves the slots; the type checker enforces `verified_by` resolution.
- **Intent blocks as compiled artefacts.** A `verified_by` path (`Entity.member.clause` or `function.clause`) is resolved by the semantic checker, not convention; unresolved references are compile errors, preventing prose-level drift.
- **Z3 as optional static layer, runtime checks as the floor.** `intentc verify` translates IR contracts to SMT-LIB and invokes Z3; per-contract results: `verified` / `unverified` / `error` / `timeout`. Without Z3 the compiler degrades gracefully; runtime asserts still run.
- **Rust as IR, not the only backend.** Explicit IR (~30 node types, contracts as first-class IR nodes, `OldCapture`/`OldRef` as explicit pre-state) feeds three sibling backends: a Rust generator handing off to `cargo`, a direct JavaScript emitter, and a direct WebAssembly binary emitter that needs no Rust toolchain. Each enforces the same contracts at runtime.
- **Property-based test generation from contracts.** `intentc test-gen` derives property-based tests from `requires`/`ensures`, complementing — not replacing — SMT discharge.

## Maturity.

Go workspace at v0.2.0 (released 16 February 2026), 45 commits, 5 stars at cataloguing. `docs/ROADMAP.md` records milestones 1–6 complete: usable surface (loops, arrays, enums, pattern matching, `Result`/`Option`, try operator, multi-file imports), Z3 verifier (`internal/verify/`), three backends (Rust via `internal/rustbe/`, JavaScript via `internal/jsbe/`, direct WASM via `internal/wasmbe/`). `docs/DESIGN.md` spec runs 1,764 lines; traits, generics, async/await, and a package manager with semver constraints landed since the POC. CLI: `intentc build / check / verify / fmt / lint / test-gen`, plus `intentc pkg init / add / remove / install`. A four-target showcase (CLI binary, browser dashboard, Node server, browser WASM at 155 bytes) runs on unmodified compiler output. Last commit 25 March 2026; LSP, REPL, and release-mode contract stripping sit on the milestone-8 roadmap, not yet built.

## Agent tooling.

`AGENTS.md` (~18 KB): Codex/general-agent orientation. `CLAUDE.md`: Claude-specific working guide. `INTENT.md` (~26 KB): language reference written as agent instructions — opens "You are generating code in Intent", ends with ten explicit guidelines for AI code generation including "Write contracts first" and "Every function should have contracts." `docs/REPRODUCE.md` documents reproducing the compiler with the reader's agent of choice. Diagnostics are textual rather than structured JSON; the LSP that would expose them is roadmap-only.

### Design DNA

- **Vera** *(Verification)* — Closest relative: both ship mandatory contracts on every function, both use Z3, both treat the agent as primary author. Vera abolishes parameter names via typed DeBruijn slots and tracks LLM inference as a first-class `<Inference>` effect; Intent keeps names, concentrating novelty in intent blocks binding natural-language goals to verified contract references.
- **Aver** *(Verification)* — Same camp, different proof story: Aver exports `verify` blocks as Lean 4 theorems or Dafny lemmas through `aver proof`, lifting effectful code into proof artefacts via Oracle; Intent commits to Z3 SMT with runtime enforcement on every backend. Same diagnosis, different upper-bound check.
- **MoonBit** *(Verification)* — Closest sibling on compilation strategy: four backends on an OCaml compiler vs Intent's three on a Go one. MoonBit's edge is years of training data; Intent's framing is auditability over breadth.
- **Prove** *(Verification)* — Same contract machinery, opposite politics: Prove ships refinement types and verb-based IO under the Prove Source License v1.0, which prohibits use as AI training data; Intent ships under Apache-2.0 and addresses its agent-instruction documents to the model directly.

*Detail page: https://agentlanguages.dev/languages/intent/  ·  Markdown companion: https://agentlanguages.dev/languages/intent.md*

## MoonBit

**Camp:** Verification
**Author:** Hongbo Zhang / IDEA Shenzhen
**Implementation language:** OCaml
**Compilation target:** WASM GC, JavaScript, native (C codegen), LLVM
**Licence:** Unknown
**First seen:** January 2023
**Maturity:** working compiler
**Site:** https://www.moonbitlang.com
**Agent tooling:** `moon doc` AI symbol lookup; MoonBit Pilot coding agent; `declare` keyword for AI-native specification

### Key idea

AI-friendly general-purpose language with the deepest history in the space — three years of training data, full toolchain across four backends, package registry (mooncakes.io), cloud IDE, IDE plugins. ICSE 2024 paper on a real-time semantics-aware token sampler. Backed by the International Digital Economy Academy (Shenzhen).

## The thesis.

The catalogue's exception that proves the rule: most entries are recent (Jan–May 2026) single-author or small-team experiments; MoonBit has shipped since 2023, backed by IDEA Shenzhen and led by Hongbo Zhang (created ReScript, contributed to OCaml), with four backends, a package registry, a cloud IDE, and two IDE plug-ins. Pullquote: "The model doesn't need to be retrained. The sampler needs to know the type system."

The ICSE 2024 paper describes a real-time semantics-aware token sampler: as the model generates code, a fast type-checker prunes ill-typed continuations at the token level — the model can still hallucinate, but hallucinations never get past the sampler. Closer to the verification camp's "make it checkable" than the syntactic camp's "make it easier to generate", applied at the layer where generation actually happens.

## Distinctive moves.

- **Real-time semantics-aware sampling.** The compiler participates in token generation, not just post-hoc checking.
- **`declare` keyword.** First-class form for AI-native specification of intent and constraints, distinct from regular function signatures.
- **Four backends.** WASM GC, JavaScript, native (via C codegen), LLVM — no other catalogue entry targets this breadth.
- **mooncakes.io.** First-party package registry; most catalogued languages have no ecosystem to need one, MoonBit has the ecosystem.
- **Three years of training data.** The unmatched advantage; every other entry is racing to generate examples.

## Maturity.

Most mature project in the catalogue by a clear margin: 2,115+ stars (second-highest after Zero), full toolchain, multiple backends in active production use, IDE integrations across both major desktop IDEs, working debugger; documentation depth and developer experience no other entry approaches. Open question: whether general-purpose-plus-AI-aware-tooling beats agent-native-plus-narrow-ecosystem as the field matures. The next two years will test it.

## Agent tooling.

`moon doc` symbol lookup, the Pilot agent, and `declare` (above) are less prominent than the SKILL.md/AGENTS.md pattern other entries use — MoonBit's bet is that an agent that knows the language outperforms an agent reading instructions about the language.

### Design DNA

- **Vera** *(Verification)* — Both verification camp, opposite breadth: full-stack general-purpose vs narrowing to checkability and dropping names entirely. MoonBit assumes the model needs help; Vera assumes it needs supervision.
- **Zero** *(Verification)* — Closest in industrial backing (Vercel Labs vs IDEA Shenzhen) and product framing. Zero leans syntactic (one obvious way to express things); MoonBit leans typed sampling at the model level.
- **AILANG** *(Verification)* — Both ship effect typing: MoonBit's conventional, AILANG's row-polymorphic with capability-based carving (IO/FS/Net/Clock/AI). MoonBit's edge is training-data depth no other entry has.

### Timeline

- **2023** — Project initiated at IDEA Shenzhen under Hongbo Zhang, pre-LLM-craze; framing changes over the following two years.
- **Jan 2024** — ICSE 2024 paper on real-time semantics-aware token sampling for MoonBit code generation.
- **2024–2025** — Toolchain hardens: the four backends, mooncakes.io, cloud IDE, VS Code and IntelliJ plugins, debugger.
- **2026** — `declare` keyword and MoonBit Pilot agent ship, repositioning the language explicitly as AI-native rather than just AI-friendly.

*Detail page: https://agentlanguages.dev/languages/moonbit/  ·  Markdown companion: https://agentlanguages.dev/languages/moonbit.md*

## NanoLang

**Camp:** Verification
**Also spans:** Syntactic
**Author:** Jordan Hubbard
**Implementation language:** C
**Compilation target:** C (default), WebAssembly, LLVM IR, PTX, RISC-V
**Licence:** Apache-2.0
**First seen:** September 2025
**Maturity:** working compiler
**Site:** https://jordanhubbard.github.io/nanolang
**Repo:** https://github.com/jordanhubbard/nanolang
**Agent tooling:** AGENTS.md; CLAUDE.md; MEMORY.md ("training reference for patterns and idioms"); spec.json (machine-readable formal specification); .mcp.json; .claude/, .cursor/, .factory/ config folders; VS Code extension with LSP (nanolang-lsp) and DAP (nanolang-dap); web playground with CodeMirror-6, share permalinks, live evaluation

### Key idea

Mandatory shadow test blocks on every function (the compiler refuses to compile without one) plus 6,170 lines of Coq proving the language's core. NanoCore is the proved subset, not the entire surface language. The first-person README persona is documented as deliberate design, not flourish; the README announces: "I refuse to compile a function unless you provide a shadow test block for it."

## The thesis.

Takes the verification diagnosis literally: the language refuses LLM work without tests, and the core has proofs behind its promises. Every function declaration must pair with a `shadow` test block. NanoCore: 6,170 lines of Coq across 9 files, 193 theorems and lemmas, zero axioms, zero `Admitted` — covering preservation, progress, determinism, semantic equivalence of big-step and small-step, and evaluator soundness against a fuel-based reference interpreter extractable to OCaml. Pullquote: "I am a minimal programming language designed for machines to write and humans to read. I require tests, I use unambiguous syntax, and my core is formally proved."

NanoCore covers integers, booleans, strings, arithmetic, conditionals, mutable variables, while loops, lambda/application, arrays, records, recursive functions, variants, and pattern matching; algebraic effects, async/await, FFI, the VM, and multi-target codegen are outside the proof set, and `formal/README.md` says so plainly. Vera proves contracts at the function level via Z3; NanoLang proves the type system itself, from below — same camp, complementary layers.

## What it looks like.

```
fn greet(name: string) -> string {
  return (+ "Hello, " name)
}

shadow greet {
  assert (== (greet "World") "Hello, World")
}

fn main() -> int {
  (println (greet "World"))
  return 0
}

shadow main { assert true }
```

## Distinctive moves.

- **Mandatory shadow tests.** No function compiles without a `shadow` block. The smallest legal program contains `shadow main { assert true }` — the discipline applies to the trivial case alongside the substantive one.
- **Coq proofs, zero axioms.** 193 theorems and lemmas across `Syntax.v`, `Semantics.v`, `Typing.v`, `Soundness.v`, `Progress.v`, `Determinism.v`, `Equivalence.v`, `EvalFn.v`, `Extract.v`; no `Admitted` lemmas. Built on Rocq Prover (Coq) ≥ 9.0.
- **NanoISA VM with co-process FFI.** Stack-based VM, 178 opcodes, reference-counted GC. External calls run in a separate co-process (`nano_cop`); if they crash, the VM survives. Trap model separates computation from I/O.
- **Multi-target codegen.** Default C transpilation; also `--target wasm` (source-map sidecar, Ed25519 signing), `llvm`, `ptx`, `riscv`. Production parity claimed for C and WebAssembly; the other backends are present in the toolchain.
- **Dual notation, prefix and infix.** `(+ a b)` and `a + b` both legal; the prefix form is described as unambiguous and is what the formal semantics is stated against.
- **First-person persona.** README, diagnostics, and `AGENTS.md` instruct agents to write in NanoLang's voice; documented under `docs/PERSONA.md` as a design choice, not a quirk.

## Maturity.

v3.3.7 (April 2026), 51 tagged releases, ~2,156 commits, bootstrap 100% (the compiler compiles itself). Apache-2.0. Hardware: Ubuntu 22.04+ on x86_64 and ARM64, macOS 14+ Apple Silicon, FreeBSD; Windows via WSL2 only. Author Jordan Hubbard — co-founder of FreeBSD in 1993, currently Senior Director for GPU Compute Software at Nvidia; the project's GitHub topics include `thought-exercise` and `vibe-coding`, applied by the author himself. The README's "Totally True and Not At All Embellished History" notes the language has "been used in production by exactly one person, who also wrote it." The engineering is real; the framing is honest about what it's for.

## Agent tooling.

Repo root ships everything in the metadata list above (`MEMORY.md` self-describes as "my training reference for patterns and idioms"); the Language Server and Debug Adapter live at `bin/nanolang-lsp` and `bin/nanolang-dap`. Wider agent-facing surface than most entries — NanoLang ships its own corpus, not just orientation files.

### Design DNA

- **Vera** *(Verification)* — Same camp, different layer of the same idea: Vera proves function-level contracts via Z3, NanoLang proves the core type system from below via Coq. Vera's `<Inference>` effect has no NanoLang analogue.
- **Aver** *(Verification)* — Same-camp neighbour on the ship-the-verification-artefacts axis: Aver exports per-function proofs to Lean 4 and Dafny; NanoLang ships its Coq proofs alongside the source.
- **Magpie** *(Syntactic)* — Cross-camp foil: Magpie strips ambiguity via SSA-as-surface; NanoLang via prefix-call disambiguation, mandatory annotations, and one canonical form. Different mechanisms, same diagnosis.
- **NERD** *(Syntactic)* — Syntactic-camp direction without the formalism: token-friendly surface, no type system, vs NanoLang's Coq proofs. The contrast clarifies what 'unambiguous syntax' costs to back up.

*Detail page: https://agentlanguages.dev/languages/nanolang/  ·  Markdown companion: https://agentlanguages.dev/languages/nanolang.md*

## Pact

**Camp:** Verification
**Author:** Viktor Kikot
**Implementation language:** Rust
**Compilation target:** Interpreted (tree-walking)
**Licence:** MIT
**First seen:** April 2026
**Maturity:** working compiler
**Site/Repo:** https://github.com/KikotVit/pact-lang
**Agent tooling:** CLAUDE.md; .mcp.json; built-in MCP server (pact mcp) with 5 tools; LSP server (pact lsp); built-in docs (pact docs)

### Key idea

Every function and route opens with an `intent` clause and a `needs` list declaring effects. Errors are part of the type signature (`-> User or NotFound`), data flows through left-to-right pipelines, and a single ~5MB binary ships the whole runtime. Bet: surfacing intent, effects, and outcomes at the signature level lets agents skip the reverse-engineering pass.

## The thesis.

Diagnosis: most backend code is glue, and glue is exactly where agents waste iterations — intent hides in comments that drift, effects in implementation bodies, errors in exception hierarchies. The verification-camp move is to drag all three back into the signature; the compiler reads them as the contract, and the type checker, formatter, LSP, and MCP server all consume the same declarations. Pullquote: "Every function says why it exists. Errors are data, not explosions."

The distinctive breadth ships in one binary: a `.pact` file declares `app Notes { port: 8080, db: "sqlite://notes.db" }` and `pact run` brings up an HTTP server with SSE streaming, SQLite in WAL mode, JWT auth, a structured logger, and a built-in MCP server — no dependencies, no ORM, tables auto-created from struct fields. Close to Aver in design DNA (declared intent + declared effects + colocated checks), but where Aver lifts verify blocks into Lean 4 and Dafny, Pact spends its complexity budget on the runtime an agent will actually drive.

## What it looks like.

```
intent "create a new user with default Viewer role"
fn create_user(data: NewUser) -> User or BadRequest
  needs db, time, rng
{
  let existing: User = find_by_email(data.email)
  return BadRequest { message: "Email already taken" } if existing != nothing

  let user: User = {
    id: rng.uuid(),
    email: data.email,
    role: "Viewer",
    active: true,
    created_at: time.now(),
  }

  db.insert("users", user)
}
```

## Distinctive moves.

- **Intent in the signature.** Every `fn` and `route` carries a one-line `intent` string read before the body — the author argues this lets a model skip reverse-engineering purpose from implementation.
- **Effects in the signature.** `needs db, time, rng, auth, log, env, http` declares side effects up front; tests swap them deterministically: `using time = time.fixed(...)`, `using db = db.memory()`.
- **Errors as types.** Sum types replace exceptions; `| on NotFound: respond 404 with ...` handles each variant; Rust-style `?` propagates.
- **Pipelines as the default control flow.** `data | filter where .x > 0 | sort by .name | take first 10` is the canonical shape for data transforms and route handlers alike.
- **One binary, one runtime.** ~5MB Rust binary bundles lexer, parser, tree-walking interpreter, HTTP server, SSE, SQLite, JWT, HTTP client, LSP, MCP server, formatter, and docs.

## Maturity.

Single-author, MIT, currently at v0.5 with six tagged releases (latest v0.3.1, Apr 2026), 204 commits and 496+ tests on master. The README is explicit it works for small APIs and CRUD services and is not production-ready; the web playground is the next planned milestone. Stars and forks at zero, which understates the shipped surface: deep type checker, formatter, LSP, MCP server, VS Code extension, macOS/Linux install script, and a Docker image are all in the tree today.

## Agent tooling.

`CLAUDE.md` and a checked-in `.mcp.json` orient Claude Code at the project level. `pact mcp` exposes five tools over stdio JSON-RPC: `pact_run`, `pact_check`, `pact_docs`, `pact_format`, `pact_test`. `pact lsp` provides diagnostics, hover, and autocomplete for any LSP-capable editor. `pact docs <topic>` makes documentation CLI-queryable so an agent can pull a topic and a working example before generating code.

### Design DNA

- **Aver** *(Verification)* — Closest relative: both attach declared intent and effects to every function; Aver lifts the verify block into Lean 4 and Dafny exports, Pact keeps the surface lighter and ships a working web stack.
- **Vera** *(Verification)* — Vera's requires/ensures clauses are the strict cousin of Pact's intent blocks: Vera mechanically discharges via Z3; Pact treats intent as documentation the type checker and MCP server consume.
- **Boruna** *(Orchestration)* — Another single-author Rust project whose engineering depth runs well ahead of its public profile; both ship MCP servers as the agent-facing surface.

*Detail page: https://agentlanguages.dev/languages/pact/  ·  Markdown companion: https://agentlanguages.dev/languages/pact.md*

## Prove

**Camp:** Verification
**Author:** Magnus Knutas
**Implementation language:** Python (bootstrap)
**Compilation target:** C (then native via gcc/clang)
**Licence:** Prove Source License v1.0 (language & .prv source) / Apache-2.0 (tooling)
**First seen:** February 2026
**Maturity:** working compiler
**Site:** https://prove.botwork.se

### Key idea

Verbs encode intent and IO category in the function declaration. The compiler enforces verb semantics, refinement-type constraints, and ensures/requires/explain contracts. The Prove Source License v1.0 covers all .prv source and prohibits AI training use.

## The thesis.

Same verification-camp diagnosis — AI-generated code is cheap to produce, expensive to trust — and the same general moves: intent-first declarations, hard postconditions (`ensures`), refinement types, no `if`/`else`, errors-as-values, no nulls. Every function carries a verb: pure (`transforms`, `validates`, `derives`, `creates`, `matches`), IO (`inputs`, `outputs`, `dispatches`, `streams`), structured concurrency (`attached`, `detached`, `listens`, `renders`). Verbs are enforced — a `transforms` function cannot call IO functions (diagnostics E361–E363); `explain` blocks document the chain of operations in controlled natural language. Pullquote: "Source code is covered by an anti-training licence."

Divergence from Vera and Aver is politics, not mechanics: Vera publishes a benchmark and invites models to compete; Aver exports proofs and ships an `llms.txt`; Prove ships the anti-training Prove Source License v1.0 — applied automatically by `proof new` to every project — prohibiting use of `.prv` source as training data, dataset inclusion, vector stores, RAG indices, embedding databases, synthetic data generation, sublicensing for AI use, and downstream propagation. AILANG sits closest on effect typing, but Prove encodes effect category in the verb itself rather than a row-polymorphic effect list. The project frames its stance as "AI resistance" and states that generating semantically correct Prove code "requires genuine understanding, not pattern matching."

## What it looks like.

```
matches apply_discount(discount Discount, amount Price) Price
  ensures result >= 0
  ensures result <= amount
  requires amount >= 0
from
    FlatOff(off) => max(0, amount - off)
    PercentOff(rate) => amount * (1 - rate)
```

A pure verb (matches), hard postconditions, and a precondition — all enforced at compile time.

## Distinctive moves.

- **Verbs as intent, enforced.** Each function declares its purpose with a verb; pure verbs cannot perform IO; the same name can have multiple verbs, resolved from context by the compiler.
- **Anti-training licence.** The Prove Source License v1.0 covers the language, its specification, and all `.prv` source; the compiler tooling (Python bootstrap, docs, editor integrations) is separately Apache-2.0, with the reasoning published under "AI Transparency."
- **Refinement types.** `type Port is Integer:[16 Unsigned] where 1..65535` — constraints are part of the type, validated at compile time, and used to drive auto-generated edge cases.
- **`explain` against contracts.** The compiler parses each controlled-natural-language row, checks operations match called functions' behaviours, and rejects explanations referencing identifiers that aren't real.
- **Refutation challenges.** `proof check` runs by default and generates plausible-but-wrong mutations of the function body, requiring the author to address each with a `why_not` annotation.
- **Functional iteration only.** `map`, `filter`, `reduce` — no loops. Errors propagate with `!`.

## Maturity.

v1.3.1 (April 2026), with a clear release history (see Timeline). Source hosted on a self-hosted Gitea instance at code.botwork.se rather than GitHub. Author: Magnus Knutas. Bootstrap compiler Python 3.11+; output language C. Roadmap names v2.0 as a self-hosted compiler. The bet: the same intent-first mechanism that resists external AI generation is also the substrate for the project's "local, self-contained" generation model — a deterministic toolchain producing code from the project's own declarations.

## Agent tooling.

None of the catalogue's usual surface: no `SKILL.md`, no `AGENTS.md`, no `llms.txt`, no MCP server — the licence actively prohibits the dominant tooling pattern. Ships editor integrations instead: `tree-sitter-prove` (syntax highlighting), `pygments-prove` (MkDocs/Sphinx), `chroma-lexer-prove` (Gitea/Hugo), plus a single-file installer (`curl -sSf install.sh | sh`) placing a `proof` binary in `~/.local/bin/`. Roadmap lists binary AST format, semantic normalisation, fragmented source, and identity-bound compilation as post-1.0 anti-training features.

### Design DNA

- **Vera** *(Verification)* — Same diagnosis, opposite politics: both ship mandatory contracts and explicit effects; Vera publishes a benchmark and invites models to compete, Prove ships an anti-training licence prohibiting training use of source.
- **Aver** *(Verification)* — Camp neighbour with proof export: both intent-first with contract-style verification; Aver exports to Lean 4 / Dafny and ships llms.txt, Prove ships the anti-training licence and self-hosted Gitea.
- **AILANG** *(Verification)* — Both effect-typed: AILANG carves effects via row polymorphism (IO/FS/Net/Clock/AI); Prove encodes IO category in the verb itself.

### Timeline

- **Feb 2026** — v1.0.0 first stable: 22-module standard library, intent-driven compiler (verb enforcement, contracts, refinement types), C code generation with region-based memory and a 13-pass optimiser, ML-powered LSP.
- **Mar 2026** — v1.1.0: structured concurrency (`attached`, `detached`, `listens`, `renders` backed by stackful coroutines), terminal UI, GUI via SDL2 + Nuklear, the `proof` CLI wrapper.
- **Mar 2026** — v1.2.0: verb semantic guarantees enforced across 22 stdlib modules (~105 corrections); recursive variant types and Value<T> phantom types land.
- **Apr 2026** — v1.3.0: tree-sitter becomes sole parser, `reads` renamed to `derives`, `dispatches` verb added, linting integrated into the check pipeline. v1.3.1 is a bugfix release.

*Detail page: https://agentlanguages.dev/languages/prove/  ·  Markdown companion: https://agentlanguages.dev/languages/prove.md*
## Vera

Camp: Verification (also spans Orchestration). Author: Alasdair Allan. Impl: Python. Target: WebAssembly. Licence: MIT. First seen: Feb 2026. Maturity: working compiler.
Site: https://veralang.dev · Repo: https://github.com/aallan/vera · vera-bench: https://github.com/aallan/vera-bench
Agent tooling: SKILL.md, AGENTS.md, CLAUDE.md, LLM-oriented diagnostics, stable error codes (E001–E702), JSON diagnostic output.

### Key idea

Mandatory requires/ensures/effects contracts on every function. Three-tier Z3 SMT verification. Typed De Bruijn slot references (@T.n) instead of variable names — the only language in the space that drops names. LLM inference is a first-class typed effect. Thesis: "The model doesn't need to be right. It needs to be checkable."

## The thesis.

Takes the verification camp's diagnosis literally: if LLMs make semantic errors faster than humans can catch by reading code, the compiler must do the catching. Empirical literature shows models particularly vulnerable to naming errors — misleading names, incorrect reuse, losing track of which name refers to which value — so Vera removes names from the language entirely.

## What it looks like.

```
public fn safe_divide(@Int, @Int -> @Int)
  requires(@Int.1 != 0)
  ensures(@Int.result == @Int.0 / @Int.1)
  effects(pure)
{
  @Int.0 / @Int.1
}
```
Division by zero is a type error, not a runtime error: a caller that can't prove the denominator non-zero won't compile. @Int.1 = first parameter (next-most-recent binding); @Int.0 = second (most-recent).

## Distinctive moves.

- Mandatory contracts: requires/ensures/effects on every function; grammar rejects functions without them — no opt-out.
- De Bruijn slot references: `@T.n` = n-th-most-recent binding of type T; no parameter-level variable names.
- Typed effects incl. inference: LLM calls are an `<Inference>` effect; a function not declaring it can't make model calls; effect system tracks model usage up the call graph.
- Three-tier verification before any code runs: a contract discharges at compile time via Z3, becomes a runtime guard, or becomes a runtime check; tier determined by which arithmetic fragment the clause lives in.
- LLM-oriented diagnostics: stable codes E001–E702; every diagnostic carries a fix hint and spec reference; CLI emits JSON.

## Maturity.

v0.0.157+; 300+ stars; 3,400+ tests at 96% coverage; 76 conformance programs; 13-chapter spec. Python reference compiler; compiles to WebAssembly via wasmtime, runs in browser. VeraBench: 93% flagship correctness on zero training data, matching Python. Focus: stdlib breadth. Open: monomorphisation reindexing, GC-rooting around inference calls. Usable, but ecosystem (LSP, package registry, IDE plug-ins) still building.

## Agent tooling.

SKILL.md (complete language reference for agents writing Vera), AGENTS.md (setup for any agent system), CLAUDE.md (Claude Code orientation); all three rendered into llms.txt and llms-full.txt for agent-framework ingestion. JSON diagnostics on request; stable codes referenceable from agent prompts.

### Design DNA

- Aver (Verification) — closest design relative: co-located verify blocks, Lean 4 proof export, decision blocks; different syntax, same diagnosis; now integrated into VeraBench.
- Prove (Verification) — same diagnosis, opposite politics: licence explicitly prohibits AI training use; refinement types, verb-based IO tracking.
- AILANG (Verification) — capability-based effects with row polymorphism; Vera tracks `<Inference>` as one effect, AILANG carves it into IO/FS/Net/Clock/AI.
- Magpie (Syntactic) — cross-camp foil: strips ambiguity at the surface (SSA form) where Vera adds mechanical checks; different bets on the error budget.

### Timeline

- Feb 2026 — first public release (v0.0.4): grammar, parser, type checker; no verifier yet.
- Mar 2026 — Z3 verifier lands; three-tier scheme published; first externally-contributed example merges.
- Apr 2026 — VeraBench published (93% flagship correctness vs Python baseline, zero training data).
- Apr 2026 — `<Inference>` effect added; Aver becomes first external language integrated into VeraBench.
- May 2026 — v0.0.157 releases.

Detail: https://agentlanguages.dev/languages/vera/ · MD: https://agentlanguages.dev/languages/vera.md

## Vow

Camp: Verification. Author: Paulo Matos. Impl: Rust (stage 0); self-hosted in Vow. Target: native via Cranelift; C only for the ESBMC verification pipeline. Licence: MIT. First seen: Feb 2026. Maturity: working compiler.
Site: https://vow-lang.com · Repo: https://github.com/vow-lang/vow
Agent tooling: CLAUDE.md; compiler-bundled Claude Code skill (auto-installed to .claude/skills/vow/); `vowc skill print --bundle` (self-contained markdown for non-Claude harnesses); structured JSON diagnostics with counterexamples and blame.

### Key idea

Every function declares a `vow` block of requires/ensures; loops carry invariant. The compiler lowers these to ESBMC bounded-model-checker obligations before any code ships. Diagnostics emit JSON in parallel with human text, with explicit Caller/Callee blame on every violation. The compiler binary embeds and auto-installs a Claude Code skill generated from the same source/compiler version as the toolchain — "the source of truth for any harness writing Vow code; cannot drift from the toolchain you are running."

## The thesis.

Stated audience is not human readers; homepage: "The syntax is not for you. The semantics is not for you. The language is not for you. Yours is only the product." Vows lower to obligations for the [ESBMC](https://esbmc.org) bounded model checker; counterexamples return as structured records. Intended workflow is CEGIS: write, compile, verify, read counterexamples, fix, iterate. Distinctive move is the checker choice: Vera and Intent dispatch to Z3 SMT; Aver exports Lean 4 theorems or Dafny lemmas; NanoLang proves its core type system in Coq from below. Vow picks ESBMC and accepts the trade — counterexamples are concrete re-runnable inputs, but soundness holds only up to the unwinding bound chosen per verification call. Repo CLAUDE.md on the consequence: "Bounds like `n <= 10` (to fit within `--unwind 10`) or `a <= 100` (to help the SMT solver) are verification artefacts. They do not belong in `requires`/`ensures` clauses... If ESBMC can't prove a correct contract, that's ESBMC's problem."

## What it looks like.

```
module Bisect

fn bisect(lo: i64, hi: i64) -> i64 vow {
  requires: hi >= lo
} {
  let mut lo: i64 = lo;
  let mut hi: i64 = hi;
  while lo + 1 < hi vow {
    invariant: hi - lo >= 0
  } {
    let mid: i64 = lo + (hi - lo) / 2;
    lo = mid;
  }
  lo
}

fn main() -> i32 [io] {
  let r: i64 = bisect(0, 64);
  print_i64(r);
  0
}
```
The `vow` block follows the signature; loop vows carry `invariant` clauses. The `[io]` effect set on `main` is the program's only impurity — pure functions carry no annotation; calling an effectful function from a pure one is a type error.

## Distinctive moves.

- ESBMC over SMT or theorem provers: contracts discharge before codegen; the C emitter exists only to feed that pipeline; native code comes from Cranelift directly.
- Structured blame: `requires` violations fault the Caller; `ensures`/`invariant` violations fault the Callee; JSON record carries the verdict: `{"error":"VowViolation","vow_id":N,"blame":"Caller"|"Callee",...}`.
- Compiler-bundled agent skill: `vowc build` in a project that already has `.claude/` writes the skill to `.claude/skills/vow/` on first run, sourced from the running compiler binary; non-Claude harnesses get the same bundle via `vowc skill print --bundle`.
- Canonical form as a compiler pass: the printer is a stage, not a formatter; parse → print → parse enforced idempotent by tests; one preferred way per construct.
- Linear types and checked arithmetic at the grammar level: `linear struct` values must be consumed exactly once (type checker tracks the obligation); `+!`, `-!` etc. are token kinds distinct from `+`, `-` — checked and wrapping arithmetic are grammar productions, not library functions.
- Deliberately absent: generics, traits, closures, higher-order functions, macros, GC. CLAUDE.md design rule rejects any feature that "introduces a new type-system axis."

## Maturity.

v0.2.0 released 20 May 2026 (repo created 25 Feb 2026), MIT, vow-lang GitHub org. Nine Rust crates (vow-syntax, vow-types, vow-ir, vow-codegen, vow-verify, vow-runtime, vow-diag, vow-clif-shim, vow) feeding a Pizlo-style SSA IR and Cranelift backend. Self-hosted compiler in compiler/ (13 modules) compiles itself byte-identically under the bootstrap triple test: stage 0 → A → B → C, sha256sum of B and C must match. Mutation testing: `vowc mutants` subcommand, tiered oracle, structured JSON output. examples/ ships a deterministic CDCL SAT solver (watched literals, first-UIP learning, non-chronological backtracking, Luby restarts), a UCI-compatible chess engine, and a Lean 4 kernel checker (vow-lean-kernel) targeting the Lean Kernel Arena. Author's announcement names the standing gap: "ESBMC integration is in place and discharges contracts for the example programs, but the corners are still being found... The compiler is written in Vow but its own vows are not all verified end-to-end. Closing that loop is the single most important piece of work ahead." benchmarks/ holds Vow's implementation of vericoding ([arXiv:2509.22908](https://arxiv.org/abs/2509.22908)): 40 problems (15 Easy, 15 Medium, 10 Hard), with a Python harness in bench/ running frontier models against paper baselines (Dafny 82%, Verus/Rust 44%, Lean 27%). Published numbers for Vow itself: roadmap, not released.

## Agent tooling.

CLAUDE.md (~20 KB) addresses language design rules to Claude directly; AGENTS.md is a nine-byte placeholder. The substantive surface is the compiler-emitted skill: `vowc skill install --local` writes SKILL.md plus reference/, examples/, schemas/ to .claude/skills/vow/; `vowc build` auto-installs the same payload on first run where .claude/ exists; `vowc skill print --bundle` emits one self-contained markdown document for raw-API harnesses. All diagnostics flow through vow-diag, always JSON and human text in parallel — "this is by design, not a flag." `vowc --help` returns a structured JSON capability description by default, human text under `--human`.

### Design DNA

- Vera (Verification) — closest relative: both mandate contracts on every function and treat the agent as primary author. Vera: Z3 SMT, drops parameter names via typed De Bruijn slots. Vow: ESBMC, keeps names, adds linear types plus distinct checked/wrapping arithmetic tokens.
- Intent (Verification) — same camp, same era, different checker: Z3 SMT with runtime enforcement on three backends (Rust, JavaScript, WebAssembly) vs Vow's ESBMC with a single Cranelift native backend and verification-only C emitter.
- Aver (Verification) — same diagnosis, different upper-bound check: exports verify blocks as Lean 4 theorems or Dafny lemmas via Oracle, lifting effectful code into proof artefacts; Vow discharges in-process with ESBMC and emits structured counterexamples to fix against.
- NanoLang (Verification) — different proof tool, different layer: 193 Coq theorems with zero axioms proving the core type system from below; Vow ships ESBMC obligations on every function from above. Both pair proof discipline with a self-hosted compiler.

### Timeline

- Feb 2026 — first commit (25 Feb): Rust stage 0 compiler, ESBMC integration, vow block grammar.
- May 2026 — v0.2.0 (20 May): self-hosted compiler reaches byte-identical bootstrap fixed point.
- May 2026 — public announcement; author publishes *What's in a Vow*; SAT solver, chess engine, and Lean 4 kernel checker ship, all written in Vow.

Detail: https://agentlanguages.dev/languages/vow/ · MD: https://agentlanguages.dev/languages/vow.md

## Zero

Camp: Verification (also spans Syntactic). Author: Chris Tate and Matt Van Horn / Vercel Labs. Impl: C (zero-c bootstrap); self-hosted compiler-zero in progress. Target: native binaries (direct ELF/Mach-O/PE emitters, no LLVM), WebAssembly. Licence: Apache-2.0. First seen: May 2026. Maturity: early implementation.
Site: https://zerolang.ai · Repo: https://github.com/vercel-labs/zerolang
Agent tooling: structured JSON diagnostics, stable error codes, typed repair plans, zero skills (version-matched agent guidance), zero explain, zero fix --plan --json, zero doctor.

### Key idea

Vercel Labs' bet on agent-first systems programming: stable error codes (NAM003 means "unknown identifier" and will keep meaning that), typed repair plans an agent can apply without parsing prose, version-matched guidance served through the CLI rather than scraped from a docs site. Intentionally explicit language: capability objects on main, no hidden allocator, no implicit async, one obvious path for most things.

## The thesis.

The bet: the bottleneck in agentic coding is the compiler↔agent channel, not the language. The standard loop is fragile — prose errors written for human engineers, agent parses text, guesses a fix, next compile yields new prose in a slightly different format. Zero replaces the prose channel: `zero check --json` emits stable code (NAM003), human-readable message, line number, and a typed `repair` object; `zero fix --plan --json` returns a machine-readable edit plan; `zero explain` returns structured explanations against the installed compiler version. "Humans read the message. Agents read the JSON." At the language level it collapses the syntactic and verification camps into one product decision: docs prefer "one obvious way to express most things, even when that makes code more explicit than a human might choose" (syntactic framing), bought with capability objects on `main`, explicit `raises` markers, and effect-visible signatures (verification machinery). MoonBit, the catalogue's other industrially backed verification entry, invests in semantic-aware token sampling; Zero invests in a surface small enough an agent needs no help generating it.

## What it looks like.

```
pub fun main(world: World) -> Void raises {
  check world.out.write("hello from zero\n")
}
```
No hidden global process object: `world: World` is an explicit capability passed in by the runtime; `raises` declares the function can propagate errors; `check` handles a fallible operation. A function that doesn't ask for `World` cannot write to stdout.

## Distinctive moves.

- Stable diagnostic codes, contractually stable across compiler versions; agents match on the code, not the prose.
- Typed repair plans: `zero fix --plan --json` returns a structured edit plan, not advice; the agent applies it rather than inferring from the message.
- Version-matched skills: `zero skills get zero --full` returns syntax, diagnostics, build, package, stdlib, testing, and edit-loop guidance pinned to the installed compiler version; guidance lives in the toolchain, not a possibly-drifted webpage.
- No LLVM, sub-10 KiB binaries: direct ELF/Mach-O/PE/WebAssembly emitters; the size budget is a load-bearing design constraint, not marketing.
- One CLI surface: zero check, run, build, graph, size, routes, skills, explain, fix, doctor — subcommands of a single binary, all supporting --json.

## Maturity.

v0.1.1, Apache-2.0, released 15 May 2026; 3.3k stars on vercel-labs/zerolang at first cataloguing. README/homepage explicit "pre-1 experiment": syntax and APIs not a contract, breaking changes expected, security vulnerabilities should be expected — Vercel Labs recommends running Zero only in isolated environments. Two compilers in-repo: zero-c (C bootstrap) and compiler-zero (self-hosting, written in Zero). Cross-compilation limited to a documented target subset; no package registry yet; VS Code syntax highlighting ships in-repo. Named contributors: Chris Tate, Matt Van Horn. The bet: structured agent-first compiler output becomes table stakes; even if Zero doesn't win, the pattern — stable codes, typed repairs, version-matched skills — is a concrete argument for what every other compiler should ship.

## Agent tooling.

The toolchain is the agent tooling: `zero check --json` (diagnostics), `zero explain <code>` (explanations), `zero fix --plan --json` (edit plans), `zero skills get zero --full` (version-matched workflows), plus `zero graph --json`, `zero size --json`, `zero routes --json`, `zero doctor --json` for inspection. Vercel's separately released skills.sh ecosystem is consumable by Claude Code, Cursor, Codex, and other agent harnesses through the same Agent Skills spec `zero skills` follows.

### Design DNA

- MoonBit (Verification) — industrial-backing parallel: Vercel Labs and IDEA Shenzhen are the catalogue's two best-resourced bets; MoonBit invests in semantic-aware sampling, Zero in structured compiler output and version-matched skills.
- NERD (Syntactic) — cross-camp foil: both lean on a small keyword vocabulary and 'one obvious way'; NERD for syntactic legibility, Zero inside a verification project with capability-typed effects and a typed repair API.
- Boruna (Orchestration) — structured-diagnostics parallel: Zero ships JSON diagnostics with typed repair IDs at the language level; Boruna ships hash-chained evidence bundles at the runtime level. Both reject prose as an interface for agents.

Detail: https://agentlanguages.dev/languages/zero/ · MD: https://agentlanguages.dev/languages/zero.md

---

# Orchestration camp (5)

> It isn't a language problem. It's an agent-coordination problem. The camp re-frames the question: the trouble with LLM-authored code isn't any specific defect in the code — agents need to be sequenced, sandboxed, audited, and approved by humans at the right points. The language is just the substrate; the runtime is where the action is.

## Boruna

Camp: Orchestration (also spans Verification). Author: escapeboy. Impl: Rust. Target: bytecode (custom VM). Licence: MIT. First seen: Apr 2026. Maturity: working compiler.
Site/Repo: https://github.com/escapeboy/boruna
Agent tooling: MCP server with 10 tools, AGENTS.md, CLAUDE.md, diagnostics and auto-repair commands.

### Key idea

Deterministic, capability-safe workflow execution for auditable AI systems. DAG workflows whose steps are `.ax` source files. Every side effect — LLM calls, HTTP, database, filesystem — declared in source and policy-gated at the VM level. Hash-chained tamper-evident evidence bundles; deterministic replay; approval gates for human-in-the-loop. Pitch: when a regulator asks what exactly ran and what the model returned, you can prove it.

## The thesis.

The problem isn't the code: when an agent system does something consequential — sends an email, transfers money, modifies a database — you must be able to prove what ran, what the model said, and who approved it. That's a runtime problem, so Boruna builds the runtime. The VM refuses to execute a step that would perform an undeclared effect; the policy layer lets administrators forbid specific declared effects per workflow or per role. Every executed step writes to a hash-chained evidence bundle sufficient to replay the workflow deterministically (same inputs, recorded model responses, same outputs).

## Distinctive moves.

- Capability-safe by construction: a step can't reach an undeclared effect; the VM is the enforcement point, not a linter.
- Hash-chained evidence bundles: every step's inputs, outputs, model responses, and approvals chain into a Merkle structure; tampering breaks the chain.
- Deterministic replay: re-running against the evidence bundle produces bit-identical results — no "it worked on my machine" for LLM-driven workflows.
- Approval gates: human-in-the-loop steps are a first-class workflow primitive, not bolted on; the approval becomes part of the evidence.
- MCP server with 10 tools: a coding agent can author, validate, and run workflows without leaving the protocol.

## Maturity.

v0.2.0, 34 commits, 1 release. 557+ tests passing across a 9-crate Rust workspace: compiler (lexer, parser, type checker, code generator), bytecode VM, orchestrator, MCP server. Single author; zero stars at cataloguing, dramatically understating the engineering depth — the architecture is more carefully thought through than entries with two orders of magnitude more attention. The bet: regulated industries (financial services, healthcare, government) discover Boruna before the broader market; once regulators reach the agent gold rush, "I can prove what ran" stops being a feature and becomes a requirement.

## Agent tooling.

AGENTS.md and CLAUDE.md orient agents in the repository. MCP server exposes ten tools: draft workflows, run in dry-run mode, validate effect declarations against policy, inspect evidence bundles, trigger approvals. Diagnostics ship auto-repair commands — type-checker rejections suggest the specific edit that would satisfy.

### Design DNA

- Vera (Verification) — cross-camp cousin: both treat agent code as untrusted by default; Vera builds trust at the type level (contracts), Boruna at the runtime level (policy-gated effects + evidence bundles). Vera's `<Inference>` effect is conceptually close to Boruna's declared LLM call.
- Pel (Orchestration) — same camp, different stack: Pel argues grammar-level capability control and exists as an academic paper; Boruna implements bytecode-level gating and ships as a 9-crate Rust workspace.
- Quasar (Orchestration) — shared approval-gate intuition: Quasar measured 52% fewer user-approval interactions by lifting approval into the language; Boruna lifts it into the runtime with deterministic replay so the approval can be audited after the fact.
- Plumbing (Adjacent) — Plumbing types the wiring between agents (typed channels, structural morphisms); Boruna defines what runs inside one agent and how it's audited. Complementary, not competing.

### Timeline

- Apr 2026 — v0.2.0 published: 9-crate Rust workspace (compiler, bytecode VM, orchestrator, MCP server).
- Apr 2026 — 557+ tests passing; hash-chained evidence bundle format stabilises.
- May 2026 — catalogued; still 0 stars; engineering depth runs ahead of public profile.

Detail: https://agentlanguages.dev/languages/boruna/ · MD: https://agentlanguages.dev/languages/boruna.md

## Lumen

Camp: Orchestration. Author: alliecatowo. Impl: Rust. Target: LIR bytecode → register-based VM (~100 opcodes); WebAssembly via lumen-wasm. Licence: MIT. First seen: Feb 2026. Maturity: working compiler.
Site: https://alliecatowo.github.io/lumen/ · Repo: https://github.com/alliecatowo/lumen
Agent tooling: AGENTS.md (multi-agent dev team config), CLAUDE.md, .opencode/agents/, LSP server (lumen-lsp) with semantic search, VS Code extension, Tree-sitter grammar, MCP provider crate (lumen-provider-mcp), emit-bytecode-as-JSON CLI (`lumen emit`).

### Key idea

A language for humans authoring agent workflows, not for agents to author general code — it earns catalogue inclusion via the orchestration-camp criterion: first-class effect declarations for model calls and agent-coordination primitives. Algebraic effects appear in function signatures after a slash; grants constrain every tool call with explicit caps; @deterministic enforces rejection of nondeterministic ops at compile time; pipeline / machine / memory are first-class process kinds. Markdown-native source: .lm.md files unify code and documentation in one artefact.

## The thesis.

Targets the substrate above the model; the catalogue's nominal bar is "designed for LLMs to author code," and Lumen qualifies via the orchestration criterion. The vocabulary is the giveaway: `cell` (function), `effect`, `grant`, `agent`, `pipeline`, `machine`, `memory` are all language keywords. Effects declare in the return type after a slash (`cell main() -> String / {Log}`); `@deterministic true` mode rejects nondeterministic operations at compile time, not runtime. Pitch: "Build deterministic agent workflows with static types, first-class AI primitives, and markdown-native source files." Distinctive move: the source file is the same artefact as the documentation. Three extensions ship: .lm.md (markdown with fenced Lumen blocks), .lm (raw source), .lumen (markdown-native); code and prose share one file, and the model writing one writes the other. Where Boruna enforces deterministic workflows at the bytecode VM (policy-gated effects, hash-chained evidence), Lumen does it at the type system (algebraic effects, grants, @deterministic) — same orchestration-camp diagnosis, different layer.

## What it looks like.

```
effect Log
  cell info(msg: String) -> Unit
end

cell main() -> String / {Log}
  perform Log.info("Starting")
  return "Done"
end

handle main() with Log.info(msg) -> resume(unit)
  print("LOG: {msg}")
end
```
`/ {Log}` in the return type declares the effect; `perform` invokes it; `handle ... with ...` discharges it. One-shot delimited continuations under the hood. `cell` is the function keyword — Lumen does not use `fn`.

## Distinctive moves.

- Markdown-native source: .lm.md = markdown prose with fenced `lumen` blocks; extraction is the compiler's first pipeline stage; documentation and implementation are one artefact.
- `cell` is the function keyword, not `fn`; cells take typed parameters and declare effects in the return type after a slash.
- Algebraic effects, first-class: `effect Log` declarations, `perform` to invoke, `handle ... with ...` to discharge; one-shot delimited continuations; opcodes Perform, HandlePush, HandlePop, Resume are first-class in the VM.
- Grants as syntactic policy: `grant Chat max_tokens 1024 temperature 0.7` constrains every call to that tool; policy lives in source, not configuration — Boruna's effect declarations lifted to the language surface.
- `@deterministic true` mode: compile-time annotation rejecting `uuid()`, `timestamp()`, and other nondeterministic operations; the static analogue of Boruna's runtime deterministic replay.
- Three process kinds: `pipeline` for auto-chained stages (extract → transform → load), `machine` for state graphs, `memory` for key-value stores; each a first-class language construct, not a library pattern.
- MCP as a provider crate: lumen-provider-mcp ships alongside lumen-provider-http, lumen-provider-json, lumen-provider-fs, lumen-provider-gemini, lumen-provider-crypto, lumen-provider-env; MCP is one tool source among several, not the privileged one.
## Maturity.

v0.1.10 (February 2026), 352 commits, ~5,300 passing tests across all crates (AGENTS.md figure; README's 1,365+ is at a different cut). MIT, Rust (96.5% of source). Compiles to LIR bytecode for a register-based VM: ~100 opcodes, 32-bit fixed-width encoding, COW collections via `Rc::make_mut`. 12+ crates: compiler, VM, runtime, CLI, LSP, JIT codegen, WebAssembly bindings, tensor ops, provider integrations. Single human author (`alliecatowo`); AGENTS.md notes "only the Delegator agent commits code" — the contributors listing reflects agent runs of the project's own multi-agent dev team.

## Agent tooling.

Unusually elaborate. AGENTS.md declares a multi-agent dev team — Delegator (Gemini 3 Pro), Auditor, Debugger (Claude Opus 4.6), Coder (Claude Sonnet 4.5), Worker (Claude Haiku 4.5), Tester, Task Manager, Performance, Planner — each with a defined role; only the Delegator may commit. CLAUDE.md and `.opencode/agents/` add orientation. LSP includes semantic search; VS Code extension covers `.lm.md`; tree-sitter grammar at `tree-sitter-lumen/`; `lumen emit` outputs bytecode as JSON for downstream agent consumption.

### Design DNA

- **Boruna** (Orchestration) — closest design relative; both target deterministic AI workflows, both ship MCP. Boruna enforces at the bytecode VM (policy-gated effects, hash-chained evidence bundles); Lumen at the type system (algebraic effects, grants, compile-time `@deterministic`). Lumen targets humans authoring workflows; Boruna auditable execution.
- **Plumbing** (Adjacent) — Plumbing wires agents (typed channels, copy-discard symmetric monoidal category); Lumen is what runs inside a single node of that wiring. Complementary substrates, not competitors.
- **AILANG** (Verification) — AILANG's row-polymorphic Hindley-Milner with capability categories (IO/FS/Net/Clock/AI) is the verification cousin of Lumen's effect-row syntax. Different mechanism, same diagnosis: model calls must be visible in the signature.

*Detail page: https://agentlanguages.dev/languages/lumen/ · Markdown companion: https://agentlanguages.dev/languages/lumen.md*

## Marsha

**Camp:** Orchestration · **Author:** David Ellis (Alan Technologies) · **Implementation language:** Python · **Compilation target:** Python (LLM-generated, with auto-generated tests) · **Licence:** MIT · **First seen:** July 2023 · **Maturity:** early implementation · **Site/Repo:** https://github.com/alantech/marsha

### Key idea

Functional, English-based language; the LLM is the compiler: `.mrsh` spec in, tested Python out.

## The thesis.

The framing predates almost every other catalogue entry. A `.mrsh` file is a Markdown-shaped spec with three sections per function: typed declaration (`# func name(InputType): OutputType`), English behaviour description, and bullet list of input/output examples including expected error cases. The toolchain prompts an LLM to generate Python satisfying the declaration, synthesises a pytest suite from the examples, and iterates with corrective feedback until tests pass or the attempt budget is exhausted. CLI flags expose iteration parameters: `-a` attempts, `-n` parallel "thought" threads, `-q` quick-and-dirty, `--exclude-main-helper`. Generated programs ship an auto-attached CLI wrapper and optional REST-server mode (`-s`).

## Published results.

Repo ships an alpha implementation, an examples directory (general-purpose, web-scraping, data-mangling), and a CI workflow timing compilations. Only quantitative target is the README's own roadmap: "We aim for 80%+ accuracy on our examples", to push "above 90%". Requires `OPENAI_ORG` and `OPENAI_SECRET_KEY`; other/local LLMs planned, unimplemented. setup.py PyPI classifier: `Development Status :: 2 - Pre-Alpha`.

## Status.

MIT; Show HN 1 August 2023 (news.ycombinator.com item 36864021); high-hundreds stars, ~a dozen forks. Last maintainer activity on main: PRs #159–#164 from `dfellis` and issue #165 from `depombo`, dated 1–8 Aug 2023 (PR #164 "Embed Llama.cpp into Marsha for local usage" 7 Aug; issue #165 "Add LlamaCPP support" 8 Aug); no maintainer PRs/issues since. Both principals moved on (David Ellis to IaSQL then the Alan-lang project; Luis de Pombo's LinkedIn lists continued tenure at Alan Technologies). Catalogued because the 2023 "LLM is the compiler" framing anticipates the 2025 orchestration papers, not for active development.

### Design DNA

- **Pel** (Orchestration) — same camp, two years apart: Marsha (2023) treats the LLM as a compiler emitting Python under English+examples; Pel (2025) as a code emitter constrained by a grammar designed for it. Both predate the now-common 'agents write code' framing.
- **Boruna** (Orchestration) — inverted topology: Marsha puts the LLM at the compiler back end; Boruna treats every LLM call as a policy-gated effect inside a deterministic VM.
- **Quasar** (Orchestration) — different generation: Quasar measures execution-time and approval-interaction reductions on ViperGPT/CaMeL; Marsha measures end-to-end compile success against its own examples ('we aim for 80%+ accuracy').

*Detail page: https://agentlanguages.dev/languages/marsha/ · Markdown companion: https://agentlanguages.dev/languages/marsha.md*

## Pel

**Camp:** Orchestration · **Author:** Behnam Mohammadi (CMU) · **Implementation/compilation target:** N/A (paper-only) · **Licence:** N/A (academic paper) · **First seen:** April 2025 · **Maturity:** research paper · **Site/Paper:** https://arxiv.org/abs/2505.13453

### Key idea

Lisp-flavoured orchestration language where the grammar itself is the capability surface: the LLM emits Pel under constrained generation, so an action the grammar cannot express is an action the agent cannot take.

## The thesis.

Diagnosis: tool calling cannot express control flow; Python is too expressive to run safely without a sandbox. Pel is Lisp-inspired, homoiconic, minimal-grammar — small enough to serve as a constrained-decoding target, making capability control a property of generation rather than runtime sandboxing. Cues from Elixir (piping for linear composition), Gleam (typing discipline), Haskell (first-class closures, partial application). A REPeL — Read-Eval-Print-Loop with Common Lisp-style restarts — couples the evaluator to LLM-powered helper agents that propose restart choices when an error is signalled, making error recovery a language feature rather than an application concern.

## Published results.

Design/rationale paper, not a benchmark study: specifies grammar, data types, closure semantics, piping operators, list operations, control flow, natural-language conditions evaluated by an LLM, and automatic asynchronicity/parallelisation via static dependency analysis. Pel is the implementation substrate for BEACON (Business Enhancement through Adaptive COordinated Networks), Mohammadi's separate SSRN paper (abstract 5191583): a hierarchical multi-agent framework distributing specialised knowledge across marketing, finance, HR, and strategic-planning agents for small and family-owned businesses; BEACON reports advantages over single-model generative AI on information-retrieval accuracy, cost-efficiency, and interpretability, citing Pel as orchestration substrate.

## Status.

arXiv preprint (v1 3 Apr 2025; v2 9 Jun 2025), single author; PhD CMU Tepper 2025 (thesis "Human-AI Interaction in the Era of Large Language Models (LLMs)", KiltHub 9 Jul 2025); now tenure-track in Quantitative Marketing at UT Dallas's Naveen Jindal School of Management. BEACON is supported by a BNY Foundation of Southwestern Pennsylvania fellowship via Tepper's Center for Intelligent Business. No public implementation, package, or repository; independent evaluation requires a reference compiler or access to the BEACON codebase.

### Design DNA

- **Boruna** (Orchestration) — same camp, different layer: Pel argues grammar-level capability control; Boruna gates the same effects at the bytecode VM. Pel is a paper; Boruna ships a 9-crate Rust workspace.
- **Quasar** (Orchestration) — the other 2025 academic orchestration paper: Quasar transpiles a Python subset and instruments it with conformal prediction and approval gates; Pel replaces the surface language entirely and constrains generation against its grammar.
- **Marsha** (Orchestration) — two-year-earlier predecessor on the same axis: LLM as compiler back end producing Python vs LLM as code emitter constrained by a purpose-designed grammar.

*Detail page: https://agentlanguages.dev/languages/pel/ · Markdown companion: https://agentlanguages.dev/languages/pel.md*

## Quasar

**Camp:** Orchestration (also spans Verification) · **Author:** Stephen Mell et al. (Penn) · **Implementation/compilation target:** N/A (paper-only) · **Licence:** N/A (academic paper) · **First seen:** June 2025 · **Maturity:** research paper · **Site/Paper:** https://arxiv.org/abs/2506.12202

### Key idea

Quasar (backronym: QUick And Secure And Reliable) accepts code actions in a Python subset transpiled to a custom language with three built-in properties: automatic parallelisation of independent external calls, compositional conformal prediction for uncertainty quantification, and explicit user-approval gates around sensitive tool invocations. The LLM keeps writing the Python it knows; the runtime supplies the guarantees Python lacks.

## The thesis.

LLM agents increasingly act by writing code, and Python is the default because LLMs are fluent in it, not because it is suited — it lacks built-in performance, security, and reliability support. Performance: automatic parallelisation of independent external calls, drawing on Mell, Kallas, Zdancewic and Bastani, "Opportunistically Parallel Lambda Calculus" (arXiv:2405.11361; Proc. ACM Program. Lang. 9, OOPSLA2, October 2025). Reliability: compositional conformal prediction (Ramalingam, Park and Bastani, "Uncertainty Quantification for Neurosymbolic Programs via Compositional Conformal Prediction", arXiv:2405.15912), converting model outputs into prediction sets with a user-chosen target error rate. Security: user-validated action gates surfaced only when static analysis cannot rule out a sensitive effect.

## Published results.

arXiv v1 (13 Jun 2025): on the ViperGPT visual question answering agent over GQA, LLMs emitting Quasar instead of Python retain task performance while reducing execution time when possible by 42% and user-approval interactions when possible by 52%; conformal prediction achieves a chosen target coverage. The OpenReview revision (id TvpaeQVTGQ) extends evaluation to the CaMeL agent on the AgentDojo prompt-injection benchmark and revises headlines upward to "up to 56%" execution-time reduction and "up to 53%" fewer user approvals.

## Status.

No public implementation, repository, or release; the OpenReview submission is under review at cataloguing time, conference acceptance unannounced. Independent evaluation needs the authors' transpiler/runtime or a reimplementation against the published semantics; the ViperGPT/GQA and CaMeL/AgentDojo baselines are public and reproducible. Full author list: Stephen Mell, Botong Zhang, David Mell, Shuo Li, Ramya Ramalingam, Nathan Yu, Steve Zdancewic, Osbert Bastani.

### Design DNA

- **Boruna** (Orchestration) — both lift approval gates into a first-class language primitive: Quasar reports 52% fewer user-approval interactions by inferring when approval is unnecessary; Boruna routes the same primitive through a deterministic VM chaining every approval into a tamper-evident evidence bundle.
- **Pel** (Orchestration) — the other 2025 academic orchestration paper: Pel replaces the surface language with a Lisp-shaped grammar for constrained generation; Quasar keeps a Python subset and inserts the guarantees underneath.
- **Vera** (Verification) — cross-camp foil on 'make it checkable': Vera discharges contracts via Z3 at compile time; Quasar layers conformal prediction over LLM outputs for a target coverage probability at runtime.

*Detail page: https://agentlanguages.dev/languages/quasar/ · Markdown companion: https://agentlanguages.dev/languages/quasar.md*

---

# Adjacent (1)

> Infrastructure operating around agent-authored code rather than authored by agents itself: wiring layers, runtime substrates, and tooling the three-camp argument depends on but doesn't directly produce.

## Plumbing

**Camp:** Adjacent · **Author:** William Waites / Leith Document Company · **Implementation language:** not publicly disclosed · **Compilation target:** native binaries (Linux x86_64, macOS Apple Silicon) · **Licence:** free for educational/personal use; commercial licence on request · **First seen:** March 2026 · **Maturity:** early implementation · **Site:** https://johncarlosbaez.wordpress.com/2026/03/11/a-typed-language-for-agent-coordination/ · **Paper:** https://arxiv.org/abs/2602.13275 · **Agent tooling:** MCP server

### Key idea

A typed language for the wiring between agents — the substrate orchestration-camp languages run on top of. Objects: typed channels carrying infinite streams. Morphisms: processes. Agents: stateful morphisms. Type-checking happens before the graph runs.

## The thesis.

Not an orchestration language in the sense Boruna, Pel, or Marsha are — it is the typed substrate orchestration languages can be expressed on top of. Where existing frameworks (n8n, LangGraph, CrewAI) coordinate agents with ad hoc engineering, Plumbing uses morphisms in a copy-discard symmetric monoidal category: `!A` is a stream of `A`s, `!string` a stream of strings; morphisms are processes with typed inputs/outputs; the compiler statically checks every wiring is well-formed before any agent runs. Pullquote: "Static typing prevents the waste." The distinctive move refuses the orchestration camp's framing: Boruna's unit is an `.ax` workflow with declared effects, Pel's a grammar-level capability; Plumbing's unit is the channel between two processes, the agent reduced to a stateful morphism with a typed protocol — main input/output plus control ports for runtime parameter modulation (e.g. temperature), tool-call ports, operator-in-the-loop ports, and telemetry. A judge agent cooling a debate sends `set_temp` on the debaters' control ports; that wiring is type-checked the same as the data path.

## What it looks like.

```
type Verdict = { verdict: bool, commentary: string, draft: string }
type Review  = { score: int, review: string, draft: string }

let composer : !string  -> !string  = agent { ... }
let checker  : !string  -> !Verdict = agent { ... }
let critic   : !Verdict -> !Review  = agent { ... }

let main : !string -> !string = plumb(input, output) {
  input   ; composer ; checker
  checker ; filter(verdict = false)
          ; map({verdict, commentary}) ; composer
  checker ; filter(verdict = true) ; critic
  critic  ; filter(score < 85)
          ; map({score, review}) ; composer
  critic  ; filter(score >= 85).draft ; output
}
```

Caption: an adversarial cover-letter composer with two feedback loops. The critic cannot see source materials — the information partition is a type-level consequence of the wiring, not a prompt instruction.

## Distinctive moves.

- **Typed channels, not typed messages.** Objects are streams; sequential composition glues stream-producing morphisms; the tensor product runs them in parallel; well-formedness is a category-theoretic property checked at compile time.
- **Four structural morphisms** (plus utilities map, filter): copy duplicates a stream, discard throws it away, merge interleaves two same-type streams (after coproduct injection), barrier synchronises two streams into a pair — the synchronisation primitive that unlocks session types.
- **The κ-calculus "don't care, don't write" convention.** Output ports unmentioned in the program are implicitly connected to discard; the textual surface stays small while the type system still tracks every port.
- **MCP server in the release.** The first public drop ships compiler, interpreter, and MCP server — agent harnesses are first-class consumers of the language, not an afterthought.

## Maturity.

Version 0p1, first public release March 2026; binaries for Linux x86_64 and macOS Apple Silicon; ships compiler, interpreter, MCP server. No public Git repository; free for educational/personal use, commercial licence from Leith Document Company. Author William Waites, Chancellor's Fellow at the University of Strathclyde; the broader programme is his arXiv paper *Artificial organisations* (arXiv:2602.13275). A second blog post, "The agent that doesn't know itself," extends the calculus with session types and context compaction. The bet: orchestration languages eventually need a category-theoretic substrate, more valuable as a typed coordination layer than as another orchestration framework competing for workflow attention.

## Agent tooling.

The MCP server is the entire agent-facing surface — no SKILL.md, AGENTS.md, or CLAUDE.md in this drop, because the framing is that the language *is* the agent tooling for everything above it: harnesses consume Plumbing through MCP, and what agents author *in* Plumbing is the wiring diagram for other agents.

### Design DNA

- **Boruna** (Orchestration) — Plumbing defines the wiring between agents; Boruna defines what runs inside one agent and how it is audited. Substrate vs workload.
- **Pel** (Orchestration) — same orchestration adjacency, different formalism: Pel is a grammar-level capability calculus on an academic paper; Plumbing a copy-discard category with session types, with a working compiler and runtime.

*Detail page: https://agentlanguages.dev/languages/plumbing/ · Markdown companion: https://agentlanguages.dev/languages/plumbing.md*

---

# Unclassified (3)

> Candidates that haven't shipped enough machinery — or enough public evidence — to classify yet. Presence is a marker of position, not a placement claim.

## Koru

**Camp:** Unclassified · **Author:** anonymous (korulang) · **Implementation language:** Koru (metacircular, bootstrapped through Zig) · **Compilation target:** Zig (then native via Zig's backends) · **Licence:** unknown · **First seen:** December 2025 · **Maturity:** early implementation · **Site:** https://www.korulang.org/ · **Repo:** https://github.com/korulang/koru

### Key idea

Zig-superset systems language: every `.kz` file is valid Zig; Koru constructs carry a `~` leader. Distinctive move: event continuations with mandatory branch handling — events declare their inputs and possible output branches in advance, and invoking an event requires explicitly handling each branch.

## What it is.

Pre-alpha. Phantom typing tracks compile-time resources; purity is tracked through the type system; the compiler is metacircular (Koru compiles to Zig). Author anonymous behind the korulang GitHub org and Twitter account; the site lists Claude Opus 4.1–4.5 and Sonnet 4.5 as the models that authored the compiler itself. Tagline, intentionally tongue-in-cheek: "The Hyper-Performant AI-First Postmodern Zero-Cost Fractal Metacircular Phantom-Typed Auto-Disposing Monadic Event Continuation Language with Semantic Space Lifting and Event Taps." README is candid: "Pre-Alpha — Koru is pre-alpha. It has only ever been compiled on a single computer. Use and testing at your own risk." No SKILL.md, AGENTS.md, MCP server, or structured-JSON diagnostics ship; the "AI-First" claim is architectural (event boundaries provide bounded contexts that aid AI reasoning), not machinery-based, and the closest planned agent-facing surface is the Compiler Control Protocol (CCP), described as "soon" — a Koru-internal proposal, not the Model Context Protocol.

## Why it's here.

Marker of the position where "AI-First" became a tagline trope: the architectural claim is a real design move, the buzzword-cascade tagline real satire, and the unclassified bucket exists for projects whose camp placement isn't yet evidenced by shipped agent-authoring machinery. Companion to Valea and Spec: a real project with substantive design, candidly aware of its pre-alpha state; that disclosure is the editorial centre of gravity.

### Design DNA

- **Valea** (Unclassified) — companion: both stake an 'AI-native systems programming language' position with substantive design proposals but limited public evidence and no agent-authoring machinery shipped. Valea is a Rust MVP announced on Hacker News with JSON diagnostics planned; Koru a Zig-superset metacircular compiler with event continuations and an explicitly tongue-in-cheek marketing voice.
- **Spec** (Unclassified) — adjacent on the 'architecture as AI-friendliness' axis: Spec proposes a two-domain IR for multi-agent collaboration; Koru proposes architectural primitives (event boundaries, mandatory branch handling) that aid AI reasoning at the language level. Both make architectural AI-friendliness claims without shipping agent-authoring tooling.

*Detail page: https://agentlanguages.dev/languages/koru/ · Markdown companion: https://agentlanguages.dev/languages/koru.md*

## Spec

**Camp:** Unclassified (also spans Orchestration) · **Author:** M. Abdullah Onus · **Implementation language:** TypeScript (React POC) · **Compilation target:** not applicable — IR artefacts (.spec.ir files), not executable · **Licence:** MIT · **First seen:** April 2026 · **Maturity:** thought experiment · **Site/Repo:** https://github.com/mronus/spec · **Agent tooling:** browser-based React/TypeScript POC at mronus.github.io/spec orchestrating the six agents end-to-end — multi-agent pipeline with feedback loops, state persistence, resume support; supports Claude and GPT models; API keys remain in the browser

### Key idea

Two domains. Spec Domain, language-agnostic: six specialised agents (Product, Architect, Scrum, Developer, Tester, DevOps) collaborate to produce ten .spec.ir artefacts describing what the system should do. External Agents Domain, language-specific: separate language agents (Java, Go, Terraform, etc.) consume the IR and produce code. The bet: one specification targets multiple languages, and incremental modification takes the proposal's claimed 200 tokens of context instead of the 1,500 a comparable Java change requires.

## What it is.

A draft design proposal at v0.2 (`spec-language-design-proposal-v0.2.md`), not a working compiler; the README explicitly labels it a draft for discussion. Ten artefact types — contract.spec.ir, module.spec.ir, infrastructure.spec.ir, data.spec.ir, types.spec.ir, interfaces/*.spec.ir, functions/*.spec.ir, events.spec.ir, tests.spec.ir, pipeline.spec.ir — each owned by a specific agent role. The external language agents that would translate IR to running code (Java, Go, Rust, Terraform, others) are listed as future work, not yet implemented.

## Why it's here.

Marker of an architectural position spanning into orchestration: where the syntactic and verification camps argue about what an agent should write, Spec argues who should write what, in what order — Product before Architect before Developer, with explicit IR handoffs between roles — treating the multi-agent pipeline as the primary artefact and language-specific code generation as deferrable downstream work. Not rated against working compilers; catalogued as a structured argument that "language for agents to write" might be the wrong unit of analysis and "IR for agents to coordinate over" the unit that matters.

### Design DNA

- **Boruna** (Orchestration) — cross-camp neighbour: Boruna runs DAG workflows with policy-gated effects and hash-chained evidence; Spec coordinates specialist agents producing shared IR artefacts. Both treat the language as one layer in a larger orchestration story.
- **Pel** (Orchestration) — academic-leaning kin: both propose agent-collaboration architectures that have not yet shipped a usable language — Pel as a CMU paper, Spec as a draft proposal with a browser-based POC.

*Detail page: https://agentlanguages.dev/languages/spec/ · Markdown companion: https://agentlanguages.dev/languages/spec.md*

## Valea

**Camp:** Unclassified · **Author:** Hans Voetsch (Google) · **Implementation language:** Rust · **Compilation target:** C (via the emit-c command) · **Licence:** MIT · **First seen:** March 2026 · **Maturity:** early implementation · **Site/Repo:** https://github.com/hvoetsch/valea · **Agent tooling:** JSON diagnostics (`valea check --json`) with stable error codes such as E001; JSON AST export (`valea ast --json`); canonical formatter (`valea fmt`); AGENTS.md and CLAUDE.md present for agent orientation

### Key idea

Five declared design properties: deterministic syntax; explicit semantics with no hidden allocations or exceptions; machine-readable diagnostics; canonical formatting; small language surface to reduce edge cases. README's intended workflow: agent receives a goal, writes Valea, reads JSON diagnostics from the compiler, applies fixes, produces a program that compiles and runs.

## What it is.

Not enough public information to classify — that honesty is the entry's defining quality. Public record: a Hacker News post from March 2026 titled "Valea: An AI-native systems programming language", the `hvoetsch/valea` GitHub repository, and a README framing a community language experiment with an example function in a Rust-flavoured surface. The compiler is a Rust MVP with four subcommands: `check` (JSON output), `ast` (JSON output), `fmt`, `emit-c`. The repo contains SPEC.md, MANIFESTO.md, ROADMAP.md, AGENTS.md, and CLAUDE.md, with 24 commits and a single asciinema demo recording. The Google affiliation on the catalogue card is not stated in the repository itself.

## Why it's here.

Marker of the field's noise floor: projects with this much intent, this little code, and this much manifesto are now common enough to need the unclassified bucket. The relevant observation is not what Valea is but that "AI-native systems programming language" has become a recognisable Hacker News category with a recognisable shape (Rust compiler, JSON diagnostics, agent-oriented README files) — an early signal that the field's design vocabulary is stabilising before implementations ship enough to evaluate. Not rated against working compilers with measured benchmarks.

*Detail page: https://agentlanguages.dev/languages/valea/ · Markdown companion: https://agentlanguages.dev/languages/valea.md*

---

## See also

- Homepage (HTML): https://agentlanguages.dev/
- Homepage (markdown): https://agentlanguages.dev/index.md
- Short index: https://agentlanguages.dev/llms.txt
- Sitemap: https://agentlanguages.dev/sitemap.xml
- Source repository: https://github.com/aallan/agentlanguages
