/** Unix 秒时间戳 → 中文相对时间 */
export function formatRelativeTime(tsSeconds: number | null): string {
  if (!tsSeconds) return "从未";
  const diff = Date.now() / 1000 - tsSeconds;
  if (diff < 60) return "刚刚";
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} 小时前`;
  return `${Math.floor(diff / 86400)} 天前`;
}

