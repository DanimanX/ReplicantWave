pub mod envelope;
pub mod filter;
pub mod fx;
pub mod lfo;
pub mod oscillator;
pub mod step_sequencer;
pub mod voice;
pub mod wavetable;

use fx::FxChain;
use step_sequencer::{SeqMode, SeqStepEvent, StepSequencer};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use voice::Voice;
use wavetable::WavetableBank;

pub const MAX_VOICES: usize = 16;
pub const SCOPE_BUFFER_SIZE: usize = 512;

pub struct ScopeBuffer {
    pub buffer: [f32; SCOPE_BUFFER_SIZE],
    pub write_idx: usize,
}

impl Default for ScopeBuffer {
    fn default() -> Self {
        Self {
            buffer: [0.0; SCOPE_BUFFER_SIZE],
            write_idx: 0,
        }
    }
}

pub struct SynthEngine {
    pub voices: [Voice; MAX_VOICES],
    pub bank: WavetableBank,
    pub sequencer: StepSequencer,
    pub fx: FxChain,

    pub master_gain: f32,
    pub mono_mode: bool,
    pub legato: bool,
    pub glide_time_s: f32,
    pub pitch_bend_semitones: f32,

    pub scope_data: Arc<parking_lot::Mutex<ScopeBuffer>>,
    pub active_voice_count: Arc<AtomicUsize>,
    pub held_midi_keys: usize,
    last_host_playing: bool,
    next_voice_age: u64,
    last_note: u8,
    sample_rate: f32,
}

impl Default for SynthEngine {
    fn default() -> Self {
        let bank = WavetableBank::new();
        let voices: [Voice; MAX_VOICES] = std::array::from_fn(|_| Voice::default());

        Self {
            voices,
            bank,
            sequencer: StepSequencer::default(),
            fx: FxChain::default(),
            master_gain: 0.8,
            mono_mode: false,
            legato: false,
            glide_time_s: 0.05,
            pitch_bend_semitones: 0.0,
            scope_data: Arc::new(parking_lot::Mutex::new(ScopeBuffer::default())),
            active_voice_count: Arc::new(AtomicUsize::new(0)),
            held_midi_keys: 0,
            last_host_playing: false,
            next_voice_age: 1,
            last_note: 60,
            sample_rate: 44100.0,
        }
    }
}

impl SynthEngine {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1000.0);
        for v in self.voices.iter_mut() {
            v.set_sample_rate(sr);
        }
        self.sequencer.set_sample_rate(sr);
        self.fx.set_sample_rate(sr);
    }

    pub fn reset(&mut self) {
        for v in self.voices.iter_mut() {
            v.kill();
        }
        self.sequencer.reset();
        self.fx.reset();
        self.held_midi_keys = 0;
        self.last_host_playing = false;
        self.active_voice_count.store(0, Ordering::Relaxed);
    }

    /// Smart MIDI note on bridging: routes to 64-step sequencer transposition
    /// or normal polyphonic synth voices cleanly without conflicts.
    pub fn handle_midi_note_on(&mut self, note: u8, vel: f32) {
        self.held_midi_keys += 1;
        if self.sequencer.is_active_for_synth() {
            // Transpose sequencer pattern to the pressed MIDI key
            self.sequencer.set_root_note(note);
            if self.sequencer.key_trigger {
                self.sequencer.running = true;
                self.sequencer.reset();
            }
        } else {
            self.note_on(note, vel);
        }
    }

    /// Smart MIDI note off bridging.
    pub fn handle_midi_note_off(&mut self, note: u8) {
        self.held_midi_keys = self.held_midi_keys.saturating_sub(1);
        if self.sequencer.is_active_for_synth() {
            if self.sequencer.key_trigger
                && !self.sequencer.latch
                && self.held_midi_keys == 0
                && !self.sequencer.host_sync
            {
                self.sequencer.running = false;
                for v in self.voices.iter_mut() {
                    v.note_off();
                }
            }
        } else {
            self.note_off(note);
        }
    }

    pub fn note_on(&mut self, note: u8, vel: f32) {
        self.next_voice_age += 1;
        let age_id = self.next_voice_age;
        let glide = self.glide_time_s > 0.001;

        if self.mono_mode {
            // Mono mode: always use voice 0
            self.voices[0].note_on(note, vel, glide, self.glide_time_s, age_id);
            self.last_note = note;
            return;
        }

        // Poly mode: find inactive voice or steal oldest
        let mut target_idx = None;

        // 1. First look for inactive voice
        for (i, v) in self.voices.iter().enumerate() {
            if !v.is_active() {
                target_idx = Some(i);
                break;
            }
        }

        // 2. If all active, steal voice in release stage with lowest level
        if target_idx.is_none() {
            let mut oldest_age = u64::MAX;
            let mut oldest_idx = 0;
            for (i, v) in self.voices.iter().enumerate() {
                if v.age < oldest_age {
                    oldest_age = v.age;
                    oldest_idx = i;
                }
            }
            target_idx = Some(oldest_idx);
        }

        let idx = target_idx.unwrap_or(0);
        self.voices[idx].note_on(note, vel, false, 0.0, age_id);
        self.last_note = note;
    }

    pub fn note_off(&mut self, note: u8) {
        if self.mono_mode {
            if self.voices[0].active_midi_note == note {
                self.voices[0].note_off();
            }
        } else {
            for v in self.voices.iter_mut() {
                if v.is_active() && v.active_midi_note == note && !v.is_releasing {
                    v.note_off();
                }
            }
        }
    }

    pub fn legato_glide(&mut self, note: u8, vel: f32) {
        // Find active sounding voice and glide frequency smoothly without envelope retrigger
        let mut glided = false;
        for v in self.voices.iter_mut() {
            if v.is_active() && !v.is_releasing {
                v.glide_to(note, vel, 0.05); // 50ms smooth acid slide
                self.last_note = note;
                glided = true;
                break;
            }
        }
        if !glided {
            self.note_on(note, vel);
        }
    }

    pub fn set_pitch_bend(&mut self, norm_bend: f32, max_semitones: f32) {
        self.pitch_bend_semitones = norm_bend * max_semitones;
    }

    /// Process a block of samples. Returns stereo output buffer.
    #[inline(always)]
    pub fn process_block(
        &mut self,
        out_left: &mut [f32],
        out_right: &mut [f32],
        host_bpm: f32,
        host_playing: bool,
    ) {
        let num_samples = out_left.len().min(out_right.len());

        // Detect host transport start/stop transitions
        if host_playing && !self.last_host_playing && self.sequencer.host_sync {
            self.sequencer.reset();
        } else if !host_playing && self.last_host_playing && self.sequencer.host_sync {
            // Host stopped: release active sequencer voice notes
            for v in self.voices.iter_mut() {
                if v.is_active() {
                    v.note_off();
                }
            }
        }
        self.last_host_playing = host_playing;

        let mut active_count = 0;

        for i in 0..num_samples {
            // Check Step Sequencer trigger with full host DAW & standalone bridging
            match self.sequencer.process_sample(host_bpm, host_playing) {
                SeqStepEvent::Trigger {
                    note,
                    velocity,
                    slide,
                    accent,
                } => {
                    if self.sequencer.mode == SeqMode::InternalSynth {
                        let final_vel = if accent { (velocity * 1.3).min(1.0) } else { velocity };
                        if slide {
                            self.legato_glide(note, final_vel);
                        } else {
                            self.note_on(note, final_vel);
                        }
                    }
                }
                SeqStepEvent::Release { note } => {
                    if self.sequencer.mode == SeqMode::InternalSynth {
                        self.note_off(note);
                    }
                }
                SeqStepEvent::None => {}
            }

            let extra_cutoff = match self.sequencer.mode {
                SeqMode::ModCutoff => (self.sequencer.mod_output - 0.5) * 5000.0,
                _ => 0.0,
            };

            if self.sequencer.mode == SeqMode::ModWTPos {
                let pos = self.sequencer.mod_output;
                for v in self.voices.iter_mut() {
                    v.osc1.position = pos;
                }
            }

            let mut mix_l = 0.0;
            let mut mix_r = 0.0;
            let mut active_voices = 0;

            // Render active voices (user friendly CPU: skip inactive!)
            for v in self.voices.iter_mut() {
                if v.is_active() {
                    active_voices += 1;
                    let (vl, vr) = v.process_sample(&self.bank, host_bpm, self.pitch_bend_semitones, extra_cutoff);
                    mix_l += vl;
                    mix_r += vr;
                }
            }

            active_count = active_voices;

            // Apply global FX chain
            let (fx_l, fx_r) = self.fx.process_sample(mix_l, mix_r, host_bpm);

            // Master volume & safety soft clipper
            let final_l = soft_clip(fx_l * self.master_gain);
            let final_r = soft_clip(fx_r * self.master_gain);

            out_left[i] = final_l;
            out_right[i] = final_r;
        }

        self.active_voice_count.store(active_count, Ordering::Relaxed);

        // Push to oscilloscope buffer ONCE per block without locking inside sample loop
        if let Some(mut scope) = self.scope_data.try_lock() {
            let step = (num_samples / 64).max(1);
            let mut s = 0;
            while s < num_samples {
                let idx = scope.write_idx;
                scope.buffer[idx] = (out_left[s] + out_right[s]) * 0.5;
                scope.write_idx = (idx + 1) % SCOPE_BUFFER_SIZE;
                s += step;
            }
        }
    }
}

#[inline(always)]
fn soft_clip(x: f32) -> f32 {
    if x > 1.0 {
        1.0
    } else if x < -1.0 {
        -1.0
    } else {
        x - (x * x * x) * (1.0 / 3.0)
    }
}
