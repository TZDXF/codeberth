<script setup lang="ts">
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";
import { GitBranch, X } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import OpenWithMenu from "@/components/open/OpenWithMenu.vue";
import { useProjectsStore } from "@/stores/projects";
import type { Project } from "@/types";

const props = defineProps<{ project: Project }>();

const router = useRouter();
const store = useProjectsStore();

function open() {
  router.push(`/projects/${props.project.id}`);
}

async function remove() {
  if (!window.confirm(`确定删除项目「${props.project.name}」吗?(不会删除磁盘文件)`)) return;
  try {
    await store.deleteProject(props.project.id);
    toast.success(`已删除项目「${props.project.name}」`);
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <div
    class="group cursor-pointer rounded-md border px-3 py-2 transition-colors hover:bg-accent/60"
    @click="open"
  >
    <div class="flex items-center justify-between gap-2">
      <span class="truncate text-sm font-medium">{{ project.name }}</span>
      <div
        class="flex shrink-0 items-center opacity-0 transition-opacity group-hover:opacity-100"
      >
        <OpenWithMenu :project="project" compact />
        <Button
          variant="ghost"
          size="icon"
          class="h-7 w-7"
          title="删除项目"
          @click.stop="remove"
        >
          <X class="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
    <p
      v-if="project.description"
      class="mt-0.5 truncate text-xs"
      :title="project.description"
    >
      {{ project.description }}
    </p>
    <p class="truncate text-xs text-muted-foreground" :title="project.path">
      {{ project.path }}
    </p>
    <div
      v-if="project.git?.is_repo"
      class="mt-1 flex items-center gap-2 text-[11px] text-muted-foreground"
    >
      <span class="flex min-w-0 items-center gap-1">
        <GitBranch class="h-3 w-3 shrink-0" />
        <span class="truncate">{{ project.git.branch ?? "未知" }}</span>
      </span>
      <span v-if="project.git.staged" class="text-emerald-600">
        +{{ project.git.staged }}
      </span>
      <span v-if="project.git.modified" class="text-amber-600">
        ~{{ project.git.modified }}
      </span>
      <span v-if="project.git.untracked" class="text-sky-600">
        ?{{ project.git.untracked }}
      </span>
      <span v-if="project.git.remote_ahead" class="text-amber-600" title="远端领先">
        ↓{{ project.git.remote_ahead }}
      </span>
    </div>
    <div v-if="project.tags.length" class="mt-1.5 flex flex-wrap gap-1">
      <Badge
        v-for="tag in project.tags"
        :key="tag.id"
        variant="secondary"
        class="px-1.5 py-0 text-[11px]"
        :style="{ backgroundColor: tag.color + '22', color: tag.color }"
      >
        {{ tag.name }}
      </Badge>
    </div>
  </div>
</template>
