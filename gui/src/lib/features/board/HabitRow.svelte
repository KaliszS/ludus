<script lang="ts">
	import { Check } from '@lucide/svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import { tint } from '$lib/domain/palette';
	import { board } from '$lib/state/board.svelte';
	import type { Habit } from '$lib/api/types';
	import Stepper from './Stepper.svelte';

	let { habit }: { habit: Habit } = $props();

	const accent = $derived(habit.color ?? 'var(--color-accent)');
	const done = $derived(board.done(habit.id));
</script>

<li class="flex items-center gap-3 py-2">
	<span
		class="grid size-9 shrink-0 place-items-center rounded-xl transition-all duration-300"
		style:background-color={done ? tint(habit.color, '24') : 'transparent'}
		style:color={done ? accent : 'var(--color-muted)'}
		style:box-shadow={done ? 'none' : 'inset 0 0 0 1px var(--color-line)'}
	>
		<Icon name={habit.icon} size={17} />
	</span>

	<span class="min-w-0 flex-1 truncate text-sm" class:font-medium={done}>{habit.name}</span>

	{#if habit.tracking === 'quantity'}
		<Stepper
			value={done}
			unit={habit.unit}
			accent={String(accent)}
			onset={(next) => board.set(habit, board.selectedDay, next)}
		/>
	{:else}
		<button
			onclick={() => board.toggle(habit, board.selectedDay)}
			aria-pressed={done > 0}
			aria-label="{done ? 'Undo' : 'Check'} {habit.name}"
			class="grid size-9 shrink-0 place-items-center rounded-full transition-all duration-200
			       active:scale-90"
			style:background-color={done ? accent : 'transparent'}
			style:color={done ? 'white' : 'var(--color-muted)'}
			style:box-shadow={done ? 'none' : 'inset 0 0 0 1.5px var(--color-line)'}
		>
			<Check size={17} strokeWidth={2.5} />
		</button>
	{/if}
</li>
