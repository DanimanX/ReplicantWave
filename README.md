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

Featuring **2 multi-frame wavetable oscillators**, a **64-step pattern sequencer with a sliding navigation drawer** (inspired by *Replicant303*), full **Cytomic ZDF multimode filtering**, a multi-algorithm **Cyber Distortion engine**, modular analog patch cables, real-time FX response monitors, and **16 factory presets** tailored for dark electronic music.

---

## 🚀 Key Features

### 🔊 Wavetable Sound Generation
- **2 Dual Wavetable Oscillators**:
  - Continuous frame position morphing with linear & cubic table interpolation.
  - Up to **8 Unison voices** per oscillator with detune, stereo spread, and precomputed phase gains.
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

### 🎛️ Pure Analog Hardware Rotary Knobs (Zero Sliders)
- **Authentic Potentiometers**: 11 radial graduation marks, metallic knurled chassis, brushed dual-tone dial cap, illuminated neon arc track, high-contrast needle with neon pip, and live formatted parameter values displayed directly underneath.
- Mouse-wheel support, fine-tune precision drag (`Shift` modifier), double-click reset, and contextual logarithmic/linear scaling.
- Modular rack enclosures with corner chassis screws and realistic curved analog patch cables.

### 🔮 3D GPU-Accelerated Animated Holographic Wavetable Engine
- **Orbital 3D Perspective Waterfall Mesh**: Dynamic 3D undulating terrain rendered directly via hardware GPU triangles (`egui::Mesh`) with depth fog, dynamic perspective tilt, and wireframe ribbons.
- **Sweeping Depth Laser Scanner**: Sweeps back and forth through depth, casting glowing highlights on terrain slices.
- **Peak Target Marker & Traveling Cyber Orb**: Real-time laser sweep synchronized to the wavetable frame index and flying cyber energy particle nodes at 60 FPS.

### ⚡ 64-Step Sliding Sequencer (Replicant303 Style)
- **Collapsible Drawer Handle**:
  - Slides open and closed with a click (`[ ⚡ 64-STEP PATTERN SEQUENCER // EXPAND / COLLAPSE ⚡ ]`).
- **Interactive 64-Step Radar Overview Bar**:
  - Miniature timeline showing all 64 steps, gate status, accent, active playhead, and clickable viewport jump bracket.
- **Rotary Dial & Quick-Page Jump**:
  - Smooth rotary dial navigation `VIEW [0..48]` plus quick-page buttons: `[1-16]`, `[17-32]`, `[33-48]`, `[49-64]`.
  - **FOLLOW Mode**: Automatically tracks the active playhead across all 64 steps in real time.
- **DAW & Standalone Seamless Bridging**:
  - **SYNC: DAW**: Automatically locks and triggers with host DAW transport playback in VST3/CLAP/AU.
  - **KEY TRIG**: Incoming MIDI notes transpose the pattern to that key (`root_note = note`) and trigger playback in sync.
  - **FREE RUN**: Internal standalone tempo clock with dedicated `BPM` analog rotary knob (40 to 240 BPM).
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

### 🧪 Cyberpunk Multi-FX Engine with Live Curve Monitors
- **Trance Gate**: 16-step stereo rhythmic chopper with attack/release smoothing.
- **Stereo Chorus**: Dual LFO modulated delay lines with live sine modulation monitor.
- **Cyber Distortion**: Mini real-time saturation curve monitor (*Overdrive*, *Bitcrusher*, *Fire Amp*, *Grind Amp*).
- **Ping-Pong Delay**: Tempo-synced stereo delay with live ping-pong bounce taps monitor.
- **Diffusion Reverb**: Algorithmic stereo plate/hall reverb with tuned comb filters and live diffusion decay monitor.

### 🔋 CPU-Friendly Ultra-Optimized DSP Architecture
- **Eliminated CPU Spikes**: Precalculated envelope decay/release multipliers eliminate transcendental `.exp()` in the sample loop; cached SVF coefficients avoid `tan()` per sample; unison phase gains precalculated outside loops.
- **Zero Audio-Thread Allocations & Lock-Free Scope**: Scopes decoupled from per-sample loop for sub-millisecond block execution.
- **Voice Inactivity Sleeping**: Inactive voices immediately exit the processing loop once their envelopes finish.

---

## 🎚️ Factory Presets (Cyberpunk Soundbank)

ReplicantWave comes loaded with 16 high-voltage cyberpunk presets:

| Preset Name | Category | Characteristics |
| :--- | :--- | :--- |
| **Night City Bass** | Bass | Cyber Neuro table, ZDF 24dB lowpass, aggressive overdrive & stereo chorus. |
| **Arasaka Tower** | Lead | Analog Morph table, 7th interval tuning, bitcrushed delay & long hall reverb. |
| **Combat Zone Lead** | Lead | Digital FM table, high pitch drive, wide chorus & rhythmic trance gate. |
| **Megabuilding Pad** | Pad | Slow attack vocaloid morph, subtle PWM detune & deep cathedral diffusion. |
| **Neon Highway** | Sequence | 16-step rolling acid sequence, 132 BPM, key-tracked resonance squeal. |
| **Chrome Acid 303** | Acid | Cyber Acid wavetable, aggressive portamento slide, snappy accent resonance. |
| **Braindance Sub** | Bass | Sub Zero table, sine sub-oscillator sub-bass reinforcement & softclip. |
| **Cybernetic Choir** | Pad | Vocaloid synth table, multi-frame morphing with stereo ping-pong spatial width. |
| **Blackwall Protocol** | Lead | Dystopian steel table, aggressive fire amp distortion & ping-pong delay. |
| **Neuro Grid Pluck** | Pluck | Fast 5ms ADSR transient, steep 24dB bandpass filter & rhythmic ping-pong. |
| **Netrunner Drone** | Drone | PWM hyper table, unison 8-voice super-detune & endless ambient reverb. |
| **Cyber Glitch 64-Step** | Sequence | Full 64-step fast bitcrushed modular sequence with sliding viewport navigation. |
| **Tactical Scanner** | Sequence | 64-step tactical scanner sweep with rhythmic bandpass filter modulation. |
| **Orbital Space Station** | Pad | Dystopian metallic wavetable with slow 1.2s attack and deep diffusion reverb. |
| **Subsurface 808** | Bass | Heavy sub-bass with soft analog saturation and pitch envelope punch. |
| **Valkyrie Chime** | Lead | Digital FM bell harmonic wavetable with stereo ping-pong delay and wide chorus. |

---

## 🛠️ Building & Packaging from Source

### Prerequisites
- [Rust 1.85+ (MSRV)](https://www.rust-lang.org/tools/install)
- ALSA / JACK / OpenGL dev headers (Linux):
  ```bash
  sudo apt-get install -y libasound2-dev libgl1-mesa-dev libx11-xcb-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libjack-jackd2-dev
  ```

### Build Everything
```bash
# Clone the repository
git clone https://github.com/DanimanX/ReplicantWave.git
cd ReplicantWave

# Build release standalone executable and plugins
cargo xtask bundle replicantwave --release
cargo build --release --bin replicantwave-standalone
```

Compiled bundles will be generated in `target/bundled/`:
- `target/bundled/replicantwave.vst3`
- `target/bundled/replicantwave.clap`
- `target/release/replicantwave-standalone`

---

## 📜 License

ReplicantWave is licensed under the [MIT License](LICENSE).
Created with ⚡ by Daniel Rojas (`daniel.rojas@chileatiende.cl`).
