<script setup lang="ts">
import { computed, ref, type Component } from "vue";
import { useRouter } from "vue-router";
import { ArrowLeft, Languages, Palette, Tags } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import ThemeSettings from "@/components/settings/ThemeSettings.vue";
import LanguageSettings from "@/components/settings/LanguageSettings.vue";
import TagSettings from "@/components/settings/TagSettings.vue";

interface Category {
  id: string;
  label: string;
  icon: Component;
  component: Component;
}

// 归档功能就绪后追加: { id: "archive", label: "归档项目", icon: Archive, component: ArchiveSettings }
const categories: Category[] = [
  { id: "theme", label: "主题", icon: Palette, component: ThemeSettings },
  { id: "language", label: "语言", icon: Languages, component: LanguageSettings },
  { id: "tags", label: "标签管理", icon: Tags, component: TagSettings },
];

const router = useRouter();
const activeId = ref(categories[0].id);
const active = computed(() => categories.find((c) => c.id === activeId.value) ?? categories[0]);
</script>

<template>
  <div class="flex h-full flex-col">
    <header class="flex shrink-0 items-center gap-2 border-b px-4 py-2.5">
      <Button
        variant="ghost"
        size="icon"
        class="h-8 w-8"
        title="返回项目列表"
        @click="router.push('/')"
      >
        <ArrowLeft class="h-4 w-4" />
      </Button>
      <h1 class="text-sm font-semibold">设置</h1>
    </header>

    <div class="flex flex-1 overflow-hidden">
      <nav class="w-44 shrink-0 border-r p-2">
        <button
          v-for="c in categories"
          :key="c.id"
          type="button"
          class="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
          :class="activeId === c.id && 'bg-accent font-medium text-foreground'"
          @click="activeId = c.id"
        >
          <component :is="c.icon" class="h-3.5 w-3.5" />
          {{ c.label }}
        </button>
      </nav>

      <ScrollArea class="flex-1">
        <div class="max-w-xl p-6">
          <component :is="active.component" />
        </div>
      </ScrollArea>
    </div>
  </div>
</template>
