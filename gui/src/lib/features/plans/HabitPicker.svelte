<script lang="ts">
	import Icon from '$lib/ui/Icon.svelte';
	import HabitGroups from '$lib/ui/HabitGroups.svelte';
	import { groups } from '$lib/domain/categories';
	import { tint } from '$lib/domain/palette';
	import { habits } from '$lib/state/habits.svelte';

	interface Props {
		selected: string[];
		ontoggle: (id: string) => void;
	}
	let { selected, ontoggle }: Props = $props();

	const grouped = $derived(groups(habits.categories, habits.active));
</script>

<HabitGroups groups={grouped} line="h-[26px]">
	{#snippet item(habit)}
		{@const on = selected.includes(habit.id)}
		<li>
			<button
				onclick={() => ontoggle(habit.id)}
				aria-pressed={on}
				class="flex h-[26px] items-center gap-1.5 rounded-full px-2.5 text-xs transition"
				style:background-color={on ? tint(habit.color, '24') : 'transparent'}
				style:color={on ? (habit.color ?? 'var(--color-accent)') : 'var(--color-muted)'}
				style:box-shadow={on ? 'none' : 'inset 0 0 0 1px var(--color-line)'}
			>
				<Icon name={habit.icon} size={13} />
				{habit.name}
			</button>
		</li>
	{/snippet}
</HabitGroups>
