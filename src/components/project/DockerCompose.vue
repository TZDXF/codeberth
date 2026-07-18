<script setup lang="ts">
import { ref, watch } from "vue";
import { toast } from "vue-sonner";
import { Container, Play, RotateCw, Square } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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

interface ComposeAction {
  label: string;
  args: string;
}

async function run(action: ComposeAction) {
  try {
    await runInTerminal(props.project, `docker compose ${action.args}`);
    toast.success(`已在终端启动 docker compose ${action.args}`);
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <Card v-if="loaded && compose">
    <CardHeader class="pb-3">
      <CardTitle class="flex items-center gap-2 text-sm font-semibold">
        <Container class="h-4 w-4" />
        Docker Compose
        <span class="text-xs font-normal text-muted-foreground">
          {{ compose.file_name }}
        </span>
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="flex flex-wrap gap-2">
        <Button size="sm" variant="outline" @click="run({ label: '启动', args: 'up -d' })">
          <Play class="h-3.5 w-3.5 text-emerald-600" />
          启动 (up -d)
        </Button>
        <Button size="sm" variant="outline" @click="run({ label: '重启', args: 'restart' })">
          <RotateCw class="h-3.5 w-3.5 text-amber-600" />
          重启 (restart)
        </Button>
        <Button size="sm" variant="outline" @click="run({ label: '停止', args: 'down' })">
          <Square class="h-3.5 w-3.5 text-red-600" />
          停止 (down)
        </Button>
      </div>
    </CardContent>
  </Card>
</template>
