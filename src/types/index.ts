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

/** 一个 git remote 及其地址 */
export interface GitRemote {
  name: string;
  url: string;
}

/** 一个可读取文本内容的未跟踪新文件(二进制/超限文件不在此列) */
export interface GitUntrackedFile {
  path: string;
  content: string;
  /** 内容是否因超长被截断 */
  truncated: boolean;
}

/** 生成提交信息所需的变更上下文(diff 可能已被截断) */
export interface GitCommitContext {
  /** `git diff --stat` 摘要 */
  stat: string;
  /** 相对 HEAD 的完整 diff(超长时截断;已排除锁文件等噪声) */
  diff: string;
  /** diff 是否因超长被截断 */
  truncated: boolean;
  /** 全部未跟踪文件名(含无内容的,供模型感知新增文件) */
  untracked: string[];
  /** 未跟踪文件中可读取的文本内容(跳过二进制与超限文件) */
  untracked_files: GitUntrackedFile[];
  /** 最近提交信息 subject(风格锚定用,新仓库为空) */
  recent_commits: string[];
}

/** 一条 git 提交记录(日报生成用) */
export interface GitCommitInfo {
  hash: string;
  author: string;
  /** 本地时间 "YYYY-MM-DD HH:MM" */
  date: string;
  subject: string;
}

/** 仓库当前 git 用户身份(user.name / user.email) */
export interface GitUser {
  name: string;
  email: string;
}

/** 用户自定义 AI 提示词(~/.pm/prompts/*.md);空字符串表示使用内置默认模板 */
export interface AiPrompts {
  /** 提交信息生成提示词 */
  commit: string;
  /** 日报生成提示词 */
  report: string;
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

/** 日报历史列表项 */
export interface ReportHistoryItem {
  id: number;
  projectIds: number[];
  dateFrom: string;
  dateTo: string;
  rangeLabel: string;
  authorMode: string;
  language: string;
  createdAt: number;
  projectNames: string[];
  totalCommits: number;
}

/** 日报历史详情(含 Markdown 正文与各项目提交记录) */
export interface ReportHistoryDetail {
  id: number;
  projectIds: number[];
  dateFrom: string;
  dateTo: string;
  rangeLabel: string;
  authorMode: string;
  language: string;
  createdAt: number;
  projectNames: string[];
  totalCommits: number;
  result: string;
  commits: ReportCommitItem[];
}

/** 日报历史中单个项目的提交记录 */
export interface ReportCommitItem {
  projectId: number | null;
  projectName: string;
  projectDescription: string;
  commits: GitCommitInfo[];
}

/** 保存日报时传入的提交数据 */
export interface SaveReportCommit {
  projectId: number | null;
  projectName: string;
  projectDescription?: string;
  commits: GitCommitInfo[];
}

/** 定时任务配置 */
export interface ReportSchedule {
  id: string;
  name: string;
  enabled: boolean;
  projectIds: number[];
  authorMode: "me" | "all";
  timeOfDay: string;
  weekdaysOnly: boolean;
  chineseWorkdayOnly: boolean;
  lastRunAt: number | null;
}

/** 定时任务触发后发送给前端的通知 */
export interface ReportGeneratedPayload {
  scheduleName: string;
  historyId: number;
  dateFrom: string;
  dateTo: string;
}

/** 日历标注数据：某月每天报告数 + 节假日/调休 */
export interface CalendarMeta {
  dates: Record<string, number>;
  holidays: string[];
  workdays: string[];
}
