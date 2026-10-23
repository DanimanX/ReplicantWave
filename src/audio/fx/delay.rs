//! Stereo Ping-Pong Delay with tempo sync and feedback damping.

const MAX_DELAY_SAMPLES: usize = 192000; // ~4 seconds at 48kHz

#[derive(Clone, Debug)]
pub struct StereoDelay {
    pub enabled: bool,
    pub sync: bool,
    pub beat_division: f32, // 0.25 = 1/16th, 0.5 = 1/8th, 0.75 = 1/8 dotted, 1.0 = 1/4
    pub time_ms: f32,       // 1.0 to 1000.0 ms
    pub feedback: f32,      // 0.0 to 0.95
    pub damping: f32,       // 0.0 to 1.0 (lowpass filter on feedback)
    pub ping_pong: bool,
    pub mix: f32,           // 0.0 to 1.0

    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_idx: usize,
    damp_state_l: f32,
    damp_state_r: f32,
    sample_rate: f32,
}

impl Default for StereoDelay {
    fn default() -> Self {
        Self {
            enabled: false,
            sync: true,
            beat_division: 0.5, // 1/8 note default
            time_ms: 300.0,
            feedback: 0.45,
            damping: 0.3,
            ping_pong: true,
            mix: 0.35,
            buffer_l: vec![0.0; MAX_DELAY_SAMPLES],
            buffer_r: vec![0.0; MAX_DELAY_SAMPLES],
            write_idx: 0,
            damp_state_l: 0.0,
            damp_state_r: 0.0,
            sample_rate: 44100.0,
        }
    }
}

impl StereoDelay {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1000.0);
    }

    pub fn reset(&mut self) {
        self.buffer_l.fill(0.0);
        self.buffer_r.fill(0.0);
        self.write_idx = 0;
        self.damp_state_l = 0.0;
        self.damp_state_r = 0.0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32, host_bpm: f32) -> (f32, f32) {
        if !self.enabled || self.mix <= 0.001 {
            return (in_l, in_r);
        }

        let sr = self.sample_rate;
        let buf_len = self.buffer_l.len();

        let delay_sec = if self.sync && host_bpm > 20.0 {
            let beats_per_sec = host_bpm / 60.0;
            self.beat_division / beats_per_sec
        } else {
            self.time_ms * 0.001
        };

        let delay_samples = (delay_sec * sr).clamp(10.0, (buf_len - 100) as f32);

        // Read delay lines with fast indexing
        let mut read_pos = self.write_idx as f32 + buf_len as f32 - delay_samples;
        if read_pos >= buf_len as f32 {
            read_pos -= buf_len as f32;
        }
        let idx0 = read_pos as usize;
        let idx1 = if idx0 + 1 >= buf_len { 0 } else { idx0 + 1 };
        let frac = read_pos - idx0 as f32;

        let wet_l = self.buffer_l[idx0] + frac * (self.buffer_l[idx1] - self.buffer_l[idx0]);
        let wet_r = self.buffer_r[idx0] + frac * (self.buffer_r[idx1] - self.buffer_r[idx0]);

        // Damping lowpass filter
        let damp_coeff = (1.0 - self.damping.clamp(0.0, 0.95)).clamp(0.05, 1.0);
        self.damp_state_l += damp_coeff * (wet_l - self.damp_state_l);
        self.damp_state_r += damp_coeff * (wet_r - self.damp_state_r);

        let filtered_l = self.damp_state_l;
        let filtered_r = self.damp_state_r;

        let fb = self.feedback.clamp(0.0, 0.95);

        // Ping-pong cross feedback or stereo feedback
        let (next_in_l, next_in_r) = if self.ping_pong {
            (in_l + filtered_r * fb, in_r + filtered_l * fb)
        } else {
            (in_l + filtered_l * fb, in_r + filtered_r * fb)
        };

        self.buffer_l[self.write_idx] = next_in_l;
        self.buffer_r[self.write_idx] = next_in_r;

        self.write_idx += 1;
        if self.write_idx >= buf_len {
            self.write_idx = 0;
        }

        let dry_gain = 1.0 - self.mix * 0.5;
        let wet_gain = self.mix;

        (in_l * dry_gain + wet_l * wet_gain, in_r * dry_gain + wet_r * wet_gain)
    }
}
