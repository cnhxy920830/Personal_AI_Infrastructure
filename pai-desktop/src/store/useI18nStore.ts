import { create } from "zustand";
import { persist } from "zustand/middleware";
import { Locale, translations, defaultLocale, Translations } from "../i18n";

interface I18nStore {
  locale: Locale;
  t: Translations;
  setLocale: (locale: Locale) => void;
}

export const useI18nStore = create<I18nStore>()(
  persist(
    (set) => ({
      locale: defaultLocale,
      t: translations[defaultLocale],
      setLocale: (locale: Locale) => {
        set({ locale, t: translations[locale] });
      },
    }),
    {
      name: "pai-i18n-storage",
    }
  )
);
