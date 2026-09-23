<script lang="ts">
	import { Plus, Trash2, X } from '@lucide/svelte';
	import Button from '$lib/ui/Button.svelte';
	import Card from '$lib/ui/Card.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import IconButton from '$lib/ui/IconButton.svelte';
	import Segmented from '$lib/ui/Segmented.svelte';
	import TextField from '$lib/ui/TextField.svelte';
	import { tint } from '$lib/domain/palette';
	import { habits } from '$lib/state/habits.svelte';
	import { levels } from '$lib/state/levels.svelte';
	import type { Measure, Period, PlanLevel } from '$lib/api/types';

	const PERIODS = ['day', 'week', 'month', 'quarter', 'year'] as const satisfies readonly Period[];

	let { level }: { level: PlanLevel } = $props();

	let picked = $state<string[]>([]);
	let quota = $state('1');
	let groupName = $state('');
	let adding = $state(false);
	/** Requirement whose member list is open for editing. */
	let editing = $state<string | null>(null);
	let draft = $state<string[]>([]);

	const byId = $derived(new Map(habits.active.map((habit) => [habit.id, habit])));
	const hasQuantity = (ids: string[]) => ids.some((id) => byId.get(id)?.tracking === 'quantity');

	const flip = (list: string[], id: string) =>
		list.includes(id) ? list.filter((value) => value !== id) : [...list, id];

	async function add() {
		if (picked.length === 0) return;
		// A group counts days, because summing "45 words" with "1 session" is nonsense.
		const measure: Measure = picked.length > 1 ? 'occurrences' : 'amount';
		await levels.addRequirement(level.id, picked, Number(quota), measure, groupName || null);
		picked = [];
		quota = '1';
		groupName = '';
		adding = false;
	}

	/** Archived members are not offered as chips, so carry them over untouched
	 *  instead of dropping them the moment someone edits the visible ones. */
	async function saveMembers(id: string, original: string[]) {
		const hidden = original.filter((habitId) => !byId.has(habitId));
		await levels.setMembers(id, [...draft, ...hidden]);
		editing = null;
	}

	async function remove() {
		if (!confirm(`Delete level “${level.name}”?`)) return;
		await levels.remove(level.id);
	}
</script>

<Card>
	<header class="mb-3 flex items-center gap-2">
		<TextField
			class="min-w-0 flex-1 px-2 py-1.5 text-[15px] font-semibold"
			value={level.name}
			onchange={(raw) => levels.rename(level.id, raw)}
		/>
		<IconButton icon={Trash2} label="Delete level" tone="danger" onclick={remove} />
	</header>

	<div class="mb-3">
		<Segmented
			options={PERIODS}
			value={level.period}
			onchange={(period) => levels.setPeriod(level.id, period)}
		/>
	</div>

	<ul class="divide-y divide-line">
		{#each levels.for(level.id) as requirement (requirement.id)}
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
								onchange={(raw) => levels.setName(requirement.id, raw)}
							/>
						{:else}
							<span class="min-w-0 flex-1 truncate text-sm">{lead?.name}</span>
						{/if}

						{#if hasQuantity(requirement.habit_ids)}
							<button
								onclick={() =>
									levels.setMeasure(
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

						<TextField
							type="number"
							min="1"
							step="any"
							class="w-16 px-2 py-1.5 text-center tabular-nums"
							value={String(requirement.quota)}
							onchange={(raw) => levels.setQuota(requirement.id, raw)}
						/>

						<IconButton
							icon={X}
							label="Remove requirement"
							onclick={() => levels.removeRequirement(requirement.id)}
						/>
					</div>

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

			<div class="flex items-center gap-3">
				<TextField
					type="number"
					min="1"
					step="any"
					class="w-20 px-2 py-1.5 text-center tabular-nums"
					bind:value={quota}
				/>
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
</Card>
