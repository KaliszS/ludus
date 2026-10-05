<script lang="ts">
	import Medal from '$lib/ui/Medal.svelte';
	import { tierLabel } from '$lib/domain/plan';
	import type { Tier } from '$lib/api/types';

	interface Props {
		tier: Tier;
		/** Its place among the live tiers, numbering the ones without a medal. */
		index: number;
		size?: number;
	}
	let { tier, index, size = 16 }: Props = $props();
</script>

<!-- A tier without a medal still needs a marker to line its quotas up with: a plain
     numbered disc. -->
{#if tier.medal}
	<Medal medal={tier.medal} {size} title={tierLabel(tier, index)} />
{:else}
	<span
		class="grid shrink-0 place-items-center rounded-full font-semibold text-muted ring-1 ring-line ring-inset"
		style:width="{size}px"
		style:height="{size}px"
		style:font-size="{Math.round(size * 0.5)}px"
		title={tierLabel(tier, index)}
	>
		{index + 1}
	</span>
{/if}
