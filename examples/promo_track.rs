//! Offline promo-track renderer for ReplicantWave.
//!
//! Generates a crushing, authentic Cyberpunk / Darkwave cue (120 BPM in D minor)
//! using the REAL ReplicantWave DSP engine and factory presets:
//! - "Neon Blade 2049" (Vangelis Blade Runner CS-80 lead)
//! - "Nightside Razor Lead" (Screaming Darksynth sync lead)
//! - "Neuro Reaper Bass" (Distorted wavetable growl bass)
//! - "Chrome Acid 303" (Resonant 303 acid line)
//! - "Netrunner Arp" (16th cyber matrix arpeggios)
//! - "Darksynth Power Stab" (Heavy Carpenter Brut analog brass hits)
//! - "Ghost Protocol Choir" (Ethereal cyberpunk vocaloid choir)
//! - "Synthetic Tears Pad" (Lush, melancholic analog pad)
//! - "Blackwall Drone" (Deep dystopian sub drone)
//!
//! Mastered to 48 kHz stereo 16-bit WAV for video compositing.

use replicantwave::audio::SynthEngine;
use replicantwave::presets::factory::get_factory_presets;
use replicantwave::presets::SynthPreset;
use std::f32::consts::TAU;
use std::path::PathBuf;

pub const SR: f32 = 48_000.0;
pub const BPM: f32 = 120.0;
pub const BEAT: f32 = 60.0 / BPM; // 0.50 s
pub const BAR: f32 = BEAT * 4.0;  // 2.00 s
pub const BARS: usize = 32;       // 64.0 s
pub const TAIL: f32 = 2.0;
pub const LEN: f32 = BAR * BARS as f32 + TAIL; // 66.0 s

#[derive(Clone, Copy, Debug)]
pub struct Note {
    pub time: f32,
    pub note: u8,
    pub vel: f32,
    pub dur: f32,
}

pub fn bar_time(bar: usize) -> f32 {
    bar as f32 * BAR
}

pub fn beat_time(bar: usize, beat: f32) -> f32 {
    bar_time(bar) + beat * BEAT
}

pub fn find_preset(name: &str) -> SynthPreset {
    get_factory_presets()
        .into_iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("factory preset '{name}' not found"))
}

/// Render one synth stem from a note list using the plugin's own engine.
pub fn render_stem(preset: &SynthPreset, notes: &[Note]) -> Vec<f32> {
    let total = (LEN * SR) as usize;
    let mut out = vec![0.0f32; total * 2];

    let mut engine = SynthEngine::default();
    engine.set_sample_rate(SR);
    preset.apply_to_engine(&mut engine);
    engine.sequencer.running = false;

    let mut notes: Vec<Note> = notes.to_vec();
    notes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

    let mut cursor = 0usize;
    let mut l = vec![0.0f32; 256];
    let mut r = vec![0.0f32; 256];

    let mut render_until = |engine: &mut SynthEngine, out: &mut Vec<f32>, from: usize, to: usize| {
        let mut pos = from;
        while pos < to {
            let n = (to - pos).min(256);
            engine.process_block(&mut l[..n], &mut r[..n], BPM, false);
            for i in 0..n {
                out[(pos + i) * 2] += l[i];
                out[(pos + i) * 2 + 1] += r[i];
            }
            pos += n;
        }
    };

    for note in notes {
        let at = ((note.time * SR) as usize).min(total);
        if at > cursor {
            render_until(&mut engine, &mut out, cursor, at);
            cursor = at;
        }
        engine.note_on(note.note, note.vel);
        let off = (((note.time + note.dur) * SR) as usize).min(total);
        if off > cursor {
            render_until(&mut engine, &mut out, cursor, off);
            cursor = off;
        }
        engine.note_off(note.note);
    }
    if cursor < total {
        render_until(&mut engine, &mut out, cursor, total);
    }

    out
}

// ---------------------------------------------------------------------------
// High-Octane Darkwave Drum & Impact Synthesizer
// ---------------------------------------------------------------------------

pub struct Rng(u32);
impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(seed | 1)
    }
    pub fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Punchy analog/darkwave kick: transient click + saturated 120 -> 48 Hz pitch drop.
pub fn add_kick(buf: &mut [f32], t0: f32, gain: f32) {
    let start = (t0 * SR) as usize;
    let len = (0.38 * SR) as usize;
    for i in 0..len {
        let idx = start + i;
        if idx * 2 + 1 >= buf.len() {
            break;
        }
        let t = i as f32 / SR;
        // Pitch envelope: 130 Hz dropping to 48 Hz.
        let f = 48.0 + 82.0 * (-t * 38.0).exp();
        let env = (-t * 8.2).exp();
        let body = (TAU * f * t).sin() * env;
        // Hard transient click.
        let click = (-t * 550.0).exp() * (TAU * 850.0 * t).sin() * 0.45;
        let raw = (body * 1.1 + click) * gain;
        // Soft tube saturation.
        let s = (raw * 1.3).tanh();
        buf[idx * 2] += s;
        buf[idx * 2 + 1] += s;
    }
}

/// Gated 80s Cyberpunk snare: punchy body + gated stereo noise.
pub fn add_snare(buf: &mut [f32], t0: f32, gain: f32, rng: &mut Rng) {
    let start = (t0 * SR) as usize;
    let len = (0.28 * SR) as usize;
    let mut hp = 0.0f32;
    let mut prev = 0.0f32;
    for i in 0..len {
        let idx = start + i;
        if idx * 2 + 1 >= buf.len() {
            break;
        }
        let t = i as f32 / SR;
        let n = rng.next();
        hp = 0.85 * (hp + n - prev);
        prev = n;
        // Gated envelope: rapid release after 0.18s
        let noise_env = if t < 0.16 { (-t * 9.0).exp() } else { (-t * 30.0).exp() };
        let tone = (TAU * 195.0 * t).sin() * (-t * 24.0).exp() * 0.65;
        let raw = (hp * noise_env * 0.95 + tone) * gain;
        let s = (raw * 1.25).tanh();
        buf[idx * 2] += s * 0.96;
        buf[idx * 2 + 1] += s * 1.04;
    }
}

/// Cyber stereo clap.
pub fn add_clap(buf: &mut [f32], t0: f32, gain: f32, rng: &mut Rng) {
    for (k, offset) in [0.0f32, 0.010, 0.022].iter().enumerate() {
        let start = ((t0 + offset) * SR) as usize;
        let len = (0.18 * SR) as usize;
        let mut hp = 0.0f32;
        let mut prev = 0.0f32;
        for i in 0..len {
            let idx = start + i;
            if idx * 2 + 1 >= buf.len() {
                break;
            }
            let t = i as f32 / SR;
            let n = rng.next();
            hp = 0.88 * (hp + n - prev);
            prev = n;
            let env = (-t * 22.0).exp() * (1.0 - k as f32 * 0.20);
            let s = hp * env * gain * 0.65;
            buf[idx * 2] += s * 1.05;
            buf[idx * 2 + 1] += s * 0.95;
        }
    }
}

/// Sizzling closed or open hi-hat.
pub fn add_hat(buf: &mut [f32], t0: f32, gain: f32, open: bool, rng: &mut Rng) {
    let start = (t0 * SR) as usize;
    let len = ((if open { 0.24 } else { 0.055 }) * SR) as usize;
    let mut prev = 0.0f32;
    let mut hp = 0.0f32;
    for i in 0..len {
        let idx = start + i;
        if idx * 2 + 1 >= buf.len() {
            break;
        }
        let t = i as f32 / SR;
        let n = rng.next();
        hp = 0.92 * (hp + n - prev);
        prev = n;
        let env = (-t * if open { 14.0 } else { 65.0 }).exp();
        let s = hp * env * gain * 0.7;
        let pan = if i % 2 == 0 { 0.8 } else { 1.2 };
        buf[idx * 2] += s * pan;
        buf[idx * 2 + 1] += s * (2.0 - pan);
    }
}

/// Sweeping cyber riser with rising tone and opening lowpass.
pub fn add_riser(buf: &mut [f32], t0: f32, dur: f32, gain: f32, rng: &mut Rng) {
    let start = (t0 * SR) as usize;
    let len = (dur * SR) as usize;
    let mut lp = 0.0f32;
    for i in 0..len {
        let idx = start + i;
        if idx * 2 + 1 >= buf.len() {
            break;
        }
        let t = i as f32 / SR;
        let u = (t / dur).clamp(0.0, 1.0);
        let coeff = 0.003 + 0.15 * u * u;
        lp += coeff * (rng.next() - lp);
        let env = u.powf(2.0);
        let f_rise = 80.0 * (5.5f32).powf(u);
        let tone = (TAU * f_rise * t).sin() * 0.16 * env;
        let s = (lp * 2.2 + tone) * env * gain;
        buf[idx * 2] += s * (0.85 + 0.15 * u);
        buf[idx * 2 + 1] += s * (1.15 - 0.15 * u);
    }
}

/// Heavy cinematic sub impact / crash drop.
pub fn add_impact(buf: &mut [f32], t0: f32, gain: f32) {
    let start = (t0 * SR) as usize;
    let len = (2.2 * SR) as usize;
    let mut lp = 0.0f32;
    let mut rng = Rng::new(0x2077_0001);
    for i in 0..len {
        let idx = start + i;
        if idx * 2 + 1 >= buf.len() {
            break;
        }
        let t = i as f32 / SR;
        let sub_pitch = (TAU * (62.0 - 24.0 * (t / 1.5).min(1.0)) * t).sin();
        let sub_boom = sub_pitch * (-t * 2.2).exp();
        lp += 0.06 * (rng.next() - lp);
        let crash = lp * (-t * 3.0).exp() * 1.5;
        let s = (sub_boom * 1.1 + crash) * gain;
        buf[idx * 2] += s;
        buf[idx * 2 + 1] += s;
    }
}

// ---------------------------------------------------------------------------
// Darkwave Musical Composition in D Minor
// Chords: Dm (D,F,A), Bb (Bb,D,F), Gm (G,Bb,D), A (A,C#,E)
// ---------------------------------------------------------------------------

/// Root notes in MIDI: D1 (38), Bb0 (34), G0 (31), A0 (33).
const ROOTS: [u8; 4] = [38, 34, 31, 33];

/// Triad semitone intervals from root.
const TRIADS: [[i8; 3]; 4] = [
    [0, 3, 7], // Dm
    [0, 4, 7], // Bb
    [0, 3, 7], // Gm
    [0, 4, 7], // A
];

pub fn chord(bar: usize) -> [u8; 3] {
    let c = bar % 4;
    let root = ROOTS[c];
    let t = TRIADS[c];
    [
        (root as i16 + t[0] as i16) as u8,
        (root as i16 + t[1] as i16) as u8,
        (root as i16 + t[2] as i16) as u8,
    ]
}

/// Driving 16th-note Darkwave bassline with accented octave jumps and slides.
pub fn bass_line(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let root = ROOTS[bar % 4] as i16; // D1 etc.
        // Relentless 16th darkwave chug with octave jumps and syncopations.
        let pattern: [(f32, i16, f32, f32); 16] = [
            (0.00,  0, 1.00, 0.18),
            (0.25,  0, 0.85, 0.14),
            (0.50, 12, 0.95, 0.18),
            (0.75,  0, 0.80, 0.14),
            (1.00,  0, 0.95, 0.16),
            (1.25, 12, 0.90, 0.16),
            (1.50,  3, 0.95, 0.18), // Minor third pop
            (1.75,  0, 0.80, 0.14),
            (2.00,  0, 1.00, 0.18),
            (2.25,  0, 0.85, 0.14),
            (2.50, 12, 0.95, 0.18),
            (2.75,  0, 0.80, 0.14),
            (3.00, 12, 0.95, 0.16),
            (3.25,  7, 0.90, 0.16), // Fifth pop
            (3.50,  1, 0.95, 0.18), // Half-step slide
            (3.75, 12, 0.85, 0.14),
        ];
        for (beat, semi, vel, dur) in pattern {
            out.push(Note {
                time: beat_time(bar, beat),
                note: (root + semi).clamp(0, 127) as u8,
                vel,
                dur,
            });
        }
    }
    out
}

/// High-speed 16th-note cyber matrix arpeggios ("Netrunner Arp").
pub fn arp_line(bars: std::ops::Range<usize>, octave: i16) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let c = chord(bar);
        let tones = [
            c[0] as i16 + octave,
            c[1] as i16 + octave,
            c[2] as i16 + octave,
            c[0] as i16 + octave + 12,
            c[1] as i16 + octave + 12,
            c[2] as i16 + octave + 12,
            c[0] as i16 + octave + 24,
            c[2] as i16 + octave + 12,
        ];
        for step in 0..16 {
            let idx = (step * 3) % tones.len();
            out.push(Note {
                time: beat_time(bar, step as f32 * 0.25),
                note: tones[idx].clamp(0, 127) as u8,
                vel: if step % 4 == 0 { 0.95 } else if step % 2 == 0 { 0.85 } else { 0.70 },
                dur: 0.14,
            });
        }
    }
    out
}

/// Lush, melancholic pad progression ("Synthetic Tears Pad").
pub fn pad_line(bars: std::ops::Range<usize>, octave: i16, vel: f32) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let c = chord(bar);
        for tone in c {
            out.push(Note {
                time: bar_time(bar),
                note: (tone as i16 + octave + 12).clamp(0, 127) as u8,
                vel,
                dur: BAR * 0.98,
            });
        }
        // Stereo octave spread
        out.push(Note {
            time: bar_time(bar),
            note: (c[0] as i16 + octave + 24).clamp(0, 127) as u8,
            vel: vel * 0.75,
            dur: BAR * 0.98,
        });
    }
    out
}

/// Vangelis Blade Runner CS-80 lead melody ("Neon Blade 2049") for Intro & Outro.
pub fn blade_runner_lead(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    // D4 = 62, E4 = 64, F4 = 65, G4 = 67, A4 = 69, C#4 = 61
    let phrases: [(usize, f32, u8, f32, f32); 10] = [
        (0, 0.0, 62, 0.95, 1.8), // D4
        (0, 3.5, 64, 0.85, 0.4), // E4
        (1, 0.0, 65, 0.95, 1.6), // F4
        (1, 3.0, 69, 0.90, 0.8), // A4
        (2, 0.0, 67, 0.95, 1.4), // G4
        (2, 2.5, 65, 0.85, 0.6), // F4
        (2, 3.5, 64, 0.85, 0.6), // E4
        (3, 0.0, 62, 0.90, 1.2), // D4
        (3, 2.0, 61, 0.85, 0.8), // C#4
        (3, 3.0, 62, 0.95, 1.4), // D4
    ];
    for bar in bars.clone() {
        let local_bar = bar - bars.start;
        for (b, beat, note, vel, dur) in phrases {
            if b == (local_bar % 4) {
                out.push(Note {
                    time: bar_time(bar) + beat * BEAT,
                    note,
                    vel,
                    dur: dur * BEAT * 2.0,
                });
            }
        }
    }
    out
}

/// Screaming Darksynth Razor Lead ("Nightside Razor Lead").
/// Piercing, high-octane cyberpunk melody over the drops and climax.
pub fn razor_lead(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    // 2-bar motif in D minor (Octave 5: D5 = 74)
    let motif_a: [(f32, u8, f32, f32); 11] = [
        (0.00, 74, 1.00, 0.55), // D5
        (1.00, 77, 0.90, 0.38), // F5
        (1.75, 76, 0.85, 0.30), // E5
        (2.50, 74, 0.95, 0.55), // D5
        (3.50, 69, 0.85, 0.35), // A4
        (4.00, 70, 1.00, 0.55), // Bb4
        (5.00, 72, 0.90, 0.38), // C5
        (5.75, 74, 0.95, 0.45), // D5
        (6.75, 72, 0.85, 0.30), // C5
        (7.25, 70, 0.90, 0.55), // Bb4
        (7.75, 72, 0.85, 0.25), // C5
    ];
    let motif_b: [(f32, u8, f32, f32); 11] = [
        (0.00, 67, 1.00, 0.55), // G4
        (1.00, 70, 0.90, 0.38), // Bb4
        (1.75, 74, 0.95, 0.45), // D5
        (2.50, 72, 0.90, 0.38), // C5
        (3.25, 70, 0.85, 0.45), // Bb4
        (4.00, 69, 1.00, 0.55), // A4
        (5.00, 73, 0.90, 0.38), // C#5
        (5.75, 76, 1.00, 0.55), // E5
        (6.75, 74, 0.90, 0.35), // D5
        (7.25, 73, 0.95, 0.65), // C#5
        (7.75, 74, 1.00, 0.25), // D5
    ];

    for bar in bars.step_by(4) {
        // Part 1: Motif A (2 bars)
        for (beat, note, vel, dur) in motif_a {
            let t = bar_time(bar) + beat * BEAT;
            if t < LEN {
                out.push(Note { time: t, note, vel, dur: dur * BEAT * 1.5 });
            }
        }
        // Part 2: Motif B (2 bars)
        for (beat, note, vel, dur) in motif_b {
            let t = bar_time(bar + 2) + beat * BEAT;
            if t < LEN {
                out.push(Note { time: t, note, vel, dur: dur * BEAT * 1.5 });
            }
        }
    }
    out
}

/// Squelching 303 Acid Bassline ("Chrome Acid 303").
pub fn acid_303_line(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let root = (ROOTS[bar % 4] as i16 + 12).clamp(0, 127) as u8; // D2 etc.
        let pattern: [(f32, i8, f32, f32); 16] = [
            (0.00,  0, 0.95, 0.16),
            (0.25,  0, 0.70, 0.12),
            (0.50, 12, 1.00, 0.22), // Accent octave pop
            (0.75,  3, 0.80, 0.14),
            (1.00,  0, 0.90, 0.14),
            (1.25,  7, 0.75, 0.12),
            (1.50,  3, 0.95, 0.18),
            (1.75, 12, 1.00, 0.24), // Accent slide
            (2.00,  0, 0.90, 0.14),
            (2.25, 10, 0.85, 0.14),
            (2.50, 12, 1.00, 0.20),
            (2.75,  7, 0.80, 0.14),
            (3.00,  3, 0.90, 0.16),
            (3.25,  5, 0.85, 0.14),
            (3.50,  6, 0.95, 0.18), // Acid tritone slide
            (3.75,  7, 1.00, 0.20),
        ];
        for (beat, semi, vel, dur) in pattern {
            out.push(Note {
                time: beat_time(bar, beat),
                note: ((root as i16 + semi as i16).clamp(0, 127)) as u8,
                vel,
                dur,
            });
        }
    }
    out
}

/// Heavy French Touch / Carpenter Brut analog brass power stabs ("Darksynth Power Stab").
pub fn power_stabs(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let c = chord(bar);
        // Stabs on beats: 0.0, 1.75, 2.5, 3.5
        for &beat in &[0.0f32, 1.75, 2.5, 3.5] {
            for &tone in &c {
                out.push(Note {
                    time: beat_time(bar, beat),
                    note: (tone as i16 + 12).clamp(0, 127) as u8,
                    vel: 1.0,
                    dur: 0.22,
                });
                out.push(Note {
                    time: beat_time(bar, beat),
                    note: (tone as i16 + 24).clamp(0, 127) as u8,
                    vel: 0.85,
                    dur: 0.22,
                });
            }
        }
    }
    out
}

/// Complete Darkwave Drum & Percussion Track.
pub fn drums(rng: &mut Rng) -> Vec<f32> {
    let mut buf = vec![0.0f32; ((LEN * SR) as usize) * 2];

    for bar in 0..BARS {
        let is_intro = bar < 4;
        let is_breakdown = (20..24).contains(&bar);
        let is_drop = (4..20).contains(&bar) || (24..30).contains(&bar);
        let is_climax = (24..30).contains(&bar);

        if is_intro {
            // Intro heartbeats: soft pulses on bar 1 and 3 downbeats
            if bar == 1 || bar == 3 {
                add_kick(&mut buf, bar_time(bar), 0.6);
            }
            continue;
        }

        if is_breakdown {
            // Half-time kick pulse on downbeats
            add_kick(&mut buf, bar_time(bar), 0.85);
            add_kick(&mut buf, beat_time(bar, 2.0), 0.75);
            // Accelerating snare fill in bar 23 before the climax drop!
            if bar == 23 {
                for k in 0..8 {
                    add_snare(&mut buf, beat_time(bar, 2.0 + k as f32 * 0.25), 0.45 + k as f32 * 0.07, rng);
                }
            }
            continue;
        }

        if is_drop {
            // Four-on-the-floor heavy kick
            for beat in 0..4 {
                let g = if beat == 0 { 1.05 } else { 0.95 };
                add_kick(&mut buf, beat_time(bar, beat as f32), g);
            }
            // Syncopated ghost kick for darkwave bounce
            if bar % 2 == 1 {
                add_kick(&mut buf, beat_time(bar, 2.75), 0.7);
            }

            // Gated Snare on beats 1.0 and 3.0 (i.e. beats 2 and 4 of 4/4)
            add_snare(&mut buf, beat_time(bar, 1.0), 0.75, rng);
            add_snare(&mut buf, beat_time(bar, 3.0), 0.80, rng);
            // Layer stereo clap
            add_clap(&mut buf, beat_time(bar, 1.0), 0.55, rng);
            add_clap(&mut buf, beat_time(bar, 3.0), 0.60, rng);

            // Driving 16th-note Hi-Hats with sizzling open hats on offbeats (&)
            for step in 0..16 {
                let t = beat_time(bar, step as f32 * 0.25);
                let open = step % 4 == 2; // off-beats: 0.5, 1.5, 2.5, 3.5
                let g = if open {
                    0.28
                } else if step % 2 == 0 {
                    0.20
                } else {
                    0.13
                };
                add_hat(&mut buf, t, g, open, rng);
            }

            // Snare roll fills before transitions
            if bar == 11 || bar == 19 || bar == 29 {
                for k in 0..6 {
                    add_snare(&mut buf, beat_time(bar, 2.5 + k as f32 * 0.25), 0.5 + k as f32 * 0.08, rng);
                }
            }
            if is_climax && bar % 2 == 0 {
                // Extra crash accent on climax downbeats
                add_impact(&mut buf, bar_time(bar), 0.6);
            }
        }
    }

    // Risers and Impacts at structural milestones
    add_riser(&mut buf, bar_time(2) + BEAT * 2.0, BAR * 1.5, 0.65, rng);
    add_impact(&mut buf, bar_time(4), 0.95); // DROP 1

    add_riser(&mut buf, bar_time(10) + BEAT * 2.0, BAR * 1.5, 0.70, rng);
    add_impact(&mut buf, bar_time(12), 1.00); // DROP 2 (Razor Lead + Acid)

    add_riser(&mut buf, bar_time(22), BAR * 2.0, 0.85, rng); // BREAKDOWN RISER
    add_impact(&mut buf, bar_time(24), 1.10); // CLIMAX DROP

    add_impact(&mut buf, bar_time(30), 0.80); // OUTRO RESOLUTION

    buf
}

/// Pure deep sine sub-bass (40-60 Hz) following the bassline roots.
pub fn sub_stem(notes: &[Note]) -> Vec<f32> {
    let total = (LEN * SR) as usize;
    let mut out = vec![0.0f32; total * 2];
    for note in notes {
        let f = 440.0 * 2.0f32.powf((note.note as f32 - 69.0) / 12.0);
        let start = (note.time * SR) as usize;
        let len = ((note.dur + 0.05) * SR) as usize;
        let mut phase = 0.0f32;
        for i in 0..len {
            let idx = start + i;
            if idx >= total {
                break;
            }
            let t = i as f32 / SR;
            let attack = (t / 0.008).min(1.0);
            let release = ((note.dur + 0.05 - t) / 0.05).clamp(0.0, 1.0);
            let env = attack * release * note.vel;
            let s = (phase * TAU).sin() * env;
            out[idx * 2] += s;
            out[idx * 2 + 1] += s;
            phase = (phase + f / SR).fract();
        }
    }
    out
}

pub fn lowpass(x: &mut [f32], hz: f32) {
    let coeff = 1.0 - (-TAU * hz / SR).exp();
    let mut a_l = 0.0f32;
    let mut a_r = 0.0f32;
    let mut b_l = 0.0f32;
    let mut b_r = 0.0f32;
    for i in 0..x.len() / 2 {
        a_l += coeff * (x[i * 2] - a_l);
        a_r += coeff * (x[i * 2 + 1] - a_r);
        b_l += coeff * (a_l - b_l);
        b_r += coeff * (a_r - b_r);
        x[i * 2] = b_l;
        x[i * 2 + 1] = b_r;
    }
}

pub fn highpass(x: &mut [f32], hz: f32) {
    let coeff = 1.0 - (-TAU * hz / SR).exp();
    let mut lp_l = 0.0f32;
    let mut lp_r = 0.0f32;
    for i in 0..x.len() / 2 {
        lp_l += coeff * (x[i * 2] - lp_l);
        lp_r += coeff * (x[i * 2 + 1] - lp_r);
        x[i * 2] -= lp_l;
        x[i * 2 + 1] -= lp_r;
    }
}

pub fn sidechain(total: &mut [f32], amount: f32, release: f32) {
    let mut gain = 1.0f32;
    let total_samples = total.len() / 2;
    let coeff = 1.0 - (-1.0 / (release * SR)).exp();
    for i in 0..total_samples {
        let t = i as f32 / SR;
        let beat_pos = (t / BEAT).fract();
        if beat_pos < 0.025 {
            gain = 1.0 - amount;
        } else {
            gain += (1.0 - gain) * coeff;
        }
        let duck = if t < bar_time(4) { 1.0 } else { gain };
        total[i * 2] *= duck;
        total[i * 2 + 1] *= duck;
    }
}

fn main() {
    let out_dir = PathBuf::from("target/promo");
    std::fs::create_dir_all(&out_dir).expect("create target/promo");

    println!("Rendering stems with the ReplicantWave engine at {BPM} BPM, {BARS} bars ({LEN:.1}s)...");

    let (vangelis_lead, razor, bass, arp, pad, acid, stabs, choir, drone) =
        std::thread::scope(|s| {
            let vangelis_h = s.spawn(|| {
                let p = find_preset("Neon Blade 2049");
                let mut notes = blade_runner_lead(0..4);
                notes.extend(blade_runner_lead(30..32));
                render_stem(&p, &notes)
            });
            let razor_h = s.spawn(|| {
                let p = find_preset("Nightside Razor Lead");
                let mut notes = razor_lead(12..20);
                notes.extend(razor_lead(24..30));
                render_stem(&p, &notes)
            });
            let bass_h = s.spawn(|| {
                let p = find_preset("Neuro Reaper Bass");
                render_stem(&p, &bass_line(4..30))
            });
            let arp_h = s.spawn(|| {
                let p = find_preset("Netrunner Arp");
                let mut notes = arp_line(4..20, 12);
                notes.extend(arp_line(24..30, 12));
                render_stem(&p, &notes)
            });
            let pad_h = s.spawn(|| {
                let p = find_preset("Synthetic Tears Pad");
                render_stem(&p, &pad_line(0..32, 0, 0.65))
            });
            let acid_h = s.spawn(|| {
                let p = find_preset("Chrome Acid 303");
                let mut notes = acid_303_line(12..20);
                notes.extend(acid_303_line(24..30));
                render_stem(&p, &notes)
            });
            let stabs_h = s.spawn(|| {
                let p = find_preset("Darksynth Power Stab");
                let mut notes = power_stabs(8..20);
                notes.extend(power_stabs(24..30));
                render_stem(&p, &notes)
            });
            let choir_h = s.spawn(|| {
                let p = find_preset("Ghost Protocol Choir");
                let mut notes = pad_line(12..24, 12, 0.55);
                notes.extend(pad_line(24..30, 12, 0.60));
                render_stem(&p, &notes)
            });
            let drone_h = s.spawn(|| {
                let p = find_preset("Blackwall Drone");
                let notes = vec![
                    Note { time: 0.0, note: 38, vel: 0.8, dur: BAR * 4.0 },
                    Note { time: bar_time(20), note: 38, vel: 0.7, dur: BAR * 4.0 },
                    Note { time: bar_time(30), note: 38, vel: 0.75, dur: BAR * 2.0 },
                ];
                render_stem(&p, &notes)
            });
            (
                vangelis_h.join().unwrap(),
                razor_h.join().unwrap(),
                bass_h.join().unwrap(),
                arp_h.join().unwrap(),
                pad_h.join().unwrap(),
                acid_h.join().unwrap(),
                stabs_h.join().unwrap(),
                choir_h.join().unwrap(),
                drone_h.join().unwrap(),
            )
        });

    println!("Rendering drums and sub-bass...");
    let bass_notes = bass_line(4..30);
    let mut sub = sub_stem(&bass_notes);
    lowpass(&mut sub, 95.0);

    let mut rng = Rng::new(0x2077_D41C);
    let mut drum = drums(&mut rng);

    // Warm the top of the drums
    let mut drum_soft = drum.clone();
    lowpass(&mut drum_soft, 9500.0);
    for i in 0..drum.len() {
        drum[i] = drum[i] * 0.30 + drum_soft[i] * 0.70;
    }

    // Tame neuro bass sizzle slightly so lead shines through
    let mut bass_clean = bass.clone();
    lowpass(&mut bass_clean, 3600.0);
    for i in 0..bass_clean.len() {
        bass_clean[i] = bass_clean[i] * 0.75 + bass[i] * 0.25;
    }

    // Save diagnostic stems if PROMO_STEMS is set
    for (name, stem) in [
        ("vangelis", &vangelis_lead),
        ("razor", &razor),
        ("bass", &bass_clean),
        ("sub", &sub),
        ("arp", &arp),
        ("pad", &pad),
        ("acid", &acid),
        ("stabs", &stabs),
        ("choir", &choir),
        ("drone", &drone),
        ("drums", &drum),
    ] {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: SR as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let path = out_dir.join(format!("stem_{name}.wav"));
        let mut w = hound::WavWriter::create(&path, spec).expect("stem wav");
        for s in stem.iter() {
            w.write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).ok();
        }
        w.finalize().ok();
    }
    println!("Stems written to {}", out_dir.display());

    // Mixdown & Gain Staging
    let total = bass.len();
    let mut mix = vec![0.0f32; total];
    let stems: [(&Vec<f32>, f32); 10] = [
        (&vangelis_lead, 0.45),
        (&razor,         0.48),
        (&bass_clean,    0.42),
        (&sub,           0.58),
        (&arp,           0.28),
        (&pad,           0.65),
        (&acid,          0.44),
        (&stabs,         0.38),
        (&choir,         0.32),
        (&drone,         0.40),
    ];
    for (stem, gain) in stems {
        for i in 0..total {
            mix[i] += stem[i] * gain;
        }
    }

    // Drums mix
    for i in 0..total {
        mix[i] += drum[i] * 0.95;
    }

    // Sidechain pumping on the melodic/harmonic bus
    let mut pumped = mix.clone();
    sidechain(&mut pumped, 0.38, 0.12);
    for i in 0..total {
        mix[i] = pumped[i] * 0.82 + mix[i] * 0.18;
    }

    // Dynamic section curve (breakdown breathes, drops hit massive)
    let n_samples = mix.len() / 2;
    for i in 0..n_samples {
        let t = i as f32 / SR;
        let bar = t / BAR;
        let g = if bar < 4.0 {
            0.92
        } else if bar < 20.0 {
            1.00
        } else if bar < 24.0 {
            0.62 + 0.12 * ((bar - 20.0) / 4.0)
        } else if bar < 30.0 {
            1.06
        } else {
            0.88
        };
        mix[i * 2] *= g;
        mix[i * 2 + 1] *= g;
    }

    highpass(&mut mix, 25.0);

    // Warm analog saturation + normalize to -0.5 dBFS
    for s in mix.iter_mut() {
        *s = (*s * 1.18).tanh() / 1.18f32.tanh();
    }
    let peak = mix.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    let target_peak = 0.944; // -0.5 dBFS
    let gain = if peak > 0.0 { target_peak / peak } else { 1.0 };
    for s in mix.iter_mut() {
        *s *= gain;
    }

    // Smooth fade in and fade out
    let fade_in = (0.25 * SR) as usize;
    let fade_out = (2.0 * SR) as usize;
    for i in 0..n_samples {
        let mut g = 1.0f32;
        if i < fade_in {
            g *= i as f32 / fade_in as f32;
        }
        if i + fade_out > n_samples {
            g *= ((n_samples - i) as f32 / fade_out as f32).clamp(0.0, 1.0);
        }
        mix[i * 2] *= g;
        mix[i * 2 + 1] *= g;
    }

    // Write final stereo WAV
    let path = out_dir.join("promo_track.wav");
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: SR as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec).expect("create wav");
    for s in &mix {
        writer
            .write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .expect("write sample");
    }
    writer.finalize().expect("finalize wav");

    let rms = (mix.iter().map(|s| s * s).sum::<f32>() / mix.len() as f32).sqrt();
    println!(
        "Mastered: {} ({:.1}s, peak {:.3}, rms {:.3})",
        path.display(),
        LEN,
        peak * gain,
        rms
    );
}
