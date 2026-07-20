/** git remote 地址解析:识别托管平台并归一化为可浏览器访问的 https 地址 */

export type GitProvider = "github" | "gitee" | "gitlab" | "generic";

export interface GitRemoteInfo {
  provider: GitProvider;
  /** 可在浏览器打开的仓库主页地址 */
  url: string;
}

function detectProvider(host: string): GitProvider {
  const h = host.toLowerCase();
  if (h.includes("github")) {
    return "github";
  }
  if (h.includes("gitee")) {
    return "gitee";
  }
  if (h.includes("gitlab")) {
    return "gitlab";
  }
  return "generic";
}

/** 解析 `git remote get-url` 输出,支持 https / ssh:// / scp 风格(git@host:owner/repo.git) */
export function parseGitRemote(remote: string): GitRemoteInfo | null {
  const raw = remote.trim();
  if (!raw) {
    return null;
  }

  let host = "";
  let repoPath = "";

  // scp 风格: git@github.com:owner/repo.git(无协议头,且冒号不在端口位)
  const scp = /^[\w.-]+@(?<host>[\w.-]+):(?<path>.+)$/.exec(raw);
  if (scp?.groups && !raw.includes("://")) {
    host = scp.groups.host;
    repoPath = scp.groups.path;
  } else {
    let url: URL;
    try {
      url = new URL(raw.includes("://") ? raw : `https://${raw}`);
    } catch {
      return null;
    }
    host = url.hostname;
    repoPath = url.pathname.replace(/^\//, "");
  }

  repoPath = repoPath.replace(/\.git$/i, "").replace(/\/+$/, "");
  if (!host || !repoPath) {
    return null;
  }

  return { provider: detectProvider(host), url: `https://${host}/${repoPath}` };
}
