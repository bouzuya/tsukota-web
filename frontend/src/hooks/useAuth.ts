import { useAtom, useAtomValue } from "jotai";
import { useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import {
	currentUserAtom,
	authLoadingAtom,
	isAuthenticatedAtom,
	getManualUserId,
	setManualUserId,
	getAuthToken,
	getDeviceId,
	setAuthToken,
	setDeviceId,
	setDeviceSecret,
} from "../atoms/auth";

export function useAuth() {
	const [currentUser, setCurrentUser] = useAtom(currentUserAtom);
	const [authLoading, setAuthLoading] = useAtom(authLoadingAtom);
	const isAuthenticated = useAtomValue(isAuthenticatedAtom);
	const navigate = useNavigate();

	const checkAuth = useCallback(() => {
		setAuthLoading(true);

		// 優先: トークンベース認証
		const authToken = getAuthToken();
		if (authToken) {
			const deviceId = getDeviceId();
			setCurrentUser({
				id: deviceId ?? "unknown",
				email: "",
				displayName: deviceId ?? "User",
				createdAt: new Date().toISOString(),
			});
			setAuthLoading(false);
			return;
		}

		// フォールバック: 開発モードの X-User-Id
		const manualUserId = getManualUserId();
		if (manualUserId) {
			setCurrentUser({
				id: manualUserId,
				email: `${manualUserId}@example.com`,
				displayName: `User ${manualUserId}`,
				createdAt: new Date().toISOString(),
			});
		} else {
			setCurrentUser(null);
		}

		setAuthLoading(false);
	}, [setCurrentUser, setAuthLoading]);

	const logout = useCallback(() => {
		// トークン認証のクリア
		setAuthToken(null);
		setDeviceId(null);
		setDeviceSecret(null);
		// 開発モードのクリア
		setManualUserId(null);
		setCurrentUser(null);
		navigate("/login");
	}, [setCurrentUser, navigate]);

	return {
		currentUser,
		isAuthenticated,
		authLoading,
		checkAuth,
		logout,
	};
}

export function useRequireAuth() {
	const isAuthenticated = useAtomValue(isAuthenticatedAtom);
	const authLoading = useAtomValue(authLoadingAtom);
	const navigate = useNavigate();

	useEffect(() => {
		if (!authLoading && !isAuthenticated) {
			navigate("/login");
		}
	}, [authLoading, isAuthenticated, navigate]);

	return { isAuthenticated, authLoading };
}
