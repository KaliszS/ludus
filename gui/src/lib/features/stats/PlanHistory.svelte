<script lang="ts">
	import Medal from '$lib/ui/Medal.svelte';
	import { MEDALS, METALS } from '$lib/domain/plan';
	import type { PeriodOutcome, PlanOutcome } from '$lib/api/types';

	interface Props {
		outcomes: PeriodOutcome[];
	}
	let { outcomes }: Props = $props();

	/** One row per plan that appears in any period, so an archived plan keeps its
	 *  timeline even though the current period no longer has it. Archived ones go last. */
	const plans = $derived.by(() => {
		const seen = new Map<string, { id: string; name: string; archived: boolean }>();
		for (const outcome of outcomes) {
			for (const plan of outcome.plans) {
				seen.set(plan.id, { id: plan.id, name: plan.name, archived: plan.archived });
			}
		}
		return [...seen.values()]
			.sort((a, b) => Number(a.archived) - Number(b.archived))
			.map((plan) => ({
				...plan,
				cells: outcomes.map((outcome) => outcome.plans.find((item) => item.id === plan.id))
			}))
			.map((plan) => ({
				...plan,
				tally: MEDALS.map((medal) => ({
					medal,
					count: plan.cells.filter((cell) => cell?.medals.includes(medal)).length
				})).filter((entry) => entry.count > 0)
			}));
	});

	/** A period wears its best medal; a plain pass is green, a miss the accent. */
	function fill(cell: PlanOutcome | undefined) {
		const best = MEDALS.findLast((medal) => cell?.medals.includes(medal));
		if (best) return METALS[best].mid;
		return cell?.met
			? 'var(--color-good)'
			: 'color-mix(in oklab, var(--color-accent) 70%, transparent)';
	}

	const met = $derived(outcomes.filter((outcome) => outcome.plans.some((plan) => plan.met)).length);
</script>

<div class="space-y-3">
	{#each plans as plan (plan.id)}
		<div>
			<div class="mb-1.5 flex items-baseline justify-between">
				<span class="text-xs font-semibold tracking-wide uppercase">
					{plan.name}
					{#if plan.archived}
						<span class="ml-1 font-normal tracking-normal text-muted normal-case">archived</span>
					{/if}
				</span>
				<span class="flex items-center gap-2.5 text-[11px] text-muted tabular-nums">
					{#each plan.tally as entry (entry.medal)}
						<span class="flex items-center gap-0.5" title="{entry.count}× {entry.medal}">
							<Medal medal={entry.medal} size={14} />
							{entry.count}
						</span>
					{/each}
					<!-- Only periods the plan existed in: weeks before it began are not misses. -->
					<span>
						{plan.cells.filter((cell) => cell?.met).length}/{plan.cells.filter(Boolean).length}
					</span>
				</span>
			</div>
			<div class="flex gap-1">
				{#each plan.cells as cell, index (index)}
					{@const ratio = cell && cell.total > 0 ? cell.reached / cell.total : 0}
					<span
						class="h-6 flex-1 overflow-hidden rounded"
						style:background-color="var(--color-sunken)"
						title={cell ? [`${cell.reached}/${cell.total}`, ...cell.medals].join(' · ') : ''}
					>
						<span
							class="block h-full rounded transition-[height] duration-500"
							style:height="{ratio * 100}%"
							style:margin-top="{(1 - ratio) * 100}%"
							style:background-color={fill(cell)}
						></span>
					</span>
				{/each}
			</div>
		</div>
	{/each}

	{#if plans.length}
		<p class="text-[11px] text-muted">
			{met} of {outcomes.length} periods with at least one plan complete
		</p>
	{/if}
</div>
