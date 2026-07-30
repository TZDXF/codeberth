<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { toast } from "vue-sonner";
import { Toaster } from "@/components/ui/sonner";
import TitleBar from "@/components/TitleBar.vue";
import BatchProgressFloat from "@/components/report/BatchProgressFloat.vue";
import { onListen } from "@/lib/tauri";
import { usePinsStore } from "@/stores/pins";
import { useProjectsStore } from "@/stores/projects";
import { useSettingsStore } from "@/stores/settings";
import { useTagsStore } from "@/stores/tags";
import { useUpdateStore } from "@/stores/update";
import type { GitUpdatedPayload, ReportGeneratedPayload } from "@/types";

const router = useRouter();
const store = useProjectsStore();
const pinsStore = usePinsStore();
const tagsStore = useTagsStore();
const settingsStore = useSettingsStore();
const updateStore = useUpdateStore();

// 托盘迷你弹窗窗口:仅加载主题/语言与项目列表,跳过标题栏、更新检查等主窗口专属逻辑
const isTrayPopup = getCurrentWindow().label === "tray-popup";

onMounted(() => {
  settingsStore.init().then(() => {
    if (isTrayPopup) return;
    updateStore.init();
    // 启动后静默检查更新(dev 环境跳过,避免无签名产物时无意义报错)
    if (settingsStore.autoCheckUpdate && !import.meta.env.DEV) {
      updateStore.checkForUpdate(false);
    }
  });
  store.fetchProjects({ withGit: !isTrayPopup });
  pinsStore.fetchPins();
  // 另一窗口(托盘弹窗/主窗口)切换收藏后同步刷新;保留已有 git 状态避免闪烁
  onListen("projects://favorite-changed", () => {
    store.fetchProjects({ withGit: false });
  });
  // 另一窗口变更「常用命令」标记后同步刷新
  onListen("projects://pins-changed", () => {
    pinsStore.fetchPins();
  });
  if (isTrayPopup) return;
  store.startGitAutoRefresh();
  tagsStore.fetchTags();
  onListen<GitUpdatedPayload>("git://updated", (payload) => {
    store.updateGitRemote(payload.project_id, payload);
  });
  onListen<ReportGeneratedPayload>("report://generated", (payload) => {
    toast.success(`定时任务「${payload.scheduleName}」已自动生成日报 (${payload.dateFrom})`, {
      action: {
        label: "查看",
        onClick: () => {
          // navigate to history page — use window.location since router not available here
          window.location.hash = "#/report-history";
        },
      },
    });
  });
  // 托盘弹窗/菜单请求跳转到项目详情页
  onListen<{ projectId: number }>("main://navigate", (payload) => {
    router.push(`/projects/${payload.projectId}`);
  });
});

onUnmounted(() => {
  if (!isTrayPopup) {
    store.stopGitAutoRefresh();
  }
});
</script>

<template>
  <main class="flex h-screen flex-col overflow-hidden bg-background text-foreground">
    <TitleBar v-if="!isTrayPopup" />
    <div class="min-h-0 flex-1">
      <router-view />
    </div>
  </main>
  <Toaster position="bottom-right" />
  <BatchProgressFloat v-if="!isTrayPopup" />
</template>
