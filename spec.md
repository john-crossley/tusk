# 🦣 Tusk – daily CLI todos with zero friction

## Philosophy
- No args = operate on **today**.
- Human-readable **JSON per day**.
- Short, memorable verbs. Works great aliased as `t`.

---

## Global usage

    tusk [GLOBAL OPTIONS] <SUBCOMMAND> [ARGS]
    tusk [GLOBAL OPTIONS]            # shorthand for `tusk ls` (today)
    tusk "Buy milk #home !low"       # shorthand for `tusk add ...`

### Global options
- --date YYYY-MM-DD · Act on a specific date (default: today)
- --range START..END · Act across dates (e.g. 2025-09-01..2025-09-30)
- --data-dir PATH · Override data directory
- --no-color / --color · Force color off/on
- --json · Machine-readable output for the current command
- -q, --quiet · Suppress non-essential output
- -v, --verbose · Extra diagnostics
- --version · Print version
- -h, --help · Help

---

## Quick syntax for item text
Tokens parsed anywhere you pass TEXT:

- Priority: !high | !med | !low
- Tags: #tag (repeatable)
- Estimate: @15m | @1h | @1h30m
- Due time/date: >14:00 (today’s time) or >2025-09-13T14:00
- Notes: --notes "..." (explicit flag)

---

## Core subcommands

### add
Add a new item (creates today’s file if missing).

    tusk add "Call Dave about invoices #work @15m !high >16:00"
    tusk "Draft weekly update #work !med"

Options:
- --notes "Free-form notes"
- --priority high|med|low

Output (human):

    Added #5: Call Dave about invoices  [!high  @15m  >16:00  #work]

---

### ls (default)
List items for a date (or range with filters).

    tusk ls
    tusk ls --open
    tusk ls --done --range 2025-09-01..2025-09-07 --tag work

Filters & formatting:
- --open / --done
- --tag TAG (repeatable)
- --priority p (one of high|med|low)
- --sort created|due|priority|status|index (default: index)
- --reverse
- --limit N
- --count (prints counts only)

Example:

    #  Status  Text                                   Meta
    1  [ ]     Draft weekly update                    !med  #work
    2  [x]     Pay council tax                        !high >09:00 #home
    3  [ ]     Call Dave about invoices               !high @15m >16:00 #work

---

### done / undone / toggle

    tusk done 3
    tusk toggle 1
    tusk undone --id 7f3a9c...

---

### rm

    tusk rm 2
    tusk rm 1 3 4
    tusk rm --id 7f3a9c... 91de2f...

---

### edit

    tusk edit 3 "Call Dave about invoices #work !med @20m >17:00"
    tusk edit 3 --notes "Ask about PO number"
    tusk edit --id 7f3a9c... --priority high

---

### mv

    tusk mv 5 --index 1
    tusk mv 3 --up
    tusk mv 4 --down
    tusk mv 6 --before 2
    tusk mv 6 --after 2

---

### show

    tusk show 2
    tusk show --id 7f3a9c...

---

### export

    tusk export --md                 # today to stdout
    tusk export --md --date 2025-09-12 > 2025-09-12.md
    tusk export --json --range 2025-09-01..2025-09-30 > sep.json

Options:
- --md | --json (one required)
- --out PATH (default stdout)
- Same filters as ls apply (--open, --tag, etc.)

---

### grep

    tusk grep "invoice" --range 2025-09-01..2025-09-30 --tag work
    tusk grep --regex "pay.*tax" --open

Options:
- --regex
- --case-sensitive

---

### review

    tusk review --week
    tusk review --month 2025-09
    tusk review --range 2025-09-01..2025-09-30 --tag work

Outputs: counts (open/done), top tags, total/avg estimates, completion rate, overdue items.

---

### web (optional v1)

    tusk web --port 8787

Options:
- --read-write (enable edits when you’re ready)
- --host 127.0.0.1

---

### init

    tusk init

---

## IDs vs indices
- Stable ID: UUID stored in JSON. Never changes. Use with --id.
- Index: Number shown by ls for current view/sort/filter. Fast but ephemeral.

---

## Data layout (defaults)
- macOS: ~/Library/Application Support/tusk/2025/09/2025-09-13.json
- Linux: ~/.local/share/tusk/2025/09/2025-09-13.json
- Windows: %APPDATA%\tusk\2025\09\2025-09-13.json

Config (optional):
- macOS: ~/Library/Preferences/tusk/config.json
- Linux: ~/.config/tusk/config.json
- Windows: %APPDATA%\tusk\config.json

### config.json example

    {
      "data_dir": "/custom/path",
      "default_priority": "med",
      "time_zone": "Europe/London",
      "color": true,
      "short_add": true,
      "default_sort": "index",
      "editor": "nano"
    }

---

## Daily file format (YYYY-MM-DD.json)

    {
      "date": "2025-09-13",
      "tz": "Europe/London",
      "items": [
        {
          "id": "7f3a9c12-3b5b-4bcd-9c6d-2af2a1c9e001",
          "text": "Call Dave about invoices",
          "created_at": "2025-09-13T08:12:45Z",
          "done_at": null,
          "priority": "high",
          "tags": ["work"],
          "estimate": 900,
          "due": "2025-09-13T16:00:00+01:00",
          "notes": "",
          "index": 3
        }
      ]
    }

---

## Exit codes
- 0 success
- 1 general error (parse, IO)
- 2 invalid arguments
- 3 item not found
- 4 conflicting options
- 5 server error (for web)

---

## Environment variables
- TUSK_DATA_DIR
- TUSK_CONFIG
- NO_COLOR
- PAGER

---

## Examples

**Morning:**

    t "Draft weekly update #work !med"
    t add "Call Dave about invoices #work @15m !high >16:00"
    t ls --open --sort priority

**During the day:**

    t done 1
    t mv 3 --index 1
    t edit 2 "Draft weekly update for stakeholders #work !med"

**Evening:**

    t export --md > today.md
    t review --day

**Search last month:**

    t grep "invoice" --range 2025-08-01..2025-08-31 --tag work --json