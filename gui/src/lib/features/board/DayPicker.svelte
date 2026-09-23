<script lang="ts">
	import { ChevronLeft, ChevronRight } from '@lucide/svelte';
	import { dayNumber, fullLabel, isFuture, today, weekdayLabel } from '$lib/domain/date';
	import { board } from '$lib/state/board.svelte';

	const now = today();
	const MONTH = new Intl.DateTimeFormat(undefined, { month: 'long', year: 'numeric' });

	const label = $derived(MONTH.format(new Date(`${board.week[3]}T00:00:00`)));
	const atCurrentWeek = $derived(board.week.includes(now));
</script>

<div class="mb-2 flex items-center justify-between">
	<button
		onclick={() => board.shiftWeek(-1)}
		aria-label="Previous week"
		class="grid size-8 place-items-center rounded-full text-muted transition hover:bg-sunken
		       hover:text-ink active:scale-90"
	>
		<ChevronLeft size={17} strokeWidth={2} />
	</button>

	<span class="text-xs font-medium capitalize">{label}</span>

	<button
		onclick={() => board.shiftWeek(1)}
		disabled={atCurrentWeek}
		aria-label="Next week"
		class="grid size-8 place-items-center rounded-full text-muted transition hover:bg-sunken
		       hover:text-ink active:scale-90 disabled:pointer-events-none disabled:opacity-25"
	>
		<ChevronRight size={17} strokeWidth={2} />
	</button>
</div>

<div class="grid grid-cols-7 gap-1">
	{#each board.week as day (day)}
		{@const selected = day === board.selectedDay}
		{@const activity = board.activityOn(day)}
		<button
			onclick={() => board.selectDay(day)}
			disabled={isFuture(day)}
			aria-pressed={selected}
			aria-label={fullLabel(day)}
			class="group flex flex-col items-center gap-1.5 rounded-2xl py-1 transition
			       disabled:pointer-events-none disabled:opacity-30"
		>
			<span
				class="text-[10px] font-medium tracking-wide uppercase transition-colors"
				class:text-accent={day === now && !selected}
				class:text-muted={day !== now || selected}
			>
				{weekdayLabel(day)}
			</span>

			<span
				class="grid size-9 place-items-center rounded-full text-[15px] font-semibold tabular-nums
				       transition-all duration-200
				       {selected
					? 'bg-accent text-white'
					: day === now
						? 'bg-accent-soft text-accent'
						: 'text-ink group-hover:bg-sunken'}"
			>
				{dayNumber(day)}
			</span>

			<span class="flex h-1 items-center gap-0.5">
				{#each Array.from({ length: Math.min(activity, 3) }, (_, i) => i) as index (index)}
					<span
						class="size-1 rounded-full"
						style:background-color={selected ? 'var(--color-accent)' : 'var(--color-muted)'}
					></span>
				{/each}
			</span>
		</button>
	{/each}
</div>
