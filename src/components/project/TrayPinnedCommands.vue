<script setup lang="ts">
// 托盘弹窗项目行下方行内展开的「常用命令」列表:点击直接在新终端执行,
// compose 条目默认 up -d,行尾下拉提供其它动作;hover 可取消标记(清理失效标记的出口)
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import {
  Container,
  MoreHorizontal,
  Package,
  Play,
  RotateCw,
  Square,
  Star,
  TerminalSquare,
} from "@lucide/vue";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { runInTerminal } from "@/lib/tauri";
import { usePinsStore } from "@/stores/pins";
import type { PinnedCommand, Project } from "@/types";

const { t } = useI18n();
const props = defineProps<{ project: Project; pins: PinnedCommand[] }>();
const pinsStore = usePinsStore();

type ComposeAction = "up -d" | "down" | "restart" | "stop";

/** compose 文件级可用动作;服务级不含 down(down 作用于整个 compose 项目) */
const FILE_ACTIONS: ComposeAction[] = ["up -d", "down", "restart", "stop"];
const SERVICE_ACTIONS: ComposeAction[] = ["up -d", "restart", "stop"];

const isCompose = (p: PinnedCommand) => p.kind === "composeFile" || p.kind === "composeService";

/** composeService 的服务名存于 target_key 第二段("<file>\n<service>") */
function serviceOf(p: PinnedCommand): string | undefined {
  return p.kind === "composeService" ? p.target_key.split("\n")[1] : undefined;
}

function actionsOf(p: PinnedCommand): ComposeAction[] {
  return p.kind === "composeService" ? SERVICE_ACTIONS : FILE_ACTIONS;
}

const ACTION_ICONS: Record<ComposeAction, typeof Play> = {
  "up -d": Play,
  restart: RotateCw,
  down: Square,
  stop: Square,
};

const ACTION_LABEL_KEYS: Record<ComposeAction, string> = {
  "up -d": "docker.up",
  restart: "docker.restart",
  down: "docker.down",
  stop: "docker.stop",
};

/** 行点击:npm/自定义直接执行存好的命令;compose 执行默认动作 up -d */
async function runDefault(p: PinnedCommand) {
  if (isCompose(p)) {
    await runCompose(p, "up -d");
    return;
  }
  try {
    // cwd 存的是相对项目根的目录(monorepo 子包),执行时用当前 project.path 拼接,迁移目录后仍可用
    const cwd = p.cwd ? `${props.project.path}/${p.cwd}` : undefined;
    await runInTerminal(props.project, p.command, cwd);
    toast.success(t("pins.started", { name: p.label }));
  } catch (e) {
    toast.error(String(e));
  }
}

/** compose 条目执行指定动作(command 为基础前缀,在此拼接动作与服务名) */
async function runCompose(p: PinnedCommand, action: ComposeAction) {
  const service = serviceOf(p);
  const command = `${p.command} ${service ? `${action} ${service}` : action}`;
  try {
    await runInTerminal(props.project, command);
    toast.success(t("pins.started", { name: p.label }));
  } catch (e) {
    toast.error(String(e));
  }
}

async function unpin(p: PinnedCommand) {
  try {
    await pinsStore.setPinned(
      props.project.id,
      {
        kind: p.kind,
        targetKey: p.target_key,
        label: p.label,
        command: p.command,
        cwd: p.cwd ?? undefined,
      },
      false,
    );
  } catch (e) {
    toast.error(String(e));
  }
}

function kindIcon(p: PinnedCommand) {
  if (p.kind === "packageScript") {
    return Package;
  }
  if (isCompose(p)) {
    return Container;
  }
  return TerminalSquare;
}
</script>

<template>
  <div v-if="pins.length" class="flex flex-col gap-0.5 pb-1 pl-4">
    <div
      v-for="p in pins"
      :key="p.id"
      class="group/pin flex items-center gap-1.5 rounded-md px-2 py-1 transition-colors hover:bg-accent"
    >
      <button
        type="button"
        class="flex min-w-0 flex-1 items-center gap-1.5 text-left"
        :title="p.command"
        @click.stop="runDefault(p)"
      >
        <component :is="kindIcon(p)" class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        <span class="min-w-0 flex-1 truncate text-xs">{{ p.label }}</span>
        <span
          v-if="p.kind === 'composeService'"
          class="shrink-0 truncate font-mono text-[10px] text-muted-foreground"
          :title="p.target_key.split('\n')[0]"
        >
          {{ p.target_key.split("\n")[0] }}
        </span>
      </button>
      <DropdownMenu v-if="isCompose(p)">
        <DropdownMenuTrigger as-child>
          <Button
            variant="ghost"
            size="icon"
            class="h-6 w-6 shrink-0 text-muted-foreground"
            :title="t('docker.more')"
            @click.stop
          >
            <MoreHorizontal class="h-3.5 w-3.5" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" class="w-36">
          <DropdownMenuItem
            v-for="action in actionsOf(p)"
            :key="action"
            class="gap-2 text-xs"
            @click.stop="runCompose(p, action)"
          >
            <component :is="ACTION_ICONS[action]" class="h-3.5 w-3.5" />
            {{ t(ACTION_LABEL_KEYS[action]) }}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <Button
        variant="ghost"
        size="icon"
        class="h-6 w-6 shrink-0 text-yellow-500 opacity-0 transition-opacity group-hover/pin:opacity-100"
        :title="t('pins.unmark')"
        @click.stop="unpin(p)"
      >
        <Star class="h-3.5 w-3.5 fill-yellow-400" />
      </Button>
    </div>
  </div>
</template>
