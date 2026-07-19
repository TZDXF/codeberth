<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Folder, Package } from "@lucide/vue";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import ScriptItem from "@/components/scripts/ScriptItem.vue";
import { cmd, runInTerminal } from "@/lib/tauri";
import type { PackageScript, PackageScriptsGroup, Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project }>();

const groups = ref<PackageScriptsGroup[]>([]);
const loaded = ref(false);

watch(
  () => props.project.id,
  async () => {
    loaded.value = false;
    try {
      groups.value = await cmd<PackageScriptsGroup[]>("list_package_scripts", {
        path: props.project.path,
      });
    } catch {
      groups.value = [];
    } finally {
      loaded.value = true;
    }
  },
  { immediate: true },
);

function groupLabel(g: PackageScriptsGroup): string {
  return g.dir === "." ? t("scripts.package.rootDir") : g.dir;
}

async function run(group: PackageScriptsGroup, script: PackageScript) {
  // monorepo 子包:在其所在目录内执行 npm run
  const cwd = group.dir === "." ? undefined : `${props.project.path}/${group.dir}`;
  try {
    await runInTerminal(props.project, `npm run ${script.name}`, cwd);
    toast.success(t("scripts.package.started", { name: script.name }));
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
        {{ t("scripts.package.title") }}
      </CardTitle>
    </CardHeader>
    <CardContent>
      <p v-if="!loaded" class="text-sm text-muted-foreground">{{ t("scripts.package.loading") }}</p>
      <p v-else-if="!groups.length" class="text-sm text-muted-foreground">
        {{ t("scripts.package.empty") }}
      </p>
      <ScrollArea v-else class="max-h-[420px]">
        <div class="flex flex-col">
          <template v-for="(g, gi) in groups" :key="g.dir">
            <div
              v-if="groups.length > 1"
              class="flex items-center gap-2 px-2 py-1.5"
              :class="{ 'mt-2 border-t border-border pt-2': gi > 0 }"
            >
              <Folder class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
              <span class="min-w-0 flex-1 truncate font-mono text-xs font-medium" :title="g.dir">
                {{ groupLabel(g) }}
              </span>
              <span
                v-if="g.package_name"
                class="shrink-0 truncate text-xs text-muted-foreground"
                :title="g.package_name"
              >
                {{ g.package_name }}
              </span>
            </div>
            <ScriptItem
              v-for="s in g.scripts"
              :key="`${g.dir}:${s.name}`"
              :name="s.name"
              :command="s.command"
              @run="run(g, s)"
            />
          </template>
        </div>
      </ScrollArea>
    </CardContent>
  </Card>
</template>
