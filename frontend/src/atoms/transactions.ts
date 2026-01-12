import { atom } from 'jotai';
import type { Transaction } from '../api/types';

// Transactions for current account
export const transactionsAtom = atom<Transaction[]>([]);

// Loading state
export const transactionsLoadingAtom = atom<boolean>(false);

// Cursor for pagination
export const transactionsCursorAtom = atom<string | null>(null);

// Whether there are more transactions to load
export const hasMoreTransactionsAtom = atom((get) => {
  return get(transactionsCursorAtom) !== null;
});
