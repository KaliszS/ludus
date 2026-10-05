export type Tracking = 'binary' | 'quantity';
export type Period = 'day' | 'week' | 'month' | 'quarter' | 'year';

export interface Habit {
	id: string;
	name: string;
	description: string;
	icon: string | null;
	color: string | null;
	unit: string | null;
	tracking: Tracking;
	weekdays: number[];
	position: number;
	archived: boolean;
	created_at: string;
	updated_at: string;
}

export interface Checkin {
	day: string;
	times: number;
}

export interface UserCheckin extends Checkin {
	habit_id: string;
}

export type Medal = 'bronze' | 'silver' | 'gold';

/** A level of a plan. The lowest one is what keeps the streak; the rest are extra. */
export interface Tier {
	id: string;
	name: string | null;
	medal: Medal | null;
	position: number;
	/** Retired tiers stay listed, since past periods were judged against them. */
	retired_on: string | null;
}

export interface Plan {
	id: string;
	name: string;
	period: Period;
	position: number;
	/** The day it was archived; null while it is in use. */
	archived_on: string | null;
	/** Lowest first. */
	tiers: Tier[];
}

/** A requirement asks for `quota` at this tier; tiers without one are skipped. */
export interface TierQuota {
	tier_id: string;
	quota: number;
}

export type Measure = 'amount' | 'occurrences';

/** One member is a plain quota; several accept any mix from the set. */
export interface Requirement {
	id: string;
	plan_id: string;
	/** Set when the joined member names are too long to read, e.g. "Cardio". */
	name: string | null;
	habit_ids: string[];
	/** Rising with the tier. */
	quotas: TierQuota[];
	measure: Measure;
	position: number;
}

export interface RequirementProgress {
	id: string;
	name: string | null;
	habits: Habit[];
	quotas: TierQuota[];
	measure: Measure;
	done: number;
	/** Holds its part of the lowest tier; a goal asked only higher up always does. */
	met: boolean;
}

export interface TierStanding {
	tier_id: string;
	reached: boolean;
}

export interface PlanProgress extends Plan {
	met: boolean;
	/** Consecutive completed periods; an unfinished current one does not break it. */
	streak: number;
	/** The tiers asked this period, lowest first; reaching one needs every one below. */
	standing: TierStanding[];
	items: RequirementProgress[];
}

export interface PlanOutcome {
	id: string;
	name: string;
	archived: boolean;
	met: boolean;
	reached: number;
	total: number;
	medals: Medal[];
}

export interface PeriodOutcome {
	period_start: string;
	period_end: string;
	plans: PlanOutcome[];
}

export interface PeriodProgress {
	period: Period;
	period_start: string;
	period_end: string;
	plans: PlanProgress[];
}

export interface NewHabit {
	name: string;
	description?: string;
	icon?: string | null;
	color?: string | null;
	unit?: string | null;
	tracking?: Tracking;
	weekdays?: number[];
}

/** null clears the field, undefined leaves it alone. */
export type HabitPatch = Partial<NewHabit> & { archived?: boolean; position?: number };

export interface User {
	id: string;
	email: string;
	display_name: string;
	avatar_url: string | null;
}

export interface SessionGrant {
	token: string;
	expires_at: string;
	user: User;
}

/** The signed-in user plus how they can sign in, e.g. ["google", "password"]. */
export interface Account extends User {
	logins: string[];
}

export interface AuthMethods {
	providers: string[];
	registration: 'open' | 'approval' | 'closed';
}

/** One shape a requirement had; `valid_from` marks where this version began. */
export interface RequirementVersion {
	id: string;
	name: string | null;
	quotas: TierQuota[];
	measure: Measure;
	valid_from: string;
	habits: Habit[];
}

/** A stretch of a plan's life with the same requirements; `to` is exclusive, null for now. */
export interface PlanEra {
	from: string;
	to: string | null;
	requirements: RequirementVersion[];
}

export interface PlanVersions {
	plan: Plan;
	eras: PlanEra[];
}
