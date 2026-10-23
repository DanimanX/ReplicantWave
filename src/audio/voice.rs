//! Individual synthesizer voice containing oscillators, filter, envelopes, and modulations.

use super::envelope::AdsrEnvelope;
use super::filter::StereoFilter;
use super::lfo::Lfo;
use super::oscillator::{NoiseGenerator, SubOscillator, WavetableOscillator};
use super::wavetable::WavetableBank;

#[derive(Clone, Debug)]
pub struct Voice {
    pub osc1: WavetableOscillator,
    pub osc2: WavetableOscillator,
    pub sub: SubOscillator,
    pub noise: NoiseGenerator,
    pub filter: StereoFilter,
    pub amp_env: AdsrEnvelope,
    pub filter_env: AdsrEnvelope,
    pub mod_env: AdsrEnvelope,
    pub lfo1: Lfo,
    pub lfo2: Lfo,

    pub osc1_table_idx: usize,
    pub osc2_table_idx: usize,

    pub current_note: f32,
    pub target_note: f32,
    pub glide_coeff: f32,
    pub velocity: f32,
    pub is_releasing: bool,
    pub age: u64,
    pub active_midi_note: u8,
    sample_rate: f32,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            osc1: WavetableOscillator::default(),
            osc2: {
                let mut o = WavetableOscillator::default();
                o.enabled = true;
                o.tune_semitones = 7.0; // 5th above by default or 0
                o
            },
            sub: SubOscillator::default(),
            noise: NoiseGenerator::default(),
            filter: StereoFilter::default(),
            amp_env: AdsrEnvelope::default(),
            filter_env: {
                let mut e = AdsrEnvelope::default();
                e.attack_s = 0.05;
                e.decay_s = 0.3;
                e.sustain_level = 0.2;
                e.release_s = 0.3;
                e
            },
            mod_env: AdsrEnvelope::default(),
            lfo1: Lfo::default(),
            lfo2: Lfo::default(),
            osc1_table_idx: 0,
            osc2_table_idx: 1,
            current_note: 60.0,
            target_note: 60.0,
            glide_coeff: 1.0,
            velocity: 1.0,
            is_releasing: false,
            age: 0,
            active_midi_note: 60,
            sample_rate: 44100.0,
        }
    }
}

impl Voice {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1000.0);
        self.amp_env.set_sample_rate(sr);
        self.filter_env.set_sample_rate(sr);
        self.mod_env.set_sample_rate(sr);
        self.lfo1.set_sample_rate(sr);
        self.lfo2.set_sample_rate(sr);
    }

    pub fn is_active(&self) -> bool {
        self.amp_env.is_active()
    }

    pub fn note_on(&mut self, note: u8, vel: f32, glide: bool, glide_time_s: f32, age_id: u64) {
        self.active_midi_note = note;
        self.target_note = note as f32;
        self.velocity = vel.clamp(0.01, 1.0);
        self.is_releasing = false;
        self.age = age_id;

        if glide && glide_time_s > 0.001 {
            let glide_samples = (glide_time_s * self.sample_rate).max(1.0);
            self.glide_coeff = (-1.0 / glide_samples).exp();
        } else {
            self.current_note = self.target_note;
            self.glide_coeff = 0.0;
        }

        self.osc1.reset_phase(Some((note as f32 * 0.13) % 1.0));
        self.osc2.reset_phase(Some((note as f32 * 0.29) % 1.0));
        self.sub.reset_phase();

        self.amp_env.start(self.velocity);
        self.filter_env.start(self.velocity);
        self.mod_env.start(self.velocity);
        self.lfo1.reset_phase();
        self.lfo2.reset_phase();
    }

    pub fn note_off(&mut self) {
        self.is_releasing = true;
        self.amp_env.release();
        self.filter_env.release();
        self.mod_env.release();
    }

    pub fn kill(&mut self) {
        self.amp_env.reset();
        self.filter_env.reset();
        self.mod_env.reset();
        self.filter.reset();
        self.is_releasing = false;
    }

    #[inline(always)]
    pub fn process_sample(
        &mut self,
        bank: &WavetableBank,
        host_bpm: f32,
        pitch_bend_semitones: f32,
        extra_cutoff_mod: f32,
    ) -> (f32, f32) {
        if !self.is_active() {
            return (0.0, 0.0);
        }

        // Portamento / Glide smoothing
        if self.glide_coeff > 0.0 {
            self.current_note = self.target_note + (self.current_note - self.target_note) * self.glide_coeff;
        } else {
            self.current_note = self.target_note;
        }

        let effective_note = self.current_note + pitch_bend_semitones;
        // MIDI note to Hz formula
        let base_freq = 440.0 * 2.0f32.powf((effective_note - 69.0) / 12.0);

        // Process envelopes & LFOs
        let amp_level = self.amp_env.process_sample();
        let f_env_val = self.filter_env.process_sample();
        let _m_env_val = self.mod_env.process_sample();
        let lfo1_val = self.lfo1.process_sample(host_bpm);
        let _lfo2_val = self.lfo2.process_sample(host_bpm);

        // Oscillators
        let wt1 = bank.get_table(self.osc1_table_idx);
        let wt2 = bank.get_table(self.osc2_table_idx);

        let (o1_l, o1_r) = self.osc1.process_sample(wt1, base_freq, self.sample_rate);
        let (o2_l, o2_r) = self.osc2.process_sample(wt2, base_freq, self.sample_rate);
        let (sub_l, sub_r) = self.sub.process_sample(base_freq, self.sample_rate);
        let (ns_l, ns_r) = self.noise.process_sample();

        // Filter routing
        let mut pre_flt_l = 0.0;
        let mut pre_flt_r = 0.0;
        let mut bypass_l = 0.0;
        let mut bypass_r = 0.0;

        if self.filter.route_osc1 {
            pre_flt_l += o1_l; pre_flt_r += o1_r;
        } else {
            bypass_l += o1_l; bypass_r += o1_r;
        }

        if self.filter.route_osc2 {
            pre_flt_l += o2_l; pre_flt_r += o2_r;
        } else {
            bypass_l += o2_l; bypass_r += o2_r;
        }

        if self.filter.route_sub {
            pre_flt_l += sub_l; pre_flt_r += sub_r;
        } else {
            bypass_l += sub_l; bypass_r += sub_r;
        }

        if self.filter.route_noise {
            pre_flt_l += ns_l; pre_flt_r += ns_r;
        } else {
            bypass_l += ns_l; bypass_r += ns_r;
        }

        // Calculate filter cutoff with key tracking, envelope amount, and external mods
        let key_track_offset = (effective_note - 60.0) * self.filter.key_tracking * 100.0;
        let env_mod = f_env_val * self.filter.env_amount * 5000.0;
        let lfo_mod = lfo1_val * 400.0;
        let seq_mod = extra_cutoff_mod * 4000.0;

        let dynamic_cutoff = (self.filter.cutoff_hz + key_track_offset + env_mod + lfo_mod + seq_mod).clamp(20.0, 20000.0);

        let (flt_l, flt_r) = self.filter.process_sample(pre_flt_l, pre_flt_r, dynamic_cutoff, self.sample_rate);

        let final_l = (flt_l + bypass_l) * amp_level;
        let final_r = (flt_r + bypass_r) * amp_level;

        (final_l, final_r)
    }
}
