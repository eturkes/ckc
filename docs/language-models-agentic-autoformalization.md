# Category 8: Language Models, Constrained Decoding, and Agentic Autoformalization

## 1. Closed Frontier Model Ensembles: GPT-5.5, Claude Opus 4.7, Gemini 3.1 Pro

### Purpose
Provide top-tier general reasoning, long-context understanding, multilingual (incl. Japanese) comprehension, tool use, and structured output for autoformalization of clinical guideline prose into IR.

### Maintainer/Standards body
OpenAI (GPT-5, GPT-5.5, GPT-5.5 Pro; released April 23, 2026); Anthropic (Claude Opus 4.5 released Nov 24, 2025; Opus 4.7 released April 16, 2026 — current GA; Sonnet 4.5; Haiku 4.5); Google DeepMind (Gemini 3 Pro released Nov 18, 2025; Gemini 3.1 Pro released Feb 19, 2026; Deep Think mode).

### Conceptual model
Closed-weight transformer LLMs with extended-thinking / "reasoning effort" parameters (low/medium/high), native multimodality (Gemini 3.x), and adaptive thinking budgets (Opus 4.7). Used as ensemble members where outputs are aggregated by self-consistency, multi-agent debate, or judge models.

### Expressiveness/Semantics
Free-form natural language with strong instruction following; JSON / Structured Output modes; function-calling schemas; citation modes (Anthropic citations API, OpenAI structured outputs). All three support 200K–1M token context windows (Gemini 3.1 Pro: 1M input / 65,536 output; Opus 4.5: 200K; Opus 4.7: up to 1M context; GPT-5.5: 1M context).

### Composability/Modularity
High. Each supports tool use / function calling, MCP (all three are MCP-aware), prompt caching (Opus 4.7 offers up to 90% input cost savings with prompt caching and 50% with batch processing; GPT-5.5 batch at 50% discount), and batch APIs. Compose well in agent graphs (LangGraph, OpenAI Agents SDK, Claude Code).

### Suitability for autoformalization to IR
Strong. Frontier reasoning + JSON-schema structured output yields high pass@1 IR generation from Japanese clinical text. Opus 4.7 reported best agentic/tool-calling consistency (XBOW reports "98.5% on our visual-acuity benchmark versus 54.5% for Opus 4.6"); Gemini 3.1 Pro leads ARC-AGI-2 (77.1%) and abstract reasoning; GPT-5.5 leads Terminal-Bench 2.0 (82.7%) and FrontierMath. None are specialized for Lean; pair with prover models (§3).

### Formal verification potential
Indirect: closed models generate IR / Lean stubs but cannot themselves verify. Verification delegated to Lean 4 + Mathlib4 and SMT solvers via tool calls. Opus 4.7 noted by Vercel to "do proofs on systems code before starting work."

### Tooling/Ecosystem maturity
Mature. SDKs in Python/TS, Bedrock/Vertex/Azure availability for all three. MCP donated by Anthropic to the Linux Foundation Agentic AI Foundation on December 9, 2025 (Linux Foundation press release: "SAN FRANCISCO, Dec. 9, 2025 – The Linux Foundation…today announced the formation of the Agentic AI Foundation (AAIF), and founding contributions of three leading projects…Anthropic's Model Context Protocol"); 10,000+ published MCP servers, 97M+ monthly SDK downloads.

### Japan-specific considerations
All three handle Japanese well; Gemini 3.x and Claude Opus 4.x have explicit Japanese benchmark coverage. Data residency: Vertex AI Tokyo region, Bedrock Tokyo, Azure Japan East. Cross-border PHI handling restricted under APPI; on-prem/VPC inference is not available — only API.

### Interoperability (with Categories 1–7)
Native MCP clients; OpenAPI function-calling; integrate with FHIR servers, UMLS / BioPortal terminology services, Lean-as-MCP-tool, SMT-solver-as-tool, vector DBs. Structured Output binds directly to Pydantic / JSON Schema IR.

### Limitations/Known issues
Closed weights (cannot self-host for PHI); price ($5/$25 per Mtok for Opus 4.7, $30/$180 per Mtok for GPT-5.5 Pro output); rate limits; non-determinism even at temperature 0; hallucination on rare Japanese clinical terminology; benchmark contamination concerns (Anthropic notes decontamination changes in Opus 4.5 evaluations).

### Training data proxy
Web crawl + licensed data + RLHF + RLAIF; medical/Japanese coverage not disclosed quantitatively. GPT-5.5 explicitly trained for agentic computer-use trajectories; Opus 4.5 trained for long-horizon coding; Gemini 3.1 Pro built on Gemini 3 Pro foundation with extra reasoning RL.

---

## 2. Domain Medical Models: Med-Gemini, MedGemma 1.5, Meditron, GatorTron, LLaVA-Med, JMedLLM, UTH-BERT

### Purpose
Domain-specialized weights for clinical text and image comprehension, medical QA, EHR extraction, clinical NER, terminology normalization (drug names, lab codes, diagnosis codes), de-identification, and — most relevant to this CDS context — pre-parsing Japanese clinical guidelines into structured medical concepts before frontier-LLM formalization. Secondary CDS roles include serving as upstream encoders for §7 premise retrieval, as judges or critics in §8, and as constrained-decoding-ready generators when paired with §4 grammars over clinical vocabularies.

### Maintainer/Standards body

**Frontier closed medical models.** Google DeepMind / Google Research line: Med-PaLM 2 (Singhal et al., 2023; 86.5% MedQA at release); Med-Gemini (Saab et al., arXiv:2404.18416, 2024; 91.1% MedQA, surpassing Med-PaLM 2 by 4.6 pp, SOTA on 10/14 medical benchmarks at release including multimodal and long-context categories); AMIE — Articulate Medical Intelligence Explorer (Tu et al., Google DeepMind; diagnostic-dialogue agent, published in Nature 2025; uses simulated patient self-play and clinician RLHF over a Gemini foundation); MedLM — the productized Vertex AI offering layered on the Med-PaLM / Med-Gemini stack under Healthcare-tuned commercial terms.

**Open-weight medical LLMs.** Google DeepMind (MedGemma 4B / 27B text-only / 27B multimodal under HAI-DEF; **MedGemma 1.5 released January 2026** on the Gemma 3 backbone, ~91% MedQA, currently the de-facto default open clinical base); EPFL LiGo (Meditron-7B / Meditron-70B on Llama 2, GAP-Replay + PubMed full-text); Stanford CRFM (BioMedLM 2.7B); Shanghai Jiao Tong (PMC-LLaMA on Llama 2); M42 Health (Med42 / Med42-v2 70B on Llama 2); Saama AI Research (OpenBioLLM-8B / OpenBioLLM-70B on Llama 3); BioMistral team (BioMistral 7B on Mistral); Microsoft Research (BioGPT — early generative biomedical model); Chinese University of Hong Kong / HKUST (HuatuoGPT / HuatuoGPT-II Chinese medical); FreedomIntelligence (Apollo / Apollo2 multilingual medical 7B spanning English / Chinese / French / Spanish / Hindi / Arabic).

**Clinical-text encoders (still the workhorse for NER, extraction, and embedding).** UF Health + NVIDIA (GatorTron 345M–8.9B; trained on >90B words of clinical text from UF Health EHR, SOTA on i2b2 subtasks at release); Microsoft Research (PubMedBERT / BiomedNLP-PubMedBERT — domain-pretrained from scratch on PubMed); KU Leuven / DMIS Lab (BioBERT — Lee et al.); MIT CSAIL + BIDMC (ClinicalBERT — Alsentzer et al.; pretrained on MIMIC-III discharge summaries); AI2 (SciBERT); NIH (BlueBERT).

**Multimodal medical models.** Google DeepMind (MedGemma 27B-MM with SigLIP-400M vision encoder; Med-Gemini multimodal); Microsoft (LLaVA-Med — Li et al., arXiv:2306.00890 — biomedical visual question answering on the LLaVA architecture; BiomedCLIP image-text contrastive encoder); LLaVA-Rad — radiology-specialized variant that "outperforms publicly available report generation models … generating fewer errors than LLaVA-Med and general-domain LLaVA" on chest X-ray; Stanford (CheXagent for chest X-ray interpretation); UCSD (BiomedGPT generalist medical multimodal); Med-PLIB pixel-level biomedical multimodal model (arXiv:2412.09278; SOTA on pixel grounding for biomedicine); Meissa — multi-modal medical agentic intelligence (arXiv:2603.09018).

**Japanese medical models.** University of Tokyo Hospital (**UTH-BERT** — Kawazoe et al.; first Japanese medical LM, pretrained BERT on ~120M clinical texts from UTH); NTT (**JMedRoBERTa** — RoBERTa pretrained on Japanese medical paper abstracts and main text); University of Tokyo Cardiovascular Medicine + Systems Innovation (JMedLoRA — Sukeda, Suzuki, Kodera, Sakaji; arXiv:2310.10083; JMedLLM-v1-7B — arXiv:2409.11783, Qwen2-7B base + bilingual medical pretraining + LoRA); a 70B-parameter Japanese-medical-QA evaluation (Sukeda et al., arXiv:2406.14882); and as the Japanese-capable substrate for medical fine-tunes, the **Swallow** project (Tokyo Tech / AIST / NII): Llama-3.3-Swallow-70B (2025), GPT-OSS-120B Swallow variant (2026), continuation-pretrained on Swallow Corpus v2/v3 + Japanese web + math/code mixtures (arXiv:2404.17790 documents the methodology). No major Japanese medical foundation model is multimodal as of May 2026.

World models for patient state and trajectory — the JEPA family (I-JEPA, V-JEPA, V-JEPA 2), Dreamer-class latent-dynamics models, generative trajectory predictors (ETHOS, Foresight, EHRWorld), and the recent JEPA-class clinical instantiations Clin-JEPA (Yang et al., arXiv:2605.10840) and SMB-Structure (Adam et al., arXiv:2601.22128) — are covered separately in §12, since they predict in latent / event space rather than generate IR.

### Conceptual model
Three architectural families dominate. **Decoder-only transformers** fine-tuned or continuation-pretrained on biomedical corpora: MedGemma 1.5 on the Gemma 3 backbone; Med-Gemini on Gemini Pro; Meditron / Med42 / OpenBioLLM / BioMistral / PMC-LLaMA / Apollo on Llama / Mistral families; JMedLLM-v1-7B on Qwen2-7B. **Encoder-only transformers (BERT-class)** for clinical NER, extraction, embedding, and classification: PubMedBERT, BioBERT, ClinicalBERT, SciBERT, BlueBERT, GatorTron-base, UTH-BERT, JMedRoBERTa. **Multimodal architectures**, typically vision-encoder + projector + LLM, optionally with a contrastive image-text head: MedGemma-MM (SigLIP-400M vision encoder, 896×896 medical images interleaved with text), LLaVA-Med / LLaVA-Rad, CheXagent, BiomedGPT, BiomedCLIP, Med-PLIB. **AMIE** is not a new architecture but an orchestration pattern on top of a Gemini-class base — self-play diagnostic dialogue with simulated patients plus expert-clinician RLHF — included here because it is the canonical published example of a *clinical dialogue agent* and is directly relevant to clarification-question generation against ambiguous guideline text. The complementary world-model paradigms — latent-dynamics (Dreamer-class), joint-embedding predictive (JEPA-class), and observation-level generative trajectory prediction (ETHOS / EHRWorld-class) — are covered separately in §12.

### Expressiveness/Semantics
Strong on medical QA across English benchmarks. **Med-Gemini** 91.1% MedQA (USMLE; Saab et al. 2024), SOTA on 10/14 medical benchmarks at release. **MedGemma 1.5** (January 2026) ~91% MedQA among open-weight models. **MedGemma 27B text-only** 87.7% MedQA at approximately one-tenth the inference cost of DeepSeek R1 (per Google Research). Multimodal **MedGemma 27B-MM** accepts arbitrarily interleaved 896×896 medical images and text. **JMedLLM-v1-7B** achieves >50% on IgakuQA (Japanese medical licensing exam). Encoder-only specialists hold the line on extraction tasks: **GatorTron** SOTA on i2b2 challenge subtasks at release, **ClinicalBERT** the standard for clinical phenotyping. Multimodal-specialization gradient on radiology — LLaVA-Rad < LLaVA-Med < generic LLaVA in error rate, confirming domain specialization still pays for image tasks. Important caveat: frontier general-purpose models from §1 now match or exceed many domain fine-tunes on aggregate medical benchmarks; the remaining advantages of medical-specific models are open-weight licensing (deployability for PHI workloads), cost, and latency, not raw accuracy.

### Composability/Modularity
Open weights for the full open-medical line (MedGemma 1.5, Meditron, BioMedLM, PMC-LLaMA, BioMistral, Med42, OpenBioLLM, HuatuoGPT-II, Apollo, BioGPT, JMedLLM-v1-7B; all on Hugging Face under varying licenses) → composable as §1 ensemble members, §7 RAG retriever encoders, or §8 judges. **MedSigLIP** and **BiomedCLIP** image-text encoders reusable as multimodal embedding models. Clinical text encoders (PubMedBERT, BioBERT, ClinicalBERT, SciBERT, BlueBERT, GatorTron-base, UTH-BERT, JMedRoBERTa) consumable directly by vector databases for §7 premise retrieval over MEDIS-DC / Minds / JLAC10 / PubMed corpora — and these are usually faster and cheaper than passing every retrieval through a frontier LLM. AMIE-style diagnostic-dialogue orchestration composable with §5 tool-calling agents when guideline text requires clarification rounds. (World-model encoders — JEPA, Dreamer-latent, tokenized-EHR — usable as upstream feature extractors or downstream trajectory simulators: see §12.)

### Suitability for autoformalization to IR
Indirect. Best used as (a) **Japanese clinical-term normalizer** that converts guideline prose into a tagged stream of canonical MEDIS-DC / JLAC10 / ICD-10-J concepts before §1 frontier-LLM formalization, (b) **embedder for premise retrieval** in §7, (c) **judge / critic** in §8, (d) **dialogue front-end** (AMIE-style) for clarification questions against ambiguous guideline statements, or (e) **multimodal extractor** when guidelines include scanned figures, decision trees, or tables. Encoder-only models cannot generate IR but are the most efficient option for the embedding and clinical-NER subtasks. None of these models is strong at producing Lean / SMT directly: medical LLMs are trained for QA / dialogue / extraction, not formal-language generation.

### Formal verification potential
None native. Output piped to provers / SMT (§3) or wrapped behind a §5 tool-calling agent that delegates verification.

### Tooling/Ecosystem maturity
**Open-weight side.** MedGemma 1.5 ships on Hugging Face, Vertex Model Garden, GKE deployment recipes, and is compatible with vLLM / TGI / TensorRT-LLM. Meditron, BioMedLM, BioMistral, Med42, OpenBioLLM, HuatuoGPT-II, Apollo, BioGPT, JMedLLM, JMedLoRA, LLaVA-Med, LLaVA-Rad, CheXagent, BiomedGPT, BiomedCLIP all released as research / community artifacts on Hugging Face and / or GitHub. GatorTron weights gated through UF Health BRIDGE2AI access. PubMedBERT, BioBERT, ClinicalBERT, SciBERT, BlueBERT, UTH-BERT, JMedRoBERTa freely available. **Closed side.** Med-Gemini, Med-PaLM 2, and AMIE accessible only via Vertex AI under MedLM commercial agreement. **Benchmarks and leaderboards.** MedQA-USMLE, MedMCQA, PubMedQA, MMLU-Medical, MultiMedBench, HealthBench (OpenAI, 2025), and **Medmarks v0.1** (arXiv:2605.01417, 2026 open-source benchmark suite for medical tasks) are the current standard suites; the Medical LLM Leaderboard tracks scores publicly. Google Health publishes a MedQA relabeling artifact (`Google-Health/med-gemini-medqa-relabelling`) documenting label-noise issues on the canonical benchmark.

### Japan-specific considerations
**Benchmarks.** JMedBench (NAIST Social Computing Lab) is the canonical Japanese biomedical LLM benchmark — 20 datasets across MCQA / MT / NER / DC / STS, including IgakuQA (Japanese medical licensing exam), JMMLU-medical, and translated MedQA / PubMedQA. A September 2025 medRxiv comparative evaluation (Japanese National Medical Examination) reports current standings of frontier general models, MedGemma, and Japanese-medical fine-tunes against each other on the national licensing exam. **Specialized encoders.** UTH-BERT (University of Tokyo Hospital, ~120M clinical-text pretraining) and JMedRoBERTa (NTT, Japanese medical paper corpus) are the canonical Japanese clinical encoders; UTH-BERT is particularly useful for NER over hospital-style free text since it sees actual EHR prose, while JMedRoBERTa biases toward biomedical-literature style. **Generative models and adapters.** JMedLLM-v1-7B (arXiv:2409.11783; Qwen2-7B base) and JMedLoRA (arXiv:2310.10083) cover the generative side. JMedLoRA's central finding: QLoRA on a larger English-centric base (Llama2-70B-chat-hf) outperforms LoRA on Japanese-centric bases (OpenCALM-7B), with Japanese-centric models showing "deterioration of 1-shot performance after instruction-tuning." **Japanese-capable substrate.** The Swallow project (Tokyo Tech / AIST / NII) provides the standard Japanese-extended Llama / Mistral lineage — **Llama-3.3-Swallow-70B** (2025), **GPT-OSS-120B Swallow variant** (2026) — and is the most common base over which new Japanese medical fine-tunes are built. **Catastrophic forgetting** remains a documented failure mode: Meditron's Japanese degradation is the standard cautionary example, and most direct Japanese fine-tuning of English-centric medical bases sees this. **No Japanese multimodal medical foundation model** has been released as of May 2026 — Japanese radiology / pathology / ophthalmology multimodal work currently piggybacks on MedGemma-MM or LLaVA-Med with translated prompts.

### Interoperability (with Categories 1–7)
Hugging Face transformers, vLLM, Outlines / xgrammar constrained decoding (§4) compatible across the open line; SigLIP / BiomedCLIP / MedSigLIP embeddings consumable by vector DBs; encoder-only clinical models (PubMedBERT, ClinicalBERT, GatorTron-base, UTH-BERT, JMedRoBERTa) plug directly into RAG indices for §7 premise retrieval over MEDIS-DC / Minds / JLAC10 corpora. All can be served behind MCP (§5). Med-Gemini / AMIE accessible via Vertex AI APIs (MedLM commercial terms) for the cases where open weights are not required.

### Limitations/Known issues
MedGemma requires validation per use case (Google explicitly labels it a "developer model"); none of these is regulator-approved CDS in Japan, and U.S. FDA clearance for any specific medical LLM remains rare even amid >950 cleared AI medical devices as of early 2026, most of which are imaging / signals devices rather than text generators. Meditron multilingual degradation on Japanese, and analogous catastrophic-forgetting failures are common across Llama-based medical fine-tunes when pushed cross-lingual. PHI / data-licensing for Japanese fine-tuning datasets remains restrictive under APPI. **Benchmark contamination** concerns are now well-documented for MedQA and PubMedQA (frontier-model pretraining corpora overlap test sets); Google's own MedQA relabeling work documents label-noise issues independently. Hallucination on rare Japanese drug brand names, less-common ICD-10-J codes, JLAC10 lab codes, and pediatric / rare-disease guidelines is significant; AMIE-style dialogue agents introduce additional failure modes — sycophancy under user pressure, premature anchoring on initial differential. Encoder-only models cap out at modest context lengths (~512 tokens for vanilla BERT-class), which constrains use on long guideline prose without chunking. (For limitations of the 2026 clinical world models — Clin-JEPA's single-cohort MIMIC-IV-ICU training, SMB-Structure's MSK / INSPECT cohort biases, EHRWorld's qualitative-only public benchmark numbers, and the absence of Japanese variants for any of them — see §12.)

### Training data proxy
**Frontier closed.** Med-Gemini: Gemini Pro base + medical fine-tune corpus (PubMed abstracts, clinical case discussions, USMLE prep, medical textbooks; mixture not fully disclosed). AMIE: simulated diagnostic dialogues via self-play plus RLHF over expert clinician feedback. **Open generalist.** MedGemma 1.5: medical QA pairs, FHIR EHR (27B-MM), radiology / histopath / ophthalmology / dermatology images, public medical text. Meditron: GAP-Replay + PubMed full-text. BioMedLM: PubMed abstracts. PMC-LLaMA: PubMed Central full-text on Llama base. Med42 / OpenBioLLM: curated medical instruction data on Llama 2 / Llama 3. BioMistral: PubMed + clinical guidelines on Mistral 7B. HuatuoGPT-II: Chinese medical corpora (HuatuoCorpus). Apollo: multilingual medical corpora spanning 6 languages. **Encoders.** GatorTron: 82B+ words of UF Health clinical text augmented with PubMed and MIMIC. PubMedBERT / BioBERT: PubMed abstracts and full-text. ClinicalBERT: MIMIC-III discharge summaries on BioBERT base. **Japanese.** UTH-BERT: ~120M clinical texts from University of Tokyo Hospital. JMedRoBERTa: Japanese medical paper abstracts and main text. JMedLoRA: Japanese medical instruction pairs derived from licensing-exam material on OpenCALM-7B / Llama2-70B-chat-hf bases. JMedLLM-v1-7B: Qwen2-7B base + bilingual medical pretraining. Swallow substrate: Wikipedia, DCLM-baseline-1.0, Swallow Corpus v2 / v3, Cosmopedia, Laboro ParaCorpus, FineMath-4+, Swallow Code. Synthetic data generation from frontier models acceptable for adapter training but must be checked for hallucination (see §11 for the adapter-training discussion).

---

## 3. Formal-Proof Models and Environments: DeepSeek-Prover-V2, LeanDojo, Leanstral

### Purpose
Generate machine-checkable Lean 4 proofs of theorems / goals; verify consistency of IR-encoded clinical rules expressed as Lean propositions; detect contradictions between guideline statements.

### Maintainer/Standards body
DeepSeek AI (Prover-V1.5, Prover-V2-671B, Apr 2025); Princeton / Tsinghua / Meta FAIR / Numina (Goedel-Prover V1 Feb 11, 2025 by Yong Lin, Shange Tang, Bohan Lyu, Jiayun Wu, Hongzhou Lin, Kaiyu Yang, Jia Li, Mengzhou Xia, Danqi Chen, Sanjeev Arora, Chi Jin — arXiv:2502.07640; V2 Aug 5, 2025 — arXiv:2508.03613); Numina + Moonshot Kimi (Kimina-Prover 72B); Google DeepMind (AlphaProof, IMO 2024 silver, Nature Nov 2025); Caltech / NVIDIA (LeanDojo + ReProver — Yang, Swope, Gu, Chalamala, Song, Yu, Godil, Prenger, Anandkumar; NeurIPS 2023 Datasets & Benchmarks oral); Mistral AI (Leanstral, released March 16, 2026 — 120B total / 6B active sparse MoE, Apache 2.0, for Lean 4 proof engineering); Lean FRO / Microsoft Research (Lean 4, Mathlib4).

### Conceptual model
LLMs fine-tuned / RL-trained on Lean tactic sequences or whole proofs. Two paradigms: (a) tactic-step models with tree search (ReProver, InternLM2.5-StepProver, AlphaProof MCTS), (b) whole-proof reasoning models (Kimina-Prover, DeepSeek-Prover-V2 CoT mode, Goedel-V2). Some use recursive subgoal decomposition (DeepSeek-V3 decomposes → 7B solves subgoals → recombine).

### Expressiveness/Semantics
Lean 4 with Mathlib4 covers higher-order logic, dependent types, classical and constructive math. Sufficient to encode clinical rule logic (temporal, deontic via embedded modal logic libraries, arithmetic comparisons over lab values).

### Composability/Modularity
LeanDojo provides programmatic Lean interaction (extracts tactic states, premises). Kimina-Lean-Server / kimina-client offer parallelized proof checking with LRU caching (~10× throughput). Provers usable as tool behind MCP.

### Suitability for autoformalization to IR
High when paired with a frontier autoformalizer. Kimina-Autoformalizer-7B explicitly trained for NL→Lean 4 statement translation. DeepSeek-Prover-V2 integrates informal+formal reasoning. Goedel-Prover-V2-32B achieves 88.1% pass@32 on miniF2F in standard mode and 90.4% in self-correction mode; solves 86 PutnamBench problems at pass@184 (surpassing DeepSeek-Prover-V2-671B's 47 at pass@1024).

### Formal verification potential
Maximal — Lean kernel is the verifier. Contradictions between two clinical rules can be encoded as a goal `r1 ∧ r2 → False` and proved/disproved.

### Tooling/Ecosystem maturity
Mathlib4 has ~1.5M+ lines and active Lean community. miniF2F, ProofNet, PutnamBench, ProverBench (325 problems, DeepSeek). DeepSeek-Prover-V2-671B: 88.9% miniF2F pass-rate. Kimina-Prover-72B: 80.7% miniF2F pass@8192. AlphaProof: IMO 2024 P1/P2/P6 (28/42 points = silver-medal standard).

### Japan-specific considerations
No Japanese-specific Lean infrastructure; Mathlib4 and tooling are English. Lean community Japan exists informally. Japanese clinical ontology terms must be normalized to ASCII / UTF-8 Lean identifiers.

### Interoperability (with Categories 1–7)
LeanDojo extracts premise data for retrieval (§7); Kimina-Lean-Server exposable as MCP tool (§5); whole-proof outputs verifiable by SMT cross-check (Lean Smt tactic).

### Limitations/Known issues
Frontier provers are compute-heavy (DeepSeek-V2 671B); benchmark contamination concerns (Kimina identified at least 5 problems in the miniF2F-test dataset that were wrongly formalized). Mathlib4 lacks clinical-domain libraries — must be built. Leanstral: only reports FLTEval (real-repo) numbers, no miniF2F; Mistral marketing-framed.

### Training data proxy
Mathlib4 corpus + synthetic theorem decomposition (DeepSeek-V3 prompted); Numina Math 1.5; Lean-Workbook-Plus; Goedel-V1 expert-iteration over 1.64M auto-formalized statements across 8 rounds generating 29.7K Lean Workbook proofs.

---

## 4. Constrained Decoding Against IR Grammar and Schema

### Purpose
Guarantee that LLM output is parseable into the target IR (JSON Schema, Pydantic class, context-free grammar, regex) on every sample, eliminating syntax-level retries.

### Maintainer/Standards body
xgrammar (Yixin Dong, Charlie F. Ruan, Yaxing Cai, Ruihang Lai, Ziyi Xu, Yilong Zhao, Tianqi Chen; CMU + NVIDIA + SJTU + UC Berkeley; MLSys 2025; default backend for vLLM, SGLang, TensorRT-LLM, MLC-LLM); Outlines (Rémi Louf, .txt / dottxt-ai); LMQL (ETH Zurich); Guidance (Microsoft / guidance-ai); LM Format Enforcer; jsonformer; BAML (Boundary); Pydantic AI; OpenAI Structured Outputs / Strict JSON Mode; Anthropic Tool Use schemas.

### Conceptual model
At each decoding step, build a token mask from a pushdown automaton / FSM derived from the schema / grammar; zero out invalid tokens before sampling. xgrammar splits vocabulary into context-independent (precomputed) and context-dependent (runtime) tokens; achieves up to 100× speedup, near-zero overhead on JSON.

### Expressiveness/Semantics
Regex < JSON Schema < CFG < dependent-type grammars. xgrammar supports general CFG. Cannot enforce semantic constraints (e.g., "drug code must exist in JMDC dictionary") — those need post-validation or constrained sampling against a trie.

### Composability/Modularity
Plugs into inference engines (vLLM, SGLang, TensorRT-LLM, MLC-LLM). Co-design APIs for rollback (speculative decoding) and jump-forward decoding. Stackable with KV caching.

### Suitability for autoformalization to IR
Critical. Forces IR JSON to match Pydantic schema; combined with semantic post-check (NLI, §8) gives idempotency at the surface level. JSONSchemaBench (Guidance AI, 10K real schemas) benchmarks Guidance, Outlines, Llamacpp, XGrammar, OpenAI, Gemini frameworks.

### Formal verification potential
Structural-only; not semantic. Schema conformance ≠ logical consistency.

### Tooling/Ecosystem maturity
High. xgrammar is the de-facto open-source default; integrated in major engines. Closed APIs (OpenAI strict JSON, Anthropic tool-use, Gemini structured output) cover same use case.

### Japan-specific considerations
UTF-8 / Japanese identifiers fully supported in schemas; CJK tokens require care for tokenizer-aware FSM compilation (xgrammar handles BPE merges). Japanese ICD-10, JLAC10 lab code dictionaries enforceable as enumerations.

### Interoperability (with Categories 1–7)
Outputs feed Lean (§3) / SMT directly; consumed by program-aided executors (§9); compatible with all tool-calling agents (§5).

### Limitations/Known issues
Quality degradation if model's top tokens are all masked (forced low-probability fallback). Complex regex / nested CFGs can cost 2–5× without optimization. Closed APIs have less schema coverage than open libraries.

### Training data proxy
N/A — inference-time technique; no training data. Some recent work (e.g., AdapTrack) trains models to anticipate constraints to avoid output distortion.

---

## 5. Tool-Calling Agents Connected to Lean, SMT Solvers, and Terminology Services

### Purpose
Allow LLMs to invoke Lean (proof checker), Z3 / CVC5 (SMT), UMLS / MeSH-J / JMDC / MEDIS-DC (terminology), FHIR servers, and Python sandboxes as external tools during formalization and verification.

### Maintainer/Standards body
Anthropic Tool Use; OpenAI Function Calling / Agents SDK; Google Gemini Function Calling; Model Context Protocol (MCP — created at Anthropic by David Soria Parra and Justin Spahr-Summers, donated by Anthropic to the Linux Foundation Agentic AI Foundation on Dec 9, 2025, co-founded with Block and OpenAI; 10,000+ published MCP servers, 97M+ monthly SDK downloads); LangGraph (LangChain); LlamaIndex; AutoGen (Microsoft); CrewAI; Claude Code; OpenAI Codex.

### Conceptual model
Agent loop: LLM emits tool call → host executes → result returned in context → LLM iterates. MCP standardizes the JSON-RPC layer with three primitives (tools / resources / prompts) and STDIO / StreamableHTTP transports. "Code execution with MCP" pattern (Anthropic engineering blog, Nov 2025) loads tools as code-on-filesystem to reduce token usage.

### Expressiveness/Semantics
Full Turing-complete via code execution tools; deterministic via Lean / Z3; ontology lookups via REST or MCP wrapping of UMLS / BioPortal / OBO Foundry / HL7 terminology services.

### Composability/Modularity
High. MCP servers reusable across all major hosts (Claude, ChatGPT, Cursor, Gemini, Microsoft Copilot, Visual Studio Code). Tool Search + Programmatic Tool Calling in Anthropic API optimize many-tool deployments.

### Suitability for autoformalization to IR
Core architecture for the proposed CDS: LLM → call `lookup_icd10_jp(code)` → call `lean_check(proposition)` → call `z3_check(formula)` → emit IR.

### Formal verification potential
Verification is delegated to the Lean / SMT tool; agent orchestrates. Goedel-Prover-V2 uses verifier-guided self-correction with 2 rounds of Lean compiler feedback (40K total tokens).

### Tooling/Ecosystem maturity
Very high. SDKs in Python / TypeScript / Java / C# / Ruby / Elixir. Pre-built reference MCP servers (Google Drive, Slack, GitHub, Postgres, Puppeteer); enterprise infra on AWS / Cloudflare / GCP / Azure.

### Japan-specific considerations
MCP wrappers around Japanese terminology services (MEDIS-DC standard master, JLAC10, ICD-10 国内ベース, YJ code for drugs, HOT code) must be custom-built. Minds guideline corpora can be exposed via MCP resources.

### Interoperability (with Categories 1–7)
Connects everything. Frontier models (§1), domain models (§2), provers (§3), constrained decoders (§4) all sit behind or in front of MCP.

### Limitations/Known issues
Context bloat with many tools (mitigated by Tool Search and code-execution pattern). Prompt-injection risk via tool outputs (Opus 4.5 hardened but not eliminated). Latency for serial tool chains.

### Training data proxy
Models trained with tool-use trajectories (Anthropic, OpenAI, Google internal mixtures); no public dataset.

---

## 6. Multi-Run Self-Consistency, Semantic Convergence, and Idempotency Checks

### Purpose
Quantify and enforce that repeated formalization of the same guideline statement yields semantically equivalent IR — the success criterion of the CDS.

### Maintainer/Standards body
Self-consistency: Xuezhi Wang, Jason Wei, Dale Schuurmans, Quoc Le, Ed Chi, Sharan Narang, Aakanksha Chowdhery, Denny Zhou, "Self-Consistency Improves Chain of Thought Reasoning in Language Models," ICLR 2023 (arXiv:2203.11171) — reported gains of +17.9% on GSM8K, +11.0% on SVAMP, +12.2% on AQuA. Autoformalization self-consistency: Zenan Li et al. (Nanjing / Microsoft), "Autoformalize Mathematical Statements by Symbolic Equivalence and Semantic Consistency," arXiv:2410.20936 (NeurIPS 2024). ReForm (arXiv:2510.24592) — Reflective Autoformalization with Prospective Bounded Sequence Optimization, +22.6 pp over strongest baselines.

### Conceptual model
Sample k candidates → cluster by (a) symbolic equivalence via ATP (Isabelle Sledgehammer / Lean `decide`), (b) semantic similarity of round-trip back-translation (informalize each, embed, cosine), (c) majority vote on canonical-form hash. Idempotency = f(f(x)) ≡ f(x); ASR = autoformalization self-consistency rate.

### Expressiveness/Semantics
Symbolic equivalence captures logical homogeneity; semantic consistency captures meaning preservation. Combined score addresses the pass@1 vs pass@k gap (19.5–26.5% on MATH / miniF2F per Li et al.).

### Composability/Modularity
Drop-in scoring layer over any generator. Compatible with constrained decoding (§4), retrieval (§7), critique (§8).

### Suitability for autoformalization to IR
Direct fit. Cluster k=8–32 IR candidates per guideline; promote majority cluster as canonical IR; flag low-consistency cases for human review.

### Formal verification potential
Symbolic-equivalence step uses an ATP, providing partial verification. Semantic-consistency step is heuristic (embedding-based).

### Tooling/Ecosystem maturity
Self-consistency is standard. Symbolic-equivalence pipelines (Isa-AutoFormal repo) exist; few production-grade libraries. Generalized self-consistency for open-ended generation (Jain et al. HF Papers 2307.06857).

### Japan-specific considerations
Back-translation step requires Japanese-capable informalizer. Embedding similarity should use multilingual-E5 / SimCSE-Ja / Ruri-large.

### Interoperability (with Categories 1–7)
Sits above all generator approaches; consumes outputs from §1–§3; feeds §8 (judge).

### Limitations/Known issues
Failure modes when all k candidates are wrong but mutually consistent (mode collapse). ATP timeouts on complex statements. Embedding metrics insensitive to logical operators (¬, ∀).

### Training data proxy
No training; selection-time technique. Some methods (ReForm) train the validation step jointly with generation via RL on Lean compiler reward.

---

## 7. Retrieval-Augmented Autoformalization with Premise/Theorem Retrieval

### Purpose
Retrieve relevant Mathlib4 / ontology / past-formalization premises at generation time to improve correctness and consistency.

### Maintainer/Standards body
LeanDojo + ReProver (Yang et al., Caltech / NVIDIA, NeurIPS 2023 Datasets & Benchmarks oral); Magnushammer (Isabelle); Thor (DeepMind); COPRA; Lean Copilot (Song et al.); LLMStep; FormalAlign (Jianqiao Lu et al., ICLR 2025); MS-RAG + Auto-SEF ("Consistent Autoformalization for Constructing Mathematical Libraries," arXiv:2410.04194).

### Conceptual model
Dense retriever (ByT5 / SBERT encoder) indexes premises with state-aware tokens. At each tactic step, retrieve top-k accessible premises, concatenate with goal state, feed encoder-decoder generator. LeanDojo Benchmark: 98,734 theorems with premise annotations; ReProver outperforms BM25 and GPT-4 zero-shot on the novel-premises split.

### Expressiveness/Semantics
Premise = any named Mathlib definition / lemma. In CDS context, premise = canonical clinical term, ICD / SNOMED-CT / SNOMED-Japan code, prior formalized rule, ontology axiom.

### Composability/Modularity
Retriever and generator independently swappable. Indexes recomputable when ontology updates.

### Suitability for autoformalization to IR
Highly suitable. Use prior formalizations as exemplars (few-shot) and canonical ontology entries as retrieval targets to enforce term consistency across runs (drives idempotency).

### Formal verification potential
Indirect — improves provability but does not itself verify.

### Tooling/Ecosystem maturity
LeanDojo (Lean 4 main branch), ReProver, Lean Copilot integrated into VS Code Lean extension. FormalAlign code/data public — outperforms GPT-4 by 11.58% AS on FormL4-Basic (99.21% vs 88.91%). MS-RAG +5.47–33.58% syntactic correctness improvement reported.

### Japan-specific considerations
Build dense index over MEDIS-DC, JLAC10, Minds CQ database, and Japanese society guidelines. SimCSE-Ja or BGE-M3 multilingual embeddings recommended.

### Interoperability (with Categories 1–7)
Retriever exposed as MCP tool; results feed constrained-decoded generator; clusters with §6 idempotency.

### Limitations/Known issues
Hard-negative selection critical (LeanDojo leverages program analysis). Retrieval can amplify majority biases. Premises in non-English may be under-represented.

### Training data proxy
LeanDojo Benchmark; MathLibForm; FormL4. For Japanese: must be assembled from Minds + society guidelines + national master files.

---

## 8. Independent Critique and Adjudication Passes for Conflict Explanations

### Purpose
Detect semantic errors in generated IR and resolve inter-guideline contradictions via independent judge models and multi-agent debate.

### Maintainer/Standards body
Constitutional AI (Anthropic); Reflexion (Shinn et al.); Self-Refine (Madaan et al.); Society of Minds / multi-agent debate — Yilun Du, Shuang Li, Antonio Torralba, Joshua B. Tenenbaum, Igor Mordatch, "Improving Factuality and Reasoning in Language Models through Multiagent Debate," ICML 2024 (PMLR 235:11733–11763, arXiv:2305.14325), 3 agents × 2 rounds; ChatEval (Chan et al.); PRD (Li et al.); D3 framework (Debate, Deliberate, Decide); Agent-as-a-Judge (arXiv:2508.02994); Multi-Agent Judge with HAJailBench (Lin, Shen, Yang, Liu, Zhao, Zeng; arXiv:2511.06396); NLI models (DeBERTa-v3-mnli, JaNLI for Japanese).

### Conceptual model
Critic-defender-judge or advocate-juror architectures. Critic agent produces structured critique under a rubric; defender rebuts; judge or jury of personas adjudicates. Bounded N rounds (typically 2–3) capture most gain; HAJailBench ablation peaks at three rounds on Qwen3-14B. NLI classifier checks entailment / contradiction between IR rule and source NL guideline.

### Expressiveness/Semantics
Critique can be free-text, structured rubric scores, or formal counterexample. Contradiction = explicit NLI label or proven `r1 ∧ r2 → ⊥` in Lean.

### Composability/Modularity
Orthogonal layer over generators. Compose with self-consistency (§6) and verifier feedback (§10).

### Suitability for autoformalization to IR
Required for inter-guideline contradiction detection — the central CDS use case. Use frontier model A as generator and frontier model B (different family) as judge to reduce self-enhancement bias.

### Formal verification potential
Judge can invoke Lean (§3) / SMT to ground its verdict, turning soft critique into hard proof of contradiction.

### Tooling/Ecosystem maturity
Mature for evaluation; production CDS adjudication pipelines emerging. Cost-aware D3 frameworks address compute concerns.

### Japan-specific considerations
Cross-lingual critique: judge prompts must include Japanese source for grounding; Japanese-specialized models can introduce safety regressions (JMedEthicBench finding: medical fine-tunes weaken safety alignment, with safety scores declining significantly across conversation turns — median 9.5 to 5.5, p < 0.001).

### Interoperability (with Categories 1–7)
Consumes outputs of §1–§7; produces structured contradiction reports as IR annotations.

### Limitations/Known issues
Judge quality is ceiling; weaker judge can miss subtle errors. Positional, verbosity, self-enhancement biases. Cost compounds with rounds.

### Training data proxy
HAJailBench (11,100 labeled jailbreak interactions); MT-Bench; AlpacaEval; Japanese benchmarks scarce for clinical critique.

---

## 9. Program-Aided Language Models for Executable Intermediate Code

### Purpose
Generate executable Python / SQL / CQL (HL7 Clinical Quality Language) as the IR; offload arithmetic, eligibility logic, and rule evaluation to a deterministic interpreter.

### Maintainer/Standards body
PAL (Luyu Gao, Aman Madaan, Shuyan Zhou, Uri Alon, Pengfei Liu, Yiming Yang, Jamie Callan, Graham Neubig; CMU; ICML 2023, arXiv:2211.10435); Program of Thoughts (Chen et al.); OpenAI Code Interpreter; Anthropic code execution tool; e2b sandboxes; Pyodide; RestrictedPython; HL7 CQL (clinical quality language) for guidelines.

### Conceptual model
LLM emits a program; Python interpreter (or CQL engine) executes; final answer is execution output. PAL with Codex achieved 72.0% top-1 on GSM8K, "surpassing PaLM-540B which uses chain-of-thought by absolute 15% top-1" (Gao et al., ICML 2023).

### Expressiveness/Semantics
Python is Turing-complete; CQL is explicitly designed for clinical decision logic (CMS eCQM standard). SQL / FHIRPath for EHR queries.

### Composability/Modularity
Interpreters as tools (§5); sandboxed; pure functions enable replay.

### Suitability for autoformalization to IR
Strong alternative to Lean for procedural rules ("if HbA1c > 7.0 and eGFR < 30, contraindicate metformin") — emit CQL or Python; reserve Lean for cross-guideline contradiction proofs. Hybrid pattern: PAL for evaluation, Lean for verification.

### Formal verification potential
Lower than Lean (no kernel-level guarantee) but supports symbolic execution / property-based testing (Hypothesis). CQL has formal grammar.

### Tooling/Ecosystem maturity
Very mature for Python; CQL ecosystem (cqf-tooling, CQL-to-ELM compiler, OpenCDS) is HL7-standard but smaller community.

### Japan-specific considerations
CQL supports Japanese identifiers; mapping CQL value sets to MEDIS-DC / JLAC10 is manual. Pyodide can run Japanese tokenizer libraries (fugashi, sudachipy) in sandbox.

### Interoperability (with Categories 1–7)
Pairs with constrained decoding (CQL grammar, §4) and tool-calling (§5). Outputs verifiable by Lean-encoded shadow specs (§3).

### Limitations/Known issues
Execution-output equivalence ≠ logical equivalence; sandbox escape risks; numeric edge cases. PAL fails when problem decomposition itself is wrong.

### Training data proxy
GitHub Python; CQL examples scarce. Models inherit from base coding training.

---

## 10. Verifier-Guided Decoding and Proof-Repair Loops

### Purpose
Use a verifier (Lean compiler, SMT solver, PRM) to score, prune, or repair candidate generations during decoding or in an outer loop.

### Maintainer/Standards body
Baldur (Emily First, Markus Rabe, Talia Ringer, Yuriy Brun; UMass / Google; ESEC/FSE 2023; arXiv:2303.04910) — whole-proof + repair on Isabelle/HOL; +8.7% over Thor; combined 65.7%. "Let's Verify Step by Step" (Hunter Lightman, Vineet Kosaraju, Yura Burda, Harri Edwards, Bowen Baker, Teddy Lee, Jan Leike, John Schulman, Ilya Sutskever, Karl Cobbe; OpenAI, arXiv:2305.20050) — process reward model + PRM800K (800K human step labels); 78% MATH subset. HyperTree Proof Search (Guillaume Lample et al., Meta, NeurIPS 2022; arXiv:2205.11491) — AlphaZero-style MCTS; improves Lean-based miniF2F-curriculum from 31% to 42%; 41% pass@64 on miniF2F-test. InternLM2.5-StepProver (Shanghai AI Lab, arXiv:2410.15700, Oct 2024) — critic-guided tree search; 59.2% miniF2F pass@64; ProofNet BF 22.3% + CG 23.9% → 27.0% combined. Self-Debug (Xinyun Chen, Maxwell Lin, Nathanael Schärli, Denny Zhou; Google / UC Berkeley, arXiv:2304.05128, Apr 2023) — execution-feedback rubber-duck loop, up to +12% on MBPP. Goedel-Prover-V2-32B (Princeton / Tsinghua, Aug 2025) — verifier-guided self-correction with 2 rounds of Lean compiler feedback, 88.1% → 90.4% miniF2F.

### Conceptual model
Two modes: (a) in-decoder verifier guides beam search / MCTS (HTPS, InternLM2.5-StepProver, PRMs); (b) outer loop — generate proof, run Lean, feed error message back into prompt, regenerate (Baldur, Goedel-V2 self-correction, Chen Self-Debug).

### Expressiveness/Semantics
Verifier signal = Boolean (compiles / proves) or scalar (PRM step score) or text (error message). Loops typically bounded (2 rounds in Goedel-V2, capped error-fix turns in Kimina).

### Composability/Modularity
Generator and verifier independently swappable; verifier wrappable as MCP tool.

### Suitability for autoformalization to IR
Essential for production CDS. Lean compiler is a perfect oracle for syntactic correctness; SMT for arithmetic decidability; PRM for soft step-level scoring of IR construction.

### Formal verification potential
Maximal when verifier is Lean kernel.

### Tooling/Ecosystem maturity
Kimina-Lean-Server (parallel, ~10× speedup, LRU cache); PRM800K open; vLLM / SGLang support beam search; Goedel-V2 self-correction pipeline open-source.

### Japan-specific considerations
Verifier feedback is in Lean (English error messages); not a language issue. Building a Japanese-clinical Mathlib lemma library is the prerequisite blocker.

### Interoperability (with Categories 1–7)
Closes the loop with §3 (Lean), §4 (constrained decoding), §6 (consistency over k repaired samples), §8 (judge can request repair).

### Limitations/Known issues
Reward hacking on PRMs; loop divergence; latency compounds with rounds. Self-debug effective up to ~3 rounds, then plateaus.

### Training data proxy
PRM800K; Lean-Workbook-Plus; Baldur's PISA (6,336 Isabelle theorems); Goedel-V2's scaffolded synthetic data.

---

## 11. LoRA/QLoRA Adapters for Japanese Guideline Normalization, Only if Data Justifies It

### Purpose
Parameter-efficient domain adaptation of a base LLM to Japanese clinical-guideline terminology and writing style — applied only when prompt-engineering + RAG + frontier models hit a residual ceiling on a measurable, held-out Japanese-guideline benchmark.

### Maintainer/Standards body
LoRA (Hu et al., Microsoft, ICLR 2022); QLoRA (Dettmers et al., NeurIPS 2023, 4-bit NF4 quantization); DoRA (NVIDIA, 2024); PEFT library (Hugging Face); JMedLoRA (Issey Sukeda, Masahiro Suzuki, Satoshi Kodera, Hiroki Sakaji; University of Tokyo, arXiv:2310.10083); JMedLLM-v1-7B (arXiv:2409.11783).

### Conceptual model
Freeze base weights; train low-rank update matrices on adapter layers. QLoRA quantizes base to 4-bit, enabling 70B fine-tuning on a single 48GB GPU. DoRA decomposes magnitude and direction for tighter approximation of full fine-tune.

### Expressiveness/Semantics
Captures style, terminology, format preferences — not new factual knowledge reliably. JMedLoRA finding: QLoRA on larger English-centric base (Llama2-70B-chat-hf) outperforms LoRA on Japanese-centric bases (OpenCALM-7B); Japanese-centric models show "deterioration of 1-shot performance after instruction-tuning."

### Composability/Modularity
Multiple adapters can be hot-swapped (S-LoRA, LoRAX). Mergeable into base for deployment.

### Suitability for autoformalization to IR
Useful for normalizing Japanese clinical phrasing → canonical clinical-concept tokens before formalization. Apply only if JMedBench-style holdout shows ≥3–5 pp gain over RAG-only on the specific guideline corpus.

### Formal verification potential
None.

### Tooling/Ecosystem maturity
Hugging Face PEFT, bitsandbytes, Unsloth, Axolotl. Inference servers (vLLM, LoRAX) support adapter swap.

### Japan-specific considerations
JMedBench (20 datasets, NAIST) + JMedLLM-v1-7B + JMedLoRA (UTokyo) are the canonical benchmarks. JaCWIR, JSQuAD, JMMLU-medical, IgakuQA for held-out evaluation. ELMTEX clinical-extraction study (60K PubMed Central summaries, 15 categories) shows "LoRA improved metrics by 10–20 points over the base model, while QLoRA improved by 8–14 points—only 2–4 points below LoRA." Catastrophic forgetting (Meditron→Japanese) is real.

### Interoperability (with Categories 1–7)
Adapter sits on §2 base; output consumed by §4 constrained decoder; gated by §6 idempotency metric to justify deployment.

### Limitations/Known issues
Small datasets cause overfit; instruction-tuning can degrade 1-shot performance (JMedLoRA finding). Catastrophic forgetting under additional pretraining with scarce data. Adapter for closed frontier models (§1) is unavailable — only open bases (§2, Llama, Qwen, Mistral, Gemma).

### Training data proxy
JMedLoRA: Japanese medical instruction pairs derived from licensing exam material on OpenCALM-7B / Llama2-70B-chat-hf bases. JMedLLM-v1-7B: 7B Qwen2 base + bilingual medical MFPT/MPEFT. Synthetic data generation from frontier models acceptable for adapter training but must be checked for hallucination.

---

## 12. World Models for Patient State and Trajectory: JEPA, Latent-Dynamics, Generative-Trajectory Backbones, and Clinical Instantiations (Clin-JEPA, SMB-Structure, EHRWorld, ETHOS, Foresight)

### Purpose
Provide predictive representations of clinical state — patient trajectories, disease progression, latent dynamics under candidate treatment policies — usable as **upstream encoders** of patient state for rule-evaluation stages, as **forward simulators** of guideline applications and counterfactual alternatives, or as **representation learners** over guideline prose for §7 retrieval. Architecturally orthogonal to the autoformalization stack of §1–§11: world models predict missing or future content (in latent space, in token space, or in pixel / event space) rather than emit IR; they are tools for representation and forecasting, not for producing Lean / SMT / JSON-schema IR. Included here because (a) any CDS that conditions on patient state must obtain that state estimate from somewhere, (b) guideline-impact and counterfactual-treatment evaluation increasingly use world-model rollouts as a substitute for trial simulation, and (c) the clinical world-model literature shifted in early 2026 from "design pattern" to "released artifacts," with Clin-JEPA, SMB-Structure, and EHRWorld all appearing as concrete models within four months of each other.

### Maintainer/Standards body

**General-purpose world models.** Three architectural families dominate the broader literature as of May 2026:

- *Joint-Embedding Predictive Architectures (JEPA) — Meta FAIR / LeCun lineage.* I-JEPA (Mahmoud Assran, Quentin Duval, Ishan Misra, Piotr Bojanowski, Pascal Vincent, Michael Rabbat, Yann LeCun, Nicolas Ballas; CVPR 2023; arXiv:2301.08243); V-JEPA (Adrien Bardes, Quentin Garrido, Jean Ponce, Xinlei Chen, Michael Rabbat, Yann LeCun, Mahmoud Assran, Nicolas Ballas; Meta, Feb 2024); V-JEPA 2 / V-JEPA 2-AC (Meta, 2025 — 1.2B params, internet-scale video pretraining plus action-conditioned post-training for robotic control; ViT-L / ViT-H / ViT-g checkpoints, non-commercial research license; zero-shot robot planning demonstrated on ~62 hours of robot data after generic video pretraining); H-JEPA hierarchical world-model proposal (LeCun, "A Path Towards Autonomous Machine Intelligence," OpenReview, June 2022); variational JEPA extensions positioning JEPA as a probabilistic world model (arXiv:2601.14354).
- *Latent-dynamics / RSSM-class world models — DeepMind lineage.* World Models foundational paper (David Ha, Jürgen Schmidhuber; NeurIPS 2018; arXiv:1803.10122); Dreamer / DreamerV2 / DreamerV3 (Danijar Hafner et al., Google DeepMind; DreamerV3 in Nature, January 2025, "Mastering diverse control tasks through world models" — single hyperparameter set across 150+ tasks); MuZero (Schrittwieser et al., DeepMind, Nature 2020); HyperTree Proof Search lineage shares the MCTS-over-latent-model design.
- *Observation-level generative world models — broad maintainer set.* Genie / Genie 2 / Genie 3 (Google DeepMind; Genie 3 released August 2025 as the first real-time interactive general-purpose world model rendering navigable 3D worlds at 24 fps, currently a limited research preview); Sora 2 (OpenAI); Veo 3 (Google); Cosmos (NVIDIA) — video-diffusion and autoregressive-video backbones increasingly framed as world models because their rollouts encode physical and behavioral dynamics implicitly.

Survey coverage: "Understanding World or Predicting Future? A Comprehensive Survey of World Models" (Tsinghua FIB Lab; ACM Computing Surveys 2025; arXiv:2411.14499) and "A Comprehensive Survey on World Models for Embodied AI" (arXiv:2510.16732) provide the now-standard taxonomy used below.

**Clinical world models / patient-trajectory foundation models.** Two waves are visible:

- *First-wave tokenized-EHR generative predictors (sometimes retroactively called "patient language models").* ETHOS (Pawel Renc, Yugang Jia, Anthony E. Samir et al., npj Digital Medicine 2024 — Enhanced Transformer for Health Outcome Simulation, zero-shot trajectory prediction on MIMIC-IV); Foresight (Zeljko Kraljevic, Dan Bean, Anthony Shek et al., King's College London, Lancet Digital Health 2024 — generative pretrained transformer over SNOMED-coded EHR); BEHRT (Yikuan Li et al., Scientific Reports 2020); Med-BERT (Laila Rasmy et al., npj Digital Medicine 2021); CEHR-BERT, CLMBR, MOTOR. These autoregressively predict event tokens (diagnoses, medications, labs, time deltas) and are observation-level generative WMs in the survey taxonomy.
- *Second-wave clinical world models (2026).* **Clin-JEPA** (Yixuan Yang, Mehak Arora, Ryan Zhang, Baraa Abed, Junseob Kim, Tilendra Choudhary, Md Hassanuzzaman, Kevin Zhu, Ayman Ali, Chengkun Yang, Alasdair Edward Gent, Victor Moas, Rishikesan Kamaleswaran; arXiv:2605.10840, submitted May 11 2026; v2 May 12 2026; CC BY 4.0; code at `github.com/YeungYathin/Clin-JEPA`). **SMB-Structure** ("The Patient is not a Moving Document: A World Model Training Paradigm for Longitudinal EHR" — Irsyad Adam, Zekai Chen, David Laprade, Shaun Porwal, David Laub, Erik Reinertsen, Arda Pekis, Kevin Brown; arXiv:2601.22128, Jan 29 2026; 1.7B-param weights on Hugging Face at `standardmodelbio/SMB-v1-1.7B-Structure`). **EHRWorld** ("EHRWorld: A Patient-Centric Medical World Model for Long-Horizon Clinical Trajectories" — Linjie Mu, Zhongzhen Huang, Yannian Gu, Shengqian Qin, Shaoting Zhang, Xiaofan Zhang; arXiv:2602.03569, Feb 3 2026; released alongside EHRWorld-110K, a 110K-record longitudinal dataset).

### Conceptual model

The 2025 surveys converge on a two-axis taxonomy: prediction target (observation-level vs. latent-embedding) × generativity (with vs. without an explicit decoder back to observations). The three resulting families that matter for clinical use:

**(a) Observation-level generative world models.** Predict the next *observation* directly — frames, tokens, mesh updates. Autoregressive (Sora 2, Cosmos, Genie 3 autoregressive variants), NeRF / 3D-Gaussian-splatting, and video diffusion. Cost: must model fine-grained surface detail; risk of hallucinated detail. Benefit: outputs are human-interpretable and can be inspected directly. Clinical analogues are the *tokenized-EHR generative predictors* (ETHOS, Foresight, BEHRT, Med-BERT, EHRWorld): events are tokenized (ICD / SNOMED / RxNorm / lab codes plus time deltas), and a transformer autoregressively emits future events. EHRWorld explicitly frames itself as a "patient-centric medical world model" and emphasizes a "causal sequential paradigm" for long-horizon trajectory simulation.

**(b) Latent-dynamics world models with generative reconstruction (RSSM-class).** Learn a recurrent latent state $s_t$, a transition $s_{t+1} = T(s_t, a_t)$, an observation decoder, and a reward predictor; train agents by rolling out *imagined* trajectories through $T$ and updating policies on the imagined rollouts. DreamerV3 is the canonical example; MuZero combines this with MCTS planning over the learned model. The decoder grounds the latents to observations during training but is typically discarded at planning time. Clinical instantiation has been limited — no production clinical Dreamer exists — though "digital twin" patient simulators in cardiology and ICU research occupy this architectural niche.

**(c) Joint-Embedding Predictive Architectures (JEPA) — latent prediction without reconstruction.** Given a context $x$ and a target $y$, learn an encoder $f_\theta$ and a predictor $g_\phi$ such that $g_\phi(f_\theta(x))$ matches $f_\theta(y)$ in latent space (typically with an EMA-tracked target encoder to prevent representation collapse). No decoder; the predictor reconstructs the *embedding* of the target, not its pixels / tokens. Surveys frame the central distinction as: "RSSM relies on generative reconstruction of observations to model latent dynamics, whereas JEPA employs self-supervised predictive coding in embedding spaces directly forecasting future state representations without decoding to raw sensory inputs." Pretext task variants: masked image regions (I-JEPA), masked spatio-temporal video tubes (V-JEPA), action-conditioned future prediction (V-JEPA 2 / 2-AC).

**Clinical instantiations of family (c).**
- **Clin-JEPA** (Yang et al., arXiv:2605.10840). Applies JEPA to longitudinal EHR by co-training a *Qwen3-8B-based encoder* and a 92M-parameter latent trajectory predictor. The paper's core diagnosis is that prior JEPAs either discard the predictor after pretraining (I-JEPA, V-JEPA) or train the predictor on a frozen encoder (V-JEPA 2-AC), leaving the encoder unaware of the rollout signal the retained predictor must use at inference. Naïve co-training collapses; Clin-JEPA's five-phase pretraining curriculum (predictor warmup → joint refinement → EMA target alignment → hard sync → predictor finalization) stabilizes co-training by phase, addressing representation collapse and online/target drift. The result is a *single backbone* used both for autoregressive latent rollout and for multi-task downstream risk prediction without per-task fine-tuning. Trained on MIMIC-IV ICU.
- **SMB-Structure** (Adam et al., arXiv:2601.22128). A 1.7B-parameter hybrid: a JEPA prediction objective is *grounded* by an SFT next-token-prediction objective on the same structured EHR sequence. SFT forces token-space reconstruction of future patient states; the JEPA head simultaneously predicts those futures in latent space from the initial patient representation alone, "forcing trajectory dynamics to be encoded before the next state is observed." The framing — "the patient is not a moving document to summarize but a dynamical system to simulate" — explicitly contrasts world-model training against next-token-only clinical LLM pretraining. Validated on Memorial Sloan Kettering oncology (23,319 patients, 323,000+ patient-years) and INSPECT pulmonary-embolism cohort (19,402 patients).

In short, family (a) gives interpretable token-level trajectories at the cost of surface-detail hallucination; family (b) gives planning-ready latent dynamics at the cost of decoder modeling; family (c) gives clean state embeddings without a decoder at the cost of needing a separate evaluation interface. Clin-JEPA, SMB-Structure, and EHRWorld together span (c), (c)+(a) hybrid, and (a) respectively for clinical data.

### Expressiveness/Semantics

None of these architectures natively expresses the LTL / MTL / deontic / arithmetic constraints that clinical-rule verification requires; they predict, they do not reason in formal calculi. Family (a) generative predictors output event tokens drawn from ICD / SNOMED / RxNorm / MEDIS-DC vocabularies, so the terminology layer is exposed downstream — but the events are predicted, not entailed by a logical rule. Family (b) supports counterfactual rollouts of the form "$\hat{s}_{t+k}$ given action $a_t$" but the counterfactual is statistical: it is *not* a causal estimate unless training was interventional or the model was designed as a structural causal model (g-formula / target-trial emulation / DAG-conditioned). Family (c) JEPA encoders output dense vectors with no symbolic structure. Clin-JEPA specifically reports two semantic-geometry findings worth quoting: latent ℓ₁ rollout drift uniquely *converges* (−15.7%) over a 48-hour horizon while baselines and ablations diverge by +3% to +4951%, and the learned latent geometry is clinically discriminative — deteriorating-patient cohorts displace 4.83× further in latent space than stable patients, vs. ≤2.62× for the baseline encoders the authors evaluate.

### Composability/Modularity

JEPA encoders are clean upstream modules: pretrained weights produce embeddings that any classifier, retriever, regressor, or LLM-with-adapters can consume. RSSM-class agents are typically end-to-end pipelines, but the learned latent state $s_t$ is reusable. Tokenized-EHR generative predictors (ETHOS, Foresight, EHRWorld) are pretrained transformers — fine-tunable, distillable, embeddable, and exposable behind MCP. Clin-JEPA inherits a Qwen3-8B backbone, which means the encoder embeddings live in a space that LLM-side adapters can be conditioned against directly; SMB-Structure publishes 1.7B-param weights on Hugging Face under a research license. V-JEPA 2 publishes ViT-L / ViT-H / ViT-g checkpoints under non-commercial research license; commercial use requires separate agreement with Meta.

### Suitability for autoformalization to IR

Low to none **directly**. World models do not emit Lean, SMT, or JSON-schema IR. Their roles in a CDS pipeline are indirect and orthogonal to the autoformalization stack:

- **Upstream state encoders.** Supply patient-state embeddings (Clin-JEPA, SMB-Structure encoder outputs) or cluster labels to the rule-evaluation stage, so an IR rule can be conditioned on a latent state — e.g., "patient currently in trajectory cluster matching pre-decompensation pattern" → triggers a Lean-encoded surveillance rule from §3.
- **Forward / counterfactual simulators for guideline evaluation.** Roll out a world model under policy $\pi$ = "apply guideline G as written" versus a counterfactual policy, and compare predicted outcomes. EHRWorld and ETHOS support this directly via autoregressive token generation; Dreamer-style latent rollouts in principle as well. Useful for *guideline-impact estimation*, not for proving guideline correctness.
- **Representation learners for guideline prose.** Applying a JEPA-style objective to guideline text (context = surrounding paragraph, target = masked clinical-statement embedding) could improve §7 premise retrieval beyond standard contrastive sentence embeddings; no published clinical-guideline JEPA exists yet.

Best paired with frontier-LLM autoformalizers from §1, not used in their place.

### Formal verification potential

None native. World models are statistical predictors; their outputs carry no certificate. Verification still routes through §3 (Lean) and SMT. World-model outputs can however become the *subject* of verification — e.g., prove that a Lean-encoded rule, evaluated on a world-model-predicted state, agrees with the rule evaluated on the ground-truth state when that state is later observed. This is regression testing of the predictor, not a proof of model correctness, but it slots cleanly into §10 verifier-guided loops as a soft signal.

### Tooling/Ecosystem maturity

Modest for general world models and uneven for clinical ones; gap relative to the LLM ecosystem of §1 remains large.

- *General.* Meta FAIR releases I-JEPA, V-JEPA, and V-JEPA 2 weights and training code on GitHub (`facebookresearch/jepa`, `facebookresearch/vjepa2`); Hugging Face hosts V-JEPA 2 checkpoints. DreamerV3 reference implementation by Hafner on GitHub. Genie 3 is currently a limited DeepMind research preview, not generally available. No equivalent of vLLM / TensorRT-LLM serving stacks for JEPA-class inference; reproducibility of JEPA training is sensitive to EMA target schedules and masking strategies. Two recent surveys (Tsinghua FIB CSUR 2025 / arXiv:2411.14499; arXiv:2510.16732) and a curated reading list (`tsinghua-fib-lab/World-Model`) give the current overview.
- *Clinical.* ETHOS code public; Foresight ships through the CogStack ecosystem at King's College London. **Clin-JEPA**: code at `github.com/YeungYathin/Clin-JEPA`, 17-page paper with 4 figures and 8 tables, no production-serving recipe yet. **SMB-Structure**: 1.7B model weights on Hugging Face (`standardmodelbio/SMB-v1-1.7B-Structure`). **EHRWorld**: paper plus EHRWorld-110K dataset release; production-serving status unclear at time of writing.

### Japan-specific considerations

No Japanese variant of any general world model has been released. None of the three 2026 clinical world models (Clin-JEPA, SMB-Structure, EHRWorld) is trained on Japanese cohorts — Clin-JEPA is MIMIC-IV ICU only; SMB-Structure is MSK + INSPECT (US); EHRWorld trains on EHRWorld-110K (provenance not Japan-specific). Building a Japanese clinical world model faces the same PHI / data-licensing constraints as §2 medical LLM training: SS-MIX2 / MID-NET / NDB / Tokushima 千年カルテ linkage requires institutional access and cross-institution sharing under APPI is restrictive. Architectural choices that *could* port to Japanese data with comparatively modest adaptation: image / video JEPA on Japanese medical imaging (X-ray, endoscopy, pathology, ophthalmology), which is text-modality-free; and ETHOS-style tokenized-EHR predictors using MEDIS-DC, JLAC10, YJ drug codes, and HOT codes as the event vocabulary, with tokenizer compatibility checked against the JMedBench evaluation harness (see §2 Japan-specific considerations). A Japanese Clin-JEPA — say, swapping the Qwen3-8B encoder for a JMedLLM- or Llama2-70B-chat-hf-style Japanese-capable base, and pretraining on SS-MIX2-derived ICU cohorts — is technically straightforward but legally non-trivial.

### Interoperability (with Categories 1–7)

State embeddings consumable by retrieval indices in the broader RAG / IR stack (Categories 6–7 of the overall CDS taxonomy) and by frontier-LLM prompts (described as numeric features in text, or — for open-weight bases — passed as adapter-conditioning tokens; Clin-JEPA's Qwen3-8B encoder is particularly convenient for the latter). Trajectory predictors can be wrapped as MCP tools (§5) for query-time consultation: `tool: predict_trajectory(patient_state, candidate_policy, horizon_hours) → predicted_events_or_latent_state`. Cross-modal alignment with FHIR, SNOMED CT, ICD-10, RxNorm, and Japanese MEDIS-DC / JLAC10 vocabularies is a prerequisite for clinical grounding of token-level outputs from family (a) models.

### Limitations/Known issues

Family-(a) generative predictors hallucinate event tokens at the surface level (the same failure mode as text-LLM hallucination, transposed to clinical events) and inherit EHR coding noise, missingness, and selection bias from source datasets (MIMIC-IV, CPRD, Cerner Health Facts, MSK, INSPECT, EHRWorld-110K). Family-(b) latent-dynamics models suffer compounding error over long horizons (DreamerV3 documents this; rollouts beyond ~50 imagined steps degrade rapidly), which is acute over clinical timescales of months to years. Family-(c) JEPA non-generativity is a feature for representation learning and a liability for regulatory review — the encoder is a black box without a complementary decoder, and downstream usage must be designed around that. Counterfactual rollouts across all families are *not* causal estimates absent design (no g-formula structure, no target-trial emulation), a well-known pitfall when these models are repurposed for treatment-effect estimation. Specific to **Clin-JEPA**: single-cohort training (MIMIC-IV ICU only); generalization to non-ICU, non-US, and pediatric populations is unevaluated; the Qwen3-8B encoder is large for clinical deployment compared to 1.7B SMB-Structure; the five-phase curriculum is reproducible from the paper but sensitive to phase-boundary scheduling. Specific to **SMB-Structure**: oncology- and PE-cohort biases; the JEPA+SFT hybrid is novel and may not transfer to settings with sparser longitudinal data. Specific to **EHRWorld**: numerical benchmark results in the public abstract are qualitative ("significantly outperforms naive LLM-based baselines"); independent reproduction pending. None of the three has CDS regulatory clearance in any jurisdiction.

### Training data proxy

*General.* I-JEPA: ImageNet-1K / 22K. V-JEPA: ~2M internet video clips. V-JEPA 2: large-scale internet video plus ~62 hours of robot data for action-conditioned post-training. DreamerV3: 150+ tasks spanning Atari, DMControl, ProcGen, Crafter, Minecraft. Genie 3 / Sora 2 / Veo 3: undisclosed large internet-video corpora.

*Clinical.* ETHOS: MIMIC-IV (~300K ICU and ED stays). Foresight: King's College Hospital + Maudsley NHS Foundation Trust records, SNOMED-coded. BEHRT: CPRD UK primary care. Med-BERT: Cerner Health Facts. **Clin-JEPA**: MIMIC-IV ICU, evaluated with ICareFM EEP and an 8-task binary risk benchmark — reported mean AUROC 0.851 on ICareFM EEP and 0.883 across the 8 binary tasks (+0.038 and +0.041 vs. baseline average). **SMB-Structure**: MSK oncology (23,319 patients, 323,000+ patient-years) + INSPECT (19,402 PE patients). **EHRWorld**: EHRWorld-110K longitudinal clinical records.

*For a Japanese clinical world model:* training data would need to be assembled from SS-MIX2 / MID-NET / NDB / 千年カルテ extracts under explicit IRB approval and APPI compliance; no such public training corpus exists as of May 2026, and the legal pathway to construct one without per-institution data-use agreements is not established.