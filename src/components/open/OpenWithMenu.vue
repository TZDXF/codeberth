<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { ChevronDown } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { OPEN_WITH_OPTIONS } from "@/lib/open-with";
import { cmd } from "@/lib/tauri";
import { useSettingsStore } from "@/stores/settings";
import type { EditorKind, Project } from "@/types";

const { t } = useI18n();
const props = withDefaults(defineProps<{ project: Project; compact?: boolean }>(), {
  compact: false,
});

const settings = useSettingsStore();

const current = computed(
  () =>
    OPEN_WITH_OPTIONS.find((opt) => opt.kind === settings.defaultOpenWith) ?? OPEN_WITH_OPTIONS[0],
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

async function openWith(kind: EditorKind) {
  try {
    await cmd("open_with", { path: props.project.path, kind });
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <div class="flex items-center">
    <Button
      :variant="compact ? 'ghost' : 'outline'"
      :size="compact ? 'icon' : 'sm'"
      :class="[compact ? 'h-7 w-7' : 'rounded-r-none']"
      :disabled="isDisabled(current.kind)"
      @click.stop="openWith(current.kind)"
    >
      <component :is="current.icon" :class="compact ? 'h-3.5 w-3.5' : 'h-4 w-4'" />
      <template v-if="!compact">{{ t(current.labelKey) }}</template>
    </Button>
    <DropdownMenu>
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
      <DropdownMenuContent align="end" class="w-52" @click.stop>
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
      </DropdownMenuContent>
    </DropdownMenu>
  </div>
</template>
