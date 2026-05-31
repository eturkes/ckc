<script lang="ts">
	import type { ConflictCard } from '$lib/types';

	let { card }: { card: ConflictCard } = $props();

	// Render an arbitrary normalized_view / witness / repair / table_cell JSON
	// value as indented text. The report artifact is already canonical, so this
	// is a faithful read-only echo, not a re-serialization of meaning.
	const pretty = (value: unknown): string => JSON.stringify(value, null, 2);
</script>

<!-- One conflict card in the SPEC §21 card order, JA source-first. -->
<article class="card">
	<header class="card-head">
		<h2 lang="en">{card.conflict_type}</h2>
		<p class="meta">
			<span class="badge severity">{card.severity}</span>
			<span class="badge class">{card.classification}</span>
			<code class="id">{card.conflict_id}</code>
		</p>
	</header>

	<!-- (1) JA exact source spans + table cells. -->
	<section>
		<h3>出典スパン <span class="en">Source spans</span></h3>
		{#each card.source_spans as span (span.span_id)}
			<div class="span">
				<p class="ja" lang="ja">{span.raw_text}</p>
				{#if span.display_text !== span.raw_text}
					<p class="display" lang="ja">{span.display_text}</p>
				{/if}
				<p class="span-meta">
					<code>{span.span_id}</code>
					<span class="lang">{span.language}</span>
					{#if span.table_cell}
						<code class="cell">{pretty(span.table_cell)}</code>
					{/if}
				</p>
			</div>
		{/each}
	</section>

	<!-- (2) EN gloss (with JA source gloss when present). -->
	{#if card.gloss_ja || card.gloss_en}
		<section>
			<h3>グロス <span class="en">Gloss</span></h3>
			{#if card.gloss_ja}<p class="ja" lang="ja">{card.gloss_ja}</p>{/if}
			{#if card.gloss_en}<p class="en-text" lang="en">{card.gloss_en}</p>{/if}
		</section>
	{/if}

	<!-- (3) normalized CKC view. -->
	<section>
		<h3>正規化ビュー <span class="en">Normalized CKC view</span></h3>
		<pre>{pretty(card.normalized_view)}</pre>
	</section>

	<!-- (4) bilingual explanation. -->
	<section>
		<h3>説明 <span class="en">Explanation</span></h3>
		<p class="ja" lang="ja">{card.explanation_ja}</p>
		<p class="en-text" lang="en">{card.explanation_en}</p>
	</section>

	<!-- (5) minimal witness / model. -->
	{#if card.witness.length > 0}
		<section>
			<h3>ウィットネス <span class="en">Witness</span></h3>
			{#each card.witness as item, i (i)}
				<pre>{pretty(item)}</pre>
			{/each}
		</section>
	{/if}

	<!-- (6) certificate evidence + (7) certificate-depth badge. -->
	<section>
		<h3>
			証明書エビデンス <span class="en">Certificate evidence</span>
			{#if card.certificate_depth}
				<span class="badge depth">{card.certificate_depth}</span>
			{/if}
		</h3>
		{#if card.certificate_evidence.length > 0}
			<ul class="certs">
				{#each card.certificate_evidence as cert (cert.certificate_id)}
					<li>
						<span class="badge depth">{cert.certificate_class}</span>
						<code>{cert.certificate_id}</code>
						<span class="solver">{cert.solver_or_checker}</span>
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<!-- (8) repair candidates as review prompts. -->
	{#if card.repair_candidates.length > 0}
		<section>
			<h3>修正候補 <span class="en">Repair candidates</span></h3>
			{#each card.repair_candidates as repair, i (i)}
				<pre>{pretty(repair)}</pre>
			{/each}
		</section>
	{/if}

	<!-- (9) human review question. -->
	<section>
		<h3>レビュー質問 <span class="en">Review question</span></h3>
		<p class="ja" lang="ja">{card.human_review_question_ja}</p>
		<p class="en-text" lang="en">{card.human_review_question_en}</p>
	</section>

	<!-- (10) adjudication status. -->
	<section>
		<h3>判定ステータス <span class="en">Adjudication status</span></h3>
		<p><span class="badge status">{card.adjudication_status}</span></p>
	</section>
</article>

<style>
	.card {
		border: 1px solid #ccc;
		border-radius: 6px;
		padding: 1rem 1.25rem;
		margin: 1.5rem 0;
		background: #fff;
	}
	.card-head {
		border-bottom: 2px solid #333;
		padding-bottom: 0.5rem;
		margin-bottom: 0.75rem;
	}
	.card-head h2 {
		margin: 0;
		font-family: ui-monospace, monospace;
		font-size: 1.15rem;
	}
	.meta {
		margin: 0.4rem 0 0;
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		align-items: center;
	}
	section {
		margin: 0.9rem 0;
	}
	h3 {
		font-size: 0.95rem;
		margin: 0 0 0.35rem;
		color: #222;
	}
	.en {
		color: #777;
		font-weight: normal;
		font-size: 0.85em;
	}
	.ja {
		margin: 0.2rem 0;
		line-height: 1.6;
	}
	.display {
		margin: 0.1rem 0;
		color: #555;
	}
	.en-text {
		margin: 0.2rem 0;
		color: #444;
	}
	.span {
		border-left: 3px solid #e0a000;
		padding-left: 0.6rem;
		margin: 0.5rem 0;
	}
	.span-meta {
		margin: 0.2rem 0 0;
		font-size: 0.8rem;
		color: #666;
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		align-items: center;
	}
	pre {
		background: #f5f5f5;
		border-radius: 4px;
		padding: 0.6rem;
		overflow-x: auto;
		font-size: 0.8rem;
		margin: 0.3rem 0;
	}
	code {
		font-family: ui-monospace, monospace;
		font-size: 0.85em;
	}
	.cell {
		background: #f0f0f0;
		padding: 0 0.3rem;
		border-radius: 3px;
	}
	.certs {
		list-style: none;
		padding: 0;
		margin: 0;
	}
	.certs li {
		display: flex;
		gap: 0.5rem;
		align-items: center;
		padding: 0.2rem 0;
	}
	.solver {
		color: #555;
		font-size: 0.85rem;
	}
	.badge {
		display: inline-block;
		padding: 0.1rem 0.5rem;
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 600;
		border: 1px solid #aaa;
		background: #eee;
		color: #333;
	}
	.badge.depth {
		background: #1f4e79;
		color: #fff;
		border-color: #1f4e79;
	}
	.badge.severity {
		background: #b03030;
		color: #fff;
		border-color: #b03030;
	}
	.badge.status {
		background: #6a4caf;
		color: #fff;
		border-color: #6a4caf;
	}
</style>
