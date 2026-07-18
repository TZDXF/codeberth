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

const props = withDefaults(defineProps<{ project: Project; compact?: boolean }>(), {
  compact: false,
});

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
    <div class="flex items-center" :class="compact ? 'gap-1' : 'gap-2'">
      <Tooltip>
        <TooltipTrigger as-child>
          <span class="inline-flex">
            <Button
              :variant="compact ? 'ghost' : 'outline'"
              :size="compact ? 'icon' : 'sm'"
              :class="compact && 'h-7 w-7'"
              :disabled="vscodeAvailable === false"
              @click.stop="openWith('vscode')"
            >
              <Code :class="compact ? 'h-3.5 w-3.5' : 'h-4 w-4'" />
              <template v-if="!compact">VSCode</template>
            </Button>
          </span>
        </TooltipTrigger>
        <TooltipContent>
          {{ vscodeAvailable === false ? "未检测到 VSCode(code 命令不可用)" : "在 VSCode 中打开" }}
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger as-child>
          <Button
            :variant="compact ? 'ghost' : 'outline'"
            :size="compact ? 'icon' : 'sm'"
            :class="compact && 'h-7 w-7'"
            @click.stop="openWith('explorer')"
          >
            <FolderOpen :class="compact ? 'h-3.5 w-3.5' : 'h-4 w-4'" />
            <template v-if="!compact">资源管理器</template>
          </Button>
        </TooltipTrigger>
        <TooltipContent>在系统文件管理器中打开</TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger as-child>
          <Button
            :variant="compact ? 'ghost' : 'outline'"
            :size="compact ? 'icon' : 'sm'"
            :class="compact && 'h-7 w-7'"
            @click.stop="openWith('terminal')"
          >
            <Terminal :class="compact ? 'h-3.5 w-3.5' : 'h-4 w-4'" />
            <template v-if="!compact">终端</template>
          </Button>
        </TooltipTrigger>
        <TooltipContent>在系统终端中打开该目录</TooltipContent>
      </Tooltip>
    </div>
  </TooltipProvider>
</template>
