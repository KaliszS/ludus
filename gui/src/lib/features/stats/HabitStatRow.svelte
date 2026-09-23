<script lang="ts">
	import { Flame } from '@lucide/svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import { tint } from '$lib/domain/palette';
	import type { HabitStat } from '$lib/domain/stats';

	let { stat }: { stat: HabitStat } = $props();

	const accent = $derived(stat.habit.color ?? 'var(--color-accent)');
</script>

<li class="flex items-center gap-3 py-2.5">
	<span
		class="grid size-9 shrink-0 place-items-center rounded-xl"
		style:background-color={tint(stat.habit.color, '24')}
		style:color={accent}
	>
		<Icon name={stat.habit.icon} size={17} />
	</span>

	<div class="min-w-0 flex-1">
		<div class="truncate text-sm font-medium">{stat.habit.name}</div>
		<div class="text-[11px] text-muted">
			{stat.days} days · {stat.perWeek}/week
			{#if stat.habit.tracking === 'quantity'}
				· {stat.total}{stat.habit.unit ? ` ${stat.habit.unit}` : ''}
			{/if}
		</div>
	</div>

	<div class="shrink-0 text-right">
		<div
			class="flex items-center justify-end gap-1 text-sm font-semibold tabular-nums"
			style:color={stat.weekStreak > 0 ? accent : 'var(--color-muted)'}
		>
			<Flame size={14} strokeWidth={2} />
			{stat.weekStreak} wk
		</div>
		<div class="text-[11px] text-muted tabular-nums">
			{stat.dayStreak} d · best {stat.bestDayStreak}
		</div>
	</div>
</li>
