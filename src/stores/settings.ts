import { ref } from "vue";
import { defineStore } from "pinia";
import { load, type Store } from "@tauri-apps/plugin-store";

export type ThemeMode = "system" | "light" | "dark";
export type Language = "zh-CN" | "en-US";

const STORE_FILE = "settings.json";

export const useSettingsStore = defineStore("settings", () => {
  const theme = ref<ThemeMode>("system");
  const language = ref<Language>("zh-CN");

  let fileStore: Store | null = null;
  let initialized = false;

  const systemDark = window.matchMedia("(prefers-color-scheme: dark)");

  function applyTheme() {
    const dark =
      theme.value === "dark" || (theme.value === "system" && systemDark.matches);
    document.documentElement.classList.toggle("dark", dark);
  }

  function onSystemThemeChange() {
    if (theme.value === "system") applyTheme();
  }

  async function init() {
    if (initialized) return;
    initialized = true;

    fileStore = await load(STORE_FILE, {
      defaults: { theme: "system", language: "zh-CN" },
    });
    const savedTheme = await fileStore.get<ThemeMode>("theme");
    if (savedTheme === "light" || savedTheme === "dark" || savedTheme === "system") {
      theme.value = savedTheme;
    }
    const savedLanguage = await fileStore.get<Language>("language");
    if (savedLanguage === "zh-CN" || savedLanguage === "en-US") {
      language.value = savedLanguage;
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

  async function setLanguage(value: Language) {
    language.value = value;
    await persist("language", value);
  }

  return { theme, language, init, setTheme, setLanguage };
});
