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

/**
 * 金額文字列を i64 相当の数値に変換する。NaN なら 0 にフォールバックする。
 */
function parseAmount(amount: string | undefined): number {
	if (amount === undefined) return 0;
	const num = Number.parseInt(amount, 10);
	return Number.isNaN(num) ? 0 : num;
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

	// incomes/expenses に出現する月キーの和集合を降順 (新しい月が上) でソート
	const sortedMonthKeys: string[] = summary
		? Array.from(
				new Set([
					...Object.keys(summary.incomes),
					...Object.keys(summary.expenses),
				]),
			).sort((a, b) => (a < b ? 1 : -1))
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

			{sortedMonthKeys.length === 0 ? (
				<div className="bg-white rounded-lg shadow p-8 text-center">
					<p className="text-gray-600">集計データがありません</p>
				</div>
			) : (
				<div className="bg-white rounded-lg shadow overflow-hidden">
					<div className="grid grid-cols-4 gap-2 px-4 py-3 text-xs font-medium text-gray-500 bg-gray-50 border-b">
						<div>月</div>
						<div className="text-right">収入</div>
						<div className="text-right">支出</div>
						<div className="text-right">差引</div>
					</div>
					<div className="divide-y">
						{sortedMonthKeys.map((monthKey) => {
							const incomeNum = parseAmount(summary?.incomes[monthKey]);
							const expenseNum = parseAmount(summary?.expenses[monthKey]);
							// 差引は incomes + expenses (expenses は負値で保存)
							const netNum = incomeNum + expenseNum;
							const netIsPositive = netNum >= 0;
							return (
								<div
									key={monthKey}
									className="grid grid-cols-4 gap-2 px-4 py-3 items-center"
								>
									<div className="font-medium text-gray-900">
										{formatMonthKey(monthKey)}
									</div>
									<div className="text-right text-green-600">
										{formatAmount(incomeNum.toString())}
									</div>
									<div className="text-right text-red-600">
										{/* 支出は負値で保存されているため絶対値表記にする */}
										{formatAmount(Math.abs(expenseNum).toString())}
									</div>
									<div
										className={`text-right font-semibold ${
											netIsPositive ? "text-green-600" : "text-red-600"
										}`}
									>
										{formatAmount(netNum.toString())}
									</div>
								</div>
							);
						})}
					</div>
				</div>
			)}
		</Layout>
	);
}
