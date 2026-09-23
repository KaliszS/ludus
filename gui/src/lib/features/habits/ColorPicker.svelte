<script lang="ts">
	import { PALETTE } from '$lib/domain/palette';

	interface Props {
		value: string;
		onchange: (color: string) => void;
	}
	let { value, onchange }: Props = $props();
</script>

<div class="grid grid-cols-12 gap-1.5">
	{#each PALETTE as color (color)}
		<button
			type="button"
			onclick={() => onchange(color)}
			aria-label={color}
			aria-pressed={value === color}
			class="aspect-square rounded-full transition active:scale-90"
			style:background-color={color}
			style:box-shadow={value === color
				? `0 0 0 2px var(--color-surface), 0 0 0 3.5px ${color}`
				: 'none'}
		></button>
	{/each}
</div>
