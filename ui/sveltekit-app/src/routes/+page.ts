import type { PageLoad } from './$types';
import type { Report } from '$lib/types';

// Load the build-copied static report artifact at prerender. The package.json
// `prebuild` hook copies examples/research_kernel/fixtures/report.json ->
// static/report.json; a relative fetch resolves it from the static dir during
// prerendering, and SvelteKit inlines the response into the prerendered page so
// the UI renders from the committed artifact alone with no server runtime
// (prerender = true is inherited from src/routes/+layout.ts).
export const load: PageLoad = async ({ fetch }) => {
	const response = await fetch('/report.json');
	const report: Report = await response.json();
	return { report };
};
