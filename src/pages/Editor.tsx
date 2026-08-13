import { useEffect, useMemo, useState } from "react";
import { ChangeSummary } from "../components/ChangeSummary";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { Field, validate } from "../components/Field";
import { RecoveryList } from "../components/RecoveryList";
import { useI18n } from "../i18n";
import * as api from "../services/api";
import type {
  BulkActionView,
  Change,
  EditorDocument,
  FieldValue,
  FieldView,
  GameSummary,
  ListView,
  SaveSummary,
  Warning,
} from "../types";

interface Props {
  game: GameSummary;
  save: SaveSummary;
  /** Told whether there is unsaved work, so leaving can be guarded. */
  onDirtyChange?: (dirty: boolean) => void;
}

/** Walk every field on the screen, list items included. */
function allFields(doc: EditorDocument): FieldView[] {
  return doc.groups.flatMap((g) => [
    ...g.fields,
    ...g.lists.flatMap((l) => l.items.flatMap((i) => i.fields)),
  ]);
}

export function Editor({ game, save, onDirtyChange }: Props) {
  const { t, tag } = useI18n();
  const [doc, setDoc] = useState<EditorDocument | null>(null);
  const [values, setValues] = useState<Record<string, FieldValue>>({});
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  /** Non-null while the confirmation dialog is up. */
  const [pending, setPending] = useState<Warning[] | null>(null);
  /** Set when the pending confirmation came from a preset rather than edits. */
  const [pendingPreset, setPendingPreset] = useState<string | null>(null);
  /** Which section is showing; `null` means all of them. */
  const [tab, setTab] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  /**
   * Item artwork read out of the game's own files, keyed by option value.
   *
   * Fetched separately from the save because it depends on the game being
   * installed, not on the file being edited, and because it is cached in the
   * backend for the whole session.
   */
  const [icons, setIcons] = useState<Record<string, string>>({});

  useEffect(() => {
    api.itemIcons(game.id).then(setIcons).catch(() => setIcons({}));
  }, [game.id]);

  const load = () => {
    setError(null);
    setStatus(null);
    setPending(null);
    api
      .openSave(game.id, save.path, tag)
      .then((d) => {
        setDoc(d);
        const seed: Record<string, FieldValue> = {};
        for (const f of allFields(d)) seed[f.pointer] = f.value;
        setValues(seed);
      })
      .catch((e) => setError(api.errorMessage(e, t)));
  };

  // Reloads on a language change too, so plugin labels follow the setting.
  useEffect(load, [game.id, save.path, tag]);

  const fields = useMemo(() => (doc ? allFields(doc) : []), [doc]);

  /** Only values that actually differ from what is on disk. */
  const changes: Change[] = useMemo(
    () =>
      fields
        .filter((f) => !f.readOnly && !f.missing)
        .filter((f) => JSON.stringify(values[f.pointer]) !== JSON.stringify(f.value))
        .map((f) => ({
          pointer: f.pointer,
          label: f.label,
          before: f.value,
          after:
            f.type === "integer" || f.type === "number"
              ? Number(values[f.pointer])
              : values[f.pointer],
        })),
    [fields, values],
  );

  /** Problems that block saving. Out-of-range values are not among them. */
  const blocking = useMemo(
    () =>
      fields
        .filter((f) => !f.readOnly && !f.missing)
        .filter((f) => validate(f, values[f.pointer], t)?.level === "error").length,
    [fields, values, t],
  );

  useEffect(() => {
    onDirtyChange?.(changes.length > 0);
    return () => onDirtyChange?.(false);
  }, [changes.length, onDirtyChange]);

  /**
   * The sections actually on screen, after the tab choice and the search box.
   *
   * Searching looks at field and row labels rather than values: you are
   * hunting for "money", not for "250". A section whose explanation is showing
   * has no fields to match, so it only survives an empty search.
   */
  const visibleGroups = useMemo(() => {
    if (!doc) return [];
    const q = search.trim().toLowerCase();

    return doc.groups
      .filter((g) => tab === null || g.id === tab)
      .map((g) => {
        if (!q) return g;
        return {
          ...g,
          fields: g.fields.filter((f) => f.label.toLowerCase().includes(q)),
          lists: g.lists
            .map((l) => ({
              ...l,
              items: l.items.filter(
                (i) =>
                  i.label.toLowerCase().includes(q) ||
                  i.fields.some((f) => f.label.toLowerCase().includes(q)),
              ),
            }))
            .filter((l) => l.items.length > 0 || l.label.toLowerCase().includes(q)),
        };
      })
      .filter(
        (g) =>
          !q ||
          g.fields.length > 0 ||
          g.lists.length > 0 ||
          g.label.toLowerCase().includes(q),
      );
  }, [doc, tab, search]);

  const onChange = (pointer: string, value: FieldValue) =>
    setValues((v) => ({ ...v, [pointer]: value }));

  /**
   * The picture for a list row, taken from whichever of its fields currently
   * holds a value the artwork is keyed by.
   */
  const rowIcon = (item: { fields: FieldView[] }) => {
    for (const f of item.fields) {
      const v = values[f.pointer] ?? f.value;
      if (typeof v === "string" && icons[v]) return icons[v];
    }
    return undefined;
  };

  /** Put one field back to what is on disk. */
  const revert = (pointer: string) => {
    const original = fields.find((f) => f.pointer === pointer);
    if (original) setValues((v) => ({ ...v, [pointer]: original.value }));
  };

  /**
   * Set one field across every row of a list.
   *
   * Nothing is written here, it only fills in the same pending edits the
   * player could have typed, so bulk changes still go through validation,
   * confirmation and the backup before reaching disk.
   */
  const applyBulk = (list: ListView, action: BulkActionView) => {
    setValues((v) => {
      const next = { ...v };
      for (const item of list.items) {
        const target = action.field
          ? item.fields.find((f) => f.id === action.field)
          : item.fields[0];
        if (target && !target.readOnly && !target.missing) {
          next[target.pointer] = action.value;
        }
      }
      return next;
    });
  };

  /**
   * Run one of the plugin's presets.
   *
   * Refused while there are unsaved edits, for the same reason as row changes:
   * it writes immediately and reloads, which would discard them.
   */
  const runPreset = async (presetId: string, confirm: boolean) => {
    if (!doc) return;
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      const report = await api.applyPreset(game.id, save.path, presetId, doc.stamp, confirm);
      setPending(null);
      setPendingPreset(null);
      setStatus(
        report.changedFields === 0
          ? t("editor.savedNothing")
          : t("editor.saved", { count: report.changedFields }),
      );
      load();
    } catch (e) {
      const warnings = api.confirmationNeeded(e);
      if (warnings) {
        setPending(warnings);
        setPendingPreset(presetId);
      } else {
        setPending(null);
        setPendingPreset(null);
        setError(api.errorMessage(e, t));
      }
    } finally {
      setBusy(false);
    }
  };

  /**
   * Add or delete a row.
   *
   * Written straight away, unlike field edits, because it renumbers the rows
   * around it. Refused while there are unsaved edits, reloading afterwards
   * would throw them away, and losing work silently is worse than a disabled
   * button.
   */
  const changeRow = async (list: ListView, change: "add" | "remove", index: number) => {
    if (!doc) return;
    setBusy(true);
    setError(null);
    try {
      await api.changeListRow(game.id, save.path, list.id, change, index, doc.stamp);
      setStatus(change === "add" ? t("list.added") : t("list.removed"));
      load();
    } catch (e) {
      setError(api.errorMessage(e, t));
    } finally {
      setBusy(false);
    }
  };

  /**
   * Save, asking once if the backend says some values are outside the safe
   * range. `confirm` is false the first time round; the dialog re-runs this
   * with true.
   */
  const write = async (confirm: boolean) => {
    if (!doc) return;
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      const report = await api.writeSave(
        game.id,
        save.path,
        changes.map((c) => ({ pointer: c.pointer, value: c.after as FieldValue })),
        doc.stamp,
        confirm,
      );
      setPending(null);
      setStatus(
        report.changedFields === 0
          ? t("editor.savedNothing")
          : t("editor.saved", { count: report.changedFields }),
      );
      load();
    } catch (e) {
      const warnings = api.confirmationNeeded(e);
      if (warnings) {
        setPending(warnings);
      } else {
        setPending(null);
        setError(api.errorMessage(e, t));
      }
    } finally {
      setBusy(false);
    }
  };

  if (error && !doc) return <div className="notice error">{error}</div>;
  if (!doc) return <div className="empty">{t("editor.opening")}</div>;

  return (
    <>
      {/* Standing warnings about the file's surroundings, not about any edit. */}
      {doc.gameRunning.length > 0 && (
        <div className="notice warn">
          {t("banner.gameRunning", { game: doc.gameName })}
        </div>
      )}
      {doc.cloudSynced && <div className="notice info">{t("banner.cloud")}</div>}

      {status && <div className="notice ok">{status}</div>}
      {error && <div className="notice error">{error}</div>}

      {doc.presets.length > 0 && (
        <section className="group">
          <h2>{t("presets.title")}</h2>
          <p className="desc">{t("presets.help")}</p>
          <div className="bulk">
            {doc.presets.map((p) => (
              <button
                key={p.id}
                className="small"
                disabled={busy || changes.length > 0}
                title={changes.length > 0 ? t("list.saveFirst") : p.description}
                onClick={() => runPreset(p.id, false)}
              >
                {p.label}
              </button>
            ))}
          </div>
        </section>
      )}

      {doc.groups.length > 1 && (
        <div className="tabs">
          <button
            className={tab === null ? "active" : ""}
            onClick={() => setTab(null)}
          >
            {t("editor.allSections")}
          </button>
          {doc.groups.map((g) => (
            <button
              key={g.id}
              className={tab === g.id ? "active" : ""}
              onClick={() => setTab(g.id)}
            >
              {g.label}
            </button>
          ))}
        </div>
      )}

      <input
        className="search"
        type="text"
        placeholder={t("editor.searchFields")}
        value={search}
        onChange={(e) => setSearch(e.target.value)}
      />

      {visibleGroups.map((group) => (
        <section className="group" key={group.id}>
          <h2>{group.label}</h2>
          {group.absentReason ? (
            <p className="desc">{group.absentReason}</p>
          ) : (
            group.description && <p className="desc">{group.description}</p>
          )}

          {group.fields.map((f) => (
            <Field
              key={f.pointer}
              field={f}
              value={values[f.pointer]}
              onChange={onChange}
              onRevert={revert}
              icons={icons}
            />
          ))}

          {group.lists.map((list) => (
            <div key={list.id} style={{ marginTop: group.fields.length ? 16 : 0 }}>
              {(list.label || list.description) && (
                <>
                  <h2 style={{ marginTop: 12 }}>{list.label}</h2>
                  {list.description && <p className="desc">{list.description}</p>}
                </>
              )}
              {list.bulkActions.length > 0 && list.items.length > 0 && (
                <div className="bulk">
                  {list.bulkActions.map((a) => (
                    <button
                      key={a.id}
                      className="small"
                      onClick={() => applyBulk(list, a)}
                    >
                      {a.label}
                    </button>
                  ))}
                </div>
              )}
              {list.items.length === 0 && <p className="desc">{t("editor.nothingHere")}</p>}
              {list.items.map((item, i) => (
                <div className="list-item" key={`${list.id}-${i}`}>
                  <div className="row">
                    {rowIcon(item) && (
                      <img className="item-icon large" src={rowIcon(item)} alt="" />
                    )}
                    <div className="name" style={{ flex: 1 }}>
                      {item.label}
                    </div>
                    {list.allowRemove && (
                      <button
                        className="danger small"
                        disabled={busy || changes.length > 0}
                        title={changes.length > 0 ? t("list.saveFirst") : undefined}
                        onClick={() => changeRow(list, "remove", item.index)}
                      >
                        {t("list.remove")}
                      </button>
                    )}
                  </div>
                  {item.fields.map((f) => (
                    <Field
                      key={f.pointer}
                      field={f}
                      value={values[f.pointer]}
                      onChange={onChange}
                      onRevert={revert}
                      icons={icons}
                    />
                  ))}
                </div>
              ))}

              {list.allowAdd && (
                <button
                  className="small"
                  disabled={busy || changes.length > 0}
                  title={changes.length > 0 ? t("list.saveFirst") : undefined}
                  onClick={() => changeRow(list, "add", 0)}
                >
                  + {t("list.add")}
                </button>
              )}
            </div>
          ))}
        </section>
      ))}

      {changes.length > 0 && (
        <section className="group">
          <h2>{t("changes.title")}</h2>
          <ChangeSummary changes={changes} />
        </section>
      )}

      <div className="savebar">
        <button
          className="primary"
          disabled={busy || changes.length === 0 || blocking > 0}
          onClick={() => write(false)}
        >
          {busy ? t("editor.saving") : t("editor.save")}
        </button>
        <span className="count">
          {blocking > 0
            ? t("editor.needFixing", { count: blocking })
            : changes.length === 0
              ? t("editor.noChanges")
              : t("editor.ready", { count: changes.length })}
        </span>
      </div>

      <p className="safety">{t("editor.backupNote")}</p>

      {/* Only appears when the game left copies of its own beside this save. */}
      <RecoveryList game={game} save={save} onRestored={load} />

      {pending && (
        <ConfirmDialog
          changes={changes}
          warnings={pending}
          busy={busy}
          onCancel={() => {
            setPending(null);
            setPendingPreset(null);
          }}
          onConfirm={() =>
            pendingPreset ? runPreset(pendingPreset, true) : write(true)
          }
        />
      )}
    </>
  );
}
