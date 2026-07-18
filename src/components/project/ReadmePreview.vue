<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { marked } from "marked";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { BookOpen, RefreshCw } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cmd } from "@/lib/tauri";
import type { Project, ReadmeContent } from "@/types";

marked.use({ gfm: true, breaks: true });

const props = defineProps<{ project: Project }>();

const readme = ref<ReadmeContent | null>(null);
const html = ref("");
const loaded = ref(false);
const bodyRef = ref<HTMLElement | null>(null);

async function load() {
  loaded.value = false;
  try {
    readme.value = await cmd<ReadmeContent | null>("read_readme", {
      path: props.project.path,
    });
    html.value = readme.value
      ? (marked.parse(readme.value.content, { async: false }) as string)
      : "";
  } catch {
    readme.value = null;
    html.value = "";
  } finally {
    loaded.value = true;
  }
}

watch(() => props.project.id, load, { immediate: true });

// 渲染完成后把相对路径图片换成本地 asset 协议地址
watch(html, async () => {
  await nextTick();
  fixRelativeImages();
});

/** 带协议头的 URL(http:, data:, asset: 等) */
function hasScheme(url: string): boolean {
  return /^[a-z][a-z0-9+.-]*:/i.test(url);
}

/** 把 README 里的相对路径解析成项目内的绝对路径 */
function resolvePath(base: string, rel: string): string {
  const clean = decodeURIComponent(rel).split("#")[0].split("?")[0];
  // 已是绝对路径(Windows 盘符 / UNC / POSIX 根)
  if (/^([a-zA-Z]:[\\/]|\\\\|\/)/.test(clean)) return clean;
  return `${base.replace(/[\\/]+$/, "")}/${clean}`;
}

function fixRelativeImages() {
  const el = bodyRef.value;
  if (!el) return;
  el.querySelectorAll("img").forEach((img) => {
    const src = img.getAttribute("src");
    if (!src || hasScheme(src)) return;
    img.src = convertFileSrc(resolvePath(props.project.path, src));
  });
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
  <Card v-if="!loaded || readme">
    <CardHeader class="flex-row items-center justify-between pb-3">
      <CardTitle class="flex items-center gap-2 text-sm font-semibold">
        <BookOpen class="h-4 w-4" />
        README
        <span v-if="readme" class="text-xs font-normal text-muted-foreground">
          {{ readme.file_name }}
        </span>
      </CardTitle>
      <Button size="sm" variant="ghost" title="重新加载" @click="load">
        <RefreshCw class="h-3.5 w-3.5" />
      </Button>
    </CardHeader>
    <CardContent>
      <p v-if="!loaded" class="text-sm text-muted-foreground">加载中...</p>
      <ScrollArea v-else class="max-h-[420px]">
        <!-- README 来自本地项目目录,内容由用户自己控制 -->
        <div
          ref="bodyRef"
          class="markdown-body pr-3 text-sm"
          @click="onBodyClick"
          v-html="html"
        />
      </ScrollArea>
    </CardContent>
  </Card>
</template>

<style scoped>
.markdown-body {
  line-height: 1.7;
  color: var(--foreground);
  word-break: break-word;
}
.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4) {
  margin: 1em 0 0.5em;
  font-weight: 600;
  line-height: 1.3;
}
.markdown-body :deep(h1) {
  font-size: 1.4em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--border);
}
.markdown-body :deep(h2) {
  font-size: 1.2em;
  padding-bottom: 0.25em;
  border-bottom: 1px solid var(--border);
}
.markdown-body :deep(h3) {
  font-size: 1.05em;
}
.markdown-body :deep(h4) {
  font-size: 0.95em;
}
.markdown-body :deep(h1:first-child),
.markdown-body :deep(h2:first-child),
.markdown-body :deep(h3:first-child) {
  margin-top: 0;
}
.markdown-body :deep(p) {
  margin: 0.5em 0;
}
.markdown-body :deep(a) {
  color: var(--primary);
  text-decoration: underline;
  text-underline-offset: 2px;
  cursor: pointer;
}
.markdown-body :deep(code) {
  font-family: ui-monospace, monospace;
  font-size: 0.85em;
  background: var(--muted);
  padding: 0.15em 0.4em;
  border-radius: 4px;
}
.markdown-body :deep(pre) {
  margin: 0.75em 0;
  padding: 0.75em 1em;
  background: var(--muted);
  border-radius: 6px;
  overflow-x: auto;
}
.markdown-body :deep(pre code) {
  background: transparent;
  padding: 0;
}
.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin: 0.5em 0;
  padding-left: 1.5em;
}
.markdown-body :deep(ul) {
  list-style: disc;
}
.markdown-body :deep(ol) {
  list-style: decimal;
}
.markdown-body :deep(li + li) {
  margin-top: 0.2em;
}
.markdown-body :deep(blockquote) {
  margin: 0.75em 0;
  padding: 0.25em 1em;
  border-left: 3px solid var(--border);
  color: var(--muted-foreground);
}
.markdown-body :deep(img) {
  max-width: 100%;
  border-radius: 4px;
}
.markdown-body :deep(hr) {
  margin: 1em 0;
  border: none;
  border-top: 1px solid var(--border);
}
.markdown-body :deep(table) {
  margin: 0.75em 0;
  border-collapse: collapse;
  font-size: 0.9em;
}
.markdown-body :deep(th),
.markdown-body :deep(td) {
  padding: 0.35em 0.75em;
  border: 1px solid var(--border);
}
.markdown-body :deep(th) {
  background: var(--muted);
  font-weight: 600;
}
</style>
