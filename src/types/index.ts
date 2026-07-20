export interface Tag {
  id: number;
  name: string;
  color: string;
}

export interface GitStatus {
  is_repo: boolean;
  branch: string | null;
  ahead: number;
  behind: number;
  staged: number;
  /** 未暂存修改数(含冲突文件) */
  modified: number;
  untracked: number;
  /** 合并冲突文件数 */
  conflicted: number;
  remote_ahead: number;
  last_fetch_at: number | null;
}

/** `git pull` 的结果:最新状态 + 产生的合并冲突文件(为空表示无冲突) */
export interface GitPullResult {
  status: GitStatus;
  conflicts: string[];
}

/** 本地/远程分支列表(remote 不含 origin/HEAD 这类符号引用) */
export interface GitBranches {
  local: string[];
  remote: string[];
}

export interface Project {
  id: number;
  path: string;
  name: string;
  description: string;
  tags: Tag[];
  git: GitStatus | null;
  archived_at: number | null;
  created_at: number;
  updated_at: number;
}

export interface PackageScript {
  name: string;
  command: string;
}

/** 一个 package.json 的 scripts 分组(monorepo 下可能有多个) */
export interface PackageScriptsGroup {
  /** package.json 所在目录的相对路径('/' 分隔),根目录为 "." */
  dir: string;
  /** package.json 的 name 字段,可能为空 */
  package_name: string | null;
  scripts: PackageScript[];
}

export interface CustomCommand {
  id: number;
  project_id: number;
  name: string;
  command: string;
  description: string;
  icon: string;
  sort_order: number;
}

export interface ReadmeContent {
  file_name: string;
  content: string;
}

/** compose 文件中的一个服务及其对外可访问的宿主机端口 */
export interface ComposeService {
  name: string;
  /** 映射到宿主机的端口(去重升序);仅含可浏览器访问的固定发布端口 */
  ports: number[];
}

export interface ComposeFile {
  /** 相对项目根的路径('/' 分隔),如 "compose.yml" 或 "deploy/app.yml" */
  path: string;
  file_name: string;
  services: ComposeService[];
}

/** `docker compose ps` 查询到的单个服务运行状态 */
export interface ComposeServiceState {
  name: string;
  running: boolean;
  /** 原始状态文案,如 "Up 2 hours" / "Exited (0) 5 minutes ago" */
  status: string;
}

export type EditorKind = "vscode" | "explorer" | "terminal";

export interface GitUpdatedPayload {
  project_id: number;
  remote_ahead: number;
  last_fetch_at: number;
}
