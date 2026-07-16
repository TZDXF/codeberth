<script setup lang="ts">
import { onMounted } from "vue";
import { Toaster } from "@/components/ui/sonner";
import Sidebar from "@/components/layout/Sidebar.vue";
import { onListen } from "@/lib/tauri";
import { useProjectsStore } from "@/stores/projects";
import type { GitUpdatedPayload } from "@/types";

const store = useProjectsStore();

onMounted(() => {
  store.fetchProjects();
  onListen<GitUpdatedPayload>("git://updated", (payload) => {
    store.updateGitRemote(payload.project_id, payload);
  });
});
</script>

<template>
  <div class="flex h-screen overflow-hidden bg-background text-foreground">
    <Sidebar />
    <main class="min-w-0 flex-1 overflow-hidden">
      <router-view />
    </main>
    <Toaster />
  </div>
</template>
