<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { FolderOpen, RotateCcw } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import {
  DEFAULT_COMMIT_PROMPT,
  DEFAULT_REPORT_PROMPT,
  loadAiPrompts,
  openPromptsDir,
  saveAiPrompts,
} from "@/lib/ai-prompts";

const { t } = useI18n();

// 本地副本,显式保存后才写入 ~/.pm/prompts/*.md;空串 = 使用内置默认模板
const commitPrompt = ref("");
const reportPrompt = ref("");

onMounted(async () => {
  try {
    const prompts = await loadAiPrompts();
    commitPrompt.value = prompts.commit;
    reportPrompt.value = prompts.report;
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    toast.error(t("settings.prompts.loadFailed", { error: message }));
  }
});

async function save() {
  try {
    await saveAiPrompts({ commit: commitPrompt.value, report: reportPrompt.value });
    toast.success(t("settings.prompts.saved"));
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    toast.error(t("settings.prompts.saveFailed", { error: message }));
  }
}

async function openDir() {
  try {
    await openPromptsDir();
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    toast.error(t("settings.prompts.openDirFailed", { error: message }));
  }
}
</script>

<template>
  <section>
    <div class="flex items-start justify-between gap-4">
      <div>
        <h2 class="text-base font-semibold">{{ t("settings.prompts.title") }}</h2>
        <p class="mt-1 text-sm text-muted-foreground">{{ t("settings.prompts.description") }}</p>
      </div>
      <Button size="sm" variant="outline" class="shrink-0 gap-1.5" @click="openDir">
        <FolderOpen class="h-3.5 w-3.5" />
        {{ t("settings.prompts.openDir") }}
      </Button>
    </div>

    <div class="mt-6 flex flex-col gap-6">
      <div class="flex flex-col gap-1.5">
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium" for="prompt-commit">
            {{ t("settings.prompts.commit") }}
          </label>
          <Button
            size="sm"
            variant="ghost"
            class="h-7 gap-1 px-2 text-xs text-muted-foreground"
            :disabled="!commitPrompt"
            @click="commitPrompt = ''"
          >
            <RotateCcw class="h-3 w-3" />
            {{ t("settings.prompts.reset") }}
          </Button>
        </div>
        <p class="text-xs text-muted-foreground">{{ t("settings.prompts.commitDescription") }}</p>
        <Textarea
          id="prompt-commit"
          v-model="commitPrompt"
          :placeholder="DEFAULT_COMMIT_PROMPT"
          rows="10"
          spellcheck="false"
          class="font-mono text-xs"
        />
      </div>

      <Separator />

      <div class="flex flex-col gap-1.5">
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium" for="prompt-report">
            {{ t("settings.prompts.report") }}
          </label>
          <Button
            size="sm"
            variant="ghost"
            class="h-7 gap-1 px-2 text-xs text-muted-foreground"
            :disabled="!reportPrompt"
            @click="reportPrompt = ''"
          >
            <RotateCcw class="h-3 w-3" />
            {{ t("settings.prompts.reset") }}
          </Button>
        </div>
        <p class="text-xs text-muted-foreground">{{ t("settings.prompts.reportDescription") }}</p>
        <Textarea
          id="prompt-report"
          v-model="reportPrompt"
          :placeholder="DEFAULT_REPORT_PROMPT"
          rows="10"
          spellcheck="false"
          class="font-mono text-xs"
        />
      </div>

      <p class="text-xs text-muted-foreground">{{ t("settings.prompts.note") }}</p>

      <div>
        <Button size="sm" @click="save">{{ t("common.save") }}</Button>
      </div>
    </div>
  </section>
</template>
