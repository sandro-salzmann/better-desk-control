# LINAK Desk BLE Protocol

Reverse-engineered notes on the LINAK desk BLE protocol: the characteristics,
command bytes, and exchanges observed on the wire between the official app and
the desk.

**Provenance.** Captured from the official Android app
(`com.linak.deskcontrol`) driving a "Desk 6420" — connect, UP, DOWN, then a
saved-preset move — following [ble-sniffing.md](ble-sniffing.md). The desk
speaks LINAK's standard DPG protocol. The `99fa…` UUIDs below are stable across
devices; the GATT handles in parentheses are from this one capture and are not.

## Characteristics

The desk exposes a vendor service `99fa0001-338a-1024-8a49-009c0215f78a` with
these characteristics:

| UUID (`99fa…-338a-…`) | Handle   | Role                                   |
| --------------------- | -------- | -------------------------------------- |
| `…0002`               | `0x0010` | direct UP/DOWN/STOP — write            |
| `…0021`               | `0x001a` | height + speed — notify                |
| `…0011`               | `0x0016` | info / preset channel (DPG) — write + notify |
| `…0031`               | `0x0022` | move-to-target / release — write       |

A fifth characteristic, `…0003` (handle `0x0012`, "error/feedback"), only
appears in the autonomous-move bug below.

On connect the app enables notifications by writing `01 00` to the CCCD of
`…0021` and `…0011`.

## Direct movement — `…0002`

Two-byte writes, **repeated every ~200 ms while the button is held**: the desk
has a dead-man timer and halts if it doesn't see the next packet within
~250 ms.

| Bytes   | Meaning                          |
| ------- | -------------------------------- |
| `47 00` | up                               |
| `46 00` | down                             |
| `ff 00` | stop                             |
| `01 80` | release the move-to-target latch |

On finger-up the app always sends `ff 00` (stop) followed by `01 80` (release).
So a press-and-hold looks like: the direction byte every ~200 ms, then stop +
release.

## Height + speed — `…0021`

4-byte notifications, fired ~20×/s while moving:

```
height_lo height_hi  speed_lo speed_hi
    u16 LE               i16 LE
```

- `height` is in raw counts; `cm = raw / 100 + 62` (the LINAK convention used by
  other open-source projects). e.g. raw 953 → 71.5 cm, raw 2344 → 85.4 cm. The
  `62` base may need shifting if a tape measurement disagrees.
- `speed` is signed: positive = up, negative = down, `0` = stopped.

A plain read of the characteristic returns the same 4 bytes — the app reads it
once to snapshot the current height before subscribing.

## Info / preset channel (DPG) — `…0011`

Request/response over one characteristic: write `7f <subcmd> 00`, and the desk
replies by notification with `01 <len> <payload>` (or `01 00` for a bodyless
ack).

**Presets** — one sub-command per UI slot; the value is a little-endian `u16`
raw height. The save form is `7f 8X 80 01 <lo> <hi>`.

| Sub-command | Slot | Read `7f 8X 00` →  | Value    |
| ----------- | ---- | ------------------ | -------- |
| `0x89`      | 1    | `01 07 01 61 15 …` | `0x1561` |
| `0x8a`      | 2    | `01 07 01 1b 15 …` | `0x151b` |
| `0x8b`      | 3    | `01 07 01 2c 04 …` | `0x042c` |

The saved-preset move in the capture targeted `0x151b`, i.e. slot 2.

Other `7f` exchanges were logged but their meaning is unconfirmed: `7f 80`
(capability handshake), `7f 86` (16-byte device id), `7f 87` (a counter that
increments each session — probably usage count), `7f 88` (an 11-byte settings
blob).

## Move-to-target & the autonomous-move bug — `…0031`

`…0031` takes a `u16` LE target in raw counts, repeated every ~200 ms until the
desk reports `speed == 0`; `01 80` is the no-target / release sentinel.

The desk's autonomous move is broken on this firmware — it halts after ~5 mm
and the app gives up:

```
TX …0031    = 2c 04      # target
RX …0021    = … speed     # desk starts moving
RX …0003    = 01 00 10    # error/feedback notify
TX …0002    = ff 00       # app reacts by stopping
TX …0031    = 01 80       # release
```

The `01 00 10` notify on `…0003` never appears during direct UP/DOWN moves —
only on autonomous moves via `…0031`. Direct UP/DOWN moves are unaffected.
