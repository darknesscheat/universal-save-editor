$ErrorActionPreference = 'Stop'
# Regenerate ../manifest.json.
#
# Two inputs, both checked in:
#   translations.json  holds the label text, keyed by the English string
#   tools.json         holds the 77 tool ids, produced by extract-tools.py from an
#                       installed copy of the game
#
# The three save slots have identical structure, which is the whole reason this
# generator exists: the slot section is written once here and emitted three
# times, instead of ~500 lines of hand-copied JSON drifting apart.

$trFile = Join-Path $PSScriptRoot 'translations.json'
$script:TR = [System.IO.File]::ReadAllText($trFile, [System.Text.Encoding]::UTF8) | ConvertFrom-Json
$script:LANGS = $script:TR.languages

function Tr([string]$english) {
    $entry = $script:TR.strings.PSObject.Properties[$english]
    if (-not $entry) { return $null }
    $out = [ordered]@{}
    foreach ($lang in $script:LANGS) {
        $val = $entry.Value.PSObject.Properties[$lang]
        if ($val -and $val.Value) { $out[$lang] = $val.Value }
    }
    return $out
}

# Walk the finished manifest and attach a *_i18n map next to every label,
# description and help string we have translations for. Strings with no entry,
# such as tool and card names, stay in English.
$script:Localised = 0
function Localise($node) {
    if ($node -is [System.Collections.Specialized.OrderedDictionary]) {
        foreach ($key in @('label','description','help','when_absent')) {
            if ($node.Contains($key) -and $node[$key]) {
                $tr = Tr $node[$key]
                if ($tr -and $tr.Count -gt 0) {
                    $node["${key}_i18n"] = $tr
                    $script:Localised++
                }
            }
        }
        foreach ($key in @($node.Keys)) { Localise $node[$key] }
    }
    elseif ($node -is [System.Array]) {
        foreach ($item in $node) { Localise $item }
    }
}

$toolsFile = Join-Path $PSScriptRoot 'tools.json'
$toolsDoc = [System.IO.File]::ReadAllText($toolsFile, [System.Text.Encoding]::UTF8) | ConvertFrom-Json

# An empty id is how the game marks a slot with nothing in it, so it has to be
# offered as a choice. Otherwise a slot could be filled but never cleared.
$toolChoices = @([ordered]@{ value = ''; label = '(empty)' })
foreach ($prop in $toolsDoc.tools.PSObject.Properties) {
    $toolChoices += [ordered]@{ value = $prop.Name; label = $prop.Value.label }
}
Write-Host "tools.json: $($toolChoices.Count - 1) tools"
if ($toolChoices.Count -lt 50) {
    throw "only $($toolChoices.Count - 1) tools. Re-run extract-tools.py against the game's .pck"
}

function ToolSlots([string]$slot, [string]$key, [string]$label, [string]$description) {
    # Godot writes these as an object keyed "0", "1", … rather than an array,
    # and each value is a record. Hence source=object with `fields`.
    [ordered]@{
        id          = "slot${slot}_$key"
        label       = $label
        description = $description
        pointer     = "/tracked_progress/save_slots/$slot/$key"
        source      = 'object'
        fields      = @(
            [ordered]@{
                id = 'id'; label = 'Tool'; pointer = '/id'
                type = 'choice'; options_ref = 'tools'
            },
            [ordered]@{
                id = 'durability'; label = 'Durability'; pointer = '/durability'
                type = 'integer'; min = 0; max = 999
                help = 'How much use the tool has left. The game caps this per tool, so a very high number may be trimmed when the tool is next used.'
            }
        )
    }
}

function SlotGroup([string]$slot) {
    $base = "/tracked_progress/save_slots/$slot"
    [ordered]@{
        id    = "slot$slot"
        label = "Save slot $slot"
        # An unstarted slot is written as {}: present, but with nothing in it.
        # `requires` therefore points at a field, not at the slot itself.
        requires    = "$base/currency"
        when_absent = 'This slot is empty. Start a game in it and its fields appear here.'
        fields = @(
            [ordered]@{ id = 'currency'; label = 'Money'; pointer = "$base/currency"
                        type = 'number'; min = 0; max = 999999 },
            [ordered]@{ id = 'mission'; label = 'Mission'; pointer = "$base/mission_index"
                        type = 'integer'; min = 0; max = 99 },
            [ordered]@{ id = 'difficulty'; label = 'Difficulty'; pointer = "$base/difficulty"
                        type = 'integer'; min = 0; max = 9 },
            [ordered]@{ id = 'location'; label = 'Location'; pointer = "$base/location"
                        type = 'integer'; min = 0; max = 99 }
        )
        lists = @(
            (ToolSlots $slot 'tools' 'Carried tools' 'The six slots you carry into the pit. An empty slot holds no tool.'),
            (ToolSlots $slot 'van_tools' 'Van tools' 'Four slots kept in the van between missions.')
        )
    }
}

$saveRoots = @(
    @{ platforms = @('windows');          root = '{APPDATA}/Godot/app_userdata/Feed The Pit/data' },
    @{ platforms = @('linux','macos');    root = '{APPDATA}/godot/app_userdata/Feed The Pit/data' }
)
$saveLocations = @()
foreach ($r in $saveRoots) {
    $saveLocations += [ordered]@{
        platforms = $r.platforms; root = $r.root; pattern = 'progress.save'
        label = 'Progress'
        identify = @(@{ pointer = '/tracked_progress/save_slots' })
    }
    $saveLocations += [ordered]@{
        platforms = $r.platforms; root = $r.root; pattern = 'character_memories.save'
        label = 'Character memories'
        identify = @(@{ pointer = '/death_amount' })
    }
}

$manifest = [ordered]@{
    id      = 'feed-the-pit'
    name    = 'Feed The Pit'
    version = '1.0.0'
    format  = 'json'
    # No `icon` key: bundling store art would put copyrighted images in an MIT
    # repo. These read from the player's own Steam cache instead (app 3278290).
    #
    # Sources are tried in order and the portrait capsule comes first, because
    # the game grid is built around 2:3 cover art the way a Steam library is.
    # The landscape header is the fallback for games with no capsule, and the
    # 32x32 top-level icon is last because it turns to mush at any size worth
    # looking at. Steam names the same portrait art two different ways
    # depending on when it was cached, so both are listed.
    icon_sources = @(
        @{ path = '{STEAM}/appcache/librarycache/3278290/*/library_600x900.jpg' },
        @{ path = '{STEAM}/appcache/librarycache/3278290/*/library_capsule.jpg' },
        @{ path = '{STEAM}/appcache/librarycache/3278290/*/library_header.jpg' },
        @{ path = '{STEAM}/appcache/librarycache/3278290/*/logo.png' },
        @{ path = '{STEAM}/appcache/librarycache/3278290/*.jpg' }
    )
    process_names = @('ftp')
    # There is no item_icons block. The game's .pck has an
    # encrypted directory, so the tool sprites cannot be located without
    # breaking that encryption, which is out of scope for this project. Tool names are
    # shown as text, which is a perfectly good outcome.
    save_locations = $saveLocations
    option_sets = [ordered]@{ tools = $toolChoices }
    groups = @(
        (SlotGroup '1'),
        (SlotGroup '2'),
        (SlotGroup '3'),
        [ordered]@{
            id = 'tracking'; label = 'Tracking'
            requires = '/tracking/mushrooms_collected'
            fields = @(
                [ordered]@{ id = 'mushrooms'; label = 'Mushrooms collected'
                            pointer = '/tracking/mushrooms_collected'
                            type = 'number'; min = 0; max = 999999 },
                [ordered]@{ id = 'version'; label = 'Save version'; pointer = '/version'
                            type = 'number'; read_only = $true
                            help = 'Written by the game. Shown so you can tell whether this plugin still matches your copy.' }
            )
        },
        [ordered]@{
            id = 'memories'; label = 'Character memories'
            requires = '/death_amount'
            fields = @(
                [ordered]@{ id = 'deaths'; label = 'Deaths'; pointer = '/death_amount'
                            type = 'integer'; min = 0; max = 999999 },
                [ordered]@{ id = 'death_dialogue'; label = 'Death dialogue index'
                            pointer = '/death_amount_dialogue_index'
                            type = 'integer'; min = 0; max = 999 },
                [ordered]@{ id = 'death_cards'; label = 'Death card encounters'
                            pointer = '/death_card_encounters'
                            type = 'integer'; min = 0; max = 999 },
                [ordered]@{ id = 'talked'; label = 'Has talked with the Cardmaster'
                            pointer = '/has_talked_with_cardmaster'; type = 'boolean' }
            )
            lists = @(
                [ordered]@{
                    id = 'cardmaster'; label = 'Cardmaster memories'
                    description = 'Four flags the game sets as its conversation with you progresses.'
                    pointer = '/cardmaster'; source = 'object'
                    # An object row *is* its value, so `entry` needs no pointer
                    # of its own: the row's key supplies the whole path.
                    entry = [ordered]@{ id = 'value'; label = 'Value'; pointer = ''
                                        type = 'integer'; min = 0; max = 99 }
                }
            )
        }
    )
    presets = @(
        [ordered]@{
            id = 'rich'; label = 'Get rich'
            description = 'Set the money in save slot 1 to 999,999.'
            requires = '/tracked_progress/save_slots/1/currency'
            edits = @(@{ pointer = '/tracked_progress/save_slots/1/currency'; value = 999999 })
        }
    )
}

Localise $manifest

$out = Join-Path $PSScriptRoot '..\manifest.json'
$json = $manifest | ConvertTo-Json -Depth 24
# No BOM: the app reads UTF-8, and a BOM is a byte serde_json will not accept.
[System.IO.File]::WriteAllText(
    [System.IO.Path]::GetFullPath($out), $json, (New-Object System.Text.UTF8Encoding($false)))
Write-Host "wrote $out with $($manifest.groups.Count) groups, $script:Localised localised strings"
