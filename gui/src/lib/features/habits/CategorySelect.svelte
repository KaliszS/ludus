<script lang="ts">
	import { shelve } from '$lib/domain/categories';
	import { habits } from '$lib/state/habits.svelte';

	interface Props {
		value: string | null;
		onchange: (id: string | null) => void;
	}
	let { value, onchange }: Props = $props();

	const shelves = $derived(shelve(habits.categories, []).shelves);
</script>

<!-- A native select: on a phone it opens the system picker, which beats any custom list. -->
<select
	value={value ?? ''}
	onchange={(event) => onchange(event.currentTarget.value || null)}
	aria-label="Category"
	class="rounded-xl border border-line bg-canvas px-3 py-2 text-sm transition outline-none
	       focus:border-accent"
>
	<option value="">No category</option>
	{#each shelves as top (top.id)}
		<option value={top.id}>{top.category.name}</option>
		{#each top.subs as sub (sub.id)}
			<option value={sub.id}>{top.category.name} › {sub.category.name}</option>
		{/each}
	{/each}
</select>
