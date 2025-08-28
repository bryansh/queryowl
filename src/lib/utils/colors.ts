// Predefined connection colors - a nice selection for databases
export const CONNECTION_COLORS = [
	'#22c55e', // green-500
	'#3b82f6', // blue-500  
	'#8b5cf6', // violet-500
	'#f59e0b', // amber-500
	'#ef4444', // red-500
	'#06b6d4', // cyan-500
	'#ec4899', // pink-500
	'#84cc16', // lime-500
	'#f97316', // orange-500
	'#6366f1', // indigo-500
	'#10b981', // emerald-500
	'#14b8a6', // teal-500
];

// Get a random color from the predefined list
export function getRandomConnectionColor(): string {
	return CONNECTION_COLORS[Math.floor(Math.random() * CONNECTION_COLORS.length)];
}

// Get a color based on connection name (consistent for same name)
export function getConnectionColorByName(name: string): string {
	let hash = 0;
	for (let i = 0; i < name.length; i++) {
		const char = name.charCodeAt(i);
		hash = ((hash << 5) - hash) + char;
		hash = hash & hash; // Convert to 32bit integer
	}
	const index = Math.abs(hash) % CONNECTION_COLORS.length;
	return CONNECTION_COLORS[index];
}

// Convert hex color to RGB values for use in Tailwind
export function hexToRgb(hex: string): string {
	const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
	if (result) {
		const r = parseInt(result[1], 16);
		const g = parseInt(result[2], 16);
		const b = parseInt(result[3], 16);
		return `${r} ${g} ${b}`;
	}
	return '34 197 94'; // fallback to green
}