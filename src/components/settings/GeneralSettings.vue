<script setup lang="ts">
import { Separator } from "@/components/ui/separator";
import ThemeSettings from "@/components/settings/ThemeSettings.vue";
import LanguageSettings from "@/components/settings/LanguageSettings.vue";
import OpenWithSettings from "@/components/settings/OpenWithSettings.vue";
</script>

<template>
  <div class="flex flex-col gap-6">
    <ThemeSettings />
    <Separator />
    <OpenWithSettings />
    <Separator />
    <LanguageSettings />
  </div>
</template>
