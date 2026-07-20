<script setup lang="ts">
import { computed, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { getLocalTimeZone, today as calendarToday } from "@internationalized/date";
import type { RangeCalendarRootProps } from "reka-ui";
import {
  Calendar as CalendarIcon,
  Copy,
  ChevronRight,
  Loader2,
  Search,
  Sparkles,
  Tags,
  X,
} from "@lucide/vue";
import { Markdown, type ControlsConfig } from "vue-stream-markdown";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { RangeCalendar } from "@/components/ui/range-calendar";
import { ScrollArea } from "@/components/ui/scroll-area";
import TagCheckList from "@/components/tags/TagCheckList.vue";
import { generateDailyReport, type ProjectCommits } from "@/lib/ai";
import { cmd } from "@/lib/tauri";
import { useProjectsStore } from "@/stores/projects";
import { useSettingsStore } from "@/stores/settings";
import { useTagsStore } from "@/stores/tags";
import type { GitCommitInfo, GitUser } from "@/types";

type RangeKey = "today" | "yesterday" | "last3" | "last7" | "custom";
type AuthorMode = "me" | "all";

const RANGE_OPTIONS: { value: RangeKey; labelKey: string }[] = [
  { value: "today", labelKey: "report.today" },
  { value: "yesterday", labelKey: "report.yesterday" },
  { value: "last3", labelKey: "report.last3Days" },
  { value: "last7", labelKey: "report.last7Days" },
  { value: "custom", labelKey: "report.custom" },
];

const AUTHOR_OPTIONS: { value: AuthorMode; labelKey: string }[] = [
  { value: "me", labelKey: "report.authorMe" },
  { value: "all", labelKey: "report.authorAll" },
];

const { t } = useI18n();
const props = defineProps<{ presetProjectId?: number }>();
const open = defineModel<boolean>("open", { required: true });

const store = useProjectsStore();
const settings = useSettingsStore();
const tagsStore = useTagsStore();

const activeProjects = computed(() => store.projects.filter((p) => !p.archived_at));
/** 详情页传入 presetProjectId 时锁定单项目,隐藏项目选择 */
const locked = computed(() => props.presetProjectId != null);

const selectedIds = ref<number[]>([]);
// 项目筛选:关键字(名称/路径)+ 标签(与首页一致,多标签为 AND 语义),仅作用于本弹窗
const keyword = ref("");
const filterTagIds = ref<number[]>([]);

const visibleProjects = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  return activeProjects.value.filter((p) => {
    if (kw && !p.name.toLowerCase().includes(kw) && !p.path.toLowerCase().includes(kw)) {
      return false;
    }
    return filterTagIds.value.every((id) => p.tags.some((t) => t.id === id));
  });
});

const selectedFilterTags = computed(() =>
  tagsStore.tags.filter((tag) => filterTagIds.value.includes(tag.id)),
);

function toggleTagFilter(id: number) {
  filterTagIds.value = filterTagIds.value.includes(id)
    ? filterTagIds.value.filter((x) => x !== id)
    : [...filterTagIds.value, id];
}
// reka-ui 的 d.ts 存在内联日期类型与 @internationalized/date 两套声明,直接索引组件 props
// 的 modelValue 类型,保证 v-model / max-value 与 RangeCalendar 期望的类型严格一致
type RangeModel = NonNullable<RangeCalendarRootProps["modelValue"]>;
type RangeDateValue = NonNullable<RangeModel["start"]>;

const rangeKey = ref<RangeKey>("today");
/** 自定义范围(reka RangeCalendar 的 DateRange,起止可为空表示未选完) */
// 用 shallowRef:ref<T> 的 UnwrapRef 会把日期类实例展开成结构类型,破坏名义类型匹配
const customRange = shallowRef<RangeModel>({ start: undefined, end: undefined });
/** 日历可选上限:今天(提交记录不可能来自未来);运行时与 reka 内部日期实现相同,仅类型需断言 */
const maxDate = calendarToday(getLocalTimeZone()) as unknown as RangeDateValue;
const authorMode = ref<AuthorMode>("me");
const generating = ref(false);
const result = ref("");
/** 本次拉取到的提交记录(驱动提交条数与可展开列表;生成前展示,AI 失败也保留) */
const commitData = ref<ProjectCommits[]>([]);
/** 各项目提交列表展开状态,key 为项目名 */
const commitOpen = ref<Record<string, boolean>>({});

const totalCommits = computed(() => commitData.value.reduce((sum, d) => sum + d.commits.length, 0));

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

/** DateValue → 本地当天 00:00 的 Date */
function toLocalDate(d: RangeDateValue) {
  return d.toDate(getLocalTimeZone());
}

/** 自定义范围触发按钮的展示文案 */
const customRangeLabel = computed(() => {
  const { start, end } = customRange.value;
  if (start && end) return `${fmt(toLocalDate(start))} - ${fmt(toLocalDate(end))}`;
  const single = start ?? end;
  return single ? fmt(toLocalDate(single)) : t("report.pickRange");
});

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
      const { start, end } = customRange.value;
      return {
        from: start ? toLocalDate(start) : today,
        to: end ? toLocalDate(end) : today,
      };
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
  customRange.value = { start: undefined, end: undefined };
  authorMode.value = "me";
  commitData.value = [];
  commitOpen.value = {};
  keyword.value = "";
  filterTagIds.value = [];
  if (!tagsStore.tags.length) void tagsStore.fetchTags();
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
  commitData.value = [];
  generating.value = true;
  try {
    const since = `${fmt(range.value.from)} 00:00:00`;
    const until = `${fmt(range.value.to)} 23:59:59`;
    let data: ProjectCommits[];
    try {
      data = await Promise.all(
        projects.map(async (p) => {
          // "仅自己":取该仓库 git 用户身份作为 --author 过滤;未配置则不过滤
          let author: string | undefined;
          if (authorMode.value === "me") {
            const user = await cmd<GitUser>("git_current_user", { path: p.path });
            author = user.name || user.email || undefined;
          }
          return {
            projectName: p.name,
            commits: await cmd<GitCommitInfo[]>("git_log", {
              path: p.path,
              since,
              until,
              maxCount: 500,
              author,
            }),
          };
        }),
      );
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      toast.error(t("report.loadFailed", { error: message }));
      return;
    }
    commitData.value = data;
    commitOpen.value = {};
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
                @click="
                  selectedIds = [...new Set([...selectedIds, ...visibleProjects.map((p) => p.id)])]
                "
              >
                {{ t("report.selectAll") }}
              </Button>
              <Button variant="ghost" size="sm" class="h-6 px-2 text-xs" @click="selectedIds = []">
                {{ t("report.clear") }}
              </Button>
            </div>
          </div>
          <div class="flex items-center gap-1.5">
            <div class="relative flex-1">
              <Search
                class="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground"
              />
              <Input
                v-model="keyword"
                :placeholder="t('report.projectSearchPlaceholder')"
                class="h-7 pl-7 text-xs"
              />
            </div>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <Button variant="outline" size="sm" class="h-7 gap-1.5 px-2 text-xs">
                  <Tags class="h-3.5 w-3.5" />
                  {{ t("projects.home.filterTags") }}
                  <span
                    v-if="filterTagIds.length"
                    class="rounded-full bg-primary px-1.5 text-[11px] leading-4 text-primary-foreground"
                  >
                    {{ filterTagIds.length }}
                  </span>
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" class="w-52">
                <TagCheckList
                  :tags="tagsStore.tags"
                  :checked-ids="filterTagIds"
                  @toggle="toggleTagFilter"
                />
                <template v-if="filterTagIds.length">
                  <DropdownMenuSeparator />
                  <DropdownMenuItem class="gap-2 text-xs" @click="filterTagIds = []">
                    <X class="h-3.5 w-3.5" />
                    {{ t("projects.home.clearFilter") }}
                  </DropdownMenuItem>
                </template>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
          <div v-if="selectedFilterTags.length" class="flex flex-wrap items-center gap-1.5">
            <button
              v-for="tag in selectedFilterTags"
              :key="tag.id"
              type="button"
              class="flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] transition-opacity hover:opacity-80"
              :style="{ backgroundColor: tag.color, borderColor: tag.color, color: '#fff' }"
              :title="t('projects.home.removeFilterTag', { name: tag.name })"
              @click="toggleTagFilter(tag.id)"
            >
              {{ tag.name }}
              <X class="h-2.5 w-2.5" />
            </button>
          </div>
          <div class="grid max-h-36 grid-cols-2 gap-x-2 overflow-y-auto rounded-md border p-2">
            <label
              v-for="p in visibleProjects"
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
            <p
              v-if="!visibleProjects.length"
              class="col-span-2 px-1.5 py-2 text-xs text-muted-foreground"
            >
              {{ t("report.noMatch") }}
            </p>
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
            <Popover v-if="rangeKey === 'custom'">
              <PopoverTrigger as-child>
                <Button
                  variant="outline"
                  size="sm"
                  class="h-7 gap-1.5 px-2.5 text-xs font-normal"
                  :class="{ 'text-muted-foreground': !customRange.start && !customRange.end }"
                >
                  <CalendarIcon class="h-3.5 w-3.5" />
                  {{ customRangeLabel }}
                </Button>
              </PopoverTrigger>
              <PopoverContent class="w-auto p-0" align="start">
                <RangeCalendar
                  v-model="customRange"
                  :number-of-months="2"
                  :locale="settings.language"
                  :max-value="maxDate"
                />
              </PopoverContent>
            </Popover>
          </div>
        </div>

        <div class="flex flex-col gap-1.5">
          <label class="text-sm font-medium">{{ t("report.author") }}</label>
          <div class="flex flex-wrap items-center gap-1.5">
            <Button
              v-for="opt in AUTHOR_OPTIONS"
              :key="opt.value"
              size="sm"
              :variant="authorMode === opt.value ? 'default' : 'outline'"
              class="h-7 px-2.5 text-xs"
              @click="authorMode = opt.value"
            >
              {{ t(opt.labelKey) }}
            </Button>
          </div>
        </div>

        <div class="flex justify-end">
          <Button size="sm" class="gap-1.5" :disabled="generating" @click="generate">
            <Loader2 v-if="generating" class="h-3.5 w-3.5 animate-spin" />
            <Sparkles v-else class="h-3.5 w-3.5" />
            {{ generating ? t("report.generating") : t("report.generate") }}
          </Button>
        </div>

        <div v-if="commitData.length" class="flex flex-col gap-1.5">
          <div class="flex items-center justify-between">
            <label class="text-sm font-medium">{{ t("report.commits") }}</label>
            <Badge variant="secondary" class="text-xs">
              {{ t("report.commitCount", { count: totalCommits }) }}
            </Badge>
          </div>
          <div class="rounded-md border">
            <Collapsible
              v-for="d in commitData"
              :key="d.projectName"
              v-slot="{ open: expanded }"
              :open="commitOpen[d.projectName]"
              @update:open="commitOpen[d.projectName] = $event"
            >
              <CollapsibleTrigger
                class="flex w-full cursor-pointer items-center gap-2 px-2.5 py-1.5 text-left text-sm hover:bg-accent"
              >
                <ChevronRight
                  class="h-3.5 w-3.5 shrink-0 text-muted-foreground transition-transform"
                  :class="{ 'rotate-90': expanded }"
                />
                <span class="min-w-0 flex-1 truncate">{{ d.projectName }}</span>
                <span class="shrink-0 text-xs text-muted-foreground">
                  {{ t("report.commitCount", { count: d.commits.length }) }}
                </span>
              </CollapsibleTrigger>
              <CollapsibleContent>
                <div v-if="d.commits.length" class="max-h-40 overflow-y-auto border-t">
                  <div
                    v-for="c in d.commits"
                    :key="c.hash + c.date"
                    class="flex items-center gap-2 px-3 py-1 text-xs"
                  >
                    <code class="shrink-0 rounded bg-muted px-1 py-0.5 font-mono text-[11px]">
                      {{ c.hash }}
                    </code>
                    <span class="min-w-0 flex-1 truncate" :title="c.subject">{{ c.subject }}</span>
                    <span class="shrink-0 text-muted-foreground">{{ c.author }}</span>
                    <span class="shrink-0 text-muted-foreground">{{ c.date }}</span>
                  </div>
                </div>
                <p v-else class="border-t px-3 py-2 text-xs text-muted-foreground">
                  {{ t("report.projectNoCommits") }}
                </p>
              </CollapsibleContent>
            </Collapsible>
          </div>
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
