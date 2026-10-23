//! Analog-modeled ADSR envelope with exponential curves.
//! Includes idle detection and precalculated coefficients for ultra-low CPU DSP.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Debug)]
pub struct AdsrEnvelope {
    pub attack_s: f32,
    pub decay_s: f32,
    pub sustain_level: f32, // 0.0 to 1.0
    pub release_s: f32,
    pub velocity_tracking: f32, // 0.0 to 1.0

    state: EnvelopeState,
    current_value: f32,
    velocity_scale: f32,
    sample_rate: f32,

    attack_rate: f32,
    decay_coeff: f32,
    release_coeff: f32,
    last_attack_s: f32,
    last_decay_s: f32,
    last_release_s: f32,
    last_sr: f32,
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        let mut env = Self {
            attack_s: 0.01,
            decay_s: 0.2,
            sustain_level: 0.7,
            release_s: 0.3,
            velocity_tracking: 1.0,
            state: EnvelopeState::Idle,
            current_value: 0.0,
            velocity_scale: 1.0,
            sample_rate: 44100.0,
            attack_rate: 0.0,
            decay_coeff: 0.0,
            release_coeff: 0.0,
            last_attack_s: -1.0,
            last_decay_s: -1.0,
            last_release_s: -1.0,
            last_sr: -1.0,
        };
        env.update_coeffs();
        env
    }
}

impl AdsrEnvelope {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1000.0);
        self.update_coeffs();
    }

    #[inline(always)]
    pub fn update_coeffs(&mut self) {
        let sr = self.sample_rate;
        let attack_samples = (self.attack_s * sr).max(1.0);
        self.attack_rate = 1.0 / attack_samples;

        let decay_samples = (self.decay_s * sr).max(1.0);
        self.decay_coeff = (-1.0 / decay_samples).exp();

        let release_samples = (self.release_s * sr).max(1.0);
        self.release_coeff = (-1.0 / release_samples).exp();

        self.last_attack_s = self.attack_s;
        self.last_decay_s = self.decay_s;
        self.last_release_s = self.release_s;
        self.last_sr = sr;
    }

    #[inline(always)]
    fn check_coeffs(&mut self) {
        if self.attack_s != self.last_attack_s
            || self.decay_s != self.last_decay_s
            || self.release_s != self.last_release_s
            || self.sample_rate != self.last_sr
        {
            self.update_coeffs();
        }
    }

    pub fn start(&mut self, velocity: f32) {
        self.check_coeffs();
        let vel = velocity.clamp(0.01, 1.0);
        self.velocity_scale = (1.0 - self.velocity_tracking) + self.velocity_tracking * vel;
        self.state = EnvelopeState::Attack;
    }

    pub fn release(&mut self) {
        if self.state != EnvelopeState::Idle {
            self.check_coeffs();
            self.state = EnvelopeState::Release;
        }
    }

    pub fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.current_value = 0.0;
    }

    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }

    #[inline(always)]
    pub fn state(&self) -> EnvelopeState {
        self.state
    }

    #[inline(always)]
    pub fn current_value(&self) -> f32 {
        self.current_value * self.velocity_scale
    }

    /// Advance one sample and return envelope amplitude (0.0 to 1.0).
    /// Ultra-fast execution with zero per-sample transcendental functions.
    #[inline(always)]
    pub fn process_sample(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Idle => {
                return 0.0;
            }
            EnvelopeState::Attack => {
                self.current_value += self.attack_rate;
                if self.current_value >= 1.0 {
                    self.current_value = 1.0;
                    self.state = EnvelopeState::Decay;
                }
            }
            EnvelopeState::Decay => {
                let diff = self.current_value - self.sustain_level;
                self.current_value = self.sustain_level + diff * self.decay_coeff;

                if (self.current_value - self.sustain_level).abs() < 0.001 {
                    self.current_value = self.sustain_level;
                    self.state = EnvelopeState::Sustain;
                }
            }
            EnvelopeState::Sustain => {
                self.current_value = self.sustain_level;
            }
            EnvelopeState::Release => {
                self.current_value *= self.release_coeff;

                if self.current_value < 0.0001 {
                    self.current_value = 0.0;
                    self.state = EnvelopeState::Idle;
                    return 0.0;
                }
            }
        }

        self.current_value * self.velocity_scale
    }
}
