# ProjectDev

基于 **Tauri 2 + Vue 3 + TypeScript** 的本地开发项目管理中心(桌面端)。把散落在各处的项目集中登记,在一个窗口里完成日常高频操作:跑脚本、管 Docker Compose、看 Git 状态并提交/推送、读 README、用喜欢的编辑器打开。

## 功能特性

- **项目管理**:添加/归档/删除本地项目,卡片与表格两种视图,支持按名称/描述搜索、标签筛选
- **脚本执行**:解析 `package.json` scripts 与自定义命令,分组折叠展示,一键在系统终端运行(Windows 优先 Windows Terminal,回退 cmd)
- **Docker Compose**:自动扫描 compose 文件,解析服务与端口,`compose ps` 运行状态指示(绿/黄/灰),浏览器直达服务端口
- **Git 集成**:状态总览、分支切换(本地+远程,自动建跟踪分支)、提交/拉取/推送;拉取冲突弹窗引导解决,push 被拒提示先拉取;未跟踪文件提交前显式勾选
- **Markdown 预览**:README 抽屉式渲染(Shiki 代码高亮、代码复制/折叠、表格复制与导出 CSV/TSV/MD),四套 MD 主题
- **多方式打开**:资源管理器 / VSCode(自动检测)/ 终端
- **个性化**:亮/暗主题 + island 皮肤、Markdown 主题、中/英文界面,设置持久化

## 技术栈

| 层 | 技术 |
| --- | --- |
| 前端 | Vue 3(`<script setup>`)、Vite、TypeScript、Pinia、Vue Router |
| UI | shadcn-vue(reka-ui)、Tailwind CSS v4、lucide 图标 |
| 渲染 | vue-stream-markdown(Shiki) |
| 后端 | Tauri 2(Rust)、rusqlite(bundled SQLite)、tokio |
| 国际化 | vue-i18n(zh-CN 默认 / en-US 回退) |
| 工具链 | oxlint(静态检查)、oxfmt(代码格式化) |

## 开发环境

- [Node.js](https://nodejs.org/) 18+ 与 [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/) 工具链
- Tauri 2 系统依赖(Windows 需 WebView2,见 [Tauri 官方文档](https://v2.tauri.app/start/prerequisites/))

## 常用命令

```bash
pnpm install        # 安装依赖
pnpm start          # tauri dev:完整桌面端开发(前端 + Rust 热更新)
pnpm dev            # 仅 Vite 前端(端口 1420)
pnpm build          # 类型检查(vue-tsc)+ 前端构建
pnpm build:desktop  # 打包桌面安装包(Windows NSIS)
pnpm lint           # oxlint 静态检查
pnpm lint:fix       # 自动修复可修 lint 问题
pnpm format         # oxfmt 格式化 src/
pnpm format:check   # 仅检查格式(CI 用)
```

> 已配置 oxlint(静态检查)与 oxfmt(代码格式化),测试框架暂未配置;提交前建议 `pnpm lint` + `pnpm build`。

## 目录结构

```
src/                    Vue 前端
  views/                页面:ProjectsHome / ProjectDetail / Settings
  components/           ui/(shadcn-vue) + git/markdown/project/scripts/settings/tags/open 业务组件
  stores/               Pinia:projects / settings / tags
  i18n/locales/         zh-CN.ts、en-US.ts(键需对齐)
  lib/                  cmd() Tauri 桥、工具函数
  styles/markdown/      MD 样式:base.css + themes/ 四套主题
src-tauri/
  src/commands/         Tauri 命令(project/git/script/docker/files/open/tag/walk)
  src/db/               SQLite 连接与迁移执行器
  migrations/           SQL 迁移(NNN_name.sql,按 user_version 幂等应用)
```

## 数据存储

应用数据统一存放在用户主目录下的 `~/.pm/`(Windows: `C:\Users\<用户名>\.pm\`):

- `projects.db` — SQLite 数据库(项目、标签、自定义命令)
- `settings.json` — 界面设置(主题、语言、默认打开方式等)

## 推荐 IDE 配置

[VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) + [Oxc](https://marketplace.visualstudio.com/items?itemName=oxc.oxc-vscode)(oxlint + oxfmt 一体)
