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
  modified: number;
  untracked: number;
  remote_ahead: number;
  last_fetch_at: number | null;
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

export interface ComposeFile {
  /** 相对项目根的路径('/' 分隔),如 "compose.yml" 或 "deploy/app.yml" */
  path: string;
  file_name: string;
  services: string[];
}

export type EditorKind = "vscode" | "explorer" | "terminal";

export interface GitUpdatedPayload {
  project_id: number;
  remote_ahead: number;
  last_fetch_at: number;
}

