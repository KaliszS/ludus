<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { session } from '$lib/state/session.svelte';

	/** Either way the address bar loses the code, so a reload cannot try to spend it twice. */
	onMount(async () => {
		const code = page.url.searchParams.get('code');
		const error = page.url.searchParams.get('error');
		if (!code) return goto(`/login?error=${error ?? 'sign_in_failed'}`, { replaceState: true });

		try {
			await session.complete(code);
			await goto('/', { replaceState: true });
		} catch {
			await goto('/login?error=expired', { replaceState: true });
		}
	});
</script>

<p class="py-24 text-center text-sm text-muted">Signing you in…</p>
