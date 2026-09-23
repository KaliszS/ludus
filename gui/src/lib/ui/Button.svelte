<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		variant?: 'primary' | 'ghost' | 'danger';
		type?: 'button' | 'submit';
		disabled?: boolean;
		onclick?: () => void;
		children: Snippet;
	}

	let { variant = 'ghost', type = 'button', disabled, onclick, children }: Props = $props();

	const styles = {
		primary: 'bg-accent text-white hover:brightness-110',
		ghost: 'text-muted hover:text-ink hover:bg-sunken',
		danger: 'text-muted hover:text-accent hover:bg-accent-soft'
	};
</script>

<button
	{type}
	{disabled}
	{onclick}
	class="shrink-0 rounded-full px-4 py-2 text-sm font-medium transition duration-200
	       active:scale-95 disabled:pointer-events-none disabled:opacity-40 {styles[variant]}"
>
	{@render children()}
</button>
