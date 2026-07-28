<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  ChevronRight,
  Container,
  Eye,
  EyeOff,
  FileCode,
  Hammer,
  Play,
  RotateCw,
  Square,
} from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useCollapsibleOpen } from "@/composables/useCollapsibleOpen";
import { cmd, runInTerminal } from "@/lib/tauri";
import type { ComposeFile, ComposeServiceState, HiddenItem, Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project }>();

const { isOpen, setOpen } = useCollapsibleOpen("compose");

const files = ref<ComposeFile[]>([]);
const loaded = ref(false);
/** 各 compose 文件展开状态,key 为文件路径 */
const openStates = ref<Record<string, boolean>>({});
/** 服务运行状态,key 为 `${file.path}\n${service}`;无记录表示未创建/docker 不可用 */
const statuses = ref<Record<string, ComposeServiceState>>({});
const refreshing = ref(false);
/** 已隐藏的 compose 文件路径 */
const hiddenFiles = ref<Set<string>>(new Set());
/** 临时显示已隐藏文件(灰显,可逐个恢复) */
const showHidden = ref(false);

const stateKey = (f: ComposeFile, name: string) => `${f.path}\n${name}`;

const hiddenCount = computed(() => hiddenFiles.value.size);

/** 当前应展示的文件:过滤隐藏文件;showHidden 时全部显示但标记 hidden 灰显 */
const displayFiles = computed(() =>
  files.value
    .map((f) => ({ file: f, hidden: hiddenFiles.value.has(f.path) }))
    .filter((x) => showHidden.value || !x.hidden),
);

watch(
  () => props.project.id,
  async () => {
    loaded.value = false;
    showHidden.value = false;
    try {
      const [fs, items] = await Promise.all([
        cmd<ComposeFile[]>("scan_compose_files", { path: props.project.path }),
        cmd<HiddenItem[]>("list_hidden_items", { projectId: props.project.id }),
      ]);
      files.value = fs;
      hiddenFiles.value = new Set(
        items.filter((i) => i.kind === "composeFile").map((i) => i.targetKey),
      );
    } catch {
      files.value = [];
      hiddenFiles.value = new Set();
    } finally {
      loaded.value = true;
    }
    openStates.value = Object.fromEntries(
      files.value.map((f) => [
        f.path,
        isOpen(`${props.project.id}:${f.path}`, files.value.length === 1),
      ]),
    );
    loadStatuses();
  },
  { immediate: true },
);

function onToggle(f: ComposeFile, open: boolean) {
  openStates.value[f.path] = open;
  setOpen(`${props.project.id}:${f.path}`, open);
}

async function toggleFileHidden(path: string, hidden: boolean) {
  try {
    await cmd("set_hidden_item", {
      projectId: props.project.id,
      kind: "composeFile",
      targetKey: path,
      hidden: !hidden,
    });
    const next = new Set(hiddenFiles.value);
    if (hidden) next.delete(path);
    else next.add(path);
    hiddenFiles.value = next;
  } catch (e) {
    toast.error(String(e));
  }
}

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
  action: "up -d" | "up -d --build" | "restart" | "down" | "stop",
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
  <!-- 全部隐藏时保留头部,以便通过「显示已隐藏」恢复 -->
  <Card v-if="loaded && (displayFiles.length || hiddenCount)" class="group/card">
    <CardHeader class="pb-3">
      <CardTitle class="flex items-center gap-2 text-sm font-semibold">
        <Container class="h-4 w-4" />
        {{ t("docker.title") }}
        <template v-if="hiddenCount">
          <Button
            variant="ghost"
            size="icon"
            class="ml-auto h-6 w-6 shrink-0 text-muted-foreground transition-opacity"
            :class="{ 'opacity-0 group-hover/card:opacity-100': !showHidden }"
            :title="showHidden ? t('common.hideShown') : t('common.showHidden')"
            @click="showHidden = !showHidden"
          >
            <EyeOff v-if="showHidden" class="h-3.5 w-3.5" />
            <Eye v-else class="h-3.5 w-3.5" />
          </Button>
        </template>
      </CardTitle>
    </CardHeader>
    <CardContent>
      <ScrollArea class="max-h-[320px]">
        <div class="flex flex-col">
          <Collapsible
            v-for="(d, i) in displayFiles"
            :key="`${project.id}:${d.file.path}`"
            v-slot="{ open }"
            :open="files.length > 1 ? openStates[d.file.path] : true"
            :class="{ 'mt-2 border-t border-border pt-2': i > 0 }"
            @update:open="onToggle(d.file, $event)"
          >
            <div
              class="group flex items-center gap-2 rounded-md px-2 py-1.5"
              :class="{ 'opacity-50': d.hidden }"
            >
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
                <span class="min-w-0 flex-1 truncate font-mono text-xs" :title="d.file.path">
                  {{ d.file.path }}
                </span>
                <span v-if="!open" class="shrink-0 text-xs text-muted-foreground">
                  {{ t("docker.serviceCount", { count: d.file.services.length }) }}
                </span>
              </CollapsibleTrigger>
              <template v-else>
                <FileCode class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                <span class="min-w-0 flex-1 truncate font-mono text-xs" :title="d.file.path">
                  {{ d.file.path }}
                </span>
              </template>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 transition-opacity"
                :class="d.hidden ? 'text-muted-foreground' : 'opacity-0 group-hover:opacity-100'"
                :title="d.hidden ? t('common.unhide') : t('docker.hideFile')"
                @click="toggleFileHidden(d.file.path, d.hidden)"
              >
                <Eye v-if="d.hidden" class="h-3.5 w-3.5" />
                <EyeOff v-else class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-emerald-600"
                :title="t('docker.up')"
                @click="run(d.file, 'up -d')"
              >
                <Play class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-sky-600"
                :title="t('docker.rebuild')"
                @click="run(d.file, 'up -d --build')"
              >
                <Hammer class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-amber-600"
                :title="t('docker.restart')"
                @click="run(d.file, 'restart')"
              >
                <RotateCw class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-red-600"
                :title="t('docker.stop')"
                @click="run(d.file, 'down')"
              >
                <Square class="h-3.5 w-3.5" />
              </Button>
            </div>
            <CollapsibleContent>
              <div
                v-for="s in d.file.services"
                :key="s.name"
                class="group flex items-center gap-2 rounded-md px-2 py-1.5 pl-7 hover:bg-accent"
              >
                <span
                  class="h-2 w-2 shrink-0 rounded-full"
                  :class="dotClass(d.file, s.name)"
                  :title="stateTitle(d.file, s.name)"
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
                  :title="t('docker.up')"
                  @click="run(d.file, 'up -d', s.name)"
                >
                  <Play class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 shrink-0 text-sky-600 opacity-0 transition-opacity group-hover:opacity-100"
                  :title="t('docker.rebuild')"
                  @click="run(d.file, 'up -d --build', s.name)"
                >
                  <Hammer class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 shrink-0 text-amber-600 opacity-0 transition-opacity group-hover:opacity-100"
                  :title="t('docker.restart')"
                  @click="run(d.file, 'restart', s.name)"
                >
                  <RotateCw class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 shrink-0 text-red-600 opacity-0 transition-opacity group-hover:opacity-100"
                  :title="t('docker.stop')"
                  @click="run(d.file, 'stop', s.name)"
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
