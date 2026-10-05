import { ApiError } from '$lib/api/http';

class Toast {
	message = $state<string | null>(null);
	private timer: ReturnType<typeof setTimeout> | undefined;

	show(message: string) {
		this.message = message;
		clearTimeout(this.timer);
		// Long enough to read: four seconds, or about 50 ms a character for longer ones.
		this.timer = setTimeout(() => (this.message = null), Math.max(4000, message.length * 50));
	}

	dismiss() {
		this.message = null;
		clearTimeout(this.timer);
	}

	/** Every mutation funnels through here so pages carry no try/catch of their own. */
	async guard<T>(action: () => Promise<T>): Promise<T | undefined> {
		try {
			return await action();
		} catch (error) {
			// The request layer is already sending the browser to sign in.
			if (error instanceof ApiError && error.code === 'unauthenticated') return undefined;
			this.show(error instanceof ApiError ? error.message : String(error));
			return undefined;
		}
	}
}

export const toast = new Toast();
