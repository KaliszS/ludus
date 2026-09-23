<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { page } from '$app/state';
	import TabBar from '$lib/ui/TabBar.svelte';
	import Toast from '$lib/ui/Toast.svelte';

	let { children } = $props();

	const links = [
		{ href: '/', label: 'Dashboard' },
		{ href: '/habits', label: 'Habits' },
		{ href: '/plans', label: 'Plans' },
		{ href: '/stats', label: 'Stats' }
	];

	const title = $derived(links.find((link) => link.href === page.url.pathname)?.label ?? 'Ludus');
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{title} · Ludus</title>
</svelte:head>

<div class="mx-auto flex min-h-dvh max-w-2xl flex-col px-4 pt-[env(safe-area-inset-top)]">
	<header class="flex items-center justify-between gap-4 py-5 sm:py-7">
		<a href="/" class="text-lg font-semibold tracking-tight">
			Ludus<span class="text-accent">.</span>
		</a>

		<nav class="hidden gap-0.5 rounded-full border border-line bg-sunken p-0.5 text-xs sm:flex">
			{#each links as link (link.href)}
				<a
					href={link.href}
					aria-current={page.url.pathname === link.href ? 'page' : undefined}
					class="rounded-full px-3.5 py-1.5 font-medium transition-colors duration-200
					       {page.url.pathname === link.href
						? 'bg-surface text-ink shadow-sm'
						: 'text-muted hover:text-ink'}"
				>
					{link.label}
				</a>
			{/each}
		</nav>
	</header>

	<main class="flex-1 pb-28 sm:pb-16">
		{@render children()}
	</main>
</div>

<TabBar />
<Toast />
