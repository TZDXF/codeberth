# AGENTS.md

本文件供 ZCode 代理快速了解本仓库的工作方式。只记录从代码中不易直接看出的约定。

## 项目概述

Tauri 2 + Vue 3 + TypeScript 桌面应用(项目名称 `tauri-appproject-dev`):本地开发项目管理中心,功能包括项目登记/归档、npm scripts 与自定义命令执行、docker compose 服务管理、git 状态与写操作(提交/拉取/推送/分支切换)、README Markdown 渲染、标签、多编辑器打开。

- 包管理器:**pnpm**(有 `pnpm-lock.yaml` / `pnpm-workspace.yaml`,勿用 npm)。
- 主要平台:Windows(终端启动优先 `wt.exe`,失败回退 `cmd`);存在 macOS 交叉编译产物。
- 提交信息:中文 + conventional 前缀(`feat:` 等),单行详细描述,参考 `git log`。

## 常用命令

| 命令 | 说明 |
| --- | --- |
| `pnpm start` | `tauri dev`,完整桌面端开发(前端 + Rust) |
| `pnpm dev` | 仅 Vite 前端(固定端口 1420,strictPort) |
| `pnpm build` | `vue-tsc --noEmit && vite build`,**唯一的类型检查手段** |
| `pnpm build:desktop` | `tauri build` 打包 |

仓库**没有配置 lint 和测试框架**;改动后用 `pnpm build`(或 `npx vue-tsc --noEmit`)验证。Rust 侧用 `cargo check`(在 `src-tauri/` 下)。

## 架构与分层

```
src/                  Vue 3 前端(<script setup> SFC)
  views/              三个页面:ProjectsHome / ProjectDetail / Settings
  components/ui/      shadcn-vue(reka-ui)组件,勿手改生成文件风格
  components/{git,markdown,project,scripts,settings,tags,open}/  业务组件
  stores/             Pinia(projects / settings / tags)
  i18n/locales/       zh-CN.ts(默认)、en-US.ts(回退),两文件键必须对齐
  lib/tauri.ts        前后端桥:cmd<T>() 封装 invoke
  styles/markdown/    MD 样式分层:base.css(结构) + themes/*.css(四套主题)
src-tauri/src/
  lib.rs              插件注册、Db 初始化、invoke_handler 命令清单
  commands/*.rs       Tauri 命令(project/git/script/docker/files/open/tag/walk)
  db/                 rusqlite 连接(全局 Mutex 单连接) + 迁移执行器
  error.rs            AppError / AppResult,错误序列化为中文字符串传前端
src-tauri/migrations/ SQL 迁移,NNN_name.sql
```

关键规则:

1. **新增 Rust 命令**:在 `commands/*.rs` 实现(返回 `AppResult<T>`)后,必须在 `lib.rs` 的 `invoke_handler!` 里注册,前端经 `cmd<T>("snake_case 名", { camelCase 参数 })` 调用(Tauri 自动做参数名映射)。
2. **数据库**:SQLite 文件在 `~/.pm/projects.db`(Windows: `C:\Users\<user>\.pm\`)。改表结构 = 新增 `migrations/00N_xxx.sql` + 在 `db/migrations.rs` 按 `PRAGMA user_version` 顺序应用,保证幂等。不要改已发布的迁移文件。
3. **应用数据目录名 `.pm`** 在 Rust(`lib.rs` 的 `APP_DATA_DIR_NAME`)和前端(`stores/settings.ts`)各有一份常量,改动需同步。设置持久化走 `tauri-plugin-store` → `~/.pm/settings.json`。
4. **路径别名** `@/` → `src/`(tsconfig + vite 均已配置)。

## 前端约定

- **UI 体系**:shadcn-vue + Tailwind CSS v4 + lucide 图标;样式合并用 `@/lib/utils` 的 `cn()`。主题用 CSS 变量,亮/暗经根节点 `.dark` 类切换,皮肤经 `data-theme="island"`。
- **Markdown 渲染**:用 `vue-stream-markdown`(Shiki 高亮);MD 主题经根节点 `data-md-theme` 属性切换(default/github/notion/serif);自定义图片/链接渲染器要保留本地 `asset:` 协议与系统打开行为(Cargo.toml 已启用 `protocol-asset` feature)。
- **i18n**:所有用户可见文案走 `vue-i18n`,键定义在 `src/i18n/locales/zh-CN.ts` 与 `en-US.ts`,新增键两语言必须同时补。仓库有专用翻译子代理(`.zcode/skills/i18n-translator/`,用法见 `docs/i18n-translator.md`),批量翻译/审计时优先调用它。Rust 侧错误文案(error.rs)目前是硬编码中文,属已知现状。
- **TS 严格模式**:`noUnusedLocals` / `noUnusedParameters` 开启,未用变量会导致 build 失败。

## 注意事项(Gotchas)

- git 相关命令已禁用终端凭据交互询问;涉及凭证的改动注意保持非交互。
- `run_in_terminal` 在系统终端新窗口执行命令,Windows 优先 Windows Terminal。
- Vite dev 端口 1420 被占用会直接失败(strictPort),`tauri dev` 前先确认端口空闲。
- `src-tauri/target-cdk/` 与 `scripts/` 是未跟踪的本地目录,勿当源码处理。
- 改动设置项时同步检查:`stores/settings.ts` 的持久化默认值、Settings 页面 UI、i18n 词条三处。
