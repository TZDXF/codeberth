<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { RefreshCw } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { useSettingsStore } from "@/stores/settings";
import { useUpdateStore } from "@/stores/update";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const updateStore = useUpdateStore();

const autoCheck = computed({
  get: () => settingsStore.autoCheckUpdate,
  set: (v: boolean) => settingsStore.setAutoCheckUpdate(v),
});

function onCheck() {
  if (updateStore.hasUpdate) {
    updateStore.dialogOpen = true;
  } else {
    updateStore.checkForUpdate(true);
  }
}
</script>

<template>
  <section>
    <h2 class="text-base font-semibold">{{ t("settings.update.title") }}</h2>
    <p class="mt-1 text-sm text-muted-foreground">
      {{ t("settings.update.description") }}
    </p>

    <div class="mt-4 flex items-center gap-3">
      <span class="text-sm">{{ t("update.currentVersionLabel") }}</span>
      <span class="rounded-md bg-muted px-2 py-0.5 font-mono text-xs">
        v{{ updateStore.currentVersion || "-" }}
      </span>
      <Button
        variant="outline"
        size="sm"
        :disabled="updateStore.status === 'checking' || updateStore.status === 'downloading'"
        @click="onCheck"
      >
        <RefreshCw v-if="updateStore.status === 'checking'" class="h-3.5 w-3.5 animate-spin" />
        <template v-else>{{ t("update.check") }}</template>
      </Button>
      <span v-if="updateStore.hasUpdate" class="text-sm text-primary">
        {{ t("update.available", { version: updateStore.update?.version ?? "" }) }}
      </span>
    </div>

    <div class="mt-4 flex items-center justify-between rounded-lg border px-3 py-2.5">
      <div class="flex flex-col gap-0.5">
        <span class="text-sm font-medium">{{ t("settings.update.autoCheck") }}</span>
        <span class="text-xs text-muted-foreground">{{ t("settings.update.autoCheckHint") }}</span>
      </div>
      <Switch v-model="autoCheck" />
    </div>
  </section>
</template>
