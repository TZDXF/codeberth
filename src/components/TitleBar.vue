<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { ArrowUpCircle, Copy, Minus, RefreshCw, Square, X } from "@lucide/vue";
import UpdateDialog from "@/components/update/UpdateDialog.vue";
import { useUpdateStore } from "@/stores/update";

const { t } = useI18n();
const appWindow = getCurrentWindow();
const isMaximized = ref(false);
const updateStore = useUpdateStore();
let unlistenResize: UnlistenFn | undefined;

onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized();
  unlistenResize = await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized();
  });
});

onBeforeUnmount(() => {
  unlistenResize?.();
});

function onUpdateClick() {
  // 按钮仅在检测到新版本后出现,点击打开更新详情对话框
  updateStore.dialogOpen = true;
}

function onDragRegionDblClick(event: MouseEvent) {
  // 仅在空白拖拽区域响应双击最大化，避免在按钮上双击时误触发
  if ((event.target as HTMLElement).hasAttribute("data-tauri-drag-region")) {
    appWindow.toggleMaximize();
  }
}
</script>

<template>
  <!--
    弹窗(Dialog)打开时,reka-ui 会渲染 z-50 的全屏 overlay 并把 body 设为 pointer-events: none,
    导致标题栏无法拖动。这里将标题栏提到 overlay 之上并恢复指针事件;
    @pointerdown.stop 阻止冒泡到 document,避免触发 reka-ui 的"点击外部关闭弹窗"
    (Tauri 窗口拖动监听的是 mousedown,不受影响)。
  -->
  <div
    data-tauri-drag-region
    class="pointer-events-auto relative z-[60] flex h-9 shrink-0 select-none items-center border-b bg-background pl-3"
    @dblclick="onDragRegionDblClick"
    @pointerdown.stop
  >
    <div
      data-tauri-drag-region
      class="flex flex-1 items-center gap-2 text-xs font-medium text-muted-foreground"
    >
      <span class="pointer-events-none">{{ t("app.title") }} · {{ t("app.name") }}</span>
    </div>
    <div class="flex h-full items-stretch">
      <button
        v-if="updateStore.update"
        class="relative flex w-11 items-center justify-center text-primary transition-colors hover:bg-accent"
        :title="t('titleBar.updateAvailable', { version: updateStore.update.version })"
        @click="onUpdateClick"
      >
        <RefreshCw v-if="updateStore.status === 'downloading'" class="h-4 w-4 animate-spin" />
        <ArrowUpCircle v-else class="h-4 w-4" />
        <span
          v-if="updateStore.hasUpdate"
          class="absolute right-2.5 top-2 h-1.5 w-1.5 rounded-full bg-destructive"
        />
      </button>
      <button
        class="flex w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        :title="t('titleBar.minimize')"
        @click="appWindow.minimize()"
      >
        <Minus class="h-4 w-4" />
      </button>
      <button
        class="flex w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        :title="isMaximized ? t('titleBar.restore') : t('titleBar.maximize')"
        @click="appWindow.toggleMaximize()"
      >
        <Copy v-if="isMaximized" class="h-3.5 w-3.5" />
        <Square v-else class="h-3.5 w-3.5" />
      </button>
      <button
        class="flex w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-destructive hover:text-white"
        :title="t('titleBar.close')"
        @click="appWindow.close()"
      >
        <X class="h-4 w-4" />
      </button>
    </div>
    <UpdateDialog />
  </div>
</template>
