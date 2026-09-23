/** One warm-to-cool sweep at even lightness, so any two picks still look related. */
export const PALETTE = [
	'#c1583f',
	'#d9663a',
	'#d98324',
	'#c9992a',
	'#a3a52c',
	'#7d9e3a',
	'#5c8d5a',
	'#3f8f6f',
	'#3f8d8c',
	'#3f7d8c',
	'#3f6d9e',
	'#4a5fa5',
	'#5a54a8',
	'#7a5aa0',
	'#93519b',
	'#a8508a',
	'#b04f70',
	'#b0564f',
	'#8a6f5e',
	'#8a8578',
	'#6f7a80',
	'#5f6b75',
	'#4a5560',
	'#3b4450'
] as const;

/** Same hue as the swatch, faint enough to sit behind an icon. */
export const tint = (color: string | null, alpha = '1f') => `${color ?? '#8a8578'}${alpha}`;
