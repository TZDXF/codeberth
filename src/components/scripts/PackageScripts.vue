<script setup lang="ts">
import { ref, watch } from "vue";
import { toast } from "vue-sonner";
import { Package } from "@lucide/vue";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import ScriptItem from "@/components/scripts/ScriptItem.vue";
import { cmd, runInTerminal } from "@/lib/tauri";
import type { PackageScript, Project } from "@/types";

const props = defineProps<{ project: Project }>();

const scripts = ref<PackageScript[]>([]);
const loaded = ref(false);

watch(
  () => props.project.id,
  async () => {
    loaded.value = false;
    try {
      scripts.value = await cmd<PackageScript[]>("list_package_scripts", {
        path: props.project.path,
      });
    } catch {
      scripts.value = [];
    } finally {
      loaded.value = true;
    }
  },
  { immediate: true },
);

async function run(script: PackageScript) {
  try {
    await runInTerminal(props.project, `npm run ${script.name}`);
    toast.success(`已在终端启动 npm run ${script.name}`);
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <Card>
    <CardHeader class="pb-3">
      <CardTitle class="flex items-center gap-2 text-sm font-semibold">
        <Package class="h-4 w-4" />
        NPM Scripts
      </CardTitle>
    </CardHeader>
    <CardContent>
      <p v-if="!loaded" class="text-sm text-muted-foreground">加载中...</p>
      <p v-else-if="!scripts.length" class="text-sm text-muted-foreground">
        未找到 package.json 或没有 scripts
      </p>
      <div v-else class="flex flex-col">
        <ScriptItem
          v-for="s in scripts"
          :key="s.name"
          :name="s.name"
          :command="s.command"
          @run="run(s)"
        />
      </div>
    </CardContent>
  </Card>
</template>

