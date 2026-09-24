<script lang="ts">
	import Icon from '$lib/ui/Icon.svelte';
	import type { Habit } from '$lib/api/types';

	interface Props {
		habits: Habit[];
		/** Tints the icons in their own colours; otherwise they stay muted. */
		active?: boolean;
		size?: number;
		/** Allows a second row, where the surrounding layout has the height for it. */
		stacked?: boolean;
	}
	let { habits, active = false, size = 18, stacked = false }: Props = $props();

	/** Fixed slot, or a bigger group would shift whatever follows it out of line. */
	const width = $derived(size * 2 + 8);

	const twoRows = $derived(stacked && habits.length > 2);
	/** Two rows only fit beside a two-line label once the glyphs shrink. */
	const glyph = $derived(twoRows ? Math.round(size * 0.8) : size);

	const rows = $derived.by(() => {
		if (!twoRows) return [habits];
		const top = Math.ceil(habits.length / 2);
		return [habits.slice(0, top), habits.slice(top)];
	});

	/** Two fit side by side; beyond that they overlap so the slot keeps its width. */
	const spacing = (count: number) =>
		count <= 2 ? 3 : Math.max(-glyph * 0.55, (width - count * glyph) / (count - 1));
</script>

<span class="flex shrink-0 flex-col items-center justify-center gap-[3px]" style:width="{width}px">
	{#each rows as row, index (index)}
		<span class="flex items-center overflow-hidden">
			{#each row as habit, position (habit.id)}
				<!-- Each sits on an opaque disc so the top one masks the one under it. -->
				<span
					class="grid shrink-0 place-items-center rounded-full bg-surface transition-colors"
					style:width="{glyph}px"
					style:height="{glyph}px"
					style:margin-left={position === 0 ? '0' : `${spacing(row.length)}px`}
					style:color={active ? (habit.color ?? 'var(--color-accent)') : 'var(--color-muted)'}
				>
					<Icon name={habit.icon} size={glyph - 3} />
				</span>
			{/each}
		</span>
	{/each}
</span>
