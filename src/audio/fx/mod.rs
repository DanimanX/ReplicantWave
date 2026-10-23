pub mod chorus;
pub mod delay;
pub mod distortion;
pub mod gate;
pub mod reverb;

use chorus::Chorus;
use delay::StereoDelay;
use distortion::Distortion;
use gate::TranceGate;
use reverb::StereoReverb;

#[derive(Clone, Debug, Default)]
pub struct FxChain {
    pub gate: TranceGate,
    pub chorus: Chorus,
    pub distortion: Distortion,
    pub delay: StereoDelay,
    pub reverb: StereoReverb,
}

impl FxChain {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.gate.set_sample_rate(sr);
        self.chorus.set_sample_rate(sr);
        self.delay.set_sample_rate(sr);
        self.reverb.set_sample_rate(sr);
    }

    pub fn reset(&mut self) {
        self.gate.reset();
        self.chorus.reset();
        self.distortion.reset();
        self.delay.reset();
        self.reverb.reset();
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32, host_bpm: f32) -> (f32, f32) {
        // Chain order: Gate -> Chorus -> Distortion -> Delay -> Reverb
        let (s1_l, s1_r) = self.gate.process_sample(in_l, in_r, host_bpm);
        let (s2_l, s2_r) = self.chorus.process_sample(s1_l, s1_r);
        let (s3_l, s3_r) = self.distortion.process_sample(s2_l, s2_r);
        let (s4_l, s4_r) = self.delay.process_sample(s3_l, s3_r, host_bpm);
        let (out_l, out_r) = self.reverb.process_sample(s4_l, s4_r);

        (out_l, out_r)
    }
}
