import { computed, ref, type ComputedRef } from "vue";
import type { ComposerTranslation } from "vue-i18n";
import { getLocalTimeZone, type DateValue } from "@internationalized/date";
import { cmd } from "@/lib/tauri";
import type { HolidayData } from "@/types";

// 模块级缓存:节假日/调休为全集数据(get_holiday_data 一次返回 2004–2026),
// 一次拉取供多个日历组件共享;数据到达前为空集合,日历先按常规周末着色
const holidayDates = ref<string[]>([]);
const workdayDates = ref<string[]>([]);
let loadPromise: Promise<void> | null = null;

/** 法定节假日/调休补班日期集合(响应式;首次调用时从后端拉取,失败回退空集合) */
export function useHolidayData(): {
  holidaySet: ComputedRef<Set<string>>;
  workdaySet: ComputedRef<Set<string>>;
} {
  if (!loadPromise) {
    loadPromise = cmd<HolidayData>("get_holiday_data")
      .then((data) => {
        holidayDates.value = data.holidays;
        workdayDates.value = data.workdays;
      })
      .catch(() => {
        // 拉取失败:保持空集合,日历退化为常规周末着色
      });
  }
  return {
    holidaySet: computed(() => new Set(holidayDates.value)),
    workdaySet: computed(() => new Set(workdayDates.value)),
  };
}

/** 是否周末(本地时区) */
export function isWeekendDate(dv: DateValue): boolean {
  const d = dv.toDate(getLocalTimeZone());
  return d.getDay() === 0 || d.getDay() === 6;
}

/** 日历日期 class:法定节假日红 > 调休补班绿 > 普通周末淡红(与报告历史日历一致) */
export function getHolidayDayClass(
  ds: string,
  weekend: boolean,
  holidaySet: Set<string>,
  workdaySet: Set<string>,
): string {
  const classes: string[] = [];
  if (holidaySet.has(ds)) {
    classes.push("report-calendar-holiday");
  } else if (weekend && !workdaySet.has(ds)) {
    classes.push("report-calendar-weekend");
  }
  if (workdaySet.has(ds)) {
    classes.push("report-calendar-makeup");
  }
  return classes.join(" ");
}

/** 日历日期悬停提示:节假日/调休上班/周末 */
export function getHolidayDayTitle(
  ds: string,
  weekend: boolean,
  holidaySet: Set<string>,
  workdaySet: Set<string>,
  t: ComposerTranslation,
): string | undefined {
  if (holidaySet.has(ds)) return t("reportHistory.holiday");
  if (workdaySet.has(ds)) return t("reportHistory.makeupWorkday");
  if (weekend) return t("reportHistory.weekend");
  return undefined;
}
