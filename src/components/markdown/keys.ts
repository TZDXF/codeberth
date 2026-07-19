import type { InjectionKey } from "vue";

/** Markdown 内容所在的基础目录(用于解析相对路径图片/文件),返回函数以保响应式 */
export const MD_BASE_PATH_KEY: InjectionKey<() => string> = Symbol("md-base-path");
