import type { ApiDeleteTransactionCommand } from "./apiTypes";
import { apiPost } from "./client";

export interface DeleteTransactionCommand {
	accountId: string;
	transactionId: string;
}

export async function deleteTransaction(
	command: DeleteTransactionCommand,
): Promise<void> {
	const apiCommand: ApiDeleteTransactionCommand = {
		account_id: command.accountId,
		transaction_id: command.transactionId,
	};
	return apiPost<void>("/commands/delete_transaction", apiCommand);
}
