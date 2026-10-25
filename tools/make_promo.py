#!/usr/bin/env python3
"""Build the ReplicantWave Cyberpunk Midtempo Promo Video (1920x1080 @ 30fps).

Showcases the 3D Holographic Oscillator Effect and a crushing Midtempo Cyberpunk
soundtrack (105 BPM in F minor, inspired by Zeneth / Reinelex Music mixes):
- Dedicated 3D Holographic Wavetable Oscillators with 11 volumetric depth slices,
  3D orbital perspective camera, longitudinal mesh ribbons, and traveling scan planes
- Zeneth Cyber Midtempo growls, Reinelex Neuro Reese, & Cyberpunk 2077 Combat Lead
- Analog-modeled Zero-Delay Feedback (ZDF) filter with dynamic frequency curves
- Integrated 64-step matrix sequencer with real-time animated LEDs at 105 BPM
- 84 host-automatable parameters & 42 factory presets

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
DUR = 66.0  # 28 bars @ 105 BPM = 64.0s + 2.0s tail
FRAMES = int(DUR * FPS)
BPM = 105.0
BEAT = 60.0 / BPM
BAR = BEAT * 4.0

# Cyberpunk Palette
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
    edges = np.geomspace(35.0, 13000.0, bands + 1)
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
    # 105 BPM midtempo beat energy from bar 4 (approx 9.14s)
    t0 = 4.0 * BAR
    if t < t0:
        return 0.0
    phase = ((t - t0) % BEAT)
    return max(0.0, 1.0 - phase / 0.13)

# ---------------------------------------------------------------------------
# Visual Backgrounds & Overlays
# ---------------------------------------------------------------------------

def make_backdrop():
    img = Image.new("RGB", (W, H), DARK_BG)
    px = np.zeros((H, W, 3), dtype=np.float32)
    for y in range(H):
        u = y / (H - 1)
        top = np.array([22, 8, 42], dtype=np.float32)
        bot = np.array([4, 2, 10], dtype=np.float32)
        px[y, :, :] = top * (1 - u) + bot * u

    yy, xx = np.mgrid[0:H, 0:W].astype(np.float32)
    d = np.sqrt((xx - W / 2) ** 2 + (yy - 80) ** 2)
    bloom = np.clip(1.0 - d / 980.0, 0, 1) ** 3 * 54
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
# 3D Wavetable Synthesis Functions (Authentic ReplicantWave Tables)
# ---------------------------------------------------------------------------

def wavetable_cyber_neuro(phase: float, frame_t: float) -> float:
    """Cyber Neuro: Wavefolded harmonics morphing with frame_t."""
    s = 0.0
    for h in range(1, 9):
        amp = 1.0 / (h ** (0.8 - 0.3 * frame_t))
        phase_shift = frame_t * h * 0.7
        s += amp * math.sin((phase * h + phase_shift) * 2.0 * math.pi)
    drive = 1.0 + 3.0 * frame_t
    return math.sin(s * drive) * 0.88


def wavetable_cyber_acid(phase: float, frame_t: float) -> float:
    """Cyber Acid: Squelching saw wave with resonant harmonic folds."""
    saw = 2.0 * (phase - 0.5)
    f_res = 4.0 + 12.0 * frame_t
    res = math.sin(phase * f_res * 2.0 * math.pi) * (0.35 + 0.65 * frame_t)
    return (saw * 0.75 + res * 0.45).clamp(-1.05, 1.05) if hasattr(float, "clamp") else max(-1.05, min(1.05, saw * 0.75 + res * 0.45))


def wavetable_analog_morph(phase: float, frame_t: float) -> float:
    """Analog Morph: Saw -> Square -> Pulse."""
    saw = 2.0 * (phase - 0.5)
    pw = 0.5 - 0.35 * frame_t
    square = 1.0 if phase < pw else -1.0
    return saw * (1.0 - frame_t) + square * frame_t

# ---------------------------------------------------------------------------
# 3D Holographic Oscillator Renderer
# ---------------------------------------------------------------------------

def draw_3d_wavetable_hologram(
    img: Image.Image,
    box: tuple[int, int, int, int],
    table_func,
    color: tuple[int, int, int],
    title: str,
    table_name: str,
    t: float,
    pos: float = 0.5,
    audio_reactive: float = 0.0,
):
    """Renders the true 3D Holographic Wavetable Display of ReplicantWave.

    Uses full 3D perspective projection, orbital camera yaw/pitch, 11 volumetric depth
    slices, longitudinal wireframe mesh ribbons, traveling scan laser plane, and
    an active morphed waveform with translucent vertical fill.
    """
    x0, y0, x1, y1 = box
    bw = x1 - x0
    bh = y1 - y0

    # 1. Dark CRT Well with glow border
    d = ImageDraw.Draw(img)
    d.rectangle([x0, y0, x1, y1], fill=(6, 3, 14, 235))
    d.rectangle([x0, y0, x1, y1], outline=color + (140,), width=1)
    draw_hud_brackets(d, (x0 - 4, y0 - 4, x1 + 4, y1 + 4), color, length=18, width=2)

    # Header in holographic well
    d.text((x0 + 14, y0 + 10), title, font=F_HUD, fill=color + (255,))
    d.text((x0 + 14, y0 + 32), f"TABLE: {table_name} // 3D VOLUMETRIC VIEW", font=F_TINY, fill=DIM + (255,))
    d.text((x1 - 14, y0 + 14), f"POS: {pos:.2f}", font=F_SMALL, fill=WHITE + (240,), anchor="ra")

    # Internal CRT grid lines
    for i in range(1, 8):
        gx = x0 + int(bw * i / 8)
        d.line([(gx, y0 + 50), (gx, y1 - 10)], fill=color + (18,), width=1)
    for j in range(1, 4):
        gy = y0 + 50 + int((bh - 60) * j / 4)
        d.line([(x0 + 10, gy), (x1 - 10, gy)], fill=color + (16,), width=1)

    # 3D Perspective Geometry (matches src/gui/wavetable_view.rs)
    slices_count = 11
    pts_per_slice = 84

    top_y = y0 + 64.0
    bottom_y = y1 - 20.0
    span_y = bottom_y - top_y

    # Orbital camera parallax
    cam_yaw = math.sin(t * 0.70) * 16.0
    cam_pitch = 0.5 + 0.5 * math.sin(t * 0.50 + 1.1)
    depth_x = 28.0 + cam_pitch * 16.0
    margin_x = 16.0

    amp_budget = span_y * 0.32
    band = span_y - 2.0 * amp_budget
    y_front = top_y + amp_budget + band
    y_back = top_y + amp_budget
    amp_base = amp_budget * (0.88 + 0.12 * cam_pitch)

    def project_3d(phase: float, amp: float, z: float) -> tuple[int, int]:
        persp = 0.74 + 0.26 * z
        baseline = y_back + z * (y_front - y_back)
        start_x = x0 + margin_x + (1.0 - z) * depth_x + cam_yaw * (1.0 - z)
        avail_w = (bw - margin_x * 2.0 - depth_x) * persp * (1.0 + 0.04 * cam_yaw / 9.0)
        px = start_x + phase * avail_w
        py = baseline - amp * amp_base * persp
        return int(px), int(py)

    # Traveling Depth Scan Plane (triangle sweep along Z)
    scan_phase = (t * 0.40) % 1.0
    scan_z = scan_phase * 2.0 if scan_phase < 0.5 else (1.0 - scan_phase) * 2.0

    all_slices = []

    # Draw 11 Depth Slices
    for s in range(slices_count):
        z = s / (slices_count - 1)
        dist = abs(z - scan_z)
        lit = max(0.0, 1.0 - dist / 0.24)

        alpha = int(min(240, 30 + z * 80 + lit * 140))
        tint = 0.4 + z * 0.6 + lit * 0.3
        col = (
            min(255, int(color[0] * tint)),
            min(255, int(color[1] * tint)),
            min(255, int(color[2] * tint)),
            alpha,
        )

        slice_phase = z * 5.2 + s * 0.4
        pts = []
        for i in range(pts_per_slice):
            phase = i / (pts_per_slice - 1)
            raw_amp = table_func(phase, z)
            # Audio-reactive ripple
            ripple = 1.0 + 0.08 * math.sin(phase * 12.0 - t * 3.0 + slice_phase)
            if audio_reactive > 0.05:
                ripple += audio_reactive * 0.20 * math.sin(phase * 18.0 + t * 6.0)
            drift = 0.02 * math.sin(phase * 5.0 + t * 1.5 + slice_phase)
            amp = raw_amp * ripple + drift
            pts.append(project_3d(phase, amp, z))

        all_slices.append(pts)

        # Draw slice line with laser scan glow
        if lit > 0.35:
            d.line(pts, fill=color + (int(lit * 90),), width=4)
        d.line(pts, fill=col, width=2 if lit > 0.3 else 1)

    # Draw 5 Longitudinal 3D Wireframe Ribbons connecting slices
    for idx in [
        pts_per_slice // 5,
        2 * pts_per_slice // 5,
        pts_per_slice // 2,
        3 * pts_per_slice // 5,
        4 * pts_per_slice // 5,
    ]:
        ribbon = [all_slices[s][idx] for s in range(slices_count)]
        d.line(ribbon, fill=color + (42,), width=1)

    # Draw Active Morphed Wave Ribbon (at current POS)
    z_active = 1.0 - max(0.0, min(1.0, pos))
    active_pts = []
    base_pts = []
    for i in range(pts_per_slice):
        phase = i / (pts_per_slice - 1)
        pulse = 1.0 + 0.08 * math.sin(phase * 14.0 - t * 4.0)
        drift = 0.015 * math.sin(phase * 6.0 + t * 2.0)
        amp = table_func(phase, pos) * pulse + drift
        pt = project_3d(phase, amp, z_active)
        b_pt = project_3d(phase, 0.0, z_active)
        active_pts.append(pt)
        base_pts.append(b_pt)

    # Translucent vertical 3D ribbon under active wave
    poly = active_pts + list(reversed(base_pts))
    d.polygon(poly, fill=color + (40,))

    # Glowing halo line + core sharp line (vector phosphor effect)
    d.line(active_pts, fill=color + (140,), width=5)
    d.line(active_pts, fill=WHITE + (255,), width=2)

# ---------------------------------------------------------------------------
# Motion Graphics & Telemetry Overlays
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


def draw_top_hud(img: Image.Image, t: float, section_name: str, beat: float):
    d = ImageDraw.Draw(img)

    d.rectangle([0, 0, W, 76], fill=(4, 2, 10, 220))
    d.line([(0, 76), (W, 76)], fill=CYAN + (90,), width=1)

    d.text((48, 20), "REPLICANT", font=F_MED, fill=WHITE + (255,))
    lw = d.textlength("REPLICANT", font=F_MED)
    d.text((48 + lw + 8, 20), "WAVE", font=F_MED, fill=MAGENTA + (255,))
    d.text((48, 48), "NIGHT CITY AUDIO OS // v1.2.1 · 3D WAVETABLE DSP", font=F_TINY, fill=DIM + (255,))

    d.text((W // 2, 22), section_name, font=F_HUD, fill=CYAN + (240,), anchor="ma")
    for i in range(12):
        x = W // 2 - 180 + i * 32
        on = (i / 11.0) < (t / DUR)
        d.rectangle([x, 50, x + 24, 53], fill=(CYAN if on else (45, 30, 65)) + (220,))

    sec = int(t)
    frames = int((t - sec) * FPS)
    d.text((W - 48, 20), f"2077:{sec // 60:02d}:{sec % 60:02d}:{frames:02d}", font=F_MED, fill=WHITE + (240,), anchor="ra")

    if int(t * 2) % 2 == 0:
        d.ellipse([W - 300, 26, W - 286, 40], fill=(255, 30, 80, 255))
    d.text((W - 280, 24), "REC ● LIVE", font=F_SMALL, fill=(255, 90, 130, 240))
    d.text((W - 48, 48), "TEMPO: 105 BPM // 3D DUAL OSCILLATORS", font=F_TINY, fill=DIM + (255,), anchor="ra")


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


def draw_active_preset_badge(img: Image.Image, num: str, name: str, category: str, desc: str, accent=CYAN, box=(32, 96, 330, 210)):
    draw_cyber_card(img, box, accent, fill_alpha=220)
    x0, y0, x1, y1 = box
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
    d.text((W - 48, y_pos + 26), "REPLICANTWAVE // 3D SYNTHESIS DEMO", font=F_TINY, fill=accent + (160,), anchor="ra")

# ---------------------------------------------------------------------------
# Scene Rendering Engine (3D Oscillator Showcase + Midtempo Cyberpunk)
# ---------------------------------------------------------------------------

def render_scene(t: float, spec_row: np.ndarray, wave_slice: np.ndarray) -> Image.Image:
    img = Image.new("RGBA", (W, H), (0, 0, 0, 255))
    img.alpha_composite(BACKDROP.convert("RGBA"))
    beat = beat_energy(t)
    frame_idx = int(t * FPS)

    # Grid floor on intro and outro
    if t < 9.5 or t > 58.0:
        alpha = int(140 * (1.0 - (t - 58.0) / 8.0) if t > 58.0 else min(1.0, t / 1.5))
        grid = GRID.copy()
        grid.putalpha(grid.getchannel("A").point(lambda a: int(a * alpha / 255)))
        img.alpha_composite(grid)

    # -----------------------------------------------------------------------
    # SCENE 1: BOOT SEQUENCE & HERO INTERFACE (0.0s - 9.1s | Bars 0 - 4)
    # Atmospheric Intro: Corpo Tower Dystopia Pad + Dystopian Drone
    # -----------------------------------------------------------------------
    if t < 9.14:
        section_name = "BOOT SEQUENCE // DUAL 3D OSCILLATORS ONLINE"
        if t < 3.2:
            title = "REPLICANTWAVE"
            d = ImageDraw.Draw(img)
            tw = d.textlength(title, font=F_TITLE)
            text_glow(img, (int(W / 2 - tw / 2), 340), title, F_TITLE, CYAN, glow=MAGENTA, radius=18)
            text_glow(img, (W // 2, 470), "NIGHT CITY AUDIO OS // CYBERPUNK SYNTHESIZER", F_MED, MAGENTA, radius=10, anchor="ma")
            d.text((W // 2, 530), "INITIALIZING 3D DUAL WAVETABLE DSP · 42 FACTORY PATCHES · 84 HOST PARAMS", font=F_SMALL, fill=DIM + (255,), anchor="ma")

            log_box = (W // 2 - 290, 580, W // 2 + 290, 680)
            draw_cyber_card(img, log_box, CYAN, fill_alpha=200)
            ld = ImageDraw.Draw(img)
            ld.text((log_box[0] + 20, log_box[1] + 16), "> 3D HOLOGRAPHIC OSCILLATORS A & B... [ARMED]", font=F_TINY, fill=LIME + (255,))
            ld.text((log_box[0] + 20, log_box[1] + 38), "> 11-SLICE VOLUMETRIC ORBITAL DISPLAY... [SYNCED]", font=F_TINY, fill=CYAN + (255,))
            ld.text((log_box[0] + 20, log_box[1] + 60), "> MIDTEMPO BASS ENGINE (105 BPM)... [READY]", font=F_TINY, fill=MAGENTA + (255,))
        else:
            # Full UI centered at (360, 96, 1200, 850) with ZERO crop
            push = ease((t - 3.2) / 5.9)
            zoom = lerp(1.03, 1.0, push)
            place_screenshot(img, "hero_gui", (360, 96, 1200, 850), zoom=zoom, focus=(0.5, 0.5), accent=CYAN)

            # Left margin badge
            draw_active_preset_badge(
                img, "07/42", "CORPO DYSTOPIA", "PAD",
                "Atmospheric pad with 3D stereo detune", CYAN,
                box=(32, 96, 330, 210)
            )
            # Left telemetry stat block
            stat1 = (32, 230, 330, 480)
            draw_cyber_card(img, stat1, CYAN, fill_alpha=215)
            d = ImageDraw.Draw(img)
            d.text((stat1[0] + 16, stat1[1] + 14), "3D SYNTH ARCHITECTURE", font=F_HUD, fill=CYAN + (255,))
            d.line([(stat1[0] + 16, stat1[1] + 40), (stat1[2] - 16, stat1[1] + 40)], fill=CYAN + (70,), width=1)
            for i, (v, lbl) in enumerate([
                ("3D", "VOLUMETRIC OSCILLATORS"),
                ("42", "CYBERPUNK PRESETS"),
                ("84", "HOST PARAMETERS"),
                ("64", "STEP MATRIX"),
            ]):
                yy = stat1[1] + 52 + i * 46
                d.text((stat1[0] + 16, yy), v, font=F_MED, fill=WHITE + (255,))
                d.text((stat1[0] + 62, yy + 6), lbl, font=F_TINY, fill=DIM + (255,))

            # Right margin: Live vector oscilloscope
            osc_box = (1590, 96, W - 32, 260)
            draw_cyber_card(img, osc_box, MAGENTA, fill_alpha=220)
            d = ImageDraw.Draw(img)
            d.text((osc_box[0] + 16, osc_box[1] + 12), "LIVE 3D VECTOR SCOPE", font=F_HUD, fill=MAGENTA + (255,))
            d.text((osc_box[0] + 16, osc_box[1] + 34), "Dual phosphor persistence trace", font=F_TINY, fill=DIM + (255,))
            draw_vector_oscilloscope(img, (osc_box[0] + 16, osc_box[1] + 54, osc_box[2] - 16, osc_box[3] - 16), wave_slice, CYAN, MAGENTA)

            # Right margin lower: Core telemetry
            stat2 = (1590, 280, W - 32, 480)
            draw_cyber_card(img, stat2, AMBER, fill_alpha=215)
            d = ImageDraw.Draw(img)
            d.text((stat2[0] + 16, stat2[1] + 14), "CORE TELEMETRY", font=F_HUD, fill=AMBER + (255,))
            d.line([(stat2[0] + 16, stat2[1] + 40), (stat2[2] - 16, stat2[1] + 40)], fill=AMBER + (70,), width=1)
            d.text((stat2[0] + 16, stat2[1] + 54), "DSP ENGINE: ACTIVE", font=F_SMALL, fill=LIME + (255,))
            d.text((stat2[0] + 16, stat2[1] + 78), "TEMPO: 105.0 BPM", font=F_SMALL, fill=WHITE + (255,))
            d.text((stat2[0] + 16, stat2[1] + 102), "AUDIO: 48 kHz 32-BIT", font=F_SMALL, fill=CYAN + (255,))
            d.text((stat2[0] + 16, stat2[1] + 126), "SIMD: AVX2 OPTIMIZED", font=F_SMALL, fill=DIM + (255,))

    # -----------------------------------------------------------------------
    # SCENE 2: DROP 1 - DUAL 3D HOLOGRAPHIC OSCILLATORS SHOWCASE (9.14s - 27.4s | Bars 4 - 12)
    # Zeneth Cyber Midtempo Growl + Night City Midtempo Chug
    # -----------------------------------------------------------------------
    elif t < 27.43:
        section_name = "3D OSCILLATORS // VOLUMETRIC HOLOGRAPHIC ENGINE"
        local = (t - 9.14) / 18.29

        draw_feature_banner(
            img,
            "DUAL 3D HOLOGRAPHIC WAVETABLE OSCILLATORS",
            "11 VOLUMETRIC SLICES · ORBITAL CAMERA PARALLAX · REAL-TIME DEPTH SCAN · WAVEFOLDING",
            CYAN,
            y_pos=90,
        )

        # Large 3D Hologram of OSCILLATOR 1 (Cyber Neuro) on the left
        osc1_box = (48, 180, 940, 680)
        pos1 = 0.45 + 0.35 * math.sin(t * 1.5)
        draw_3d_wavetable_hologram(
            img,
            osc1_box,
            wavetable_cyber_neuro,
            CYAN,
            "OSCILLATOR 1 // 3D HOLOGRAPHIC DISPLAY",
            "CYBER NEURO",
            t,
            pos=pos1,
            audio_reactive=beat,
        )

        # Large 3D Hologram of OSCILLATOR 2 (Cyber Acid) on the right
        osc2_box = (980, 180, W - 48, 680)
        pos2 = 0.65 + 0.25 * math.sin(t * 2.1)
        draw_3d_wavetable_hologram(
            img,
            osc2_box,
            wavetable_cyber_acid,
            MAGENTA,
            "OSCILLATOR 2 // VOLUMETRIC STEREO DISPLAY",
            "CYBER ACID",
            t,
            pos=pos2,
            audio_reactive=beat,
        )

        # Lower Telemetry Bar: Active Preset & Controls Info
        lower_box1 = (48, 700, 940, 856)
        draw_cyber_card(img, lower_box1, CYAN, fill_alpha=225)
        d = ImageDraw.Draw(img)
        d.text((lower_box1[0] + 20, lower_box1[1] + 16), "▶ ACTIVE PRESET: ZENETH CYBER MIDTEMPO", font=F_HUD, fill=CYAN + (255,))
        d.text((lower_box1[0] + 20, lower_box1[1] + 44), "8-VOICE UNISON // STEREO SPREAD: 95% // WAVEFOLDED DRIVE: 2.8x", font=F_SMALL, fill=WHITE + (255,))
        d.text((lower_box1[0] + 20, lower_box1[1] + 70), "WARP: FORMANT SHIFT · HARD TUBE CLIPPING · 105 BPM MIDTEMPO GROOVE", font=F_TINY, fill=DIM + (255,))

        lower_box2 = (980, 700, W - 48, 856)
        draw_cyber_card(img, lower_box2, MAGENTA, fill_alpha=225)
        d = ImageDraw.Draw(img)
        d.text((lower_box2[0] + 20, lower_box2[1] + 16), "▶ 3D PERSPECTIVE RENDERING ENGINE", font=F_HUD, fill=MAGENTA + (255,))
        d.text((lower_box2[0] + 20, lower_box2[1] + 44), "DYNAMIC DEPTH SCAN // REAL-TIME ORBITAL PITCH & YAW // VOLUMETRIC MESH", font=F_SMALL, fill=WHITE + (255,))
        d.text((lower_box2[0] + 20, lower_box2[1] + 70), "SUB-ZERO LATENCY · GPU-ACCELERATED HOLOGRAPHIC PROJECTION", font=F_TINY, fill=LIME + (255,))

    # -----------------------------------------------------------------------
    # SCENE 3: DROP 2 - 3D WAVETABLE MORPH & COMBAT LEAD (27.43s - 45.71s | Bars 12 - 20)
    # Cyberpunk 2077 Combat Lead + Reinelex Neuro Reese
    # -----------------------------------------------------------------------
    elif t < 45.71:
        section_name = "SIGNAL CHAIN // 3D MORPHING & COMBAT LEAD"
        local = (t - 27.43) / 18.28

        draw_feature_banner(
            img,
            "REAL-TIME 3D WAVETABLE POSITION MORPHING & ZDF FILTER",
            "SEARING HARD-SYNC COMBAT LEAD · REINELEX NEURO REESE · 24dB RESONANT SQUELCH",
            AMBER,
            y_pos=90,
        )

        # Left: Large 3D Hologram sweeping position continuously
        osc_box = (48, 180, 940, 680)
        pos_sweep = (math.sin(t * 1.8) * 0.5 + 0.5)
        draw_3d_wavetable_hologram(
            img,
            osc_box,
            wavetable_analog_morph,
            AMBER,
            "OSCILLATOR 1 // CONTINUOUS 3D MORPH",
            "ANALOG MORPH (SAW -> SQUARE)",
            t,
            pos=pos_sweep,
            audio_reactive=beat,
        )

        # Right Top: Filter Response Curve
        flt_box = (980, 180, W - 48, 450)
        draw_cyber_card(img, flt_box, AMBER, fill_alpha=225)
        d = ImageDraw.Draw(img)
        d.text((flt_box[0] + 20, flt_box[1] + 16), "ZDF 24dB FILTER FREQUENCY RESPONSE", font=F_HUD, fill=AMBER + (255,))
        d.text((flt_box[0] + 20, flt_box[1] + 40), "Zero-Delay Feedback analog ladder with non-linear saturation", font=F_TINY, fill=DIM + (255,))

        cutoff_hz = 1400.0 + 3600.0 * (0.5 + 0.5 * math.sin(t * 3.5))
        draw_filter_response(img, (flt_box[0] + 20, flt_box[1] + 70, flt_box[2] - 20, flt_box[3] - 20), cutoff_hz, reso=0.85)

        # Right Middle: Active Stems Telemetry
        stems_box = (980, 474, W - 48, 700)
        draw_cyber_card(img, stems_box, CYAN, fill_alpha=225)
        d = ImageDraw.Draw(img)
        d.text((stems_box[0] + 20, stems_box[1] + 16), "MIDTEMPO COMBAT STEMS // 105 BPM", font=F_HUD, fill=CYAN + (255,))
        d.line([(stems_box[0] + 20, stems_box[1] + 44), (stems_box[2] - 20, stems_box[1] + 44)], fill=CYAN + (70,), width=1)
        d.text((stems_box[0] + 20, stems_box[1] + 58), "▶ PRESET 39: CYBERPUNK 2077 COMBAT LEAD", font=F_SMALL, fill=WHITE + (255,))
        d.text((stems_box[0] + 40, stems_box[1] + 80), "Hard-synced screamer tearing through stereo delay", font=F_TINY, fill=DIM + (255,))
        d.text((stems_box[0] + 20, stems_box[1] + 108), "▶ PRESET 40: REINELEX NEURO REESE", font=F_SMALL, fill=WHITE + (255,))
        d.text((stems_box[0] + 40, stems_box[1] + 130), "Wide detuned stereo Reese bass with chorus & drive", font=F_TINY, fill=DIM + (255,))
        d.text((stems_box[0] + 20, stems_box[1] + 158), "▶ PRESET 41: NIGHT CITY MIDTEMPO CHUG", font=F_SMALL, fill=LIME + (255,))

        # Lower Stats
        stat_b = (48, 700, W - 48, 856)
        draw_cyber_card(img, stat_b, LIME, fill_alpha=220)
        d = ImageDraw.Draw(img)
        d.text((stat_b[0] + 20, stat_b[1] + 16), "COMPLETE DAW HOST AUTOMATION", font=F_HUD, fill=LIME + (255,))
        d.text((stat_b[0] + 20, stat_b[1] + 44), "84 AUTOMATABLE PARAMETERS · REAL-TIME MIDI LEARN · AUTOMATE WAVETABLE POS & FILTER", font=F_SMALL, fill=WHITE + (255,))
        d.text((stat_b[0] + 20, stat_b[1] + 70), "COMPATIBLE WITH ABLETON LIVE, FL STUDIO, REAPER, BITWIG, LOGIC PRO & CUBASE", font=F_TINY, fill=CYAN + (255,))

    # -----------------------------------------------------------------------
    # SCENE 4: BREAKDOWN - 64-STEP SEQUENCER & 3D PHOSPHOR SCOPE (45.71s - 54.86s | Bars 20 - 24)
    # Ethereal Choir + Pad + ZDF Filter Sweep + Accelerating Snare Roll
    # -----------------------------------------------------------------------
    elif t < 54.86:
        section_name = "PATTERN ENGINE // 64-STEP MATRIX & 3D PHOSPHOR SCOPE"
        place_screenshot(img, "seq", (32, 180, 1856, 530), zoom=1.0, focus=(0.5, 0.82), accent=MAGENTA)

        # Active animated step cursor in real-time (16th note = BEAT/4 at 105 BPM)
        step_dur = BEAT / 4.0
        step_idx = int((t - 45.71) / step_dur) % 16
        draw_animated_sequencer_leds(img, (32, 180, 1856, 710), step_idx)

        draw_feature_banner(img, "INTEGRATED 64-STEP NEON MATRIX SEQUENCER", "4 PAGES · PER-STEP NOTE, VELOCITY, SLIDE & ACCENT · SWING · KEY TRANSPOSE", MAGENTA, y_pos=90)

        # Lower 3-column stats
        cw = 586
        for i, (col, title, sub, val) in enumerate([
            (MAGENTA, "MATRIX SEQUENCER", "4 Pages x 16 Steps // Slide & Accent", "64 STEPS"),
            (CYAN, "CYBER FX SUITE", "Distortion · Stereo Chorus · Delay · Reverb", "04 MODULES"),
            (LIME, "FACTORY PATCH BANK", "Tested & Audited by Automated CI", "42 PATCHES"),
        ]):
            bx = 32 + i * (cw + 48)
            card = (bx, 736, bx + cw, 856)
            draw_cyber_card(img, card, col, fill_alpha=220)
            d = ImageDraw.Draw(img)
            d.text((bx + 20, 752), val, font=F_HERO, fill=col + (255,))
            d.text((bx + 20, 804), title, font=F_SMALL, fill=WHITE + (255,))
            d.text((bx + 20, 826), sub, font=F_TINY, fill=DIM + (255,))

    # -----------------------------------------------------------------------
    # SCENE 5: MAXIMUM CLIMAX - FULL ENGINE & DUAL 3D OSCILLATORS OVERDRIVE (54.86s - 61.71s | Bars 24 - 27)
    # -----------------------------------------------------------------------
    elif t < 61.71:
        section_name = "MAXIMUM CLIMAX // FULL ENGINE 3D OVERDRIVE"
        zoom = 1.0 + 0.02 * beat
        place_screenshot(img, "hero_gui", (360, 96, 1200, 850), zoom=zoom, focus=(0.5, 0.5), accent=MAGENTA)

        # Dynamic patch highlight during climax
        c_sub = t - 54.86
        if c_sub < 2.3:
            p_num, p_name, p_cat, p_desc, p_col = "38/42", "ZENETH MIDTEMPO", "BASS", "Heavy neuro growls", CYAN
        elif c_sub < 4.6:
            p_num, p_name, p_cat, p_desc, p_col = "39/42", "COMBAT LEAD", "LEAD", "Hard-synced screamer", AMBER
        else:
            p_num, p_name, p_cat, p_desc, p_col = "40/42", "NEURO REESE", "BASS", "Wide detuned bass", NEON_PURPLE

        draw_active_preset_badge(img, p_num, p_name, p_cat, p_desc, p_col, box=(32, 96, 330, 210))

        # Left lower telemetry
        stat1 = (32, 230, 330, 480)
        draw_cyber_card(img, stat1, p_col, fill_alpha=215)
        d = ImageDraw.Draw(img)
        d.text((stat1[0] + 16, stat1[1] + 14), "FULL ENGINE STATUS", font=F_HUD, fill=p_col + (255,))
        d.line([(stat1[0] + 16, stat1[1] + 40), (stat1[2] - 16, stat1[1] + 40)], fill=p_col + (70,), width=1)
        d.text((stat1[0] + 16, stat1[1] + 54), "10 SYNTH STEMS ACTIVE", font=F_SMALL, fill=WHITE + (255,))
        d.text((stat1[0] + 16, stat1[1] + 78), "16/16 VOICES ENGAGED", font=F_SMALL, fill=LIME + (255,))
        d.text((stat1[0] + 16, stat1[1] + 102), "PEAK LEVEL: -0.5 dBFS", font=F_SMALL, fill=CYAN + (255,))
        d.text((stat1[0] + 16, stat1[1] + 126), "TEMPO: 105 BPM MIDTEMPO", font=F_SMALL, fill=DIM + (255,))

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
        d.text((stat2[0] + 16, stat2[1] + 54), "RMS POWER: 0.367", font=F_SMALL, fill=WHITE + (255,))
        d.text((stat2[0] + 16, stat2[1] + 78), "SIDECHAIN PUMP: ON", font=F_SMALL, fill=LIME + (255,))
        d.text((stat2[0] + 16, stat2[1] + 102), "ANALOG SATURATION: ON", font=F_SMALL, fill=AMBER + (255,))
        d.text((stat2[0] + 16, stat2[1] + 126), "ZERO LATENCY // 105 BPM", font=F_SMALL, fill=CYAN + (255,))

    # -----------------------------------------------------------------------
    # SCENE 6: OUTRO - 3D DUAL HOLOGRAPHIC ROTATION & LINKS (61.71s - 66.0s | Bars 27 - 28)
    # -----------------------------------------------------------------------
    else:
        section_name = "END OF TRANSMISSION // FREE & OPEN SOURCE"
        d = ImageDraw.Draw(img)
        y = 260
        title = "REPLICANTWAVE"
        tw = d.textlength(title, font=F_TITLE)
        text_glow(img, (int(W / 2 - tw / 2), y), title, F_TITLE, CYAN, glow=MAGENTA, radius=18)

        sub = "CYBERNETIC 3D WAVETABLE SYNTHESIZER FOR MIDTEMPO, DARKSYNTH & INDUSTRIAL"
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
    elif 9.05 <= t <= 9.45:
        g = 1.0 * (1.0 - (t - 9.05) / 0.40)
    elif 27.35 <= t <= 27.75:
        g = 0.95 * (1.0 - (t - 27.35) / 0.40)
    elif 54.75 <= t <= 55.25:
        g = 1.1 * (1.0 - (t - 54.75) / 0.50)
    for boundary in (45.71, 61.71):
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
    parser = argparse.ArgumentParser(description="Render ReplicantWave 3D Oscillator Promo Video")
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
        test_times = [2.0, 5.0, 12.0, 18.0, 30.0, 38.0, 48.0, 56.0, 58.0, 63.0]
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
