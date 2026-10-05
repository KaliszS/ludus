import { PUBLIC_DEFAULT_API_URL } from '$env/static/public';
import { leave, token } from '$lib/auth/token';

export const BASE = `${PUBLIC_DEFAULT_API_URL || 'http://127.0.0.1:7530'}/v1`;

export class ApiError extends Error {
	constructor(
		readonly code: string,
		message: string,
		readonly field?: string
	) {
		super(message);
	}
}

export async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
	const headers = new Headers(init.headers);
	if (init.body) headers.set('content-type', 'application/json');
	const session = token.get();
	if (session) headers.set('authorization', `Bearer ${session}`);

	let res: Response;
	try {
		res = await fetch(BASE + path, { ...init, headers });
	} catch {
		throw new ApiError('offline', 'Cannot reach the daemon');
	}

	const text = await res.text();
	const body = text ? JSON.parse(text) : null;

	// A wrong password is a 401 too; only a dead session means leaving the page.
	if (res.status === 401 && body?.error?.code === 'unauthenticated') {
		token.clear();
		leave();
	}
	if (!res.ok) {
		const err = body?.error;
		throw new ApiError(err?.code ?? 'internal', err?.message ?? res.statusText, err?.field);
	}
	return body as T;
}

export const collection = <T>(path: string) => request<{ items: T[] }>(path).then((r) => r.items);

export const body = (value: unknown) => JSON.stringify(value);
