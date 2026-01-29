import { apiPost } from "./client";
import type {
	ApiAddTransactionCommand,
	ApiAddTransactionResponse,
} from "./apiTypes";
import type { AddTransactionResponse } from "./types";
import { toAddTransactionResponse } from "./toAddTransactionResponse";

export interface AddTransactionCommand {
	accountId: string;
	amount: string;
	categoryId: string;
	comment: string;
	date: string;
}

export async function addTransaction(
	command: AddTransactionCommand,
): Promise<AddTransactionResponse> {
	const apiCommand: ApiAddTransactionCommand = {
		account_id: command.accountId,
		amount: command.amount,
		category_id: command.categoryId,
		comment: command.comment,
		date: command.date,
	};
	const response = await apiPost<ApiAddTransactionResponse>(
		"/commands/add_transaction",
		apiCommand,
	);
	return toAddTransactionResponse(response);
}
