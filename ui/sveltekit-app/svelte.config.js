import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

// Read-only bilingual report UI: fully prerendered static site.
// adapter-static with fallback off (default); prerender enabled via
// src/routes/+layout.ts so every route emits static HTML.
/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter()
	}
};

export default config;
