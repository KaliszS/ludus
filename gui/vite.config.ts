import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import { defineConfig, type Plugin } from 'vite';

/** Stores live in plain modules, and hot-swapping one gives the modules re-imported
 *  since then a fresh, empty copy while the rest keep the old one - after a branch
 *  switch the page ends up split between the two. A change to a plain module reloads
 *  the page instead, so every import keeps one URL. Components still update in place. */
const reloadOnModuleChange: Plugin = {
	name: 'ludus:reload-on-module-change',
	hotUpdate({ file }) {
		if (this.environment.name !== 'client' || !/\/src\/.+\.(ts|js)$/.test(file)) return;
		this.environment.hot.send({ type: 'full-reload' });
		return [];
	}
};

export default defineConfig({
	// The single .env lives at the repo root, next to the daemon's.
	envDir: '..',
	server: { port: 7531, strictPort: true },
	plugins: [
		reloadOnModuleChange,
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			adapter: adapter({ fallback: 'index.html', strict: false }),
			env: { dir: '..' }
		}),
		SvelteKitPWA({
			registerType: 'autoUpdate',
			manifest: {
				name: 'Ludus',
				short_name: 'Ludus',
				display: 'standalone',
				start_url: '/',
				background_color: '#0b0b0d',
				theme_color: '#0b0b0d'
			}
		})
	]
});
