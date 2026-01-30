import { useAtomValue, useSetAtom } from "jotai";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { createSessionToken } from "../api/createSessionToken";
import { getMe } from "../api/getMe";
import {
	authLoadingAtom,
	currentUserAtom,
	isAuthenticatedAtom,
	setAuthToken,
	setDeviceId,
	setDeviceSecret,
} from "../atoms/auth";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { PageLoader } from "../components/LoadingSpinner";
import {
	generateDeviceId,
	generateDeviceSecret,
} from "../utils/deviceCredentials";

type Mode = "select" | "new" | "migrate";

export function LoginPage() {
	const isAuthenticated = useAtomValue(isAuthenticatedAtom);
	const authLoading = useAtomValue(authLoadingAtom);
	const setCurrentUser = useSetAtom(currentUserAtom);
	const setAuthLoading = useSetAtom(authLoadingAtom);
	const navigate = useNavigate();

	const [mode, setMode] = useState<Mode>("select");
	const [deviceIdInput, setDeviceIdInput] = useState("");
	const [deviceSecretInput, setDeviceSecretInput] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [isLoggingIn, setIsLoggingIn] = useState(false);

	useEffect(() => {
		if (!authLoading && isAuthenticated) {
			navigate("/");
		}
	}, [authLoading, isAuthenticated, navigate]);

	const performLogin = async (deviceId: string, deviceSecret: string) => {
		setError(null);
		setIsLoggingIn(true);

		try {
			const token = await createSessionToken({
				deviceId,
				deviceSecret,
			});

			// トークンとデバイス情報を保存
			setAuthToken(token);
			setDeviceId(deviceId);
			setDeviceSecret(deviceSecret);

			// ユーザー情報を取得
			const meResponse = await getMe();
			setCurrentUser({
				id: meResponse.userId,
				email: "",
				displayName: meResponse.userId,
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

	const handleStartNew = async () => {
		setMode("new");
		const newDeviceId = generateDeviceId();
		const newDeviceSecret = generateDeviceSecret();
		await performLogin(newDeviceId, newDeviceSecret);
	};

	const handleMigrateLogin = async () => {
		const trimmedDeviceId = deviceIdInput.trim();
		const trimmedDeviceSecret = deviceSecretInput.trim();

		if (!trimmedDeviceId || !trimmedDeviceSecret) return;

		await performLogin(trimmedDeviceId, trimmedDeviceSecret);
	};

	const handleKeyDown = (e: React.KeyboardEvent) => {
		if (e.key === "Enter" && !isLoggingIn) {
			handleMigrateLogin();
		}
	};

	const handleBack = () => {
		setMode("select");
		setError(null);
		setDeviceIdInput("");
		setDeviceSecretInput("");
	};

	if (authLoading) {
		return <PageLoader />;
	}

	const isFormValid = deviceIdInput.trim() && deviceSecretInput.trim();

	// 選択画面
	if (mode === "select") {
		return (
			<div className="min-h-screen bg-gray-100 flex flex-col items-center justify-center px-4">
				<div className="bg-white rounded-lg shadow-lg p-8 w-full max-w-md">
					<div className="text-center mb-8">
						<h1 className="text-3xl font-bold text-gray-900 mb-2">tsukota</h1>
						<p className="text-gray-600">アカウント・支出管理アプリ</p>
					</div>

					<div className="space-y-4">
						<Button
							onClick={handleStartNew}
							className="w-full"
							size="lg"
							disabled={isLoggingIn}
						>
							{isLoggingIn ? "サインイン中..." : "はじめる"}
						</Button>

						<Button
							onClick={() => setMode("migrate")}
							className="w-full"
							size="lg"
							variant="secondary"
							disabled={isLoggingIn}
						>
							他のデバイスからの移行
						</Button>
					</div>

					{error && (
						<p className="mt-4 text-sm text-center text-red-600">{error}</p>
					)}
				</div>
			</div>
		);
	}

	// 新規作成中（自動生成でサインイン中）
	if (mode === "new") {
		return (
			<div className="min-h-screen bg-gray-100 flex flex-col items-center justify-center px-4">
				<div className="bg-white rounded-lg shadow-lg p-8 w-full max-w-md">
					<div className="text-center mb-8">
						<h1 className="text-3xl font-bold text-gray-900 mb-2">tsukota</h1>
						<p className="text-gray-600">アカウント・支出管理アプリ</p>
					</div>

					<div className="text-center">
						{isLoggingIn ? (
							<p className="text-gray-600">サインイン中...</p>
						) : error ? (
							<>
								<p className="text-sm text-red-600 mb-4">{error}</p>
								<Button onClick={handleBack} variant="secondary">
									戻る
								</Button>
							</>
						) : null}
					</div>
				</div>
			</div>
		);
	}

	// 移行モード（手動入力）
	return (
		<div className="min-h-screen bg-gray-100 flex flex-col items-center justify-center px-4">
			<div className="bg-white rounded-lg shadow-lg p-8 w-full max-w-md">
				<div className="text-center mb-8">
					<h1 className="text-3xl font-bold text-gray-900 mb-2">tsukota</h1>
					<p className="text-gray-600">他のデバイスからの移行</p>
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
						onClick={handleMigrateLogin}
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

				<div className="mt-6">
					<button
						onClick={handleBack}
						className="text-gray-600 hover:text-gray-900 text-sm"
						disabled={isLoggingIn}
						type="button"
					>
						← 戻る
					</button>
				</div>
			</div>
		</div>
	);
}
