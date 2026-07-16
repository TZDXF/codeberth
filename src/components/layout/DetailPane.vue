<script setup lang="ts">
import { computed, watch } from "vue";
import { useRoute } from "vue-router";
import GitStatusCard from "@/components/git/GitStatusCard.vue";
import OpenWithMenu from "@/components/open/OpenWithMenu.vue";
import { useProjectsStore } from "@/stores/projects";

const route = useRoute();
const store = useProjectsStore();

const project = computed(() => {
  const id = Number(route.params.id);
  return Number.isFinite(id) ? store.projects.find((p) => p.id === id) : undefined;
});

// 选中项目进入详情页时优先触发一次远端 fetch
watch(
  () => project.value?.id,
  () => {
    if (project.value) store.triggerRemoteFetch(project.value);
  },
  { immediate: true },
);
</script>

<template>
  <div v-if="project" class="flex h-full flex-col overflow-y-auto">
    <header class="border-b px-6 py-4">
      <div class="flex items-start justify-between gap-4">
        <h1 class="text-lg font-semibold">{{ project.name }}</h1>
        <OpenWithMenu :project="project" />
      </div>
      <p class="text-sm text-muted-foreground">{{ project.path }}</p>
      <p v-if="project.description" class="mt-1 text-sm">{{ project.description }}</p>
    </header>
    <div class="flex flex-col gap-4 p-6">
      <GitStatusCard :project="project" />
    </div>
  </div>
  <div
    v-else
    class="flex h-full items-center justify-center text-sm text-muted-foreground"
  >
    选择左侧项目查看详情,或点击「添加」创建新项目
  </div>
</template>
