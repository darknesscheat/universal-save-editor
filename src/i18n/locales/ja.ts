import type { LocaleStrings } from "..";

/** Japanese has a single plural category, so only `_other` is needed. */
export const ja: LocaleStrings = {
  "app.title": "Universal Save Editor",
  "app.pickGame": "ゲームを選んでください",
  "app.chooseSave": "編集するセーブデータを選んでください",
  "app.backups": "バックアップ",
  "app.backupsSub": "どのセーブデータも元の状態に戻せます",
  "app.settings": "設定",
  "app.back": "戻る",

  "common.loading": "読み込み中…",
  "common.cancel": "キャンセル",

  "games.search": "ゲームを検索…",
  "games.noneInstalled": "ゲームプラグインがインストールされていません。",
  "games.addPlugin": "プラグインフォルダーに追加してください（設定を参照）。",
  "games.noMatch": "「{query}」に一致するゲームはありません。",
  "games.moreSoon":
    "対応ゲームは今後増えます。どれもプラグインなので、誰でも作れます。",

  "saves.looking": "セーブファイルを探しています…",
  "saves.none": "このパソコンに {game} のセーブデータが見つかりませんでした。",
  "saves.playOnce":
    "一度プレイしてセーブデータを作成してから、また来てください。",
  "saves.found_other":
    "セーブファイルが {count} 件見つかりました。選んで保存を押すまで、何も変更されません。",
  "saves.lastPlayed": "最終プレイ {when}",

  "editor.opening": "セーブデータを開いています…",
  "editor.save": "変更を保存",
  "editor.saving": "保存中…",
  "editor.noChanges": "変更はまだありません",
  "editor.ready_other": "{count} 件の変更が準備できました",
  "editor.needFixing_other": "{count} 件の値を直す必要があります",
  "editor.savedNothing": "変更の必要はありませんでした。セーブデータはそのままです。",
  "editor.saved_other":
    "保存しました。{count} 件の値を更新し、その前に元のバックアップを作成しました。",
  "editor.backupNote":
    "セーブデータを変更する前にバックアップを作成します。バックアップ画面からいつでも元に戻せます。",
  "editor.notPresent": "このセーブファイルには存在しません。",
  "editor.nothingHere": "このセーブデータにはここに何もありません。",
  "editor.notInList": "{value}（一覧にありません）",

  "backups.none": "バックアップはまだありません。",
  "backups.autoNote": "変更を保存するたびに自動で作成されます。",
  "backups.restore": "復元",
  "backups.delete": "削除",
  "backups.confirm": "現在のセーブデータを置き換えますか？",
  "backups.yesRestore": "はい、復元します",
  "backups.restored":
    "復元しました。セーブデータは元の状態に戻り、置き換えられた方もバックアップされています。",

  "settings.language": "言語",
  "settings.languageDesc": "ゲームプラグインが提供する項目名にも使われます。",
  "settings.backupsDesc":
    "このアプリが作成したバックアップはすべてここにあります。",
  "settings.openFolder": "フォルダーを開く",
  "settings.plugins": "プラグイン",
  "settings.pluginsDesc":
    "ゲームごとに 1 フォルダーです。いずれかにプラグインを置いて「再読み込み」を押してください。",
  "settings.reload": "プラグインを再読み込み",
  "settings.reloaded_other": "{count} 本のゲームが利用できます。",
  "settings.failedPlugins": "読み込めなかったプラグイン",
  "settings.cantOpenFolder": "そのフォルダーを開けませんでした。",
  "settings.about": "このアプリについて",
  "settings.aboutText":
    "Universal Save Editor {version} — 生のセーブファイルに触れずに、オフラインゲームのセーブデータを編集できます。すべてこのパソコン上で動作し、どこにも送信されません。",

  "field.enterValue": "値を入力してください。",
  "field.enterNumber": "数値を入力してください。",
  "field.wholeNumber": "小数点なしの整数を入力してください。",
  "field.tooSmall": "{limit} より小さくできません。",
  "field.tooLarge": "{limit} より大きくできません。",
  "field.tooLong": "{limit} 文字以内にしてください。",

  "error.pluginNotFound": "プラグイン「{id}」が見つかりませんでした。",
  "error.pluginLoad": "プラグインフォルダーを読み込めませんでした: {detail}",
  "error.saveMissing": "このセーブファイルはディスク上に存在しません。",
  "error.saveRead": "このセーブファイルを読み込めませんでした: {detail}",
  "error.saveFormat": "これは {game} のセーブファイルではないようです。",
  "error.validation": "「{field}」{reason}",
  "error.fieldRule": "「{field}」{reason}",
  "error.unknownField": "「{field}」はこのセーブデータで編集できる項目ではありません。",
  "error.pathNotAllowed":
    "そのファイルは、このプラグインが管理するフォルダーの外にあります。",
  "error.backupFailed":
    "バックアップを作成できなかったため、セーブデータには手を加えていません: {detail}",
  "error.backupNotFound": "そのバックアップは見つかりませんでした。",
  "error.writeFailed":
    "セーブデータを書き込めませんでした: {detail} 元のセーブデータは変更されていません。",
  "error.io": "{detail}",
  "error.unknown": "問題が発生しました。",

  "rule.notWholeNumber": "は整数でなければなりません。",
  "rule.hasDecimalPoint": "は小数点なしの整数でなければなりません。",
  "rule.notANumber": "は数値でなければなりません。",
  "rule.notText": "は文字列でなければなりません。",
  "rule.notABoolean": "はオンまたはオフでなければなりません。",
  "rule.notAnOption": "は選択できる項目にありません。",
  "rule.tooSmall": "は {limit} より小さくできません。",
  "rule.tooLarge": "は {limit} より大きくできません。",
  "rule.tooLong": "は {limit} 文字以内にしてください。",
  "rule.tooLargeForGame": "はこのゲームには大きすぎます。",
  "rule.notPresent": "はこのセーブファイルに存在しません。",

  // --- Added with the confirmation flow ---
  "confirm.title": "本当によろしいですか？",
  "confirm.intro": "{count} 件の値が、このプラグインが安全とみなす範囲を超えています。",
  "confirm.intro_other": "{count} 件の値が、このプラグインが安全とみなす範囲を超えています。",
  "confirm.risk": "ゲームの動作がおかしくなったり、セーブデータが拒否されたり、クラッシュしたりする可能性があります。それでも先にバックアップは作成されるので、元に戻せます。",
  "confirm.suggestedMax": "推奨の上限 {limit}",
  "confirm.suggestedMin": "推奨の下限 {limit}",
  "confirm.go": "それでも保存",
  "changes.title": "変更される内容",
  "banner.gameRunning": "{game} が現在起動しています。終了時に変更が上書きされる可能性があります。",
  "banner.cloud": "Steam Cloud がこのフォルダーを同期しています。クラウド側が新しい場合、編集が取り消されることがあります。",
  "banner.staleReload": "このセーブデータはディスク上で変更されました。再読み込みしてください。",
  "banner.reload": "再読み込み",
  "editor.discardConfirm": "保存していない変更があります。保存せずに移動しますか？",
  "error.saveChangedOnDisk": "開いたあとにゲームがこのセーブデータを変更しました。保存する前に再読み込みしてください。そうしないと、ゲームが書き込んだ内容を取り消してしまいます。",
  "error.constraint": "{message}",
  "error.needsConfirmation": "一部の値が安全な範囲を超えています。",

  // --- Added with quick actions, list editing and themes ---
  "presets.title": "クイック操作",
  "presets.help": "どれも普通の変更のまとまりです。バックアップも確認も同じように行われます。",
  "editor.allSections": "すべて",
  "editor.searchFields": "項目を検索…",
  "editor.revert": "この変更を元に戻す",
  "combo.search": "入力して絞り込み…",
  "combo.noMatch": "「{query}」に一致するものはありません。",
  "combo.more": "他に {count} 件 — 入力を続けてください。",
  "list.add": "追加",
  "list.remove": "削除",
  "list.saveFirst": "先に変更を保存してください。",
  "list.added": "追加しました。先にバックアップを作成しています。",
  "list.removed": "削除しました。先にバックアップを作成しています。",
  "recovery.title": "ゲーム自身のバックアップ",
  "recovery.help": "このゲームが自分のために作り、セーブデータの隣に置いているコピーです。読み取るだけで、書き込むことはありません。復元すると、置き換えられる側もバックアップされます。",
  "recovery.compare": "比較",
  "recovery.use": "これを復元",
  "recovery.identical": "現在のセーブデータと同じです。",
  "recovery.andMore": "…ほか {count} 件。",
  "settings.appearance": "外観",
  "settings.theme": "テーマ",
  "settings.themeSystem": "システムに合わせる",
  "settings.themeLight": "ライト",
  "settings.themeDark": "ダーク",
  "error.listNotEditable": "「{list}」では行の追加や削除はできません。",
  "error.listFull": "「{list}」には {max} 件までしか入りません。",
  "error.listAtMinimum": "「{list}」は少なくとも {min} 件を残す必要があります。",
};
