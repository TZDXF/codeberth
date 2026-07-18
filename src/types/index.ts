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
  file_name: string;
}

export type EditorKind = "vscode" | "explorer" | "terminal";

export interface GitUpdatedPayload {
  project_id: number;
  remote_ahead: number;
  last_fetch_at: number;
}

