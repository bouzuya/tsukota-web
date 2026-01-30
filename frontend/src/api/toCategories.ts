import type { ApiCategory } from "./apiTypes";
import { toCategory } from "./toCategory";
import type { Category } from "./types";

export function toCategories(api: ApiCategory[]): Category[] {
	return api.map(toCategory);
}
