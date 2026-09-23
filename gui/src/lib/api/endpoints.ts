import { body, collection, request } from './http';
import type {
	Checkin,
	Measure,
	Habit,
	HabitPatch,
	NewHabit,
	Period,
	Plan,
	PlanLevel,
	PeriodOutcome,
	Requirement,
	UserCheckin
} from './types';

export const habitsApi = {
	list: (archived = false) => collection<Habit>(`/habits?archived=${archived}`),
	create: (habit: NewHabit) => request<Habit>('/habits', { method: 'POST', body: body(habit) }),
	update: (id: string, patch: HabitPatch) =>
		request<Habit>(`/habits/${id}`, { method: 'PATCH', body: body(patch) }),
	remove: (id: string) => request<void>(`/habits/${id}`, { method: 'DELETE' })
};

export const checkinsApi = {
	/** Every habit at once, so a board needs one request instead of one per habit. */
	all: (from: string, to: string) => collection<UserCheckin>(`/checkins?from=${from}&to=${to}`),
	list: (habitId: string, from?: string, to?: string) => {
		const query = new URLSearchParams();
		if (from) query.set('from', from);
		if (to) query.set('to', to);
		return collection<Checkin>(`/habits/${habitId}/checkins?${query}`);
	},
	set: (habitId: string, day: string, times?: number) =>
		request<Checkin>(`/habits/${habitId}/checkins/${day}`, {
			method: 'PUT',
			body: times === undefined ? undefined : body({ times })
		}),
	clear: (habitId: string, day: string) =>
		request<void>(`/habits/${habitId}/checkins/${day}`, { method: 'DELETE' })
};

export const levelsApi = {
	list: (period?: Period) =>
		collection<PlanLevel>(`/plan-levels${period ? `?period=${period}` : ''}`),
	create: (name: string, period: Period) =>
		request<PlanLevel>('/plan-levels', { method: 'POST', body: body({ name, period }) }),
	update: (id: string, patch: { name?: string; period?: Period }) =>
		request<PlanLevel>(`/plan-levels/${id}`, { method: 'PATCH', body: body(patch) }),
	remove: (id: string) => request<void>(`/plan-levels/${id}`, { method: 'DELETE' }),
	requirements: (levelId: string) =>
		collection<Requirement>(`/plan-levels/${levelId}/requirements`),
	addRequirement: (
		levelId: string,
		habitIds: string[],
		quota: number,
		measure: Measure,
		name: string | null
	) =>
		request<Requirement>(`/plan-levels/${levelId}/requirements`, {
			method: 'POST',
			body: body({ habit_ids: habitIds, quota, measure, name })
		}),
	updateRequirement: (
		id: string,
		patch: { habit_ids?: string[]; quota?: number; measure?: Measure; name?: string | null }
	) => request<void>(`/plan-requirements/${id}`, { method: 'PATCH', body: body(patch) }),
	removeRequirement: (id: string) => request<void>(`/plan-requirements/${id}`, { method: 'DELETE' })
};

export const planApi = {
	history: (period: Period, count: number) =>
		collection<PeriodOutcome>(`/plan/history?period=${period}&count=${count}`),
	get: (period: Period, on?: string) => {
		const query = new URLSearchParams({ period });
		if (on) query.set('on', on);
		return request<Plan>(`/plan?${query}`);
	}
};
