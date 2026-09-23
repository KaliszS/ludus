import { habitsApi } from '$lib/api/endpoints';
import type { Habit, HabitPatch, NewHabit } from '$lib/api/types';
import { toast } from './toast.svelte';

class Habits {
	items = $state<Habit[]>([]);
	includeArchived = $state(false);
	loading = $state(false);

	active = $derived(this.items.filter((habit) => !habit.archived));

	async load() {
		this.loading = true;
		const items = await toast.guard(() => habitsApi.list(this.includeArchived));
		if (items) this.items = items;
		this.loading = false;
	}

	async create(input: NewHabit) {
		const created = await toast.guard(() => habitsApi.create(input));
		if (created) await this.load();
		return created;
	}

	async update(id: string, patch: HabitPatch) {
		const updated = await toast.guard(() => habitsApi.update(id, patch));
		if (updated) await this.load();
	}

	/** Positions are spaced so a later single insert does not need a full renumber. */
	async reorder(ordered: Habit[]) {
		this.items = ordered;

		const changed = ordered
			.map((habit, index) => ({ habit, position: (index + 1) * 100 }))
			.filter((entry) => entry.habit.position !== entry.position);
		if (changed.length === 0) return;

		const ok = await toast.guard(() =>
			Promise.all(
				changed.map((entry) => habitsApi.update(entry.habit.id, { position: entry.position }))
			)
		);
		if (ok) await this.load();
	}

	async remove(id: string) {
		const ok = await toast.guard(() => habitsApi.remove(id).then(() => true));
		if (ok) await this.load();
	}
}

export const habits = new Habits();
