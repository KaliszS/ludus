<script lang="ts">
	import { fly } from 'svelte/transition';
	import { toast } from '$lib/state/toast.svelte';
</script>

{#if toast.message}
	<div
		transition:fly={{ y: 16, duration: 260, opacity: 0 }}
		class="fixed inset-x-4 bottom-24 z-50 mx-auto max-w-sm sm:bottom-6"
	>
		<button
			onclick={() => toast.dismiss()}
			class="w-full rounded-2xl bg-accent px-4 py-3 text-left text-sm text-white shadow-xl
			       shadow-black/10 transition active:scale-[0.98]"
		>
			{toast.message}
		</button>
	</div>
{/if}
