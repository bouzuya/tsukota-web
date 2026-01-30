import { useAtom, useSetAtom } from "jotai";
import { useCallback, useEffect } from "react";
import { Link, useParams } from "react-router-dom";
import { getCategories } from "../api/getCategories";
import { getTransactions } from "../api/getTransactions";
import type { Transaction } from "../api/types";
import { selectedAccountIdAtom } from "../atoms/accounts";
import { categoriesAtom } from "../atoms/categories";
import { Button } from "../components/Button";
import { Layout } from "../components/Layout";
import { LoadingSpinner } from "../components/LoadingSpinner";
import { useRequireAuth } from "../hooks/useAuth";
import { usePagination } from "../hooks/usePagination";
import { formatDate } from "../utils/date";
import { formatAmount } from "../utils/format";

export function TransactionListPage() {
	const { id } = useParams<{ id: string }>();
	const { authLoading } = useRequireAuth();
	const setSelectedAccountId = useSetAtom(selectedAccountIdAtom);
	const [categories, setCategories] = useAtom(categoriesAtom);

	const fetchTransactions = useCallback(
		async (cursor?: string) => {
			if (!id) throw new Error("Account ID is required");
			return getTransactions(id, cursor);
		},
		[id],
	);

	const {
		items: transactions,
		loading,
		hasMore,
		loadInitial,
		loadMore,
	} = usePagination<Transaction>({ fetchFn: fetchTransactions });

	useEffect(() => {
		if (id) {
			setSelectedAccountId(id);
		}
	}, [id, setSelectedAccountId]);

	useEffect(() => {
		async function fetchData() {
			if (!id || authLoading) return;
			try {
				const cats = await getCategories(id);
				setCategories(cats.items);
				loadInitial();
			} catch (error) {
				console.error("Failed to fetch data:", error);
			}
		}
		fetchData();
	}, [id, authLoading, setCategories, loadInitial]);

	const getCategoryName = (categoryId: string) => {
		const category = categories.find((c) => c.id === categoryId);
		return category?.name ?? "不明";
	};

	if (authLoading) {
		return (
			<Layout>
				<div className="flex items-center justify-center py-12">
					<LoadingSpinner size="lg" />
				</div>
			</Layout>
		);
	}

	return (
		<Layout>
			<div className="mb-6 flex justify-between items-center">
				<h1 className="text-2xl font-bold text-gray-900">収支一覧</h1>
				<Link to={`/accounts/${id}/new`}>
					<Button>追加</Button>
				</Link>
			</div>

			{loading && transactions.length === 0 ? (
				<div className="flex items-center justify-center py-12">
					<LoadingSpinner size="lg" />
				</div>
			) : transactions.length === 0 ? (
				<div className="bg-white rounded-lg shadow p-8 text-center">
					<p className="text-gray-600 mb-4">取引がありません</p>
					<Link to={`/accounts/${id}/new`}>
						<Button>最初の取引を追加</Button>
					</Link>
				</div>
			) : (
				<>
					<div className="bg-white rounded-lg shadow divide-y">
						{transactions.map((tx) => (
							<Link
								key={tx.id}
								to={`/accounts/${id}/edit/${tx.id}`}
								className="block p-4 hover:bg-gray-50 transition-colors"
							>
								<div className="flex justify-between items-start">
									<div>
										<p className="text-sm text-gray-500">
											{formatDate(tx.date)}
										</p>
										<p className="font-medium text-gray-900 mt-1">
											{getCategoryName(tx.categoryId)}
										</p>
										{tx.comment && (
											<p className="text-sm text-gray-600 mt-1">{tx.comment}</p>
										)}
									</div>
									<p
										className={`text-lg font-semibold ${
											parseInt(tx.amount, 10) >= 0
												? "text-green-600"
												: "text-red-600"
										}`}
									>
										{formatAmount(tx.amount)}
									</p>
								</div>
							</Link>
						))}
					</div>

					{hasMore && (
						<div className="mt-6 text-center">
							<Button variant="secondary" onClick={loadMore} disabled={loading}>
								{loading ? "ロード中..." : "もっと見る"}
							</Button>
						</div>
					)}
				</>
			)}
		</Layout>
	);
}
