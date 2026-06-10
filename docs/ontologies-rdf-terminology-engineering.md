# Category 3: Ontologies, RDF, and Terminology Engineering

## 1. OWL 2 EL/RL/DL Ontology Profiles

### Purpose
W3C standardized subsets of the OWL 2 Web Ontology Language that trade expressivity for tractable reasoning. Profiles let clinical knowledge engineers choose a fragment whose worst-case complexity matches their workload (large TBox classification, query answering over large ABox, or rule-based inference).

### Maintainer/Standards body
W3C OWL Working Group. Current normative reference: "OWL 2 Web Ontology Language Profiles (Second Edition)," W3C Recommendation, 11 December 2012, edited by Boris Motik, Bernardo Cuenca Grau, Ian Horrocks, Zhe Wu, Achille Fokoue, Carsten Lutz. OWL 2 Direct Semantics and RDF-Based Semantics are normative companion documents.

### Conceptual model
- TBox (terminology: class/property axioms), RBox (role hierarchy/characteristics), ABox (assertions about individuals).
- OWL 2 DL: SROIQ(D) description logic; class expressions with nominals, qualified cardinality, inverse roles, role chains, datatypes.
- OWL 2 EL: based on EL++; supports existential restrictions, intersection, class equivalence, role chains, reflexive roles; no disjunction, no universal restrictions, no negation.
- OWL 2 QL: based on DL-Lite_R; supports inclusion axioms suitable for query rewriting against relational data.
- OWL 2 RL: syntactic restriction implementable by Datalog/rule engines.
- OWL 2 Full: RDF-based semantics, undecidable in general.

### Expressiveness/Semantics
- EL: PTime classification; ideal for large terminologies (SNOMED CT, GO, FMA). Cannot express cardinality, disjunctions, or universal restrictions.
- QL: NLogSpace data complexity; conjunctive query answering by FOL rewriting.
- RL: PTime combined complexity for instance checking via rules.
- DL: N2ExpTime worst-case combined complexity for satisfiability; full expressivity for clinical constraints like disjointness, role chains (`hasPart o partOf -> hasPart`), enumerated value sets.
- Profiles are mutually incomparable: each contains constructors absent from the others.

### Composability/Modularity
- `owl:imports` axiom pulls full ontology graphs; locality-based modules (⊥-/⊤-locality, Cuenca Grau et al.) extract conservative sub-modules.
- Profile membership is syntactic; mixing axioms across profiles drops to OWL 2 DL or Full.
- Annotation axioms are profile-agnostic; punning allows the same IRI to be class and individual under direct semantics.

### Suitability for autoformalization to IR
- OWL 2 EL is the dominant target for autoformalization of large clinical terminologies: classification is fast; axioms are mostly Horn-like, reducing LLM generation ambiguity; idempotency testable by classified-hierarchy hashing.
- Risk: LLMs frequently emit axioms outside the chosen profile (e.g., universal restrictions), requiring ROBOT `reduce`/`relax` and profile validation steps.

### Formal verification potential
- Decidable consistency checking, class satisfiability, subsumption, instance checking, conjunctive query answering (the last decidable in EL/QL/RL; open for full OWL 2 DL).
- Contradiction detection = unsatisfiable class detection or proof of `owl:Nothing` entailment.
- Justification/explanation services in ELK, Pellet, HermiT produce minimal axiom sets for inferences — directly usable for guideline conflict traceability.

### Tooling/Ecosystem maturity
- Reasoners: ELK (EL, parallel; per Kazakov, Krötzsch, Simančík, "The Incredible ELK," Journal of Automated Reasoning 53(1):1-61, 2014, doi:10.1007/s10817-013-9296-3; SNOMED CT can be classified by ELK in as little as 5 seconds on a quad-core computer vs. ~25 seconds for Snorocket and ~10 minutes for CEL/jcel; Protégé Wiki notes "less than 4 seconds on a modern laptop"); HermiT (DL); Pellet/Openllet; FaCT++; Konclude; RDFox (RL/Datalog).
- Editors: Protégé 5.x, WebProtégé.
- Libraries: OWL API (Java), owlready2 (Python), funowl, py-horned-owl.
- Serializations: OWL/XML, RDF/XML, Manchester, Functional, OBO (subset), Turtle.

### Japan-specific considerations
- DBCLS hosts numerous OBO/OWL ontologies via TogoGenome, NBDC RDF portal (60 datasets / 221.1 billion triples as of 2026); BioHackathon 2025 (BH25, Mie/VISON, 14-20 September 2025) produced `on2vec` (Steinberg, Kulmanov, Queralt-Rosinach, Chiba, Ellis et al., BioHackrXiv 2025) for OWL→vector embedding. BodyParts3D (DBCLS, CC BY-SA 2.1 JP, v3.0 with 1,523 body parts keyed by FMA IDs) is distributed in OWL-compatible structures. Life Science Dictionary (LSD) operation transferred to DBCLS on 31 March 2026.

### Interoperability with other methods
- Underpins SKOS (SKOS Core is itself in OWL 2 Full).
- BFO/DOLCE formalized in OWL; ROBOT/ODK/DOSDP serialize to OWL.
- MIREOT trims OWL imports.
- SHACL validates instance data against ontology; SHACL 1.2 Rules adds inference complementary to OWL.
- Category 1: FHIR Terminology operations can use OWL-EL classifiers; openEHR archetypes can reference OWL classes; CQL/FHIRPath bind via terminology services.
- Category 2: ICD-11 published in OWL (WHO ICD-11 Foundation Component); SNOMED CT International distributed in OWL 2 EL; MedDRA not OWL-native; MEDIS masters tabular but mappable to SKOS-on-OWL.

### Limitations/Known issues
- Worst-case reasoning blow-up under DL semantics.
- Open-world assumption clashes with closed clinical data semantics ("no allergy recorded" ≠ "no allergy").
- Unique Name Assumption absent — patient identifiers must be explicitly disjoint.
- Negation is monotonic; defeasible clinical rules cannot be expressed natively.

### Training data proxy
- Very high: tens of thousands of GitHub repositories tagged `owl-ontology`; W3C documents extensively in LLM pretraining; Protégé tutorials on YouTube; Stack Overflow tag `owl` has tens of thousands of questions. Japanese-language documentation moderate (DBCLS/NBDC tutorials, BioHackathon writeups).

## 2. SHACL 1.2 Validation Shapes

### Purpose
Shapes Constraint Language defines a declarative way to describe, validate, and infer over RDF graphs by stating the structural conditions data must satisfy. Used to express data-quality constraints on FHIR-RDF, guideline IR triples, and clinical knowledge graphs.

### Maintainer/Standards body
W3C Data Shapes Working Group. SHACL 1.0 Recommendation: 20 July 2017. SHACL 1.2 First Public Working Drafts published 18 March 2025 (Core and SPARQL Extensions); Working Drafts updated through 16 May 2026 (Core), with SHACL 1.2 Node Expressions FPWD published 8 January 2026. SHACL 1.2 Rules FPWD published 2025; SHACL 1.2 Node Expressions and SHACL 1.2 Profiling also Working Drafts. The Data Shapes Working Group charter extends through December 2026, with SHACL 1.2 expected to reach Recommendation in late 2026 or early 2027. Editors of 1.2 Core: Holger Knublauch, Thomas Bergwinkl, Yousouf Taghzouti, Jesse Wright.

### Conceptual model
- A shapes graph (RDF) declares Node Shapes (`sh:NodeShape`) and Property Shapes (`sh:PropertyShape`).
- Targeting: by class (`sh:targetClass`), by node (`sh:targetNode`), by subject/object of predicate, or by SPARQL.
- Constraint components: `sh:datatype`, `sh:class`, `sh:minCount`, `sh:maxCount`, `sh:pattern`, `sh:node`, `sh:qualifiedValueShape`, `sh:and`/`sh:or`/`sh:not`/`sh:xone`, `sh:sparql`.
- Validation report (`sh:ValidationReport`) is RDF; results have severity (`sh:Violation`, `sh:Warning`, `sh:Info`).
- 1.2 additions: Derived Properties, list-syntax unions of datatypes/classes, constraints on RDF 1.2 reification (triple terms), SHACL Rules with declarative inference, SHACL Node Expressions, SHACL Profiling.

### Expressiveness/Semantics
- SHACL Core: closed-world, per-shape, deterministic validation. Recursion semantics left to implementations.
- SHACL-SPARQL: arbitrary SPARQL queries as constraint components — Turing-complete validation.
- SHACL Rules (1.2 WD): forward-chaining inference distinct from OWL entailment.
- Can express closed-world constraints OWL cannot (e.g., "every Patient resource MUST have exactly one birthDate"), key constraints, and string-pattern validation.

### Composability/Modularity
- Shapes graphs are RDF and compose via `owl:imports` or by union.
- `sh:node` enables shape reuse; `sh:and` and `sh:property` chains compose constraints.
- SHACL 1.2 Profiling (`sh:Profile`) lets implementers declare supported subsets.
- SHACL 1.2 Packaging recommendation: group shapes into `owl:Ontology` instances; link with `rdfs:isDefinedBy` and `owl:imports`.

### Suitability for autoformalization to IR
- Excellent constraint surface for guideline IR: required slots, value-set bindings, cardinality of clinical statements.
- LLM-generated shapes are testable — apply to a corpus of example RDF and inspect violations.
- Idempotency: shape IRIs and target classes must be deterministically minted; otherwise duplicate shapes accumulate.
- Pairs naturally with OWL: OWL for entailment-based reasoning, SHACL for closed-world validation.

### Formal verification potential
- Validation = model checking against a shapes graph.
- SHACL semantics decidable for Core (no recursion); SPARQL extensions inherit SPARQL complexity.
- Cannot natively express transitive constraints across unbounded paths without `sh:path` with `*`/`+` operators.
- Contradiction detection between guidelines: encode each guideline's claims as shapes; conflicts surface as violations on shared data.

### Tooling/Ecosystem maturity
- TopBraid SHACL API (Java, reference), pySHACL (Python), Apache Jena SHACL, RDF4J SHACL, Zazuko `shacl-engine`.
- SHACL Playground, RDFShape, SHACL 1.2 test suite at https://w3c.github.io/data-shapes/data-shapes-test-suite/.
- Editors: TopBraid Composer, Stardog Studio.

### Japan-specific considerations
- DBCLS RDF Portal datasets often ship SHACL constraints for validation; SSBD Ontology (RIKEN, Yamagata, Kyoda, Itoga, Fujisawa, arXiv:2508.02084) explicitly cites SHACL-style validation. Japanese-language SHACL tutorials are sparse but present at LOD Challenge Japan.

### Interoperability with other methods
- Complements OWL profiles: SHACL closed-world validation, OWL open-world entailment.
- Named graphs/TriG: SHACL 1.2 can target named graphs via SPARQL extensions and PROV-O datasets.
- SKOS: SHACL shapes commonly used to validate SKOS thesauri.
- ROBOT does not natively run SHACL but can chain with pyshacl in ODK pipelines.
- Category 1: FHIR Shorthand → SHACL conversions exist (BabelFSH toolkit); FHIR-RDF can be validated by SHACL; openEHR archetypes (ADL) have been mapped to SHACL by community projects.
- Category 2: SS-MIX2 export validation can be expressed as SHACL once converted to RDF.

### Limitations/Known issues
- Recursion semantics underspecified — implementations diverge.
- No standard provenance for individual violation messages until 1.2.
- LLM tendency to over-generate `sh:sparql` constraints reduces portability.
- SHACL Rules (1.2 WD) overlaps with SWRL and OWL — confusion about which inference layer "owns" a derivation.

### Training data proxy
- Moderate-high: ~5,000+ GitHub repos referencing SHACL; pySHACL widely used in life-science FAIR pipelines. Stack Overflow tag `shacl` has hundreds of questions. Japanese: limited but growing via DBCLS Stanza tutorials.

## 3. RDF Named Graphs / TriG for Source-Scoped Guideline Claims

### Purpose
Mechanism to package sets of triples (a graph) under an IRI so that statements can be attributed, dated, versioned, or scoped to a source — essential for tracking which guideline made which claim and managing provenance of LLM-generated assertions.

### Maintainer/Standards body
W3C. RDF 1.1 (25 February 2014) introduced datasets/named graphs normatively. RDF 1.2 Concepts and Abstract Data Model and RDF 1.2 Semantics reached Candidate Recommendation status on 7 April 2026 (call for implementations issued; not expected to advance to Recommendation any earlier than 5 May 2026) and add triple terms (RDF-star), keeping named graphs. RDF 1.2 TriG and N-Quads remain textual serializations for datasets.

### Conceptual model
- An RDF Dataset = one default graph + zero or more named graphs `(IRI, graph)` pairs.
- TriG: Turtle extended with graph statements `<g> { ... }`.
- N-Quads: line-oriented `<s> <p> <o> <g> .`.
- RDF 1.2 triple terms `<< s p o >>` enable annotation of individual triples directly (RDF-star); reifying triple denotes the proposition without asserting it.
- Quad stores (Jena TDB2, Virtuoso, Stardog, GraphDB, Oxigraph) treat the graph component as a first-class index dimension.

### Expressiveness/Semantics
- A named graph is itself an RDF graph; the IRI labeling it is metadata-accessible.
- No standard formal semantics for what a named graph means about its triples — interpretation is application-specific.
- Common convention: the IRI denotes a "claim/state" or "provenance bundle." PROV-O `prov:Bundle` formalizes this.
- SPARQL 1.1+ has `GRAPH`, `FROM`, `FROM NAMED`; SPARQL Update has `INSERT DATA { GRAPH <g> { ... } }`.

### Composability/Modularity
- Datasets compose via union of named graphs; quad stores allow per-graph access control and per-graph TTL.
- Versioning by minting a fresh graph IRI per ontology release; OpenCitations Data Model is a published exemplar.

### Suitability for autoformalization to IR
- Strong: scoping each guideline's IR to its own named graph yields per-source provenance "for free," enables straightforward diff between guideline versions (set difference of triples), and supports per-guideline override semantics.
- Idempotency: with stable graph IRIs (deterministic from guideline DOI + section + chunk) re-runs of an LLM produce overwriteable graphs.
- Per-graph SHACL validation supported in 1.2 via SPARQL extensions.

### Formal verification potential
- Contradiction detection across guidelines is a query problem: `SELECT ?g1 ?g2 WHERE { GRAPH ?g1 { ?x :recommends ?A } . GRAPH ?g2 { ?x :recommends ?B } . FILTER(?A != ?B) }`.
- Combined with reasoning: a quad-aware reasoner (e.g., RDFox with named graphs) can detect cross-graph inconsistencies.
- RDF-star reification supports confidence/uncertainty annotations on individual claims for downstream non-monotonic reasoning.

### Tooling/Ecosystem maturity
- Quad stores: Apache Jena Fuseki/TDB2, Virtuoso, Stardog, GraphDB, Oxigraph (Rust), Blazegraph, RDFox.
- Parsers: rdflib (Python), Apache Jena, N3.js, Comunica.
- SPARQL endpoints with named-graph support are ubiquitous.

### Japan-specific considerations
- NBDC RDF Portal (60 datasets, 221.1 billion triples as of 2026) exposes per-dataset SPARQL endpoints, each effectively a named-graph-scoped store. TogoVar (DBCLS, Mitsuhashi et al., Hum Genome Var 2022, doi:10.1038/s41439-022-00222-9) integrates Japanese variation datasets per named graph (JGA-WGS, ToMMo 8.3KJPN, GEM-J WGA). PDBj RDF and the DBCLS Med2RDF effort routinely use TriG/N-Quads.

### Interoperability with other methods
- Carries PROV-O annotations to record `prov:wasGeneratedBy` (LLM run), `prov:wasDerivedFrom` (guideline PDF), `prov:generatedAtTime`.
- SHACL 1.2 SPARQL Extensions can target graphs.
- OWL: `owl:imports` is graph-level; named graphs let an ontology be served from multiple URIs without breaking imports.
- Category 1: FHIR Provenance resource (R5) is conceptually parallel; openEHR's COMPOSITION-level versioning maps to named-graph snapshots.
- Category 2: MID-NET and NDB pseudonymous extracts can be partitioned per cohort using named graphs.

### Limitations/Known issues
- No universal semantics for nesting/inheritance between named graphs.
- "Triple bloat" if PROV-O is materialized exhaustively per triple — RDF-star mitigates but tooling support is uneven.
- SPARQL `GRAPH` semantics differ subtly across endpoints.
- Quoted triples (RDF-star) are new; downstream tools (SHACL validators, OWL reasoners) treat them inconsistently.

### Training data proxy
- High for RDF/TriG basics, moderate for advanced quad patterns. RDF-star is recent (2024+), so LLM coverage is shallower. Japanese RDF documentation moderate via DBCLS workshops and JST-funded RDF Portal docs.

## 4. SKOS and FHIR ValueSet/ConceptMap Governance

### Purpose
SKOS is a lightweight RDF vocabulary for representing thesauri, classifications, and subject heading schemes; FHIR ValueSet/ConceptMap are HL7 resources for managing terminology bindings and translations in clinical systems. Used together to govern controlled vocabularies referenced by clinical guidelines.

### Maintainer/Standards body
- SKOS: W3C Recommendation 18 August 2009; Semantic Web Deployment Working Group; editors Alistair Miles and Sean Bechhofer. Namespace `http://www.w3.org/2004/02/skos/core#`. Status: stable; no successor recommendation.
- FHIR Terminology: HL7 International. Current published version FHIR R5 (5.0.0, March 2023); R6 in active balloting (v6.0.0-ballot4 generated 18 May 2026, full Normative ANSI ballot; final R6 expected in 2027 or later, subject to multiple ballot rounds). ConceptMap relationship vocabulary explicitly references SKOS in R5/R6: "well-known relationships, such as those from the Simple Knowledge Organization System (SKOS), should be used where possible."

### Conceptual model
- SKOS: `skos:Concept` instances grouped into `skos:ConceptScheme`; relations `skos:broader`/`skos:narrower`/`skos:related`; labels `skos:prefLabel`, `skos:altLabel`, `skos:hiddenLabel` (per-language); mapping relations `skos:exactMatch`, `skos:closeMatch`, `skos:broadMatch`, `skos:narrowMatch`, `skos:relatedMatch`.
- FHIR CodeSystem: defines codes and their meaning; ValueSet: a named, versioned selection (intension or extension) from one or more CodeSystems; ConceptMap: directed mappings between ValueSets with relationships (`equivalent`, `source-is-narrower-than-target`, etc.).
- Governance pattern (MedCom DK and similar IGs): independent semantic versioning per artifact (Major.Minor.Patch) plus IG-level version; `$expand`, `$validate-code`, `$translate` operations.

### Expressiveness/Semantics
- SKOS deliberately avoids OWL DL strictness: `skos:broader` is NOT transitive by default (use `skos:broaderTransitive` for closure). Concepts are individuals, not classes.
- FHIR ValueSet binding strengths: `required`, `extensible`, `preferred`, `example`.
- ConceptMap `dependsOn` allows context-conditional mapping.

### Composability/Modularity
- SKOS schemes compose via `skos:inScheme`; mappings link schemes without modifying them.
- FHIR canonical URLs + version strings provide stable composition; ImplementationGuide resources bundle CodeSystem/ValueSet/ConceptMap artifacts.

### Suitability for autoformalization to IR
- SKOS is an attractive lightweight IR for terminology-heavy portions of guidelines (lists of conditions, drug classes). LLMs converge well because the vocabulary is small.
- FHIR ValueSet JSON is highly structured — LLMs can emit valid ValueSets reliably with schema-constrained decoding.
- Idempotency depends on deterministic concept URI minting.
- Limitation: SKOS cannot encode logical class definitions; guideline conditional logic must go elsewhere (CQL, FHIR PlanDefinition).

### Formal verification potential
- SKOS integrity constraints (e.g., `skos:exactMatch` disjoint from `skos:relatedMatch`, no cycles in `skos:broaderTransitive`) checkable via SHACL or qSKOS.
- ConceptMap translations checkable via FHIR Validator's `$translate` operation.
- Cross-guideline contradiction in terminology mappings detectable as conflicting `skos:exactMatch` targets.

### Tooling/Ecosystem maturity
- SKOS: PoolParty, VocBench 3, Skosmos browser, iQVoc, qSKOS validator, rdflib SKOS support.
- FHIR Terminology: HAPI FHIR, Snowstorm (SNOMED CT), Ontoserver, tx.fhir.org, Pathling.
- BabelFSH (Wiedekopf, Ohlsen, Kock-Schoppenhauer, Ingenerf, J Biomed Semantics 16:18, 2025, doi:10.1186/s13326-025-00343-4) converts SKOS/OWL-style terminologies to FHIR CodeSystem/ValueSet/ConceptMap.

### Japan-specific considerations
- Japanese MeSH (J-MeSH) is not openly licensed; JAMAS Thesaurus 10th edition (January 2023, 33,165 headwords, ~90% MeSH-derived, revised every 4 years since 1983) has restricted use.
- Life Science Dictionary (LSD), developed since 1993 by Kyoto University LSD Project (Prof. Shuji Kaneko, CC-BY-ND 2.1 JP) — operation transferred to DBCLS as of 31 March 2026.
- Open alternatives: `open-japanese-mesh` (Yamada & Tateisi, Genomics Inform 18(2):e22, 2020; 12,457-word Japanese-MeSH dictionary at github.com/roy29fuku/open-japanese-mesh) and O-JMeSH (Soares, Tateisi, Takatsuki, Yamaguchi, Genomics Inform 19(3):e26, 2021, doi:10.5808/gi.21014).
- MEDIS-DC standard masters (病名/手術処置/臨床検査/医薬品HOT) are tabular and freely downloadable from medis.or.jp. Current reference "標準マスターの概要と使い方 第24版" (July 2025, 11 masters); 病名マスター v2.1 contained 19,032 terms.
- MedDRA/J is maintained by JMO (Japanese Maintenance Organization) inside PMRJ (Pharmaceutical and Medical Device Regulatory Science Society of Japan), not PMDA directly; v28.0 released 1 March 2025, v28.1 on 1 September 2025, v29.0 on 1 March 2026 (1,380 change requests considered, 1,042 approved; 198 new PTs and 611 new LLTs; available MedDRA languages increased to 28 with the addition of Danish); two releases per year synchronized with English MedDRA. v28.1 made PT/LLT-only changes (1,412 change requests, no SOC/HLGT/HLT changes).

### Interoperability with other methods
- SKOS embeds in OWL ontologies as annotation layer for ICD-10, SNOMED CT.
- SHACL validates SKOS schemes (Eurostat, NCBO BioPortal use this).
- ROBOT can convert SKOS to OWL.
- Category 1: CQL `Code`/`ValueSet` directly reference FHIR ValueSets; CDS Hooks payloads carry ValueSet-bound codings; openEHR archetypes' value sets exportable as FHIR ValueSets.
- Category 2: MedDRA, ICD-10/11, YJ codes, ATC, MEDIS masters all expressible as FHIR CodeSystems.

### Limitations/Known issues
- SKOS lacks rich semantics for compositional terminologies like SNOMED CT post-coordinated expressions.
- FHIR ConceptMap's `dependsOn` is underused and inconsistently implemented.
- Governance of bilingual labels (Japanese/English) requires explicit `xml:lang` discipline that LLM outputs frequently miss.
- Round-tripping SNOMED CT compositional grammar between SKOS and OWL EL loses information.

### Training data proxy
- High for both SKOS and FHIR Terminology in English. Japanese: moderate via Aizawa-lab, MEDIS-DC publications, JAMI conference proceedings.

## 5. OBO Foundry Methods: ROBOT, ODK, DOSDP

### Purpose
A coordinated tool chain for building, releasing, and quality-checking biomedical ontologies according to OBO Foundry principles: ROBOT automates ontology operations; ODK packages best-practice build pipelines; DOSDP parameterizes axiom templates with YAML+TSV data.

### Maintainer/Standards body
OBO Foundry Operations Committee (community-governed, NIH/EMBL/MRC backed). Key maintainers: James Overton, Becky Tauber (ROBOT); Nicolas Matentzoglu, Chris Mungall (ODK); David Osumi-Sutherland, Jim Balhoff (DOSDP).
- ROBOT current major release line v1.9.x (Java/OWL API based; v1.9.10 released 18 February 2026).
- ODK v1.5+ (Docker image; ODK Lite and ODK Full variants).
- DOSDP-tools v0.x (Scala-based).
- Documented in Jackson et al. 2019 (ROBOT, BMC Bioinformatics 20:407, doi:10.1186/s12859-019-3002-3) and Matentzoglu et al. 2022 (ODK, Database/Oxford Academic, baac087, doi:10.1093/database/baac087).

### Conceptual model
- ROBOT: command-line pipeline (`robot extract --method MIREOT ... | robot reason --reasoner ELK | robot reduce | robot annotate ...`); each operation is a function over OWL ontologies, composable via Unix pipes.
- ODK: scaffolds a Git repository with `src/ontology/<onto>-edit.owl`, Makefile targets (`make prepare_release`, `make test`), GitHub Actions CI, Dockerized toolchain. Source files separate from release artifacts; release variants `-base`, `-full`, `-simple`.
- DOSDP: YAML pattern declares variables, axiom templates (Manchester syntax with placeholders); a TSV provides per-row variable bindings; `dosdp-tools` generates OWL axioms.

### Expressiveness/Semantics
- Pure OWL 2 in/out for all operations; reasoner support delegated (ELK default).
- DOSDP enforces uniform axiom structure across thousands of similar terms (e.g., "X is a part of Y" patterns in anatomy ontologies).
- ROBOT report performs lint-style QC: dangling references, missing labels, illegal whitespace, cross-reference syntax.

### Composability/Modularity
- ROBOT `extract` supports MIREOT, SLME (⊥/⊤ locality), STAR, BOT, TOP methods.
- ODK pipeline imports are configured in YAML (`product:` definitions); imports refreshed by `make refresh-imports`.
- DOSDP patterns can themselves import other patterns (`from:` directive).

### Suitability for autoformalization to IR
- DOSDP is uniquely well-suited as an LLM target: the LLM fills a TSV row given source text, then deterministic tools materialize axioms. This separation of generation from formalization sharply improves convergence/idempotency.
- ROBOT operations are idempotent by construction (set-based axiom manipulation) — repeated runs converge to the same release artifact.
- ODK enforces a release process that yields reproducible IRIs and version IRIs.

### Formal verification potential
- ROBOT `verify` runs SPARQL queries that must return no rows (gate the build).
- ROBOT `reason --reasoner ELK` exposes unsatisfiable classes.
- ROBOT `diff` compares ontology versions axiom-wise.
- ODK's QC step gates merges on consistency, profile conformance, structural checks.

### Tooling/Ecosystem maturity
- ROBOT GitHub repository ontodev/robot: 304 stars, 79 forks as of May 2026; used by ~all major OBO ontologies (GO, HPO, Mondo, Uberon, ChEBI).
- ODK: Docker image with ROBOT, owltools, fastobo-validator, dosdp-tools, OAK, Konclude, Apache Jena. "Seed my repo" generator scaffolds new ontologies.
- DOSDP-tools, OAK (Ontology Access Kit), sssom-py form a complementary Python ecosystem.

### Japan-specific considerations
- DBCLS BioHackathon series (BH25 in Mie/VISON, 14-20 September 2025) actively uses ROBOT/ODK in cross-ontology integration sessions; on2vec (OWL→vector via graph neural networks and sentence transformers; Steinberg, Kulmanov, Queralt-Rosinach, Chiba, Ellis et al., BioHackrXiv 2025) emerged from BH25. Japanese ontology authors (Kozaki, Yamagata) at JAIST/RIKEN/UTokyo publish using ROBOT pipelines. Japanese-language tutorials scarce; OBO Academy docs English-first.

### Interoperability with other methods
- Implements MIREOT (ROBOT `extract --method MIREOT`).
- Consumes/produces SSSOM mapping files (Matentzoglu et al., Database 2022, baac035, doi:10.1093/database/baac035) for ontology alignment governance.
- Compatible with BFO (ROBOT imports BFO and checks subClassOf to BFO classes).
- Compatible with SHACL via external pyshacl invocation in ODK pipelines (not native).
- Category 1: integrates indirectly when FHIR ImplementationGuides import OWL-derived ValueSets.
- Category 2: ICD-11, SNOMED CT, MedDRA can be wrapped as MIREOT imports.

### Limitations/Known issues
- JVM-heavy toolchain (Docker mitigates).
- DOSDP TSV editing in Excel introduces line-ending and encoding bugs.
- ODK assumes Git/GitHub workflow — internal corporate setups need adaptation.
- ROBOT diff is axiom-level, not change-language; KGCL aims to fill this gap.

### Training data proxy
- High in English OBO community (mailing lists, OBO Slack, OBO Academy `obook`, Oxford Academic Database papers). Lower outside life sciences. Japanese: minimal direct documentation.

## 6. BFO or DOLCE Upper-Ontology Evaluation

### Purpose
Upper (foundational) ontologies provide top-level categories (continuants/occurrents, qualities, processes) into which domain ontologies anchor, enabling cross-domain integration of clinical knowledge.

### Maintainer/Standards body
- BFO: Barry Smith (SUNY Buffalo) et al.; National Center for Ontological Research; BFO 2020 release, codified as ISO/IEC 21838-2:2021. GitHub: BFO-ontology/BFO-2020. Adopted by 650+ ontology projects per Wikipedia/BFO docs.
- DOLCE: Laboratory for Applied Ontology (LOA-CNR, Trento), Nicola Guarino, Claudio Masolo, Stefano Borgo, Aldo Gangemi, Alessandro Oltramari; codified as ISO/IEC 21838-3:2023.
- ISO/IEC 21838-1:2021 specifies top-level ontology requirements.

### Conceptual model
- BFO: realist; bipartite SNAP (continuants: object, quality, role, disposition, function) and SPAN (occurrents: process, process boundary, temporal region). Relations defined in BFO Relations Ontology (RO).
- DOLCE: descriptivist/cognitive; endurants vs. perdurants, qualities (with `Quale` for cognitive abstractions like color), abstracts (mathematical/conceptual entities). Allows "objects of thought" as basic units that BFO rejects.

### Expressiveness/Semantics
- BFO 2020 is axiomatized in OWL and Common Logic (CL); CL version normative for ISO conformance.
- DOLCE axiomatized in first-order logic (DOLCE-FOL) with an OWL approximation DOLCE-Lite.
- BFO is realist; DOLCE permits cognitive/conceptual entities.
- Mapping studies (Temal et al.; Seyed 2009 Nature Precedings npre.2009.3481.1; AKENATON project): ~100% BFO category coverage in DOLCE and ~81% reverse — six equivalence relations and thirteen subsumption relations established.

### Composability/Modularity
- BFO is a top-level "hub": OBO Foundry mandates BFO conformance; domain ontologies (Uberon, ChEBI, GO, OBI, HPO, IDO, OGMS) MIREOT BFO terms.
- DOLCE has DnS (Descriptions and Situations) and Ontology Design Patterns library; used in cultural heritage, robotics, telecardiology (AKENATON).

### Suitability for autoformalization to IR
- BFO provides a small, stable, well-documented vocabulary — LLMs anchor clinical concepts to BFO categories consistently across runs (e.g., "Diabetes Mellitus" → `bfo:Disposition`).
- DOLCE richer but cognitive layer more ambiguous, leading to lower LLM convergence in clinical contexts.
- For a CDS IR, BFO + OGMS (Ontology for General Medical Science) is the most idiomatic anchoring.

### Formal verification potential
- BFO CL version supports automated theorem proving (e.g., via Macleod).
- OWL versions support classification by ELK/HermiT.
- DOLCE-FOL supports first-order theorem proving (Vampire, E).

### Tooling/Ecosystem maturity
- BFO: Protégé, ROBOT, OWL API; BFO 2020 distributed in OWL/Functional/CL.
- DOLCE: DOLCE-Lite-Plus OWL distributions; OntologyDesignPatterns.org.
- Both have extensive published handbooks: "Building Ontologies with Basic Formal Ontology" (Arp, Smith, Spear, 2015, MIT Press); DOLCE technical reports from ISTC-CNR.

### Japan-specific considerations
- JAIST (Riichiro Mizoguchi's group) historically developed YAMATO (Yet Another More Advanced Top Ontology) — an alternative upper ontology contrasting with BFO/DOLCE. Mizoguchi's work on Hozo ontology editor is Japan-rooted. RIKEN SSBD Ontology references BFO indirectly via OBO imports. Japanese clinical ontology paper: Kozaki et al., "医療知識基盤の構築に向けた疾患オントロジーのLinked Open Data化," Trans JSAI 29(4), 2014, UTokyo Hospital.

### Interoperability with other methods
- BFO ↔ PROV-O alignment published (Prudhomme, De Colle, Liebers, Sculley, Xie, Cohen, Beverley, Scientific Data 12:282, 2025, doi:10.1038/s41597-025-04580-1) — total alignment of W3C PROV extensions (PROV-AQ, PROV-Dictionary, PROV-Links, PROV-Inverses, PROV Dublin Core) to BFO/CCO/RO.
- BFO ↔ SKOS: SKOS concepts typically treated as `iao:InformationContentEntity`.
- BFO is the de facto root for OBO Foundry; ROBOT/ODK pipelines assume BFO.
- Category 1: openEHR's reference model has informal mappings to BFO/DOLCE in academic literature but no official endorsement; HL7 RIM has historical alignment proposals.
- Category 2: ICD-11 Foundation has informal BFO-style continuant/process distinctions; SNOMED CT's top-level maps roughly to BFO.

### Limitations/Known issues
- BFO's realism excludes "patient-reported subjective experiences" as native entities, requiring workarounds (IAO information content entities).
- DOLCE's complexity raises onboarding cost; LLM coverage thinner.
- Both upper ontologies are debated philosophically; pragmatic mid-level ontologies (CCO, OGMS, IDO) absorb most modeling load.
- ICD-10 mapping to BFO/DOLCE incomplete (Héja, Varga, Surján, "Design principles of DOLCE-based formal representation of ICD10," Stud Health Technol Inform 2008;136:821-826).

### Training data proxy
- BFO: moderate-high (Smith textbook, OBO docs, NCOR materials). DOLCE: moderate. YAMATO: low (mostly Japanese). LLM grounding stronger on BFO.

## 7. MIREOT Modular Ontology Imports

### Purpose
A pragmatic guideline (and toolset) for importing only the minimum required information from external ontologies, avoiding the inconsistency and editing burdens of full `owl:imports`.

### Maintainer/Standards body
Originally Mélanie Courtot, Frank Gibson, Allyson L. Lister, James Malone, Daniel Schober, Ryan R. Brinkman, Alan Ruttenberg. Published in Applied Ontology 6(1):23–33, 2011 (SAGE/IOS Press, doi:10.3233/AO-2011-0087). Reference implementations in OntoFox and ROBOT.

### Conceptual model
- Specify: external class IRI, its direct asserted parent in source, and a (configurable) superclass in target.
- Bring in minimal triple set: rdfs:label, IAO definitions, selected annotation axioms.
- Three import "cases" depending on whether one needs (1) the class only, (2) the class plus annotations, (3) the class plus structural context.

### Expressiveness/Semantics
- Preserves IRIs from source ontology (URL persistence).
- Does not preserve full logical context; some entailments from the source are lost.
- Locality-based module extraction (SLME, used by ROBOT `extract --method BOT/TOP/STAR`) is a logically conservative alternative; MIREOT is "lighter but less safe."

### Composability/Modularity
- The defining modularization pattern in OBO Foundry: imports/ folder contains MIREOT'ed slices of external ontologies, refreshed automatically by ODK.
- Combines with DOSDP: patterns reference MIREOT'ed parents.

### Suitability for autoformalization to IR
- LLMs can be prompted to emit MIREOT directives (term IRI + parent IRI), then ROBOT materializes the slice. This separation improves idempotency since the slice is deterministically computed.
- Reduces hallucinated IRIs by constraining the LLM to declared external terms.

### Formal verification potential
- The imported slice can be reasoned over with ELK/HermiT alongside the local ontology.
- Inconsistencies introduced by partial imports are detectable but may be artifacts of missing axioms — diagnostic care needed.

### Tooling/Ecosystem maturity
- OntoFox web server (Xiang, Courtot, Brinkman, Ruttenberg, He et al.); ROBOT `extract --method MIREOT --upper-term ... --lower-term ...`; Protégé MIREOT plugin (UAMS-DBMI/MIREOT-plugin on GitHub).
- Widely used by OBI, IAO, VO, HPO, Mondo.

### Japan-specific considerations
- Used by Japanese OBO contributors (RIKEN SSBD Ontology, Yamagata et al. 2025) referencing external biomedical ontologies. Documentation in Japanese sparse; BioHackathon community trains contributors directly.

### Interoperability with other methods
- ROBOT/ODK natively orchestrate MIREOT.
- SSSOM mappings can complement MIREOT by capturing cross-ontology equivalences not expressible as imports.
- SHACL can validate that all referenced external IRIs resolve to declared imports.
- Category 1/2: relevant when authoring OWL artifacts referencing clinical terminologies.

### Limitations/Known issues
- Not logically conservative — can break inferences depending on omitted axioms.
- Manual maintenance of import declarations error-prone; ODK refresh helps.
- Recent OBO Foundry guidance often prefers SLME locality modules over MIREOT for tooling that supports both.

### Training data proxy
- Moderate-low. Specialized to OBO; ~hundreds of papers cite MIREOT. Stack Overflow presence minimal. Japanese coverage minimal.

## 8. Ontology Alignment and Repair: LogMap, AgreementMakerLight

### Purpose
Automated systems that compute mappings between ontologies (e.g., FMA↔SNOMED CT↔NCI Thesaurus), with built-in or post-hoc repair to remove logically inconsistent mappings.

### Maintainer/Standards body
- LogMap: Ernesto Jiménez-Ruiz, Bernardo Cuenca Grau et al. (Oxford / City, University of London). Open-source at github.com/ernestojimenezruiz/logmap-matcher.
- AgreementMakerLight (AML): Daniel Faria, Catia Pesquita, Isabel Cruz et al. (Lisbon / UIC). Open-source at github.com/AgreementMakerLight/AgreementMakerLight.
- Benchmark: Ontology Alignment Evaluation Initiative (OAEI), annual since 2004; tracks include Anatomy, Large BioMed, Conference, Disease and Phenotype, Bio-ML.

### Conceptual model
- Lexical matchers (string similarity, edit distance) propose candidate equivalences.
- Structural matchers (locality-based module signatures, similarity flooding) refine candidates.
- LogMap performs "built-in" reasoning + diagnosis during alignment; AML applies global repair via modularization and confidence-based heuristics post-alignment (Santos, Faria, Pesquita, Couto, PLoS ONE 2015, doi:10.1371/journal.pone.0144807).
- Output: alignments as 5-tuples (entity1, entity2, relation, confidence, type) — increasingly serialized as SSSOM TSV.

### Expressiveness/Semantics
- Alignments are typically equivalence (`owl:equivalentClass`, `skos:exactMatch`), subsumption (`rdfs:subClassOf`, `skos:broadMatch`/`narrowMatch`), or relatedness.
- Coherence: alignment is coherent iff union of source + target + alignment has no unsatisfiable classes.
- Repair = removing a minimal subset of mappings to restore coherence.

### Composability/Modularity
- Both systems use locality-based modularization to scale to FMA/SNOMED/NCI sizes (tens of thousands of classes).
- OAEI Large BioMed reference alignments are refined with ALCOMO + LogMap repair, with disagreement-flagged mappings marked "?" (e.g., FMA-NCI: 2,686 "=" + 338 "?"; FMA-SNOMED: 6,026 "=" + 2,982 "?"; SNOMED-NCI: 17,210 "=" + 1,634 "?").

### Suitability for autoformalization to IR
- LLM-generated cross-guideline mappings benefit from automated repair: feed LLM candidate `skos:exactMatch` triples into LogMap/AML for coherence repair.
- Improves convergence: multiple LLM runs producing slightly different mappings can be unioned and repaired into a single coherent alignment.

### Formal verification potential
- LogMap exposes inconsistent-class explanations; AML uses HermiT + ELK in its repair pipeline.
- Repair guarantees logical coherence under a chosen reasoner; does not guarantee semantic correctness (some valid mappings may be sacrificed).

### Tooling/Ecosystem maturity
- LogMap: Java, OWL API, integrated with HOBBIT and MELT (Matching EvaLuation Toolkit since OAEI 2021).
- AML: Java, lightweight, top OAEI performer; per Faria et al. OAEI 2014 system description: "AML's participation in the OAEI 2014 was very successful, as it obtained the highest F-measure in 6 of the 8 ontology matching tracks."
- Other systems: AML-Compound, ALCOMO, BERTMap (LLM-based, Oxford 2022, AAAI 2022; He, Chen, Jiménez-Ruiz et al., arXiv:2112.02682).
- SSSOM tooling (sssom-py) for downstream mapping management.

### Japan-specific considerations
- Limited direct Japan-specific deployment in published literature; OAEI participation from Japan sparse. Cross-lingual matching of Japanese clinical terminologies (ICD-10 JP, MEDIS病名 ↔ SNOMED CT) is an open research area; BERTMap and LogMap-ML more naturally applied to cross-lingual cases.

### Interoperability with other methods
- Produces SSSOM-formatted alignments consumable by ROBOT and OAK.
- Feeds FHIR ConceptMap generation (each LogMap mapping → one ConceptMap.group.element.target).
- SHACL can validate alignments for required metadata.
- Category 1: FHIR Terminology `$translate` operation can be backed by aligned ConceptMaps.
- Category 2: relevant for SNOMED CT↔ICD-10/11↔MedDRA↔MEDIS cross-walks underlying Japanese CDS.

### Limitations/Known issues
- Repair may remove correct mappings (precision/recall trade-off).
- Lexical bias: cross-lingual (Japanese ↔ English) alignment performance poor without translation pre-step.
- Reproducibility across OAEI versions depends on which reference repair was applied.
- OAEI 2020 Large BioMed report: "even the most precise alignment sets may lead to a large amount of unsatisfiable classes."

### Training data proxy
- Moderate in academic ML/Semantic Web communities; LLMs see substantial OAEI papers but limited code-level Q&A. Japanese coverage low.

## 9. Japanese Clinical Entity Linking and Concept Normalization

### Purpose
Identify mentions of clinical entities (diseases, drugs, anatomical sites, lab tests) in Japanese clinical text and link them to standard identifiers (ICD-10, SNOMED CT, MEDIS病名コード, UMLS CUIs, MedDRA/J LLT codes).

### Maintainer/Standards body
- No single standards body. Major research groups:
  - NAIST Social Computing Lab (sociocom, Eiji Aramaki) — MedNER-J, MedEX/J, MANBYO Dictionary, Hyakuyaku Dictionary, JMED-DICT, JMED-LLM.
  - NII alabnii / Akiko Aizawa lab — JMedRoBERTa.
  - University of Tokyo Hospital — UTH-BERT (first Japanese clinical BERT, ~120M clinical texts).
  - Kyoto University / NICT — clinical NLP, Japanese terminology resources.
  - Cabinet Office SIP / ELYZA × UTokyo Matsuo-Iwasawa Lab — ELYZA-LLM-Med.
- Public dictionaries: MANBYO_202106 (NAIST sociocom, ~74 MB MeCab UTF-8 user-dic); JMED-DICT mini (sip3-d2.naist.jp/jmed-dict.html); J-MeDic (Ito, Nagai, Okahisa, Wakamiya, Iwao, Aramaki, LREC 2018 pp. 2365-2369).

### Conceptual model
- Standard NLP pipeline: tokenization (MeCab/Sudachi/Juman++) → BERT/RoBERTa encoding → BIO sequence labeling for NER → candidate generation via dictionary + dense vector matching → reranking/disambiguation.
- MedNER-J adds positive/negative classification (a mention's polarity: "C" = positive symptom/disease, "CN" = negative).
- For cross-lingual linking, mention → Japanese standard term → mapping (via O-JMeSH or open-japanese-mesh) → MeSH/UMLS CUI.

### Expressiveness/Semantics
- Output is span + label + (optional) code + polarity + timestamp; no formal logic.
- Joint extraction of disease + body part + temporal + negation studied in JMED-LLM benchmark and JMedBench (Jiang, Huang, Aizawa, arXiv:2409.13317; COLING 2025).

### Composability/Modularity
- Plugins for spaCy, GiNZA, Stanza.
- Pipelines integrate with Apache cTAKES-style stages but Japanese pipelines less standardized than English.

### Suitability for autoformalization to IR
- A prerequisite for guideline autoformalization: the LLM must ground Japanese terminology mentions to controlled-vocabulary IRIs before emitting RDF/OWL/SHACL.
- Convergence depends critically on stable mention→code resolution; running entity linking before LLM formalization (and providing the linked codes back as prompt context) sharply improves idempotency.
- LLMs (GPT-4-class) alone show high recall but inconsistent code grounding; pairing with dictionary lookup (MANBYO + MEDIS病名 + JMED-DICT) is necessary.

### Formal verification potential
- Not direct; produces structured inputs to downstream verification systems.
- Inter-annotator agreement and entity-linking F1 are the verification metrics; JMedBench provides Japanese biomedical evaluation across 20 datasets and 5 tasks.

### Tooling/Ecosystem maturity
- Models on Hugging Face: alabnii/jmedroberta-base-sentencepiece, alabnii/jmedroberta-base-manbyo-wordpiece, alabnii/jmedroberta-base-sentencepiece-vocab50000 (all CC-BY-NC-SA-4.0); UTH-BERT (restricted access); stardust-coder/jmedllm-7b-v1.
- GitHub: sociocom/MedNER-J, sociocom/JMED-LLM, roy29fuku/open-japanese-mesh.
- Reported MedNER-J F1 on pharmaceutical-care records (Ohno et al., JMIR Form Res 2024;8:e55798): 0.76 on Assessment data, 0.70 on Objective data, 0.46 on Subjective data, 0.35 on Plan data.
- JMedLoRA (Sukeda, Suzuki, Sakaji, Kodera, UTokyo, arXiv:2310.10083) for LoRA fine-tuning of LLMs on Japanese medical text.

### Japan-specific considerations
- License complexity: JMedRoBERTa is CC-BY-NC-SA-4.0 (non-commercial); MANBYO is open but license terms are loose; JAMAS Thesaurus is restricted. Commercial CDS deployment requires careful licensing audit.
- UMLS Japanese MeSH translation is created by JAMAS; access requires UMLS Metathesaurus license.
- Real EHR data hard to access; SS-MIX2 archives, MID-NET, and NDB pseudonymous data are gated.

### Interoperability with other methods
- Output codes plug into FHIR Coding (Category 1) and into named-graph-scoped guideline triples (Category 3).
- Output IRIs/codes can feed SKOS concept schemes for terminology normalization governance.
- LogMap/AML can align Japanese-extracted concept schemes against SNOMED CT/UMLS.
- Category 2: directly relevant for MEDIS masters, MedDRA/J, ICD-10/11 JP, YJ/HOT codes, JLAC10 lab codes.

### Limitations/Known issues
- Domain shift: models trained on academic papers (JMedRoBERTa) underperform on EHR / pharmaceutical-care notes.
- Negation, hedging, and family history inconsistently handled.
- Code coverage gaps: 28.3% (~17,000 of ~62,000) of EHR-derived symptom/disease expressions are NOT covered by ICD10対応標準病名マスター (NAIST sociocom MANBYO pre-survey, sociocom.jp/~data/2018-manbyo/), hence the MANBYO supplementation.
- No publicly available large-scale Japanese clinical NER gold standard at MIMIC-IV scale.

### Training data proxy
- Moderate for Japanese clinical NLP (NAIST sociocom GitHub; NLP Annual Meeting proceedings; J-STAGE). Low compared to English clinical NLP. LLM coding agents can use Hugging Face transformers idiomatically.

## 10. Versioned Ontology/Terminology Diffing and Change-Impact Analysis

### Purpose
Tools and methods to compare ontology versions, classify changes (effectual vs. ineffectual; safe vs. breaking), and propagate the impact of changes to downstream applications (annotations, ML models, CDS rules).

### Maintainer/Standards body
- Bubastis: EMBL-EBI SPOT team (GitHub EBISPOT/bubastis); integrated into BioPortal/OntoPortal, runs automatically per new ontology release.
- OWLDiff: Czech Technical University Prague / NeOn toolkit lineage; Protégé and NeOn plugins.
- ROBOT diff: OBO Foundry (Jackson et al.); axiom-level set diff over OWL/OBO syntaxes.
- Ecco: Manchester / Bio-Health Informatics Group (Gonçalves, Parsia, Sattler); "hybrid" syntactic+semantic diff distinguishing effectual from ineffectual changes.
- KGCL (Knowledge Graph Change Language): Hegde, Vendetti, Goutte-Gattat, Caufield et al., "A change language for ontologies and knowledge graphs," Database (Oxford Academic) 2025:baae133, published 22 January 2025, doi:10.1093/database/baae133. Integrated with Ontobot LLM-based change agent.
- Other: COnto-diff, ChImp (Visual), DynDiff, OnDeT, QuickGO Change Log, GOtrack, OVCS, Quit Diff.

### Conceptual model
- Syntactic diff: set difference over axioms.
- Semantic diff: difference in entailment closure under chosen reasoner (HermiT for DL, ELK for EL).
- Effectual change taxonomy (Gonçalves, Parsia, Sattler): added/removed entailments classified by complexity.
- KGCL: structured change instructions (`Rename`, `Obsolete`, `MoveSubtree`, `AddSubclass`) — both human-readable CNL and JSON/YAML.

### Expressiveness/Semantics
- ROBOT diff and Bubastis: axiom set difference; miss higher-level operations like "merged two classes."
- Ecco: distinguishes which removed/added axioms had logical impact.
- KGCL: explicit change-operation semantics, transactional.

### Composability/Modularity
- Diff tools chain into ODK pipelines.
- KGCL operations can be replayed; supports RAG-style change explanation.

### Suitability for autoformalization to IR
- Crucial for iterative LLM autoformalization: between successive LLM runs, diff to detect drift; if IR is supposed to be idempotent, diff should be empty.
- KGCL aligns with LLM-generated change descriptions — LLM emits KGCL, deterministic tools apply changes (separating reasoning from execution improves convergence).

### Formal verification potential
- Ecco-style effectual/ineffectual classification IS a form of semantic verification (does this change alter entailments?).
- Change-impact analysis on downstream CDS rules: identify rules referencing changed IRIs and re-validate.

### Tooling/Ecosystem maturity
- ROBOT diff is stable and widely used in OBO release pipelines.
- Bubastis (EBISPOT/bubastis) is mature, integrated into BioPortal/OntoPortal end points, runs automatically per ontology release.
- KGCL is newer (2023–2025), part of OBO tooling roadmap; integrated with Ontobot (LLM-based change agent).
- Hegde et al. (Database 2025) reports from a pre-publication community survey on ontology change tracking that 82% of ontology users rate staying informed about changes as "extremely or very important."

### Japan-specific considerations
- DBCLS does not publish a Japanese-specific diff tool; uses ROBOT diff/Bubastis. TogoVar versioning uses dataset-level snapshots in named graphs. MEDIS-DC publishes versioned masters annually but does not provide formal diff tooling — third parties (HL7 Japan, MEDIS subscribers) compute deltas ad hoc.

### Interoperability with other methods
- Pairs with ROBOT/ODK release workflows.
- Pairs with SHACL: re-run SHACL validation after changes to catch breaking constraint impacts.
- Pairs with named graphs/TriG: per-version graphs allow set-difference queries via SPARQL.
- Category 1: critical when ValueSets/ConceptMaps version-bump; FHIR canonical URLs + versioning conventions assist; FHIR ImplementationGuide cross-version tooling complements.
- Category 2: ICD-10→ICD-11, MedDRA semi-annual releases (e.g., v28.0 March 2025 → v28.1 September 2025 with 1,412 change requests → v29.0 March 2026 with 1,380 change requests considered), MEDIS master annual revisions all benefit from diff/CIA tooling.

### Limitations/Known issues
- Axiom-level diff produces noise; users want change operations.
- Cross-format diffing (OBO vs. OWL/XML vs. Turtle) requires normalization first (ROBOT canonical).
- No standard for FHIR Terminology change-impact analysis equivalent to KGCL.
- Embedding-based ontology versions (vector indices) are not diffable at axiom level; "mind the change, bridge the gap" research (Babaei Giglou et al.) addresses this.

### Training data proxy
- Moderate for ROBOT diff and Bubastis; low for KGCL (new). Stack Overflow presence minimal. Japanese: minimal.
