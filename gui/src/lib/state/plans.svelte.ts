import { plansApi } from '$lib/api/endpoints';
import type { Measure, Medal, Period, Plan, Requirement, TierQuota } from '$lib/api/types';
import { toast } from './toast.svelte';

class Plans {
	items = $state<Plan[]>([]);
	active = $derived(this.items.filter((plan) => !plan.archived_on));
	archived = $derived(this.items.filter((plan) => plan.archived_on));
	/** plan id -> its requirements, in display order */
	requirements = $state<Record<string, Requirement[]>>({});
	loading = $state(false);

	for(planId: string) {
		return this.requirements[planId] ?? [];
	}

	async load() {
		this.loading = true;
		const plans = await toast.guard(() => plansApi.list(true));
		if (plans) {
			this.items = plans;
			// Archived plans are never edited, so their requirements are not needed here.
			const editable = plans.filter((plan) => !plan.archived_on);
			const lists = await toast.guard(() =>
				Promise.all(editable.map((plan) => plansApi.requirements(plan.id)))
			);
			if (lists) {
				this.requirements = Object.fromEntries(
					editable.map((plan, index) => [plan.id, lists[index]])
				);
			}
		}
		this.loading = false;
	}

	async create(name: string, period: Period) {
		const created = await toast.guard(() => plansApi.create(name, period));
		if (created) await this.load();
	}

	async rename(id: string, raw: string) {
		const name = raw.trim();
		if (!name) return;
		const ok = await toast.guard(() => plansApi.update(id, { name }).then(() => true));
		if (ok) await this.load();
	}

	async setPeriod(id: string, period: Period) {
		const ok = await toast.guard(() => plansApi.update(id, { period }).then(() => true));
		if (ok) await this.load();
	}

	async setMembers(id: string, habitIds: string[]) {
		if (habitIds.length === 0) return;
		const ok = await toast.guard(() =>
			plansApi.updateRequirement(id, { habit_ids: habitIds }).then(() => true)
		);
		if (ok) await this.load();
	}

	/** Stops it counting from this period on; every earlier period keeps its result. */
	async setArchived(id: string, archived: boolean) {
		const ok = await toast.guard(() => plansApi.update(id, { archived }).then(() => true));
		if (ok) await this.load();
	}

	async remove(id: string) {
		const ok = await toast.guard(() => plansApi.remove(id).then(() => true));
		if (ok) await this.load();
	}

	async addTier(planId: string, medal: Medal | null) {
		const ok = await toast.guard(() => plansApi.addTier(planId, medal).then(() => true));
		if (ok) await this.load();
	}

	/** Labels only: renaming a tier or swapping its medal relabels past periods too. */
	async setTierMedal(id: string, medal: Medal | null) {
		const ok = await toast.guard(() => plansApi.updateTier(id, { medal }).then(() => true));
		if (ok) await this.load();
	}

	async setTierName(id: string, raw: string) {
		const name = raw.trim() || null;
		const ok = await toast.guard(() => plansApi.updateTier(id, { name }).then(() => true));
		if (ok) await this.load();
	}

	/** Stops asking at it from this period on; earlier periods keep their medals. */
	async retireTier(id: string) {
		const ok = await toast.guard(() => plansApi.retireTier(id).then(() => true));
		if (ok) await this.load();
	}

	async addRequirement(
		planId: string,
		habitIds: string[],
		quotas: TierQuota[],
		measure: Measure,
		name: string | null
	) {
		const ok = await toast.guard(() =>
			plansApi.addRequirement(planId, habitIds, quotas, measure, name).then(() => true)
		);
		if (ok) await this.load();
	}

	async setName(id: string, raw: string) {
		const name = raw.trim() || null;
		const ok = await toast.guard(() => plansApi.updateRequirement(id, { name }).then(() => true));
		if (ok) await this.load();
	}

	/** Replaces a requirement's quotas; a tier left out skips this requirement. */
	async setQuotas(id: string, quotas: TierQuota[]) {
		const ok = await toast.guard(() => plansApi.updateRequirement(id, { quotas }).then(() => true));
		if (ok) await this.load();
		return Boolean(ok);
	}

	async setMeasure(id: string, measure: Measure) {
		const ok = await toast.guard(() =>
			plansApi.updateRequirement(id, { measure }).then(() => true)
		);
		if (ok) await this.load();
	}

	async removeRequirement(id: string) {
		const ok = await toast.guard(() => plansApi.removeRequirement(id).then(() => true));
		if (ok) await this.load();
	}
}

export const plans = new Plans();
