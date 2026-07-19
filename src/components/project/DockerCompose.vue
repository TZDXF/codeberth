<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Container, FileCode, Play, RotateCw, Square } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cmd, runInTerminal } from "@/lib/tauri";
import type { ComposeFile, Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project }>();

const files = ref<ComposeFile[]>([]);
const loaded = ref(false);

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
  },
  { immediate: true },
);

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
      </CardTitle>
    </CardHeader>
    <CardContent>
      <ScrollArea class="max-h-[320px]">
        <div class="flex flex-col">
          <div
            v-for="(f, i) in files"
            :key="f.path"
            :class="{ 'mt-2 border-t border-border pt-2': i > 0 }"
          >
            <div class="group flex items-center gap-2 rounded-md px-2 py-1.5">
              <FileCode class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
              <span class="min-w-0 flex-1 truncate font-mono text-xs" :title="f.path">
                {{ f.path }}
              </span>
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
            <div
              v-for="s in f.services"
              :key="s"
              class="group flex items-center gap-2 rounded-md px-2 py-1.5 pl-7 hover:bg-accent"
            >
              <Container class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
              <span class="min-w-0 flex-1 truncate font-mono text-sm" :title="s">
                {{ s }}
              </span>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-emerald-600 opacity-0 transition-opacity group-hover:opacity-100"
                :title="t('docker.upService', { service: s })"
                @click="run(f, 'up -d', s)"
              >
                <Play class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-amber-600 opacity-0 transition-opacity group-hover:opacity-100"
                :title="t('docker.restartService', { service: s })"
                @click="run(f, 'restart', s)"
              >
                <RotateCw class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 shrink-0 text-red-600 opacity-0 transition-opacity group-hover:opacity-100"
                :title="t('docker.stopService', { service: s })"
                @click="run(f, 'stop', s)"
              >
                <Square class="h-3.5 w-3.5" />
              </Button>
            </div>
          </div>
        </div>
      </ScrollArea>
    </CardContent>
  </Card>
</template>
