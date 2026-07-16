<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import AddProjectDialog from "@/components/project/AddProjectDialog.vue";
import ProjectCard from "@/components/project/ProjectCard.vue";
import { useProjectsStore } from "@/stores/projects";

const store = useProjectsStore();
</script>

<template>
  <aside class="flex w-72 shrink-0 flex-col border-r bg-muted/30">
    <div class="flex h-12 shrink-0 items-center justify-between border-b px-3">
      <span class="text-sm font-semibold">项目</span>
      <AddProjectDialog>
        <Button size="sm" variant="outline">
          <Plus class="h-4 w-4" />
          添加
        </Button>
      </AddProjectDialog>
    </div>
    <ScrollArea class="flex-1">
      <div class="flex flex-col gap-2 p-3">
        <ProjectCard v-for="p in store.projects" :key="p.id" :project="p" />
        <p
          v-if="!store.projects.length"
          class="py-10 text-center text-xs text-muted-foreground"
        >
          还没有项目,点击右上角「添加」
        </p>
      </div>
    </ScrollArea>
  </aside>
</template>

