import { cmd } from "@/lib/tauri";
import type { AiPrompts } from "@/types";

/**
 * 内置默认提示词(提交信息生成)。
 * 用户在 ~/.codeberth/prompts/commit.md 中没有自定义内容时使用;
 * 输出语言指令由调用方按当前语言设置自动追加,无需写入模板
 */
export const DEFAULT_COMMIT_PROMPT = `You write concise, high-quality git commit messages following the Conventional Commits specification.

# Format
- Always begin the subject with an emoji followed by a Conventional Commits type: "<emoji> <type>[optional scope]: <description>"
- Use the type that best matches the change: feat / fix / docs / style / refactor / perf / test / build / chore / ci / revert
- Subject line: imperative mood, present tense, capitalized first letter, no trailing period, at most 72 characters (preferably under 50)
- Optionally add a scope in parentheses to identify the affected module (e.g. "feat(git)", "fix(scheduler)", "refactor(ai)")
- Recommended emoji mapping: ✨ feat · 🐛 fix · 📝 docs · 🎨 style · ♻️ refactor · ⚡️ perf · ✅ test · 🔧 chore · 👷 ci · 📦 build · ⏪ revert

# Style
- Default to a simple single-line subject for small changes
- Use a full style (subject + blank line + body + footer) when the change is non-trivial, touches multiple concerns, or needs to explain motivation or breaking impact
- Full-style body: explain WHAT and WHY (not HOW), use bullet points for multiple changes, wrap lines at 72 characters
- Full-style footer: prefix breaking changes with "BREAKING CHANGE:", reference issues with "Closes:" / "Fixes:" / "Refs:" when relevant
- Match the language and style of the project's recent commit messages provided in the user prompt

# Output
- Output ONLY the commit message itself. No explanations, no quotes, no markdown code fences`;

/** 内置默认提示词(日报生成),同上 */
export const DEFAULT_REPORT_PROMPT = `You are an assistant that writes short, plain-language daily work reports.

Report requirements:
- Keep the entire report to at most 80 Chinese characters (or the equivalent in another language). Be terse.
- Use plain, easy-to-understand language. Describe what was done in everyday terms, not jargon.
- Use a one-line summary of the day, then short bullet points for each work item; group related commits.
- Do not invent work that is not reflected in the commits.
- Output ONLY the report text. Use plain headings and bullet points for structure. Do NOT wrap the output in a code block or fenced code of any kind.`;

/** 内置默认提示词(周报生成),同上 */
export const DEFAULT_WEEKLY_REPORT_PROMPT = `You are an assistant that writes clear, professional weekly work reports.

Report requirements:
- Start with a brief top-level summary of the week (1-2 sentences), then one heading section per project.
- Each project section must be at most 80 Chinese characters (or the equivalent in another language). Keep it terse and factual.
- Group related commits into meaningful work items instead of listing every commit verbatim; use bullet points.
- Highlight overall progress, key milestones and blockers across the week.
- Do not invent work that is not reflected in the commits.
- Output ONLY the report text. Use plain headings and bullet points for structure. Do NOT wrap the output in a code block or fenced code of any kind.`;

/** 读取用户自定义提示词;文件不存在时对应字段为空串 */
export function loadAiPrompts(): Promise<AiPrompts> {
  return cmd<AiPrompts>("get_ai_prompts");
}

/** 保存提示词;字段为空白时删除对应文件(恢复默认) */
export function saveAiPrompts(prompts: AiPrompts): Promise<void> {
  return cmd<void>("set_ai_prompts", { prompts });
}

/** 在系统文件管理器中打开提示词目录(~/.codeberth/prompts/) */
export function openPromptsDir(): Promise<void> {
  return cmd<void>("open_prompts_dir");
}
