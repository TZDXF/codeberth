import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { cmd } from "@/lib/tauri";
import type { GitBranches, GitPullResult, GitStatus, GitUpdatedPayload, Project } from "@/types";

export const useProjectsStore = defineStore("projects", () => {
  const projects = ref<Project[]>([]);
  const archivedProjects = ref<Project[]>([]);
  const loading = ref(false);
  const query = ref("");
  const selectedTagIds = ref<number[]>([]);

  /**
   * 拉取项目列表。
   * withGit 为 false 时(搜索/筛选等高频操作)不重新拉取 git 状态,
   * 仅按项目 id 保留已有的 git 信息,避免频繁触发 git 更新。
   */
  async function fetchProjects(options: { withGit?: boolean } = {}) {
    const withGit = options.withGit ?? true;
    loading.value = true;
    try {
      const list = await cmd<Project[]>("list_projects", {
        query: query.value.trim() ? query.value.trim() : null,
        tagIds: selectedTagIds.value.length ? selectedTagIds.value : null,
      });
      if (withGit) {
        projects.value = list;
        // Git 状态后台补齐,不阻塞列表渲染
        refreshAllGitStatus().then(triggerAllRemoteFetches);
      } else {
        const prevGit = new Map(projects.value.map((p) => [p.id, p.git]));
        list.forEach((p) => {
          p.git = prevGit.get(p.id) ?? p.git;
        });
        projects.value = list;
      }
    } finally {
      loading.value = false;
    }
  }

  function setQuery(value: string) {
    query.value = value;
    fetchProjects({ withGit: false });
  }

  function toggleTagFilter(tagId: number) {
    selectedTagIds.value = selectedTagIds.value.includes(tagId)
      ? selectedTagIds.value.filter((id) => id !== tagId)
      : [...selectedTagIds.value, tagId];
    fetchProjects({ withGit: false });
  }

  function clearTagFilters() {
    if (!selectedTagIds.value.length) return;
    selectedTagIds.value = [];
    fetchProjects({ withGit: false });
  }

  /** 重新拉取单个项目(保留已有的 git 状态,后端不返回) */
  async function refreshProject(id: number) {
    const fresh = await cmd<Project>("get_project", { id });
    const idx = projects.value.findIndex((p) => p.id === id);
    if (idx >= 0) {
      fresh.git = projects.value[idx].git;
      projects.value[idx] = fresh;
    }
    return fresh;
  }

  async function refreshGitStatus(project: Project) {
    const run = () => cmd<GitStatus>("get_git_status", { path: project.path });
    try {
      project.git = await run();
    } catch {
      // 失败重试一次;仍失败时保留旧值(原本就没有则保持 null),避免卡片 git 信息闪烁丢失
      try {
        project.git = await run();
      } catch {
        /* 保留旧值,等待下一轮定时刷新 */
      }
    }
  }

  function refreshAllGitStatus() {
    return Promise.all(projects.value.map((p) => refreshGitStatus(p)));
  }

  /** 触发单个项目的后台远端 fetch(后端限流,结果走 git://updated 事件) */
  function triggerRemoteFetch(project: Project) {
    if (project.git?.is_repo) {
      cmd("fetch_git_remote_async", { projectId: project.id, path: project.path }).catch(() => {});
    }
  }

  function triggerAllRemoteFetches() {
    projects.value.forEach(triggerRemoteFetch);
  }

  /** git 状态定时刷新间隔(本地 status + 后台远端 fetch) */
  const GIT_REFRESH_INTERVAL_MS = 30_000;
  let gitRefreshTimer: ReturnType<typeof setInterval> | null = null;
  let gitRefreshing = false;

  /** 启动 git 状态定时刷新(App 挂载时调用,重复调用会重置计时) */
  function startGitAutoRefresh() {
    stopGitAutoRefresh();
    gitRefreshTimer = setInterval(async () => {
      // 上一轮未结束时跳过,避免慢仓库上刷新堆叠
      if (gitRefreshing || !projects.value.length) return;
      gitRefreshing = true;
      try {
        await refreshAllGitStatus();
        triggerAllRemoteFetches();
      } finally {
        gitRefreshing = false;
      }
    }, GIT_REFRESH_INTERVAL_MS);
  }

  /** 停止 git 状态定时刷新(App 卸载时调用) */
  function stopGitAutoRefresh() {
    if (gitRefreshTimer) {
      clearInterval(gitRefreshTimer);
      gitRefreshTimer = null;
    }
  }

  async function addProject(path: string, name: string, description?: string) {
    const project = await cmd<Project>("add_project", {
      path,
      name,
      description: description?.trim() ? description.trim() : null,
    });
    await fetchProjects();
    return project;
  }

  /** 克隆仓库到本地,返回克隆后的路径;可被 cancelClone 中断。
   *  accountId 传入时后端用绑定账号 token 克隆(成功后重置 origin 为干净 URL) */
  async function cloneProject(
    url: string,
    targetPath: string,
    jobId: string,
    accountId?: number,
  ): Promise<string> {
    return cmd<string>("git_clone", { url, targetPath, jobId, accountId: accountId ?? null });
  }

  /** 取消进行中的克隆(后端 kill 子进程并清理半成品目录) */
  async function cancelClone(jobId: string) {
    await cmd("cancel_git_clone", { jobId });
  }

  async function updateProject(id: number, name: string, description: string) {
    const project = await cmd<Project>("update_project", { id, name, description });
    const idx = projects.value.findIndex((p) => p.id === id);
    if (idx >= 0) projects.value[idx] = project;
    return project;
  }

  /** 重新指定项目目录(目录被移动后修复登记路径),成功后重新探测 git 状态 */
  async function updateProjectPath(id: number, path: string) {
    const project = await cmd<Project>("update_project_path", { id, path });
    const idx = projects.value.findIndex((p) => p.id === id);
    if (idx >= 0) projects.value[idx] = project;
    if (project.path_exists) {
      await refreshGitStatus(project);
      triggerRemoteFetch(project);
    }
    return project;
  }

  /** 应用内移动项目目录到新的父目录下(可改名),成功后重新探测 git 状态 */
  async function moveProjectDir(id: number, targetParent: string, dirName: string) {
    const project = await cmd<Project>("move_project_dir", { id, targetParent, dirName });
    const idx = projects.value.findIndex((p) => p.id === id);
    if (idx >= 0) projects.value[idx] = project;
    await refreshGitStatus(project);
    triggerRemoteFetch(project);
    return project;
  }

  /** 归档项目:软删除,历史数据保留;归档后不再展示、不再获取 git 状态 */
  async function archiveProject(id: number) {
    await cmd("archive_project", { id });
    projects.value = projects.value.filter((p) => p.id !== id);
  }

  /** 设置/取消收藏:成功后就地更新 favorited_at,列表排序由 computed 响应 */
  async function setFavorite(id: number, favorite: boolean) {
    await cmd("set_project_favorite", { id, favorite });
    const p = projects.value.find((x) => x.id === id);
    if (p) {
      p.favorited_at = favorite ? Math.floor(Date.now() / 1000) : null;
    }
  }

  /** 拉取已归档项目列表(设置页归档管理用) */
  async function fetchArchivedProjects() {
    archivedProjects.value = await cmd<Project[]>("list_archived_projects");
  }

  /** 取消归档:恢复到项目列表 */
  async function unarchiveProject(id: number) {
    await cmd("unarchive_project", { id });
    archivedProjects.value = archivedProjects.value.filter((p) => p.id !== id);
    await fetchProjects();
  }

  /** 彻底删除项目(不可恢复,历史数据一并删除;不会删除磁盘文件) */
  async function deleteProject(id: number) {
    await cmd("delete_project", { id });
    archivedProjects.value = archivedProjects.value.filter((p) => p.id !== id);
  }

  /** 后台 fetch 完成后由 git://updated 事件调用 */
  function updateGitRemote(projectId: number, payload: GitUpdatedPayload) {
    const p = projects.value.find((x) => x.id === projectId);
    if (p?.git) {
      p.git.remote_ahead = payload.remote_ahead;
      p.git.last_fetch_at = payload.last_fetch_at;
    }
  }

  // --- Git 写操作:错误向上抛出由 UI toast,成功后用返回的最新状态就地更新 ---

  function listBranches(project: Project) {
    return cmd<GitBranches>("list_git_branches", { path: project.path });
  }

  /**
   * 切换分支。create: 创建并切换;remote: branch 形如 "origin/feature",
   * 本地无同名分支时自动创建跟踪分支
   */
  async function checkoutBranch(
    project: Project,
    branch: string,
    options: { create?: boolean; remote?: boolean } = {},
  ) {
    project.git = await cmd<GitStatus>("git_checkout", {
      path: project.path,
      branch,
      create: options.create ?? false,
      remote: options.remote ?? false,
    });
  }

  /** 提交更改(未暂存修改始终纳入;includeUntracked 控制是否包含未跟踪文件) */
  async function commitChanges(project: Project, message: string, includeUntracked: boolean) {
    project.git = await cmd<GitStatus>("git_commit", {
      path: project.path,
      message,
      includeUntracked,
    });
  }

  /** 拉取远端;返回冲突文件列表(非空表示产生了合并冲突) */
  async function pullRepository(project: Project) {
    const result = await cmd<GitPullResult>("git_pull", { path: project.path });
    project.git = result.status;
    return result.conflicts;
  }

  /** 推送当前分支(无 upstream 时后端自动 -u origin HEAD) */
  async function pushRepository(project: Project) {
    project.git = await cmd<GitStatus>("git_push", { path: project.path });
  }

  const byId = computed(() => {
    return (id: number) => projects.value.find((p) => p.id === id);
  });

  return {
    projects,
    archivedProjects,
    loading,
    query,
    selectedTagIds,
    fetchProjects,
    setQuery,
    toggleTagFilter,
    clearTagFilters,
    refreshProject,
    addProject,
    cloneProject,
    cancelClone,
    updateProject,
    updateProjectPath,
    moveProjectDir,
    archiveProject,
    setFavorite,
    fetchArchivedProjects,
    unarchiveProject,
    deleteProject,
    refreshGitStatus,
    triggerRemoteFetch,
    startGitAutoRefresh,
    stopGitAutoRefresh,
    updateGitRemote,
    listBranches,
    checkoutBranch,
    commitChanges,
    pullRepository,
    pushRepository,
    byId,
  };
});
