<script setup lang="ts">
import { onMounted, ref } from "vue";
import { toast } from "vue-sonner";
import { Archive, Code, FolderOpen, MoreHorizontal, Terminal } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cmd } from "@/lib/tauri";
import { useProjectsStore } from "@/stores/projects";
import type { EditorKind, Project } from "@/types";

const props = defineProps<{ project: Project }>();

const store = useProjectsStore();

// VSCode 探测结果模块级共享,避免每个项目实例重复 invoke
let vscodePromise: Promise<boolean> | null = null;
function detectVscode(): Promise<boolean> {
  vscodePromise ??= cmd<boolean>("detect_vscode").catch(() => false);
  return vscodePromise;
}

const vscodeAvailable = ref<boolean | null>(null);

onMounted(async () => {
  vscodeAvailable.value = await detectVscode();
});

async function openWith(kind: EditorKind) {
  try {
    await cmd("open_with", { path: props.project.path, kind });
  } catch (e) {
    toast.error(String(e));
  }
}

async function archive() {
  const ok = window.confirm(
    `确定归档项目「${props.project.name}」吗?\n归档后将不再显示,历史数据会保留。`,
  );
  if (!ok) return;
  try {
    await store.archiveProject(props.project.id);
    toast.success(`已归档项目「${props.project.name}」`);
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="ghost" size="icon" class="h-7 w-7" title="更多操作" @click.stop>
        <MoreHorizontal class="h-3.5 w-3.5" />
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end" class="w-44" @click.stop>
      <DropdownMenuItem
        class="gap-2 text-xs"
        :disabled="vscodeAvailable === false"
        @click="openWith('vscode')"
      >
        <Code class="h-3.5 w-3.5" />
        在 VSCode 中打开
      </DropdownMenuItem>
      <DropdownMenuItem class="gap-2 text-xs" @click="openWith('explorer')">
        <FolderOpen class="h-3.5 w-3.5" />
        在资源管理器中打开
      </DropdownMenuItem>
      <DropdownMenuItem class="gap-2 text-xs" @click="openWith('terminal')">
        <Terminal class="h-3.5 w-3.5" />
        在终端中打开
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem variant="destructive" class="gap-2 text-xs" @click="archive">
        <Archive class="h-3.5 w-3.5" />
        归档项目
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
