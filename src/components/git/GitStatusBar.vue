<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { ChevronDown, GitBranch } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import GitBranchMenu from "@/components/git/GitBranchMenu.vue";
import GitRemoteLink from "@/components/git/GitRemoteLink.vue";
import type { Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project }>();

const git = computed(() => props.project.git);
</script>

<template>
  <div class="flex flex-wrap items-center gap-x-3 gap-y-1.5 text-xs text-muted-foreground">
    <span v-if="git && !git.is_repo">{{ t("git.notARepo") }}</span>
    <template v-else-if="git">
      <GitBranchMenu :project="project">
        <Badge
          variant="secondary"
          class="cursor-pointer gap-1 transition-colors hover:bg-accent"
          :title="t('git.branch.switch')"
        >
          <GitBranch class="h-3 w-3" />
          {{ git.branch ?? t("git.unknownBranch") }}
          <ChevronDown class="h-3 w-3 opacity-60" />
        </Badge>
      </GitBranchMenu>
      <GitRemoteLink :project="project" />
      <span v-if="git.conflicted > 0" class="text-red-600">
        {{ t("git.conflicted") }}
        <span class="font-medium">{{ git.conflicted }}</span>
      </span>
    </template>
  </div>
</template>
