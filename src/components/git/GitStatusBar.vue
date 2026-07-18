<script setup lang="ts">
import { computed } from "vue";
import { ArrowDown, ArrowUp, GitBranch } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import type { Project } from "@/types";

const props = defineProps<{ project: Project }>();

const git = computed(() => props.project.git);
</script>

<template>
  <div class="flex flex-wrap items-center gap-x-3 gap-y-1.5 text-xs text-muted-foreground">
    <span v-if="git && !git.is_repo">非 Git 仓库</span>
    <template v-else-if="git">
      <Badge variant="secondary" class="gap-1">
        <GitBranch class="h-3 w-3" />
        {{ git.branch ?? "未知分支" }}
      </Badge>
      <span
        v-if="git.ahead > 0"
        class="flex items-center gap-0.5 text-emerald-600"
        title="领先远端"
      >
        <ArrowUp class="h-3 w-3" />
        {{ git.ahead }}
      </span>
      <span
        v-if="git.behind > 0"
        class="flex items-center gap-0.5 text-amber-600"
        title="落后远端"
      >
        <ArrowDown class="h-3 w-3" />
        {{ git.behind }}
      </span>
      <span>
        暂存
        <span class="font-medium text-emerald-600">{{ git.staged }}</span>
      </span>
      <span>
        未暂存
        <span class="font-medium text-amber-600">{{ git.modified }}</span>
      </span>
      <span>
        未跟踪
        <span class="font-medium text-sky-600">{{ git.untracked }}</span>
      </span>
      <span v-if="git.remote_ahead > 0" class="text-amber-600">
        远端领先
        <span class="font-medium">{{ git.remote_ahead }}</span>
      </span>
    </template>
  </div>
</template>
