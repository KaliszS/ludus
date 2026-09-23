<script lang="ts">
	interface Props {
		done: number;
		quota: number;
		segments: number;
		color: string;
	}
	let { done, quota, segments, color }: Props = $props();

	const ratio = $derived(quota > 0 ? Math.min(done / quota, 1) : 0);
</script>

{#if segments > 1}
	<div class="flex gap-1" style:--fill={color}>
		{#each Array.from({ length: segments }, (_, index) => index) as index (index)}
			<span
				class="h-2 flex-1 rounded-full transition-all duration-500 ease-out"
				style:background-color={index < done ? color : 'var(--color-line)'}
				style:transition-delay="{Math.min(index, 10) * 35}ms"
			></span>
		{/each}
	</div>
{:else}
	<div class="h-2 overflow-hidden rounded-full bg-line">
		<span
			class="block h-full rounded-full"
			style:width="{ratio * 100}%"
			style:background-color={color}
			style="transition: width 700ms var(--ease-out-expo)"
		></span>
	</div>
{/if}
