import type { CategoryPatch, Habit, HabitCategory, HabitPatch } from '$lib/api/types';

/** One category as the habits page lays it out, with its habits in order. */
export interface Shelf {
	id: string;
	category: HabitCategory;
	habits: Habit[];
	/** Subcategories; always empty below the top level. */
	subs: Shelf[];
}

export interface Layout {
	shelves: Shelf[];
	/** Habits outside any category, shown last. */
	loose: Habit[];
}

const byPosition = (a: { position: number }, b: { position: number }) => a.position - b.position;

export function shelve(categories: HabitCategory[], habits: Habit[]): Layout {
	const known = new Set(categories.map((category) => category.id));
	const filedUnder = (habit: Habit) =>
		habit.category_id && known.has(habit.category_id) ? habit.category_id : null;
	const habitsIn = (id: string | null) =>
		habits.filter((habit) => filedUnder(habit) === id).sort(byPosition);
	const shelf = (category: HabitCategory): Shelf => ({
		id: category.id,
		category,
		habits: habitsIn(category.id),
		subs: categories
			.filter((child) => child.parent_id === category.id)
			.sort(byPosition)
			.map(shelf)
	});

	return {
		shelves: categories
			.filter((category) => category.parent_id === null)
			.sort(byPosition)
			.map(shelf),
		loose: habitsIn(null)
	};
}

/** A run of habits under one label, for views that list habits flat rather than as a tree. */
export interface HabitGroup {
	id: string;
	/** null only when nothing is categorized, so a lone group needs no label. */
	label: string | null;
	nested: boolean;
	habits: Habit[];
}

/** One group per category and subcategory that holds any of `habits`, in page order,
 *  then the uncategorized ones as "Other". */
export function groups(categories: HabitCategory[], habits: Habit[]): HabitGroup[] {
	const { shelves, loose } = shelve(categories, habits);
	const runs: HabitGroup[] = shelves.flatMap((top) => [
		{ id: top.id, label: top.category.name, nested: false, habits: top.habits },
		...top.subs.map((sub) => ({
			id: sub.id,
			label: sub.category.name,
			nested: true,
			habits: sub.habits
		}))
	]);
	const filled = runs.filter((run) => run.habits.length);
	if (loose.length) {
		filled.push({
			id: 'loose',
			label: filled.length ? 'Other' : null,
			nested: false,
			habits: loose
		});
	}
	return filled;
}

/** Every habit in reading order: a category's own habits, then its subcategories'. */
function filed({ shelves, loose }: Layout): { habit: Habit; category: string | null }[] {
	const walk = (shelf: Shelf): { habit: Habit; category: string | null }[] => [
		...shelf.habits.map((habit) => ({ habit, category: shelf.id })),
		...shelf.subs.flatMap(walk)
	];
	return [...shelves.flatMap(walk), ...loose.map((habit) => ({ habit, category: null }))];
}

/** The patches that make the stored order match the page. Habit positions run across
 *  the whole list, so every other view that sorts by position follows the categories. */
export function layoutChanges(layout: Layout) {
	const categories: { id: string; patch: CategoryPatch }[] = [];
	layout.shelves.forEach((top, index) => {
		const place = (category: HabitCategory, parent: string | null, position: number) => {
			const patch: CategoryPatch = {};
			if (category.parent_id !== parent) patch.parent_id = parent;
			if (category.position !== position) patch.position = position;
			if (Object.keys(patch).length) categories.push({ id: category.id, patch });
		};
		place(top.category, null, (index + 1) * 100);
		top.subs.forEach((sub, subIndex) => place(sub.category, top.id, (subIndex + 1) * 100));
	});

	const habits: { id: string; patch: HabitPatch }[] = [];
	filed(layout).forEach(({ habit, category }, index) => {
		const patch: HabitPatch = {};
		const position = (index + 1) * 100;
		if (habit.category_id !== category) patch.category_id = category;
		if (habit.position !== position) patch.position = position;
		if (Object.keys(patch).length) habits.push({ id: habit.id, patch });
	});

	return { categories, habits };
}
