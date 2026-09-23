import { checkinsApi, levelsApi, planApi } from '$lib/api/endpoints';
import type { Habit, Period, Plan } from '$lib/api/types';
import { shift, today, weekOf } from '$lib/domain/date';
import { habits } from './habits.svelte';
import { toast } from './toast.svelte';

const PERIOD_ORDER: Period[] = ['day', 'week', 'month', 'quarter', 'year'];

class Board {
	selectedDay = $state(today());
	/** -1 when moving back in time, +1 forward; drives which way the list slides in. */
	direction = $state(1);
	/** habit id -> day -> amount */
	counts = $state<Record<string, Record<string, number>>>({});
	plans = $state<Plan[]>([]);
	loading = $state(true);

	week = $derived(weekOf(this.selectedDay));
	active = $derived(habits.active);

	done(habitId: string, day = this.selectedDay) {
		return this.counts[habitId]?.[day] ?? 0;
	}

	/** How many habits were touched on a day, for the dots under the day picker. */
	activityOn(day: string) {
		return this.active.filter((habit) => this.done(habit.id, day) > 0).length;
	}

	async load() {
		this.loading = true;
		await habits.load();
		await Promise.all([this.loadCheckins(), this.loadPlans()]);
		this.loading = false;
	}

	async selectDay(day: string) {
		this.direction = day < this.selectedDay ? -1 : 1;
		this.selectedDay = day;
		await Promise.all([this.loadCheckins(), this.loadPlans()]);
	}

	/** Nothing to see in the future, so stepping forward stops at today. */
	shiftDay(days: number) {
		const target = shift(this.selectedDay, days);
		if (target > today()) return Promise.resolve();
		return this.selectDay(target);
	}

	/** Landing on a future day would show an empty board, so clamp to today. */
	shiftWeek(weeks: number) {
		const target = shift(this.selectedDay, weeks * 7);
		return this.selectDay(target > today() ? today() : target);
	}

	private async loadCheckins() {
		const week = weekOf(this.selectedDay);
		const rows = await toast.guard(() => checkinsApi.all(week[0], week[6]));
		if (!rows) return;

		const counts: Record<string, Record<string, number>> = {};
		for (const row of rows) {
			(counts[row.habit_id] ??= {})[row.day] = row.times;
		}
		this.counts = counts;
	}

	private async loadPlans() {
		const levels = await toast.guard(() => levelsApi.list());
		if (!levels) return;

		const periods = PERIOD_ORDER.filter((period) =>
			levels.some((level) => level.period === period)
		);
		const plans = await toast.guard(() =>
			Promise.all(periods.map((period) => planApi.get(period, this.selectedDay)))
		);
		if (plans) this.plans = plans;
	}

	private write(habitId: string, day: string, times: number) {
		const forHabit = { ...(this.counts[habitId] ?? {}) };
		if (times === 0) delete forHabit[day];
		else forHabit[day] = times;
		this.counts = { ...this.counts, [habitId]: forHabit };
	}

	/** Zero means "no check-in", which the daemon expresses by the row being absent. */
	async set(habit: Habit, day: string, times: number) {
		const next = Math.max(0, Math.round(times));
		const previous = this.done(habit.id, day);
		if (next === previous) return;

		// Applied straight away so a tap feels instant; rolled back if the write fails.
		this.write(habit.id, day, next);

		const ok = await toast.guard(async () => {
			if (next === 0) await checkinsApi.clear(habit.id, day);
			else await checkinsApi.set(habit.id, day, next);
			return true;
		});

		if (!ok) {
			this.write(habit.id, day, previous);
			return;
		}
		await this.loadPlans();
	}

	toggle(habit: Habit, day: string) {
		return this.set(habit, day, this.done(habit.id, day) ? 0 : 1);
	}
}

export const board = new Board();
