# Architecture

## The one rule

**Nothing about any specific game may live in the core.**

No filename, no field name, no "if game == …". Every game-shaped fact belongs to
a plugin. If you find yourself wanting to special-case a game inside
`src-tauri/src/`, the manifest schema is missing a feature. Add the feature.

That rule is what makes "support another game" a data change rather than a code
change, and it is worth defending.

## Layout

```
universal-save-editor/
├── src/                        React + TypeScript frontend
│   ├── components/Field.tsx    one editable value, any type
│   ├── pages/                  the four screens
│   ├── services/api.ts         the only place that calls Rust
│   └── types/                  mirrors core/model.rs
│
├── src-tauri/src/              Rust core
│   ├── core/
│   │   ├── error.rs            one error type, messages written for players
│   │   ├── model.rs            what crosses the boundary to the GUI
│   │   ├── paths.rs            {APPDATA} and friends -> real folders
│   │   ├── icon.rs             game artwork -> data URI
│   │   └── i18n.rs             locale matching for plugin text
│   ├── plugins/
│   │   ├── manifest.rs         the plugin schema
│   │   ├── registry.rs         loads and validates plugin folders
│   │   └── adapter.rs          bytes <-> JSON, per container format
│   ├── save/
│   │   ├── detect.rs           find saves, recognise them, load one
│   │   ├── editor.rs           build the screen; decide what may be written
│   │   ├── validate.rs         check and coerce incoming values
│   │   ├── io.rs               atomic write
│   │   └── mod.rs              the write pipeline
│   ├── backup/mod.rs           copy, list, restore
│   └── commands.rs             the Tauri command surface
│
└── plugins/                    bundled game plugins
```

## How a save becomes a screen

```
manifest.json ──┐
                ├─→ editor::build ──→ EditorDocument ──→ React renders it
save file ──────┘
```

`EditorDocument` pairs each declared field with the value found at its pointer.
The frontend renders whatever it is given: it has no idea what "Plasmid
fragments" means, only that it is an integer between 0 and 999999.

Two mechanisms let one plugin cover a game with several kinds of save file:

- **`group.requires`**: a group appears only when its pointer resolves. In
  Pathogenic, `Character` needs `/player`, `Progression` needs
  `/plasmids/fragment_num`. Open a run-in-progress and you see one set; open a
  profile and you see the other.
- **`list.item_filter`**: one array surfaced as several lists. Pathogenic's
  equipment array becomes *Weapon slots* and *Organ slots*, each offering only
  the parts that legitimately fit, so the GUI cannot suggest a combination the
  game would choke on.

## How an edit becomes a file

`save::apply_and_write` is the **only** path by which a save is modified. The
order is the safety story:

```
0. is the file still the one we read?  ── no? stop, the game got there first
1. parse the file on disk              ── unreadable? stop, nothing was touched
2. validate + apply in memory          ── one bad value? reject the whole batch
   and check cross-field constraints   ── nothing on disk touched yet
3. create a backup                     ── backup failed? stop, refuse to edit blind
4. re-serialise
5. re-parse what we produced           ── our own bug must not reach the game
6. atomic write via temp file          ── reader sees all-old or all-new, never half
7. prune old backups                   ── after the write, so it can never cost a save
```

Step 0 exists because many games hold their state in memory and write the whole
file out when they exit. Without it, pressing Save after the game had rewritten
the file would silently undo whatever the game just wrote. That is the failure that ate
a set of edits during this app's own development.

Step 2 comes before the backup because it changes nothing, so a rejected
edit should not leave a backup behind for the player to wonder about.

Step 5 exists because a save editor's worst failure is producing a file the game
refuses to load. Verifying our own output before it replaces anything costs
microseconds.

### Atomic write

`io::write_atomically` writes to `.<name>.use-tmp` **in the same folder** (rename
is only atomic within one filesystem), `fsync`s it, reads it back and compares
byte for byte, then renames it over the target. A failure anywhere before the
rename leaves the original untouched and cleans up the temporary file.

### Types are preserved

Some engines reject `6.0` where they wrote `6`, and vice versa. So an `integer`
field is always written as a JSON integer and a `number` field always keeps a
fractional part. Set a decimal field to `9999` and the file gets `9999.0`.
Getting this wrong by hand is one of the most common ways a save is corrupted;
here it is structural, not a matter of remembering.

Key order is preserved too (`serde_json`'s `preserve_order`), so a rewritten
save stays a small diff from the original.

## Limits are advice

`min`/`max` mark the range a plugin knows to be safe. Exceeding one produces a
`Warning`, not an `Error`: `apply_and_write` stops with `NeedsConfirmation`
before taking a backup or touching the file, and the same call with
`confirm: true` goes through.

This was a correction, not a design. Ranges used to be hard limits, and a real
Pathogenic profile holding `stats0/max_hp` = 1009 against a declared ceiling of
999, a value the game wrote itself, jammed the editor shut for that entire
file. Type errors remain hard refusals, because a decimal where the engine
stores an integer produces a file the game will reject.

## Structural changes travel alone

`save::structure` handles adding and removing list rows, and it is kept
out of the edit batch. Inserting or deleting renumbers every row after it,
so `/player/loadout/2/rarity` submitted alongside "delete row 1" would land on
the wrong item. Each structural change writes immediately through the same
backup-and-verify pipeline, and the editor reloads with fresh indices. The GUI
disables the buttons while edits are pending rather than silently discarding
them on reload.

## Recovery

`save::recovery` surfaces copies the *game* made: rolling `.bak` files, and
anything it quarantined with a Unix timestamp in the name. Read-only, matched by
globs the plugin declares, and confined to the save's own folder. `save::diff`
compares one against the live save through the manifest, so a restore is a
decision rather than a guess; only declared fields are compared, because
everything else is the game's own bookkeeping.

## Security boundary

`editor::writable_fields` builds, from the manifest **and the document just read
from disk**, the exact set of JSON pointers that may be written. An edit whose
pointer is not in that set is refused.

This is what stops an edit from reaching `/player/loadout/999`, a read-only
field, a hidden group, or `/../../etc/passwd`. Because it is rebuilt per
document, list indices always reflect reality.

Separately, `commands::checked_path` confines every read and write to the
folders the plugin declares, so a malformed request cannot make the app touch an
arbitrary file.

The frontend re-implements the range and type checks in `components/Field.tsx`.
That is a courtesy so the player sees a problem while typing. It is **not** a
boundary. The backend never trusts it.

## Game artwork

`core::icon` turns a plugin's declared picture into a `data:` URI carried on
`GameSummary`. A data URI sidesteps the asset protocol and CSP entirely, and at
a 512 KB cap for a handful of games the payload is trivial.

The game screen draws these as a grid of 2:3 covers, so `icon_sources` should
name the portrait capsule before the landscape header. A 460x215 header used as
a cover is cropped down to a strip of its middle. Steam has cached that portrait
art under two different filenames over the years, so a plugin lists both;
`portrait_cover_art_is_preferred_over_the_landscape_header` keeps the order
honest for every bundled plugin.

The ordering (bundled `icon` first, then `icon_sources` globs) encodes a
licensing constraint, not a technical one. Game art is copyrighted, so no
bundled plugin may ship any; artwork instead comes from a copy the player
already has, such as Steam's thumbnail cache. `no_bundled_plugin_ships_artwork`
enforces that for everything in this repository.

Resolution failure is not an error. A player who does not own the game, or
installed Steam somewhere unusual, gets `None`, and the GUI draws a tile from
the game's initials with a hue hashed from the plugin id. It is deterministic, so a
game keeps the same colour between launches.

`{STEAM}` is resolved by `core::paths` from `STEAM_PATH` or the usual
per-platform locations, which is also what makes the behaviour testable without
Steam installed.

## Item artwork

`plugins::archive` reads pictures for dropdown options out of the game's own
installed archive. One engine is implemented, `godot_pck`, and the seam is
`ItemIcons::format`, so adding another means adding a module rather than
touching the editor.

The chain matters more than the container: a plugin names the **resource** for
each option and the reader follows it to whatever texture that resource
references. Matching option values against filenames was tried first and reached
36%. Pathogenic's `assault_rifle` draws `Player weapon - 3_shot_burst.png`.
Following the resource reaches 100%.

One non-obvious detail about the archive: a `.pck` keeps its file index at the
**end**, and a header field at offset 32 says where. Looking just past the
header, which is the obvious guess, lands in file data.

Results are cached per game in `AppState`. Pulling 118 pictures out of a 1.4 GB
archive costs about 100 ms once, which is not worth paying again on every return
to the editor.

A `.pck` can also decline to be read: Feed The Pit ships one with an encrypted
directory (pack format 2, `flags=1`), so there is no table to follow from a
resource to its texture. That plugin therefore declares no `item_icons` and
shows tool names as text, which is a perfectly good outcome. Nothing in the app
tries to work around an encrypted archive.

Other engines stay out of scope for now. Unreal archives are often
AES-encrypted, and obtaining that key means reading it out of a running game,
squarely in the DRM territory this project stays out of. Unity is unencrypted
but needs a full asset parser. Games in either engine fall back to names, which
is a perfectly good outcome, and games that keep icons as loose files need no
reader at all.

## Languages

Text reaches the player from three places, and each is handled where it belongs:

| Source | Translated in | How |
|---|---|---|
| App chrome: buttons, notices | `src/i18n/locales/` | key lookup, English fallback |
| Backend errors | `src/i18n/locales/` | Rust sends `{ code, message, params }`; the frontend translates the code |
| Plugin labels such as "Money", "Rarity" | the plugin's `manifest.json` | `label_i18n` maps resolved by `core::i18n::pick` |

The frontend sends its locale tag to `list_saves` and `open_save`, so an
`EditorDocument` comes back already in the right language and React renders
whatever it is handed. Nothing about language leaks into the editor, validation
or write paths.

Two decisions worth keeping:

- **Errors carry codes, not just prose.** `Error::code()` and `Error::params()`
  give every failure a stable identifier and its values, so a message can be
  reworded per language with the numbers in the right grammatical place. The
  English sentence travels alongside and is used whenever a translation is
  missing, so a missing string can never produce a blank error.
- **Item names are not translated.** A plugin translates its own vocabulary
  ("Weapon slots", "Legendary") but leaves the game's ("Rocket Launcher") alone,
  because that is the name players search for in guides. `core::i18n::pick`
  simply finds no entry and returns the original.

Locale matching is exact tag → base language → any regional variant → the
untranslated default, so `de-AT` finds `de` and `pt` finds `pt-BR`.

See [translating.md](translating.md) for the contributor-facing side.

## Adding a save format

x in `adapter_for`. Everything above
(fields, validation, lists, backup, atomic write, the entire GUI) then works for
that format with no further changes.

`JsonAdapter` is the reference implementation and is about thirty lines.

## Frontend

Four screens and a plain state machine in `App.tsx`: no router, no state
library, no component framework. The dependency list is kept short: this
is an app people run to protect their save files, and every dependency is
something that could break them.
