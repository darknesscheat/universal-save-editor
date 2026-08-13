import { useEffect, useState } from "react";
import { useI18n } from "../i18n";
import * as api from "../services/api";
import { formatWhen } from "../theme";
import type { FieldChange, GameSummary, RecoveryFile, SaveSummary } from "../types";

interface Props {
  game: GameSummary;
  save: SaveSummary;
  onRestored: () => void;
}

/**
 * Copies of the save that the *game* made, offered for recovery.
 *
 * Games keep their own safety nets and never mention them: a rolling pair of
 * `.bak` files, and anything they refused to load set aside with a timestamp.
 * Someone whose save has gone wrong is usually standing next to three working
 * copies with no way to reach them.
 */
export function RecoveryList({ game, save, onRestored }: Props) {
  const { t, tag } = useI18n();
  const [files, setFiles] = useState<RecoveryFile[] | null>(null);
  const [preview, setPreview] = useState<{ path: string; changes: FieldChange[] } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .listRecoveryFiles(game.id, save.path)
      .then(setFiles)
      .catch(() => setFiles([]));
  }, [game.id, save.path]);

  if (!files || files.length === 0) return null;

  const look = async (file: RecoveryFile) => {
    setBusy(true);
    setError(null);
    try {
      const changes = await api.previewRestore(game.id, save.path, file.path, tag);
      setPreview({ path: file.path, changes });
    } catch (e) {
      setError(api.errorMessage(e, t));
    } finally {
      setBusy(false);
    }
  };

  const restore = async (file: RecoveryFile) => {
    setBusy(true);
    setError(null);
    try {
      await api.restoreRecoveryFile(game.id, save.path, file.path);
      setPreview(null);
      onRestored();
    } catch (e) {
      setError(api.errorMessage(e, t));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="group">
      <h2>{t("recovery.title")}</h2>
      <p className="desc">{t("recovery.help")}</p>
      {error && <div className="notice error">{error}</div>}

      {files.map((f) => (
        <div className="card" key={f.path}>
          <div className="row">
            <div style={{ flex: 1, minWidth: 0 }}>
              <div className="title">{f.name}</div>
              <div className="meta">{formatWhen(f.created, tag)}</div>
            </div>
            <button className="small" disabled={busy} onClick={() => look(f)}>
              {t("recovery.compare")}
            </button>
            <button className="small" disabled={busy} onClick={() => restore(f)}>
              {t("recovery.use")}
            </button>
          </div>

          {preview?.path === f.path && (
            <div style={{ marginTop: 10 }}>
              {preview.changes.length === 0 ? (
                <p className="desc">{t("recovery.identical")}</p>
              ) : (
                <ul className="changes">
                  {preview.changes.slice(0, 40).map((c) => (
                    <li key={c.pointer}>
                      <span className="name">{c.label}</span>
                      <span className="from">{render(c.before)}</span>
                      <span className="arrow">{t("changes.arrow")}</span>
                      <span className="to">{render(c.after)}</span>
                    </li>
                  ))}
                  {preview.changes.length > 40 && (
                    <li className="desc">
                      {t("recovery.andMore", { count: preview.changes.length - 40 })}
                    </li>
                  )}
                </ul>
              )}
            </div>
          )}
        </div>
      ))}
    </section>
  );
}

function render(v: unknown): string {
  if (v === null || v === undefined) return "—";
  if (typeof v === "boolean") return v ? "✓" : "✗";
  return String(v);
}
