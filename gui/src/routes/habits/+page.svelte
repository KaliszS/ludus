<script lang="ts">
	import { flip } from 'svelte/animate';
	import { dndzone, type DndEvent } from 'svelte-dnd-action';
	import HabitForm from '$lib/features/habits/HabitForm.svelte';
	import HabitRow from '$lib/features/habits/HabitRow.svelte';
	import { habits } from '$lib/state/habits.svelte';
	import type { Habit } from '$lib/api/types';
	import Empty from '$lib/ui/Empty.svelte';
	import Skeleton from '$lib/ui/Skeleton.svelte';

	const FLIP_MS = 180;

	let ordered = $state<Habit[]>([]);
	// A drag only starts from the grip, so taps on the row's buttons still work.
	let dragDisabled = $state(true);
	// finalize also fires when the items array is swapped out from under the zone,
	// so persisting needs proof that a drag actually happened.
	let dragged = $state(false);

	function consider(event: CustomEvent<DndEvent<Habit>>) {
		dragged = true;
		ordered = event.detail.items;
	}

	async function finalize(event: CustomEvent<DndEvent<Habit>>) {
		ordered = event.detail.items;
		dragDisabled = true;
		if (!dragged) return;
		dragged = false;
		await habits.reorder(ordered);
	}

	$effect(() => {
		ordered = habits.items;
	});

	$effect(() => {
		void habits.includeArchived;
		habits.load();
	});
</script>

<div class="mb-8">
	<HabitForm />
</div>

<div class="mb-4 flex items-center justify-between">
	<h2 class="text-xs font-medium tracking-wide text-muted uppercase">
		{habits.items.length} habit{habits.items.length === 1 ? '' : 's'}
	</h2>
	<label class="flex items-center gap-2 text-xs text-muted">
		<input
			type="checkbox"
			bind:checked={habits.includeArchived}
			class="accent-[var(--color-accent)]"
		/>
		Show archived
	</label>
</div>

{#if ordered.length}
	<ul
		class="space-y-2"
		use:dndzone={{ items: ordered, flipDurationMs: FLIP_MS, dragDisabled, dropTargetStyle: {} }}
		onconsider={consider}
		onfinalize={finalize}
	>
		{#each ordered as habit (habit.id)}
			<li animate:flip={{ duration: FLIP_MS }}>
				<HabitRow {habit} ongrab={() => (dragDisabled = false)} />
			</li>
		{/each}
	</ul>
{:else if habits.loading}
	<Skeleton />
{:else}
	<Empty text="No habits yet." />
{/if}
