<script lang="ts">
	import LevelEditor from '$lib/features/levels/LevelEditor.svelte';
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

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		await levels.create(name, period);
		name = '';
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

{#if levels.items.length}
	<div class="space-y-4">
		{#each levels.items as level (level.id)}
			<LevelEditor {level} />
		{/each}
	</div>
{:else if !levels.loading}
	<Empty text="No levels yet. A level is how ambitious a period should be." />
{/if}
