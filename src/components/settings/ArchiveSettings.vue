<script setup lang="ts">
import { onMounted } from "vue";
import { toast } from "vue-sonner";
import { ArchiveRestore, Trash2 } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { formatRelativeTime } from "@/lib/format";
import { useProjectsStore } from "@/stores/projects";

const store = useProjectsStore();

onMounted(() => store.fetchArchivedProjects());

async function restore(id: number, name: string) {
  try {
    await store.unarchiveProject(id);
    toast.success(`已恢复项目「${name}」`);
  } catch (e) {
    toast.error(String(e));
  }
}

async function remove(id: number, name: string) {
  if (
    !window.confirm(
      `确定彻底删除项目「${name}」吗?此操作不可恢复,标签指派、自定义命令等历史数据将一并删除。(不会删除磁盘文件)`,
    )
  )
    return;
  try {
    await store.deleteProject(id);
    toast.success(`已删除项目「${name}」`);
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <section>
    <h2 class="text-base font-semibold">归档项目</h2>
    <p class="mt-1 text-sm text-muted-foreground">
      已归档的项目保留历史数据,可恢复到项目列表或彻底删除
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
              {{ p.path }} · 归档于 {{ formatRelativeTime(p.archived_at) }}
            </p>
          </div>
          <span
            class="flex shrink-0 items-center opacity-0 transition-opacity group-hover:opacity-100"
          >
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7"
              title="恢复项目"
              @click="restore(p.id, p.name)"
            >
              <ArchiveRestore class="h-3.5 w-3.5" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 text-destructive hover:text-destructive"
              title="彻底删除"
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
          没有已归档的项目
        </p>
      </div>
    </ScrollArea>
  </section>
</template>
