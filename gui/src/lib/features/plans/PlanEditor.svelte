<script lang="ts">
	import { Archive, History, Plus, X } from '@lucide/svelte';
	import PlanVersions from './PlanVersions.svelte';
	import Button from '$lib/ui/Button.svelte';
	import Card from '$lib/ui/Card.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import IconButton from '$lib/ui/IconButton.svelte';
	import Segmented from '$lib/ui/Segmented.svelte';
	import TextField from '$lib/ui/TextField.svelte';
	import TierMark from './TierMark.svelte';
	import { tint } from '$lib/domain/palette';
	import { MEDALS, liveTiers, tierLabel } from '$lib/domain/plan';
	import { habits } from '$lib/state/habits.svelte';
	import { plans } from '$lib/state/plans.svelte';
	import type { Measure, Medal, Period, Plan, Requirement, Tier } from '$lib/api/types';

	const PERIODS = ['day', 'week', 'month', 'quarter', 'year'] as const satisfies readonly Period[];

	let { plan }: { plan: Plan } = $props();

	const tiers = $derived(liveTiers(plan));
	const tiered = $derived(tiers.length > 1);

	let picked = $state<string[]>([]);
	/** tier id -> the quota typed for it; the lowest tier starts at 1, the rest empty. */
	let drafts = $state<Record<string, string>>({});
	let groupName = $state('');
	/** Bumped when a quota is refused, so its fields re-render with the stored values. */
	let refused = $state<Record<string, number>>({});
	let adding = $state(false);
	/** Requirement whose member list is open for editing. */
	let editing = $state<string | null>(null);
	let draft = $state<string[]>([]);
	let showHistory = $state(false);

	const byId = $derived(new Map(habits.active.map((habit) => [habit.id, habit])));
	const hasQuantity = (ids: string[]) => ids.some((id) => byId.get(id)?.tracking === 'quantity');

	const flip = (list: string[], id: string) =>
		list.includes(id) ? list.filter((value) => value !== id) : [...list, id];

	const draftFor = (tier: Tier, index: number) => drafts[tier.id] ?? (index === 0 ? '1' : '');

	/** The medal after the highest one in use, so adding tiers climbs bronze, silver, gold. */
	function nextMedal(): Medal | null {
		const used = tiers.map((tier) => (tier.medal ? MEDALS.indexOf(tier.medal) : -1));
		return MEDALS[Math.max(...used) + 1] ?? null;
	}

	/** The first extra tier makes the plain plan its bronze floor, unless it has a medal already. */
	async function addTier() {
		if (!tiered && !tiers[0].medal) await plans.setTierMedal(tiers[0].id, 'bronze');
		await plans.addTier(plan.id, nextMedal());
	}

	function cycleMedal(tier: Tier) {
		const index = tier.medal ? MEDALS.indexOf(tier.medal) : -1;
		plans.setTierMedal(tier.id, MEDALS[index + 1] ?? null);
	}

	async function add() {
		if (picked.length === 0) return;
		// A group counts days, because summing "45 words" with "1 session" is nonsense.
		const measure: Measure = picked.length > 1 ? 'occurrences' : 'amount';
		const quotas = tiers
			.map((tier, index) => ({ tier_id: tier.id, quota: Number(draftFor(tier, index)) }))
			.filter((quota) => quota.quota > 0);
		await plans.addRequirement(plan.id, picked, quotas, measure, groupName || null);
		picked = [];
		drafts = {};
		groupName = '';
		adding = false;
	}

	async function setQuota(requirement: Requirement, tier: Tier, raw: string) {
		if (!(await plans.setQuota(requirement, tier.id, raw))) {
			refused[requirement.id] = (refused[requirement.id] ?? 0) + 1;
		}
	}

	const quotaAt = (requirement: Requirement, tier: Tier) =>
		requirement.quotas.find((quota) => quota.tier_id === tier.id)?.quota;

	/** Archived members are not offered as chips, so carry them over untouched
	 *  instead of dropping them the moment someone edits the visible ones. */
	async function saveMembers(id: string, original: string[]) {
		const hidden = original.filter((habitId) => !byId.has(habitId));
		await plans.setMembers(id, [...draft, ...hidden]);
		editing = null;
	}
</script>

<Card>
	<header class="mb-3 flex items-center gap-2">
		<TextField
			class="min-w-0 flex-1 px-2 py-1.5 text-[15px] font-semibold"
			value={plan.name}
			onchange={(raw) => plans.rename(plan.id, raw)}
		/>
		<IconButton
			icon={History}
			label={showHistory ? 'Hide history' : 'Show history'}
			onclick={() => (showHistory = !showHistory)}
		/>
		<IconButton
			icon={Archive}
			label="Archive plan (keeps its history)"
			onclick={() => plans.setArchived(plan.id, true)}
		/>
	</header>

	<div class="mb-3">
		<Segmented
			options={PERIODS}
			value={plan.period}
			onchange={(period) => plans.setPeriod(plan.id, period)}
		/>
		<p class="mt-2 px-1 text-[11px] text-muted">
			A new quota, measure or set of habits applies from this {plan.period} on. Earlier
			{plan.period}s keep the result they had.
		</p>
	</div>

	<!-- Tiers climb left to right. The lowest keeps the streak; a medal and a name are
	     both optional labels, and changing either relabels the past as well. -->
	<div class="mb-3 flex flex-wrap items-center gap-1.5">
		{#each tiers as tier, index (tier.id)}
			<div
				class="flex items-center gap-1 rounded-full py-0.5 pr-1 pl-0.5 ring-1 ring-line ring-inset"
			>
				<button
					onclick={() => cycleMedal(tier)}
					class="grid size-7 place-items-center rounded-full transition hover:bg-sunken active:scale-90"
					aria-label="Change medal of {tierLabel(tier, index)}"
					title="Medal: {tier.medal ?? 'none'} (click to change)"
				>
					<TierMark {tier} {index} size={20} />
				</button>
				<input
					value={tier.name ?? ''}
					placeholder={tier.medal ? 'Name' : tierLabel(tier, index)}
					aria-label="Name of {tierLabel(tier, index)}"
					onchange={(event) => plans.setTierName(tier.id, event.currentTarget.value)}
					class="w-[4.5rem] bg-transparent text-xs outline-none placeholder:text-muted/70"
				/>
				{#if tiered}
					<button
						onclick={() => plans.retireTier(tier.id)}
						class="grid size-5 place-items-center rounded-full text-muted transition hover:bg-sunken hover:text-ink"
						aria-label="Retire {tierLabel(tier, index)}"
						title="Retire: stops asking from this {plan.period} on, past medals stay"
					>
						<X size={12} strokeWidth={2.5} />
					</button>
				{/if}
			</div>
		{/each}
		<button
			onclick={addTier}
			class="flex items-center gap-1 rounded-full px-2.5 py-1.5 text-xs font-medium text-muted
			       transition hover:bg-sunken hover:text-ink"
		>
			<Plus size={13} strokeWidth={2} />
			{tiered ? 'Tier' : 'Add tiers'}
		</button>
	</div>

	<ul class="divide-y divide-line">
		{#each plans.for(plan.id) as requirement (requirement.id)}
			{@const members = requirement.habit_ids
				.map((id) => byId.get(id))
				.filter((habit) => habit !== undefined)}
			{@const lead = members[0]}
			<!-- All members archived: the requirement is dormant, not worth a blank row. -->
			{#if members.length}
				<li class="py-2">
					<div class="flex items-center gap-3">
						<button
							onclick={() => {
								editing = editing === requirement.id ? null : requirement.id;
								draft = [...requirement.habit_ids];
							}}
							aria-label="Edit habits in this requirement"
							class="grid size-8 shrink-0 place-items-center rounded-lg transition active:scale-90"
							style:background-color={tint(lead?.color ?? null, '24')}
							style:color={lead?.color ?? 'var(--color-accent)'}
						>
							<span class="flex gap-0.5">
								{#each members.slice(0, 2) as habit (habit.id)}
									<Icon name={habit.icon} size={14} />
								{/each}
							</span>
						</button>

						{#if members.length > 1}
							<TextField
								class="min-w-0 flex-1 px-2 py-1.5 text-sm"
								placeholder={members.map((habit) => habit.name).join(' / ')}
								value={requirement.name ?? ''}
								onchange={(raw) => plans.setName(requirement.id, raw)}
							/>
						{:else}
							<span class="min-w-0 flex-1 truncate text-sm">{lead?.name}</span>
						{/if}

						{#if hasQuantity(requirement.habit_ids)}
							<button
								onclick={() =>
									plans.setMeasure(
										requirement.id,
										requirement.measure === 'amount' ? 'occurrences' : 'amount'
									)}
								class="shrink-0 rounded-full bg-sunken px-2 py-1 text-[10px] font-medium text-muted
							       transition hover:text-ink"
								title="Switch between total amount and number of days"
							>
								{requirement.measure === 'amount' ? 'total' : 'days'}
							</button>
						{/if}

						{#if !tiered}
							{#key refused[requirement.id]}
								<TextField
									type="number"
									min="1"
									step="any"
									class="w-16 px-2 py-1.5 text-center tabular-nums"
									value={String(quotaAt(requirement, tiers[0]) ?? '')}
									onchange={(raw) => setQuota(requirement, tiers[0], raw)}
								/>
							{/key}
						{/if}

						<IconButton
							icon={X}
							label="Remove requirement"
							onclick={() => plans.removeRequirement(requirement.id)}
						/>
					</div>

					{#if tiered}
						<!-- One field per tier; leaving one empty skips that tier for this goal. -->
						{#key refused[requirement.id]}
							<div class="mt-1.5 flex flex-wrap justify-end gap-x-3 gap-y-1.5 pl-11">
								{#each tiers as tier, index (tier.id)}
									<label class="flex items-center gap-1">
										<TierMark {tier} {index} size={16} />
										<TextField
											type="number"
											min="1"
											step="any"
											placeholder="–"
											class="w-20 py-1 text-center text-sm tabular-nums"
											value={String(quotaAt(requirement, tier) ?? '')}
											onchange={(raw) => setQuota(requirement, tier, raw)}
										/>
									</label>
								{/each}
							</div>
						{/key}
					{/if}

					{#if editing === requirement.id}
						<div class="mt-2 space-y-2 rounded-xl border border-line p-2.5">
							<div class="flex flex-wrap gap-1.5">
								{#each habits.active as habit (habit.id)}
									{@const on = draft.includes(habit.id)}
									<button
										onclick={() => (draft = flip(draft, habit.id))}
										aria-pressed={on}
										class="flex items-center gap-1.5 rounded-full px-2.5 py-1.5 text-xs transition"
										style:background-color={on ? tint(habit.color, '24') : 'transparent'}
										style:color={on ? (habit.color ?? 'var(--color-accent)') : 'var(--color-muted)'}
										style:box-shadow={on ? 'none' : 'inset 0 0 0 1px var(--color-line)'}
									>
										<Icon name={habit.icon} size={13} />
										{habit.name}
									</button>
								{/each}
							</div>
							<div class="flex justify-end gap-1">
								<Button onclick={() => (editing = null)}>Cancel</Button>
								<Button
									variant="primary"
									onclick={() => saveMembers(requirement.id, requirement.habit_ids)}>Save</Button
								>
							</div>
						</div>
					{/if}
				</li>
			{/if}
		{/each}
	</ul>

	{#if adding}
		<div class="mt-3 space-y-3 rounded-xl border border-line p-3">
			<p class="text-[11px] text-muted">
				Pick one habit for a plain quota, or several to accept any mix of them.
			</p>

			<div class="flex flex-wrap gap-1.5">
				{#each habits.active as habit (habit.id)}
					{@const on = picked.includes(habit.id)}
					<button
						onclick={() => (picked = flip(picked, habit.id))}
						aria-pressed={on}
						class="flex items-center gap-1.5 rounded-full px-2.5 py-1.5 text-xs transition"
						style:background-color={on ? tint(habit.color, '24') : 'transparent'}
						style:color={on ? (habit.color ?? 'var(--color-accent)') : 'var(--color-muted)'}
						style:box-shadow={on ? 'none' : 'inset 0 0 0 1px var(--color-line)'}
					>
						<Icon name={habit.icon} size={13} />
						{habit.name}
					</button>
				{/each}
			</div>

			{#if picked.length > 1}
				<TextField
					bind:value={groupName}
					placeholder="Name this group, e.g. Cardio"
					class="w-full px-3 py-2 text-sm"
				/>
			{/if}

			<div class="flex flex-wrap items-center gap-3">
				{#each tiers as tier, index (tier.id)}
					<label class="flex items-center gap-1">
						{#if tiered}<TierMark {tier} {index} size={16} />{/if}
						<TextField
							type="number"
							min="1"
							step="any"
							placeholder="–"
							class="w-20 px-2 py-1.5 text-center tabular-nums"
							value={draftFor(tier, index)}
							onchange={(raw) => (drafts[tier.id] = raw)}
						/>
					</label>
				{/each}
				<span class="text-[11px] text-muted">
					{picked.length > 1 ? 'times across the set' : 'per period'}
				</span>
				<div class="ml-auto flex gap-1">
					<Button onclick={() => ((adding = false), (picked = []))}>Cancel</Button>
					<Button variant="primary" onclick={add}>Add</Button>
				</div>
			</div>
		</div>
	{:else}
		<button
			onclick={() => (adding = true)}
			class="mt-2 flex w-full items-center justify-center gap-1 rounded-xl py-2
			       text-xs font-medium text-muted transition hover:bg-sunken hover:text-ink"
		>
			<Plus size={14} strokeWidth={2} />
			Add requirement
		</button>
	{/if}
	{#if showHistory}
		<div class="mt-4 border-t border-line pt-4">
			<PlanVersions planId={plan.id} />
		</div>
	{/if}
</Card>
