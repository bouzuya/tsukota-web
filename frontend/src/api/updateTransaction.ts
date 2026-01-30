import type { ApiUpdateTransactionCommand } from "./apiTypes";
import { apiPost } from "./client";

export interface UpdateTransactionCommand {
	accountId: string;
	transactionId: string;
	amount: string;
	categoryId: string;
	comment: string;
	date: string;
}

export async function updateTransaction(
	command: UpdateTransactionCommand,
): Promise<void> {
	const apiCommand: ApiUpdateTransactionCommand = {
		account_id: command.accountId,
		transaction_id: command.transactionId,
		amount: command.amount,
		category_id: command.categoryId,
		comment: command.comment,
		date: command.date,
	};
	return apiPost<void>("/commands/update_transaction", apiCommand);
}
