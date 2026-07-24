import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { toast } from "vue-sonner";
import { i18n } from "@/i18n";
import { runBatchItems, type BatchItem, type BatchRunOptions } from "@/lib/batch-report";

/**
 * 批量生成报告的全局任务状态。
 * 进度展示从生成弹窗抽离为右下角浮窗(BatchProgressFloat),
 * 弹窗关闭/页面切换后任务与进度仍然存活。
 */
export const useBatchReportStore = defineStore("batchReport", () => {
  /** 当前批次时段列表(空 = 无批次) */
  const items = ref<BatchItem[]>([]);
  const running = ref(false);
  /** 浮窗是否可见(有批次即显示,结束后跳转历史或手动关闭时隐藏) */
  const active = ref(false);
  /** 浮窗展开(进度条+明细) / 折叠(环形进度) */
  const expanded = ref(false);
  /** 取消标志:仅停止派发新任务,运行中的任务自然完成 */
  let cancelFlag = false;

  const stats = computed(() => {
    const list = items.value;
    const finished = list.filter((i) => i.status !== "pending" && i.status !== "running").length;
    return {
      total: list.length,
      finished,
      done: list.filter((i) => i.status === "done").length,
      skipped: list.filter((i) => i.status.startsWith("skipped") || i.status === "cancelled")
        .length,
      failed: list.filter((i) => i.status === "failed").length,
    };
  });

  /** 0-1 进度,驱动环形/条形进度 */
  const progress = computed(() =>
    stats.value.total ? stats.value.finished / stats.value.total : 0,
  );

  /** 当前批次是否已全部结束(成功/跳过/失败/取消) */
  const settled = computed(() => active.value && !running.value);

  /** 启动批量生成;同时只允许一个批次在跑(调用方需先禁用入口) */
  async function start(batch: BatchItem[], options: BatchRunOptions) {
    if (running.value) {
      return;
    }
    items.value = batch;
    active.value = true;
    expanded.value = true;
    running.value = true;
    cancelFlag = false;
    try {
      await runBatchItems(batch, options, () => cancelFlag);
    } finally {
      running.value = false;
    }
    const s = stats.value;
    const message = i18n.global.t("report.batchDoneToast", {
      done: s.done,
      skipped: s.skipped,
      failed: s.failed,
    });
    if (s.failed) {
      toast.warning(message);
    } else {
      toast.success(message);
    }
  }

  function cancel() {
    cancelFlag = true;
  }

  function toggleExpanded() {
    expanded.value = !expanded.value;
  }

  /** 关闭浮窗(仅批次结束后允许) */
  function dismiss() {
    if (running.value) {
      return;
    }
    active.value = false;
    expanded.value = false;
    items.value = [];
  }

  return {
    items,
    running,
    active,
    expanded,
    stats,
    progress,
    settled,
    start,
    cancel,
    toggleExpanded,
    dismiss,
  };
});
