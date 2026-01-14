import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { useAtom } from "jotai";
import { getAccounts, createAccount } from "../api/accounts";
import { accountsAtom, accountsLoadingAtom } from "../atoms/accounts";
import { Layout } from "../components/Layout";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { Modal } from "../components/Modal";
import { LoadingSpinner } from "../components/LoadingSpinner";
import { useRequireAuth } from "../hooks/useAuth";

export function DashboardPage() {
	const { authLoading } = useRequireAuth();
	const [accounts, setAccounts] = useAtom(accountsAtom);
	const [loading, setLoading] = useAtom(accountsLoadingAtom);
	const [showCreateModal, setShowCreateModal] = useState(false);
	const [newAccountName, setNewAccountName] = useState("");
	const [creating, setCreating] = useState(false);

	useEffect(() => {
		async function fetchAccounts() {
			setLoading(true);
			try {
				const data = await getAccounts();
				setAccounts(data.items);
			} finally {
				setLoading(false);
			}
		}
		if (!authLoading) {
			fetchAccounts();
		}
	}, [authLoading, setAccounts, setLoading]);

	const handleCreateAccount = async () => {
		if (!newAccountName.trim()) return;

		setCreating(true);
		try {
			await createAccount({ name: newAccountName.trim() });
			const updatedAccounts = await getAccounts();
			setAccounts(updatedAccounts.items);
			setShowCreateModal(false);
			setNewAccountName("");
		} finally {
			setCreating(false);
		}
	};

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
			<div className="mb-6 flex justify-between items-center">
				<h1 className="text-2xl font-bold text-gray-900">アカウント一覧</h1>
				<Button onClick={() => setShowCreateModal(true)}>新規作成</Button>
			</div>

			{accounts.length === 0 ? (
				<div className="bg-white rounded-lg shadow p-8 text-center">
					<p className="text-gray-600 mb-4">アカウントがありません</p>
					<Button onClick={() => setShowCreateModal(true)}>
						最初のアカウントを作成
					</Button>
				</div>
			) : (
				<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
					{accounts.map((account) => (
						<Link
							key={account.id}
							to={`/accounts/${account.id}`}
							className="block bg-white rounded-lg shadow p-6 hover:shadow-md transition-shadow"
						>
							<h2 className="text-lg font-semibold text-gray-900">
								{account.name}
							</h2>
							<p className="text-sm text-gray-500 mt-1">
								オーナー: {account.ownerIds.length}人
							</p>
						</Link>
					))}
				</div>
			)}

			<Modal
				isOpen={showCreateModal}
				onClose={() => setShowCreateModal(false)}
				title="アカウント作成"
				actions={
					<>
						<Button
							variant="secondary"
							onClick={() => setShowCreateModal(false)}
							disabled={creating}
						>
							キャンセル
						</Button>
						<Button
							onClick={handleCreateAccount}
							disabled={creating || !newAccountName.trim()}
						>
							{creating ? "作成中..." : "作成"}
						</Button>
					</>
				}
			>
				<Input
					label="アカウント名"
					value={newAccountName}
					onChange={(e) => setNewAccountName(e.target.value)}
					placeholder="例: 家計簿"
					autoFocus
				/>
			</Modal>
		</Layout>
	);
}
