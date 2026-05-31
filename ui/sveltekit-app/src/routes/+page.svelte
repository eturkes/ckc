<script lang="ts">
	import type { PageData } from './$types';
	import ConflictCard from '$lib/components/ConflictCard.svelte';

	let { data }: { data: PageData } = $props();
	const report = $derived(data.report);
</script>

<main>
	<header class="report-head">
		<h1>CKC レポート <span class="en">CKC Report</span></h1>
		<p class="command"><code>{report.command}</code></p>
		<p class="version">producer: {report.producer_version}</p>
	</header>

	<section class="summary">
		<h2>サマリー <span class="en">Summary</span></h2>
		<ul class="counts">
			<li><span class="n">{report.summary.n_documents}</span> 文書 / documents</li>
			<li><span class="n">{report.summary.n_spans}</span> スパン / spans</li>
			<li><span class="n">{report.summary.n_claims}</span> 主張 / claims</li>
			<li><span class="n">{report.summary.n_rules}</span> ルール / rules</li>
			<li><span class="n">{report.summary.n_conflicts}</span> 不整合 / conflicts</li>
		</ul>

		<h3>証明書深度分布 <span class="en">Certificate depth distribution</span></h3>
		<ul class="dist">
			{#each report.summary.certificate_depth_distribution as d (d.certificate_class)}
				<li><span class="badge depth">{d.certificate_class}</span> × {d.count}</li>
			{/each}
		</ul>

		<h3>不整合分類 <span class="en">Conflict taxonomy</span></h3>
		<ul class="dist">
			{#each report.summary.conflict_taxonomy_counts as t (t.conflict_type)}
				<li><code>{t.conflict_type}</code> × {t.count}</li>
			{/each}
		</ul>
	</section>

	<section class="conflicts">
		<h2>不整合カード <span class="en">Conflict cards</span></h2>
		{#each report.conflict_cards as card (card.conflict_id)}
			<ConflictCard {card} />
		{/each}
	</section>
</main>

<style>
	:global(body) {
		margin: 0;
		background: #f0f0f0;
		color: #1a1a1a;
		font-family:
			system-ui,
			'Hiragino Sans',
			'Noto Sans JP',
			sans-serif;
	}
	main {
		max-width: 920px;
		margin: 0 auto;
		padding: 1.5rem 1rem 4rem;
	}
	.report-head h1 {
		margin: 0 0 0.3rem;
	}
	.command code {
		font-family: ui-monospace, monospace;
		background: #e4e4e4;
		padding: 0.15rem 0.4rem;
		border-radius: 4px;
	}
	.version {
		color: #666;
		font-size: 0.85rem;
		margin: 0.2rem 0 0;
	}
	.summary {
		background: #fff;
		border: 1px solid #ccc;
		border-radius: 6px;
		padding: 1rem 1.25rem;
		margin: 1.5rem 0;
	}
	.counts {
		list-style: none;
		padding: 0;
		margin: 0.5rem 0 1rem;
		display: flex;
		flex-wrap: wrap;
		gap: 1rem;
	}
	.counts .n {
		font-weight: 700;
		font-size: 1.2rem;
	}
	.dist {
		list-style: none;
		padding: 0;
		margin: 0.3rem 0 0.8rem;
		display: flex;
		flex-wrap: wrap;
		gap: 0.6rem;
		align-items: center;
	}
	.en {
		color: #777;
		font-weight: normal;
		font-size: 0.85em;
	}
	.badge {
		display: inline-block;
		padding: 0.1rem 0.5rem;
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 600;
	}
	.badge.depth {
		background: #1f4e79;
		color: #fff;
	}
	code {
		font-family: ui-monospace, monospace;
		font-size: 0.85em;
	}
</style>
