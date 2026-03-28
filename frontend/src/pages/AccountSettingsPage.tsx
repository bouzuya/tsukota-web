import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { addOwner } from "../api/addOwner";
import { deleteAccount } from "../api/deleteAccount";
import { getAccount } from "../api/getAccount";
import type { Account } from "../api/types";
import { updateAccount } from "../api/updateAccount";
import { currentUserAtom } from "../atoms/auth";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { Layout } from "../components/Layout";
import { LoadingSpinner } from "../components/LoadingSpinner";
import { ConfirmModal } from "../components/Modal";
import { useRequireAuth } from "../hooks/useAuth";

export function AccountSettingsPage() {
	const { id } = useParams<{ id: string }>();
	const navigate = useNavigate();
	const { authLoading } = useRequireAuth();
	const currentUser = useAtomValue(currentUserAtom);

	const [account, setAccount] = useState<Account | null>(null);
	const [ownerIds, setOwnerIds] = useState<string[]>([]);
	const [loading, setLoading] = useState(true);
	const [accountName, setAccountName] = useState("");
	const [saving, setSaving] = useState(false);
	const [showDeleteModal, setShowDeleteModal] = useState(false);
	const [newOwnerUserId, setNewOwnerUserId] = useState("");
	const [addingOwner, setAddingOwner] = useState(false);
	const [addOwnerError, setAddOwnerError] = useState<string | null>(null);

	useEffect(() => {
		async function fetchData() {
			if (!id || authLoading) return;

			setLoading(true);
			try {
				const acc = await getAccount(id);
				setAccount(acc);
				setAccountName(acc.name);
				setOwnerIds(acc.ownerIds);
			} finally {
				setLoading(false);
			}
		}
		fetchData();
	}, [id, authLoading]);

	const handleUpdateName = async () => {
		if (!id || !accountName.trim() || !account) return;

		setSaving(true);
		try {
			await updateAccount({ accountId: id, name: accountName.trim() });
			setAccount({ ...account, name: accountName.trim() });
		} finally {
			setSaving(false);
		}
	};

	const handleAddOwner = async () => {
		if (!id || !newOwnerUserId.trim()) return;

		setAddingOwner(true);
		setAddOwnerError(null);
		try {
			await addOwner({ accountId: id, userId: newOwnerUserId.trim() });
			setOwnerIds([...ownerIds, newOwnerUserId.trim()]);
			setNewOwnerUserId("");
		} catch {
			setAddOwnerError(
				"オーナーの追加に失敗しました。ユーザー ID を確認してください。",
			);
		} finally {
			setAddingOwner(false);
		}
	};

	const handleDelete = async () => {
		if (!id) return;

		setSaving(true);
		try {
			await deleteAccount({ accountId: id });
			navigate("/");
		} finally {
			setSaving(false);
			setShowDeleteModal(false);
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

	if (!account) {
		return (
			<Layout>
				<div className="text-center py-12">
					<p className="text-gray-600">アカウントが見つかりません</p>
				</div>
			</Layout>
		);
	}

	return (
		<Layout>
			<h1 className="text-2xl font-bold text-gray-900 mb-6">アカウント設定</h1>

			<div className="space-y-6">
				{/* Account Name */}
				<div className="bg-white rounded-lg shadow p-6">
					<h2 className="text-lg font-semibold text-gray-900 mb-4">
						アカウント名
					</h2>
					<div className="flex gap-3">
						<Input
							value={accountName}
							onChange={(e) => setAccountName(e.target.value)}
							className="flex-1"
						/>
						<Button
							onClick={handleUpdateName}
							disabled={
								saving || !accountName.trim() || accountName === account.name
							}
						>
							{saving ? "保存中..." : "保存"}
						</Button>
					</div>
				</div>

				{/* Owners */}
				<div className="bg-white rounded-lg shadow p-6">
					<h2 className="text-lg font-semibold text-gray-900 mb-4">
						オーナー一覧
					</h2>
					<div className="space-y-2">
						{ownerIds.map((ownerId) => (
							<div
								key={ownerId}
								className="flex items-center justify-between py-2 px-3 bg-gray-50 rounded"
							>
								<div>
									<p className="font-medium text-gray-900">
										{ownerId}
									</p>
								</div>
								{ownerId === currentUser?.id && (
									<span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
										あなた
									</span>
								)}
							</div>
						))}
					</div>
					<div className="mt-4">
						<h3 className="text-sm font-medium text-gray-700 mb-2">
							オーナーを追加
						</h3>
						<div className="flex gap-3">
							<Input
								value={newOwnerUserId}
								onChange={(e) => {
									setNewOwnerUserId(e.target.value);
									setAddOwnerError(null);
								}}
								placeholder="ユーザー ID"
								className="flex-1"
							/>
							<Button
								onClick={handleAddOwner}
								disabled={addingOwner || !newOwnerUserId.trim()}
							>
								{addingOwner ? "追加中..." : "追加"}
							</Button>
						</div>
						{addOwnerError && (
							<p className="mt-1 text-sm text-red-600">{addOwnerError}</p>
						)}
					</div>
				</div>

				{/* Delete Account */}
				<div className="bg-white rounded-lg shadow p-6 border-2 border-red-200">
					<h2 className="text-lg font-semibold text-red-600 mb-4">
						危険な操作
					</h2>
					<p className="text-gray-600 mb-4">
						アカウントを削除すると、すべての取引データと区分が削除されます。この操作は取り消せません。
					</p>
					<Button variant="danger" onClick={() => setShowDeleteModal(true)}>
						アカウントを削除
					</Button>
				</div>
			</div>

			<ConfirmModal
				isOpen={showDeleteModal}
				onClose={() => setShowDeleteModal(false)}
				onConfirm={handleDelete}
				title="アカウントを削除"
				message={`「${account.name}」を削除しますか？すべてのデータが完全に削除され、復元できません。`}
				confirmText="削除する"
				variant="danger"
			/>
		</Layout>
	);
}
