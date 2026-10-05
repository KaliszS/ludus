const KEY = 'ludus.session';

/** Storage can be missing or throw (private windows, blocked site data), so every
 *  access is guarded and a failure simply means "signed out". */
export const token = {
	get(): string | null {
		try {
			return localStorage.getItem(KEY);
		} catch {
			return null;
		}
	},
	set(value: string) {
		try {
			localStorage.setItem(KEY, value);
		} catch {
			/* the session lasts until reload */
		}
	},
	clear() {
		try {
			localStorage.removeItem(KEY);
		} catch {
			/* nothing to clear */
		}
	}
};

/** Sends the browser to the sign-in page as a full load, so no store keeps the
 *  previous account's data in memory. */
export function leave(path = '/login') {
	if (location.pathname !== path) location.assign(path);
}
