import { levelsApi } from '$lib/api/endpoints';
import type { Measure, Period, PlanLevel, Requirement } from '$lib/api/types';
import { toast } from './toast.svelte';

class Levels {
	items = $state<PlanLevel[]>([]);
	active = $derived(this.items.filter((level) => !level.archived_on));
	archived = $derived(this.items.filter((level) => level.archived_on));
	/** level id -> its requirements, in display order */
	requirements = $state<Record<string, Requirement[]>>({});
	loading = $state(false);

	for(levelId: string) {
		return this.requirements[levelId] ?? [];
	}

	async load() {
		this.loading = true;
		const levels = await toast.guard(() => levelsApi.list(true));
		if (levels) {
			this.items = levels;
			// Archived levels are never edited, so their requirements are not needed here.
			const editable = levels.filter((level) => !level.archived_on);
			const lists = await toast.guard(() =>
				Promise.all(editable.map((level) => levelsApi.requirements(level.id)))
			);
			if (lists) {
				this.requirements = Object.fromEntries(
					editable.map((level, index) => [level.id, lists[index]])
				);
			}
		}
		this.loading = false;
	}

	async create(name: string, period: Period) {
		const created = await toast.guard(() => levelsApi.create(name, period));
		if (created) await this.load();
	}

	async rename(id: string, raw: string) {
		const name = raw.trim();
		if (!name) return;
		const ok = await toast.guard(() => levelsApi.update(id, { name }).then(() => true));
		if (ok) await this.load();
	}

	async setPeriod(id: string, period: Period) {
		const ok = await toast.guard(() => levelsApi.update(id, { period }).then(() => true));
		if (ok) await this.load();
	}

	async setMembers(id: string, habitIds: string[]) {
		if (habitIds.length === 0) return;
		const ok = await toast.guard(() =>
			levelsApi.updateRequirement(id, { habit_ids: habitIds }).then(() => true)
		);
		if (ok) await this.load();
	}

	/** Stops it counting from this period on; every earlier period keeps its result. */
	async setArchived(id: string, archived: boolean) {
		const ok = await toast.guard(() => levelsApi.update(id, { archived }).then(() => true));
		if (ok) await this.load();
	}

	async remove(id: string) {
		const ok = await toast.guard(() => levelsApi.remove(id).then(() => true));
		if (ok) await this.load();
	}

	async addRequirement(
		levelId: string,
		habitIds: string[],
		quota: number,
		measure: Measure,
		name: string | null
	) {
		const ok = await toast.guard(() =>
			levelsApi.addRequirement(levelId, habitIds, quota, measure, name).then(() => true)
		);
		if (ok) await this.load();
	}

	async setName(id: string, raw: string) {
		const name = raw.trim() || null;
		const ok = await toast.guard(() => levelsApi.updateRequirement(id, { name }).then(() => true));
		if (ok) await this.load();
	}

	async setQuota(id: string, raw: string) {
		const quota = Number(raw);
		if (!Number.isFinite(quota) || quota <= 0) return;
		const ok = await toast.guard(() => levelsApi.updateRequirement(id, { quota }).then(() => true));
		if (ok) await this.load();
	}

	async setMeasure(id: string, measure: Measure) {
		const ok = await toast.guard(() =>
			levelsApi.updateRequirement(id, { measure }).then(() => true)
		);
		if (ok) await this.load();
	}

	async removeRequirement(id: string) {
		const ok = await toast.guard(() => levelsApi.removeRequirement(id).then(() => true));
		if (ok) await this.load();
	}
}

export const levels = new Levels();
