import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { planBatchItems } from "@/lib/batch-report";
import type { BatchRange, ReportDateRange } from "@/types";

// Tauri invoke 在 Node 测试环境不可用,改为受控 mock;
// vi.hoisted 保证 mock 工厂在 import 之前注册,避免 import(first) 警告
const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

const baseOptions = {
  dateFrom: "2026-07-20",
  dateTo: "2026-07-26",
  workdaysOnly: false,
  skipExisting: false,
  makeLabel: (a: string, b: string) => `${a}~${b}`,
};

function mockPlanRanges(ranges: BatchRange[]) {
  invokeMock.mockResolvedValueOnce(ranges);
}

function mockListDates(dates: ReportDateRange[]) {
  invokeMock.mockResolvedValueOnce(dates);
}

describe("planBatchItems", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("日报:把后端切分的每个时段原样映射为 BatchItem,默认 pending", async () => {
    mockPlanRanges([
      { dateFrom: "2026-07-20", dateTo: "2026-07-20", isWorkday: true },
      { dateFrom: "2026-07-21", dateTo: "2026-07-21", isWorkday: true },
    ]);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "daily",
    });

    expect(items).toEqual([
      {
        dateFrom: "2026-07-20",
        dateTo: "2026-07-20",
        label: "2026-07-20~2026-07-20",
        status: "pending",
      },
      {
        dateFrom: "2026-07-21",
        dateTo: "2026-07-21",
        label: "2026-07-21~2026-07-21",
        status: "pending",
      },
    ]);
    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith("plan_batch_report_ranges", {
      periodType: "daily",
      dateFrom: "2026-07-20",
      dateTo: "2026-07-26",
    });
  });

  it("周报:透传后端按工作周切分的范围", async () => {
    mockPlanRanges([
      { dateFrom: "2026-07-20", dateTo: "2026-07-26", isWorkday: true },
      { dateFrom: "2026-07-27", dateTo: "2026-08-02", isWorkday: true },
    ]);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "weekly",
    });

    expect(items.map((i) => [i.dateFrom, i.dateTo])).toEqual([
      ["2026-07-20", "2026-07-26"],
      ["2026-07-27", "2026-08-02"],
    ]);
    expect(items.every((i) => i.status === "pending")).toBe(true);
  });

  it("工作日 only(日报):剔除 isWorkday=false 的时段,其余保留", async () => {
    mockPlanRanges([
      { dateFrom: "2026-07-20", dateTo: "2026-07-20", isWorkday: true },
      { dateFrom: "2026-07-21", dateTo: "2026-07-21", isWorkday: false },
      { dateFrom: "2026-07-22", dateTo: "2026-07-22", isWorkday: true },
    ]);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "daily",
      workdaysOnly: true,
    });

    expect(items.map((i) => i.dateTo)).toEqual(["2026-07-20", "2026-07-22"]);
  });

  it("工作日 only 对周报不生效:周报本身已是工作周集合,不应按 isWorkday 过滤", async () => {
    mockPlanRanges([{ dateFrom: "2026-07-20", dateTo: "2026-07-26", isWorkday: false }]);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "weekly",
      workdaysOnly: true,
    });

    expect(items).toHaveLength(1);
  });

  it("跨度上限:不限制由后端切分,前端只透传(后端返回多少就列出多少)", async () => {
    const ranges: BatchRange[] = Array.from({ length: 31 }, (_, i) => ({
      dateFrom: `2026-07-${String(i + 1).padStart(2, "0")}`,
      dateTo: `2026-07-${String(i + 1).padStart(2, "0")}`,
      isWorkday: true,
    }));
    mockPlanRanges(ranges);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "daily",
    });

    expect(items).toHaveLength(31);
  });

  it("skipExisting:日报按 dateTo 匹配,命中的时段标记为 skipped-existing,未命中的仍是 pending", async () => {
    mockPlanRanges([
      { dateFrom: "2026-07-20", dateTo: "2026-07-20", isWorkday: true },
      { dateFrom: "2026-07-21", dateTo: "2026-07-21", isWorkday: true },
      { dateFrom: "2026-07-22", dateTo: "2026-07-22", isWorkday: true },
    ]);
    mockListDates([
      { dateFrom: "2026-07-20", dateTo: "2026-07-20" },
      { dateFrom: "2026-07-22", dateTo: "2026-07-22" },
    ]);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "daily",
      skipExisting: true,
    });

    expect(items[0].status).toBe("skipped-existing");
    expect(items[1].status).toBe("pending");
    expect(items[2].status).toBe("skipped-existing");
    // 调用顺序:先 plan_batch_report_ranges,再 list_report_dates
    expect(invokeMock.mock.calls[0][0]).toBe("plan_batch_report_ranges");
    expect(invokeMock.mock.calls[1][0]).toBe("list_report_dates");
  });

  it("skipExisting:周报按 (dateFrom,dateTo) 配对匹配", async () => {
    mockPlanRanges([
      { dateFrom: "2026-07-20", dateTo: "2026-07-26", isWorkday: true },
      { dateFrom: "2026-07-27", dateTo: "2026-08-02", isWorkday: true },
    ]);
    mockListDates([{ dateFrom: "2026-07-20", dateTo: "2026-07-26" }]);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "weekly",
      skipExisting: true,
    });

    expect(items[0].status).toBe("skipped-existing");
    expect(items[1].status).toBe("pending");
  });

  it("skipExisting + workdaysOnly 可叠加:同时剔除非工作日和已存在", async () => {
    mockPlanRanges([
      { dateFrom: "2026-07-20", dateTo: "2026-07-20", isWorkday: true },
      { dateFrom: "2026-07-21", dateTo: "2026-07-21", isWorkday: false },
      { dateFrom: "2026-07-22", dateTo: "2026-07-22", isWorkday: true },
    ]);
    // 两个工作日时段均命中 existing,叠加后应全部 skipped-existing
    mockListDates([
      { dateFrom: "2026-07-20", dateTo: "2026-07-20" },
      { dateFrom: "2026-07-22", dateTo: "2026-07-22" },
    ]);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "daily",
      skipExisting: true,
      workdaysOnly: true,
    });

    expect(items.map((i) => i.status)).toEqual(["skipped-existing", "skipped-existing"]);
  });

  it("skipExisting + workdaysOnly:非工作日被剔除,工作日中只有命中的 skipped", async () => {
    mockPlanRanges([
      { dateFrom: "2026-07-20", dateTo: "2026-07-20", isWorkday: true },
      { dateFrom: "2026-07-21", dateTo: "2026-07-21", isWorkday: false },
      { dateFrom: "2026-07-22", dateTo: "2026-07-22", isWorkday: true },
    ]);
    mockListDates([{ dateFrom: "2026-07-20", dateTo: "2026-07-20" }]);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "daily",
      skipExisting: true,
      workdaysOnly: true,
    });

    expect(items.map((i) => i.status)).toEqual(["skipped-existing", "pending"]);
  });

  it("skipExisting=false:不调用 list_report_dates", async () => {
    mockPlanRanges([{ dateFrom: "2026-07-20", dateTo: "2026-07-20", isWorkday: true }]);

    await planBatchItems({
      ...baseOptions,
      periodType: "daily",
    });

    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock.mock.calls[0][0]).toBe("plan_batch_report_ranges");
  });

  it("空项目列表:后端无范围时返回空数组,行为正常", async () => {
    mockPlanRanges([]);
    // skipExisting=true 时,即使 ranges 为空也会先调 list_report_dates
    mockListDates([]);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "daily",
      skipExisting: true,
    });

    expect(items).toEqual([]);
    expect(invokeMock).toHaveBeenCalledTimes(2);
  });

  it("skipExisting=false 时即使 ranges 为空也只调一次 plan_batch_report_ranges", async () => {
    mockPlanRanges([]);

    const items = await planBatchItems({
      ...baseOptions,
      periodType: "daily",
      skipExisting: false,
    });

    expect(items).toEqual([]);
    expect(invokeMock).toHaveBeenCalledTimes(1);
  });
});
