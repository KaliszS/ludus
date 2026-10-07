<script lang="ts">
	import { flip } from 'svelte/animate';
	import { dndzone, type DndEvent } from 'svelte-dnd-action';
	import { FolderPlus, Plus, X } from '@lucide/svelte';
	import CategoryHeader from '$lib/features/habits/CategoryHeader.svelte';
	import HabitForm from '$lib/features/habits/HabitForm.svelte';
	import HabitRow from '$lib/features/habits/HabitRow.svelte';
	import { shelve, type Shelf } from '$lib/domain/categories';
	import { habits } from '$lib/state/habits.svelte';
	import type { Habit } from '$lib/api/types';
	import Button from '$lib/ui/Button.svelte';
	import Empty from '$lib/ui/Empty.svelte';
	import Skeleton from '$lib/ui/Skeleton.svelte';
	import TextField from '$lib/ui/TextField.svelte';

	const FLIP_MS = 180;
	/** The zone of habits outside any category. */
	const LOOSE = 'loose';

	type Kind = 'category' | 'subcategory' | 'habit';

	/** What kind of grip was pressed. Only zones of that kind accept a drag, so taps on
	 *  the rows' own buttons never start one, and a habit never lands among categories. */
	let grabbing = $state<Kind | null>(null);
	// finalize also fires when the items array is swapped out from under a zone,
	// so persisting needs proof that a drag actually happened.
	let dragged = false;
	let saving: ReturnType<typeof setTimeout> | undefined;

	let shelves = $state.raw<Shelf[]>([]);
	let loose = $state.raw<Habit[]>([]);

	let creating = $state(false);
	/** The category whose plus opened a form inside it. */
	let addingTo = $state<string | null>(null);
	let addingCategory = $state(false);
	let categoryName = $state('');

	$effect(() => {
		const layout = shelve(habits.categories, habits.items);
		shelves = layout.shelves;
		loose = layout.loose;
	});

	$effect(() => {
		void habits.includeArchived;
		habits.load();
	});

	const zone = <T,>(items: T[], kind: Kind) => ({
		items,
		type: kind,
		flipDurationMs: FLIP_MS,
		dragDisabled: grabbing !== kind,
		dropTargetStyle: {}
	});

	/** Replaces one shelf wherever it sits, leaving the others untouched. */
	function patchShelf(id: string, change: (shelf: Shelf) => Shelf) {
		shelves = shelves.map((top) => {
			if (top.id === id) return change(top);
			if (!top.subs.some((sub) => sub.id === id)) return top;
			return { ...top, subs: top.subs.map((sub) => (sub.id === id ? change(sub) : sub)) };
		});
	}

	function setHabits(id: string, items: Habit[]) {
		if (id === LOOSE) loose = items;
		else patchShelf(id, (shelf) => ({ ...shelf, habits: items }));
	}

	function settle() {
		grabbing = null;
		if (!dragged) return;
		dragged = false;
		// A move between zones finalizes both of them in one go: save once, after both.
		clearTimeout(saving);
		saving = setTimeout(() => habits.saveLayout({ shelves, loose }));
	}

	function moveHabits(id: string, event: CustomEvent<DndEvent<Habit>>, done = false) {
		setHabits(id, event.detail.items);
		if (done) settle();
		else dragged = true;
	}

	function moveSubs(parent: string, event: CustomEvent<DndEvent<Shelf>>, done = false) {
		patchShelf(parent, (shelf) => ({ ...shelf, subs: event.detail.items }));
		if (done) settle();
		else dragged = true;
	}

	function moveTops(event: CustomEvent<DndEvent<Shelf>>, done = false) {
		shelves = event.detail.items;
		if (done) settle();
		else dragged = true;
	}

	const toggleAdding = (id: string) => (addingTo = addingTo === id ? null : id);

	async function addCategory(name: string) {
		if (!name.trim()) return;
		if (await habits.addCategory(name, null)) {
			categoryName = '';
			addingCategory = false;
		}
	}

	const card = 'rounded-[var(--radius-card)] border border-line bg-surface px-2 py-2.5 sm:px-3';
</script>

<!-- A grip pressed and released without moving never finalizes, so let go of it here. -->
<svelte:window onpointerup={() => !dragged && (grabbing = null)} />

{#snippet addForm(id: string)}
	{#if addingTo === id}
		<div class="my-2 ml-5">
			<HabitForm category={id} inline onclose={() => (addingTo = null)} />
		</div>
	{/if}
{/snippet}

{#snippet habitZone(id: string, items: Habit[], quiet = false)}
	{@const open = items.length === 0 && (grabbing === 'habit' || !quiet)}
	<div class="relative">
		<ul
			class:min-h-10={open}
			use:dndzone={zone(items, 'habit')}
			onconsider={(event) => moveHabits(id, event)}
			onfinalize={(event) => moveHabits(id, event, true)}
		>
			{#each items as habit (habit.id)}
				<li animate:flip={{ duration: FLIP_MS }}>
					<HabitRow {habit} ongrab={() => (grabbing = 'habit')} />
				</li>
			{/each}
		</ul>
		<!-- Outside the zone: every child of a zone has to be one of its items. -->
		{#if open}
			<p class="pointer-events-none absolute inset-0 grid place-items-center text-xs text-muted/70">
				{grabbing === 'habit' ? 'Drop here' : 'No habits here yet'}
			</p>
		{/if}
	</div>
{/snippet}

<div class="mb-4 flex flex-wrap items-center gap-x-2 gap-y-3">
	<h2 class="mr-auto text-xs font-medium tracking-wide text-muted uppercase">
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
	<Button onclick={() => (addingCategory = !addingCategory)}>
		<span class="flex items-center gap-1.5"><FolderPlus size={14} /> Category</span>
	</Button>
	<Button variant="primary" onclick={() => (creating = !creating)}>
		<span class="flex items-center gap-1.5"><Plus size={14} strokeWidth={2.25} /> Habit</span>
	</Button>
</div>

{#if creating}
	<div class="mb-4">
		<HabitForm onclose={() => (creating = false)} />
	</div>
{/if}

{#if addingCategory}
	<div class="mb-4 flex items-center gap-1">
		<TextField
			bind:value={categoryName}
			placeholder="Category name, then Enter"
			class="min-w-0 flex-1"
			onenter={addCategory}
		/>
		<button
			onclick={() => ((addingCategory = false), (categoryName = ''))}
			aria-label="Cancel"
			class="grid size-9 place-items-center rounded-full text-muted transition hover:bg-sunken hover:text-ink"
		>
			<X size={16} strokeWidth={2} />
		</button>
	</div>
{/if}

{#if habits.items.length || shelves.length}
	<div
		class="space-y-3"
		use:dndzone={zone(shelves, 'category')}
		onconsider={(event) => moveTops(event)}
		onfinalize={(event) => moveTops(event, true)}
	>
		{#each shelves as top (top.id)}
			<section animate:flip={{ duration: FLIP_MS }} class={card}>
				<CategoryHeader
					shelf={top}
					ongrab={() => (grabbing = 'category')}
					onadd={() => toggleAdding(top.id)}
				/>
				{@render addForm(top.id)}
				{@render habitZone(top.id, top.habits, top.subs.length > 0)}
				<div
					class:min-h-8={grabbing === 'subcategory' && top.subs.length === 0}
					use:dndzone={zone(top.subs, 'subcategory')}
					onconsider={(event) => moveSubs(top.id, event)}
					onfinalize={(event) => moveSubs(top.id, event, true)}
				>
					{#each top.subs as sub (sub.id)}
						<div animate:flip={{ duration: FLIP_MS }} class="ml-3">
							<CategoryHeader
								shelf={sub}
								nested
								ongrab={() => (grabbing = 'subcategory')}
								onadd={() => toggleAdding(sub.id)}
							/>
							{@render addForm(sub.id)}
							{@render habitZone(sub.id, sub.habits)}
						</div>
					{/each}
				</div>
			</section>
		{/each}
	</div>

	<!-- Mounted on the grip press, before the drag starts, so the zone is there to take it. -->
	{#if loose.length || grabbing === 'habit' || !shelves.length}
		<section class="{card} {shelves.length ? 'mt-3' : ''}">
			{#if shelves.length}
				<h3 class="px-1 pb-1 text-[15px] font-semibold text-muted">Uncategorized</h3>
			{/if}
			{@render habitZone(LOOSE, loose)}
		</section>
	{/if}
{:else if habits.loading}
	<Skeleton />
{:else}
	<Empty text="No habits yet." />
{/if}
