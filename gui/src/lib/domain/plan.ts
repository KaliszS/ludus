import type { Habit, Measure, Medal, Plan, Tier, TierQuota } from '$lib/api/types';

/** What a quota counts: days for occurrences, the habit's own unit for a single
 *  amount, and nothing for a group adding up habits that may not share one. */
export function quotaUnit(measure: Measure, quota: number, habits: Habit[]): string | null {
	if (measure === 'occurrences') return quota === 1 ? 'day' : 'days';
	return habits.length === 1 ? habits[0].unit : null;
}

export const MEDALS = ['bronze', 'silver', 'gold'] as const satisfies readonly Medal[];

/** Three stops of each metal, lit from the top left, plus the star pressed into it. */
export const METALS: Record<Medal, { light: string; mid: string; dark: string; star: string }> = {
	bronze: { light: '#f6c39a', mid: '#c97b42', dark: '#7a4116', star: '#ffe6d1' },
	silver: { light: '#ffffff', mid: '#b4bec9', dark: '#5f6b78', star: '#ffffff' },
	gold: { light: '#fff3b0', mid: '#eeb127', dark: '#935c00', star: '#fff8d6' }
};

export const liveTiers = (plan: Plan) => plan.tiers.filter((tier) => !tier.retired_on);

export function tierLabel(tier: Tier, index: number): string {
	if (tier.name) return tier.name;
	if (tier.medal) return tier.medal[0].toUpperCase() + tier.medal.slice(1);
	return `Tier ${index + 1}`;
}

/** The first quota still ahead, or the top one once every one is reached. */
export function nextQuota(quotas: TierQuota[], done: number): number {
	return (quotas.find((quota) => done < quota.quota) ?? quotas.at(-1))?.quota ?? 0;
}

/** The medal of the highest tier this amount reaches, if that tier has one. */
export function medalAt(tiers: Tier[], quotas: TierQuota[], done: number): Medal | null {
	const medal = (quota: TierQuota) => tiers.find((tier) => tier.id === quota.tier_id)?.medal;
	return (
		quotas
			.filter((quota) => done >= quota.quota && medal(quota))
			.map(medal)
			.at(-1) ?? null
	);
}
