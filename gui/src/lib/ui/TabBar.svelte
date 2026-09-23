<script lang="ts">
	import { page } from '$app/state';
	import { ChartNoAxesColumn, LayoutDashboard, Repeat, Target } from '@lucide/svelte';

	const tabs = [
		{ href: '/', label: 'Dashboard', icon: LayoutDashboard },
		{ href: '/habits', label: 'Habits', icon: Repeat },
		{ href: '/plans', label: 'Plans', icon: Target },
		{ href: '/stats', label: 'Stats', icon: ChartNoAxesColumn }
	];
</script>

<nav
	class="fixed inset-x-0 bottom-0 z-40 border-t border-line bg-surface/85
	       pb-[env(safe-area-inset-bottom)] backdrop-blur-xl sm:hidden"
>
	<ul class="mx-auto flex max-w-lg">
		{#each tabs as tab (tab.href)}
			{@const active = page.url.pathname === tab.href}
			{@const Glyph = tab.icon}
			<li class="flex-1">
				<a
					href={tab.href}
					aria-current={active ? 'page' : undefined}
					class="flex flex-col items-center gap-1 py-2.5 transition-colors duration-200
					       {active ? 'text-accent' : 'text-muted'}"
				>
					<Glyph size={22} strokeWidth={active ? 2 : 1.75} />
					<span class="text-[11px] font-medium">{tab.label}</span>
				</a>
			</li>
		{/each}
	</ul>
</nav>
