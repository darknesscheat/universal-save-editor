# Changelog

Notable changes to Universal Save Editor. Dates are release dates; the project
is pre-1.0, so anything may still move.

## Unreleased

### Added

- **A second game: Feed The Pit.** Three save slots, the six tool slots you
  carry and the four kept in the van, with a dropdown of all 77 tools and cards
  the game defines. Health is absent because the game keeps it in the run
  and never writes it to disk.
- **Records inside object-backed lists.** An object list could already surface
  one value per key; it can now surface several, which is how Feed The Pit
  stores a tool slot: keys `"0"`–`"5"`, each holding an id and a durability.
- **Object-backed lists.** A list can read a JSON object as well as an array,
  which made 161 unlock flags and kill counters in Pathogenic editable for the
  first time.
- **Starting equipment.** Pathogenic's permanent loadout lives in the profile
  save, so equipment is editable with no run in progress.
- **Quick actions.** Plugin-declared presets such as refill health or make
  everything legendary, expanded into ordinary edits so they get the same
  validation,
  confirmation and backup.
- **Add and remove list rows**, where the plugin allows it. Mutations can grow
  and shrink; equipment slots cannot, because the game defines them.
- **Searchable dropdowns** above a dozen options, instead of scrolling past 118
  body parts.
- **The window remembers its size and position.** The size in the config only
  ever applied to a first run, so every resize was thrown away on exit. A
  position saved on a monitor that is no longer connected is dropped rather than
  restored, so the window can never open somewhere you cannot reach it.
  Geometry is stored in logical pixels: on a display scaled above 100%, storing
  physical ones made the window shrink by the scale factor on every launch until
  it stuck at its minimum size.
- **Section tabs, field search, and per-field undo** in the editor.
- **Light theme**, following the system by default.
- **The game's own backups**, surfaced for recovery: rolling `.bak` files and
  anything the game quarantined, with the timestamp read out of the filename.
- **Restore previews**, showing field-by-field what a restore would change.
- **Warnings** when the game is running or the save folder is synced by Steam
  Cloud.
- Automatic pruning of old backups, keeping the newest 20 per game.

- **Item artwork in the equipment dropdowns**, read from the player's own
  installed copy of the game. Nothing is bundled. For Godot the app reads the
  `.pck` directly: 118 of 118 Pathogenic body parts, in about 100 ms. Matching
  names against image files had reached only 36%, because `assault_rifle` draws
  a file called `Player weapon - 3_shot_burst.png`; following each part's
  resource to the texture it actually uses reaches all of them.

### Changed

- **The game screen is a cover grid**, laid out the way a game library is:
  portrait 2:3 artwork with the name underneath, reflowing to the window width.
  Plugins now ask Steam for the portrait capsule first and fall back to the
  landscape header, which would otherwise be cropped to a strip of its middle.
- **Range limits are advice, not law.** Exceeding `min`/`max` now warns and asks
  for confirmation instead of blocking the save. A real Pathogenic profile held
  a max health of 1009 against a declared ceiling of 999, written by the game
  itself, and the editor had refused to save anything at all until it was
  "fixed".
- Validation runs before the backup, so a rejected edit no longer leaves a
  stray backup behind.
- Sections that do not apply to a save now explain why instead of vanishing.

### Fixed

- **Late-game weapon slots were invisible.** The Pathogenic plugin enumerated
  `ESlot1`–`ESlot4`, but the game uses `ESlot5` and `ESlot6`; those weapons
  could not be seen or edited. Slot filters now match on a prefix.
- **Saving could undo the game's own work.** If the game rewrote a save while
  the editor had it open, pressing Save wrote the stale copy back. Writes now
  verify the file is still the one that was read.
- The plugin folder list showed the same directory twice on Windows, once with
  the `\\?\` extended-length prefix.
- Backups made within the same second could not be ordered, because the recorded
  time has second precision. Automatic pruning could therefore keep an older
  copy and delete a newer one. The folder name now breaks the tie.
- Game artwork on the game-selection screen was blurry: the 32x32 icon was being
  used instead of the full-size store header.
- **An installed copy would have shipped with no plugins at all.** The bundler
  rewrites `../plugins` to a folder literally named `_up_`, so the installer put
  them in `_up_\plugins\` while the app looked in `plugins\`, so every install
  would have opened with an empty game list. The resource is now declared with
  an explicit destination.
- A release build could run against stale plugins. `tauri.conf.json` declares
  `plugins/` as a bundled resource, but that copy only happens when the bundler
  runs, so a binary started straight out of `target/release/` loaded whatever
  had been left beside it, silently omitting any plugin added since. The copy
  now happens in `build.rs`, alongside the binary it belongs to.
- A test could fail roughly one run in a hundred. Two tests set the process-wide
  `STEAM_PATH` variable, and `cargo test` runs them on separate threads, so one
  could clear it while the other was reading it.

## 0.1.0

First working version: manifest-driven plugin system, atomic writes with
automatic backups, 13 languages, and a Pathogenic plugin.