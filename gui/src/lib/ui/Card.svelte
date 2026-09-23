<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children, class: extra = '' }: { children: Snippet; class?: string } = $props();
</script>

<section class="rounded-[var(--radius-card)] border border-line bg-surface p-4 sm:p-5 {extra}">
	{@render children()}
</section>
