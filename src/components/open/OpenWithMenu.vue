<script setup lang="ts">
import { computed, onMounted, ref, type Component } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Check, ChevronDown, Code, FolderOpen, Terminal } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cmd } from "@/lib/tauri";
import { useSettingsStore } from "@/stores/settings";
import type { EditorKind, Project } from "@/types";

const { t } = useI18n();
const props = withDefaults(defineProps<{ project: Project; compact?: boolean }>(), {
  compact: false,
});

const settings = useSettingsStore();

const OPTIONS: { kind: EditorKind; icon: Component; labelKey: string; descKey: string }[] = [
  { kind: "explorer", icon: FolderOpen, labelKey: "openWith.explorer", descKey: "openWith.openInExplorer" },
  { kind: "vscode", icon: Code, labelKey: "openWith.vscode", descKey: "openWith.openInVscode" },
  { kind: "terminal", icon: Terminal, labelKey: "openWith.terminal", descKey: "openWith.openInTerminal" },
];

const current = computed(
  () => OPTIONS.find((opt) => opt.kind === settings.defaultOpenWith) ?? OPTIONS[0],
);

const vscodeAvailable = ref<boolean | null>(null);

onMounted(async () => {
  try {
    vscodeAvailable.value = await cmd<boolean>("detect_vscode");
  } catch {
    vscodeAvailable.value = false;
  }
});

function isDisabled(kind: EditorKind) {
  return kind === "vscode" && vscodeAvailable.value === false;
}

const primaryTooltip = computed(() =>
  current.value.kind === "vscode" && vscodeAvailable.value === false
    ? t("openWith.vscodeUnavailable")
    : t(current.value.descKey),
);

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
    <div class="flex items-center">
      <Tooltip>
        <TooltipTrigger as-child>
          <span class="inline-flex">
            <Button
              :variant="compact ? 'ghost' : 'outline'"
              :size="compact ? 'icon' : 'sm'"
              :class="[compact ? 'h-7 w-7' : 'rounded-r-none']"
              :disabled="isDisabled(current.kind)"
              @click.stop="openWith(current.kind)"
            >
              <component
                :is="current.icon"
                :class="compact ? 'h-3.5 w-3.5' : 'h-4 w-4'"
              />
              <template v-if="!compact">{{ t(current.labelKey) }}</template>
            </Button>
          </span>
        </TooltipTrigger>
        <TooltipContent>{{ primaryTooltip }}</TooltipContent>
      </Tooltip>
      <DropdownMenu>
        <Tooltip>
          <TooltipTrigger as-child>
            <DropdownMenuTrigger as-child>
              <Button
                :variant="compact ? 'ghost' : 'outline'"
                :size="compact ? 'icon' : 'sm'"
                :class="[compact ? 'h-7 w-7' : 'rounded-l-none border-l-0 px-2']"
                @click.stop
              >
                <ChevronDown :class="compact ? 'h-3.5 w-3.5' : 'h-4 w-4'" />
              </Button>
            </DropdownMenuTrigger>
          </TooltipTrigger>
          <TooltipContent>{{ t("openWith.more") }}</TooltipContent>
        </Tooltip>
        <DropdownMenuContent align="end" class="w-52" @click.stop>
          <DropdownMenuItem
            v-for="opt in OPTIONS"
            :key="opt.kind"
            class="gap-2 text-xs"
            :disabled="isDisabled(opt.kind)"
            @click="openWith(opt.kind)"
          >
            <component :is="opt.icon" class="h-3.5 w-3.5" />
            {{ t(opt.descKey) }}
            <Check
              v-if="opt.kind === settings.defaultOpenWith"
              class="ml-auto h-3.5 w-3.5 text-primary"
            />
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  </TooltipProvider>
</template>
