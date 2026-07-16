<script setup lang="ts">
import { Button } from "@/components/ui/button";
</script>

<template>
  <main class="flex h-screen flex-col bg-background text-foreground">
    <header class="flex h-12 items-center border-b px-4">
      <h1 class="text-sm font-semibold">ProjectDev</h1>
    </header>
    <div class="flex flex-1 flex-col items-center justify-center gap-4">
      <p class="text-sm text-muted-foreground">项目管理客户端</p>
      <Button>开始使用</Button>
    </div>
  </main>
</template>

