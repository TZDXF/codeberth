---
name: release-tagger
description: 在此仓库（tzdxf/repomeow，Tauri 2 + Vue 3）发布新版本。读取 `src-tauri/tauri.conf.json` 的 `version` 字段作为发布号，做一致性检查与构建验证后，走**本地打包发布**：`pnpm release:all` 在本机构建 NSIS 安装包并用 `~/.tauri/` 下的私钥签名，再用 `gh release create` 直接发布正式 Release（setup.exe + .sig + latest.json）。不推送 tag、不触发 GitHub Actions 构建。在用户提及"发版"、"发布新版本"、"本地打包发布"、"cut a release"、"v0.X.0"或想发起 GitHub Release 流程时调用此 skill——即便用户只是说"准备发版"，也走这条流程。
---

# 本地打包发布工作流

此 skill 走**本地打包发布**：构建、签名、生成 `latest.json` 全部在本机完成（脚本：`scripts/release/release.mjs`，用法详见 `scripts/release/README.md`)，最后用 `gh` CLI 发布 Release。**不要 `git push` 推 tag**——推送 `v*` tag 会触发 `.github/workflows/release.yml` 在 CI 重复打包。远程 tag 由 `gh release create` 在创建 Release 时自动生成。

## 前置条件

- Node 18+、pnpm 11+（仓库 `packageManager` 已锁版本）
- **签名私钥在 `~/.tauri/` 下**（流水线脚本自动发现唯一的 `*.key` 并交叉校验 `.pub` 与 `tauri.conf.json` 的 `plugins.updater.pubkey` 一致；不一致会直接报错，防止签出 updater 装不上的包）
- `gh` CLI 已登录 `TZDXF` 账号（`gh auth status`)
- CI 工作流 `release.yml` 仍然存在，作为本地不可用时的后备；本流程与其互不干扰

## 何时调用

- 用户说"发版"、"发布新版本"、"准备发版"、"本地打包发布"、"cut a release"
- 用户给出形如"发 0.2.0"、"v0.2.0"的具体版本号时，若与仓库当前 `version` 不一致，先 bump（见第 1 步）再继续——用户明确说"发布新版 X.Y.Z"即视为授权 bump

## 何时不调用

- 用户只想 bump 版本号、不真的发版 → 不调用
- 用户只想查看现有 tag → 不调用

## 工作流（按顺序执行，每步失败立即中止）

### 1. 读出版本号，必要时 bump

```bash
VERSION=$(grep -m1 '"version"' src-tauri/tauri.conf.json | sed -E 's/.*"version":\s*"([^"]+)".*/\1/')
REMOTE=$(git remote get-url github 2>/dev/null || git remote get-url origin)
```

期望 `REMOTE` 含 `github.com/TZDXF/repomeow`，否则中止并提示"仓库远程异常，请确认 github remote 配置"。

若用户要求的版本号 ≠ `$VERSION`，同步修改四处：

- `src-tauri/tauri.conf.json` 的 `"version"`
- `package.json` 的 `"version"`
- `src-tauri/Cargo.toml` 的 `version = "..."`
- `src-tauri/Cargo.lock` 中 `name = "repomeow"` 紧随的 `version`（改完 Cargo.toml 后跑任意 cargo 命令也会自动更新）

改完用 `pnpm release:check` 校验三处一致。

### 2. 工作区状态

```bash
git status --short
git rev-list --count github/main..HEAD
```

若 `git status` 非空（未提交改动或 untracked 文件）：
- 默认行为：**中止**并提示"工作区有未提交改动，请先 commit"
- 如果改动只有版本号 bump 相关文件，允许代提交后继续，提交信息沿用历史风格：`🔖 chore(release): bump 版本号至 X.Y.Z`，且必须明确告诉用户你做了什么

### 3. 构建验证

```bash
pnpm lint      # 0 errors 即可,style 告警不阻塞
pnpm build     # 含 vue-tsc --noEmit 类型检查,最高保真预发布闸门
```

任何一条失败 → 中止，把输出原样贴回给用户。

### 4. 提交 bump 并推送 main

```bash
git add package.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json
git commit -m "🔖 chore(release): bump 版本号至 $VERSION"
git push github main
```

main 必须先于 Release 推送——`gh release create` 从远程 main HEAD 生成 tag，要保证 tag 指向 bump 提交。

### 5. 创建本地 tag（不推送）

```bash
git tag -a "v$VERSION" -m "Release v$VERSION"
```

- 若 tag 已存在 → 中止并用 `git tag -l "v*"` 列出
- **绝不 `git push github "v$VERSION"`**：推送 `v*` tag 会触发 CI `release.yml` 重复打包并产生草稿 Release

### 6. 本地构建 + 签名 + latest.json

```bash
pnpm release:all
```

串行执行 check → build(`pnpm install --frozen-lockfile` + `pnpm build:desktop`)→ sign → latest。耗时主要在同机 Rust release 编译（约 1–3 分钟），建议后台运行。产物在 `src-tauri/target/release/bundle/nsis/`:

- `RepoMeow_<ver>_x64-setup.exe`
- `RepoMeow_<ver>_x64-setup.exe.sig`
- `latest.json`

完成后**核对 `latest.json`**:`version` 正确、`platforms.windows-x86_64.url` 指向 `releases/latest/download/RepoMeow_<ver>_x64-setup.exe`、`signature` 非空。

### 7. 整理 Release Notes

```bash
git log --oneline v<上一版本>..HEAD
```

按提交归纳为用户可读的变更说明（参考 v0.1.3 的 Release 风格），写入临时文件（如 `scripts/release/_local/v<ver>-notes.md`,`_local/` 已在 gitignore)。

### 8. 用 gh 发布正式 Release

```bash
gh release create "v$VERSION" \
  --repo TZDXF/repomeow \
  --title "v$VERSION" \
  --notes-file <notes 文件> \
  src-tauri/target/release/bundle/nsis/RepoMeow_${VERSION}_x64-setup.exe \
  src-tauri/target/release/bundle/nsis/RepoMeow_${VERSION}_x64-setup.exe.sig \
  src-tauri/target/release/bundle/nsis/latest.json
```

- 不加 `--draft` 即直接发布为正式 latest release；用户若想先检查后发布，加 `--draft` 并提示其手动 Publish
- `gh` 会自动在远程创建 `v$VERSION` 轻量 tag（与本地 annotated tag 同指 bump 提交；之后 `git fetch --tags` 若报 `would clobber existing tag` 属正常，无需处理）

### 9. 发布验证

```bash
gh release view "v$VERSION" --json isDraft,isPrerelease,assets
gh api repos/TZDXF/repomeow/releases/latest --jq '.tag_name'   # 应为 v$VERSION
curl -sL https://github.com/TZDXF/repomeow/releases/latest/download/latest.json   # version 应为新版本
```

三项全部通过后告诉用户：旧版应用下次启动会自动检查到 `v$VERSION`(updater 端点即上面的 latest.json)。注意端点刚发布可能有几分钟 CDN 缓存，curl 失败可隔几秒重试。

## 失败模式速查

| 现象 | 原因 | 处置 |
|---|---|---|
| `git remote get-url github` 报错 | 该机器未配 `github` remote 名 | 中止，请用户用 `git remote -v` 检查 |
| `pnpm build` 失败 | TS 错误或前端构建错误 | 中止，把 build 输出原样贴回给用户 |
| tag 已存在 | 之前发过或本地残留 | 中止并附 `git tag -l "v*"` 输出 |
| 三处版本号不一致 | 用户或脚本只改了其一 | 中止并指出每个文件的当前值，等用户修齐再跑 |
| 工作区脏 | 还有未提交改动 | 默认中止；若是 bump 文件可代 commit 后继续，告知细节 |
| `No .key file found in ~/.tauri` | 本机没签名私钥 | 中止；用 `cargo tauri signer generate` 生成并把 pubkey 写进 `tauri.conf.json` |
| `Public key mismatch` | `~/.tauri/*.key.pub` 与 `tauri.conf.json` 的 pubkey 不一致 | 中止；用 `--key` 指定正确私钥或修齐 pubkey（详见 `scripts/release/README.md` 故障排查表） |
| 打印 `Signing without password.` 但没 .sig | Tauri 2.11.4 signer 静默 bug | 流水线脚本已内置规避（强制空密码环境变量）；手工签名时需自设 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""` |
| `gh release create` 报 tag 已存在 | 远程已有该 tag（之前发过） | 中止；确认是否重复发版，必要时先 `gh release delete` |
| `git fetch --tags` 报 `would clobber existing tag` | 本地 annotated tag 与 gh 创建的轻量 tag 对象不同，但同指一个提交 | 正常，忽略即可 |
