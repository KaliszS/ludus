<script lang="ts">
	import type { PeriodOutcome } from '$lib/api/types';

	interface Props {
		outcomes: PeriodOutcome[];
	}
	let { outcomes }: Props = $props();

	/** Level identity is stable across periods, so one row per level reads as a timeline. */
	const levels = $derived(
		outcomes.at(-1)?.levels.map((level) => ({
			id: level.id,
			name: level.name,
			cells: outcomes.map((outcome) => outcome.levels.find((item) => item.id === level.id))
		})) ?? []
	);

	const met = $derived(
		outcomes.filter((outcome) => outcome.levels.some((level) => level.met)).length
	);
</script>

<div class="space-y-3">
	{#each levels as level (level.id)}
		<div>
			<div class="mb-1.5 flex items-baseline justify-between">
				<span class="text-xs font-semibold tracking-wide uppercase">{level.name}</span>
				<span class="text-[11px] text-muted tabular-nums">
					{level.cells.filter((cell) => cell?.met).length}/{level.cells.length}
				</span>
			</div>
			<div class="flex gap-1">
				{#each level.cells as cell, index (index)}
					{@const ratio = cell && cell.total > 0 ? cell.reached / cell.total : 0}
					<span
						class="h-6 flex-1 overflow-hidden rounded"
						style:background-color="var(--color-line)"
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
