<script setup lang="ts">
// 托盘迷你项目列表窗口(类似 JetBrains Toolbox):单击托盘图标弹出,
// 头部搜索 + 精简项目行,点击行跳主窗口详情,行尾可展开「打开方式」。
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Search } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import OpenWithMenu from "@/components/open/OpenWithMenu.vue";
import { cmd } from "@/lib/tauri";
import { useProjectsStore } from "@/stores/projects";
import type { Project } from "@/types";

const { t } = useI18n();
const store = useProjectsStore();
const searchInput = ref("");

// 客户端过滤:弹窗有独立 Pinia 实例,与主窗口的查询状态互不影响
const filtered = computed(() => {
  const q = searchInput.value.trim().toLowerCase();
  if (!q) return store.projects;
  return store.projects.filter(
    (p) =>
      p.name.toLowerCase().includes(q) ||
      p.path.toLowerCase().includes(q) ||
      p.description.toLowerCase().includes(q),
  );
});

/** 点击项目行:显示主窗口并跳转到该项目详情页(弹窗随后因失焦自动收起) */
async function openProject(project: Project) {
  try {
    await cmd("show_main_window", { projectId: project.id });
  } catch {
    // 主窗口未就绪等情况静默失败即可
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    cmd("hide_tray_popup").catch(() => {});
  }
}

onMounted(() => {
  // 项目列表由 App.vue 统一拉取(弹窗内 withGit: false,不拉 git 状态)
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden border bg-background shadow-2xl">
    <header class="shrink-0 border-b px-3 py-2">
      <div class="relative">
        <Search class="absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          v-model="searchInput"
          :placeholder="t('trayPopup.searchPlaceholder')"
          class="h-8 pl-8 text-sm"
          autofocus
        />
      </div>
    </header>
    <ScrollArea class="min-h-0 flex-1">
      <div class="flex flex-col gap-0.5 p-2">
        <button
          v-for="project in filtered"
          :key="project.id"
          type="button"
          class="group flex items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors hover:bg-accent"
          @click="openProject(project)"
        >
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-1.5">
              <span class="truncate text-sm font-medium">{{ project.name }}</span>
              <Badge
                v-if="!project.path_exists"
                variant="destructive"
                class="shrink-0 px-1.5 py-0 text-[11px]"
                :title="t('projects.status.pathMissingHint')"
              >
                {{ t("projects.status.pathMissing") }}
              </Badge>
            </div>
            <p class="truncate text-xs text-muted-foreground" :title="project.path">
              {{ project.path }}
            </p>
          </div>
          <div class="shrink-0 opacity-0 transition-opacity group-hover:opacity-100">
            <OpenWithMenu :project="project" compact />
          </div>
        </button>
        <p v-if="!filtered.length" class="py-10 text-center text-sm text-muted-foreground">
          {{ t("trayPopup.empty") }}
        </p>
      </div>
    </ScrollArea>
  </div>
</template>
