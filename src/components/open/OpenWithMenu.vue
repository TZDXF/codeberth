<script setup lang="ts">
import { onMounted, ref } from "vue";
import { toast } from "vue-sonner";
import { Code, FolderOpen, Terminal } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cmd } from "@/lib/tauri";
import type { EditorKind, Project } from "@/types";

const props = defineProps<{ project: Project }>();

const vscodeAvailable = ref<boolean | null>(null);

onMounted(async () => {
  try {
    vscodeAvailable.value = await cmd<boolean>("detect_vscode");
  } catch {
    vscodeAvailable.value = false;
  }
});

async function openWith(kind: EditorKind) {
  try {
    await cmd("open_with", { path: props.project.path, kind });
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <TooltipProvider :delay-duration="300">
    <div class="flex items-center gap-2">
      <Tooltip>
        <TooltipTrigger as-child>
          <span>
            <Button
              variant="outline"
              size="sm"
              :disabled="vscodeAvailable === false"
              @click="openWith('vscode')"
            >
              <Code class="h-4 w-4" />
              VSCode
            </Button>
          </span>
        </TooltipTrigger>
        <TooltipContent v-if="vscodeAvailable === false">
          未检测到 VSCode(code 命令不可用)
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger as-child>
          <Button variant="outline" size="sm" @click="openWith('explorer')">
            <FolderOpen class="h-4 w-4" />
            资源管理器
          </Button>
        </TooltipTrigger>
        <TooltipContent>在系统文件管理器中打开</TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger as-child>
          <Button variant="outline" size="sm" @click="openWith('terminal')">
            <Terminal class="h-4 w-4" />
            终端
          </Button>
        </TooltipTrigger>
        <TooltipContent>在系统终端中打开该目录</TooltipContent>
      </Tooltip>
    </div>
  </TooltipProvider>
</template>

