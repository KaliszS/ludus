<script lang="ts" generics="T extends string">
	interface Props {
		options: readonly T[];
		value: T;
		onchange: (value: T) => void;
	}

	let { options, value, onchange }: Props = $props();
</script>

<div
	class="flex max-w-full [scrollbar-width:none] gap-0.5 overflow-x-auto rounded-full border border-line bg-sunken
	       p-0.5 text-xs [&::-webkit-scrollbar]:hidden"
>
	{#each options as option (option)}
		<button
			type="button"
			onclick={() => onchange(option)}
			aria-pressed={option === value}
			class="shrink-0 rounded-full px-3 py-1.5 font-medium capitalize transition-colors duration-200
			       {option === value ? 'bg-surface text-ink shadow-sm' : 'text-muted hover:text-ink'}"
		>
			{option}
		</button>
	{/each}
</div>
