import { i18n } from "@/i18n";

/** Unix 秒时间戳 → 当前语言的相对时间 */
export function formatRelativeTime(tsSeconds: number | null): string {
  if (!tsSeconds) return i18n.global.t("common.never");
  const diff = Date.now() / 1000 - tsSeconds;
  if (diff < 60) return i18n.global.t("common.justNow");
  if (diff < 3600) return i18n.global.t("common.minutesAgo", { count: Math.floor(diff / 60) });
  if (diff < 86400) return i18n.global.t("common.hoursAgo", { count: Math.floor(diff / 3600) });
  return i18n.global.t("common.daysAgo", { count: Math.floor(diff / 86400) });
}