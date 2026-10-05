<script lang="ts">
	import { Check } from '@lucide/svelte';

	interface Props {
		/** Arcs the ring is cut into: one per requirement, or per expected check-in. */
		segments: number;
		/** How many arcs are filled; a fraction partially fills the one in progress. */
		filled: number;
		size?: number;
	}
	let { segments, filled, size = 32 }: Props = $props();

	const count = $derived(Math.max(Math.round(segments), 1));
	const radius = $derived(size / 2 - 3);
	const circumference = $derived(2 * Math.PI * radius);

	const arc = $derived(circumference / count);
	/** One arc closes into a full ring; several need a visible break between them. */
	const gap = $derived(count > 1 ? Math.min(5, arc * 0.35) : 0);
	const drawn = $derived(arc - gap);

	const parts = $derived(
		Array.from({ length: count }, (_, index) => ({
			index,
			fraction: Math.min(Math.max(filled - index, 0), 1)
		}))
	);
	const complete = $derived(filled >= count);
</script>

<span
	class="relative grid shrink-0 place-items-center"
	style:width="{size}px"
	style:height="{size}px"
>
	<svg viewBox="0 0 {size} {size}" class="absolute inset-0 -rotate-90" aria-hidden="true">
		{#each parts as part (part.index)}
			<circle
				cx={size / 2}
				cy={size / 2}
				r={radius}
				fill="none"
				stroke="var(--color-line)"
				stroke-width="3"
				stroke-linecap="round"
				stroke-dasharray="{drawn} {circumference - drawn}"
				stroke-dashoffset={-part.index * arc}
			/>
			{#if part.fraction > 0}
				<circle
					cx={size / 2}
					cy={size / 2}
					r={radius}
					fill="none"
					stroke="var(--color-good)"
					stroke-width="3"
					stroke-linecap="round"
					stroke-dasharray="{drawn * part.fraction} {circumference - drawn * part.fraction}"
					stroke-dashoffset={-part.index * arc}
					style="transition: stroke-dasharray 600ms cubic-bezier(0.16, 1, 0.3, 1)"
					style:transition-delay="{Math.min(part.index, 8) * 60}ms"
				/>
			{/if}
		{/each}
	</svg>

	<span
		class="grid place-items-center rounded-full transition-colors duration-300"
		style:width="{size * 0.6}px"
		style:height="{size * 0.6}px"
		style:background-color={complete ? 'var(--color-good-soft)' : 'transparent'}
		style:color={complete ? 'var(--color-good)' : 'var(--color-muted)'}
	>
		<Check size={size * 0.44} strokeWidth={3} />
	</span>
</span>
