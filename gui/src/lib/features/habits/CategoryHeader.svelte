<script lang="ts">
	import { FolderPlus, GripVertical, X } from '@lucide/svelte';
	import TextField from '$lib/ui/TextField.svelte';
	import { habits } from '$lib/state/habits.svelte';
	import type { Shelf } from '$lib/domain/categories';

	interface Props {
		shelf: Shelf;
		/** Subcategories sit under a top-level category and cannot hold more of their own. */
		nested?: boolean;
		ongrab: () => void;
	}
	let { shelf, nested = false, ongrab }: Props = $props();

	let addingSub = $state(false);
	let subName = $state('');

	const count = $derived(
		shelf.habits.length + shelf.subs.reduce((sum, sub) => sum + sub.habits.length, 0)
	);

	async function addSub(name: string) {
		if (!name.trim()) return;
		if (await habits.addCategory(name, shelf.id)) {
			subName = '';
			addingSub = false;
		}
	}

	function remove() {
		const subs = shelf.subs.length ? ' and its subcategories' : '';
		if (!confirm(`Delete “${shelf.category.name}”${subs}? Its habits stay, uncategorized.`)) {
			return;
		}
		habits.removeCategory(shelf.id);
	}

	const action =
		'grid size-7 shrink-0 place-items-center rounded-full text-muted opacity-0 transition ' +
		'group-hover/header:opacity-100 hover:bg-sunken hover:text-ink focus:opacity-100 ' +
		'[@media(hover:none)]:opacity-100';
</script>

<div class="group/header flex items-center gap-1 {nested ? 'pt-2' : ''}">
	<span
		role="button"
		tabindex="-1"
		aria-label="Reorder {shelf.category.name}"
		onpointerdown={ongrab}
		class="grid h-7 w-5 shrink-0 cursor-grab touch-none place-items-center text-muted opacity-0
		       transition group-hover/header:opacity-100 hover:text-ink active:cursor-grabbing
		       [@media(hover:none)]:opacity-60"
	>
		<GripVertical size={14} strokeWidth={1.75} />
	</span>

	<!-- Renaming in place: the header is the name. -->
	<input
		value={shelf.category.name}
		aria-label="Category name"
		onchange={(event) => habits.renameCategory(shelf.id, event.currentTarget.value)}
		class="-ml-1 min-w-0 flex-1 truncate rounded-md bg-transparent px-1 py-0.5 transition outline-none
		       hover:bg-sunken focus:bg-sunken
		       {nested
			? 'text-[11px] font-semibold tracking-wide text-muted uppercase focus:text-ink'
			: 'text-[15px] font-semibold'}"
	/>

	<span class="shrink-0 px-1 text-[11px] text-muted tabular-nums">{count}</span>

	{#if !nested}
		<button
			onclick={() => (addingSub = !addingSub)}
			aria-label="Add a subcategory"
			title="Add a subcategory"
			class={action}
		>
			<FolderPlus size={14} strokeWidth={1.75} />
		</button>
	{/if}
	<button onclick={remove} aria-label="Delete category" title="Delete category" class={action}>
		<X size={14} strokeWidth={2} />
	</button>
</div>

{#if addingSub}
	<div class="mt-1 mb-2 ml-5 flex items-center gap-1">
		<TextField
			bind:value={subName}
			placeholder="Subcategory name, then Enter"
			class="min-w-0 flex-1 px-3 py-1.5 text-sm"
			onenter={addSub}
		/>
		<button
			onclick={() => ((addingSub = false), (subName = ''))}
			aria-label="Cancel"
			class="grid size-7 place-items-center rounded-full text-muted transition hover:bg-sunken hover:text-ink"
		>
			<X size={14} strokeWidth={2} />
		</button>
	</div>
{/if}
