<script lang="ts">
	import { Check, Flame } from '@lucide/svelte';
	import Card from '$lib/ui/Card.svelte';
	import LevelBadge from './LevelBadge.svelte';
	import HabitIcons from './HabitIcons.svelte';
	import QuotaBar from './QuotaBar.svelte';
	import { formatRange } from '$lib/domain/date';
	import { quotaUnit } from '$lib/domain/plan';
	import type { Plan, RequirementProgress } from '$lib/api/types';

	let { plan }: { plan: Plan } = $props();

	/** Few enough whole ticks to count at a glance; anything else reads better as numbers. */
	const countable = (item: RequirementProgress) =>
		Number.isInteger(item.quota) &&
		item.quota <= 10 &&
		item.habits.every((habit) => habit.tracking === 'binary');

	const segmentsFor = (item: RequirementProgress) => (countable(item) ? item.quota : 1);

	const labelFor = (item: RequirementProgress) =>
		item.name ?? (item.habits.length > 1 ? `Any of ${item.habits.length}` : item.habits[0]?.name);

	const colorOf = (item: RequirementProgress) => item.habits[0]?.color ?? 'var(--color-accent)';

	const ratioOf = (item: RequirementProgress) => Math.max(item.done, 0) / item.quota;
</script>

{#snippet streak(count: number)}
	{#if count > 0}
		<span
			class="flex shrink-0 items-center gap-1 text-sm font-semibold text-accent tabular-nums"
			title="{count} {plan.period}s in a row"
		>
			<Flame size={15} strokeWidth={2.25} />
			{count}
		</span>
	{/if}
{/snippet}

<!-- One mark per expected tick, in the completion green of the ring beside it: a
     tick says "done", not which habit did it. Ticked ones fill in with a small
     overshoot, one after another. Past five they drop a size and split into two even
     rows - left to wrap, ten would break 7 + 3 on a phone and 5 + 5 on a desktop. -->
{#snippet checks(item: RequirementProgress)}
	{@const color = 'var(--color-good)'}
	{@const size = item.quota <= 5 ? 22 : 18}
	{@const columns = item.quota <= 5 ? item.quota : Math.ceil(item.quota / 2)}
	{@const faint = `color-mix(in oklab, ${color} 15%, transparent)`}
	<span class="mt-1.5 flex items-center gap-2" role="img" aria-label="{item.done} of {item.quota}">
		<span class="grid gap-[5px]" style:grid-template-columns="repeat({columns}, {size}px)">
			{#each Array.from({ length: item.quota }, (_, index) => index) as index (index)}
				{@const ticked = index < item.done}
				<span
					class="grid shrink-0 place-items-center rounded-full
				       transition-[transform,background-color,box-shadow,color] duration-500
				       ease-[cubic-bezier(0.34,1.56,0.64,1)]"
					style:width="{size}px"
					style:height="{size}px"
					style:transition-delay="{Math.min(index, 9) * 45}ms"
					style:transform={ticked ? 'scale(1)' : 'scale(0.86)'}
					style:background-color={ticked ? color : faint}
					style:color={ticked
						? 'var(--color-surface)'
						: `color-mix(in oklab, ${color} 45%, transparent)`}
					style:box-shadow={ticked
						? `0 1px 6px color-mix(in oklab, ${color} 40%, transparent)`
						: 'none'}
				>
					<Check size={Math.round(size * 0.58)} strokeWidth={3.25} />
				</span>
			{/each}
		</span>
		{#if item.done > item.quota}
			<span
				class="rounded-full px-1.5 py-0.5 text-[11px] leading-none font-semibold tabular-nums"
				style:background-color={faint}
				style:color
			>
				+{item.done - item.quota}
			</span>
		{/if}
	</span>
{/snippet}

<!-- The amount counterpart of the ticks: the same green, as a slim bar that fills
     with a little overshoot, and the figures beside it - what is done in full, the
     target and its unit quieter. -->
{#snippet amount(item: RequirementProgress)}
	{@const unit = quotaUnit(item.measure, item.quota, item.habits)}
	<span class="mt-2 flex items-center gap-2.5">
		<span
			class="h-1.5 min-w-10 flex-1 overflow-hidden rounded-full"
			style:background-color="color-mix(in oklab, var(--color-good) 15%, transparent)"
		>
			<span
				class="block h-full rounded-full bg-good transition-[width] duration-700
				       ease-[cubic-bezier(0.34,1.56,0.64,1)]"
				style:width="{Math.min(ratioOf(item), 1) * 100}%"
				style:box-shadow="0 0 6px color-mix(in oklab, var(--color-good) 50%, transparent)"
			></span>
		</span>
		<span class="shrink-0 text-xs tabular-nums">
			<span class="font-semibold {item.met ? 'text-good' : 'text-ink'}">{item.done}</span>
			<span class="text-muted">/ {item.quota}{unit ? ` ${unit}` : ''}</span>
		</span>
	</span>
{/snippet}

<section>
	<!-- The period is context, so it sits above the cards as a label rather than
	     competing with the level names inside them. -->
	<h2 class="mb-2 flex items-baseline gap-2 px-1 text-[11px] tracking-wide text-muted uppercase">
		{plan.period}
		<span class="normal-case opacity-70">{formatRange(plan.period_start, plan.period_end)}</span>
	</h2>

	<!-- Single-requirement levels are small enough to pair up on a wide screen;
	     the rest keep the full width of the row. -->
	<div class="grid gap-3 sm:grid-cols-2">
		{#each plan.levels as level (level.id)}
			{@const solo = level.items.length === 1 ? level.items[0] : null}
			<Card class={solo ? '' : 'sm:col-span-2'}>
				{#if solo}
					<!-- The level name would only repeat the habit here, so the habit carries
					     the label and the ring carries the progress. -->
					<div class="flex items-center gap-3" title={level.name}>
						<HabitIcons habits={solo.habits} active={solo.done > 0} size={22} stacked />

						<div class="min-w-0 flex-1">
							<p
								class="truncate text-sm font-medium"
								title={solo.habits.map((h) => h.name).join(' / ')}
							>
								{labelFor(solo)}
							</p>
							{#if countable(solo)}
								{@render checks(solo)}
							{:else}
								{@render amount(solo)}
							{/if}
						</div>

						{@render streak(level.streak)}
						<LevelBadge
							segments={segmentsFor(solo)}
							filled={ratioOf(solo) * segmentsFor(solo)}
							size={40}
						/>
					</div>
				{:else}
					<div class="mb-3 flex items-center gap-2">
						<h3 class="min-w-0 flex-1 truncate text-[15px] font-semibold capitalize">
							{level.name}
						</h3>
						{@render streak(level.streak)}
						<LevelBadge
							segments={level.items.length}
							filled={level.items.filter((item) => item.met).length}
						/>
					</div>

					{#if level.items.length === 0}
						<p class="text-xs text-muted">No quotas.</p>
					{:else}
						<ul class="space-y-3">
							{#each level.items as item (item.id)}
								<li class="flex items-center gap-2.5">
									<HabitIcons habits={item.habits} active={item.done > 0} />

									<span
										class="min-w-0 flex-1 truncate text-sm"
										title={item.habits.map((habit) => habit.name).join(' / ')}
									>
										{labelFor(item)}
									</span>

									<div class="w-20 shrink-0 sm:w-28">
										<QuotaBar
											done={item.done}
											quota={item.quota}
											segments={segmentsFor(item)}
											color={String(colorOf(item))}
										/>
									</div>

									<span
										class="w-12 shrink-0 text-right text-xs font-medium tabular-nums"
										class:text-muted={!item.met}
										style:color={item.met ? colorOf(item) : undefined}
									>
										{item.done}/{item.quota}
									</span>
								</li>
							{/each}
						</ul>
					{/if}
				{/if}
			</Card>
		{/each}
	</div>
</section>
