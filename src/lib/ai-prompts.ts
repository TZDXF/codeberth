import { cmd } from "@/lib/tauri";
import type { AiPrompts } from "@/types";

/**
 * 内置默认提示词(提交信息生成)。
 * 用户在 ~/.pm/prompts/commit.md 中没有自定义内容时使用;
 * 输出语言指令由调用方按当前语言设置自动追加,无需写入模板
 */
export const DEFAULT_COMMIT_PROMPT = `You write concise, high-quality git commit messages.

Requirements:
- Use the Conventional Commits format: "type: summary", e.g. feat / fix / refactor / docs / chore / perf / test / build.
- The first line (subject) must be a single line of at most 72 characters.
- Optionally add one blank line followed by a short body with bullet points for important details; omit the body for small changes.
- Output ONLY the commit message itself. No explanations, no quotes, no markdown code fences.`;

/** 内置默认提示词(日报生成),同上 */
export const DEFAULT_REPORT_PROMPT = `You are an assistant that writes clear, professional daily work reports in Markdown.

Report requirements:
- Output Markdown. Start with a top-level summary (2-4 sentences), then one section per project.
- Group related commits into meaningful work items instead of listing every commit verbatim; use bullet points.
- Keep it factual and concise; do not invent work that is not reflected in the commits.
- Output ONLY the report Markdown itself.`;

/** 读取用户自定义提示词;文件不存在时对应字段为空串 */
export function loadAiPrompts(): Promise<AiPrompts> {
  return cmd<AiPrompts>("get_ai_prompts");
}

/** 保存提示词;字段为空白时删除对应文件(恢复默认) */
export function saveAiPrompts(prompts: AiPrompts): Promise<void> {
  return cmd<void>("set_ai_prompts", { prompts });
}

/** 在系统文件管理器中打开提示词目录(~/.pm/prompts/) */
export function openPromptsDir(): Promise<void> {
  return cmd<void>("open_prompts_dir");
}
