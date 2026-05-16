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
const SIGNIN_URL = "/lab/tsukota/auth/signin";

export interface ApiRequestOptions {
	/** 401 を受け取ったとき自動で `/auth/signin` に遷移するか (default: true) */
	redirectOn401?: boolean;
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
	{ redirectOn401 = true }: ApiRequestOptions = {},
): Promise<T> {
	const url = `${API_BASE}${path}`;
	const config: RequestInit = {
		...options,
		credentials: "include",
		headers: {
			"Content-Type": "application/json",
			...options.headers,
		},
	};

	const response = await fetch(url, config);

	if (response.status === 401) {
		if (redirectOn401) {
			window.location.href = SIGNIN_URL;
		}
		throw new ApiError(401);
	}

	return handleResponse<T>(response);
}

export async function apiGet<T>(
	path: string,
	opts?: ApiRequestOptions,
): Promise<T> {
	return apiRequest<T>(path, { method: "GET" }, opts);
}

export async function apiPost<T, B = unknown>(
	path: string,
	body?: B,
	opts?: ApiRequestOptions,
): Promise<T> {
	return apiRequest<T>(
		path,
		{
			method: "POST",
			body: body ? JSON.stringify(body) : undefined,
		},
		opts,
	);
}
