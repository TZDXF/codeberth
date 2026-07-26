<script setup lang="ts">
import { Separator } from "@/components/ui/separator";
import ThemeSettings from "@/components/settings/ThemeSettings.vue";
import MdThemeSettings from "@/components/settings/MdThemeSettings.vue";
import LanguageSettings from "@/components/settings/LanguageSettings.vue";
import OpenWithSettings from "@/components/settings/OpenWithSettings.vue";
import UpdateSettings from "@/components/settings/UpdateSettings.vue";
</script>

<template>
  <div class="flex flex-col gap-6">
    <ThemeSettings />
    <Separator />
    <MdThemeSettings />
    <Separator />
    <OpenWithSettings />
    <Separator />
    <LanguageSettings />
    <Separator />
    <UpdateSettings />
  </div>
</template>
