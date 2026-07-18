import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { cmd } from "@/lib/tauri";
import type { GitStatus, GitUpdatedPayload, Project } from "@/types";

export const useProjectsStore = defineStore("projects", () => {
  const projects = ref<Project[]>([]);
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
    try {
      project.git = await cmd<GitStatus>("get_git_status", { path: project.path });
    } catch {
      project.git = null;
    }
  }

  function refreshAllGitStatus() {
    return Promise.all(projects.value.map((p) => refreshGitStatus(p)));
  }

  /** 触发单个项目的后台远端 fetch(后端限流,结果走 git://updated 事件) */
  function triggerRemoteFetch(project: Project) {
    if (project.git?.is_repo) {
      cmd("fetch_git_remote_async", { projectId: project.id, path: project.path }).catch(
        () => {},
      );
    }
  }

  function triggerAllRemoteFetches() {
    projects.value.forEach(triggerRemoteFetch);
  }

  async function addProject(path: string, name: string) {
    const project = await cmd<Project>("add_project", { path, name });
    await fetchProjects();
    return project;
  }

  async function updateProject(id: number, name: string, description: string) {
    const project = await cmd<Project>("update_project", { id, name, description });
    const idx = projects.value.findIndex((p) => p.id === id);
    if (idx >= 0) projects.value[idx] = project;
    return project;
  }

  async function deleteProject(id: number) {
    await cmd("delete_project", { id });
    projects.value = projects.value.filter((p) => p.id !== id);
  }

  /** 后台 fetch 完成后由 git://updated 事件调用 */
  function updateGitRemote(projectId: number, payload: GitUpdatedPayload) {
    const p = projects.value.find((x) => x.id === projectId);
    if (p?.git) {
      p.git.remote_ahead = payload.remote_ahead;
      p.git.last_fetch_at = payload.last_fetch_at;
    }
  }

  const byId = computed(() => {
    return (id: number) => projects.value.find((p) => p.id === id);
  });

  return {
    projects,
    loading,
    query,
    selectedTagIds,
    fetchProjects,
    setQuery,
    toggleTagFilter,
    clearTagFilters,
    refreshProject,
    addProject,
    updateProject,
    deleteProject,
    refreshGitStatus,
    triggerRemoteFetch,
    updateGitRemote,
    byId,
  };
});
