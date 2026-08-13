import { useI18n } from "../i18n";
import type { Change, Warning } from "../types";

interface Props {
  changes: Change[];
  /** Pointers the backend flagged as outside the safe range. */
  risky?: Set<string>;
  warnings?: Warning[];
}

/**
 * `Money 250 → 999999`, one line per change.
 *
 * Shown before saving so the player can check their own work, and reused
 * inside the confirmation dialog rather than written twice.
 */
export function ChangeSummary({ changes, risky, warnings }: Props) {
  const { t, tag } = useI18n();
  if (changes.length === 0) return null;

  const limitFor = (pointer: string) => warnings?.find((w) => w.pointer === pointer);

  return (
    <ul className="changes">
      {changes.map((c) => {
        const warning = limitFor(c.pointer);
        return (
          <li key={c.pointer} className={risky?.has(c.pointer) ? "risky" : undefined}>
            <span className="name">{c.label}</span>
            <span className="from">{format(c.before, tag)}</span>
            <span className="arrow">{t("changes.arrow")}</span>
            <span className="to">{format(c.after, tag)}</span>
            {warning && (
              <span className="limit">
                {warning.rule === "rule.tooSmall"
                  ? t("confirm.suggestedMin", { limit: warning.limit })
                  : t("confirm.suggestedMax", { limit: warning.limit })}
              </span>
            )}
          </li>
        );
      })}
    </ul>
  );
}

/**
 * Group digits so a long number can be read at a glance. `99999999` says very
 * little, `99,999,999` says a lot. Formatting follows the chosen language.
 */
function format(value: unknown, locale: string): string {
  if (typeof value === "number" && Number.isFinite(value)) {
    return new Intl.NumberFormat(locale).format(value);
  }
  if (typeof value === "boolean") return value ? "✓" : "✗";
  if (value === null || value === undefined) return "—";
  return String(value);
}
