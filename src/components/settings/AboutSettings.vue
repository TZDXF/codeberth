<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Separator } from "@/components/ui/separator";
import UpdateSettings from "@/components/settings/UpdateSettings.vue";
import { useUpdateStore } from "@/stores/update";

const { t } = useI18n();
const updateStore = useUpdateStore();
</script>

<template>
  <div class="flex flex-col gap-6">
    <section>
      <h2 class="text-base font-semibold">{{ t("settings.about.title") }}</h2>
      <div class="mt-4 flex items-center gap-3">
        <div class="flex flex-col gap-0.5">
          <span class="text-base font-medium">{{ t("app.title") }}</span>
          <span class="text-xs text-muted-foreground">{{ t("app.name") }}</span>
        </div>
        <span class="rounded-md bg-muted px-2 py-0.5 font-mono text-xs">
          v{{ updateStore.currentVersion || "-" }}
        </span>
      </div>
      <p class="mt-1 text-sm text-muted-foreground">
        {{ t("settings.about.description") }}
      </p>
    </section>
    <Separator />
    <UpdateSettings />
  </div>
</template>
