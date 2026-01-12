import { apiGet, apiPost } from './client';
import type {
  Account,
  CreateAccountCommand,
  CreateAccountResponse,
  UpdateAccountCommand,
  DeleteAccountCommand,
} from './types';

export async function getAccounts(): Promise<Account[]> {
  return apiGet<Account[]>('/accounts');
}

export async function getAccount(accountId: string): Promise<Account> {
  return apiGet<Account>(`/accounts/${accountId}`);
}

export async function createAccount(
  command: CreateAccountCommand
): Promise<CreateAccountResponse> {
  return apiPost<CreateAccountResponse>('/commands/create_account', command);
}

export async function updateAccount(command: UpdateAccountCommand): Promise<void> {
  return apiPost<void>('/commands/update_account', command);
}

export async function deleteAccount(command: DeleteAccountCommand): Promise<void> {
  return apiPost<void>('/commands/delete_account', command);
}

export function getExportUrl(accountId: string, year: number, month: number): string {
  return `/api/accounts/${accountId}/export/json?year=${year}&month=${month}`;
}
