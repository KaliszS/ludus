<script lang="ts">
	let { rows = 3 }: { rows?: number } = $props();
</script>

<ul class="animate-pulse divide-y divide-line" aria-hidden="true">
	{#each Array.from({ length: rows }, (_, index) => index) as index (index)}
		<li class="flex items-center gap-3 py-2">
			<span class="size-9 shrink-0 rounded-xl bg-sunken"></span>
			<span class="h-3 rounded-full bg-sunken" style:width="{40 + ((index * 17) % 35)}%"></span>
			<span class="ml-auto size-9 shrink-0 rounded-full bg-sunken"></span>
		</li>
	{/each}
</ul>
