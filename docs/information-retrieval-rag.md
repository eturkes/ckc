# Category 7: Information Retrieval and Retrieval-Augmented Generation

## 1. Hybrid Retrieval: BM25 + Dense Embeddings

### Purpose
Combine sparse lexical (BM25/BM25+/BM25F) + dense vector retrieval (DPR, ANCE, Contriever) to recover both exact-token matches (drug names, ICD codes, gene symbols) and semantic paraphrase (synonyms, cross-lingual near-matches) in one ranked list.

### Maintainer / Standards body
No single body. BM25 from Robertson/Sparck-Jones (Okapi BM25, 1994). OSS impls: Pyserini (U Waterloo, Jimmy Lin), Anserini (Lucene), OpenSearch (AWS fork of Elasticsearch), Vespa (Yahoo/Vespa.ai), Qdrant, Weaviate, Elastic. RRF (Reciprocal Rank Fusion) from Cormack, Clarke & Büttcher (SIGIR 2009).

### Conceptual model
Two parallel retrievers produce `L_sparse`, `L_dense` over same corpus. Fusion: RRF `1/(k+rank)`, `k=60` by convention; or weighted sum of normalized scores; or relative score fusion. Top-N fused → cross-encoder reranker (typical).

### Expressiveness / Semantics
Lexical channel preserves verbatim token identity (critical for ICD-10-JP codes, JAN drug codes, kana-only generic names). Dense channel captures paraphrase, multilingual alignment, abbreviation expansion. RRF is ordinal — scale-invariant but discards score magnitude; weighted sum requires score normalization.

### Composability / Modularity
Modular: retrievers, fusion, reranking separable. Standard in BEIR/MTEB pipelines. Major vector DBs (Vespa, Weaviate, Qdrant, OpenSearch 2.x, Elasticsearch 8.x) ship native hybrid query DSLs + built-in RRF.

### Suitability for autoformalization to IR
High. Recommended first stage per BGE-M3 paper: "We recommend to use the following pipeline: hybrid retrieval + re-ranking." Idempotency: BM25 fully deterministic (Lucene analyzer + index = bit-stable); dense recall deterministic given fixed model weights + ANN index params. Akarsu et al. (arXiv:2604.01733, Apr 2026, "From BM25 to Corrective RAG: Benchmarking Retrieval Strategies for Text-and-Table Documents") on a 23,088-query financial QA benchmark: hybrid RRF + Cohere Rerank achieves Recall@5 = 0.816, MRR@3 = 0.605, "outperforming all single-stage methods by a large margin"; MRR@3 rises from BM25 baseline 0.433 (+39.7%). Citation grounding: lexical hits preserve verbatim spans for source-span attribution.

### Formal verification potential
Auditable per-stage: BM25 scoring closed-form; dense similarity (cosine, dot) deterministic; RRF aggregation closed-form. Per-query relevance measurable against gold qrels (nDCG@10, Recall@k, MRR). Index reproducibility verifiable via fingerprinting (Lucene segment hashes).

### Tooling / Ecosystem maturity
Pyserini 2.1k GitHub stars (castorini/pyserini), latest release pyserini-2.0.0 (Apr 2026). OpenSearch Neural Search plugin (2024–2026 releases ship native RRF), Vespa hybrid ranking, Weaviate hybrid module, Qdrant `query_points` with prefetch + fusion (1.10+, 2024). BEIR/MTEB leaderboards exhaustively benchmark hybrid configs.

### Japan-specific considerations
BM25 requires a Japanese analyzer: Lucene `kuromoji`, MeCab+IPAdic/UniDic, or SudachiPy; segmentation choice changes recall significantly. JaColBERTv2.5 reproducibility work used Lucene Analyzer vs xlm-roberta tokenizer, reported notable differences. Mixed kanji/kana/Latin (e.g., "COVID-19 後遺症") demands analyzer-aware normalization (NFKC, half/full-width unification).

### Interoperability
Cat 1–2 (FHIR/CQL/openEHR/DMN/BPMN): docs are FHIR `Bundle`/`DocumentReference` payloads; retrieved spans cited by FHIR Provenance. Cat 3 (Minds/JP Core/PMDA): lexical channel preserves PMDA package-insert verbatim wording. Cat 4 (OWL/SHACL): candidates linked to UMLS/SNOMED/MEDIS concepts via downstream NEL. Cat 5–6 (Lean/Rocq/SMT/ASP/defeasible/temporal): retrieval upstream; IR layer receives ranked evidence with attribution.

### Limitations / Known issues
RRF discards score magnitude. Weighted sum requires per-corpus tuning. Query-time/index-time tokenizer mismatch silently degrades sparse recall. Dense ANN recall depends on `ef_search`/`nprobe`; non-deterministic if HNSW build seeds differ.

### Training data proxy
Massive: BM25 decades of academic + StackOverflow coverage; DPR/Contriever HF cards >1M downloads; Pyserini/Vespa/OpenSearch hybrid examples abundant on GitHub. Stack Overflow tags `[bm25]`, `[opensearch]`, `[elasticsearch]`, `[vespa]` all active.

## 2. Multilingual Biomedical Embeddings and Rerankers (BGE-M3, Multilingual-E5, Jina, BioBERT, PubMedBERT, MedCPT, JMedRoBERTa, JaColBERTv2)

### Purpose
Multilingual, biomedically-tuned dense (optionally sparse + multi-vector) representations for query/document encoding and cross-encoder reranking of retrieved candidates.

### Maintainer / Standards body
BGE-M3, BGE-reranker-v2-m3: BAAI (Beijing Academy of Artificial Intelligence). Multilingual-E5: Microsoft / `intfloat`. Jina embeddings v3/v4: Jina AI. BioBERT: Korea Univ. (DMIS lab). PubMedBERT: Microsoft Research. MedCPT: NCBI/NLM (`ncbi/MedCPT`). JMedRoBERTa: alabnii (Aizawa Lab, NII / U-Tokyo). JaColBERT/JaColBERTv2/v2.5: Benjamin Clavié / Answer.AI (`bclavie/JaColBERTv2`, `answerdotai/JaColBERTv2.5`). Cohere Rerank: Cohere. mxbai-rerank: Mixedbread AI.

### Conceptual model
Bi-encoder for retrieval (typ. 1024-d dense, e.g. BGE-M3); cross-encoder for reranking (concatenated `[query; doc]` → scalar relevance). BGE-M3 uniquely produces dense + sparse (lexical weights) + multi-vector (ColBERT-style) from a single XLM-RoBERTa backbone, max 8192 tokens, 100+ languages.

### Expressiveness / Semantics
BGE-M3 dense: single 1024-d vector; sparse: token-weight dict (BM25-compatible); multi-vector: per-token embeddings with MaxSim. MedCPT retriever + reranker contrastively co-trained on 255M PubMed query–article pairs (retriever) and 18M (reranker); PubMedBERT-initialised.

### Composability / Modularity
Hot-swappable encoders behind sentence-transformers / FlagEmbedding APIs. Reranker stage independent. BGE-M3's three output modes can be fused (e.g., `0.4*dense + 0.2*sparse + 0.4*colbert`).

### Suitability for autoformalization to IR
Very high for Japanese biomedical CDS. Per Chen, Xiao, Zhang, Luo, Lian & Liu (BAAI / USTC, 2024), arXiv:2402.03216, "BGE M3-Embedding: Multi-Lingual, Multi-Functionality, Multi-Granularity Text Embeddings Through Self-Knowledge Distillation": "BGE-M3 achieved the highest average ranking score (nDCG@10 = 70.0 using all modes) across languages, outperforming the best prior multi-lingual embedder (mE5, ~65.4)"; dense-only 67.8. MKQA cross-lingual recall@100 = 75.5%. MedCPT SOTA on 3/5 BEIR biomedical tasks (Jin et al., Bioinformatics 2023, btad651; arXiv:2307.00589). JMedRoBERTa trained on Japan Science and Technology Agency academic-medical abstracts (≈10M sentences, 1.6GB) plus body text (1.4M sentences, 0.2GB). Idempotency: deterministic given fixed model weights, float precision, batch size.

### Formal verification potential
Embedding outputs deterministic (modulo FP nondeterminism). Recall/precision auditable against MIRACL-ja, JSQuAD, JaCWIR, JaQuAD. Cross-encoder rerankers expose pointwise scores enabling threshold-based gating.

### Tooling / Ecosystem maturity
BGE-M3: `BAAI/bge-m3` on HF, long-standing strong performer on MTEB/MIRACL leaderboards; FlagEmbedding repo active. As of April 2026 MTEB top spots shifted to Qwen3-Embedding-8B (Apache 2.0; 70.58 multilingual avg) and Gemini Embedding 001 (68.32 English avg) / NV-Embed-v2 (72.31 English avg), but BGE-M3 remains competitive for hybrid dense+sparse+multi-vector multilingual retrieval and still widely deployed. Multilingual-E5: `intfloat/multilingual-e5-large` widely deployed. JaColBERTv2.5: 110M params, trained 15h on 4× A100 with 3.2M triplets, "outperforms all existing models on all benchmarks, at any evaluation scale" per Answer.AI 2024-08 release. MedCPT: `ncbi/MedCPT-Query-Encoder` / `-Article-Encoder` / `-Cross-Encoder` on HF.

### Japan-specific considerations
JMedRoBERTa (`alabnii/jmedroberta-base-sentencepiece`, `…-manbyo-wordpiece`): RoBERTa-base, ~110M params, CC BY-NC-SA 4.0 (non-commercial), latest weights committed March 2023 (Sugimoto, Iki, Chida, Kanazawa, Aizawa, NLP 2023). JaColBERTv2 trained on Japanese MMARCO; JaColBERTv2.5 fine-tuned on higher-quality datasets including MIRACL-ja. BGE-M3 MIRACL-ja dev nDCG@10 ≈ 72.8 (dense). For commercial CDS deployment, JMedRoBERTa's NC-SA license is blocking — prefer BGE-M3 (MIT) or JaColBERTv2.5 (MIT).

### Interoperability
Encoders output to any vector store (Qdrant, Weaviate, Vespa, Milvus, pgvector). Reranker scores feed Cat 4 SHACL/OWL confidence weights or Cat 6 defeasible-logic strength priors. Embeddings index FHIR `DocumentReference.content` and openEHR archetype free-text fields.

### Limitations / Known issues
JMedRoBERTa non-commercial license. BioBERT/PubMedBERT English-only — useless for Japanese guideline body text but useful for cross-lingual PubMed evidence pull. BGE-M3 training data unbalanced across languages. Cross-encoder rerankers O(N) per query, cost-prohibitive past N≈100.

### Training data proxy
BGE-M3: top-tier HF presence, MTEB leaderboard fixture, hundreds of derivative papers 2024–2026. PubMedBERT/BioBERT: thousands of citations. JMedRoBERTa: niche, limited Stack Overflow presence. JaColBERT family: covered in Answer.AI blog and arXiv:2407.20750.

## 3. ColBERT / Late-Interaction Retrieval

### Purpose
Multi-vector retrieval representing each document and query as a bag of token-level contextualised embeddings, scored by MaxSim, for cross-encoder-like effectiveness at bi-encoder-like latency.

### Maintainer / Standards body
ColBERTv1/v2: Khattab, Santhanam, Zaharia (Stanford / Future Data Systems). PLAID: Santhanam, Khattab, Potts & Zaharia, CIKM 2022 (arXiv:2205.09707). RAGatouille: Benjamin Clavié / Answer.AI. JaColBERT/JaColBERTv2/v2.5: Benjamin Clavié. Aya-ColBERT: Cohere For AI lineage.

### Conceptual model
For query token `q_i` and doc token `d_j`, compute `cos(q_i, d_j)`; document score = `Σ_i max_j cos(q_i, d_j)` (MaxSim / late interaction). ColBERTv2 adds residual compression (centroid + 2-bit residuals) for storage efficiency. PLAID adds centroid pruning + centroid interaction for ≤7× GPU and ≤45× CPU speedup vs vanilla ColBERTv2 at unchanged quality.

### Expressiveness / Semantics
Token-level matching handles compositional queries (e.g., "ARB + 妊娠中の禁忌") where single-vector pooling smears critical token signals. Strong out-of-domain generalisation: JaColBERT outperformed multilingual-e5-large on JSQuAD despite e5 being in-domain.

### Composability / Modularity
PLAID engine is a drop-in for ColBERTv2 checkpoints. RAGatouille wraps training + index + search; JaColBERTv2.5 ships as a RAGatouille-compatible checkpoint. Indexes not interoperable with single-vector ANN stores (FAISS/HNSW) without adapters — Vespa and Qdrant support multi-vector but ColBERT-style scoring is bespoke.

### Suitability for autoformalization to IR
Very high for guideline retrieval. Token-level grounding supports citation-span attribution (each query token's MaxSim picks a specific document token, yielding evidence spans). JaColBERTv1 reaches Recall@10 = 0.813 (vs prior monolingual best 0.716; multilingual-e5-base 0.820; -large 0.856) per Clavié, arXiv:2312.16144, "Towards Better Monolingual Japanese Retrievers with Multi-Vector Models"; JaColBERTv2.5 surpasses all per Answer.AI 2024-08 (arXiv:2407.20750). Idempotency: deterministic given fixed weights, compression centroids, PLAID parameters.

### Formal verification potential
Per-token max-similarity inspectable; PLAID candidate generation parameters (`nprobe`, `ndocs`, centroid_threshold) make the retrieval funnel auditable. Evaluation against BEIR, LoTTE, MIRACL-ja, JSQuAD.

### Tooling / Ecosystem maturity
`stanford-futuredata/ColBERT` (active 2024–2025), `bclavie/RAGatouille`. Vespa 8.x native ColBERT support; Qdrant multi-vector preview (2024); Weaviate experimental. PLAID is the standard inference engine; SIGIR '25 work refines pruning and multi-vector retrieval beyond PLAID.

### Japan-specific considerations
JaColBERTv1 (10M Japanese MMARCO triplets, BERT-BASE-JAPANESE-v3 init), JaColBERTv2 (knowledge-distilled, 32-way negatives, 8M triplets, 20h on 8× A100), JaColBERTv2.5 (3.2M high-quality triplets including MIRACL-ja, 15h on 4× A100, 110M params). Answer.AI: "outperforms all existing models on all benchmarks, at any evaluation scale, and with just 110M parameters." Best monolingual Japanese retriever as of 2024-08.

### Interoperability
Token-span output is ideal feed for Cat 1 FHIR `Provenance.entity` and Anthropic Citations API custom-content chunks. Cat 4 NEL pipelines (each matched token → UMLS/J-MeSH concept). Cat 5–6 verification unaffected.

### Limitations / Known issues
Index size: even with 2-bit compression, ColBERT indexes ~5–10× larger than single-vector. Latency without PLAID poor; PLAID adds tuning surface. Multi-vector storage in commodity vector DBs immature. RAGatouille training requires ColBERT-specific data formatting.

### Training data proxy
ColBERT family: well-covered in ACL/SIGIR/EMNLP; PLAID at CIKM 2022 plus a 2024 reproducibility study (MacAvaney & Tonellotto, arXiv:2404.14989, SIGIR 2024). RAGatouille: active GitHub, frequent Answer.AI blog coverage 2024. JaColBERT family: niche but rising; primary references arXiv:2312.16144 and arXiv:2407.20750.

## 4. Recommendation-Level Segmentation: PICO, Evidence Tables, Recommendation Strength

### Purpose
Decompose clinical practice guideline narrative into atomic, machine-readable units: PICO (Population/Intervention/Comparator/Outcome) frames, GRADE evidence-to-decision tables, recommendation-strength tags (strong / weak / conditional, for / against).

### Maintainer / Standards body
PICO: Sackett et al. (EBM literature). GRADE: GRADE Working Group (gradeworkinggroup.org); GRADEpro GDT (Evidence Prime / McMaster). Japan: Minds Tokyo GRADE Center (Japan Council for Quality Health Care / 公益財団法人日本医療機能評価機構), Minds 診療ガイドライン作成マニュアル 2020 Ver.3.0. EBM-NLP corpus: Nye et al., ACL 2018 (with EBM-NLPmod, EBM-COMET, EBM-NLPrev, EBM-NLPh derivative datasets).

### Conceptual model
Guideline → CQ (Clinical Question) → PICO frame → Body of Evidence (per outcome, per study design) → EtD criteria (benefit/harm balance, certainty, patient values, resources, equity, acceptability, feasibility) → Recommendation (direction × strength). Minds method mandates three-layered committees (managing, guideline creation, systematic review), explicit COI, dual benefit/harm evaluation.

### Expressiveness / Semantics
GRADE certainty: high / moderate / low / very low. Strength: strong / weak (a.k.a. conditional). Empirical: "Probability of making strong recommendation was 62% when evidence is moderate, while it was only 23% and 13% when evidence was low or very low, respectively" (Alonso-Coello et al., BMC Health Services Research 2009; PMC2722589).

### Composability / Modularity
PICO frame is a 4-tuple amenable to JSON Schema / OWL class. EtD is a fixed criteria checklist amenable to DMN decision tables. Recommendation strength maps cleanly to deontic operators (strong = obligation, weak = permission / defeasible default).

### Suitability for autoformalization to IR
Excellent. PICO frames serve directly as IR predicates: `Population(patients with X) ∧ Intervention(Y) ∧ Comparator(Z) → Outcome(O, effect, certainty)`. PubMedBERT-fine-tuned NER pipelines achieve micro-F1 0.833 (token-level) / 0.712 (entity-level) on EBM-NLPmod; 0.928 / 0.850 on COVID-19 RCT abstracts (Hu, Keloth, Raja, Chen & Xu, Bioinformatics 39(9), btad542, 2023). AlpaPICO (Ghosh et al., arXiv:2409.09704, 2024) demonstrates LLM in-context PICO extraction. Idempotency requires fixed extractor + canonicalised PICO vocabulary.

### Formal verification potential
EtD frameworks inherently auditable — each judgment cell checkable against the underlying SR. Cross-guideline contradiction detection = constraint problem over PICO-keyed recommendations: two guidelines disagree iff identical PICO yields opposite-direction strong recommendations.

### Tooling / Ecosystem maturity
GRADEpro GDT (commercial), MAGICapp (MAGIC Evidence Ecosystem Foundation), Cochrane RevMan. EBM-NLP corpus on GitHub; PubMedBERT-PICO fine-tunes on HF. Minds: published Manual 2020 v3.0 (PDF), Minds Guideline Library website (minds.jcqhc.or.jp). EvidenceOutcomes (arXiv:2506.05380, 2025) provides refined Outcome annotations with IAA 0.76.

### Japan-specific considerations
Minds adopted GRADE in 2014 manual; current Ver.3.0 (2020). Minds Tokyo GRADE Center founded March 2019 to promote GRADE in Japan. Minds Guideline Library indexes Japanese society guidelines with AGREE II evaluations. PICO/CQ structure mandatory for Minds-listed guidelines. Japanese PICO NER corpora scarce — EBM-NLP is English-only; cross-lingual transfer via mDPR/BGE-M3 needed.

### Interoperability
PICO → Cat 1 FHIR `EvidenceVariable` resource (HL7 FHIR R5 EBMonFHIR IG). EtD → Cat 1 DMN tables. Strength tags → Cat 6 deontic / defeasible-logic priorities. Outcomes → Cat 4 OWL classes anchored to MedDRA/J, J-MeSH, ICD-10-JP, MEDIS standard disease master.

### Limitations / Known issues
PICO Outcomes have low inter-annotator agreement historically. Many Japanese society guidelines pre-date GRADE adoption and use bespoke recommendation grading (A/B/C1/C2/D — Japanese Minds 2007 scheme). Comparator field often implicit.

### Training data proxy
GRADE: very high (Cochrane, WHO, BMJ series). EBM-NLP family: substantial (GitHub `bepnye/EBM-NLP`, `BIDS-Xu-Lab/section_specific_annotation_of_PICO`). Minds: Japanese-language presence significant in JST/JSTAGE; English presence limited.

## 5. Layout-Aware Japanese PDF and Table Extraction

### Purpose
Convert clinical guideline PDFs (Minds, society guidelines, package inserts) into structured Markdown / JSON preserving reading order, vertical text, tables, formulas, figures, footnotes; necessary upstream of any retrieval or autoformalisation.

### Maintainer / Standards body
Yomitoku: Kotaro Kinoshita (`kotaro-kinoshita/yomitoku`, CC BY-NC-SA 4.0 OSS; commercial via MLism Inc.). MinerU: Shanghai AI Laboratory / OpenDataLab (`opendatalab/MinerU`; AGPL→permissive after 3.1.0). Marker: Datalab.to (Vik Paruchuri). Unstructured.io: Unstructured. PyMuPDF: Artifex. Tabula: Tabula Project. Camelot: Atlan / community. Adobe PDF Extract API: Adobe. Reducto, Mathpix: commercial. LLM vision: GPT-4V (OpenAI), Claude Vision (Anthropic), Gemini (Google). PaddleOCR-JP: PaddlePaddle.

### Conceptual model
Multi-stage pipeline: (1) page rendering, (2) layout detection (LayoutLMv3 / DocLayout-YOLO / DETR variants), (3) text detection + recognition (OCR), (4) table structure recognition (TableMaster, StructEqTable, table-transformer), (5) reading-order resolution, (6) export (Markdown/HTML/JSON). Yomitoku ships four Japanese-tuned models: text detection, text recognition (ParseQ-based), layout analysis, table structure recognition.

### Expressiveness / Semantics
Yomitoku supports >7,000 Japanese characters incl. vertical text (縦書き) and complex tables. MinerU 2.5 (vLLM-backed) supports formula → LaTeX, table → HTML, OCR across 84–109 languages. Marker excels at academic-style English layouts; less proven for Japanese.

### Composability / Modularity
Yomitoku CLI + Python API outputs JSON/CSV/HTML/Markdown with per-element configs (e.g., OCR on GPU + layout on CPU). MinerU offers CLI / FastAPI / Gradio / SDK. Composable with downstream chunkers (LlamaIndex `MarkdownNodeParser`).

### Suitability for autoformalization to IR
Critical preprocessing layer; output quality bounds downstream retrieval/IR. Yomitoku is currently the strongest open option for Japanese guideline PDFs with vertical text and society-specific layouts. Per Wang, Xu, Zhao et al. (Shanghai AI Lab, arXiv:2409.18839, 2024), "MinerU: An Open-Source Solution for Precise Document Content Extraction", Table 3 reports MinerU's LayoutLMv3-SFT layout detection at 77.6% mAP on academic paper validation set vs DocXchain at 52.8% mAP; formula CDM 0.968 (comparable to Mathpix 0.951). Idempotency: deterministic given fixed model weights; element coordinates reproducible.

### Formal verification potential
Bounding-box outputs and reading-order indices are inspectable artefacts. Round-trip checks: extract → re-render → diff vs source. Per-table cell extraction validated against pixel-level OCR confidence.

### Tooling / Ecosystem maturity
MinerU: 63.5k GitHub stars (opendatalab/MinerU, May 2026), latest release mineru-3.1.14 (May 15, 2026), native PPTX/XLSX support and vLLM integration. Yomitoku: actively maintained; commercial edition on AWS Marketplace (YomiToku-Pro, MLism Inc.). Unstructured.io: enterprise tier mature. Marker: rapidly evolving 2024–2026.

### Japan-specific considerations
Yomitoku is the only OSS engine specifically trained on Japanese document images with native vertical-text + 7000+ kanji coverage and supports handwritten text and 縦書き layouts unique to Japanese documents. MinerU's general OCR (PaddleOCR) handles Japanese but weaker than Yomitoku on dense vertical-text or handwritten cases. Furigana, ruby annotations, and mixed half/full-width characters require explicit normalization. Yomitoku core OSS is CC BY-NC-SA 4.0 — blocking for commercial CDS; commercial license required.

### Interoperability
Output Markdown/JSON feeds any chunker → Cat 1 FHIR `DocumentReference` / openEHR COMPOSITION. Tables → DMN inputs. Element bounding boxes → Anthropic Citations API custom-content blocks for verifiable source-span tracking.

### Limitations / Known issues
Yomitoku non-commercial OSS license. MinerU on Arabic / Hindi / Urdu reportedly unreliable; Japanese performance good but not Yomitoku-grade. Vision LLMs (GPT-4V, Claude, Gemini) hallucinate at the layout level. No tool consistently handles multi-page tables with merged cells across page breaks. Society guideline PDFs often embed scanned-image annexes requiring OCR.

### Training data proxy
MinerU: very high (63.5k stars, 2024 arXiv paper, 2026 vLLM integration). Yomitoku: niche, Japan-domestic Zenn/Qiita coverage. PaddleOCR: massive Chinese community. Adobe/Reducto/Mathpix: commercial, limited public benchmarks.

## 6. GraphRAG / Knowledge-Graph-Augmented Retrieval

### Purpose
Index documents by extracted entities and relations into a knowledge graph; retrieve via graph traversal, community summarization, or PageRank-style signals, complementing or replacing chunk-level vector retrieval for global / multi-hop / multi-document queries.

### Maintainer / Standards body
Microsoft GraphRAG: Microsoft Research (Edge, Trinh, Truitt, Larson; "From Local to Global: A Graph RAG Approach to Query-Focused Summarization", arXiv:2404.16130; codebase released on GitHub July 2024; Microsoft Research blog "GraphRAG: Unlocking LLM discovery on narrative private data" Feb 2024). LazyGraphRAG: Microsoft (Nov 2024). HippoRAG / HippoRAG 2: Bernal Jiménez Gutiérrez, Yiheng Shu, Yu Gu, Michihiro Yasunaga, Yu Su (OSU NLP Group, NeurIPS 2024; arXiv:2405.14831; HippoRAG 2 at ICML 2025). LightRAG: Guo, Xia, Yu, Ao, Huang (HKU Data Science, arXiv:2410.05779, Oct 2024; accepted EMNLP 2025).

### Conceptual model
Indexing: LLM extracts (entity, relation, entity) triples + descriptive summaries from each chunk; entities clustered into communities (Leiden algorithm in MS GraphRAG); per-community summaries pre-generated. Query time: global queries use map-reduce over community summaries; local queries use entity-anchored subgraph + chunk retrieval. HippoRAG uses Personalized PageRank over an extracted KG to mimic hippocampal indexing. LightRAG uses dual-level (entity / relationship) keyword retrieval.

### Expressiveness / Semantics
Captures inter-document relations and global structure beyond chunk-level cosine similarity. HippoRAG outperforms IRCoT-style iterative retrieval on multi-hop QA at 10–30× lower cost (per HippoRAG NeurIPS 2024 paper and OSU-NLP-Group/HippoRAG repo).

### Composability / Modularity
MS GraphRAG: modular CLI + Python; KG schemaless by default but user-extendable. LightRAG: dual-level retrieval easily swappable with other graph stores. Compatible with Neo4j, NetworkX, kuzu, ArangoDB.

### Suitability for autoformalization to IR
High for cross-guideline contradiction detection. Triples extracted from multiple guidelines become directly comparable: contradiction = two triples with same `(subject, predicate)` but conflicting `object` or modality. However, idempotency is a major risk — LLM extraction non-deterministic; entity disambiguation drifts across runs unless constrained by a fixed ontology (UMLS, J-MeSH, MEDIS).

### Formal verification potential
Graph artefacts inspectable and diff-able. Community detection (Leiden) deterministic given seed. Triple-level provenance preserved by MS GraphRAG. Allows contradiction queries via SPARQL/Cypher. Schema-constrained variants (e.g., OG-RAG) reduce hallucinations through ontology grounding.

### Tooling / Ecosystem maturity
`microsoft/graphrag` v3.0.x (v3.0.0 released January 28, 2026 introducing a monorepo restructure with new packages; subsequent v3.0.x patch releases through April 2026), 30.8k GitHub stars (May 2026); HippoRAG (`OSU-NLP-Group/HippoRAG`, NeurIPS 2024); LightRAG widely forked. Awesome-GraphRAG (DEEP-PolyU) curates the field. Neo4j published "GraphRAG Manifesto" 2024.

### Japan-specific considerations
LLM-based entity/relation extraction on Japanese guideline text works with GPT-4o / Claude / Gemini but accuracy on Japanese disease-name and drug-name normalization is materially lower than English. Pre-anchor entities to MedNER-J + MEDIS-DC standard disease master (ICD10対応標準病名マスター Ver.5.14, June 2024) + HOT医薬品マスター before triple extraction. Community summaries should be generated in Japanese to preserve clinical nuance.

### Interoperability
Native fit with Cat 4 (OWL/SHACL/ontology) — KG exportable to RDF/OWL with SHACL constraints. Triples feed Cat 5 (Lean/Rocq/Isabelle/TLA+) facts. Cat 6 deontic/defeasible/argumentation: recommendation triples become defeasible rules with strength priorities. FHIR mapping: entities ↔ `Condition`, `Medication`, `Procedure`.

### Limitations / Known issues
Indexing cost: O(N) LLM calls for triple extraction, prohibitive at scale per Edge et al. and 2024–2025 surveys. Entity duplication: "LLMs" vs "LLM" vs "Large Language Models" — string-matching merge leaves duplicates, degrades retrieval efficiency. Schemaless KGs lack a true "world view"; ontology-grounded variants needed for medical CDS.

### Training data proxy
GraphRAG: extensive Microsoft Research blog series (Feb/July/Sep/Oct/Nov/Dec 2024, 2025 Discovery launch); covered by Neo4j, Dagstuhl, DEEP-PolyU survey. HippoRAG: NeurIPS 2024 oral profile. LightRAG: active GitHub.

## 7. Query Decomposition and Retrieval Routing

### Purpose
Decompose complex clinical queries into sub-questions, route each to the most appropriate retriever / index / tool, and synthesize answers, enabling multi-hop QA, conditional retrieval, corrective loops.

### Maintainer / Standards body
Self-RAG: Asai, Wu, Wang, Sil & Hajishirzi (UW / IBM AI, ICLR 2024 Oral; arXiv:2310.11511). Plan-and-Solve: Wang et al. (ACL 2023; arXiv:2305.04091). LangGraph: LangChain Inc. LlamaIndex Workflows / AgentQueryEngine: LlamaIndex (Jerry Liu). Adaptive RAG: Jeong et al., NAACL 2024 (arXiv:2403.14403). Corrective RAG (CRAG): Yan, Gu et al., 2024 (arXiv:2401.15884). Reflexion: Shinn et al., NeurIPS 2023 (arXiv:2303.11366). Microsoft AutoGen: Microsoft Research.

### Conceptual model
A stateful agent graph: nodes for retrieve, grade, decompose, rewrite, generate, critique. Routing strategies: semantic routing (embedding similarity to route description), classifier-based (fine-tuned router), LLM-driven (function calling). Self-RAG inserts retrieval-control tokens learned during fine-tuning. LangGraph models the entire agent as an explicit state machine with checkpointing; LlamaIndex Workflows uses event-driven `@step` functions.

### Expressiveness / Semantics
Captures conditional logic ("if PICO-Population is pediatric, route to JSP guidelines; else route to JSIM"), iterative refinement, human-in-the-loop. Sub-questions can be parallel (independent) or sequential (dependency-ordered, e.g., resolve drug name → query interactions).

### Composability / Modularity
Both LangGraph and LlamaIndex Workflows compose retrievers from §1–3, citation generators from §8, evaluators from §10 into a single DAG. Tool abstraction (LangChain `@tool`, LlamaIndex `QueryEngineTool`) allows mixing keyword search, vector search, SQL, SPARQL, Lean proof checker.

### Suitability for autoformalization to IR
High when the autoformalisation pipeline itself benefits from explicit planning (e.g., "extract PICO → find supporting evidence → check GRADE → emit defeasible rule"). Idempotency challenging — LLM-driven decomposition stochastic; mitigations: `temperature=0`, fixed seeds, response-format constraints, deterministic routers. Citation grounding improves: each sub-question's retrieval can be audited.

### Formal verification potential
LangGraph's explicit state-machine model is inspectable and replayable; checkpoints enable time-travel debugging. Each routing decision and tool call is loggable. Evaluators (RAGAS, TruLens) score per-node outputs. CRAG-style retrieval grading produces machine-readable verdicts (correct/ambiguous/irrelevant).

### Tooling / Ecosystem maturity
LangGraph: stable 1.x (GA October 2025); widely adopted in agentic RAG production stacks (2024–2026). LlamaIndex Workflows: GA 2024; mature RAG primitives. AutoGen v0.4 (2025). Self-RAG: open-source reference implementation.

### Japan-specific considerations
Japanese queries often blend kanji-only medical jargon with kana paraphrase ("リウマチ" / "関節リウマチ" / "RA"); query rewriting / expansion to all variants before routing materially improves recall. Routers should be trained or prompted with Japanese examples; off-the-shelf English semantic routers underperform on Japanese embeddings.

### Interoperability
Tool nodes can wrap any Cat 1–6 component: FHIR `$cql` evaluation, openEHR AQL, SMT solver, Lean proof step, SHACL validator, defeasible-logic engine. State machine becomes the orchestration layer above the IR.

### Limitations / Known issues
Latency multiplier: each sub-question costs an additional LLM call + retrieval roundtrip. Failure-mode amplification: hallucinated sub-questions propagate downstream. LangGraph has steeper learning curve than LlamaIndex Workflows per 2025 community comparisons. Non-determinism in routing decisions complicates regression testing.

### Training data proxy
LangGraph, LlamaIndex: massive blog/Medium/Stack Overflow presence; official docs and >100 reference notebooks. Self-RAG, CRAG, Adaptive RAG: well-cited 2024 papers, multiple implementations on HF and GitHub.

## 8. Citation-Grounded Generation with Source-Span Tracking

### Purpose
Constrain generation to cite specific source spans for each claim, enabling verifiability, hallucination mitigation, downstream attribution audit. Essential for CDS legal/regulatory defensibility.

### Maintainer / Standards body
ALCE: Tianyu Gao, Howard Yen, Jiatong Yu, Danqi Chen (Princeton NLP, EMNLP 2023, arXiv:2305.14627). AIS (Attributable to Identified Sources) framework: Rashkin et al. 2021/2023 (arXiv:2112.12870; Computational Linguistics 49(4):777–840); Bohnet et al. 2022 follow-up on attributed QA modeling. Anthropic Citations API: Anthropic (GA January 2025, AWS Bedrock June 2025). Perplexity: Perplexity AI. Nelson Liu, Tianyi Zhang & Percy Liang ("Evaluating Verifiability in Generative Search Engines", Findings of EMNLP 2023, arXiv:2304.09848). LongCite / LongBench-Cite: THUDM 2024 (arXiv:2409.02897).

### Conceptual model
Two paradigms: (a) post-hoc citation — generate answer, then retrieve evidence and align spans; (b) inline / pre-hoc citation — model emits answer with `[doc_id]` markers during decoding. Anthropic Citations API chunks plain-text documents into sentences (or custom blocks) and returns per-claim citation arrays with character-index ranges for plain-text and page-number ranges for PDFs.

### Expressiveness / Semantics
ALCE metrics: Fluency (MAUVE), Correctness (claim recall), Citation Quality (recall + precision via NLI). Citation recall: every claim must be entailed by its cited passages; citation precision: every cited passage must be relevant. Anthropic API guarantees `cited_text` is a verbatim substring (no lossy rephrasing).

### Composability / Modularity
Anthropic Citations API is a single-call wrapper; LongCite-8B/9B provide open-weight alternatives. Composable with any RAG retriever upstream and any RAGAS/TruLens evaluator downstream.

### Suitability for autoformalization to IR
Excellent. Source-span attribution is exactly what an autoformalisation IR needs: each IR predicate (PICO frame, recommendation, rule) annotated with `(doc_id, span_start, span_end, verbatim_quote)`. Per Anthropic's official Citations launch blog (January 2025): "Our internal evaluations show that Claude's built-in citation capabilities outperform most custom implementations, increasing recall accuracy by up to 15%." Endex (cited in same launch) reported "reduced source hallucinations and formatting issues from 10% to 0% and saw a 20% increase in references per response." Idempotency: deterministic given fixed model + temperature=0 + chunking policy.

### Formal verification potential
Per-claim NLI check automatable. Citation recall/precision are direct benchmark metrics on ALCE / LongBench-Cite. GPT-4o on LongBench-Cite achieves citation recall 75.0% / citation precision 88.8% against human-annotated gold (Zhang et al., LongCite, arXiv:2409.02897, 2024).

### Tooling / Ecosystem maturity
Anthropic Citations API: GA Jan 2025 on Anthropic API + Vertex AI; added to Bedrock June 2025; cookbook examples in `anthropics/anthropic-cookbook`. ALCE: standard benchmark since 2023, ASQA + QAMPARI + ELI5 splits. LongCite-45k SFT dataset publicly released. LongBench-Cite added 2024.

### Japan-specific considerations
Anthropic Citations API supports Japanese plain-text and PDF citations (sentence chunking is character-aware via Claude tokenizer; works on Japanese with caveats around sentence-boundary detection where Japanese 句点 「。」 is reliable but some clinical PDFs lack periods after section headings). For furigana-annotated kanji, citation spans may include ruby characters depending on extraction layer (see §5). Citing PMDA package-insert specific clauses or Minds CQ text directly satisfies Japanese regulatory traceability requirements.

### Interoperability
Citations naturally map to Cat 1 FHIR `Provenance` resources (`Provenance.entity.what = DocumentReference`, `Provenance.entity.role = source`). Anthropic Citations custom-content blocks pair well with §3 ColBERT span outputs and §5 layout-extracted bounding boxes. For Cat 5 (Lean/Rocq), each lemma/axiom can be tagged with its source citation.

### Limitations / Known issues
ALCE NLI-based evaluation limited by NLI model accuracy; cannot detect "partial support" — per Gao et al. 2023 ALCE paper. MAUVE sensitive to output length. Post-hoc citation underperforms pre-hoc when the model is citation-capable (LongCite findings). Custom-chunking required for granularity below sentence (e.g., per-bullet). PDF-image citations not yet supported by Anthropic API ("Citing images from PDFs is not currently supported").

### Training data proxy
Anthropic Citations API: official docs, cookbook, Simon Willison coverage, AWS announcements, integrations in LiteLLM, Spring AI. ALCE: highly cited (2023–2026 ACL/EMNLP). LongBench-Cite / LongCite: active 2024 release.

## 9. Japanese-English Cross-Lingual Alignment for Terminologies and Evidence

### Purpose
Bridge Japanese guideline content with English-language evidence corpora (PubMed, Cochrane, UpToDate) and international terminologies, via cross-lingual retrieval and terminology mapping.

### Maintainer / Standards body
J-MeSH: NPO Japan Medical Abstracts Society (JAMAS / 医学中央雑誌刊行会). MEDIS-DC: 一般財団法人医療情報システム開発センター (maintains ICD10対応標準病名マスター, HOTマスター, 手術・処置マスター, 臨床検査マスター, 看護用語マスター, 歯科病名マスター). MedDRA/J: JMO (Japanese Maintenance Organization) within PMRJ (一般財団法人医薬品医療機器レギュラトリーサイエンス財団). JADER: PMDA. ICD-10-JP / ICD-11 Japanese: MHLW 政策統括官付参事官付国際分類情報管理室. SNOMED CT: SNOMED International — note Japan is not a member; SNOMED is not nationally adopted. MIRACL: Zhang, Thakur, Ogundepo, Kamalloo, Alfonso-Hermelo, Li, Liu, Rezagholizadeh & Lin (U Waterloo et al., TACL 2023). NLLB-200: Meta AI. mBART/mT5: Meta / Google. MarianMT: Helsinki-NLP. M2M-100: Meta.

### Conceptual model
Cross-lingual dense retrieval (mDPR, BGE-M3, multilingual-E5, Cohere multilingual-v3): single embedding space across languages. Terminology mapping: dictionary alignment (J-MeSH ↔ MeSH via UMLS, JADER MedDRA/J ↔ MedDRA EN), ontology crosswalks (MEDIS病名マスター ↔ ICD-10 ↔ ICD-11). MT pivot: translate JP guideline excerpt → EN → retrieve PubMed; or translate EN evidence → JP for in-language synthesis.

### Expressiveness / Semantics
MIRACL Japanese subset: 3,477 train / 860 dev / 650 test-A / 1,141 test-B queries; 6,953,614 passages from 1,133,444 Wikipedia articles. Dev nDCG@10 baselines on ja split: BM25 ≈ 36.9, mDPR ≈ 43.9, BGE-M3 dense ≈ 72.8. MedDRA/J versions: Ver. 29.0 (March 2026), 28.0 (2025-03-01), 27.1 (2024-09-01), 27.0 (2024-03-01).

### Composability / Modularity
Encoders (BGE-M3, mE5) drop-in. Terminology mappings ship as TSV/RDF crosswalks. MT models (NLLB-200, mBART-50) plug into preprocessing or postprocessing.

### Suitability for autoformalization to IR
High. A canonical IR concept layer anchors each entity to a triple `(J-MeSH/MEDIS code, MeSH/UMLS code, ICD-10/ICD-11 code)`. BGE-M3 enables direct JP-query → EN-PubMed retrieval. Idempotency: terminology versions must be pinned (MedDRA/J Ver.29.0 vs 28.0 changes coding); deterministic given pinned mappings.

### Formal verification potential
Round-trip checks (JP→EN→JP) validate alignment fidelity. Concept equivalence verifiable via UMLS metathesaurus when both sides mapped. SHACL constraints can enforce that every extracted entity has at least one Japanese + one English code.

### Tooling / Ecosystem maturity
BGE-M3, multilingual-E5: top MTEB/MIRACL leaderboard fixtures. NLLB-200: open weights (600M, 1.3B, 3.3B). MedDRA/J: licensed via JMO subscription. MEDIS-DC standard masters: free download with registration, updated quarterly (Mar/Jun/Oct/Jan); ICD10対応標準病名マスター Ver.5.14 (June 2024), with the 24th edition of "標準マスターの概要と使い方" published July 2025. J-MeSH current version released 2015 (per PMC review), distributed under restricted license via UMLS.

### Japan-specific considerations
SNOMED CT not nationally adopted in Japan; MEDIS病名マスター is the de facto standard disease vocabulary, ICD-coded. ICD-11 Japanese term translation proposals published by MHLW 国際分類情報管理室 on 2024-07-24 (based on the WHO ICD-11 2023-01 release). MeSH-J in UMLS is from 2015 — outdated for novel diseases (COVID-19, novel therapeutics). MedDRA/J is the only PMDA-required terminology for ADR coding.

### Interoperability
Direct fit with Cat 1 (FHIR `CodeableConcept` with multiple `Coding`), Cat 3 (Minds/JP Core/PMDA — JP Core terminology bindings reference MEDIS), Cat 4 (OWL/SHACL ontology imports from UMLS, BioPortal). Cross-lingual retrieval feeds §6 GraphRAG with bilingual entity anchors.

### Limitations / Known issues
J-MeSH update lag (2015). SNOMED gap creates Japan/international interoperability friction. NLLB / NMT medical-domain accuracy uncalibrated — no medical FLORES subset exists (FLORES-200 is general/Wikipedia); numerical and dosage values must not be MT-paraphrased (use copy mechanism or extraction-based translation). MedDRA/J licensing fees may apply for commercial deployment.

### Training data proxy
MIRACL, BGE-M3, NLLB-200, mBART: extensive ACL/EMNLP / arXiv coverage 2023–2026. J-MeSH, MEDIS, MedDRA/J: Japan-domestic medical informatics presence (J-STAGE, 日本医療情報学会); limited English presence on Stack Overflow / GitHub.

## 10. RAG Evaluation: RAGAS, TruLens, ARES, Citation Precision

### Purpose
Quantify RAG system quality across retriever and generator dimensions: faithfulness (no hallucination), answer relevance, context precision/recall, citation precision/recall, factuality.

### Maintainer / Standards body
RAGAS: Exploding Gradients (open-source `explodinggradients/ragas`). TruLens: TruEra / Snowflake (RAG Triad). ARES: Saad-Falcon et al. (Stanford, 2023). DeepEval: Confident AI (`confident-ai/deepeval`). LlamaIndex evaluation: LlamaIndex. ALCE: §8. CRAG: Meta (KDD Cup 2024). FreshQA: Vu et al., 2023. AttributionBench, HaluEval: Tsinghua / multiple. Vectara HHEM (Hughes Hallucination Evaluation Model).

### Conceptual model
RAGAS metrics: faithfulness (fraction of generated claims entailed by retrieved context, via LLM-judge NLI), answer relevance (generated-question round-trip similarity), context precision (precision@k for relevance of retrieved chunks ranked highly), context recall (coverage of ground-truth claims), context entity recall, noise sensitivity. TruLens RAG Triad: Context Relevance × Groundedness × Answer Relevance. ARES uses synthetic queries + LLM judge fine-tuned for evaluation.

### Expressiveness / Semantics
LLM-as-judge metrics calibratable with human-labelled subsets (~50–200 examples per RAGAS docs). Faithfulness `=` (#supported claims) / (#total claims). Citation precision/recall via NLI (ALCE), or via GPT-4o (LongBench-Cite finds GPT-4o citation precision 88.8% vs human gold).

### Composability / Modularity
RAGAS plugs into LangChain / LlamaIndex / Langfuse with one-line evaluators. TruLens instruments any chain via feedback functions. Composable per-metric.

### Suitability for autoformalization to IR
Critical for closed-loop quality control of the autoformalisation pipeline. Each pipeline run can emit (faithfulness, citation_precision, context_recall) scores; thresholds gate IR commit. Idempotency check itself can be operationalised as a RAGAS-style metric: "same input → same IR triples?" computed via Jaccard / graph edit distance.

### Formal verification potential
All metrics auditable per-example. RAGAS judge prompts versioned ("V1-identical" prompt layout) — judge config must be pinned. Provides regression-testable CI signal. Combine with deterministic checks (exact-match, schema validation) to bound LLM-judge noise.

### Tooling / Ecosystem maturity
RAGAS: large GitHub footprint; collections-based API supersedes legacy in v0.4+; deprecation roadmap to v1.0. TruLens: Snowflake-backed enterprise distribution. DeepEval: rapid 2024–2026 release cadence. CRAG benchmark from KDD Cup 2024. AttributionBench, HaluEval 2.0 (2024). LlamaIndex `RetrieverEvaluator`, `FaithfulnessEvaluator` stable.

### Japan-specific considerations
LLM-judge prompts ship in English by default; Japanese-language judges should be used for Japanese RAG (otherwise reverse-translation noise inflates faithfulness false-negatives). MEMERAG benchmark (Blandón, Talur et al., Amazon Science, arXiv:2502.17163, ACL 2025) extends MIRACL with native multilingual answer generation and expert annotation across multiple languages including Japanese — a multilingual meta-evaluation RAG benchmark with high inter-annotator agreement. Use to calibrate any RAGAS judge on Japanese CDS.

### Interoperability
Evaluator scores attach as metadata to Cat 1 FHIR `Provenance` extensions. Faithfulness signal can drive Cat 6 defeasible defeat (low-faithfulness conclusions are defeasible). Citation precision feeds Cat 5 proof-obligation generation (every formal claim must trace to a high-precision citation).

### Limitations / Known issues
LLM-judge cost and latency. Judge-model drift across versions (GPT-4 vs GPT-4o vs Claude — scores shift). MAUVE / fluency metrics unstable for long outputs (ALCE noted). Faithfulness ≠ correctness (a faithful answer to wrong retrieval is still wrong). Most benchmarks English; Japanese-language meta-evaluation only emerging via MEMERAG.

### Training data proxy
RAGAS: official docs, Langfuse/Redis/Confident-AI integrations, Medium tutorials. TruLens: Snowflake-backed marketing. DeepEval: very active GitHub. ALCE: standard reference benchmark. CRAG, HaluEval: KDD/ACL coverage. MEMERAG: 2025 ACL paper, low community uptake yet — flag as emerging.
