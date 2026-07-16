import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { cmd } from "@/lib/tauri";
import type { GitStatus, GitUpdatedPayload, Project } from "@/types";

export const useProjectsStore = defineStore("projects", () => {
  const projects = ref<Project[]>([]);
  const loading = ref(false);

  async function fetchProjects(query?: string, tagIds?: number[]) {
    loading.value = true;
    try {
      projects.value = await cmd<Project[]>("list_projects", {
        query: query?.trim() ? query.trim() : null,
        tagIds: tagIds?.length ? tagIds : null,
      });
      // Git 状态后台补齐,不阻塞列表渲染
      refreshAllGitStatus().then(triggerAllRemoteFetches);
    } finally {
      loading.value = false;
    }
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
    fetchProjects,
    addProject,
    updateProject,
    deleteProject,
    refreshGitStatus,
    triggerRemoteFetch,
    updateGitRemote,
    byId,
  };
});
