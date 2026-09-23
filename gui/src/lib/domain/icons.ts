import {
	Activity,
	Apple,
	Bath,
	BedDouble,
	BedSingle,
	Beer,
	Bike,
	Bird,
	BookOpen,
	BowArrow,
	Brain,
	Briefcase,
	Broom,
	Brush,
	Calendar,
	Camera,
	Cat,
	ChefHat,
	Clapperboard,
	Cloud,
	Code,
	CodeXml,
	Coffee,
	Compass,
	CookingPot,
	Cpu,
	Crown,
	Database,
	Dices,
	Dog,
	Drum,
	Dumbbell,
	Fish,
	FishingRod,
	Flame,
	Flower2,
	Footprints,
	Gamepad2,
	GitBranch,
	GlassWater,
	Goal,
	GraduationCap,
	Guitar,
	Hammer,
	Headphones,
	Heart,
	HeartPulse,
	House,
	Joystick,
	Keyboard,
	LampDesk,
	Landmark,
	Languages,
	Laptop,
	Leaf,
	Lightbulb,
	Mail,
	Map,
	Medal,
	Microscope,
	MicVocal,
	Monitor,
	Moon,
	Motorbike,
	Mountain,
	MountainSnow,
	Music,
	Network,
	NotebookPen,
	Package,
	Palette,
	PenLine,
	PersonStanding,
	Phone,
	Piano,
	PiggyBank,
	Pill,
	Plane,
	Popcorn,
	Presentation,
	Puzzle,
	Salad,
	Scissors,
	Server,
	Shield,
	ShoppingCart,
	Sofa,
	Sparkles,
	Sprout,
	Sun,
	Target,
	TentTree,
	Terminal,
	Trees,
	Trophy,
	Tv,
	Utensils,
	Volleyball,
	Wallet,
	WashingMachine,
	WavesLadder,
	Webhook,
	Weight,
	Whistle,
	Wine,
	Wrench
} from '@lucide/svelte';
import type { LucideIcon } from '@lucide/svelte';

export type IconDef =
	| { kind: 'lucide'; glyph: LucideIcon }
	| { kind: 'flag'; code: string }
	| { kind: 'digit'; text: string }
	/** Lucide has no ball sports beyond volleyball, so these are drawn in its style. */
	| { kind: 'drawn'; body: string };

const lucide = (glyph: LucideIcon): IconDef => ({ kind: 'lucide', glyph });
const flag = (code: string): IconDef => ({ kind: 'flag', code });
const drawn = (body: string): IconDef => ({ kind: 'drawn', body });

const BALL = '<circle cx="12" cy="12" r="10"/>';

const DIGITS = Object.fromEntries(
	Array.from({ length: 10 }, (_, index) => [
		`num-${index}`,
		{ kind: 'digit', text: String(index) } as IconDef
	])
);

export const ICONS: Record<string, IconDef> = {
	// Move
	dumbbell: lucide(Dumbbell),
	run: lucide(Footprints),
	bike: lucide(Bike),
	swim: lucide(WavesLadder),
	walk: lucide(PersonStanding),
	hike: lucide(Mountain),
	ski: lucide(MountainSnow),
	moto: lucide(Motorbike),
	football: drawn(
		`${BALL}<path d="m12 7.2 3.6 2.6-1.4 4.2H9.8L8.4 9.8z"/><path d="M12 2v5.2M2.6 9.1l5.8.7M21.4 9.1l-5.8.7M6.2 19.6l3.6-5.6M17.8 19.6l-3.6-5.6"/>`
	),
	basketball: drawn(
		`${BALL}<path d="M12 2v20M2 12h20"/><path d="M5.5 5.5a12 12 0 0 1 0 13"/><path d="M18.5 5.5a12 12 0 0 0 0 13"/>`
	),
	tennis: drawn(
		`${BALL}<path d="M5.5 5.5a12 12 0 0 1 0 13"/><path d="M18.5 5.5a12 12 0 0 0 0 13"/>`
	),
	volleyball: lucide(Volleyball),
	archery: lucide(BowArrow),
	net: lucide(Goal),
	whistle: lucide(Whistle),
	weight: lucide(Weight),
	pulse: lucide(Activity),
	health: lucide(HeartPulse),

	// Learn
	book: lucide(BookOpen),
	study: lucide(GraduationCap),
	brain: lucide(Brain),
	language: lucide(Languages),
	write: lucide(PenLine),
	notes: lucide(NotebookPen),
	idea: lucide(Lightbulb),
	puzzle: lucide(Puzzle),
	science: lucide(Microscope),
	history: lucide(Landmark),

	// Work
	work: lucide(Briefcase),
	present: lucide(Presentation),
	calendar: lucide(Calendar),
	mail: lucide(Mail),
	call: lucide(Phone),
	package: lucide(Package),

	// Tech
	code: lucide(Code),
	markup: lucide(CodeXml),
	terminal: lucide(Terminal),
	server: lucide(Server),
	database: lucide(Database),
	git: lucide(GitBranch),
	cpu: lucide(Cpu),
	keyboard: lucide(Keyboard),
	monitor: lucide(Monitor),
	laptop: lucide(Laptop),
	cloud: lucide(Cloud),
	shield: lucide(Shield),
	network: lucide(Network),
	webhook: lucide(Webhook),

	// Care
	food: lucide(Salad),
	fruit: lucide(Apple),
	water: lucide(GlassWater),
	coffee: lucide(Coffee),
	meds: lucide(Pill),
	sleep: lucide(Moon),
	rest: lucide(BedDouble),
	bath: lucide(Bath),
	nature: lucide(Leaf),
	sun: lucide(Sun),

	// Make
	music: lucide(Music),
	guitar: lucide(Guitar),
	piano: lucide(Piano),
	drum: lucide(Drum),
	voice: lucide(MicVocal),
	headphones: lucide(Headphones),
	art: lucide(Brush),
	palette: lucide(Palette),
	photo: lucide(Camera),
	film: lucide(Clapperboard),
	craft: lucide(Scissors),

	// Play
	game: lucide(Gamepad2),
	joystick: lucide(Joystick),
	tv: lucide(Tv),
	movie: lucide(Popcorn),
	dice: lucide(Dices),
	travel: lucide(Plane),
	camp: lucide(TentTree),
	map: lucide(Map),
	compass: lucide(Compass),
	fishing: lucide(FishingRod),
	beer: lucide(Beer),
	wine: lucide(Wine),

	// Home
	home: lucide(House),
	cook: lucide(CookingPot),
	chef: lucide(ChefHat),
	utensils: lucide(Utensils),
	shopping: lucide(ShoppingCart),
	laundry: lucide(WashingMachine),
	clean: lucide(Broom),
	repair: lucide(Wrench),
	hammer: lucide(Hammer),
	plant: lucide(Sprout),
	flower: lucide(Flower2),
	trees: lucide(Trees),
	dog: lucide(Dog),
	cat: lucide(Cat),
	bird: lucide(Bird),
	fish: lucide(Fish),
	sofa: lucide(Sofa),
	lamp: lucide(LampDesk),
	bed: lucide(BedSingle),

	// Goals
	money: lucide(PiggyBank),
	budget: lucide(Wallet),
	streak: lucide(Flame),
	goal: lucide(Target),
	win: lucide(Trophy),
	medal: lucide(Medal),
	crown: lucide(Crown),
	spark: lucide(Sparkles),
	heart: lucide(Heart),

	// Flags, walking outwards from home
	'flag-pl': flag('pl'),
	'flag-cz': flag('cz'),
	'flag-lt': flag('lt'),
	'flag-de': flag('de'),
	'flag-at': flag('at'),
	'flag-ch': flag('ch'),
	'flag-be': flag('be'),
	'flag-nl': flag('nl'),
	'flag-fr': flag('fr'),
	'flag-gb': flag('gb'),
	'flag-en': flag('gb-eng'),
	'flag-es': flag('es'),
	'flag-ct': flag('es-ct'),
	'flag-pt': flag('pt'),
	'flag-it': flag('it'),
	'flag-va': flag('va'),
	'flag-gr': flag('gr'),
	'flag-tr': flag('tr'),
	'flag-dk': flag('dk'),
	'flag-se': flag('se'),
	'flag-no': flag('no'),
	'flag-fi': flag('fi'),
	'flag-us': flag('us'),
	'flag-ca': flag('ca'),
	'flag-br': flag('br'),
	'flag-ar': flag('ar'),
	'flag-cn': flag('cn'),
	'flag-kr': flag('kr'),
	'flag-jp': flag('jp'),
	'flag-eu': flag('eu'),

	...DIGITS
};

export type IconKey = string;

export const ICON_GROUPS: { label: string; keys: string[] }[] = [
	{
		label: 'Move',
		keys: [
			'dumbbell',
			'run',
			'bike',
			'swim',
			'walk',
			'hike',
			'ski',
			'moto',
			'football',
			'basketball',
			'tennis',
			'volleyball',
			'archery',
			'net',
			'whistle',
			'weight',
			'pulse',
			'health'
		]
	},
	{
		label: 'Learn',
		keys: [
			'book',
			'study',
			'brain',
			'language',
			'write',
			'notes',
			'idea',
			'puzzle',
			'science',
			'history'
		]
	},
	{ label: 'Work', keys: ['work', 'present', 'calendar', 'mail', 'call', 'package'] },
	{
		label: 'Tech',
		keys: [
			'code',
			'markup',
			'terminal',
			'server',
			'database',
			'git',
			'cpu',
			'keyboard',
			'monitor',
			'laptop',
			'cloud',
			'shield',
			'network',
			'webhook'
		]
	},
	{
		label: 'Care',
		keys: ['food', 'fruit', 'water', 'coffee', 'meds', 'sleep', 'rest', 'bath', 'nature', 'sun']
	},
	{
		label: 'Make',
		keys: [
			'music',
			'guitar',
			'piano',
			'drum',
			'voice',
			'headphones',
			'art',
			'palette',
			'photo',
			'film',
			'craft'
		]
	},
	{
		label: 'Play',
		keys: [
			'game',
			'joystick',
			'tv',
			'movie',
			'dice',
			'travel',
			'camp',
			'map',
			'compass',
			'fishing',
			'beer',
			'wine'
		]
	},
	{
		label: 'Home',
		keys: [
			'home',
			'cook',
			'chef',
			'utensils',
			'shopping',
			'laundry',
			'clean',
			'repair',
			'hammer',
			'plant',
			'flower',
			'trees',
			'dog',
			'cat',
			'bird',
			'fish',
			'sofa',
			'lamp',
			'bed'
		]
	},
	{
		label: 'Goals',
		keys: ['money', 'budget', 'streak', 'goal', 'win', 'medal', 'crown', 'spark', 'heart']
	},
	{
		label: 'Flags',
		keys: [
			'flag-pl',
			'flag-cz',
			'flag-lt',
			'flag-de',
			'flag-at',
			'flag-ch',
			'flag-be',
			'flag-nl',
			'flag-fr',
			'flag-gb',
			'flag-en',
			'flag-es',
			'flag-ct',
			'flag-pt',
			'flag-it',
			'flag-va',
			'flag-gr',
			'flag-tr',
			'flag-dk',
			'flag-se',
			'flag-no',
			'flag-fi',
			'flag-us',
			'flag-ca',
			'flag-br',
			'flag-ar',
			'flag-cn',
			'flag-kr',
			'flag-jp',
			'flag-eu'
		]
	},
	{ label: '123', keys: Object.keys(DIGITS) }
];

export const iconFor = (key: string | null): IconDef => ICONS[key ?? ''] ?? ICONS.heart;
