import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import { defineConfig } from 'vite';

export default defineConfig({
	// The single .env lives at the repo root, next to the daemon's.
	envDir: '..',
	server: { port: 7531, strictPort: true },
	plugins: [
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
