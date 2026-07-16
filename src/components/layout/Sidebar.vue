<script setup lang="ts">
import { ref, watch } from "vue";
import { Plus, Search, Settings2 } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import AddProjectDialog from "@/components/project/AddProjectDialog.vue";
import ProjectCard from "@/components/project/ProjectCard.vue";
import TagManager from "@/components/tags/TagManager.vue";
import { useProjectsStore } from "@/stores/projects";
import { useTagsStore } from "@/stores/tags";

const store = useProjectsStore();
const tagsStore = useTagsStore();

const searchInput = ref(store.query);
let debounceTimer: number | undefined;

watch(searchInput, (value) => {
  window.clearTimeout(debounceTimer);
  debounceTimer = window.setTimeout(() => store.setQuery(value), 250);
});
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
    <div class="shrink-0 border-b px-3 py-2">
      <div class="relative">
        <Search class="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
        <Input v-model="searchInput" placeholder="搜索项目..." class="h-8 pl-8 text-sm" />
      </div>
      <div v-if="tagsStore.tags.length" class="mt-2 flex flex-wrap items-center gap-1.5">
        <button
          v-for="tag in tagsStore.tags"
          :key="tag.id"
          type="button"
          class="rounded-full border px-2 py-0.5 text-[11px] transition-colors"
          :style="
            store.selectedTagIds.includes(tag.id)
              ? { backgroundColor: tag.color, borderColor: tag.color, color: '#fff' }
              : { borderColor: tag.color + '66', color: tag.color }
          "
          @click="store.toggleTagFilter(tag.id)"
        >
          {{ tag.name }}
        </button>
        <TagManager @refresh-projects="store.fetchProjects()">
          <Button variant="ghost" size="icon" class="h-6 w-6" title="管理标签">
            <Settings2 class="h-3.5 w-3.5" />
          </Button>
        </TagManager>
      </div>
    </div>
    <ScrollArea class="flex-1">
      <div class="flex flex-col gap-2 p-3">
        <ProjectCard v-for="p in store.projects" :key="p.id" :project="p" />
        <p
          v-if="!store.projects.length"
          class="py-10 text-center text-xs text-muted-foreground"
        >
          {{ store.query || store.selectedTagIds.length ? "没有匹配的项目" : "还没有项目,点击右上角「添加」" }}
        </p>
      </div>
    </ScrollArea>
  </aside>
</template>
