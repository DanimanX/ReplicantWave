//! Offline promo-track renderer for ReplicantWave.
//!
//! Generates a crystal-clear, punchy Midtempo Cyberpunk cue (102 BPM in B minor)
//! inspired by "Extra Terra - Cyberpunk" (Convergence 2045):
//! - "Extra Terra Cyber Bass" (Crisp wavefolded growls with punchy mid bite)
//! - "Extra Terra Cyber Lead" (Futuristic razor-sharp saw lead soaring through stereo delay)
//! - "Night City Midtempo Chug" (Rolling 16th-note electro-bass chug)
//! - "Reinelex Neuro Reese" (Wide detuned stereo Reese bass for massive width)
//! - "Cyberpunk 2077 Combat Lead" (Hard-synced screamer lead counter-melody)
//! - "Darksynth Power Stab" (Punchy analog brass stabs)
//! - "Netrunner Arp" (Rapid cyber arpeggiator)
//! - "Ghost Protocol Choir" (Ethereal vocaloid choir in the breakdown)
//! - "Corpo Tower Dystopia" (Lush cinematic pad)
//! - "Blackwall Drone" (Dystopian sub drone)
//!
//! Features the signature Extra Terra "Cyberdrums" kick with clean, punchy transient,
//! solid 49 Hz sub thump, dedicated sidechain ducking, and transparent soft-knee mastering
//! (Target RMS ~0.22 - 0.25, Peak -0.6 dBFS) with zero harsh distortion or low-end mud.

use replicantwave::audio::SynthEngine;
use replicantwave::presets::factory::get_factory_presets;
use replicantwave::presets::SynthPreset;
use std::f32::consts::TAU;
use std::path::PathBuf;

pub const SR: f32 = 48_000.0;
pub const BPM: f32 = 102.0; // Extra Terra - Cyberpunk tempo (102 BPM)
pub const BEAT: f32 = 60.0 / BPM; // ~0.5882 s
pub const BAR: f32 = BEAT * 4.0;  // ~2.3529 s
pub const BARS: usize = 28;       // ~65.88 s
pub const TAIL: f32 = 2.0;
pub const LEN: f32 = BAR * BARS as f32 + TAIL; // ~67.88 s

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
// Clean, Punchy Extra Terra "Cyberdrums" Kick & Percussion (102 BPM)
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

/// Extra Terra "Cyberdrums" Kick:
/// - Crisp, snappy transient click (1600 Hz, fast decay in 3 ms)
/// - Punchy mid knock diving smoothly from 180 Hz to 49 Hz
/// - Deep, clean 49 Hz sub body (B0) with natural decay
/// - Soft-clipped to 0.85 without buzzing or squaring off the waveform
pub fn add_extra_terra_kick(buf: &mut [f32], t0: f32, gain: f32) {
    let start = (t0 * SR) as usize;
    let len = (0.32 * SR) as usize;
    for i in 0..len {
        let idx = start + i;
        if idx * 2 + 1 >= buf.len() {
            break;
        }
        let t = i as f32 / SR;

        // 1. Transient click (sharp electronic click, fast decay in 3ms)
        let click = (TAU * 1600.0 * t).sin() * (-t * 680.0).exp() * 0.45;

        // 2. Punch knock & pitch dive (starts at 180 Hz, dives into 49 Hz in 35ms)
        let f = 49.0 + 135.0 * (-t * 36.0).exp();
        let phase = TAU * f * t;
        let env = (-t * 8.8).exp();
        let body = phase.sin() * env;

        // Clean soft saturation without harsh clipping
        let raw = (click + body * 0.90) * gain;
        let s = if raw.abs() > 0.72 {
            let sign = raw.signum();
            let excess = raw.abs() - 0.72;
            sign * (0.72 + excess / (1.0 + excess * 1.6))
        } else {
            raw
        };
        buf[idx * 2] += s;
        buf[idx * 2 + 1] += s;
    }
}

/// Crisp Midtempo Snare: 196 Hz body + stereo white noise burst with reverb.
pub fn add_midtempo_snare(buf: &mut [f32], t0: f32, gain: f32, rng: &mut Rng) {
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
        hp = 0.88 * (hp + n - prev);
        prev = n;
        let noise_env = (-t * 14.0).exp();
        let tone_f = 196.0 + 95.0 * (-t * 45.0).exp();
        let tone = (TAU * tone_f * t).sin() * (-t * 26.0).exp() * 0.65;
        let s = (hp * noise_env * 0.75 + tone) * gain * 0.82;
        buf[idx * 2] += s * 0.95;
        buf[idx * 2 + 1] += s * 1.05;
    }
}

/// Cyber metallic clap layer.
pub fn add_midtempo_clap(buf: &mut [f32], t0: f32, gain: f32, rng: &mut Rng) {
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
            hp = 0.90 * (hp + n - prev);
            prev = n;
            let env = (-t * 22.0).exp() * (1.0 - k as f32 * 0.18);
            let s = hp * env * gain * 0.55;
            buf[idx * 2] += s * 1.05;
            buf[idx * 2 + 1] += s * 0.95;
        }
    }
}

/// Sizzling closed / open hi-hat.
pub fn add_midtempo_hat(buf: &mut [f32], t0: f32, gain: f32, open: bool, rng: &mut Rng) {
    let start = (t0 * SR) as usize;
    let len = ((if open { 0.22 } else { 0.05 }) * SR) as usize;
    let mut prev = 0.0f32;
    let mut hp = 0.0f32;
    for i in 0..len {
        let idx = start + i;
        if idx * 2 + 1 >= buf.len() {
            break;
        }
        let t = i as f32 / SR;
        let n = rng.next();
        hp = 0.93 * (hp + n - prev);
        prev = n;
        let env = (-t * if open { 15.0 } else { 65.0 }).exp();
        let s = hp * env * gain * 0.65;
        let pan = if i % 2 == 0 { 0.88 } else { 1.12 };
        buf[idx * 2] += s * pan;
        buf[idx * 2 + 1] += s * (2.0 - pan);
    }
}

/// Sweeping cyber riser with rising tone.
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
        let coeff = 0.003 + 0.16 * u * u;
        lp += coeff * (rng.next() - lp);
        let env = u.powf(2.1);
        let f_rise = 49.0 * (8.0f32).powf(u);
        let tone = (TAU * f_rise * t).sin() * 0.16 * env;
        let s = (lp * 2.0 + tone) * env * gain * 0.75;
        buf[idx * 2] += s * (0.85 + 0.15 * u);
        buf[idx * 2 + 1] += s * (1.15 - 0.15 * u);
    }
}

/// Cinematic braam / sub impact.
pub fn add_braam_impact(buf: &mut [f32], t0: f32, gain: f32) {
    let start = (t0 * SR) as usize;
    let len = (2.2 * SR) as usize;
    let mut lp = 0.0f32;
    let mut rng = Rng::new(0x2077_BEEF);
    for i in 0..len {
        let idx = start + i;
        if idx * 2 + 1 >= buf.len() {
            break;
        }
        let t = i as f32 / SR;
        let sub_pitch = (TAU * (58.0 - 18.0 * (t / 1.6).min(1.0)) * t).sin();
        let sub_boom = sub_pitch * (-t * 2.2).exp();
        lp += 0.07 * (rng.next() - lp);
        let crash = lp * (-t * 3.0).exp() * 1.2;
        let s = (sub_boom * 0.95 + crash) * gain * 0.85;
        buf[idx * 2] += s;
        buf[idx * 2 + 1] += s;
    }
}

// ---------------------------------------------------------------------------
// Midtempo Musical Composition in B Minor (Extra Terra Style)
// Chords: Bm (B, D, F#), G (G, B, D), Em (E, G, B), F# (F#, A#, C#)
// ---------------------------------------------------------------------------

/// Root notes in MIDI: B1 (35), G1 (31), E1 (28), F#1 (30).
const ROOTS: [u8; 4] = [35, 31, 28, 30];

const TRIADS: [[i8; 3]; 4] = [
    [0, 3, 7], // Bm
    [0, 4, 7], // G
    [0, 3, 7], // Em
    [0, 4, 7], // F# (Major Dominant for tension)
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

/// Extra Terra Midtempo Growl Pattern ("Extra Terra Cyber Bass").
/// Heavily syncopated midtempo groove: downbeat punch, 16th-note stutters, octave leaps, and wavefolding.
pub fn extra_terra_growl_line(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let root = ROOTS[bar % 4] as i16 + 12; // B2 (47) etc.
        // Extra Terra signature midtempo rhythm
        let pattern: [(f32, i16, f32, f32); 10] = [
            (0.00,  0, 0.95, 0.24), // Heavy downbeat stomp
            (0.75, 12, 0.90, 0.32), // Octave growl
            (1.25,  3, 0.85, 0.20), // Minor third stab
            (1.75,  0, 0.90, 0.22), // Syncopated chug
            (2.00,  0, 0.95, 0.24), // Beat 3 punch
            (2.50, 12, 0.90, 0.30), // High scream
            (3.00,  7, 0.85, 0.20), // Fifth pop
            (3.50,  1, 0.90, 0.26), // Half-step slide tension
            (3.75, 12, 0.80, 0.16),
            (2.75,  0, 0.75, 0.15),
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

/// Rolling 16th-note electro chug bassline ("Night City Midtempo Chug").
/// Leaves space during the major growl stabs for crystal-clear call-and-response!
pub fn midtempo_chug_line(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let root = ROOTS[bar % 4] as i16 + 12; // B2
        for step in 0..16 {
            // Leave space during beats 2.5 and 3.5 so the main growl cuts through cleanly
            if step == 10 || step == 11 || step == 14 || step == 15 {
                continue;
            }
            let beat = step as f32 * 0.25;
            let is_downbeat = step % 4 == 0;
            let vel = if is_downbeat { 0.90 } else if step % 2 == 0 { 0.80 } else { 0.70 };
            out.push(Note {
                time: beat_time(bar, beat),
                note: root.clamp(0, 127) as u8,
                vel,
                dur: 0.14,
            });
        }
    }
    out
}

/// Razor-sharp Cyber Lead Melody ("Extra Terra Cyber Lead").
/// Heroic, dark cyberpunk theme inspired by Extra Terra - Cyberpunk at 102 BPM.
pub fn extra_terra_lead_melody(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    // B minor motif in Octaves 4 and 5 (B4 = 71, D5 = 74, F#5 = 78, B5 = 83)
    let motif_a: [(f32, u8, f32, f32); 10] = [
        (0.00, 83, 0.95, 0.60), // B5
        (1.00, 86, 0.85, 0.45), // D6
        (1.75, 85, 0.80, 0.35), // C#6
        (2.50, 83, 0.90, 0.60), // B5
        (3.50, 78, 0.80, 0.40), // F#5
        (4.00, 79, 0.95, 0.60), // G5
        (5.00, 81, 0.85, 0.45), // A5
        (5.75, 83, 0.90, 0.50), // B5
        (6.75, 81, 0.80, 0.35), // A5
        (7.25, 79, 0.85, 0.60), // G5
    ];
    let motif_b: [(f32, u8, f32, f32); 10] = [
        (0.00, 76, 0.95, 0.60), // E5
        (1.00, 79, 0.85, 0.45), // G5
        (1.75, 83, 0.90, 0.50), // B5
        (2.50, 81, 0.85, 0.45), // A5
        (3.25, 79, 0.80, 0.50), // G5
        (4.00, 78, 0.95, 0.60), // F#5
        (5.00, 82, 0.85, 0.45), // A#5
        (5.75, 85, 0.95, 0.60), // C#6
        (6.75, 83, 0.85, 0.40), // B5
        (7.25, 82, 0.90, 0.70), // A#5
    ];

    for bar in bars.step_by(4) {
        for (beat, note, vel, dur) in motif_a {
            let t = bar_time(bar) + beat * BEAT;
            if t < LEN {
                out.push(Note { time: t, note, vel, dur: dur * BEAT * 1.4 });
            }
        }
        for (beat, note, vel, dur) in motif_b {
            let t = bar_time(bar + 2) + beat * BEAT;
            if t < LEN {
                out.push(Note { time: t, note, vel, dur: dur * BEAT * 1.4 });
            }
        }
    }
    out
}

/// Searing Cyberpunk Combat Lead ("Cyberpunk 2077 Combat Lead").
pub fn combat_lead_counter(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    let motif: [(f32, u8, f32, f32); 8] = [
        (0.00, 71, 0.90, 0.45), // B4
        (0.75, 74, 0.85, 0.35), // D5
        (1.50, 78, 0.90, 0.45), // F#5
        (2.25, 83, 0.95, 0.55), // B5
        (3.00, 82, 0.85, 0.40), // A#5
        (3.75, 78, 0.80, 0.35), // F#5
        (4.50, 79, 0.90, 0.50), // G5
        (5.25, 78, 0.85, 0.60), // F#5
    ];
    for bar in bars.step_by(2) {
        for (beat, note, vel, dur) in motif {
            let t = bar_time(bar) + beat * BEAT;
            if t < LEN {
                out.push(Note { time: t, note, vel, dur: dur * BEAT * 1.3 });
            }
        }
    }
    out
}

/// Wide detuned stereo Reese bed ("Reinelex Neuro Reese").
pub fn reese_line(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let root = ROOTS[bar % 4] as i16 + 12; // B2
        out.push(Note {
            time: bar_time(bar),
            note: root.clamp(0, 127) as u8,
            vel: 0.80,
            dur: BAR * 0.95,
        });
    }
    out
}

/// Cyber Arp Sequence ("Netrunner Arp").
pub fn alarm_arp(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let c = chord(bar);
        let tones = [
            c[0] as i16 + 24,
            c[1] as i16 + 24,
            c[2] as i16 + 24,
            c[0] as i16 + 36,
        ];
        for step in 0..16 {
            let idx = (step * 2) % tones.len();
            out.push(Note {
                time: beat_time(bar, step as f32 * 0.25),
                note: tones[idx].clamp(0, 127) as u8,
                vel: if step % 4 == 0 { 0.85 } else { 0.65 },
                dur: 0.11,
            });
        }
    }
    out
}

/// Dystopian pad foundation ("Corpo Tower Dystopia").
pub fn pad_chords(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let c = chord(bar);
        for tone in c {
            out.push(Note {
                time: bar_time(bar),
                note: (tone as i16 + 24).clamp(0, 127) as u8,
                vel: 0.65,
                dur: BAR * 0.96,
            });
        }
    }
    out
}

/// Analog brass stabs ("Darksynth Power Stab").
pub fn power_stabs(bars: std::ops::Range<usize>) -> Vec<Note> {
    let mut out = Vec::new();
    for bar in bars {
        let c = chord(bar);
        for &beat in &[0.0f32, 1.75, 2.5] {
            for &tone in &c {
                out.push(Note {
                    time: beat_time(bar, beat),
                    note: (tone as i16 + 24).clamp(0, 127) as u8,
                    vel: 0.85,
                    dur: 0.20,
                });
            }
        }
    }
    out
}

/// Pure deep sine sub-bass (40-60 Hz) following the bassline roots.
pub fn sub_stem(notes: &[Note]) -> Vec<f32> {
    let total = (LEN * SR) as usize;
    let mut out = vec![0.0f32; total * 2];
    for note in notes {
        let f = 440.0 * 2.0f32.powf((note.note as f32 - 69.0) / 12.0);
        let start = (note.time * SR) as usize;
        let len = ((note.dur + 0.04) * SR) as usize;
        let mut phase = 0.0f32;
        for i in 0..len {
            let idx = start + i;
            if idx >= total {
                break;
            }
            let t = i as f32 / SR;
            let attack = (t / 0.01).min(1.0);
            let release = ((note.dur + 0.04 - t) / 0.04).clamp(0.0, 1.0);
            let env = attack * release * note.vel;
            let s = (phase * TAU).sin() * env;
            out[idx * 2] += s;
            out[idx * 2 + 1] += s;
            phase = (phase + f / SR).fract();
        }
    }
    out
}

/// Complete Midtempo Drum & Percussion Track (102 BPM).
pub fn drums(rng: &mut Rng) -> Vec<f32> {
    let mut buf = vec![0.0f32; ((LEN * SR) as usize) * 2];

    for bar in 0..BARS {
        let is_intro = bar < 4;
        let is_breakdown = (20..24).contains(&bar);
        let is_drop = (4..20).contains(&bar) || (24..27).contains(&bar);

        if is_intro {
            if bar == 2 || bar == 3 {
                add_extra_terra_kick(&mut buf, bar_time(bar), 0.70);
                if bar == 3 {
                    add_extra_terra_kick(&mut buf, beat_time(bar, 2.0), 0.70);
                }
            }
            continue;
        }

        if is_breakdown {
            // Half-time pulse with deep sub knock
            add_extra_terra_kick(&mut buf, bar_time(bar), 0.80);
            add_extra_terra_kick(&mut buf, beat_time(bar, 2.0), 0.75);
            if bar == 23 {
                // Snare build into climax drop
                for k in 0..12 {
                    add_midtempo_snare(&mut buf, beat_time(bar, 1.0 + k as f32 * 0.25), 0.40 + k as f32 * 0.04, rng);
                }
            }
            continue;
        }

        if is_drop {
            // Signature Extra Terra 4-on-the-floor stomping kick
            for beat in 0..4 {
                let g = if beat == 0 { 1.00 } else { 0.88 };
                add_extra_terra_kick(&mut buf, beat_time(bar, beat as f32), g);
            }
            // Subtle syncopated ghost kick
            add_extra_terra_kick(&mut buf, beat_time(bar, 1.75), 0.65);

            // Snare + Clap on beats 1.0 and 3.0 (i.e. beats 2 and 4 in 4/4)
            add_midtempo_snare(&mut buf, beat_time(bar, 1.0), 0.82, rng);
            add_midtempo_snare(&mut buf, beat_time(bar, 3.0), 0.86, rng);
            add_midtempo_clap(&mut buf, beat_time(bar, 1.0), 0.60, rng);
            add_midtempo_clap(&mut buf, beat_time(bar, 3.0), 0.64, rng);

            // Sizzling 16th hats with open hats on offbeats
            for step in 0..16 {
                let t = beat_time(bar, step as f32 * 0.25);
                let open = step % 4 == 2;
                let g = if open { 0.28 } else if step % 2 == 0 { 0.20 } else { 0.12 };
                add_midtempo_hat(&mut buf, t, g, open, rng);
            }

            // Snare roll fills before transitions
            if bar == 11 || bar == 19 || bar == 26 {
                for k in 0..6 {
                    add_midtempo_snare(&mut buf, beat_time(bar, 2.5 + k as f32 * 0.25), 0.45 + k as f32 * 0.06, rng);
                }
            }
        }
    }

    // Risers and Impacts at structural milestones
    add_riser(&mut buf, bar_time(2) + BEAT * 2.0, BAR * 1.5, 0.70, rng);
    add_braam_impact(&mut buf, bar_time(4), 0.95); // DROP 1

    add_riser(&mut buf, bar_time(10) + BEAT * 2.0, BAR * 1.5, 0.75, rng);
    add_braam_impact(&mut buf, bar_time(12), 1.00); // DROP 2

    add_riser(&mut buf, bar_time(22), BAR * 2.0, 0.85, rng); // BREAKDOWN RISER
    add_braam_impact(&mut buf, bar_time(24), 1.10); // CLIMAX DROP

    add_braam_impact(&mut buf, bar_time(27), 0.80); // OUTRO

    buf
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

/// Precision Sidechain Compression:
/// Ducks the synth bus by ~22 dB whenever the kick strikes, then smoothly releases
/// with a quadratic curve over 190 ms, creating the signature midtempo pumping groove
/// and ensuring the kick punch never collides with bass frequencies.
pub fn sidechain_duck(bus: &mut [f32]) {
    let n_samples = bus.len() / 2;
    for i in 0..n_samples {
        let t = i as f32 / SR;
        if t < bar_time(4) {
            continue;
        }
        let beat_pos = (t / BEAT).fract();
        let time_in_beat = beat_pos * BEAT;
        let duck = if time_in_beat < 0.045 {
            0.08 // -22 dB during kick transient
        } else if time_in_beat < 0.22 {
            let u = (time_in_beat - 0.045) / 0.175;
            0.08 + 0.92 * (u * u) // smooth quadratic release
        } else {
            1.0
        };
        bus[i * 2] *= duck;
        bus[i * 2 + 1] *= duck;
    }
}

fn main() {
    let out_dir = PathBuf::from("target/promo");
    std::fs::create_dir_all(&out_dir).expect("create target/promo");

    println!("Rendering stems with the ReplicantWave engine at {BPM} BPM (Extra Terra Cyberpunk Style)...");

    let (et_bass, et_lead, combat, reese, chug, arp, pad, stabs, choir, drone) =
        std::thread::scope(|s| {
            let et_bass_h = s.spawn(|| {
                let p = find_preset("Extra Terra Cyber Bass");
                let mut notes = extra_terra_growl_line(4..20);
                notes.extend(extra_terra_growl_line(24..27));
                render_stem(&p, &notes)
            });
            let et_lead_h = s.spawn(|| {
                let p = find_preset("Extra Terra Cyber Lead");
                let mut notes = extra_terra_lead_melody(12..20);
                notes.extend(extra_terra_lead_melody(24..27));
                let mut stem = render_stem(&p, &notes);
                highpass(&mut stem, 240.0); // Keep lead high and clear
                stem
            });
            let combat_h = s.spawn(|| {
                let p = find_preset("Cyberpunk 2077 Combat Lead");
                let notes = combat_lead_counter(24..27);
                let mut stem = render_stem(&p, &notes);
                highpass(&mut stem, 250.0);
                stem
            });
            let reese_h = s.spawn(|| {
                let p = find_preset("Reinelex Neuro Reese");
                let mut notes = reese_line(12..20);
                notes.extend(reese_line(24..27));
                let mut stem = render_stem(&p, &notes);
                highpass(&mut stem, 110.0); // Reese adds stereo width above sub
                stem
            });
            let chug_h = s.spawn(|| {
                let p = find_preset("Night City Midtempo Chug");
                render_stem(&p, &midtempo_chug_line(4..27))
            });
            let arp_h = s.spawn(|| {
                let p = find_preset("Netrunner Arp");
                let mut notes = alarm_arp(2..20);
                notes.extend(alarm_arp(24..27));
                let mut stem = render_stem(&p, &notes);
                highpass(&mut stem, 220.0);
                stem
            });
            let pad_h = s.spawn(|| {
                let p = find_preset("Corpo Tower Dystopia");
                let mut stem = render_stem(&p, &pad_chords(0..28));
                highpass(&mut stem, 160.0);
                stem
            });
            let stabs_h = s.spawn(|| {
                let p = find_preset("Darksynth Power Stab");
                let mut notes = power_stabs(4..20);
                notes.extend(power_stabs(24..27));
                let mut stem = render_stem(&p, &notes);
                highpass(&mut stem, 180.0);
                stem
            });
            let choir_h = s.spawn(|| {
                let p = find_preset("Ghost Protocol Choir");
                let mut notes = pad_chords(12..24);
                notes.extend(pad_chords(24..27));
                let mut stem = render_stem(&p, &notes);
                highpass(&mut stem, 200.0);
                stem
            });
            let drone_h = s.spawn(|| {
                let p = find_preset("Blackwall Drone");
                let notes = vec![
                    Note { time: 0.0, note: 35, vel: 0.75, dur: BAR * 4.0 },
                    Note { time: bar_time(20), note: 35, vel: 0.70, dur: BAR * 4.0 },
                    Note { time: bar_time(27), note: 35, vel: 0.70, dur: BAR * 1.5 },
                ];
                render_stem(&p, &notes)
            });
            (
                et_bass_h.join().unwrap(),
                et_lead_h.join().unwrap(),
                combat_h.join().unwrap(),
                reese_h.join().unwrap(),
                chug_h.join().unwrap(),
                arp_h.join().unwrap(),
                pad_h.join().unwrap(),
                stabs_h.join().unwrap(),
                choir_h.join().unwrap(),
                drone_h.join().unwrap(),
            )
        });

    println!("Rendering clean sub-bass and percussion...");
    let bass_notes = extra_terra_growl_line(4..27);
    let mut sub = sub_stem(&bass_notes);
    lowpass(&mut sub, 85.0); // Pure clean sub below 85 Hz

    let mut rng = Rng::new(0x2077_D41C);
    let drum = drums(&mut rng);

    // Save diagnostic stems
    for (name, stem) in [
        ("et_bass",      &et_bass),
        ("et_lead",      &et_lead),
        ("combat_lead",  &combat),
        ("reese",        &reese),
        ("chug",         &chug),
        ("sub",          &sub),
        ("arp",          &arp),
        ("pad",          &pad),
        ("stabs",        &stabs),
        ("choir",        &choir),
        ("drone",        &drone),
        ("drums",        &drum),
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

    // Mixdown: Synth bus balance with plenty of headroom
    let total = et_bass.len();
    let mut synth_bus = vec![0.0f32; total];
    let stems: [(&Vec<f32>, f32); 10] = [
        (&et_bass, 0.38),
        (&et_lead, 0.34),
        (&combat,  0.26),
        (&reese,   0.24),
        (&chug,    0.28),
        (&sub,     0.34),
        (&arp,     0.18),
        (&pad,     0.28),
        (&stabs,   0.24),
        (&choir,   0.22),
    ];
    for (stem, gain) in stems {
        for i in 0..total {
            synth_bus[i] += stem[i] * gain;
        }
    }
    for i in 0..total {
        synth_bus[i] += drone[i] * 0.28;
    }

    // Highpass synth bus at 28 Hz to remove inaudible sub-rumble
    highpass(&mut synth_bus, 28.0);

    // Apply precision sidechain ducking to the synth bus (makes kick punch 100% clean)
    sidechain_duck(&mut synth_bus);

    // Sum synths and drums together
    let mut mix = vec![0.0f32; total];
    for i in 0..total {
        mix[i] = synth_bus[i] + drum[i] * 0.85;
    }

    // Dynamic section volume curve
    let n_samples = mix.len() / 2;
    for i in 0..n_samples {
        let t = i as f32 / SR;
        let bar = t / BAR;
        let g = if bar < 4.0 {
            0.90
        } else if bar < 20.0 {
            1.00
        } else if bar < 24.0 {
            0.75 + 0.10 * ((bar - 20.0) / 4.0)
        } else if bar < 27.0 {
            1.05
        } else {
            0.88
        };
        mix[i * 2] *= g;
        mix[i * 2 + 1] *= g;
    }

    // Transparent soft-knee limiter (zero harsh clipping, punchy open dynamics)
    let knee_start = 0.70;
    let ceiling = 0.933; // -0.6 dBFS
    for s in mix.iter_mut() {
        let a = s.abs();
        if a > knee_start {
            let excess = a - knee_start;
            let compressed = knee_start + (1.0 - knee_start) * (excess / (1.0 - knee_start)).tanh();
            *s = s.signum() * compressed.min(ceiling);
        }
    }

    // Normalize to exact -0.6 dBFS ceiling
    let current_peak = mix.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if current_peak > 0.0 {
        let norm = ceiling / current_peak;
        for s in mix.iter_mut() {
            *s *= norm;
        }
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

    let peak = mix.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    let rms = (mix.iter().map(|s| s * s).sum::<f32>() / mix.len() as f32).sqrt();
    println!(
        "Mastered: {} ({:.1}s, peak {:.3}, rms {:.3})",
        path.display(),
        LEN,
        peak,
        rms
    );
}
