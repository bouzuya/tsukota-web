import { apiGet, apiPost } from "./client";
import type {
	ApiAccount,
	ApiCreateAccountCommand,
	ApiCreateAccountResponse,
	ApiUpdateAccountCommand,
	ApiDeleteAccountCommand,
	ApiPaginatedResponse,
} from "./apiTypes";
import type {
	Account,
	CreateAccountResponse,
	PaginatedResponse,
} from "./types";
import {
	toAccount,
	toCreateAccountResponse,
	toPaginatedResponse,
} from "./converters";

export async function getAccounts(): Promise<PaginatedResponse<Account>> {
	const response = await apiGet<ApiPaginatedResponse<ApiAccount>>("/accounts");
	return toPaginatedResponse(response, toAccount);
}

export async function getAccount(accountId: string): Promise<Account> {
	const response = await apiGet<ApiAccount>(`/accounts/${accountId}`);
	return toAccount(response);
}

export interface CreateAccountCommand {
	name: string;
}

export async function createAccount(
	command: CreateAccountCommand,
): Promise<CreateAccountResponse> {
	const apiCommand: ApiCreateAccountCommand = {
		name: command.name,
	};
	const response = await apiPost<ApiCreateAccountResponse>(
		"/commands/create_account",
		apiCommand,
	);
	return toCreateAccountResponse(response);
}

export interface UpdateAccountCommand {
	accountId: string;
	name: string;
}

export async function updateAccount(
	command: UpdateAccountCommand,
): Promise<void> {
	const apiCommand: ApiUpdateAccountCommand = {
		account_id: command.accountId,
		name: command.name,
	};
	return apiPost<void>("/commands/update_account", apiCommand);
}

export interface DeleteAccountCommand {
	accountId: string;
}

export async function deleteAccount(
	command: DeleteAccountCommand,
): Promise<void> {
	const apiCommand: ApiDeleteAccountCommand = {
		account_id: command.accountId,
	};
	return apiPost<void>("/commands/delete_account", apiCommand);
}
