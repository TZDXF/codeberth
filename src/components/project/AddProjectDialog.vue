<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "vue-sonner";
import { FolderOpen } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { useProjectsStore } from "@/stores/projects";

const store = useProjectsStore();
const router = useRouter();

const visible = ref(false);
const path = ref("");
const name = ref("");
const submitting = ref(false);

async function pickFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "选择项目文件夹",
  });
  if (typeof selected === "string") {
    path.value = selected;
    if (!name.value) {
      name.value = selected.split(/[\\/]/).filter(Boolean).pop() ?? "";
    }
  }
}

async function submit() {
  if (!path.value || !name.value.trim() || submitting.value) return;
  submitting.value = true;
  try {
    const project = await store.addProject(path.value, name.value.trim());
    toast.success(`已添加项目「${project.name}」`);
    visible.value = false;
    path.value = "";
    name.value = "";
    router.push(`/projects/${project.id}`);
  } catch (e) {
    toast.error(String(e));
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <Dialog v-model:open="visible">
    <DialogTrigger as-child>
      <slot />
    </DialogTrigger>
    <DialogContent>
      <DialogHeader>
        <DialogTitle>添加项目</DialogTitle>
        <DialogDescription>选择一个文件夹作为项目进行管理</DialogDescription>
      </DialogHeader>
      <form class="flex flex-col gap-4" @submit.prevent="submit">
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">项目路径</label>
          <div class="flex gap-2">
            <Input v-model="path" placeholder="选择文件夹..." readonly class="flex-1" />
            <Button type="button" variant="outline" @click="pickFolder">
              <FolderOpen class="h-4 w-4" />
              浏览
            </Button>
          </div>
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">项目名称</label>
          <Input v-model="name" placeholder="输入项目名称" autofocus />
        </div>
        <DialogFooter>
          <Button type="submit" :disabled="!path || !name.trim() || submitting">
            {{ submitting ? "添加中..." : "添加" }}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>

