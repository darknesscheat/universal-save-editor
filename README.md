# Universal Save Editor

**Edit your offline game saves without touching raw save files.**

Change your health, your money, your inventory from a normal-looking settings
screen. No hex editors, no JSON, no guessing which of `stat_04` and `stat_07` is
the one you wanted.

Every game is a **plugin**: a folder describing where that game keeps its saves
and which values are safe to change. The app itself knows nothing about any
particular game, so adding a new one usually means writing a file rather than
writing code.

Available in 13 languages, including the labels a game plugin supplies, so the
editor screen reads in your language and not just the buttons around it.

Games are shown with their artwork, read from the copy already on your computer.
Nothing is downloaded and no game art is bundled. A game without artwork gets a
generated tile instead of a bare line of text.

> Screenshot placeholder: `docs/images/games.png`, `docs/images/editor.png`

---

## How it works

```
Open the app  →  Pick a game  →  Pick a save  →  Change a value  →  Save
```

That is the whole flow. Behind the last step, in this order:

1. The save on disk is **parsed**. If it will not parse, nothing else happens.
2. Your changes are **validated** against the plugin's rules. If any one of them
   is wrong, none of them are applied. Nothing on disk has been touched yet.
3. A **backup is taken**, and if that fails the edit is abandoned. Your save is
   never modified without a copy existing first.
4. The file is rebuilt, **re-parsed to prove it is still valid**, then written to
   a temporary file and **atomically swapped in**.

If anything goes wrong at any point, your original save is exactly as it was.

Validation runs before the backup so that a rejected edit does not leave a stray
backup behind for you to wonder about.

**Limits are advice, not law.** A plugin marks the range it knows to be safe.
Going past it turns the field amber and asks once before writing. It does not
stop you. Games write values past their own apparent limits all the time, and an
editor that refused to open such a save would be wrong more often than right.

**The app also watches its surroundings.** It warns when the game is running, or
when Steam Cloud syncs the folder, and refuses to save if the game rewrote the
file while you had it open. Without that check, pressing Save would silently undo
whatever the game had just written.

---

## Languages

English · Türkçe · Deutsch · Español · Français · Italiano · Português (Brasil) ·
Русский · Polski · Українська · 日本語 · 한국어 · 简体中文

The app follows your operating system's language on first run; Settings has a
picker. Plugin-supplied field labels are translated too, so "Money" becomes
"Para", "お金", "Деньги". Item names stay as the game writes them, since that is
how players look them up.

Translations were machine-assisted and are **not yet reviewed by native
speakers**. Corrections are the easiest useful contribution there is: see
[docs/translating.md](docs/translating.md).

---

## Supported games

| Game | What you can edit | Save format |
|------|-------------------|-------------|
| **Pathogenic** | Health, money, armor, DNA, stamina, floor, rerolls; equipment (45 weapons × 4 rarities, 73 organs); 53 mutations; permanent starting loadout; 161 discovery and kill-count entries; profile progression | JSON |
| **Feed The Pit** | Money, mission, difficulty and location for each of the three save slots; the six carried tool slots and four van slots, choosing from all 77 tools and cards the game defines; mushroom count; death counters and Cardmaster memories | JSON |
| **Sort Them Ducks** | Money and the shelf, egg and speedrun counters; all ten abilities; the three upgrade tracks; the ten hidden eggs individually or all at once | JSON |

Sort Them Ducks is the first Unity game here, and it needed no new code: the save
is plain JSON. Its 4,015 ducks are not offered for editing, because each one is a
position and a rotation in the world rather than anything a player would want to
change.

Pathogenic and Feed The Pit keep more than one kind of save file, and each
plugin handles all of them, showing only the sections that apply to the file you opened. Pathogenic
splits a run in progress from a permanent profile; Feed The Pit splits slot
progress from character memories.

Health is missing from the Feed The Pit list because the game does not store it.
It lives in the run and never reaches disk. An editor that offered to change it
would be lying.

More to come, and each one is a plugin, so it does not have to be us who writes
it. See [docs/plugin-development.md](docs/plugin-development.md).

---

## Installation

Prebuilt binaries are not published yet. Build it yourself; see Development
below.

The app is fully local. It makes no network requests, runs no background
service, and needs no account.

---

## Development

Requirements:

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/). The version is pinned by `rust-toolchain.toml`, so
  `rustup` installs the right one for you.
- The platform prerequisites for [Tauri 2](https://v2.tauri.app/start/prerequisites/)

```bash
npm install
npm run tauri dev      # run with hot reload
npm run tauri build    # produce an installer
```

On Windows, `npm run tauri build -- --bundles msi` skips the NSIS installer,
which needs a second toolchain download that often fails behind a proxy.

Tests:

```bash
cd src-tauri
cargo test                                 # everything
cargo test -- --ignored --nocapture        # also probe games installed on this machine
```

The suite covers save detection, parsing, writing, backup and restore,
validation, plugin loading, corrupted-save handling, and most importantly that a
`parse → modify → write → parse` round trip leaves every byte you did not edit
alone.

---

## Writing a plugin

A plugin is a folder with a `manifest.json`:

```
plugins/
  my-game/
    manifest.json
```

It declares where saves live, how to recognise one, and which fields may be
edited, with the rules for each:

```json
{
  "id": "my-game",
  "name": "My Game",
  "version": "1.0.0",
  "format": "json",
  "save_locations": [
    { "root": "{APPDATA}/MyGame", "pattern": "slot_*/save.json" }
  ],
  "identify": [{ "pointer": "/player/health" }],
  "groups": [
    {
      "id": "character",
      "label": "Character",
      "fields": [
        { "id": "money", "label": "Money", "pointer": "/player/money",
          "type": "integer", "min": 0, "max": 999999999 }
      ]
    }
  ]
}
```

Drop the folder in, press **Reload plugins** in Settings, and your game appears.
The full reference covers every field type, list and inventory editing,
per-platform paths and validation rules:
[docs/plugin-development.md](docs/plugin-development.md).

---

## Known limitations

Worth knowing before you trust it with a save you care about. Backups are taken
either way, so the worst case is a restore.

- **Feed The Pit's tool list is unverified against the game.** The 77 ids were
  read out of the game's own archive, not out of a save the game wrote, because
  a save only records a tool once the player has picked it up. If the `id` field
  wants something other than the bare name, every option in that dropdown is
  wrong. Pathogenic does not have this problem: its options are checked against
  real `past_runs.json` data by a test.
- **The GUI is covered by tests, not by eyes.** WebView2 ignores synthetic mouse
  input, so the confirmation dialog, the tabs, the light theme and the recovery
  list have automated coverage of the logic behind them and no automated check
  that they are laid out sensibly.
- **No installer for Linux or macOS**, and on Windows only the MSI builds;
  the NSIS target needs a toolchain download that fails behind some proxies.
- **Nothing is code-signed**, so Windows will show a SmartScreen warning.
- Item artwork works for Godot only, and not for Godot games whose archive
  directory is encrypted.

## Roadmap

- [ ] Prebuilt releases for Windows, macOS and Linux
- [ ] More save formats: XML, INI, base64-wrapped and binary containers
- [x] Adding and removing inventory items, not only editing existing ones
- [x] Automatic pruning of old backups
- [x] Item icons read from the player's own game files
- [ ] More bundled game plugins
- [ ] Native-speaker review of the shipped translations

## Contributing

Plugins are the easiest and most useful contribution: one file, no Rust needed,
and Settings can generate a starting draft from any save file. Translation fixes
are just as welcome and even smaller.

See [CONTRIBUTING.md](CONTRIBUTING.md), and
[docs/translating.md](docs/translating.md) for languages.

Anything that touches the write path needs a test. That code is the reason
people can trust the app with their save files.

## Scope

This is a local save editor for **offline, single-player** games. It reads and
writes files that already belong to you on your own computer.

It does not, and will not, help with circumventing DRM or anti-cheat, editing
multiplayer or server-side state, or breaking encryption. If a game keeps its
progression on someone else's server, this tool cannot change it, and a plugin
that tried would be manipulating other players' experience rather than the
author's own save file.

## License

[MIT](LICENSE)
