const VERIFIER_KEY = 'ludus.pkce';

const base64url = (bytes: Uint8Array) =>
	btoa(String.fromCharCode(...bytes))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=+$/, '');

/** RFC 7636 S256. The verifier never leaves this tab; only its digest is sent. */
export async function createChallenge(): Promise<string> {
	const verifier = base64url(crypto.getRandomValues(new Uint8Array(32)));
	sessionStorage.setItem(VERIFIER_KEY, verifier);
	const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier));
	return base64url(new Uint8Array(digest));
}

/** Read once: a verifier that could be replayed is no verifier at all. */
export function takeVerifier(): string | null {
	const verifier = sessionStorage.getItem(VERIFIER_KEY);
	sessionStorage.removeItem(VERIFIER_KEY);
	return verifier;
}
