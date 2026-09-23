import { PUBLIC_DEFAULT_API_URL } from '$env/static/public';

const BASE = `${PUBLIC_DEFAULT_API_URL || 'http://127.0.0.1:7530'}/v1`;

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

	let res: Response;
	try {
		res = await fetch(BASE + path, { ...init, headers });
	} catch {
		throw new ApiError('offline', 'Cannot reach the daemon');
	}

	const text = await res.text();
	const body = text ? JSON.parse(text) : null;

	if (!res.ok) {
		const err = body?.error;
		throw new ApiError(err?.code ?? 'internal', err?.message ?? res.statusText, err?.field);
	}
	return body as T;
}

export const collection = <T>(path: string) => request<{ items: T[] }>(path).then((r) => r.items);

export const body = (value: unknown) => JSON.stringify(value);
