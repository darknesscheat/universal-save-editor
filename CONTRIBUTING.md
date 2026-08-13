# Contributing

Two kinds of contribution need no Rust and no build setup, and both are more
useful than they sound.

## Add a game

A plugin is one file. It says where a game keeps its saves and which values may
be changed; the app supplies the editor, the validation, the backups and the
translations around it.

Start from [`docs/example-plugin/`](docs/example-plugin/) and read
[`docs/plugin-development.md`](docs/plugin-development.md). Drop your folder
into the plugins directory shown in Settings, press **Reload plugins**, and your
game appears.

What makes a plugin good:

- **Ranges that describe what is safe, not what is possible.** `min` and `max`
  are advice: going past them warns the player and asks. Set them to values you
  have actually seen the game handle, and leave them off when you do not know.
- **Only fields you understand.** A field nobody can explain is a field nobody
  should edit.
- **`identify` markers that are specific.** They are what stops the app opening
  a different game's file and showing nonsense.
- **No bundled artwork.** Game art is copyrighted; point `icon_sources` at a
  copy the player already has.

## Fix a translation

Everything shipped was machine-assisted and **has not been reviewed by native
speakers**. Corrections are welcome and are the smallest possible first patch.
See [`docs/translating.md`](docs/translating.md).

---

## Working on the app

```bash
npm install
npm run tauri dev
```

Requirements: Node 18+, Rust (pinned by `rust-toolchain.toml`), and the
[Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your
platform.

Before opening a pull request:

```bash
npx tsc --noEmit
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

CI runs all of that on Linux, Windows and macOS.

### The rule that matters

**Nothing about a specific game may live in `src-tauri/src/`.** No filename, no
field name, no special case. If supporting a game seems to need one, the
manifest schema is missing a feature, so add the feature. That rule is what keeps
"support another game" a data change instead of a code change.

### Anything touching the write path needs a test

`save::apply_and_write` is the only way a save file is modified, and the reason
people can trust this app with their saves. Changes there need a test that
shows the failure mode is handled, not just that the happy path still works.

The most important existing test is the round trip: parse → modify → write →
parse must leave every byte you did not edit alone.

### Reporting a save that will not open

Please say which game, which store or platform, and attach the save file if you
can. Without the file it is usually impossible to tell whether the problem is
the plugin, the format, or the app.
