import { generateText } from "ai";
import { createOpenAI } from "@ai-sdk/openai";
import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
import { i18n, type SupportedLocale } from "@/i18n";
import { DEFAULT_COMMIT_PROMPT, DEFAULT_REPORT_PROMPT, loadAiPrompts } from "@/lib/ai-prompts";
import { useSettingsStore } from "@/stores/settings";
import type { GitCommitContext, GitCommitInfo } from "@/types";

/** 一个项目在给定时间范围内的提交记录(日报输入) */
export interface ProjectCommits {
  projectName: string;
  /** 项目描述,帮助模型理解业务语境;可能为空串 */
  projectDescription: string;
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

/** 组装 system prompt:用户自定义(~/.pm/prompts/*.md)优先,空则回退内置默认;输出语言指令统一追加 */
function buildSystemPrompt(custom: string, fallback: string, language: SupportedLocale) {
  const base = custom.trim() || fallback;
  return `${base}\n\nRespond in ${languageName(language)}.`;
}

/** 根据当前变更上下文生成 git 提交信息;user 提示词携带项目名称与描述帮助模型理解业务语境 */
export async function generateCommitMessage(
  ctx: GitCommitContext,
  project: { name: string; description: string },
  language: SupportedLocale,
): Promise<string> {
  const prompts = await loadAiPrompts();
  const description = project.description.trim();
  const projectSection = `Project: ${project.name}${description ? `\nDescription: ${description}` : ""}`;
  const untracked = ctx.untracked.length
    ? `\nUntracked new files (no diff content available):\n${ctx.untracked.join("\n")}`
    : "";
  const truncatedNote = ctx.truncated ? "\n(Note: the diff was truncated due to length.)" : "";
  const { text } = await generateText({
    model: getChatModel(),
    system: buildSystemPrompt(prompts.commit, DEFAULT_COMMIT_PROMPT, language),
    prompt: `${projectSection}

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
  const prompts = await loadAiPrompts();
  const sections = data
    .map((p) => {
      const lines = p.commits
        .map((c) => `- [${c.date}] ${c.subject} (${c.hash}, ${c.author})`)
        .join("\n");
      const description = p.projectDescription.trim();
      const heading = description ? `${p.projectName} — ${description}` : p.projectName;
      return `### ${heading}\n${lines || "(no commits)"}`;
    })
    .join("\n\n");
  const { text } = await generateText({
    model: getChatModel(),
    system: buildSystemPrompt(prompts.report, DEFAULT_REPORT_PROMPT, language),
    prompt: `Time range: ${rangeLabel}.

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
