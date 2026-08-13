"""Rebuild tools.json from an installed copy of Feed The Pit.

Run this on a machine that owns the game. It writes only identifiers and the
game's own display names into tools.json, never any artwork, which is why this
step is offline and manual rather than something the app does at runtime.

    python extract-tools.py "C:/Program Files (x86)/Steam/steamapps/common/Feed The Pit/v77-2-Steam/ftp.pck"

Why parse the archive at all: the game never writes a tool id into the save
until the player picks that tool up, so a save file alone reveals almost
nothing. The ids have to come from the game's own resources.

TODO: nobody has confirmed that the save's `tools[n].id` field wants the bare
`id_name` this produces. It is the obvious reading of the Tool resource, but
the only proof is a save the game wrote after a tool was picked up. If someone
checks, delete this note.

Two things about this .pck are worth knowing before you edit this script.

Its directory is encrypted (pack format 2, flags=1), so the usual approach of
reading the file table and then the file you want is not available here. The
*file data* is plaintext, though, so this scans for resource blobs directly.
That is a weaker technique and it is why the result is checked in rather than
read live: a scan can silently drift when the game updates.

Tools come in two shapes. A plain Tool declares `id_name` first; a Tool_Card
declares `card_title` first, so "the first string after the resource path"
finds the wrong value for every card. Hence the snake_case test below. An id
is always `[a-z][a-z0-9_]*` and a title never is.
"""

import collections
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).parent
# Enough to cover a resource; far short of the distance to the next one.
BLOB = 40_000


def extract(pck: pathlib.Path) -> dict:
    data = pck.read_bytes()
    starts = [m.start() for m in re.finditer(rb"RSRC", data)]
    tools = {}

    for i, start in enumerate(starts):
        nxt = starts[i + 1] if i + 1 < len(starts) else len(data)
        blob = data[start : min(nxt, start + BLOB)]
        # Both shapes carry these; other resources carry neither.
        if b"id_name" not in blob or b"max_durability" not in blob:
            continue

        strings = [
            m.group().decode("latin-1")
            for m in re.finditer(rb"[\x20-\x7e]{2,90}", blob)
        ]
        source = next(
            (
                s
                for s in strings
                if s.startswith("res://scripts/tools/") and s.endswith(".tres")
            ),
            None,
        )
        if source is None:
            continue

        tail = strings[strings.index(source) + 1 :]
        for j, value in enumerate(tail):
            if not re.fullmatch(r"[a-z][a-z0-9_]{1,40}", value):
                continue
            if value in ("res", "tres", "png"):
                continue
            # display_name always directly follows id_name in both shapes.
            label = tail[j + 1] if j + 1 < len(tail) else value
            tools[value] = {"label": label, "card": value.startswith("card_")}
            break

    return collections.OrderedDict(
        sorted(tools.items(), key=lambda kv: (kv[1]["card"], kv[0]))
    )


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 2

    pck = pathlib.Path(sys.argv[1])
    if not pck.is_file():
        print(f"not a file: {pck}")
        return 1

    tools = extract(pck)
    if not tools:
        print("found nothing; has the resource layout changed?")
        return 1

    doc = {
        "_note": (
            "Extracted from the player's own installed copy of Feed The Pit by "
            "tools/extract-tools.py. Identifiers only - no game assets."
        ),
        "tools": tools,
    }
    out = HERE / "tools.json"
    out.write_text(json.dumps(doc, ensure_ascii=False, indent=2), encoding="utf-8")

    cards = sum(1 for t in tools.values() if t["card"])
    print(f"{out}: {len(tools)} tools ({len(tools) - cards} plain, {cards} cards)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
