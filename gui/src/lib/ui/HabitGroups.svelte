<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Habit } from '$lib/api/types';
	import type { HabitGroup } from '$lib/domain/categories';

	interface Props {
		groups: HabitGroup[];
		/** Renders one habit; it is placed inside a list, so it should be an <li>. */
		item: Snippet<[Habit]>;
		/** The height of one item, so a label lines up with the first row of its items. */
		line?: string;
	}
	let { groups, item, line = 'h-9' }: Props = $props();

	const labelled = $derived(groups.some((group) => group.label !== null));
</script>

<!-- One row per category, its label in a column of its own so the items of every row
     start at the same edge and still wrap at their natural width. -->
{#if labelled}
	<div class="grid grid-cols-[auto_minmax(0,1fr)] items-start gap-x-3 gap-y-1.5">
		{#each groups as group (group.id)}
			<span
				class="flex max-w-28 items-center text-[11px] font-medium tracking-wide uppercase {line}
				       {group.nested ? 'text-muted/70' : 'text-muted'}"
				title={group.label}
			>
				<span class="truncate">{group.label}</span>
			</span>
			<ul class="flex flex-wrap gap-1.5">
				{#each group.habits as habit (habit.id)}
					{@render item(habit)}
				{/each}
			</ul>
		{/each}
	</div>
{:else if groups.length}
	<ul class="flex flex-wrap gap-1.5">
		{#each groups[0].habits as habit (habit.id)}
			{@render item(habit)}
		{/each}
	</ul>
{/if}
