<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { ChevronDown, FolderGit2, GitBranch, Loader2 } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import GitBranchMenu from "@/components/git/GitBranchMenu.vue";
import GitRemoteLink from "@/components/git/GitRemoteLink.vue";
import { useProjectsStore } from "@/stores/projects";
import type { Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project }>();
const store = useProjectsStore();

const git = computed(() => props.project.git);
const initializing = ref(false);

async function initRepo() {
  if (initializing.value) return;
  initializing.value = true;
  try {
    await store.initRepository(props.project);
    toast.success(t("git.init.success"));
  } catch (e) {
    toast.error(String(e));
  } finally {
    initializing.value = false;
  }
}
</script>

<template>
  <div class="flex flex-wrap items-center gap-x-3 gap-y-1.5 text-xs text-muted-foreground">
    <Button
      v-if="git && !git.is_repo"
      variant="outline"
      size="xs"
      :disabled="initializing"
      @click="initRepo"
    >
      <Loader2 v-if="initializing" class="h-3.5 w-3.5 animate-spin" />
      <FolderGit2 v-else class="h-3.5 w-3.5" />
      {{ t("git.init.action") }}
    </Button>
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
