/**
 * Format date string (YYYY-MM-DD) to localized format
 */
export function formatDate(dateStr: string): string {
	const date = new Date(dateStr);
	return date.toLocaleDateString("ja-JP", {
		year: "numeric",
		month: "short",
		day: "numeric",
	});
}

/**
 * Get today's date in YYYY-MM-DD format
 */
export function getTodayString(): string {
	const today = new Date();
	return today.toISOString().split("T")[0];
}
