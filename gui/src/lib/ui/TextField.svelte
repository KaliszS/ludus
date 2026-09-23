<script lang="ts">
	interface Props {
		value: string;
		placeholder?: string;
		type?: 'text' | 'number';
		required?: boolean;
		min?: string;
		step?: string;
		class?: string;
		onenter?: (value: string) => void;
		onchange?: (value: string) => void;
	}

	let {
		value = $bindable(),
		placeholder,
		type = 'text',
		required,
		min,
		step,
		class: extra = 'w-full',
		onenter,
		onchange
	}: Props = $props();
</script>

<input
	{type}
	{placeholder}
	{required}
	{min}
	{step}
	bind:value
	onkeydown={(event) => {
		if (event.key === 'Enter' && onenter) {
			event.preventDefault();
			onenter(event.currentTarget.value);
		}
	}}
	onchange={(event) => onchange?.(event.currentTarget.value)}
	class="rounded-xl border border-line bg-canvas px-3.5 py-2.5 text-sm transition
	       outline-none placeholder:text-muted/60 focus:border-accent {extra}"
/>
