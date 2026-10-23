//! Algorithmic Stereo Diffusion Reverb (Freeverb architecture).

const NUM_COMBS: usize = 8;
const NUM_ALLPASSES: usize = 4;

// Tuned prime delays for natural spatial diffusion
const COMB_TUNING_L: [usize; NUM_COMBS] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const COMB_TUNING_R: [usize; NUM_COMBS] = [1139, 1211, 1300, 1379, 1445, 1514, 1580, 1640];
const ALLPASS_TUNING_L: [usize; NUM_ALLPASSES] = [556, 441, 341, 225];
const ALLPASS_TUNING_R: [usize; NUM_ALLPASSES] = [579, 464, 364, 248];

#[derive(Clone, Debug)]
struct CombFilter {
    buffer: Vec<f32>,
    buf_size: usize,
    buf_idx: usize,
    filter_store: f32,
    damp1: f32,
    damp2: f32,
    feedback: f32,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            buf_size: size,
            buf_idx: 0,
            filter_store: 0.0,
            damp1: 0.5,
            damp2: 0.5,
            feedback: 0.8,
        }
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.buf_idx = 0;
        self.filter_store = 0.0;
    }

    fn set_damp(&mut self, damp: f32) {
        self.damp1 = damp;
        self.damp2 = 1.0 - damp;
    }

    fn set_feedback(&mut self, fb: f32) {
        self.feedback = fb;
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.buf_idx];
        self.filter_store = output * self.damp2 + self.filter_store * self.damp1;
        self.buffer[self.buf_idx] = input + self.filter_store * self.feedback;
        self.buf_idx = (self.buf_idx + 1) % self.buf_size;
        output
    }
}

#[derive(Clone, Debug)]
struct AllpassFilter {
    buffer: Vec<f32>,
    buf_size: usize,
    buf_idx: usize,
    feedback: f32,
}

impl AllpassFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            buf_size: size,
            buf_idx: 0,
            feedback: 0.5,
        }
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.buf_idx = 0;
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        let buf_out = self.buffer[self.buf_idx];
        let output = -input + buf_out;
        self.buffer[self.buf_idx] = input + (buf_out * self.feedback);
        self.buf_idx = (self.buf_idx + 1) % self.buf_size;
        output
    }
}

#[derive(Clone, Debug)]
pub struct StereoReverb {
    pub enabled: bool,
    pub room_size: f32, // 0.0 to 1.0
    pub damping: f32,   // 0.0 to 1.0
    pub width: f32,     // 0.0 to 1.0
    pub mix: f32,       // 0.0 to 1.0

    last_room_size: f32,
    last_damping: f32,
    combs_l: Vec<CombFilter>,
    combs_r: Vec<CombFilter>,
    allpasses_l: Vec<AllpassFilter>,
    allpasses_r: Vec<AllpassFilter>,
    sample_rate: f32,
}

impl Default for StereoReverb {
    fn default() -> Self {
        let mut rev = Self {
            enabled: false,
            room_size: 0.75,
            damping: 0.3,
            width: 1.0,
            mix: 0.3,
            last_room_size: -1.0,
            last_damping: -1.0,
            combs_l: COMB_TUNING_L.iter().map(|&sz| CombFilter::new(sz)).collect(),
            combs_r: COMB_TUNING_R.iter().map(|&sz| CombFilter::new(sz)).collect(),
            allpasses_l: ALLPASS_TUNING_L.iter().map(|&sz| AllpassFilter::new(sz)).collect(),
            allpasses_r: ALLPASS_TUNING_R.iter().map(|&sz| AllpassFilter::new(sz)).collect(),
            sample_rate: 44100.0,
        };
        rev.update_params();
        rev
    }
}

impl StereoReverb {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1000.0);
    }

    pub fn reset(&mut self) {
        for c in &mut self.combs_l { c.reset(); }
        for c in &mut self.combs_r { c.reset(); }
        for a in &mut self.allpasses_l { a.reset(); }
        for a in &mut self.allpasses_r { a.reset(); }
    }

    pub fn update_params(&mut self) {
        let fb = 0.7 + self.room_size.clamp(0.0, 1.0) * 0.28;
        let damp = self.damping.clamp(0.0, 1.0) * 0.4;

        for c in &mut self.combs_l {
            c.set_feedback(fb);
            c.set_damp(damp);
        }
        for c in &mut self.combs_r {
            c.set_feedback(fb);
            c.set_damp(damp);
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        if !self.enabled || self.mix <= 0.001 {
            return (in_l, in_r);
        }

        if (self.room_size - self.last_room_size).abs() > 0.001 || (self.damping - self.last_damping).abs() > 0.001 {
            self.last_room_size = self.room_size;
            self.last_damping = self.damping;
            self.update_params();
        }

        let input = (in_l + in_r) * 0.015;

        // Parallel comb filters
        let mut out_l = 0.0;
        let mut out_r = 0.0;
        for c in &mut self.combs_l {
            out_l += c.process(input);
        }
        for c in &mut self.combs_r {
            out_r += c.process(input);
        }

        // Series allpass diffusers
        for a in &mut self.allpasses_l {
            out_l = a.process(out_l);
        }
        for a in &mut self.allpasses_r {
            out_r = a.process(out_r);
        }

        let wet_1 = self.mix * ((self.width / 2.0) + 0.5);
        let wet_2 = self.mix * ((1.0 - self.width) / 2.0);
        let dry = 1.0 - self.mix;

        let res_l = out_l * wet_1 + out_r * wet_2 + in_l * dry;
        let res_r = out_r * wet_1 + out_l * wet_2 + in_r * dry;

        (res_l, res_r)
    }
}
