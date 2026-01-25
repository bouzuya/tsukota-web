import { atom } from "jotai";
import type { User } from "../api/types";

// トークンベース認証用
const AUTH_TOKEN_KEY = "tsukota_auth_token";
const DEVICE_ID_KEY = "tsukota_device_id";
const DEVICE_SECRET_KEY = "tsukota_device_secret";

export function getAuthToken(): string | null {
	return localStorage.getItem(AUTH_TOKEN_KEY);
}

export function setAuthToken(token: string | null): void {
	if (token) {
		localStorage.setItem(AUTH_TOKEN_KEY, token);
	} else {
		localStorage.removeItem(AUTH_TOKEN_KEY);
	}
}

export function getDeviceId(): string | null {
	return localStorage.getItem(DEVICE_ID_KEY);
}

export function setDeviceId(deviceId: string | null): void {
	if (deviceId) {
		localStorage.setItem(DEVICE_ID_KEY, deviceId);
	} else {
		localStorage.removeItem(DEVICE_ID_KEY);
	}
}

export function getDeviceSecret(): string | null {
	return localStorage.getItem(DEVICE_SECRET_KEY);
}

export function setDeviceSecret(deviceSecret: string | null): void {
	if (deviceSecret) {
		localStorage.setItem(DEVICE_SECRET_KEY, deviceSecret);
	} else {
		localStorage.removeItem(DEVICE_SECRET_KEY);
	}
}

// Current user state
export const currentUserAtom = atom<User | null>(null);

// Derived atom for authentication status
export const isAuthenticatedAtom = atom<boolean>(
	(get) => get(currentUserAtom) !== null,
);

// Loading state for auth
export const authLoadingAtom = atom<boolean>(true);
