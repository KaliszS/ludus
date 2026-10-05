<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { LogOut } from '@lucide/svelte';
	import IconButton from '$lib/ui/IconButton.svelte';
	import TabBar from '$lib/ui/TabBar.svelte';
	import Toast from '$lib/ui/Toast.svelte';
	import { session } from '$lib/state/session.svelte';

	let { children } = $props();

	/** Everything else needs an account; these are how you get one. */
	const PUBLIC = ['/login', '/auth/callback'];
	const isPublic = $derived(PUBLIC.includes(page.url.pathname));

	onMount(() => session.restore());

	$effect(() => {
		if (!session.ready || session.failure) return;
		if (!session.user && !isPublic) goto('/login', { replaceState: true });
		else if (session.user && page.url.pathname === '/login') goto('/', { replaceState: true });
	});

	const initial = $derived(session.user?.display_name.trim().charAt(0).toUpperCase() ?? '');

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

		{#if session.user}
			<div class="flex items-center gap-2">
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

				{#if session.user.avatar_url}
					<img
						src={session.user.avatar_url}
						alt=""
						referrerpolicy="no-referrer"
						class="size-8 rounded-full border border-line"
						title={session.user.email}
					/>
				{:else}
					<span
						class="grid size-8 place-items-center rounded-full bg-accent-soft text-xs font-semibold text-accent"
						title={session.user.email}
					>
						{initial}
					</span>
				{/if}
				<IconButton icon={LogOut} label="Sign out" onclick={() => session.signOut()} />
			</div>
		{/if}
	</header>

	<main class="flex-1 pb-28 sm:pb-16">
		{#if session.failure}
			<p class="py-24 text-center text-sm text-muted">{session.failure}</p>
		{:else if session.ready && (session.user || isPublic)}
			{@render children()}
		{/if}
	</main>
</div>

{#if session.user}
	<TabBar />
{/if}
<Toast />
