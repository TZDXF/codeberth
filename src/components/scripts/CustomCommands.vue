<script setup lang="ts">
import { ref, watch } from "vue";
import { toast } from "vue-sonner";
import { Plus, TerminalSquare } from "@lucide/vue";
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
import ScriptItem from "@/components/scripts/ScriptItem.vue";
import { cmd, runInTerminal } from "@/lib/tauri";
import type { CustomCommand, Project } from "@/types";

const props = defineProps<{ project: Project }>();

const commands = ref<CustomCommand[]>([]);

const dialogOpen = ref(false);
const editingId = ref<number | null>(null);
const formName = ref("");
const formCommand = ref("");
const formDescription = ref("");
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
  dialogOpen.value = true;
}

function openEdit(c: CustomCommand) {
  editingId.value = c.id;
  formName.value = c.name;
  formCommand.value = c.command;
  formDescription.value = c.description;
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
    };
    if (editingId.value == null) {
      await cmd("create_custom_command", { projectId: props.project.id, ...args });
      toast.success("命令已创建");
    } else {
      await cmd("update_custom_command", { id: editingId.value, ...args });
      toast.success("命令已更新");
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
  if (!window.confirm(`确定删除命令「${c.name}」吗?`)) return;
  try {
    await cmd("delete_custom_command", { id: c.id });
    await load();
    toast.success("命令已删除");
  } catch (e) {
    toast.error(String(e));
  }
}

async function run(c: CustomCommand) {
  try {
    await runInTerminal(props.project, c.command);
    toast.success(`已在终端启动「${c.name}」`);
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
        自定义命令
      </CardTitle>
      <Button size="sm" variant="outline" @click="openCreate">
        <Plus class="h-4 w-4" />
        新建
      </Button>
    </CardHeader>
    <CardContent>
      <p v-if="!commands.length" class="text-sm text-muted-foreground">
        还没有自定义命令
      </p>
      <div v-else class="flex flex-col">
        <ScriptItem
          v-for="c in commands"
          :key="c.id"
          :name="c.name"
          :command="c.command"
          :description="c.description"
          editable
          @run="run(c)"
          @edit="openEdit(c)"
          @delete="remove(c)"
        />
      </div>
    </CardContent>
  </Card>

  <Dialog v-model:open="dialogOpen">
    <DialogContent>
      <DialogHeader>
        <DialogTitle>{{ editingId == null ? "新建命令" : "编辑命令" }}</DialogTitle>
      </DialogHeader>
      <form class="flex flex-col gap-3" @submit.prevent="submit">
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">名称</label>
          <Input v-model="formName" placeholder="例如: 启动后端" />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">命令</label>
          <Input v-model="formCommand" placeholder="例如: cargo run" class="font-mono" />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">描述(可选)</label>
          <Input v-model="formDescription" placeholder="命令用途说明" />
        </div>
        <DialogFooter>
          <Button
            type="submit"
            :disabled="!formName.trim() || !formCommand.trim() || submitting"
          >
            {{ submitting ? "保存中..." : "保存" }}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>

