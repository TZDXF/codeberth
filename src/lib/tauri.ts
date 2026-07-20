import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Project } from "@/types";

/** 调用 Rust 命令,参数 key 用 camelCase(Tauri 自动映射 snake_case) */
export function cmd<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(name, args);
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
