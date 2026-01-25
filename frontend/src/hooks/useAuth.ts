import { useAtom, useAtomValue } from "jotai";
import { useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import {
	currentUserAtom,
	authLoadingAtom,
	isAuthenticatedAtom,
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

		const authToken = getAuthToken();
		if (authToken) {
			const deviceId = getDeviceId();
			setCurrentUser({
				id: deviceId ?? "unknown",
				email: "",
				displayName: deviceId ?? "User",
				createdAt: new Date().toISOString(),
			});
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
