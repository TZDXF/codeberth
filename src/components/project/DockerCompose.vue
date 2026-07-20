<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ChevronRight, Container, FileCode, Play, RefreshCw, RotateCw, Square } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cmd, runInTerminal } from "@/lib/tauri";
import type { ComposeFile, ComposeServiceState, Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project }>();

const files = ref<ComposeFile[]>([]);
const loaded = ref(false);
/** 服务运行状态,key 为 `${file.path}\n${service}`;无记录表示未创建/docker 不可用 */
const statuses = ref<Record<string, ComposeServiceState>>({});
const refreshing = ref(false);

const stateKey = (f: ComposeFile, name: string) => `${f.path}\n${name}`;

watch(
  () => props.project.id,
  async () => {
    loaded.value = false;
    try {
      files.value = await cmd<ComposeFile[]>("scan_compose_files", {
        path: props.project.path,
      });
    } catch {
      files.value = [];
    } finally {
      loaded.value = true;
    }
    loadStatuses();
  },
  { immediate: true },
);

/** 查询每个 compose 文件的服务运行状态(失败静默,全部按未知处理) */
async function loadStatuses() {
  if (!files.value.length) {
    statuses.value = {};
    return;
  }
  refreshing.value = true;
  try {
    const results = await Promise.all(
      files.value.map(async (f) => {
        try {
          return await cmd<ComposeServiceState[]>("compose_ps", {
            path: props.project.path,
            file: f.path,
          });
        } catch {
          return [] as ComposeServiceState[];
        }
      }),
    );
    const map: Record<string, ComposeServiceState> = {};
    files.value.forEach((f, i) => {
      for (const st of results[i]) map[stateKey(f, st.name)] = st;
    });
    statuses.value = map;
  } finally {
    refreshing.value = false;
  }
}

function stateOf(f: ComposeFile, name: string): ComposeServiceState | undefined {
  return statuses.value[stateKey(f, name)];
}

/** 状态点颜色:绿=运行中;黄=容器存在但未运行(exited 等);灰=未创建或 docker 不可用 */
function dotClass(f: ComposeFile, name: string): string {
  const st = stateOf(f, name);
  if (!st) return "bg-muted-foreground/40";
  return st.running ? "bg-emerald-500" : "bg-amber-500";
}

function stateTitle(f: ComposeFile, name: string): string {
  const st = stateOf(f, name);
  if (!st) return t("docker.statusUnknown");
  return st.status || t(st.running ? "docker.running" : "docker.stopped");
}

/** 在浏览器访问服务暴露到宿主机的端口 */
async function openPort(port: number) {
  try {
    await openUrl(`http://localhost:${port}`);
  } catch (e) {
    toast.error(String(e));
  }
}

/** 在项目终端执行 docker compose 命令;service 为空表示作用于该文件的所有服务 */
async function run(
  file: ComposeFile,
  action: "up -d" | "restart" | "down" | "stop",
  service?: string,
) {
  const args = `-f "${file.path}" ${service ? `${action} ${service}` : action}`;
  try {
    await runInTerminal(props.project, `docker compose ${args}`);
    toast.success(t("docker.started", { args }));
    // 命令在新终端窗口中异步执行,延迟刷新一次状态(拉取镜像时可能仍偏早,可手动刷新)
    setTimeout(loadStatuses, 4000);
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <Card v-if="loaded && files.length">
    <CardHeader class="pb-3">
      <CardTitle class="flex items-center gap-2 text-sm font-semibold">
        <Container class="h-4 w-4" />
        {{ t("docker.title") }}
        <span class="text-xs font-normal text-muted-foreground">
          {{ t("docker.fileCount", { count: files.length }) }}
        </span>
        <Button
          variant="ghost"
          size="icon"
          class="ml-auto h-6 w-6 shrink-0 text-muted-foreground"
          :title="t('docker.refreshStatus')"
          @click="loadStatuses"
        >
          <RefreshCw class="h-3.5 w-3.5" :class="{ 'animate-spin': refreshing }" />
        </Button>
      </CardTitle>
    </CardHeader>
    <CardContent>
      <ScrollArea class="max-h-[320px]">
        <div class="flex flex-col">
          <Collapsible
            v-for="(f, i) in files"
            :key="`${project.id}:${f.path}`"
            v-slot="{ open }"
            :default-open="files.length === 1"
            :class="{ 'mt-2 border-t border-border pt-2': i > 0 }"
          >
            <div class="group flex items-center gap-2 rounded-md px-2 py-1.5">
              <!-- 多文件时文件名区域可点击折叠;单文件保持静态展示 -->
              <CollapsibleTrigger
                v-if="files.length > 1"
                class="flex min-w-0 flex-1 cursor-pointer items-center gap-2 self-stretch rounded-md text-left hover:bg-accent"
                :title="open ? t('common.collapse') : t('common.expand')"
              >
                <ChevronRight
                  class="h-3.5 w-3.5 shrink-0 text-muted-foreground transition-transform"
                  :class="{ 'rotate-90': open }"
                />
                <FileCode class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                <span class="min-w-0 flex-1 truncate font-mono text-xs" :title="f.path">
                  {{ f.path }}
                </span>
                <span v-if="!open" class="shrink-0 text-xs text-muted-foreground">
                  {{ t("docker.serviceCount", { count: f.services.length }) }}
                </span>
              </CollapsibleTrigger>
              <template v-else>
                <FileCode class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                <span class="min-w-0 flex-1 truncate font-mono text-xs" :title="f.path">
                  {{ f.path }}
                </span>
              </template>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-emerald-600"
                :title="t('docker.upAll')"
                @click="run(f, 'up -d')"
              >
                <Play class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-amber-600"
                :title="t('docker.restartAll')"
                @click="run(f, 'restart')"
              >
                <RotateCw class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-red-600"
                :title="t('docker.stopAll')"
                @click="run(f, 'down')"
              >
                <Square class="h-3.5 w-3.5" />
              </Button>
            </div>
            <CollapsibleContent>
              <div
                v-for="s in f.services"
                :key="s.name"
                class="group flex items-center gap-2 rounded-md px-2 py-1.5 pl-7 hover:bg-accent"
              >
                <span
                  class="h-2 w-2 shrink-0 rounded-full"
                  :class="dotClass(f, s.name)"
                  :title="stateTitle(f, s.name)"
                />
                <span class="min-w-0 truncate font-mono text-sm" :title="s.name">
                  {{ s.name }}
                </span>
                <button
                  v-for="p in s.ports"
                  :key="p"
                  class="shrink-0 rounded border border-border px-1 font-mono text-[10px] leading-4 text-sky-600 hover:bg-accent dark:text-sky-400"
                  :title="t('docker.openPort', { port: p })"
                  @click.stop="openPort(p)"
                >
                  :{{ p }}
                </button>
                <span class="min-w-0 flex-1" />
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 shrink-0 text-emerald-600 opacity-0 transition-opacity group-hover:opacity-100"
                  :title="t('docker.upService', { service: s.name })"
                  @click="run(f, 'up -d', s.name)"
                >
                  <Play class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 shrink-0 text-amber-600 opacity-0 transition-opacity group-hover:opacity-100"
                  :title="t('docker.restartService', { service: s.name })"
                  @click="run(f, 'restart', s.name)"
                >
                  <RotateCw class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 shrink-0 text-red-600 opacity-0 transition-opacity group-hover:opacity-100"
                  :title="t('docker.stopService', { service: s.name })"
                  @click="run(f, 'stop', s.name)"
                >
                  <Square class="h-3.5 w-3.5" />
                </Button>
              </div>
            </CollapsibleContent>
          </Collapsible>
        </div>
      </ScrollArea>
    </CardContent>
  </Card>
</template>
