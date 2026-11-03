# Negative tests: what the app does when something goes wrong

Written 2026-11-03, before the first release build.

## The rule

1. An accepted scan is **never lost or corrupted**.
2. The app **never silently writes wrong numbers**: no foot means no measurement, and a strange shape carries a warning.
3. Every failure shows a **plain message that says what to do**.
4. Bonus: the app recovers by itself, without a restart.

Crashes count as failures of rule 3. Destructive tests never run on the real data folder: the automated ones use throwaway temp folders, and the manual ones use `D:\SmartFoot_Test` (set it in Settings, then set it back).

Run the automated tests from `src-tauri/`: `cargo test` (images: `cargo test --test negative_images -- --nocapture` prints the full table).

## A. Automated tests (run by the developer, all pass 2026-11-03)

| # | Scenario | Expected | Result | Found and fixed |
|---|---|---|---|---|
| A1 | Empty glass (lid open, grey) | no foot, no measurement | pass | showed "no foot" **and** "touches the edge" together; edge warning now only when a foot was found |
| A2 | All white (lid closed, clipped), all black, light gradient, random noise | no foot, no measurement | pass | |
| A3 | Dust speck 5 mm | no foot | pass | |
| A4 | Foot cut off at the left edge, forefoot cut off at the bottom edge | edge warning | pass | |
| A5 | Ankle-like 60 mm stretch entering at the bottom edge | accepted as a limb, no edge warning | pass | by design (limb rule, 80 mm) |
| A6 | Two feet on the glass | warning "two objects" | pass | only the largest was outlined and nothing told the operator; `TWO_OBJECTS` warning added |
| A7 | Ruler 15 x 300 mm, credit card | "odd shape" warning | pass | both were measured as feet (the ruler even got an arch index); `ODD_SHAPE` warning added (provisional thresholds) |
| A8 | Images from 1x1 to 40x40 px, 4x4000 and 4000x4 | clean error, no crash | pass | **crashed** (index out of bounds) on 1x1 and 3x3; images under 160 px now give `BAD_IMAGE` |
| A9 | Truncated PNG, text file named .png, empty file, missing file | clean error | pass | |
| A10 | Scanner driver hangs (script never returns) | killed after a limit, `TIMEOUT` message | pass | would have **frozen the app forever**; PowerShell now has time limits (list 20 s, scan 150 s) and big outputs cannot block |
| A11 | Device list: none, preferred unplugged, only Canon IJ route, two unrelated scanners, LiDE 110 only | sensible choice or "not found", never the LiDE 110 | pass | |
| A12 | Blank "LOCKED" scan (all rows identical) vs a real noisy scan | LOCKED only for the blank one | pass | |
| A13 | Accept fails while recording (session.json cannot be written) | scan goes back to `_pending`, Accept can be pressed again, nothing orphaned | pass | the scan used to stay in `scans/` with **no entry in session.json**, and the retry then failed |
| A14 | New-session form bypassing the UI: empty/`=1+1`/`-cmd`/too long operator, age 121 or huge, sex missing, ID `CON`/`nul`/`com1`/starts with `-`/over 32 characters/only symbols/`..\..\Windows`, note with control characters or 900 characters | refused with a code, or reduced to a safe value | pass | none of this was checked in the backend; reserved names would have failed on disk, `=...` would run as a formula in Excel |
| A15 | CSV cells starting with `= + - @`, commas, quotes, newlines, Unicode | cannot run as a formula, stays one cell | pass | text cells are now prefixed with `'` |
| A16 | Settings: empty or relative data folder, rounds 2 or 0, bad laptop code, blank backup folder | refused / blank = off | pass | an empty data folder was accepted |
| A17 | settings.json saved with a byte-order mark, or garbage | BOM tolerated; garbage copied to `settings.json.bad` before defaults replace it | pass | a BOM made the app **silently fall back to defaults and invent a new laptop code** (happened for real on 2026-11-03) |
| A18 | Damaged `session.json`, scans without a session.json, empty folders, `_` folders | damaged ones reported, ID never reused, healthy sessions unaffected | pass | damaged sessions used to be invisible; start screen now warns (never deletes) |
| A19 | Backup: target is a file / under a file / a missing drive / blocked by a file / session folder vanished / half-copied leftover | clean error, main data untouched, leftover overwritten | pass | |
| A20 | Backup target equal to, inside or around the data folder | refused | pass | |

## B. Hands-on checklist (needs a person, hardware and a clock). Status: not run yet

Use `D:\SmartFoot_Test` as the data folder. Tick **pass / fail** and write what you saw.

| # | Do this | Expect | Result |
|---|---|---|---|
| B1 | Unplug the scanner USB **during** a scan | error within about a minute (not an endless "Memindai…"), chip turns red, after replugging "Ulangi" works, no half scan left behind | |
| B2 | Close the app window **during** a scan, reopen | "Sesi belum selesai" banner; resume works (the scanner may say busy for up to ~30 s) | |
| B3 | Close the LOCK switch right after a scan, press the next scan | message about the LOCK switch, chip amber; after opening it the retry works | |
| B4 | Leave the app idle 15 min, close the LOCK, press "Mulai pindai" | the warm-up test catches the lock **before** the patient screen | |
| B5 | Lid closed (white lid on the glass), scan | "no foot found", **not** a LOCK message | |
| B6 | Scan: empty glass, a hand, a ruler, a card, two objects, a small object, a hand half off the edge | each gives the expected warning (see A1-A7); never a confident foot | |
| B7 | Unplug the backup drive, accept a scan | the scan is saved; the yellow warning appears on the start screen; replug and press "Cadangkan sekarang" copies it | |
| B8 | Task Manager: end `smartfoot-app.exe` right after pressing Accept | on reopen no half files; resume works; no duplicate or missing scan | |
| B9 | Open the app twice at once | no corruption; the second instance gets a clear message or a different ID | |
| B10 | Put the laptop to sleep with the app open, wake it | the chip recovers by itself within a few seconds | |
| B11 | Replug the scanner into another USB port | detected again, scanning works | |
| B12 | In Settings, type a data folder on a drive that does not exist, then a relative one (`data`) | a plain refusal each time | |
| B13 | In the new-session form: ID `CON`, operator `=1+1`, age `-5`, `45.5`, `200`, a very long note | each refused with a clear message (age) or tidied | |
| B14 | Fill the data drive (or use a tiny test drive) and scan | an error, no half-written session | |

## C. Known limits (not tested, or by design)

- **Null Driver**: Windows installing "Canon LiDE 300 Null Driver" cannot be forced. The code path exists and the detection query works, but it has not been seen end to end since the app was built.
- **Disk full** while a scan is saved, **power cut** during a write: only the parts we can inject (rename/write failures) are tested; B8 and B14 cover the real thing.
- **Odd-shape thresholds** (length 90-420 mm, length/width 1.6-5) are provisional and were tuned on synthetic shapes only. They only warn, never block. Re-tune on real feet in stage 2.
- **Two objects** warning fires when the second object is more than 25% of the foot's area.
- Only the **largest** object is outlined and measured, by design.
- **Seated Cavanagh cutoffs** are not validated; the app records numbers only, no labels.
