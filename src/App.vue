<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
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
import type { GitStatusItem, GitUpdatedPayload, ReportGeneratedPayload } from "@/types";

const router = useRouter();
const store = useProjectsStore();
const pinsStore = usePinsStore();
const tagsStore = useTagsStore();
const settingsStore = useSettingsStore();
const updateStore = useUpdateStore();

// 托盘迷你弹窗窗口:仅加载主题/语言与项目列表,跳过标题栏、更新检查等主窗口专属逻辑
const isTrayPopup = getCurrentWindow().label === "tray-popup";

let unlistenFocus: UnlistenFn | undefined;

onMounted(async () => {
  // 托盘弹窗窗口已开 OS 级透明,body 不再刷底色,让玻璃皮肤能透出桌面
  if (isTrayPopup) {
    document.body.classList.add("tray-popup-window");
  }
  await settingsStore.init();
  if (!isTrayPopup) {
    updateStore.init();
    // 启动后静默检查更新(dev 环境跳过,避免无签名产物时无意义报错)
    if (settingsStore.autoCheckUpdate && !import.meta.env.DEV) {
      updateStore.checkForUpdate(false);
    }
  }
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
  // 另一窗口变更主题/皮肤/MD 主题后同步当前窗口的 DOM(主要是托盘弹窗跟随主窗口)
  // 托盘窗口仅在 init 时从 settings.json 读一次主题,期间切换需要靠广播同步。
  // 广播带全量最新值,接收方先覆盖自己的 ref(各 webview 的 Pinia store 互相独立)再 apply,
  // 否则 applyTheme 用的是本地 ref,等于用旧值覆盖,主题不会变
  onListen<{ theme: string; themeSkin: string; mdTheme: string }>(
    "settings://theme-changed",
    (payload) => {
      settingsStore.syncThemeFromExternal(payload);
    },
  );
  if (isTrayPopup) return;
  // git 状态由 Rust 后台循环每 30s 批量推送;窗口重新聚焦时兜底拉一次,
  // 覆盖循环异常退出/事件通道重建后错过推送的场景(命中缓存,开销可忽略)
  unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload }) => {
    if (payload) store.refreshAllGitStatus();
  });
  onListen<GitStatusItem[]>("git://status-updated", (items) => {
    store.applyGitStatusItems(items);
  });
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
  unlistenFocus?.();
});
</script>

<template>
  <!-- 托盘弹窗窗口:整体透明,背景交给 TrayPopup 根节点(玻璃皮肤下透出系统模糊) -->
  <main
    class="flex h-screen flex-col overflow-hidden text-foreground"
    :class="isTrayPopup ? 'bg-transparent' : 'bg-background'"
  >
    <TitleBar v-if="!isTrayPopup" />
    <div class="min-h-0 flex-1">
      <router-view />
    </div>
  </main>
  <Toaster position="bottom-right" />
  <BatchProgressFloat v-if="!isTrayPopup" />
</template>
