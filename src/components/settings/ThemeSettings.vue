<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { Component } from "vue";
import { Check, Monitor, Moon, Sun } from "@lucide/vue";
import { useSettingsStore, type ThemeMode } from "@/stores/settings";

const { t } = useI18n();
const store = useSettingsStore();

const OPTIONS: { value: ThemeMode; labelKey: string; descriptionKey: string; icon: Component }[] = [
  { value: "system", labelKey: "settings.theme.system", descriptionKey: "settings.theme.systemDesc", icon: Monitor },
  { value: "light", labelKey: "settings.theme.light", descriptionKey: "settings.theme.lightDesc", icon: Sun },
  { value: "dark", labelKey: "settings.theme.dark", descriptionKey: "settings.theme.darkDesc", icon: Moon },
];
</script>

<template>
  <section>
    <h2 class="text-base font-semibold">{{ t("settings.general.theme") }}</h2>
    <p class="mt-1 text-sm text-muted-foreground">{{ t("settings.general.themeDescription") }}</p>
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
          <span class="block text-sm font-medium">{{ t(opt.labelKey) }}</span>
          <span class="block text-xs text-muted-foreground">{{ t(opt.descriptionKey) }}</span>
        </span>
        <Check v-if="store.theme === opt.value" class="h-4 w-4 shrink-0 text-primary" />
      </button>
    </div>
  </section>
</template>