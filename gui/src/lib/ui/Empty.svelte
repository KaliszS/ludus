<script lang="ts">
	interface Props {
		text: string;
		href?: string;
		action?: string;
	}
	let { text, href, action }: Props = $props();
</script>

<p class="py-8 text-center text-sm text-muted">
	{text}
	{#if href && action}
		<a {href} class="font-medium text-accent hover:underline">{action}</a>
	{/if}
</p>
