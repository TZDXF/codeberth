<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Archive, MoreHorizontal } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { OPEN_WITH_OPTIONS } from "@/lib/open-with";
import { cmd } from "@/lib/tauri";
import { useProjectsStore } from "@/stores/projects";
import type { EditorKind, Project } from "@/types";

const { t } = useI18n();
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

function isDisabled(kind: EditorKind): boolean {
  return kind === "vscode" && vscodeAvailable.value === false;
}

async function openWith(kind: EditorKind) {
  try {
    await cmd("open_with", { path: props.project.path, kind });
  } catch (e) {
    toast.error(String(e));
  }
}

async function archive() {
  const ok = window.confirm(t("projects.actions.archiveConfirm", { name: props.project.name }));
  if (!ok) return;
  try {
    await store.archiveProject(props.project.id);
    toast.success(t("projects.actions.archiveSuccess", { name: props.project.name }));
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7"
        :title="t('projects.actions.more')"
        @click.stop
      >
        <MoreHorizontal class="h-3.5 w-3.5" />
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end" class="w-44" @click.stop>
      <DropdownMenuItem
        v-for="opt in OPEN_WITH_OPTIONS"
        :key="opt.kind"
        class="gap-2 text-xs"
        :disabled="isDisabled(opt.kind)"
        @click="openWith(opt.kind)"
      >
        <component :is="opt.icon" class="h-3.5 w-3.5" />
        {{ t(opt.descKey) }}
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem variant="destructive" class="gap-2 text-xs" @click="archive">
        <Archive class="h-3.5 w-3.5" />
        {{ t("projects.actions.archive") }}
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
