import { en } from "./en";
import { zh } from "./zh";

export type Locale = "en" | "zh";

export type Translations = typeof en;

export const translations: Record<Locale, Translations> = {
  en,
  zh,
};

export const defaultLocale: Locale = "en";
