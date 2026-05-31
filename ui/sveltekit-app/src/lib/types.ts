// TypeScript mirror of the ckc-report `Report` model (crates/ckc-report/src/lib.rs,
// SPEC 21/23). The UI is a structural consumer of the committed report.json
// artifact, so these interfaces track the Rust serde shape (snake_case fields,
// string-valued enums). Severity / classification / conflict_type stay `string`
// to track the Rust enums exactly; the C0–C9 certificate ladder (SPEC 12.2)
// is a stable closed set worth enumerating for the depth badge.

export type CertificateClass =
	| 'C0-Parsed'
	| 'C1-Schema'
	| 'C2-Normal'
	| 'C3-Grounded'
	| 'C4-Executable'
	| 'C5-Portfolio'
	| 'C6-ProofObject'
	| 'C7-Kernel'
	| 'C8-Adjudicated'
	| 'C9-Assured';

/** One source span on a conflict card (SPEC 21 element 1): JA exact text + cell. */
export interface CardSpan {
	span_id: string;
	raw_text: string;
	display_text: string;
	table_cell: unknown | null;
	language: string;
}

/** One certificate backing a conflict card (SPEC 21 element 6). */
export interface CardCertificate {
	certificate_id: string;
	certificate_class: CertificateClass;
	solver_or_checker: string;
}

/** One conflict card (SPEC 21) in §21 card order. */
export interface ConflictCard {
	conflict_id: string;
	conflict_type: string;
	severity: string;
	classification: string;
	source_spans: CardSpan[];
	gloss_ja: string;
	gloss_en: string;
	normalized_view: unknown;
	explanation_ja: string;
	explanation_en: string;
	witness: unknown[];
	certificate_evidence: CardCertificate[];
	certificate_depth: CertificateClass | null;
	repair_candidates: unknown[];
	human_review_question_ja: string;
	human_review_question_en: string;
	adjudication_status: string;
}

/** One bucket of the certificate-depth distribution (SPEC 12.2/23). */
export interface DepthCount {
	certificate_class: CertificateClass;
	count: number;
}

/** One bucket of the conflict-taxonomy counts (SPEC 15/23). */
export interface TaxonomyCount {
	conflict_type: string;
	count: number;
}

/** Run-level tallies (SPEC 23). */
export interface ReportSummary {
	n_documents: number;
	n_spans: number;
	n_claims: number;
	n_rules: number;
	n_conflicts: number;
	certificate_depth_distribution: DepthCount[];
	conflict_taxonomy_counts: TaxonomyCount[];
}

/** The Phase-0 bilingual report (SPEC 21/23). */
export interface Report {
	command: string;
	producer_version: string;
	summary: ReportSummary;
	conflict_cards: ConflictCard[];
}
