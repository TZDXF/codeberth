<script setup lang="ts">
import type { Component } from "vue";
import { Check, Monitor, Moon, Sun } from "@lucide/vue";
import { useSettingsStore, type ThemeMode } from "@/stores/settings";

const store = useSettingsStore();

const OPTIONS: { value: ThemeMode; label: string; description: string; icon: Component }[] = [
  { value: "system", label: "跟随系统", description: "根据系统外观自动切换", icon: Monitor },
  { value: "light", label: "亮色", description: "始终使用亮色主题", icon: Sun },
  { value: "dark", label: "暗色", description: "始终使用暗色主题", icon: Moon },
];
</script>

<template>
  <section>
    <h2 class="text-base font-semibold">主题</h2>
    <p class="mt-1 text-sm text-muted-foreground">选择应用的外观</p>
    <div class="mt-4 flex flex-col gap-2">
      <button
        v-for="opt in OPTIONS"
        :key="opt.value"
        type="button"
        class="flex items-center gap-3 rounded-lg border px-3 py-2.5 text-left transition-colors hover:bg-accent"
        :class="store.theme === opt.value && 'border-primary'"
        @click="store.setTheme(opt.value)"
      >
        <component :is="opt.icon" class="h-4 w-4 shrink-0 text-muted-foreground" />
        <span class="flex-1">
          <span class="block text-sm font-medium">{{ opt.label }}</span>
          <span class="block text-xs text-muted-foreground">{{ opt.description }}</span>
        </span>
        <Check v-if="store.theme === opt.value" class="h-4 w-4 shrink-0 text-primary" />
      </button>
    </div>
  </section>
</template>
