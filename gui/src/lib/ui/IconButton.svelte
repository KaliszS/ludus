<script lang="ts">
	import type { LucideIcon } from '@lucide/svelte';

	interface Props {
		icon: LucideIcon;
		label: string;
		tone?: 'muted' | 'danger';
		onclick: () => void;
	}
	let { icon: Glyph, label, tone = 'muted', onclick }: Props = $props();
</script>

<button
	type="button"
	{onclick}
	aria-label={label}
	title={label}
	class="grid size-9 shrink-0 place-items-center rounded-full text-muted transition
	       duration-200 active:scale-90
	       {tone === 'danger'
		? 'hover:bg-accent-soft hover:text-accent'
		: 'hover:bg-sunken hover:text-ink'}"
>
	<Glyph size={17} strokeWidth={1.75} />
</button>
