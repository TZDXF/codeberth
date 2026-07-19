import { ref } from "vue";
import { defineStore } from "pinia";
import { load, type Store } from "@tauri-apps/plugin-store";
import { setI18nLocale, type SupportedLocale } from "@/i18n";
import type { EditorKind } from "@/types";

export type ThemeMode = "system" | "light" | "dark";
export type ThemeSkin = "default" | "island";
export type Language = SupportedLocale;

const STORE_FILE = "settings.json";

export const useSettingsStore = defineStore("settings", () => {
  const theme = ref<ThemeMode>("system");
  const themeSkin = ref<ThemeSkin>("default");
  const language = ref<Language>("zh-CN");
  const defaultOpenWith = ref<EditorKind>("explorer");

  let fileStore: Store | null = null;
  let initialized = false;

  const systemDark = window.matchMedia("(prefers-color-scheme: dark)");

  function applyTheme() {
    const dark =
      theme.value === "dark" || (theme.value === "system" && systemDark.matches);
    const root = document.documentElement;
    root.classList.toggle("dark", dark);
    if (themeSkin.value === "island") {
      root.setAttribute("data-theme", "island");
    } else {
      root.removeAttribute("data-theme");
    }
  }

  function onSystemThemeChange() {
    if (theme.value === "system") applyTheme();
  }

  async function init() {
    if (initialized) return;
    initialized = true;

    fileStore = await load(STORE_FILE, {
      defaults: {
        theme: "system",
        themeSkin: "default",
        language: "zh-CN",
        defaultOpenWith: "explorer",
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
    applyTheme();
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

  async function setLanguage(value: Language) {
    language.value = value;
    setI18nLocale(value);
    await persist("language", value);
  }

  async function setDefaultOpenWith(value: EditorKind) {
    defaultOpenWith.value = value;
    await persist("defaultOpenWith", value);
  }

  return {
    theme,
    themeSkin,
    language,
    defaultOpenWith,
    init,
    setTheme,
    setThemeSkin,
    setLanguage,
    setDefaultOpenWith,
  };
});