---
name: release-tagger
description: 在此仓库（tzdxf/repomeow，Tauri 2 + Vue 3）发布新版本。读取 `src-tauri/tauri.conf.json` 的 `version` 字段作为发布号，做一致性检查与构建验证后创建 `vX.Y.Z` tag 并推送到 `github` 远程，触发既有的 `.github/workflows/release.yml` 打包草稿 Release。在用户提及"发版"、"发布新版本"、"打 tag 上传"、"cut a release"、"v0.X.0"或想发起 GitHub Release 流程时调用此 skill——即便用户只是说"准备发版"，也走这条流程。
---

# 发布 Tag 工作流

此 skill 不替用户调用 GitHub Release API。它只做"前奏"：校验版本、运行类型/构建检查、暂存脏改动（可选）、打 tag、推送 `github` 远程。后续打包与草稿 Release 由既有的 `.github/workflows/release.yml` 完成。

## 何时调用

- 用户说"发版"、"发布新版本"、"准备发版"、"cut a release"
- 用户给出形如"发 0.2.0"、"v0.2.0"的具体版本号时，**仍然以仓库的 `version` 为准**作为单一真理源；但可以在回复中提示用户"仓库当前为 0.1.0，与你期望的 0.2.0 不一致，是否要先改版本号？"

## 何时不调用

- 用户只想 bump 版本号、不真的发版 → 不调用
- 用户只想查看现有 tag → 不调用

## 工作流（按顺序执行，每步失败立即中止）

### 1. 读出版本号并定位远程

```bash
VERSION=$(grep -m1 '"version"' src-tauri/tauri.conf.json | sed -E 's/.*"version":\s*"([^"]+)".*/\1/')
REMOTE=$(git remote get-url github 2>/dev/null || git remote get-url origin)
```

期望 `REMOTE` 含 `github.com/TZDXF/repomeow`，否则中止并提示"仓库远程异常，请确认 github remote 配置"。

若用户传入可选参数 `version=<X.Y.Z>`，先 echo "仓库当前版本 $VERSION，期望版本 <X.Y.Z>，是否继续？" 并停在此步等用户确认；用户确认前不要执行后续步骤。

### 2. 三处版本号一致性检查

```bash
PKG=$(grep -m1 '"version"' package.json | sed -E 's/.*"version":\s*"([^"]+)".*/\1/')
CARGO=$(grep -m1 '^version =' src-tauri/Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')
```

`tauri.conf.json == package.json == Cargo.toml` 三者必须相等。任何一对不一致 → 中止并指出哪一个不一致以及建议在哪个文件里改齐。

### 3. 工作区状态

```bash
git status --short
git rev-list --count @{u}..HEAD 2>/dev/null   # 领先上游多少个提交,可能为 0
```

若 `git status` 非空（未提交改动或 untracked 文件）：
- 默认行为：**中止**并提示"工作区有未提交改动，请先 commit"
- 如果改动是更新版本号三个文件之一（dev 中刚刚 bump），允许暂存并继续，但必须明确告诉用户你做了什么（哪些文件被 `git add -A`，哪些被 `git commit`）

分支应领先 `github/main` 至少 1 个提交（否则没必要发版）。若落后/分叉，提示 `git pull --rebase` 或 `git push` 同步。

### 4. 构建验证

```bash
pnpm install --frozen-lockfile
pnpm build
```

任何一条失败 → 中止。该仓库 build 包含 `vue-tsc --noEmit` 类型检查，是最高保真的预发布闸门。`pnpm lint` 可选（CI 不强制），但建议在 `pnpm build` 前手动跑一次。

### 5. 创建并推送 tag

```bash
git tag -a "v$VERSION" -m "Release v$VERSION"
git push github "v$VERSION"
```

- 远程名固定 `github`（仓库已配）
- 不要用 `--force`，增量发版不要覆盖既有历史；若 tag 已存在 → 中止并用 `git tag -l "v*"` 列出

### 6. 输出后续提示

推送成功后清晰告诉用户：

```
已推送 tag v$VERSION → github
GitHub Actions 正在构建 Windows NSIS 安装包。
后续步骤:
  1. 打开 https://github.com/TZDXF/repomeow/releases → 进入 draft Release
  2. 检查 assets: <baseName>_<version>_x64-setup.exe + latest.json + .sig 文件是否齐全
  3. 勾选 "Set as the latest release" 后点 Publish
  4. latest.json 端点 https://github.com/TZDXF/repomeow/releases/latest/download/latest.json 生效后,旧版应用下次启动会自动检查到 v$VERSION
```

提醒：仓库使用的 `TAURI_SIGNING_PRIVATE_KEY` Secret 必须已配置（在仓库 Settings → Secrets），否则构建会在签名步骤失败。

## 用法模板（给模型作为参考）

用户 prompt 形如：
- "发版"、"帮我发个新版"
- "发布 0.2.0"、"切个 release"
- "/release-tagger"

按本工作流 1–6 跑即可。

## 失败模式速查

| 现象 | 原因 | 处置 |
|---|---|---|
| `git remote get-url github` 报错 | 该机器未配 `github` remote 名 | 中止，请用户用 `git remote -v` 检查 |
| `pnpm build` 失败 | TS 错误或前端构建错误 | 中止,把 build 输出原样贴回给用户 |
| tag 已存在 | 之前发过或本地残留 | 中止并附 `git tag -l "v*"` 输出 |
| 三处版本号不一致 | 用户或脚本只改了其一 | 中止并指出每个文件的当前值,等用户修齐再跑 |
| 工作区脏 | 还有未提交改动 | 默认中止;若是 bump 文件可代 commit 后继续,告知细节 |
