import { useEffect, useMemo, useRef, useState } from "react";
import { useI18n } from "../i18n";
import type { Choice, FieldValue } from "../types";

interface Props {
  id: string;
  value: FieldValue;
  options: Choice[];
  disabled?: boolean;
  /** Option value -> picture, when the game's own artwork could be read. */
  icons?: Record<string, string>;
  onChange: (value: FieldValue) => void;
}

/** Rendered at once; the rest wait behind the filter. */
const MAX_RENDERED = 80;

/**
 * A dropdown you can type into.
 *
 * A plain `<select>` is fine for a handful of options and miserable for 118
 * body parts: finding `rocket_launcher` means scrolling past everything
 * alphabetically before it. Above a dozen options the editor switches to this.
 *
 * Only the visible slice is rendered, so a long list stays responsive; the
 * value the save already holds is always shown even when the plugin does not
 * list it, the same as the plain select does.
 */
export function Combobox({ id, value, options, disabled, icons, onChange }: Props) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const boxRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const selected = useMemo(
    () => options.find((o) => JSON.stringify(o.value) === JSON.stringify(value)),
    [options, value],
  );

  const matches = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return options;
    // Names starting with the query come first: typing "ro" should offer
    // "Rocket Launcher" before "Orb Gun".
    const starts: Choice[] = [];
    const contains: Choice[] = [];
    for (const o of options) {
      const label = o.label.toLowerCase();
      if (label.startsWith(q)) starts.push(o);
      else if (label.includes(q)) contains.push(o);
    }
    return [...starts, ...contains];
  }, [options, query]);

  const shown = matches.slice(0, MAX_RENDERED);

  // Close when the click lands anywhere else.
  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (!boxRef.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    return () => document.removeEventListener("mousedown", onDown);
  }, [open]);

  useEffect(() => {
    if (open) inputRef.current?.focus();
  }, [open]);

  const choose = (choice: Choice) => {
    onChange(choice.value as FieldValue);
    setOpen(false);
    setQuery("");
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      setActive((i) => {
        const next = e.key === "ArrowDown" ? i + 1 : i - 1;
        return Math.max(0, Math.min(shown.length - 1, next));
      });
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (shown[active]) choose(shown[active]);
    } else if (e.key === "Escape") {
      e.preventDefault();
      setOpen(false);
      setQuery("");
    }
  };

  const label = selected?.label ?? t("editor.notInList", { value: String(value) });
  const pictureFor = (choice: Choice) =>
    typeof choice.value === "string" ? icons?.[choice.value] : undefined;

  return (
    <div className="combo" ref={boxRef}>
      <button
        id={id}
        type="button"
        className="combo-value"
        disabled={disabled}
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => {
          setOpen((o) => !o);
          setActive(0);
        }}
      >
        {selected && pictureFor(selected) && (
          <img className="item-icon" src={pictureFor(selected)} alt="" />
        )}
        <span style={{ flex: 1, minWidth: 0 }}>{label}</span>
        <span className="combo-caret">▾</span>
      </button>

      {open && (
        <div className="combo-menu">
          <input
            ref={inputRef}
            type="text"
            className="combo-search"
            placeholder={t("combo.search")}
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setActive(0);
            }}
            onKeyDown={onKeyDown}
          />

          <div className="combo-list" role="listbox">
            {shown.map((o, i) => (
              <button
                key={JSON.stringify(o.value)}
                type="button"
                role="option"
                aria-selected={o === selected}
                className={
                  "combo-option" +
                  (i === active ? " active" : "") +
                  (o === selected ? " selected" : "")
                }
                onMouseEnter={() => setActive(i)}
                onClick={() => choose(o)}
              >
                {pictureFor(o) && <img className="item-icon" src={pictureFor(o)} alt="" />}
                <span>{o.label}</span>
              </button>
            ))}

            {shown.length === 0 && (
              <div className="combo-empty">{t("combo.noMatch", { query })}</div>
            )}
            {matches.length > shown.length && (
              <div className="combo-empty">
                {t("combo.more", { count: matches.length - shown.length })}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
