import { ApiError } from '$lib/api/http';

class Toast {
	message = $state<string | null>(null);
	private timer: ReturnType<typeof setTimeout> | undefined;

	show(message: string) {
		this.message = message;
		clearTimeout(this.timer);
		this.timer = setTimeout(() => (this.message = null), 4000);
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
			this.show(error instanceof ApiError ? error.message : String(error));
			return undefined;
		}
	}
}

export const toast = new Toast();
