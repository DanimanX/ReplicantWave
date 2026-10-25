//! Offline audio QA for every factory preset.
//!
//! Each patch is loaded into a fresh engine, a chord is played, and the rendered
//! stereo signal is checked for NaNs, silence, runaway loudness, DC offset and
//! hard-clipping. Run with `cargo test --test preset_audio_test -- --nocapture`
//! to see the per-preset report.

use replicantwave::audio::SynthEngine;
use replicantwave::presets::factory::get_factory_presets;
use replicantwave::presets::SynthPreset;

const SR: f32 = 48_000.0;
const BLOCK: usize = 256;

#[derive(Clone, Debug)]
struct Report {
    name: String,
    category: String,
    peak: f32,
    rms: f32,
    dc: f32,
    clipped: f32,
    max_jump: f32,
}

fn render_preset(preset: &SynthPreset) -> Report {
    let mut engine = SynthEngine::default();
    engine.set_sample_rate(SR);
    preset.apply_to_engine(&mut engine);
    engine.sequencer.running = false;
    engine.sequencer.mode = replicantwave::audio::step_sequencer::SeqMode::Off;

    // Play each family the way it is actually used: mono basses, single-note
    // leads and arps, sustained chords for pads and drones.
    let cat = preset.category.to_ascii_lowercase();
    let notes: &[u8] = match cat.as_str() {
        "bass" | "sub" => &[36],
        "lead" | "acid" | "stab" | "sync" => &[60],
        "arp" | "sequence" | "combat" | "rave" => &[48],
        "pad" | "choir" | "atmosphere" | "drone" => &[36, 43, 48, 55],
        _ => &[48, 55],
    };
    for (i, note) in notes.iter().enumerate() {
        engine.note_on(*note, 0.82 - i as f32 * 0.03);
    }

    let total_blocks = ((SR * 3.0) as usize) / BLOCK;
    let is_fx = preset.category.eq_ignore_ascii_case("FX");
    let skip_blocks = if is_fx {
        ((SR * 0.05) as usize) / BLOCK
    } else {
        ((SR * 0.6) as usize) / BLOCK
    };
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];

    let mut peak = 0.0f32;
    let mut sum_sq = 0.0f64;
    let mut sum = 0.0f64;
    let mut n = 0usize;
    let mut clipped = 0usize;
    let mut max_jump = 0.0f32;
    let mut last = 0.0f32;
    let mut non_finite = 0usize;

    // Percussive patches (sustain = 0) are played like an arpeggiator would:
    // retrigger every 250 ms so the QA measures the sound, not the silence
    // between one-shot hits.
    let retrigger = preset.amp_sustain < 0.08 && !is_fx;
    let retrigger_blocks = ((SR * 0.25) as usize / BLOCK).max(1);

    for block in 0..total_blocks {
        if retrigger && block > 0 && block % retrigger_blocks == 0 {
            for note in notes {
                engine.note_off(*note);
            }
            for (i, note) in notes.iter().enumerate() {
                engine.note_on(*note, 0.82 - i as f32 * 0.03);
            }
        }
        engine.process_block(&mut l, &mut r, 120.0, false);
        for i in 0..BLOCK {
            let s = (l[i] + r[i]) * 0.5;
            if !s.is_finite() {
                non_finite += 1;
                continue;
            }
            if block >= skip_blocks {
                peak = peak.max(s.abs());
                sum_sq += (s as f64) * (s as f64);
                sum += s as f64;
                n += 1;
                if s.abs() >= 0.999 {
                    clipped += 1;
                }
                let jump = (s - last).abs();
                if jump > max_jump {
                    max_jump = jump;
                }
            }
            last = s;
        }
    }

    assert_eq!(non_finite, 0, "{} produced NaN/Inf samples", preset.name);

    let rms = if n > 0 {
        (sum_sq / n as f64).sqrt() as f32
    } else {
        0.0
    };
    let dc = if n > 0 { (sum / n as f64) as f32 } else { 0.0 };
    let clipped_pct = if n > 0 {
        clipped as f32 * 100.0 / n as f32
    } else {
        0.0
    };

    Report {
        name: preset.name.clone(),
        category: preset.category.clone(),
        peak,
        rms,
        dc,
        clipped: clipped_pct,
        max_jump,
    }
}

#[test]
fn every_factory_preset_renders_clean_audio() {
    let presets = get_factory_presets();
    assert!(
        presets.len() >= 36,
        "expected the expansion bank, got {}",
        presets.len()
    );

    let mut failures = Vec::new();

    println!(
        "\n{:<30} {:<12} {:>7} {:>7} {:>8} {:>7} {:>7}",
        "PRESET", "CATEGORY", "PEAK", "RMS", "DC", "CLIP%", "JUMP"
    );
    println!("{}", "-".repeat(86));

    for preset in &presets {
        let report = render_preset(preset);
        println!(
            "{:<30} {:<12} {:>7.3} {:>7.3} {:>8.5} {:>6.2}% {:>7.3}",
            report.name,
            report.category,
            report.peak,
            report.rms,
            report.dc,
            report.clipped,
            report.max_jump
        );

        // Hard requirements: the plugin must never emit invalid values, must be
        // audible, must not slam the output into the limiter for long periods,
        // and must stay DC-free.
        let mut problems = Vec::new();
        if report.rms < 0.004 {
            problems.push(format!("too quiet (RMS {:.4})", report.rms));
        }
        if report.rms > 0.55 {
            problems.push(format!("too loud (RMS {:.3})", report.rms));
        }
        if report.peak > 1.0001 {
            problems.push(format!("peak above 0 dBFS ({:.3})", report.peak));
        }
        if report.peak < 0.01 {
            problems.push(format!("effectively silent (peak {:.4})", report.peak));
        }
        if report.dc.abs() > 0.02 {
            problems.push(format!("DC offset ({:+.4})", report.dc));
        }
        if report.clipped > 2.0 {
            problems.push(format!("clipping {:.1}% of samples", report.clipped));
        }
        if !problems.is_empty() {
            failures.push(format!("{}: {}", report.name, problems.join("; ")));
        }
    }

    println!("{}", "-".repeat(86));
    if !failures.is_empty() {
        panic!("preset QA failed:\n  - {}", failures.join("\n  - "));
    }
}

#[test]
fn expansion_bank_covers_all_requested_categories() {
    let presets = get_factory_presets();
    let count = |needle: &str| {
        presets
            .iter()
            .filter(|p| {
                p.category.eq_ignore_ascii_case(needle)
                    || (needle.eq_ignore_ascii_case("Bass")
                        && p.category.eq_ignore_ascii_case("Sub"))
            })
            .count()
    };

    assert!(count("Pad") >= 6, "need more pads, got {}", count("Pad"));
    assert!(
        count("Drone") >= 4,
        "need more drones, got {}",
        count("Drone")
    );
    assert!(
        count("Bass") >= 6,
        "need more bass patches, got {}",
        count("Bass")
    );
    assert!(count("Lead") >= 6, "need more leads, got {}", count("Lead"));
    assert!(count("Arp") >= 3, "need more arps, got {}", count("Arp"));
    assert!(
        count("Sequence") >= 4,
        "need more sequences, got {}",
        count("Sequence")
    );
    assert!(
        count("FX") >= 2,
        "need more FX patches, got {}",
        count("FX")
    );
}

#[test]
fn loud_chord_is_smoothly_limited_not_hard_clipped() {
    // Force a very hot patch: maximum master, all oscillators, high drive.
    let mut preset = get_factory_presets()[0].clone();
    preset.osc1_level = 1.5;
    preset.osc2_level = 1.5;
    preset.osc1_unison = 8;
    preset.osc2_unison = 8;
    preset.sub_level = 1.0;
    preset.dist_enabled = true;
    preset.dist_amount = 1.0;
    preset.dist_mix = 1.0;
    preset.reverb_enabled = true;
    preset.reverb_mix = 0.5;
    preset.delay_enabled = true;
    preset.delay_mix = 0.5;
    preset.delay_fb = 0.9;

    let mut engine = SynthEngine::default();
    engine.set_sample_rate(SR);
    preset.apply_to_engine(&mut engine);
    engine.master_gain = 1.5;
    engine.sequencer.running = false;
    engine.sequencer.mode = replicantwave::audio::step_sequencer::SeqMode::Off;

    for note in [24u8, 36, 48, 60, 72] {
        engine.note_on(note, 1.0);
    }

    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut over = 0usize;
    let mut total = 0usize;
    for _ in 0..((SR * 2.0) as usize / BLOCK) {
        engine.process_block(&mut l, &mut r, 120.0, false);
        for i in 0..BLOCK {
            assert!(l[i].is_finite() && r[i].is_finite());
            if l[i].abs() > 1.0 || r[i].abs() > 1.0 {
                over += 1;
            }
            total += 1;
        }
    }
    assert_eq!(over, 0, "the output stage must never exceed 0 dBFS");
    assert_eq!(total, (SR * 2.0) as usize);
}
