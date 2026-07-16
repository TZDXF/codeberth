<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useProjectsStore } from "@/stores/projects";

const route = useRoute();
const store = useProjectsStore();

const project = computed(() => {
  const id = Number(route.params.id);
  return Number.isFinite(id) ? store.projects.find((p) => p.id === id) : undefined;
});
</script>

<template>
  <div v-if="project" class="flex h-full flex-col overflow-y-auto">
    <header class="border-b px-6 py-4">
      <h1 class="text-lg font-semibold">{{ project.name }}</h1>
      <p class="text-sm text-muted-foreground">{{ project.path }}</p>
      <p v-if="project.description" class="mt-1 text-sm">{{ project.description }}</p>
    </header>
  </div>
  <div
    v-else
    class="flex h-full items-center justify-center text-sm text-muted-foreground"
  >
    选择左侧项目查看详情,或点击「添加」创建新项目
  </div>
</template>

