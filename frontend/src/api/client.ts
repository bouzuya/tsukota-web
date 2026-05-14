import { getAuthToken } from "../atoms/auth";
import type { ApiProblemDetails } from "./apiTypes";

export class ApiError extends Error {
	status: number;
	problem?: ApiProblemDetails;

	constructor(status: number, problem?: ApiProblemDetails) {
		super(problem?.detail ?? problem?.title ?? `HTTP Error ${status}`);
		this.name = "ApiError";
		this.status = status;
		this.problem = problem;
	}
}

const API_BASE = import.meta.env.VITE_API_BASE ?? "/lab/tsukota/api";

function getHeaders(): Record<string, string> {
	const headers: Record<string, string> = {
		"Content-Type": "application/json",
	};

	const authToken = getAuthToken();
	if (authToken) {
		headers.Authorization = `Bearer ${authToken}`;
	}

	return headers;
}

async function refreshToken(): Promise<boolean> {
	try {
		const response = await fetch(`${API_BASE}/auth/refresh`, {
			method: "POST",
			credentials: "include",
		});
		return response.ok;
	} catch {
		return false;
	}
}

async function handleResponse<T>(response: Response): Promise<T> {
	if (response.status === 204) {
		return undefined as T;
	}

	if (!response.ok) {
		let problem: ApiProblemDetails | undefined;
		try {
			problem = await response.json();
		} catch {
			// Not a JSON response
		}
		throw new ApiError(response.status, problem);
	}

	return response.json();
}

export async function apiRequest<T>(
	path: string,
	options: RequestInit = {},
): Promise<T> {
	const url = `${API_BASE}${path}`;
	const config: RequestInit = {
		...options,
		credentials: "include",
		headers: {
			...getHeaders(),
			...options.headers,
		},
	};

	let response = await fetch(url, config);

	// Handle 401 by trying to refresh token
	if (response.status === 401) {
		const refreshed = await refreshToken();
		if (refreshed) {
			response = await fetch(url, config);
		}
	}

	return handleResponse<T>(response);
}

export async function apiGet<T>(path: string): Promise<T> {
	return apiRequest<T>(path, { method: "GET" });
}

export async function apiPost<T, B = unknown>(
	path: string,
	body?: B,
): Promise<T> {
	return apiRequest<T>(path, {
		method: "POST",
		body: body ? JSON.stringify(body) : undefined,
	});
}
