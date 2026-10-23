# ⚡ ReplicantWave

<div align="center">

![Rust](https://img.shields.io/badge/Language-Rust_1.85+-orange?logo=rust&style=for-the-badge)
![Formats](https://img.shields.io/badge/Formats-VST3_|_CLAP_|_AU_|_Standalone-00f0ff?style=for-the-badge)
![Architecture](https://img.shields.io/badge/DSP-Zero--Allocation_SIMD-39ff14?style=for-the-badge)
![License](https://img.shields.io/badge/License-MIT-ff007f?style=for-the-badge)

<br/><br/>

<img src="assets/replicantwave_gui.png" alt="ReplicantWave Cyberpunk Wavetable Synthesizer" width="920" />

<br/><br/>

[![Donate with PayPal](https://img.shields.io/badge/Donate-PayPal-00457C?logo=paypal&logoColor=white&style=for-the-badge)](https://paypal.me/DanimanX?locale.x=en_US&country.x=CL)

<p align="center">
  <b>A CPU-Friendly Cyberpunk Wavetable Synthesizer with 64-Step Sliding Sequencer</b><br/>
  <i>Built in pure Rust using NIH-plug and egui. Pure cybernetic sound design for Darksynth, Industrial, and Cyberpunk audio production.</i>
</p>

</div>

---

## ☕ Support the Project

If you love **ReplicantWave** and want to support ongoing development, maintenance, and new features:

👉 **[Support / Buy me a coffee on PayPal (paypal.me/DanimanX)](https://paypal.me/DanimanX?locale.x=en_US&country.x=CL)**

There is also a direct **☕ DONATE** button in the top navigation bar of the plugin itself.

---

## 🌌 Overview

**ReplicantWave** is a next-generation Rust-powered clone and reimagining of the Wavetable architecture, infused with a **Cyberpunk 2077 / Neon Noir** aesthetic and engineered for ultra-low CPU utilization.

Featuring **2 multi-frame wavetable oscillators**, a **64-step pattern sequencer with a sliding navigation drawer** (inspired by *Replicant303*), full **Cytomic ZDF multimode filtering**, a multi-algorithm **Cyber Distortion engine**, and **16 factory presets** tailored for dark electronic music.

---

## 🚀 Key Features

### 🔊 Wavetable Sound Generation
- **2 Dual Wavetable Oscillators**:
  - Continuous frame position morphing with linear & cubic table interpolation.
  - Up to **8 Unison voices** per oscillator with detune, stereo spread, and random phase initializers.
  - **Formant Warp** (phase frequency windowing) and **Bend Warp** (non-linear phase curves).
  - Semitone tuning (-36 to +36 st), Fine tuning (-100 to +100 ct), stereo panning, and level gain.
- **Embedded Cyberpunk Wavetable Banks**:
  1. *Analog Morph* (Saw → Triangle → Sine → Square → Pulse)
  2. *Cyber Neuro* (Aggressive folded neuro-bass harmonic morph)
  3. *Digital FM* (Carrier-Modulator dynamic 2-op frequency modulation)
  4. *Vocaloid Synth* (A → E → I → O → U vocal formants)
  5. *Cyber Acid 303* (Resonant harmonic squelch tables)
  6. *Dystopian Steel* (Metallic inharmonic bell to harsh digital grind)
  7. *PWM Hyper* (Continuous pulse-width sweep 50% to 4%)
  8. *Sub Zero* (Deep saturation to wavefolded bass fundamentals)
- **Custom WAV Wavetable Loader**:
  - Drag and drop or browse standard 256, 512, 1024, or 2048-sample WAV files (compatible with Serum, Vital, and SocaLabs Wavetable formats).
- **Sub Oscillator**:
  - 6 selectable waveforms: Sine, Triangle, Saw, Pulse 50%, Pulse 25%, Pulse 12%.
  - Octave shifting (-2, -1, 0) and stereo panning.
- **Noise Generator**:
  - White Noise, Pink Noise (Paul Kellet's filter), and CyberBit (quantized digital artifact noise).

### 🎛️ Cytomic ZDF Multimode Filter
- Zero-Delay Feedback (ZDF) state-variable filter based on Andrew Simper's topology:
  - Modes: **LP 12dB**, **LP 24dB** (cascaded), **HP 12dB**, **HP 24dB**, **BP 12dB**, **BP 24dB**, and **Notch**.
  - Non-linear drive saturation (1.0x to 4.0x).
  - Dedicated Filter ADSR Envelope with bipolar modulation depth (-1.0 to +1.0).
  - Per-source routing matrix: route or bypass OSC 1, OSC 2, Sub, and Noise independently.
  - Key tracking and Velocity tracking.

### 🎛️ Analog Rotary Hardware Controls
- **Analog Rotary Potentiometers**: Hardware-style rotary knobs with illuminated neon angle arcs, metallic dial caps, fine-tune precision drag (`Shift` modifier), double-click reset, and contextual logarithmic/linear scaling.
- Clean 2-row analog synth arrays for OSC 1, OSC 2, ZDF Filter, ADSR Envelopes, and FX modules.

### 🔮 3D GPU Animated Wavetable Engine
- **Live 3D Perspective Waterfall Mesh**: Dynamic 3D undulating terrain rendered directly via GPU with depth fog, dynamic perspective tilt, and wireframe ribbons.
- **Neon Laser Scanline & Particle Orb**: Real-time laser sweep synchronized to the wavetable frame index and flying cyber energy particle nodes.

### ⚡ 64-Step Sliding Sequencer (Replicant303 Style)
- **Collapsible Drawer Handle**:
  - Slides open and closed with a click (`[ ⚡ 64-STEP PATTERN SEQUENCER // EXPAND / COLLAPSE ⚡ ]`).
- **Smooth Viewport Sliding Bar & Auto-Follow**:
  - Continuously slide across all 64 steps smoothly, or jump via quick-page buttons: `[1-16]`, `[17-32]`, `[33-48]`, `[49-64]`.
  - **FOLLOW Mode**: Automatically tracks the active playhead across all 64 steps in real time.
- **DAW & Standalone Seamless Bridging**:
  - **SYNC: DAW**: Automatically locks and triggers with host DAW transport playback in VST3/CLAP/AU.
  - **SYNC: FREE**: Internal standalone tempo clock with dedicated `BPM` analog rotary knob (40 to 240 BPM).
- **Per-Step Hardware Controls**:
  - Gate (On/Off toggle) with live neon LED cursor.
  - Analog Pitch Knob (-24 to +24 semitones with real-time musical note display: C2, D#2, etc.).
  - Analog Velocity Knob (1 to 127).
  - Slide flag (`SLD`): True TB-303 style acid legato glide without retriggering envelopes.
  - Accent flag (`ACC`): Velocity boost with dynamic filter resonance expansion.
- **Sequencer Modes**:
  - *Internal Synth*: Triggers internal synth voices directly with full polyphony management and automatic note-offs.
  - *Cutoff Mod*: Modulates filter cutoff rhythmically.
  - *WT Pos Mod*: Modulates wavetable position rhythmically.
- **Clock & Groove**:
  - Rates: 1/4, 1/8, 1/8T, 1/16, 1/16T, 1/16D, 1/32.
  - Groove Swing (0% to 75% analog rotary knob).
  - 🎲 Randomize (musical pentatonic/minor cyberpunk intervals) and ✖ Clear utilities.

### 🧪 Cyberpunk Multi-FX Engine
- **Trance Gate**: 16-step stereo rhythmic chopper with attack/release smoothing.
- **Stereo Chorus**: Dual LFO modulated delay lines with phase width spread.
- **Cyber Distortion**:
  - *Overdrive* (smooth hyperbolic softclip).
  - *Bitcrusher* (sample rate decimation & bit quantization).
  - *Fire Amp* (asymmetric warm saturation).
  - *Grind Amp* (wavefolding fuzz).
- **Ping-Pong Delay**: Tempo-synced (or ms) stereo cross-feedback delay with damping filter.
- **Diffusion Reverb**: Algorithmic stereo plate/hall reverb with tuned comb filters, cached coefficients, and allpass diffusers.

### 🔋 CPU-Friendly Ultra-Optimized DSP Architecture
- **Eliminated CPU Spikes**: Bitmasked power-of-two table lookups, fast polynomial SVF filter coefficient calculations, and hardware-accelerated `exp2()` note-to-frequency conversions.
- **Zero Audio-Thread Allocations**: Everything is pre-allocated at initialization.
- **Voice Inactivity Sleeping**: Inactive voices immediately exit the processing loop once their envelopes finish.
- **Voice Stealing Algorithm**: Smart allocation reuses the oldest or lowest-volume releasing voice without audio clicks.

---

## 🎚️ Factory Presets (Cyberpunk Soundbank)

1. **Night City Bass**: Heavy rolling neuro-bass with resonant filter drive and tape crunch.
2. **Arasaka Matrix Pad**: Lush dystopian corporate soundscape with stereo chorus and shimmering reverb.
3. **Neon Blade 2049**: Heroic Vangelis-inspired brass/saw lead soaring above rainy neon towers.
4. **Chrome Acid 303**: Iconic screaming resonant TB-style acid squelch with portamento slides.
5. **Netrunner Arp**: Bouncing 16th-note FM plucks cascading into a stereo cyber-delay.
6. **Darksynth Power Stab**: Aggressive 8-voice supersaw power stab for hard electronic drops.
7. **Corpo Tower Dystopia**: Dark metallic industrial drone with notch sweep and infinite diffusion.
8. **Megacity Combat 2077**: Relentless mid-tempo cyber-industrial bass with wavefolded fuzz and pump gate.
9. **Ghost In The Wire Choir**: Ethereal vocal formant choir floating through high-density diffusion.
10. **Subdermal 808 Rumble**: Chest-rattling 808 sub bass with saturated low harmonics.
11. **Synthetic Tears Pad**: Melancholic CS-80 inspired synthwave brass pad soaked in tape delay.
12. **Cyber Glitch 64-Step**: 64-step fast bitcrushed modular sequence using sliding bar navigation.
13. **Kiroshi Scanner Sequence**: Tactical cybernetic scanner audio sweep with 64-step bandpass modulation.
14. **Overclocked Sync Screamer**: Hard-synced screamer lead that cuts cleanly through dense cyberpunk mixes.
15. **Afterlife Future Rave**: Underground Afterlife club pumping bass/pad with trance gate rhythm.
16. **Braindance Memory**: Nostalgic digital artifacts and shimmering space echoing in virtual memory.

---

## 📦 Supported Formats

| Format | Platform | Description |
|---|---|---|
| **VST3** | Linux, Windows, macOS | Standard Steinberg VST3 plugin |
| **CLAP** | Linux, Windows, macOS | Modern Clever Audio Plugin with polyphonic modulation |
| **AU (AudioUnit v2/v3)** | macOS | Apple AudioUnit format for Logic Pro, GarageBand |
| **Standalone** | Linux, Windows, macOS | Standalone executable (ALSA, JACK, CoreAudio, WASAPI) |

---

## 🛠️ Building From Source

### Prerequisites
- [Rust](https://rustup.rs/) (version 1.85 or newer).
- Linux audio libraries (for Linux build):
  ```bash
  sudo apt-get install -y libasound2-dev libgl1-mesa-dev libx11-xcb-dev
  ```

### Build VST3 & CLAP
```bash
cargo xtask bundle replicantwave --release
```
The bundles will be generated in `target/bundled/`:
- `target/bundled/replicantwave.clap`
- `target/bundled/replicantwave.vst3`

### Build Standalone Executable
```bash
cargo build --release --bin replicantwave-standalone
```
Binary located at: `target/release/replicantwave-standalone`

### Build macOS AudioUnit (AU)
On macOS (or with `clap-wrapper`):
```bash
bash tools/build_au.sh
```

### Build All Formats Into Release Package
```bash
bash tools/package_all.sh
```

---

## 📜 License

MIT License - Copyright (c) 2026 Daniel Rojas (DanimanX). See [LICENSE](LICENSE) for details.
