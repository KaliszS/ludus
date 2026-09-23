<script lang="ts">
	import { Flame } from '@lucide/svelte';
	import Card from '$lib/ui/Card.svelte';
	import LevelBadge from './LevelBadge.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import { formatRange } from '$lib/domain/date';
	import type { LevelProgress, Plan } from '$lib/api/types';
	import QuotaBar from './QuotaBar.svelte';

	let { plan }: { plan: Plan } = $props();

	/** Few enough steps to count at a glance stay segmented; the rest read as one bar. */
	const segmentsFor = (item: LevelProgress['items'][number]) =>
		item.quota <= 10 && item.habits.every((habit) => habit.tracking === 'binary') ? item.quota : 1;
</script>

<section>
	<!-- The period is context, so it sits above the card as a label rather than
	     competing with the level names inside it. -->
	<h2 class="mb-2 flex items-baseline gap-2 px-1 text-[11px] tracking-wide text-muted uppercase">
		{plan.period}
		<span class="normal-case opacity-70">{formatRange(plan.period_start, plan.period_end)}</span>
	</h2>

	<Card>
		<div class="divide-y divide-line">
			{#each plan.levels as level (level.id)}
				<div class="py-4 first:pt-0 last:pb-0">
					<div class="mb-3 flex items-center gap-2">
						<h3 class="min-w-0 flex-1 truncate text-[15px] font-semibold capitalize">
							{level.name}
						</h3>

						<!-- Streak and badge both describe the level's state, so they travel together. -->
						{#if level.streak > 0}
							<span
								class="flex shrink-0 items-center gap-1 text-sm font-semibold text-accent tabular-nums"
								title="{level.streak} {plan.period}s in a row"
							>
								<Flame size={15} strokeWidth={2.25} />
								{level.streak}
							</span>
						{/if}

						<LevelBadge
							reached={level.items.filter((item) => item.met).length}
							total={level.items.length}
						/>
					</div>

					{#if level.items.length === 0}
						<p class="text-xs text-muted">No quotas.</p>
					{:else}
						<ul class="space-y-3">
							{#each level.items as item (item.id)}
								{@const color = item.habits[0]?.color ?? 'var(--color-accent)'}
								{@const names = item.habits.map((habit) => habit.name).join(' / ')}
								{@const spacing =
									item.habits.length <= 2
										? 3
										: Math.max(-10, (44 - item.habits.length * 18) / (item.habits.length - 1))}
								<li class="flex items-center gap-2.5">
									<!-- Fixed slot, or a bigger group would shift its bar out of line.
									     Up to two sit side by side; beyond that they overlap to fit, each on
									     an opaque disc so the top one masks the one under it. -->
									<span class="flex w-11 shrink-0 items-center overflow-hidden">
										{#each item.habits as habit, index (habit.id)}
											<span
												class="grid size-[18px] shrink-0 place-items-center rounded-full bg-surface
												       transition-colors"
												style:margin-left={index === 0 ? '0' : `${spacing}px`}
												style:color={item.done > 0 ? (habit.color ?? color) : 'var(--color-muted)'}
											>
												<Icon name={habit.icon} size={15} />
											</span>
										{/each}
									</span>

									<span class="min-w-0 flex-1 truncate text-sm" title={names}>
										{item.name ?? (item.habits.length > 1 ? `Any of ${item.habits.length}` : names)}
									</span>

									<div class="w-20 shrink-0 sm:w-28">
										<QuotaBar
											done={item.done}
											quota={item.quota}
											segments={segmentsFor(item)}
											color={String(color)}
										/>
									</div>

									<span
										class="w-12 shrink-0 text-right text-xs font-medium tabular-nums"
										class:text-muted={!item.met}
										style:color={item.met ? color : undefined}
									>
										{item.done}/{item.quota}
									</span>
								</li>
							{/each}
						</ul>
					{/if}
				</div>
			{/each}
		</div>
	</Card>
</section>
