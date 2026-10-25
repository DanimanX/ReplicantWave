//! Regression tests for the click/crackle fixes: voice stealing, FX bypass
//! crossfades and the master saturation stage.

use replicantwave::audio::fx::FxChain;
use replicantwave::audio::voice::Voice;
use replicantwave::audio::wavetable::WavetableBank;

const SR: f32 = 48_000.0;

fn max_jump(samples: &[f32]) -> f32 {
    samples
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max)
}

#[test]
fn stolen_voice_fades_before_retrigger() {
    let mut voice = Voice::default();
    voice.set_sample_rate(SR);
    voice.osc1_table_idx = 0;
    voice.osc1.position = 0.62; // sine-ish frame of the Analog Morph table
    voice.osc1.unison = 1;
    voice.osc1.level_gain = 0.6;
    voice.osc2.enabled = false;
    voice.sub.enabled = false;
    voice.noise.enabled = false;
    voice.filter.cutoff_hz = 18_000.0;
    voice.filter.resonance = 0.0;
    voice.filter.env_amount = 0.0;
    voice.amp_env.attack_s = 0.01;
    voice.amp_env.decay_s = 1.0;
    voice.amp_env.sustain_level = 0.9;
    voice.amp_env.release_s = 5.0;

    let bank = WavetableBank::new();
    voice.note_on(60, 0.8, false, 0.0, 1);

    let mut out = Vec::new();
    for _ in 0..(SR * 0.25) as usize {
        let (l, r) = voice.process_sample(&bank, 120.0, 0.0, 0.0);
        out.push((l + r) * 0.5);
    }
    let before = max_jump(&out);
    assert!(before < 0.1, "baseline waveform jumps too much: {before}");

    // Force a steal while the old note is at full sustain.
    voice.steal_to(67, 0.8, 2);
    out.clear();
    for _ in 0..(SR * 0.05) as usize {
        let (l, r) = voice.process_sample(&bank, 120.0, 0.0, 0.0);
        out.push((l + r) * 0.5);
    }

    let during = max_jump(&out);
    assert!(
        during < 0.12,
        "voice steal introduced a discontinuity: {during}"
    );
    assert_eq!(voice.active_midi_note, 67, "the new note must be playing");
}

#[test]
fn fx_bypass_crossfades_do_not_click() {
    let mut fx = FxChain::default();
    fx.set_sample_rate(SR);

    // Turn everything on with deliberately extreme (but valid) settings.
    fx.distortion.enabled = true;
    fx.distortion.mode = replicantwave::audio::fx::distortion::DistortionMode::SoftOverdrive;
    fx.distortion.amount = 0.7;
    fx.distortion.mix = 0.8;
    fx.chorus.enabled = true;
    fx.chorus.rate_hz = 2.5;
    fx.chorus.depth_ms = 4.0;
    fx.chorus.mix = 0.6;
    fx.delay.enabled = true;
    fx.delay.sync = false;
    fx.delay.time_ms = 120.0;
    fx.delay.feedback = 0.55;
    fx.delay.mix = 0.5;
    fx.reverb.enabled = true;
    fx.reverb.room_size = 0.8;
    fx.reverb.mix = 0.5;
    fx.gate.enabled = true;

    let mut samples = Vec::with_capacity(48_000);
    let mut toggles = 0u32;
    for i in 0..48_000 {
        if i > 0 && i % 2_400 == 0 {
            toggles += 1;
            let on = toggles % 2 == 0;
            fx.distortion.enabled = on;
            fx.chorus.enabled = on;
            fx.delay.enabled = on;
            fx.reverb.enabled = on;
            fx.gate.enabled = on;
        }
        let phase = (i as f32 * 440.0 / SR * std::f32::consts::TAU).sin() * 0.35;
        let (l, r) = fx.process_sample(phase, phase, 120.0);
        let s = (l + r) * 0.5;
        assert!(s.is_finite(), "FX produced a non-finite sample");
        samples.push(s);
    }

    let jump = max_jump(&samples);
    assert!(
        jump < 0.35,
        "FX enable/disable produced a click-sized discontinuity: {jump}"
    );
}

#[test]
fn master_gain_smoothing_is_not_a_step() {
    // A raw gain change from 0.6 to 0.1 on a steady sine must ramp, not step.
    let mut engine = replicantwave::audio::SynthEngine::default();
    engine.set_sample_rate(SR);
    for v in engine.voices.iter_mut() {
        v.osc1_table_idx = 0;
        v.osc1.position = 0.62;
        v.osc1.unison = 1;
        v.osc1.level_gain = 0.5;
        v.osc2.enabled = false;
        v.sub.enabled = false;
        v.noise.enabled = false;
        v.filter.cutoff_hz = 18_000.0;
        v.filter.env_amount = 0.0;
        v.amp_env.attack_s = 0.01;
        v.amp_env.sustain_level = 0.9;
        v.amp_env.release_s = 5.0;
    }
    engine.note_on(60, 0.7);

    let mut l = vec![0.0f32; 256];
    let mut r = vec![0.0f32; 256];
    for _ in 0..40 {
        engine.process_block(&mut l, &mut r, 120.0, false);
    }

    engine.master_gain = 0.1;
    let mut out = Vec::new();
    for _ in 0..20 {
        engine.process_block(&mut l, &mut r, 120.0, false);
        out.extend_from_slice(&l);
    }
    let jump = max_jump(&out);
    assert!(jump < 0.12, "master gain smoothing stepped: {jump}");
}
