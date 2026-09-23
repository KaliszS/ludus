<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import Card from '$lib/ui/Card.svelte';
	import TextField from '$lib/ui/TextField.svelte';
	import type { IconKey } from '$lib/domain/icons';
	import { PALETTE } from '$lib/domain/palette';
	import { habits } from '$lib/state/habits.svelte';
	import type { Tracking } from '$lib/api/types';
	import ColorPicker from './ColorPicker.svelte';
	import IconPicker from './IconPicker.svelte';

	let name = $state('');
	let icon = $state<IconKey>('dumbbell');
	let color = $state<string>(PALETTE[0]);
	let tracking = $state<Tracking>('binary');
	let unit = $state('');

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const created = await habits.create({
			name,
			icon,
			color,
			tracking,
			unit: tracking === 'quantity' && unit.trim() ? unit.trim() : null
		});
		if (created) {
			name = '';
			unit = '';
		}
	}
</script>

<Card>
	<form onsubmit={submit} class="space-y-5">
		<TextField bind:value={name} placeholder="What do you want to keep doing?" required />

		<IconPicker value={icon} {color} onchange={(key) => (icon = key)} />
		<ColorPicker value={color} onchange={(next) => (color = next)} />

		<div class="flex flex-wrap items-center gap-3 border-t border-line pt-4">
			<div class="inline-flex gap-0.5 rounded-full border border-line bg-sunken p-0.5 text-xs">
				{#each [['binary', 'a tick'], ['quantity', 'an amount']] as [key, label] (key)}
					<button
						type="button"
						onclick={() => (tracking = key as Tracking)}
						class="rounded-full px-3 py-1.5 font-medium transition
						       {tracking === key ? 'bg-surface text-ink shadow-sm' : 'text-muted'}"
					>
						{label}
					</button>
				{/each}
			</div>

			{#if tracking === 'quantity'}
				<TextField bind:value={unit} placeholder="unit, e.g. word" class="w-40" />
			{/if}

			<div class="ml-auto">
				<Button type="submit" variant="primary">Add habit</Button>
			</div>
		</div>
	</form>
</Card>
