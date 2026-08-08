import { ref } from "vue";
import { defineStore } from "pinia";
import { cmd } from "@/lib/tauri";
import type { Project, ProjectAssets } from "@/types";

/**
 * 项目资产(package scripts + compose 文件)扫描结果共享 store。
 * PackageScripts 与 DockerCompose 卡片挂载时都要这份数据,这里合并为一次 IPC
 * (后端 scan_project_assets 单次目录遍历 + 30s walk 缓存),并进行中去重。
 * 不做前端长缓存:每次进入详情页都会重新拉取,后端 walk 缓存保证重复拉取足够便宜。
 */
export const useProjectAssetsStore = defineStore("project-assets", () => {
  /** 按项目 id 存放最近一次扫描结果 */
  const byProject = ref<Record<number, ProjectAssets>>({});
  /** 进行中的请求,key 为项目 id:两个卡片同时挂载只发一次 IPC */
  const inflight = new Map<number, Promise<void>>();

  function assetsOf(id: number): ProjectAssets | undefined {
    return byProject.value[id];
  }

  /** 拉取(或复用进行中的)扫描结果;失败写入空结果,不向上抛错(卡片按无数据显示) */
  function refresh(project: Project): Promise<void> {
    const pending = inflight.get(project.id);
    if (pending) return pending;
    const p = (async () => {
      try {
        byProject.value[project.id] = await cmd<ProjectAssets>("scan_project_assets", {
          path: project.path,
        });
      } catch {
        byProject.value[project.id] = { package_scripts: [], compose_files: [] };
      }
    })().finally(() => {
      inflight.delete(project.id);
    });
    inflight.set(project.id, p);
    return p;
  }

  return { byProject, assetsOf, refresh };
});
