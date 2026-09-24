<script lang="ts">
	import { Flame } from '@lucide/svelte';
	import Card from '$lib/ui/Card.svelte';
	import LevelBadge from './LevelBadge.svelte';
	import HabitIcons from './HabitIcons.svelte';
	import QuotaBar from './QuotaBar.svelte';
	import { formatRange } from '$lib/domain/date';
	import type { Plan, RequirementProgress } from '$lib/api/types';

	let { plan }: { plan: Plan } = $props();

	/** Few enough steps to count at a glance stay segmented; the rest read as one bar. */
	const segmentsFor = (item: RequirementProgress) =>
		item.quota <= 10 && item.habits.every((habit) => habit.tracking === 'binary') ? item.quota : 1;

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
							<p
								class="text-xs font-medium tabular-nums"
								class:text-muted={!solo.met}
								style:color={solo.met ? colorOf(solo) : undefined}
							>
								{solo.done} / {solo.quota}
							</p>
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
