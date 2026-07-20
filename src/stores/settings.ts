import { ref } from "vue";
import { defineStore } from "pinia";
import { load, type Store } from "@tauri-apps/plugin-store";
import { homeDir, join } from "@tauri-apps/api/path";
import { setI18nLocale, type SupportedLocale } from "@/i18n";
import type { EditorKind } from "@/types";

export type ThemeMode = "system" | "light" | "dark";
export type ThemeSkin = "default" | "island";
export type MdTheme = "default" | "github" | "notion" | "serif";
export type Language = SupportedLocale;

// 应用数据统一存放于用户主目录下的 .pm 目录(与 Rust 端 APP_DATA_DIR_NAME 保持一致)
const APP_DATA_DIR_NAME = ".pm";
const STORE_FILE = "settings.json";

/** AI 服务默认接入参数(OpenAI Chat Completions 兼容) */
export const AI_DEFAULT_BASE_URL = "https://api.openai.com/v1";
export const AI_DEFAULT_MODEL = "gpt-4o-mini";

export const useSettingsStore = defineStore("settings", () => {
  const theme = ref<ThemeMode>("system");
  const themeSkin = ref<ThemeSkin>("default");
  const mdTheme = ref<MdTheme>("default");
  const language = ref<Language>("zh-CN");
  const defaultOpenWith = ref<EditorKind>("explorer");
  const aiBaseUrl = ref(AI_DEFAULT_BASE_URL);
  const aiApiKey = ref("");
  const aiModel = ref(AI_DEFAULT_MODEL);

  let fileStore: Store | null = null;
  let initialized = false;

  const systemDark = window.matchMedia("(prefers-color-scheme: dark)");

  function applyTheme() {
    const dark = theme.value === "dark" || (theme.value === "system" && systemDark.matches);
    const root = document.documentElement;
    root.classList.toggle("dark", dark);
    if (themeSkin.value === "island") {
      root.setAttribute("data-theme", "island");
    } else {
      root.removeAttribute("data-theme");
    }
  }

  function applyMdTheme() {
    const root = document.documentElement;
    if (mdTheme.value === "default") {
      root.removeAttribute("data-md-theme");
    } else {
      root.setAttribute("data-md-theme", mdTheme.value);
    }
  }

  function onSystemThemeChange() {
    if (theme.value === "system") applyTheme();
  }

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
        aiBaseUrl: AI_DEFAULT_BASE_URL,
        aiApiKey: "",
        aiModel: AI_DEFAULT_MODEL,
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
    const savedOpenWith = await fileStore.get<EditorKind>("defaultOpenWith");
    if (
      savedOpenWith === "vscode" ||
      savedOpenWith === "explorer" ||
      savedOpenWith === "terminal"
    ) {
      defaultOpenWith.value = savedOpenWith;
    }
    // AI 配置为自由文本:空值回退默认(baseUrl/model),apiKey 允许为空
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
    aiBaseUrl.value = value.trim() || AI_DEFAULT_BASE_URL;
    await persist("aiBaseUrl", aiBaseUrl.value);
  }

  async function setAiApiKey(value: string) {
    aiApiKey.value = value.trim();
    await persist("aiApiKey", aiApiKey.value);
  }

  async function setAiModel(value: string) {
    aiModel.value = value.trim() || AI_DEFAULT_MODEL;
    await persist("aiModel", aiModel.value);
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
    init,
    setTheme,
    setThemeSkin,
    setMdTheme,
    setLanguage,
    setDefaultOpenWith,
    setAiBaseUrl,
    setAiApiKey,
    setAiModel,
  };
});
