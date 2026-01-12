import { apiGet, apiPost } from './client';
import type {
  Transaction,
  AddTransactionCommand,
  AddTransactionResponse,
  UpdateTransactionCommand,
  DeleteTransactionCommand,
} from './types';

export interface TransactionsResponse {
  items: Transaction[];
  nextCursor: string | null;
}

export async function getTransactions(
  accountId: string,
  cursor?: string
): Promise<TransactionsResponse> {
  const params = cursor ? `?after=${cursor}` : '';
  return apiGet<TransactionsResponse>(`/accounts/${accountId}/transactions${params}`);
}

export async function addTransaction(
  command: AddTransactionCommand
): Promise<AddTransactionResponse> {
  return apiPost<AddTransactionResponse>('/commands/add_transaction', command);
}

export async function updateTransaction(
  command: UpdateTransactionCommand
): Promise<void> {
  return apiPost<void>('/commands/update_transaction', command);
}

export async function deleteTransaction(
  command: DeleteTransactionCommand
): Promise<void> {
  return apiPost<void>('/commands/delete_transaction', command);
}
