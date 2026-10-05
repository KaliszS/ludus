<script module lang="ts">
	/** A five-point star around the centre of a 24 unit coin. */
	const STAR = Array.from({ length: 10 }, (_, index) => {
		const angle = ((index * 36 - 90) * Math.PI) / 180;
		const radius = index % 2 ? 2.1 : 4.9;
		return `${(12 + radius * Math.cos(angle)).toFixed(2)},${(12.4 + radius * Math.sin(angle)).toFixed(2)}`;
	}).join(' ');
</script>

<script lang="ts">
	import type { Medal } from '$lib/api/types';
	import { METALS } from '$lib/domain/plan';

	interface Props {
		medal: Medal;
		size?: number;
		/** Unearned medals stay visible as a faint promise of what is up there. */
		earned?: boolean;
		title?: string;
	}
	let { medal, size = 18, earned = true, title }: Props = $props();

	const id = $props.id();
	const metal = $derived(METALS[medal]);
</script>

<!-- The rim is lit from the opposite corner to the face, which is what makes the
     face read as sunk into it. -->
<svg
	width={size}
	height={size}
	viewBox="0 0 24 24"
	role="img"
	aria-label={title ?? `${medal} medal${earned ? '' : ' (not yet)'}`}
	class="shrink-0 transition-[opacity,filter,transform] duration-500
	       ease-[cubic-bezier(0.34,1.56,0.64,1)]"
	style:opacity={earned ? 1 : 0.3}
	style:transform={earned ? 'scale(1)' : 'scale(0.88)'}
	style:filter={earned
		? `drop-shadow(0 1px 2.5px color-mix(in oklab, ${metal.mid} 70%, transparent))`
		: 'grayscale(0.7)'}
>
	{#if title}<title>{title}</title>{/if}
	<defs>
		<linearGradient id="{id}-face" x1="0.15" y1="0.1" x2="0.85" y2="0.95">
			<stop offset="0" stop-color={metal.light} />
			<stop offset="0.55" stop-color={metal.mid} />
			<stop offset="1" stop-color={metal.dark} />
		</linearGradient>
		<linearGradient id="{id}-rim" x1="0.85" y1="0.95" x2="0.15" y2="0.1">
			<stop offset="0" stop-color={metal.light} />
			<stop offset="0.5" stop-color={metal.mid} />
			<stop offset="1" stop-color={metal.dark} />
		</linearGradient>
	</defs>
	<circle cx="12" cy="12" r="11.25" fill="url(#{id}-rim)" />
	<circle cx="12" cy="12" r="8.5" fill="url(#{id}-face)" />
	<polygon points={STAR} fill={metal.star} opacity="0.92" />
	<ellipse
		cx="9"
		cy="7.2"
		rx="4.2"
		ry="1.9"
		fill="#fff"
		opacity="0.28"
		transform="rotate(-28 9 7.2)"
	/>
</svg>
