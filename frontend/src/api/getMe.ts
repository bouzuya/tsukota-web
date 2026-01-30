import { apiGet } from "./client";

interface GetMeApiResponse {
	user_id: string;
}

export interface GetMeResponse {
	userId: string;
}

export async function getMe(): Promise<GetMeResponse> {
	const response = await apiGet<GetMeApiResponse>("/me");
	return {
		userId: response.user_id,
	};
}
