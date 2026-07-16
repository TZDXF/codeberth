<script setup lang="ts">
import { onMounted } from "vue";
import { Toaster } from "@/components/ui/sonner";
import Sidebar from "@/components/layout/Sidebar.vue";
import { useProjectsStore } from "@/stores/projects";

const store = useProjectsStore();

onMounted(() => store.fetchProjects());
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

