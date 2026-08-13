import { useEffect, useState } from "react";
import { useI18n } from "../i18n";
import * as api from "../services/api";
import { formatWhen } from "../theme";
import type { BackupEntry } from "../types";

export function Backups({ gameId }: { gameId?: string }) {
  const { t, tag } = useI18n();
  const [items, setItems] = useState<BackupEntry[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [confirming, setConfirming] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = () => {
    api.listBackups(gameId).then(setItems).catch((e) => setError(api.errorMessage(e, t)));
  };

  useEffect(load, [gameId]);

  const restore = async (entry: BackupEntry) => {
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      await api.restoreBackup(entry.id);
      setStatus(t("backups.restored"));
      setConfirming(null);
      load();
    } catch (e) {
      setError(api.errorMessage(e, t));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (entry: BackupEntry) => {
    setBusy(true);
    try {
      await api.deleteBackup(entry.id);
      load();
    } catch (e) {
      setError(api.errorMessage(e, t));
    } finally {
      setBusy(false);
    }
  };

  if (error && !items) return <div className="notice error">{error}</div>;
  if (!items) return <div className="empty">{t("common.loading")}</div>;

  if (items.length === 0) {
    return (
      <div className="empty">
        {t("backups.none")}
        <br />
        {t("backups.autoNote")}
      </div>
    );
  }

  return (
    <>
      {status && <div className="notice ok">{status}</div>}
      {error && <div className="notice error">{error}</div>}

      {items.map((b) => (
        <div className="card" key={b.id}>
          <div className="row">
            <div style={{ flex: 1, minWidth: 0 }}>
              <div className="title">{formatWhen(b.created, tag)}</div>
              <div className="path">{b.originalPath}</div>
            </div>

            {confirming === b.id ? (
              <>
                <span className="count" style={{ color: "var(--warn)" }}>
                  {t("backups.confirm")}
                </span>
                <button className="primary small" disabled={busy} onClick={() => restore(b)}>
                  {t("backups.yesRestore")}
                </button>
                <button className="ghost small" onClick={() => setConfirming(null)}>
                  {t("common.cancel")}
                </button>
              </>
            ) : (
              <>
                <button className="small" disabled={busy} onClick={() => setConfirming(b.id)}>
                  {t("backups.restore")}
                </button>
                <button className="danger small" disabled={busy} onClick={() => remove(b)}>
                  {t("backups.delete")}
                </button>
              </>
            )}
          </div>
        </div>
      ))}
    </>
  );
}
