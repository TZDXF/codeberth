<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Loader2, Plug } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { testAiConnection } from "@/lib/ai";
import { useSettingsStore } from "@/stores/settings";

const { t } = useI18n();
const store = useSettingsStore();

// 本地副本,显式保存后再持久化(API Key 类输入不适合边敲边存)
const baseUrl = ref(store.aiBaseUrl);
const apiKey = ref(store.aiApiKey);
const model = ref(store.aiModel);
const testing = ref(false);

async function save() {
  await store.setAiBaseUrl(baseUrl.value);
  await store.setAiApiKey(apiKey.value);
  await store.setAiModel(model.value);
  toast.success(t("settings.ai.saved"));
}

async function testConnection() {
  if (testing.value) return;
  testing.value = true;
  try {
    // 先落库当前表单值,测试使用的就是界面上看到的配置
    await store.setAiBaseUrl(baseUrl.value);
    await store.setAiApiKey(apiKey.value);
    await store.setAiModel(model.value);
    await testAiConnection();
    toast.success(t("settings.ai.testSuccess"));
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    toast.error(t("settings.ai.testFailed", { error: message }));
  } finally {
    testing.value = false;
  }
}
</script>

<template>
  <section>
    <h2 class="text-base font-semibold">{{ t("settings.ai.title") }}</h2>
    <p class="mt-1 text-sm text-muted-foreground">{{ t("settings.ai.description") }}</p>

    <div class="mt-4 flex flex-col gap-4">
      <div class="flex flex-col gap-1.5">
        <label class="text-sm font-medium" for="ai-base-url">{{ t("settings.ai.baseUrl") }}</label>
        <Input
          id="ai-base-url"
          v-model="baseUrl"
          :placeholder="t('settings.ai.baseUrlPlaceholder')"
          spellcheck="false"
        />
      </div>
      <div class="flex flex-col gap-1.5">
        <label class="text-sm font-medium" for="ai-api-key">{{ t("settings.ai.apiKey") }}</label>
        <Input
          id="ai-api-key"
          v-model="apiKey"
          type="password"
          :placeholder="t('settings.ai.apiKeyPlaceholder')"
          autocomplete="off"
          spellcheck="false"
        />
        <p class="text-xs text-muted-foreground">{{ t("settings.ai.apiKeyNote") }}</p>
      </div>
      <div class="flex flex-col gap-1.5">
        <label class="text-sm font-medium" for="ai-model">{{ t("settings.ai.model") }}</label>
        <Input
          id="ai-model"
          v-model="model"
          :placeholder="t('settings.ai.modelPlaceholder')"
          spellcheck="false"
        />
      </div>
      <div class="flex items-center gap-2">
        <Button size="sm" @click="save">{{ t("common.save") }}</Button>
        <Button
          size="sm"
          variant="outline"
          class="gap-1.5"
          :disabled="testing || !apiKey.trim()"
          @click="testConnection"
        >
          <Loader2 v-if="testing" class="h-3.5 w-3.5 animate-spin" />
          <Plug v-else class="h-3.5 w-3.5" />
          {{ testing ? t("settings.ai.testing") : t("settings.ai.test") }}
        </Button>
      </div>
    </div>
  </section>
</template>
