<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useProjectsStore } from "@/stores/projects";
import type { Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project }>();
const open = defineModel<boolean>("open", { required: true });
const store = useProjectsStore();

const message = ref("");
const submitting = ref(false);
// 参考 IDEA:未跟踪文件默认不纳入提交,需显式勾选
const includeUntracked = ref(false);

const git = computed(() => props.project.git);
const untrackedCount = computed(() => git.value?.untracked ?? 0);
/** 本次实际会提交的变更数(未跟踪文件仅在勾选时计入) */
const committable = computed(() => {
  if (!git.value) return 0;
  return git.value.staged + git.value.modified + (includeUntracked.value ? git.value.untracked : 0);
});

// 每次打开时重置为初始状态
watch(open, (v) => {
  if (v) {
    message.value = "";
    includeUntracked.value = false;
  }
});

async function submit() {
  if (!message.value.trim() || committable.value === 0 || submitting.value) return;
  submitting.value = true;
  try {
    await store.commitChanges(props.project, message.value.trim(), includeUntracked.value);
    toast.success(t("git.commit.success"));
    open.value = false;
  } catch (e) {
    toast.error(String(e));
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent>
      <DialogHeader>
        <DialogTitle>{{ t("git.commit.title") }}</DialogTitle>
        <DialogDescription>{{ t("git.commit.description") }}</DialogDescription>
      </DialogHeader>
      <form class="flex flex-col gap-4" @submit.prevent="submit">
        <div v-if="git" class="flex gap-3 text-xs text-muted-foreground">
          <span>
            {{ t("git.staged") }}
            <span class="font-medium text-emerald-600">{{ git.staged }}</span>
          </span>
          <span>
            {{ t("git.modified") }}
            <span class="font-medium text-amber-600">{{ git.modified }}</span>
          </span>
          <span>
            {{ t("git.untracked") }}
            <span class="font-medium text-sky-600">{{ git.untracked }}</span>
          </span>
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("git.commit.messageLabel") }}</label>
          <textarea
            v-model="message"
            rows="3"
            :placeholder="t('git.commit.messagePlaceholder')"
            class="w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus-visible:ring-1 focus-visible:ring-ring"
            autofocus
            @keydown.enter.ctrl.prevent="submit"
          />
        </div>
        <label
          class="flex w-fit items-center gap-2 text-sm"
          :class="untrackedCount === 0 ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'"
        >
          <input
            v-model="includeUntracked"
            type="checkbox"
            :disabled="untrackedCount === 0"
            class="h-3.5 w-3.5 accent-primary"
          />
          {{ t("git.commit.includeUntracked") }}
          <span class="text-xs text-muted-foreground">({{ untrackedCount }})</span>
        </label>
        <p v-if="committable === 0" class="text-xs text-muted-foreground">
          {{ t("git.commit.empty") }}
        </p>
        <DialogFooter>
          <Button type="submit" :disabled="!message.trim() || committable === 0 || submitting">
            {{ submitting ? t("git.commit.submitting") : t("git.actions.commit") }}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
