Fix transaction list showing "no transactions" during loading

Change usePagination items initial state from empty array to null
so that the UI can distinguish between "not yet loaded" and "loaded
but empty". Add explicit return type UsePaginationResult<T> to
usePagination hook.
