<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import {
  CalendarClock,
  Clock,
  Pencil,
  Plus,
  Power,
  PowerOff,
  Search,
  Tags,
  Trash2,
  X,
} from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
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
import TagCheckList from "@/components/tags/TagCheckList.vue";
import { cmd } from "@/lib/tauri";
import { useProjectsStore } from "@/stores/projects";
import { useTagsStore } from "@/stores/tags";
import type { ReportSchedule } from "@/types";

const { t } = useI18n();
const projectStore = useProjectsStore();
const tagsStore = useTagsStore();

const schedules = ref<ReportSchedule[]>([]);
const loading = ref(false);

const activeProjects = computed(() => projectStore.projects.filter((p) => !p.archived_at));

// ── CRUD ───────────────────────────────────────────────────────────────

async function load() {
  loading.value = true;
  try {
    schedules.value = await cmd<ReportSchedule[]>("list_report_schedules");
  } catch (e) {
    toast.error(t("reportSchedule.saveFailed"));
  } finally {
    loading.value = false;
  }
}

async function saveAll(items: ReportSchedule[]) {
  try {
    await cmd("save_report_schedules", { schedules: items });
  } catch (e) {
    toast.error(t("reportSchedule.saveFailed"));
  }
}

async function toggleSchedule(s: ReportSchedule) {
  s.enabled = !s.enabled;
  await saveAll(schedules.value);
}

async function deleteSchedule(id: string) {
  schedules.value = schedules.value.filter((s) => s.id !== id);
  await saveAll(schedules.value);
  toast.success(t("reportSchedule.deleted"));
}

// ── dialog ─────────────────────────────────────────────────────────────

const dialogOpen = ref(false);
const editing = ref<ReportSchedule | null>(null);

// form
const formName = ref("");
const formTime = ref("18:00");
const formWeekdayMode = ref<"everyday" | "weekdays" | "chineseWorkday">("everyday");
const formAuthorMode = ref<"me" | "all">("me");
const formProjectIds = ref<number[]>([]);

// project filter (same pattern as DailyReportDialog)
const keyword = ref("");
const filterTagIds = ref<number[]>([]);

const visibleProjects = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  return activeProjects.value.filter((p) => {
    if (kw && !p.name.toLowerCase().includes(kw) && !p.path.toLowerCase().includes(kw))
      return false;
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

function toggleProject(id: number) {
  formProjectIds.value = formProjectIds.value.includes(id)
    ? formProjectIds.value.filter((x) => x !== id)
    : [...formProjectIds.value, id];
}

function openCreate() {
  editing.value = null;
  formName.value = "";
  formTime.value = "18:00";
  formWeekdayMode.value = "everyday";
  formAuthorMode.value = "me";
  formProjectIds.value = [];
  keyword.value = "";
  filterTagIds.value = [];
  dialogOpen.value = true;
}

function openEdit(s: ReportSchedule) {
  editing.value = s;
  formName.value = s.name;
  formTime.value = s.timeOfDay;
  formAuthorMode.value = s.authorMode;
  formProjectIds.value = [...s.projectIds];
  if (s.chineseWorkdayOnly) formWeekdayMode.value = "chineseWorkday";
  else if (s.weekdaysOnly) formWeekdayMode.value = "weekdays";
  else formWeekdayMode.value = "everyday";
  keyword.value = "";
  filterTagIds.value = [];
  dialogOpen.value = true;
}

async function submit() {
  if (!formProjectIds.value.length) {
    toast.error(t("report.noProjects"));
    return;
  }
  const data: ReportSchedule = {
    id: editing.value?.id ?? crypto.randomUUID(),
    name: formName.value.trim(),
    enabled: editing.value?.enabled ?? true,
    projectIds: [...formProjectIds.value],
    authorMode: formAuthorMode.value,
    timeOfDay: formTime.value,
    weekdaysOnly: formWeekdayMode.value === "weekdays",
    chineseWorkdayOnly: formWeekdayMode.value === "chineseWorkday",
    lastRunAt: editing.value?.lastRunAt ?? null,
  };

  if (editing.value) {
    const idx = schedules.value.findIndex((s) => s.id === editing.value!.id);
    if (idx !== -1) schedules.value[idx] = data;
  } else {
    schedules.value.push(data);
  }
  await saveAll(schedules.value);
  toast.success(t("reportSchedule.saved"));
  dialogOpen.value = false;
}

// ── helpers ────────────────────────────────────────────────────────────

function weekdayLabel(mode: string) {
  if (mode === "chineseWorkday") return t("reportSchedule.chineseWorkdayOnly");
  if (mode === "weekdays") return t("reportSchedule.weekdaysOnly");
  return t("reportSchedule.everyday");
}

function weekdayMode(s: ReportSchedule) {
  if (s.chineseWorkdayOnly) return "chineseWorkday";
  if (s.weekdaysOnly) return "weekdays";
  return "everyday";
}

function projectNames(ids: number[]) {
  return ids
    .map((id) => activeProjects.value.find((p) => p.id === id)?.name ?? "")
    .filter(Boolean);
}

function lastRun(ts: number | null) {
  if (!ts) return t("reportSchedule.never");
  const d = new Date(ts * 1000);
  return d.toLocaleString();
}

// ── init ───────────────────────────────────────────────────────────────

watch(
  () => projectStore.projects.length,
  (n) => {
    if (n) load();
  },
  { immediate: true },
);
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-base font-semibold">{{ t("reportSchedule.title") }}</h2>
        <p class="text-sm text-muted-foreground">{{ t("reportSchedule.description") }}</p>
      </div>
      <Button size="sm" class="gap-1.5" @click="openCreate">
        <Plus class="h-3.5 w-3.5" />
        {{ t("reportSchedule.create") }}
      </Button>
    </div>

    <!-- empty -->
    <div
      v-if="!loading && !schedules.length"
      class="rounded-md border border-dashed p-8 text-center text-sm text-muted-foreground"
    >
      <CalendarClock class="mx-auto mb-2 h-8 w-8 opacity-40" />
      {{ t("reportSchedule.empty") }}
    </div>

    <!-- list -->
    <div v-if="schedules.length" class="flex flex-col gap-2">
      <div
        v-for="s in schedules"
        :key="s.id"
        class="flex items-center gap-3 rounded-md border p-3"
      >
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium truncate">{{
              s.name || t("reportSchedule.title")
            }}</span>
            <Badge :variant="s.enabled ? 'default' : 'secondary'" class="text-[11px]">
              {{ s.enabled ? t("reportSchedule.enabled") : t("reportSchedule.disabled") }}
            </Badge>
          </div>
          <div class="mt-0.5 flex flex-wrap items-center gap-x-3 gap-y-0.5 text-xs text-muted-foreground">
            <span class="flex items-center gap-1">
              <Clock class="h-3 w-3" />
              {{ s.timeOfDay }}
            </span>
            <span>{{ weekdayLabel(weekdayMode(s)) }}</span>
            <span>{{ t("reportSchedule.authorLabel") }}: {{ s.authorMode === "me" ? t("reportSchedule.authorMe") : t("reportSchedule.authorAll") }}</span>
            <span class="truncate max-w-48" :title="projectNames(s.projectIds).join(', ')">
              {{ projectNames(s.projectIds).slice(0, 2).join(", ")
              }}{{ projectNames(s.projectIds).length > 2 ? ` +${projectNames(s.projectIds).length - 2}` : "" }}
            </span>
          </div>
          <div class="mt-1 text-[11px] text-muted-foreground">
            {{ t("reportSchedule.lastRun") }}: {{ lastRun(s.lastRunAt) }}
          </div>
        </div>
        <div class="flex items-center gap-1 shrink-0">
          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7"
            :title="s.enabled ? t('reportSchedule.disabled') : t('reportSchedule.enabled')"
            @click="toggleSchedule(s)"
          >
            <Power v-if="s.enabled" class="h-3.5 w-3.5 text-green-500" />
            <PowerOff v-else class="h-3.5 w-3.5 text-muted-foreground" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7"
            :title="t('reportSchedule.edit')"
            @click="openEdit(s)"
          >
            <Pencil class="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7 text-destructive hover:text-destructive"
            :title="t('reportSchedule.delete')"
            @click="deleteSchedule(s.id)"
          >
            <Trash2 class="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>
    </div>

    <!-- create / edit dialog -->
    <Dialog v-model:open="dialogOpen">
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{{
            editing ? t("reportSchedule.edit") : t("reportSchedule.create")
          }}</DialogTitle>
          <DialogDescription>{{ t("reportSchedule.description") }}</DialogDescription>
        </DialogHeader>

        <div class="flex flex-col gap-4 py-2">
          <!-- name -->
          <div class="flex flex-col gap-1.5">
            <label class="text-sm font-medium">{{ t("reportSchedule.nameLabel") }}</label>
            <Input v-model="formName" :placeholder="t('reportSchedule.namePlaceholder')" class="h-8" />
          </div>

          <!-- time -->
          <div class="flex flex-col gap-1.5">
            <label class="text-sm font-medium">{{ t("reportSchedule.timeLabel") }}</label>
            <Input v-model="formTime" type="time" class="h-8 w-28" />
          </div>

          <!-- weekday filter -->
          <div class="flex flex-col gap-1.5">
            <label class="text-sm font-medium">{{ t("reportSchedule.weekdayLabel") }}</label>
            <div class="flex flex-wrap gap-1.5">
              <Button
                size="sm"
                class="h-7 text-xs"
                :variant="formWeekdayMode === 'everyday' ? 'default' : 'outline'"
                @click="formWeekdayMode = 'everyday'"
              >
                {{ t("reportSchedule.everyday") }}
              </Button>
              <Button
                size="sm"
                class="h-7 text-xs"
                :variant="formWeekdayMode === 'weekdays' ? 'default' : 'outline'"
                @click="formWeekdayMode = 'weekdays'"
              >
                {{ t("reportSchedule.weekdaysOnly") }}
              </Button>
              <Button
                size="sm"
                class="h-7 text-xs"
                :variant="formWeekdayMode === 'chineseWorkday' ? 'default' : 'outline'"
                @click="formWeekdayMode = 'chineseWorkday'"
              >
                {{ t("reportSchedule.chineseWorkdayOnly") }}
              </Button>
            </div>
            <p
              v-if="formWeekdayMode === 'chineseWorkday'"
              class="text-[11px] text-muted-foreground"
            >
              {{ t("reportSchedule.chineseWorkdayHint") }}
            </p>
          </div>

          <!-- author -->
          <div class="flex flex-col gap-1.5">
            <label class="text-sm font-medium">{{ t("reportSchedule.authorLabel") }}</label>
            <div class="flex gap-1.5">
              <Button
                size="sm"
                class="h-7 text-xs"
                :variant="formAuthorMode === 'me' ? 'default' : 'outline'"
                @click="formAuthorMode = 'me'"
              >
                {{ t("reportSchedule.authorMe") }}
              </Button>
              <Button
                size="sm"
                class="h-7 text-xs"
                :variant="formAuthorMode === 'all' ? 'default' : 'outline'"
                @click="formAuthorMode = 'all'"
              >
                {{ t("reportSchedule.authorAll") }}
              </Button>
            </div>
          </div>

          <!-- projects -->
          <div class="flex flex-col gap-1.5">
            <label class="text-sm font-medium">{{ t("reportSchedule.projectsLabel") }}</label>
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
                @click="toggleTagFilter(tag.id)"
              >
                {{ tag.name }}
                <X class="h-2.5 w-2.5" />
              </button>
            </div>
            <div class="grid max-h-40 grid-cols-1 gap-x-2 overflow-y-auto rounded-md border p-2">
              <label
                v-for="p in visibleProjects"
                :key="p.id"
                class="flex cursor-pointer items-center gap-2 rounded px-1.5 py-1 text-sm hover:bg-accent"
              >
                <input
                  type="checkbox"
                  class="h-3.5 w-3.5 shrink-0 accent-primary"
                  :checked="formProjectIds.includes(p.id)"
                  @change="toggleProject(p.id)"
                />
                <span class="truncate" :title="p.path">{{ p.name }}</span>
              </label>
              <p v-if="!visibleProjects.length" class="px-1.5 py-2 text-xs text-muted-foreground">
                {{ t("report.noMatch") }}
              </p>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" size="sm" @click="dialogOpen = false">
            {{ t("common.cancel") }}
          </Button>
          <Button size="sm" @click="submit">{{ t("common.save") }}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
