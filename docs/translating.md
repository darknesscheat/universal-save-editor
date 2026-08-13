# Translating

The app ships in 13 languages. There are two separate things to translate, and
they live in different places.

| What | Where | Who |
|---|---|---|
| The app's own text: buttons, messages, errors | `src/i18n/locales/` | anyone |
| A game's field labels such as "Money", "Weapon slots" | that plugin's `manifest.json` | plugin authors |

Everything currently shipped was machine-assisted and **has not been reviewed by
native speakers**. Corrections are genuinely welcome and are the easiest
possible first contribution.

---

## Languages

`en` · `tr` · `de` · `es` · `fr` · `it` · `pt-BR` · `ru` · `pl` · `uk` · `ja` ·
`ko` · `zh-CN`

The app picks one at startup: a saved choice first, otherwise the operating
system's language, otherwise English. Settings has a picker, and each language
is listed in its own words, so someone who cannot read the current language can
still find theirs.

---

## Fixing or adding an app translation

`src/i18n/locales/en.ts` is the reference. Every key lives there; other files
override what they have and **fall through to English for the rest**, so a
half-finished language is still perfectly usable.

To fix a string, edit that language's file:

```ts
// src/i18n/locales/tr.ts
export const tr: LocaleStrings = {
  "editor.save": "Değişiklikleri kaydet",
};
```

To add a language:

1. Copy `en.ts` to `<tag>.ts` and translate the values.
2. Change the type to `LocaleStrings` and rename the export.
3. Register it in `src/i18n/index.ts`:

```ts
import { nl } from "./locales/nl";

export const LANGUAGES: Language[] = [
  // …
  { tag: "nl", name: "Nederlands", strings: nl },
];
```

`name` is the language's own name for itself, never translated.

TypeScript checks your keys: a typo is a build error, not a string that
silently never appears.

### Counted strings

`{count}` strings are resolved through the language's real plural rules, using
`Intl.PluralRules`. English needs two forms:

```ts
"editor.ready_one": "{count} change ready",
"editor.ready_other": "{count} changes ready",
```

Russian, Polish and Ukrainian need four: `_one`, `_few`, `_many`, `_other`:

```ts
"editor.ready_one": "{count} zmiana gotowa",
"editor.ready_few": "{count} zmiany gotowe",
"editor.ready_many": "{count} zmian gotowych",
"editor.ready_other": "{count} zmian gotowych",
```

Japanese, Korean and Chinese need only `_other`. Supply whichever categories
your language actually has; missing ones fall back to `_other`.

### Error messages

Errors come from Rust as a stable code plus parameters, and are translated in
the frontend. `error.fieldRule` embeds a nested `rule.*` string, which is why
both exist:

```ts
"error.fieldRule": "“{field}” {reason}",
"rule.tooSmall": "cannot be lower than {limit}.",
// → “Money” cannot be lower than 0.
```

If your language cannot put the field name first, rewrite `error.fieldRule` so
it reads naturally. `{field}` and `{reason}` can go anywhere.

An untranslated error code falls back to the English sentence Rust sent along,
so a missing translation can never leave a blank message.

---

## Translating a plugin's labels

Any `label`, `description` or `help` in a manifest can carry a companion map:

```json
{
  "id": "money",
  "label": "Money",
  "label_i18n": { "tr": "Para", "de": "Geld", "ja": "お金" },
  "pointer": "/player/money",
  "type": "integer"
}
```

The same applies to `description_i18n` and `help_i18n`, to group and list
labels, to a `save_location`'s `label`, and to a `choice`'s `label`.

Matching goes exact tag → base language → any regional variant → the plain
`label`. So `de-AT` finds a `de` entry, `pt` finds a `pt-BR` one, and an unknown
locale simply keeps the manifest's own wording. Nothing ever comes out blank.

### What not to translate

**Item names stay in the game's own words.** The Pathogenic plugin translates
"Weapon slots" and "Rarity", but leaves "Rocket Launcher" and "Damage Mult"
exactly as the game writes them. Players look those up in guides and wikis by
their English names, and a translated name would make the app harder to use,
not easier.

Translate the editor's vocabulary; leave the game's vocabulary alone.

### Long option lists

The Pathogenic plugin has 118 body parts and 53 mutations. Its manifest is
generated rather than hand-written:

- `plugins/pathogenic/tools/translations.json` holds the label translations
- `plugins/pathogenic/tools/generate-manifest.ps1` reads the game's own data
  files for the option lists, then attaches the translations

To fix a Pathogenic label, edit `translations.json` and re-run the generator.
Editing `manifest.json` by hand works too, but the next regeneration overwrites
it.

---

## Checking your work

```bash
npx tsc --noEmit          # catches typos in translation keys
cd src-tauri && cargo test # checks bundled plugins cover every shipped language
```

`pathogenic_labels_are_translated_into_every_shipped_language` fails if a
bundled plugin gains a language the app ships but the plugin does not, so
adding a language to the app is a prompt to translate the plugins too.
