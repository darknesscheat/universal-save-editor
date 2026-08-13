import { Combobox } from "./Combobox";
import { useI18n, type Translator } from "../i18n";
import type { FieldValue, FieldView } from "../types";

/**
 * Beyond this many options a plain dropdown stops being usable and the
 * searchable box takes over.
 */
const SEARCHABLE_FROM = 12;

/**
 * What is wrong with a value, and how wrong.
 *
 * `error` means the game would reject the file: a decimal where it stores an
 * integer, text that is too long. Saving is blocked.
 *
 * `warn` means the value is merely outside the range the plugin calls safe.
 * Saving stays possible; the player is asked to confirm once. Ranges used to
 * block too, which was wrong: a real Pathogenic save held a max health of 1009
 * against a declared ceiling of 999, written by the game itself, and the
 * editor refused to save anything at all until that "mistake" was corrected.
 *
 * This is a courtesy, not a security boundary: the backend re-validates
 * everything and is the authority on both levels.
 */
export interface Problem {
  level: "error" | "warn";
  message: string;
}

export function validate(
  field: FieldView,
  raw: FieldValue,
  t: Translator,
): Problem | null {
  const error = (message: string): Problem => ({ level: "error", message });
  const warn = (message: string): Problem => ({ level: "warn", message });

  if (field.type === "integer" || field.type === "number") {
    if (raw === "" || raw === null) return error(t("field.enterValue"));
    const n = Number(raw);
    if (!Number.isFinite(n)) return error(t("field.enterNumber"));
    if (field.type === "integer" && !Number.isInteger(n))
      return error(t("field.wholeNumber"));

    if (field.min !== null && n < field.min)
      return warn(t("field.tooSmall", { limit: field.min }));
    if (field.max !== null && n > field.max)
      return warn(t("field.tooLarge", { limit: field.max }));
  }
  if (field.type === "text" && field.maxLength !== null) {
    const s = String(raw ?? "");
    if ([...s].length > field.maxLength)
      return error(t("field.tooLong", { limit: field.maxLength }));
  }
  return null;
}

interface Props {
  field: FieldView;
  value: FieldValue;
  onChange: (pointer: string, value: FieldValue) => void;
  /** Put this one field back to the value on disk. */
  onRevert?: (pointer: string) => void;
  /** Option value -> picture, for fields whose choices have artwork. */
  icons?: Record<string, string>;
}

export function Field({ field, value, onChange, onRevert, icons }: Props) {
  const { t } = useI18n();
  const disabled = field.readOnly || field.missing;
  const changed =
    !disabled && JSON.stringify(value) !== JSON.stringify(field.value);
  const problem = disabled ? null : validate(field, value, t);
  const invalid = problem?.level === "error";
  const risky = problem?.level === "warn";

  const set = (v: FieldValue) => onChange(field.pointer, v);

  return (
    <div className={`field${field.missing ? " missing" : ""}${changed ? " changed" : ""}`}>
      <label htmlFor={field.pointer}>
        {field.label}
        {changed && onRevert && (
          <button
            type="button"
            className="revert"
            title={t("editor.revert")}
            aria-label={t("editor.revert")}
            onClick={() => onRevert(field.pointer)}
          >
            ↺
          </button>
        )}
      </label>

      {field.type === "choice" && field.options.length > SEARCHABLE_FROM ? (
        // Long lists get a box you can type into. Scrolling to
        // "rocket_launcher" past 117 other parts is not a reasonable ask.
        <Combobox
          id={field.pointer}
          value={value}
          options={field.options}
          disabled={disabled}
          icons={icons}
          onChange={set}
        />
      ) : field.type === "choice" ? (
        <select
          id={field.pointer}
          disabled={disabled}
          value={JSON.stringify(value)}
          onChange={(e) => set(JSON.parse(e.target.value) as FieldValue)}
        >
          {/* A save may hold a value the plugin does not list. Show it rather
              than silently snapping the field to something else. */}
          {!field.options.some((o) => JSON.stringify(o.value) === JSON.stringify(value)) && (
            <option value={JSON.stringify(value)}>
              {t("editor.notInList", { value: String(value) })}
            </option>
          )}
          {field.options.map((o) => (
            <option key={JSON.stringify(o.value)} value={JSON.stringify(o.value)}>
              {o.label}
            </option>
          ))}
        </select>
      ) : field.type === "boolean" ? (
        <input
          id={field.pointer}
          type="checkbox"
          style={{ width: "auto", justifySelf: "start" }}
          disabled={disabled}
          checked={value === true}
          onChange={(e) => set(e.target.checked)}
        />
      ) : field.type === "text" ? (
        <input
          id={field.pointer}
          type="text"
          disabled={disabled}
          className={invalid ? "invalid" : risky ? "risky" : ""}
          value={String(value ?? "")}
          onChange={(e) => set(e.target.value)}
        />
      ) : (
        <input
          id={field.pointer}
          type="number"
          disabled={disabled}
          className={invalid ? "invalid" : risky ? "risky" : ""}
          step={field.type === "integer" ? 1 : "any"}
          // No min/max attributes here: the range is advice, and the
          // browser would clamp the spinner and mark the input :invalid for a
          // value the app is perfectly willing to write.
          value={value === null ? "" : String(value)}
          onChange={(e) => set(e.target.value === "" ? "" : Number(e.target.value))}
        />
      )}

      {field.missing ? (
        <div className="help">{t("editor.notPresent")}</div>
      ) : problem ? (
        <div className={problem.level === "error" ? "error" : "warn-text"}>
          {problem.message}
        </div>
      ) : field.help ? (
        <div className="help">{field.help}</div>
      ) : null}
    </div>
  );
}
