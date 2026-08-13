/**
 * Light, dark, or whatever the operating system says.
 *
 * The palette was dark-only to begin with. Every colour is already a custom
 * property, so a light theme is a second set of values rather than a second
 * stylesheet, and "system" simply stops overriding and lets the media query
 * decide.
 */
export type Theme = "system" | "light" | "dark";

const STORAGE_KEY = "use.theme";

export function loadTheme(): Theme {
  const saved = localStorage.getItem(STORAGE_KEY);
  return saved === "light" || saved === "dark" ? saved : "system";
}

export function applyTheme(theme: Theme) {
  localStorage.setItem(STORAGE_KEY, theme);

  const root = document.documentElement;
  if (theme === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", theme);
  }
}

/**
 * A date the way this language writes it, plus how long ago that was.
 *
 * The backup list used to show a fixed `2026-08-12 01:30:42`, which is precise
 * and hard to place. "2 hours ago" is what someone actually wants to know when
 * choosing which copy to restore.
 */
export function formatWhen(iso: string, locale: string): string {
  // Backups store `YYYY-MM-DD HH:MM:SS` in local time.
  const parsed = new Date(iso.replace(" ", "T"));
  if (Number.isNaN(parsed.getTime())) return iso;

  const absolute = new Intl.DateTimeFormat(locale, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(parsed);

  const relative = relativeTo(parsed, locale);
  return relative ? `${absolute} · ${relative}` : absolute;
}

function relativeTo(when: Date, locale: string): string | null {
  let rtf: Intl.RelativeTimeFormat;
  try {
    rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  } catch {
    return null;
  }

  const seconds = Math.round((when.getTime() - Date.now()) / 1000);
  const units: [Intl.RelativeTimeFormatUnit, number][] = [
    ["year", 31_536_000],
    ["month", 2_592_000],
    ["week", 604_800],
    ["day", 86_400],
    ["hour", 3_600],
    ["minute", 60],
  ];

  for (const [unit, size] of units) {
    if (Math.abs(seconds) >= size) {
      return rtf.format(Math.round(seconds / size), unit);
    }
  }
  return rtf.format(seconds, "second");
}
