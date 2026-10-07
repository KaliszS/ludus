import { categoriesApi, habitsApi } from '$lib/api/endpoints';
import type { Habit, HabitCategory, HabitPatch, NewHabit } from '$lib/api/types';
import { layoutChanges, type Layout } from '$lib/domain/categories';
import { toast } from './toast.svelte';

class Habits {
	items = $state<Habit[]>([]);
	categories = $state<HabitCategory[]>([]);
	includeArchived = $state(false);
	loading = $state(false);

	active = $derived(this.items.filter((habit) => !habit.archived));

	async load() {
		this.loading = true;
		// Apart, so a failing category request cannot leave the habits looking gone.
		const [items, categories] = await Promise.all([
			toast.guard(() => habitsApi.list(this.includeArchived)),
			toast.guard(() => categoriesApi.list())
		]);
		if (items) this.items = items;
		if (categories) this.categories = categories;
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
		return updated;
	}

	async remove(id: string) {
		const ok = await toast.guard(() => habitsApi.remove(id).then(() => true));
		if (ok) await this.load();
	}

	async addCategory(name: string, parentId: string | null) {
		const created = await toast.guard(() => categoriesApi.create(name, parentId));
		if (created) await this.load();
		return created;
	}

	async renameCategory(id: string, name: string) {
		const renamed = await toast.guard(() => categoriesApi.update(id, { name }));
		await this.load();
		return renamed;
	}

	async removeCategory(id: string) {
		const ok = await toast.guard(() => categoriesApi.remove(id).then(() => true));
		if (ok) await this.load();
	}

	/** Stores the order and filing the page shows after a drag. Categories go first,
	 *  so a subcategory has its new parent before anything else depends on it. */
	async saveLayout(layout: Layout) {
		const { categories, habits } = layoutChanges(layout);
		if (categories.length === 0 && habits.length === 0) return;

		await toast.guard(async () => {
			await Promise.all(categories.map(({ id, patch }) => categoriesApi.update(id, patch)));
			await Promise.all(habits.map(({ id, patch }) => habitsApi.update(id, patch)));
		});
		await this.load();
	}
}

export const habits = new Habits();
