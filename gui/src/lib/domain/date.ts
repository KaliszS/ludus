export const today = () => toKey(new Date());

export const toKey = (date: Date) =>
	`${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;

const parse = (key: string) => new Date(`${key}T00:00:00`);

export const shift = (key: string, days: number) => {
	const date = parse(key);
	date.setDate(date.getDate() + days);
	return toKey(date);
};

/** Monday through Sunday of the week containing `key`. */
export function weekOf(key: string): string[] {
	const date = parse(key);
	const offset = (date.getDay() + 6) % 7;
	const monday = shift(key, -offset);
	return Array.from({ length: 7 }, (_, index) => shift(monday, index));
}

const WEEKDAY = new Intl.DateTimeFormat(undefined, { weekday: 'narrow' });
const MONTH_DAY = new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric' });
const FULL = new Intl.DateTimeFormat(undefined, { weekday: 'long', month: 'long', day: 'numeric' });

export const weekdayLabel = (key: string) => WEEKDAY.format(parse(key));
export const dayNumber = (key: string) => parse(key).getDate();
export const fullLabel = (key: string) => FULL.format(parse(key));
export const isFuture = (key: string) => key > today();

/** The daemon's period_end is exclusive; humans expect the last day it covers. */
export function formatRange(start: string, endExclusive: string): string {
	const from = parse(start);
	const to = parse(shift(endExclusive, -1));
	return from.getTime() === to.getTime()
		? MONTH_DAY.format(from)
		: `${MONTH_DAY.format(from)} – ${MONTH_DAY.format(to)}`;
}
