//! Diagnostics: the run's numbers on the picture (`VICTORY_PANEL=1`, or `=2`
//! for the wide panel in colours with a graph of the frames' times; off by
//! default; F3 shows and hides it while our window has the focus): white
//! text on a black box in the upper left corner of what is shown, so that a
//! screenshot carries them, with what it costs itself on the submitting
//! thread. Made again twice a second from the counters of
//! the frame-rate line, memory and textures; a frame itself only copies the
//! box out of a buffer in the card's memory. Copied onto the swap chain's
//! buffer, behind the title's picture: saved frames and the frame recorder
//! see the title's own.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F3};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use super::{Host, HostList, transition};

/// How often the numbers change.
const EVERY: Duration = Duration::from_millis(500);

/// A character's dots, and with the space beside and below it.
const GLYPH: (u32, u32) = (5, 7);
const CELL: (u32, u32) = (6, 9);
/// Characters in a line, lines, and the box's rim in dots.
const COLUMNS: u32 = 40;
const LINES: u32 = 9;
const RIM: u32 = 3;

/// The texts are put into one of these in turn: the card reads the others,
/// at most half a second old.
const UPLOADS: usize = 4;

fn asked() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| crate::switch::on("VICTORY_PANEL"))
}

/// Whether it is shown now.
fn shown() -> &'static AtomicBool {
    static SHOWN: OnceLock<AtomicBool> = OnceLock::new();
    SHOWN.get_or_init(|| AtomicBool::new(asked()))
}

/// F3 was pressed since the last call, with one of our windows in front.
fn pressed() -> bool {
    static DOWN: AtomicBool = AtomicBool::new(false);
    // SAFETY: a plain query.
    let down = unsafe { GetAsyncKeyState(i32::from(VK_F3)) } < 0;
    if DOWN.swap(down, Ordering::Relaxed) || !down {
        return false;
    }
    let mut process = 0;
    // SAFETY: plain queries; room for the answer.
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut process) };
    process == std::process::id()
}

/// Seven rows of five dots, the leftmost the highest bit.
#[rustfmt::skip]
const fn glyph(c: u8) -> [u8; 7] {
    match c {
        b'0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        b'1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        b'2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        b'3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        b'4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        b'5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        b'6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        b'7' => [0b11111, 0b00001, 0b00010, 0b00010, 0b00100, 0b00100, 0b00100],
        b'8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        b'9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        b'A' => [0b00100, 0b01010, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001],
        b'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        b'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111],
        b'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        b'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        b'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        b'G' => [0b01111, 0b10000, 0b10000, 0b10011, 0b10001, 0b10001, 0b01111],
        b'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        b'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        b'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        b'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        b'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        b'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        b'N' => [0b10001, 0b11001, 0b11001, 0b10101, 0b10011, 0b10011, 0b10001],
        b'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        b'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        b'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        b'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        b'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        b'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        b'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        b'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        b'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        b'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        b'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        b'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        b'.' => [0, 0, 0, 0, 0, 0b01100, 0b01100],
        b'/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        b'-' => [0, 0, 0, 0b11111, 0, 0, 0],
        b'|' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        b'%' => [0b11001, 0b11010, 0b00010, 0b00100, 0b01000, 0b01011, 0b10011],
        b' ' => [0; 7],
        _ => [0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111],
    }
}

/// The box's size in pixels, a dot being `scale` of them each way.
const fn size(scale: u32) -> (u32, u32) {
    ((COLUMNS * CELL.0 + 2 * RIM) * scale, (LINES * CELL.1 + 2 * RIM) * scale)
}

/// The size in pixels of the box that is shown: the wide panel's or the plain one's.
fn box_size(scale: u32) -> (u32, u32) {
    if wide() { (WIDE.0 * scale, WIDE.1 * scale) } else { size(scale) }
}

/// Pixels to a dot for a picture that high: a 720-line screenshot of it
/// shows one and a third.
fn scale(height: u32) -> u32 {
    (height / 540).max(1)
}

/// Whether a dot of the picture of `lines` is white.
fn lit(lines: &[String], x: u32, y: u32) -> bool {
    let (Some(x), Some(y)) = (x.checked_sub(RIM), y.checked_sub(RIM)) else {
        return false;
    };
    if x % CELL.0 >= GLYPH.0 || y % CELL.1 >= GLYPH.1 || x / CELL.0 >= COLUMNS {
        return false;
    }
    let Some(&c) = lines.get((y / CELL.1) as usize).and_then(|l| l.as_bytes().get((x / CELL.0) as usize)) else {
        return false;
    };
    (glyph(c.to_ascii_uppercase())[(y % CELL.1) as usize] >> (GLYPH.0 - 1 - x % CELL.0)) & 1 != 0
}

/// The bytes of a black and a white pixel of a swap chain's format.
fn colours(format: DXGI_FORMAT) -> Option<[&'static [u8]; 2]> {
    Some(match format {
        DXGI_FORMAT_R8G8B8A8_UNORM | DXGI_FORMAT_B8G8R8A8_UNORM => [&[0, 0, 0, 0xFF], &[0xFF; 4]],
        DXGI_FORMAT_R10G10B10A2_UNORM => [&[0, 0, 0, 0xC0], &[0xFF; 4]],
        // Halves of 0 and 1.
        DXGI_FORMAT_R16G16B16A16_FLOAT => [
            &[0, 0, 0, 0, 0, 0, 0, 0x3C],
            &[0, 0x3C, 0, 0x3C, 0, 0x3C, 0, 0x3C],
        ],
        _ => return None,
    })
}

/// The box as rows of `pitch` bytes.
fn painted(lines: &[String], scale: u32, colours: [&[u8]; 2], pitch: usize) -> Vec<u8> {
    let (width, height) = size(scale);
    let bytes = colours[0].len();
    let mut out = vec![0; pitch * height as usize];
    for y in 0..height / scale {
        let top = pitch * (y * scale) as usize;
        for x in 0..width / scale {
            let colour = colours[usize::from(lit(lines, x, y))];
            for i in 0..scale {
                let at = top + (x * scale + i) as usize * bytes;
                out[at..at + bytes].copy_from_slice(colour);
            }
        }
        for i in 1..scale as usize {
            out.copy_within(top..top + pitch, top + i * pitch);
        }
    }
    out
}

/// `VICTORY_PANEL=2`: the wide panel, in colours: the frame rate large, the
/// output's size, draws and the card's time, a graph of the last frames'
/// times, the card and the processor with their load (a bar a core), and
/// memory: the card's, the process's, the PC's.
fn wide() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_PANEL").is_some_and(|v| v == "2"))
}

/// The wide panel's box in dots, and the frames its graph holds (a dot each).
const WIDE: (u32, u32) = (COLUMNS * CELL.0 + 2 * RIM, 136);
const GRAPH: usize = (COLUMNS * CELL.0) as usize;
/// The graph's and the cores' bars' heights in dots, and the frame time (ms)
/// that fills the graph.
const GRAPH_HIGH: u32 = 24;
const BARS_HIGH: u32 = 9;
const GRAPH_MS: f32 = 100.0;

/// The wide panel's colours (red, green, blue, 0 to 1), by number.
const BACK: u8 = 0;
const WHITE: u8 = 1;
const GREY: u8 = 2;
const YELLOW: u8 = 3;
const GREEN: u8 = 4;
const RED: u8 = 5;
const BLUE: u8 = 6;
const CYAN: u8 = 7;
const DIM: u8 = 8;
const PALETTE: [[f32; 3]; 9] = [
    [0.02, 0.03, 0.07],
    [0.95, 0.95, 0.95],
    [0.55, 0.58, 0.62],
    [0.98, 0.85, 0.20],
    [0.35, 0.85, 0.35],
    [0.95, 0.30, 0.25],
    [0.35, 0.60, 0.98],
    [0.30, 0.80, 0.90],
    [0.12, 0.15, 0.22],
];

/// A half float of a value from 0 to 1.
fn half(v: f32) -> u16 {
    let bits = v.clamp(0.0, 1.0).to_bits();
    let exponent = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    if exponent <= 0 { 0 } else { (exponent as u16) << 10 | ((bits >> 13) & 0x3ff) as u16 }
}

/// A pixel of colour `rgb` in a swap chain's format.
fn pixel(format: DXGI_FORMAT, [r, g, b]: [f32; 3]) -> Option<Vec<u8>> {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    let ten = |v: f32| (v.clamp(0.0, 1.0) * 1023.0 + 0.5) as u32;
    Some(match format {
        DXGI_FORMAT_R8G8B8A8_UNORM => vec![byte(r), byte(g), byte(b), 0xFF],
        DXGI_FORMAT_B8G8R8A8_UNORM => vec![byte(b), byte(g), byte(r), 0xFF],
        DXGI_FORMAT_R10G10B10A2_UNORM => (ten(r) | ten(g) << 10 | ten(b) << 20 | 3 << 30).to_le_bytes().to_vec(),
        DXGI_FORMAT_R16G16B16A16_FLOAT => [half(r), half(g), half(b), half(1.0)].iter().flat_map(|h| h.to_le_bytes()).collect(),
        _ => return None,
    })
}

/// A picture in dots, each a colour's number.
struct Dots {
    width: u32,
    height: u32,
    dots: Vec<u8>,
}

impl Dots {
    fn new((width, height): (u32, u32)) -> Dots {
        Dots { width, height, dots: vec![BACK; (width * height) as usize] }
    }

    fn rect(&mut self, (x, y): (u32, u32), (w, h): (u32, u32), colour: u8) {
        for row in y..(y + h).min(self.height) {
            let from = (row * self.width + x.min(self.width)) as usize;
            let to = (row * self.width + (x + w).min(self.width)) as usize;
            self.dots[from..to].fill(colour);
        }
    }

    /// `text` from `(x, y)` on, a glyph's dot `times` dots each way; where
    /// it ends.
    fn text(&mut self, (x, y): (u32, u32), times: u32, colour: u8, text: &str) -> u32 {
        let mut at = x;
        for c in text.bytes() {
            let rows = glyph(c.to_ascii_uppercase());
            for (gy, row) in rows.iter().enumerate() {
                for gx in 0..GLYPH.0 {
                    if row >> (GLYPH.0 - 1 - gx) & 1 != 0 {
                        self.rect((at + gx * times, y + gy as u32 * times), (times, times), colour);
                    }
                }
            }
            at += CELL.0 * times;
        }
        at
    }

    /// Pieces of text one after the other on the line at `y`.
    fn row(&mut self, y: u32, pieces: &[(u8, &str)]) {
        let mut x = RIM;
        for &(colour, text) in pieces {
            x = self.text((x, y), 1, colour, text);
        }
    }

    /// The picture as rows of `pitch` bytes, a dot `scale` pixels each way.
    fn painted(&self, scale: u32, palette: &[Vec<u8>], pitch: usize) -> Vec<u8> {
        let bytes = palette[0].len();
        let mut out = vec![0; pitch * (self.height * scale) as usize];
        for y in 0..self.height {
            let top = pitch * (y * scale) as usize;
            for x in 0..self.width {
                let colour = &palette[usize::from(self.dots[(y * self.width + x) as usize]) % palette.len()];
                for i in 0..scale {
                    let at = top + (x * scale + i) as usize * bytes;
                    out[at..at + bytes].copy_from_slice(colour);
                }
            }
            for i in 1..scale as usize {
                out.copy_within(top..top + pitch, top + i * pitch);
            }
        }
        out
    }
}

/// What the wide panel shows.
#[derive(Default)]
struct Shown {
    fps: f64,
    ms: f64,
    worst: f64,
    low: f64,
    output: (u32, u32),
    draws: f64,
    dropped: f64,
    card_ms: f64,
    card_load: f64,
    submit_ms: f64,
    back_ms: f64,
    record_ms: f64,
    card: String,
    /// The card's memory in use and its budget, and the PC's memory it
    /// shares in use (GB).
    video: [f64; 3],
    processor: String,
    load: f64,
    cores: Vec<f32>,
    /// The process's private memory, the PC's memory in use and all of it,
    /// and what the PC has committed (GB).
    memory: [f64; 4],
    frames: Vec<f32>,
}

/// Green, yellow or red for a share of 1 that is fine below `fine` and bad
/// from `bad` on.
fn grade(value: f64, fine: f64, bad: f64) -> u8 {
    if value < fine {
        GREEN
    } else if value < bad {
        YELLOW
    } else {
        RED
    }
}

/// The wide panel's picture.
fn wide_picture(s: &Shown) -> Dots {
    let mut d = Dots::new(WIDE);
    let line = CELL.1;
    let mut y = RIM;
    // The frame rate, twice the size; the exact figure small beside it.
    let rate = grade(1.0 / s.fps.max(0.001), 1.0 / 25.0, 1.0 / 15.0);
    d.text((RIM, y), 2, if rate == GREEN { YELLOW } else { rate }, &format!("{:.0} FPS", s.fps));
    let exact = format!("{:.1} FPS", s.fps);
    d.text((WIDE.0 - RIM - exact.len() as u32 * CELL.0, y), 1, GREY, &exact);
    y += 2 * line;
    d.row(y, &[
        (WHITE, &format!("{}X{}", s.output.0, s.output.1)),
        (GREY, " | "),
        (WHITE, &format!("{:.0} DRAWS", s.draws)),
        (GREY, " | GPU "),
        (WHITE, &format!("{:.1}MS", s.card_ms)),
    ]);
    y += line;
    d.row(y, &[
        (GREY, "FRAME "),
        (WHITE, &format!("{:.1}", s.ms)),
        (GREY, " | SUB "),
        (WHITE, &format!("{:.1}", s.submit_ms)),
        (GREY, " | 1% "),
        (WHITE, &format!("{:.0}", s.low)),
        (GREY, " | MAX "),
        (WHITE, &format!("{:.0}", s.worst)),
    ]);
    y += line + 1;
    // The last frames' times, the newest at the right.
    d.rect((RIM, y), (GRAPH as u32, GRAPH_HIGH + 2), DIM);
    let first = GRAPH.saturating_sub(s.frames.len());
    for (i, &ms) in s.frames.iter().rev().take(GRAPH).rev().enumerate() {
        let high = ((ms / GRAPH_MS * GRAPH_HIGH as f32) as u32).clamp(1, GRAPH_HIGH);
        let colour = grade(f64::from(ms), 50.0, 100.0);
        d.rect((RIM + (first + i) as u32, y + 1 + GRAPH_HIGH - high), (1, high), colour);
    }
    y += GRAPH_HIGH + 4;
    let percent = |share: f64| format!(" {:.0}%", share * 100.0);
    d.row(y, &[(BLUE, "GPU  "), (WHITE, &s.card), (grade(s.card_load, 0.85, 0.97), &percent(s.card_load))]);
    y += line;
    d.row(y, &[
        (BLUE, "VRAM "),
        (WHITE, &format!("{:.1}/{:.1} GB", s.video[0], s.video[1])),
        (GREY, " | SHR "),
        (WHITE, &format!("{:.1} GB", s.video[2])),
    ]);
    y += line;
    d.row(y, &[
        (GREY, "BACK "),
        (WHITE, &format!("{:.1}", s.back_ms)),
        (GREY, " | REC "),
        (WHITE, &format!("{:.0}", s.record_ms)),
        (GREY, " | DROP "),
        (if s.dropped >= 1.0 { YELLOW } else { WHITE }, &format!("{:.0}", s.dropped)),
    ]);
    y += line;
    d.row(y, &[(BLUE, "CPU  "), (WHITE, &s.processor), (grade(s.load, 0.75, 0.92), &percent(s.load))]);
    y += line;
    // A bar a core.
    if !s.cores.is_empty() {
        let slot = (GRAPH as u32 / s.cores.len() as u32).max(2);
        for (i, &load) in s.cores.iter().enumerate() {
            let x = RIM + i as u32 * slot;
            d.rect((x, y), (slot - 1, BARS_HIGH), DIM);
            let high = ((load.clamp(0.0, 1.0) * BARS_HIGH as f32).round() as u32).clamp(1, BARS_HIGH);
            d.rect((x, y + BARS_HIGH - high), (slot - 1, high), CYAN);
        }
    }
    y += BARS_HIGH + 3;
    d.row(y, &[(BLUE, "VICTORY "), (GREY, "RAM "), (WHITE, &format!("{:.1} GB", s.memory[0]))]);
    y += line;
    d.row(y, &[
        (BLUE, "RAM  "),
        (grade(s.memory[1] / s.memory[2].max(0.001), 0.85, 0.95), &format!("{:.1}/{:.1} GB", s.memory[1], s.memory[2])),
        (GREY, " | CMT "),
        (WHITE, &format!("{:.1}", s.memory[3])),
    ]);
    d
}

/// The processor's name as it gives it, without what says nothing
/// ("8-Core Processor"), in capitals.
fn processor_name() -> String {
    #[cfg(target_arch = "x86_64")]
    {
        let mut bytes = Vec::new();
        for leaf in 0x8000_0002u32..=0x8000_0004 {
            #[allow(unused_unsafe)]
            // SAFETY: every x86-64 processor has these leaves.
            let r = unsafe { std::arch::x86_64::__cpuid(leaf) };
            for word in [r.eax, r.ebx, r.ecx, r.edx] {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        let name = String::from_utf8_lossy(&bytes).replace('\0', " ").to_ascii_uppercase();
        let words: Vec<&str> = name
            .split_whitespace()
            .filter(|w| !["PROCESSOR", "CPU", "@"].contains(w) && !w.ends_with("-CORE") && !w.ends_with("GHZ"))
            .collect();
        words.join(" ").replace("(R)", "").replace("(TM)", "")
    }
    #[cfg(not(target_arch = "x86_64"))]
    String::new()
}

/// The time each logical processor has idled and run since the system
/// started (100 ns units): for the load between two looks.
fn processor_times() -> Vec<(i64, i64)> {
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Times {
        idle: i64,
        kernel: i64,
        user: i64,
        rest: [i64; 2],
        interrupts: u64,
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtQuerySystemInformation(class: u32, information: *mut Times, length: u32, returned: *mut u32) -> i32;
    }
    const PROCESSOR_PERFORMANCE: u32 = 8;
    let mut times = [Times::default(); 256];
    let mut returned = 0u32;
    // SAFETY: a plain query into our own array, which says how much it wrote.
    let status = unsafe {
        NtQuerySystemInformation(PROCESSOR_PERFORMANCE, times.as_mut_ptr(), size_of_val(&times) as u32, &mut returned)
    };
    if status < 0 {
        return Vec::new();
    }
    // (The kernel's time holds the idle time.)
    times[..returned as usize / size_of::<Times>()].iter().map(|t| (t.idle, t.kernel + t.user)).collect()
}

/// The counters read for it. They grow, and go back to 0 with the frame-rate
/// line (right behind a look of ours).
#[derive(Clone, Copy, Default)]
struct Counters([u64; COUNTERS]);

const COUNTERS: usize = 11;

impl Counters {
    fn now() -> Counters {
        use super::{CARD_CLOCK_US, COPY_BACK, PREPARING};
        use crate::copystat::{C, peek};
        let get = |a: &std::sync::atomic::AtomicU64| a.load(Ordering::Relaxed);
        Counters([
            super::worked(),
            get(&CARD_CLOCK_US[0]),
            get(&CARD_CLOCK_US[1]),
            get(&CARD_CLOCK_US[2]),
            get(&PREPARING[3]),
            get(&COPY_BACK[0]),
            get(&COPY_BACK[1]),
            get(&crate::shadow::CARD_STATS[0]),
            crate::draw::dropped_now(),
            peek(C::RecordNs),
            crate::draw::drawn(),
        ])
    }

    /// What came since `before`, per frame.
    fn since(self, before: Counters, frames: f64) -> [f64; COUNTERS] {
        let mut out = [0.0; COUNTERS];
        for (i, out) in out.iter_mut().enumerate() {
            let (was, is) = (before.0[i], self.0[i]);
            *out = if is >= was { is - was } else { is } as f64 / frames;
        }
        out
    }
}

/// The text. `c`: [`Counters`] per frame, `ms` a frame's time, `worst` the
/// longest frame's, `cost` what this takes of a frame, in microseconds.
fn text(c: &[f64; COUNTERS], ms: f64, worst: f64, cost: f64, host: &Host) -> Vec<String> {
    let gb = |bytes: u64| bytes as f64 / (1u64 << 30) as f64;
    let (working, private) = crate::copystat::process_memory();
    let [budget, used, _, shared] = host.video_memory().unwrap_or_default();
    let idle = (c[1] / 1e3).min(ms);
    vec![
        format!("FPS {:.1}  FRAME {ms:.1} MS  LONGEST {worst:.0}", 1e3 / ms.max(0.001)),
        format!("DRAWS {:.0}  DROPPED {:.0}", c[10], c[8]),
        format!("SUBMIT BUSY {:.1}  WAITS FOR GPU {:.1}", c[0] / 1e3, c[3] / 1e3),
        format!("GPU BUSY {:.1}  IDLE BEHIND US {:.1}", ms - idle, c[2] / 1e3),
        format!("RECORDING {:.0} MS  COPY BACK {:.1}", c[9] / 1e6, c[5] / 1e3),
        format!("MB UP {:.0}  TEXTURES {:.0}  BACK {:.0}", c[7] / 1e6, c[4] / 1e6, c[6] / 1e6),
        format!("RAM {:.1} GB  IN MEMORY {:.1}", gb(private), gb(working)),
        format!("VRAM {:.1}/{:.1} GB  SHARED {:.1}", gb(used), gb(budget), gb(shared)),
        format!("THIS PANEL {:.0} US A FRAME", cost),
    ]
}

/// The buffers of a box that size and format.
struct Buffers {
    format: DXGI_FORMAT,
    scale: u32,
    pitch: u32,
    uploads: Vec<(ID3D12Resource, *mut u8)>,
    card: ID3D12Resource,
    /// The upload written last.
    turn: usize,
    /// It is not in the card's buffer yet.
    fresh: bool,
}

// SAFETY: free-threaded resources, and memory only written under the presenter's lock.
unsafe impl Send for Buffers {}

impl Buffers {
    fn new(host: &Host, format: DXGI_FORMAT, scale: u32) -> Option<Buffers> {
        let (width, height) = box_size(scale);
        let pitch = (width * colours(format)?[0].len() as u32).next_multiple_of(256);
        let bytes = u64::from(pitch) * u64::from(height);
        let uploads = (0..UPLOADS).map(|_| host.upload_buffer(bytes)).collect::<Option<Vec<_>>>()?;
        let heap = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            ..Default::default()
        };
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: bytes,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            ..Default::default()
        };
        let mut card: Option<ID3D12Resource> = None;
        // SAFETY: plain creation of a buffer.
        unsafe {
            host.device
                .CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &desc, D3D12_RESOURCE_STATE_COMMON, None, &mut card)
                .ok()?;
        }
        Some(Buffers { format, scale, pitch, uploads, card: card?, turn: 0, fresh: false })
    }

    fn write(&mut self, lines: &[String], dots: Option<&Dots>) {
        let Some(colours) = colours(self.format) else { return };
        let picture = match dots {
            Some(dots) => {
                let Some(palette) = PALETTE.iter().map(|&rgb| pixel(self.format, rgb)).collect::<Option<Vec<_>>>() else {
                    return;
                };
                dots.painted(self.scale, &palette, self.pitch as usize)
            }
            None => painted(lines, self.scale, colours, self.pitch as usize),
        };
        self.turn = (self.turn + 1) % UPLOADS;
        // SAFETY: a mapped upload buffer of the picture's size, which the card read a second or more ago.
        unsafe { std::ptr::copy_nonoverlapping(picture.as_ptr(), self.uploads[self.turn].1, picture.len()) };
        self.fresh = true;
    }
}

/// The panel of a presenter.
pub(super) struct Panel {
    since: Instant,
    frames: u32,
    last: Instant,
    worst: f64,
    /// What this took since the last text.
    cost: Duration,
    before: Counters,
    lines: Vec<String>,
    changed: bool,
    buffers: Option<Buffers>,
    failed: bool,
    /// The wide panel (`wide`): its picture, the last frames' times, the
    /// processors' times at the last look, and the output's size.
    dots: Option<Dots>,
    history: std::collections::VecDeque<f32>,
    processors: Vec<(i64, i64)>,
    output: (u32, u32),
}

impl Panel {
    pub(super) fn new() -> Panel {
        let now = Instant::now();
        Panel {
            since: now,
            frames: 0,
            last: now,
            worst: 0.0,
            cost: Duration::ZERO,
            before: Counters::default(),
            lines: Vec::new(),
            changed: false,
            buffers: None,
            failed: false,
            dots: None,
            history: std::collections::VecDeque::new(),
            processors: Vec::new(),
            output: (0, 0),
        }
    }

    /// A frame was presented; `line`: the counters go back to 0 right behind
    /// this call. (Not shown it asks for the key's state and no more.)
    pub(super) fn frame(&mut self, line: bool) {
        if pressed() {
            shown().fetch_xor(true, Ordering::Relaxed);
        }
        if !shown().load(Ordering::Relaxed) {
            self.frames = 0;
            return;
        }
        let now = Instant::now();
        if self.frames == 0 && now - self.last > 2 * EVERY {
            // (Shown from here on.)
            (self.since, self.last, self.worst, self.before) = (now, now, 0.0, Counters::now());
            self.frames = 0;
        }
        self.frames += 1;
        let took = (now - self.last).as_secs_f64() * 1e3;
        self.worst = self.worst.max(took);
        if wide() && self.frames > 1 {
            if self.history.len() == GRAPH {
                self.history.pop_front();
            }
            self.history.push_back(took as f32);
        }
        self.last = now;
        let elapsed = now - self.since;
        if elapsed < EVERY && !(line && elapsed * 2 >= EVERY) {
            return;
        }
        let Some(host) = super::host() else { return };
        let frames = f64::from(self.frames);
        let counters = Counters::now();
        let c = counters.since(self.before, frames);
        self.before = counters;
        let cost = self.cost.as_secs_f64() * 1e6 / frames;
        let ms = elapsed.as_secs_f64() * 1e3 / frames;
        self.lines = text(&c, ms, self.worst, cost, host);
        if wide() {
            self.dots = Some(wide_picture(&self.figures(&c, ms, host)));
        }
        self.changed = true;
        (self.since, self.frames, self.worst) = (now, 0, 0.0);
        self.cost = now.elapsed();
    }

    /// What the wide panel shows now. `c`: [`Counters`] per frame, `ms` a
    /// frame's time.
    fn figures(&mut self, c: &[f64; COUNTERS], ms: f64, host: &Host) -> Shown {
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        let gb = |bytes: u64| bytes as f64 / (1u64 << 30) as f64;
        let idle = (c[1] / 1e3).min(ms);
        // The slowest hundredth of the frames kept, and the slowest.
        let mut sorted: Vec<f32> = self.history.iter().copied().collect();
        sorted.sort_by(f32::total_cmp);
        let slow = sorted.get(sorted.len().saturating_sub(1 + sorted.len() / 100)).map_or(ms, |&ms| f64::from(ms));
        let worst = sorted.last().map_or(ms, |&ms| f64::from(ms));
        // Each processor's load since the last look.
        let now = processor_times();
        let cores: Vec<f32> = now
            .iter()
            .zip(&self.processors)
            .map(|(&(idle, all), &(was_idle, was_all))| {
                (1.0 - (idle - was_idle) as f32 / (all - was_all).max(1) as f32).clamp(0.0, 1.0)
            })
            .collect();
        self.processors = now;
        let load = if cores.is_empty() { 0.0 } else { f64::from(cores.iter().sum::<f32>()) / cores.len() as f64 };
        let [budget, used, _, shared] = host.video_memory().unwrap_or_default();
        let (_, private) = crate::copystat::process_memory();
        // SAFETY: plain data the call fills in; its size is set.
        let mut pc: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        pc.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
        // SAFETY: a sized structure of ours.
        unsafe { GlobalMemoryStatusEx(&mut pc) };
        static CARD: OnceLock<String> = OnceLock::new();
        let card = CARD.get_or_init(|| {
            // SAFETY: a plain query of a live adapter.
            let desc = host.adapter.as_ref().and_then(|a| unsafe { a.GetDesc1() }.ok());
            desc.map(|d| String::from_utf16_lossy(&d.Description).trim_end_matches('\0').to_ascii_uppercase())
                .unwrap_or_default()
        });
        static PROCESSOR: OnceLock<String> = OnceLock::new();
        Shown {
            fps: 1e3 / ms.max(0.001),
            ms,
            worst,
            low: 1e3 / slow.max(0.001),
            output: self.output,
            draws: c[10],
            dropped: c[8],
            card_ms: ms - idle,
            card_load: ((ms - idle) / ms.max(0.001)).clamp(0.0, 1.0),
            submit_ms: c[0] / 1e3,
            back_ms: c[5] / 1e3,
            record_ms: c[9] / 1e6,
            card: card.clone(),
            video: [gb(used), gb(budget), gb(shared)],
            processor: PROCESSOR.get_or_init(processor_name).clone(),
            load,
            cores,
            memory: [
                gb(private),
                gb(pc.ullTotalPhys.saturating_sub(pc.ullAvailPhys)),
                gb(pc.ullTotalPhys),
                gb(pc.ullTotalPageFile.saturating_sub(pc.ullAvailPageFile)),
            ],
            frames: self.history.iter().copied().collect(),
        }
    }

    /// Records the copy of the box onto `back`, a buffer of the swap chain of
    /// that size and format in the state to be copied into.
    pub(super) fn draw(
        &mut self,
        host: &Host,
        list: &HostList,
        back: &ID3D12Resource,
        (width, height): (u32, u32),
        format: DXGI_FORMAT,
    ) {
        if !shown().load(Ordering::Relaxed) || self.lines.is_empty() || self.failed {
            return;
        }
        let began = Instant::now();
        self.output = (width, height);
        // Not the title's: the frame recorder leaves it out.
        let _ours = crate::replay::record::unrecorded();
        let scale = scale(height);
        if self.buffers.as_ref().is_none_or(|b| (b.format, b.scale) != (format, scale)) {
            // (What the card may still read stays: a picture's size changes once or twice in a run.)
            std::mem::forget(self.buffers.take());
            self.buffers = Buffers::new(host, format, scale);
            self.failed = self.buffers.is_none();
            self.changed = true;
        }
        let Some(buffers) = &mut self.buffers else { return };
        if std::mem::take(&mut self.changed) {
            buffers.write(&self.lines, self.dots.as_ref());
        }
        let (w, h) = box_size(scale);
        let list = &list.list;
        // SAFETY: an open list and live resources; the box lies within both.
        unsafe {
            if std::mem::take(&mut buffers.fresh) {
                list.CopyResource(&buffers.card, &buffers.uploads[buffers.turn].0);
                list.ResourceBarrier(&[transition(
                    &buffers.card,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                )]);
            }
            let from = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&buffers.card),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                        Offset: 0,
                        Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                            Format: format,
                            Width: w,
                            Height: h,
                            Depth: 1,
                            RowPitch: buffers.pitch,
                        },
                    },
                },
            };
            let to = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(back),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
            };
            let part = D3D12_BOX {
                left: 0,
                top: 0,
                front: 0,
                right: w.min(width),
                bottom: h.min(height),
                back: 1,
            };
            list.CopyTextureRegion(&to, 0, 0, 0, &from, Some(&part));
        }
        self.cost += began.elapsed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every character the lines use has its own picture, and no two look
    /// alike.
    #[test]
    fn characters_differ() {
        let used: Vec<u8> = (b'0'..=b'9').chain(b'A'..=b'Z').chain(*b"./- |%").collect();
        for (i, a) in used.iter().enumerate() {
            assert_ne!(glyph(*a), glyph(b'#'), "{} has a picture", *a as char);
            assert!(glyph(*a).iter().all(|row| row >> GLYPH.0 == 0));
            for b in &used[i + 1..] {
                assert_ne!(glyph(*a), glyph(*b), "{} {}", *a as char, *b as char);
            }
        }
    }

    #[test]
    fn a_box_is_painted() {
        let lines = vec!["I".to_owned(), " i".to_owned()];
        for scale in [1, 2, 4] {
            let (width, height) = size(scale);
            let pitch = (width * 4).next_multiple_of(256) as usize;
            let colours = colours(DXGI_FORMAT_R10G10B10A2_UNORM).unwrap();
            let picture = painted(&lines, scale, colours, pitch);
            assert_eq!(picture.len(), pitch * height as usize);
            let at = |x: u32, y: u32| &picture[y as usize * pitch + x as usize * 4..][..4];
            // The rim is black, the first letter's top row lit in its middle.
            assert_eq!(at(0, 0), colours[0]);
            assert_eq!(at((RIM + 2) * scale, RIM * scale), colours[1]);
            assert_eq!(at((RIM + 2) * scale + scale - 1, RIM * scale + scale - 1), colours[1]);
            assert_eq!(at(RIM * scale, RIM * scale), colours[0]);
            // Small letters are shown as capitals, a line and a cell on.
            assert_eq!(at((RIM + CELL.0 + 2) * scale, (RIM + CELL.1) * scale), colours[1]);
            // Nothing past a line's end.
            assert_eq!(at((RIM + CELL.0 * 2 + 2) * scale, (RIM + CELL.1) * scale), colours[0]);
        }
    }

    #[test]
    fn counters_that_went_back_to_nothing_count_from_there() {
        let (mut a, mut b) = (Counters::default(), Counters::default());
        (a.0[0], b.0[0]) = (10, 30);
        (a.0[1], b.0[1]) = (50, 20);
        let c = b.since(a, 10.0);
        assert_eq!((c[0], c[1]), (2.0, 2.0));
    }

    /// The wide panel: its size, every colour of its layout, a picture in
    /// each format a swap chain has, and the figures it reads of the PC.
    #[test]
    fn the_wide_panel_is_painted_in_colours() {
        let shown = Shown {
            fps: 23.3,
            ms: 42.9,
            worst: 60.0,
            low: 11.0,
            output: (2560, 1440),
            draws: 6001.0,
            card: "AMD RADEON RX 9070 XT".into(),
            card_load: 0.71,
            video: [9.8, 12.6, 9.3],
            processor: "AMD RYZEN 7 9800X3D".into(),
            load: 0.84,
            cores: vec![0.5; 16],
            memory: [27.0, 38.2, 61.4, 59.7],
            frames: vec![40.0, 120.0, 60.0],
            ..Default::default()
        };
        let d = wide_picture(&shown);
        assert_eq!((d.width, d.height), WIDE);
        for colour in [BACK, WHITE, GREY, YELLOW, GREEN, RED, BLUE, CYAN, DIM] {
            assert!(d.dots.contains(&colour), "colour {colour} is used");
        }
        // The rim stays the box's own colour, above and below.
        assert!(d.dots[..(WIDE.0 * RIM) as usize].iter().all(|&c| c == BACK));
        assert!(d.dots[(WIDE.0 * (WIDE.1 - RIM)) as usize..].iter().all(|&c| c == BACK));
        for format in [
            DXGI_FORMAT_R8G8B8A8_UNORM,
            DXGI_FORMAT_B8G8R8A8_UNORM,
            DXGI_FORMAT_R10G10B10A2_UNORM,
            DXGI_FORMAT_R16G16B16A16_FLOAT,
        ] {
            let palette: Vec<Vec<u8>> = PALETTE.iter().map(|&rgb| pixel(format, rgb).unwrap()).collect();
            let pitch = (WIDE.0 * 2 * palette[0].len() as u32).next_multiple_of(256) as usize;
            let picture = d.painted(2, &palette, pitch);
            assert_eq!(picture.len(), pitch * (WIDE.1 * 2) as usize);
            assert_eq!(&picture[..palette[0].len()], &palette[usize::from(BACK)][..]);
        }
        assert_eq!(pixel(DXGI_FORMAT_R8G8B8A8_UNORM, [1.0, 0.0, 0.5]).unwrap(), [255, 0, 128, 255]);
        assert_eq!(pixel(DXGI_FORMAT_B8G8R8A8_UNORM, [1.0, 0.0, 0.5]).unwrap(), [128, 0, 255, 255]);
        assert_eq!((half(1.0), half(0.5), half(0.0)), (0x3C00, 0x3800, 0));
        // Of the PC: a name, and times that grow for every processor.
        assert!(!processor_name().is_empty());
        let (before, after) = (processor_times(), processor_times());
        assert!(!before.is_empty() && before.len() == after.len());
        assert!(before.iter().zip(&after).all(|(b, a)| a.1 >= b.1 && a.0 >= b.0));
    }

    /// Writes the wide panel's picture as a PPM where `VICTORY_PANEL_PREVIEW`
    /// names a file (three pixels to a dot): to look at the layout without a
    /// run. Does nothing otherwise.
    #[test]
    fn the_wide_panel_can_be_looked_at() {
        let Some(path) = std::env::var_os("VICTORY_PANEL_PREVIEW") else { return };
        let frames: Vec<f32> = (0..GRAPH).map(|i| 38.0 + ((i * 37) % 23) as f32 + if i % 41 == 0 { 90.0 } else { 0.0 }).collect();
        let shown = Shown {
            fps: 23.3,
            ms: 42.9,
            worst: 131.0,
            low: 11.0,
            output: (2560, 1440),
            draws: 6001.0,
            dropped: 153.0,
            card_ms: 30.5,
            card_load: 0.71,
            submit_ms: 28.5,
            back_ms: 18.2,
            record_ms: 370.0,
            card: "AMD RADEON RX 9070 XT".into(),
            video: [9.8, 12.6, 9.3],
            processor: processor_name(),
            load: 0.84,
            cores: (0..16).map(|i| 0.3 + (i % 5) as f32 * 0.15).collect(),
            memory: [27.0, 28.2, 31.1, 59.7],
            frames,
        };
        let d = wide_picture(&shown);
        let palette: Vec<Vec<u8>> = PALETTE.iter().map(|&rgb| pixel(DXGI_FORMAT_R8G8B8A8_UNORM, rgb).unwrap()).collect();
        let pitch = (WIDE.0 * 3 * 4) as usize;
        let picture = d.painted(3, &palette, pitch);
        let mut out = format!("P6\n{} {}\n255\n", WIDE.0 * 3, WIDE.1 * 3).into_bytes();
        out.extend(picture.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]));
        std::fs::write(path, out).unwrap();
    }
}
