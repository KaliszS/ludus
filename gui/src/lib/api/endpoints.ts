import { today } from '$lib/domain/date';
import { BASE, body, collection, request } from './http';
import type {
	Account,
	AuthMethods,
	Checkin,
	Measure,
	Medal,
	Habit,
	HabitPatch,
	PlanVersions,
	NewHabit,
	Period,
	PeriodProgress,
	Plan,
	TierQuota,
	PeriodOutcome,
	Requirement,
	SessionGrant,
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

/** Edits land in the period this screen shows, which is the client's today. */
const asOfToday = () => `on=${today()}`;

export const plansApi = {
	list: (archived = false) => collection<Plan>(`/plans?archived=${archived}`),
	create: (name: string, period: Period) =>
		request<Plan>('/plans', { method: 'POST', body: body({ name, period }) }),
	update: (id: string, patch: { name?: string; period?: Period; archived?: boolean }) =>
		request<Plan>(`/plans/${id}?${asOfToday()}`, {
			method: 'PATCH',
			body: body(patch)
		}),
	remove: (id: string) => request<void>(`/plans/${id}`, { method: 'DELETE' }),
	requirements: (planId: string) => collection<Requirement>(`/plans/${planId}/requirements`),
	versions: (planId: string) => request<PlanVersions>(`/plans/${planId}/versions`),
	addTier: (planId: string, medal: Medal | null) =>
		request<Plan>(`/plans/${planId}/tiers`, { method: 'POST', body: body({ medal }) }),
	updateTier: (id: string, patch: { name?: string | null; medal?: Medal | null }) =>
		request<Plan>(`/plan-tiers/${id}`, { method: 'PATCH', body: body(patch) }),
	retireTier: (id: string) =>
		request<Plan>(`/plan-tiers/${id}?${asOfToday()}`, { method: 'DELETE' }),
	addRequirement: (
		planId: string,
		habitIds: string[],
		quotas: TierQuota[],
		measure: Measure,
		name: string | null
	) =>
		request<Requirement>(`/plans/${planId}/requirements?${asOfToday()}`, {
			method: 'POST',
			body: body({ habit_ids: habitIds, quotas, measure, name })
		}),
	updateRequirement: (
		id: string,
		patch: {
			habit_ids?: string[];
			quotas?: TierQuota[];
			measure?: Measure;
			name?: string | null;
		}
	) =>
		request<void>(`/plan-requirements/${id}?${asOfToday()}`, {
			method: 'PATCH',
			body: body(patch)
		}),
	removeRequirement: (id: string) =>
		request<void>(`/plan-requirements/${id}?${asOfToday()}`, { method: 'DELETE' })
};

export const progressApi = {
	history: (period: Period, count: number) =>
		collection<PeriodOutcome>(`/progress/history?period=${period}&count=${count}`),
	get: (period: Period, on?: string) => {
		const query = new URLSearchParams({ period });
		if (on) query.set('on', on);
		return request<PeriodProgress>(`/progress?${query}`);
	}
};

export const authApi = {
	/** A page the browser navigates to, not a fetch: it ends at the provider. */
	startUrl: (provider: string, redirectUri: string, challenge: string) =>
		`${BASE}/auth/${provider}/start?${new URLSearchParams({
			redirect_uri: redirectUri,
			code_challenge: challenge
		})}`,
	redeem: (code: string, verifier: string) =>
		request<SessionGrant>('/auth/token', {
			method: 'POST',
			body: body({ code, code_verifier: verifier })
		}),
	methods: () => request<AuthMethods>('/auth/methods'),
	passwordLogin: (email: string, password: string) =>
		request<SessionGrant>('/auth/password/login', {
			method: 'POST',
			body: body({ email, password })
		}),
	/** A pending account gets no session: it waits for an administrator. */
	register: (email: string, password: string, displayName: string) =>
		request<SessionGrant | { pending: true }>('/auth/password/register', {
			method: 'POST',
			body: body({ email, password, display_name: displayName || null })
		}),
	changePassword: (current: string | null, next: string) =>
		request<SessionGrant>('/me/password', {
			method: 'PUT',
			body: body({ current_password: current, new_password: next })
		}),
	me: () => request<Account>('/me'),
	logout: () => request<void>('/auth/logout', { method: 'POST' })
};
