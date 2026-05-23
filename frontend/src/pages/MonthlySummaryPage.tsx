import { useAtom, useSetAtom } from "jotai";
import { useEffect } from "react";
import { useParams } from "react-router-dom";
import { getMonthlySummary } from "../api/getMonthlySummary";
import { selectedAccountIdAtom } from "../atoms/accounts";
import {
	monthlySummaryAtom,
	monthlySummaryLoadingAtom,
} from "../atoms/monthlySummary";
import { Layout } from "../components/Layout";
import { LoadingSpinner } from "../components/LoadingSpinner";
import { useRequireAuth } from "../hooks/useAuth";
import { formatAmount } from "../utils/format";

/**
 * 月キー ("YYYY-MM") を日本語表記に変換する
 */
function formatMonthKey(monthKey: string): string {
	const [yearStr, monthStr] = monthKey.split("-");
	const year = Number.parseInt(yearStr ?? "", 10);
	const month = Number.parseInt(monthStr ?? "", 10);
	if (Number.isNaN(year) || Number.isNaN(month)) {
		return monthKey;
	}
	return `${year}年${month}月`;
}

export function MonthlySummaryPage() {
	const { id } = useParams<{ id: string }>();
	const { authLoading } = useRequireAuth();
	const setSelectedAccountId = useSetAtom(selectedAccountIdAtom);
	const [summary, setSummary] = useAtom(monthlySummaryAtom);
	const [loading, setLoading] = useAtom(monthlySummaryLoadingAtom);

	useEffect(() => {
		if (id) {
			setSelectedAccountId(id);
		}
	}, [id, setSelectedAccountId]);

	useEffect(() => {
		async function fetchSummary() {
			if (!id || authLoading) return;
			setLoading(true);
			try {
				const data = await getMonthlySummary(id);
				setSummary(data);
			} catch (error) {
				console.error("Failed to fetch monthly summary:", error);
				setSummary(null);
			} finally {
				setLoading(false);
			}
		}
		fetchSummary();
	}, [id, authLoading, setSummary, setLoading]);

	// "YYYY-MM" を降順 (新しい月が上) でソートしたエントリ
	const sortedEntries = summary
		? Object.entries(summary.totals).sort(([a], [b]) => (a < b ? 1 : -1))
		: [];

	if (authLoading || loading) {
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
			<div className="mb-6">
				<h1 className="text-2xl font-bold text-gray-900">月別集計</h1>
			</div>

			{sortedEntries.length === 0 ? (
				<div className="bg-white rounded-lg shadow p-8 text-center">
					<p className="text-gray-600">集計データがありません</p>
				</div>
			) : (
				<div className="bg-white rounded-lg shadow divide-y">
					{sortedEntries.map(([monthKey, total]) => {
						const num = Number.parseInt(total, 10);
						const isPositive = !Number.isNaN(num) && num >= 0;
						return (
							<div
								key={monthKey}
								className="flex justify-between items-center p-4"
							>
								<p className="font-medium text-gray-900">
									{formatMonthKey(monthKey)}
								</p>
								<p
									className={`text-lg font-semibold ${
										isPositive ? "text-green-600" : "text-red-600"
									}`}
								>
									{formatAmount(total)}
								</p>
							</div>
						);
					})}
				</div>
			)}
		</Layout>
	);
}
