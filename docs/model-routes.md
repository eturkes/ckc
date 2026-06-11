# Language Models, Constrained Decoding, and Agentic Autoformalization

## 1. Closed Frontier Model Ensembles: GPT-5.5, Claude Opus 4.7, Gemini 3.1 Pro
### Purpose
Top-tier general reasoning, long-context, multilingual (incl. Japanese) comprehension, tool use, structured output for autoformalizing clinical guideline prose into IR.
### Maintainer/Standards body
OpenAI (GPT-5, GPT-5.5, GPT-5.5 Pro; released April 23, 2026); Anthropic (Claude Opus 4.5 released Nov 24, 2025; Opus 4.7 released April 16, 2026 — current GA; Sonnet 4.5; Haiku 4.5); Google DeepMind (Gemini 3 Pro released Nov 18, 2025; Gemini 3.1 Pro released Feb 19, 2026; Deep Think mode).
### Conceptual model
Closed-weight transformer LLMs with extended-thinking / "reasoning effort" params (low/medium/high), native multimodality (Gemini 3.x), adaptive thinking budgets (Opus 4.7). Ensemble members; outputs aggregated by self-consistency, multi-agent debate, or judge models.
### Expressiveness/Semantics
Free-form NL with strong instruction following; JSON / Structured Output modes; function-calling schemas; citation modes (Anthropic citations API, OpenAI structured outputs). All support 200K–1M token context (Gemini 3.1 Pro: 1M input / 65,536 output; Opus 4.5: 200K; Opus 4.7: up to 1M; GPT-5.5: 1M).
### Composability/Modularity
High. All support tool use / function calling, MCP-aware, prompt caching (Opus 4.7: up to 90% input cost savings via prompt caching, 50% via batch; GPT-5.5 batch 50% discount), batch APIs. Compose in agent graphs (LangGraph, OpenAI Agents SDK, Claude Code).
### Suitability for autoformalization to IR
Strong. Frontier reasoning + JSON-schema structured output → high pass@1 IR from Japanese clinical text. Opus 4.7 best agentic/tool-calling consistency (XBOW reports "98.5% on our visual-acuity benchmark versus 54.5% for Opus 4.6"); Gemini 3.1 Pro leads ARC-AGI-2 (77.1%) and abstract reasoning; GPT-5.5 leads Terminal-Bench 2.0 (82.7%) and FrontierMath. None specialized for Lean; pair with prover models (§3).
### Formal verification potential
Indirect: generate IR / Lean stubs, cannot self-verify. Verification delegated to Lean 4 + Mathlib4 and SMT solvers via tool calls. Opus 4.7 noted by Vercel to "do proofs on systems code before starting work."
### Tooling/Ecosystem maturity
Mature. SDKs in Python/TS; Bedrock/Vertex/Azure for all three. MCP donated by Anthropic to the Linux Foundation Agentic AI Foundation on December 9, 2025 (Linux Foundation press release: "SAN FRANCISCO, Dec. 9, 2025 – The Linux Foundation…today announced the formation of the Agentic AI Foundation (AAIF), and founding contributions of three leading projects…Anthropic's Model Context Protocol"); 10,000+ published MCP servers, 97M+ monthly SDK downloads.
### Japan-specific considerations
All handle Japanese well; Gemini 3.x and Claude Opus 4.x have explicit Japanese benchmark coverage. Data residency: Vertex AI Tokyo, Bedrock Tokyo, Azure Japan East. Cross-border PHI restricted under APPI; no on-prem/VPC inference — API only.
### Interoperability (with Categories 1–7)
Native MCP clients; OpenAPI function-calling; integrate with FHIR servers, UMLS / BioPortal terminology services, Lean-as-MCP-tool, SMT-solver-as-tool, vector DBs. Structured Output binds directly to Pydantic / JSON Schema IR.
### Limitations/Known issues
Closed weights (no self-host for PHI); price ($5/$25 per Mtok Opus 4.7, $30/$180 per Mtok GPT-5.5 Pro output); rate limits; non-determinism even at temperature 0; hallucination on rare Japanese clinical terminology; benchmark contamination concerns (Anthropic notes decontamination changes in Opus 4.5 evals).
### Training data proxy
Web crawl + licensed data + RLHF + RLAIF; medical/Japanese coverage not quantitatively disclosed. GPT-5.5 trained for agentic computer-use trajectories; Opus 4.5 for long-horizon coding; Gemini 3.1 Pro on Gemini 3 Pro foundation + extra reasoning RL.

## 2. Domain Medical Models: Med-Gemini, MedGemma 1.5, Meditron, GatorTron, LLaVA-Med, JMedLLM, UTH-BERT
### Purpose
Domain-specialized weights for clinical text/image comprehension, medical QA, EHR extraction, clinical NER, terminology normalization (drug names, lab codes, diagnosis codes), de-identification, and — most relevant here — pre-parsing Japanese clinical guidelines into structured medical concepts before frontier-LLM formalization. Secondary roles: upstream encoders for §7 premise retrieval, judges/critics in §8, constrained-decoding-ready generators paired with §4 grammars over clinical vocabularies.
### Maintainer/Standards body
**Frontier closed medical.** Google DeepMind/Research: Med-PaLM 2 (Singhal et al., 2023; 86.5% MedQA at release); Med-Gemini (Saab et al., arXiv:2404.18416, 2024; 91.1% MedQA, +4.6 pp over Med-PaLM 2, SOTA 10/14 medical benchmarks at release incl. multimodal and long-context); AMIE — Articulate Medical Intelligence Explorer (Tu et al., Google DeepMind; diagnostic-dialogue agent, Nature 2025; simulated patient self-play + clinician RLHF over Gemini foundation); MedLM — productized Vertex AI offering on Med-PaLM/Med-Gemini stack under Healthcare-tuned commercial terms.

**Open-weight medical LLMs.** Google DeepMind (MedGemma 4B / 27B text-only / 27B multimodal under HAI-DEF; **MedGemma 1.5 released January 2026** on Gemma 3 backbone, ~91% MedQA, de-facto default open clinical base); EPFL LiGo (Meditron-7B / Meditron-70B on Llama 2, GAP-Replay + PubMed full-text); Stanford CRFM (BioMedLM 2.7B); Shanghai Jiao Tong (PMC-LLaMA on Llama 2); M42 Health (Med42 / Med42-v2 70B on Llama 2); Saama AI Research (OpenBioLLM-8B / OpenBioLLM-70B on Llama 3); BioMistral team (BioMistral 7B on Mistral); Microsoft Research (BioGPT — early generative biomedical model); CUHK/HKUST (HuatuoGPT / HuatuoGPT-II Chinese medical); FreedomIntelligence (Apollo / Apollo2 multilingual medical 7B: English / Chinese / French / Spanish / Hindi / Arabic).

**Clinical-text encoders (workhorse for NER, extraction, embedding).** UF Health + NVIDIA (GatorTron 345M–8.9B; >90B words clinical text from UF Health EHR, SOTA on i2b2 subtasks at release); Microsoft Research (PubMedBERT / BiomedNLP-PubMedBERT — domain-pretrained from scratch on PubMed); KU Leuven / DMIS Lab (BioBERT — Lee et al.); MIT CSAIL + BIDMC (ClinicalBERT — Alsentzer et al.; pretrained on MIMIC-III discharge summaries); AI2 (SciBERT); NIH (BlueBERT).

**Multimodal medical.** Google DeepMind (MedGemma 27B-MM with SigLIP-400M vision encoder; Med-Gemini multimodal); Microsoft (LLaVA-Med — Li et al., arXiv:2306.00890 — biomedical VQA on LLaVA architecture; BiomedCLIP image-text contrastive encoder); LLaVA-Rad — radiology-specialized variant that "outperforms publicly available report generation models … generating fewer errors than LLaVA-Med and general-domain LLaVA" on chest X-ray; Stanford (CheXagent, chest X-ray); UCSD (BiomedGPT generalist medical multimodal); Med-PLIB pixel-level biomedical multimodal (arXiv:2412.09278; SOTA on pixel grounding for biomedicine); Meissa — multi-modal medical agentic intelligence (arXiv:2603.09018).

**Japanese medical.** UTokyo Hospital (**UTH-BERT** — Kawazoe et al.; first Japanese medical LM, BERT pretrained on ~120M UTH clinical texts); NTT (**JMedRoBERTa** — RoBERTa on Japanese medical paper abstracts + main text); UTokyo Cardiovascular Medicine + Systems Innovation (JMedLoRA — Sukeda, Suzuki, Kodera, Sakaji; arXiv:2310.10083; JMedLLM-v1-7B — arXiv:2409.11783, Qwen2-7B base + bilingual medical pretraining + LoRA); 70B-param Japanese-medical-QA evaluation (Sukeda et al., arXiv:2406.14882); Japanese-capable substrate for medical fine-tunes — **Swallow** project (Tokyo Tech / AIST / NII): Llama-3.3-Swallow-70B (2025), GPT-OSS-120B Swallow variant (2026), continuation-pretrained on Swallow Corpus v2/v3 + Japanese web + math/code (methodology: arXiv:2404.17790). No major Japanese medical foundation model is multimodal as of May 2026.

World models for patient state/trajectory — JEPA family (I-JEPA, V-JEPA, V-JEPA 2), Dreamer-class latent-dynamics, generative trajectory predictors (ETHOS, Foresight, EHRWorld), JEPA-class clinical instantiations Clin-JEPA (Yang et al., arXiv:2605.10840) and SMB-Structure (Adam et al., arXiv:2601.22128) — covered in §12; they predict in latent/event space rather than generate IR.
### Conceptual model
Three families. **Decoder-only transformers** fine-tuned/continuation-pretrained on biomedical corpora: MedGemma 1.5 on Gemma 3; Med-Gemini on Gemini Pro; Meditron/Med42/OpenBioLLM/BioMistral/PMC-LLaMA/Apollo on Llama/Mistral; JMedLLM-v1-7B on Qwen2-7B. **Encoder-only (BERT-class)** for clinical NER, extraction, embedding, classification: PubMedBERT, BioBERT, ClinicalBERT, SciBERT, BlueBERT, GatorTron-base, UTH-BERT, JMedRoBERTa. **Multimodal** (vision-encoder + projector + LLM, optionally contrastive image-text head): MedGemma-MM (SigLIP-400M vision encoder, 896×896 medical images interleaved with text), LLaVA-Med/LLaVA-Rad, CheXagent, BiomedGPT, BiomedCLIP, Med-PLIB. **AMIE** is an orchestration pattern on a Gemini-class base (self-play diagnostic dialogue with simulated patients + expert-clinician RLHF), the canonical published *clinical dialogue agent*, relevant to clarification-question generation against ambiguous guideline text. Complementary world-model paradigms — latent-dynamics (Dreamer-class), JEPA-class, observation-level generative trajectory prediction (ETHOS/EHRWorld-class) — in §12.
### Expressiveness/Semantics
Strong on medical QA (English benchmarks). **Med-Gemini** 91.1% MedQA (USMLE; Saab et al. 2024), SOTA 10/14 medical benchmarks at release. **MedGemma 1.5** (Jan 2026) ~91% MedQA among open-weight. **MedGemma 27B text-only** 87.7% MedQA at ~one-tenth DeepSeek R1 inference cost (per Google Research). Multimodal **MedGemma 27B-MM** accepts arbitrarily interleaved 896×896 medical images + text. **JMedLLM-v1-7B** >50% on IgakuQA (Japanese medical licensing exam). Encoder-only on extraction: **GatorTron** SOTA on i2b2 subtasks at release, **ClinicalBERT** standard for clinical phenotyping. Radiology specialization gradient: LLaVA-Rad < LLaVA-Med < generic LLaVA in error rate (domain specialization still pays for image tasks). Caveat: §1 frontier models now match/exceed many domain fine-tunes on aggregate medical benchmarks; medical-specific advantages are open-weight licensing (PHI deployability), cost, latency — not raw accuracy.
### Composability/Modularity
Open weights for full open-medical line (MedGemma 1.5, Meditron, BioMedLM, PMC-LLaMA, BioMistral, Med42, OpenBioLLM, HuatuoGPT-II, Apollo, BioGPT, JMedLLM-v1-7B; all on Hugging Face, varying licenses) → §1 ensemble members, §7 RAG retriever encoders, or §8 judges. **MedSigLIP** and **BiomedCLIP** image-text encoders reusable as multimodal embedding models. Clinical text encoders (PubMedBERT, BioBERT, ClinicalBERT, SciBERT, BlueBERT, GatorTron-base, UTH-BERT, JMedRoBERTa) feed vector DBs for §7 premise retrieval over MEDIS-DC/Minds/JLAC10/PubMed — usually faster/cheaper than routing every retrieval through a frontier LLM. AMIE-style diagnostic-dialogue orchestration composes with §5 tool-calling agents for clarification rounds. (World-model encoders — JEPA, Dreamer-latent, tokenized-EHR — as upstream feature extractors or downstream trajectory simulators: §12.)
### Suitability for autoformalization to IR
Indirect. Use as (a) **Japanese clinical-term normalizer** → tagged stream of canonical MEDIS-DC/JLAC10/ICD-10-J concepts before §1 formalization, (b) **embedder for §7 premise retrieval**, (c) **judge/critic in §8**, (d) **AMIE-style dialogue front-end** for clarification on ambiguous guideline statements, (e) **multimodal extractor** for scanned figures/decision trees/tables. Encoder-only models cannot generate IR but are most efficient for embedding and clinical-NER. None produce Lean/SMT directly: medical LLMs are trained for QA/dialogue/extraction, not formal-language generation.
### Formal verification potential
None native. Output piped to provers/SMT (§3) or wrapped behind a §5 tool-calling agent delegating verification.
### Tooling/Ecosystem maturity
**Open-weight.** MedGemma 1.5 on Hugging Face, Vertex Model Garden, GKE recipes; compatible with vLLM/TGI/TensorRT-LLM. Meditron, BioMedLM, BioMistral, Med42, OpenBioLLM, HuatuoGPT-II, Apollo, BioGPT, JMedLLM, JMedLoRA, LLaVA-Med, LLaVA-Rad, CheXagent, BiomedGPT, BiomedCLIP — research/community artifacts on Hugging Face and/or GitHub. GatorTron weights gated through UF Health BRIDGE2AI access. PubMedBERT, BioBERT, ClinicalBERT, SciBERT, BlueBERT, UTH-BERT, JMedRoBERTa freely available. **Closed.** Med-Gemini, Med-PaLM 2, AMIE only via Vertex AI under MedLM commercial agreement. **Benchmarks/leaderboards.** MedQA-USMLE, MedMCQA, PubMedQA, MMLU-Medical, MultiMedBench, HealthBench (OpenAI, 2025), **Medmarks v0.1** (arXiv:2605.01417, 2026 open-source medical benchmark suite); Medical LLM Leaderboard tracks scores publicly. Google Health publishes a MedQA relabeling artifact (`Google-Health/med-gemini-medqa-relabelling`) documenting label-noise on the canonical benchmark.
### Japan-specific considerations
**Benchmarks.** JMedBench (NAIST Social Computing Lab) — canonical Japanese biomedical LLM benchmark, 20 datasets across MCQA/MT/NER/DC/STS, incl. IgakuQA (Japanese medical licensing exam), JMMLU-medical, translated MedQA/PubMedQA. A September 2025 medRxiv comparative evaluation (Japanese National Medical Examination) reports standings of frontier general models, MedGemma, and Japanese-medical fine-tunes on the national licensing exam. **Specialized encoders.** UTH-BERT (UTokyo Hospital, ~120M clinical-text pretraining) and JMedRoBERTa (NTT, Japanese medical paper corpus) are canonical; UTH-BERT better for NER over hospital-style free text (sees actual EHR prose), JMedRoBERTa biases toward biomedical-literature style. **Generative/adapters.** JMedLLM-v1-7B (arXiv:2409.11783; Qwen2-7B base) and JMedLoRA (arXiv:2310.10083). JMedLoRA finding: QLoRA on a larger English-centric base (Llama2-70B-chat-hf) outperforms LoRA on Japanese-centric bases (OpenCALM-7B), with Japanese-centric models showing "deterioration of 1-shot performance after instruction-tuning." **Substrate.** Swallow (Tokyo Tech/AIST/NII) — standard Japanese-extended Llama/Mistral lineage, **Llama-3.3-Swallow-70B** (2025), **GPT-OSS-120B Swallow variant** (2026) — most common base for new Japanese medical fine-tunes. **Catastrophic forgetting** documented: Meditron's Japanese degradation is the standard example; most direct Japanese fine-tuning of English-centric medical bases shows it. **No Japanese multimodal medical foundation model** as of May 2026 — Japanese radiology/pathology/ophthalmology multimodal work piggybacks on MedGemma-MM or LLaVA-Med with translated prompts.
### Interoperability (with Categories 1–7)
Hugging Face transformers, vLLM, Outlines/xgrammar constrained decoding (§4) across the open line; SigLIP/BiomedCLIP/MedSigLIP embeddings consumable by vector DBs; encoder-only clinical models (PubMedBERT, ClinicalBERT, GatorTron-base, UTH-BERT, JMedRoBERTa) plug into RAG indices for §7 premise retrieval over MEDIS-DC/Minds/JLAC10. All servable behind MCP (§5). Med-Gemini/AMIE via Vertex AI APIs (MedLM commercial terms) where open weights aren't required.
### Limitations/Known issues
MedGemma requires per-use-case validation (Google labels it a "developer model"); none is regulator-approved CDS in Japan; U.S. FDA clearance for any specific medical LLM remains rare even amid >950 cleared AI medical devices as of early 2026, most imaging/signals rather than text generators. Meditron Japanese degradation and analogous catastrophic-forgetting failures are common across Llama-based medical fine-tunes pushed cross-lingual. PHI/data-licensing for Japanese fine-tuning datasets restrictive under APPI. **Benchmark contamination** well-documented for MedQA and PubMedQA (frontier pretraining corpora overlap test sets); Google's MedQA relabeling documents label-noise independently. Hallucination on rare Japanese drug brand names, less-common ICD-10-J codes, JLAC10 lab codes, pediatric/rare-disease guidelines is significant; AMIE-style agents add failure modes — sycophancy under user pressure, premature anchoring on initial differential. Encoder-only models cap at ~512 tokens (vanilla BERT-class), constraining long guideline prose without chunking. (2026 clinical world-model limits — Clin-JEPA single-cohort MIMIC-IV-ICU, SMB-Structure MSK/INSPECT cohort biases, EHRWorld qualitative-only public benchmarks, no Japanese variants — see §12.)
### Training data proxy
**Frontier closed.** Med-Gemini: Gemini Pro base + medical fine-tune corpus (PubMed abstracts, clinical case discussions, USMLE prep, medical textbooks; mixture not fully disclosed). AMIE: simulated diagnostic dialogues via self-play + RLHF over expert clinician feedback. **Open generalist.** MedGemma 1.5: medical QA pairs, FHIR EHR (27B-MM), radiology/histopath/ophthalmology/dermatology images, public medical text. Meditron: GAP-Replay + PubMed full-text. BioMedLM: PubMed abstracts. PMC-LLaMA: PubMed Central full-text on Llama. Med42/OpenBioLLM: curated medical instruction data on Llama 2/Llama 3. BioMistral: PubMed + clinical guidelines on Mistral 7B. HuatuoGPT-II: Chinese medical corpora (HuatuoCorpus). Apollo: multilingual medical corpora, 6 languages. **Encoders.** GatorTron: 82B+ words UF Health clinical text + PubMed + MIMIC. PubMedBERT/BioBERT: PubMed abstracts + full-text. ClinicalBERT: MIMIC-III discharge summaries on BioBERT. **Japanese.** UTH-BERT: ~120M UTokyo Hospital clinical texts. JMedRoBERTa: Japanese medical paper abstracts + main text. JMedLoRA: Japanese medical instruction pairs from licensing-exam material on OpenCALM-7B/Llama2-70B-chat-hf. JMedLLM-v1-7B: Qwen2-7B base + bilingual medical pretraining. Swallow substrate: Wikipedia, DCLM-baseline-1.0, Swallow Corpus v2/v3, Cosmopedia, Laboro ParaCorpus, FineMath-4+, Swallow Code. Synthetic data from frontier models acceptable for adapter training but must be checked for hallucination (§11).

## 3. Formal-Proof Models and Environments: DeepSeek-Prover-V2, LeanDojo, Leanstral
### Purpose
Generate machine-checkable Lean 4 proofs of theorems/goals; verify consistency of IR-encoded clinical rules as Lean propositions; detect contradictions between guideline statements.
### Maintainer/Standards body
DeepSeek AI (Prover-V1.5, Prover-V2-671B, Apr 2025); Princeton/Tsinghua/Meta FAIR/Numina (Goedel-Prover V1 Feb 11 2025 — Yong Lin, Shange Tang, Bohan Lyu, Jiayun Wu, Hongzhou Lin, Kaiyu Yang, Jia Li, Mengzhou Xia, Danqi Chen, Sanjeev Arora, Chi Jin — arXiv:2502.07640; V2 Aug 5 2025 — arXiv:2508.03613); Numina + Moonshot Kimi (Kimina-Prover 72B); Google DeepMind (AlphaProof, IMO 2024 silver, Nature Nov 2025); Caltech/NVIDIA (LeanDojo + ReProver — Yang, Swope, Gu, Chalamala, Song, Yu, Godil, Prenger, Anandkumar; NeurIPS 2023 Datasets & Benchmarks oral); Mistral AI (Leanstral, released March 16 2026 — 120B total / 6B active sparse MoE, Apache 2.0, for Lean 4 proof engineering); Lean FRO/Microsoft Research (Lean 4, Mathlib4).
### Conceptual model
LLMs fine-tuned/RL-trained on Lean tactic sequences or whole proofs. Two paradigms: (a) tactic-step models with tree search (ReProver, InternLM2.5-StepProver, AlphaProof MCTS); (b) whole-proof reasoning models (Kimina-Prover, DeepSeek-Prover-V2 CoT mode, Goedel-V2). Some use recursive subgoal decomposition (DeepSeek-V3 decomposes → 7B solves subgoals → recombine).
### Expressiveness/Semantics
Lean 4 + Mathlib4 covers higher-order logic, dependent types, classical and constructive math. Sufficient for clinical rule logic (temporal, deontic via embedded modal logic libraries, arithmetic comparisons over lab values).
### Composability/Modularity
LeanDojo: programmatic Lean interaction (extracts tactic states, premises). Kimina-Lean-Server/kimina-client: parallelized proof checking with LRU caching (~10× throughput). Provers usable as tool behind MCP.
### Suitability for autoformalization to IR
High when paired with a frontier autoformalizer. Kimina-Autoformalizer-7B trained for NL→Lean 4 statement translation. DeepSeek-Prover-V2 integrates informal+formal reasoning. Goedel-Prover-V2-32B: 88.1% pass@32 miniF2F standard, 90.4% self-correction; solves 86 PutnamBench problems at pass@184 (vs DeepSeek-Prover-V2-671B's 47 at pass@1024).
### Formal verification potential
Maximal — Lean kernel is the verifier. Contradiction between two clinical rules encoded as goal `r1 ∧ r2 → False` and proved/disproved.
### Tooling/Ecosystem maturity
Mathlib4 ~1.5M+ lines, active Lean community. miniF2F, ProofNet, PutnamBench, ProverBench (325 problems, DeepSeek). DeepSeek-Prover-V2-671B: 88.9% miniF2F pass-rate. Kimina-Prover-72B: 80.7% miniF2F pass@8192. AlphaProof: IMO 2024 P1/P2/P6 (28/42 points = silver-medal standard).
### Japan-specific considerations
No Japanese-specific Lean infrastructure; Mathlib4 and tooling are English. Lean community Japan exists informally. Japanese clinical ontology terms must normalize to ASCII/UTF-8 Lean identifiers.
### Interoperability (with Categories 1–7)
LeanDojo extracts premise data for retrieval (§7); Kimina-Lean-Server exposable as MCP tool (§5); whole-proof outputs verifiable by SMT cross-check (Lean Smt tactic).
### Limitations/Known issues
Frontier provers compute-heavy (DeepSeek-V2 671B); benchmark contamination (Kimina identified ≥5 wrongly-formalized problems in miniF2F-test). Mathlib4 lacks clinical-domain libraries — must be built. Leanstral: reports only FLTEval (real-repo) numbers, no miniF2F; Mistral marketing-framed.
### Training data proxy
Mathlib4 corpus + synthetic theorem decomposition (DeepSeek-V3 prompted); Numina Math 1.5; Lean-Workbook-Plus; Goedel-V1 expert-iteration over 1.64M auto-formalized statements across 8 rounds generating 29.7K Lean Workbook proofs.

## 4. Constrained Decoding Against IR Grammar and Schema
### Purpose
Guarantee LLM output parses into target IR (JSON Schema, Pydantic class, CFG, regex) on every sample, eliminating syntax-level retries.
### Maintainer/Standards body
xgrammar (Yixin Dong, Charlie F. Ruan, Yaxing Cai, Ruihang Lai, Ziyi Xu, Yilong Zhao, Tianqi Chen; CMU+NVIDIA+SJTU+UC Berkeley; MLSys 2025; default backend for vLLM, SGLang, TensorRT-LLM, MLC-LLM); Outlines (Rémi Louf, .txt/dottxt-ai); LMQL (ETH Zurich); Guidance (Microsoft/guidance-ai); LM Format Enforcer; jsonformer; BAML (Boundary); Pydantic AI; OpenAI Structured Outputs/Strict JSON Mode; Anthropic Tool Use schemas.
### Conceptual model
Per decoding step, build a token mask from a pushdown automaton/FSM derived from schema/grammar; zero out invalid tokens before sampling. xgrammar splits vocabulary into context-independent (precomputed) and context-dependent (runtime) tokens; up to 100× speedup, near-zero overhead on JSON.
### Expressiveness/Semantics
Regex < JSON Schema < CFG < dependent-type grammars. xgrammar supports general CFG. Cannot enforce semantic constraints (e.g., "drug code must exist in JMDC dictionary") — need post-validation or constrained sampling against a trie.
### Composability/Modularity
Plugs into inference engines (vLLM, SGLang, TensorRT-LLM, MLC-LLM). Co-design APIs for rollback (speculative decoding) and jump-forward decoding. Stackable with KV caching.
### Suitability for autoformalization to IR
Critical. Forces IR JSON to match Pydantic schema; with semantic post-check (NLI, §8) gives surface-level idempotency. JSONSchemaBench (Guidance AI, 10K real schemas) benchmarks Guidance, Outlines, Llamacpp, XGrammar, OpenAI, Gemini frameworks.
### Formal verification potential
Structural-only; not semantic. Schema conformance ≠ logical consistency.
### Tooling/Ecosystem maturity
High. xgrammar is the de-facto open-source default; integrated in major engines. Closed APIs (OpenAI strict JSON, Anthropic tool-use, Gemini structured output) cover same use case.
### Japan-specific considerations
UTF-8/Japanese identifiers fully supported in schemas; CJK tokens need care for tokenizer-aware FSM compilation (xgrammar handles BPE merges). Japanese ICD-10, JLAC10 lab code dictionaries enforceable as enumerations.
### Interoperability (with Categories 1–7)
Outputs feed Lean (§3) / SMT directly; consumed by program-aided executors (§9); compatible with all tool-calling agents (§5).
### Limitations/Known issues
Quality degrades if model's top tokens are all masked (forced low-probability fallback). Complex regex/nested CFGs cost 2–5× without optimization. Closed APIs have less schema coverage than open libraries.
### Training data proxy
N/A — inference-time technique; no training data. Some recent work (e.g., AdapTrack) trains models to anticipate constraints, avoiding output distortion.

## 5. Tool-Calling Agents Connected to Lean, SMT Solvers, and Terminology Services
### Purpose
Let LLMs invoke Lean (proof checker), Z3/CVC5 (SMT), UMLS/MeSH-J/JMDC/MEDIS-DC (terminology), FHIR servers, Python sandboxes as external tools during formalization and verification.
### Maintainer/Standards body
Anthropic Tool Use; OpenAI Function Calling/Agents SDK; Google Gemini Function Calling; Model Context Protocol (MCP — created at Anthropic by David Soria Parra and Justin Spahr-Summers, donated by Anthropic to the Linux Foundation Agentic AI Foundation on Dec 9 2025, co-founded with Block and OpenAI; 10,000+ published MCP servers, 97M+ monthly SDK downloads); LangGraph (LangChain); LlamaIndex; AutoGen (Microsoft); CrewAI; Claude Code; OpenAI Codex.
### Conceptual model
Agent loop: LLM emits tool call → host executes → result returned in context → LLM iterates. MCP standardizes the JSON-RPC layer with three primitives (tools/resources/prompts) and STDIO/StreamableHTTP transports. "Code execution with MCP" pattern (Anthropic engineering blog, Nov 2025) loads tools as code-on-filesystem to reduce token usage.
### Expressiveness/Semantics
Turing-complete via code-execution tools; deterministic via Lean/Z3; ontology lookups via REST or MCP wrapping of UMLS/BioPortal/OBO Foundry/HL7 terminology services.
### Composability/Modularity
High. MCP servers reusable across all major hosts (Claude, ChatGPT, Cursor, Gemini, Microsoft Copilot, Visual Studio Code). Tool Search + Programmatic Tool Calling in Anthropic API optimize many-tool deployments.
### Suitability for autoformalization to IR
Core architecture for the proposed CDS: LLM → call `lookup_icd10_jp(code)` → call `lean_check(proposition)` → call `z3_check(formula)` → emit IR.
### Formal verification potential
Verification delegated to the Lean/SMT tool; agent orchestrates. Goedel-Prover-V2 uses verifier-guided self-correction with 2 rounds of Lean compiler feedback (40K total tokens).
### Tooling/Ecosystem maturity
Very high. SDKs in Python/TypeScript/Java/C#/Ruby/Elixir. Pre-built reference MCP servers (Google Drive, Slack, GitHub, Postgres, Puppeteer); enterprise infra on AWS/Cloudflare/GCP/Azure.
### Japan-specific considerations
MCP wrappers around Japanese terminology services (MEDIS-DC standard master, JLAC10, ICD-10 国内ベース, YJ drug code, HOT code) must be custom-built. Minds guideline corpora exposable via MCP resources.
### Interoperability (with Categories 1–7)
Connects everything. Frontier models (§1), domain models (§2), provers (§3), constrained decoders (§4) all sit behind or in front of MCP.
### Limitations/Known issues
Context bloat with many tools (mitigated by Tool Search and code-execution pattern). Prompt-injection risk via tool outputs (Opus 4.5 hardened but not eliminated). Latency for serial tool chains.
### Training data proxy
Models trained with tool-use trajectories (Anthropic, OpenAI, Google internal mixtures); no public dataset.

## 6. Multi-Run Self-Consistency, Semantic Convergence, and Idempotency Checks
### Purpose
Quantify and enforce that repeated formalization of the same guideline statement yields semantically equivalent IR — the CDS success criterion.
### Maintainer/Standards body
Self-consistency: Xuezhi Wang, Jason Wei, Dale Schuurmans, Quoc Le, Ed Chi, Sharan Narang, Aakanksha Chowdhery, Denny Zhou, "Self-Consistency Improves Chain of Thought Reasoning in Language Models," ICLR 2023 (arXiv:2203.11171) — +17.9% GSM8K, +11.0% SVAMP, +12.2% AQuA. Autoformalization self-consistency: Zenan Li et al. (Nanjing/Microsoft), "Autoformalize Mathematical Statements by Symbolic Equivalence and Semantic Consistency," arXiv:2410.20936 (NeurIPS 2024). ReForm (arXiv:2510.24592) — Reflective Autoformalization with Prospective Bounded Sequence Optimization, +22.6 pp over strongest baselines.
### Conceptual model
Sample k candidates → cluster by (a) symbolic equivalence via ATP (Isabelle Sledgehammer/Lean `decide`), (b) semantic similarity of round-trip back-translation (informalize, embed, cosine), (c) majority vote on canonical-form hash. Idempotency = f(f(x)) ≡ f(x); ASR = autoformalization self-consistency rate.
### Expressiveness/Semantics
Symbolic equivalence captures logical homogeneity; semantic consistency captures meaning preservation. Combined score addresses pass@1 vs pass@k gap (19.5–26.5% on MATH/miniF2F per Li et al.).
### Composability/Modularity
Drop-in scoring layer over any generator. Compatible with constrained decoding (§4), retrieval (§7), critique (§8).
### Suitability for autoformalization to IR
Direct fit. Cluster k=8–32 IR candidates per guideline; promote majority cluster as canonical IR; flag low-consistency cases for human review.
### Formal verification potential
Symbolic-equivalence step uses an ATP, providing partial verification. Semantic-consistency step is heuristic (embedding-based).
### Tooling/Ecosystem maturity
Self-consistency is standard. Symbolic-equivalence pipelines (Isa-AutoFormal repo) exist; few production-grade libraries. Generalized self-consistency for open-ended generation (Jain et al. HF Papers 2307.06857).
### Japan-specific considerations
Back-translation requires Japanese-capable informalizer. Embedding similarity should use multilingual-E5/SimCSE-Ja/Ruri-large.
### Interoperability (with Categories 1–7)
Sits above all generator approaches; consumes outputs from §1–§3; feeds §8 (judge).
### Limitations/Known issues
Failure modes when all k candidates are wrong but mutually consistent (mode collapse). ATP timeouts on complex statements. Embedding metrics insensitive to logical operators (¬, ∀).
### Training data proxy
No training; selection-time technique. Some methods (ReForm) train the validation step jointly with generation via RL on Lean compiler reward.

## 7. Retrieval-Augmented Autoformalization with Premise/Theorem Retrieval
### Purpose
Retrieve relevant Mathlib4/ontology/past-formalization premises at generation time to improve correctness and consistency.
### Maintainer/Standards body
LeanDojo + ReProver (Yang et al., Caltech/NVIDIA, NeurIPS 2023 Datasets & Benchmarks oral); Magnushammer (Isabelle); Thor (DeepMind); COPRA; Lean Copilot (Song et al.); LLMStep; FormalAlign (Jianqiao Lu et al., ICLR 2025); MS-RAG + Auto-SEF ("Consistent Autoformalization for Constructing Mathematical Libraries," arXiv:2410.04194).
### Conceptual model
Dense retriever (ByT5/SBERT encoder) indexes premises with state-aware tokens. Per tactic step, retrieve top-k accessible premises, concatenate with goal state, feed encoder-decoder generator. LeanDojo Benchmark: 98,734 theorems with premise annotations; ReProver outperforms BM25 and GPT-4 zero-shot on the novel-premises split.
### Expressiveness/Semantics
Premise = any named Mathlib definition/lemma. In CDS context, premise = canonical clinical term, ICD/SNOMED-CT/SNOMED-Japan code, prior formalized rule, ontology axiom.
### Composability/Modularity
Retriever and generator independently swappable. Indexes recomputable when ontology updates.
### Suitability for autoformalization to IR
Highly suitable. Use prior formalizations as exemplars (few-shot) and canonical ontology entries as retrieval targets to enforce term consistency across runs (drives idempotency).
### Formal verification potential
Indirect — improves provability but does not itself verify.
### Tooling/Ecosystem maturity
LeanDojo (Lean 4 main branch), ReProver, Lean Copilot integrated into VS Code Lean extension. FormalAlign code/data public — outperforms GPT-4 by 11.58% AS on FormL4-Basic (99.21% vs 88.91%). MS-RAG +5.47–33.58% syntactic correctness improvement.
### Japan-specific considerations
Build dense index over MEDIS-DC, JLAC10, Minds CQ database, and Japanese society guidelines. SimCSE-Ja or BGE-M3 multilingual embeddings recommended.
### Interoperability (with Categories 1–7)
Retriever exposed as MCP tool; results feed constrained-decoded generator; clusters with §6 idempotency.
### Limitations/Known issues
Hard-negative selection critical (LeanDojo leverages program analysis). Retrieval can amplify majority biases. Premises in non-English may be under-represented.
### Training data proxy
LeanDojo Benchmark; MathLibForm; FormL4. For Japanese: must be assembled from Minds + society guidelines + national master files.

## 8. Independent Critique and Adjudication Passes for Conflict Explanations
### Purpose
Detect semantic errors in generated IR and resolve inter-guideline contradictions via independent judge models and multi-agent debate.
### Maintainer/Standards body
Constitutional AI (Anthropic); Reflexion (Shinn et al.); Self-Refine (Madaan et al.); Society of Minds/multi-agent debate — Yilun Du, Shuang Li, Antonio Torralba, Joshua B. Tenenbaum, Igor Mordatch, "Improving Factuality and Reasoning in Language Models through Multiagent Debate," ICML 2024 (PMLR 235:11733–11763, arXiv:2305.14325), 3 agents × 2 rounds; ChatEval (Chan et al.); PRD (Li et al.); D3 framework (Debate, Deliberate, Decide); Agent-as-a-Judge (arXiv:2508.02994); Multi-Agent Judge with HAJailBench (Lin, Shen, Yang, Liu, Zhao, Zeng; arXiv:2511.06396); NLI models (DeBERTa-v3-mnli, JaNLI for Japanese).
### Conceptual model
Critic-defender-judge or advocate-juror architectures. Critic produces structured critique under a rubric; defender rebuts; judge/jury of personas adjudicates. Bounded N rounds (typically 2–3) capture most gain; HAJailBench ablation peaks at three rounds on Qwen3-14B. NLI classifier checks entailment/contradiction between IR rule and source NL guideline.
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
Cross-lingual critique: judge prompts must include Japanese source for grounding; Japanese-specialized models can introduce safety regressions (JMedEthicBench: medical fine-tunes weaken safety alignment, safety scores declining significantly across conversation turns — median 9.5 to 5.5, p < 0.001).
### Interoperability (with Categories 1–7)
Consumes outputs of §1–§7; produces structured contradiction reports as IR annotations.
### Limitations/Known issues
Judge quality is ceiling; weaker judge can miss subtle errors. Positional, verbosity, self-enhancement biases. Cost compounds with rounds.
### Training data proxy
HAJailBench (11,100 labeled jailbreak interactions); MT-Bench; AlpacaEval; Japanese benchmarks scarce for clinical critique.

## 9. Program-Aided Language Models for Executable Intermediate Code
### Purpose
Generate executable Python/SQL/CQL (HL7 Clinical Quality Language) as the IR; offload arithmetic, eligibility logic, rule evaluation to a deterministic interpreter.
### Maintainer/Standards body
PAL (Luyu Gao, Aman Madaan, Shuyan Zhou, Uri Alon, Pengfei Liu, Yiming Yang, Jamie Callan, Graham Neubig; CMU; ICML 2023, arXiv:2211.10435); Program of Thoughts (Chen et al.); OpenAI Code Interpreter; Anthropic code execution tool; e2b sandboxes; Pyodide; RestrictedPython; HL7 CQL (clinical quality language) for guidelines.
### Conceptual model
LLM emits a program; Python interpreter (or CQL engine) executes; final answer = execution output. PAL with Codex: 72.0% top-1 on GSM8K, "surpassing PaLM-540B which uses chain-of-thought by absolute 15% top-1" (Gao et al., ICML 2023).
### Expressiveness/Semantics
Python is Turing-complete; CQL designed for clinical decision logic (CMS eCQM standard). SQL/FHIRPath for EHR queries.
### Composability/Modularity
Interpreters as tools (§5); sandboxed; pure functions enable replay.
### Suitability for autoformalization to IR
Strong alternative to Lean for procedural rules ("if HbA1c > 7.0 and eGFR < 30, contraindicate metformin") — emit CQL or Python; reserve Lean for cross-guideline contradiction proofs. Hybrid: PAL for evaluation, Lean for verification.
### Formal verification potential
Lower than Lean (no kernel-level guarantee) but supports symbolic execution/property-based testing (Hypothesis). CQL has formal grammar.
### Tooling/Ecosystem maturity
Very mature for Python; CQL ecosystem (cqf-tooling, CQL-to-ELM compiler, OpenCDS) is HL7-standard but smaller community.
### Japan-specific considerations
CQL supports Japanese identifiers; mapping CQL value sets to MEDIS-DC/JLAC10 is manual. Pyodide can run Japanese tokenizer libraries (fugashi, sudachipy) in sandbox.
### Interoperability (with Categories 1–7)
Pairs with constrained decoding (CQL grammar, §4) and tool-calling (§5). Outputs verifiable by Lean-encoded shadow specs (§3).
### Limitations/Known issues
Execution-output equivalence ≠ logical equivalence; sandbox escape risks; numeric edge cases. PAL fails when problem decomposition itself is wrong.
### Training data proxy
GitHub Python; CQL examples scarce. Models inherit from base coding training.

## 10. Verifier-Guided Decoding and Proof-Repair Loops
### Purpose
Use a verifier (Lean compiler, SMT solver, PRM) to score, prune, or repair candidate generations during decoding or in an outer loop.
### Maintainer/Standards body
Baldur (Emily First, Markus Rabe, Talia Ringer, Yuriy Brun; UMass/Google; ESEC/FSE 2023; arXiv:2303.04910) — whole-proof + repair on Isabelle/HOL; +8.7% over Thor; combined 65.7%. "Let's Verify Step by Step" (Hunter Lightman, Vineet Kosaraju, Yura Burda, Harri Edwards, Bowen Baker, Teddy Lee, Jan Leike, John Schulman, Ilya Sutskever, Karl Cobbe; OpenAI, arXiv:2305.20050) — process reward model + PRM800K (800K human step labels); 78% MATH subset. HyperTree Proof Search (Guillaume Lample et al., Meta, NeurIPS 2022; arXiv:2205.11491) — AlphaZero-style MCTS; Lean-based miniF2F-curriculum 31%→42%; 41% pass@64 miniF2F-test. InternLM2.5-StepProver (Shanghai AI Lab, arXiv:2410.15700, Oct 2024) — critic-guided tree search; 59.2% miniF2F pass@64; ProofNet BF 22.3% + CG 23.9% → 27.0% combined. Self-Debug (Xinyun Chen, Maxwell Lin, Nathanael Schärli, Denny Zhou; Google/UC Berkeley, arXiv:2304.05128, Apr 2023) — execution-feedback rubber-duck loop, up to +12% on MBPP. Goedel-Prover-V2-32B (Princeton/Tsinghua, Aug 2025) — verifier-guided self-correction, 2 rounds Lean compiler feedback, 88.1%→90.4% miniF2F.
### Conceptual model
Two modes: (a) in-decoder verifier guides beam search/MCTS (HTPS, InternLM2.5-StepProver, PRMs); (b) outer loop — generate proof, run Lean, feed error message back into prompt, regenerate (Baldur, Goedel-V2 self-correction, Chen Self-Debug).
### Expressiveness/Semantics
Verifier signal = Boolean (compiles/proves), scalar (PRM step score), or text (error message). Loops typically bounded (2 rounds in Goedel-V2, capped error-fix turns in Kimina).
### Composability/Modularity
Generator and verifier independently swappable; verifier wrappable as MCP tool.
### Suitability for autoformalization to IR
Essential for production CDS. Lean compiler is a perfect oracle for syntactic correctness; SMT for arithmetic decidability; PRM for soft step-level scoring of IR construction.
### Formal verification potential
Maximal when verifier is Lean kernel.
### Tooling/Ecosystem maturity
Kimina-Lean-Server (parallel, ~10× speedup, LRU cache); PRM800K open; vLLM/SGLang support beam search; Goedel-V2 self-correction pipeline open-source.
### Japan-specific considerations
Verifier feedback is in Lean (English error messages); not a language issue. Building a Japanese-clinical Mathlib lemma library is the prerequisite blocker.
### Interoperability (with Categories 1–7)
Closes the loop with §3 (Lean), §4 (constrained decoding), §6 (consistency over k repaired samples), §8 (judge can request repair).
### Limitations/Known issues
Reward hacking on PRMs; loop divergence; latency compounds with rounds. Self-debug effective up to ~3 rounds, then plateaus.
### Training data proxy
PRM800K; Lean-Workbook-Plus; Baldur's PISA (6,336 Isabelle theorems); Goedel-V2's scaffolded synthetic data.

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
JMedBench (20 datasets, NAIST) + JMedLLM-v1-7B + JMedLoRA (UTokyo) are canonical benchmarks. JaCWIR, JSQuAD, JMMLU-medical, IgakuQA for held-out evaluation. ELMTEX clinical-extraction study (60K PubMed Central summaries, 15 categories): "LoRA improved metrics by 10–20 points over the base model, while QLoRA improved by 8–14 points—only 2–4 points below LoRA." Catastrophic forgetting (Meditron→Japanese) is real.
### Interoperability (with Categories 1–7)
Adapter sits on §2 base; output consumed by §4 constrained decoder; gated by §6 idempotency metric to justify deployment.
### Limitations/Known issues
Small datasets cause overfit; instruction-tuning can degrade 1-shot performance (JMedLoRA finding). Catastrophic forgetting under additional pretraining with scarce data. Adapter for closed frontier models (§1) is unavailable — only open bases (§2, Llama, Qwen, Mistral, Gemma).
### Training data proxy
JMedLoRA: Japanese medical instruction pairs from licensing-exam material on OpenCALM-7B/Llama2-70B-chat-hf. JMedLLM-v1-7B: 7B Qwen2 base + bilingual medical MFPT/MPEFT. Synthetic data from frontier models acceptable for adapter training but must be checked for hallucination.

## 12. World Models for Patient State and Trajectory: JEPA, Latent-Dynamics, Generative-Trajectory Backbones, and Clinical Instantiations (Clin-JEPA, SMB-Structure, EHRWorld, ETHOS, Foresight)
### Purpose
Predictive representations of clinical state — patient trajectories, disease progression, latent dynamics under candidate treatment policies — as **upstream encoders** for rule-evaluation, **forward simulators** of guideline applications and counterfactuals, or **representation learners** over guideline prose for §7 retrieval. Orthogonal to the §1–§11 autoformalization stack: world models predict missing/future content (latent, token, or pixel/event space) rather than emit IR; tools for representation and forecasting, not Lean/SMT/JSON-schema IR. Included because (a) any CDS conditioning on patient state must get that estimate somewhere, (b) guideline-impact and counterfactual-treatment evaluation increasingly use world-model rollouts as a trial-simulation substitute, (c) clinical world-model literature shifted in early 2026 from "design pattern" to "released artifacts" — Clin-JEPA, SMB-Structure, EHRWorld all appeared within four months.
### Maintainer/Standards body
**General-purpose world models.** Three architectural families dominate the broader literature as of May 2026:

- *Joint-Embedding Predictive Architectures (JEPA) — Meta FAIR / LeCun lineage.* I-JEPA (Mahmoud Assran, Quentin Duval, Ishan Misra, Piotr Bojanowski, Pascal Vincent, Michael Rabbat, Yann LeCun, Nicolas Ballas; CVPR 2023; arXiv:2301.08243); V-JEPA (Adrien Bardes, Quentin Garrido, Jean Ponce, Xinlei Chen, Michael Rabbat, Yann LeCun, Mahmoud Assran, Nicolas Ballas; Meta, Feb 2024); V-JEPA 2 / V-JEPA 2-AC (Meta, 2025 — 1.2B params, internet-scale video pretraining + action-conditioned post-training for robotic control; ViT-L / ViT-H / ViT-g checkpoints, non-commercial research license; zero-shot robot planning on ~62 hours of robot data after generic video pretraining); H-JEPA hierarchical world-model proposal (LeCun, "A Path Towards Autonomous Machine Intelligence," OpenReview, June 2022); variational JEPA extensions positioning JEPA as a probabilistic world model (arXiv:2601.14354).
- *Latent-dynamics / RSSM-class world models — DeepMind lineage.* World Models foundational paper (David Ha, Jürgen Schmidhuber; NeurIPS 2018; arXiv:1803.10122); Dreamer / DreamerV2 / DreamerV3 (Danijar Hafner et al., Google DeepMind; DreamerV3 in Nature, January 2025, "Mastering diverse control tasks through world models" — single hyperparameter set across 150+ tasks); MuZero (Schrittwieser et al., DeepMind, Nature 2020); HyperTree Proof Search lineage shares the MCTS-over-latent-model design.
- *Observation-level generative world models — broad maintainer set.* Genie / Genie 2 / Genie 3 (Google DeepMind; Genie 3 released August 2025 as the first real-time interactive general-purpose world model rendering navigable 3D worlds at 24 fps, currently a limited research preview); Sora 2 (OpenAI); Veo 3 (Google); Cosmos (NVIDIA) — video-diffusion and autoregressive-video backbones increasingly framed as world models because their rollouts encode physical and behavioral dynamics implicitly.

Survey coverage: "Understanding World or Predicting Future? A Comprehensive Survey of World Models" (Tsinghua FIB Lab; ACM Computing Surveys 2025; arXiv:2411.14499) and "A Comprehensive Survey on World Models for Embodied AI" (arXiv:2510.16732) provide the now-standard taxonomy used below.

**Clinical world models / patient-trajectory foundation models.** Two waves:

- *First-wave tokenized-EHR generative predictors (sometimes retroactively called "patient language models").* ETHOS (Pawel Renc, Yugang Jia, Anthony E. Samir et al., npj Digital Medicine 2024 — Enhanced Transformer for Health Outcome Simulation, zero-shot trajectory prediction on MIMIC-IV); Foresight (Zeljko Kraljevic, Dan Bean, Anthony Shek et al., King's College London, Lancet Digital Health 2024 — generative pretrained transformer over SNOMED-coded EHR); BEHRT (Yikuan Li et al., Scientific Reports 2020); Med-BERT (Laila Rasmy et al., npj Digital Medicine 2021); CEHR-BERT, CLMBR, MOTOR. These autoregressively predict event tokens (diagnoses, medications, labs, time deltas) and are observation-level generative WMs in the survey taxonomy.
- *Second-wave clinical world models (2026).* **Clin-JEPA** (Yixuan Yang, Mehak Arora, Ryan Zhang, Baraa Abed, Junseob Kim, Tilendra Choudhary, Md Hassanuzzaman, Kevin Zhu, Ayman Ali, Chengkun Yang, Alasdair Edward Gent, Victor Moas, Rishikesan Kamaleswaran; arXiv:2605.10840, submitted May 11 2026; v2 May 12 2026; CC BY 4.0; code at `github.com/YeungYathin/Clin-JEPA`). **SMB-Structure** ("The Patient is not a Moving Document: A World Model Training Paradigm for Longitudinal EHR" — Irsyad Adam, Zekai Chen, David Laprade, Shaun Porwal, David Laub, Erik Reinertsen, Arda Pekis, Kevin Brown; arXiv:2601.22128, Jan 29 2026; 1.7B-param weights on Hugging Face at `standardmodelbio/SMB-v1-1.7B-Structure`). **EHRWorld** ("EHRWorld: A Patient-Centric Medical World Model for Long-Horizon Clinical Trajectories" — Linjie Mu, Zhongzhen Huang, Yannian Gu, Shengqian Qin, Shaoting Zhang, Xiaofan Zhang; arXiv:2602.03569, Feb 3 2026; released alongside EHRWorld-110K, a 110K-record longitudinal dataset).
### Conceptual model
The 2025 surveys converge on a two-axis taxonomy: prediction target (observation-level vs. latent-embedding) × generativity (with/without explicit decoder back to observations). Three families matter clinically:

**(a) Observation-level generative world models.** Predict the next *observation* directly — frames, tokens, mesh updates. Autoregressive (Sora 2, Cosmos, Genie 3 autoregressive variants), NeRF/3D-Gaussian-splatting, video diffusion. Cost: must model fine-grained surface detail, risk of hallucinated detail. Benefit: human-interpretable, directly inspectable. Clinical analogues = *tokenized-EHR generative predictors* (ETHOS, Foresight, BEHRT, Med-BERT, EHRWorld): events tokenized (ICD/SNOMED/RxNorm/lab codes + time deltas), transformer autoregressively emits future events. EHRWorld frames itself as a "patient-centric medical world model," emphasizing a "causal sequential paradigm" for long-horizon trajectory simulation.

**(b) Latent-dynamics world models with generative reconstruction (RSSM-class).** Learn recurrent latent state $s_t$, transition $s_{t+1} = T(s_t, a_t)$, observation decoder, reward predictor; train agents by rolling out *imagined* trajectories through $T$ and updating policies on them. DreamerV3 canonical; MuZero adds MCTS planning over the learned model. Decoder grounds latents to observations during training, typically discarded at planning time. Clinical instantiation limited — no production clinical Dreamer — though "digital twin" patient simulators in cardiology and ICU research occupy this niche.

**(c) Joint-Embedding Predictive Architectures (JEPA) — latent prediction without reconstruction.** Given context $x$ and target $y$, learn encoder $f_\theta$ and predictor $g_\phi$ such that $g_\phi(f_\theta(x))$ matches $f_\theta(y)$ in latent space (typically EMA-tracked target encoder to prevent representation collapse). No decoder; predictor reconstructs the target *embedding*, not pixels/tokens. Survey distinction: "RSSM relies on generative reconstruction of observations to model latent dynamics, whereas JEPA employs self-supervised predictive coding in embedding spaces directly forecasting future state representations without decoding to raw sensory inputs." Pretext variants: masked image regions (I-JEPA), masked spatio-temporal video tubes (V-JEPA), action-conditioned future prediction (V-JEPA 2/2-AC).

**Clinical instantiations of family (c).**
- **Clin-JEPA** (Yang et al., arXiv:2605.10840). JEPA on longitudinal EHR by co-training a *Qwen3-8B-based encoder* and a 92M-param latent trajectory predictor. Core diagnosis: prior JEPAs either discard the predictor after pretraining (I-JEPA, V-JEPA) or train it on a frozen encoder (V-JEPA 2-AC), leaving the encoder unaware of the rollout signal the retained predictor uses at inference. Naïve co-training collapses; Clin-JEPA's five-phase pretraining curriculum (predictor warmup → joint refinement → EMA target alignment → hard sync → predictor finalization) stabilizes co-training by phase, addressing representation collapse and online/target drift. Result: a *single backbone* for both autoregressive latent rollout and multi-task downstream risk prediction without per-task fine-tuning. Trained on MIMIC-IV ICU.
- **SMB-Structure** (Adam et al., arXiv:2601.22128). 1.7B-param hybrid: a JEPA prediction objective *grounded* by an SFT next-token-prediction objective on the same structured EHR sequence. SFT forces token-space reconstruction of future patient states; the JEPA head simultaneously predicts those futures in latent space from the initial patient representation alone, "forcing trajectory dynamics to be encoded before the next state is observed." Framing — "the patient is not a moving document to summarize but a dynamical system to simulate" — contrasts world-model training against next-token-only clinical LLM pretraining. Validated on Memorial Sloan Kettering oncology (23,319 patients, 323,000+ patient-years) and INSPECT pulmonary-embolism cohort (19,402 patients).

Family (a): interpretable token-level trajectories at cost of surface-detail hallucination; (b): planning-ready latent dynamics at cost of decoder modeling; (c): clean state embeddings without a decoder at cost of a separate evaluation interface. Clin-JEPA, SMB-Structure, EHRWorld span (c), (c)+(a) hybrid, and (a) respectively for clinical data.
### Expressiveness/Semantics
None natively expresses the LTL/MTL/deontic/arithmetic constraints clinical-rule verification needs; they predict, not reason in formal calculi. Family (a) predictors output event tokens from ICD/SNOMED/RxNorm/MEDIS-DC vocabularies (terminology layer exposed downstream) — but events are predicted, not entailed by a logical rule. Family (b) supports counterfactual rollouts "$\hat{s}_{t+k}$ given action $a_t$" but the counterfactual is statistical, *not* a causal estimate unless training was interventional or the model is a structural causal model (g-formula/target-trial emulation/DAG-conditioned). Family (c) JEPA encoders output dense vectors with no symbolic structure. Clin-JEPA reports two semantic-geometry findings: latent ℓ₁ rollout drift uniquely *converges* (−15.7%) over a 48-hour horizon while baselines/ablations diverge +3% to +4951%; learned latent geometry is clinically discriminative — deteriorating-patient cohorts displace 4.83× further in latent space than stable patients, vs. ≤2.62× for baseline encoders.
### Composability/Modularity
JEPA encoders are clean upstream modules: embeddings consumable by any classifier, retriever, regressor, or LLM-with-adapters. RSSM-class agents are typically end-to-end, but latent state $s_t$ is reusable. Tokenized-EHR generative predictors (ETHOS, Foresight, EHRWorld) are pretrained transformers — fine-tunable, distillable, embeddable, exposable behind MCP. Clin-JEPA's Qwen3-8B backbone means encoder embeddings live in a space LLM-side adapters can condition against directly; SMB-Structure publishes 1.7B-param weights on Hugging Face under research license. V-JEPA 2 publishes ViT-L/ViT-H/ViT-g checkpoints under non-commercial research license; commercial use requires separate Meta agreement.
### Suitability for autoformalization to IR
Low to none **directly**. World models do not emit Lean, SMT, or JSON-schema IR. Indirect CDS roles:
- **Upstream state encoders.** Supply patient-state embeddings (Clin-JEPA, SMB-Structure encoder outputs) or cluster labels to rule-evaluation, so an IR rule conditions on a latent state — e.g., "patient currently in trajectory cluster matching pre-decompensation pattern" → triggers a Lean-encoded surveillance rule from §3.
- **Forward/counterfactual simulators for guideline evaluation.** Roll out under policy $\pi$ = "apply guideline G as written" vs. a counterfactual, compare predicted outcomes. EHRWorld and ETHOS support this via autoregressive token generation; Dreamer-style latent rollouts in principle too. Useful for *guideline-impact estimation*, not proving guideline correctness.
- **Representation learners for guideline prose.** A JEPA-style objective on guideline text (context = surrounding paragraph, target = masked clinical-statement embedding) could improve §7 premise retrieval beyond standard contrastive sentence embeddings; no published clinical-guideline JEPA exists yet.

Best paired with §1 frontier-LLM autoformalizers, not used in their place.
### Formal verification potential
None native. World models are statistical predictors; outputs carry no certificate. Verification routes through §3 (Lean) and SMT. World-model outputs can be the *subject* of verification — e.g., prove a Lean-encoded rule evaluated on a predicted state agrees with the rule on the ground-truth state once observed. This is regression testing of the predictor, not proof of model correctness, but slots into §10 verifier-guided loops as a soft signal.
### Tooling/Ecosystem maturity
Modest for general world models, uneven for clinical; gap vs. §1 LLM ecosystem remains large.
- *General.* Meta FAIR releases I-JEPA, V-JEPA, V-JEPA 2 weights + training code on GitHub (`facebookresearch/jepa`, `facebookresearch/vjepa2`); Hugging Face hosts V-JEPA 2 checkpoints. DreamerV3 reference implementation by Hafner on GitHub. Genie 3 currently a limited DeepMind research preview, not GA. No vLLM/TensorRT-LLM-equivalent serving stack for JEPA-class inference; JEPA training reproducibility is sensitive to EMA target schedules and masking strategies. Surveys (Tsinghua FIB CSUR 2025/arXiv:2411.14499; arXiv:2510.16732) and a curated reading list (`tsinghua-fib-lab/World-Model`) give the overview.
- *Clinical.* ETHOS code public; Foresight ships via the CogStack ecosystem at King's College London. **Clin-JEPA**: code at `github.com/YeungYathin/Clin-JEPA`, 17-page paper, 4 figures, 8 tables, no production-serving recipe yet. **SMB-Structure**: 1.7B weights on Hugging Face (`standardmodelbio/SMB-v1-1.7B-Structure`). **EHRWorld**: paper + EHRWorld-110K dataset release; production-serving status unclear.
### Japan-specific considerations
No Japanese variant of any general world model has been released. None of the three 2026 clinical world models is trained on Japanese cohorts — Clin-JEPA MIMIC-IV ICU only; SMB-Structure MSK + INSPECT (US); EHRWorld on EHRWorld-110K (provenance not Japan-specific). A Japanese clinical world model faces the same PHI/data-licensing constraints as §2: SS-MIX2/MID-NET/NDB/Tokushima 千年カルテ linkage requires institutional access; cross-institution sharing under APPI is restrictive. Choices that *could* port with modest adaptation: image/video JEPA on Japanese medical imaging (X-ray, endoscopy, pathology, ophthalmology), text-modality-free; ETHOS-style tokenized-EHR predictors using MEDIS-DC, JLAC10, YJ drug codes, HOT codes as event vocabulary, tokenizer compatibility checked against the JMedBench harness (see §2). A Japanese Clin-JEPA — swapping the Qwen3-8B encoder for a JMedLLM- or Llama2-70B-chat-hf-style Japanese-capable base, pretraining on SS-MIX2-derived ICU cohorts — is technically straightforward but legally non-trivial.
### Interoperability (with Categories 1–7)
State embeddings consumable by retrieval indices in the broader RAG/IR stack (Categories 6–7 of the overall CDS taxonomy) and by frontier-LLM prompts (as numeric features in text, or — for open-weight bases — adapter-conditioning tokens; Clin-JEPA's Qwen3-8B encoder is convenient for the latter). Trajectory predictors wrappable as MCP tools (§5) for query-time consultation: `tool: predict_trajectory(patient_state, candidate_policy, horizon_hours) → predicted_events_or_latent_state`. Cross-modal alignment with FHIR, SNOMED CT, ICD-10, RxNorm, and Japanese MEDIS-DC/JLAC10 vocabularies is prerequisite for clinical grounding of family-(a) token-level outputs.
### Limitations/Known issues
Family-(a) predictors hallucinate event tokens at the surface level (text-LLM hallucination transposed to clinical events) and inherit EHR coding noise, missingness, selection bias from source datasets (MIMIC-IV, CPRD, Cerner Health Facts, MSK, INSPECT, EHRWorld-110K). Family-(b) latent-dynamics models suffer compounding error over long horizons (DreamerV3 documents this; rollouts beyond ~50 imagined steps degrade rapidly), acute over clinical timescales of months to years. Family-(c) JEPA non-generativity is a feature for representation learning and a liability for regulatory review — the encoder is a black box without a complementary decoder; downstream usage must design around that. Counterfactual rollouts across all families are *not* causal estimates absent design (no g-formula structure, no target-trial emulation) — a well-known pitfall when repurposed for treatment-effect estimation. **Clin-JEPA**: single-cohort (MIMIC-IV ICU only); generalization to non-ICU, non-US, pediatric unevaluated; Qwen3-8B encoder large for clinical deployment vs. 1.7B SMB-Structure; five-phase curriculum reproducible but sensitive to phase-boundary scheduling. **SMB-Structure**: oncology/PE-cohort biases; JEPA+SFT hybrid novel, may not transfer to sparser longitudinal data. **EHRWorld**: public-abstract numerical results qualitative ("significantly outperforms naive LLM-based baselines"); independent reproduction pending. None of the three has CDS regulatory clearance in any jurisdiction.
### Training data proxy
*General.* I-JEPA: ImageNet-1K/22K. V-JEPA: ~2M internet video clips. V-JEPA 2: large-scale internet video + ~62 hours robot data for action-conditioned post-training. DreamerV3: 150+ tasks (Atari, DMControl, ProcGen, Crafter, Minecraft). Genie 3/Sora 2/Veo 3: undisclosed large internet-video corpora.
*Clinical.* ETHOS: MIMIC-IV (~300K ICU+ED stays). Foresight: King's College Hospital + Maudsley NHS Foundation Trust records, SNOMED-coded. BEHRT: CPRD UK primary care. Med-BERT: Cerner Health Facts. **Clin-JEPA**: MIMIC-IV ICU, evaluated with ICareFM EEP and an 8-task binary risk benchmark — mean AUROC 0.851 on ICareFM EEP, 0.883 across the 8 binary tasks (+0.038 and +0.041 vs. baseline average). **SMB-Structure**: MSK oncology (23,319 patients, 323,000+ patient-years) + INSPECT (19,402 PE patients). **EHRWorld**: EHRWorld-110K longitudinal clinical records.
*Japanese clinical world model:* training data would need assembly from SS-MIX2/MID-NET/NDB/千年カルテ extracts under explicit IRB approval and APPI compliance; no such public training corpus exists as of May 2026, and the legal pathway to construct one without per-institution data-use agreements is not established.
