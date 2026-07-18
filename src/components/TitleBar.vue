<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { Copy, Minus, Square, X } from "@lucide/vue";

const appWindow = getCurrentWindow();
const isMaximized = ref(false);
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

function onDragRegionDblClick(event: MouseEvent) {
  // 仅在空白拖拽区域响应双击最大化，避免在按钮上双击时误触发
  if ((event.target as HTMLElement).hasAttribute("data-tauri-drag-region")) {
    appWindow.toggleMaximize();
  }
}
</script>

<template>
  <div
    data-tauri-drag-region
    class="flex h-9 shrink-0 select-none items-center border-b bg-background pl-3"
    @dblclick="onDragRegionDblClick"
  >
    <div
      data-tauri-drag-region
      class="flex flex-1 items-center gap-2 text-xs font-medium text-muted-foreground"
    >
      <span class="pointer-events-none">ProjectDev</span>
    </div>
    <div class="flex h-full items-stretch">
      <button
        class="flex w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        title="最小化"
        @click="appWindow.minimize()"
      >
        <Minus class="h-4 w-4" />
      </button>
      <button
        class="flex w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        :title="isMaximized ? '还原' : '最大化'"
        @click="appWindow.toggleMaximize()"
      >
        <Copy v-if="isMaximized" class="h-3.5 w-3.5" />
        <Square v-else class="h-3.5 w-3.5" />
      </button>
      <button
        class="flex w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-destructive hover:text-white"
        title="关闭"
        @click="appWindow.close()"
      >
        <X class="h-4 w-4" />
      </button>
    </div>
  </div>
</template>
