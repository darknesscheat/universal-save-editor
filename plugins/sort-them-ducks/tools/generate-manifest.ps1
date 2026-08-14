$ErrorActionPreference = 'Stop'
# Regenerate ../manifest.json.
#
# Label text lives in translations.json so it can be edited without touching
# PowerShell, and so non-Latin scripts survive however this file is opened.
#
# Unlike the other two plugins this one reads nothing out of the game: Sort Them
# Ducks writes plain JSON and every editable value is a named field, so there is
# no list to extract. The generator exists only to attach the translations.

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

function Flag($id, $label, $pointer) {
    [ordered]@{ id = $id; label = $label; pointer = $pointer; type = 'boolean' }
}
function Count($id, $label, $pointer, $max, $help) {
    $f = [ordered]@{ id = $id; label = $label; pointer = $pointer
                     type = 'integer'; min = 0; max = $max }
    if ($help) { $f.help = $help }
    $f
}

$manifest = [ordered]@{
    id          = 'sort-them-ducks'
    name        = 'Sort Them Ducks'
    version     = '1.0.0'
    author      = 'Universal Save Editor contributors'
    description = 'Unity game. The save is plain JSON, one file per player.'
    format      = 'json'

    # Read from the player's own Steam cache; nothing is bundled. Portrait
    # capsule first, because the game grid draws 2:3 covers.
    icon_sources = @(
        [ordered]@{ path = '{STEAM}/appcache/librarycache/4992070/*/library_600x900.jpg' }
        [ordered]@{ path = '{STEAM}/appcache/librarycache/4992070/*/library_capsule.jpg' }
        [ordered]@{ path = '{STEAM}/appcache/librarycache/4992070/*/library_header.jpg' }
        [ordered]@{ path = '{STEAM}/appcache/librarycache/4992070/*/logo.png' }
        [ordered]@{ path = '{STEAM}/appcache/librarycache/4992070/*.jpg' }
    )

    # The executable is "Sort Them Ducks.exe"; the check lowercases and drops
    # the extension before comparing.
    process_names = @('sort them ducks')

    # Unity's per-game folder. The company name is spelled "Mr_Duck" here and
    # "Mr.Duck" in the folder the log goes to, which is the game's own
    # inconsistency rather than a typo.
    save_locations = @(
        [ordered]@{
            platforms = @('windows')
            root      = '{LOCALLOW}/Mr_Duck/Sort Them Ducks'
            pattern   = 'duckgame_save.json'
            label     = 'Save'
            identify  = @(@{ pointer = '/money' }, @{ pointer = '/ducks' })
        }
    )

    groups = @(
        [ordered]@{
            id = 'progress'; label = 'Progress'
            requires = '/money'
            fields = @(
                [ordered]@{ id = 'money'; label = 'Money'; pointer = '/money'
                            type = 'number'; min = 0; max = 999999 },
                [ordered]@{ id = 'total_earned'; label = 'Total money earned'
                            pointer = '/totalMoneyEarned'; type = 'number'
                            min = 0; max = 9999999
                            help = 'A running total the game keeps for its own statistics. Changing money does not change it.' },
                (Count 'ducks_on_shelves' 'Ducks on shelves' '/totalDucksOnShelves' 9999 $null),
                (Count 'filled_shelves' 'Filled shelves' '/totalFilledShelves' 999 $null),
                (Count 'eggs_counter' 'Eggs collected' '/totalCollectedEggs' 10 'This counter is separate from the eggs listed below. Setting one does not update the other.'),
                (Count 'find_same_uses' 'Times "find the same duck" was used' '/findSameDuckUseCount' 9999 $null),
                [ordered]@{ id = 'speedrun'; label = 'Speedrun time'
                            pointer = '/speedRunElapsedSeconds'; type = 'number'
                            min = 0; max = 999999; help = 'In seconds.' }
            )
        },
        [ordered]@{
            id = 'abilities'; label = 'Abilities'
            requires = '/sprintUnlocked'
            fields = @(
                (Flag 'sprint' 'Sprint' '/sprintUnlocked'),
                (Flag 'crouch' 'Crouch' '/crouchUnlocked'),
                (Flag 'dunk' 'Dunk a duck' '/dunkADuckUnlocked'),
                (Flag 'find_same' 'Find the same duck' '/findSameDuckUnlocked'),
                (Flag 'collect' 'Collect duck' '/collectDuckUnlocked'),
                (Flag 'find_shelf' 'Find a shelf for a duck' '/findShelfForDuckUnlocked'),
                (Flag 'egg_unlocked' 'Player egg unlocked' '/playerEggUnlocked'),
                (Flag 'egg_cracked' 'Player egg cracked' '/playerEggCracked'),
                (Flag 'cashier' 'Cashier money collected' '/cashierMoneyCollected'),
                (Flag 'magic' 'Has used magic' '/hasUsedMagic')
            )
        },
        [ordered]@{
            id = 'upgrades'; label = 'Upgrades'
            description = "The game's own ceiling for these is unknown, so the range here is a guess. A value the game rejects will be corrected the next time it saves."
            requires = '/inventorySizeLevel'
            fields = @(
                (Count 'inventory' 'Inventory size' '/inventorySizeLevel' 10 $null),
                (Count 'throw' 'Throw strength' '/strongerThrowLevel' 10 $null),
                (Count 'reach' 'Reach' '/longerReachLevel' 10 $null)
            )
        },
        [ordered]@{
            id = 'collectibles'; label = 'Collectibles'
            requires = '/eggs'
            lists = @(
                [ordered]@{
                    id = 'eggs'; label = 'Eggs'; description = 'The ten hidden eggs.'
                    pointer = '/eggs'; source = 'array'
                    # The array is not in egg order, so the row is named from
                    # the id in the data rather than from its position.
                    item_label_pointer = '/eggId'
                    fields = @(
                        [ordered]@{ id = 'collected'; label = 'Collected'
                                    pointer = '/isCollected'; type = 'boolean' }
                    )
                    bulk_actions = @(
                        [ordered]@{ id = 'all_on'; label = 'Collect all'
                                    field = 'collected'; value = $true }
                        [ordered]@{ id = 'all_off'; label = 'Clear all'
                                    field = 'collected'; value = $false }
                    )
                },
                [ordered]@{
                    id = 'bombs'; label = 'Bombs'
                    pointer = '/bombs'; source = 'array'
                    item_label_pointer = '/bombId'
                    fields = @(
                        [ordered]@{ id = 'active'; label = 'Active'
                                    pointer = '/isActive'; type = 'boolean' }
                    )
                }
            )
        }
    )

    presets = @(
        [ordered]@{
            id = 'rich'; label = 'Get rich'; description = 'Set money to 999,999.'
            requires = '/money'
            set = @(@{ pointer = '/money'; value = 999999 })
        },
        [ordered]@{
            id = 'abilities'; label = 'Unlock every ability'
            description = 'Turn on sprint, crouch, dunking and the three finding aids.'
            requires = '/sprintUnlocked'
            set = @(
                @{ pointer = '/sprintUnlocked'; value = $true }
                @{ pointer = '/crouchUnlocked'; value = $true }
                @{ pointer = '/dunkADuckUnlocked'; value = $true }
                @{ pointer = '/findSameDuckUnlocked'; value = $true }
                @{ pointer = '/collectDuckUnlocked'; value = $true }
                @{ pointer = '/findShelfForDuckUnlocked'; value = $true }
            )
        },
        [ordered]@{
            id = 'upgrades'; label = 'Max out the upgrades'
            description = 'Set inventory size, throw strength and reach to 10. The real ceiling is unknown, so the game may trim these.'
            requires = '/inventorySizeLevel'
            set = @(
                @{ pointer = '/inventorySizeLevel'; value = 10 }
                @{ pointer = '/strongerThrowLevel'; value = 10 }
                @{ pointer = '/longerReachLevel'; value = 10 }
            )
        }
    )
}

Localise $manifest

$out = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\manifest.json'))
$json = $manifest | ConvertTo-Json -Depth 24
# No BOM: the app reads UTF-8 and serde_json will not accept one.
[System.IO.File]::WriteAllText($out, $json, (New-Object System.Text.UTF8Encoding($false)))
Write-Host "wrote $out with $($manifest.groups.Count) groups, $script:Localised localised strings"
