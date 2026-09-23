<script lang="ts">
	import DayPicker from '$lib/features/board/DayPicker.svelte';
	import HabitRow from '$lib/features/board/HabitRow.svelte';
	import PlanCard from '$lib/features/board/PlanCard.svelte';
	import { fullLabel, today } from '$lib/domain/date';
	import { board } from '$lib/state/board.svelte';
	import Card from '$lib/ui/Card.svelte';
	import Empty from '$lib/ui/Empty.svelte';
	import Skeleton from '$lib/ui/Skeleton.svelte';
	import { swipe } from '$lib/ui/swipe';
	import { fly } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';

	/** Arrow keys are the desktop equivalent of the swipe, minus any new chrome. */
	function onkeydown(event: KeyboardEvent) {
		const target = event.target as HTMLElement | null;
		if (target?.closest('input, textarea, select')) return;
		if (event.key === 'ArrowLeft') board.shiftDay(-1);
		if (event.key === 'ArrowRight') board.shiftDay(1);
	}

	$effect(() => {
		board.load();
	});
</script>

<svelte:window {onkeydown} />

<div class="space-y-4">
	<div
		class="touch-pan-y"
		use:swipe={{ onprevious: () => board.shiftDay(-1), onnext: () => board.shiftDay(1) }}
	>
		<Card>
			<DayPicker />

			<div class="mt-4 border-t border-line pt-4">
				<!-- Keyed on the day so switching it replays the slide, showing the move happened. -->
				{#key board.selectedDay}
					<div in:fly={{ x: board.direction * 24, opacity: 1, duration: 200, easing: cubicOut }}>
						<h2 class="text-sm font-semibold">
							{board.selectedDay === today() ? 'Today' : fullLabel(board.selectedDay)}
						</h2>

						{#if board.active.length}
							<ul class="mt-1 divide-y divide-line">
								{#each board.active as habit (habit.id)}
									<HabitRow {habit} />
								{/each}
							</ul>
						{:else if board.loading}
							<div class="mt-1"><Skeleton /></div>
						{:else}
							<Empty text="Nothing to track yet." href="/habits" action="Add a habit" />
						{/if}
					</div>
				{/key}
			</div>
		</Card>
	</div>

	{#each board.plans as plan (plan.period)}
		<PlanCard {plan} />
	{/each}

	{#if board.plans.length === 0 && !board.loading}
		<Empty text="No plans yet." href="/plans" action="Create one" />
	{/if}
</div>
