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
import FavoriteToggle from "@/components/project/FavoriteToggle.vue";
import TrayPinnedCommands from "@/components/project/TrayPinnedCommands.vue";
import { compareFavorited } from "@/lib/favorites";
import { cmd, onListen } from "@/lib/tauri";
import { usePinsStore } from "@/stores/pins";
import { useProjectsStore } from "@/stores/projects";
import type { Project } from "@/types";

const { t } = useI18n();
const store = useProjectsStore();
const pinsStore = usePinsStore();
const searchInput = ref("");

// 客户端过滤 + 按最近更新倒序;弹窗有独立 Pinia 实例,与主窗口的查询状态互不影响
const filtered = computed(() => {
  const q = searchInput.value.trim().toLowerCase();
  const list = q
    ? store.projects.filter(
        (p) =>
          p.name.toLowerCase().includes(q) ||
          p.path.toLowerCase().includes(q) ||
          p.description.toLowerCase().includes(q) ||
          p.tags.some((tag) => tag.name.toLowerCase().includes(q)),
      )
    : store.projects;
  // 收藏项目置顶(组内按收藏时间倒序),其余按最近更新倒序
  return [...list].sort((a, b) => compareFavorited(a, b) || b.updated_at - a.updated_at);
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
  // 每次弹窗显示时后端会发刷新事件,重新拉取以同步主窗口的数据变更
  onListen("tray-popup://refresh", () => {
    store.fetchProjects({ withGit: false });
    pinsStore.fetchPins();
  });
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <div
    data-slot="tray-popup"
    class="flex h-full flex-col overflow-hidden border bg-background shadow-2xl"
  >
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
        <div v-for="project in filtered" :key="project.id">
          <button
            type="button"
            class="group flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors hover:bg-accent"
            @click="openProject(project)"
          >
            <div class="min-w-0 flex-1">
              <div class="flex min-w-0 items-center gap-1.5">
                <span class="truncate text-sm font-medium">{{ project.name }}</span>
                <Badge
                  v-if="!project.path_exists"
                  variant="destructive"
                  class="shrink-0 px-1.5 py-0 text-[11px]"
                  :title="t('projects.status.pathMissingHint')"
                >
                  {{ t("projects.status.pathMissing") }}
                </Badge>
                <div
                  v-if="project.tags.length"
                  class="flex min-w-0 items-center gap-1 overflow-hidden"
                >
                  <Badge
                    v-for="tag in project.tags"
                    :key="tag.id"
                    variant="secondary"
                    class="shrink-0 px-1.5 py-0 text-[11px]"
                    :style="{ backgroundColor: tag.color + '22', color: tag.color }"
                  >
                    {{ tag.name }}
                  </Badge>
                </div>
              </div>
              <p
                v-if="project.description"
                class="mt-0.5 truncate text-xs text-muted-foreground"
                :title="project.description"
              >
                {{ project.description }}
              </p>
            </div>
            <div class="flex shrink-0 items-center">
              <FavoriteToggle :project="project" />
              <div class="opacity-0 transition-opacity group-hover:opacity-100">
                <OpenWithMenu :project="project" compact />
              </div>
            </div>
          </button>
          <!-- 被标记为「常用」的命令行内展开,点击直接执行 -->
          <TrayPinnedCommands :project="project" :pins="pinsStore.pinsOf(project.id)" />
        </div>
        <p v-if="!filtered.length" class="py-10 text-center text-sm text-muted-foreground">
          {{ t("trayPopup.empty") }}
        </p>
      </div>
    </ScrollArea>
  </div>
</template>
