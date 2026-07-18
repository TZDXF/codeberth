<script setup lang="ts">
import { Check } from "@lucide/vue";
import { useSettingsStore, type Language } from "@/stores/settings";

const store = useSettingsStore();

const OPTIONS: { value: Language; label: string; nativeLabel: string }[] = [
  { value: "zh-CN", label: "简体中文", nativeLabel: "简体中文" },
  { value: "en-US", label: "English", nativeLabel: "English (US)" },
];
</script>

<template>
  <section>
    <h2 class="text-base font-semibold">语言</h2>
    <p class="mt-1 text-sm text-muted-foreground">选择界面显示语言</p>
    <div class="mt-4 flex flex-col gap-2">
      <button
        v-for="opt in OPTIONS"
        :key="opt.value"
        type="button"
        class="flex items-center gap-3 rounded-lg border px-3 py-2.5 text-left transition-colors hover:bg-accent"
        :class="store.language === opt.value && 'border-primary'"
        @click="store.setLanguage(opt.value)"
      >
        <span class="flex-1 text-sm font-medium">{{ opt.nativeLabel }}</span>
        <Check v-if="store.language === opt.value" class="h-4 w-4 shrink-0 text-primary" />
      </button>
    </div>
    <p class="mt-3 text-xs text-muted-foreground">
      界面翻译将在后续版本生效,当前仅保存偏好设置。
    </p>
  </section>
</template>
