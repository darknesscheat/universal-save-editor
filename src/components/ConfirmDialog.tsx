import { useEffect, useRef } from "react";
import { useI18n } from "../i18n";
import type { Change, Warning } from "../types";
import { ChangeSummary } from "./ChangeSummary";

interface Props {
  changes: Change[];
  warnings: Warning[];
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}

/**
 * Asked before writing values the plugin considers risky.
 *
 * It shows the full list of changes, not only the risky ones: someone about to
 * do something unusual should see everything that is about to happen, and the
 * risky rows are marked within that list rather than shown out of context.
 */
export function ConfirmDialog({ changes, warnings, busy, onCancel, onConfirm }: Props) {
  const { t } = useI18n();
  const confirmRef = useRef<HTMLButtonElement>(null);

  // Focus the safe-looking action, and let Escape back out: a dialog that
  // traps you is worse than no dialog.
  useEffect(() => {
    confirmRef.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  const risky = new Set(warnings.map((w) => w.pointer));

  return (
    <div className="scrim" onClick={onCancel}>
      <div
        className="dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-title"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="confirm-title">{t("confirm.title")}</h2>

        <p className="desc">{t("confirm.intro", { count: warnings.length })}</p>

        <ChangeSummary changes={changes} risky={risky} warnings={warnings} />

        <p className="desc warn-text">{t("confirm.risk")}</p>

        <div className="dialog-actions">
          <button className="ghost" onClick={onCancel} disabled={busy}>
            {t("common.cancel")}
          </button>
          <button ref={confirmRef} className="danger" onClick={onConfirm} disabled={busy}>
            {busy ? t("editor.saving") : t("confirm.go")}
          </button>
        </div>
      </div>
    </div>
  );
}
