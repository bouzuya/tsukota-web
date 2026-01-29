import type { ApiCategory } from "./apiTypes";
import type { Category } from "./types";
import { toCategory } from "./toCategory";

export function toCategories(api: ApiCategory[]): Category[] {
	return api.map(toCategory);
}
