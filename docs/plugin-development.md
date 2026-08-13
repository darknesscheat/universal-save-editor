# Writing a plugin

A plugin teaches the app about one game. For a game that stores its saves as
JSON, a plugin is **one file and no code**.

```
plugins/
  my-game/
    manifest.json
```

Put that folder in any of the plugin folders listed in **Settings**, press
**Reload plugins**, and your game appears in the list.

---

## Engines: what is easy and what is not

The two questions get confused, so keep them apart:

**Reading the save** is what this app is for, and it is usually easy. It depends
on how the game *serialises* its data, not on which engine drew the graphics.

**Reading the game's artwork** is optional decoration. That depends on the
engine's asset format and is much harder.

A plugin that does the first and skips the second is a complete, useful plugin.

| Engine | Save file | Needs new code? |
|---|---|---|
| Godot | JSON, `.cfg` | No, plain JSON is built in |
| Unity + **Easy Save 3** (`.es3`) | JSON, unless the game turned encryption on | **No.** It is plain JSON with a `{"__type":…,"value":…}` wrapper, so pointers just go one level deeper: `/PlayerBodyColor/value` |
| Unity + PlayerPrefs | Windows registry | Yes, a registry source |
| Unity + custom binary | varies | Yes, a format adapter |
| GameMaker | often INI, sometimes base64-wrapped | Yes, a small adapter |
| Anything encrypted | n/a | **Out of scope.** See the Scope section of the README. |

The lesson from surveying real installations: reach for a plugin before
assuming an engine is hard. An `.es3` file that opens in a text editor and
starts with `{` needs nothing but a manifest.

## The seven steps

1. **Find the save.** Play the game once, then look in the usual places:
   `%APPDATA%`, `%LOCALAPPDATA%`, `Documents\My Games`, or
   `~/.local/share`. Sort by "last modified" while the game is closing. The
   file that just changed is the one.
2. **Open it in a text editor.** If it starts with `{` you are in luck. If it is
   binary, you will also need a format adapter (see the end of this page).
3. **Change one value in the game** (buy something, take damage), quit, and
   diff the file. That tells you which field is which, far faster than guessing
   from names.
4. **Write the manifest** using the reference below.
5. **Test.** Reload plugins, open your save, change a value, save, and load the
   game.
6. **Add a fixture.** Drop an anonymised save in
   `src-tauri/tests/fixtures/<your-game>/` and extend
   `src-tauri/tests/bundled_plugins.rs`. The important test asserts every
   pointer you declared actually resolves, which catches a typo before a
   player does.
7. **Open a pull request.**

---

## Manifest reference

### Top level

| Key | Required | What it is |
|-----|----------|------------|
| `id` | ✅ | Short unique slug, e.g. `"stardew-valley"` |
| `name` | ✅ | Shown in the game list |
| `version` | ✅ | Your plugin's version |
| `author` | | Shown in Settings |
| `description` | | One line under the game name |
| `format` | ✅ | Container format. `"json"` ships in the box |
| `save_locations` | ✅ | Where to look, per platform |
| `identify` | | Pointers that must exist for a file to be one of ours |
| `label` | | How to title a save in the picker |
| `option_sets` | | Reusable dropdown lists |
| `groups` | ✅ | The editor screen |

### `save_locations`

```json
"save_locations": [
  {
    "platforms": ["windows"],
    "root": "{APPDATA}/MyGame/Saves",
    "pattern": "slot_*/save.json",
    "label": "Current run",
    "identify": [{ "pointer": "/player/hp" }]
  }
]
```

- `platforms`: omit for "all platforms".
- `root`: never a literal path. Use a placeholder:

  | Placeholder | Windows | Linux | macOS |
  |---|---|---|---|
  | `{HOME}` | `C:\Users\me` | `/home/me` | `/Users/me` |
  | `{APPDATA}` | `AppData\Roaming` | `~/.local/share` | `~/Library/Application Support` |
  | `{LOCALAPPDATA}` | `AppData\Local` | `~/.local/share` | `~/Library/Application Support` |
  | `{DOCUMENTS}` | `Documents` | `~/Documents` | `~/Documents` |
  | `{CONFIG}` | `AppData\Roaming` | `~/.config` | `~/Library/Application Support` |

- `pattern`: a glob relative to `root`, always written with `/`.
- `label`: what to call saves found here. Use it when a game keeps more than
  one kind of save side by side.
- `identify`: markers specific to this pattern; falls back to the top-level
  `identify` when absent.

### `identify`

```json
"identify": [{ "pointer": "/player/hp" }, { "pointer": "/player/money" }]
```

Every pointer must resolve or the file is ignored. Pick two or three fields that
a save always has and another game's file never would. This is what stops the
picker filling up with `.bak` copies and config files, and what produces a clear
"this does not look like a … save" instead of a screen of empty boxes.

### `groups`

```json
"groups": [
  {
    "id": "character",
    "label": "Character",
    "description": "Shown under the heading.",
    "requires": "/player",
    "fields": [ … ],
    "lists": [ … ]
  }
]
```

`requires` makes the group appear only when that pointer resolves. Use it when
one plugin covers several kinds of save file (a run in progress and a permanent
profile, say) so the player never sees a section that does not apply.

A hidden group is also **not writable**: the GUI and the backend always agree
about what exists.

### `fields`

```json
{
  "id": "money",
  "label": "Money",
  "help": "Shown in small text under the input.",
  "pointer": "/player/money",
  "type": "integer",
  "min": 0,
  "max": 999999999,
  "read_only": false
}
```

`pointer` is an [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901) JSON pointer.

| `type` | Editor shown | Rules |
|---|---|---|
| `integer` | number box | `min`, `max`; written as a JSON integer |
| `number` | number box | `min`, `max`; always keeps a decimal point |
| `text` | text box | `max_length` |
| `boolean` | checkbox | none |
| `choice` | dropdown | `options` or `options_ref` |

**Get `integer` versus `number` right.** Look at the raw file: if the game wrote
`"stamina": 100.0`, the field is a `number` and the app will write `9999.0`, not
`9999`. Some engines reject the wrong one, and it is a common way to corrupt a
save by hand. Declaring it correctly makes the problem go away permanently.

`read_only: true` shows a value without letting it be changed. Good for a seed
or a save version.

### `option_sets` and `choice`

```json
"option_sets": {
  "rarity": [
    { "value": 0, "label": "Common" },
    { "value": 3, "label": "Legendary" }
  ]
},
"groups": [{ "id": "g", "label": "G", "fields": [
  { "id": "rarity", "label": "Rarity", "pointer": "/rarity",
    "type": "choice", "options_ref": "rarity" }
]}]
```

A `choice` field rejects anything not in its list, so a plugin author decides
exactly what can end up in the file. Use `options` for a short inline list and
`options_ref` when the same list is reused.

### Start from a draft

Settings can read any save file and propose a `manifest.json` with a field for
every scalar it finds, typed from the values it sees. It cannot know which
number is health and which is a random seed. That judgement is yours, but it
saves counting JSON pointers by hand.

It leaves arrays and nulls alone and tells you so: a list needs a decision about
what its rows mean, which is covered below.

### Ranges are advice

`min` and `max` describe the range you know to be **safe**, not the range that
is possible. Going past one turns the field amber and asks the player to confirm
once; it does not stop them.

This matters more than it sounds. A real Pathogenic profile held
`stats0/max_hp` = 1009 against a declared ceiling of 999, written by the game
itself, and while ranges were hard limits the editor refused to save *anything*
in that file until the "mistake" was corrected. Set ranges to values you have
watched the game handle, and leave them off when you do not know.

### Rules between fields

Some saves are valid field by field and nonsense as a whole. Declare the
relationship and it is checked on every write:

```json
"constraints": [
  { "left": "/player/hp", "right": "/player/max_hp", "rule": "lte",
    "message": "Health cannot be higher than max health." }
]
```

`lte` and `gte` are available. A rule says nothing about a save that lacks
either side, so a run-only constraint does not trouble a profile save.

### Warning that the game is open

```json
"process_names": ["pathogenic"]
```

Used only to show a banner. Many games hold their state in memory and write the
whole file out when they exit, discarding whatever was changed underneath them.
this is how a set of edits was lost while the app was being built.

### The game's own backups

Games keep safety nets they never mention. Point at them and they become a
recovery list, compared field by field before anything is replaced:

```json
"recovery_patterns": ["*.json.bak", "*.json.bak2", "corrupted_*_save.json"]
```

Globs are relative to the save's own folder, and these files are only ever
read. A Unix timestamp in the filename is decoded into a readable date.

### Explaining a section that does not apply

```json
{
  "id": "equipment",
  "label": "Equipment",
  "requires": "/player/loadout",
  "when_absent": "Only while a run is in progress."
}
```

Without `when_absent` the section simply disappears, and a player concludes the
feature does not exist. That is not hypothetical: it is exactly what happened
with Pathogenic's equipment, which lives in a file the game deletes when a run
ends.

### Quick actions

A preset is a named set of edits. It becomes ordinary edits on the way through,
so it gets the same validation, the same confirmation and the same backup as
anything typed by hand:

```json
"presets": [
  {
    "id": "refill",
    "label": "Refill health",
    "requires": "/player/max_hp",
    "set": [{ "pointer": "/player/hp", "value": 999 }],
    "set_in_lists": [{ "list": "weapons", "field": "rarity", "value": 3 }]
  }
]
```

`requires` works as it does for a group, so a preset is only offered for the
kind of save it fits.

### Giving your game a picture

The game-selection screen shows a picture beside each name, because scanning a
list of words is slower than recognising a logo. There are two ways to supply
one, and a fallback if you supply neither.

**`icon_sources`, a file the player already has.** This is the right choice
for a commercial game. Steam caches artwork for every game you own, so the
plugin can point at it. Sources are tried **in order**, first hit wins:

```json
"icon_sources": [
  { "path": "{STEAM}/appcache/librarycache/3808690/*/library_header.jpg" },
  { "path": "{STEAM}/appcache/librarycache/3808690/*/logo.png" },
  { "path": "{STEAM}/appcache/librarycache/3808690/*/library_capsule.jpg" }
]
```

Replace `3808690` with the game's Steam app id, the number in the store
URL.

**Order by resolution, not convenience.** Steam also keeps a 32×32 icon at the
top level of that folder. It is the easiest thing to glob and it looks terrible:
the card renders around 104×49 CSS pixels, which is already an upscale on a
normal display and a bad one at 2x. Prefer, roughly in this order:

| File | Size | Shape |
|---|---|---|
| `*/library_header.jpg` | 460×215 | landscape, matches the card, no crop |
| `*/logo.png` | 640×360 | transparent logo |
| `*/library_capsule.jpg` | 300×450 | portrait box art, gets centre-cropped |
| `*.jpg` (top level) | 32×32 | last resort only |

Globs are allowed; when one source matches several files the largest is used.
Add `"platforms": ["windows"]` to limit a source to one platform. Anything over
512 KB, or in a format a browser cannot render, is skipped.

**`icon`, a file inside your plugin folder.** Only for art you are actually
allowed to redistribute: your own game, or something under a licence that
permits it.

```json
"icon": "icon.png"
```

The path is confined to the plugin folder; `../` will not escape it.

> **Do not bundle artwork you do not own.** Game art is copyrighted, and a
> plugin that ships it makes the whole repository un-redistributable. The
> Pathogenic plugin bundles nothing and reads Steam's cache
> instead. A test enforces this for every plugin in the repo.

**Neither?** The app draws a tile with the game's initials, coloured from a
hash of your plugin id, stable across launches and different from its
neighbours. A plugin with no artwork still looks deliberate.

### Lists that read an object

Not all repeated data is an array. Games keep a surprising amount as a plain
object of key/value pairs. Pathogenic stores 161 unlock flags and kill counters
across five of them, none of it reachable while lists could only be arrays.

```json
{
  "id": "enemy_discoveries",
  "label": "Enemies discovered",
  "pointer": "/enemy_discoveries",
  "source": "object",
  "entry": { "id": "found", "label": "Discovered", "pointer": "", "type": "boolean" }
}
```

Each key becomes a row, prettified from `armor_maker` to *Armor Maker*, and
`entry` describes the value. Keys containing `/` or `~` are escaped properly.

If the value is a small record rather than a single thing, drop `entry` and use
`fields` instead, exactly as you would for an array source, with pointers
relative to the key. Feed The Pit stores its tool slots this way:

```json
{
  "id": "slot1_tools",
  "label": "Carried tools",
  "pointer": "/tracked_progress/save_slots/1/tools",
  "source": "object",
  "fields": [
    { "id": "id", "label": "Tool", "pointer": "/id",
      "type": "choice", "options_ref": "tools" },
    { "id": "durability", "label": "Durability", "pointer": "/durability",
      "type": "integer", "min": 0, "max": 999 }
  ]
}
```

An object list is never structural: rows come from the keys the game wrote, so
`allow_add` and `allow_remove` do not apply.

### Adding and removing rows

```json
"allow_add": true,
"allow_remove": true,
"min_items": 0,
"max_items": 20,
"new_item": { "path": "res://scn/player/mutations/all/damage_mutation.tres" }
```

Only where it is safe. Pathogenic allows it for mutations, which the game itself
grows and shrinks, and refuses it for equipment, whose slots the game names;
inventing an entry there would break the save.

Structural changes are written immediately rather than batched with field edits,
because inserting or deleting renumbers every row after it.

### Setting a whole list at once

```json
"bulk_actions": [
  { "id": "all_on", "label": "Discover all", "value": true }
]
```

Nobody should tick forty-nine flags by hand. Add `"field": "rarity"` to target
one column of an array-backed list.

### Pictures for the items themselves

Dropdown options can show the game's own artwork, read out of the installed copy
on the player's machine:

```json
"item_icons": [
  {
    "options_ref": "weapons",
    "format": "godot_pck",
    "archive": "{STEAM}/steamapps/common/Pathogenic/pathogenic.pck",
    "resource_pattern": "scn/player/bodyparts/external/{value}.tres"
  }
]
```

**Point at the resource, not at a picture.** Filenames are not an index into
game art: Pathogenic's `assault_rifle` draws `Player weapon - 3_shot_burst.png`
and `cannon` draws `Player weapon 2.png`. Matching option values against image
names covered barely a third of the parts. Following each part's resource to
whatever texture it actually references covers all 118.

For `godot_pck` the reader walks `{value}.tres` → its export remap → the binary
resource → the first image it names → the imported `.ctex` → the WebP or PNG
inside. Block-compressed textures are declined rather than guessed at, and a
game that is not installed simply yields no pictures.

Only `godot_pck` exists today. Unity and Unreal archives are a much larger job,
Unreal in particular often encrypts, and decoding a `UTexture2D` means writing a
BC7 decoder, so those games fall back to names, which is a perfectly good
outcome. If a game keeps its icons as loose image files, no reader is needed at
all: `icon_sources`-style globbing would do.

> **The same rule as store artwork applies:** nothing is bundled. Everything is
> read from the copy the player already owns.

### Translating your labels

Any `label`, `description` or `help` can carry a companion map:

```json
{
  "id": "money",
  "label": "Money",
  "label_i18n": { "tr": "Para", "de": "Geld", "ja": "お金" },
  "pointer": "/player/money",
  "type": "integer"
}
```

`description_i18n` and `help_i18n` work the same way, as do the labels on
groups, lists, `save_location`s and `choice` options.

Matching goes exact tag → base language → any regional variant → the plain
`label`, so `de-AT` finds a `de` entry and an unknown locale keeps your own
wording. Nothing ever renders blank, and a plugin with no translations at all
works perfectly.

**Translate your editor's vocabulary, not the game's.** Label the field
"Rarity" in every language you can, but leave "Rocket Launcher" as the game
writes it, because that is the name players search for in guides.

Full details, including how to handle a hundred options at once, are in
[translating.md](translating.md).

### `lists`: inventories, loadouts, quest logs

```json
{
  "id": "inventory",
  "label": "Inventory",
  "pointer": "/inventory",
  "item_label_pointer": "/name",
  "item_label_options_ref": "items",
  "item_filter": { "pointer": "/kind", "equals": ["weapon"] },
  "fields": [
    { "id": "qty", "label": "Quantity", "pointer": "/qty",
      "type": "integer", "min": 0, "max": 999 }
  ]
}
```

- `pointer` points at the array.
- `fields[].pointer` is relative to **an item**.
- `item_label_pointer` picks the item's display name;
  `item_label_options_ref` prettifies it through an option set, so the player
  sees `Rocket Launcher` rather than `rocket_launcher`.
- `item_filter` shows only matching items. This is how one array becomes several
  lists with different rules. Pathogenic splits its equipment array into
  *Weapon slots* and *Organ slots* so an internal organ is never offered for an
  external slot.

The MVP edits existing items; adding and removing them is on the roadmap.

---

## A complete minimal example

For a save that looks like this:

```json
{ "player": { "name": "Alex", "health": 100, "gold": 250 } }
```

```json
{
  "id": "example-game",
  "name": "Example Game",
  "version": "1.0.0",
  "description": "A worked example for plugin authors.",
  "format": "json",
  "save_locations": [
    { "root": "{APPDATA}/ExampleGame", "pattern": "save_*.json" }
  ],
  "identify": [{ "pointer": "/player/health" }],
  "groups": [
    {
      "id": "character",
      "label": "Character",
      "fields": [
        { "id": "name", "label": "Name", "pointer": "/player/name",
          "type": "text", "max_length": 24 },
        { "id": "health", "label": "Health", "pointer": "/player/health",
          "type": "integer", "min": 1, "max": 9999 },
        { "id": "gold", "label": "Gold", "pointer": "/player/gold",
          "type": "integer", "min": 0, "max": 99999999 }
      ]
    }
  ]
}
```

A ready-to-copy version of this, with every field type, a read-only field, a
conditional group and a list, is in
[`docs/example-plugin/`](example-plugin/manifest.json). It lives under `docs/`
rather than `plugins/` so it does not show up as a real game. Copy the folder
into `plugins/` and rename it to start.

A fuller one, with option sets, filtered lists and two kinds of save file, is in
[`plugins/pathogenic/manifest.json`](../plugins/pathogenic/manifest.json). Its
long option lists are generated from the game's own data files by
[`plugins/pathogenic/tools/generate-manifest.ps1`](../plugins/pathogenic/tools/generate-manifest.ps1)
This is worth copying if your game has a hundred items and you would rather not type
them.

---

## When the manifest will not load

Settings lists every plugin that failed and why. Common causes:

| Message | Fix |
|---|---|
| `no manifest.json in this folder` | The file must be named exactly that |
| `unsupported save format 'xml'` | Only `json` ships; see below |
| `pointer 'player/money' must be a JSON pointer starting with '/'` | Add the leading `/` |
| `two fields share the pointer '/x'` | Two fields edit the same value |
| `field 'r' references unknown option set 'rarities'` | Check the `option_sets` key |
| `field 'r' is a choice but has no options` | Add `options` or `options_ref` |

---

## Games that are not JSON

Implement `FormatAdapter` in `src-tauri/src/plugins/adapter.rs`:

```rust
pub trait FormatAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn parse(&self, bytes: &[u8]) -> Result<Value>;
    fn write(&self, value: &Value) -> Result<Vec<u8>>;
}
```

Convert the save to `serde_json::Value` and back, register it in `adapter_for`,
and set `"format"` to your id. Everything else (fields, validation, lists,
backups, atomic writes, the whole GUI) works unchanged.

`parse` and `write` must round-trip: `write(parse(x))` has to stay loadable by
the game. Please add a test proving it does.

---

## Please don't

Plugins are the app's extension point, so it is worth saying plainly what is out
of scope:

- editing state that lives on someone else's server
- anything that works around DRM, anti-cheat, or licence checks
- multiplayer or competitive progression, where a change affects other players

A plugin should edit files that already belong to the person running it, for a
game they are playing on their own.
