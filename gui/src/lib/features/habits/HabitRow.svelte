<script lang="ts">
	import { Archive, ArchiveRestore, ChevronDown, GripVertical, Trash2 } from '@lucide/svelte';
	import Button from '$lib/ui/Button.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import TextField from '$lib/ui/TextField.svelte';
	import { PALETTE, tint } from '$lib/domain/palette';
	import { habits } from '$lib/state/habits.svelte';
	import type { Habit } from '$lib/api/types';
	import CategorySelect from './CategorySelect.svelte';
	import ColorPicker from './ColorPicker.svelte';
	import IconPicker from './IconPicker.svelte';

	interface Props {
		habit: Habit;
		ongrab?: () => void;
	}
	let { habit, ongrab }: Props = $props();

	let editing = $state(false);
	/** The pickers take a lot of room, so they open only when asked for. */
	let styling = $state(false);
	// Seeded by open(), so the drafts survive the habit reloading underneath.
	let name = $state('');
	let notes = $state('');
	let icon = $state('heart');
	let color = $state<string>(PALETTE[0]);
	let unit = $state('');
	let category = $state<string | null>(null);

	const firstLine = $derived(habit.description.split('\n')[0].trim());

	function open() {
		name = habit.name;
		notes = habit.description;
		icon = habit.icon ?? 'heart';
		color = habit.color ?? PALETTE[0];
		unit = habit.unit ?? '';
		category = habit.category_id;
		styling = false;
		editing = true;
	}

	async function save() {
		const saved = await habits.update(habit.id, {
			name,
			description: notes.trim(),
			icon,
			color,
			unit: habit.tracking === 'quantity' ? unit.trim() || null : null,
			category_id: category
		});
		if (saved) editing = false;
	}

	async function remove() {
		if (
			!confirm(
				`Delete “${habit.name}” and every check-in for it? Archiving keeps its history instead.`
			)
		)
			return;
		await habits.remove(habit.id);
	}
</script>

<div class="group">
	<div class="flex items-center gap-1.5 py-1 pr-1">
		{#if ongrab}
			<span
				role="button"
				tabindex="-1"
				aria-label="Reorder {habit.name}"
				onpointerdown={ongrab}
				class="grid h-8 w-5 shrink-0 cursor-grab touch-none place-items-center text-muted opacity-0
				       transition group-hover:opacity-100 hover:text-ink active:cursor-grabbing
				       [@media(hover:none)]:opacity-60"
			>
				<GripVertical size={14} strokeWidth={1.75} />
			</span>
		{/if}

		<button
			onclick={() => (editing ? (editing = false) : open())}
			aria-expanded={editing}
			class="flex min-w-0 flex-1 items-center gap-2.5 rounded-lg py-0.5 text-left"
		>
			<span
				class="grid size-8 shrink-0 place-items-center rounded-lg"
				class:opacity-40={habit.archived}
				style:background-color={tint(habit.color, '24')}
				style:color={habit.color ?? 'var(--color-accent)'}
			>
				<Icon name={habit.icon} size={16} />
			</span>

			<span class="min-w-0 flex-1">
				<span class="flex items-baseline gap-1.5">
					<span class="truncate text-sm font-medium" class:line-through={habit.archived}>
						{habit.name}
					</span>
					{#if habit.tracking === 'quantity'}
						<span class="shrink-0 text-[11px] text-muted">{habit.unit ?? 'amount'}</span>
					{/if}
				</span>
				{#if firstLine}
					<span class="block truncate text-xs text-muted">{firstLine}</span>
				{/if}
			</span>

			<ChevronDown
				size={15}
				strokeWidth={1.75}
				class="shrink-0 text-muted transition-transform {editing ? 'rotate-180' : ''}"
			/>
		</button>
	</div>

	{#if editing}
		<div class="mb-2 ml-6 space-y-3 rounded-xl bg-sunken/60 p-3">
			<div class="flex items-center gap-2">
				<button
					onclick={() => (styling = !styling)}
					aria-label="Change icon and colour"
					title="Change icon and colour"
					aria-pressed={styling}
					class="grid size-9 shrink-0 place-items-center rounded-xl transition active:scale-90"
					style:background-color={tint(color, '24')}
					style:color
				>
					<Icon name={icon} size={18} />
				</button>
				<TextField bind:value={name} placeholder="Name" class="min-w-0 flex-1 px-3 py-2 text-sm" />
			</div>

			{#if styling}
				<IconPicker value={icon} {color} onchange={(key) => (icon = key)} />
				<ColorPicker value={color} onchange={(next) => (color = next)} />
			{/if}

			<!-- Notes are the habit's own instructions - how it is done, what counts -
			     so the first line doubles as the subtitle in the list. -->
			<textarea
				bind:value={notes}
				rows="3"
				placeholder="Notes: how you do it, what counts, reminders…"
				class="block w-full resize-y rounded-xl border border-line bg-canvas px-3 py-2 text-sm
				       transition outline-none placeholder:text-muted/60 focus:border-accent"></textarea>

			<div class="flex flex-wrap items-center gap-2">
				<CategorySelect value={category} onchange={(id) => (category = id)} />
				{#if habit.tracking === 'quantity'}
					<TextField
						bind:value={unit}
						placeholder="unit, e.g. word"
						class="w-32 px-3 py-2 text-sm"
					/>
				{/if}
				<!-- Tracking stays fixed: past check-ins were recorded under its meaning. -->
				<span class="text-[11px] text-muted">
					{habit.tracking === 'quantity' ? 'records an amount' : 'records a tick'}
				</span>
			</div>

			<div class="flex items-center gap-1 border-t border-line/60 pt-2">
				<button
					onclick={() => habits.update(habit.id, { archived: !habit.archived })}
					class="flex items-center gap-1.5 rounded-full px-2.5 py-1.5 text-xs font-medium text-muted
					       transition hover:bg-surface hover:text-ink"
				>
					{#if habit.archived}
						<ArchiveRestore size={14} strokeWidth={1.75} /> Restore
					{:else}
						<Archive size={14} strokeWidth={1.75} /> Archive
					{/if}
				</button>
				<button
					onclick={remove}
					class="flex items-center gap-1.5 rounded-full px-2.5 py-1.5 text-xs font-medium text-muted
					       transition hover:bg-accent-soft hover:text-accent"
				>
					<Trash2 size={14} strokeWidth={1.75} /> Delete
				</button>
				<div class="ml-auto flex gap-1">
					<Button onclick={() => (editing = false)}>Cancel</Button>
					<Button variant="primary" onclick={save}>Save</Button>
				</div>
			</div>
		</div>
	{/if}
</div>
