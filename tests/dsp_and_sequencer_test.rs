use replicantwave::audio::step_sequencer::{SeqMode, SeqStepEvent, StepSequencer};
use replicantwave::audio::SynthEngine;

#[test]
fn test_sequencer_64_steps_cycling() {
    let mut seq = StepSequencer::default();
    seq.set_sample_rate(44100.0);
    seq.length = 64;
    seq.running = true;
    seq.mode = SeqMode::InternalSynth;
    seq.reset();

    let mut trigger_count = 0;
    let mut visited_steps = std::collections::HashSet::new();

    // Run for 3 seconds of audio at 125 BPM
    let total_samples = 44100 * 3;
    for _ in 0..total_samples {
        visited_steps.insert(seq.current_step);
        match seq.process_sample(125.0, true) {
            SeqStepEvent::Trigger { .. } => {
                trigger_count += 1;
            }
            _ => {}
        }
    }

    assert!(trigger_count > 10, "Sequencer should trigger steps");
    assert!(
        visited_steps.len() > 16,
        "Sequencer should advance across steps, visited: {}",
        visited_steps.len()
    );
}

#[test]
fn test_midi_notes_never_start_sequencer() {
    let mut engine = SynthEngine::default();
    engine.set_sample_rate(44100.0);
    engine.sequencer.mode = SeqMode::InternalSynth;
    // Both defaults and KEY TRIG must not turn a MIDI note into a transport start.
    assert!(!engine.sequencer.key_trigger);
    assert!(!engine.sequencer.running);

    engine.handle_midi_note_on(60, 0.8);
    assert!(
        !engine.sequencer.running,
        "MIDI note-on must never start the sequencer"
    );
    assert!(
        engine
            .voices
            .iter()
            .any(|v| v.is_active() && v.active_midi_note == 60),
        "MIDI note-on must play a synth voice"
    );

    engine.handle_midi_note_off(60);

    // Even with key trigger enabled, a stopped sequencer stays stopped.
    engine.sequencer.key_trigger = true;
    engine.handle_midi_note_on(64, 0.8);
    assert!(
        !engine.sequencer.running,
        "KEY TRIG must not start a stopped sequencer"
    );
    assert!(
        engine
            .voices
            .iter()
            .any(|v| v.is_active() && v.active_midi_note == 64),
        "MIDI note-on must still play the synth when KEY TRIG is armed"
    );
}

#[test]
fn test_midi_transposes_only_a_running_sequencer() {
    let mut engine = SynthEngine::default();
    engine.set_sample_rate(44100.0);
    engine.sequencer.mode = SeqMode::InternalSynth;
    engine.sequencer.key_trigger = true;
    engine.sequencer.running = true;

    engine.handle_midi_note_on(72, 0.9);
    assert_eq!(engine.sequencer.root_note, 72);
    assert!(engine.sequencer.running);
}

#[test]
fn test_loading_presets_never_starts_transport() {
    use replicantwave::presets::factory::get_factory_presets;

    for preset in get_factory_presets() {
        let mut engine = SynthEngine::default();
        engine.set_sample_rate(48000.0);
        preset.apply_to_engine(&mut engine);
        assert!(
            !engine.sequencer.running,
            "preset '{}' started the sequencer on load",
            preset.name
        );
        assert!(
            engine.sequencer.host_sync == false,
            "preset '{}' must not force DAW sync on load",
            preset.name
        );
    }
}

#[test]
fn test_manual_play_button_toggle() {
    let mut engine = SynthEngine::default();
    engine.set_sample_rate(44100.0);

    // Initial state: sequencer stopped, host transport stopped (standalone or paused DAW)
    let host_bpm = 0.0;
    let host_playing = false;

    // Simulate clicking PLAY button in GUI:
    engine.sequencer.running = true;
    if engine.sequencer.mode == SeqMode::Off {
        engine.sequencer.mode = SeqMode::InternalSynth;
    }
    engine.sequencer.reset();

    let mut out_l = [0.0f32; 512];
    let mut out_r = [0.0f32; 512];

    // Process blocks while manual PLAY is active and host transport is false
    let mut produced_audio = false;
    for _ in 0..50 {
        engine.process_block(&mut out_l, &mut out_r, host_bpm, host_playing);
        for &s in out_l.iter() {
            if s.abs() > 0.0001 {
                produced_audio = true;
                break;
            }
        }
    }
    assert!(
        produced_audio,
        "Manual PLAY must produce sound even when host_playing is false!"
    );
    assert!(
        engine.sequencer.current_step > 0,
        "Sequencer step should advance!"
    );

    // Simulate clicking STOP button in GUI:
    engine.sequencer.running = false;
    for v in engine.voices.iter_mut() {
        v.note_off();
    }

    // Process remaining release tails (~3 seconds of audio to let 300ms exponential tail decay completely)
    for _ in 0..300 {
        engine.process_block(&mut out_l, &mut out_r, host_bpm, host_playing);
    }

    // Active voice count should return to 0
    let active_voices = engine
        .active_voice_count
        .load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(active_voices, 0, "All voices must be silenced after STOP!");
}

#[test]
fn test_dsp_performance_benchmark() {
    let mut engine = SynthEngine::default();
    engine.set_sample_rate(44100.0);

    // Trigger multiple voices with unison
    for i in 0..12 {
        engine.note_on(48 + i * 2, 0.8);
    }

    let mut out_l = [0.0f32; 512];
    let mut out_r = [0.0f32; 512];

    let num_blocks = 200; // ~2.3 seconds of audio

    // Warm up: the first pass pays for page faults and cold caches, which are
    // not representative of steady-state DSP cost.
    for _ in 0..num_blocks {
        engine.process_block(&mut out_l, &mut out_r, 125.0, true);
    }

    // The test harness runs tests in parallel, so a single wall-clock sample
    // can be inflated by scheduler/CPU contention. Take the best of three
    // passes: real regressions slow down every pass, transient noise does not.
    let mut best = std::time::Duration::MAX;
    for _ in 0..3 {
        let start = std::time::Instant::now();
        for _ in 0..num_blocks {
            engine.process_block(&mut out_l, &mut out_r, 125.0, true);
        }
        best = best.min(start.elapsed());
    }

    println!(
        "Processed {} blocks ({} samples) across 12 voices in {:?} (best of 3)",
        num_blocks,
        num_blocks * 512,
        best
    );
    assert!(
        best.as_millis() < 250,
        "DSP processing must be ultra-fast! Elapsed: {:?}",
        best
    );
}
