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
    assert!(visited_steps.len() > 16, "Sequencer should advance across steps, visited: {}", visited_steps.len());
}

#[test]
fn test_sequencer_midi_transpose() {
    let mut engine = SynthEngine::default();
    engine.set_sample_rate(44100.0);
    engine.sequencer.mode = SeqMode::InternalSynth;
    engine.sequencer.key_trigger = true;

    // Simulate MIDI note on (note 60 = C4)
    engine.handle_midi_note_on(60, 0.8);
    assert_eq!(engine.sequencer.root_note, 60);
    assert!(engine.sequencer.running);

    // Simulate MIDI note on (note 72 = C5)
    engine.handle_midi_note_on(72, 0.9);
    assert_eq!(engine.sequencer.root_note, 72);
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
    assert!(produced_audio, "Manual PLAY must produce sound even when host_playing is false!");
    assert!(engine.sequencer.current_step > 0, "Sequencer step should advance!");

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
    let active_voices = engine.active_voice_count.load(std::sync::atomic::Ordering::Relaxed);
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

    let start = std::time::Instant::now();
    let num_blocks = 200; // ~2.3 seconds of audio

    for _ in 0..num_blocks {
        engine.process_block(&mut out_l, &mut out_r, 125.0, true);
    }

    let elapsed = start.elapsed();
    println!("Processed 200 blocks (102,400 samples) across 12 voices in {:?}", elapsed);
    assert!(elapsed.as_millis() < 250, "DSP processing must be ultra-fast! Elapsed: {:?}", elapsed);
}
