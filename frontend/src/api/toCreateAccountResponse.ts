import type { ApiCreateAccountResponse } from "./apiTypes";
import type { CreateAccountResponse } from "./types";

export function toCreateAccountResponse(
	api: ApiCreateAccountResponse,
): CreateAccountResponse {
	return {
		accountId: api.account_id,
	};
}
