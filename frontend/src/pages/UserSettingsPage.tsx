import { useAtomValue, useSetAtom } from "jotai";
import { useState } from "react";
import { updateUser } from "../api/updateUser";
import { currentUserAtom, getDeviceId, getDeviceSecret } from "../atoms/auth";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { Layout } from "../components/Layout";
import { LoadingSpinner } from "../components/LoadingSpinner";
import { useAuth, useRequireAuth } from "../hooks/useAuth";

export function UserSettingsPage() {
	const { authLoading } = useRequireAuth();
	const { logout } = useAuth();
	const currentUser = useAtomValue(currentUserAtom);
	const setCurrentUser = useSetAtom(currentUserAtom);

	const [displayName, setDisplayName] = useState(
		currentUser?.displayName ?? "",
	);
	const [saving, setSaving] = useState(false);
	const [showSecret, setShowSecret] = useState(false);

	const deviceId = getDeviceId();
	const deviceSecret = getDeviceSecret();

	const handleUpdateName = async () => {
		if (!currentUser || !displayName.trim()) return;

		setSaving(true);
		try {
			await updateUser({
				userId: currentUser.id,
				displayName: displayName.trim(),
			});
			setCurrentUser({ ...currentUser, displayName: displayName.trim() });
		} finally {
			setSaving(false);
		}
	};

	if (authLoading || !currentUser) {
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
			<h1 className="text-2xl font-bold text-gray-900 mb-6">ユーザー設定</h1>

			<div className="space-y-6 max-w-lg">
				{/* User Info */}
				<div className="bg-white rounded-lg shadow p-6">
					<h2 className="text-lg font-semibold text-gray-900 mb-4">
						アカウント情報
					</h2>
					<div className="space-y-3">
						<div>
							<div className="block text-sm font-medium text-gray-700">
								メールアドレス
							</div>
							<p className="mt-1 text-gray-900">{currentUser.email}</p>
						</div>
					</div>
				</div>

				{/* Display Name */}
				<div className="bg-white rounded-lg shadow p-6">
					<h2 className="text-lg font-semibold text-gray-900 mb-4">表示名</h2>
					<div className="flex gap-3">
						<Input
							value={displayName}
							onChange={(e) => setDisplayName(e.target.value)}
							className="flex-1"
						/>
						<Button
							onClick={handleUpdateName}
							disabled={
								saving ||
								!displayName.trim() ||
								displayName === currentUser.displayName
							}
						>
							{saving ? "保存中..." : "保存"}
						</Button>
					</div>
				</div>

				{/* Device Info */}
				<div className="bg-white rounded-lg shadow p-6">
					<h2 className="text-lg font-semibold text-gray-900 mb-4">
						デバイス情報
					</h2>
					<div className="space-y-4">
						<div>
							<div className="block text-sm font-medium text-gray-700">
								Device ID
							</div>
							<p className="mt-1 text-gray-900 font-mono text-sm break-all">
								{deviceId ?? "-"}
							</p>
						</div>
						<div>
							<div className="block text-sm font-medium text-gray-700">
								Device Secret
							</div>
							<div className="mt-1 flex items-center gap-2">
								<p className="text-gray-900 font-mono text-sm break-all flex-1">
									{showSecret ? (deviceSecret ?? "-") : "********"}
								</p>
								<button
									onClick={() => setShowSecret(!showSecret)}
									className="text-sm text-blue-600 hover:text-blue-800"
									type="button"
								>
									{showSecret ? "隠す" : "表示"}
								</button>
							</div>
						</div>
						<p className="text-sm text-amber-600">
							これらの情報は他のデバイスからサインインする際に必要です。第三者に共有しないでください。
						</p>
					</div>
				</div>

				{/* Logout */}
				<div className="bg-white rounded-lg shadow p-6">
					<h2 className="text-lg font-semibold text-gray-900 mb-4">
						ログアウト
					</h2>
					<p className="text-gray-600 mb-4">
						ログアウトすると、再度ログインするまでアプリを使用できません。
					</p>
					<Button variant="secondary" onClick={logout}>
						ログアウト
					</Button>
				</div>
			</div>
		</Layout>
	);
}
