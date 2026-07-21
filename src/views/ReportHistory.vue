<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";
import {
  ArrowLeft,
  ChevronRight,
  Loader2,
  Trash2,
  X,
} from "@lucide/vue";
import { Markdown, type ControlsConfig } from "vue-stream-markdown";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cmd } from "@/lib/tauri";
import { formatCommitTime } from "@/lib/format";
import { useSettingsStore } from "@/stores/settings";
import { useProjectsStore } from "@/stores/projects";
import type { ReportHistoryDetail, ReportHistoryItem } from "@/types";

const { t } = useI18n();
const router = useRouter();
const settings = useSettingsStore();
const projectStore = useProjectsStore();

const items = ref<ReportHistoryItem[]>([]);
const loading = ref(false);
const selectedId = ref<number | null>(null);
const detail = ref<ReportHistoryDetail | null>(null);
const detailLoading = ref(false);

// filter
const filterProjectId = ref<number | null>(null);

const activeProjects = computed(() => projectStore.projects.filter((p) => !p.archived_at));

const controls: ControlsConfig = {
  table: { copy: true, download: true, fullscreen: true },
  code: { copy: true, collapse: true },
};
const detachedThemeEl = document.createElement("div");
const themeElement = () => detachedThemeEl;

// commit collapsible state
const commitOpen = ref<Record<string, boolean>>({});

// ── data loading ───────────────────────────────────────────────────────

async function loadList() {
  loading.value = true;
  try {
    items.value = await cmd<ReportHistoryItem[]>("list_report_history", {
      limit: 100,
      offset: 0,
      ...(filterProjectId.value ? { projectId: filterProjectId.value } : {}),
    });
  } catch (e) {
    toast.error(t("reportHistory.loadFailed"));
  } finally {
    loading.value = false;
  }
}

async function loadDetail(id: number) {
  detailLoading.value = true;
  selectedId.value = id;
  commitOpen.value = {};
  try {
    detail.value = await cmd<ReportHistoryDetail>("get_report_history", { id });
  } catch (e) {
    toast.error(t("reportHistory.loadFailed"));
    detail.value = null;
  } finally {
    detailLoading.value = false;
  }
}

async function deleteReport(id: number) {
  if (!confirm(t("reportHistory.deleteConfirm"))) return;
  try {
    await cmd("delete_report_history", { id });
    items.value = items.value.filter((i) => i.id !== id);
    if (selectedId.value === id) {
      selectedId.value = null;
      detail.value = null;
    }
    toast.success(t("reportHistory.deleted"));
  } catch (e) {
    toast.error(String(e));
  }
}

function formatDate(d: string) {
  const date = new Date(d + "T00:00:00");
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${m}-${day}`;
}

function formatCreated(ts: number) {
  const d = new Date(ts * 1000);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  return `${y}-${m}-${day} ${hh}:${mm}`;
}

function clearFilter() {
  filterProjectId.value = null;
}

// ── init ───────────────────────────────────────────────────────────────

watch(filterProjectId, () => loadList(), { immediate: true });
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- header -->
    <header class="flex shrink-0 items-center gap-2 border-b px-4 py-2.5">
      <Button
        variant="ghost"
        size="icon"
        class="h-8 w-8"
        :title="t('reportHistory.back')"
        @click="router.push('/')"
      >
        <ArrowLeft class="h-4 w-4" />
      </Button>
      <h1 class="text-sm font-semibold">{{ t("reportHistory.title") }}</h1>
    </header>

    <!-- content: left list + right detail -->
    <div class="flex min-h-0 flex-1">
      <!-- left: list -->
      <div class="flex w-80 shrink-0 flex-col border-r">
        <!-- filter -->
        <div class="flex items-center gap-1 border-b px-3 py-2">
          <select
            v-model="filterProjectId"
            class="h-7 flex-1 rounded-md border bg-background px-2 text-xs"
          >
            <option :value="null">{{ t("reportHistory.allProjects") }}</option>
            <option v-for="p in activeProjects" :key="p.id" :value="p.id">
              {{ p.name }}
            </option>
          </select>
          <Button
            v-if="filterProjectId"
            variant="ghost"
            size="icon"
            class="h-7 w-7"
            @click="clearFilter"
          >
            <X class="h-3 w-3" />
          </Button>
        </div>

        <!-- list -->
        <ScrollArea class="min-h-0 flex-1">
          <div v-if="loading" class="flex items-center justify-center py-12">
            <Loader2 class="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
          <div
            v-else-if="!items.length"
            class="flex flex-col items-center gap-2 px-4 py-12 text-center text-sm text-muted-foreground"
          >
            <p>{{ t("reportHistory.empty") }}</p>
            <p class="text-xs">{{ t("reportHistory.emptyHint") }}</p>
          </div>
          <div v-else class="flex flex-col">
            <button
              v-for="item in items"
              :key="item.id"
              type="button"
              class="flex flex-col gap-0.5 border-b px-3 py-2.5 text-left transition-colors hover:bg-accent"
              :class="selectedId === item.id && 'bg-accent'"
              @click="loadDetail(item.id)"
            >
              <div class="flex items-center justify-between gap-2">
                <span class="text-sm font-medium">{{ formatCreated(item.createdAt) }}</span>
                <Badge variant="secondary" class="text-[11px] shrink-0">
                  {{ t("reportHistory.totalCommits", { count: item.totalCommits }) }}
                </Badge>
              </div>
              <div class="flex flex-wrap gap-1 text-[11px] text-muted-foreground">
                <span>{{ item.dateFrom === item.dateTo ? formatDate(item.dateFrom) : `${formatDate(item.dateFrom)} ~ ${formatDate(item.dateTo)}` }}</span>
                <span v-if="item.projectNames.length" class="truncate max-w-48" :title="item.projectNames.join(', ')">
                  · {{ item.projectNames.slice(0, 2).join(", ")
                  }}{{ item.projectNames.length > 2 ? ` +${item.projectNames.length - 2}` : "" }}
                </span>
              </div>
            </button>
          </div>
        </ScrollArea>
      </div>

      <!-- right: detail -->
      <div class="flex min-h-0 min-w-0 flex-1 flex-col">
        <template v-if="!selectedId">
          <div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">
            {{ t("reportHistory.emptyHint") }}
          </div>
        </template>
        <template v-else-if="detailLoading">
          <div class="flex flex-1 items-center justify-center">
            <Loader2 class="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        </template>
        <template v-else-if="detail">
          <!-- toolbar -->
          <div class="flex shrink-0 items-center justify-between border-b px-3 py-1.5">
            <div class="flex items-center gap-2 text-xs text-muted-foreground">
              <span>{{ t("reportHistory.generatedAt") }}: {{ formatCreated(detail.createdAt) }}</span>
              <span>{{ detail.dateFrom === detail.dateTo ? formatDate(detail.dateFrom) : `${formatDate(detail.dateFrom)} ~ ${formatDate(detail.dateTo)}` }}</span>
            </div>
            <Button
              variant="ghost"
              size="sm"
              class="h-6 gap-1 px-2 text-xs text-destructive hover:text-destructive"
              @click="deleteReport(detail.id)"
            >
              <Trash2 class="h-3 w-3" />
              {{ t("common.delete") }}
            </Button>
          </div>

          <!-- content: commits left + markdown right -->
          <div class="flex min-h-0 min-w-0 flex-1 gap-0">
            <!-- commits panel -->
            <div class="w-72 shrink-0 overflow-y-auto border-r p-2">
              <h3 class="mb-2 text-xs font-medium text-muted-foreground">
                {{ t("reportHistory.commits") }}
              </h3>
              <div class="flex flex-col gap-1">
                <Collapsible
                  v-for="c in detail.commits"
                  :key="c.projectName"
                  v-slot="{ open: expanded }"
                  :open="commitOpen[c.projectName]"
                  @update:open="commitOpen[c.projectName] = $event"
                >
                  <CollapsibleTrigger
                    class="flex w-full cursor-pointer items-center gap-1.5 rounded px-1.5 py-1 text-left text-xs hover:bg-accent"
                  >
                    <ChevronRight
                      class="h-3 w-3 shrink-0 text-muted-foreground transition-transform"
                      :class="{ 'rotate-90': expanded }"
                    />
                    <span class="min-w-0 flex-1 truncate">{{ c.projectName }}</span>
                    <span class="shrink-0 text-muted-foreground">{{ c.commits.length }}</span>
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <div
                      v-if="c.commits.length"
                      class="ml-3 max-h-40 overflow-y-auto border-l"
                    >
                      <div
                        v-for="commit in c.commits"
                        :key="commit.hash + commit.date"
                        class="flex min-w-0 items-center gap-1.5 border-b px-2 py-0.5 text-[11px]"
                      >
                        <code class="shrink-0 rounded bg-muted px-1 py-px font-mono text-[10px]">
                          {{ commit.hash }}
                        </code>
                        <span class="min-w-0 flex-1 truncate" :title="commit.subject">{{
                          commit.subject
                        }}</span>
                        <span class="shrink-0 text-muted-foreground">{{
                          formatCommitTime(commit.date)
                        }}</span>
                      </div>
                    </div>
                  </CollapsibleContent>
                </Collapsible>
              </div>
            </div>

            <!-- markdown panel -->
            <ScrollArea class="min-h-0 min-w-0 flex-1">
              <div class="p-4 text-sm">
                <Markdown
                  mode="static"
                  :content="detail.result"
                  :controls="controls"
                  :theme-element="themeElement"
                  :locale="settings.language"
                />
              </div>
            </ScrollArea>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>
