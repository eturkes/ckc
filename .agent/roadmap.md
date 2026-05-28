# CKC Roadmap

Phases contain top-level tasks; top-level tasks may contain indented subtasks.
See `.agent/prompt.md` for the session workflow that governs progression.

## Phase 0: Proof-Carrying Research Kernel

- [x] 0.1 Project scaffold
- [x] 0.2 CKC schema v0 (SPEC 10)
- [x] 0.3 Normal Form (SPEC 11)
- [x] 0.4 Content-addressed store
- [x] 0.5 Toy fixtures (SPEC 20 Phase 0 scenarios)
- [x] 0.6 Terminology and e-graph
- [ ] 0.7 Retrieval fixture
  - [x] 0.7.1 Retrieval types and ArtifactKind extension: populate ckc-retrieve with core retrieval types. Define `AnalyzerConfig` (name, dictionary, mode), `RetrievalQuery` (query_id, query_text, language, analyzer_config), `RetrievalHit` (span_id, score, rank), `RetrievalResult` (query, hits, index_fingerprint, corpus_hash), and `QrelJudgment` (query_id, span_id, relevance grade). Add `ArtifactKind::RetrievalResult` to ckc-core/envelope.rs. Derive Serialize/Deserialize/JsonSchema for all types. Update ArtifactKind golden schema tests. Add ckc-core as ckc-retrieve dependency. Gate: `cargo test -p ckc-retrieve` passes with round-trip serde tests for every type; ArtifactKind schema golden updated; all workspace tests pass.
  - [x] 0.7.2 Tantivy/Lindera Japanese sparse index: add `tantivy` and `lindera-tantivy` (ipadic feature) as workspace dependencies. Implement `SparseIndex` in ckc-retrieve: build from `Vec<SourceSpan>` into a Tantivy in-memory index with fields span_id (stored), search_text (indexed+stored, Lindera IPAdic tokenizer), nfkc_text (indexed, Lindera IPAdic tokenizer), doc_id (stored), section_path (stored). BM25 ranked search: accept query text, return top-k `Vec<RetrievalHit>`. Index fingerprint: SHA-256 of sorted (span_id, content_hash) pairs, deterministic regardless of insertion order. Unit tests: index all 17 toy spans; query "敗血症 βラクタム 投与" returns span_rec_sepsis_bl in top 3; query "体温" returns vitals table-cell spans; query "アナフィラキシー 禁忌" returns span_contra_bl_allergy; index fingerprint identical across two independent builds from same spans. Gate: `cargo test -p ckc-retrieve` passes; Lindera IPAdic morphological analysis produces expected token counts; BM25 returns relevant spans for JA queries; fingerprint is deterministic.
  - [ ] 0.7.3 Toy queries, qrel evaluation, CAS integration, and pipeline test: define 6–8 toy queries in JA targeting Phase 0 scenarios (sepsis recommendation, beta-lactam contraindication, allergy history, vitals thresholds, provenance metadata, terminology variants). Create qrel fixture JSON with manual relevance judgments (query_id → [(span_id, relevance_grade)]). Implement evaluation metrics: Recall@k (k=1,3,5), MRR, nDCG@5. Run full pipeline in integration test: load toy spans → build SparseIndex → execute all queries → produce RetrievalResult per query → evaluate against qrels → store RetrievalResult artifacts via CAS → verify manifest determinism across two independent runs. Commit qrel fixture at `examples/toy_research_kernel/fixtures/qrels.json` and retrieval result fixtures. Gate: integration test passes end-to-end; expected top spans retrieved per query; MRR > 0.5 for well-formed queries; all RetrievalResult artifacts stored and content-hashed in CAS; manifest hash deterministic; all workspace tests pass.
- [ ] 0.8 Compiler targets (SPEC 14)
- [ ] 0.9 Verification and certificates (SPEC 13)
- [ ] 0.10 Conflict detection (SPEC 15)
- [ ] 0.11 CLI commands (SPEC 18)
- [ ] 0.12 Report and UI
- [ ] 0.13 Replay and determinism
