//! 16-Step Trance Gate / Rhythmic Chopper.

pub const NUM_GATE_STEPS: usize = 16;

#[derive(Clone, Debug)]
pub struct TranceGate {
    pub enabled: bool,
    pub steps_l: [bool; NUM_GATE_STEPS],
    pub steps_r: [bool; NUM_GATE_STEPS],
    pub beat_division: f32, // 0.25 = 1/16th notes
    pub attack_s: f32,
    pub release_s: f32,

    current_step: usize,
    sample_counter: f32,
    envelope_l: f32,
    envelope_r: f32,
    sample_rate: f32,
}

impl Default for TranceGate {
    fn default() -> Self {
        let mut steps_l = [true; NUM_GATE_STEPS];
        let mut steps_r = [true; NUM_GATE_STEPS];
        for i in 0..NUM_GATE_STEPS {
            steps_l[i] = i % 2 == 0 || i % 5 == 0;
            steps_r[i] = i % 2 == 0 || i % 3 == 0;
        }

        Self {
            enabled: false,
            steps_l,
            steps_r,
            beat_division: 0.25,
            attack_s: 0.005,
            release_s: 0.015,
            current_step: 0,
            sample_counter: 0.0,
            envelope_l: 1.0,
            envelope_r: 1.0,
            sample_rate: 44100.0,
        }
    }
}

impl TranceGate {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1000.0);
    }

    pub fn reset(&mut self) {
        self.current_step = 0;
        self.sample_counter = 0.0;
        self.envelope_l = 1.0;
        self.envelope_r = 1.0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32, host_bpm: f32) -> (f32, f32) {
        if !self.enabled {
            return (in_l, in_r);
        }

        let bpm = host_bpm.max(20.0);
        let beats_per_sec = bpm / 60.0;
        let step_time = self.beat_division / beats_per_sec;
        let step_samples = step_time * self.sample_rate;

        self.sample_counter += 1.0;
        if self.sample_counter >= step_samples {
            self.sample_counter = 0.0;
            self.current_step = (self.current_step + 1) % NUM_GATE_STEPS;
        }

        let target_l = if self.steps_l[self.current_step] { 1.0 } else { 0.0 };
        let target_r = if self.steps_r[self.current_step] { 1.0 } else { 0.0 };

        // Smoothing attack/release
        let sr = self.sample_rate;
        let atk_coeff = 1.0 / (self.attack_s * sr).max(1.0);
        let rel_coeff = 1.0 / (self.release_s * sr).max(1.0);

        if target_l > self.envelope_l {
            self.envelope_l = (self.envelope_l + atk_coeff).min(1.0);
        } else {
            self.envelope_l = (self.envelope_l - rel_coeff).max(0.0);
        }

        if target_r > self.envelope_r {
            self.envelope_r = (self.envelope_r + atk_coeff).min(1.0);
        } else {
            self.envelope_r = (self.envelope_r - rel_coeff).max(0.0);
        }

        (in_l * self.envelope_l, in_r * self.envelope_r)
    }
}
