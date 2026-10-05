import type { Habit, Measure } from '$lib/api/types';

/** What a quota counts: days for occurrences, the habit's own unit for a single
 *  amount, and nothing for a group adding up habits that may not share one. */
export function quotaUnit(measure: Measure, quota: number, habits: Habit[]): string | null {
	if (measure === 'occurrences') return quota === 1 ? 'day' : 'days';
	return habits.length === 1 ? habits[0].unit : null;
}
