<script lang="ts">
	import { Archive, ArchiveRestore, GripVertical, Pencil, Trash2 } from '@lucide/svelte';
	import Button from '$lib/ui/Button.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import IconButton from '$lib/ui/IconButton.svelte';
	import TextField from '$lib/ui/TextField.svelte';
	import { PALETTE, tint } from '$lib/domain/palette';
	import { habits } from '$lib/state/habits.svelte';
	import type { Habit } from '$lib/api/types';
	import ColorPicker from './ColorPicker.svelte';
	import IconPicker from './IconPicker.svelte';

	interface Props {
		habit: Habit;
		ongrab?: () => void;
	}
	let { habit, ongrab }: Props = $props();

	let editing = $state(false);
	// Seeded by open(), so the drafts survive the habit reloading underneath.
	let name = $state('');
	let icon = $state('heart');
	let color = $state<string>(PALETTE[0]);
	let unit = $state('');

	function open() {
		name = habit.name;
		icon = habit.icon ?? 'heart';
		color = habit.color ?? PALETTE[0];
		unit = habit.unit ?? '';
		editing = true;
	}

	async function save() {
		await habits.update(habit.id, {
			name,
			icon,
			color,
			unit: habit.tracking === 'quantity' ? unit.trim() || null : null
		});
		editing = false;
	}

	async function remove() {
		if (!confirm(`Delete “${habit.name}” and every check-in for it?`)) return;
		await habits.remove(habit.id);
	}
</script>

<div class="rounded-2xl border border-line bg-surface py-2.5 pr-3 pl-1">
	<div class="flex items-center gap-2">
		{#if ongrab}
			<span
				role="button"
				tabindex="-1"
				aria-label="Reorder {habit.name}"
				onpointerdown={ongrab}
				class="grid size-7 shrink-0 cursor-grab touch-none place-items-center text-muted
				       transition hover:text-ink active:cursor-grabbing"
			>
				<GripVertical size={16} strokeWidth={1.75} />
			</span>
		{/if}

		<span
			class="grid size-11 shrink-0 place-items-center rounded-xl transition"
			class:opacity-40={habit.archived}
			style:background-color={tint(habit.color, '24')}
			style:color={habit.color ?? 'var(--color-accent)'}
		>
			<Icon name={habit.icon} size={20} />
		</span>

		<div class="min-w-0 flex-1">
			<div class="truncate font-medium" class:line-through={habit.archived}>{habit.name}</div>
			<div class="text-xs text-muted">
				{habit.tracking === 'quantity' ? `amount${habit.unit ? ` · ${habit.unit}` : ''}` : 'tick'}
			</div>
		</div>

		<IconButton
			icon={Pencil}
			label={editing ? 'Close editor' : 'Edit habit'}
			onclick={() => (editing ? (editing = false) : open())}
		/>
		<IconButton
			icon={habit.archived ? ArchiveRestore : Archive}
			label={habit.archived ? 'Restore' : 'Archive'}
			onclick={() => habits.update(habit.id, { archived: !habit.archived })}
		/>
		<IconButton icon={Trash2} label="Delete" tone="danger" onclick={remove} />
	</div>

	{#if editing}
		<div class="mt-2.5 ml-1 space-y-3 border-t border-line pt-3">
			<TextField bind:value={name} placeholder="Name" class="w-full px-3 py-2 text-sm" />

			<IconPicker value={icon} {color} onchange={(key) => (icon = key)} />
			<ColorPicker value={color} onchange={(next) => (color = next)} />

			<div class="flex flex-wrap items-center gap-3">
				{#if habit.tracking === 'quantity'}
					<TextField
						bind:value={unit}
						placeholder="unit, e.g. word"
						class="w-36 px-3 py-2 text-sm"
					/>
				{/if}
				<!-- Tracking stays fixed: past check-ins were recorded under its meaning. -->
				<span class="text-[11px] text-muted">
					{habit.tracking === 'quantity' ? 'records an amount' : 'records a tick'}
				</span>

				<div class="ml-auto flex gap-1">
					<Button onclick={() => (editing = false)}>Cancel</Button>
					<Button variant="primary" onclick={save}>Save</Button>
				</div>
			</div>
		</div>
	{/if}
</div>
