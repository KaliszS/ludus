<script lang="ts">
	import { onMount } from 'svelte';
	import { plansApi } from '$lib/api/endpoints';
	import type { PlanEra, PlanVersions, RequirementVersion } from '$lib/api/types';
	import { formatRange, monthDay } from '$lib/domain/date';
	import { quotaUnit } from '$lib/domain/plan';
	import Icon from '$lib/ui/Icon.svelte';
	import TierMark from './TierMark.svelte';
	import { toast } from '$lib/state/toast.svelte';

	let { planId }: { planId: string } = $props();

	let history = $state<PlanVersions | null>(null);
	const eras = $derived(history?.eras ?? null);
	const archivedOn = $derived(history?.plan.archived_on ?? null);

	onMount(async () => {
		history = (await toast.guard(() => plansApi.versions(planId))) ?? null;
	});

	const span = (era: PlanEra) =>
		era.to ? formatRange(era.from, era.to) : `since ${monthDay(era.from)}`;

	/** A version that began where this stretch begins is what changed here. The
	 *  oldest stretch is where the plan started, so nothing in it counts as a change. */
	const isNew = (era: PlanEra, version: RequirementVersion, index: number) =>
		index < (eras?.length ?? 0) - 1 && version.valid_from === era.from;

	const tiers = $derived(history?.plan.tiers ?? []);
	const tiered = $derived(tiers.length > 1);

	const unit = (version: RequirementVersion) => {
		const top = version.quotas.at(-1)?.quota ?? 0;
		return quotaUnit(version.measure, top, version.habits) ?? 'total';
	};
</script>

{#if archivedOn}
	<p class="mb-3 text-xs text-muted">
		Archived {monthDay(archivedOn)}: it stopped counting from that {history?.plan.period}.
	</p>
{/if}

{#if eras === null}
	<p class="text-xs text-muted">Loading history…</p>
{:else if eras.length <= 1}
	<p class="text-xs text-muted">
		{eras.length
			? `Unchanged since ${monthDay(eras[0].from)}.`
			: 'Nothing has been asked of it yet.'}
	</p>
{:else}
	<ol class="space-y-4">
		{#each eras as era, index (era.from)}
			<li class="relative pl-5">
				<!-- The rail ties the stretches into one timeline; the newest dot is filled. -->
				<span
					class="absolute top-1.5 left-0 size-2 rounded-full border border-line
					       {index === 0 ? 'border-accent bg-accent' : 'bg-surface'}"
				></span>
				{#if index < eras.length - 1}
					<span class="absolute top-4 bottom-[-1.25rem] left-[3.5px] w-px bg-line"></span>
				{/if}

				<p class="text-xs font-medium {index === 0 ? 'text-ink' : 'text-muted'}">
					{span(era)}{index === 0 && !era.to && !archivedOn ? ' · current' : ''}
				</p>

				{#if era.requirements.length === 0}
					<p class="mt-1 text-xs text-muted">Nothing asked.</p>
				{:else}
					<ul class="mt-1.5 space-y-1">
						{#each era.requirements as version (version.id)}
							<li class="flex items-center gap-2 text-sm">
								<span class="flex shrink-0 gap-0.5">
									{#each version.habits as habit (habit.id)}
										<span style:color={habit.color ?? 'var(--color-muted)'}>
											<Icon name={habit.icon} size={14} />
										</span>
									{/each}
								</span>
								<span class="min-w-0 flex-1 truncate">
									{version.name ?? version.habits.map((habit) => habit.name).join(' / ')}
								</span>
								{#if isNew(era, version, index)}
									<span
										class="shrink-0 rounded-full bg-accent-soft px-1.5 py-0.5 text-[10px] font-medium text-accent"
										title="This version begins here"
									>
										new
									</span>
								{/if}
								<!-- Retired tiers still show here: this is what those periods asked. -->
								<span class="flex shrink-0 items-center gap-1.5 text-xs text-muted tabular-nums">
									{#each version.quotas as quota (quota.tier_id)}
										{@const index = tiers.findIndex((tier) => tier.id === quota.tier_id)}
										<span class="flex items-center gap-0.5">
											{#if tiered && index >= 0}
												<TierMark tier={tiers[index]} {index} size={13} />
											{/if}
											{quota.quota}
										</span>
									{/each}
									{unit(version)}
								</span>
							</li>
						{/each}
					</ul>
				{/if}
			</li>
		{/each}
	</ol>
{/if}
