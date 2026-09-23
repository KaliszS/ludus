<script lang="ts">
	import { Minus, Plus } from '@lucide/svelte';

	interface Props {
		value: number;
		unit?: string | null;
		accent: string;
		onset: (value: number) => void;
	}
	let { value, unit, accent, onset }: Props = $props();

	let editing = $state(false);
	let draft = $state('');
	let input = $state<HTMLInputElement | null>(null);

	function open() {
		draft = String(value);
		editing = true;
	}

	function commit() {
		if (!editing) return;
		editing = false;
		const next = Number(draft);
		if (Number.isFinite(next) && next >= 0) onset(Math.round(next));
	}

	$effect(() => {
		if (editing && input) {
			input.focus();
			input.select();
		}
	});
</script>

<div class="flex items-center gap-0.5 rounded-full border border-line bg-canvas p-0.5">
	<button
		onclick={() => onset(value - 1)}
		disabled={value === 0 || editing}
		aria-label="Decrease"
		class="grid size-7 place-items-center rounded-full text-muted transition hover:bg-sunken
		       hover:text-ink active:scale-90 disabled:pointer-events-none disabled:opacity-25"
	>
		<Minus size={14} strokeWidth={2} />
	</button>

	{#if editing}
		<input
			bind:this={input}
			bind:value={draft}
			type="number"
			min="0"
			inputmode="numeric"
			onblur={commit}
			onkeydown={(event) => {
				if (event.key === 'Enter') commit();
				if (event.key === 'Escape') editing = false;
			}}
			class="min-w-11 rounded-full bg-transparent text-center text-sm font-semibold text-ink
			       tabular-nums outline-none"
			style:width="{Math.max(draft.length, 2) + 1}ch"
		/>
	{:else}
		<!-- Tapping the number is the way in; stepping one at a time is painful at 45. -->
		<button
			onclick={open}
			aria-label="Set exact amount"
			class="min-w-11 rounded-full px-1 text-center text-sm font-semibold tabular-nums transition"
			style:color={value > 0 ? accent : 'var(--color-muted)'}
		>
			{value}{unit ? ` ${unit.slice(0, 4)}` : ''}
		</button>
	{/if}

	<button
		onclick={() => onset(value + 1)}
		disabled={editing}
		aria-label="Increase"
		class="grid size-7 place-items-center rounded-full text-muted transition hover:bg-sunken
		       hover:text-ink active:scale-90 disabled:pointer-events-none disabled:opacity-25"
	>
		<Plus size={14} strokeWidth={2} />
	</button>
</div>
