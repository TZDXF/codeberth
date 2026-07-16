<script setup lang="ts">
import { ref, watch } from "vue";
import { toast } from "vue-sonner";
import { Save } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import TagPicker from "@/components/tags/TagPicker.vue";
import { useProjectsStore } from "@/stores/projects";
import type { Project } from "@/types";

const props = defineProps<{ project: Project }>();

const store = useProjectsStore();

const name = ref(props.project.name);
const description = ref(props.project.description);
const saving = ref(false);

watch(
  () => props.project.id,
  () => {
    name.value = props.project.name;
    description.value = props.project.description;
  },
);

async function save() {
  if (!name.value.trim() || saving.value) return;
  saving.value = true;
  try {
    await store.updateProject(props.project.id, name.value.trim(), description.value.trim());
    toast.success("项目信息已保存");
  } catch (e) {
    toast.error(String(e));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Card>
    <CardHeader class="pb-3">
      <CardTitle class="text-sm font-semibold">项目信息</CardTitle>
    </CardHeader>
    <CardContent>
      <form class="flex flex-col gap-3" @submit.prevent="save">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground">名称</label>
          <Input v-model="name" placeholder="项目名称" />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground">描述</label>
          <textarea
            v-model="description"
            rows="2"
            placeholder="项目描述(可选)"
            class="flex w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-foreground">标签</label>
          <TagPicker :project="project" />
        </div>
        <div class="flex justify-end">
          <Button type="submit" size="sm" :disabled="!name.trim() || saving">
            <Save class="h-4 w-4" />
            {{ saving ? "保存中..." : "保存" }}
          </Button>
        </div>
      </form>
    </CardContent>
  </Card>
</template>

