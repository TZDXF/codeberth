<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { toast } from "vue-sonner";
import { X } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useProjectsStore } from "@/stores/projects";
import type { Project } from "@/types";

const props = defineProps<{ project: Project }>();

const route = useRoute();
const router = useRouter();
const store = useProjectsStore();

const active = computed(() => Number(route.params.id) === props.project.id);

function open() {
  router.push(`/projects/${props.project.id}`);
}

async function remove() {
  if (!window.confirm(`确定删除项目「${props.project.name}」吗?(不会删除磁盘文件)`)) return;
  try {
    await store.deleteProject(props.project.id);
    toast.success(`已删除项目「${props.project.name}」`);
    if (active.value) router.push("/");
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <div
    class="group cursor-pointer rounded-md border px-3 py-2 transition-colors"
    :class="active ? 'border-primary/50 bg-accent' : 'hover:bg-accent/60'"
    @click="open"
  >
    <div class="flex items-center justify-between gap-2">
      <span class="truncate text-sm font-medium">{{ project.name }}</span>
      <Button
        variant="ghost"
        size="icon"
        class="h-6 w-6 shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
        title="删除项目"
        @click.stop="remove"
      >
        <X class="h-3.5 w-3.5" />
      </Button>
    </div>
    <p class="truncate text-xs text-muted-foreground" :title="project.path">
      {{ project.path }}
    </p>
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

