import { ref } from "vue";
import { defineStore } from "pinia";
import { load, type Store } from "@tauri-apps/plugin-store";
import { homeDir, join } from "@tauri-apps/api/path";
import { setI18nLocale, type SupportedLocale } from "@/i18n";
import { OPEN_WITH_OPTIONS } from "@/lib/open-with";
import type { EditorKind } from "@/types";

export type ThemeMode = "system" | "light" | "dark";
export type ThemeSkin = "default" | "island";
export type MdTheme = "default" | "github" | "notion" | "serif";
export type Language = SupportedLocale;
export type ProjectsViewMode = "grid" | "table";
export type ProjectsSortKey = "name" | "updated" | "created";

// 应用数据统一存放于用户主目录下的 .repomeow 目录(与 Rust 端 APP_DATA_DIR_NAME 保持一致)
const APP_DATA_DIR_NAME = ".repomeow";
const STORE_FILE = "settings.json";

// AI 接入参数(OpenAI Chat Completions 兼容):baseUrl/apiKey/model 均无默认值,
// 由用户在设置页填写;任一缺失时调用方需先校验。

export const useSettingsStore = defineStore("settings", () => {
  const theme = ref<ThemeMode>("system");
  const themeSkin = ref<ThemeSkin>("default");
  const mdTheme = ref<MdTheme>("default");
  const language = ref<Language>("zh-CN");
  const defaultOpenWith = ref<EditorKind>("explorer");
  const aiBaseUrl = ref("");
  const aiApiKey = ref("");
  const aiModel = ref("");
  /** AI 调用并发上限(1-5),适用于批量生成报告等所有 AI 请求场景 */
  const aiConcurrency = ref(2);
  /** 项目列表视图模式(grid / table) */
  const projectsViewMode = ref<ProjectsViewMode>("grid");
  /** 项目列表排序方式 */
  const projectsSortKey = ref<ProjectsSortKey>("name");
  /** 启动时自动检查更新 */
  const autoCheckUpdate = ref(true);

  let fileStore: Store | null = null;
  let initialized = false;

  const systemDark = window.matchMedia("(prefers-color-scheme: dark)");

  // 将主题相关键镜像到 localStorage,供 index.html 内联脚本在首帧绘制前同步读取,
  // 避免异步加载 settings.json 期间的主题闪烁。权威来源仍是 tauri-plugin-store。
  function syncThemeCache() {
    try {
      window.localStorage.setItem(
        "repomeow:theme-cache",
        JSON.stringify({
          theme: theme.value,
          themeSkin: themeSkin.value,
          mdTheme: mdTheme.value,
        }),
      );
    } catch {
      /* localStorage 不可用时静默降级:首屏可能仍闪烁一次,不影响功能 */
    }
  }

  function applyTheme() {
    const dark = theme.value === "dark" || (theme.value === "system" && systemDark.matches);
    const root = document.documentElement;
    root.classList.toggle("dark", dark);
    if (themeSkin.value === "island") {
      root.setAttribute("data-theme", "island");
    } else {
      root.removeAttribute("data-theme");
    }
    syncThemeCache();
  }

  function applyMdTheme() {
    const root = document.documentElement;
    if (mdTheme.value === "default") {
      root.removeAttribute("data-md-theme");
    } else {
      root.setAttribute("data-md-theme", mdTheme.value);
    }
    syncThemeCache();
  }

  function onSystemThemeChange() {
    if (theme.value === "system") applyTheme();
  }

  // ── lifecycle ─────────────────────────────────────────────

  async function init() {
    if (initialized) return;
    initialized = true;

    fileStore = await load(await join(await homeDir(), APP_DATA_DIR_NAME, STORE_FILE), {
      defaults: {
        theme: "system",
        themeSkin: "default",
        mdTheme: "default",
        language: "zh-CN",
        defaultOpenWith: "explorer",
        aiBaseUrl: "",
        aiApiKey: "",
        aiModel: "",
        aiConcurrency: "2",
        projectsViewMode: "grid",
        projectsSortKey: "name",
        autoCheckUpdate: "true",
      },
    });
    const savedTheme = await fileStore.get<ThemeMode>("theme");
    if (savedTheme === "light" || savedTheme === "dark" || savedTheme === "system") {
      theme.value = savedTheme;
    }
    const savedSkin = await fileStore.get<ThemeSkin>("themeSkin");
    if (savedSkin === "default" || savedSkin === "island") {
      themeSkin.value = savedSkin;
    }
    const savedMdTheme = await fileStore.get<MdTheme>("mdTheme");
    if (
      savedMdTheme === "default" ||
      savedMdTheme === "github" ||
      savedMdTheme === "notion" ||
      savedMdTheme === "serif"
    ) {
      mdTheme.value = savedMdTheme;
    }
    const savedLanguage = await fileStore.get<Language>("language");
    if (savedLanguage === "zh-CN" || savedLanguage === "en-US") {
      language.value = savedLanguage;
      setI18nLocale(savedLanguage);
    }
    const savedOpenWith = await fileStore.get<string>("defaultOpenWith");
    // 以 OPEN_WITH_OPTIONS 为白名单校验,新增打开方式无需改这里
    if (OPEN_WITH_OPTIONS.some((opt) => opt.kind === savedOpenWith)) {
      defaultOpenWith.value = savedOpenWith as EditorKind;
    }
    // AI 配置为自由文本:trim 后非空才赋值,空值保持初始空(无默认值可回退)
    const savedAiBaseUrl = await fileStore.get<string>("aiBaseUrl");
    if (typeof savedAiBaseUrl === "string" && savedAiBaseUrl.trim()) {
      aiBaseUrl.value = savedAiBaseUrl.trim();
    }
    const savedAiApiKey = await fileStore.get<string>("aiApiKey");
    if (typeof savedAiApiKey === "string") {
      aiApiKey.value = savedAiApiKey;
    }
    const savedAiModel = await fileStore.get<string>("aiModel");
    if (typeof savedAiModel === "string" && savedAiModel.trim()) {
      aiModel.value = savedAiModel.trim();
    }
    // 并发上限存为字符串,解析后限制在 1-5,非法值回退默认 2
    const savedConcurrency = await fileStore.get<string>("aiConcurrency");
    if (typeof savedConcurrency === "string") {
      const n = Number.parseInt(savedConcurrency, 10);
      if (Number.isFinite(n)) {
        aiConcurrency.value = Math.min(5, Math.max(1, n));
      }
    }
    // 视图模式:白名单校验,非法值回退 grid
    const savedViewMode = await fileStore.get<ProjectsViewMode>("projectsViewMode");
    if (savedViewMode === "grid" || savedViewMode === "table") {
      projectsViewMode.value = savedViewMode;
    }
    // 排序键:白名单校验,非法值回退 name
    const savedSortKey = await fileStore.get<ProjectsSortKey>("projectsSortKey");
    if (savedSortKey === "name" || savedSortKey === "updated" || savedSortKey === "created") {
      projectsSortKey.value = savedSortKey;
    }
    // 自动检查更新:存为字符串 "true"/"false",非法值回退 true
    const savedAutoCheckUpdate = await fileStore.get<string>("autoCheckUpdate");
    if (savedAutoCheckUpdate === "true" || savedAutoCheckUpdate === "false") {
      autoCheckUpdate.value = savedAutoCheckUpdate === "true";
    }
    applyTheme();
    applyMdTheme();
    systemDark.addEventListener("change", onSystemThemeChange);
  }

  async function persist(key: string, value: string) {
    if (!fileStore) return;
    await fileStore.set(key, value);
    await fileStore.save();
  }

  async function setTheme(value: ThemeMode) {
    theme.value = value;
    applyTheme();
    await persist("theme", value);
  }

  async function setThemeSkin(value: ThemeSkin) {
    themeSkin.value = value;
    applyTheme();
    await persist("themeSkin", value);
  }

  async function setMdTheme(value: MdTheme) {
    mdTheme.value = value;
    applyMdTheme();
    await persist("mdTheme", value);
  }

  async function setLanguage(value: Language) {
    language.value = value;
    setI18nLocale(value);
    await persist("language", value);
  }

  async function setDefaultOpenWith(value: EditorKind) {
    defaultOpenWith.value = value;
    await persist("defaultOpenWith", value);
  }

  async function setAiBaseUrl(value: string) {
    aiBaseUrl.value = value.trim();
    await persist("aiBaseUrl", aiBaseUrl.value);
  }

  async function setAiApiKey(value: string) {
    aiApiKey.value = value.trim();
    await persist("aiApiKey", aiApiKey.value);
  }

  async function setAiModel(value: string) {
    aiModel.value = value.trim();
    await persist("aiModel", aiModel.value);
  }

  async function setAiConcurrency(value: number) {
    const n = Math.min(5, Math.max(1, Math.round(value)));
    aiConcurrency.value = n;
    await persist("aiConcurrency", String(n));
  }

  async function setProjectsViewMode(value: ProjectsViewMode) {
    if (value !== "grid" && value !== "table") return;
    projectsViewMode.value = value;
    await persist("projectsViewMode", value);
  }

  async function setProjectsSortKey(value: ProjectsSortKey) {
    if (value !== "name" && value !== "updated" && value !== "created") return;
    projectsSortKey.value = value;
    await persist("projectsSortKey", value);
  }

  async function setAutoCheckUpdate(value: boolean) {
    autoCheckUpdate.value = value;
    await persist("autoCheckUpdate", String(value));
  }

  return {
    theme,
    themeSkin,
    mdTheme,
    language,
    defaultOpenWith,
    aiBaseUrl,
    aiApiKey,
    aiModel,
    aiConcurrency,
    projectsViewMode,
    projectsSortKey,
    autoCheckUpdate,
    init,
    setTheme,
    setThemeSkin,
    setMdTheme,
    setLanguage,
    setDefaultOpenWith,
    setAiBaseUrl,
    setAiApiKey,
    setAiModel,
    setAiConcurrency,
    setProjectsViewMode,
    setProjectsSortKey,
    setAutoCheckUpdate,
  };
});
