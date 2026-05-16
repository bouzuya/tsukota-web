import { useAtom, useAtomValue } from "jotai";
import { useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { getMe } from "../api/getMe";
import {
	authLoadingAtom,
	currentUserAtom,
	isAuthenticatedAtom,
} from "../atoms/auth";

const SIGNOUT_URL = "/lab/tsukota/auth/signout";

export function useAuth() {
	const [currentUser, setCurrentUser] = useAtom(currentUserAtom);
	const [authLoading, setAuthLoading] = useAtom(authLoadingAtom);
	const isAuthenticated = useAtomValue(isAuthenticatedAtom);
	const navigate = useNavigate();

	const checkAuth = useCallback(async () => {
		setAuthLoading(true);

		try {
			const meResponse = await getMe();
			setCurrentUser({
				id: meResponse.userId,
				email: "",
				displayName: meResponse.userId,
				createdAt: new Date().toISOString(),
			});
		} catch {
			// Cookie が無い or 失効
			setCurrentUser(null);
		}

		setAuthLoading(false);
	}, [setCurrentUser, setAuthLoading]);

	const logout = useCallback(async () => {
		try {
			await fetch(SIGNOUT_URL, {
				method: "POST",
				credentials: "include",
			});
		} finally {
			setCurrentUser(null);
			navigate("/login");
		}
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
