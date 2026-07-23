<script setup lang="ts">
import { Check } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { OPEN_WITH_OPTIONS } from "@/lib/open-with";
import { useSettingsStore } from "@/stores/settings";

const { t } = useI18n();
const store = useSettingsStore();
</script>

<template>
  <section>
    <h2 class="text-base font-semibold">{{ t("settings.general.openWith") }}</h2>
    <p class="mt-1 text-sm text-muted-foreground">
      {{ t("settings.general.openWithDescription") }}
    </p>
    <div class="mt-4 flex flex-col gap-2">
      <button
        v-for="opt in OPEN_WITH_OPTIONS"
        :key="opt.kind"
        type="button"
        class="flex items-center gap-3 rounded-lg border px-3 py-2.5 text-left transition-colors hover:bg-accent"
        :class="store.defaultOpenWith === opt.kind && 'border-primary'"
        @click="store.setDefaultOpenWith(opt.kind)"
      >
        <component :is="opt.icon" class="h-4 w-4 shrink-0 text-muted-foreground" />
        <span class="flex-1">
          <span class="block text-sm font-medium">{{ t(opt.labelKey) }}</span>
          <span class="mt-0.5 block text-xs text-muted-foreground">{{ t(opt.descKey) }}</span>
        </span>
        <Check v-if="store.defaultOpenWith === opt.kind" class="h-4 w-4 shrink-0 text-primary" />
      </button>
    </div>
  </section>
</template>
