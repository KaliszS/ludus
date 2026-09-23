<script lang="ts">
	import { fullLabel, shift, weekOf } from '$lib/domain/date';
	import { dailyActivity } from '$lib/domain/stats';
	import type { UserCheckin } from '$lib/api/types';

	interface Props {
		rows: UserCheckin[];
		from: string;
		to: string;
	}
	let { rows, from, to }: Props = $props();

	const counts = $derived(dailyActivity(rows));
	const peak = $derived(Math.max(1, ...counts.values()));

	/** Columns are weeks, so the grid starts on the Monday on or before `from`. */
	const columns = $derived.by(() => {
		const weeks: string[][] = [];
		for (let start = weekOf(from)[0]; start <= to; start = shift(start, 7)) {
			weeks.push(weekOf(start));
		}
		return weeks;
	});

	const MONTH = new Intl.DateTimeFormat(undefined, { month: 'short' });
	const monthOf = (day: string) => new Date(`${day}T00:00:00`).getMonth();

	/** Only label a column when its month differs from the previous one. */
	const monthMarks = $derived(
		columns.map((week, index) =>
			index === 0 || monthOf(week[0]) !== monthOf(columns[index - 1][0])
				? MONTH.format(new Date(`${week[0]}T00:00:00`))
				: ''
		)
	);

	const shade = (day: string) => {
		const value = counts.get(day) ?? 0;
		return value === 0 ? 0 : Math.min(1, 0.35 + (value / peak) * 0.65);
	};
</script>

<div class="[scrollbar-width:none] overflow-x-auto [&::-webkit-scrollbar]:hidden">
	<div class="flex gap-[3px]">
		<div class="mt-[15px] mr-1 flex flex-col gap-[3px]">
			{#each ['M', '', 'W', '', 'F', '', 'S'] as label, index (index)}
				<span class="h-[11px] text-[9px] leading-[11px] text-muted">{label}</span>
			{/each}
		</div>

		{#each columns as week, index (week[0])}
			<div class="flex flex-col gap-[3px]">
				<span class="h-3 text-[9px] leading-3 text-muted">{monthMarks[index]}</span>
				{#each week as day (day)}
					{@const level = shade(day)}
					<span
						title="{fullLabel(day)} · {counts.get(day) ?? 0}"
						class="size-[11px] rounded-[3px] transition-colors"
						style:background-color={day > to || day < from
							? 'transparent'
							: level === 0
								? 'var(--color-line)'
								: `color-mix(in oklab, var(--color-accent) ${level * 100}%, transparent)`}
					></span>
				{/each}
			</div>
		{/each}
	</div>
</div>
