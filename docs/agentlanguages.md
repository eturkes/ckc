# agentlanguages.dev — full catalogue

> Complete machine-readable text of the agentlanguages.dev catalogue. 32 languages designed for AI agents to author code, organised across five buckets: three philosophical camps (syntactic, verification, orchestration) plus adjacent and unclassified.

This file is the full-text companion to the short index at https://agentlanguages.dev/llms.txt — included here so that an agent that wants the entire catalogue can fetch it in one HTTP round-trip rather than 33 (homepage + one per entry).

Each entry below carries a `## Name` heading and a metadata block (camp, author, implementation language, compilation target, licence, first seen, maturity, site, repo, paper, agent tooling) followed by the editorial prose and, where present, design-DNA cross-references and timeline events.

Originally catalogued in a post "Three camps alike in dignity" (https://negroniventurestudios.com/2026/05/20/three-camps-alike-in-dignity/) written for the Negroni Venture Studios blog. Maintained by Alasdair Allan.

Editorial principles: descriptive, not promotional. No ranking. Inclusion is based on whether a language's designers explicitly target LLMs or agents as authors. Tools that *use* an LLM at runtime are out of scope.

---

# Syntactic camp (12)

> If the problem is that models trip on syntax, the fix is to strip ambiguity from the syntax itself. The syntactic camp treats the problem as one of representation — models choke on tokens that mean different things in different positions, on operators that need disambiguation, on whitespace that might or might not be load-bearing. Their answer: build a syntax where every token has one job.

## Axis

> Backend API language for small LLMs (1B/3B/7B). Twelve top-level constructs cover a production backend; LL(1) grammar and prefix-only expressions ship per-state logit masks for constrained decoding.

**Camp:** Syntactic
**Also spans:** Verification
**Author:** Vladimir Melnic
**Implementation language:** Rust
**Compilation target:** Native artefacts (SQL DDL, Rust/axum server, TypeScript and Rust client SDKs, OpenAPI, GraphQL); also interpreted directly from the AST via axum + sqlx
**Licence:** Unknown
**First seen:** May 2026
**Maturity:** working compiler
**Site:** https://github.com/vmelnic/axis
**Repo:** https://github.com/vmelnic/axis
**Agent tooling:**
- CLAUDE.md
- axis --constrain (grammar state machine as JSON)
- axis --logit-masks (per-state binary masks over the Axis token vocabulary)
- axis --completions (parser state and valid next tokens at cursor)
- axis --lsp (language server over stdio)

### Key idea

Axis targets small LLMs (1B/3B/7B) explicitly. The bet is that a
backend authored in twelve constructs (SHAPE, SOURCE, REALM, FLOW,
SAGA, SURFACE, POLICY, SERVICE, MIGRATE, STREAM, FUNC, STORAGE) with
an LL(1) grammar and prefix-only expressions fails less than
free-form framework code. The compiler ships grammar-aware logit
masks as JSON; the same source either compiles to SQL / OpenAPI /
TypeScript / Rust artefacts, or is served directly from the AST by
axum + sqlx against Postgres, MySQL, or SQLite.

## The thesis.

Axis names a different bet from the rest of the syntactic camp. Most catalogue entries target "LLMs" generically and assume the model has enough context and capability to navigate a large surface. Axis targets the smallest practical agents explicitly &mdash; the v0.2 spec declares "target authors: 1B, 3B, 7B parameter models with 2K&ndash;32K context windows" &mdash; and constrains the surface accordingly. Twelve top-level keywords (`SHAPE`, `SOURCE`, `REALM`, `FLOW`, `SAGA`, `SURFACE`, `POLICY`, `SERVICE`, `MIGRATE`, `STREAM`, `FUNC`, `STORAGE`) cover a production backend; the grammar is LL(1) with one token of lookahead and no backtracking; expressions are prefix-only (`FILTER id EQ path.id`, never `id == path.id`); the "one operation per binding" principle forbids compound expressions.

<p class="pullquote">Axis is not a better Python. It is the negation of human-centric programming.</p>

The distinctive move is constrained decoding shipped as toolchain output. `axis --constrain` exports the parser as a state machine in JSON; `axis --logit-masks` exports a per-state binary mask over the Axis token vocabulary. The implementation in `src/editor/constrain.rs` enumerates 41 parse states (`TopLevel`, `ShapeBody`, `FlowBodyField`, `FilterClause`, &hellip;) and emits the allowed token set for each. A host harness maps these masks onto an LLM tokeniser at decode time; the model can then only emit syntactically valid Axis. Most agent-targeted languages assume free generation followed by verification; Axis tries to make syntactic invalidity unreachable at generation time. Mog reaches a comparable surface bound from the spec direction (3,200 tokens fits the language); Axis reaches it from the grammar direction and adds the masks.

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

- **Targeted at 1B/3B/7B-parameter models.** The spec names the audience explicitly. Design principles include "context locality: a single endpoint must be fully generatable within 512 Axis tokens (~2K LLM tokens)" and "failure is syntax: unhandled errors, missing auth, unindexed queries are parse or compile errors, not runtime surprises."
- **Twelve constructs cover a production backend.** `SHAPE` (data model), `SOURCE` (database binding), `REALM` (permission scope and tenancy), `FLOW` (HTTP endpoint), `SAGA` (distributed transaction with compensating steps), `SURFACE` (API routing and versioning), `POLICY` (compile-time invariants across flows), `SERVICE` (external integration), `MIGRATE` (schema migration), `STREAM` (WebSocket/SSE), `FUNC` (pure function), `STORAGE` (file backend).
- **Grammar-aware logit masks shipped as JSON.** `axis --constrain` emits the parser state machine; `axis --logit-masks` emits a binary mask per state over the Axis token vocabulary. The harness maps these masks onto an LLM tokeniser at decode time. The masks live in `src/editor/constrain.rs`; the CLI plumbing in `src/main.rs` writes them as `{vocabulary, masks}` JSON.
- **Two execution modes from one source.** The compile pipeline (`lex → parse → link → verify → plan → codegen`) emits SQL DDL (`--sql`), an axum server (`--rust`), TypeScript and Rust client SDKs, OpenAPI 3.0, GraphQL, Kubernetes manifests, migrations, and a generated test suite, each behind its own flag. `axis --serve` runs `lex → parse → verify → serve`, interpreting the AST directly through axum + sqlx against Postgres, MySQL, or SQLite (auto-detected from `DATABASE_URL`). No codegen step on the serve path.
- **Prefix operators, one operation per binding.** `FILTER id EQ path.id` puts the operator first; the design principle forbids compound expressions like `a + b * c`. Every `LET` binds exactly one operation, removing precedence ambiguity at the syntax level.
- **POLICY as compile-time cross-cutting rules.** Tenant isolation, capability checks, rate limits, and auth requirements are declared once in a `POLICY` block and verified across every `FLOW` at compile time rather than re-asserted per endpoint.

## Maturity.

A single initial commit on 24 May 2026 dropped the entire project as a working Rust workspace: compiler front-end (`lexer`, `parser`, `link`, `verify`, `plan`), codegen backends for SQL, Rust/axum, WASM stubs, and client SDKs, generators for OpenAPI, TypeScript, GraphQL, Kubernetes manifests, migrations, and test suites, plus a stdio LSP, a VS Code extension, and a 73 KB specification (`docs/axis-spec.md` v0.2). Cargo metadata reports `version = "0.1.0"` against Rust 1.85 (edition 2024). The repository ships three working projects: `todolist/` (CRUD proof, 20/20 integration tests), `advanced/` (auth, tenants, guards, rules, funcs, MATCH, TRY/RECOVER, rate limiting, caching, surfaces &mdash; 32/32 tests), and `helperbook/` (a client-provider marketplace with Postgres, Redis, Meilisearch, Prometheus, and Grafana wired through Docker Compose, "zero application source code" outside `.axis` files).

3 stars, 0 forks, 0 open issues at time of cataloguing; no GitHub releases tagged. **The repository does not ship a `LICENSE` file** and the GitHub API reports the licence field as null; the `Cargo.toml` manifest declares `license = "MIT"`, but the absence of a top-level licence file leaves the question open until the author resolves it. The bet is the one named in the spec's framing sentence ("the negation of human-centric programming") &mdash; that a domain-specific surface drawn tight enough for a 1B model to navigate, plus grammar-aware decoding to keep it on rails, will outperform a general-purpose language plus a frontier model. No catalogue entry has previously shipped grammar-aware logit masks; integration with a downstream decoder is the next milestone to watch for.

## Agent tooling.

`CLAUDE.md` (7.5 KB) targets agents writing the Axis compiler itself, not agents writing in Axis: it names the build commands, maps the repository module by module, names canonical AST field names to head off guessing ("`TypeExpr` &mdash; not `FieldType`, not `ScalarType`"), and states the two pipelines explicitly. No `AGENTS.md`, no `SKILL.md`, no MCP server, no `llms.txt`. The surface for *authoring* Axis is the constrained-decoding pipeline: `axis --constrain` for the grammar state machine, `axis --logit-masks` for the per-state vocabulary masks, `axis --completions` for parser state and valid next tokens at a cursor position, and `axis --lsp` for editor integration. Compile-mode outputs (`--plan`, `--openapi`, `--testgen`, &hellip;) emit structured JSON for downstream agents to consume.

### Design DNA

- **Mog** *(Syntactic)* — Closest editorial sibling on the 'small constrained surface' bet. Mog fits its full spec in 3,200 tokens for a general-purpose embedded language; Axis fits a production backend in twelve top-level constructs. Different scopes, same wager that bounding the surface beats scaling the model.
- **NERD** *(Syntactic)* — Same camp, different lever. NERD strips operators down to English keywords on the bet that BPE tokenisers prefer words to symbols; Axis strips an entire backend stack down to twelve construct keywords and forbids compound expressions (one operation per <code>LET</code>).
- **X07** *(Syntactic)* — Both treat the AST as the executable artefact. X07 stores programs as canonical JSON ASTs and edits them via RFC 6902 JSON Patch; Axis keeps a textual <code>.axis</code> surface but the <code>--serve</code> mode runs the parsed AST directly through axum + sqlx with no codegen step.
- **Codong** *(Syntactic)* — Same diagnosis (choice paralysis), different scope of canonicality. Codong ships one canonical function per task across a nine-module general-purpose stdlib; Axis ships one canonical construct per backend concern — auth is <code>REALM</code>, distributed transactions are <code>SAGA</code>, cross-cutting rules are <code>POLICY</code>.

*Detail page: https://agentlanguages.dev/languages/axis/  ·  Markdown companion: https://agentlanguages.dev/languages/axis.md*

## B-IR

> A Jason Hall blog post on three attempts at an LLM-optimised language: B-IR with unicode opcodes, TBIR with control characters the model rewrote into English keywords, and Loom with pre/postconditions and stable error codes.

**Camp:** Syntactic
**Author:** Jason Hall (Chainguard)
**Implementation language:** Python (bootstrap)
**Compilation target:** Arm64 assembly (Mach-O via clang)
**Licence:** Unknown
**First seen:** January 2026
**Maturity:** thought experiment
**Site:** https://articles.imjasonh.com/llm-programming-language.md
**Repo:** https://github.com/imjasonh/loom

### Key idea

B-IR is a written narrative of three iterations. Gemini produced B-IR with multi-byte unicode opcodes, then proved too cumbersome to bootstrap. Claude Opus replaced it with TBIR using single-byte control characters in the 0x80-0x8B range, then on its own decided the unreadable characters were getting in the way and substituted short English keywords (init, fetch, emit, print, loop, exit). The final iteration, Loom, keeps token density but adds unambiguous scope, mandatory pre/postconditions, and stable error codes that the model is expected to look up rather than re-read in prose.

## What it is.

B-IR is not a project. It is a blog post by Jason Hall, principal engineer at Chainguard, first published on his personal articles site on 11 January 2026. The article describes a Sunday spent prompted by a prediction made on the Oxide and Friends "Predictions for 2026" episode that 2026 would be the year LLMs got a programming language not intelligible to humans. Hall asked first Gemini and then Claude Opus to design such a language and recorded what each model produced. The experimental artefacts &mdash; manual.md, an l1-compiler.tbir clocking in at just under 700 lines, and the loom.md specification &mdash; live in the companion GitHub repository `imjasonh/loom` (the repo was originally published as `imjasonh/b-ir`, and that URL still redirects). The article is candid about the iteration arc: the first design (multi-byte unicode opcodes) was too unwieldy for the model itself to bootstrap; the second (single-byte control characters) was abandoned mid-implementation by the model, which substituted short English keywords on its own initiative; the third (Loom) keeps token density but adds the kind of structure the verification camp has long argued for.

## Why it's here.

The catalogue includes B-IR as a marker of a meta-question. The interesting result is not the languages themselves but what happens when a working engineer asks two frontier models to design for their own consumption and reports on it honestly. Hall's own observation is that the third iteration ends up resembling existing languages with cleaner error codes and unambiguous scope &mdash; which the catalogue reads as the design space converging on the same concerns the verification camp arrived at independently. The catalogue does not rate B-IR against working compilers. It marks the article as a different kind of evidence: a candid record of what the model gravitates toward when given a blank page.

### Design DNA

- **Sever** *(Syntactic)* — Both are catalogue-meta companions. Each captures what falls out when a frontier model is asked to design a language for itself; both authors keep the result at arm's length from any production claim.
- **Laze** *(Syntactic)* — Both are small explorations by a single author working with one model over a weekend. Laze ships a compiler; B-IR ships an article.
- **Vera** *(Verification)* — Loom's conclusions — unambiguous scope, mandatory pre/postconditions, stable error codes — converge on the diagnosis Vera arrived at independently in the verification camp.

*Detail page: https://agentlanguages.dev/languages/b-ir/  ·  Markdown companion: https://agentlanguages.dev/languages/b-ir.md*

## Codong

> Designed for AI to write, with one canonical way to express every operation. Nine bundled modules, structured JSON errors with fix/retry fields, ? operator for propagation. Compiles to Go IR, then native via go build.

**Camp:** Syntactic
**Author:** Brett (brettinhere)
**Implementation language:** Go
**Compilation target:** Native binary via Go IR + `go build`
**Licence:** MIT
**First seen:** March 2026
**Maturity:** working compiler
**Site:** https://codong.org
**Repo:** https://github.com/brettinhere/Codong
**Codong Arena:** https://codong.org/arena/
**Agent tooling:**
- SPEC_FOR_AI.md (system-prompt injection — Markdown spec with paired CORRECT/WRONG examples for every rule)
- Structured JSON errors with `fix` and `retry` repair fields
- Compact error format (project-reported ~39% token reduction)

### Key idea

Codong is designed for AI to write, humans to review, machines to
execute. The syntactic-camp move is to collapse choice paralysis:
one canonical function per task, nine bundled modules covering most
AI workloads with zero external dependencies, structured JSON errors
with fix/retry fields, ? operator for error propagation. Compiles
through Go — .cod source goes to Go IR, then `go build` produces a
static native binary.

## The thesis.

Codong's diagnosis is choice paralysis. Python has five ways to make an HTTP request. JavaScript has four state-management libraries. Every choice costs tokens to navigate and produces unpredictable output. The README states it directly: "Codong is designed for AI to write, humans to review, and machines to execute." The syntactic-camp move is to collapse the language and its standard library to one canonical form per task &mdash; `http.get(url)`, `web.serve(port: N)`, `db.connect(url)`, `json.parse(s)`. Nine modules bundled, zero external dependencies, no package manager required.

<p class="pullquote">Codong has exactly one way to do everything.</p>

The distinctive move is which kind of choice gets eliminated. NERD strips operators down to English keywords; Magpie strips ambiguity at the surface by making SSA the user-facing form. Codong leaves both operators and surface alone and instead collapses the standard library &mdash; one HTTP function, one JSON parser, one error shape. Compilation routes through Go: `.cod` source passes through lexer, parser, AST, Go IR, then `go build` for a static native binary. The compiler is essentially a frontend for Go's toolchain, in the same way TypeScript is a frontend for JavaScript or Kotlin is a frontend for JVM bytecode.

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

- **One canonical function per task.** `http.get(url)` is the only way to make an HTTP request. `db.connect(url)` is the only way to open a database. No choice between five libraries; the bundled stdlib *is* the ecosystem.
- **Nine bundled modules, zero external dependencies.** `web`, `db`, `http`, `llm`, `fs`, `json`, `env`, `time`, `error` ship with the compiler. No package manager required; no version-resolution tax. (The README headline counts eight; the table lists nine — `error` ships alongside the rest.)
- **Structured JSON errors with `fix` and `retry` fields.** Errors carry a stable code, a message, a human-readable fix suggestion, and a retry boolean. Agents can match on the code, apply the fix, and decide whether to retry without parsing prose.
- **`?` operator for error propagation.** Unary postfix at the highest precedence alongside `()` and `.`. `content = fs.read(path)?` either binds the value or returns the error up the stack. No nested `if err != nil` chains.
- **Three execution modes from one source.** `codong eval` (AST interpreter, sub-second startup), `codong run` (Go IR &rarr; `go run`, 0.3–2s startup), `codong build` (Go IR &rarr; static native binary). Same `.cod` file, three deployment shapes.
- **Self-reported token savings.** The project's Arena benchmark (codong.org/arena) reports a 955-token Codong solution against 1,867 for Python, 1,710 for JavaScript, and 4,367 for Java on a Posts-CRUD task with Claude Sonnet 4 &mdash; a single workload measured by the project itself, not an independent study.

## Maturity.

Working compiler, MIT-licensed, v0.1.3 (28 March 2026) with four tagged releases since v0.1.0 first shipped on 24 March 2026. 92 commits, 67 stars, 7 forks. 1,427 tests across three suites (1,425 passing, 2 skipped for unconfigured MySQL/PostgreSQL environments). Written in Go (95.9% of source), with binaries published for Linux and macOS on amd64 and arm64; no Windows binary yet. v0.1.3 added a compilation cache the project reports as a "~170× speedup" on repeat runs. Single-author project (Brett, `brettinhere`); the repository's contributors list also shows a `claude` bot account, consistent with the project's "AI to write, humans to review" framing.

## Agent tooling.

`SPEC_FOR_AI.md` ships at the repo root &mdash; a structured Markdown spec (~6,000 words, ~1,600 lines, 20+ sections) with paired `// CORRECT` and `// WRONG` examples for every rule, designed for paste-in to any LLM system prompt. Structured JSON errors with `fix` and `retry` fields handle the repair-loop side. A `set_format("compact")` toggle produces single-line errors (`err_code:E_MATH|src:divide|fix:check divisor|retry:false`) for token-constrained agent contexts, with a project-reported ~39% token reduction in error output. An MCP server for Claude Desktop is listed as Stage 7 in the v0.1.3 Roadmap — planned, not yet shipped.

### Design DNA

- **NERD** *(Syntactic)* — Same camp, same diagnosis — choice paralysis burns tokens — opposite lever. NERD strips operators down to English keywords; Codong keeps conventional operators and collapses the standard library to one canonical function per task. Both self-report token-savings benchmarks from single-author runs.
- **Zero** *(Verification)* — Cross-camp foil sharing the 'one X way' design slogan. Zero buys obviousness with capability-typed effects, <code>raises</code> markers, and a typed <code>zero fix --plan --json</code> API inside a verification project; Codong buys it with a single canonical stdlib inside a pure syntactic project. Industrial-backing contrast: Vercel Labs vs single author.
- **Magpie** *(Syntactic)* — Same camp, opposite mechanism. Magpie surfaces SSA — every value <code>%</code>-prefixed, ~2.3× more tokens per operation — so the model has nowhere to be wrong; Codong keeps conventional surface but ships one canonical function per task so the model never has to choose. Magpie pays in tokens for unambiguity; Codong pays in stdlib scope.

*Detail page: https://agentlanguages.dev/languages/codong/  ·  Markdown companion: https://agentlanguages.dev/languages/codong.md*

## Laze

> Minimal indentation-based syntax with no punctuation. A Python bootstrap compiler emits C in memory and pipes it to cc -O2. Framed by its author as a weekend experiment in what an LLM produces when asked to design a language for itself.

**Camp:** Syntactic
**Author:** kerv
**Implementation language:** Python (bootstrap)
**Compilation target:** C (via gcc/clang)
**Licence:** Unknown
**First seen:** April 2026
**Maturity:** early implementation
**Site:** https://github.com/kerv/laze
**Repo:** https://github.com/kerv/laze

### Key idea

Laze is an indentation-based language with infix operators and no punctuation. The compiler is a single Python script (laze/lazec.py) that parses .laze files into an AST, generates C in memory without writing it to disk, and pipes the result to a C compiler. The bet is that LLMs are most accurate when emitting text-shaped, readable input; ergonomic syntax for the model is treated as more important than expressive power or efficiency at the language layer.

## What it is.

Laze is a weekend experiment, not a production tool. The README opens with the warning, verbatim: "This was just an experiment in which I asked Claude Opus 4.7 to create a programming language in the most efficient way it could." The surface is indentation-based, drops most punctuation, and uses infix operators. The compiler is a Python script that emits C internally &mdash; never written to disk &mdash; and pipes it to `cc -O2` for a native macOS binary. The repository (linked from the author's LinkedIn handle millerkev) contains four commits, two example files, and a single demonstration: `nes.laze`, a 2,000-plus-line NES emulator file covering a 6502 CPU, PPU sprites and scrolling, an APU, and mappers 0, 1, and 4. The author reports Super Mario Bros. as fully playable and Legend of Zelda playable with minor glitches.

## Why it's here.

The catalogue includes Laze as a marker of a position in the syntactic camp: optimise the surface for what an LLM finds easiest to produce, not for what a compiler can analyse. The thesis, stated in the README, is that an LLM specialises in text-shaped input because that is what it is trained on, so the right target is whichever syntax it generates most correctly and fastest. The catalogue does not rate Laze against working compilers shipped by larger teams. It marks it as a different kind of evidence: a single contributor's snapshot of what falls out when an LLM is allowed to design its own language and a human supplies only the prompt.

### Design DNA

- **Magpie** *(Syntactic)* — Same camp, opposite end of the same axis. Magpie chooses an explicit SSA surface to remove ambiguity; Laze strips punctuation and indentation rules to maximise the model's generation speed.
- **B-IR** *(Syntactic)* — Both are small individual explorations — a weekend's worth of letting an LLM design its own language and recording what came out.

*Detail page: https://agentlanguages.dev/languages/laze/  ·  Markdown companion: https://agentlanguages.dev/languages/laze.md*

## LLMLang

> Prefix-arity AST with single-character ASCII operators and De Bruijn variable indices. Linear ownership enforced at compile time. Compiler-injected OpenTelemetry spans triggered by a metadata marker. LLVM IR via Rust, OpenCL JIT for GPU map operations.

**Camp:** Syntactic
**Also spans:** Verification
**Author:** Paul Williams (paulprogrammer)
**Implementation language:** Rust
**Compilation target:** LLVM IR (then native via clang); OpenCL JIT for GPU map kernels at runtime
**Licence:** GPL-3.0 with Runtime Exception
**First seen:** May 2026
**Maturity:** working compiler
**Site:** https://github.com/paulprogrammer/llmlang
**Repo:** https://github.com/paulprogrammer/llmlang
**Agent tooling:**
- MCP server (llm-mcp binary, stdio transport)
- MCP tools: analyze_codebase, search_symbols, get_definition, get_diagnostics, find_callers, structural_search, patch_symbol
- MCP resources: llm://spec (LLM_SPEC.md), llm://agent-workflow (MCP_GUIDE.md)
- GEMINI.md (Gemini CLI orientation)
- Stable diagnostic codes (E000-E018, W001) catalogued in DIAGNOSTICS.md
- .llmi signature files for cross-module imports

### Key idea

LLMLang takes the token-efficiency move to its density extreme. Source
is a prefix-arity AST in single-character ASCII operators (`+`, `-`,
`>`, `$`, `~`, `?`, `.`, `#`); variables are De Bruijn indices
(`^0`, `^1`) rather than names; affine ownership is enforced at compile
time (move `>`, borrow `$`, mut-borrow `~`). The compiler ships
OpenTelemetry auto-instrumentation as a metadata marker
(`M "otel" "span_name" : func`) that injects span entry/exit and
timing around the function body, plus an OpenCL JIT that translates
pure `map` bodies to GPU kernels at runtime and falls back to CPU
vectorisation if OpenCL is absent.

## The thesis.

LLMLang takes the syntactic camp's premise &mdash; that the symbols an LLM emits cost tokens, so the language surface should minimise them &mdash; to its density extreme. The `LLM_SPEC.md` header is `[TOKEN_OPTIMIZED: HIGH_DENSITY]` and the design guide names the audience directly: "Target Audience: Large Language Models (LLMs). Non-Goal: Human readability." Source is a prefix-arity AST written in single-character ASCII operators: `+ 10 20` is addition, `> ^0` consumes the most-recent binding, `$ ^1` borrows the next-most-recent, `? cond t f` is a branch, `# Point x y` declares a struct-of-arrays shape, `: name args body` defines a function, `. e1 e2` sequences. There are no parentheses, no semicolons, no infix precedence to disambiguate. Variables are referenced by their De Bruijn index in the binding stack &mdash; `^0`, `^1`, `^2` &mdash; rather than by names; the parser also accepts named identifiers but resolves them to indices before the AST stores anything.

<p class="pullquote">"Target Audience: Large Language Models (LLMs). Non-Goal: Human readability."</p>

The distinctive move sits in two places at once. The first is the density lever: where NERD bets on English keywords because BPE tokenisers fragment punctuation, LLMLang bets the opposite &mdash; that single ASCII characters cost one token each in the right tokeniser and the win is biggest when there is no punctuation to fragment. The second is enforcement: affine ownership (`>` move, `$` borrow, `~` mut-borrow) is verified at compile time in `src/compiler/analysis/verify.rs`, with a `VariableState` stack that issues `E004` for use-after-move, `E005` for double-move, `E009` for branch-state mismatch, and `E016` for moving a borrowed variable. The same syntactic-camp surface ships a Rust-style borrow checker rather than relying on convention, which is why the entry spans into verification &mdash; the safety story is enforced, not advisory.

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

- **Prefix-arity AST in single-character ASCII.** `+ - * / > $ ~ ? . # : ^ ( ) & | =` cover binary math, ownership, branching, sequencing, shape declarations, definitions, De Bruijn lookup, and read/write handles. Short keywords (`sl`, `sc`, `ss`, `sf`, `sr`, `sp`, `jp`, `ju`, `map`, `flt`, `oe`) handle the rest. Compound expressions parse without parentheses because arity is fixed per operator.
- **De Bruijn variable references.** `^0` is the most-recent binding, `^1` the next-most-recent. The parser accepts named identifiers and resolves them to indices on the way in, but the AST stores only `Expr::DeBruijn(usize)`. Sample programs in `examples/` use the bare-index form directly (`: fact n ? ^0 * $ ^0 @ fact - > ^0 1 > ^0`).
- **Compile-time affine type checking.** `src/compiler/analysis/verify.rs` walks every function body with a `VariableState` stack tracking `Available`, `Borrowed`, and `Moved` per binding. `E004` fires on use-after-move, `E005` on double-move, `E009` when an `if`'s two branches leave the stack in different states, `E016` on moving a borrowed value. Unconsumed bindings emit `W001` and are auto-dropped at scope exit.
- **Compiler-injected OpenTelemetry spans.** Tagging a function with `M "otel" "span_name" : func` is recognised in `src/main.rs` and routed to a `gen_function` overload that wraps the body in calls to `llm_otel_enter_span`, `llm_get_time_ns`, `llm_otel_emit_span`, and `llm_otel_exit_span`. Nested tagged functions propagate trace context via thread-local storage in the C runtime; `OTEL_EXPORTER_OTLP_ENDPOINT` toggles between stdout JSON lines and HTTP POST. No other catalogue entry ships compiler-injected OpenTelemetry.
- **OpenCL JIT for `map` over SoA columns.** When a `map` is applied to a struct-of-arrays column with a pure function, `translate_to_opencl` in `src/compiler/codegen/expr.rs` synthesises an OpenCL `__kernel void map_kernel(...)` from the function body, and `src/runtime/driver_src/opencl_driver.c` `dlopen`s `libOpenCL.so` at runtime to compile and dispatch. Absent OpenCL, the runtime falls back to LLVM's loop and SLP vectorisers, with implicit parallelism hoisting pure subtrees above a complexity threshold into `parallel_task_N` functions dispatched through a work-stealing thread pool (`llm_fork`).
- **`.llmi` signature files for cross-module imports.** Compiling a module with `-o` generates a high-density header file listing exported symbols and shape definitions; the `I` operator reads it for downstream type and arity resolution. The `Money` primitive (`%+`, `%-`, `%*`, `%/` over 4-decimal fixed-point integers, `%str` formatting) and the Kubernetes Service Bindings integration in `src/runtime/db.c` (which reads `SERVICE_BINDING_ROOT` to assemble database connection strings from projected files) are part of the same "production backend primitives in the language" framing.

## Maturity.

`v0.4.0` at the time of cataloguing, sixteen tagged releases (`v0.1.0` to `v0.4.0`) cut between 18 and 24 May 2026 against a repository created 18 May 2026 &mdash; one feature wave per day for roughly a week, then consolidation commits through 27 May. Roughly 13,300 lines of Rust and C across 46 source files (`src/compiler/{lexer,parser,ast,analysis,codegen}` and a C runtime covering HTTP client and server with `picohttpparser`, TLS via `mbedtls`, `cJSON`, SQLite/Redis/MongoDB drivers, OpenCL dispatcher, MPSC emission queue, and a libtai-baseline temporal module); 31 self-hosted test programs under `tests/lang/` and 47 Rust unit tests in `tests/compiler_tests.rs`. GPLv3 with the `llmlang` Runtime Exception &mdash; a GCC-style carve-out that keeps the compiler copyleft but lets generated binaries link the runtime libraries into proprietary code without the licence propagating. Single author Paul Williams (`paulprogrammer`, Denver, Colorado, GitHub bio "Barefoot Coders"); 0 stars and 0 forks at time of cataloguing.

The README opens with the disclosure: "This entire repository has been largely vibecoded with humans acting as the product owners, and the LLM acting as the developer." That places LLMLang in the same factual family as AILANG's "written autonomously by AI agents" framing and Codong's "designed for AI to write, humans to review" position &mdash; what is shipped is real engineering with real automated tests, and the catalogue notes the authorship model as context rather than judgement. `MAYBE.md` separates roadmap from shipped: first-class AST manipulation beyond the existing `patch_symbol`, formal intent-and-contract metadata nodes, and TDD/BDD scenario nodes are not yet in the compiler, with OpenTelemetry already crossed off the list. The bet is the syntactic camp's bet intensified &mdash; that a surface compressed to single-character prefix operators with indexed variables, plus an MCP server that exposes the same AST the compiler sees, will produce more correct output per token than a conventional language plus a smarter model.

## Agent tooling.

The `llm-mcp` binary is the primary agent surface and ships as a second cargo target alongside the compiler. It exposes seven tools over stdio: `analyze_codebase` walks a directory and parses every `.llm` file into the same AST the compiler uses; `search_symbols` looks up functions and shapes by name; `get_definition` returns the realised AST and file location of a symbol; `get_diagnostics` runs the parser front-end against a file and returns `E00x`/`W00x` codes; `find_callers` traverses the call graph; `structural_search` computes a SHA-256 hash of the operator-and-control-flow shape of a function body (literals and names omitted) and returns other functions sharing the same fingerprint &mdash; an LLM can ask "what else does the same thing?" without relying on name similarity. `patch_symbol` accepts a JSON AST for a new function body, parses the source file, swaps the matching `Define` node's body, and rewrites the file through the compiler's own pretty-printer (`PrettyExpr` in `src/compiler/ast/display.rs`), so edits stay syntactically valid by construction. Two MCP resources back the tools: `llm://spec` embeds `LLM_SPEC.md` directly (the token-density grammar reference), and `llm://agent-workflow` embeds `MCP_GUIDE.md` (the analyse &rarr; locate &rarr; extract &rarr; patch workflow). Stable diagnostic codes (`E000`&ndash;`E018`, `W001`) are catalogued in `DIAGNOSTICS.md` so the same identifiers appear in compiler output, MCP responses, and the spec text the model receives from `llm://spec`.

### Design DNA

- **NERD** *(Syntactic)* — Closest editorial sibling on the token-efficiency axis, opposite lever. NERD swaps operators for English keywords (<code>plus</code>, <code>minus</code>, <code>eq</code>) on the bet that BPE tokenisers fragment punctuation; LLMLang collapses operators to single ASCII characters (<code>+</code>, <code>></code>, <code>$</code>, <code>~</code>) on the opposite bet that the right tokeniser maps each symbol to one token. Same camp, same diagnosis, opposite side of the symbol-vs-word spectrum.
- **Magpie** *(Syntactic)* — Same camp, more extreme densification. Magpie surfaces SSA with <code>%</code>-prefixed typed values and accepts ~2.3&times; more tokens per operation for unambiguity; LLMLang strips the surface further to prefix-arity with single-character operators and indexed variables, betting on density over explicitness. Both ship structured diagnostics with stable codes.
- **Vera** *(Verification)* — Cross-camp foil on De Bruijn indices. Vera uses typed slot references <code>@T.n</code> as a verification-camp move &mdash; the empirical case is that LLMs make naming errors faster than they make logic errors. LLMLang uses <code>^0</code>, <code>^1</code> as a syntactic-camp move &mdash; the case is that names cost tokens. Same mechanism, different camp.
- **Lumen** *(Orchestration)* — Also ships MCP integration but at different positioning. Lumen's <code>lumen-provider-mcp</code> is one provider crate among several (alongside HTTP, Gemini, custom-model providers) inside a human-facing orchestration language; LLMLang's <code>llm-mcp</code> binary is the primary agent surface and exposes structural-fingerprint search and a <code>patch_symbol</code> tool that rewrites source via the compiler's own pretty-printer.

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

Lume is an AI-first backend language transpiling to Go. The syntactic
move is the standard one — immutable by default, inferred types,
errors as values, one canonical way to express each operation — but
the distinctive move sits in the toolchain rather than the syntax:
`lume kb` builds a local Markdown knowledge base from the project's
own docs, examples, and diagnostic catalog, then `lume kb pack
"<question>" --ai --max-tokens N` scores the pages against the query
and assembles a token-budgeted context pack the host can paste
straight into an LLM prompt.

## The thesis.

Lume's diagnosis is the one most syntactic-camp projects start from: ambient choice in a conventional backend language wastes tokens and produces unpredictable LLM output. The README's framing sentence: "Immutable by default, designed for concise LLM-generated code, and currently implemented as an experimental compiler that transpiles `.lm` files to Go before invoking `go build`." The shipped subset implements that frame at the surface level — `=` rebinds rather than mutates, type annotations are optional and inferred, functions return their final expression, classes derive new values via `.with()` rather than field assignment, `switch` and `match` cover literal dispatch and pattern matching with exhaustiveness checks. The `docs/design.md` principles read as a syntactic-camp manifesto: "Tokens are a real cost. Syntax should be concise without becoming cryptic. Immutability is the default. Errors should become values, not hidden control flow. The language should prefer one canonical way to express common backend tasks."

<p class="pullquote">The goal is to avoid sending the same language reference, examples, diagnostics, and compiler notes to an LLM on every interaction.</p>

The distinctive move sits in the toolchain rather than the syntax. `lume kb pack "implement pipe" --ai --max-tokens 1200` tokenises the query, scores each page in a locally-built Markdown knowledge base by path-match and body-match weight, then assembles an "AI Context Pack" header plus the highest-scoring page bodies until the next page would exceed the budget. Codong ships `SPEC_FOR_AI.md` for whole-spec injection; Mog ships `docs/context.md` as a hand-curated compact reference; Lume ships a query-scoped extractor with a caller-set ceiling and treats the on-disk knowledge base as build output rather than maintained prose.

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

- **`lume kb pack` as the agent-facing surface.** A single CLI call (`lume kb pack "<question>" --ai --max-tokens N`) reads the on-disk knowledge base, scores its pages against the query, and prints a Markdown context pack the host can paste straight into an LLM prompt. The pack carries a header (`# Lume AI Context Pack`, `query:`, `budget:`, listed concepts, examples, error codes, source refs) followed by chunked page bodies, with the implementation in `internal/kb/kb.go` (~830 lines) tracking the running token estimate and stopping before the budget is breached.
- **Diagnostics as a structured catalog.** `internal/sema/diagnostics.go` ships 37 semantic diagnostics, each a `{code, feature, message, fix_hint}` record (`E2805` is "match expression is not exhaustive", with `fix_hint: "add case(_) or cover true/false for bool"`). `lume kb build` writes one Markdown page per code under `kb/errors/`, wikilinked from the relevant concept page; `lume kb pack` can pull error context by code into a budgeted pack.
- **Immutable bindings, no field mutation.** `=` introduces or rebinds a name; class fields and object fields cannot be reassigned in the current subset. `.with()` is the canonical way to derive a modified class value. The language reference is explicit: "Same-scope rebinding as a new value, not mutation."
- **Go transpilation as the v0 strategy.** `internal/codegen/golang.go` emits Go source; `internal/driver/driver.go` invokes `go build` for a native binary. `lume gen <file.lm>` prints the generated Go for debugging. The `docs/design.md` names this as deliberately interim: "This keeps the runtime, scheduler, garbage collector, and native binary story simple while the language surface is still changing."
- **Sequential `let` expressions.** `let(base = price * qty, fee = 2){ base + fee }` introduces multiple local bindings in order inside a scoped block, available as the function's final expression. One construct for both same-scope and nested local binding rather than two.

## Maturity.

`v0.1.0-experimental` at the time of cataloguing. The repository contains seven commits, all dated 16 May 2026, which together drop the entire project — CLI entry (`cmd/lume/main.go`), compiler (lexer, parser, AST, semantic checker, Go codegen, driver), knowledge-base internals (`internal/kb/kb.go`), 10 example programs under `examples/`, a four-document `docs/` set (language reference, compiler architecture, design notes, roadmap), a VS Code extension scaffold under `vscode/`, and the standard project files (CHANGELOG, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, LICENSE). Roughly 6,000 lines of Go in total; 69 test functions across `internal/parser`, `internal/sema`, `internal/driver`, and `internal/kb`. MIT-licensed. The single author is Marcelo Augusto Vilas Boas, who describes himself in his GitHub bio as "Tech Lead | Itaú Unibanco | 3x AWS | 1x Azure | MBA." 1 star, 0 forks, no tagged GitHub releases at time of cataloguing.

The README states what is shipped and what is not. The README's "Planned Language Ideas" section lists Hindley-Milner inference, ADTs and union pattern matching, `Result`/`Option` with `?` propagation, the pipe operator, modules, lambdas, effect annotations, refinement types, spec blocks, and a backend standard library as target-language ideas not yet in the compiler. The GitHub project description claims the syntax is "strict enough for the compiler to prove correctness"; what is actually shipped is conventional type-system soundness — name resolution, type compatibility, exhaustiveness checks on `match`, branch-type agreement on `if`/`switch`, list-element homogeneity, class-field validation — not refinement types or contract discharge. Refinement types are on the roadmap; on present evidence the entry sits in the syntactic camp without spanning into verification.

A separate "Lume" was announced as a manifesto on 25 May 2026 by David Brown (LinkedIn `dbrown01`, ex-TechnologyOne principal architect), with no public code, no GitHub presence, and no company entity at time of cataloguing. Mavboas's Lume predates that announcement by nine days and is the Lume with shipped code; the catalogue follows its standing convention of cataloguing whichever project ships code first under a given name. If David Brown's Lume later ships code, the catalogue will need to disambiguate the two; until then, "Lume" in this catalogue refers to mavboas/lume.

## Agent tooling.

`lume kb build` reads `docs/language.md`, `docs/compiler.md`, every `.lm` example, and the structured diagnostic catalog, and writes one Markdown page per concept (`kb/language/let.md`), per example (`kb/examples/let.lm.md`), and per error code (`kb/errors/E2805.md`), with `[[wikilink]]` cross-references and a top-level `kb/index.md`. `lume kb pack` scores pages by query terms, assembles a budgeted pack listing the included concepts, examples, error codes, and source refs, and stops before the next page would breach the cap. `lume kb lint` flags broken wikilinks and undocumented examples; `lume kb stats` reports raw versus packed token estimates. The repository ships no `SKILL.md`, `AGENTS.md`, `CLAUDE.md`, `llms.txt`, or MCP server at the project root — `lume kb` is the equivalent agent-facing surface, and the structured diagnostic catalog under `internal/sema/diagnostics.go` is the repair-loop substrate it pulls from.

### Design DNA

- **Codong** *(Syntactic)* — Closest editorial sibling on the 'one canonical way' bet. Codong collapses choice paralysis across a nine-module general-purpose stdlib with one canonical function per task; Lume applies the same diagnosis to a smaller backend-oriented surface and transpiles through Go in the same way. Codong ships <code>SPEC_FOR_AI.md</code> for system-prompt injection; Lume ships <code>lume kb pack</code> for query-scoped context extraction. Different mechanisms for the same context-budget concern.
- **Axis** *(Syntactic)* — Same wave (May 2026), same backend-DSL framing, opposite end of the syntactic-camp lever. Axis bounds the surface to twelve top-level constructs with an LL(1) grammar and ships per-state logit masks so 1B/3B/7B models can't emit invalid syntax; Lume keeps a conventional curly-brace surface and instead bounds the context the model sees via <code>lume kb pack --max-tokens N</code>.
- **NERD** *(Syntactic)* — Same camp, different lever. NERD swaps operators for English keywords on the bet that BPE tokenisers prefer words to symbols; Lume keeps conventional operators and bets that the bigger token win is in what context the model receives, not in how each operator is spelled.
- **Mog** *(Syntactic)* — Both small projects addressing the same context-budget concern from opposite directions. Mog fits its full spec in 3,200 tokens by designing the language under budget; Lume ships a larger surface and tools that extract a query-relevant subset under budget at call time. Mog also ships <code>docs/context.md</code> as a compact reference; Lume's equivalent is the generated <code>kb/</code> tree the CLI rebuilds from sources.

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

The textual program is already in SSA form. Every binding is %-prefixed and typed at definition, basic blocks are explicit, branches and ownership transitions are first-class operations. The bet is that removing surface ambiguity reduces LLM error rates more than added verification does.

## The thesis.

Magpie is a syntactic-camp project that takes the camp's premise to its logical end: don't add verification, remove ambiguity. The site states the goal directly &mdash; "Magpie eliminates ambiguity so LLMs can write perfect code on the first try" &mdash; and the language realises it by making the textual program identical to the compiler's intermediate representation. Every value is named at the point of definition with a `%`-prefixed identifier, typed inline, and assigned exactly once. Basic blocks are explicit (`bb0:`). Branches are explicit (`cbr`, `br`). Ownership transitions (`borrow`, `mutborrow`, `share`) are first-class operations. The premise is that the hidden semantics of conventional syntax &mdash; operator overloading, implicit conversions, invisible lifetime rules &mdash; are exactly the places LLMs hallucinate.

<p class="pullquote">~2.3× more tokens per operation, but eliminates the hidden rules that cause AI retries and borrow checker failures.</p>

The distinctive move shows up in the cross-camp comparison with Vera. Vera adds verification on top of conventional surface syntax (mandatory contracts, Z3 discharge, the `<Inference>` effect). Magpie strips the surface itself. Vera says "let the compiler catch what the model gets wrong." Magpie says "don't give the model anywhere to be wrong in the first place." NERD takes a similar diagnosis on the syntactic side but bets on minimal English-like tokens; Magpie bets on machine-style explicit SSA.

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

- **SSA as the surface.** The textual program is already in SSA: every binding is `%`-prefixed, typed, assigned exactly once. The parser doesn't construct an IR &mdash; the source is the IR.
- **Operations as records.** `i.add { lhs=%a, rhs=%b }` instead of `a + b`. Overflow behaviour, type coercion, and operand order are explicit in the syntax.
- **Explicit ownership operations.** `borrow.shared`, `mutborrow`, `share` are statements, not inferences. The borrow checker has nowhere to hide.
- **One way per concept.** Branching is `cbr` and `br`, full stop. The site reports a vocabulary-complexity ratio of 0.107 against Rust's 0.225 and TypeScript's 0.231.
- **Token cost made explicit.** The project publishes the trade: ~2.3× more tokens per operation, against fewer retry loops and borrow-checker failures.

## Maturity.

A Rust workspace at v0.1: 44 commits, 3 stars, 1 fork, no releases, MIT-licensed, attributed in the homepage footer to "© 2026 Magpie Language Developers." The crate set covers lexer, parser, semantic analysis, type checking, ownership checking, an MPIR lowering with a verifier, ARC insertion, and codegen paths for LLVM-text and WASM. Benchmarks published on the site report a 155&nbsp;ms compile time for the sample program against Rust's 234&nbsp;ms and TypeScript's 268&nbsp;ms, with execution speed matching Rust at 32&nbsp;ms and peak memory at 1.6&nbsp;MB against Rust's 1.4&nbsp;MB and TypeScript's 69.2&nbsp;MB. Diagnostics ship stable codes (`magpie explain MPT2014`) and JSON output (`--output json`/`jsonl`). The standard library is small; the surrounding ecosystem (LSP, registry, IDE plug-ins) doesn't exist yet. The bet is that a small, machine-shaped surface plus structured diagnostics will outperform conventional surface plus verification for first-try generation.

## Agent tooling.

The repository ships `SKILL.md` (a coding-and-diagnostic guide written for agents) and `AGENTS.md` alongside `DOCUMENTATION.md` and `DOCUMENTATION_QUICKSTART.md`. The CLI exposes `magpie mcp serve`, `magpie memory build`/`query`, and `magpie ctx pack` for agent workflows; `--output json` and `--output jsonl` modes emit structured diagnostics with stable codes. Token-efficiency claims live in `BENCHMARK.md`.

### Design DNA

- **Vera** *(Verification)* — Cross-camp foil. Vera adds a verification layer on top of conventional surface syntax; Magpie strips the surface itself. Vera bets on mechanical checks; Magpie bets on no place to be wrong.
- **NERD** *(Syntactic)* — Same syntactic camp, opposite tactic. NERD strips down to minimal English-like tokens; Magpie expands to explicit SSA. Both bet that one canonical shape beats many.
- **X07** *(Syntactic)* — Adjacent syntactic move. X07 ships canonical JSON x07AST and JSON Patch quickfixes; Magpie ships textual SSA. Different bets on what the canonical form should look like.

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

Mog is a statically typed, embedded-only language explicitly designed
for LLMs to write. The full spec fits in 3,200 tokens. Flat operators
mean a + b * c is a compile error; capabilities (fs, http, log) must
be declared in source with `requires` or `optional` and granted by
the host. There is no standard library — the host provides every
capability. Compiles to native via rqbe, a ~15,000-line safe-Rust
port of Quentin Carbonneaux's QBE backend. The first version was
authored by Voltropy's Volt coding agent in a single three-week
continuous session.

## The thesis.

Mog's diagnosis sits at the intersection of two camp moves: at the syntactic level, ambiguity is the enemy; at the verification level, ambient authority is the enemy. The flat-operators move makes `a + b * c` a compile error &mdash; every mix of operators requires parentheses, no precedence table to memorise. The capability move makes I/O a declaration: a `requires http, log;` line at the top of a Mog script declares what the host must provide; everything else is unreachable. The project's site frames it as "statically typed Lua, designed to be written by LLMs."

<p class="pullquote">The full spec fits in 3,200 tokens.</p>

The distinctive move is what Mog refuses to do. It is not standalone; it has no standard library; it ships only as an embedded language inside a host application that provides every capability. Compilation is in-process &mdash; no JIT, no interpreter overhead, no process startup cost; the frontend compiles a `.mog` file via rqbe (a safe-Rust port of QBE, ~15,000 lines) and produces a `.dylib` or `.so` the host can `dlopen`. The first version of Mog itself was authored by Voltropy's Volt coding agent in a single three-week continuous session, using Claude Opus 4.6, Kimi k2.5, and GLM-4.7 with Voltropy's lossless context management preserving working memory across compactions. This puts Mog in the same agent-authored cluster as AILANG.

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

- **Flat operators, no precedence.** Mixing different operators requires parentheses; `a + b * c` is a compile error. Non-associative operators (`-`, `/`, `%`, comparisons) cannot chain even with themselves &mdash; `a - b - c` is also rejected; `(a - b) - c` is fine.
- **Capability-based I/O.** A script declares `requires fs, http, log;` (or `optional` for graceful degradation). The host registers what it provides via `.mogdecl` declarations; the runtime refuses calls to anything unregistered. Authority is the host's to grant, not the script's to assume.
- **Embedded only.** Mog explicitly does not target standalone systems work. The README is direct: "Not standalone. Mog is always embedded in a host application. There is no standard library for file I/O or networking &mdash; the host provides everything." The orthogonality is the point.
- **Spec fits in 3,200 tokens.** `docs/context.md` is "compact reference designed to fit in an LLM's context window." A full language spec, deliberately bounded by token budget rather than feature count.
- **rqbe.** Quentin Carbonneaux's QBE backend (2016, ~10% the code of advanced compilers for ~70% the performance) ported to safe Rust at roughly 15,000 lines. The compiler runs in-process; the pipeline shells out only to the system assembler and linker.
- **Agent-authored at origin.** The first version was created by Voltropy's Volt coding agent in a single three-week continuous session. Lossless context management preserved working memory across compactions across Claude Opus 4.6, Kimi k2.5, and GLM-4.7.

## Maturity.

128 commits on main, no tagged release, 1,146+ compiler tests plus 186 rqbe tests passing. The 17-chapter guide on the site covers everything from basics through embedding APIs, capabilities, and tensors. The security model is candidly self-described as unaudited: "Mog has not been audited, and it is presented without security guarantees. It should be possible to secure it, but that work has not yet been done." Zero public stars at the time of cataloguing &mdash; like Boruna's initial state, this understates the surface area shipped: a working compiler, a safe-Rust port of QBE, a 17-chapter spec, a capability system, async/await via LLVM-style coroutine lowering.

## Agent tooling.

`docs/context.md` is the headline agent-facing surface &mdash; the compact reference designed for LLM consumption. The full `lang_spec.md` and a 755-line `showcase.mog` (demonstrating every language feature in a single file) accompany it. There are no SKILL.md, AGENTS.md, CLAUDE.md, MCP server, or llms.txt files in the repo; the bet is that a spec small enough to fit in the model's context is the right level of agent tooling, rather than a sprawl of orientation documents. Mog ships less than Vera or Boruna do for agent authors and gets away with it because the language itself is small.

### Design DNA

- **AILANG** *(Verification)* — Closest design relative on capabilities. AILANG carves IO/FS/Net/Clock/AI as row-polymorphic effects in the type system; Mog grants per-capability at the host via .mogdecl declarations. Both bet authority must be explicit in the surface. Both originated as agent-authored projects.
- **Zero** *(Verification)* — Sister project on the 'small, one canonical way' diagnosis. Zero pairs that with verification machinery and a structured-JSON CLI; Mog pairs it with capabilities and an in-process QBE backend. Both compile to native.
- **NanoLang** *(Verification)* — Cross-camp cousin in the syntactic+verification spanning region. NanoLang ships Coq proofs and mandatory tests; Mog ships capabilities and a 3,200-token spec. Different bets on what to make load-bearing in a small language.

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

NERD strips every symbolic operator out of the surface syntax and replaces it with an English keyword: `plus`, `minus`, `times`, `eq`, `gt`, `mod`. The author's claim is that BPE tokenisers fragment punctuation but treat common English words as single tokens, so the same logic is cheaper to generate. There is no type system, no braces, no semicolons; functions and side-effects (`http get`, `mcp use`, `llm claude`) are first-class statements.

## The thesis.

NERD treats the problem as token economics. The site states "40% of code is LLM-written. That number is growing," and the syntactic-camp move is to remove the symbols that fragment under BPE. Operators become English words (`plus`, `minus`, `eq`, `gt`, `mod`), braces and semicolons disappear, control flow ends with `done` rather than a brace. The README is explicit that humans are no longer the audience: "Machines write it. Machines read it. Humans observe it."

<p class="pullquote">The irony: cryptic symbols don't save tokens. Plain words win.</p>

The distinctive move is what NERD does *not* ship: no type system, no error union, no contracts, no checker beyond the parser. This is the syntactic camp at its purest &mdash; the bet is that smoothing the generation surface buys more than verification would, and the difference shows up in the inference bill. Magpie reaches a similar diagnosis through the opposite mechanism (SSA form, every value named and typed at definition); NERD picks the lower-effort lever and accepts that "audit" rather than "verify" is the only safety net.

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
- **Agent primitives in the grammar.** `llm claude "prompt"`, `mcp tools "url"`, `mcp use "url" "tool" "args"`, `http get "url" auth bearer "token"`. MCP and HTTP are statements, not library calls.
- **`llms.txt` published at the project root.** The site exposes a teaching corpus designed for an LLM to ingest the syntax in one fetch.
- **Self-reported token savings.** The author's four-function math benchmark reports 32 NERD tokens against 70 for JavaScript (54% saving), 96 for TypeScript (67% saving), and 273 for Java (80% saving) &mdash; a single workload by a single tokeniser, not an independent study.
- **C bootstrap to LLVM IR.** Lexer and parser in C, codegen to LLVM IR, then `clang` to native. No runtime; releases ship as single binaries for macOS Apple Silicon and static Linux x86_64.

## Maturity.

Working compiler, Apache-2.0, 135 stars and two contributors, 30 commits, five tagged releases (latest v0.1.4, Jan 2026). The README labels itself "🚧 Early days" and warns the implementation might change completely. Native binaries for macOS-arm64 and static Linux are checked into the repo alongside source.

## Agent tooling.

`llms.txt` is the primary surface, served from the site root for direct ingestion. The agent capabilities table marks fifteen MCP and HTTP operations as shipping today, plus a single-line `llm claude "..."` primitive that auto-loads `ANTHROPIC_API_KEY` from `.env`. The README is explicit that this is scaffolding to experiment with, not a production agent stack &mdash; OAuth 2.1 and SSE streaming are listed as "coming next."

### Design DNA

- **Magpie** *(Syntactic)* — Same diagnosis — strip ambiguity at the surface — opposite mechanism. Magpie surfaces SSA with %-prefixed names and one canonical operation per concept; NERD strips the operators and accepts a larger surface for shorter tokens.
- **Zero** *(Verification)* — Cross-camp foil. Zero also leans on keywords and 'one obvious way' but pairs that with a type checker and a structured-JSON CLI; NERD ships neither.
- **X07** *(Syntactic)* — Same camp, most extreme contrast. X07 walks past textual syntax to JSON ASTs; NERD keeps the text and economises the tokens inside it.

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

Two surface forms front a single AST. The dense SEV format encodes programs as single-character opcodes (P, D, L, R, C) and type tags (I, F, B, S); the SIRS JSON format mirrors the same AST in human-readable form. The README claims everything below the author's disclaimer is Claude-generated, including the 29-tool MCP server that integrates the model into the compilation loop.

## What it is.

Sever is not a project in the same sense as a working compiler. The README opens with a disclaimer from the GitHub account owner, Avital Tamir (software engineer at groundcover and creator of the Cyphernetes query language), stating that everything below it was generated by Claude and that the author makes no claim to the accuracy of any line of code, design decision, or assertion in the repository &mdash; including the README itself. The codebase registers as Zig per GitHub's language statistics. The artefacts on offer are a dense SEV opcode format (single-character opcodes P/D/L/R/C with type tags I/F/B/S), a SIRS JSON mirror of the same AST, and a Model Context Protocol server that reports 29 tools spanning compilation, AST manipulation, dependency analysis, and probabilistic distributions. Whether any of this compiles and runs as the README claims is something the author explicitly declines to vouch for.

## Why it's here.

The catalogue includes Sever as a marker of a recurring move in the syntactic camp: take token-density to its conclusion and see what the resulting artefact looks like. The result reads as conceptual art adjacent to engineering &mdash; a faithful record of what a frontier model produces when handed unlimited resources and a brief to design a programming language for itself. The catalogue does not rate Sever against working compilers. It marks it as a different kind of evidence: a snapshot of the design space when the model is the author and the human is the curator.

### Design DNA

- **X07** *(Syntactic)* — Both push representational density to an extreme — X07 replaces text with JSON ASTs edited via RFC 6902 patches; Sever collapses keywords into single-character opcodes.
- **B-IR** *(Syntactic)* — Catalogue-meta companions. Both are artefacts of the question what would an LLM-optimised language be, kept by their authors at arm's length from any claim of seriousness.

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
**Agent tooling:**
- AGENTS.md
- CLAUDE.md

### Key idea

Tacit treats human-oriented surface syntax as a lossy intermediate. The AST
is the authoritative artefact; every valid AST has exactly one canonical
text serialisation, definitions are content-addressed by the BLAKE3 hash of
that canonical form, variable references are DeBruijn indices, and parser
errors produce typed `Hole` nodes with structured diagnostics rather than
failing the parse. Effects are explicit in function signatures.

## The thesis.

Tacit's diagnosis is that text-based source forces models to maintain stylistic conventions, name choices, and whitespace that the compiler immediately discards. The project's response is to make the AST the canonical artefact and treat text as a derived view. The repository's README states the position directly: &ldquo;The AST is the source of truth. Tacit does not treat a human-oriented surface syntax as the authoritative program representation.&rdquo;

<p class="pullquote">If human readability is not the primary constraint, a language can optimise for three things at once.</p>

Two views render the same tree: a dense **authoring view** sized for model token budgets, and an **inspection view** layered for debugging and human review. Both round-trip losslessly through a JSON sidecar (`.tacd`). Canonical text is byte-exact &mdash; there is exactly one serialisation per AST, which eliminates formatter debates and makes hashing meaningful. Definitions are identified by the BLAKE3 hash of their canonical text; display names live in the sidecar and carry no semantic weight. Variable references use DeBruijn indices in canonical form. The cross-camp move is the verification-adjacent effect lattice: Tacit-Lite tracks `IO` / `Alloc` / `Mut` / `Div` in signatures, and unit boundaries must declare type and effect rows explicitly.

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

Imports name exact `blake3:<64-hex>` definition hashes, not paths or version ranges. Visibility (`public` / `package` / `private`) is part of the artefact. The display names `x`, `inc`, `double` are sidecar metadata; in canonical form references are DeBruijn indices.

## Distinctive moves.

- **AST as authoritative source.** The on-disk `.tac` file is the byte-exact canonical projection of the tree; comments and free-form formatting are not in the language. Names, field order, and type/effect hints live in a `.tacd` JSON sidecar.
- **Content-addressed definitions.** Imports resolve to BLAKE3 hashes, not names. Renaming a definition leaves its hash unchanged; changing its signature, body, or referenced hashes produces a new identity. Package manifests pin a hash-indexed local cache.
- **Typed `Hole` nodes instead of parse failures.** Malformed code reduces to a `Hole` node with a structured diagnostic and a type slot, so downstream tools can keep operating on partial programs (ADR 0040, landed in Phase 2).
- **Explicit effect rows on boundaries.** Tacit-Lite's fixed lattice is `IO` / `Alloc` / `Mut` / `Div`; effect signatures are mandatory at unit exports and inferred locally elsewhere. Tacit-Full (refinement types, capability-based security, handlers) is reserved as a roadmap, not shipped.
- **Toolchain pin as a first-class artefact.** `tacit init` writes a `tacit-toolchain.toml` that pins toolchain, primer, and bundled stdlib hashes; every package-aware command refuses to run on a mismatched pin and surfaces a `toolchain-pin-*` diagnostic.

## Maturity.

A Rust workspace (five crates: `tacit-canonical`, `tacit-views`, `tacit-typecheck`, `tacit-codegen`, `tacit-cli`) at v0.7.7, released 19 May 2026. Apache-2.0 / MIT dual-licensed; 237 commits, 3 stars, 2 forks at time of cataloguing. The decision log runs to ~90 ADRs; Phase 6 was frozen by [ADR 0089](https://github.com/weetster/tacit/blob/main/decisions/0089-phase-6-frozen.md) on 2026-05-17, closing modules/units, package manifests with hash-pinned lockfiles, package tests with stable `tacit-test-v1` JSON output, fixed-width integers with wrapping/checked/saturating arithmetic, typed mutable-memory handles, source-level stdlib packages (`tacit.core`, `.bytes`, `.array`, `.text`, `.collections`, `.io`), and a constrained host-interface ABI with generated C headers and Rust bindings. A Rust embedding demo links a Tacit kernel as a static library. Phase 7 is the next planned phase; debugger, diff/blame, IDE, public registry, and arbitrary FFI are explicitly out of scope until a later ADR. LLVM 19 is pinned via `inkwell`; published release artefacts target Linux x86_64 with a glibc 2.35 floor.

## Agent tooling.

`AGENTS.md` (1.7 KB) carries the Codex-facing sealed-corpus guardrail and a pointer to `CLAUDE.md`. `CLAUDE.md` (~20 KB) functions as a full development guide rather than a SKILL.md &mdash; it enumerates frozen artefacts, ground rules, the file-extension contract (`.tac` / `.tacd` / `.taca`), and the per-phase delivered surface. The toolchain ships its own primer: `tacit primer` prints the byte-pinned Tacit-Lite primer, and `tacit primer --search` / `--list-sections` / `--section <id>` support selective disclosure designed to fit a model's context window without flooding it. Diagnostics, package tests, and `tacit version` all emit stable JSON.

### Design DNA

- **Magpie** *(Syntactic)* — Closest neighbour in the same camp. Magpie surfaces SSA as the textual source; Tacit goes a step further and declares the text itself non-authoritative &mdash; the <code>.tac</code> file is a canonical projection of the AST, not the source. Both pay a token cost to strip ambiguity.
- **X07** *(Syntactic)* — Same direction along the &lsquo;text is lossy&rsquo; axis. X07 stores programs as canonical JSON ASTs and edits them with JSON Patch; Tacit stores them as canonical text projected from the AST, with BLAKE3-addressed identity. Different surfaces, same diagnosis.
- **Vera** *(Verification)* — Cross-camp foil on names. Vera abolishes parameter names entirely in favour of typed DeBruijn slots (<code>@Int.0</code>); Tacit keeps display names as sidecar metadata but uses DeBruijn indices in canonical form. Both treat names as a source of model error rather than a feature.
- **Mog** *(Syntactic)* — Adjacent on the embedding angle. Mog is a small embedded language with a capability system and a sub-3,200-token spec; Tacit ships a constrained host-interface ABI for a Rust host and explicitly defers capabilities to Tacit-Full. Different bets on whether capability tracking belongs in v1.

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
**Agent tooling:**
- AGENT.md
- Agent portal at /agent with versioned JSON entrypoints (manifest, schemas, skills, examples, stdlib, packages)
- x07-mcp (build MCP servers in X07)
- x07lang-mcp bridge (typed access to the toolchain over MCP)
- Per-release skills pack
- Stable error codes; quickfixes emitted as JSON Patch

### Key idea

X07's canonical source is not text. A program is an x07AST JSON document (`*.x07.json`) with a versioned schema; edits are RFC 6902 JSON Patch operations the toolchain applies mechanically. The toolchain emits stable error codes as structured JSON, paired with quickfixes the agent can apply with `x07 fix --write` or `x07 ast apply-patch`. Side effects live in explicit capability worlds; sandboxing is policy-driven.

## The thesis.

X07's diagnosis is that text-based source is exactly where agents lose. Whitespace becomes load-bearing, identical ASTs serialise differently, patches collide on formatting noise, and diagnostics are written for humans. The syntactic-camp move is to delete the text layer: the canonical source is `*.x07.json` (the x07AST), patches are RFC 6902 JSON Patch documents, diagnostics are JSON with stable error codes, and quickfixes are themselves JSON Patches the toolchain applies deterministically via `x07 fix --write` or `x07 ast apply-patch`.

<p class="pullquote">One canonical approach. No "should I use a for loop or map?" decisions.</p>

The distinctive move is the breadth of the machine-facing surface around the language. `x07lang.org/agent` publishes versioned JSON entrypoints &mdash; `manifest.json`, `schemas/index.json`, `skills/index.json`, `examples/catalog.json`, `stdlib/index.json`, `packages/index.json` &mdash; explicitly so agents consume them directly rather than scraping the HTML. Side effects are gated by named capability worlds (`run-os`, `run-os-sandboxed`); the official tooling includes an MCP kit (`x07-mcp`) for authoring MCP servers in X07 and a separate bridge (`x07lang-mcp`) for connecting external agents to the toolchain. The closest direction-of-travel is Magpie, which surfaces SSA at the source level; X07 deletes the source level.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre>{
  "schema_version": "x07.x07ast@0.4.0",
  "kind": "entry",
  "module_id": "main",
  "imports": ["std.bytes"],
  "decls": [],
  "solve": ["std.bytes.reverse", "input"]
}</pre>
  </div>
  <p class="caption">A program is a JSON document. A quickfix is an array of JSON Patch operations applied to this document; the agent never edits a string of source code, only the structural tree.</p>
</div>

## Distinctive moves.

- **AST as canonical source.** The on-disk artefact is the parsed tree, not text. Schemas are versioned (`x07.x07ast@0.4.0`).
- **JSON Patch as the edit primitive.** Repairs are structural diffs that apply mechanically; whitespace cannot break a patch because there is no whitespace.
- **Capability worlds for effects.** Programs run in deterministic solve worlds or named OS worlds (`run-os`, `run-os-sandboxed`); ambient access is not available by default.
- **Stable diagnostic codes with deterministic quickfixes.** `x07 lint` emits `x07diag` JSON with a stable code and an optional JSON Patch fix; `x07 fix --write` applies it.
- **Versioned agent portal.** Each toolchain release ships an `/agent/v<version>/` directory of machine entrypoints alongside the human docs.
- **Performance claims with a published comparison repo.** The README points at `x07lang/x07-perf-compare` (v0.0.3 snapshot) reporting native-code parity with C/Rust on the included workloads and faster compile times; the methodology lives in that separate repository.

## Maturity.

The repo (`x07lang/x07`) is a multi-crate Rust workspace dual-licensed Apache-2.0/MIT with 471 commits and 108 tagged releases (latest GitHub tag v0.1.49, Mar 2026); the documentation site lists further point releases above that as the docs source of truth (latest `0.2.10`). The `/agent` portal advertises 14 skills, 258 schemas, 17 examples, 410 packages, and 19 stdlib modules &mdash; a documentation surface larger than most catalogue entries. The observable community signal lags that surface sharply: 7 stars, 0 forks, 0 open issues at time of cataloguing, and the README does not name the author. The bet is that the canonical-artefact-plus-JSON-Patch model attracts agent integrators on its own merits; the gap between the published toolchain depth and the visible user base is the entry's defining quality and worth flagging.

## Agent tooling.

`AGENT.md` at the repo root is the human-readable orientation; everything else is structured. The `/agent` portal publishes stable JSON entrypoints per-version (`entrypoints.json`, `manifest.json`, `schemas/index.json`, `skills/index.json`, `examples/index.json`, `stdlib/index.json`, `packages/index.json`). The toolchain ships an MCP kit (`x07-mcp`) for authoring MCP servers in X07 and a bridge (`x07lang-mcp`) for connecting agent runtimes to the toolchain itself. The canonical loop is `x07 init → x07 lint → x07 fix → x07 run → x07 test`, every stage emitting structured JSON.

### Design DNA

- **Magpie** *(Syntactic)* — Same-camp cousin pointing the same way. Magpie surfaces SSA but keeps source text; X07 walks past text and treats the AST itself as the canonical artefact. Same direction, further along the axis.
- **Sever** *(Syntactic)* — Both push the surface as far from human authoring as the camp allows; Sever crams meaning into single characters, X07 abandons characters as the unit altogether.
- **NERD** *(Syntactic)* — Opposite end of the same camp. NERD keeps text and economises tokens inside it; X07 declares text a lossy intermediate and edits the tree directly.

*Detail page: https://agentlanguages.dev/languages/x07/  ·  Markdown companion: https://agentlanguages.dev/languages/x07.md*

---

# Verification camp (11)

> The model doesn't need to be right. It needs to be checkable. The verification camp accepts that LLMs will keep making semantic errors and asks a different question: can the compiler catch them? Their answer is mandatory contracts, refinement types, effect systems, and SMT-backed proofs — the machinery of formal methods, repurposed as a guardrail for generative code.

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
**Agent tooling:**
- SKILL.md
- AGENTS.md
- CLAUDE.md
- MCP server
- llms.txt
- Claude Code plugin
- Gemini CLI extension
- slash commands

### Key idea

AILANG ships a purely functional, effect-typed substrate for AI-generated
code. The type system is Hindley-Milner with row polymorphism; the effect
system carves authority into capability categories (IO, FS, Net, Clock, AI)
that must be granted at the CLI with --caps. There are no loops — the
language commits to lambda calculus, pattern matching, and ADTs as the
only forms of control. The compiler itself is written autonomously by AI
agents via a coordinator.

## The thesis.

AILANG takes the verification camp's diagnosis and applies it at the layer of authority. The diagnosis is that LLMs hallucinate side effects &mdash; network calls in pure functions, filesystem writes in helpers that look read-only, model calls in code paths the operator never approved. AILANG's answer is to carve effects into capability categories &mdash; `IO`, `FS`, `Net`, `Clock`, `AI` &mdash; and make every one of them visible in the function's signature, row-polymorphically, with Hindley-Milner inference filling in the rest. A function that doesn't declare an effect can't perform it; a run that wasn't given a capability at the CLI can't grant it from inside.

<p class="pullquote">For humans, a language is a tool for expression. For AIs, it's a substrate for reasoning.</p>

The distinctive move is the no-loops decision. AILANG commits to lambda calculus, pattern matching, and ADTs as its only forms of control flow &mdash; no `for`, no `while`, no mutable accumulator. Where Vera tracks model calls as a single `<Inference>` effect, AILANG splits the world into five capability categories and refuses to let the language grow a construct that obscures any of them. The bet is that determinism, replay, and structured per-effect traces are worth giving up the loop.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">module</span> examples/hello

<span class="kw">import</span> std/io (println)

<span class="kw">export</span> <span class="kw">func</span> <span class="ty">main</span>() <span class="op">-&gt;</span> () ! {<span class="ct">IO</span>} {
  <span class="ty">println</span>(<span class="str">"Hello from AILANG!"</span>)
}</pre>
  </div>
  <p class="caption">The <code>! {IO}</code> after the return type is the effect row. A caller without an <code>IO</code> capability granted via <code>ailang run --caps IO</code> cannot invoke this function. Effect rows compose: a function that calls <code>IO</code>- and <code>FS</code>-effecting helpers must declare <code>{IO, FS}</code>.</p>
</div>

## Distinctive moves.

- **Capability carving, not capability tracking.** Effects are partitioned into `IO`, `FS`, `Net`, `Clock`, `AI`. Each is granted (or refused) separately at the CLI with `--caps`. The model can't widen authority from inside the program.
- **No loops.** Lambda calculus, recursion, and pattern matching only. The language has a dedicated "Why No Loops?" reference page; the design axioms treat the absence of mutable iteration as load-bearing for replay.
- **Row-polymorphic Hindley-Milner.** Effect rows are first-class type-level objects, inferred and unified the same way row-typed records are. A helper that doesn't touch the network has a smaller row than its caller.
- **Written by agents, end-to-end.** The README declares the language is "written autonomously by AI agents via its own coordinator"; static-analysis and supply-chain badges (Sonar, OpenSSF Scorecard, OpenSSF Best Practices) are cited as third-party verification of the output.
- **MCP-first developer surface.** A hosted MCP server at `mcp.ailang.sunholo.com` ships typed tools over stdlib, examples, and benchmarks. The Claude Code plugin and Gemini CLI extension install the compiler, the prompt, and the MCP server in one command.

## Maturity.

v0.20.1 with 110 published releases on GitHub, Apache-2.0 licensed, 2,958 commits, 26 stars. The compiler is implemented in Go (85.5% of the source) and ships native binaries for macOS (Intel and Apple Silicon) and Linux plus a WebAssembly target used by nine in-browser demos. Standard library covers `std/io`, `std/fs`, `std/json`, `std/zip`, `std/xml`, `std/crypto`, `std/http`, `std/net`. The benchmark dashboard runs 33 tasks across 8 frontier models in three modes &mdash; zero-shot, self-repair, and full agentic &mdash; on every release.

The bet is that the rest of the catalogue's verification entries are designing a language a human reads and an AI writes, while AILANG is designing a language an AI both writes and maintains. The next test is whether the agent-authored development model produces a standard library deep enough to compete with MoonBit's roughly two-year head start (MoonBit launched 18 August 2023).

## Agent tooling.

`SKILL.md`, `AGENTS.md`, and `CLAUDE.md` ship in the repository; `llms.txt` and `llms-full.txt` are served from the docs site. A remote MCP server exposes typed tools for stdlib lookup, examples, design docs, and the benchmark dashboard. The `ailang_bootstrap` plugin installs slash commands (`/ailang:prompt`, `/ailang:new`, `/ailang:run`, `/ailang:challenge`) into Claude Code and the equivalent extension into Gemini CLI; both download a platform-matched compiler binary on install. The CLI emits structured per-effect traces designed for the agent's next iteration to act on.

### Design DNA

- **Vera** *(Verification)* — Vera tracks LLM inference as one &lt;Inference&gt; effect; AILANG carves authority into IO, FS, Net, Clock, AI as separate capability categories granted per run.
- **Boruna** *(Orchestration)* — Both build capability-based effect systems; Boruna enforces declared effects at the VM, AILANG enforces them at the type system and the CLI capability flag.
- **MoonBit** *(Verification)* — Both ship effect typing on a functional core; MoonBit's effects are conventional and general-purpose, AILANG's are row-polymorphic and carved for agent-relevant authority.

### Timeline

- **Sep 2025** — First public release on GitHub under Apache-2.0.
- **Jan 2026** — AILANG reaches v0.6.2; Mark Edmondson publishes the language framing on dev.to.
- **May 2026** — v0.20.1 ships. 110 releases, 2,958 commits. 33-benchmark dashboard runs across 8 frontier models on every release. MCP server, Claude Code plugin, and Gemini CLI extension all in production.

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
**Agent tooling:**
- CLAUDE.md
- llms.txt

### Key idea

Co-locate intent, effects, and verification with the function body. Every function carries a prose intent (?), an effect declaration (!), and a colocated verify block. Pure verify blocks export as Lean 4 theorems and Dafny lemmas; effectful ones lift through Oracle, which quantifies over bounded effect parameters in the exported theorem.

## The thesis.

Aver is a verification-camp project that names its target audience explicitly: the reviewer, not the generator. Every function carries a prose intent (`? "..."`), an effect declaration (`! [Console.print]`), and a colocated `verify` block of `expression => expected` cases. Pure verify blocks export as Lean 4 theorems or Dafny lemmas through `aver proof --backend lean|dafny`; effectful functions get the same treatment via Oracle, which lifts classified effects (`Random`, `Http`, `Disk`, `Time`, `Console.readLine`, ...) into proof artefacts as explicit function parameters typed with bounded subtypes (`RandomFloatInUnit`). The exported theorem quantifies over every possible such function, not just the one the test stub provided. Architectural choices are first-class syntax: `decision UseResultNotExceptions { chosen = "Result", rejected = ["Exceptions"], ... }`.

<p class="pullquote">Code is cheap to generate. Expensive to trust.</p>

The distinctive move shows up in the comparison with Vera. The two share design DNA &mdash; mandatory verification artefacts, explicit effects, no `if`/`else`, no closures, no exceptions, no nulls, no loops &mdash; but disagree on what to drop. Vera drops names entirely via De Bruijn slot references (`@Int.0`). Aver keeps names and makes the surrounding metadata mandatory. Vera's bet is that names are the failure mode; Aver's is that absence of intent is.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">fn</span> safeDivide(a: <span class="ty">Int</span>, b: <span class="ty">Int</span>) -&gt; <span class="ty">Result</span>&lt;<span class="ty">Int</span>, <span class="ty">String</span>&gt;
    <span class="cm">? "Safe integer division. Returns Err on zero."</span>
    <span class="kw">match</span> b
        <span class="num">0</span> -&gt; <span class="ct">Result.Err</span>(<span class="str">"Division by zero"</span>)
        _ -&gt; <span class="ct">Result.Ok</span>(a / b)

<span class="kw">verify</span> safeDivide
    safeDivide(<span class="num">10</span>, <span class="num">2</span>) =&gt; <span class="ct">Result.Ok</span>(<span class="num">5</span>)
    safeDivide(<span class="num">7</span>, <span class="num">0</span>)  =&gt; <span class="ct">Result.Err</span>(<span class="str">"Division by zero"</span>)</pre>
  </div>
  <p class="caption">Prose intent (?), no if/else, and a colocated verify block. The function ships its specification.</p>
</div>

## Distinctive moves.

- **Mandatory intent.** Every function carries a `?` prose description directly after the signature. Functions with effects but no description warn.
- **Effects as type signatures.** `! [Http.get]` declares a specific capability; `! [Http]` covers the namespace. Violations are type errors, with `aver.toml` constraining which hosts and paths are reachable.
- **Verify, then prove.** The same `verify` block runs as sampled cases (`aver verify`), adversarial-profile checks (`aver verify --hostile`), or as a Lean 4 / Dafny export (`aver proof`). The four readings can disagree on identical source &mdash; the Oracle page walks one through.
- **Oracle for effectful code.** Classified effects are lifted into proof artefacts via `BranchPath` and per-branch counters; the theorem quantifies over them universally.
- **`aver context` for agents.** Token-budgeted export of types, effects, and intents (`--budget 10kb`), designed to fit an LLM window.
- **Decisions as syntax.** `decision` blocks make ADRs queryable from the codebase, not from a wiki.

## Maturity.

v0.21 on crates.io (`cargo install aver-lang`), MIT-licensed, written in Rust, primary author `jasisz`. Three backends: a bytecode VM, native Rust codegen, and a standalone WASM-GC target (additionally lowered to wasip2 / WASI 0.2 Component Model for server-side deployment) &mdash; the site demonstrates the latter with seven games compiled directly to WebAssembly GC, including Snake at 4.3&nbsp;KiB and a roguelike at 25.6&nbsp;KiB on Chrome 119+/Firefox 120+/Safari 18.2+. Proof export targets Lean 4 (via `lake build`) and Dafny. The toolchain surface is wide and functional. The bet is that the same source can serve as implementation and reviewable specification, with proof export as the upper-bound check.

## Agent tooling.

The site publishes `llms.txt` at averlang.dev/llms.txt &mdash; a long-form crib sheet covering syntax, the `=>` separator (vs `=`), constructor qualification (`Result.Ok`, never bare `Ok`), and a numbered list of the most common LLM mistakes. `CLAUDE.md` and a `.claude/skills/` directory ship in the repo. The `aver context` command exports a token-budgeted slice of the codebase. Diagnostics ship with structured hints (`Hint: add ! [Console.print]`); the playground renders the same diagnostics live.

### Design DNA

- **Vera** *(Verification)* — Closest design relative. Both ship mandatory verification artefacts, explicit effects, no if/else, no exceptions. Vera drops variable names entirely (@Int.0); Aver keeps names and makes the surrounding metadata mandatory. Aver became the first non-Python/TS baseline integrated into VeraBench, described in the bench README as &lsquo;a Haskell-inspired language with zero LLM training data.&rsquo;
- **Prove** *(Verification)* — Same camp, opposite politics. Both ship contracts and explicit effects; Aver ships an llms.txt and welcomes AI authoring, Prove ships an anti-training licence that prohibits training use of source.
- **Pact** *(Verification)* — Adjacent design DNA. Both treat intent and effects as declarations on every function (Aver's ? and ![Effect]; Pact's intent and needs db). Aver pushes proof export; Pact pushes built-in SQLite and HTTP.

### Timeline

- **Apr 2026** — First external language integrated into VeraBench as a baseline alongside Python and TypeScript &mdash; described in the bench README as &lsquo;a Haskell-inspired language with zero LLM training data, providing a second data point alongside Vera for the zero-training-data thesis.&rsquo;
- **May 2026** — v0.21 published on crates.io. Oracle proof export to Lean 4 and Dafny stabilises. Seven games shipped to WASM-GC (Snake at 4.3&nbsp;KiB, Doom at 20.4&nbsp;KiB, roguelike at 25.6&nbsp;KiB) on Chrome 119+/Firefox 120+/Safari 18.2+.

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

BHC and hx are a single editorial position more than two projects.
Schneider argues, across a public trilogy on his blog raskell.io, that
Haskell's types-as-proofs and pure-by-default already give LLMs the
formal scaffolding they need; the reason Haskell loses to procedural
languages in agent benchmarks is the surrounding toolchain — fragmented
build tools, slow compiles, one-size-fits-all runtime. The work under
the arcanist.sh organisation is the engineering response: hx (a Rust
binary wrapping GHC, Cabal, GHCup, and HLS behind one interface) and
BHC (an in-development Haskell 2026 compiler with multiple runtime
profiles).

## The thesis.

The thesis is that the verification camp has the right diagnosis but the wrong locus of intervention. In Schneider's public writing, declarative languages with strong type systems already play to what LLMs are good at: generating expressions that satisfy formal constraints, rather than simulating execution across many mutable steps. Type-checked Haskell looks like a proof; the compiler is the proof checker; once the types align, large classes of error are eliminated by construction. The reason this is not the dominant agent-coding stack today, Schneider argues, is friction outside the language &mdash; three overlapping build tools, slow cold builds, a runtime that assumes one performance profile fits every use case.

<p class="pullquote">The language was right. The surrounding infrastructure was not.</p>

The distinctive move is the refusal to design a new language at all. Where AILANG, Vera, and Aver each ship a fresh syntax with effect typing built in, BHC and hx extend Haskell. The engineering lives under the arcanist-sh GitHub organisation: hx, a Rust binary that wraps GHC, Cabal, GHCup, and HLS behind one interface; and BHC, an in-development clean-slate compiler targeting the Haskell 2026 specification with profile-specific runtimes selected at compile time. The bet is that the typed substrate is already correct and the missing layer is operational, not linguistic.

## Distinctive moves.

- **The position, stated.** A trilogy on the author's blog at raskell.io (&ldquo;The Last Programming Language Might Not Be for Humans&rdquo;, &ldquo;What Comes After the Last Programming Language&rdquo;, &ldquo;Source Code Is the New Assembly&rdquo;) makes the case that the medium-term winner is declarative-plus-typed, not procedural-plus-checked.
- **hx wraps before it replaces.** Cargo-workspace Rust binary built from ~14 crates (`hx-cli`, `hx-core`, `hx-toolchain`, `hx-cabal`, `hx-solver`, `hx-lsp`, `hx-plugins`, …); commands cover `build`, `run`, `test`, `lock`, `sync`, `watch`, `fmt`, `lint`, `docs`, `publish`; lockfile is `hx.lock`. The repo describes the strategy as wrap GHC/Cabal/GHCup/HLS first, replace last.
- **BHC targets multiple runtime profiles.** The BHC repository README lists four profiles today &mdash; default, server, numeric, edge &mdash; selected at compile time. Schneider's essays and the arcanist-sh organisation profile describe a planned six (adding `realtime` and `embedded`); the catalogue treats the four shipped in the repo as ground truth and the additional two as stated intent.
- **Conservative scope per release.** Both projects are pre-1.0. The arcanist-sh org profile advertises a 5.6× cold-build speedup over Cabal, but no methodology, benchmark suite, GHC/Cabal versions, or hardware are published anywhere reachable; treat the number as a stated marketing claim, not a verified measurement.
- **No agent-specific surface yet.** No SKILL.md, AGENTS.md, MCP server, or `llms.txt` in either repo. The argument is that a well-typed Haskell program already gives an agent what it needs; tooling for agents is downstream.

## Maturity.

Early. hx is MIT-licensed Rust, at v0.6.0 (Feb 2026), with 12 tagged releases, 129 commits, and 23 stars; it currently orchestrates GHC/Cabal/GHCup/HLS rather than replacing them. BHC is BSD-3-Clause, at v0.2.1 (Jan 2026), with 389 commits, 3 releases, 11 stars, and a single contributor. The roadmap in the BHC README marks the parser, type checker, Core IR, and one codegen path as substantially complete and WASM/GPU lowerings as in progress; no conformance suite or benchmark numbers ship in the repository today. The bet is on a multi-year arc, and the public surface reflects that &mdash; essays and infrastructure now, language-level claims later.

## Agent tooling.

None shipped at present. The position Schneider defends is that the right intervention is upstream of agent-specific files: a faster, more coherent build, a compiler whose error messages and runtime profile match the deployment target, and a type system the agent can already use as a proof obligation. Whether that bet pays off depends on whether the medium-term arc Schneider describes &mdash; declarative-plus-typed beats procedural-plus-checked once the tooling friction is gone &mdash; actually materialises before agent-native languages with built-in MCP surfaces lock in a different shape.

### Design DNA

- **AILANG** *(Verification)* — Closest design relative. Both bet on purely functional, effect-typed code as the right shape for agents to author. AILANG designed a new language; BHC/hx argues the language is already fine and rebuilds the tooling around Haskell.
- **MoonBit** *(Verification)* — Industrial-backing foil. MoonBit pairs a sampler-level verification story with three years of training data and a Shenzhen-funded team; BHC/hx is a one-person Swiss effort betting that better tooling around an established language beats a new language with a new ecosystem.
- **Vera** *(Verification)* — Adjacent rather than competing. Vera is the bespoke agent-native route; BHC/hx bets that Haskell's purity and semantic density make AI-written, verifiable compute feel natural once toolchain friction is removed. Schneider's essays cite Vera by name as the canonical example of the language-design route, and frame BHC/hx as a complementary route through tooling.

*Detail page: https://agentlanguages.dev/languages/bhc-hx/  ·  Markdown companion: https://agentlanguages.dev/languages/bhc-hx.md*

## Intent

> Mandatory preconditions, postconditions, and entity invariants. Z3 SMT verification via intentc verify. Natural-language intent blocks that resolve to specific contract references. One source file compiles to Rust, JavaScript, and WebAssembly.

**Camp:** Verification
**Author:** lhaig
**Implementation language:** Go
**Compilation target:** Native binaries (via Rust), JavaScript, WebAssembly (direct binary)
**Licence:** Apache-2.0
**First seen:** February 2026
**Maturity:** working compiler
**Site:** https://github.com/lhaig/intent
**Repo:** https://github.com/lhaig/intent
**Agent tooling:**
- AGENTS.md
- CLAUDE.md
- INTENT.md

### Key idea

Every function carries `requires`/`ensures`; every entity carries an
`invariant`; loops carry `invariant` and `decreases`. An `intent` block
links natural-language goals to specific contract references via
`verified_by`, and the compiler refuses to resolve a reference that has no
matching contract. `intentc verify` discharges what it can to Z3; what
remains runs as enforced runtime checks in Rust (panic), JavaScript
(throw), or WebAssembly (trap), all from the same source file.

## The thesis.

Intent's premise is that humans audit contracts, not implementations. The repository's framing makes the position explicit: &ldquo;Humans audit contracts, not implementations. When you generate Intent code, the human reads your `requires`, `ensures`, `invariant`, and `intent` blocks to verify correctness.&rdquo; The compiler enforces structural consistency between the natural-language declarations and the machine-checkable ones &mdash; every `verified_by` reference must resolve to an actual `requires`, `ensures`, or `invariant` clause, or the program fails to compile.

<p class="pullquote">The contract system is the product. The implementation is secondary.</p>

The distinctive move is the **intent block** itself: a named natural-language goal paired with a list of `verified_by` references that name specific contracts in the codebase (`BankAccount.invariant`, `BankAccount.deposit.requires`, `BankAccount.withdraw.ensures`). The compiler resolves each reference during semantic analysis and emits an error on a dangling one; published intent blocks therefore cannot drift from the contracts they cite. Z3 discharges what it can statically via `intentc verify`; the rest becomes runtime enforcement that fires identically across all three targets &mdash; a `requires` failure panics the Rust binary, throws an `Error` in JavaScript, and traps in WebAssembly.

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

`old(...)` captures pre-mutation state for `ensures` clauses. `forall i in 0..n: p` and `exists i in 0..n: p` quantify over integer ranges in contracts. Loops carry `invariant` and `decreases` clauses for inductive reasoning at verification time.

## Distinctive moves.

- **Mandatory contracts at three levels.** Functions carry `requires`/`ensures`; entities carry `invariant`; loops carry `invariant`/`decreases`. The grammar reserves the slots; the type checker enforces that `verified_by` references resolve.
- **Intent blocks as compiled artefacts.** A `verified_by` path (`Entity.member.clause` or `function.clause`) is resolved by the semantic checker, not by convention. Unresolved references are compile errors, which prevents prose-level drift between stated goals and machine-checkable contracts.
- **Z3 as an optional static layer, runtime checks as the floor.** `intentc verify` translates IR contracts to SMT-LIB and invokes Z3; results are reported per contract as `verified` / `unverified` / `error` / `timeout`. When Z3 is absent, the compiler degrades gracefully and the runtime asserts still run.
- **Rust as IR, not as the only backend.** An explicit IR layer (~30 node types, contracts as first-class IR nodes, `OldCapture`/`OldRef` as explicit pre-state) feeds three sibling backends: a Rust generator that hands off to `cargo`, a direct JavaScript emitter, and a direct WebAssembly binary emitter that does not require the Rust toolchain. Each enforces the same contracts at runtime.
- **Property-based test generation from contracts.** `intentc test-gen` derives property-based tests from `requires`/`ensures`, complementing &mdash; not replacing &mdash; the SMT discharge.

## Maturity.

A Go workspace at v0.2.0 (released 16 February 2026) with 45 commits and 5 stars at time of cataloguing. The roadmap (`docs/ROADMAP.md`) records milestones 1&ndash;6 as complete: usable language surface (loops, arrays, enums, pattern matching, `Result`/`Option`, try operator, multi-file imports), the Z3 verifier (`internal/verify/`), and three working backends (Rust via `internal/rustbe/`, JavaScript via `internal/jsbe/`, direct WASM via `internal/wasmbe/`). The `docs/DESIGN.md` specification runs to 1,764 lines and notes that traits, generics, async/await, and a package manager with semver constraints have all landed since the POC. The CLI surface &mdash; `intentc build / check / verify / fmt / lint / test-gen`, plus `intentc pkg init / add / remove / install` &mdash; is shippable; a four-target showcase (CLI binary, browser dashboard, Node server, browser WASM at 155 bytes) runs against unmodified compiler output. Last commit was 25 March 2026; an LSP, REPL, and release-mode contract stripping sit on the milestone-8 roadmap and are not yet built.

## Agent tooling.

Three documents target agent authors directly. `AGENTS.md` (~18 KB) is the Codex/general-agent orientation. `CLAUDE.md` is the Claude-specific working guide. `INTENT.md` (~26 KB) is the language reference written as agent instructions &mdash; it opens &ldquo;You are generating code in Intent&rdquo; and ends with ten explicit guidelines for AI code generation, including &ldquo;Write contracts first&rdquo; and &ldquo;Every function should have contracts.&rdquo; `docs/REPRODUCE.md` documents reproducing the compiler with the agent of the reader's choice. Diagnostics are textual rather than structured JSON; the LSP that would expose them programmatically is on the roadmap rather than shipped.

### Design DNA

- **Vera** *(Verification)* — Closest design relative. Both ship mandatory contracts on every function, both use Z3, both treat the agent as the primary author. Vera abolishes parameter names via typed DeBruijn slots and tracks LLM inference as a first-class <code>&lt;Inference&gt;</code> effect; Intent keeps names and concentrates its novelty in intent blocks that bind natural-language goals to verified contract references.
- **Aver** *(Verification)* — Same camp, different proof story. Aver exports <code>verify</code> blocks as Lean 4 theorems or Dafny lemmas through <code>aver proof</code>, lifting effectful code into proof artefacts via Oracle; Intent commits to Z3 SMT with runtime enforcement on every backend. Same diagnosis, different upper-bound check.
- **MoonBit** *(Verification)* — Closest sibling on compilation strategy. MoonBit ships four backends (WASM GC, JavaScript, native via C codegen, LLVM) on an OCaml-implemented compiler; Intent ships three (Rust, JavaScript, WASM) on a Go-implemented one. MoonBit's edge is years of training data; Intent's framing is auditability over breadth.
- **Prove** *(Verification)* — Same contract machinery, opposite politics. Prove ships refinement types and verb-based IO under the Prove Source License v1.0, which prohibits use as AI training data; Intent ships under Apache-2.0 and addresses its agent-instruction documents to the model directly.

*Detail page: https://agentlanguages.dev/languages/intent/  ·  Markdown companion: https://agentlanguages.dev/languages/intent.md*

## MoonBit

> AI-friendly general-purpose language. ICSE 2024 paper on real-time semantics-aware token sampling. Three years of training data.

**Camp:** Verification
**Author:** Hongbo Zhang / IDEA Shenzhen
**Implementation language:** OCaml
**Compilation target:** WASM GC, JavaScript, native (C codegen), LLVM
**Licence:** Unknown
**First seen:** January 2023
**Maturity:** working compiler
**Site:** https://www.moonbitlang.com
**Agent tooling:**
- `moon doc` AI symbol lookup
- MoonBit Pilot coding agent
- `declare` keyword for AI-native specification

### Key idea

AI-friendly general-purpose language with the deepest history in the
space — three years of training data, full toolchain across four
backends, a package registry (mooncakes.io), cloud IDE, and IDE plugins.
Published an ICSE 2024 paper on a real-time semantics-aware token
sampler. Backed by the International Digital Economy Academy (Shenzhen).

## The thesis.

MoonBit is the catalogue's exception that proves the rule. Most entries are recent (Jan–May 2026); MoonBit has been shipping since 2023. Most are single-author or small-team experiments; MoonBit is backed by the International Digital Economy Academy in Shenzhen and led by Hongbo Zhang, who created ReScript and contributed to OCaml. Most ship a thought experiment or an early implementation; MoonBit ships four backends, a package registry, a cloud IDE, and two IDE plug-ins.

<p class="pullquote">The model doesn't need to be retrained. The sampler needs to know the type system.</p>

The distinctive technical move is in how the model interacts with the compiler. The ICSE 2024 paper describes a real-time semantics-aware token sampler: as the model generates code, a fast type-checker prunes ill-typed continuations at the token level. The model can still hallucinate, but the hallucinations never get past the sampler. This is closer to the verification camp's "make it checkable" intuition than the syntactic camp's "make it easier to generate" — applied at the layer where the generation actually happens.

## Distinctive moves.

- **Real-time semantics-aware sampling.** The compiler participates in token generation, not just post-hoc checking.
- **`declare` keyword.** A first-class form for AI-native specification of intent and constraints, distinct from regular function signatures.
- **Four backends.** WASM GC, JavaScript, native (via C codegen), and LLVM. No other entry in the catalogue targets this breadth.
- **mooncakes.io.** A first-party package registry. Most catalogued languages don't have one because there's no ecosystem yet; MoonBit has the ecosystem.
- **Three years of training data.** The unmatched advantage. Every other entry is racing to generate examples; MoonBit has them.

## Maturity.

The most mature project in the catalogue by a clear margin. 2,115+ stars (the second-highest after Zero). Full toolchain, multiple backends in active production use, IDE integrations across both major desktop IDEs, working debugger. Documentation depth and developer experience are at a level no other entry approaches.

The pragmatic question is whether MoonBit's general-purpose framing remains compelling against narrower agent-native languages as the field matures. MoonBit's bet is that general-purpose plus AI-aware tooling beats agent-native plus narrow ecosystem. The next two years will test it.

## Agent tooling.

`moon doc` exposes AI-friendly symbol lookup; MoonBit Pilot is a coding agent that targets MoonBit specifically; the `declare` keyword gives agents a structured way to express intent. Less prominent than the SKILL.md/AGENTS.md pattern other catalogue entries use — MoonBit's bet is that an agent that knows the language outperforms an agent reading instructions about the language.

### Design DNA

- **Vera** *(Verification)* — Both verification camp; opposite breadth. MoonBit is a full-stack general-purpose language; Vera narrows to checkability and drops names entirely. MoonBit assumes the model needs help; Vera assumes the model needs supervision.
- **Zero** *(Verification)* — Closest in industrial backing (Vercel Labs vs IDEA Shenzhen) and product framing. Zero leans syntactic (one obvious way to express things); MoonBit leans toward typed sampling at the model level.
- **AILANG** *(Verification)* — Both ship effect typing; MoonBit's is conventional, AILANG's is row-polymorphic with capability-based carving (IO/FS/Net/Clock/AI). MoonBit's edge is the training data depth that no other entry has.

### Timeline

- **2023** — MoonBit project initiated at IDEA Shenzhen under Hongbo Zhang. Pre-LLM-craze; framing changes over the following two years.
- **Jan 2024** — ICSE 2024 paper on real-time semantics-aware token sampling for MoonBit code generation.
- **2024–2025** — Toolchain hardens: four backends (WASM GC, JavaScript, native via C codegen, LLVM), package registry (mooncakes.io), cloud IDE, VS Code and IntelliJ plugins, debugger.
- **2026** — <code>declare</code> keyword and MoonBit Pilot agent ship, repositioning the language explicitly as AI-native rather than just AI-friendly.

*Detail page: https://agentlanguages.dev/languages/moonbit/  ·  Markdown companion: https://agentlanguages.dev/languages/moonbit.md*

## NanoLang

> Mandatory shadow test blocks on every function. The proved core (NanoCore) has 193 Coq theorems with zero axioms. Multi-target codegen across C, WebAssembly, LLVM IR, PTX, and RISC-V.

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
**Agent tooling:**
- AGENTS.md
- CLAUDE.md
- MEMORY.md ("training reference for patterns and idioms")
- spec.json (machine-readable formal specification)
- .mcp.json
- .claude/, .cursor/, .factory/ config folders
- VS Code extension with LSP (nanolang-lsp) and DAP (nanolang-dap)
- Web playground with CodeMirror-6, share permalinks, live evaluation

### Key idea

Mandatory shadow test blocks on every function (the compiler refuses
to compile without one) plus 6,170 lines of Coq proving the language's
core. NanoCore is the proved subset, not the entire surface language —
algebraic effects, async/await, FFI, the VM, and multi-target codegen
sit outside the proof set. The first-person README persona ("I refuse
to compile a function unless you provide a shadow test block") is
documented as deliberate design, not flourish.

## The thesis.

NanoLang takes the verification camp's diagnosis literally: if LLMs are going to write code, the language should refuse to accept their work without tests, and the language's core should have proofs to back its promises. Every function declaration must be paired with a `shadow` test block; the compiler rejects functions without one. The proved core (NanoCore) has 6,170 lines of Coq across 9 files, 193 theorems and lemmas, zero axioms, zero `Admitted` &mdash; the proofs cover preservation, progress, determinism, semantic equivalence of big-step and small-step, and evaluator soundness against a fuel-based reference interpreter extractable to OCaml. The README announces it in first person: "I refuse to compile a function unless you provide a shadow test block for it."

<p class="pullquote">"I am a minimal programming language designed for machines to write and humans to read. I require tests, I use unambiguous syntax, and my core is formally proved."</p>

The distinctive move is the depth of what is actually proved &mdash; and the honesty about what is not. NanoCore covers integers, booleans, strings, arithmetic, conditionals, mutable variables, while loops, lambda/application, arrays, records, recursive functions, variants, and pattern matching. Algebraic effects, async/await, FFI, the VM, and the multi-target codegen sit outside the proof set, and `formal/README.md` says so plainly. Vera proves contracts at the function level via Z3; NanoLang proves the type system itself, from below. Same camp, complementary layers.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">fn</span> <span class="ty">greet</span>(name: <span class="ty">string</span>) -&gt; <span class="ty">string</span> {
  <span class="kw">return</span> (+ <span class="str">"Hello, "</span> name)
}

<span class="kw">shadow</span> greet {
  <span class="ct">assert</span> (== (greet <span class="str">"World"</span>) <span class="str">"Hello, World"</span>)
}

<span class="kw">fn</span> <span class="ty">main</span>() -&gt; <span class="ty">int</span> {
  (<span class="ty">println</span> (greet <span class="str">"World"</span>))
  <span class="kw">return</span> <span class="num">0</span>
}

<span class="kw">shadow</span> main { <span class="ct">assert</span> <span class="kw">true</span> }</pre>
  </div>
  <p class="caption">Every function needs a shadow block. <code>shadow main { assert true }</code> exists only because the compiler refuses to compile without it — and the trivial case still has to satisfy the discipline.</p>
</div>

## Distinctive moves.

- **Mandatory shadow tests.** No function compiles without a `shadow` block. The smallest legal program contains `shadow main { assert true }` &mdash; the discipline applies to the trivial case alongside the substantive one.
- **Coq proofs, zero axioms.** 193 theorems and lemmas across `Syntax.v`, `Semantics.v`, `Typing.v`, `Soundness.v`, `Progress.v`, `Determinism.v`, `Equivalence.v`, `EvalFn.v`, and `Extract.v`. No `Admitted` lemmas; the proofs go through. Built on Rocq Prover (Coq) &ge; 9.0.
- **NanoCore is the proved subset, not the whole language.** The proved fragment is honest about its scope. Effects, async, FFI, the VM, and codegen are out-of-scope for the formal work and the project documents that directly.
- **NanoISA VM with co-process FFI.** Stack-based VM, 178 opcodes, reference-counted GC. External calls run in a separate co-process (`nano_cop`); if they crash, the VM survives. Trap model separates computation from I/O.
- **Multi-target codegen.** Default target is C transpilation. Also `--target wasm` (with source-map sidecar and Ed25519 signing), `llvm`, `ptx`, `riscv`. Production parity is claimed for C and WebAssembly; the other backends are present in the toolchain.
- **Dual notation, prefix and infix.** `(+ a b)` and `a + b` are both legal. The prefix form is described as unambiguous and is the form the formal semantics is stated against.
- **First-person persona.** README, diagnostics, and `AGENTS.md` instruct agents to write in NanoLang's voice. Documented under `docs/PERSONA.md` as a design choice, not a quirk.

## Maturity.

v3.3.7 (April 2026), 51 tagged releases, ~2,156 commits, bootstrap 100% (the compiler compiles itself). Apache-2.0. Hardware support: Ubuntu 22.04+ on x86_64 and ARM64, macOS 14+ Apple Silicon, FreeBSD; Windows via WSL2 only. The author is Jordan Hubbard &mdash; co-founder of FreeBSD in 1993, currently Senior Director for GPU Compute Software at Nvidia &mdash; and the project's GitHub topics include `thought-exercise` and `vibe-coding`, applied by the author himself. The README's "Totally True and Not At All Embellished History" notes the language has "been used in production by exactly one person, who also wrote it." That candour matches the catalogue's tone: the engineering is real, the framing is honest about what the engineering is for.

## Agent tooling.

The repository root ships `AGENTS.md`, `CLAUDE.md`, `MEMORY.md` (self-described as "my training reference for patterns and idioms"), `spec.json` (machine-readable formal specification), `.mcp.json`, and config folders for Claude, Cursor, and Factory. The IDE surface includes a Language Server (`bin/nanolang-lsp`) and Debug Adapter (`bin/nanolang-dap`) plus a VS Code extension; the web playground supports share permalinks and live evaluation. The agent-facing surface is wider than most catalogue entries &mdash; NanoLang ships not just orientation files but its own corpus.

### Design DNA

- **Vera** *(Verification)* — Same camp, different layer of the same idea. Vera proves contracts at the function level via Z3; NanoLang proves the language's core type system itself, from below, via Coq. Vera's <code>&lt;Inference&gt;</code> effect has no NanoLang analogue.
- **Aver** *(Verification)* — Same-camp neighbour on the ship-the-verification-artefacts axis. Aver exports per-function proofs to Lean 4 and Dafny; NanoLang ships its proofs alongside the source as Coq, zero axioms, 193 theorems.
- **Magpie** *(Syntactic)* — Cross-camp foil on the syntactic axis. Magpie strips ambiguity via SSA-as-surface; NanoLang reduces it via prefix-call disambiguation, mandatory annotations, and one canonical form. Different mechanisms for the same diagnosis.
- **NERD** *(Syntactic)* — Syntactic-camp direction without the formalism. NERD ships a token-friendly surface and no type system; NanoLang ships Coq proofs. The contrast clarifies what 'unambiguous syntax' costs to back up.

*Detail page: https://agentlanguages.dev/languages/nanolang/  ·  Markdown companion: https://agentlanguages.dev/languages/nanolang.md*

## Pact

> Intent blocks on every function and route, pipeline syntax, explicit effects, errors as types. Single binary that ships an HTTP server, SQLite, an LSP, and an MCP server with five tools.

**Camp:** Verification
**Author:** Viktor Kikot
**Implementation language:** Rust
**Compilation target:** Interpreted (tree-walking)
**Licence:** MIT
**First seen:** April 2026
**Maturity:** working compiler
**Site:** https://github.com/KikotVit/pact-lang
**Repo:** https://github.com/KikotVit/pact-lang
**Agent tooling:**
- CLAUDE.md
- .mcp.json
- Built-in MCP server (pact mcp) with 5 tools
- LSP server (pact lsp)
- Built-in docs (pact docs)

### Key idea

Every function and route opens with an `intent` clause and a `needs` list declaring effects. Errors are part of the type signature (`-> User or NotFound`), data flows through left-to-right pipelines, and the runtime ships HTTP, SSE, SQLite, JWT, and an MCP server inside a single ~5MB binary. The bet is that surfacing intent, effects, and outcomes at the signature level lets agents skip the reverse-engineering pass.

## The thesis.

Pact's diagnosis is that most backend code is glue, and that the glue is exactly where agents waste iterations. Intent is hidden in comments that drift, effects are hidden in implementation bodies, and errors are hidden in exception hierarchies. The verification-camp move is to drag all three back into the signature: every function and every route opens with an `intent` string, declares a `needs` list of effects, and resolves to a sum type like `User or NotFound`. The compiler reads that as the contract; the type checker, formatter, LSP, and MCP server all consume the same declarations.

<p class="pullquote">Every function says why it exists. Errors are data, not explosions.</p>

The distinctive move is the breadth of what ships inside one binary. A `.pact` file declares `app Notes { port: 8080, db: "sqlite://notes.db" }` and `pact run` brings up an HTTP server with SSE streaming, SQLite in WAL mode, JWT auth, a structured logger, and a built-in MCP server &mdash; no dependencies, no ORM, tables auto-created from struct fields. This is close to Aver in design DNA (declared intent + declared effects + colocated checks), but where Aver lifts verify blocks into Lean 4 and Dafny, Pact spends its complexity budget on the runtime an agent will actually drive.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre>intent "create a new user with default Viewer role"
fn create_user(data: NewUser) -&gt; User or BadRequest
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
}</pre>
  </div>
  <p class="caption">An intent line, an effect list, and a sum-typed return &mdash; all in the signature before the body.</p>
</div>

## Distinctive moves.

- **Intent in the signature.** Every `fn` and `route` carries a one-line `intent` string read by the agent before the body. The author argues this lets a model skip the "reverse-engineer purpose from implementation" step.
- **Effects in the signature.** `needs db, time, rng, auth, log, env, http` declares side effects up front. Tests swap them deterministically: `using time = time.fixed(...)`, `using db = db.memory()`.
- **Errors as types.** Sum types replace exceptions; `| on NotFound: respond 404 with ...` handles each variant; Rust-style `?` propagates.
- **Pipelines as the default control flow.** `data | filter where .x > 0 | sort by .name | take first 10` is the canonical shape for data transforms and route handlers alike.
- **One binary, one runtime.** ~5MB Rust binary bundles lexer, parser, tree-walking interpreter, HTTP server, SSE, SQLite, JWT, HTTP client, LSP, MCP server, formatter, and docs.

## Maturity.

Single-author project, MIT-licensed, currently at v0.5 with six tagged releases (latest v0.3.1, Apr 2026), 204 commits and 496+ tests on the master branch. The README is explicit that it works for small APIs and CRUD services and is not production-ready; the web playground is the next planned milestone. Stars and forks are at zero, which understates the surface area shipped &mdash; deep type checker, formatter, LSP, MCP server, VS Code extension, install script for macOS/Linux, and a Docker image are all in the tree today.

## Agent tooling.

`CLAUDE.md` and a checked-in `.mcp.json` orient Claude Code at the project level. `pact mcp` exposes five tools over stdio JSON-RPC: `pact_run`, `pact_check`, `pact_docs`, `pact_format`, `pact_test`. `pact lsp` provides diagnostics, hover, and autocomplete for any LSP-capable editor. Documentation is queryable from the CLI (`pact docs <topic>`) so an agent can pull a topic and a working example before generating code.

### Design DNA

- **Aver** *(Verification)* — Closest design relative. Both attach declared intent and effects to every function; Aver lifts the verify block into Lean 4 and Dafny exports, while Pact keeps the surface lighter and ships a working web stack.
- **Vera** *(Verification)* — Vera's requires/ensures clauses are the strict cousin of Pact's intent blocks. Vera mechanically discharges them via Z3; Pact treats the intent as documentation the type checker and MCP server consume.
- **Boruna** *(Orchestration)* — Another single-author Rust project where the engineering depth runs well ahead of the public profile. Both ship MCP servers as the agent-facing surface.

*Detail page: https://agentlanguages.dev/languages/pact/  ·  Markdown companion: https://agentlanguages.dev/languages/pact.md*

## Prove

> Intent-first language with verb-based IO, refinement types, and contracts. Source is covered by the Prove Source License v1.0, which prohibits use as AI training data.

**Camp:** Verification
**Author:** Magnus Knutas
**Implementation language:** Python (bootstrap)
**Compilation target:** C (then native via gcc/clang)
**Licence:** Prove Source License v1.0 (language & .prv source) / Apache-2.0 (tooling)
**First seen:** February 2026
**Maturity:** working compiler
**Site:** https://prove.botwork.se

### Key idea

Verbs (transforms, validates, derives, creates, matches; inputs, outputs, dispatches; attached, detached, listens, renders) encode intent and IO category in the function declaration. The compiler enforces verb semantics, refinement-type constraints, and ensures/requires/explain contracts. The Prove Source License v1.0 covers all .prv source and prohibits AI training use, dataset inclusion, embedding, and synthetic data generation.

## The thesis.

Prove diagnoses the same problem the rest of the verification camp diagnoses &mdash; AI-generated code is cheap to produce and expensive to trust &mdash; and adopts the same general moves: intent-first declarations, hard postconditions (`ensures`), refinement types, no `if`/`else`, errors-as-values, no nulls. Every function carries a verb (`transforms`, `validates`, `derives`, `creates`, `matches` for pure code; `inputs`, `outputs`, `dispatches`, `streams` for IO; `attached`, `detached`, `listens`, `renders` for structured concurrency). The verb is enforced: a `transforms` function cannot call IO functions (diagnostics E361&ndash;E363); `explain` blocks document the chain of operations in controlled natural language, parsed and verified against called functions' contracts.

<p class="pullquote">Source code is covered by an anti-training licence.</p>

Where Prove diverges from Vera and Aver is on the politics rather than the mechanics. Vera publishes a benchmark and invites models to compete. Aver exports proofs and ships an `llms.txt`. Prove ships an *anti-training* licence &mdash; the Prove Source License v1.0, applied automatically by `proof new` to every project &mdash; that prohibits use of `.prv` source code as training data, in dataset inclusion, vector stores, RAG indices, embedding databases, synthetic data generation, sublicensing for AI use, and downstream propagation. AILANG sits closest on effect typing (both ship typed effects), but Prove encodes effect category in the verb itself rather than in a row-polymorphic effect list. The project frames its own stance as "AI resistance" and states that generating semantically correct Prove code "requires genuine understanding, not pattern matching."

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">matches</span> apply_discount(discount <span class="ty">Discount</span>, amount <span class="ty">Price</span>) <span class="ty">Price</span>
  <span class="ct">ensures</span> result &gt;= <span class="num">0</span>
  <span class="ct">ensures</span> result &lt;= amount
  <span class="ct">requires</span> amount &gt;= <span class="num">0</span>
<span class="kw">from</span>
    <span class="ct">FlatOff</span>(off) =&gt; max(<span class="num">0</span>, amount - off)
    <span class="ct">PercentOff</span>(rate) =&gt; amount * (<span class="num">1</span> - rate)</pre>
  </div>
  <p class="caption">A pure verb (matches), hard postconditions, and a precondition &mdash; all enforced at compile time.</p>
</div>

## Distinctive moves.

- **Verbs as intent, enforced.** Each function declares its purpose with a verb. Pure verbs cannot perform IO. The same name can have multiple verbs and the compiler resolves which to call from context.
- **Anti-training licence.** The Prove Source License v1.0 covers the language, its specification, and all `.prv` source. The compiler tooling (Python bootstrap, docs, editor integrations) is separately Apache-2.0; the project publishes its reasoning for the split under "AI Transparency."
- **Refinement types.** `type Port is Integer:[16 Unsigned] where 1..65535`. Constraints are part of the type, validated at compile time, and used to drive auto-generated edge cases.
- **`explain` against contracts.** Controlled natural language documents the implementation; the compiler parses each row, checks that operations match called functions' behaviours, and rejects explanations whose references aren't real identifiers.
- **Refutation challenges.** `proof check` runs by default and generates plausible-but-wrong mutations of the function body, requiring the author to address each with a `why_not` annotation.
- **Functional iteration only.** `map`, `filter`, `reduce` &mdash; no loops. Errors propagate with `!`.

## Maturity.

v1.3.1 (April 2026), tracked through a clear release history: v1.0.0 (first stable, 22-module standard library, C codegen, region-based memory, 13-pass optimiser, ML-powered LSP), v1.1.0 (March 2026, structured concurrency + GUI + `proof` CLI), v1.2.0 (March 2026, verb consistency overhaul across 22 stdlib modules), v1.3.0/v1.3.1 (April 2026, tree-sitter as sole parser, `dispatches` verb). Source is hosted on a self-hosted Gitea instance at code.botwork.se rather than GitHub. Author: Magnus Knutas. Bootstrap compiler is Python 3.11+; output language is C. The roadmap names v2.0 as a self-hosted compiler. The bet is that the same intent-first mechanism that resists external AI generation is also the substrate for the project's "local, self-contained" generation model &mdash; a deterministic toolchain that produces code from the project's own declarations.

## Agent tooling.

None of the catalogue's usual surface: no `SKILL.md`, no `AGENTS.md`, no `llms.txt`, no MCP server. The licence actively prohibits the dominant tooling pattern. The project ships editor integrations instead &mdash; `tree-sitter-prove` for syntax highlighting, `pygments-prove` for MkDocs/Sphinx, `chroma-lexer-prove` for Gitea/Hugo &mdash; and a single-file installer (`curl -sSf install.sh | sh`) that places a `proof` binary in `~/.local/bin/`. The roadmap lists binary AST format, semantic normalisation, fragmented source, and identity-bound compilation as post-1.0 anti-training features.

### Design DNA

- **Vera** *(Verification)* — Same diagnosis, opposite politics. Both ship mandatory contracts and explicit effects; Vera publishes a benchmark and invites models to compete, Prove ships an anti-training licence that prohibits training use of source.
- **Aver** *(Verification)* — Camp neighbour with proof export. Both ship intent-first design and contract-style verification; Aver exports to Lean 4 / Dafny and ships llms.txt, Prove ships the anti-training licence and self-hosted Gitea.
- **AILANG** *(Verification)* — Both ship effect-typed designs. AILANG carves effects via row polymorphism (IO/FS/Net/Clock/AI); Prove encodes IO category in the verb itself (inputs/outputs/dispatches vs transforms/validates/derives/creates/matches).

### Timeline

- **Feb 2026** — v1.0.0 first stable release. 22-module standard library, intent-driven compiler (verb enforcement, contracts, refinement types), C code generation with region-based memory and a 13-pass optimiser, ML-powered LSP.
- **Mar 2026** — v1.1.0 ships structured concurrency (<code>attached</code>, <code>detached</code>, <code>listens</code>, <code>renders</code> backed by stackful coroutines), terminal UI, GUI via SDL2 + Nuklear, and the <code>proof</code> CLI wrapper.
- **Mar 2026** — v1.2.0 enforces verb semantic guarantees across 22 stdlib modules (~105 corrections); recursive variant types and Value&lt;T&gt; phantom types land.
- **Apr 2026** — v1.3.0 makes tree-sitter the sole parser, renames <code>reads</code> to <code>derives</code>, adds the <code>dispatches</code> verb, integrates linting into the check pipeline. v1.3.1 is a bugfix release.

*Detail page: https://agentlanguages.dev/languages/prove/  ·  Markdown companion: https://agentlanguages.dev/languages/prove.md*

## Vera

> Mandatory contracts on every function. Z3 SMT verification. Typed slot references replace variable names. LLM inference is a first-class typed effect.

**Camp:** Verification
**Also spans:** Orchestration
**Author:** Alasdair Allan
**Implementation language:** Python
**Compilation target:** WebAssembly
**Licence:** MIT
**First seen:** February 2026
**Maturity:** working compiler
**Site:** https://veralang.dev
**Repo:** https://github.com/aallan/vera
**vera-bench:** https://github.com/aallan/vera-bench
**Agent tooling:**
- SKILL.md
- AGENTS.md
- CLAUDE.md
- LLM-oriented diagnostics
- stable error codes (E001–E702)
- JSON diagnostic output

### Key idea

Mandatory requires/ensures/effects contracts on every function. Three-tier
Z3 SMT verification. Typed De Bruijn slot references (@T.n) instead of
variable names — the only language in the space that drops names. LLM
inference is a first-class typed effect. The thesis: the model doesn't
need to be right, it needs to be checkable.

## The thesis.

Vera takes the verification camp's diagnosis literally. If LLMs make semantic errors faster than humans can catch them by reading code, the compiler has to do the catching. Every function declares preconditions and postconditions; the compiler discharges them via the Z3 SMT solver in a three-tier scheme (compile-time, runtime guard, runtime check) before any code runs.

<p class="pullquote">The model doesn't need to be right. It needs to be checkable.</p>

The distinctive move is replacing variable names with typed slot references. A function `safe_divide(@Int, @Int -> @Int)` has no parameter names — its arguments are referred to as `@Int.0` (most recent) and `@Int.1` (next most recent) using De Bruijn indexing. The empirical literature shows models are particularly vulnerable to naming-related errors: choosing misleading names, reusing names incorrectly, losing track of which name refers to which value. Vera's answer is to remove names from the language entirely.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">public</span> <span class="kw">fn</span> safe_divide(<span class="sl">@Int</span>, <span class="sl">@Int</span> -&gt; <span class="sl">@Int</span>)
  <span class="ct">requires</span>(<span class="sl">@Int.1</span> != <span class="num">0</span>)
  <span class="ct">ensures</span>(<span class="sl">@Int.result</span> == <span class="sl">@Int.0</span> / <span class="sl">@Int.1</span>)
  <span class="ct">effects</span>(pure)
{
  <span class="sl">@Int.0</span> / <span class="sl">@Int.1</span>
}</pre>
  </div>
  <p class="caption">Division by zero is not a runtime error — it is a type error. A caller that can't prove the denominator is non-zero won't compile. <code>@Int.1</code> is the first parameter (next-most-recent binding); <code>@Int.0</code> is the second (most-recent).</p>
</div>

## Distinctive moves.

- **Mandatory contracts.** Every function carries requires/ensures/effects clauses. There's no opt-out; the grammar rejects functions without them.
- **De Bruijn slot references.** No variable names at the parameter level. `@T.n` denotes the *n*-th-most-recent binding of type `T`.
- **Typed effects, including inference.** LLM calls are an `<Inference>` effect. A function that doesn't declare it can't make model calls. The effect system tracks model usage up the call graph.
- **Three-tier verification.** Some contracts discharge at compile time via Z3; some become runtime guards; some become runtime checks. The tier is determined by which fragment of arithmetic the clause lives in.
- **LLM-oriented diagnostics.** Every error code is stable (E001–E702); every diagnostic carries a fix hint and a spec reference. The CLI emits JSON for tooling.

## Maturity.

Vera is at v0.0.157+ with 300+ stars, 3,400+ tests at 96% coverage, 76 conformance programs validating every language feature, and a 13-chapter specification. The reference compiler is Python; programs compile to WebAssembly via wasmtime and run in the browser. Published VeraBench results report 93% flagship correctness on zero training data, matching Python.

Current focus is standard-library breadth. Open questions remain in monomorphisation reindexing and GC-rooting around inference calls. The language is usable but the surrounding ecosystem (LSP, package registry, IDE plug-ins) is still building.

## Agent tooling.

Three documents in the repo target agent authors directly: `SKILL.md` (complete language reference for agents writing Vera), `AGENTS.md` (setup for any agent system), and `CLAUDE.md` (project orientation for Claude Code specifically). All three are rendered into `llms.txt` and `llms-full.txt` for ingestion by agent frameworks. The diagnostics output JSON when asked, with stable error codes that can be referenced from agent prompts.

### Design DNA

- **Aver** *(Verification)* — Closest design relative. Co-located verify blocks, Lean 4 proof export, decision blocks. Different syntax for the same diagnosis. Now integrated into VeraBench.
- **Prove** *(Verification)* — Same verification diagnosis, opposite politics. Licence explicitly prohibits AI training use. Refinement types, verb-based IO tracking.
- **AILANG** *(Verification)* — Capability-based effects with row polymorphism. Where Vera tracks <code>&lt;Inference&gt;</code> as one effect, AILANG carves it into IO/FS/Net/Clock/AI.
- **Magpie** *(Syntactic)* — Cross-camp foil. Magpie strips ambiguity at the surface (SSA form); Vera adds a layer of mechanical checks. Different bets on where the error budget should be spent.

### Timeline

- **Feb 2026** — First public release (v0.0.4). Grammar, parser, and type checker. No verifier yet.
- **Mar 2026** — Z3 verifier lands. Three-tier verification scheme published. First externally-contributed example merges.
- **Apr 2026** — VeraBench published with 93% flagship correctness vs Python baseline on zero training data.
- **Apr 2026** — <code>&lt;Inference&gt;</code> effect added as first-class typed effect. Aver becomes first external language integrated into VeraBench.
- **May 2026** — v0.0.157 releases. 300+ stars, 76 conformance programs, 13-chapter spec.

*Detail page: https://agentlanguages.dev/languages/vera/  ·  Markdown companion: https://agentlanguages.dev/languages/vera.md*

## Vow

> Every function carries machine-checked vows. ESBMC bounded model checking discharges them at compile time. The compiler binary ships its own Claude Code skill, generated from the same source as the toolchain.

**Camp:** Verification
**Author:** Paulo Matos
**Implementation language:** Rust (stage 0); self-hosted in Vow
**Compilation target:** Native (via Cranelift); C (for the ESBMC verification pipeline only)
**Licence:** MIT
**First seen:** February 2026
**Maturity:** working compiler
**Site:** https://vow-lang.com
**Repo:** https://github.com/vow-lang/vow
**Agent tooling:**
- CLAUDE.md
- compiler-bundled Claude Code skill (auto-installed to .claude/skills/vow/)
- vowc skill print --bundle (self-contained markdown for non-Claude harnesses)
- structured JSON diagnostics with counterexamples and blame

### Key idea

Every function declares a `vow` block of `requires`/`ensures`; loops carry
`invariant`. The compiler lowers these to obligations for the ESBMC bounded
model checker before any code ships. Diagnostics emit JSON in parallel with
human text, with explicit Caller/Callee blame on every violation. The
compiler binary embeds and auto-installs a Claude Code skill generated from
the same compiler version &mdash; "the source of truth for any harness
writing Vow code; cannot drift from the toolchain you are running."

## The thesis.

Vow is a verification-camp project whose stated audience is not human readers. The homepage announces it directly: "The syntax is not for you. The semantics is not for you. The language is not for you. Yours is only the product." Every module makes vows &mdash; preconditions, postconditions, and loop invariants &mdash; that the compiler lowers to obligations for the [ESBMC](https://esbmc.org) bounded model checker. The CLI emits JSON in parallel with human text on every invocation; counterexamples come back as structured records the agent can read. The intended workflow is CEGIS: write, compile, verify, read counterexamples, fix, iterate.

<p class="pullquote">The syntax is not for you. The semantics is not for you. The language is not for you. Yours is only the product.</p>

The distinctive move is the choice of checker. Vera and Intent dispatch contracts to Z3 SMT; Aver exports them as Lean 4 theorems or Dafny lemmas; NanoLang proves the core type system in Coq from below. Vow chooses ESBMC, a bounded model checker, and accepts the trade that comes with it &mdash; counterexamples are concrete inputs the agent can re-run against, but soundness holds only up to the unwinding bound chosen for each verification call. The repository's `CLAUDE.md` is explicit about the consequence: "Bounds like `n <= 10` (to fit within `--unwind 10`) or `a <= 100` (to help the SMT solver) are verification artefacts. They do not belong in `requires`/`ensures` clauses... If ESBMC can't prove a correct contract, that's ESBMC's problem."

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">module</span> <span class="ty">Bisect</span>

<span class="kw">fn</span> bisect(lo: <span class="ty">i64</span>, hi: <span class="ty">i64</span>) -&gt; <span class="ty">i64</span> <span class="kw">vow</span> {
  <span class="ct">requires</span>: hi &gt;= lo
} {
  <span class="kw">let mut</span> lo: <span class="ty">i64</span> = lo;
  <span class="kw">let mut</span> hi: <span class="ty">i64</span> = hi;
  <span class="kw">while</span> lo + <span class="num">1</span> &lt; hi <span class="kw">vow</span> {
    <span class="ct">invariant</span>: hi - lo &gt;= <span class="num">0</span>
  } {
    <span class="kw">let</span> mid: <span class="ty">i64</span> = lo + (hi - lo) / <span class="num">2</span>;
    lo = mid;
  }
  lo
}

<span class="kw">fn</span> main() -&gt; <span class="ty">i32</span> [<span class="ct">io</span>] {
  <span class="kw">let</span> r: <span class="ty">i64</span> = bisect(<span class="num">0</span>, <span class="num">64</span>);
  print_i64(r);
  <span class="num">0</span>
}</pre>
  </div>
  <p class="caption">A <code>vow</code> block follows the function signature; loop vows carry <code>invariant</code> clauses. The <code>[io]</code> effect set on <code>main</code> declares the only impurity in the program &mdash; pure functions carry no annotation, and calling an effectful function from a pure one is a type error.</p>
</div>

## Distinctive moves.

- **ESBMC over SMT or theorem provers.** Contracts lower to verification obligations the bounded model checker discharges before codegen. The C emitter exists only to feed that pipeline; native code comes from Cranelift directly.
- **Blame is structured.** `requires` violations fault the Caller; `ensures` and `invariant` violations fault the Callee. The JSON violation record carries the verdict explicitly (`{"error":"VowViolation","vow_id":N,"blame":"Caller"|"Callee",...}`).
- **Compiler-bundled agent skill.** Running `vowc build` in a project that already has a `.claude/` directory writes the skill to `.claude/skills/vow/` the first time, sourced from the running compiler binary. The repository describes the bundle as "the source of truth for any harness writing Vow code... it cannot drift from the toolchain you are running." Non-Claude harnesses get the same bundle via `vowc skill print --bundle`.
- **Canonical form as a compiler pass.** The printer is a stage, not a formatter; `parse → print → parse` is enforced to be idempotent by tests. There is one preferred way to write each construct.
- **Linear types and checked arithmetic at the grammar level.** `linear struct` values must be consumed exactly once; the type checker tracks the obligation. `+!`, `-!` and the rest are distinct token kinds from `+`, `-` &mdash; checked and wrapping arithmetic are not library functions but grammar productions.
- **What is deliberately absent.** No generics, no traits, no closures, no higher-order functions, no macros, no garbage collector. The `CLAUDE.md` design rule rejects any feature that "introduces a new type-system axis."

## Maturity.

`v0.2.0` released 20 May 2026 on a repository created 25 February 2026, MIT-licensed under the `vow-lang` GitHub organisation. The Rust workspace splits across nine crates (`vow-syntax`, `vow-types`, `vow-ir`, `vow-codegen`, `vow-verify`, `vow-runtime`, `vow-diag`, `vow-clif-shim`, `vow`) feeding a Pizlo-style SSA IR and a Cranelift backend. The self-hosted compiler in `compiler/` (13 modules) compiles itself to a byte-identical binary under the project's bootstrap triple test &mdash; stage 0 produces compiler A, compiler A produces compiler B, compiler B produces compiler C, and `sha256sum` of B and C must match. Mutation testing ships as a `vowc mutants` subcommand with a tiered oracle and structured JSON output.

Three substantial programs ship in `examples/` alongside the small contract demonstrations: a deterministic CDCL SAT solver (watched literals, first-UIP learning, non-chronological backtracking, Luby restarts), a UCI-compatible chess engine, and a Lean 4 kernel checker (`vow-lean-kernel`) targeting the Lean Kernel Arena. The author's announcement names the standing gap explicitly: "ESBMC integration is in place and discharges contracts for the example programs, but the corners are still being found... The compiler is written in Vow but its own vows are not all verified end-to-end. Closing that loop is the single most important piece of work ahead." The `benchmarks/` directory holds Vow's implementation of vericoding ([arXiv:2509.22908](https://arxiv.org/abs/2509.22908)) &mdash; 40 problems across 15 Easy, 15 Medium, and 10 Hard tiers, with a Python harness in `bench/` that runs frontier models against paper baselines (Dafny 82%, Verus/Rust 44%, Lean 27%). Published numbers for Vow itself sit on the roadmap rather than in a release.

## Agent tooling.

`CLAUDE.md` runs to ~20 KB and addresses the language design rules to Claude directly; `AGENTS.md` is a nine-byte placeholder. The substantive agent surface is the compiler-emitted skill: `vowc skill install --local` writes `SKILL.md` plus `reference/`, `examples/`, and `schemas/` to `.claude/skills/vow/`; `vowc build` auto-installs the same payload the first time it runs in a project that already has `.claude/`; `vowc skill print --bundle` emits a single self-contained markdown document for raw-API harnesses. Diagnostics from every crate flow through `vow-diag`, which always emits JSON and human text in parallel &mdash; "this is by design, not a flag." `vowc --help` returns a structured JSON capability description by default and human text under `--human`.

### Design DNA

- **Vera** *(Verification)* — Closest design relative. Both ship mandatory contracts on every function and treat the agent as the primary author. Vera dispatches to Z3 SMT and drops parameter names via typed DeBruijn slots; Vow dispatches to the ESBMC bounded model checker, keeps names, and adds linear types plus distinct checked/wrapping arithmetic tokens at the grammar level.
- **Intent** *(Verification)* — Same camp, same era, different checker. Intent commits to Z3 SMT with runtime enforcement on three backends (Rust, JavaScript, WebAssembly); Vow commits to ESBMC bounded model checking with a single Cranelift native backend and a C emitter that exists only because the verification pipeline needs it.
- **Aver** *(Verification)* — Same diagnosis, different upper-bound check. Aver exports <code>verify</code> blocks as Lean 4 theorems or Dafny lemmas via Oracle, lifting effectful code into proof artefacts; Vow discharges contracts in-process with ESBMC and emits structured counterexamples for the agent to fix against.
- **NanoLang** *(Verification)* — Different proof tool, different layer. NanoLang ships 193 Coq theorems with zero axioms proving the language's core type system from below; Vow ships ESBMC obligations on every function from above. Both pair the proof discipline with a self-hosted compiler.

### Timeline

- **Feb 2026** — First commit lands (25 February). Rust stage 0 compiler, ESBMC integration, vow block grammar.
- **May 2026** — <code>v0.2.0</code> released (20 May). Self-hosted compiler (13 modules in <code>compiler/</code>) achieves byte-identical binary fixed point under the bootstrap triple test.
- **May 2026** — Public announcement. Author publishes <em>What's in a Vow</em> and ships CDCL SAT solver, UCI chess engine, and a Lean 4 kernel checker (<code>vow-lean-kernel</code>) targeting the Lean Kernel Arena, all written in Vow.

*Detail page: https://agentlanguages.dev/languages/vow/  ·  Markdown companion: https://agentlanguages.dev/languages/vow.md*

## Zero

> Vercel Labs' agent-first systems language. Sub-10 KiB native binaries. Structured JSON diagnostics with stable codes and typed repair plans. One obvious path.

**Camp:** Verification
**Also spans:** Syntactic
**Author:** Chris Tate and Matt Van Horn / Vercel Labs
**Implementation language:** C (zero-c bootstrap); self-hosted compiler-zero in progress
**Compilation target:** Native binaries (direct ELF/Mach-O/PE emitters, no LLVM), WebAssembly
**Licence:** Apache-2.0
**First seen:** May 2026
**Maturity:** early implementation
**Site:** https://zerolang.ai
**Repo:** https://github.com/vercel-labs/zerolang
**Agent tooling:**
- structured JSON diagnostics
- stable error codes
- typed repair plans
- zero skills (version-matched agent guidance)
- zero explain
- zero fix --plan --json
- zero doctor

### Key idea

Zero is Vercel Labs' bet on agent-first systems programming. The compiler
emits structured JSON diagnostics with stable error codes (NAM003 means
"unknown identifier" and will keep meaning that), typed repair plans an
agent can apply without parsing prose, and version-matched guidance served
through the CLI itself rather than scraped from a docs site. The language
is intentionally explicit: capability objects on main, no hidden allocator,
no implicit async, one obvious path for most things.

## The thesis.

Zero is Vercel Labs' bet that the bottleneck in agentic coding is not the language but the channel between the compiler and the agent. The standard loop is fragile: the compiler emits prose written for human engineers, the agent parses it as text, the agent guesses at a fix, the next compile produces a new prose error in a slightly different format. Zero's answer is to replace the prose channel with a structured one. `zero check --json` emits a stable error code (`NAM003`), a human-readable message, a line number, and a typed `repair` object an agent can act on. `zero fix --plan --json` returns a machine-readable edit plan. `zero explain` returns structured explanations against the installed compiler version.

<p class="pullquote">Humans read the message. Agents read the JSON.</p>

The distinctive move sits at the language level, not the toolchain level: Zero collapses the syntactic and verification camps into a single product decision. The language documents itself as preferring "one obvious way to express most things, even when that makes code more explicit than a human might choose," which is syntactic-camp framing; but the obviousness is bought with capability objects on `main`, explicit `raises` markers, and effect-visible signatures, which is verification-camp machinery. Where MoonBit, the catalogue's other industrially backed verification entry, invests in semantic-aware token sampling, Zero invests in making the surface area small enough that an agent doesn't need help generating it in the first place.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">pub</span> <span class="kw">fun</span> <span class="ty">main</span>(world: <span class="ty">World</span>) <span class="op">-&gt;</span> <span class="ty">Void</span> <span class="kw">raises</span> {
  <span class="kw">check</span> world.out.write(<span class="str">"hello from zero\n"</span>)
}</pre>
  </div>
  <p class="caption">There is no hidden global process object. <code>world: World</code> is an explicit capability passed in by the runtime; <code>raises</code> declares the function can propagate errors; <code>check</code> handles a fallible operation. A function that doesn't ask for <code>World</code> cannot write to stdout.</p>
</div>

## Distinctive moves.

- **Stable diagnostic codes.** Errors carry codes (`NAM003` for unknown identifier) that are contractually stable across compiler versions. Agents can match on the code, not the prose.
- **Typed repair plans.** `zero fix --plan --json` returns a structured edit plan, not advice. The agent applies the plan rather than inferring it from the message.
- **Version-matched skills.** `zero skills get zero --full` returns syntax, diagnostics, build, package, stdlib, testing, and edit-loop guidance pinned to the installed compiler version. The guidance lives in the toolchain, not on a webpage that may have drifted.
- **No LLVM, sub-10 KiB binaries.** Direct emitters for ELF, Mach-O, PE, and WebAssembly. The size budget is a load-bearing design constraint, not a marketing claim.
- **One CLI surface.** `zero check`, `zero run`, `zero build`, `zero graph`, `zero size`, `zero routes`, `zero skills`, `zero explain`, `zero fix`, `zero doctor` &mdash; all subcommands of a single binary that all support `--json`.

## Maturity.

v0.1.1, Apache-2.0, released 15 May 2026, 3.3k stars on `vercel-labs/zerolang` at first cataloguing. The README and homepage are explicit that this is a "pre-1 experiment": syntax and APIs are not a contract, breaking changes are expected, and security vulnerabilities should be expected &mdash; Vercel Labs recommends running Zero only in isolated environments. The repo maintains two compilers: `zero-c`, the C bootstrap; and `compiler-zero`, a self-hosting compiler written in Zero. Cross-compilation is limited to a documented target subset; there is no package registry yet; VS Code syntax highlighting ships in-repo. Named contributors are Chris Tate and Matt Van Horn.

The bet is that structured agent-first compiler output becomes table stakes once developers see what it does for repair loops. Even if Zero itself doesn't win, the design pattern &mdash; stable codes, typed repairs, version-matched skills &mdash; is a concrete argument for what every other compiler should ship.

## Agent tooling.

The toolchain *is* the agent tooling. `zero check --json` returns diagnostics; `zero explain <code>` returns explanations; `zero fix --plan --json` returns edit plans; `zero skills get zero --full` returns version-matched workflows. `zero graph --json`, `zero size --json`, `zero routes --json`, and `zero doctor --json` round out the inspection surface. Vercel's separately released `skills.sh` ecosystem is consumable by Claude Code, Cursor, Codex, and other agent harnesses through the same Agent Skills spec that `zero skills` follows.

### Design DNA

- **MoonBit** *(Verification)* — Industrial backing parallel. Vercel Labs and IDEA Shenzhen are the two best-resourced bets in the catalogue; MoonBit invests in semantic-aware sampling, Zero invests in structured compiler output and version-matched skills.
- **NERD** *(Syntactic)* — Cross-camp foil. Both lean on a small keyword vocabulary and 'one obvious way' framing; NERD does it for syntactic legibility, Zero does it inside a verification project with capability-typed effects and a typed repair API.
- **Boruna** *(Orchestration)* — Structured-diagnostics parallel. Zero ships JSON diagnostics with typed repair IDs at the language level; Boruna ships hash-chained evidence bundles at the runtime level. Both reject prose as an interface for agents.

*Detail page: https://agentlanguages.dev/languages/zero/  ·  Markdown companion: https://agentlanguages.dev/languages/zero.md*

---

# Orchestration camp (5)

> It isn't a language problem. It's an agent-coordination problem. The orchestration camp re-frames the question — the trouble with LLM-authored code, they argue, isn't any specific defect in the code; it's that agents need to be sequenced, sandboxed, audited, and approved by humans at the right points. The language is just the substrate; the runtime is where the action is.

## Boruna

> Deterministic, capability-safe workflow execution. Every effect declared, policy-gated. Hash-chained tamper-evident evidence bundles.

**Camp:** Orchestration
**Also spans:** Verification
**Author:** escapeboy
**Implementation language:** Rust
**Compilation target:** Bytecode (custom VM)
**Licence:** MIT
**First seen:** April 2026
**Maturity:** working compiler
**Site:** https://github.com/escapeboy/boruna
**Repo:** https://github.com/escapeboy/boruna
**Agent tooling:**
- MCP server with 10 tools
- AGENTS.md
- CLAUDE.md
- diagnostics and auto-repair commands

### Key idea

Deterministic, capability-safe workflow execution for auditable AI
systems. DAG workflows where steps are `.ax` source files. Every side
effect — LLM calls, HTTP, database, filesystem — is declared and
policy-gated at the VM level. Hash-chained tamper-evident evidence
bundles. Deterministic replay. Approval gates for human-in-the-loop.
The pitch: when a regulator asks what exactly ran and what the model
returned, you can prove it.

## The thesis.

Boruna doesn't think the problem with LLM code is the code. It thinks the problem is that when an agent system does something consequential — sends an email, transfers money, modifies a database — you need to be able to prove what ran, what the model said, and who approved it. That's not a language problem. That's a runtime problem. So Boruna builds the runtime.

<p class="pullquote">When a regulator asks what exactly ran, you can prove it.</p>

The unit of computation is a DAG workflow. Each step is an `.ax` source file. Every side effect — LLM call, HTTP request, database write, filesystem mutation — is declared in the source and policy-gated at the VM level. The VM refuses to execute a step that would perform an undeclared effect; the policy layer lets administrators forbid specific declared effects per workflow or per role. Every executed step writes to a hash-chained evidence bundle that's tamper-evident; the bundle is sufficient to replay the workflow deterministically (same inputs, same model responses recorded, same outputs).

## Distinctive moves.

- **Capability-safe by construction.** A step can't reach for an effect it didn't declare. The VM is the enforcement point, not a linter.
- **Hash-chained evidence bundles.** Every step's inputs, outputs, model responses, and approvals chain into a Merkle structure. Tampering breaks the chain.
- **Deterministic replay.** Re-running a workflow against its evidence bundle produces bit-identical results. No "it worked on my machine" for LLM-driven workflows.
- **Approval gates.** Human-in-the-loop steps are a first-class workflow primitive, not bolted on. The approval becomes part of the evidence.
- **MCP server with 10 tools.** Boruna's agent-facing surface lets a coding agent author, validate, and run workflows without leaving the protocol.

## Maturity.

v0.2.0 with 34 commits and 1 release. 557+ tests passing across a 9-crate Rust workspace covering the compiler (lexer, parser, type checker, code generator), the bytecode VM, the orchestrator, and the MCP server. Single-author project; zero stars at the time of cataloguing, which dramatically understates the engineering depth here. The architecture is more carefully thought through than several entries with two orders of magnitude more attention.

The bet is that the regulated-industries angle (financial services, healthcare, government) will discover Boruna before the broader market does. The agent-system gold rush will eventually hit regulators, and when it does, "I can prove what ran" stops being a feature and starts being a requirement.

## Agent tooling.

`AGENTS.md` and `CLAUDE.md` orient agents working in the repository. The MCP server exposes ten tools an agent can call to draft workflows, run them in dry-run mode, validate effect declarations against policy, inspect evidence bundles, and trigger approvals. Diagnostics ship with auto-repair commands — when the type checker rejects a workflow, the diagnostic suggests the specific edit that would satisfy it.

### Design DNA

- **Vera** *(Verification)* — Cross-camp cousin. Both treat agent code as untrusted by default; Vera builds the trust at the type level (contracts), Boruna builds it at the runtime level (policy-gated effects + evidence bundles). Vera's <code>&lt;Inference&gt;</code> effect is conceptually close to Boruna's declared LLM call.
- **Pel** *(Orchestration)* — Same camp, different stack. Pel argues for grammar-level capability control; Boruna implements bytecode-level capability gating. Pel exists as an academic paper; Boruna ships as a 9-crate Rust workspace.
- **Quasar** *(Orchestration)* — Shares the approval-gate intuition. Quasar measured 52% fewer user-approval interactions by lifting approval into the language; Boruna lifts it into the runtime with deterministic replay so the approval can be audited after the fact.
- **Plumbing** *(Adjacent)* — Plumbing defines the wiring between agents at the type level (typed channels, structural morphisms); Boruna defines what runs inside one agent and how it's audited. Complementary rather than competing.

### Timeline

- **Apr 2026** — v0.2.0 published. 9-crate Rust workspace: compiler (lexer, parser, type checker, code generator), bytecode VM, orchestrator, MCP server.
- **Apr 2026** — 557+ tests passing. Hash-chained evidence bundle format stabilises.
- **May 2026** — Catalogued. Still 0 stars; the engineering depth runs ahead of the public profile.

*Detail page: https://agentlanguages.dev/languages/boruna/  ·  Markdown companion: https://agentlanguages.dev/languages/boruna.md*

## Lumen

> Markdown-native source (.lm.md). Algebraic effects, grants for tool and model calls, @deterministic compile-time enforcement, and pipeline / machine / memory process kinds. A language for humans authoring agent workflows.

**Camp:** Orchestration
**Author:** alliecatowo
**Implementation language:** Rust
**Compilation target:** LIR bytecode → register-based VM (~100 opcodes); WebAssembly via lumen-wasm
**Licence:** MIT
**First seen:** February 2026
**Maturity:** working compiler
**Site:** https://alliecatowo.github.io/lumen/
**Repo:** https://github.com/alliecatowo/lumen
**Agent tooling:**
- AGENTS.md (multi-agent dev team config)
- CLAUDE.md
- .opencode/agents/
- LSP server (lumen-lsp) with semantic search
- VS Code extension
- Tree-sitter grammar
- MCP provider crate (lumen-provider-mcp)
- emit-bytecode-as-JSON CLI (`lumen emit`)

### Key idea

Lumen is for humans authoring agent workflows, not for agents to
author general code — it earns its place in the catalogue via the
orchestration-camp criterion of first-class effect declarations for
model calls and agent-coordination primitives. Algebraic effects
appear in function signatures after a slash; grants constrain every
call to a tool with explicit caps; @deterministic enforces rejection
of nondeterministic ops at compile time; pipeline / machine / memory
are first-class process kinds. Source is markdown-native — .lm.md
files unify code and documentation in one artefact.

## The thesis.

Lumen is a language for humans, but its target is the substrate above the model rather than the model itself. The catalogue's nominal inclusion bar is "designed for LLMs to author code"; Lumen earns its place via the orchestration-camp criterion of first-class effect declarations for model calls and agent-coordination primitives. The vocabulary is the giveaway: `cell` (function), `effect`, `grant`, `agent`, `pipeline`, `machine`, `memory` are all language keywords. Functions declare effects in the return type after a slash (`cell main() -> String / {Log}`); tools must be granted with explicit caps (`grant Chat max_tokens 1024 temperature 0.7`); the runtime can be locked into `@deterministic true` mode that rejects nondeterministic operations at compile time, not at runtime.

<p class="pullquote">Build deterministic agent workflows with static types, first-class AI primitives, and markdown-native source files.</p>

The distinctive move is making the source file the same artefact as the documentation. Three source extensions ship: `.lm.md` (markdown with fenced Lumen blocks), `.lm` (raw source), and `.lumen` (markdown-native). The compiler's first pipeline stage is markdown extraction &mdash; code and prose share one file, and the model writing one writes the other. Where Boruna does deterministic-workflow enforcement at the bytecode VM via policy-gated effects and hash-chained evidence, Lumen does it at the type system via algebraic effects, grants, and the `@deterministic` annotation. Same orchestration-camp diagnosis, different layer.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">effect</span> <span class="ty">Log</span>
  <span class="kw">cell</span> info(msg: <span class="ty">String</span>) -&gt; <span class="ty">Unit</span>
<span class="kw">end</span>

<span class="kw">cell</span> <span class="ty">main</span>() -&gt; <span class="ty">String</span> / {<span class="ct">Log</span>}
  <span class="kw">perform</span> <span class="ty">Log</span>.info(<span class="str">"Starting"</span>)
  <span class="kw">return</span> <span class="str">"Done"</span>
<span class="kw">end</span>

<span class="kw">handle</span> main() <span class="kw">with</span> <span class="ty">Log</span>.info(msg) -&gt; resume(unit)
  print(<span class="str">"LOG: {msg}"</span>)
<span class="kw">end</span></pre>
  </div>
  <p class="caption">The <code>/ {Log}</code> in the return type declares the effect. <code>perform</code> invokes it; <code>handle ... with ...</code> discharges it. One-shot delimited continuations under the hood. <code>cell</code> is the function keyword — Lumen does not use <code>fn</code>.</p>
</div>

## Distinctive moves.

- **Markdown-native source.** `.lm.md` files contain markdown prose with fenced `lumen` blocks. The compiler extracts code as its first pipeline stage. Documentation and implementation are one artefact.
- **`cell` is the function keyword.** Not `fn`. Cells take typed parameters and declare effects in the return type after a slash.
- **Algebraic effects, first-class.** `effect Log` declarations, `perform` to invoke, `handle ... with ...` to discharge. The runtime uses one-shot delimited continuations; opcodes `Perform`, `HandlePush`, `HandlePop`, and `Resume` are first-class in the VM.
- **Grants as syntactic policy.** `grant Chat max_tokens 1024 temperature 0.7` constrains every call to that tool. Policy lives in source, not configuration &mdash; Boruna's effect declarations lifted to the language surface.
- **`@deterministic true` mode.** A compile-time annotation that rejects `uuid()`, `timestamp()`, and other nondeterministic operations. The static analogue of Boruna's runtime deterministic replay.
- **Three process kinds.** `pipeline` for auto-chained stages (extract → transform → load), `machine` for state graphs, `memory` for key-value stores. Each is a first-class language construct rather than a library pattern.
- **MCP as a provider crate.** `lumen-provider-mcp` ships alongside `lumen-provider-http`, `lumen-provider-json`, `lumen-provider-fs`, `lumen-provider-gemini`, `lumen-provider-crypto`, `lumen-provider-env`. MCP is one tool source among several, not the privileged one.

## Maturity.

v0.1.10 (February 2026), 352 commits, ~5,300 passing tests across all crates (AGENTS.md figure; the README's 1,365+ is at a different cut). MIT-licensed, written in Rust (96.5% of the source), compiles to LIR bytecode for a register-based VM with ~100 opcodes, 32-bit fixed-width encoding, and COW collections via `Rc::make_mut`. The workspace contains 12+ crates covering compiler, VM, runtime, CLI, LSP, JIT codegen, WebAssembly bindings, tensor operations, and provider integrations. Single-author at the human level (`alliecatowo`); AGENTS.md notes that "only the Delegator agent commits code" &mdash; the contributors listing reflects agent runs of the project's own multi-agent dev team.

## Agent tooling.

The agent-facing surface is unusually elaborate. `AGENTS.md` declares a multi-agent development team &mdash; Delegator (Gemini 3 Pro), Auditor, Debugger (Claude Opus 4.6), Coder (Claude Sonnet 4.5), Worker (Claude Haiku 4.5), Tester, Task Manager, Performance, Planner &mdash; each with a defined role and only the Delegator authorised to commit. `CLAUDE.md` and `.opencode/agents/` provide further orientation. The LSP includes semantic search; the VS Code extension covers `.lm.md` files; a tree-sitter grammar ships at `tree-sitter-lumen/`. The CLI's `lumen emit` mode outputs bytecode as JSON for downstream agent consumption.

### Design DNA

- **Boruna** *(Orchestration)* — Closest design relative. Both target deterministic AI workflows. Boruna enforces at the bytecode VM via policy-gated effects and hash-chained evidence bundles; Lumen enforces at the type system via algebraic effects, grants, and a compile-time <code>@deterministic</code> annotation. Both ship MCP integration. Lumen targets humans authoring workflows; Boruna targets auditable execution.
- **Plumbing** *(Adjacent)* — Plumbing wires agents (typed channels, copy-discard symmetric monoidal category); Lumen is what runs inside a single node of that wiring. Complementary substrates rather than competitors.
- **AILANG** *(Verification)* — Cross-camp on effect systems. AILANG's row-polymorphic Hindley-Milner with capability categories (IO/FS/Net/Clock/AI) is the verification cousin of Lumen's effect-row syntax. Different mechanism, same diagnosis: model calls must be visible in the signature.

*Detail page: https://agentlanguages.dev/languages/lumen/  ·  Markdown companion: https://agentlanguages.dev/languages/lumen.md*

## Marsha

> Functional, English-based language whose .mrsh files (declaration, description, examples) are compiled to tested Python by an LLM. Alpha implementation; last maintainer activity early Aug 2023.

**Camp:** Orchestration
**Author:** David Ellis (Alan Technologies)
**Implementation language:** Python
**Compilation target:** Python (generated by an LLM, with auto-generated tests)
**Licence:** MIT
**First seen:** July 2023
**Maturity:** early implementation
**Site:** https://github.com/alantech/marsha
**Repo:** https://github.com/alantech/marsha

### Key idea

Marsha's framing is that the LLM is the compiler. A .mrsh source file is a
Markdown-shaped specification with three sections per function: a typed
declaration, an English description, and a list of input/output examples.
The Marsha toolchain prompts an LLM to produce Python that satisfies the
declaration, uses the examples to synthesise a test suite, and iterates with
corrective feedback until the tests pass or the attempt budget is exhausted.

## The thesis.

Marsha's framing predates almost every other entry in the catalogue: the LLM is the compiler. A `.mrsh` source file is a Markdown-shaped specification with three sections per function &mdash; a typed declaration (`# func name(InputType): OutputType`), an English description of behaviour, and a bullet list of input/output examples including expected error cases. The Marsha toolchain prompts an LLM to generate Python that satisfies the declaration, uses the examples to synthesise a pytest suite, runs the suite, and iterates with corrective feedback until the tests pass or the configured attempt budget is exhausted. CLI flags (`-a` attempts, `-n` parallel "thought" threads, `-q` quick-and-dirty, `--exclude-main-helper`) expose the iteration parameters directly to the user. Generated programs ship with an auto-attached CLI wrapper and an optional REST-server mode (`-s`).

## Published results.

The repository ships an alpha implementation, an examples directory (general-purpose, web-scraping, data-mangling), and a CI workflow that times compilations. The README's only quantitative target is its own roadmap: "We aim for 80%+ accuracy on our examples", with the roadmap looking to push that "above 90%". The compiler requires `OPENAI_ORG` and `OPENAI_SECRET_KEY`; support for other or local LLMs is listed as planned but unimplemented. The `setup.py` PyPI classifier is `Development Status :: 2 - Pre-Alpha`.

## Status.

The repository is MIT-licensed, was launched alongside a Show HN post on 1 August 2023 (news.ycombinator.com item 36864021), and reached the high-hundreds in stars with around a dozen forks. The last maintainer activity on the main branch &mdash; pull requests #159&ndash;#164 from `dfellis` and issue #165 from `depombo` &mdash; is dated 1&ndash;8 August 2023 (PR #164 "Embed Llama.cpp into Marsha for local usage" opened 7 Aug 2023; issue #165 "Add LlamaCPP support" opened 8 Aug 2023); the project has received no further maintainer pull requests or issues since. Both principals subsequently moved on (David Ellis to IaSQL and later the Alan-lang project; Luis de Pombo's LinkedIn lists continued tenure at Alan Technologies). Marsha is catalogued because the "LLM is the compiler" framing it shipped in 2023 anticipates the 2025 orchestration papers in this camp, not because the alpha implementation is under active development.

### Design DNA

- **Pel** *(Orchestration)* — Same camp, two years apart. Marsha (2023) treats the LLM as a compiler emitting Python under English+examples; Pel (2025) treats the LLM as a code emitter constrained by a grammar designed for it. Both predate the now-common 'agents write code' framing.
- **Boruna** *(Orchestration)* — Opposite stance on where the LLM sits in the stack. Marsha puts the LLM at the back end of a compiler. Boruna treats every LLM call as a policy-gated effect inside a deterministic VM. Same camp, inverted topology.
- **Quasar** *(Orchestration)* — Different generation: Quasar measures execution-time and approval-interaction reductions on ViperGPT/CaMeL; Marsha measures end-to-end compile success rate against its own examples ('we aim for 80%+ accuracy').

*Detail page: https://agentlanguages.dev/languages/marsha/  ·  Markdown companion: https://agentlanguages.dev/languages/marsha.md*

## Pel

> Lisp-flavoured language for orchestrating LLM agents, with capability control enforced at the grammar level and a REPeL self-healing loop modelled on Common Lisp restarts.

**Camp:** Orchestration
**Author:** Behnam Mohammadi (CMU)
**Implementation language:** N/A (paper-only)
**Compilation target:** N/A (paper-only)
**Licence:** N/A (academic paper)
**First seen:** April 2025
**Maturity:** research paper
**Site:** https://arxiv.org/abs/2505.13453
**Paper:** https://arxiv.org/abs/2505.13453

### Key idea

Pel argues that orchestrating LLM agents should not rely on Python plus a sandbox.
Instead, the grammar itself is the capability surface: an LLM emits Pel under
constrained generation, and an action the grammar cannot express is an action
the agent cannot take. The runtime adds piping, first-class closures, natural
language conditions evaluated by an LLM, automatic parallelisation via static
dependency analysis, and a REPeL (Read-Eval-Print-Loop) with Common Lisp-style
restarts and helper agents for automated error correction.

## The thesis.

Pel's diagnosis is that function/tool calling and free-form Python code generation each fail the orchestration problem from opposite directions: tool calling cannot express control flow, and Python is too expressive to safely run without a sandbox. The paper introduces Pel as a Lisp-inspired, homoiconic, minimal-grammar language whose syntactic surface is the capability surface. Because the grammar is small enough to be used as a constrained-decoding target, an LLM cannot emit an action the grammar cannot express; capability control becomes a property of generation, not a property of runtime sandboxing. The design takes additional cues from Elixir (piping for linear composition), Gleam (typing discipline), and Haskell (first-class closures and partial application). A REPeL &mdash; Read-Eval-Print-Loop with Common Lisp-style restarts &mdash; couples an evaluator to LLM-powered helper agents that propose restart choices when an error is signalled, so error recovery is a language feature rather than an application concern.

## Published results.

The paper is a design and rationale document rather than a benchmark study. It specifies the grammar, data types, closure semantics, piping operators, list operations, control flow, the natural-language condition form, and automatic asynchronicity via static dependency analysis. Pel is the implementation substrate for BEACON (Business Enhancement through Adaptive COordinated Networks), Mohammadi's separate SSRN paper (abstract 5191583), which describes a hierarchical multi-agent framework distributing specialised knowledge across marketing, finance, HR, and strategic-planning agents for small and family-owned businesses; BEACON reports advantages over single-model generative AI on information retrieval accuracy, cost-efficiency, and interpretability, with Pel cited as the orchestration substrate.

## Status.

Pel exists as an arXiv preprint (v1 3 Apr 2025; v2 9 Jun 2025) by a single author who completed a PhD at CMU Tepper in 2025 (thesis "Human-AI Interaction in the Era of Large Language Models (LLMs)" posted to KiltHub on 9 Jul 2025) and joined UT Dallas's Naveen Jindal School of Management as tenure-track faculty in Quantitative Marketing. The paper reports that Pel is used inside BEACON, which is supported by a BNY Foundation of Southwestern Pennsylvania fellowship via the Center for Intelligent Business at Tepper. No public implementation, package, or repository has been released; independent evaluation would require either a reference compiler or access to the BEACON codebase.

### Design DNA

- **Boruna** *(Orchestration)* — Same camp, different layer. Pel argues for grammar-level capability control; Boruna gates the same effects at the bytecode VM. Pel is a paper; Boruna ships a 9-crate Rust workspace.
- **Quasar** *(Orchestration)* — The other 2025 academic orchestration paper. Quasar transpiles a Python subset and instruments it with conformal prediction and approval gates; Pel replaces the surface language entirely and constrains generation against its grammar.
- **Marsha** *(Orchestration)* — Two-year-earlier predecessor on the same axis. Marsha treats the LLM as a compiler back-end producing Python; Pel treats the LLM as a code emitter constrained by a grammar designed for it.

*Detail page: https://agentlanguages.dev/languages/pel/  ·  Markdown companion: https://agentlanguages.dev/languages/pel.md*

## Quasar

> Penn group's LLM-agent language with automatic parallelisation, conformal-prediction reliability bounds, and approval-gated security; LLMs write a Python subset that transpiles to Quasar.

**Camp:** Orchestration
**Also spans:** Verification
**Author:** Stephen Mell et al. (Penn)
**Implementation language:** N/A (paper-only)
**Compilation target:** N/A (paper-only)
**Licence:** N/A (academic paper)
**First seen:** June 2025
**Maturity:** research paper
**Site:** https://arxiv.org/abs/2506.12202
**Paper:** https://arxiv.org/abs/2506.12202

### Key idea

Quasar (a backronym for QUick And Secure And Reliable) accepts code actions
in a Python subset that is transpiled to a custom language with three built-in
properties: automatic parallelisation of independent external calls, compositional
conformal prediction for uncertainty quantification, and explicit user-approval
gates around sensitive tool invocations. The bet is that the LLM keeps writing
the Python it knows while the runtime supplies the guarantees Python lacks.

## The thesis.

Quasar starts from the observation that LLM agents increasingly act by writing code, and that Python is the default not because it is well suited but because LLMs are fluent in it. The paper enumerates Python's weaknesses for this role &mdash; limited built-in support for performance, security, and reliability &mdash; and proposes a purpose-built language that addresses all three at once. Performance comes from automatic parallelisation of independent external calls, drawing on Mell, Kallas, Zdancewic and Bastani's "Opportunistically Parallel Lambda Calculus" (arXiv:2405.11361, published as Proc. ACM Program. Lang. 9, OOPSLA2, October 2025). Reliability comes from compositional conformal prediction (Ramalingam, Park and Bastani, "Uncertainty Quantification for Neurosymbolic Programs via Compositional Conformal Prediction", arXiv:2405.15912), which converts model outputs into prediction sets with a user-chosen target error rate. Security comes from user-validated action gates that surface only when the static analysis cannot rule out a sensitive effect. To avoid asking LLMs to learn a new language, the model writes a constrained subset of Python that is transpiled to Quasar.

## Published results.

The arXiv v1 (13 Jun 2025) reports an evaluation on the ViperGPT visual question answering agent over the GQA dataset, where LLMs emitting Quasar code instead of Python retain task performance while reducing execution time when possible by 42% and reducing user-approval interactions when possible by 52%, with conformal prediction achieving a chosen target coverage. The OpenReview revision (id TvpaeQVTGQ) extends the evaluation to the CaMeL agent on the AgentDojo prompt-injection benchmark and revises the headline numbers upward to "up to 56%" execution-time reduction and "up to 53%" fewer user approvals.

## Status.

No public implementation, repository, or release has been published; the OpenReview submission is under review at the time of cataloguing, and conference acceptance has not been announced. Independent evaluation would require either the transpiler and runtime from the authors or a reimplementation against the published semantics; the ViperGPT/GQA and CaMeL/AgentDojo baselines are public and reproducible. Full author list: Stephen Mell, Botong Zhang, David Mell, Shuo Li, Ramya Ramalingam, Nathan Yu, Steve Zdancewic, Osbert Bastani.

### Design DNA

- **Boruna** *(Orchestration)* — Both lift approval gates into a first-class language primitive. Quasar reports a 52% reduction in user-approval interactions by inferring when approval is unnecessary; Boruna routes the same primitive through a deterministic VM that chains every approval into a tamper-evident evidence bundle.
- **Pel** *(Orchestration)* — The other 2025 academic orchestration paper. Pel replaces the surface language with a Lisp-shaped grammar designed for constrained generation; Quasar keeps a Python subset and inserts the guarantees underneath.
- **Vera** *(Verification)* — Cross-camp foil on what 'make it checkable' means. Vera discharges contracts via Z3 at compile time; Quasar layers conformal prediction over LLM outputs to get a target coverage probability at runtime.

*Detail page: https://agentlanguages.dev/languages/quasar/  ·  Markdown companion: https://agentlanguages.dev/languages/quasar.md*

---

# Adjacent (1)

> Infrastructure that operates around agent-authored code rather than being authored by agents itself. These are wiring layers, runtime substrates, and tooling that the three-camp argument depends on but doesn't directly produce.

## Plumbing

> A typed language for the wiring between agents. Symmetric monoidal category, typed channels, structural morphisms, agents as stateful morphisms with control ports.

**Camp:** Adjacent
**Author:** William Waites / Leith Document Company
**Implementation language:** Not publicly disclosed
**Compilation target:** Native binaries (Linux x86_64, macOS Apple Silicon)
**Licence:** Free for educational/personal use; commercial licence on request
**First seen:** March 2026
**Maturity:** early implementation
**Site:** https://johncarlosbaez.wordpress.com/2026/03/11/a-typed-language-for-agent-coordination/
**Paper:** https://arxiv.org/abs/2602.13275
**Agent tooling:**
- MCP server

### Key idea

Plumbing is a typed language for the wiring between agents — the substrate
that orchestration-camp languages run on top of. Objects are typed channels
carrying infinite streams. Morphisms are processes: four structural
(copy, discard, merge, barrier) and two utility (map, filter), composed
sequentially or in parallel via the tensor product. Agents are stateful
morphisms with main, control, tool, operator-in-the-loop, and telemetry
ports. Type-checking happens before the graph runs.

## The thesis.

Plumbing is the catalogue's piece of infrastructure. It is not an orchestration language in the sense Boruna, Pel, or Marsha are &mdash; it is the typed substrate that orchestration languages can be expressed *on top of*. Where existing agent frameworks (n8n, LangGraph, CrewAI) coordinate agents with ad hoc engineering, Plumbing coordinates them with morphisms in a copy-discard symmetric monoidal category. Objects are typed channels: `!A` is a stream of `A`s, `!string` a stream of strings. Morphisms are processes with typed inputs and outputs. Four structural morphisms (copy, discard, merge, barrier) plus two utilities (map, filter) compose sequentially and via the tensor product. The compiler statically checks that every wiring is well-formed before any agent runs.

<p class="pullquote">Static typing prevents the waste.</p>

The distinctive move is to refuse the orchestration camp's normal framing. Where Boruna treats the unit of computation as an `.ax` workflow with declared effects, and Pel treats it as a grammar-level capability, Plumbing treats the unit as a channel between two processes, with the agent itself reduced to a stateful morphism with a typed protocol &mdash; main input, main output, plus control ports for runtime parameter modulation (e.g. temperature), tool-call ports, operator-in-the-loop ports, and telemetry. A judge agent that wants to cool down a debate sends a `set_temp` message on the debaters' control ports; the wiring is type-checked the same as the data path.

## What it looks like.

<div class="code-sample">
  <div class="code">
<pre><span class="kw">type</span> <span class="ty">Verdict</span> = { verdict: <span class="ty">bool</span>, commentary: <span class="ty">string</span>, draft: <span class="ty">string</span> }
<span class="kw">type</span> <span class="ty">Review</span>  = { score: <span class="ty">int</span>, review: <span class="ty">string</span>, draft: <span class="ty">string</span> }

<span class="kw">let</span> composer : !<span class="ty">string</span>  <span class="op">-&gt;</span> !<span class="ty">string</span>  = <span class="kw">agent</span> { ... }
<span class="kw">let</span> checker  : !<span class="ty">string</span>  <span class="op">-&gt;</span> !<span class="ty">Verdict</span> = <span class="kw">agent</span> { ... }
<span class="kw">let</span> critic   : !<span class="ty">Verdict</span> <span class="op">-&gt;</span> !<span class="ty">Review</span>  = <span class="kw">agent</span> { ... }

<span class="kw">let</span> main : !<span class="ty">string</span> <span class="op">-&gt;</span> !<span class="ty">string</span> = <span class="kw">plumb</span>(input, output) {
  input   ; composer ; checker
  checker ; <span class="ty">filter</span>(verdict = <span class="kw">false</span>)
          ; <span class="ty">map</span>({verdict, commentary}) ; composer
  checker ; <span class="ty">filter</span>(verdict = <span class="kw">true</span>) ; critic
  critic  ; <span class="ty">filter</span>(score &lt; 85)
          ; <span class="ty">map</span>({score, review}) ; composer
  critic  ; <span class="ty">filter</span>(score &gt;= 85).draft ; output
}</pre>
  </div>
  <p class="caption">An adversarial cover-letter composer with two feedback loops. The critic cannot see source materials &mdash; the information partition is a type-level consequence of the wiring, not a prompt instruction.</p>
</div>

## Distinctive moves.

- **Typed channels, not typed messages.** Objects in the category are streams. `!A` is a stream of `A`s; sequential composition glues stream-producing morphisms; the tensor product runs them in parallel. Well-formedness is a category-theoretic property, checked at compile time.
- **Four structural morphisms.** Copy duplicates a stream, discard throws it away, merge interleaves two streams of the same type (after coproduct injection), barrier synchronises two streams into a pair. Barrier is the synchronisation primitive that unlocks session types.
- **Agents as stateful morphisms with control ports.** An agent has main input/output plus typed control, tool, operator-in-the-loop, and telemetry ports. Runtime parameter changes (e.g. temperature) flow through the same typed pipework as data.
- **The κ-calculus "don't care, don't write" convention.** Any output port not mentioned in the program is implicitly connected to discard. The textual surface stays small while the type system still tracks every port.
- **MCP server in the release.** The first public drop ships a compiler, an interpreter, and an MCP server &mdash; agent harnesses are first-class consumers of the language, not an afterthought.

## Maturity.

Version 0p1, first public release March 2026, binary downloads for Linux x86_64 and macOS Apple Silicon. The release ships a compiler, interpreter, and MCP server. There is no public Git repository; the licence is free for educational and personal use, with a separate commercial licence available from Leith Document Company. The author is William Waites, a Chancellor's Fellow at the University of Strathclyde; the broader programme of work is described in his arXiv paper *Artificial organisations* (arXiv:2602.13275). A second blog post, "The agent that doesn't know itself," extends the calculus with session types and context compaction.

The bet is that orchestration languages eventually need a category-theoretic substrate underneath them, and that the substrate is more valuable as a typed coordination layer than as another orchestration framework competing for workflow attention.

## Agent tooling.

The release includes an MCP server alongside the compiler and interpreter, which is the entire agent-facing surface &mdash; there is no `SKILL.md`, `AGENTS.md`, or `CLAUDE.md` in this drop, because Plumbing's framing is that the language *is* the agent tooling for everything above it. Agent harnesses consume Plumbing through MCP; what agents author *in* Plumbing is the wiring diagram for other agents.

### Design DNA

- **Boruna** *(Orchestration)* — Plumbing defines the wiring between agents; Boruna defines what runs inside one agent and how it is audited. Plumbing is substrate, Boruna is workload.
- **Pel** *(Orchestration)* — Same orchestration adjacency, different formalism. Pel is a grammar-level capability calculus on an academic paper; Plumbing is a copy-discard category with session types, with a working compiler and runtime.

*Detail page: https://agentlanguages.dev/languages/plumbing/  ·  Markdown companion: https://agentlanguages.dev/languages/plumbing.md*

---

# Unclassified (3)

> Candidates that haven't shipped enough machinery — or enough public evidence — to classify yet. Their presence in the catalogue is a marker of position rather than a placement claim.

## Koru

> Zig-superset systems language with event continuations, mandatory branch handling, phantom typing, and purity tracking. Pre-alpha — 'only ever been compiled on a single computer.' AI-First framing intentionally tongue-in-cheek.

**Camp:** Unclassified
**Author:** Author anonymous (korulang)
**Implementation language:** Koru (metacircular, bootstrapped through Zig)
**Compilation target:** Zig (then native via Zig's backends)
**Licence:** Unknown
**First seen:** December 2025
**Maturity:** early implementation
**Site:** https://www.korulang.org/
**Repo:** https://github.com/korulang/koru

### Key idea

Koru is a Zig-superset systems language. Every .kz file is valid Zig;
Koru constructs are marked by a ~ leader. The distinctive design move
is event continuations with mandatory branch handling — events
declare their inputs and possible output branches in advance, and
invoking an event requires explicitly handling each branch. Phantom
typing tracks compile-time resources; purity is tracked; the compiler
is metacircular (Koru compiles to Zig). The "AI-First" claim is
architectural (event boundaries aid AI reasoning) rather than
machinery-based — no SKILL.md, AGENTS.md, MCP server, or structured-
JSON diagnostics ship. The compiler itself was authored using
Claude Opus 4.1–4.5 and Sonnet 4.5.

## What it is.

Koru is a pre-alpha Zig-superset systems programming language. Every `.kz` file is valid Zig; Koru constructs are marked by a `~` leader. The distinctive design move is event continuations with mandatory branch handling &mdash; events declare their inputs and possible output branches in advance, and invoking an event requires explicitly handling each branch. Phantom typing tracks compile-time resources; purity is tracked through the type system; the compiler is metacircular (Koru compiles to Zig). The author is anonymous behind the `korulang` GitHub org and Twitter account; the project's site lists Claude Opus 4.1&ndash;4.5 and Sonnet 4.5 as the models that authored the compiler itself.

The site's tagline is intentionally tongue-in-cheek: "The Hyper-Performant AI-First Postmodern Zero-Cost Fractal Metacircular Phantom-Typed Auto-Disposing Monadic Event Continuation Language with Semantic Space Lifting and Event Taps." Underneath the buzzword cascade, the README is candid about state: *"Pre-Alpha &mdash; Koru is pre-alpha. It has only ever been compiled on a single computer. Use and testing at your own risk."* No `SKILL.md`, `AGENTS.md`, MCP server, or structured-JSON diagnostics ship. The closest planned agent-facing surface is the Compiler Control Protocol (CCP), described as "soon" &mdash; a Koru-internal proposal, not the Model Context Protocol.

## Why it's here.

The catalogue includes Koru as a marker of the position where "AI-First" became a tagline trope. The architectural claim (event boundaries provide bounded contexts that aid AI reasoning) is a real design move; the buzzword-cascade tagline is real satire; and the catalogue's unclassified bucket exists for projects whose camp placement isn't yet evidenced by shipped agent-authoring machinery. Companion to Valea and Spec: a real project with substantive design, candidly aware of its pre-alpha state, where the "AI-First" claim is a language-architecture argument rather than a tooling commitment. The pre-alpha disclosure quoted above is the editorial centre of gravity &mdash; the rest of the entry exists to give it context.

### Design DNA

- **Valea** *(Unclassified)* — Companion in the unclassified bucket. Both stake an 'AI-native systems programming language' position with substantive design proposals but limited public evidence and no agent-authoring machinery shipped. Valea is a Rust MVP announced on Hacker News with JSON diagnostics planned; Koru is a Zig-superset metacircular compiler with event continuations and an explicitly tongue-in-cheek marketing voice.
- **Spec** *(Unclassified)* — Adjacent unclassified entry on the 'architecture as AI-friendliness' axis. Spec proposes a two-domain IR for multi-agent collaboration; Koru proposes architectural primitives (event boundaries, mandatory branch handling) that aid AI reasoning at the language level. Both make architectural claims about AI-friendliness without shipping agent-authoring tooling.

*Detail page: https://agentlanguages.dev/languages/koru/  ·  Markdown companion: https://agentlanguages.dev/languages/koru.md*

## Spec

> v0.2 design proposal for a language-agnostic IR for agent-driven development. Six specialised agents (Product, Architect, Scrum, Developer, Tester, DevOps) collaborate on shared .spec.ir artefacts; language-specific code generation is downstream.

**Camp:** Unclassified
**Also spans:** Orchestration
**Author:** M. Abdullah Onus
**Implementation language:** TypeScript (React POC)
**Compilation target:** Not applicable — IR artefacts (.spec.ir files), not executable
**Licence:** MIT
**First seen:** April 2026
**Maturity:** thought experiment
**Site:** https://github.com/mronus/spec
**Repo:** https://github.com/mronus/spec
**Agent tooling:**
- Browser-based React/TypeScript POC at mronus.github.io/spec orchestrating six specialised agents end-to-end
- Multi-agent pipeline with feedback loops, state persistence, and resume support
- Support for Claude and GPT models; API keys remain in the browser

### Key idea

Spec separates concerns into two domains. The Spec Domain is language-agnostic: six specialised agents (Product, Architect, Scrum, Developer, Tester, DevOps) collaborate to produce a set of .spec.ir artefacts — contract, module, infrastructure, data, types, interfaces, functions, events, tests, pipeline — that describe what the system should do. The External Agents Domain is language-specific: separate language agents (Java, Go, Terraform, etc.) consume the IR and produce code. The bet is that this separation lets one specification target multiple languages and enables incremental modification with the proposal's claimed 200 tokens of context instead of the 1,500 a comparable Java change requires.

## What it is.

Spec is a draft language design proposal at v0.2, not a working compiler. The repository ships a design document (`spec-language-design-proposal-v0.2.md`), a browser-based React/TypeScript proof-of-concept that orchestrates the agent pipeline against Claude or GPT, and a README that explicitly labels the project as a draft for discussion. The IR format defines ten artefact types &mdash; contract.spec.ir, module.spec.ir, infrastructure.spec.ir, data.spec.ir, types.spec.ir, interfaces/*.spec.ir, functions/*.spec.ir, events.spec.ir, tests.spec.ir, pipeline.spec.ir &mdash; each owned by a specific agent role (Product, Architect, Scrum, Developer, Tester, DevOps). The external language agents that would translate IR to running code in Java, Go, Rust, Terraform, or other targets are listed in the future-work section as not yet implemented.

## Why it's here.

The catalogue includes Spec as a marker of an architectural position that spans into orchestration. Where the syntactic and verification camps argue about what an agent should write, Spec argues about who should write what, and in what order &mdash; Product before Architect before Developer, with explicit IR handoffs between roles. The distinctive move is to treat the multi-agent pipeline as the primary artefact and language-specific code generation as a downstream concern that can be deferred. The catalogue does not rate Spec against working compilers. It marks it as a different kind of evidence: a structured argument that "language for agents to write" might be the wrong unit of analysis, and that "IR for agents to coordinate over" is the unit that matters.

### Design DNA

- **Boruna** *(Orchestration)* — Cross-camp neighbour. Boruna runs DAG workflows with policy-gated effects and hash-chained evidence; Spec coordinates specialist agents producing shared IR artefacts. Both treat the language as one layer in a larger orchestration story.
- **Pel** *(Orchestration)* — Academic-leaning kin. Both propose architectures for agent collaboration that have not yet shipped a usable language — Pel as a CMU paper, Spec as a draft proposal with a browser-based POC.

*Detail page: https://agentlanguages.dev/languages/spec/  ·  Markdown companion: https://agentlanguages.dev/languages/spec.md*

## Valea

> An AI-native systems programming language announced on Hacker News in March 2026. The Rust MVP compiler ships JSON diagnostics, a JSON AST exporter, a formatter, and a C backend. Public information beyond the repository README is limited.

**Camp:** Unclassified
**Author:** Hans Voetsch (Google)
**Implementation language:** Rust
**Compilation target:** C (via the emit-c command)
**Licence:** MIT
**First seen:** March 2026
**Maturity:** early implementation
**Site:** https://github.com/hvoetsch/valea
**Repo:** https://github.com/hvoetsch/valea
**Agent tooling:**
- JSON-emitting diagnostics (`valea check --json`) with stable error codes such as E001
- JSON AST export (`valea ast --json`)
- Canonical formatter (`valea fmt`)
- AGENTS.md and CLAUDE.md present in the repository for agent orientation

### Key idea

Valea declares five properties as its design surface: deterministic syntax, explicit semantics with no hidden allocations or exceptions, machine-readable diagnostics, canonical formatting, and a small language surface to reduce edge cases. The README sketches the intended workflow as an agent receiving a goal, writing Valea code, reading JSON diagnostics from the compiler, applying fixes, and producing a program that compiles and runs.

## What it is.

We do not have enough public information to classify Valea, and that honesty is itself the entry's defining quality. What is publicly available is a Hacker News post from March 2026 titled "Valea: An AI-native systems programming language," a GitHub repository at `hvoetsch/valea`, and a README that frames the project as a community language experiment. The README lists five design principles (deterministic syntax, explicit semantics, machine-readable diagnostics, canonical formatting, small language surface) and an example function in a Rust-flavoured surface. The compiler is a Rust MVP exposing four subcommands: `check` (with JSON output), `ast` (with JSON output), `fmt`, and `emit-c`. The repository contains a SPEC.md, MANIFESTO.md, ROADMAP.md, AGENTS.md, and CLAUDE.md, with 24 commits and a single demo recording on asciinema. Licence is MIT. The Google affiliation listed on the catalogue card is not stated in the repository itself.

## Why it's here.

The catalogue includes Valea as a marker of the noise floor of the field. Projects with this much intent, this little code, and this much manifesto are now common enough that the catalogue has an unclassified bucket for them. The relevant observation is not what Valea is but that an entry like this exists at all &mdash; that "AI-native systems programming language" has become a recognisable category on Hacker News with a recognisable shape (Rust compiler, JSON diagnostics, agent-oriented README files). The catalogue does not rate Valea against working compilers with measured benchmarks. It marks it as a different kind of evidence: an early signal that the design vocabulary of the field is stabilising even where the implementations have not yet shipped enough to evaluate.

*Detail page: https://agentlanguages.dev/languages/valea/  ·  Markdown companion: https://agentlanguages.dev/languages/valea.md*

---

## See also

- Homepage (HTML): https://agentlanguages.dev/
- Homepage (markdown): https://agentlanguages.dev/index.md
- Short index: https://agentlanguages.dev/llms.txt
- Sitemap: https://agentlanguages.dev/sitemap.xml
- Source repository: https://github.com/aallan/agentlanguages
