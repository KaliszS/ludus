<script lang="ts">
	import { Check } from '@lucide/svelte';

	interface Props {
		reached: number;
		total: number;
	}
	let { reached, total }: Props = $props();

	const SIZE = 32;
	const RADIUS = 13;
	const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

	const segment = $derived(CIRCUMFERENCE / Math.max(total, 1));
	/** One requirement draws a full ring; several need a visible break between arcs. */
	const gap = $derived(total > 1 ? Math.min(4, segment * 0.3) : 0);
	const complete = $derived(total > 0 && reached >= total);
</script>

<span
	class="relative grid shrink-0 place-items-center transition-colors duration-300"
	style:width="{SIZE}px"
	style:height="{SIZE}px"
	title="{reached} of {total} met"
>
	<svg viewBox="0 0 {SIZE} {SIZE}" class="absolute inset-0 -rotate-90" aria-hidden="true">
		{#each Array.from({ length: Math.max(total, 1) }, (_, index) => index) as index (index)}
			<circle
				cx={SIZE / 2}
				cy={SIZE / 2}
				r={RADIUS}
				fill="none"
				stroke-width="3"
				stroke-linecap="round"
				stroke={index < reached ? 'var(--color-good)' : 'var(--color-line)'}
				stroke-dasharray="{segment - gap} {CIRCUMFERENCE - segment + gap}"
				stroke-dashoffset={-index * segment}
				class="transition-[stroke] duration-500 ease-out"
				style:transition-delay="{Math.min(index, 8) * 60}ms"
			/>
		{/each}
	</svg>

	<span
		class="grid size-5 place-items-center rounded-full transition-colors duration-300"
		style:background-color={complete ? 'var(--color-good-soft)' : 'transparent'}
		style:color={complete ? 'var(--color-good)' : 'var(--color-muted)'}
	>
		<Check size={14} strokeWidth={3} />
	</span>

	<span class="sr-only">{reached} of {total} requirements met</span>
</span>
