import { generateText } from "ai";
import { createOpenAI } from "@ai-sdk/openai";
import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
import { i18n, type SupportedLocale } from "@/i18n";
import { useSettingsStore } from "@/stores/settings";
import type { GitCommitContext, GitCommitInfo } from "@/types";

/** 一个项目在给定时间范围内的提交记录(日报输入) */
export interface ProjectCommits {
  projectName: string;
  commits: GitCommitInfo[];
}

/** 读取 AI 配置;未配置 API Key 时抛出带本地化文案的错误 */
function requireConfig() {
  const settings = useSettingsStore();
  const apiKey = settings.aiApiKey.trim();
  if (!apiKey) {
    throw new Error(i18n.global.t("ai.notConfigured"));
  }
  return {
    baseURL: settings.aiBaseUrl.trim(),
    apiKey,
    model: settings.aiModel.trim(),
  };
}

/**
 * 构造 OpenAI Chat Completions 兼容模型。
 * 显式使用 .chat()(而非默认的 Responses API),兼容 DeepSeek/Moonshot/各类中转服务;
 * fetch 走 Tauri HTTP 插件(Rust 侧发请求),规避 webview 的 CORS 限制
 */
function getChatModel() {
  const { baseURL, apiKey, model } = requireConfig();
  const openai = createOpenAI({
    baseURL,
    apiKey,
    fetch: tauriFetch as unknown as typeof globalThis.fetch,
  });
  return openai.chat(model);
}

function languageName(language: SupportedLocale) {
  return language === "zh-CN" ? "中文" : "English";
}

/** 根据当前变更上下文生成 git 提交信息 */
export async function generateCommitMessage(
  ctx: GitCommitContext,
  language: SupportedLocale,
): Promise<string> {
  const untracked = ctx.untracked.length
    ? `\nUntracked new files (no diff content available):\n${ctx.untracked.join("\n")}`
    : "";
  const truncatedNote = ctx.truncated ? "\n(Note: the diff was truncated due to length.)" : "";
  const { text } = await generateText({
    model: getChatModel(),
    system: "You write concise, high-quality git commit messages.",
    prompt: `Write a git commit message in ${languageName(language)} for the following changes.

Requirements:
- Use the Conventional Commits format: "type: summary", e.g. feat / fix / refactor / docs / chore / perf / test / build.
- The first line (subject) must be a single line of at most 72 characters.
- Optionally add one blank line followed by a short body with bullet points for important details; omit the body for small changes.
- Output ONLY the commit message itself. No explanations, no quotes, no markdown code fences.

Change summary (git diff --stat):
${ctx.stat || "(none)"}

Diff:${truncatedNote}
${ctx.diff || "(empty)"}${untracked}`,
  });
  return text.trim();
}

/** 汇总多个项目的提交记录,生成 Markdown 日报 */
export async function generateDailyReport(
  data: ProjectCommits[],
  rangeLabel: string,
  language: SupportedLocale,
): Promise<string> {
  const sections = data
    .map((p) => {
      const lines = p.commits
        .map((c) => `- [${c.date}] ${c.subject} (${c.hash}, ${c.author})`)
        .join("\n");
      return `### ${p.projectName}\n${lines || "(no commits)"}`;
    })
    .join("\n\n");
  const { text } = await generateText({
    model: getChatModel(),
    system: "You are an assistant that writes clear, professional daily work reports in Markdown.",
    prompt: `Based on the following git commit records, write a daily work report in ${languageName(language)}.

Report requirements:
- Time range: ${rangeLabel}.
- Output Markdown. Start with a top-level summary (2-4 sentences), then one section per project.
- Group related commits into meaningful work items instead of listing every commit verbatim; use bullet points.
- Keep it factual and concise; do not invent work that is not reflected in the commits.
- Output ONLY the report Markdown itself.

Commit records:
${sections}`,
  });
  return text.trim();
}

/** 测试连接:发一条极短请求验证 baseURL / apiKey / model 可用 */
export async function testAiConnection(): Promise<void> {
  await generateText({
    model: getChatModel(),
    prompt: "Reply with the single word: ok",
    maxOutputTokens: 8,
  });
}
