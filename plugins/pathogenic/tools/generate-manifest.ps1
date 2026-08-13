$ErrorActionPreference = 'Stop'
# Label translations live in translations.json so they can be edited without
# touching PowerShell, and so non-Latin scripts survive however this file is
# opened.
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
# description and help string we have translations for. Strings with no entry
# such as body-part and mutation names, stay in English.
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
# The archive is looked for where Steam usually puts it, and $env:PATHOGENIC_PCK
# overrides that for anyone whose library lives on another drive. The output
# path is relative to this script, so a checkout works wherever it sits.
$out = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\manifest.json'))

$pck = $env:PATHOGENIC_PCK
if (-not $pck) {
    $candidates = @(
        "$env:ProgramFiles(x86)\Steam\steamapps\common\Pathogenic\pathogenic.pck"
        "$env:ProgramFiles\Steam\steamapps\common\Pathogenic\pathogenic.pck"
        "C:\Program Files (x86)\Steam\steamapps\common\Pathogenic\pathogenic.pck"
    )
    $pck = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}
if (-not $pck -or -not (Test-Path $pck)) {
    throw "pathogenic.pck bulunamadi. PATHOGENIC_PCK ortam degiskenine tam yolu yaz."
}

$fs = [System.IO.File]::OpenRead($pck)
$len = [Math]::Min(48MB, $fs.Length); $fs.Position = $fs.Length - $len
$buf = New-Object byte[] $len; $null = $fs.Read($buf, 0, $len); $fs.Close()
$s = [System.Text.Encoding]::UTF8.GetString($buf)

$paths = [regex]::Matches($s, 'res://scn/player/[A-Za-z0-9_/\.\-]{4,140}\.tres') |
    ForEach-Object { $_.Value } | Sort-Object -Unique

function Pretty([string]$name) {
    ($name -split '_' | ForEach-Object {
        if ($_.Length -le 2) { $_.ToUpper() } else { $_.Substring(0,1).ToUpper() + $_.Substring(1) }
    }) -join ' '
}
function Choices($names) {
    # Body-part and mutation names stay in English: they are game content, and
    # players look them up by those names in guides.
    $names | ForEach-Object { [ordered]@{ value = $_; label = (Pretty $_) } }
}

$external = $paths | Where-Object { $_ -match '/bodyparts/external/' } |
    ForEach-Object { [IO.Path]::GetFileNameWithoutExtension($_) } | Sort-Object -Unique
$internal = $paths | Where-Object { $_ -match '/bodyparts/internal/' } |
    ForEach-Object { [IO.Path]::GetFileNameWithoutExtension($_) } | Sort-Object -Unique
$mutationPaths = $paths | Where-Object { $_ -match '/mutations/all/' } | Sort-Object -Unique

Write-Host "external: $($external.Count)  internal: $($internal.Count)  mutations: $($mutationPaths.Count)"
if ($external.Count -lt 20 -or $internal.Count -lt 20 -or $mutationPaths.Count -lt 20) {
    throw "PCK'dan cikan liste beklenenden kucuk - manifest uretilmedi"
}

$mutChoices = $mutationPaths | ForEach-Object {
    $n = [IO.Path]::GetFileNameWithoutExtension($_) -replace '_mutation$', ''
    [ordered]@{ value = $_; label = (Pretty $n) }
}

$rarity = @(
    [ordered]@{ value = 0; label = 'Common' }
    [ordered]@{ value = 1; label = 'Rare' }
    [ordered]@{ value = 2; label = 'Epic' }
    [ordered]@{ value = 3; label = 'Legendary' }
)

function IntField($id, $label, $ptr, $min, $max, $help) {
    [ordered]@{ id = $id; label = $label; pointer = $ptr; type = 'integer'; min = $min; max = $max; help = $help }
}
function NumField($id, $label, $ptr, $min, $max, $help) {
    [ordered]@{ id = $id; label = $label; pointer = $ptr; type = 'number'; min = $min; max = $max; help = $help }
}

# Filter on the slot-name prefix, not an enumerated list. The game turned out
# to use ESlot5 and ESlot6 in late runs (visible in past_runs.json) and an
# enumerated filter hid those weapons from the editor entirely. A prefix keeps
# working when the game grows another slot.
$weaponSlotPrefixes = @('ESlot','EBackSlot')
$organSlotPrefixes  = @('ISlot')

$manifest = [ordered]@{
    id          = 'pathogenic'
    name        = 'Pathogenic'
    version     = '1.0.0'
    author      = 'Universal Save Editor contributors'
    description = 'Roguelike built in Godot. Saves are plain JSON, one folder per profile.'
    format      = 'json'

    # No artwork is bundled: game art is copyrighted and this repository is
    # MIT. Instead we point at what Steam already cached for a player who owns
    # the game (app 3808690).
    #
    # Steam keeps several sizes side by side. The portrait capsule comes first
    # because the game grid is built around 2:3 cover art, the way a Steam
    # library looks; the landscape header is the fallback for games that have
    # no capsule, and the 32x32 top-level icon is last because it turns to mush
    # at any size worth looking at. Steam names the same portrait art two
    # different ways depending on when it was cached, so both are listed.
    icon_sources = @(
        [ordered]@{ path = '{STEAM}/appcache/librarycache/3808690/*/library_600x900.jpg' }
        [ordered]@{ path = '{STEAM}/appcache/librarycache/3808690/*/library_capsule.jpg' }
        [ordered]@{ path = '{STEAM}/appcache/librarycache/3808690/*/library_header.jpg' }
        [ordered]@{ path = '{STEAM}/appcache/librarycache/3808690/*/logo.png' }
        [ordered]@{ path = '{STEAM}/appcache/librarycache/3808690/*.jpg' }
    )


    # Used only to warn that the game is open and may write over the edit when
    # it exits. Nothing is started or stopped.
    process_names = @('pathogenic')

    # Copies Pathogenic keeps for itself: a rolling pair of .bak files, and
    # anything it refused to load, set aside with a unix timestamp in the name.
    # Only ever read.
    recovery_patterns = @('*.json.bak', '*.json.bak2', 'corrupted_*_save.json')

    constraints = @(
        [ordered]@{ left = '/player/hp'; right = '/player/max_hp'; rule = 'lte'
                    message = 'Health cannot be higher than max health.' }
        [ordered]@{ left = '/player/stamina'; right = '/player/max_stamina'; rule = 'lte'
                    message = 'Stamina cannot be higher than max stamina.' }
    )

    # Item artwork, read out of the player's own installed copy. Nothing is
    # bundled. Names are useless as an index here: `assault_rifle` draws a
    # file called "Player weapon - 3_shot_burst.png", so we point at each
    # part's resource and let the reader follow it to whatever texture it
    # actually uses.
    item_icons = @(
        [ordered]@{
            options_ref      = 'weapons'
            format           = 'godot_pck'
            archive          = '{STEAM}/steamapps/common/Pathogenic/pathogenic.pck'
            resource_pattern = 'scn/player/bodyparts/external/{value}.tres'
        }
        [ordered]@{
            options_ref      = 'organs'
            format           = 'godot_pck'
            archive          = '{STEAM}/steamapps/common/Pathogenic/pathogenic.pck'
            resource_pattern = 'scn/player/bodyparts/internal/{value}.tres'
        }
    )

    save_locations = @(
        # A run in progress. This file only exists while a run is unfinished;
        # the game deletes it when the run ends.
        [ordered]@{ platforms = @('windows'); root = '{APPDATA}/Godot/app_userdata/Pathogenic'
                    pattern = 'profile_*/run_save.json'; label = 'Current run'
                    identify = @([ordered]@{ pointer = '/player/hp' }, [ordered]@{ pointer = '/player/money' }) }
        [ordered]@{ platforms = @('linux','macos'); root = '{APPDATA}/godot/app_userdata/Pathogenic'
                    pattern = 'profile_*/run_save.json'; label = 'Current run'
                    identify = @([ordered]@{ pointer = '/player/hp' }, [ordered]@{ pointer = '/player/money' }) }

        # Permanent progression. Always present once the game has been played.
        [ordered]@{ platforms = @('windows'); root = '{APPDATA}/Godot/app_userdata/Pathogenic'
                    pattern = 'profile_*/save.json'; label = 'Profile'
                    identify = @([ordered]@{ pointer = '/plasmids/fragment_num' }) }
        [ordered]@{ platforms = @('linux','macos'); root = '{APPDATA}/godot/app_userdata/Pathogenic'
                    pattern = 'profile_*/save.json'; label = 'Profile'
                    identify = @([ordered]@{ pointer = '/plasmids/fragment_num' }) }
    )

    identify = @(
        [ordered]@{ pointer = '/player/hp' }
        [ordered]@{ pointer = '/player/money' }
    )

    label = [ordered]@{
        title_pointer    = '/player/player_scene'
        subtitle_pointer = '/seed'
        subtitle_prefix  = 'Seed: '
    }

    option_sets = [ordered]@{
        rarity    = $rarity
        weapons   = @(Choices $external)
        organs    = @(Choices $internal)
        mutations = @($mutChoices)
    }

    groups = @(
        [ordered]@{
            id = 'progression'; label = 'Progression'
            requires = '/plasmids/fragment_num'
            description = 'Permanent unlocks, kept between runs.'
            fields = @(
                (NumField 'fragments' 'Plasmid fragments' '/plasmids/fragment_num' 0 999999 'Spent on permanent upgrades.')
                (NumField 'best_money' 'Best money in a run' '/stats0/max_money' 0 999999999 '')
                (NumField 'best_hp'    'Best max health'    '/stats0/max_hp'    0 999 '')
                (NumField 'kills'      'Enemies killed'     '/stats0/enemies_killed' 0 9999999 '')
                (NumField 'bosses'     'Bosses beaten'      '/stats0/boss_beaten' 0 9999 '')
                (NumField 'max_floor'  'Deepest floor'      '/antibiotics0/max_floor_finished' 0 99 '')
            )
        }
        [ordered]@{
            id = 'character'; label = 'Character'
            requires = '/player'
            when_absent = 'Only while a run is in progress. The game deletes the run file when a run ends, so start one and quit to the menu to edit it here.'
            description = 'Your creature in the run that is currently in progress.'
            fields = @(
                (IntField 'hp'          'Health'      '/player/hp'          1 999       'Current health. The game shows this as hearts, so very large numbers can look odd on screen.')
                (IntField 'max_hp'      'Max health'  '/player/max_hp'      1 999       'Keep this at or above Health.')
                (IntField 'money'       'Money'       '/player/money'       0 999999999 '')
                (IntField 'armor'       'Armor'       '/player/armor'       0 99        '')
                (IntField 'dna'         'DNA'         '/player/dna'         0 999999    '')
                (NumField 'stamina'     'Stamina'     '/player/stamina'     0 9999      '')
                (NumField 'max_stamina' 'Max stamina' '/player/max_stamina' 0 9999      '')
                (IntField 'level'       'Level'       '/player/level'       1 999       '')
            )
        }
        [ordered]@{
            id = 'run'; label = 'Run'
            requires = '/level_number'
            when_absent = 'Only while a run is in progress.'
            description = 'Progress through the current run.'
            fields = @(
                (IntField 'level_number'      'Floor'            '/level_number'      1 99  '')
                (IntField 'rerolls_available' 'Rerolls'          '/rerolls_available' 0 999 '')
                (IntField 'endless_tier'      'Endless tier'     '/endless_tier'      0 99  '')
                [ordered]@{ id = 'seed'; label = 'Seed'; pointer = '/seed'; type = 'text'; read_only = $true
                            help = 'Shown for reference. Changing it would not regenerate the run you are in.' }
            )
        }
        [ordered]@{
            id = 'equipment'; label = 'Equipment'
            requires = '/player/loadout'
            when_absent = 'Only while a run is in progress. The permanent loadout you begin every run with is under Starting equipment.'
            description = 'Body parts fitted in the current run. Rarity runs Common to Legendary.'
            lists = @(
                [ordered]@{
                    id = 'weapons'; label = 'Weapon slots'
                    description = 'External parts. Only parts that fit an external slot are offered.'
                    pointer = '/player/loadout'
                    item_filter = [ordered]@{ pointer = '/slot'; starts_with = $weaponSlotPrefixes }
                    item_label_pointer = '/bodypart'; item_label_options_ref = 'weapons'
                    bulk_actions = @(
                        [ordered]@{ id = 'all_legendary'; label = 'Make all Legendary'; field = 'rarity'; value = 3 }
                    )
                    fields = @(
                        [ordered]@{ id = 'bodypart'; label = 'Part';   pointer = '/bodypart'; type = 'choice'; options_ref = 'weapons' }
                        [ordered]@{ id = 'rarity';   label = 'Rarity'; pointer = '/rarity';   type = 'choice'; options_ref = 'rarity' }
                    )
                }
                [ordered]@{
                    id = 'organs'; label = 'Organ slots'
                    description = 'Internal parts. Only parts that fit an internal slot are offered.'
                    pointer = '/player/loadout'
                    item_filter = [ordered]@{ pointer = '/slot'; starts_with = $organSlotPrefixes }
                    item_label_pointer = '/bodypart'; item_label_options_ref = 'organs'
                    bulk_actions = @(
                        [ordered]@{ id = 'all_legendary'; label = 'Make all Legendary'; field = 'rarity'; value = 3 }
                    )
                    fields = @(
                        [ordered]@{ id = 'bodypart'; label = 'Part';   pointer = '/bodypart'; type = 'choice'; options_ref = 'organs' }
                        [ordered]@{ id = 'rarity';   label = 'Rarity'; pointer = '/rarity';   type = 'choice'; options_ref = 'rarity' }
                    )
                }
            )
        }
        # Editable without an active run: this is the permanent starting
        # loadout kept in the profile save, and it has exactly the same shape
        # as the in-run one, so the existing option sets and filters apply.
        [ordered]@{
            id = 'starting_equipment'; label = 'Starting equipment'
            requires = '/loadouts/player0'
            description = 'The parts you begin every run with. Kept between runs.'
            lists = @(
                [ordered]@{
                    id = 'starting_weapons'; label = 'Weapon slots'
                    description = 'External parts. Only parts that fit an external slot are offered.'
                    pointer = '/loadouts/player0'
                    item_filter = [ordered]@{ pointer = '/slot'; starts_with = $weaponSlotPrefixes }
                    item_label_pointer = '/bodypart'; item_label_options_ref = 'weapons'
                    bulk_actions = @(
                        [ordered]@{ id = 'all_legendary'; label = 'Make all Legendary'; field = 'rarity'; value = 3 }
                    )
                    fields = @(
                        [ordered]@{ id = 'bodypart'; label = 'Part';   pointer = '/bodypart'; type = 'choice'; options_ref = 'weapons' }
                        [ordered]@{ id = 'rarity';   label = 'Rarity'; pointer = '/rarity';   type = 'choice'; options_ref = 'rarity' }
                    )
                }
                [ordered]@{
                    id = 'starting_organs'; label = 'Organ slots'
                    description = 'Internal parts. Only parts that fit an internal slot are offered.'
                    pointer = '/loadouts/player0'
                    item_filter = [ordered]@{ pointer = '/slot'; starts_with = $organSlotPrefixes }
                    item_label_pointer = '/bodypart'; item_label_options_ref = 'organs'
                    bulk_actions = @(
                        [ordered]@{ id = 'all_legendary'; label = 'Make all Legendary'; field = 'rarity'; value = 3 }
                    )
                    fields = @(
                        [ordered]@{ id = 'bodypart'; label = 'Part';   pointer = '/bodypart'; type = 'choice'; options_ref = 'organs' }
                        [ordered]@{ id = 'rarity';   label = 'Rarity'; pointer = '/rarity';   type = 'choice'; options_ref = 'rarity' }
                    )
                }
            )
        }

        # Five plain JSON objects holding 161 flags and counters between them.
        # Object-backed lists are what make these reachable at all.
        [ordered]@{
            id = 'discovery'; label = 'Discoveries'
            requires = '/enemy_discoveries'
            description = 'What the game has recorded you finding. Keys come from the game itself.'
            lists = @(
                [ordered]@{
                    id = 'enemy_discoveries'; label = 'Enemies discovered'
                    pointer = '/enemy_discoveries'; source = 'object'
                    bulk_actions = @(
                        [ordered]@{ id = 'all_on';  label = 'Discover all'; value = $true }
                        [ordered]@{ id = 'all_off'; label = 'Forget all';   value = $false }
                    )
                    entry = [ordered]@{ id = 'found'; label = 'Discovered'; pointer = ''; type = 'boolean' }
                }
                [ordered]@{
                    id = 'mutation_discoveries'; label = 'Mutations discovered'
                    pointer = '/mutation_discoveries'; source = 'object'
                    bulk_actions = @(
                        [ordered]@{ id = 'all_on';  label = 'Discover all'; value = $true }
                        [ordered]@{ id = 'all_off'; label = 'Forget all';   value = $false }
                    )
                    entry = [ordered]@{ id = 'found'; label = 'Discovered'; pointer = ''; type = 'boolean' }
                }
                [ordered]@{
                    id = 'bodypart_discoveries'; label = 'Body parts discovered'
                    pointer = '/discoveries'; source = 'object'
                    description = 'Stored as a number rather than a flag; 0 means undiscovered.'
                    bulk_actions = @(
                        [ordered]@{ id = 'all_on'; label = 'Discover all'; value = 3 }
                    )
                    entry = [ordered]@{ id = 'level'; label = 'Level'; pointer = ''; type = 'number'; min = 0; max = 99 }
                }
                [ordered]@{
                    id = 'achievements'; label = 'Achievements'
                    pointer = '/achievements'; source = 'object'
                    description = 'Flags in this local file only. Steam achievements are a separate system and are not touched.'
                    bulk_actions = @(
                        [ordered]@{ id = 'all_on'; label = 'Unlock all'; value = $true }
                    )
                    entry = [ordered]@{ id = 'earned'; label = 'Earned'; pointer = ''; type = 'boolean' }
                }
                [ordered]@{
                    id = 'kill_counts'; label = 'Enemies killed'
                    pointer = '/enemy_kill_counts'; source = 'object'
                    entry = [ordered]@{ id = 'count'; label = 'Kills'; pointer = ''; type = 'number'; min = 0; max = 9999999 }
                }
            )
        }

        [ordered]@{
            id = 'mutations'; label = 'Mutations'
            requires = '/player/mutations'
            when_absent = 'Only while a run is in progress.'
            description = 'Mutations active in this run. Swap any slot for another mutation.'
            lists = @(
                [ordered]@{
                    id = 'mutations'; label = 'Active mutations'; pointer = '/player/mutations'
                    item_label_pointer = '/path'; item_label_options_ref = 'mutations'
                    # Mutations are a plain list the game grows and shrinks, so
                    # adding one is safe. Equipment is not: its slots are named
                    # by the game and inventing an entry would break the save.
                    allow_add = $true; allow_remove = $true
                    min_items = 0; max_items = 20
                    new_item = [ordered]@{ path = 'res://scn/player/mutations/all/damage_mutation.tres' }
                    fields = @(
                        [ordered]@{ id = 'path'; label = 'Mutation'; pointer = '/path'; type = 'choice'; options_ref = 'mutations' }
                    )
                }
            )
        }
    )
}

$manifest.presets = @(
    [ordered]@{
        id = 'refill'; label = 'Refill health and stamina'
        requires = '/player/max_hp'
        description = 'Sets health to your maximum, and stamina with it.'
        set = @(
            [ordered]@{ pointer = '/player/hp';      value = 999 }
            [ordered]@{ pointer = '/player/stamina'; value = 9999 }
        )
    }
    [ordered]@{
        id = 'rich'; label = 'Plenty of money'
        requires = '/player/money'
        description = 'Enough to buy out every shop without leaving the safe range.'
        set = @([ordered]@{ pointer = '/player/money'; value = 999999 })
    }
    [ordered]@{
        id = 'legendary_run'; label = 'Make this run''s equipment Legendary'
        requires = '/player/loadout'
        set_in_lists = @(
            [ordered]@{ list = 'weapons'; field = 'rarity'; value = 3 }
            [ordered]@{ list = 'organs';  field = 'rarity'; value = 3 }
        )
    }
    [ordered]@{
        id = 'legendary_start'; label = 'Make starting equipment Legendary'
        requires = '/loadouts/player0'
        description = 'Applies to every future run, not just this one.'
        set_in_lists = @(
            [ordered]@{ list = 'starting_weapons'; field = 'rarity'; value = 3 }
            [ordered]@{ list = 'starting_organs';  field = 'rarity'; value = 3 }
        )
    }
    [ordered]@{
        id = 'discover_all'; label = 'Mark everything as discovered'
        requires = '/enemy_discoveries'
        description = 'Fills in the bestiary and the mutation list.'
        set_in_lists = @(
            [ordered]@{ list = 'enemy_discoveries';    value = $true }
            [ordered]@{ list = 'mutation_discoveries'; value = $true }
            [ordered]@{ list = 'achievements';         value = $true }
        )
    }
)

Localise $manifest
Write-Host "cevrilen metin: $script:Localised (x $($script:LANGS.Count) dil)"

New-Item -ItemType Directory -Force -Path (Split-Path $out) | Out-Null
$json = $manifest | ConvertTo-Json -Depth 12
[System.IO.File]::WriteAllText($out, $json, (New-Object System.Text.UTF8Encoding($false)))
Write-Host "yazildi: $out  ($([math]::Round((Get-Item $out).Length/1KB,1)) KB)"
