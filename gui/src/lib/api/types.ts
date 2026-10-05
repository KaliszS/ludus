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

export interface PlanLevel {
	id: string;
	name: string;
	period: Period;
	position: number;
	/** The day it was archived; null while it is in use. */
	archived_on: string | null;
}

export type Measure = 'amount' | 'occurrences';

/** One member is a plain quota; several accept any mix from the set. */
export interface Requirement {
	id: string;
	level_id: string;
	/** Set when the joined member names are too long to read, e.g. "Cardio". */
	name: string | null;
	habit_ids: string[];
	quota: number;
	measure: Measure;
	position: number;
}

export interface RequirementProgress {
	id: string;
	name: string | null;
	habits: Habit[];
	quota: number;
	measure: Measure;
	done: number;
	met: boolean;
}

export interface LevelProgress extends PlanLevel {
	met: boolean;
	/** Consecutive completed periods; an unfinished current one does not break it. */
	streak: number;
	items: RequirementProgress[];
}

export interface LevelOutcome {
	id: string;
	name: string;
	archived: boolean;
	met: boolean;
	reached: number;
	total: number;
}

export interface PeriodOutcome {
	period_start: string;
	period_end: string;
	levels: LevelOutcome[];
}

export interface Plan {
	period: Period;
	period_start: string;
	period_end: string;
	levels: LevelProgress[];
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
	quota: number;
	measure: Measure;
	valid_from: string;
	habits: Habit[];
}

/** A stretch of a level's life with the same requirements; `to` is exclusive, null for now. */
export interface LevelEra {
	from: string;
	to: string | null;
	requirements: RequirementVersion[];
}

export interface LevelVersions {
	level: PlanLevel;
	eras: LevelEra[];
}
