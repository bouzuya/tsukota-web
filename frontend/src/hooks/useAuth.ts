import { useAtom, useAtomValue } from "jotai";
import { useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { getMe } from "../api/getMe";
import {
	authLoadingAtom,
	currentUserAtom,
	getAuthToken,
	isAuthenticatedAtom,
	setAuthToken,
	setDeviceId,
	setDeviceSecret,
} from "../atoms/auth";

export function useAuth() {
	const [currentUser, setCurrentUser] = useAtom(currentUserAtom);
	const [authLoading, setAuthLoading] = useAtom(authLoadingAtom);
	const isAuthenticated = useAtomValue(isAuthenticatedAtom);
	const navigate = useNavigate();

	const checkAuth = useCallback(async () => {
		setAuthLoading(true);

		const authToken = getAuthToken();
		if (authToken) {
			try {
				const meResponse = await getMe();
				setCurrentUser({
					id: meResponse.userId,
					email: "",
					displayName: meResponse.userId,
					createdAt: new Date().toISOString(),
				});
			} catch {
				// トークンが無効な場合はログアウト状態にする
				setCurrentUser(null);
			}
		} else {
			setCurrentUser(null);
		}

		setAuthLoading(false);
	}, [setCurrentUser, setAuthLoading]);

	const logout = useCallback(() => {
		setAuthToken(null);
		setDeviceId(null);
		setDeviceSecret(null);
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
