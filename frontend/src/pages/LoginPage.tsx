import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useAtomValue, useSetAtom } from "jotai";
import {
	isAuthenticatedAtom,
	authLoadingAtom,
	currentUserAtom,
	setAuthToken,
	setDeviceId,
	setDeviceSecret,
} from "../atoms/auth";
import { createSessionToken } from "../api/createSessionToken";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { PageLoader } from "../components/LoadingSpinner";

export function LoginPage() {
	const isAuthenticated = useAtomValue(isAuthenticatedAtom);
	const authLoading = useAtomValue(authLoadingAtom);
	const setCurrentUser = useSetAtom(currentUserAtom);
	const setAuthLoading = useSetAtom(authLoadingAtom);
	const navigate = useNavigate();

	const [deviceIdInput, setDeviceIdInput] = useState("");
	const [deviceSecretInput, setDeviceSecretInput] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [isLoggingIn, setIsLoggingIn] = useState(false);

	useEffect(() => {
		if (!authLoading && isAuthenticated) {
			navigate("/");
		}
	}, [authLoading, isAuthenticated, navigate]);

	const handleLogin = async () => {
		const trimmedDeviceId = deviceIdInput.trim();
		const trimmedDeviceSecret = deviceSecretInput.trim();

		if (!trimmedDeviceId || !trimmedDeviceSecret) return;

		setError(null);
		setIsLoggingIn(true);

		try {
			const token = await createSessionToken({
				deviceId: trimmedDeviceId,
				deviceSecret: trimmedDeviceSecret,
			});

			// トークンとデバイス情報を保存
			setAuthToken(token);
			setDeviceId(trimmedDeviceId);
			setDeviceSecret(trimmedDeviceSecret);

			// ユーザー情報を設定（トークンからは取得できないため、device_id を使用）
			setCurrentUser({
				id: trimmedDeviceId,
				email: "",
				displayName: trimmedDeviceId,
				createdAt: new Date().toISOString(),
			});

			setAuthLoading(false);
			navigate("/");
		} catch (err) {
			const message =
				err instanceof Error ? err.message : "サインインに失敗しました";
			setError(message);
		} finally {
			setIsLoggingIn(false);
		}
	};

	const handleKeyDown = (e: React.KeyboardEvent) => {
		if (e.key === "Enter" && !isLoggingIn) {
			handleLogin();
		}
	};

	if (authLoading) {
		return <PageLoader />;
	}

	const isFormValid = deviceIdInput.trim() && deviceSecretInput.trim();

	return (
		<div className="min-h-screen bg-gray-100 flex flex-col items-center justify-center px-4">
			<div className="bg-white rounded-lg shadow-lg p-8 w-full max-w-md">
				<div className="text-center mb-8">
					<h1 className="text-3xl font-bold text-gray-900 mb-2">tsukota</h1>
					<p className="text-gray-600">アカウント・支出管理アプリ</p>
				</div>

				<div className="space-y-4">
					<Input
						label="Device ID"
						value={deviceIdInput}
						onChange={(e) => setDeviceIdInput(e.target.value)}
						placeholder="デバイスIDを入力"
						onKeyDown={handleKeyDown}
						disabled={isLoggingIn}
					/>

					<Input
						label="Device Secret"
						type="password"
						value={deviceSecretInput}
						onChange={(e) => setDeviceSecretInput(e.target.value)}
						placeholder="デバイスシークレットを入力"
						onKeyDown={handleKeyDown}
						disabled={isLoggingIn}
					/>

					<Button
						onClick={handleLogin}
						className="w-full"
						size="lg"
						disabled={!isFormValid || isLoggingIn}
					>
						{isLoggingIn ? "サインイン中..." : "サインイン"}
					</Button>
				</div>

				{error && (
					<p className="mt-4 text-sm text-center text-red-600">{error}</p>
				)}
			</div>
		</div>
	);
}
