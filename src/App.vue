<script setup lang="ts">
import { onMounted } from "vue";
import { Toaster } from "@/components/ui/sonner";
import TitleBar from "@/components/TitleBar.vue";
import { onListen } from "@/lib/tauri";
import { useProjectsStore } from "@/stores/projects";
import { useSettingsStore } from "@/stores/settings";
import { useTagsStore } from "@/stores/tags";
import type { GitUpdatedPayload } from "@/types";

const store = useProjectsStore();
const tagsStore = useTagsStore();
const settingsStore = useSettingsStore();

onMounted(() => {
  settingsStore.init();
  store.fetchProjects();
  tagsStore.fetchTags();
  onListen<GitUpdatedPayload>("git://updated", (payload) => {
    store.updateGitRemote(payload.project_id, payload);
  });
});
</script>

<template>
  <main class="flex h-screen flex-col overflow-hidden bg-background text-foreground">
    <TitleBar />
    <div class="min-h-0 flex-1">
      <router-view />
    </div>
  </main>
  <Toaster position="bottom-right" />
</template>
