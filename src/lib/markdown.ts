/** Markdown 内容中 URL/路径的处理工具(README 图片解析、链接拦截共用) */

/** 带协议头的 URL(http:, data:, asset: 等) */
export function hasScheme(url: string): boolean {
  return /^[a-z][a-z0-9+.-]*:/i.test(url);
}

/** 把 Markdown 里的相对路径解析成项目内的绝对路径 */
export function resolvePath(base: string, rel: string): string {
  const clean = decodeURIComponent(rel).split("#")[0].split("?")[0];
  // 已是绝对路径(Windows 盘符 / UNC / POSIX 根)
  if (/^([a-zA-Z]:[\\/]|\\\\|\/)/.test(clean)) return clean;
  return `${base.replace(/[\\/]+$/, "")}/${clean}`;
}
