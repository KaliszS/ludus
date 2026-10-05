import { authApi } from '$lib/api/endpoints';
import { ApiError } from '$lib/api/http';
import type { SessionGrant, User } from '$lib/api/types';
import { createChallenge, takeVerifier } from '$lib/auth/pkce';
import { leave, token } from '$lib/auth/token';

class Session {
	user = $state<User | null>(null);
	/** False until the stored token has been checked, so nothing renders as signed out by mistake. */
	ready = $state(false);
	/** Set when the daemon cannot be asked at all: that is not the same as being signed out. */
	failure = $state<string | null>(null);

	async restore() {
		if (token.get()) {
			try {
				this.user = await authApi.me();
			} catch (error) {
				if (!(error instanceof ApiError && error.code === 'unauthenticated')) {
					this.failure = error instanceof Error ? error.message : String(error);
				}
			}
		}
		this.ready = true;
	}

	async signIn(provider = 'google') {
		const challenge = await createChallenge();
		location.assign(authApi.startUrl(provider, `${location.origin}/auth/callback`, challenge));
	}

	async complete(code: string) {
		const verifier = takeVerifier();
		if (!verifier) throw new ApiError('no_verifier', 'This sign-in was started in another tab.');
		this.accept(await authApi.redeem(code, verifier));
	}

	async signInWithPassword(email: string, password: string) {
		this.accept(await authApi.passwordLogin(email, password));
	}

	async register(email: string, password: string, name: string): Promise<'signed-in' | 'pending'> {
		const result = await authApi.register(email, password, name);
		if ('pending' in result) return 'pending';
		this.accept(result);
		return 'signed-in';
	}

	/** Every other session ends server-side; this one carries on with the new token. */
	async changePassword(current: string | null, next: string) {
		this.accept(await authApi.changePassword(current, next));
	}

	private accept(grant: SessionGrant) {
		token.set(grant.token);
		this.user = grant.user;
	}

	async signOut() {
		try {
			await authApi.logout();
		} finally {
			token.clear();
			leave();
		}
	}
}

export const session = new Session();
