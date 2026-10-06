<script lang="ts">
	import { Archive, History, Plus, X } from '@lucide/svelte';
	import PlanVersions from './PlanVersions.svelte';
	import Button from '$lib/ui/Button.svelte';
	import Card from '$lib/ui/Card.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import IconButton from '$lib/ui/IconButton.svelte';
	import Segmented from '$lib/ui/Segmented.svelte';
	import TierMark from './TierMark.svelte';
	import { tint } from '$lib/domain/palette';
	import { MEDALS, carriedQuota, liveTiers, tierLabel } from '$lib/domain/plan';
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

	const draftQuotas = $derived(
		tiers.map((tier, index) => Number(draftFor(tier, index)) || undefined)
	);

	/** Name, one narrow column per tier, then the row's action - shared by the header,
	 *  every requirement and the add form, so quotas line up under their tier. */
	const columns = $derived(`minmax(0, 1fr) repeat(${tiers.length}, var(--cell)) 1.75rem`);

	const cell =
		'h-8 w-full min-w-0 rounded-lg text-center text-sm tabular-nums outline-none transition ' +
		'hover:ring-1 hover:ring-line focus:ring-2 focus:ring-accent';

	/** A tier's own quota sits in a filled field. Without one, the tier either still holds
	 *  the quota below it, shown faded, or does not ask for this goal at all. */
	function quotaCell(own: (number | undefined)[], index: number, fill: string) {
		const carried = carriedQuota(own, index);
		if (own[index] !== undefined) return { carried, style: `${cell} ${fill}` };
		if (carried !== undefined) {
			return {
				carried,
				style: `${cell} bg-transparent placeholder:text-muted`,
				title: `Still ${carried} from the tier below; type a higher number to raise it`
			};
		}
		return {
			carried,
			style: `${cell} border border-dashed border-line/70 bg-transparent`,
			title: 'Not asked at this tier'
		};
	}

	/** Archived members are not offered as chips, so carry them over untouched
	 *  instead of dropping them the moment someone edits the visible ones. */
	async function saveMembers(id: string, original: string[]) {
		const hidden = original.filter((habitId) => !byId.has(habitId));
		await plans.setMembers(id, [...draft, ...hidden]);
		editing = null;
	}
</script>

<Card>
	<header class="flex items-center gap-1">
		<input
			value={plan.name}
			aria-label="Plan name"
			onchange={(event) => plans.rename(plan.id, event.currentTarget.value)}
			class="-ml-2 min-w-0 flex-1 rounded-lg bg-transparent px-2 py-1 text-[17px] font-semibold
			       transition outline-none hover:bg-sunken focus:bg-sunken"
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

	<div class="mt-2 w-fit max-w-full">
		<Segmented
			options={PERIODS}
			value={plan.period}
			onchange={(period) => plans.setPeriod(plan.id, period)}
		/>
	</div>

	<div class="mt-4 [--cell:2.75rem] sm:[--cell:3.5rem]">
		{#if tiered}
			<!-- Tiers climb left to right. The lowest keeps the streak; tap a coin to change
			     its medal. Medal and name are labels, so changing them relabels the past. -->
			<div class="grid items-end gap-x-1.5 pb-1.5" style:grid-template-columns={columns}>
				<span class="pb-1 text-[11px] text-muted">Goal</span>
				{#each tiers as tier, index (tier.id)}
					<div class="group relative flex flex-col items-center gap-0.5">
						<button
							onclick={() => cycleMedal(tier)}
							class="grid size-7 place-items-center rounded-full transition hover:bg-sunken active:scale-90"
							aria-label="Change medal of {tierLabel(tier, index)}"
							title="Medal: {tier.medal ?? 'none'} (tap to change)"
						>
							<TierMark {tier} {index} size={20} />
						</button>
						<input
							value={tier.name ?? ''}
							placeholder={tierLabel(tier, index)}
							aria-label="Name of {tierLabel(tier, index)}"
							onchange={(event) => plans.setTierName(tier.id, event.currentTarget.value)}
							class="w-full min-w-0 rounded bg-transparent text-center text-[11px] text-muted outline-none
							       placeholder:text-muted/60 focus:bg-sunken focus:text-ink"
						/>
						<button
							onclick={() => plans.retireTier(tier.id)}
							class="absolute -top-1 -right-0.5 grid size-4 place-items-center rounded-full bg-surface
							       text-muted opacity-0 shadow-sm ring-1 ring-line transition group-hover:opacity-100
							       hover:text-ink focus:opacity-100 [@media(hover:none)]:opacity-100"
							aria-label="Retire {tierLabel(tier, index)}"
							title="Retire: stops asking from this {plan.period} on, past medals stay"
						>
							<X size={10} strokeWidth={2.5} />
						</button>
					</div>
				{/each}
				<button
					onclick={addTier}
					class="mb-5 grid size-7 place-items-center rounded-full border border-dashed border-line text-muted
					       transition hover:bg-sunken hover:text-ink"
					aria-label="Add a tier"
					title="Add a tier on top"
				>
					<Plus size={13} strokeWidth={2} />
				</button>
			</div>
		{/if}

		<ul class="divide-y divide-line/60 border-y border-line/60">
			{#each plans.for(plan.id) as requirement (requirement.id)}
				{@const members = requirement.habit_ids
					.map((id) => byId.get(id))
					.filter((habit) => habit !== undefined)}
				{@const lead = members[0]}
				{@const own = tiers.map((tier) => quotaAt(requirement, tier))}
				<!-- All members archived: the requirement is dormant, not worth a blank row. -->
				{#if members.length}
					<li class="py-1.5">
						<div class="grid items-center gap-x-1.5" style:grid-template-columns={columns}>
							<div class="flex min-w-0 items-center gap-2">
								<button
									onclick={() => {
										editing = editing === requirement.id ? null : requirement.id;
										draft = [...requirement.habit_ids];
									}}
									aria-label="Edit habits in this requirement"
									title="Edit habits"
									class="grid size-7 shrink-0 place-items-center rounded-lg transition active:scale-90"
									style:background-color={tint(lead?.color ?? null, '24')}
									style:color={lead?.color ?? 'var(--color-accent)'}
								>
									<span class="flex gap-px">
										{#each members.slice(0, 2) as habit (habit.id)}
											<Icon name={habit.icon} size={13} />
										{/each}
									</span>
								</button>

								{#if members.length > 1}
									<input
										value={requirement.name ?? ''}
										placeholder={members.map((habit) => habit.name).join(' / ')}
										aria-label="Name of this group"
										onchange={(event) => plans.setName(requirement.id, event.currentTarget.value)}
										class="-ml-1 min-w-0 flex-1 truncate rounded-md bg-transparent px-1 py-0.5 text-sm
										       outline-none placeholder:text-ink/70 hover:bg-sunken focus:bg-sunken"
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
										class="shrink-0 rounded-full bg-sunken px-1.5 py-0.5 text-[10px] font-medium text-muted
										       transition hover:text-ink"
										title="Count the total amount, or the days with any"
									>
										{requirement.measure === 'amount' ? 'total' : 'days'}
									</button>
								{/if}
							</div>

							{#key refused[requirement.id]}
								{#each tiers as tier, index (tier.id)}
									{@const look = quotaCell(own, index, 'bg-sunken')}
									<input
										type="number"
										min="1"
										step="any"
										placeholder={look.carried === undefined ? '' : String(look.carried)}
										aria-label="{tierLabel(tier, index)} quota"
										title={look.title}
										value={String(own[index] ?? '')}
										onchange={(event) => setQuota(requirement, tier, event.currentTarget.value)}
										class={look.style}
									/>
								{/each}
							{/key}

							<button
								onclick={() => plans.removeRequirement(requirement.id)}
								aria-label="Remove requirement"
								title="Remove requirement"
								class="grid size-7 place-items-center rounded-full text-muted transition hover:bg-sunken hover:text-ink"
							>
								<X size={14} strokeWidth={2} />
							</button>
						</div>

						{#if editing === requirement.id}
							<div class="mt-2 mb-1 space-y-2 rounded-xl bg-sunken/60 p-2">
								<div class="flex flex-wrap gap-1.5">
									{#each habits.active as habit (habit.id)}
										{@const on = draft.includes(habit.id)}
										<button
											onclick={() => (draft = flip(draft, habit.id))}
											aria-pressed={on}
											class="flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs transition"
											style:background-color={on ? tint(habit.color, '24') : 'transparent'}
											style:color={on
												? (habit.color ?? 'var(--color-accent)')
												: 'var(--color-muted)'}
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
			<div class="mt-2 space-y-2.5 rounded-xl bg-sunken/60 p-2.5">
				<p class="text-[11px] text-muted">
					One habit for a plain quota, or several to accept any mix of them.
				</p>
				<div class="flex flex-wrap gap-1.5">
					{#each habits.active as habit (habit.id)}
						{@const on = picked.includes(habit.id)}
						<button
							onclick={() => (picked = flip(picked, habit.id))}
							aria-pressed={on}
							class="flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs transition"
							style:background-color={on ? tint(habit.color, '24') : 'transparent'}
							style:color={on ? (habit.color ?? 'var(--color-accent)') : 'var(--color-muted)'}
							style:box-shadow={on ? 'none' : 'inset 0 0 0 1px var(--color-line)'}
						>
							<Icon name={habit.icon} size={13} />
							{habit.name}
						</button>
					{/each}
				</div>

				<!-- The quotas sit in the same columns as the table above. -->
				<div class="grid items-center gap-x-1.5" style:grid-template-columns={columns}>
					{#if picked.length > 1}
						<input
							bind:value={groupName}
							placeholder="Group name, e.g. Cardio"
							class="min-w-0 rounded-lg bg-surface px-2 py-1.5 text-sm outline-none
							       placeholder:text-muted/70 focus:ring-2 focus:ring-accent"
						/>
					{:else}
						<span class="text-[11px] text-muted">Quota per {plan.period}</span>
					{/if}
					{#each tiers as tier, index (tier.id)}
						{@const look = quotaCell(draftQuotas, index, 'bg-surface')}
						<input
							type="number"
							min="1"
							step="any"
							placeholder={look.carried === undefined ? '' : String(look.carried)}
							aria-label="{tierLabel(tier, index)} quota"
							title={look.title}
							value={draftFor(tier, index)}
							onchange={(event) => (drafts[tier.id] = event.currentTarget.value)}
							class={look.style}
						/>
					{/each}
				</div>

				<div class="flex justify-end gap-1">
					<Button onclick={() => ((adding = false), (picked = []))}>Cancel</Button>
					<Button variant="primary" onclick={add}>Add</Button>
				</div>
			</div>
		{/if}
	</div>

	<footer class="mt-2 flex items-center gap-1">
		{#if !adding}
			<button
				onclick={() => (adding = true)}
				class="flex items-center gap-1 rounded-full px-2.5 py-1.5 text-xs font-medium text-muted
				       transition hover:bg-sunken hover:text-ink"
			>
				<Plus size={13} strokeWidth={2} />
				Requirement
			</button>
		{/if}
		{#if !tiered}
			<button
				onclick={addTier}
				class="flex items-center gap-1 rounded-full px-2.5 py-1.5 text-xs font-medium text-muted
				       transition hover:bg-sunken hover:text-ink"
			>
				<Plus size={13} strokeWidth={2} />
				Medal tiers
			</button>
		{/if}
		<p class="ml-auto hidden text-right text-[11px] text-muted sm:block">
			Changes apply from this {plan.period} on.
		</p>
	</footer>

	{#if showHistory}
		<div class="mt-3 border-t border-line pt-4">
			<PlanVersions planId={plan.id} />
		</div>
	{/if}
</Card>
