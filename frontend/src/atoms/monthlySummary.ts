import { atom } from "jotai";
import type { MonthlySummary } from "../api/types";

// 現在表示中のアカウントの月別サマリー
export const monthlySummaryAtom = atom<MonthlySummary | null>(null);

// 月別サマリーのローディング状態
export const monthlySummaryLoadingAtom = atom<boolean>(false);
