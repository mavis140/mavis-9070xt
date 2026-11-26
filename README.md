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
| Open world, up to 3,000 draws a frame | about 42 ms a frame (24 fps) |
| 3,000 to 6,000 draws | about 48 ms (21 fps); 80 ms before the renderer patches |
| 6,000 to 9,000 draws | about 64 ms (16 fps); 99 ms before |
| Over 9,000 draws (dense streets) | about 73 ms (14 fps); 115 to 145 ms before, with draws left out |
| A session | 18.9 fps as the median of 27 minutes in the world, 26 of 319 five-second reports over 75 ms a frame and 8 over 100 (`scripts\play-9070xt.ps1`) |
| The driver's shader compiler | no longer stops: 0 pipelines refused in any session since, where 38 compute programs were refused in every frame before |
| The first minutes of a run | at the session's frame rate after `-Warm` (about 5,000 pipelines made ahead in five minutes, seconds after a rebuild that changes few); 3 to 7 fps for six minutes without it |
| Memory | 27 to 29 GB of private memory, level over the last 15 minutes of that session (31 to 33 GB and rising before kripotker's memory series and the fix below); 10.7 GB of the card, about 7.5 GB shared with it. AvatarManz measures 45 GB of RAM for this version on his 64 GB PC |
| Controls | pad, or keyboard and mouse (`VICTORY_MOUSE=1`: the mouse is the right stick and the triggers) |

Two days of sessions on this PC went from an exit after three minutes (the
driver) to 14 fps, and to about 19 with far fewer dips.

### What this fork adds

| Change | What it does |
|---|---|
| Record counts from the host's table (`VICTORY_SLOT_COUNTS`, on) | tightfall's fix for the RDNA 4 compiler crash ([pull request 55](https://github.com/avatarmanz/VIctory-7900XTX/pull/55)), carried onto this tree: the size query translated shaders made is what the driver's compiler died on |
| A guard around pipeline creation | A pipeline that still takes the driver's compiler down is written to `cache\pipelines\crashed.txt` and refused from then on, instead of ending the run (after Belobug's fork) |
| `VICTORY_TEXTURE_MAX=<pixels>` or `auto` | Host copies of streamed block-compressed textures start at a smaller level as the card fills (1024, 512, then 256 pixels a side): about 1 GB of the card left free |
| Placed views' filler words | The title leaves stack contents in the padding of its view descriptions, and every one was a new entry in a table that was full after 27 minutes; from then on the layer's own memory grew by about 200 MB a minute. They are zeroed (`VICTORY_VIEW_FILLER=1`: as before) |
| `VICTORY_MOUSE=1` | Mouse look, mouse buttons as the triggers and a third-person PC key layout for the stand-in pad ([pull request 153](https://github.com/avatarmanz/VIctory-7900XTX/pull/153)) |
| A view heap of 2^23 descriptors, searched by words | The slots for tables made per draw ran out in dense streets (up to 3,384 of 37,000 tables a frame refused, each a draw left out, and over a second of thread time a frame searching). The title's 1,245,184 slots left them 851,967 in a heap of 2^21 views; they are 2,097,152 now, and the search steps over whole words (`VICTORY_VIEW_HEAP_BITS=21`: the old heap) |
| Idle textures stay while the card has room (`VICTORY_PRESSURE_CARD_PERCENT`, 70) | kripotker's rule lets idle texture copies go after 5 s while the PC is short of memory. Here the PC always is, and the copies are on the card: the rule now holds only from 70 % of the card's budget on |
| `VICTORY_PANEL=2` | A wide panel in colours: frame rate, a graph of the last frames' times, the card and the processor with their load, memory (F3 shows and hides it) |
| `scripts\play-9070xt.ps1` | The settings measured above in one place, `-Warm` to make the pipelines ahead, `-Safe` for the slower way without the mesh fault below |

It also carries kripotker's memory series
([pull request 206](https://github.com/avatarmanz/VIctory-7900XTX/pull/206), all
of its commits but the one for the shapes, which this fork has its own of): with
it the private memory fell from about 31.5 GB to 25 GB at the same point of a
session and the pages read in from 11,400 a second to under 5,000.

The launcher turns on AvatarManz's renderer patches 2, 5, 6 and 7 below, the
title's own 1440p output (`VICTORY_OUTPUT`), `VICTORY_TEXTURE_MAX=auto`, the
panel, and two settings for textures that appear late: slots waiting for their
memory are looked at every half second (`VICTORY_TEXTURE_LOOK_MS=500`; at 12 fps
the looks came 8 to 20 s apart), and a texture whose memory is away keeps what
it showed for 15 s (`VICTORY_TEXTURE_AWAY_MS=15000`; made null 3 times a minute
instead of 23). It leaves two things of the tree's own launcher off, as measured
here: renderer patch 1 (with `VICTORY_PORT_CONSTANTS=1` the sky flickered between
dark and light; without it, not), and title memory read in place everywhere
(`VICTORY_PORT_ALL`, `VICTORY_INPLACE`, 64 KiB windows: 9,296 small regions
opened and 5,467 forgotten in three minutes, the frame rate swinging between 2
and 20 fps).

### AvatarManz's renderer patches (off by default; `play-9070xt.ps1` turns 2, 5, 6 and 7 on)

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

- **A character's or a vehicle's mesh is at times drawn as large sheets of
  polygons, with black or white patches on clothes**, most when the player
  takes control after loading, and again now and then. (Things in the world
  also flicker in and out; whether that is the same fault is not known.) It
  comes with `VICTORY_THIN=1` (list calls written down and
  translated by threads of ours a moment later), found by turning one thing off
  at a time in seven short runs: the 2^23 heap, renderer patches 1, 6 and 7,
  `VICTORY_UPLOAD_AHEAD`, `VICTORY_JOIN_PARTS`, `VICTORY_UNDER_WIDE` and
  `VICTORY_LAND_ECHO` are not it; `-Safe` is clean, at 13 fps against 17 to 19.
  Being worked on.
- **The frame rate still jumps**: 8 % of the five-second reports are over 75 ms
  a frame. A timeline of 68 frames says where: the title's `CopyPageMappings`
  is the largest cause (14 % of all frame time, 193 and 209 ms in the two worst
  frames), waiting behind the layer's own protection changes. More mapping
  threads (`VICTORY_MAP_THREADS=auto`) only moved the wait.
- Textures and plants of what just came into view appear late or come and go
  when the card is near its budget: with a browser open Windows gave the title
  11.9 GB of a 16 GB card, and 616,000 texture copies were let go and 323,000
  made again in 20 minutes.
- One pipeline of about 5,000 still ends the driver's compiler when made ahead;
  the guard refuses it.
- By day, ground and walls in the sun are far too bright, white in places.
  Light in shadow is AvatarManz's stand-in; ray tracing is off.
- A floor under the working set (`VICTORY_WORKING_SET_MB`) is not a way to keep
  the title in memory: at 20,480 the host device was removed at world load.
- With the mouse, the Caps Lock driving layout and F8 (frees the pointer) have
  not been tried in play.
- What AvatarManz lists still holds: far objects flicker in the distance in
  places, and light is too bright or too dark for a while after walking
  indoors.

## How to run it

You need Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with shader
model 6.6 and mesh shaders, Rust (the toolchain is pinned), the MSVC build
tools, and your own dump of the title, on an SSD.

```powershell
git clone https://github.com/mavis140/mavis-9070xt
cd mavis-9070xt
copy victory.example.toml victory.toml      # then set your paths, outside this folder
cargo build --locked --profile diag -p victory --features diagnostics
scripts\play-9070xt.ps1 -Exe <path of your copy's executable>
```

After the first session, and after every new build or driver, once:

```powershell
scripts\play-9070xt.ps1 -Exe <path of your copy's executable> -Warm   # five minutes, no game
```

- `-NoPorts` leaves the renderer patches out; try it when objects go missing
  or the device is removed. `-Safe` is the slower way without the mesh fault.
  `scripts\play.ps1` is the tree's own launcher with none of this fork's
  settings (`-Safe`, `-RayTracing`, `-Console series-s`).
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
