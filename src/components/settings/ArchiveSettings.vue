<script setup lang="ts">
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { ArchiveRestore, Trash2 } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { formatRelativeTime } from "@/lib/format";
import { useProjectsStore } from "@/stores/projects";

const { t } = useI18n();
const store = useProjectsStore();

onMounted(() => store.fetchArchivedProjects());

async function restore(id: number, name: string) {
  try {
    await store.unarchiveProject(id);
    toast.success(t("settings.archive.restored", { name }));
  } catch (e) {
    toast.error(String(e));
  }
}

async function remove(id: number, name: string) {
  if (!window.confirm(t("settings.archive.deleteConfirm", { name }))) return;
  try {
    await store.deleteProject(id);
    toast.success(t("settings.archive.deleted", { name }));
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <section>
    <h2 class="text-base font-semibold">{{ t("settings.archive.title") }}</h2>
    <p class="mt-1 text-sm text-muted-foreground">
      {{ t("settings.archive.description") }}
    </p>

    <ScrollArea class="mt-4 max-h-96">
      <div class="flex flex-col gap-1">
        <div
          v-for="p in store.archivedProjects"
          :key="p.id"
          class="group flex items-center justify-between gap-3 rounded-md px-2 py-1.5 hover:bg-accent"
        >
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-medium">{{ p.name }}</p>
            <p class="truncate text-xs text-muted-foreground" :title="p.path">
              {{ p.path }} · {{ t("settings.archive.archivedAt", { time: formatRelativeTime(p.archived_at) }) }}
            </p>
          </div>
          <span
            class="flex shrink-0 items-center opacity-0 transition-opacity group-hover:opacity-100"
          >
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7"
              :title="t('settings.archive.restore')"
              @click="restore(p.id, p.name)"
            >
              <ArchiveRestore class="h-3.5 w-3.5" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 text-destructive hover:text-destructive"
              :title="t('settings.archive.permanentDelete')"
              @click="remove(p.id, p.name)"
            >
              <Trash2 class="h-3.5 w-3.5" />
            </Button>
          </span>
        </div>
        <p
          v-if="!store.archivedProjects.length"
          class="py-6 text-center text-xs text-muted-foreground"
        >
          {{ t("settings.archive.empty") }}
        </p>
      </div>
    </ScrollArea>
  </section>
</template>