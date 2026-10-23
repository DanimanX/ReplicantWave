//! Multi-mode Cyberpunk Distortion Engine.
//! Modes: Overdrive (Soft), Cyber Bitcrusher, Fire Amp, Grind Amp.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DistortionMode {
    SoftOverdrive,
    BitCrusher,
    FireAmp,
    GrindAmp,
}

impl DistortionMode {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::SoftOverdrive,
            1 => Self::BitCrusher,
            2 => Self::FireAmp,
            _ => Self::GrindAmp,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Distortion {
    pub enabled: bool,
    pub mode: DistortionMode,
    pub amount: f32, // 0.0 to 1.0 (maps to drive 1.0 to 20.0)
    pub mix: f32,    // 0.0 to 1.0
    // Bitcrusher downsample accumulator
    crush_phase: f32,
    crush_hold_l: f32,
    crush_hold_r: f32,
}

impl Default for Distortion {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: DistortionMode::SoftOverdrive,
            amount: 0.2,
            mix: 1.0,
            crush_phase: 0.0,
            crush_hold_l: 0.0,
            crush_hold_r: 0.0,
        }
    }
}

impl Distortion {
    pub fn reset(&mut self) {
        self.crush_phase = 0.0;
        self.crush_hold_l = 0.0;
        self.crush_hold_r = 0.0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        if !self.enabled || self.mix <= 0.001 {
            return (in_l, in_r);
        }

        let amt = self.amount.clamp(0.0, 1.0);
        let (wet_l, wet_r) = match self.mode {
            DistortionMode::SoftOverdrive => {
                let drive = 1.0 + amt * 10.0;
                let dl = in_l * drive;
                let dr = in_r * drive;
                (dl / (1.0 + dl.abs()), dr / (1.0 + dr.abs()))
            }
            DistortionMode::BitCrusher => {
                // Downsampling + bit depth reduction
                let bits = (16.0 - amt * 12.0).clamp(2.0, 16.0);
                let levels = 2.0f32.powf(bits);
                let step = 1.0 / levels;

                let downsample_ratio = 1.0 + amt * 24.0;
                self.crush_phase += 1.0;
                if self.crush_phase >= downsample_ratio {
                    self.crush_phase = 0.0;
                    self.crush_hold_l = (in_l / step).round() * step;
                    self.crush_hold_r = (in_r / step).round() * step;
                }
                (self.crush_hold_l, self.crush_hold_r)
            }
            DistortionMode::FireAmp => {
                // Asymmetric warm tube/tape saturation
                let drive = 1.0 + amt * 15.0;
                let sat = |x: f32| -> f32 {
                    let d = x * drive;
                    if d > 0.0 {
                        (d * 1.2).tanh()
                    } else {
                        (d * 0.8).tanh()
                    }
                };
                (sat(in_l), sat(in_r))
            }
            DistortionMode::GrindAmp => {
                // Harsh wavefolding fuzz
                let drive = 1.0 + amt * 25.0;
                let fold = |x: f32| -> f32 {
                    let d = x * drive;
                    (d * 1.5).sin()
                };
                (fold(in_l) * 0.8, fold(in_r) * 0.8)
            }
        };

        let dry_gain = 1.0 - self.mix;
        let wet_gain = self.mix;
        (in_l * dry_gain + wet_l * wet_gain, in_r * dry_gain + wet_r * wet_gain)
    }
}
