<script setup lang="ts">
import { computed } from "vue";
import {
  ArrowDown,
  ArrowUp,
  CloudDownload,
  GitBranch,
  GitCommitHorizontal,
} from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { formatRelativeTime } from "@/lib/format";
import type { Project } from "@/types";

const props = defineProps<{ project: Project }>();

const git = computed(() => props.project.git);
const hasRemote = computed(
  () => git.value?.is_repo && (git.value.remote_ahead > 0 || git.value.last_fetch_at != null),
);
</script>

<template>
  <Card>
    <CardHeader class="pb-3">
      <CardTitle class="flex items-center gap-2 text-sm font-semibold">
        <GitBranch class="h-4 w-4" />
        Git 状态
      </CardTitle>
    </CardHeader>
    <CardContent v-if="!git" class="text-sm text-muted-foreground">
      检测中...
    </CardContent>
    <CardContent v-else-if="!git.is_repo" class="text-sm text-muted-foreground">
      该目录不是 Git 仓库
    </CardContent>
    <CardContent v-else class="flex flex-col gap-3 text-sm">
      <div class="flex flex-wrap items-center gap-2">
        <Badge variant="secondary" class="gap-1">
          <GitBranch class="h-3 w-3" />
          {{ git.branch ?? "未知分支" }}
        </Badge>
        <Badge v-if="git.ahead > 0" variant="outline" class="gap-1 text-emerald-600">
          <ArrowUp class="h-3 w-3" />
          {{ git.ahead }}
        </Badge>
        <Badge v-if="git.behind > 0" variant="outline" class="gap-1 text-amber-600">
          <ArrowDown class="h-3 w-3" />
          {{ git.behind }}
        </Badge>
      </div>
      <div class="flex gap-4 text-xs text-muted-foreground">
        <span>
          已暂存
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
      </div>
      <template v-if="hasRemote">
        <Separator />
        <div class="flex items-center gap-2 text-xs text-muted-foreground">
          <CloudDownload class="h-3.5 w-3.5" />
          <span v-if="git.remote_ahead > 0">
            远端领先
            <span class="font-medium text-amber-600">{{ git.remote_ahead }}</span>
            个提交
          </span>
          <span v-else>远端无新提交</span>
          <span class="ml-auto flex items-center gap-1">
            <GitCommitHorizontal class="h-3.5 w-3.5" />
            fetch {{ formatRelativeTime(git.last_fetch_at) }}
          </span>
        </div>
      </template>
    </CardContent>
  </Card>
</template>

