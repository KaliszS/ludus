<script lang="ts">
	import { onMount } from 'svelte';
	import { authApi } from '$lib/api/endpoints';
	import { ApiError } from '$lib/api/http';
	import type { Account } from '$lib/api/types';
	import Button from '$lib/ui/Button.svelte';
	import Card from '$lib/ui/Card.svelte';
	import TextField from '$lib/ui/TextField.svelte';
	import { session } from '$lib/state/session.svelte';
	import { toast } from '$lib/state/toast.svelte';

	const LOGIN_NAMES: Record<string, string> = {
		google: 'Google',
		github: 'GitHub',
		password: 'Password'
	};

	let account = $state<Account | null>(null);
	let current = $state('');
	let next = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);

	const hasPassword = $derived(account?.logins.includes('password') ?? false);

	onMount(async () => {
		account = (await toast.guard(() => authApi.me())) ?? null;
	});

	async function save(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		error = null;
		try {
			await session.changePassword(hasPassword ? current : null, next);
			toast.show(
				hasPassword
					? 'Password changed. Other devices were signed out.'
					: 'Password set. You can now sign in with it.'
			);
			current = next = '';
			account = await authApi.me();
		} catch (cause) {
			if (!(cause instanceof ApiError)) error = String(cause);
			else if (cause.field === 'current_password') error = 'Current password is incorrect.';
			else if (cause.field === 'new_password') error = 'Use at least 10 characters.';
			else if (cause.code === 'conflict')
				error = 'Another account already signs in with this email and a password.';
			else error = cause.message;
		} finally {
			busy = false;
		}
	}
</script>

{#if account}
	<div class="space-y-4">
		<Card>
			<div class="flex items-center gap-4">
				{#if account.avatar_url}
					<img
						src={account.avatar_url}
						alt=""
						referrerpolicy="no-referrer"
						class="size-12 rounded-full border border-line"
					/>
				{:else}
					<span
						class="grid size-12 place-items-center rounded-full bg-accent-soft text-lg font-semibold text-accent"
					>
						{account.display_name.trim().charAt(0).toUpperCase()}
					</span>
				{/if}
				<div class="min-w-0 flex-1">
					<p class="truncate font-semibold">{account.display_name}</p>
					<p class="truncate text-sm text-muted">{account.email}</p>
				</div>
				<Button onclick={() => session.signOut()}>Sign out</Button>
			</div>

			<div class="mt-4 flex flex-wrap items-center gap-2 border-t border-line pt-4 text-xs">
				<span class="text-muted">Signs in with</span>
				{#each account.logins as login (login)}
					<span class="rounded-full border border-line bg-sunken px-2.5 py-1 font-medium">
						{LOGIN_NAMES[login] ?? login}
					</span>
				{/each}
			</div>
		</Card>

		<Card>
			<h2 class="font-semibold">{hasPassword ? 'Change password' : 'Set a password'}</h2>
			<p class="mt-1 text-sm text-muted">
				{hasPassword
					? 'Every other device will be signed out.'
					: `Lets you sign in with ${account.email} and a password, not only through Google.`}
			</p>

			{#if error}
				<p class="mt-4 rounded-xl bg-accent-soft px-3.5 py-2.5 text-sm text-accent" role="alert">
					{error}
				</p>
			{/if}

			<form class="mt-4 space-y-2.5" onsubmit={save}>
				<!-- Lets a password manager tie the new password to this account. -->
				<input type="email" value={account.email} autocomplete="username" hidden readonly />
				{#if hasPassword}
					<TextField
						bind:value={current}
						type="password"
						placeholder="Current password"
						autocomplete="current-password"
						required
					/>
				{/if}
				<TextField
					bind:value={next}
					type="password"
					placeholder="New password, at least 10 characters"
					autocomplete="new-password"
					required
				/>
				<div class="flex justify-end">
					<Button type="submit" variant="primary" disabled={busy}>
						{busy ? 'Saving…' : hasPassword ? 'Change password' : 'Set password'}
					</Button>
				</div>
			</form>
		</Card>
	</div>
{/if}
