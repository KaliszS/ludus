interface Handlers {
	onprevious: () => void;
	onnext: () => void;
}

const DISTANCE = 60;
/** Horizontal has to clearly beat vertical, or scrolling would flip the day. */
const DOMINANCE = 1.5;

export function swipe(node: HTMLElement, handlers: Handlers) {
	let startX = 0;
	let startY = 0;
	let tracking = false;
	let current = handlers;

	function suppressClick() {
		const cancel = (event: Event) => {
			event.stopPropagation();
			event.preventDefault();
		};
		node.addEventListener('click', cancel, { capture: true, once: true });
		// The gesture may end on empty space, where no click follows at all.
		setTimeout(() => node.removeEventListener('click', cancel, { capture: true }), 0);
	}

	function down(event: PointerEvent) {
		if (event.pointerType === 'mouse') return;
		startX = event.clientX;
		startY = event.clientY;
		tracking = true;
	}

	function up(event: PointerEvent) {
		if (!tracking) return;
		tracking = false;

		const dx = event.clientX - startX;
		const dy = event.clientY - startY;
		if (Math.abs(dx) < DISTANCE || Math.abs(dx) < Math.abs(dy) * DOMINANCE) return;

		suppressClick();
		if (dx < 0) current.onnext();
		else current.onprevious();
	}

	node.addEventListener('pointerdown', down, { passive: true });
	node.addEventListener('pointerup', up, { passive: true });
	node.addEventListener('pointercancel', () => (tracking = false), { passive: true });

	return {
		update: (next: Handlers) => (current = next),
		destroy() {
			node.removeEventListener('pointerdown', down);
			node.removeEventListener('pointerup', up);
		}
	};
}
