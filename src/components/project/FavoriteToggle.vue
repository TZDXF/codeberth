<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Star } from "@lucide/vue";
import { useProjectsStore } from "@/stores/projects";
import type { Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project }>();
const store = useProjectsStore();

// 未收藏时仅在父级 group hover 时显示;收藏后实心黄星常驻
async function toggle() {
  try {
    await store.setFavorite(props.project.id, !props.project.favorited_at);
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <button
    type="button"
    class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md transition-opacity hover:bg-accent"
    :class="project.favorited_at ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
    :title="t(project.favorited_at ? 'projects.actions.unfavorite' : 'projects.actions.favorite')"
    @click.stop="toggle"
  >
    <Star
      class="h-3.5 w-3.5"
      :class="project.favorited_at ? 'fill-yellow-400 text-yellow-400' : 'text-muted-foreground'"
    />
  </button>
</template>
