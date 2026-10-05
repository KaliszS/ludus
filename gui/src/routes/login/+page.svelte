<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { authApi } from '$lib/api/endpoints';
	import { ApiError } from '$lib/api/http';
	import type { AuthMethods } from '$lib/api/types';
	import Card from '$lib/ui/Card.svelte';
	import Segmented from '$lib/ui/Segmented.svelte';
	import TextField from '$lib/ui/TextField.svelte';
	import { session } from '$lib/state/session.svelte';

	/** Codes from the daemon, whether they came back in the URL or in a response. */
	const MESSAGES: Record<string, string> = {
		cancelled: 'Sign-in was cancelled.',
		registration_closed: 'This instance is not accepting new accounts.',
		pending_approval: 'This account is waiting for an administrator to approve it.',
		sign_in_failed: 'Google could not complete the sign-in. Try again.',
		expired: 'That sign-in took too long or was opened in another tab. Start again.',
		invalid_credentials: 'Email or password is incorrect.',
		too_many_attempts: 'Too many failed attempts. Try again in a few minutes.'
	};

	const MODES = ['sign in', 'create account'] as const;
	type Mode = (typeof MODES)[number];

	let methods = $state<AuthMethods | null>(null);
	let mode = $state<Mode>('sign in');
	let name = $state('');
	let email = $state('');
	let password = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);
	let notice = $state<string | null>(null);

	const urlError = $derived(page.url.searchParams.get('error'));
	const shown = $derived(
		error ?? (urlError ? (MESSAGES[urlError] ?? MESSAGES.sign_in_failed) : null)
	);
	const creating = $derived(mode === 'create account');

	onMount(async () => {
		try {
			methods = await authApi.methods();
		} catch {
			error = 'Cannot reach the daemon.';
		}
	});

	function explain(cause: unknown): string {
		if (!(cause instanceof ApiError)) return String(cause);
		if (cause.code === 'conflict') return 'This email is already registered. Sign in instead.';
		if (cause.field === 'password') return 'Use at least 10 characters.';
		if (cause.field === 'email') return 'That does not look like an email address.';
		return MESSAGES[cause.code] ?? cause.message;
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		error = notice = null;
		try {
			if (!creating) {
				await session.signInWithPassword(email, password);
				await goto('/', { replaceState: true });
			} else if ((await session.register(email, password, name)) === 'signed-in') {
				await goto('/', { replaceState: true });
			} else {
				notice = 'Account created. An administrator needs to approve it before you can sign in.';
				mode = 'sign in';
				password = '';
			}
		} catch (cause) {
			error = explain(cause);
		} finally {
			busy = false;
		}
	}

	async function google() {
		busy = true;
		await session.signIn('google');
	}
</script>

<div class="grid min-h-[60dvh] place-items-center">
	<Card class="w-full max-w-sm">
		<h1 class="text-center text-xl font-semibold tracking-tight">
			{creating ? 'Create account' : 'Sign in'}
		</h1>
		<p class="mt-1.5 text-center text-sm text-muted">Habits, check-ins and the plans they fill.</p>

		{#if methods && methods.registration !== 'closed'}
			<div class="mt-5 flex justify-center">
				<Segmented
					options={MODES}
					value={mode}
					onchange={(next) => {
						mode = next;
						error = notice = null;
					}}
				/>
			</div>
		{/if}

		{#if shown}
			<p class="mt-5 rounded-xl bg-accent-soft px-3.5 py-2.5 text-sm text-accent" role="alert">
				{shown}
			</p>
		{:else if notice}
			<p class="mt-5 rounded-xl bg-good-soft px-3.5 py-2.5 text-sm text-good" role="status">
				{notice}
			</p>
		{/if}

		<form class="mt-5 space-y-2.5" onsubmit={submit}>
			{#if creating}
				<TextField bind:value={name} placeholder="Name (optional)" autocomplete="name" />
			{/if}
			<TextField
				bind:value={email}
				type="email"
				placeholder="Email"
				autocomplete="email"
				required
			/>
			<TextField
				bind:value={password}
				type="password"
				placeholder={creating ? 'Password, at least 10 characters' : 'Password'}
				autocomplete={creating ? 'new-password' : 'current-password'}
				required
			/>
			<button
				type="submit"
				disabled={busy}
				class="w-full rounded-full bg-accent px-4 py-2.5 text-sm font-medium text-white transition
				       duration-200 hover:brightness-110 active:scale-[0.98] disabled:opacity-60"
			>
				{busy ? 'One moment…' : creating ? 'Create account' : 'Sign in'}
			</button>
		</form>

		{#if methods?.providers.includes('google')}
			<div class="my-5 flex items-center gap-3 text-xs text-muted">
				<span class="h-px flex-1 bg-line"></span>or<span class="h-px flex-1 bg-line"></span>
			</div>
			<button
				type="button"
				onclick={google}
				disabled={busy}
				class="flex w-full items-center justify-center gap-3 rounded-full border border-line
				       bg-surface px-4 py-2.5 text-sm font-medium transition duration-200
				       hover:bg-sunken active:scale-[0.98] disabled:opacity-60"
			>
				<svg viewBox="0 0 48 48" class="size-[18px]" aria-hidden="true">
					<path
						fill="#EA4335"
						d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"
					/>
					<path
						fill="#4285F4"
						d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"
					/>
					<path
						fill="#FBBC05"
						d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"
					/>
					<path
						fill="#34A853"
						d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"
					/>
				</svg>
				Continue with Google
			</button>
		{/if}
	</Card>
</div>
