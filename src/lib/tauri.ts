import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Project } from "@/types";
import { i18n } from "@/i18n";

type SerializedAppError = {
  code?: unknown;
  message?: unknown;
};

function translateCommandError(error: unknown): string {
  // Tauri AppError 使用 `{ code, message }`;原生 Error 直接使用 message。

  if (error instanceof Error) {
    return error.message || String(error);
  }

  if (error && typeof error === "object") {
    const serialized = error as SerializedAppError;
    const message = typeof serialized.message === "string" ? serialized.message : "";
    const code = typeof serialized.code === "string" ? serialized.code : "";
    if (code && i18n.global.te(`errors.${code}`)) {
      return i18n.global.t(`errors.${code}`);
    }
    if (message) {
      return message;
    }
  }

  // 无法识别的错误统一返回稳定文案,避免显示 "[object Object]"
  return "未知错误";
}

/** 测试入口,行为与命令错误翻译一致 */
export function translateCommandErrorForTest(error: unknown): string {
  return translateCommandError(error);
}

/** 调用 Rust 命令,参数 key 用 camelCase(Tauri 自动映射 snake_case) */
export async function cmd<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (error) {
    // 保留 Tauri rejection 的原始 payload 供上层诊断
    const wrapped = new Error(translateCommandError(error)) as Error & { cause?: unknown };
    wrapped.cause = error;
    throw wrapped;
  }
}

/** 监听后端事件 */
export function onListen<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (e) => handler(e.payload));
}

/** 在系统终端里执行命令(新窗口,跑完不关);cwd 缺省为项目根目录 */
export function runInTerminal(project: Project, command: string, cwd?: string): Promise<unknown> {
  return cmd("run_in_terminal", {
    path: project.path,
    projectName: project.name,
    command,
    ...(cwd ? { cwd } : {}),
  });
}
