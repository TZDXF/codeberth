<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Code, Terminal, TriangleAlert } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cmd } from "@/lib/tauri";
import type { EditorKind, Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project; conflicts: string[] }>();
const open = defineModel<boolean>("open", { required: true });

// VSCode 可用性(与 OpenWithMenu 同一探测命令,结果有缓存)
const vscodeAvailable = ref<boolean | null>(null);

onMounted(async () => {
  try {
    vscodeAvailable.value = await cmd<boolean>("detect_vscode");
  } catch {
    vscodeAvailable.value = false;
  }
});

/** 冲突不在应用内解决:引导用户到更合适的工具中处理 */
async function openIn(kind: EditorKind) {
  try {
    await cmd("open_with", { path: props.project.path, kind });
    open.value = false;
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent>
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <TriangleAlert class="h-4 w-4 text-amber-500" />
          {{ t("git.conflict.title") }}
        </DialogTitle>
        <DialogDescription>
          {{ t("git.conflict.description", { count: conflicts.length }) }}
        </DialogDescription>
      </DialogHeader>
      <div class="flex flex-col gap-1.5">
        <p class="text-sm font-medium">{{ t("git.conflict.files") }}</p>
        <ScrollArea class="h-40 rounded-md border">
          <ul class="p-2 font-mono text-xs text-muted-foreground">
            <li v-for="f in conflicts" :key="f" class="truncate py-0.5" :title="f">
              {{ f }}
            </li>
          </ul>
        </ScrollArea>
      </div>
      <DialogFooter class="gap-2">
        <Button variant="outline" :disabled="vscodeAvailable === false" @click="openIn('vscode')">
          <Code class="h-4 w-4" />
          {{ t("git.conflict.openVscode") }}
        </Button>
        <Button variant="outline" @click="openIn('terminal')">
          <Terminal class="h-4 w-4" />
          {{ t("git.conflict.openTerminal") }}
        </Button>
        <Button variant="ghost" @click="open = false">
          {{ t("git.conflict.close") }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
