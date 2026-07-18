<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { toast } from "vue-sonner";
import { ArrowLeft, Pencil } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import GitStatusBar from "@/components/git/GitStatusBar.vue";
import OpenWithMenu from "@/components/open/OpenWithMenu.vue";
import DockerCompose from "@/components/project/DockerCompose.vue";
import ReadmePreview from "@/components/project/ReadmePreview.vue";
import CustomCommands from "@/components/scripts/CustomCommands.vue";
import PackageScripts from "@/components/scripts/PackageScripts.vue";
import TagPicker from "@/components/tags/TagPicker.vue";
import { useProjectsStore } from "@/stores/projects";

const route = useRoute();
const router = useRouter();
const store = useProjectsStore();

const project = computed(() => {
  const id = Number(route.params.id);
  return Number.isFinite(id) ? store.projects.find((p) => p.id === id) : undefined;
});

// 选中项目进入详情页时优先触发一次远端 fetch
watch(
  () => project.value?.id,
  () => {
    if (project.value) store.triggerRemoteFetch(project.value);
  },
  { immediate: true },
);

// 切换项目时退出编辑态
watch(
  () => project.value?.id,
  () => {
    editingName.value = false;
    editingDesc.value = false;
  },
);

// --- 名称内联编辑 ---
const editingName = ref(false);
const draftName = ref("");
const nameInput = ref<HTMLInputElement | null>(null);

function startEditName() {
  if (!project.value) return;
  draftName.value = project.value.name;
  editingName.value = true;
  nextTick(() => nameInput.value?.select());
}

async function saveName() {
  if (!editingName.value || !project.value) return;
  editingName.value = false;
  const name = draftName.value.trim();
  if (!name || name === project.value.name) return;
  try {
    await store.updateProject(project.value.id, name, project.value.description);
    toast.success("项目信息已保存");
  } catch (e) {
    toast.error(String(e));
  }
}

// --- 描述内联编辑 ---
const editingDesc = ref(false);
const draftDesc = ref("");
const descInput = ref<HTMLTextAreaElement | null>(null);

function startEditDesc() {
  if (!project.value) return;
  draftDesc.value = project.value.description;
  editingDesc.value = true;
  nextTick(() => descInput.value?.focus());
}

async function saveDesc() {
  if (!editingDesc.value || !project.value) return;
  editingDesc.value = false;
  const description = draftDesc.value.trim();
  if (description === project.value.description) return;
  try {
    await store.updateProject(project.value.id, project.value.name, description);
    toast.success("项目信息已保存");
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <div v-if="project" class="flex h-full flex-col overflow-y-auto">
    <header class="shrink-0 border-b px-6 py-4">
      <div class="flex items-start justify-between gap-4">
        <div class="flex min-w-0 items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8 shrink-0"
            title="返回项目列表"
            @click="router.push('/')"
          >
            <ArrowLeft class="h-4 w-4" />
          </Button>
          <input
            v-if="editingName"
            ref="nameInput"
            v-model="draftName"
            class="h-8 w-72 max-w-full rounded-md border border-input bg-transparent px-2 text-lg font-semibold outline-none focus-visible:ring-1 focus-visible:ring-ring"
            @keydown.enter.prevent="saveName"
            @keydown.esc="editingName = false"
            @blur="saveName"
          />
          <h1
            v-else
            class="group flex min-w-0 cursor-pointer items-center gap-1.5 text-lg font-semibold"
            title="点击编辑名称"
            @click="startEditName"
          >
            <span class="truncate">{{ project.name }}</span>
            <Pencil
              class="h-3.5 w-3.5 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100"
            />
          </h1>
        </div>
        <OpenWithMenu :project="project" class="shrink-0" />
      </div>

      <p class="mt-1 truncate pl-10 text-sm text-muted-foreground" :title="project.path">
        {{ project.path }}
      </p>

      <div class="mt-1 pl-10">
        <textarea
          v-if="editingDesc"
          ref="descInput"
          v-model="draftDesc"
          rows="2"
          placeholder="项目描述(可选)"
          class="w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus-visible:ring-1 focus-visible:ring-ring"
          @keydown.esc="editingDesc = false"
          @keydown.enter.ctrl.prevent="saveDesc"
          @blur="saveDesc"
        />
        <p
          v-else
          class="group flex w-fit cursor-pointer items-center gap-1.5 text-sm"
          :class="project.description ? '' : 'text-muted-foreground'"
          title="点击编辑描述"
          @click="startEditDesc"
        >
          {{ project.description || "添加描述..." }}
          <Pencil
            class="h-3 w-3 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100"
          />
        </p>
      </div>

      <div class="mt-2.5 flex flex-wrap items-center gap-x-6 gap-y-2 pl-10">
        <TagPicker :project="project" />
        <GitStatusBar :project="project" />
      </div>
    </header>

    <div
      class="grid items-start gap-4 p-6 [grid-template-columns:repeat(auto-fill,minmax(360px,1fr))]"
    >
      <PackageScripts :project="project" />
      <DockerCompose :project="project" />
      <CustomCommands :project="project" />
      <ReadmePreview :project="project" />
    </div>
  </div>

  <div
    v-else
    class="flex h-full flex-col items-center justify-center gap-3 text-sm text-muted-foreground"
  >
    <p>项目不存在或已被删除</p>
    <Button variant="outline" size="sm" @click="router.push('/')">返回项目列表</Button>
  </div>
</template>
