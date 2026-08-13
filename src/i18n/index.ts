import { createContext, useContext } from "react";
import { en } from "./locales/en";
import { tr } from "./locales/tr";
import { de } from "./locales/de";
import { es } from "./locales/es";
import { fr } from "./locales/fr";
import { it } from "./locales/it";
import { ptBR } from "./locales/pt-BR";
import { ru } from "./locales/ru";
import { pl } from "./locales/pl";
import { uk } from "./locales/uk";
import { ja } from "./locales/ja";
import { ko } from "./locales/ko";
import { zhCN } from "./locales/zh-CN";

/** English is the reference: every other locale is a partial override of it. */
export type Strings = typeof en;
export type StringKey = keyof Strings;

/** Keys that take a `{count}` and therefore have plural forms. */
type PluralBase =
  | "saves.found"
  | "editor.ready"
  | "editor.needFixing"
  | "editor.saved"
  | "settings.reloaded"
  | "confirm.intro";

/**
 * The CLDR plural categories. English needs only `one` and `other`; Russian,
 * Polish and Ukrainian also use `few` and `many`, and some languages use
 * `zero` and `two`. A locale supplies whichever ones its grammar has.
 */
type PluralCategory = "zero" | "one" | "two" | "few" | "many" | "other";

/**
 * What a translation file may contain: any key from the reference language,
 * plus the plural variants of the counted keys. Anything else is a typo and
 * TypeScript will say so.
 */
export type LocaleStrings = Partial<Strings> &
  Partial<Record<`${PluralBase}_${PluralCategory}`, string>>;

export interface Language {
  /** BCP-47 tag, also sent to Rust so plugin labels match. */
  tag: string;
  /** The language's own name for itself, never translated. */
  name: string;
  strings: LocaleStrings;
}

export const LANGUAGES: Language[] = [
  { tag: "en", name: "English", strings: en },
  { tag: "tr", name: "Türkçe", strings: tr },
  { tag: "de", name: "Deutsch", strings: de },
  { tag: "es", name: "Español", strings: es },
  { tag: "fr", name: "Français", strings: fr },
  { tag: "it", name: "Italiano", strings: it },
  { tag: "pt-BR", name: "Português (Brasil)", strings: ptBR },
  { tag: "ru", name: "Русский", strings: ru },
  { tag: "pl", name: "Polski", strings: pl },
  { tag: "uk", name: "Українська", strings: uk },
  { tag: "ja", name: "日本語", strings: ja },
  { tag: "ko", name: "한국어", strings: ko },
  { tag: "zh-CN", name: "简体中文", strings: zhCN },
];

const STORAGE_KEY = "use.language";

/**
 * Work out which language to start in.
 *
 * A saved choice always wins. Otherwise we take the browser's list, which in
 * a Tauri window is the operating system's language order, and use the first
 * one we have, matching an exact tag before falling back to the base language.
 */
export function detectLanguage(): string {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved && LANGUAGES.some((l) => l.tag === saved)) return saved;

  const wanted = navigator.languages?.length
    ? navigator.languages
    : [navigator.language ?? "en"];

  for (const raw of wanted) {
    const tag = raw.toLowerCase();
    const exact = LANGUAGES.find((l) => l.tag.toLowerCase() === tag);
    if (exact) return exact.tag;

    const base = tag.split("-")[0];
    const loose = LANGUAGES.find((l) => l.tag.toLowerCase().split("-")[0] === base);
    if (loose) return loose.tag;
  }
  return "en";
}

export function saveLanguage(tag: string) {
  localStorage.setItem(STORAGE_KEY, tag);
}

/** Substitute `{name}` placeholders. */
function fill(template: string, params?: Record<string, unknown>): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (whole, key) =>
    key in params ? String(params[key]) : whole,
  );
}

export function createTranslator(tag: string) {
  const lang = LANGUAGES.find((l) => l.tag === tag) ?? LANGUAGES[0];

  /**
   * Look up `key`, filling in `params`.
   *
   * When `params.count` is present the key is resolved through the language's
   * own plural rules, `key_one`, `key_few`, `key_many`, `key_other`, because
   * "1 change / 2 changes" is not how Russian or Polish work.
   *
   * A key missing from a translation falls through to English rather than
   * showing the raw key, so a half-finished language is still usable.
   */
  return function t(key: StringKey, params?: Record<string, unknown>): string {
    const lookup = (k: string): string | undefined =>
      (lang.strings as Record<string, string>)[k] ??
      (en as Record<string, string>)[k];

    if (params && typeof params.count === "number") {
      let category: string;
      try {
        category = new Intl.PluralRules(tag).select(params.count);
      } catch {
        category = params.count === 1 ? "one" : "other";
      }
      const plural = lookup(`${key}_${category}`) ?? lookup(`${key}_other`);
      if (plural) return fill(plural, params);
    }

    return fill(lookup(key) ?? key, params);
  };
}

export type Translator = ReturnType<typeof createTranslator>;

interface I18nValue {
  tag: string;
  t: Translator;
  setLanguage: (tag: string) => void;
}

export const I18nContext = createContext<I18nValue>({
  tag: "en",
  t: createTranslator("en"),
  setLanguage: () => {},
});

export const useI18n = () => useContext(I18nContext);
