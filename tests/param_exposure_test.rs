//! Host parameter exposure tests.
//!
//! A DAW can only automate, MIDI-learn or remote-control parameters that the
//! plugin actually exposes. These tests pin down that *every* control in the
//! editor exists as a host parameter with a stable, unique ID.

use nih_plug::prelude::{Param, Params};
use replicantwave::params::{ReplicantWaveParams, WAVETABLE_COUNT};

/// Every control the editor draws must be exposed to the host.
const REQUIRED_IDS: &[&str] = &[
    // master + legacy filter IDs kept for automation compatibility
    "gain",
    "cutoff",
    "resonance",
    // OSC 1
    "osc1_on",
    "osc1_table",
    "osc1_pos",
    "osc1_tune",
    "osc1_fine",
    "osc1_unison",
    "osc1_detune",
    "osc1_spread",
    "osc1_formant",
    "osc1_bend",
    "osc1_level",
    "osc1_pan",
    // OSC 2
    "osc2_on",
    "osc2_table",
    "osc2_pos",
    "osc2_tune",
    "osc2_fine",
    "osc2_unison",
    "osc2_detune",
    "osc2_spread",
    "osc2_formant",
    "osc2_bend",
    "osc2_level",
    "osc2_pan",
    // sub + noise
    "sub_on",
    "sub_wave",
    "sub_oct",
    "sub_level",
    "sub_pan",
    "noise_on",
    "noise_type",
    "noise_level",
    "noise_pan",
    // filter
    "flt_on",
    "flt_type",
    "flt_drive",
    "flt_key",
    "flt_vel",
    "flt_envmod",
    "flt_route_osc1",
    "flt_route_osc2",
    "flt_route_sub",
    "flt_route_noise",
    // envelopes
    "amp_atk",
    "amp_dec",
    "amp_sus",
    "amp_rel",
    "fenv_atk",
    "fenv_dec",
    "fenv_sus",
    "fenv_rel",
    // fx
    "dist_on",
    "dist_mode",
    "dist_amount",
    "dist_mix",
    "chorus_on",
    "chorus_rate",
    "chorus_depth",
    "chorus_mix",
    "delay_on",
    "delay_sync",
    "delay_pingpong",
    "delay_time",
    "delay_fb",
    "delay_damp",
    "delay_mix",
    "verb_on",
    "verb_size",
    "verb_damp",
    "verb_mix",
    "gate_on",
    // sequencer
    "seq_run",
    "seq_sync",
    "seq_key",
    "seq_mode",
    "seq_len",
    "seq_rate",
    "seq_swing",
    "seq_bpm",
    "seq_oct",
    "seq_root",
];

/// `ParamPtr::name()` returns the display name; map it back to the stable ID.
fn name_to_id(params: &ReplicantWaveParams) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for (id, ptr, _) in params.param_map() {
        let name = unsafe { ptr.name().to_string() };
        assert!(
            map.insert(name.clone(), id.clone()).is_none(),
            "duplicate display name '{name}' — hosts show these in the generic UI"
        );
    }
    map
}

fn map_ids() -> Vec<String> {
    let params = ReplicantWaveParams::default();
    params
        .param_map()
        .into_iter()
        .map(|(id, _, _)| id)
        .collect()
}

#[test]
fn every_editor_control_is_a_host_parameter() {
    let ids = map_ids();
    let missing: Vec<&str> = REQUIRED_IDS
        .iter()
        .copied()
        .filter(|id| !ids.iter().any(|have| have == id))
        .collect();

    assert!(
        missing.is_empty(),
        "these controls are not exposed to the host: {missing:?}"
    );
}

#[test]
fn parameter_ids_are_unique_and_named() {
    let params = ReplicantWaveParams::default();
    let map = params.param_map();
    assert!(
        map.len() >= REQUIRED_IDS.len(),
        "expected at least {} parameters, got {}",
        REQUIRED_IDS.len(),
        map.len()
    );

    println!("host-exposed parameters: {}", map.len());

    let mut seen = std::collections::HashSet::new();
    for (id, ptr, _) in &map {
        assert!(!id.is_empty(), "empty parameter id");
        assert!(seen.insert(id.clone()), "duplicate parameter id: {id}");
        let name = unsafe { ptr.name() };
        assert!(!name.is_empty(), "parameter {id} has an empty name");
        let default_norm = unsafe { ptr.default_normalized_value() };
        assert!(
            (0.0..=1.0).contains(&default_norm),
            "parameter {id} has an out-of-range default ({default_norm})"
        );
    }
}

#[test]
fn wavetable_selector_covers_the_whole_bank() {
    let params = ReplicantWaveParams::default();
    let default = params.osc1.table.value();
    assert!((0..WAVETABLE_COUNT).contains(&default));
    // The bank must be fully selectable from the host.
    let bank = replicantwave::audio::wavetable::WavetableBank::new();
    assert_eq!(
        bank.tables.len() as i32,
        WAVETABLE_COUNT,
        "WAVETABLE_COUNT is out of sync with the wavetable bank"
    );
}

#[test]
fn defaults_drive_a_silent_engine_until_a_note_arrives() {
    let params = ReplicantWaveParams::default();
    let mut engine = replicantwave::audio::SynthEngine::default();
    engine.set_sample_rate(48_000.0);
    params.sync_to_engine(&mut engine);

    assert!(
        !engine.sequencer.running,
        "transport must default to stopped"
    );
    assert!(!engine.sequencer.host_sync, "DAW sync must be opt-in");
    assert!(!engine.sequencer.key_trigger, "key trigger must be opt-in");

    let mut l = vec![0.0f32; 256];
    let mut r = vec![0.0f32; 256];
    for _ in 0..20 {
        engine.process_block(&mut l, &mut r, 120.0, false);
    }
    let peak = l.iter().chain(r.iter()).fold(0.0f32, |m, s| m.max(s.abs()));
    assert!(
        peak < 1e-6,
        "engine must be silent without notes, peak {peak}"
    );
}

#[test]
fn parameter_sync_reaches_every_engine_block() {
    let params = ReplicantWaveParams::default();
    let mut engine = replicantwave::audio::SynthEngine::default();
    engine.set_sample_rate(48_000.0);
    params.sync_to_engine(&mut engine);

    // Defaults must match the documented engine defaults.
    assert_eq!(
        engine.voices[0].osc1_table_idx,
        params.osc1.table.value() as usize
    );
    assert_eq!(
        engine.voices[0].osc2_table_idx,
        params.osc2.table.value() as usize
    );
    assert!((engine.master_gain - params.gain.value()).abs() < 1e-6);
    assert!((engine.voices[0].filter.cutoff_hz - params.cutoff.value()).abs() < 1e-3);
    assert!((engine.voices[0].filter.resonance - params.resonance.value()).abs() < 1e-6);
    assert_eq!(
        engine.sequencer.steps.len(),
        replicantwave::audio::step_sequencer::MAX_STEPS
    );

    // And the pattern is copied from the persisted state.
    let mut seen = 0u64;
    {
        let mut pattern = params.pattern.lock();
        pattern.steps[3].gate = true;
        pattern.steps[3].note_offset = 7;
        pattern.version += 1;
    }
    params.apply_pattern_if_changed(&mut engine, &mut seen);
    assert_eq!(engine.sequencer.steps[3].note_offset, 7);
}

// ---------------------------------------------------------------------------
// Preset -> parameter path
// ---------------------------------------------------------------------------

use nih_plug::prelude::{GuiContext, ParamPtr, ParamSetter, PluginApi, PluginState};
use parking_lot::Mutex;
use std::sync::Arc;

/// Minimal `GuiContext` that records every parameter write instead of touching
/// a host. This lets us exercise the exact code path the editor uses.
struct RecordingContext {
    writes: Mutex<Vec<(String, f32)>>,
}

impl GuiContext for RecordingContext {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Clap
    }
    fn request_resize(&self) -> bool {
        true
    }
    unsafe fn raw_begin_set_parameter(&self, _param: ParamPtr) {}
    unsafe fn raw_set_parameter_normalized(&self, param: ParamPtr, normalized: f32) {
        self.writes
            .lock()
            .push((param.name().to_string(), normalized));
    }
    unsafe fn raw_end_set_parameter(&self, _param: ParamPtr) {}
    fn get_state(&self) -> PluginState {
        unimplemented!("not used by this test")
    }
    fn set_state(&self, _state: PluginState) {
        unimplemented!("not used by this test")
    }
}

#[test]
fn every_preset_writes_every_control_through_the_host_path() {
    let params = Arc::new(ReplicantWaveParams::default());
    let ctx = Arc::new(RecordingContext {
        writes: Mutex::new(Vec::new()),
    });
    let setter = ParamSetter::new(ctx.as_ref());

    let presets = replicantwave::presets::factory::get_factory_presets();
    // Performance/session settings a patch deliberately leaves alone.
    const OPTIONAL: &[&str] = &[
        "gain", "seq_sync", "seq_key", "seq_bpm", "seq_oct", "seq_root",
    ];
    let expected: std::collections::HashSet<&str> = REQUIRED_IDS
        .iter()
        .copied()
        .filter(|id| !OPTIONAL.contains(id))
        .collect();
    let names = name_to_id(&params);

    for preset in &presets {
        ctx.writes.lock().clear();
        preset.apply_to_params(&setter, &params);

        let writes = ctx.writes.lock();
        let mut seen = std::collections::HashSet::new();
        for (name, norm) in writes.iter() {
            let id = names
                .get(name)
                .unwrap_or_else(|| panic!("{}: unknown parameter '{name}'", preset.name));
            assert!(
                norm.is_finite(),
                "{}: {id} normalized is not finite",
                preset.name
            );
            assert!(
                (-0.001..=1.001).contains(norm),
                "{}: {id} normalized out of range ({norm})",
                preset.name
            );
            seen.insert(id.clone());
        }

        let missing: Vec<&str> = expected
            .iter()
            .copied()
            .filter(|id| !seen.contains(*id))
            .collect();
        assert!(
            missing.is_empty(),
            "preset '{}' does not write: {missing:?}",
            preset.name
        );
    }
}

#[test]
fn preset_values_land_on_the_right_parameters() {
    let params = Arc::new(ReplicantWaveParams::default());
    let ctx = Arc::new(RecordingContext {
        writes: Mutex::new(Vec::new()),
    });
    let setter = ParamSetter::new(ctx.as_ref());
    let preset = &replicantwave::presets::factory::get_factory_presets()[0];

    preset.apply_to_params(&setter, &params);
    let writes = ctx.writes.lock();
    let names = name_to_id(&params);
    let normalized = |id: &str| -> f32 {
        writes
            .iter()
            .rev()
            .find(|(name, _)| names.get(name).map(|v| v.as_str()) == Some(id))
            .map(|(_, n)| *n)
            .unwrap_or_else(|| panic!("preset never wrote {id}"))
    };

    // A handful of fields with distinct values, to catch transposed mappings.
    let checks: [(&str, f32); 6] = [
        (
            "cutoff",
            params.cutoff.preview_normalized(preset.filter_cutoff),
        ),
        (
            "resonance",
            params.resonance.preview_normalized(preset.filter_reso),
        ),
        (
            "osc1_pos",
            params.osc1.position.preview_normalized(preset.osc1_pos),
        ),
        (
            "osc1_level",
            params.osc1.level.preview_normalized(preset.osc1_level),
        ),
        (
            "dist_amount",
            params.dist.amount.preview_normalized(preset.dist_amount),
        ),
        (
            "verb_mix",
            params.reverb.mix.preview_normalized(preset.reverb_mix),
        ),
    ];
    for (id, expected) in checks {
        let got = normalized(id);
        assert!(
            (got - expected).abs() < 1e-4,
            "{id}: wrote {got}, expected {expected} (preset {})",
            preset.name
        );
    }

    // The transport must never be armed by a patch.
    let run = writes
        .iter()
        .rev()
        .find(|(n, _)| names.get(n).map(|v| v.as_str()) == Some("seq_run"))
        .map(|(_, v)| *v);
    assert_eq!(
        run,
        Some(0.0),
        "loading a preset must not start the sequencer"
    );
}
