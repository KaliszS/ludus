<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Check, Flame } from '@lucide/svelte';
	import Card from '$lib/ui/Card.svelte';
	import PlanBadge from './PlanBadge.svelte';
	import HabitIcons from './HabitIcons.svelte';
	import Medal from '$lib/ui/Medal.svelte';
	import { formatRange } from '$lib/domain/date';
	import { METALS, medalAt, nextQuota, quotaUnit, tierLabel } from '$lib/domain/plan';
	import type { PeriodProgress, PlanProgress, RequirementProgress } from '$lib/api/types';

	let { period }: { period: PeriodProgress } = $props();

	const isWhole = (quota: number) => Number.isInteger(quota) && quota <= 10;

	/** Few enough whole ticks to count at a glance; anything else reads better as numbers. */
	const countable = (item: RequirementProgress) =>
		isWhole(top(item)) && item.habits.every((habit) => habit.tracking === 'binary');

	/** The lowest quota is what keeps the streak, so it is what the ring closes on. */
	const base = (item: RequirementProgress) => item.quotas[0]?.quota ?? 0;
	const top = (item: RequirementProgress) => item.quotas.at(-1)?.quota ?? 0;

	const ringSegments = (item: RequirementProgress) =>
		countable(item) && isWhole(base(item)) ? base(item) : 1;

	const labelFor = (item: RequirementProgress) =>
		item.name ?? (item.habits.length > 1 ? `Any of ${item.habits.length}` : item.habits[0]?.name);

	const tierOf = (plan: PlanProgress, id: string) => plan.tiers.find((tier) => tier.id === id);

	/** The tiers asked this period that carry a medal, and whether each is won. */
	const medalsOf = (plan: PlanProgress) =>
		plan.standing.flatMap((step) => {
			const tier = tierOf(plan, step.tier_id);
			return tier?.medal ? [{ tier, medal: tier.medal, reached: step.reached }] : [];
		});

	/** Where along the way to the top quota each medal tier sits. */
	const milestones = (plan: PlanProgress, item: RequirementProgress) =>
		item.quotas.flatMap((quota) => {
			const medal = tierOf(plan, quota.tier_id)?.medal;
			return medal ? [{ at: quota.quota, color: METALS[medal].mid }] : [];
		});
</script>

{#snippet streak(count: number)}
	{#if count > 0}
		<span
			class="flex shrink-0 items-center gap-1 text-sm font-semibold text-accent tabular-nums"
			title="{count} {period.period}s in a row"
		>
			<Flame size={15} strokeWidth={2.25} />
			{count}
		</span>
	{/if}
{/snippet}

{#snippet medalRow(plan: PlanProgress)}
	{@const won = medalsOf(plan)}
	{#if won.length}
		<span class="flex shrink-0 items-center gap-0.5">
			{#each won as step (step.tier.id)}
				<Medal medal={step.medal} earned={step.reached} size={18} title={tierLabel(step.tier, 0)} />
			{/each}
		</span>
	{/if}
{/snippet}

<!-- One mark per expected tick, in the completion green of the ring beside it: a
     tick says "done", not which habit did it. Ticked ones fill in with a small
     overshoot, one after another. Past five they drop a size and split into two even
     rows - left to wrap, ten would break 7 + 3 on a phone and 5 + 5 on a desktop.
     A tick that completes a medal tier has a dot of that metal beneath it while the
     tier is still ahead - the ticks' version of the bar's notch - and lets it go once
     ticked: the dots show only what is left to win, without ringing every tick. -->
{#snippet checks(plan: PlanProgress, item: RequirementProgress)}
	{@const color = 'var(--color-good)'}
	{@const count = top(item)}
	{@const marks = milestones(plan, item)}
	{@const size = count <= 5 ? 22 : 18}
	{@const columns = count <= 5 ? count : Math.ceil(count / 2)}
	{@const faint = `color-mix(in oklab, ${color} 15%, transparent)`}
	<span class="mt-1.5 flex items-center gap-2" role="img" aria-label="{item.done} of {count}">
		<span class="grid gap-[5px]" style:grid-template-columns="repeat({columns}, {size}px)">
			{#each Array.from({ length: count }, (_, index) => index) as index (index)}
				{@const ticked = index < item.done}
				{@const milestone = marks.find((mark) => mark.at === index + 1)}
				<span class="flex flex-col items-center gap-[3px]">
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
					<!-- Every tick keeps the slot once any has a dot, so the row stays level. -->
					{#if marks.length}
						<span
							class="size-[5px] rounded-full transition-opacity duration-500"
							style:background-color={milestone?.color ?? 'transparent'}
							style:opacity={milestone && !ticked ? 1 : 0}
						></span>
					{/if}
				</span>
			{/each}
		</span>
		{#if item.done > count}
			<span
				class="rounded-full px-1.5 py-0.5 text-[11px] leading-none font-semibold tabular-nums"
				style:background-color={faint}
				style:color
			>
				+{item.done - count}
			</span>
		{/if}
	</span>
{/snippet}

<!-- The amount counterpart of the ticks: the same green, as a slim bar that fills
     with a little overshoot, and the figures beside it - what is done in full, the
     next target and its unit quieter. The bar runs to the top tier, with a notch in
     each tier's metal where it is won - the top one closing the bar - that fades
     once the fill passes it. -->
{#snippet amount(plan: PlanProgress, item: RequirementProgress)}
	{@const target = nextQuota(item.quotas, item.done)}
	{@const unit = quotaUnit(item.measure, target, item.habits)}
	<span class="mt-2 flex items-center gap-2.5">
		<span class="relative h-1.5 min-w-10 flex-1">
			<span
				class="absolute inset-0 overflow-hidden rounded-full"
				style:background-color="color-mix(in oklab, var(--color-good) 15%, transparent)"
			>
				<span
					class="block h-full rounded-full bg-good transition-[width] duration-700
					       ease-[cubic-bezier(0.34,1.56,0.64,1)]"
					style:width="{Math.min(Math.max(item.done, 0) / top(item), 1) * 100}%"
					style:box-shadow="0 0 6px color-mix(in oklab, var(--color-good) 50%, transparent)"
				></span>
			</span>
			{#each milestones(plan, item) as mark (mark.at)}
				{@const at = (mark.at / top(item)) * 100}
				<!-- Shifted by its own position, so the last notch sits inside the bar's end. -->
				<span
					class="absolute -top-[3px] -bottom-[3px] w-[3px] rounded-full transition-opacity duration-500"
					style:left="{at}%"
					style:transform="translateX(-{at}%)"
					style:opacity={item.done >= mark.at ? 0 : 1}
					style:background-color={mark.color}
					style:box-shadow="0 0 0 1.5px var(--color-surface)"
				></span>
			{/each}
		</span>
		<span class="shrink-0 text-xs tabular-nums">
			<span class="font-semibold {item.met ? 'text-good' : 'text-ink'}">{item.done}</span>
			<span class="text-muted">/ {target}{unit ? ` ${unit}` : ''}</span>
		</span>
	</span>
{/snippet}

<!-- What a single plan shows in its tile and a larger plan in each of its rows: the
     habits, the label with medals beside it, and ticks or a bar beneath. -->
{#snippet requirement(plan: PlanProgress, item: RequirementProgress, medals: Snippet)}
	<HabitIcons habits={item.habits} active={item.done > 0} size={22} stacked />

	<div class="min-w-0 flex-1">
		<div class="flex items-center gap-2">
			<p
				class="min-w-0 truncate text-sm font-medium"
				title={item.habits.map((habit) => habit.name).join(' / ')}
			>
				{labelFor(item)}
			</p>
			{@render medals()}
		</div>
		{#if countable(item)}
			{@render checks(plan, item)}
		{:else}
			{@render amount(plan, item)}
		{/if}
	</div>
{/snippet}

<section>
	<!-- The period is context, so it sits above the cards as a label rather than
	     competing with the plan names inside them. -->
	<h2 class="mb-2 flex items-baseline gap-2 px-1 text-[11px] tracking-wide text-muted uppercase">
		{period.period}
		<span class="normal-case opacity-70">{formatRange(period.period_start, period.period_end)}</span
		>
	</h2>

	<!-- Single-requirement plans are small enough to pair up on a wide screen;
	     the rest keep the full width of the row. -->
	<div class="grid gap-3 sm:grid-cols-2">
		{#each period.plans as plan (plan.id)}
			{@const solo = plan.items.length === 1 ? plan.items[0] : null}
			<Card class={solo ? '' : 'sm:col-span-2'}>
				{#if solo}
					<!-- The plan name would only repeat the habit here, so the habit carries
					     the label and the ring carries the progress. -->
					<div class="flex items-center gap-3" title={plan.name}>
						{#snippet ladder()}{@render medalRow(plan)}{/snippet}
						{@render requirement(plan, solo, ladder)}

						{@render streak(plan.streak)}
						<PlanBadge
							segments={ringSegments(solo)}
							filled={(Math.max(solo.done, 0) / base(solo)) * ringSegments(solo)}
							size={40}
						/>
					</div>
				{:else}
					<div class="mb-3 flex items-center gap-2">
						<h3 class="min-w-0 truncate text-[15px] font-semibold capitalize">
							{plan.name}
						</h3>
						{@render medalRow(plan)}
						<span class="flex-1"></span>
						{@render streak(plan.streak)}
						<PlanBadge
							segments={plan.items.length}
							filled={plan.items.filter((item) => item.met).length}
						/>
					</div>

					{#if plan.items.length === 0}
						<p class="text-xs text-muted">No quotas.</p>
					{:else}
						<!-- Rows are tiles without the card: two abreast on a wide screen, and the
						     medal by each label is the highest tier that goal has reached. -->
						<ul class="grid gap-x-8 gap-y-4 sm:grid-cols-2">
							{#each plan.items as item (item.id)}
								{@const won = medalAt(plan.tiers, item.quotas, item.done)}
								<li class="flex items-center gap-3">
									{#snippet reached()}
										{#if won}<Medal medal={won} size={16} />{/if}
									{/snippet}
									{@render requirement(plan, item, reached)}
								</li>
							{/each}
						</ul>
					{/if}
				{/if}
			</Card>
		{/each}
	</div>
</section>
