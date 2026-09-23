import type { Habit, UserCheckin } from '$lib/api/types';
import { shift, today, weekOf } from './date';

export interface HabitStat {
	habit: Habit;
	total: number;
	days: number;
	/** Consecutive days, meaningful for habits meant to be daily. */
	dayStreak: number;
	bestDayStreak: number;
	/** Consecutive weeks with any activity, meaningful whatever the cadence. */
	weekStreak: number;
	bestWeekStreak: number;
	perWeek: number;
}

const byHabit = (rows: UserCheckin[]) => {
	const map = new Map<string, Set<string>>();
	const sums = new Map<string, number>();
	for (const row of rows) {
		(map.get(row.habit_id) ?? map.set(row.habit_id, new Set()).get(row.habit_id)!).add(row.day);
		sums.set(row.habit_id, (sums.get(row.habit_id) ?? 0) + row.times);
	}
	return { map, sums };
};

/** Weeks, not days: a habit kept on Mondays and Thursdays never has consecutive days,
 * so a day streak would read as 1 forever. The app thinks in periods anyway. */
function weekStreak(days: Set<string>): number {
	const weeks = new Set([...days].map((day) => weekOf(day)[0]));
	let cursor = weekOf(today())[0];
	if (!weeks.has(cursor)) cursor = shift(cursor, -7);

	let length = 0;
	while (weeks.has(cursor)) {
		length += 1;
		cursor = shift(cursor, -7);
	}
	return length;
}

/** Counts back from today; a gap today does not break a streak that ran until yesterday. */
function dayStreak(days: Set<string>): number {
	let cursor = days.has(today()) ? today() : shift(today(), -1);
	let length = 0;
	while (days.has(cursor)) {
		length += 1;
		cursor = shift(cursor, -1);
	}
	return length;
}

function longestDayRun(days: Set<string>): number {
	const sorted = [...days].sort();
	let best = 0;
	let run = 0;
	let previous = '';
	for (const day of sorted) {
		run = previous && shift(previous, 1) === day ? run + 1 : 1;
		best = Math.max(best, run);
		previous = day;
	}
	return best;
}

function longestWeekRun(days: Set<string>): number {
	const weeks = [...new Set([...days].map((day) => weekOf(day)[0]))].sort();
	let best = 0;
	let run = 0;
	let previous = '';
	for (const week of weeks) {
		run = previous && shift(previous, 7) === week ? run + 1 : 1;
		best = Math.max(best, run);
		previous = week;
	}
	return best;
}

/**
 * Counts follow the chart's window so the two agree; streaks come from the full
 * history, because a streak capped by the chart's zoom would simply be wrong.
 */
export function habitStats(
	habits: Habit[],
	all: UserCheckin[],
	visible: UserCheckin[],
	spanDays: number
): HabitStat[] {
	const full = byHabit(all);
	const window = byHabit(visible);
	const weeks = Math.max(spanDays / 7, 1);

	return habits.map((habit) => {
		const everyDay = full.map.get(habit.id) ?? new Set<string>();
		const spanDaysSet = window.map.get(habit.id) ?? new Set<string>();
		return {
			habit,
			total: window.sums.get(habit.id) ?? 0,
			days: spanDaysSet.size,
			dayStreak: dayStreak(everyDay),
			bestDayStreak: longestDayRun(everyDay),
			weekStreak: weekStreak(everyDay),
			bestWeekStreak: longestWeekRun(everyDay),
			perWeek: Math.round((spanDaysSet.size / weeks) * 10) / 10
		};
	});
}

/** Day -> how many distinct habits were touched, which is what the heatmap shades. */
export function dailyActivity(rows: UserCheckin[]): Map<string, number> {
	const counts = new Map<string, number>();
	for (const row of rows) {
		counts.set(row.day, (counts.get(row.day) ?? 0) + 1);
	}
	return counts;
}
