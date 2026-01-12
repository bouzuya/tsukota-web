import { apiGet, apiPost } from './client';
import type {
  ApiTransaction,
  ApiPaginatedResponse,
  ApiAddTransactionCommand,
  ApiAddTransactionResponse,
  ApiUpdateTransactionCommand,
  ApiDeleteTransactionCommand,
} from './apiTypes';
import type { Transaction, PaginatedResponse, AddTransactionResponse } from './types';
import { toTransaction, toPaginatedResponse, toAddTransactionResponse } from './converters';

export async function getTransactions(
  accountId: string,
  cursor?: string
): Promise<PaginatedResponse<Transaction>> {
  const params = cursor ? `?after=${cursor}` : '';
  const response = await apiGet<ApiPaginatedResponse<ApiTransaction>>(
    `/accounts/${accountId}/transactions${params}`
  );
  return toPaginatedResponse(response, toTransaction);
}

export interface AddTransactionCommand {
  accountId: string;
  amount: string;
  categoryId: string;
  comment: string;
  date: string;
}

export async function addTransaction(
  command: AddTransactionCommand
): Promise<AddTransactionResponse> {
  const apiCommand: ApiAddTransactionCommand = {
    account_id: command.accountId,
    amount: command.amount,
    category_id: command.categoryId,
    comment: command.comment,
    date: command.date,
  };
  const response = await apiPost<ApiAddTransactionResponse>('/commands/add_transaction', apiCommand);
  return toAddTransactionResponse(response);
}

export interface UpdateTransactionCommand {
  accountId: string;
  transactionId: string;
  amount: string;
  categoryId: string;
  comment: string;
  date: string;
}

export async function updateTransaction(
  command: UpdateTransactionCommand
): Promise<void> {
  const apiCommand: ApiUpdateTransactionCommand = {
    account_id: command.accountId,
    transaction_id: command.transactionId,
    amount: command.amount,
    category_id: command.categoryId,
    comment: command.comment,
    date: command.date,
  };
  return apiPost<void>('/commands/update_transaction', apiCommand);
}

export interface DeleteTransactionCommand {
  accountId: string;
  transactionId: string;
}

export async function deleteTransaction(
  command: DeleteTransactionCommand
): Promise<void> {
  const apiCommand: ApiDeleteTransactionCommand = {
    account_id: command.accountId,
    transaction_id: command.transactionId,
  };
  return apiPost<void>('/commands/delete_transaction', apiCommand);
}
