<script setup lang="ts">
import { Pencil, Play, Trash2 } from "@lucide/vue";
import { Button } from "@/components/ui/button";

defineProps<{
  name: string;
  command: string;
  description?: string;
  editable?: boolean;
}>();

const emit = defineEmits<{
  run: [];
  edit: [];
  delete: [];
}>();
</script>

<template>
  <div class="group flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-accent">
    <Button
      variant="ghost"
      size="icon"
      class="h-7 w-7 shrink-0 text-emerald-600"
      :title="`在终端运行: ${command}`"
      @click="emit('run')"
    >
      <Play class="h-3.5 w-3.5" />
    </Button>
    <span class="w-32 shrink-0 truncate text-sm font-medium" :title="description || name">
      {{ name }}
    </span>
    <span class="min-w-0 flex-1 truncate font-mono text-xs text-muted-foreground" :title="command">
      {{ command }}
    </span>
    <template v-if="editable">
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7 shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
        title="编辑"
        @click="emit('edit')"
      >
        <Pencil class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7 shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
        title="删除"
        @click="emit('delete')"
      >
        <Trash2 class="h-3.5 w-3.5" />
      </Button>
    </template>
  </div>
</template>

