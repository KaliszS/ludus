<script lang="ts">
	import type { PeriodOutcome } from '$lib/api/types';

	interface Props {
		outcomes: PeriodOutcome[];
	}
	let { outcomes }: Props = $props();

	/** One row per level that appears in any period, so an archived level keeps its
	 *  timeline even though the current period no longer has it. Archived ones go last. */
	const levels = $derived.by(() => {
		const seen = new Map<string, { id: string; name: string; archived: boolean }>();
		for (const outcome of outcomes) {
			for (const level of outcome.levels) {
				seen.set(level.id, { id: level.id, name: level.name, archived: level.archived });
			}
		}
		return [...seen.values()]
			.sort((a, b) => Number(a.archived) - Number(b.archived))
			.map((level) => ({
				...level,
				cells: outcomes.map((outcome) => outcome.levels.find((item) => item.id === level.id))
			}));
	});

	const met = $derived(
		outcomes.filter((outcome) => outcome.levels.some((level) => level.met)).length
	);
</script>

<div class="space-y-3">
	{#each levels as level (level.id)}
		<div>
			<div class="mb-1.5 flex items-baseline justify-between">
				<span class="text-xs font-semibold tracking-wide uppercase">
					{level.name}
					{#if level.archived}
						<span class="ml-1 font-normal tracking-normal text-muted normal-case">archived</span>
					{/if}
				</span>
				<!-- Only periods the level existed in: weeks before it began are not misses. -->
				<span class="text-[11px] text-muted tabular-nums">
					{level.cells.filter((cell) => cell?.met).length}/{level.cells.filter(Boolean).length}
				</span>
			</div>
			<div class="flex gap-1">
				{#each level.cells as cell, index (index)}
					{@const ratio = cell && cell.total > 0 ? cell.reached / cell.total : 0}
					<span
						class="h-6 flex-1 overflow-hidden rounded"
						style:background-color="var(--color-sunken)"
						title={cell ? `${cell.reached}/${cell.total}` : ''}
					>
						<span
							class="block h-full rounded transition-[height] duration-500"
							style:height="{ratio * 100}%"
							style:margin-top="{(1 - ratio) * 100}%"
							style:background-color={cell?.met
								? 'var(--color-good)'
								: 'color-mix(in oklab, var(--color-accent) 70%, transparent)'}
						></span>
					</span>
				{/each}
			</div>
		</div>
	{/each}

	{#if levels.length}
		<p class="text-[11px] text-muted">
			{met} of {outcomes.length} periods with at least one level complete
		</p>
	{/if}
</div>
