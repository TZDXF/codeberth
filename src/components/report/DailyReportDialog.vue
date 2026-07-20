<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Copy, Loader2, Sparkles } from "@lucide/vue";
import { Markdown, type ControlsConfig } from "vue-stream-markdown";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { generateDailyReport, type ProjectCommits } from "@/lib/ai";
import { cmd } from "@/lib/tauri";
import { useProjectsStore } from "@/stores/projects";
import { useSettingsStore } from "@/stores/settings";
import type { GitCommitInfo } from "@/types";

type RangeKey = "today" | "yesterday" | "last3" | "last7" | "custom";

const RANGE_OPTIONS: { value: RangeKey; labelKey: string }[] = [
  { value: "today", labelKey: "report.today" },
  { value: "yesterday", labelKey: "report.yesterday" },
  { value: "last3", labelKey: "report.last3Days" },
  { value: "last7", labelKey: "report.last7Days" },
  { value: "custom", labelKey: "report.custom" },
];

const { t } = useI18n();
const props = defineProps<{ presetProjectId?: number }>();
const open = defineModel<boolean>("open", { required: true });

const store = useProjectsStore();
const settings = useSettingsStore();

const activeProjects = computed(() => store.projects.filter((p) => !p.archived_at));
/** 详情页传入 presetProjectId 时锁定单项目,隐藏项目选择 */
const locked = computed(() => props.presetProjectId != null);

const selectedIds = ref<number[]>([]);
const rangeKey = ref<RangeKey>("today");
const customFrom = ref("");
const customTo = ref("");
const generating = ref(false);
const result = ref("");

// 表格/代码复制导出控件,与 ReadmeDrawer 保持一致
const controls: ControlsConfig = {
  table: { copy: true, download: true, fullscreen: true },
  code: { copy: true, collapse: true },
};

// 同 ReadmeDrawer:阻止库内联宿主变量,MD 主题交给 CSS 层
const detachedThemeEl = document.createElement("div");
const themeElement = () => detachedThemeEl;

function fmt(d: Date) {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

/** 当前选择的日期范围(本地时区,起止均为当天) */
const range = computed<{ from: Date; to: Date }>(() => {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const daysAgo = (n: number) => {
    const d = new Date(today);
    d.setDate(d.getDate() - n);
    return d;
  };
  switch (rangeKey.value) {
    case "yesterday":
      return { from: daysAgo(1), to: daysAgo(1) };
    case "last3":
      return { from: daysAgo(2), to: today };
    case "last7":
      return { from: daysAgo(6), to: today };
    case "custom": {
      const from = customFrom.value ? new Date(`${customFrom.value}T00:00:00`) : today;
      const to = customTo.value ? new Date(`${customTo.value}T00:00:00`) : today;
      return { from, to };
    }
    default:
      return { from: today, to: today };
  }
});

// 每次打开重置为初始状态
watch(open, (v) => {
  if (!v) return;
  result.value = "";
  rangeKey.value = "today";
  customFrom.value = "";
  customTo.value = "";
  selectedIds.value = props.presetProjectId != null ? [props.presetProjectId] : [];
});

function toggleProject(id: number) {
  selectedIds.value = selectedIds.value.includes(id)
    ? selectedIds.value.filter((x) => x !== id)
    : [...selectedIds.value, id];
}

async function generate() {
  if (generating.value) return;
  const ids = props.presetProjectId != null ? [props.presetProjectId] : selectedIds.value;
  const projects = activeProjects.value.filter((p) => ids.includes(p.id));
  if (!projects.length) {
    toast.error(t("report.noProjects"));
    return;
  }
  generating.value = true;
  try {
    const since = `${fmt(range.value.from)} 00:00:00`;
    const until = `${fmt(range.value.to)} 23:59:59`;
    let data: ProjectCommits[];
    try {
      data = await Promise.all(
        projects.map(async (p) => ({
          projectName: p.name,
          commits: await cmd<GitCommitInfo[]>("git_log", {
            path: p.path,
            since,
            until,
            maxCount: 500,
          }),
        })),
      );
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      toast.error(t("report.loadFailed", { error: message }));
      return;
    }
    if (!data.some((d) => d.commits.length)) {
      result.value = "";
      toast.info(t("report.noCommits"));
      return;
    }
    const rangeLabel = t("report.rangeLabel", {
      from: fmt(range.value.from),
      to: fmt(range.value.to),
    });
    result.value = await generateDailyReport(data, rangeLabel, settings.language);
  } catch (e) {
    toast.error(e instanceof Error ? e.message : String(e));
  } finally {
    generating.value = false;
  }
}

async function copyResult() {
  try {
    await navigator.clipboard.writeText(result.value);
    toast.success(t("report.copied"));
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="max-w-2xl">
      <DialogHeader>
        <DialogTitle>{{ t("report.title") }}</DialogTitle>
        <DialogDescription>{{ t("report.description") }}</DialogDescription>
      </DialogHeader>

      <div class="flex flex-col gap-4">
        <div v-if="!locked" class="flex flex-col gap-1.5">
          <div class="flex items-center justify-between">
            <label class="text-sm font-medium">{{ t("report.selectProjects") }}</label>
            <div class="flex gap-1">
              <Button
                variant="ghost"
                size="sm"
                class="h-6 px-2 text-xs"
                @click="selectedIds = activeProjects.map((p) => p.id)"
              >
                {{ t("report.selectAll") }}
              </Button>
              <Button variant="ghost" size="sm" class="h-6 px-2 text-xs" @click="selectedIds = []">
                {{ t("report.clear") }}
              </Button>
            </div>
          </div>
          <div class="grid max-h-36 grid-cols-2 gap-x-2 overflow-y-auto rounded-md border p-2">
            <label
              v-for="p in activeProjects"
              :key="p.id"
              class="flex cursor-pointer items-center gap-2 rounded px-1.5 py-1 text-sm hover:bg-accent"
            >
              <input
                type="checkbox"
                class="h-3.5 w-3.5 shrink-0 accent-primary"
                :checked="selectedIds.includes(p.id)"
                @change="toggleProject(p.id)"
              />
              <span class="truncate" :title="p.path">{{ p.name }}</span>
            </label>
          </div>
        </div>

        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("report.range") }}</label>
          <div class="flex flex-wrap items-center gap-1.5">
            <Button
              v-for="opt in RANGE_OPTIONS"
              :key="opt.value"
              size="sm"
              :variant="rangeKey === opt.value ? 'default' : 'outline'"
              class="h-7 px-2.5 text-xs"
              @click="rangeKey = opt.value"
            >
              {{ t(opt.labelKey) }}
            </Button>
            <template v-if="rangeKey === 'custom'">
              <input
                v-model="customFrom"
                type="date"
                :title="t('report.from')"
                class="h-7 rounded-md border border-input bg-transparent px-2 text-xs outline-none focus-visible:ring-1 focus-visible:ring-ring"
              />
              <span class="text-xs text-muted-foreground">-</span>
              <input
                v-model="customTo"
                type="date"
                :title="t('report.to')"
                class="h-7 rounded-md border border-input bg-transparent px-2 text-xs outline-none focus-visible:ring-1 focus-visible:ring-ring"
              />
            </template>
          </div>
        </div>

        <div class="flex justify-end">
          <Button size="sm" class="gap-1.5" :disabled="generating" @click="generate">
            <Loader2 v-if="generating" class="h-3.5 w-3.5 animate-spin" />
            <Sparkles v-else class="h-3.5 w-3.5" />
            {{ generating ? t("report.generating") : t("report.generate") }}
          </Button>
        </div>

        <div class="rounded-md border">
          <div class="flex items-center justify-between border-b px-3 py-1.5">
            <span class="text-xs text-muted-foreground">Markdown</span>
            <Button
              v-if="result"
              variant="ghost"
              size="sm"
              class="h-6 gap-1 px-2 text-xs"
              @click="copyResult"
            >
              <Copy class="h-3 w-3" />
              {{ t("report.copy") }}
            </Button>
          </div>
          <ScrollArea class="h-64">
            <p v-if="!result" class="p-4 text-sm text-muted-foreground">
              {{ generating ? t("report.generating") : t("report.placeholder") }}
            </p>
            <div v-else class="p-4 text-sm">
              <Markdown
                mode="static"
                :content="result"
                :controls="controls"
                :theme-element="themeElement"
                :locale="settings.language"
              />
            </div>
          </ScrollArea>
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>
