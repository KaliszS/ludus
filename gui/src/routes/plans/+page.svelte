<script lang="ts">
	import { ArchiveRestore, History, Trash2 } from '@lucide/svelte';
	import PlanVersions from '$lib/features/plans/PlanVersions.svelte';
	import PlanEditor from '$lib/features/plans/PlanEditor.svelte';
	import IconButton from '$lib/ui/IconButton.svelte';
	import { habits } from '$lib/state/habits.svelte';
	import { plans } from '$lib/state/plans.svelte';
	import Button from '$lib/ui/Button.svelte';
	import Card from '$lib/ui/Card.svelte';
	import Empty from '$lib/ui/Empty.svelte';
	import Segmented from '$lib/ui/Segmented.svelte';
	import TextField from '$lib/ui/TextField.svelte';
	import type { Period } from '$lib/api/types';

	const PERIODS = ['day', 'week', 'month', 'quarter', 'year'] as const satisfies readonly Period[];

	let name = $state('');
	let period = $state<Period>('week');
	/** Archived plan whose history is open, if any. */
	let opened = $state<string | null>(null);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		await plans.create(name, period);
		name = '';
	}

	async function erase(id: string, name: string) {
		if (!confirm(`Delete “${name}” and its whole history? This cannot be undone.`)) return;
		await plans.remove(id);
	}

	$effect(() => {
		habits.load();
		plans.load();
	});
</script>

<div class="mb-8">
	<Card>
		<form onsubmit={submit} class="space-y-3">
			<TextField bind:value={name} placeholder="Name this plan, e.g. minimum" required />
			<div class="flex items-center gap-3">
				<Segmented options={PERIODS} value={period} onchange={(p) => (period = p)} />
				<div class="ml-auto">
					<Button type="submit" variant="primary">Add</Button>
				</div>
			</div>
		</form>
	</Card>
</div>

{#if plans.active.length}
	<div class="space-y-4">
		{#each plans.active as plan (plan.id)}
			<PlanEditor {plan} />
		{/each}
	</div>
{:else if !plans.loading}
	<Empty
		text={plans.archived.length
			? 'Every plan is archived. Restore one below, or add a new one.'
			: 'No plans yet. A plan is how ambitious a period should be.'}
	/>
{/if}

{#if plans.archived.length}
	<section class="mt-8">
		<h2 class="mb-2 px-1 text-[11px] tracking-wide text-muted uppercase">Archived</h2>
		<Card>
			<ul class="-my-2 divide-y divide-line">
				{#each plans.archived as plan (plan.id)}
					<li class="py-2">
						<div class="flex items-center gap-2">
							<span class="min-w-0 flex-1 truncate text-sm">{plan.name}</span>
							<span class="text-xs text-muted">{plan.period}</span>
							<IconButton
								icon={History}
								label={opened === plan.id ? 'Hide history' : 'Show history'}
								onclick={() => (opened = opened === plan.id ? null : plan.id)}
							/>
							<IconButton
								icon={ArchiveRestore}
								label="Restore"
								onclick={() => plans.setArchived(plan.id, false)}
							/>
							<IconButton
								icon={Trash2}
								label="Delete with its history"
								tone="danger"
								onclick={() => erase(plan.id, plan.name)}
							/>
						</div>
						{#if opened === plan.id}
							<div class="mt-2 mb-1 pl-1">
								<PlanVersions planId={plan.id} />
							</div>
						{/if}
					</li>
				{/each}
			</ul>
		</Card>
	</section>
{/if}
