<script setup lang="ts">
import { provide, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { BookOpen, X } from "@lucide/vue";
import { Markdown, type ControlsConfig, type NodeRenderers } from "vue-stream-markdown";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import MdImage from "@/components/markdown/MdImage.vue";
import MdLink from "@/components/markdown/MdLink.vue";
import { MD_BASE_PATH_KEY } from "@/components/markdown/keys";
import { cmd } from "@/lib/tauri";
import { hasScheme, resolvePath } from "@/lib/markdown";
import { useSettingsStore } from "@/stores/settings";
import type { Project, ReadmeContent } from "@/types";

const { t } = useI18n();
const settingsStore = useSettingsStore();

const props = defineProps<{ project: Project }>();
const open = defineModel<boolean>("open", { default: false });

const readme = ref<ReadmeContent | null>(null);
const content = ref("");
const loading = ref(false);

// 相对路径图片/文件的解析基准(供自定义渲染器使用)
provide(MD_BASE_PATH_KEY, () => props.project.path);

// 自定义渲染器:图片走本地 asset 协议,链接输出真实 href 由外层统一拦截
const nodeRenderers: NodeRenderers = {
  image: MdImage,
  link: MdLink,
};

// 表格复制/导出(CSV/TSV/Markdown)/全屏 + 代码复制/折叠,库默认全开,这里显式声明
const controls: ControlsConfig = {
  table: { copy: true, download: true, fullscreen: true },
  code: { copy: true, collapse: true },
};

// 阻止库把宿主元素上的 shadcn 变量内联到组件根节点(island 皮肤的 hex 色值
// 会被库误包成 hsl(#xxx) 非法值),MD 主题完全交给 CSS 层(src/styles/markdown/)
const detachedThemeEl = document.createElement("div");
const themeElement = () => detachedThemeEl;

async function load() {
  loading.value = true;
  try {
    readme.value = await cmd<ReadmeContent | null>("read_readme", {
      path: props.project.path,
    });
    content.value = readme.value?.content ?? "";
  } catch {
    readme.value = null;
    content.value = "";
  } finally {
    loading.value = false;
  }
}

// 打开抽屉或切换项目时(重新)加载
watch(
  [open, () => props.project.id],
  ([isOpen]) => {
    if (isOpen) load();
  },
  { immediate: true },
);

// Esc 关闭
watch(open, (isOpen) => {
  if (isOpen) window.addEventListener("keydown", onEsc);
  else window.removeEventListener("keydown", onEsc);
});

function onEsc(e: KeyboardEvent) {
  if (e.key === "Escape") open.value = false;
}

/** 拦截链接点击:外链交给系统浏览器,相对路径用系统默认程序打开 */
async function onBodyClick(e: MouseEvent) {
  const a = (e.target as HTMLElement).closest("a");
  if (!a) return;
  const href = a.getAttribute("href");
  e.preventDefault();
  if (!href || href.startsWith("#")) return;
  try {
    if (hasScheme(href)) {
      await openUrl(href);
    } else {
      await openPath(resolvePath(props.project.path, href));
    }
  } catch {
    // 目标不存在等情况静默忽略
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="open" class="fixed inset-0 z-40 bg-black/50" @click="open = false" />
    </Transition>

    <Transition name="slide">
      <aside
        v-if="open"
        class="readme-surface fixed inset-y-0 right-0 z-50 flex w-full max-w-2xl flex-col border-l shadow-xl"
      >
        <header class="flex shrink-0 items-center justify-between gap-2 border-b px-4 py-3">
          <div class="flex min-w-0 items-center gap-2 text-sm font-semibold">
            <BookOpen class="h-4 w-4 shrink-0" />
            {{ t("readme.title") }}
            <span v-if="readme" class="truncate text-xs font-normal text-muted-foreground">
              {{ readme.file_name }}
            </span>
          </div>
          <div class="flex shrink-0 items-center gap-1">
            <Button
              size="icon"
              variant="ghost"
              class="h-8 w-8"
              :title="t('readme.closeEsc')"
              @click="open = false"
            >
              <X class="h-4 w-4" />
            </Button>
          </div>
        </header>

        <ScrollArea class="min-h-0 flex-1">
          <p v-if="loading" class="p-6 text-sm text-muted-foreground">{{ t("readme.loading") }}</p>
          <p v-else-if="!readme" class="p-6 text-sm text-muted-foreground">
            {{ t("readme.notFound") }}
          </p>
          <div v-else class="p-6 text-sm" @click="onBodyClick">
            <Markdown
              mode="static"
              :content="content"
              :controls="controls"
              :node-renderers="nodeRenderers"
              :theme-element="themeElement"
              :locale="settingsStore.language"
            />
          </div>
        </ScrollArea>
      </aside>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.25s ease;
}
.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}
</style>
