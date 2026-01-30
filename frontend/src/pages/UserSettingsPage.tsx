import { useAtomValue } from "jotai";
import { useState } from "react";
import { currentUserAtom, getDeviceId, getDeviceSecret } from "../atoms/auth";
import { Button } from "../components/Button";
import { Layout } from "../components/Layout";
import { LoadingSpinner } from "../components/LoadingSpinner";
import { useAuth, useRequireAuth } from "../hooks/useAuth";

export function UserSettingsPage() {
	const { authLoading } = useRequireAuth();
	const { logout } = useAuth();
	const currentUser = useAtomValue(currentUserAtom);

	const [showSecret, setShowSecret] = useState(false);

	const deviceId = getDeviceId();
	const deviceSecret = getDeviceSecret();

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
