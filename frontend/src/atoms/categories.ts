import { atom } from 'jotai';
import type { Category } from '../api/types';

// Categories for current account
export const categoriesAtom = atom<Category[]>([]);

// Loading state
export const categoriesLoadingAtom = atom<boolean>(false);

// Derived atom for active (non-deleted) categories
export const activeCategoriesAtom = atom((get) => {
  const categories = get(categoriesAtom);
  return categories.filter((c) => c.deletedAt === null);
});
