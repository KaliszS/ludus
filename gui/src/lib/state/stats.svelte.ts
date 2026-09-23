import { planApi } from '$lib/api/endpoints';
import { checkinsApi } from '$lib/api/endpoints';
import { levelsApi } from '$lib/api/endpoints';
import type { Period, PeriodOutcome, UserCheckin } from '$lib/api/types';
import { shift, today } from '$lib/domain/date';
import { habitStats, type HabitStat } from '$lib/domain/stats';
import { habits } from './habits.svelte';
import { toast } from './toast.svelte';

const PERIOD_ORDER: Period[] = ['day', 'week', 'month', 'quarter', 'year'];
const HISTORY_LENGTH = 12;
const YEAR_DAYS = 364;

class Stats {
	spanDays = $state(91);
	/** A full year is always fetched: a streak must not be capped by the chart's zoom. */
	checkins = $state<UserCheckin[]>([]);
	history = $state<{ period: Period; outcomes: PeriodOutcome[] }[]>([]);
	loading = $state(true);

	from = $derived(shift(today(), -(this.spanDays - 1)));
	visible = $derived(this.checkins.filter((row) => row.day >= this.from));
	perHabit = $derived<HabitStat[]>(
		habitStats(habits.active, this.checkins, this.visible, this.spanDays)
	);

	async load() {
		this.loading = true;
		await habits.load();
		await Promise.all([this.loadCheckins(), this.loadHistory()]);
		this.loading = false;
	}

	setSpan(days: number) {
		this.spanDays = days;
	}

	private async loadCheckins() {
		const rows = await toast.guard(() =>
			checkinsApi.all(shift(today(), -(YEAR_DAYS - 1)), today())
		);
		if (rows) this.checkins = rows;
	}

	private async loadHistory() {
		const levels = await toast.guard(() => levelsApi.list());
		if (!levels) return;

		const periods = PERIOD_ORDER.filter((period) =>
			levels.some((level) => level.period === period)
		);
		const outcomes = await toast.guard(() =>
			Promise.all(periods.map((period) => planApi.history(period, HISTORY_LENGTH)))
		);
		if (outcomes) {
			this.history = periods.map((period, index) => ({ period, outcomes: outcomes[index] }));
		}
	}
}

export const stats = new Stats();
