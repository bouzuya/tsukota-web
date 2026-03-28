import {
	type Dispatch,
	type SetStateAction,
	useCallback,
	useState,
} from "react";

interface UsePaginationOptions<T> {
	fetchFn: (
		cursor?: string,
	) => Promise<{ items: T[]; nextCursor: string | null }>;
}

interface UsePaginationResult<T> {
	hasMore: boolean | null;
	items: T[] | null;
	loadInitial: () => Promise<void>;
	loadMore: () => Promise<void>;
	loading: boolean;
	reset: () => void;
	setItems: Dispatch<SetStateAction<T[] | null>>;
}

export function usePagination<T>({
	fetchFn,
}: UsePaginationOptions<T>): UsePaginationResult<T> {
	const [items, setItems] = useState<T[] | null>(null);
	const [cursor, setCursor] = useState<string | null>(null);
	const [loading, setLoading] = useState<boolean>(false);
	const [hasMore, setHasMore] = useState<boolean | null>(null);

	const loadInitial = useCallback(async (): Promise<void> => {
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

	const loadMore = useCallback(async (): Promise<void> => {
		if (!cursor || loading) return;

		setLoading(true);
		try {
			const result = await fetchFn(cursor);
			setItems((prev) => [...(prev ?? []), ...result.items]);
			setCursor(result.nextCursor);
			setHasMore(result.nextCursor !== null);
		} finally {
			setLoading(false);
		}
	}, [cursor, loading, fetchFn]);

	const reset = useCallback((): void => {
		setItems(null);
		setCursor(null);
		setHasMore(null);
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
