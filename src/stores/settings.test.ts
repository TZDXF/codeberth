import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve as resolvePath } from "node:path";

/**
 * 任务描述 ── settings store legacy migration 清除
 *
 * 旧版本在浏览器 localStorage 里直接存 `codeberth:view` / `codeberth:sort`
 * 这两个键。settings store (tauri-plugin-store) 落地后,需要在初始化时把它们
 * 搬过去并删除旧键。本次清理之后:
 *
 * 1. `useSettingsStore` 不再 export 一个叫 `migrateLegacyLocalStorage` 的内部方法,
 *    也不再有 `codeberth:view` / `codeberth:sort` 的本地存储读写代码。
 * 2. `init()` 不能再去访问 `window.localStorage` 的这两个旧键(不应触发读/写),
 *    以免后端 settings 启动时把"权威来源"误解。
 *
 * 由于 store 在 src/stores 下不再导出 `migrateLegacyLocalStorage`,
 * 测试断言两层清洁度(纯静态检查,避免在 Node 环境 instantiate Pinia store
 * 那一连串的 browser-only 副作用):
 *
 *  - 模块源码中不再包含 legacy 字符串 + 函数名
 *  - 模块导出集合里不能再含 `migrateLegacyLocalStorage`
 */

const SRC_SETTINGS = resolvePath(__dirname, "./settings.ts");

interface SourceCheck {
  /** 不应在源码里出现的字符串片段(简化的形式) */
  forbidden: string[];
  /** 应保留的字符串片段 */
  required: string[];
}

const CHECKS: { name: string; check: SourceCheck }[] = [
  {
    name: "settings store 不再包含 legacy localStorage 迁移代码",
    check: {
      forbidden: [
        "migrateLegacyLocalStorage",
        'getItem("codeberth:view")',
        'getItem("codeberth:sort")',
        'removeItem("codeberth:view")',
        'removeItem("codeberth:sort")',
      ],
      required: ["projectsViewMode", "projectsSortKey"],
    },
  },
];

describe("settings store legacy migration 已被清理", () => {
  it("src/stores/settings.ts 不引用 legacy localStorage 迁移", () => {
    const src = readFileSync(SRC_SETTINGS, "utf8");
    for (const { name, check } of CHECKS) {
      for (const fragment of check.forbidden) {
        expect(src.includes(fragment), `${name}: 期望源码中不包含 "${fragment}",但找到了`).toBe(
          false,
        );
      }
      for (const fragment of check.required) {
        expect(
          src.includes(fragment),
          `${name}: 期望源码仍包含 "${fragment}"(核心能力不应被一并移除)`,
        ).toBe(true);
      }
    }
  });

  it("settings store 模块导出集合里不包含 migrateLegacyLocalStorage", () => {
    // 通过静态扫描 `return { ..., <name>: <value> }` 块来找出被显式 return 的
    // store API;若源码不再 return migrateLegacyLocalStorage,则该名字不出现在
    // 顶层 export / return 列表中——这是更明确的契约断言。
    const src = readFileSync(SRC_SETTINGS, "utf8");
    expect(
      /\bmigrateLegacyLocalStorage\s*[:,)]/m.test(src),
      "settings.ts 中不应再以属性键形式出现 migrateLegacyLocalStorage",
    ).toBe(false);
  });
});
