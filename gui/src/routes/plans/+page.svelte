<script lang="ts">
	import { ArchiveRestore, History, Trash2 } from '@lucide/svelte';
	import LevelVersions from '$lib/features/levels/LevelVersions.svelte';
	import LevelEditor from '$lib/features/levels/LevelEditor.svelte';
	import IconButton from '$lib/ui/IconButton.svelte';
	import { habits } from '$lib/state/habits.svelte';
	import { levels } from '$lib/state/levels.svelte';
	import Button from '$lib/ui/Button.svelte';
	import Card from '$lib/ui/Card.svelte';
	import Empty from '$lib/ui/Empty.svelte';
	import Segmented from '$lib/ui/Segmented.svelte';
	import TextField from '$lib/ui/TextField.svelte';
	import type { Period } from '$lib/api/types';

	const PERIODS = ['day', 'week', 'month', 'quarter', 'year'] as const satisfies readonly Period[];

	let name = $state('');
	let period = $state<Period>('week');
	/** Archived level whose history is open, if any. */
	let opened = $state<string | null>(null);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		await levels.create(name, period);
		name = '';
	}

	async function erase(id: string, name: string) {
		if (!confirm(`Delete “${name}” and its whole history? This cannot be undone.`)) return;
		await levels.remove(id);
	}

	$effect(() => {
		habits.load();
		levels.load();
	});
</script>

<div class="mb-8">
	<Card>
		<form onsubmit={submit} class="space-y-3">
			<TextField bind:value={name} placeholder="Name this level, e.g. minimum" required />
			<div class="flex items-center gap-3">
				<Segmented options={PERIODS} value={period} onchange={(p) => (period = p)} />
				<div class="ml-auto">
					<Button type="submit" variant="primary">Add</Button>
				</div>
			</div>
		</form>
	</Card>
</div>

{#if levels.active.length}
	<div class="space-y-4">
		{#each levels.active as level (level.id)}
			<LevelEditor {level} />
		{/each}
	</div>
{:else if !levels.loading}
	<Empty
		text={levels.archived.length
			? 'Every level is archived. Restore one below, or add a new one.'
			: 'No levels yet. A level is how ambitious a period should be.'}
	/>
{/if}

{#if levels.archived.length}
	<section class="mt-8">
		<h2 class="mb-2 px-1 text-[11px] tracking-wide text-muted uppercase">Archived</h2>
		<Card>
			<ul class="-my-2 divide-y divide-line">
				{#each levels.archived as level (level.id)}
					<li class="py-2">
						<div class="flex items-center gap-2">
							<span class="min-w-0 flex-1 truncate text-sm">{level.name}</span>
							<span class="text-xs text-muted">{level.period}</span>
							<IconButton
								icon={History}
								label={opened === level.id ? 'Hide history' : 'Show history'}
								onclick={() => (opened = opened === level.id ? null : level.id)}
							/>
							<IconButton
								icon={ArchiveRestore}
								label="Restore"
								onclick={() => levels.setArchived(level.id, false)}
							/>
							<IconButton
								icon={Trash2}
								label="Delete with its history"
								tone="danger"
								onclick={() => erase(level.id, level.name)}
							/>
						</div>
						{#if opened === level.id}
							<div class="mt-2 mb-1 pl-1">
								<LevelVersions levelId={level.id} />
							</div>
						{/if}
					</li>
				{/each}
			</ul>
		</Card>
	</section>
{/if}
