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

	/** A counted habit shows its count; the stepper opens on demand. */
	let open = $state(false);
	let chip = $state<HTMLElement | null>(null);

	function close(event: PointerEvent) {
		if (open && chip && !chip.contains(event.target as Node)) open = false;
	}
</script>

<svelte:window
	onpointerdown={close}
	onkeydown={(event) => event.key === 'Escape' && (open = false)}
/>

<!-- Done chips take on the habit's colour, so the day reads at a glance; the rest stay
     outlined. They wrap at their natural width, which keeps twenty habits to a few lines. -->
<li
	bind:this={chip}
	class="flex h-9 items-center rounded-full transition-colors duration-300"
	style:background-color={done ? tint(habit.color, '24') : 'transparent'}
	style:box-shadow={done ? 'none' : 'inset 0 0 0 1px var(--color-line)'}
>
	{#if habit.tracking === 'quantity' && open}
		<span class="flex items-center gap-1.5 pr-0.5 pl-3">
			<span style:color={accent}><Icon name={habit.icon} size={15} /></span>
			<span class="text-sm font-medium">{habit.name}</span>
			<Stepper
				value={done}
				unit={habit.unit}
				accent={String(accent)}
				onset={(next) => board.set(habit, board.selectedDay, next)}
			/>
		</span>
	{:else}
		<button
			onclick={() =>
				habit.tracking === 'quantity' ? (open = true) : board.toggle(habit, board.selectedDay)}
			aria-pressed={habit.tracking === 'quantity' ? undefined : done > 0}
			aria-label={habit.tracking === 'quantity'
				? `Set ${habit.name}`
				: `${done ? 'Undo' : 'Check'} ${habit.name}`}
			title={habit.description || undefined}
			class="flex h-full items-center gap-1.5 rounded-full px-3 text-sm transition active:scale-95"
		>
			<span style:color={done ? accent : 'var(--color-muted)'}>
				<Icon name={habit.icon} size={15} />
			</span>
			<span class:font-medium={done}>{habit.name}</span>
			{#if habit.tracking === 'quantity'}
				<span
					class="text-xs font-semibold tabular-nums"
					style:color={done ? accent : 'var(--color-muted)'}
				>
					{done}{habit.unit ? ` ${habit.unit}` : ''}
				</span>
			{:else if done}
				<span style:color={accent}><Check size={14} strokeWidth={3} /></span>
			{/if}
		</button>
	{/if}
</li>
