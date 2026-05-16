import { apiGet } from "./client";

interface GetMeApiResponse {
	user_id: string;
}

export interface GetMeResponse {
	userId: string;
}

export async function getMe(): Promise<GetMeResponse> {
	// 未ログイン時の checkAuth でも呼ぶため 401 で自動遷移しない
	const response = await apiGet<GetMeApiResponse>("/me", {
		redirectOn401: false,
	});
	return {
		userId: response.user_id,
	};
}
