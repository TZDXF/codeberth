<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { ArrowDownToLine, ArrowUpToLine, GitCommitHorizontal, Loader2 } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import CommitDialog from "@/components/git/CommitDialog.vue";
import ConflictDialog from "@/components/git/ConflictDialog.vue";
import { useProjectsStore } from "@/stores/projects";
import type { Project } from "@/types";

type Op = "pull" | "push" | "";

const { t } = useI18n();
const props = defineProps<{ project: Project }>();
const store = useProjectsStore();

const git = computed(() => props.project.git);
const busy = ref<Op>("");

const commitOpen = ref(false);
const conflictOpen = ref(false);
const conflicts = ref<string[]>([]);

const hasChanges = computed(
  () => !!git.value && git.value.staged + git.value.modified + git.value.untracked > 0,
);

async function pull(): Promise<boolean> {
  if (busy.value) return false;
  busy.value = "pull";
  try {
    const list = await store.pullRepository(props.project);
    if (list.length) {
      // 产生合并冲突:引导用户在 VSCode/终端中解决
      conflicts.value = list;
      conflictOpen.value = true;
      return false;
    }
    toast.success(t("git.pull.success"));
    return true;
  } catch (e) {
    toast.error(String(e));
    return false;
  } finally {
    busy.value = "";
  }
}

async function push() {
  if (busy.value) return;
  busy.value = "push";
  try {
    await store.pushRepository(props.project);
    toast.success(t("git.push.success"));
  } catch (e) {
    const msg = String(e);
    if (msg.includes("non-fast-forward") || msg.includes("fetch first")) {
      // 远端有本地缺失的提交:不再倾倒 git 原文,给出快捷修复入口
      toast.error(t("git.push.rejected"), {
        action: { label: t("git.push.pullAndPush"), onClick: () => pullThenPush() },
      });
    } else {
      toast.error(msg);
    }
  } finally {
    busy.value = "";
  }
}

/** 先拉取;无冲突则自动重试推送(有冲突则交给冲突引导流程) */
async function pullThenPush() {
  if (await pull()) await push();
}
</script>

<template>
  <div v-if="git?.is_repo" class="flex items-center gap-1.5">
    <Button
      variant="outline"
      size="xs"
      :disabled="busy !== '' || !hasChanges"
      @click="commitOpen = true"
    >
      <GitCommitHorizontal class="h-3.5 w-3.5" />
      {{ t("git.actions.commit") }}
    </Button>
    <Button variant="outline" size="xs" :disabled="busy !== ''" @click="pull">
      <Loader2 v-if="busy === 'pull'" class="h-3.5 w-3.5 animate-spin" />
      <ArrowDownToLine v-else class="h-3.5 w-3.5" />
      {{ busy === "pull" ? t("git.pull.pulling") : t("git.actions.pull") }}
    </Button>
    <Button variant="outline" size="xs" :disabled="busy !== ''" @click="push">
      <Loader2 v-if="busy === 'push'" class="h-3.5 w-3.5 animate-spin" />
      <ArrowUpToLine v-else class="h-3.5 w-3.5" />
      {{ busy === "push" ? t("git.push.pushing") : t("git.actions.push") }}
    </Button>

    <CommitDialog v-model:open="commitOpen" :project="project" />
    <ConflictDialog v-model:open="conflictOpen" :project="project" :conflicts="conflicts" />
  </div>
</template>
