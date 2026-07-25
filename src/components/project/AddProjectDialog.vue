<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "vue-sonner";
import { FolderOpen, FolderGit2, Loader2 } from "@lucide/vue";
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
const mode = ref<"local" | "clone">("local");

// 本地目录模式
const path = ref("");
const name = ref("");
const submitting = ref(false);

// 克隆仓库模式
const url = ref("");
const parentDir = ref("");
const dirName = ref("");
const cloneName = ref("");
const dirNameTouched = ref(false);
const cloneNameTouched = ref(false);
const cloning = ref(false);
const cancelling = ref(false);
let cloneJobId = "";

/** 从仓库 URL 推导目录名:取末段并去掉 .git 后缀 */
function dirNameFromUrl(raw: string): string {
  const segment =
    raw
      .trim()
      .replace(/[\\/]+$/, "")
      .split(/[\\/]/)
      .pop() ?? "";
  return segment.replace(/\.git$/i, "");
}

watch(url, (value) => {
  const derived = dirNameFromUrl(value);
  if (!dirNameTouched.value) dirName.value = derived;
});

watch(dirName, (value) => {
  if (!cloneNameTouched.value) cloneName.value = value;
});

/** 存放位置与目录名拼出的完整目标路径(分隔符跟随存放位置的写法) */
const targetPath = computed(() => {
  if (!parentDir.value || !dirName.value.trim()) return "";
  const sep = parentDir.value.includes("\\") ? "\\" : "/";
  return parentDir.value.replace(/[\\/]+$/, "") + sep + dirName.value.trim();
});

const cloneReady = computed(
  () => url.value.trim() && parentDir.value && dirName.value.trim() && cloneName.value.trim(),
);

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

async function pickParentDir() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: t("projects.add.locationDialogTitle"),
  });
  if (typeof selected === "string") {
    parentDir.value = selected;
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

async function submitClone() {
  if (!cloneReady.value || cloning.value) return;
  cloning.value = true;
  cancelling.value = false;
  cloneJobId = crypto.randomUUID();
  try {
    const clonedPath = await store.cloneProject(url.value.trim(), targetPath.value, cloneJobId);
    const project = await store.addProject(clonedPath, cloneName.value.trim());
    toast.success(t("projects.add.cloned", { name: project.name }));
    visible.value = false;
    url.value = "";
    parentDir.value = "";
    dirName.value = "";
    cloneName.value = "";
    dirNameTouched.value = false;
    cloneNameTouched.value = false;
    router.push(`/projects/${project.id}`);
  } catch (e) {
    // 用户主动取消:静默复位,不弹错误
    if (!cancelling.value) toast.error(String(e));
  } finally {
    cloning.value = false;
    cloneJobId = "";
  }
}

function cancelClone() {
  if (!cloning.value || cancelling.value) return;
  cancelling.value = true;
  store.cancelClone(cloneJobId).catch(() => {});
}

// 克隆过程中关闭弹窗(ESC/点 X/点遮罩)视为取消克隆
watch(visible, (open_) => {
  if (!open_ && cloning.value) cancelClone();
});
</script>

<template>
  <Dialog v-model:open="visible">
    <DialogTrigger as-child>
      <slot />
    </DialogTrigger>
    <DialogContent>
      <DialogHeader>
        <DialogTitle>{{ t("projects.add.title") }}</DialogTitle>
        <DialogDescription>
          {{
            mode === "local" ? t("projects.add.description") : t("projects.add.cloneDescription")
          }}
        </DialogDescription>
      </DialogHeader>

      <div class="flex gap-1 rounded-md border p-1">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          class="h-7 flex-1 gap-1.5"
          :class="mode === 'local' && 'bg-accent'"
          :disabled="cloning"
          @click="mode = 'local'"
        >
          <FolderOpen class="h-3.5 w-3.5" />
          {{ t("projects.add.modeLocal") }}
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          class="h-7 flex-1 gap-1.5"
          :class="mode === 'clone' && 'bg-accent'"
          :disabled="cloning"
          @click="mode = 'clone'"
        >
          <FolderGit2 class="h-3.5 w-3.5" />
          {{ t("projects.add.modeClone") }}
        </Button>
      </div>

      <form v-if="mode === 'local'" class="flex flex-col gap-4" @submit.prevent="submit">
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

      <form v-else class="flex flex-col gap-4" @submit.prevent="submitClone">
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("projects.add.urlLabel") }}</label>
          <Input
            v-model="url"
            :placeholder="t('projects.add.urlPlaceholder')"
            :disabled="cloning"
            autofocus
          />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("projects.add.locationLabel") }}</label>
          <div class="flex gap-2">
            <Input
              v-model="parentDir"
              :placeholder="t('projects.add.locationPlaceholder')"
              readonly
              class="flex-1"
            />
            <Button type="button" variant="outline" :disabled="cloning" @click="pickParentDir">
              <FolderOpen class="h-4 w-4" />
              {{ t("projects.add.browse") }}
            </Button>
          </div>
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("projects.add.dirNameLabel") }}</label>
          <Input
            v-model="dirName"
            :placeholder="t('projects.add.dirNamePlaceholder')"
            :disabled="cloning"
            @input="dirNameTouched = true"
          />
          <p v-if="targetPath" class="text-xs text-muted-foreground break-all">{{ targetPath }}</p>
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("projects.add.nameLabel") }}</label>
          <Input
            v-model="cloneName"
            :placeholder="t('projects.add.namePlaceholder')"
            :disabled="cloning"
            @input="cloneNameTouched = true"
          />
        </div>
        <DialogFooter>
          <Button v-if="cloning" type="button" variant="outline" @click="cancelClone">
            {{ t("common.cancel") }}
          </Button>
          <Button type="submit" :disabled="!cloneReady || cloning">
            <Loader2 v-if="cloning" class="h-4 w-4 animate-spin" />
            {{ cloning ? t("projects.add.cloning") : t("projects.add.cloneAndAdd") }}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
