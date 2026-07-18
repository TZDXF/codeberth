<script setup lang="ts">
import { onMounted } from "vue";
import { Toaster } from "@/components/ui/sonner";
import { onListen } from "@/lib/tauri";
import { useProjectsStore } from "@/stores/projects";
import { useTagsStore } from "@/stores/tags";
import type { GitUpdatedPayload } from "@/types";

const store = useProjectsStore();
const tagsStore = useTagsStore();

onMounted(() => {
  store.fetchProjects();
  tagsStore.fetchTags();
  onListen<GitUpdatedPayload>("git://updated", (payload) => {
    store.updateGitRemote(payload.project_id, payload);
  });
});
</script>

<template>
  <main class="h-screen overflow-hidden bg-background text-foreground">
    <router-view />
  </main>
  <Toaster position="bottom-right" />
</template>
