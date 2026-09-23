<script lang="ts">
	import 'flag-icons/css/flag-icons.min.css';
	import { iconFor } from '$lib/domain/icons';

	interface Props {
		name: string | null;
		size?: number;
	}
	let { name, size = 18 }: Props = $props();

	const def = $derived(iconFor(name));
</script>

{#if def.kind === 'lucide'}
	{@const Glyph = def.glyph}
	<Glyph {size} strokeWidth={1.75} />
{:else if def.kind === 'flag'}
	<span
		class="fi fi-{def.code} rounded-[2px]"
		style:width="{size}px"
		style:height="{size * 0.75}px"
		style:background-size="cover"
	></span>
{:else if def.kind === 'drawn'}
	<svg
		xmlns="http://www.w3.org/2000/svg"
		width={size}
		height={size}
		viewBox="0 0 24 24"
		fill="none"
		stroke="currentColor"
		stroke-width="1.75"
		stroke-linecap="round"
		stroke-linejoin="round"
	>
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		{@html def.body}
	</svg>
{:else}
	<span
		class="font-semibold tabular-nums"
		style:font-size="{size * 0.85}px"
		style:line-height="{size}px"
	>
		{def.text}
	</span>
{/if}
