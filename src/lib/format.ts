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

/** 本地时间串 "YYYY-MM-DD HH:MM"(git_log 返回格式)→ 相对时间;超过 30 天或解析失败回退原串 */
export function formatCommitTime(dateStr: string): string {
  // 补 T 使其按 ISO 本地时间解析("YYYY-MM-DD HH:MM" 在部分引擎会按 UTC 或解析失败)
  const ts = new Date(dateStr.replace(" ", "T")).getTime();
  if (Number.isNaN(ts)) return dateStr;
  if (Date.now() - ts >= 30 * 86400_000) return dateStr;
  return formatRelativeTime(Math.floor(ts / 1000));
}
