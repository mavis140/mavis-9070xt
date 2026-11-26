# VIctory on an RX 9070 XT with 32 GB of RAM

A fork of [avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX),
itself a fork of [VIctory](https://github.com/Marvisak/VIctory), a compatibility
layer that runs an Xbox Series title natively on Windows from files the user
supplies. It follows AvatarManz's `main` up to `2de9980` (2026-11-26) and adds
what one PC with an AMD RDNA 4 card and half the RAM needed to run it.

## The goal

**A stable 20 fps in the open world on a 16 GB card and 32 GB of RAM, then 30**,
with textures that are there when you look at them. AvatarManz's fork is measured
on an RX 7900 XTX with 64 GB; this one is where the same code meets a card whose
driver stopped on the title's shaders and a PC that runs out of memory first.

**This repository contains no game files, code or assets.** You need your own
copy of the game. It is not affiliated with or endorsed by the game's developer
or publisher.

## Where it stands (2026-11-26)

Tested on one machine only: Windows 11, AMD Ryzen 7 9800X3D, 32 GB DDR5, Radeon
RX 9070 XT 16 GB (driver 32.0.32029.2010), the title on an NVMe drive, Series X
profile, 2560x1440 output, ray tracing off. Nothing here has been run on an
NVIDIA or Intel card. The numbers are from play sessions in the open world, by
day and by night, not from a fixed scene: frame times are the medians of the
layer's own frame reports, grouped by how many draws a frame had.

| | |
|---|---|
| Open world, 3,000 to 6,000 draws a frame | about 40 ms a frame (25 fps); 80 ms in the session before, without the renderer patches |
| Same, 6,000 to 9,000 draws | about 53 ms (19 fps); 99 ms before |
| Over 9,000 draws (dense streets, the first minutes after loading) | 115 to 145 ms (7 to 9 fps) with draws left out: the first item under "What is known to be wrong" |
| A session | about 17 fps over nine minutes in the world, 30 in light scenes (`scripts\play-9070xt.ps1`) |
| The driver's shader compiler | no longer stops: 0 pipelines refused in two sessions (48 minutes), where 38 compute programs were refused in every frame before |
| The first minutes of a run | at the session's frame rate after `-Warm` (4,336 pipelines made ahead in 310 s); 3 to 7 fps for six minutes without it |
| Memory | 31 to 33 GB of private memory (a 32 GB PC pages, and the frame rate jumps with it), 11.4 GB of the card inside the 11.5 to 11.9 GB Windows gives the title, about 10 GB shared with it. AvatarManz measures 45 GB of RAM for this version on his 64 GB PC |
| Controls | pad, or keyboard and mouse (`VICTORY_MOUSE=1`: the mouse is the right stick and the triggers) |

Two days of sessions on this PC went from an exit after three minutes (the
driver) to 14 fps, and to about 20 in ordinary scenes.

### What this fork adds

| Change | What it does |
|---|---|
| Record counts from the host's table (`VICTORY_SLOT_COUNTS`, on) | tightfall's fix for the RDNA 4 compiler crash ([pull request 55](https://github.com/avatarmanz/VIctory-7900XTX/pull/55)), carried onto this tree: the size query translated shaders made is what the driver's compiler died on |
| A guard around pipeline creation | A pipeline that still takes the driver's compiler down is written to `cache\pipelines\crashed.txt` and refused from then on, instead of ending the run (after Belobug's fork) |
| `VICTORY_TEXTURE_MAX=<pixels>` or `auto` | Host copies of streamed block-compressed textures start at a smaller level as the card fills (1024, 512, then 256 pixels a side): about 1 GB of the card left free |
| Placed views' filler words | The title leaves stack contents in the padding of its view descriptions, and every one was a new entry in a table that was full after 27 minutes; from then on the layer's own memory grew by about 200 MB a minute. They are zeroed (`VICTORY_VIEW_FILLER=1`: as before) |
| `VICTORY_MOUSE=1` | Mouse look, mouse buttons as the triggers and a third-person PC key layout for the stand-in pad ([pull request 153](https://github.com/avatarmanz/VIctory-7900XTX/pull/153)) |
| `scripts\play-9070xt.ps1` | The settings measured above in one place, and `-Warm` to make the pipelines ahead |

The launcher turns on AvatarManz's five renderer patches below, the title's own
1440p output (`VICTORY_OUTPUT`), `VICTORY_TEXTURE_MAX=auto`, and two settings
for textures that appear late: slots waiting for their memory are looked at
every half second (`VICTORY_TEXTURE_LOOK_MS=500`; at 12 fps the looks came 8 to
20 s apart), and a texture whose memory is away keeps what it showed for 15 s
(`VICTORY_TEXTURE_AWAY_MS=15000`; made null 3 times a minute instead of 23).

### AvatarManz's renderer patches (off by default; `play-9070xt.ps1` turns them on)

| Switch | What it does |
|---|---|
| `VICTORY_PORT_CONSTANTS=1` | The title's 174 MB constants heap is read by the card where the title writes it: no uploads, no fills |
| `VICTORY_PORT_DESCRIPTORS=1` | The title's descriptor-table stores are patched (byte-checked) to report the slots they write, instead of a scan of ~600,000 slots a frame |
| `VICTORY_PORT_CBV=1` | Translated shaders read constant buffers in place as raw bytes (~22,000 fills a frame to none; -4.7 ms) |
| `VICTORY_PORT_GUARD=1` | The write guards' lock is shared by faults and refreshes, with a claim per 64 KiB block; the submission's planning runs beside it (title threads' waits 29 ms to ~2 ms a frame; -2 ms) |
| `VICTORY_PORT_JOURNAL=1` | The heaps' records are asked for at the title's submit, and the journal's notes are split into 64 parts with locks of their own (the submit thread's waits ~5 ms + 1 ms to ~0.3 ms; -1 to -3 ms) |

### Other switches

| Switch | What it does |
|---|---|
| `--console series-s` | The Series S profile: 1080p output, about 5 ms faster, a softer picture |
| `VICTORY_OUTPUT=1920x1080` | The title renders to that output size itself (Series X profile; about 16 ms less card time) |
| `VICTORY_FRAME_INTERVAL_MS=16.667` | Lets the title go above 30 fps (menus and videos reach 60; the game's clock stays right) |
| `VICTORY_AMBIENT_SETTLE` | How fast light settles after walking indoors (on by default, `250,3`; `0` turns it off) |
| `VICTORY_TEXTURE_STORE=1` | Converted textures kept between runs (about 8 ms a frame for now: off) |

What is known to be wrong:

- **The first one or two minutes in the world, and dense streets, glitch**:
  textures and whole objects come and go, at 4 to 7 fps. The host's descriptor
  slots for tables made per draw run out: up to 3,300 of 37,000 tables a frame
  are refused, about 1,900 indirect draws a frame are left out, and the search
  for free slots takes over a second of thread time a frame. Seen with the
  renderer patches on, not in the two sessions before them. Next to fix.
- Textures of what just came into view appear late: of those that waited for
  the title's memory, half took up to 8 s.
- 32 GB is not enough yet: the PC pages, and long sessions get slower. The last
  part of the filler fix (one more padding word) is unit-tested but has not been
  through a play session.
- One pipeline of 4,337 still ends the driver's compiler when made ahead; the
  guard refuses it.
- By day, ground and walls in the sun are far too bright, white in places.
  Light in shadow is AvatarManz's stand-in; ray tracing is off.
- Now and then a character's or a vehicle's mesh is drawn as large sheets of
  polygons fanning out from it for a moment (seen while getting into a car and
  at a crash). The cause is not known yet.
- With the mouse, the Caps Lock driving layout and F8 (frees the pointer) have
  not been tried in play.
- What AvatarManz lists still holds: trees, signs and far objects flicker in
  the distance in places, and light is too bright or too dark for a while after
  walking indoors.

## How to run it

You need Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with shader
model 6.6 and mesh shaders, Rust (the toolchain is pinned), the MSVC build
tools, and your own dump of the title, on an SSD.

```powershell
git clone https://github.com/mavis140/VIctory-7900XTX
cd VIctory-7900XTX
copy victory.example.toml victory.toml      # then set your paths, outside this folder
cargo build --locked --profile diag -p victory --features diagnostics
scripts\play-9070xt.ps1 -Exe <path of your copy's executable>
```

After the first session, and after every new build or driver, once:

```powershell
scripts\play-9070xt.ps1 -Exe <path of your copy's executable> -Warm   # five minutes, no game
```

- `-NoPorts` leaves the renderer patches out; try it when objects go missing
  or the device is removed. `scripts\play.ps1` is AvatarManz's launcher with
  none of this fork's settings (`-Safe`, `-RayTracing`).
- `-Output 1920x1080` takes less of the card.
- If an earlier build with the crash guard left `cache\pipelines\crashed.txt`
  in the layer's cache folder, delete it once: the pipelines in it stay refused.
- Close browsers and anything else that draws with the card. On this PC they
  held 2.6 to 3 GB of it, and Windows gives the title that much less.

On a hybrid-graphics PC, the runtime and offline pipeline checks prefer a
high-performance adapter with Shader Model 6.6. The runtime logs its adapter
name, PCI vendor and device IDs, and dedicated VRAM. Set
`VICTORY_D3D12_ADAPTER` to a zero-based DXGI index or part of the adapter's
name to select it explicitly. With the launcher, pass
`-More @{VICTORY_D3D12_ADAPTER='NVIDIA'}`.

Before the first run, ahead of time (it can be stopped and goes on later):

```powershell
target\diag\victory.exe shaders <the shader archives>   # every shader translated, minutes
```

Commands, debugging switches and upstream's own description:
[docs/README-fxceless.md](docs/README-fxceless.md). Every `VICTORY_...` switch is
described in a comment where it is read.

## What AvatarManz's fork changes over VIctory

In short; the reasons and the measurements are in [docs/findings.md](docs/findings.md).

- **List calls are written down, not carried out, on the title's threads**
  (`VICTORY_THIN`): eight threads of ours translate them.
- **Submissions are prepared by a thread of ours and joined**
  (`VICTORY_ASYNC_SUBMIT`, `VICTORY_JOIN_PARTS`), with what can be spread over
  threads spread: refreshes, compares (AVX2), descriptor records, landings.
- **What shaders wrote comes back beside the write guards**, by the dwords that
  changed, on a thread of its own (`VICTORY_RELAY`, `VICTORY_LAND`).
- **Buffer fills that need bytes still on the card are done on the card**
  (`VICTORY_CARD_STEPS`) instead of waiting for it.
- **The title's writes are caught more cheaply**: one view opened at a fault,
  blocks that rest, places in the title's code that look before they write.
- **Textures go up on a copy queue of their own** and a slot shows one only after
  its upload ran, which also ended a device loss that came every 15 minutes or so.
- **Boats float**: textures that shaders write and the title's CPU reads go back
  to title memory (from fxceless and Geckomacabre, on by default here).
- **Frame record and replay** with per-pass times and shader variants compared
  turn by turn ([docs/replay.md](docs/replay.md)).

## Credits

This is a tree of other people's work with more on top. The history is kept, and
commits taken from elsewhere name the commit and its author.

- **[avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX)**:
  the fork this one is built on, and nearly all of what makes it fast: the
  renderer patches, the threads beside the title's, the launcher and the
  measurements this fork's method is copied from.
- **tightfall**: the record counts from the host's table, which ended the RDNA 4
  compiler crash (pull request 55 there; its commit here keeps its author).
- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its
  contributors: the loader, the runtime services, the Direct3D 12 back end and
  the shader recompiler.
- **[fxceless/victory-fork](https://github.com/fxceless/victory-fork)**: the fork
  this one started from, and the people whose pull requests it carries.
- **[DexNom/VIctory](https://github.com/DexNom/VIctory)**: ray tracing fixes,
  cube gradients, sparse textures.
- **[Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)**:
  the ambient light idea, pipeline workers, and work by romainhedouin; in this
  fork also the guard around the driver's compiler and the idea of the texture
  cap (commits 0d2f6a6, 95e935d, ebd990d).
- **[FQY55kai/ZJOU-VICTORY](https://github.com/FQY55kai/ZJOU-VICTORY)**.
- **[myokex/victory-amd](https://github.com/myokex/victory-amd)**: the guard over
  every view of a page (with Spexivus, joroyt and tightfall), and Joshua Odom's
  reads into title memory.
- **Geckomacabre** and **cxy0072019-sketch**: the texture read-back.
- **[XCL](https://github.com/ivall/xcl)** (GPL-2.0-only) was read for how the same
  title behaves under another layer; no code of it is here.
- This fork's changes were made with [Claude Code](https://claude.com/claude-code).

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE); the one file that is not
covered is named in [docs/asset-policy.md](docs/asset-policy.md).
