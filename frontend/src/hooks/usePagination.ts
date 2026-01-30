import { useCallback, useState } from "react";

interface UsePaginationOptions<T> {
	fetchFn: (
		cursor?: string,
	) => Promise<{ items: T[]; nextCursor: string | null }>;
}

export function usePagination<T>({ fetchFn }: UsePaginationOptions<T>) {
	const [items, setItems] = useState<T[]>([]);
	const [cursor, setCursor] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);
	const [hasMore, setHasMore] = useState(true);

	const loadInitial = useCallback(async () => {
		setLoading(true);
		try {
			const result = await fetchFn();
			setItems(result.items);
			setCursor(result.nextCursor);
			setHasMore(result.nextCursor !== null);
		} finally {
			setLoading(false);
		}
	}, [fetchFn]);

	const loadMore = useCallback(async () => {
		if (!cursor || loading) return;

		setLoading(true);
		try {
			const result = await fetchFn(cursor);
			setItems((prev) => [...prev, ...result.items]);
			setCursor(result.nextCursor);
			setHasMore(result.nextCursor !== null);
		} finally {
			setLoading(false);
		}
	}, [cursor, loading, fetchFn]);

	const reset = useCallback(() => {
		setItems([]);
		setCursor(null);
		setHasMore(true);
	}, []);

	return {
		items,
		loading,
		hasMore,
		loadInitial,
		loadMore,
		reset,
		setItems,
	};
}
