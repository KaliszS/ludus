<script lang="ts">
	import { ICON_GROUPS } from '$lib/domain/icons';
	import { tint } from '$lib/domain/palette';
	import Icon from '$lib/ui/Icon.svelte';

	interface Props {
		value: string;
		color: string;
		onchange: (key: string) => void;
	}
	let { value, color, onchange }: Props = $props();

	/** Opening on the group holding the current icon keeps editing predictable. */
	let group = $state(
		ICON_GROUPS.find((entry) => entry.keys.includes(value))?.label ?? ICON_GROUPS[0].label
	);
	const keys = $derived(ICON_GROUPS.find((entry) => entry.label === group)?.keys ?? []);
</script>

<div class="space-y-2">
	<div
		class="flex [scrollbar-width:none] gap-1 overflow-x-auto pb-0.5 text-[11px] [&::-webkit-scrollbar]:hidden"
	>
		{#each ICON_GROUPS as entry (entry.label)}
			<button
				type="button"
				onclick={() => (group = entry.label)}
				aria-pressed={group === entry.label}
				class="shrink-0 rounded-full px-2.5 py-1 font-medium transition-colors
				       {group === entry.label ? 'bg-sunken text-ink' : 'text-muted hover:text-ink'}"
			>
				{entry.label}
			</button>
		{/each}
	</div>

	<div class="grid grid-cols-8 gap-1.5">
		{#each keys as key (key)}
			<button
				type="button"
				onclick={() => onchange(key)}
				aria-label={key}
				aria-pressed={value === key}
				class="grid aspect-square place-items-center rounded-xl transition-all duration-200
				       active:scale-90"
				style:background-color={value === key ? tint(color, '24') : 'transparent'}
				style:color={value === key ? color : 'var(--color-muted)'}
			>
				<Icon name={key} size={19} />
			</button>
		{/each}
	</div>
</div>
