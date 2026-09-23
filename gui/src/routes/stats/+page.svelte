<script lang="ts">
	import HabitStatRow from '$lib/features/stats/HabitStatRow.svelte';
	import Heatmap from '$lib/features/stats/Heatmap.svelte';
	import LevelHistory from '$lib/features/stats/LevelHistory.svelte';
	import { today } from '$lib/domain/date';
	import { stats } from '$lib/state/stats.svelte';
	import Card from '$lib/ui/Card.svelte';
	import Empty from '$lib/ui/Empty.svelte';
	import Segmented from '$lib/ui/Segmented.svelte';
	import Skeleton from '$lib/ui/Skeleton.svelte';

	const SPANS = { '4w': 28, '13w': 91, '26w': 182, '1y': 364 } as const;
	type SpanKey = keyof typeof SPANS;

	let span = $state<SpanKey>('13w');

	const activeDays = $derived(new Set(stats.visible.map((row) => row.day)).size);

	$effect(() => {
		stats.load();
	});
</script>

<div class="space-y-4">
	<Card>
		<div class="mb-4 flex items-baseline justify-between gap-3">
			<h2 class="text-sm font-semibold">Activity</h2>
			<Segmented
				options={Object.keys(SPANS) as SpanKey[]}
				value={span}
				onchange={(next) => {
					span = next;
					stats.setSpan(SPANS[next]);
				}}
			/>
		</div>

		<Heatmap rows={stats.visible} from={stats.from} to={today()} />

		<p class="mt-3 text-[11px] text-muted tabular-nums">
			{activeDays} active days of {stats.spanDays} · {stats.visible.length} check-ins
		</p>
	</Card>

	<Card>
		<h2 class="mb-4 text-sm font-semibold">Habits</h2>
		{#if stats.perHabit.length}
			<ul class="divide-y divide-line">
				{#each stats.perHabit as stat (stat.habit.id)}
					<HabitStatRow {stat} />
				{/each}
			</ul>
		{:else if stats.loading}
			<Skeleton />
		{:else}
			<Empty text="Nothing tracked yet." href="/habits" action="Add a habit" />
		{/if}
	</Card>

	{#each stats.history as entry (entry.period)}
		<Card>
			<h2 class="mb-4 text-sm font-semibold capitalize">{entry.period} levels</h2>
			<LevelHistory outcomes={entry.outcomes} />
		</Card>
	{/each}

	{#if stats.history.length === 0 && !stats.loading}
		<Empty text="No levels to track over time." href="/plans" action="Create one" />
	{/if}
</div>
