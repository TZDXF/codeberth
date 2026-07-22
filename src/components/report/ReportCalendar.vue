<script setup lang="ts">
import { computed, ref, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { ChevronLeft, ChevronRight } from "@lucide/vue";
import { getLocalTimeZone, parseDate, today, type DateValue } from "@internationalized/date";
import { CalendarRoot } from "reka-ui";
import {
  CalendarCell,
  CalendarCellTrigger,
  CalendarGrid,
  CalendarGridBody,
  CalendarGridHead,
  CalendarGridRow,
  CalendarHeadCell,
  CalendarHeader,
  CalendarHeading,
  CalendarNextButton,
  CalendarPrevButton,
} from "@/components/ui/calendar";
import type { CalendarMeta } from "@/types";
import { useSettingsStore } from "@/stores/settings";

const props = defineProps<{
  modelValue: string | null;
  calendarData: CalendarMeta | null;
  /** 周报时间范围高亮("YYYY-MM-DD" 起止,闭区间);null 表示不高亮 */
  highlightRange?: { start: string; end: string } | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [date: string];
  "month-change": [year: number, month: number];
}>();

const { t } = useI18n();
const settings = useSettingsStore();

const holidaySet = computed(() => new Set(props.calendarData?.holidays ?? []));
const workdaySet = computed(() => new Set(props.calendarData?.workdays ?? []));
const reportDates = computed(() => props.calendarData?.dates ?? {});

/** 使用浏览器 Intl API 生成周一~周日的短标签，避免 t() 数组不可靠 */
const weekDayLabels = computed(() => {
  const lang = settings.language;
  const fmt = new Intl.DateTimeFormat(lang, { weekday: lang === "zh-CN" ? "narrow" : "short" });
  // 2024-01-01 is Monday
  const mon = new Date(2024, 0, 1);
  return Array.from({ length: 7 }, (_, i) => {
    const d = new Date(mon);
    d.setDate(1 + i);
    return fmt.format(d);
  });
});

// ── CalendarRoot model: string ↔ DateValue ────────────────────────────

const innerValue = ref<DateValue>() as Ref<DateValue | undefined>;

// parent → calendar
watch(
  () => props.modelValue,
  (v) => {
    if (!v) {
      innerValue.value = undefined;
      return;
    }
    try {
      innerValue.value = parseDate(v);
    } catch {
      // ignore invalid date
    }
  },
  { immediate: true },
);

// calendar → parent
watch(innerValue, (v) => {
  if (v) {
    emit("update:modelValue", v.toString());
  }
});

function onCalendarUpdate(v: DateValue | undefined) {
  innerValue.value = v;
}

// ── month navigation ────────────────────────────────────────────────────

const placeholder = ref(today(getLocalTimeZone())) as Ref<DateValue>;
const lastPlaceholder = ref("");

watch(placeholder, (val) => {
  const ds = val.toString();
  if (ds === lastPlaceholder.value) return;
  lastPlaceholder.value = ds;
  emit("month-change", val.year, val.month);
});

// ── helpers ─────────────────────────────────────────────────────────────

function isWeekend(dv: DateValue): boolean {
  const d = dv.toDate(getLocalTimeZone());
  return d.getDay() === 0 || d.getDay() === 6;
}

function getDayClass(dv: DateValue): string {
  const ds = dv.toString();
  const classes: string[] = [];
  if (holidaySet.value.has(ds)) {
    classes.push("report-calendar-holiday");
  } else if (isWeekend(dv) && !workdaySet.value.has(ds)) {
    classes.push("report-calendar-weekend");
  }
  if (workdaySet.value.has(ds)) {
    classes.push("report-calendar-makeup");
  }
  return classes.join(" ");
}

function getDayTitle(dv: DateValue): string | undefined {
  const ds = dv.toString();
  if (holidaySet.value.has(ds)) return t("reportHistory.holiday");
  if (workdaySet.value.has(ds)) return t("reportHistory.makeupWorkday");
  if (isWeekend(dv)) return t("reportHistory.weekend");
  return undefined;
}

function getReportCount(dv: DateValue): number {
  return reportDates.value[dv.toString()] ?? 0;
}

/** 周报范围高亮:日期是否落在 highlightRange 闭区间内(字符串即 ISO 日期,可直接字典序比较) */
function isInHighlightRange(dv: DateValue): boolean {
  const r = props.highlightRange;
  if (!r) return false;
  const ds = dv.toString();
  return ds >= r.start && ds <= r.end;
}

/** 范围高亮 class:中段去圆角连成带,起止日保留外侧圆角(完整类名供 Tailwind 扫描) */
function getHighlightClass(dv: DateValue): string {
  if (!isInHighlightRange(dv)) return "";
  const r = props.highlightRange!;
  const ds = dv.toString();
  if (ds === r.start && ds === r.end) return "bg-primary/10";
  if (ds === r.start) return "bg-primary/10 rounded-r-none";
  if (ds === r.end) return "bg-primary/10 rounded-l-none";
  return "bg-primary/10 rounded-none";
}

/** Type helper: narrow grid cell date to DateValue for CalendarCellTrigger */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function asDateValue(dv: any): DateValue {
  return dv as DateValue;
}
</script>

<template>
  <CalendarRoot
    v-slot="{ grid }"
    :model-value="innerValue"
    @update:model-value="onCalendarUpdate"
    v-model:placeholder="placeholder"
    :week-starts-on="1"
    :locale="settings.language"
    class="p-1"
  >
    <CalendarHeader class="mb-1">
      <nav class="absolute inset-x-0 top-0 flex items-center justify-between px-1">
        <CalendarPrevButton
          class="size-7 bg-transparent p-0 opacity-50 hover:opacity-100 border rounded-md inline-flex items-center justify-center"
        >
          <ChevronLeft class="size-4" />
        </CalendarPrevButton>
        <CalendarNextButton
          class="size-7 bg-transparent p-0 opacity-50 hover:opacity-100 border rounded-md inline-flex items-center justify-center"
        >
          <ChevronRight class="size-4" />
        </CalendarNextButton>
      </nav>
      <CalendarHeading class="text-sm font-medium" />
    </CalendarHeader>

    <CalendarGrid v-for="month in grid" :key="month.value.toString()">
      <CalendarGridHead>
        <CalendarGridRow>
          <CalendarHeadCell
            v-for="day in weekDayLabels"
            :key="day"
            class="text-muted-foreground flex-1 font-normal text-[0.8rem] text-center"
          >
            {{ day }}
          </CalendarHeadCell>
        </CalendarGridRow>
      </CalendarGridHead>
      <CalendarGridBody>
        <CalendarGridRow v-for="(row, _idx) in month.rows" :key="_idx" class="flex">
          <CalendarCell
            v-for="cellDate in row"
            :key="cellDate.toString()"
            :date="asDateValue(cellDate)"
          >
            <CalendarCellTrigger
              :day="asDateValue(cellDate)"
              :month="month.value"
              :class="[getDayClass(cellDate), getHighlightClass(cellDate)]"
              :title="getDayTitle(cellDate)"
              class="relative flex size-8 items-center justify-center rounded-md p-0 font-normal text-sm"
            >
              {{ cellDate.day }}
              <span
                v-if="getReportCount(cellDate) > 0"
                class="absolute bottom-0.5 left-1/2 -translate-x-1/2 h-1 w-1 rounded-full bg-primary"
              />
            </CalendarCellTrigger>
          </CalendarCell>
        </CalendarGridRow>
      </CalendarGridBody>
    </CalendarGrid>
  </CalendarRoot>
</template>

<style scoped>
/* 法定节假日：红色 */
:deep(.report-calendar-holiday) {
  color: #f87171;
}
/* 普通周末：淡红 */
:deep(.report-calendar-weekend) {
  color: rgb(248 113 113 / 0.7);
}
/* 调休上班日：绿色 */
:deep(.report-calendar-makeup) {
  color: #4ade80;
}
</style>
