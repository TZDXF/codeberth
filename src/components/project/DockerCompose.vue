<script setup lang="ts">
import { ref, watch } from "vue";
import { toast } from "vue-sonner";
import { Container, Play, RotateCw, Square } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cmd, runInTerminal } from "@/lib/tauri";
import type { ComposeFile, Project } from "@/types";

const props = defineProps<{ project: Project }>();

const compose = ref<ComposeFile | null>(null);
const loaded = ref(false);

watch(
  () => props.project.id,
  async () => {
    loaded.value = false;
    try {
      compose.value = await cmd<ComposeFile | null>("detect_compose_file", {
        path: props.project.path,
      });
    } catch {
      compose.value = null;
    } finally {
      loaded.value = true;
    }
  },
  { immediate: true },
);

/** 在项目终端执行 docker compose 命令;service 为空表示作用于所有服务 */
async function run(action: "up -d" | "restart" | "down" | "stop", service?: string) {
  const args = service ? `${action} ${service}` : action;
  try {
    await runInTerminal(props.project, `docker compose ${args}`);
    toast.success(`已在终端启动 docker compose ${args}`);
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <Card v-if="loaded && compose">
    <CardHeader class="flex-row items-center justify-between pb-3">
      <CardTitle class="flex items-center gap-2 text-sm font-semibold">
        <Container class="h-4 w-4" />
        Docker Compose
        <span class="text-xs font-normal text-muted-foreground">
          {{ compose.file_name }}
        </span>
      </CardTitle>
      <div class="flex gap-1.5">
        <Button size="sm" variant="outline" title="启动全部服务" @click="run('up -d')">
          <Play class="h-3.5 w-3.5 text-emerald-600" />
          启动
        </Button>
        <Button size="sm" variant="outline" title="重启全部服务" @click="run('restart')">
          <RotateCw class="h-3.5 w-3.5 text-amber-600" />
          重启
        </Button>
        <Button size="sm" variant="outline" title="停止并移除全部容器" @click="run('down')">
          <Square class="h-3.5 w-3.5 text-red-600" />
          停止
        </Button>
      </div>
    </CardHeader>
    <CardContent v-if="compose.services.length">
      <ScrollArea class="max-h-[240px]">
        <div class="flex flex-col">
          <div
            v-for="s in compose.services"
            :key="s"
            class="group flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-accent"
          >
            <Container class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
            <span class="min-w-0 flex-1 truncate font-mono text-sm" :title="s">
              {{ s }}
            </span>
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 shrink-0 text-emerald-600 opacity-0 transition-opacity group-hover:opacity-100"
              :title="`启动服务: docker compose up -d ${s}`"
              @click="run('up -d', s)"
            >
              <Play class="h-3.5 w-3.5" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 shrink-0 text-amber-600 opacity-0 transition-opacity group-hover:opacity-100"
              :title="`重启服务: docker compose restart ${s}`"
              @click="run('restart', s)"
            >
              <RotateCw class="h-3.5 w-3.5" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 shrink-0 text-red-600 opacity-0 transition-opacity group-hover:opacity-100"
              :title="`停止服务: docker compose stop ${s}`"
              @click="run('stop', s)"
            >
              <Square class="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </ScrollArea>
    </CardContent>
  </Card>
</template>
