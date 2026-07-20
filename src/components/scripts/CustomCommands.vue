<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Ban, Plus, TerminalSquare } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import ScriptItem from "@/components/scripts/ScriptItem.vue";
import { COMMAND_ICONS } from "@/lib/command-icons";
import { cmd, runInTerminal } from "@/lib/tauri";
import type { CustomCommand, Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project }>();

const commands = ref<CustomCommand[]>([]);

const dialogOpen = ref(false);
const editingId = ref<number | null>(null);
const formName = ref("");
const formCommand = ref("");
const formDescription = ref("");
const formIcon = ref("");
const submitting = ref(false);

async function load() {
  try {
    commands.value = await cmd<CustomCommand[]>("list_custom_commands", {
      projectId: props.project.id,
    });
  } catch (e) {
    toast.error(String(e));
  }
}

watch(() => props.project.id, load, { immediate: true });

function openCreate() {
  editingId.value = null;
  formName.value = "";
  formCommand.value = "";
  formDescription.value = "";
  formIcon.value = "";
  dialogOpen.value = true;
}

function openEdit(c: CustomCommand) {
  editingId.value = c.id;
  formName.value = c.name;
  formCommand.value = c.command;
  formDescription.value = c.description;
  formIcon.value = c.icon;
  dialogOpen.value = true;
}

async function submit() {
  if (!formName.value.trim() || !formCommand.value.trim() || submitting.value) return;
  submitting.value = true;
  try {
    const args = {
      name: formName.value.trim(),
      command: formCommand.value.trim(),
      description: formDescription.value.trim(),
      icon: formIcon.value,
    };
    if (editingId.value == null) {
      await cmd("create_custom_command", { projectId: props.project.id, ...args });
      toast.success(t("scripts.custom.created"));
    } else {
      await cmd("update_custom_command", { id: editingId.value, ...args });
      toast.success(t("scripts.custom.updated"));
    }
    dialogOpen.value = false;
    await load();
  } catch (e) {
    toast.error(String(e));
  } finally {
    submitting.value = false;
  }
}

async function remove(c: CustomCommand) {
  if (!window.confirm(t("scripts.custom.deleteConfirm", { name: c.name }))) return;
  try {
    await cmd("delete_custom_command", { id: c.id });
    await load();
    toast.success(t("scripts.custom.deleted"));
  } catch (e) {
    toast.error(String(e));
  }
}

async function run(c: CustomCommand) {
  try {
    await runInTerminal(props.project, c.command);
    toast.success(t("scripts.custom.started", { name: c.name }));
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <Card>
    <CardHeader class="flex-row items-center justify-between pb-3">
      <CardTitle class="flex items-center gap-2 text-sm font-semibold">
        <TerminalSquare class="h-4 w-4" />
        {{ t("scripts.custom.title") }}
      </CardTitle>
      <Button size="sm" variant="outline" @click="openCreate">
        <Plus class="h-4 w-4" />
        {{ t("scripts.custom.new") }}
      </Button>
    </CardHeader>
    <CardContent>
      <p v-if="!commands.length" class="text-sm text-muted-foreground">
        {{ t("scripts.custom.empty") }}
      </p>
      <ScrollArea v-else class="max-h-[420px]">
        <div class="flex flex-col">
          <ScriptItem
            v-for="c in commands"
            :key="c.id"
            :name="c.name"
            :command="c.command"
            :description="c.description"
            :icon="c.icon"
            editable
            @run="run(c)"
            @edit="openEdit(c)"
            @delete="remove(c)"
          />
        </div>
      </ScrollArea>
    </CardContent>
  </Card>

  <Dialog v-model:open="dialogOpen">
    <DialogContent>
      <DialogHeader>
        <DialogTitle>{{
          editingId == null ? t("scripts.custom.dialogNew") : t("scripts.custom.dialogEdit")
        }}</DialogTitle>
      </DialogHeader>
      <form class="flex flex-col gap-3" @submit.prevent="submit">
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("scripts.custom.nameLabel") }}</label>
          <Input v-model="formName" :placeholder="t('scripts.custom.namePlaceholder')" />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("scripts.custom.commandLabel") }}</label>
          <Input
            v-model="formCommand"
            :placeholder="t('scripts.custom.commandPlaceholder')"
            class="font-mono"
          />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("scripts.custom.descriptionLabel") }}</label>
          <Input
            v-model="formDescription"
            :placeholder="t('scripts.custom.descriptionPlaceholder')"
          />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("scripts.custom.iconLabel") }}</label>
          <div class="grid grid-cols-8 gap-1">
            <button
              type="button"
              class="flex h-8 w-8 items-center justify-center rounded-md border text-muted-foreground transition-colors hover:bg-accent"
              :class="
                formIcon === '' ? 'border-primary bg-accent text-foreground' : 'border-transparent'
              "
              :title="t('scripts.custom.noIcon')"
              @click="formIcon = ''"
            >
              <Ban class="h-4 w-4" />
            </button>
            <button
              v-for="i in COMMAND_ICONS"
              :key="i.name"
              type="button"
              class="flex h-8 w-8 items-center justify-center rounded-md border text-muted-foreground transition-colors hover:bg-accent"
              :class="
                formIcon === i.name
                  ? 'border-primary bg-accent text-foreground'
                  : 'border-transparent'
              "
              :title="i.name"
              @click="formIcon = i.name"
            >
              <component :is="i.component" class="h-4 w-4" />
            </button>
          </div>
        </div>
        <DialogFooter>
          <Button type="submit" :disabled="!formName.trim() || !formCommand.trim() || submitting">
            {{ submitting ? t("common.saving") : t("common.save") }}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
