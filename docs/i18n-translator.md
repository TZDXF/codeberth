# i18n Translator 子代理

> 本项目专用的国际化（i18n）翻译子代理，托管在 `.zcode/skills/i18n-translator/`，由 ZCode 主代理按需通过 `Task` 工具调用。
>
> **状态**：智能体已就绪，但**未对源码做任何改动**。当前 i18n 仍处于"先观察后改造"阶段。

---

## 1. 智能体是什么

- **形态**：ZCode Skill（子代理），定义在 `.zcode/skills/i18n-translator/SKILL.md`。
- **触发方式**：由主代理在 `Task` 工具中以 `subagent_type: "i18n-translator"` 调用；**不**在输入框 `/` 菜单里暴露。
- **目标**：把项目内硬编码的用户可见中文，逐步迁移到多语言架构，首批支持 `zh-CN`（源）与 `en-US`（目标）。

## 2. 何时调用

| 场景 | 在主代理里说 |
| --- | --- |
| 想看全量硬编码中文清单 | "调用 i18n 子代理扫描 src 下所有中文" |
| 拿到一份中文文案想翻译 | "让翻译子代理把这段文案翻成英文" |
| 改造前评审 | "让翻译子代理给一份 vue-i18n 接入计划" |
| 改造后回归 | "让翻译子代理审计一下 views 是否还有硬编码" |

## 3. 智能体能做什么

四种工作模式（见 SKILL.md 的 `mode` 字段）：

- `scan` —— 扫描硬编码中文，输出"建议键名 / 原文 / 行号"清单。
- `translate` —— 给定键名或原文，返回 `zh-CN` + `en-US` 译文。
- `audit` —— 检查指定目录是否仍有硬编码或漏译。
- `plan` —— 生成最小侵入的 `vue-i18n` 接入 diff 草稿（**不会自动落地**）。

## 4. 智能体目前**不会**做的事

- 不会自动修改 `package.json` 安装 `vue-i18n`。
- 不会自动改写 `.vue` / `.ts` 源码。
- 不会翻译代码标识符、Git 提交、CHANGELOG、配置文件元信息。

任何落地动作都需用户在主代理里显式授权。

## 5. 未来的 i18n 路线图（待你点头再启动）

1. **阶段 A · 基础设施** — 安装 `vue-i18n`、新增 `src/i18n/index.ts`、在 `main.ts` 注册、新建 `useLocaleStore` 持久化语言偏好。
2. **阶段 B · 词条库** — 新建 `src/i18n/locales/{zh-CN,en-US}.ts`，由子代理首批扫描结果填充；用 `as const` + `declare module` 给键名加 TS 强类型。
3. **阶段 C · 代码替换** — 从 `App.vue` 入手，逐步把硬编码替换为 `t('...')`；store 内的 `toast` 文案走 `i18n.global.t(...)`。
4. **阶段 D · 扩展语言** — 复制词条文件到 `ja-JP` / `zh-TW` 等，子代理直接产出目标语译文；UI 增加语言切换器。

## 6. 当前已识别的文案高发区（供后续 `scan` 优先覆盖）

- `src/components/**/*.vue` — 按钮、菜单项、对话框标题/描述/占位符/空状态。
- `src/views/ProjectsHome.vue` / `ProjectDetail.vue` — 页面标题、Tab、空状态、统计文案。
- `src/stores/projects.ts` / `src/stores/tags.ts` — `vue-sonner` 的 `toast.success/error` 文本。
- `src/components/git/GitStatusBar.vue` — Git 状态展示文案。

## 7. 修改/停用这个智能体

- 改能力：编辑 `.zcode/skills/i18n-translator/SKILL.md`。
- 临时停用：把 `SKILL.md` 的 `description` 字段置空，ZCode 将不再自动匹配；或把该 skill 目录移动到 `.zcode/skills/.disabled/`。
- 升级为显式命令：把 SKILL.md 的 `description` 改为不含 trigger 关键词的简短描述，并在 `.zcode/commands/i18n-translate.md` 新建一个调用入口。
