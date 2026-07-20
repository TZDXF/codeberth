<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
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

const { t } = useI18n();
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
    title: t("projects.add.dialogTitle"),
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
    toast.success(t("projects.add.added", { name: project.name }));
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
        <DialogTitle>{{ t("projects.add.title") }}</DialogTitle>
        <DialogDescription>{{ t("projects.add.description") }}</DialogDescription>
      </DialogHeader>
      <form class="flex flex-col gap-4" @submit.prevent="submit">
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("projects.add.pathLabel") }}</label>
          <div class="flex gap-2">
            <Input
              v-model="path"
              :placeholder="t('projects.add.pathPlaceholder')"
              readonly
              class="flex-1"
            />
            <Button type="button" variant="outline" @click="pickFolder">
              <FolderOpen class="h-4 w-4" />
              {{ t("projects.add.browse") }}
            </Button>
          </div>
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("projects.add.nameLabel") }}</label>
          <Input v-model="name" :placeholder="t('projects.add.namePlaceholder')" autofocus />
        </div>
        <DialogFooter>
          <Button type="submit" :disabled="!path || !name.trim() || submitting">
            {{ submitting ? t("common.adding") : t("common.add") }}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
