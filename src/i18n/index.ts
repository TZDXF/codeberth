import { createI18n } from "vue-i18n";
import zhCN from "./locales/zh-CN";
import enUS from "./locales/en-US";

export const SUPPORTED_LOCALES = ["zh-CN", "en-US"] as const;
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];
export const DEFAULT_LOCALE: SupportedLocale = "zh-CN";
export const FALLBACK_LOCALE: SupportedLocale = "en-US";

export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: DEFAULT_LOCALE,
  fallbackLocale: FALLBACK_LOCALE,
  messages: {
    "zh-CN": zhCN,
    "en-US": enUS,
  },
});

export function setI18nLocale(locale: SupportedLocale) {
  i18n.global.locale.value = locale;
}
