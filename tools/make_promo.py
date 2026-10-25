#!/usr/bin/env python3
"""Build the ReplicantWave Cyberpunk Darkwave Promo Video (1920x1080 @ 30fps).

Composes a high-energy, cinematic demo video showcasing the capabilities
of the ReplicantWave synthesizer with an authentic Cyberpunk Darkwave score:
- Dual Wavetable Oscillators with 8 morphing tables, unison, & phase warp
- Analog-modeled Zero-Delay Feedback (ZDF) multimode filter
- Integrated 64-step neon matrix sequencer with slide & accent
- Cyber FX rack: Hard distortion, chorus, ping-pong delay, shimmer reverb
- 84 host-automatable parameters & 38 factory presets
- Real-time reactive vector oscilloscope, spectrum analyzer & animated sequencer LEDs

Usage:
    cargo run --release --example promo_track   # 1. Render soundtrack
    python3 tools/make_promo.py --preview       # 2. Inspect QA frames
    python3 tools/make_promo.py                 # 3. Render final MP4
"""

from __future__ import annotations

import argparse
import math
import os
import subprocess
import sys
import wave

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WAV = os.path.join(ROOT, "target/promo/promo_track.wav")
SHOTS = os.path.join(ROOT, "target/promo/shots")
NEW_UI = os.path.join(ROOT, "assets/new_ui.jpeg")
HERO_GUI = os.path.join(ROOT, "assets/replicantwave_gui.png")
OUT = os.path.join(ROOT, "assets/replicantwave_promo.mp4")

W, H = 1920, 1080
FPS = 30
DUR = 66.0  # 32 bars @ 120 BPM = 64.0s + 2.0s tail
FRAMES = int(DUR * FPS)
BPM = 120.0
BEAT = 60.0 / BPM
BAR = BEAT * 4.0

# Cyberpunk Color Palette
CYAN = (0, 245, 255)
MAGENTA = (255, 16, 128)
NEON_PURPLE = (168, 70, 255)
AMBER = (255, 174, 36)
LIME = (50, 255, 110)
WHITE = (242, 246, 255)
DIM = (130, 122, 160)
DARK_BG = (5, 2, 11)
PANEL_BG = (10, 6, 20)

FONT_DIRS = [
    "/home/danimanx/.local/share/fonts/fonts",
    "/usr/share/fonts/TTF",
    "/usr/share/fonts/truetype/dejavu",
]


def load_font(names: list[str], size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    for directory in FONT_DIRS:
        for name in names:
            p = os.path.join(directory, name)
            if os.path.exists(p):
                try:
                    return ImageFont.truetype(p, size)
                except Exception:
                    pass
    return ImageFont.load_default()


F_TITLE = load_font(["Montserrat-Bold.ttf", "Hack-Bold.ttf"], 96)
F_HERO = load_font(["Montserrat-Bold.ttf", "Hack-Bold.ttf"], 48)
F_MED = load_font(["JetBrains-Mono-Nerd-Font-Complete.ttf", "Hack-Bold.ttf"], 24)
F_HUD = load_font(["JetBrains-Mono-Nerd-Font-Complete.ttf", "Hack-Bold.ttf"], 18)
F_SMALL = load_font(["JetBrains-Mono-Nerd-Font-Complete.ttf", "Hack-Bold.ttf"], 14)
F_TINY = load_font(["JetBrains-Mono-Nerd-Font-Complete.ttf", "Hack-Bold.ttf"], 11)

# ---------------------------------------------------------------------------
# Audio Analysis
# ---------------------------------------------------------------------------

def load_audio():
    with wave.open(WAV, "rb") as w:
        assert w.getnchannels() == 2
        sr = w.getframerate()
        raw = w.readframes(w.getnframes())
    data = np.frombuffer(raw, dtype=np.int16).astype(np.float32) / 32768.0
    stereo = data.reshape(-1, 2)
    mono = stereo.mean(axis=1)
    return sr, mono, stereo


def compute_spectrum(mono: np.ndarray, sr: int):
    bands = 56
    edges = np.geomspace(40.0, 13000.0, bands + 1)
    win = 2048
    hann = np.hanning(win)
    freqs = np.fft.rfftfreq(win, 1.0 / sr)

    idx = []
    for i in range(bands):
        m = (freqs >= edges[i]) & (freqs < edges[i + 1])
        idx.append(np.where(m)[0])

    spec = np.zeros((FRAMES, bands), dtype=np.float32)
    smooth = np.zeros(bands, dtype=np.float32)

    for f in range(FRAMES):
        centre = int((f / FPS) * sr)
        start = max(0, min(len(mono) - win - 1, centre - win // 2))
        seg = mono[start : start + win] * hann
        mag = np.abs(np.fft.rfft(seg))
        for i, ii in enumerate(idx):
            if len(ii) == 0:
                continue
            v = float(mag[ii].mean())
            v = 20.0 * math.log10(v + 1e-7)
            v = np.clip((v + 60.0) / 60.0, 0.0, 1.0)
            smooth[i] = max(v, smooth[i] * 0.74)
        spec[f] = smooth

    ref = np.percentile(spec, 96, axis=0)
    ref[ref < 0.08] = 1.0
    return np.clip(spec / ref, 0.0, 1.0)


def get_wave_slice(stereo: np.ndarray, sr: int, f: int, n_samples: int = 400):
    centre = int((f / FPS) * sr)
    start = max(0, min(len(stereo) - n_samples - 1, centre - n_samples // 2))
    return stereo[start : start + n_samples, 0]


def beat_energy(t: float) -> float:
    if t < 8.0:
        return 0.0
    phase = ((t - 8.0) % BEAT)
    return max(0.0, 1.0 - phase / 0.11)

# ---------------------------------------------------------------------------
# Visual Backgrounds & Overlays
# ---------------------------------------------------------------------------

def make_backdrop():
    img = Image.new("RGB", (W, H), DARK_BG)
    px = np.zeros((H, W, 3), dtype=np.float32)
    for y in range(H):
        u = y / (H - 1)
        top = np.array([20, 8, 40], dtype=np.float32)
        bot = np.array([4, 2, 10], dtype=np.float32)
        px[y, :, :] = top * (1 - u) + bot * u

    yy, xx = np.mgrid[0:H, 0:W].astype(np.float32)
    d = np.sqrt((xx - W / 2) ** 2 + (yy - 80) ** 2)
    bloom = np.clip(1.0 - d / 980.0, 0, 1) ** 3 * 52
    px[:, :, 0] += bloom
    px[:, :, 2] += bloom * 0.75
    return Image.fromarray(np.clip(px, 0, 255).astype(np.uint8))


def make_grid():
    layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    vp = (W // 2, int(H * 0.60))
    for i in range(-16, 17):
        x = W // 2 + i * 160
        d.line([vp, (x, H)], fill=(255, 16, 128, 28), width=2)
    for i in range(1, 14):
        y = int(vp[1] + (H - vp[1]) * (i / 13.0) ** 2.2)
        d.line([(0, y), (W, y)], fill=(255, 16, 128, 24), width=2)
    return layer


def make_scanlines():
    layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    for y in range(0, H, 4):
        d.line([(0, y), (W, y)], fill=(0, 0, 0, 28), width=1)
    return layer


def make_vignette():
    yy, xx = np.mgrid[0:H, 0:W].astype(np.float32)
    d = np.sqrt(((xx - W / 2) / (W / 2)) ** 2 + ((yy - H / 2) / (H / 2)) ** 2)
    alpha = np.clip((d - 0.65) / 0.70, 0, 1) ** 1.6 * 215
    layer = np.zeros((H, W, 4), dtype=np.uint8)
    layer[:, :, 3] = alpha.astype(np.uint8)
    return Image.fromarray(layer)


def make_grain():
    rng = np.random.default_rng(2077)
    return [
        Image.fromarray(
            (rng.normal(128, 24, (H // 2, W // 2, 1)).repeat(3, axis=2)).clip(0, 255).astype(np.uint8)
        ).resize((W, H), Image.NEAREST)
        for _ in range(6)
    ]


BACKDROP = make_backdrop()
GRID = make_grid()
SCANLINES = make_scanlines()
VIGNETTE = make_vignette()
GRAIN = make_grain()

# ---------------------------------------------------------------------------
# UI Assets Loading
# ---------------------------------------------------------------------------

_images: dict[str, Image.Image] = {}

def get_asset(name: str) -> Image.Image:
    if name not in _images:
        if name == "new_ui" and os.path.exists(NEW_UI):
            _images[name] = Image.open(NEW_UI).convert("RGBA")
        elif name == "hero_gui" and os.path.exists(HERO_GUI):
            _images[name] = Image.open(HERO_GUI).convert("RGBA")
        else:
            p = os.path.join(SHOTS, f"{name}.png")
            if os.path.exists(p):
                _images[name] = Image.open(p).convert("RGBA")
            elif os.path.exists(HERO_GUI):
                _images[name] = Image.open(HERO_GUI).convert("RGBA")
            else:
                _images[name] = Image.new("RGBA", (1300, 920), (20, 15, 30, 255))
    return _images[name]

# ---------------------------------------------------------------------------
# Drawing Helpers & Motion Graphics
# ---------------------------------------------------------------------------

def ease(x: float) -> float:
    x = min(1.0, max(0.0, x))
    return x * x * (3 - 2 * x)


def lerp(a: float, b: float, t: float) -> float:
    return a + (b - a) * t


def glitch_effect(img: Image.Image, amount: float, seed: int) -> Image.Image:
    if amount <= 0.02:
        return img
    arr = np.array(img)
    h, w = arr.shape[:2]
    rng = np.random.default_rng(seed)
    slices = int(2 + amount * 9)
    for _ in range(slices):
        y0 = int(rng.integers(0, h - 30))
        hgt = int(rng.integers(8, int(24 + amount * 80)))
        dx = int(rng.normal(0, 8 + amount * 40))
        arr[y0 : y0 + hgt] = np.roll(arr[y0 : y0 + hgt], dx, axis=1)

    shift = int(1 + amount * 8)
    out = arr.copy()
    out[:, :, 0] = np.roll(arr[:, :, 0], shift, axis=1)
    out[:, :, 2] = np.roll(arr[:, :, 2], -shift, axis=1)
    return Image.fromarray(out)


def text_glow(img: Image.Image, xy: tuple[int, int], text: str, fnt, fill, glow=None, radius=12, anchor="la"):
    glow = glow or fill
    layer = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    d.text(xy, text, font=fnt, fill=glow + (255,), anchor=anchor)
    layer = layer.filter(ImageFilter.GaussianBlur(radius))
    img.alpha_composite(layer)
    d2 = ImageDraw.Draw(img)
    d2.text(xy, text, font=fnt, fill=fill + (255,), anchor=anchor)


def draw_hud_brackets(d: ImageDraw.ImageDraw, box: tuple[int, int, int, int], color: tuple[int, int, int], length: int = 24, width: int = 2):
    x0, y0, x1, y1 = box
    for (x, y, dx, dy) in [(x0, y0, 1, 1), (x1, y0, -1, 1), (x0, y1, 1, -1), (x1, y1, -1, -1)]:
        d.line([(x, y), (x + dx * length, y)], fill=color + (255,), width=width)
        d.line([(x, y), (x, y + dy * length)], fill=color + (255,), width=width)


def draw_cyber_card(img: Image.Image, box: tuple[int, int, int, int], accent: tuple[int, int, int], fill_alpha: int = 210, cut: int = 14):
    x0, y0, x1, y1 = box
    d = ImageDraw.Draw(img)
    pts = [
        (x0 + cut, y0),
        (x1, y0),
        (x1, y1 - cut),
        (x1 - cut, y1),
        (x0, y1),
        (x0, y0 + cut),
    ]
    d.polygon(pts, fill=PANEL_BG + (fill_alpha,))
    d.line(pts + [pts[0]], fill=accent + (220,), width=2)
    draw_hud_brackets(d, (x0 - 4, y0 - 4, x1 + 4, y1 + 4), accent, length=18, width=2)


def draw_vector_oscilloscope(img: Image.Image, box: tuple[int, int, int, int], wave_samples: np.ndarray, color=CYAN, glow=MAGENTA):
    x0, y0, x1, y1 = box
    bw = x1 - x0
    bh = y1 - y0
    mid_y = y0 + bh // 2

    trace = Image.new("RGBA", img.size, (0, 0, 0, 0))
    td = ImageDraw.Draw(trace)

    n = len(wave_samples)
    if n < 2:
        return

    pts = []
    for i in range(n):
        x = x0 + int((i / (n - 1)) * bw)
        y = mid_y - int(wave_samples[i] * (bh * 0.42))
        pts.append((x, y))

    td.line(pts, fill=glow + (180,), width=4, joint="curve")
    trace = trace.filter(ImageFilter.GaussianBlur(5))
    img.alpha_composite(trace)

    d = ImageDraw.Draw(img)
    d.line(pts, fill=color + (255,), width=2, joint="curve")


def draw_filter_response(img: Image.Image, box: tuple[int, int, int, int], cutoff_hz: float, reso: float):
    x0, y0, x1, y1 = box
    bw = x1 - x0
    bh = y1 - y0
    base_y = y1 - 10

    d = ImageDraw.Draw(img)
    pts = []
    cutoff_x = x0 + int(bw * (math.log10(cutoff_hz / 20.0) / math.log10(1000.0)))
    cutoff_x = min(max(x0 + 10, cutoff_x), x1 - 10)

    for i in range(bw):
        x = x0 + i
        f_ratio = (x - x0) / bw
        freq = 20.0 * (1000.0 ** f_ratio)
        ratio = freq / cutoff_hz
        mag = 1.0 / math.sqrt(1.0 + (ratio ** 8))
        if 0.7 < ratio < 1.4:
            mag += reso * 1.8 * math.exp(-((ratio - 1.0) ** 2) / 0.08)
        y = base_y - int(min(1.0, mag) * (bh - 20))
        pts.append((x, y))

    d.line(pts, fill=AMBER + (220,), width=2, joint="curve")
    peak_y = base_y - int(min(1.0, (1.0 + reso * 1.8)) * (bh - 20))
    d.ellipse([cutoff_x - 4, peak_y - 4, cutoff_x + 4, peak_y + 4], fill=WHITE + (255,))
    d.ellipse([cutoff_x - 8, peak_y - 8, cutoff_x + 8, peak_y + 8], outline=AMBER + (200,), width=1)


def draw_animated_sequencer_leds(img: Image.Image, box: tuple[int, int, int, int], current_step: int):
    x0, y0, x1, y1 = box
    bw = x1 - x0
    d = ImageDraw.Draw(img)
    step_w = bw / 16.0

    for i in range(16):
        cx = int(x0 + i * step_w + step_w / 2.0)
        cy = y1 - 14
        active = (i == current_step)
        accent_step = (i % 4 == 0)
        col = MAGENTA if active else (AMBER if accent_step else CYAN)
        alpha = 255 if active else (160 if accent_step else 70)
        size = 6 if active else 4

        d.ellipse([cx - size, cy - size, cx + size, cy + size], fill=col + (alpha,))
        if active:
            d.ellipse([cx - size - 4, cy - size - 4, cx + size + 4, cy + size + 4], outline=col + (200,), width=1)


def place_screenshot(
    img: Image.Image,
    asset_name: str,
    box: tuple[int, int, int, int],
    zoom: float = 1.0,
    focus: tuple[float, float] = (0.5, 0.5),
    accent: tuple[int, int, int] = CYAN,
    glow: bool = True,
):
    src = get_asset(asset_name)
    sw, sh = src.size
    bx, by, bw, bh = box

    aspect_box = bw / bh
    base_w = sw / zoom
    base_h = sh / zoom

    if base_w / base_h > aspect_box:
        crop_h = base_h
        crop_w = base_h * aspect_box
    else:
        crop_w = base_w
        crop_h = base_w / aspect_box

    cx = focus[0] * sw
    cy = focus[1] * sh
    left = min(max(0, cx - crop_w / 2), sw - crop_w)
    top = min(max(0, cy - crop_h / 2), sh - crop_h)

    crop = src.crop((int(left), int(top), int(left + crop_w), int(top + crop_h)))
    crop = crop.resize((bw, bh), Image.LANCZOS)

    frame = Image.new("RGBA", (bw + 32, bh + 32), (0, 0, 0, 0))
    frame.paste(crop, (16, 16))

    if glow:
        halo = Image.new("RGBA", frame.size, (0, 0, 0, 0))
        hd = ImageDraw.Draw(halo)
        hd.rectangle([16, 16, 16 + bw, 16 + bh], outline=accent + (160,), width=3)
        halo = halo.filter(ImageFilter.GaussianBlur(12))
        frame.alpha_composite(halo)

    fd = ImageDraw.Draw(frame)
    fd.rectangle([16, 16, 16 + bw, 16 + bh], outline=accent + (210,), width=2)
    draw_hud_brackets(fd, (10, 10, 22 + bw, 22 + bh), accent, length=24, width=3)

    img.alpha_composite(frame, (bx - 16, by - 16))

# ---------------------------------------------------------------------------
# Global Telemetry HUD & Overlay Engine
# ---------------------------------------------------------------------------

def draw_top_hud(img: Image.Image, t: float, section_name: str, beat: float):
    d = ImageDraw.Draw(img)

    # Top dark glass bar
    d.rectangle([0, 0, W, 76], fill=(4, 2, 10, 220))
    d.line([(0, 76), (W, 76)], fill=CYAN + (90,), width=1)

    # Logo
    d.text((48, 20), "REPLICANT", font=F_MED, fill=WHITE + (255,))
    lw = d.textlength("REPLICANT", font=F_MED)
    d.text((48 + lw + 8, 20), "WAVE", font=F_MED, fill=MAGENTA + (255,))
    d.text((48, 48), "NIGHT CITY AUDIO OS // v1.2.1 · 64-BIT SIMD DSP", font=F_TINY, fill=DIM + (255,))

    # Center Section Title
    d.text((W // 2, 22), section_name, font=F_HUD, fill=CYAN + (240,), anchor="ma")
    # Section timeline ticks
    for i in range(12):
        x = W // 2 - 180 + i * 32
        on = (i / 11.0) < (t / DUR)
        d.rectangle([x, 50, x + 24, 53], fill=(CYAN if on else (45, 30, 65)) + (220,))

    # Right side: REC & Timecode
    sec = int(t)
    frames = int((t - sec) * FPS)
    d.text((W - 48, 20), f"2077:{sec // 60:02d}:{sec % 60:02d}:{frames:02d}", font=F_MED, fill=WHITE + (240,), anchor="ra")

    if int(t * 2) % 2 == 0:
        d.ellipse([W - 300, 26, W - 286, 40], fill=(255, 30, 80, 255))
    d.text((W - 280, 24), "REC ● LIVE", font=F_SMALL, fill=(255, 90, 130, 240))
    d.text((W - 48, 48), "AUDIO: 48.0 kHz 32-BIT // VOICES: 8/16 [POLY]", font=F_TINY, fill=DIM + (255,), anchor="ra")


def draw_bottom_analyzer(img: Image.Image, spec_row: np.ndarray, t: float):
    d = ImageDraw.Draw(img)
    base_y = H - 32
    n = len(spec_row)
    bar_w = 24
    total_w = n * bar_w
    x0 = W // 2 - total_w // 2

    for i, v in enumerate(spec_row):
        h = int(3 + v * 44)
        x = x0 + i * bar_w
        col = CYAN if i < n * 0.45 else NEON_PURPLE if i < n * 0.75 else MAGENTA
        d.rectangle([x, base_y - h, x + bar_w - 6, base_y], fill=col + (230,))
        d.rectangle([x, base_y + 3, x + bar_w - 6, base_y + 3 + int(h * 0.28)], fill=col + (55,))

    d.line([(0, base_y), (W, base_y)], fill=(255, 255, 255, 45), width=1)

    prog_w = W - 96
    d.rectangle([48, H - 12, 48 + prog_w, H - 8], fill=(30, 20, 50, 220))
    d.rectangle([48, H - 12, 48 + int(prog_w * (t / DUR)), H - 8], fill=CYAN + (240,))


def draw_active_preset_badge(img: Image.Image, t: float, num: str, name: str, category: str, desc: str, accent=CYAN, box=(32, 96, 330, 220)):
    x0, y0, x1, y1 = box
    draw_cyber_card(img, box, accent, fill_alpha=220)

    d = ImageDraw.Draw(img)
    d.rectangle([x0 + 14, y0 + 12, x0 + 68, y0 + 32], fill=accent + (40,), outline=accent + (200,), width=1)
    d.text((x0 + 41, y0 + 16), num, font=F_TINY, fill=accent + (255,), anchor="ma")

    d.text((x0 + 76, y0 + 15), category.upper(), font=F_TINY, fill=DIM + (255,))
    d.text((x0 + 14, y0 + 40), name, font=F_HUD, fill=WHITE + (255,))
    d.text((x0 + 14, y0 + 68), desc, font=F_TINY, fill=accent + (220,))


def draw_feature_banner(img: Image.Image, title: str, subtitle: str, accent: tuple[int, int, int], y_pos: int = 90):
    band_h = 74
    layer = Image.new("RGBA", (W, band_h), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    ld.rectangle([0, 0, W, band_h], fill=(6, 3, 14, 225))
    ld.rectangle([0, 0, 10, band_h], fill=accent + (255,))
    ld.line([(0, band_h - 1), (W, band_h - 1)], fill=accent + (120,), width=1)
    img.alpha_composite(layer, (0, y_pos))

    d = ImageDraw.Draw(img)
    d.text((48, y_pos + 12), title, font=F_MED, fill=accent + (255,))
    d.text((48, y_pos + 44), subtitle, font=F_SMALL, fill=WHITE + (230,))
    d.text((W - 48, y_pos + 26), "REPLICANTWAVE // DSP DEMO", font=F_TINY, fill=accent + (160,), anchor="ra")

# ---------------------------------------------------------------------------
# Scene Rendering Engine
# ---------------------------------------------------------------------------

def render_scene(t: float, spec_row: np.ndarray, wave_slice: np.ndarray) -> Image.Image:
    img = Image.new("RGBA", (W, H), (0, 0, 0, 255))
    img.alpha_composite(BACKDROP.convert("RGBA"))
    beat = beat_energy(t)
    frame_idx = int(t * FPS)

    # Grid floor on intro and outro
    if t < 9.0 or t > 58.0:
        alpha = int(140 * (1.0 - (t - 58.0) / 8.0) if t > 58.0 else min(1.0, t / 1.5))
        grid = GRID.copy()
        grid.putalpha(grid.getchannel("A").point(lambda a: int(a * alpha / 255)))
        img.alpha_composite(grid)

    # -----------------------------------------------------------------------
    # SCENE 1: BOOT SEQUENCE & HERO INTERFACE (0.0s - 8.0s | Bars 0 - 4)
    # Vangelis Blade Runner CS-80 Lead + Dystopian Pad
    # -----------------------------------------------------------------------
    if t < 8.0:
        section_name = "BOOT SEQUENCE // ANALOG WAVETABLES"
        if t < 3.2:
            title = "REPLICANTWAVE"
            d = ImageDraw.Draw(img)
            tw = d.textlength(title, font=F_TITLE)
            text_glow(img, (int(W / 2 - tw / 2), 340), title, F_TITLE, CYAN, glow=MAGENTA, radius=18)
            text_glow(img, (W // 2, 470), "NIGHT CITY AUDIO OS // CYBERPUNK SYNTHESIZER", F_MED, MAGENTA, radius=10, anchor="ma")
            d.text((W // 2, 530), "INITIALIZING NEURAL DSP CORE · 38 FACTORY PATCHES · 84 HOST PARAMS", font=F_SMALL, fill=DIM + (255,), anchor="ma")

            log_box = (W // 2 - 280, 580, W // 2 + 280, 680)
            draw_cyber_card(img, log_box, CYAN, fill_alpha=200)
            ld = ImageDraw.Draw(img)
            ld.text((log_box[0] + 20, log_box[1] + 16), "> LOADING DUAL WAVETABLE BANK... [OK]", font=F_TINY, fill=LIME + (255,))
            ld.text((log_box[0] + 20, log_box[1] + 38), "> 24dB ZERO-DELAY FEEDBACK FILTER... [ARMED]", font=F_TINY, fill=CYAN + (255,))
            ld.text((log_box[0] + 20, log_box[1] + 60), "> 64-STEP NEON MATRIX SEQUENCER... [SYNCED]", font=F_TINY, fill=MAGENTA + (255,))
        else:
            # Full UI centered at (360, 96, 1200, 850) with ZERO crop!
            push = ease((t - 3.2) / 4.8)
            zoom = lerp(1.03, 1.0, push)
            place_screenshot(img, "hero_gui", (360, 96, 1200, 850), zoom=zoom, focus=(0.5, 0.5), accent=CYAN)

            # Left margin badge
            draw_active_preset_badge(
                img, t, "03/38", "NEON BLADE 2049", "LEAD",
                "Vangelis CS-80 lead with pitch bloom & reverb", CYAN,
                box=(32, 96, 330, 210)
            )
            # Left telemetry stat block
            stat1 = (32, 230, 330, 480)
            draw_cyber_card(img, stat1, CYAN, fill_alpha=215)
            d = ImageDraw.Draw(img)
            d.text((stat1[0] + 16, stat1[1] + 14), "SYNTH ARCHITECTURE", font=F_HUD, fill=CYAN + (255,))
            d.line([(stat1[0] + 16, stat1[1] + 40), (stat1[2] - 16, stat1[1] + 40)], fill=CYAN + (70,), width=1)
            for i, (v, lbl) in enumerate([
                ("84", "HOST PARAMETERS"),
                ("38", "FACTORY PRESETS"),
                ("08", "WAVETABLES"),
                ("64", "STEP MATRIX"),
            ]):
                yy = stat1[1] + 52 + i * 46
                d.text((stat1[0] + 16, yy), v, font=F_MED, fill=WHITE + (255,))
                d.text((stat1[0] + 62, yy + 6), lbl, font=F_TINY, fill=DIM + (255,))

            # Right margin: Live vector oscilloscope
            osc_box = (1590, 96, W - 32, 260)
            draw_cyber_card(img, osc_box, MAGENTA, fill_alpha=220)
            d = ImageDraw.Draw(img)
            d.text((osc_box[0] + 16, osc_box[1] + 12), "LIVE VECTOR SCOPE", font=F_HUD, fill=MAGENTA + (255,))
            d.text((osc_box[0] + 16, osc_box[1] + 34), "Phosphor trace // 48 kHz", font=F_TINY, fill=DIM + (255,))
            draw_vector_oscilloscope(img, (osc_box[0] + 16, osc_box[1] + 54, osc_box[2] - 16, osc_box[3] - 16), wave_slice, CYAN, MAGENTA)

            # Right margin lower: Engine status
            stat2 = (1590, 280, W - 32, 480)
            draw_cyber_card(img, stat2, AMBER, fill_alpha=215)
            d = ImageDraw.Draw(img)
            d.text((stat2[0] + 16, stat2[1] + 14), "CORE TELEMETRY", font=F_HUD, fill=AMBER + (255,))
            d.line([(stat2[0] + 16, stat2[1] + 40), (stat2[2] - 16, stat2[1] + 40)], fill=AMBER + (70,), width=1)
            d.text((stat2[0] + 16, stat2[1] + 54), "DSP ENGINE: ACTIVE", font=F_SMALL, fill=LIME + (255,))
            d.text((stat2[0] + 16, stat2[1] + 78), "POLYPHONY: 16 VOICES", font=F_SMALL, fill=WHITE + (255,))
            d.text((stat2[0] + 16, stat2[1] + 102), "HOST AUTOMATION: READY", font=F_SMALL, fill=CYAN + (255,))
            d.text((stat2[0] + 16, stat2[1] + 126), "AUDIO FORMAT: 32-BIT FLT", font=F_SMALL, fill=DIM + (255,))

    # -----------------------------------------------------------------------
    # SCENE 2: DROP 1 - DUAL WAVETABLE ENGINE & NEURO BASS (8.0s - 24.0s | Bars 4 - 12)
    # Driving 16th Neuro Bass + 16th Netrunner Arp + Darkwave Drums
    # -----------------------------------------------------------------------
    elif t < 24.0:
        section_name = "SOUND ENGINE // DUAL WAVETABLES & NEURO BASS"
        local = (t - 8.0) / 16.0
        zoom = lerp(1.72, 1.58, ease(local))
        focus_x = lerp(0.30, 0.42, ease(local))
        focus_y = lerp(0.28, 0.32, ease(local))

        place_screenshot(img, "bass", (32, 180, 1280, 760), zoom=zoom, focus=(focus_x, focus_y), accent=CYAN)
        draw_feature_banner(img, "DUAL WAVETABLE SYNTHESIS ENGINE", "8 CYBERPUNK TABLES · 8-VOICE UNISON · PHASE WARP · FORMANT SHIFT", CYAN, y_pos=90)

        # Right Telemetry Stack
        card1 = (1340, 180, W - 32, 440)
        draw_cyber_card(img, card1, CYAN, fill_alpha=225)
        d = ImageDraw.Draw(img)
        d.text((card1[0] + 20, card1[1] + 16), "ACTIVE STEMS // 120 BPM", font=F_HUD, fill=CYAN + (255,))
        d.line([(card1[0] + 20, card1[1] + 44), (card1[2] - 20, card1[1] + 44)], fill=CYAN + (80,), width=1)

        d.text((card1[0] + 20, card1[1] + 58), "▶ PRESET 27: NEURO REAPER BASS", font=F_SMALL, fill=WHITE + (255,))
        d.text((card1[0] + 40, card1[1] + 80), "Distorted wavetable growl + 16th chug", font=F_TINY, fill=DIM + (255,))

        d.text((card1[0] + 20, card1[1] + 108), "▶ PRESET 05: NETRUNNER ARP", font=F_SMALL, fill=WHITE + (255,))
        d.text((card1[0] + 40, card1[1] + 130), "16th-note ping-pong stereo datastream", font=F_TINY, fill=DIM + (255,))

        d.text((card1[0] + 20, card1[1] + 158), "▶ PRESET 06: DARKSYNTH POWER STAB", font=F_SMALL, fill=WHITE + (255,))
        d.text((card1[0] + 40, card1[1] + 180), "Carpenter Brut analog brass punches", font=F_TINY, fill=DIM + (255,))

        d.text((card1[0] + 20, card1[1] + 210), "▶ SUB LAYER: 45 Hz Pure Sine", font=F_SMALL, fill=LIME + (255,))

        # Live Vector Oscilloscope in Card 2
        card2 = (1340, 464, W - 32, 720)
        draw_cyber_card(img, card2, MAGENTA, fill_alpha=225)
        d = ImageDraw.Draw(img)
        d.text((card2[0] + 20, card2[1] + 16), "REAL-TIME DSP VECTOR SCOPE", font=F_HUD, fill=MAGENTA + (255,))
        d.text((card2[0] + 20, card2[1] + 40), "Denormal-proof 64-bit SIMD float buffer", font=F_TINY, fill=DIM + (255,))
        draw_vector_oscilloscope(img, (card2[0] + 20, card2[1] + 64, card2[2] - 20, card2[3] - 20), wave_slice, CYAN, MAGENTA)

        # Bottom stat bar
        card3 = (1340, 744, W - 32, 850)
        draw_cyber_card(img, card3, AMBER, fill_alpha=225)
        d = ImageDraw.Draw(img)
        d.text((card3[0] + 20, card3[1] + 14), "ENGINE PERFORMANCE", font=F_TINY, fill=AMBER + (255,))
        d.text((card3[0] + 20, card3[1] + 34), "CPU LOAD: 2.9% // 16 VOICES // 48 kHz", font=F_SMALL, fill=WHITE + (255,))
        d.text((card3[0] + 20, card3[1] + 60), "ZERO LATENCY · SAMPLE-ACCURATE MIDI", font=F_TINY, fill=LIME + (255,))

    # -----------------------------------------------------------------------
    # SCENE 3: DROP 2 - ACID INJECTION & ZDF FILTER (24.0s - 40.0s | Bars 12 - 20)
    # Screaming Nightside Razor Lead + Chrome Acid 303 squelch
    # -----------------------------------------------------------------------
    elif t < 40.0:
        section_name = "SIGNAL CHAIN // 24dB ZDF FILTER & ACID RESIDUALS"
        local = (t - 24.0) / 16.0
        zoom = lerp(1.75, 1.62, ease(local))
        focus_x = lerp(0.55, 0.48, ease(local))
        focus_y = lerp(0.40, 0.36, ease(local))

        place_screenshot(img, "lead", (32, 180, 1260, 760), zoom=zoom, focus=(focus_x, focus_y), accent=AMBER)
        draw_feature_banner(img, "ANALOG-MODELED ZERO-DELAY FEEDBACK (ZDF) FILTER", "LP / HP / BP / NOTCH · 24dB/OCT · DRIVE SATURATION · ENVELOPE MODULATION", AMBER, y_pos=90)

        # Right Telemetry Stack
        card1 = (1320, 180, W - 32, 420)
        draw_cyber_card(img, card1, AMBER, fill_alpha=225)
        d = ImageDraw.Draw(img)
        d.text((card1[0] + 20, card1[1] + 16), "ACID & LEAD SHOWCASE", font=F_HUD, fill=AMBER + (255,))
        d.line([(card1[0] + 20, card1[1] + 44), (card1[2] - 20, card1[1] + 44)], fill=AMBER + (80,), width=1)

        d.text((card1[0] + 20, card1[1] + 58), "▶ PRESET 29: NIGHTSIDE RAZOR LEAD", font=F_SMALL, fill=WHITE + (255,))
        d.text((card1[0] + 40, card1[1] + 80), "Hard-synced wavetable screaming through delay", font=F_TINY, fill=DIM + (255,))

        d.text((card1[0] + 20, card1[1] + 108), "▶ PRESET 04: CHROME ACID 303", font=F_SMALL, fill=WHITE + (255,))
        d.text((card1[0] + 40, card1[1] + 130), "Classic 303 squelch, accent slides & resonant bite", font=F_TINY, fill=DIM + (255,))

        d.text((card1[0] + 20, card1[1] + 158), "▶ PRESET 09: GHOST IN THE WIRE CHOIR", font=F_SMALL, fill=WHITE + (255,))
        d.text((card1[0] + 40, card1[1] + 180), "Cyberpunk vocaloid spectral ambiance", font=F_TINY, fill=DIM + (255,))

        # Animated ZDF Filter Response Curve in Card 2
        card2 = (1320, 444, W - 32, 720)
        draw_cyber_card(img, card2, AMBER, fill_alpha=225)
        d = ImageDraw.Draw(img)
        d.text((card2[0] + 20, card2[1] + 16), "DYNAMIC FILTER FREQUENCY RESPONSE", font=F_HUD, fill=AMBER + (255,))
        d.text((card2[0] + 20, card2[1] + 40), "24dB 4-Pole Ladder Emulation + Nonlinear Resonance", font=F_TINY, fill=DIM + (255,))

        cutoff_hz = 1200.0 + 3800.0 * (0.5 + 0.5 * math.sin(t * 4.0))
        draw_filter_response(img, (card2[0] + 20, card2[1] + 70, card2[2] - 20, card2[3] - 20), cutoff_hz, reso=0.82)

        # Card 3: Host Automation
        card3 = (1320, 744, W - 32, 850)
        draw_cyber_card(img, card3, LIME, fill_alpha=225)
        d = ImageDraw.Draw(img)
        d.text((card3[0] + 20, card3[1] + 14), "HOST AUTOMATION READY", font=F_TINY, fill=LIME + (255,))
        d.text((card3[0] + 20, card3[1] + 34), "84 HOST PARAMETERS · MIDI LEARN ON EVERY KNOB", font=F_SMALL, fill=WHITE + (255,))
        d.text((card3[0] + 20, card3[1] + 60), "AUTOMATE CUTOFF, RESONANCE, DRIVE & FX FROM ANY DAW", font=F_TINY, fill=CYAN + (255,))

    # -----------------------------------------------------------------------
    # SCENE 4: BREAKDOWN - 64-STEP SEQUENCER & CYBER FX (40.0s - 48.0s | Bars 20 - 24)
    # Ethereal Choir + Pad + ZDF Filter Sweep + Accelerating Snare Roll
    # -----------------------------------------------------------------------
    elif t < 48.0:
        section_name = "PATTERN ENGINE // 64-STEP MATRIX & CYBER FX"
        place_screenshot(img, "seq", (32, 180, 1856, 530), zoom=1.0, focus=(0.5, 0.82), accent=MAGENTA)

        # Active animated step cursor in real-time (16th note = 0.125s @ 120 BPM)
        step_idx = int((t - 40.0) / 0.125) % 16
        draw_animated_sequencer_leds(img, (32, 180, 1856, 710), step_idx)

        draw_feature_banner(img, "INTEGRATED 64-STEP NEON MATRIX SEQUENCER", "4 PAGES · PER-STEP NOTE, VELOCITY, SLIDE & ACCENT · SWING · KEY TRANSPOSE", MAGENTA, y_pos=90)

        # Lower 3-column stats
        cw = 586
        for i, (col, title, sub, val) in enumerate([
            (MAGENTA, "MATRIX SEQUENCER", "4 Pages x 16 Steps // Slide & Accent", "64 STEPS"),
            (CYAN, "CYBER FX SUITE", "Distortion · Stereo Chorus · Delay · Reverb", "04 MODULES"),
            (LIME, "FACTORY PATCH BANK", "Tested & Audited by Automated CI", "38 PATCHES"),
        ]):
            bx = 32 + i * (cw + 48)
            card = (bx, 736, bx + cw, 856)
            draw_cyber_card(img, card, col, fill_alpha=220)
            d = ImageDraw.Draw(img)
            d.text((bx + 20, 752), val, font=F_HERO, fill=col + (255,))
            d.text((bx + 20, 804), title, font=F_SMALL, fill=WHITE + (255,))
            d.text((bx + 20, 826), sub, font=F_TINY, fill=DIM + (255,))

    # -----------------------------------------------------------------------
    # SCENE 5: MAXIMUM CLIMAX - ALL SYSTEMS ENGAGED (48.0s - 60.0s | Bars 24 - 30)
    # The heaviest drop: Drums, Neuro Bass, Screaming Lead, Choir, Acid 303!
    # -----------------------------------------------------------------------
    elif t < 60.0:
        section_name = "MAXIMUM CLIMAX // 38 PRESETS · 84 HOST CONTROLS"
        zoom = 1.0 + 0.02 * beat
        place_screenshot(img, "hero_gui", (360, 96, 1200, 850), zoom=zoom, focus=(0.5, 0.5), accent=MAGENTA)

        # Dynamic patch highlight during climax
        c_sub = t - 48.0
        if c_sub < 2.5:
            p_num, p_name, p_cat, p_desc, p_col = "27/38", "NEURO REAPER BASS", "BASS", "Distorted wavetable growl", CYAN
        elif c_sub < 5.0:
            p_num, p_name, p_cat, p_desc, p_col = "29/38", "NIGHTSIDE RAZOR", "LEAD", "Hard-synced screamer", AMBER
        elif c_sub < 7.5:
            p_num, p_name, p_cat, p_desc, p_col = "04/38", "CHROME ACID 303", "BASS", "Resonant acid squelch", LIME
        elif c_sub < 10.0:
            p_num, p_name, p_cat, p_desc, p_col = "19/38", "GHOST PROTOCOL", "PAD", "Cyber vocaloid choir", NEON_PURPLE
        else:
            p_num, p_name, p_cat, p_desc, p_col = "01/38", "NIGHT CITY BASS", "BASS", "Classic cyberpunk chug", MAGENTA

        draw_active_preset_badge(img, t, p_num, p_name, p_cat, p_desc, p_col, box=(32, 96, 330, 210))

        # Left lower telemetry
        stat1 = (32, 230, 330, 480)
        draw_cyber_card(img, stat1, p_col, fill_alpha=215)
        d = ImageDraw.Draw(img)
        d.text((stat1[0] + 16, stat1[1] + 14), "FULL ENGINE STATUS", font=F_HUD, fill=p_col + (255,))
        d.line([(stat1[0] + 16, stat1[1] + 40), (stat1[2] - 16, stat1[1] + 40)], fill=p_col + (70,), width=1)
        d.text((stat1[0] + 16, stat1[1] + 54), "10 SYNTH STEMS ACTIVE", font=F_SMALL, fill=WHITE + (255,))
        d.text((stat1[0] + 16, stat1[1] + 78), "16/16 VOICES ENGAGED", font=F_SMALL, fill=LIME + (255,))
        d.text((stat1[0] + 16, stat1[1] + 102), "PEAK LEVEL: -0.5 dBFS", font=F_SMALL, fill=CYAN + (255,))
        d.text((stat1[0] + 16, stat1[1] + 126), "DYNAMIC RANGE: FULL", font=F_SMALL, fill=DIM + (255,))

        # Right margin: Live vector oscilloscope
        osc_box = (1590, 96, W - 32, 260)
        draw_cyber_card(img, osc_box, MAGENTA, fill_alpha=220)
        d = ImageDraw.Draw(img)
        d.text((osc_box[0] + 16, osc_box[1] + 12), "CLIMAX WAVEFORM", font=F_HUD, fill=MAGENTA + (255,))
        d.text((osc_box[0] + 16, osc_box[1] + 34), "Full harmonic spectrum", font=F_TINY, fill=DIM + (255,))
        draw_vector_oscilloscope(img, (osc_box[0] + 16, osc_box[1] + 54, osc_box[2] - 16, osc_box[3] - 16), wave_slice, CYAN, MAGENTA)

        # Right margin lower: Master bus stats
        stat2 = (1590, 280, W - 32, 480)
        draw_cyber_card(img, stat2, AMBER, fill_alpha=215)
        d = ImageDraw.Draw(img)
        d.text((stat2[0] + 16, stat2[1] + 14), "MASTER BUS TELEMETRY", font=F_HUD, fill=AMBER + (255,))
        d.line([(stat2[0] + 16, stat2[1] + 40), (stat2[2] - 16, stat2[1] + 40)], fill=AMBER + (70,), width=1)
        d.text((stat2[0] + 16, stat2[1] + 54), "RMS POWER: 0.395", font=F_SMALL, fill=WHITE + (255,))
        d.text((stat2[0] + 16, stat2[1] + 78), "SIDECHAIN PUMP: ON", font=F_SMALL, fill=LIME + (255,))
        d.text((stat2[0] + 16, stat2[1] + 102), "ANALOG SATURATION: ON", font=F_SMALL, fill=AMBER + (255,))
        d.text((stat2[0] + 16, stat2[1] + 126), "ZERO LATENCY // 120 BPM", font=F_SMALL, fill=CYAN + (255,))

    # -----------------------------------------------------------------------
    # SCENE 6: OUTRO - THE END OF TRANSMISSION (60.0s - 66.0s | Bars 30 - 32)
    # Sub impact, delay decay, resolution on D minor
    # -----------------------------------------------------------------------
    else:
        section_name = "END OF TRANSMISSION // FREE & OPEN SOURCE"
        d = ImageDraw.Draw(img)
        y = 260
        title = "REPLICANTWAVE"
        tw = d.textlength(title, font=F_TITLE)
        text_glow(img, (int(W / 2 - tw / 2), y), title, F_TITLE, CYAN, glow=MAGENTA, radius=18)

        sub = "CYBERNETIC WAVETABLE SYNTHESIZER FOR CYBERPUNK, DARKSYNTH & INDUSTRIAL"
        sw = d.textlength(sub, font=F_SMALL)
        d.text((W // 2 - sw / 2, y + 115), sub, font=F_SMALL, fill=WHITE + (240,))

        chips = [
            ("VST3", CYAN),
            ("CLAP", MAGENTA),
            ("AU", NEON_PURPLE),
            ("STANDALONE", LIME),
        ]
        cw = 210
        total_cw = len(chips) * cw + (len(chips) - 1) * 20
        start_x = W // 2 - total_cw // 2

        for i, (chip_text, col) in enumerate(chips):
            cx = start_x + i * (cw + 20)
            card = (cx, y + 175, cx + cw, y + 235)
            draw_cyber_card(img, card, col, fill_alpha=230)
            cd = ImageDraw.Draw(img)
            cd.text((cx + cw // 2, y + 195), chip_text, font=F_HUD, fill=col + (255,), anchor="ma")

        os_text = "LINUX · WINDOWS · MACOS (APPLE SILICON & INTEL)"
        ow = d.textlength(os_text, font=F_SMALL)
        d.text((W // 2 - ow / 2, y + 265), os_text, font=F_SMALL, fill=AMBER + (255,))

        url = "github.com/DanimanX/ReplicantWave"
        uw = d.textlength(url, font=F_MED)
        text_glow(img, (int(W / 2 - uw / 2), y + 315), url, F_MED, WHITE, glow=CYAN, radius=10)

        foot = "100% FREE & OPEN SOURCE · PURE RUST DSP · GPL-3.0"
        fw = d.textlength(foot, font=F_TINY)
        d.text((W // 2 - fw / 2, y + 375), foot, font=F_TINY, fill=DIM + (255,))

    # -----------------------------------------------------------------------
    # Global Glitches on Beat Drops
    # -----------------------------------------------------------------------
    g = 0.0
    if 2.4 <= t <= 3.3:
        g = 0.85 * (1.0 - (t - 2.4) / 0.9)
    elif 7.95 <= t <= 8.35:
        g = 1.0 * (1.0 - (t - 7.95) / 0.40)
    elif 23.95 <= t <= 24.35:
        g = 0.95 * (1.0 - (t - 23.95) / 0.40)
    elif 47.85 <= t <= 48.40:
        g = 1.1 * (1.0 - (t - 47.85) / 0.55)
    for boundary in (40.0, 60.0):
        if boundary <= t < boundary + 0.25:
            g = max(g, 0.75 * (1.0 - (t - boundary) / 0.25))

    if g > 0.02:
        img = glitch_effect(img, g, int(t * 1000) & 0xFFFF)

    draw_top_hud(img, t, section_name, beat)
    draw_bottom_analyzer(img, spec_row, t)

    if beat > 0.05:
        flash = Image.new("RGBA", (W, H), (140, 20, 160, int(18 * beat)))
        img.alpha_composite(flash)

    img = Image.blend(img, GRAIN[frame_idx % len(GRAIN)].convert("RGBA"), 0.04)
    img.alpha_composite(SCANLINES)
    img.alpha_composite(VIGNETTE)

    if t > DUR - 1.6:
        f_alpha = int(255 * min(1.0, (t - (DUR - 1.6)) / 1.6))
        black = Image.new("RGBA", (W, H), (0, 0, 0, f_alpha))
        img.alpha_composite(black)

    return img

# ---------------------------------------------------------------------------
# Main Video Renderer
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="Render ReplicantWave Cyberpunk Promo Video")
    parser.add_argument("--preview", action="store_true", help="Generate QA preview frames only")
    args = parser.parse_args()

    if not os.path.exists(WAV):
        sys.exit(f"Missing audio: {WAV}; run: cargo run --release --example promo_track")

    print("Loading audio and computing spectrum & waveform caches...")
    sr, mono, stereo = load_audio()
    spec = compute_spectrum(mono, sr)
    print(f"Audio: {len(mono) / sr:.2f}s @ {sr} Hz, {FRAMES} video frames")

    if args.preview:
        out_prev = os.path.join(ROOT, "target/promo/preview")
        os.makedirs(out_prev, exist_ok=True)
        test_times = [2.0, 5.0, 9.0, 16.0, 25.0, 32.0, 42.0, 50.0, 55.0, 62.0]
        for t in test_times:
            f = min(FRAMES - 1, int(t * FPS))
            w_slice = get_wave_slice(stereo, sr, f)
            frame_img = render_scene(t, spec[f], w_slice)
            p = os.path.join(out_prev, f"preview_{t:04.1f}s.png")
            frame_img.convert("RGB").save(p)
            print(f"Saved preview: {p}")
        return

    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    print(f"Rendering full video to {OUT} via ffmpeg pipe...")

    cmd = [
        "ffmpeg", "-y", "-hide_banner", "-loglevel", "error",
        "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", str(FPS), "-i", "-",
        "-i", WAV,
        "-map", "0:v", "-map", "1:a",
        "-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p",
        "-profile:v", "high", "-g", "60", "-movflags", "+faststart",
        "-c:a", "aac", "-b:a", "256k", "-ar", "48000",
        "-shortest", OUT,
    ]

    proc = subprocess.Popen(cmd, stdin=subprocess.PIPE)

    for f in range(FRAMES):
        t = f / FPS
        w_slice = get_wave_slice(stereo, sr, f)
        frame_img = render_scene(t, spec[f], w_slice)
        proc.stdin.write(frame_img.convert("RGB").tobytes())

        if f % 150 == 0 or f == FRAMES - 1:
            pct = (f / (FRAMES - 1)) * 100.0
            print(f"  Frame {f:4d}/{FRAMES} ({t:5.1f}s / {DUR:.1f}s) — {pct:5.1f}%", flush=True)

    proc.stdin.close()
    rc = proc.wait()
    if rc != 0:
        sys.exit(f"ffmpeg failed with exit code {rc}")

    size_mb = os.path.getsize(OUT) / 1e6
    print(f"Successfully generated {OUT} ({size_mb:.1f} MB, {DUR:.1f}s @ 1080p30)")


if __name__ == "__main__":
    main()
