//! 38 Cyberpunk soundbank factory presets (16 core + 22 NIGHT CITY expansion).

use super::SynthPreset;
use crate::audio::step_sequencer::{Step, MAX_STEPS};

pub fn get_factory_presets() -> Vec<SynthPreset> {
    vec![
        // 1. Night City Bass
        preset_night_city_bass(),
        // 2. Arasaka Matrix Pad
        preset_arasaka_pad(),
        // 3. Neon Blade 2049
        preset_neon_blade_lead(),
        // 4. Chrome Acid 303
        preset_chrome_acid_303(),
        // 5. Netrunner Arp
        preset_netrunner_arp(),
        // 6. Darksynth Power Stab
        preset_darksynth_stab(),
        // 7. Corpo Tower Dystopia
        preset_corpo_dystopia(),
        // 8. Megacity Combat 2077
        preset_megacity_combat(),
        // 9. Ghost In The Wire Choir
        preset_ghost_wire_choir(),
        // 10. Subdermal 808 Rumble
        preset_subdermal_808(),
        // 11. Synthetic Tears Pad
        preset_synthetic_tears(),
        // 12. Cyber Glitch 64-Step
        preset_cyber_glitch_seq(),
        // 13. Kiroshi Scanner Sequence
        preset_kiroshi_scanner(),
        // 14. Overclocked Sync Screamer
        preset_overclocked_sync(),
        // 15. Afterlife Future Rave
        preset_afterlife_rave(),
        // 16. Braindance Memory
        preset_braindance_memory(),
        // ---- NIGHT CITY EXPANSION PACK ----
        // Pads
        preset_neon_rain_pad(),
        preset_corpo_cathedral(),
        preset_ghost_protocol_choir(),
        preset_cryo_sleep_pad(),
        // Drones
        preset_blackwall_drone(),
        preset_toxic_fog_drone(),
        preset_singularity_hum(),
        preset_arasaka_orbital_drone(),
        // Bass
        preset_combat_zone_bass(),
        preset_chrome_sub_assault(),
        preset_neuro_reaper_bass(),
        preset_acid_rain_bass(),
        // Leads
        preset_nightside_razor_lead(),
        preset_laser_katana(),
        preset_neon_samurai_lead(),
        preset_void_screamer_lead(),
        // Arps / sequences
        preset_data_stream_arp(),
        preset_kiroshi_rain_arp(),
        preset_afterlife_run(),
        preset_braindance_64(),
        // FX
        preset_emp_burst(),
        preset_cyberpsychosis(),
    ]
}

fn create_default_steps(pattern: &[(i8, u8, bool, bool, bool)]) -> Vec<Step> {
    // Unused steps must stay silent; `Step::default()` has `gate: true`, which
    // used to leave ghost notes in the 64-step range of shorter patterns.
    let mut steps = vec![
        Step {
            gate: false,
            ..Step::default()
        };
        MAX_STEPS
    ];
    for (i, p) in pattern.iter().enumerate().take(MAX_STEPS) {
        steps[i] = Step {
            note_offset: p.0,
            velocity: p.1,
            gate: p.2,
            slide: p.3,
            accent: p.4,
            tie: false,
        };
    }
    steps
}

fn preset_night_city_bass() -> SynthPreset {
    let seq = create_default_steps(&[
        (0, 110, true, false, true),
        (0, 95, true, false, false),
        (3, 100, true, true, false),
        (7, 120, true, false, true),
        (0, 90, true, false, false),
        (10, 105, true, false, false),
        (12, 115, true, true, true),
        (7, 95, true, false, false),
        (0, 110, true, false, true),
        (0, 95, true, false, false),
        (2, 100, true, false, false),
        (3, 105, true, true, false),
        (5, 110, true, false, true),
        (7, 95, true, false, false),
        (12, 120, true, true, true),
        (0, 90, true, false, false),
    ]);

    SynthPreset {
        name: "Night City Bass".into(),
        category: "Bass".into(),
        author: "DanimanX".into(),
        description: "Heavy rolling neuro-bass with resonant filter drive and tape crunch.".into(),
        osc1_table: 1, // Cyber Neuro
        osc1_enabled: true,
        osc1_unison: 4,
        osc1_tune: 0.0,
        osc1_fine: 0.0,
        osc1_level: 0.9,
        osc1_pos: 0.35,
        osc1_detune: 0.15,
        osc1_spread: 0.8,
        osc1_pan: 0.0,
        osc1_formant: 0.1,
        osc1_bend: 0.0,
        osc2_table: 7, // Sub Zero
        osc2_enabled: true,
        osc2_unison: 1,
        osc2_tune: -12.0,
        osc2_fine: 0.0,
        osc2_level: 0.75,
        osc2_pos: 0.5,
        osc2_detune: 0.0,
        osc2_spread: 0.0,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: true,
        sub_wave: 3, // Pulse50
        sub_octave: -1,
        sub_level: 0.45,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1, // LowPass24
        filter_cutoff: 950.0,
        filter_reso: 0.38,
        filter_drive: 2.2,
        filter_key: 0.25,
        filter_vel: 0.4,
        filter_env_amount: 0.65,
        amp_attack: 0.005,
        amp_decay: 0.25,
        amp_sustain: 0.6,
        amp_release: 0.2,
        filter_attack: 0.008,
        filter_decay: 0.32,
        filter_sustain: 0.25,
        filter_release: 0.25,
        dist_enabled: true,
        dist_mode: 2, // Fire Amp
        dist_amount: 0.45,
        dist_mix: 0.75,
        chorus_enabled: false,
        chorus_rate: 1.0,
        chorus_depth: 3.0,
        chorus_mix: 0.0,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 250.0,
        delay_fb: 0.35,
        delay_damp: 0.4,
        delay_mix: 0.2,
        reverb_enabled: false,
        reverb_size: 0.5,
        reverb_damp: 0.3,
        reverb_mix: 0.15,
        gate_enabled: false,
        seq_running: true,
        seq_mode: 0,
        seq_length: 16,
        seq_rate: 3, // 1/16
        seq_swing: 0.1,
        seq_steps: seq,
    }
}

fn preset_arasaka_pad() -> SynthPreset {
    SynthPreset {
        name: "Arasaka Matrix Pad".into(),
        category: "Pad".into(),
        author: "DanimanX".into(),
        description:
            "Lush dystopian corporate soundscape with stereo chorus and shimmering reverb.".into(),
        osc1_table: 3, // Vocaloid Synth
        osc1_enabled: true,
        osc1_unison: 6,
        osc1_tune: 0.0,
        osc1_fine: -5.0,
        osc1_level: 0.85,
        osc1_pos: 0.55,
        osc1_detune: 0.22,
        osc1_spread: 1.0,
        osc1_pan: -0.1,
        osc1_formant: 0.2,
        osc1_bend: 0.0,
        osc2_table: 5, // Dystopian Steel
        osc2_enabled: true,
        osc2_unison: 4,
        osc2_tune: 12.0,
        osc2_fine: 6.0,
        osc2_level: 0.7,
        osc2_pos: 0.3,
        osc2_detune: 0.18,
        osc2_spread: 0.9,
        osc2_pan: 0.1,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: true,
        sub_wave: 0, // Sine
        sub_octave: -1,
        sub_level: 0.35,
        sub_pan: 0.0,
        noise_enabled: true,
        noise_type: 1, // Pink
        noise_level: 0.12,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 0, // LowPass12
        filter_cutoff: 2200.0,
        filter_reso: 0.15,
        filter_drive: 1.2,
        filter_key: 0.5,
        filter_vel: 0.3,
        filter_env_amount: 0.35,
        amp_attack: 0.45,
        amp_decay: 0.8,
        amp_sustain: 0.85,
        amp_release: 1.4,
        filter_attack: 0.6,
        filter_decay: 1.2,
        filter_sustain: 0.7,
        filter_release: 1.5,
        dist_enabled: false,
        dist_mode: 0,
        dist_amount: 0.1,
        dist_mix: 0.0,
        chorus_enabled: true,
        chorus_rate: 0.8,
        chorus_depth: 6.5,
        chorus_mix: 0.65,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 400.0,
        delay_fb: 0.5,
        delay_damp: 0.35,
        delay_mix: 0.35,
        reverb_enabled: true,
        reverb_size: 0.88,
        reverb_damp: 0.25,
        reverb_mix: 0.55,
        gate_enabled: false,
        seq_running: false,
        seq_mode: 3, // Off
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: vec![Step::default(); MAX_STEPS],
    }
}

fn preset_neon_blade_lead() -> SynthPreset {
    SynthPreset {
        name: "Neon Blade 2049".into(),
        category: "Lead".into(),
        author: "DanimanX".into(),
        description: "Heroic Vangelis-inspired brass/saw lead soaring above rainy neon towers."
            .into(),
        osc1_table: 0, // Analog Morph
        osc1_enabled: true,
        osc1_unison: 5,
        osc1_tune: 0.0,
        osc1_fine: -3.0,
        osc1_level: 0.95,
        osc1_pos: 0.08,
        osc1_detune: 0.18,
        osc1_spread: 0.85,
        osc1_pan: 0.0,
        osc1_formant: 0.0,
        osc1_bend: 0.1,
        osc2_table: 2, // Digital FM
        osc2_enabled: true,
        osc2_unison: 3,
        osc2_tune: 0.0,
        osc2_fine: 4.0,
        osc2_level: 0.8,
        osc2_pos: 0.45,
        osc2_detune: 0.12,
        osc2_spread: 0.7,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: false,
        sub_wave: 0,
        sub_octave: 0,
        sub_level: 0.0,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1, // LowPass24
        filter_cutoff: 3400.0,
        filter_reso: 0.45,
        filter_drive: 1.8,
        filter_key: 0.7,
        filter_vel: 0.5,
        filter_env_amount: 0.5,
        amp_attack: 0.04,
        amp_decay: 0.5,
        amp_sustain: 0.8,
        amp_release: 0.4,
        filter_attack: 0.06,
        filter_decay: 0.6,
        filter_sustain: 0.5,
        filter_release: 0.5,
        dist_enabled: true,
        dist_mode: 0, // Overdrive
        dist_amount: 0.35,
        dist_mix: 0.6,
        chorus_enabled: true,
        chorus_rate: 1.2,
        chorus_depth: 3.5,
        chorus_mix: 0.4,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 350.0,
        delay_fb: 0.48,
        delay_damp: 0.3,
        delay_mix: 0.4,
        reverb_enabled: true,
        reverb_size: 0.78,
        reverb_damp: 0.3,
        reverb_mix: 0.42,
        gate_enabled: false,
        seq_running: false,
        seq_mode: 3,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: vec![Step::default(); MAX_STEPS],
    }
}

fn preset_chrome_acid_303() -> SynthPreset {
    let seq = create_default_steps(&[
        (0, 100, true, false, false),
        (0, 127, true, true, true),
        (12, 110, true, false, false),
        (3, 127, true, true, true),
        (7, 95, true, false, false),
        (0, 100, true, false, false),
        (10, 127, true, true, true),
        (12, 105, true, false, false),
        (0, 100, true, false, false),
        (15, 127, true, true, true),
        (14, 110, true, false, false),
        (12, 127, true, true, true),
        (7, 95, true, false, false),
        (5, 100, true, false, false),
        (3, 127, true, true, true),
        (2, 95, true, false, false),
    ]);

    SynthPreset {
        name: "Chrome Acid 303".into(),
        category: "Acid".into(),
        author: "DanimanX".into(),
        description: "Iconic screaming resonant TB-style acid squelch with portamento slides."
            .into(),
        osc1_table: 4, // Cyber Acid 303
        osc1_enabled: true,
        osc1_unison: 1,
        osc1_tune: 0.0,
        osc1_fine: 0.0,
        osc1_level: 1.0,
        osc1_pos: 0.65,
        osc1_detune: 0.0,
        osc1_spread: 0.0,
        osc1_pan: 0.0,
        osc1_formant: 0.0,
        osc1_bend: 0.0,
        osc2_table: 0,
        osc2_enabled: false,
        osc2_unison: 1,
        osc2_tune: 0.0,
        osc2_fine: 0.0,
        osc2_level: 0.0,
        osc2_pos: 0.0,
        osc2_detune: 0.0,
        osc2_spread: 0.0,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: true,
        sub_wave: 3, // Pulse50
        sub_octave: -1,
        sub_level: 0.55,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1, // LowPass24
        filter_cutoff: 650.0,
        filter_reso: 0.88,
        filter_drive: 2.8,
        filter_key: 0.4,
        filter_vel: 0.6,
        filter_env_amount: 0.92,
        amp_attack: 0.003,
        amp_decay: 0.22,
        amp_sustain: 0.0,
        amp_release: 0.15,
        filter_attack: 0.005,
        filter_decay: 0.24,
        filter_sustain: 0.0,
        filter_release: 0.18,
        dist_enabled: true,
        dist_mode: 0, // Overdrive
        dist_amount: 0.62,
        dist_mix: 0.85,
        chorus_enabled: false,
        chorus_rate: 1.0,
        chorus_depth: 2.0,
        chorus_mix: 0.0,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 250.0,
        delay_fb: 0.38,
        delay_damp: 0.45,
        delay_mix: 0.3,
        reverb_enabled: true,
        reverb_size: 0.55,
        reverb_damp: 0.4,
        reverb_mix: 0.25,
        gate_enabled: false,
        seq_running: true,
        seq_mode: 0,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.05,
        seq_steps: seq,
    }
}

fn preset_netrunner_arp() -> SynthPreset {
    let mut pattern = Vec::new();
    let arps = [0, 3, 7, 10, 12, 15, 19, 22, 24, 22, 19, 15, 12, 10, 7, 3];
    for &note in &arps {
        pattern.push((note, 105, true, false, note >= 12));
    }
    let seq = create_default_steps(&pattern);

    SynthPreset {
        name: "Netrunner Arp".into(),
        category: "Arp".into(),
        author: "DanimanX".into(),
        description: "Bouncing 16th-note FM plucks cascading into a stereo cyber-delay.".into(),
        osc1_table: 2, // Digital FM
        osc1_enabled: true,
        osc1_unison: 2,
        osc1_tune: 0.0,
        osc1_fine: -4.0,
        osc1_level: 0.9,
        osc1_pos: 0.72,
        osc1_detune: 0.08,
        osc1_spread: 0.6,
        osc1_pan: 0.0,
        osc1_formant: 0.0,
        osc1_bend: 0.0,
        osc2_table: 6, // PWM Hyper
        osc2_enabled: true,
        osc2_unison: 2,
        osc2_tune: 12.0,
        osc2_fine: 4.0,
        osc2_level: 0.65,
        osc2_pos: 0.35,
        osc2_detune: 0.1,
        osc2_spread: 0.8,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: false,
        sub_wave: 0,
        sub_octave: 0,
        sub_level: 0.0,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 4, // BandPass12
        filter_cutoff: 1800.0,
        filter_reso: 0.55,
        filter_drive: 1.4,
        filter_key: 0.8,
        filter_vel: 0.5,
        filter_env_amount: 0.55,
        amp_attack: 0.003,
        amp_decay: 0.16,
        amp_sustain: 0.0,
        amp_release: 0.1,
        filter_attack: 0.005,
        filter_decay: 0.18,
        filter_sustain: 0.0,
        filter_release: 0.12,
        dist_enabled: true,
        dist_mode: 0,
        dist_amount: 0.2,
        dist_mix: 0.4,
        chorus_enabled: false,
        chorus_rate: 1.5,
        chorus_depth: 3.0,
        chorus_mix: 0.0,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 250.0,
        delay_fb: 0.55,
        delay_damp: 0.3,
        delay_mix: 0.45,
        reverb_enabled: true,
        reverb_size: 0.65,
        reverb_damp: 0.3,
        reverb_mix: 0.35,
        gate_enabled: false,
        seq_running: true,
        seq_mode: 0,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: seq,
    }
}

fn preset_darksynth_stab() -> SynthPreset {
    SynthPreset {
        name: "Darksynth Power Stab".into(),
        category: "Stab".into(),
        author: "DanimanX".into(),
        description: "Aggressive 8-voice supersaw power stab for hard electronic drops.".into(),
        osc1_table: 0, // Analog Morph
        osc1_enabled: true,
        osc1_unison: 8,
        osc1_tune: 0.0,
        osc1_fine: 0.0,
        osc1_level: 1.0,
        osc1_pos: 0.02,
        osc1_detune: 0.35,
        osc1_spread: 1.0,
        osc1_pan: 0.0,
        osc1_formant: 0.0,
        osc1_bend: 0.0,
        osc2_table: 1, // Cyber Neuro
        osc2_enabled: true,
        osc2_unison: 4,
        osc2_tune: -12.0,
        osc2_fine: 0.0,
        osc2_level: 0.85,
        osc2_pos: 0.4,
        osc2_detune: 0.2,
        osc2_spread: 0.8,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: true,
        sub_wave: 2, // Saw
        sub_octave: -1,
        sub_level: 0.6,
        sub_pan: 0.0,
        noise_enabled: true,
        noise_type: 0,
        noise_level: 0.1,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1, // LowPass24
        filter_cutoff: 2800.0,
        filter_reso: 0.3,
        filter_drive: 2.4,
        filter_key: 0.3,
        filter_vel: 0.5,
        filter_env_amount: 0.7,
        amp_attack: 0.005,
        amp_decay: 0.35,
        amp_sustain: 0.2,
        amp_release: 0.3,
        filter_attack: 0.008,
        filter_decay: 0.38,
        filter_sustain: 0.15,
        filter_release: 0.3,
        dist_enabled: true,
        dist_mode: 2, // Fire Amp
        dist_amount: 0.55,
        dist_mix: 0.8,
        chorus_enabled: false,
        chorus_rate: 1.0,
        chorus_depth: 3.0,
        chorus_mix: 0.0,
        delay_enabled: false,
        delay_sync: true,
        delay_time: 250.0,
        delay_fb: 0.3,
        delay_damp: 0.3,
        delay_mix: 0.2,
        reverb_enabled: true,
        reverb_size: 0.6,
        reverb_damp: 0.4,
        reverb_mix: 0.3,
        gate_enabled: false,
        seq_running: false,
        seq_mode: 3,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: vec![Step::default(); MAX_STEPS],
    }
}

fn preset_corpo_dystopia() -> SynthPreset {
    SynthPreset {
        name: "Corpo Tower Dystopia".into(),
        category: "Drone".into(),
        author: "DanimanX".into(),
        description: "Dark metallic industrial drone with notch sweep and infinite diffusion."
            .into(),
        osc1_table: 5, // Dystopian Steel
        osc1_enabled: true,
        osc1_unison: 4,
        osc1_tune: 0.0,
        osc1_fine: -8.0,
        osc1_level: 0.9,
        osc1_pos: 0.75,
        osc1_detune: 0.25,
        osc1_spread: 0.9,
        osc1_pan: -0.2,
        osc1_formant: 0.3,
        osc1_bend: 0.0,
        osc2_table: 3, // Vocaloid Synth
        osc2_enabled: true,
        osc2_unison: 3,
        osc2_tune: 7.0,
        osc2_fine: 5.0,
        osc2_level: 0.8,
        osc2_pos: 0.45,
        osc2_detune: 0.15,
        osc2_spread: 0.8,
        osc2_pan: 0.2,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: true,
        sub_wave: 0,
        sub_octave: -2,
        sub_level: 0.5,
        sub_pan: 0.0,
        noise_enabled: true,
        noise_type: 2, // CyberBit
        noise_level: 0.08,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 6, // Notch
        filter_cutoff: 1400.0,
        filter_reso: 0.65,
        filter_drive: 1.5,
        filter_key: 0.2,
        filter_vel: 0.2,
        filter_env_amount: 0.4,
        amp_attack: 0.8,
        amp_decay: 1.5,
        amp_sustain: 0.9,
        amp_release: 2.0,
        filter_attack: 1.2,
        filter_decay: 2.0,
        filter_sustain: 0.8,
        filter_release: 2.5,
        dist_enabled: true,
        dist_mode: 0,
        dist_amount: 0.25,
        dist_mix: 0.45,
        chorus_enabled: true,
        chorus_rate: 0.5,
        chorus_depth: 8.0,
        chorus_mix: 0.6,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 500.0,
        delay_fb: 0.6,
        delay_damp: 0.4,
        delay_mix: 0.4,
        reverb_enabled: true,
        reverb_size: 0.95,
        reverb_damp: 0.2,
        reverb_mix: 0.65,
        gate_enabled: false,
        seq_running: false,
        seq_mode: 3,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: vec![Step::default(); MAX_STEPS],
    }
}

fn preset_megacity_combat() -> SynthPreset {
    let seq = create_default_steps(&[
        (0, 127, true, false, true),
        (0, 100, true, false, false),
        (0, 110, true, false, false),
        (0, 127, true, false, true),
        (3, 120, true, true, true),
        (0, 95, true, false, false),
        (0, 110, true, false, false),
        (0, 127, true, false, true),
        (0, 127, true, false, true),
        (0, 100, true, false, false),
        (5, 120, true, true, true),
        (0, 95, true, false, false),
        (0, 127, true, false, true),
        (0, 100, true, false, false),
        (7, 127, true, true, true),
        (0, 110, true, false, false),
    ]);

    SynthPreset {
        name: "Megacity Combat 2077".into(),
        category: "Combat".into(),
        author: "DanimanX".into(),
        description:
            "Relentless mid-tempo cyber-industrial bass with wavefolded fuzz and pump gate.".into(),
        osc1_table: 1, // Cyber Neuro
        osc1_enabled: true,
        osc1_unison: 3,
        osc1_tune: 0.0,
        osc1_fine: 0.0,
        osc1_level: 1.0,
        osc1_pos: 0.88,
        osc1_detune: 0.22,
        osc1_spread: 0.7,
        osc1_pan: 0.0,
        osc1_formant: 0.15,
        osc1_bend: 0.2,
        osc2_table: 7, // Sub Zero
        osc2_enabled: true,
        osc2_unison: 1,
        osc2_tune: -12.0,
        osc2_fine: 0.0,
        osc2_level: 0.9,
        osc2_pos: 0.6,
        osc2_detune: 0.0,
        osc2_spread: 0.0,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: true,
        sub_wave: 2,
        sub_octave: -1,
        sub_level: 0.6,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1,
        filter_cutoff: 1200.0,
        filter_reso: 0.45,
        filter_drive: 3.0,
        filter_key: 0.3,
        filter_vel: 0.5,
        filter_env_amount: 0.75,
        amp_attack: 0.003,
        amp_decay: 0.2,
        amp_sustain: 0.3,
        amp_release: 0.15,
        filter_attack: 0.005,
        filter_decay: 0.22,
        filter_sustain: 0.1,
        filter_release: 0.18,
        dist_enabled: true,
        dist_mode: 3, // Grind Amp
        dist_amount: 0.72,
        dist_mix: 0.9,
        chorus_enabled: false,
        chorus_rate: 1.0,
        chorus_depth: 3.0,
        chorus_mix: 0.0,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 200.0,
        delay_fb: 0.3,
        delay_damp: 0.5,
        delay_mix: 0.2,
        reverb_enabled: true,
        reverb_size: 0.5,
        reverb_damp: 0.4,
        reverb_mix: 0.2,
        gate_enabled: true,
        seq_running: true,
        seq_mode: 0,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: seq,
    }
}

fn preset_ghost_wire_choir() -> SynthPreset {
    SynthPreset {
        name: "Ghost In The Wire Choir".into(),
        category: "Choir".into(),
        author: "DanimanX".into(),
        description: "Ethereal vocal formant choir floating through high-density diffusion.".into(),
        osc1_table: 3, // Vocaloid Synth
        osc1_enabled: true,
        osc1_unison: 6,
        osc1_tune: 0.0,
        osc1_fine: 0.0,
        osc1_level: 0.9,
        osc1_pos: 0.48,
        osc1_detune: 0.2,
        osc1_spread: 1.0,
        osc1_pan: -0.1,
        osc1_formant: 0.1,
        osc1_bend: 0.0,
        osc2_table: 0, // Analog Morph
        osc2_enabled: true,
        osc2_unison: 4,
        osc2_tune: 12.0,
        osc2_fine: 5.0,
        osc2_level: 0.65,
        osc2_pos: 0.5,
        osc2_detune: 0.15,
        osc2_spread: 0.9,
        osc2_pan: 0.1,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: false,
        sub_wave: 0,
        sub_octave: 0,
        sub_level: 0.0,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 0, // LowPass12
        filter_cutoff: 4200.0,
        filter_reso: 0.18,
        filter_drive: 1.1,
        filter_key: 0.6,
        filter_vel: 0.4,
        filter_env_amount: 0.3,
        amp_attack: 0.25,
        amp_decay: 0.8,
        amp_sustain: 0.85,
        amp_release: 1.2,
        filter_attack: 0.35,
        filter_decay: 1.0,
        filter_sustain: 0.7,
        filter_release: 1.2,
        dist_enabled: false,
        dist_mode: 0,
        dist_amount: 0.1,
        dist_mix: 0.0,
        chorus_enabled: true,
        chorus_rate: 0.9,
        chorus_depth: 7.0,
        chorus_mix: 0.7,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 380.0,
        delay_fb: 0.52,
        delay_damp: 0.3,
        delay_mix: 0.35,
        reverb_enabled: true,
        reverb_size: 0.86,
        reverb_damp: 0.25,
        reverb_mix: 0.55,
        gate_enabled: false,
        seq_running: false,
        seq_mode: 3,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: vec![Step::default(); MAX_STEPS],
    }
}

fn preset_subdermal_808() -> SynthPreset {
    SynthPreset {
        name: "Subdermal 808 Rumble".into(),
        category: "Sub".into(),
        author: "DanimanX".into(),
        description: "Chest-rattling 808 sub bass with saturated low harmonics.".into(),
        osc1_table: 7, // Sub Zero
        osc1_enabled: true,
        osc1_unison: 1,
        osc1_tune: 0.0,
        osc1_fine: 0.0,
        osc1_level: 1.0,
        osc1_pos: 0.15,
        osc1_detune: 0.0,
        osc1_spread: 0.0,
        osc1_pan: 0.0,
        osc1_formant: 0.0,
        osc1_bend: 0.0,
        osc2_table: 0,
        osc2_enabled: false,
        osc2_unison: 1,
        osc2_tune: 0.0,
        osc2_fine: 0.0,
        osc2_level: 0.0,
        osc2_pos: 0.0,
        osc2_detune: 0.0,
        osc2_spread: 0.0,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: true,
        sub_wave: 0, // Sine
        sub_octave: -1,
        sub_level: 0.85,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1, // LowPass24
        filter_cutoff: 400.0,
        filter_reso: 0.15,
        filter_drive: 2.0,
        filter_key: 0.1,
        filter_vel: 0.3,
        filter_env_amount: 0.4,
        amp_attack: 0.005,
        amp_decay: 0.6,
        amp_sustain: 0.75,
        amp_release: 0.35,
        filter_attack: 0.008,
        filter_decay: 0.4,
        filter_sustain: 0.3,
        filter_release: 0.3,
        dist_enabled: true,
        dist_mode: 0, // Overdrive
        dist_amount: 0.35,
        dist_mix: 0.55,
        chorus_enabled: false,
        chorus_rate: 1.0,
        chorus_depth: 2.0,
        chorus_mix: 0.0,
        delay_enabled: false,
        delay_sync: true,
        delay_time: 200.0,
        delay_fb: 0.2,
        delay_damp: 0.5,
        delay_mix: 0.0,
        reverb_enabled: false,
        reverb_size: 0.4,
        reverb_damp: 0.5,
        reverb_mix: 0.0,
        gate_enabled: false,
        seq_running: false,
        seq_mode: 3,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: vec![Step::default(); MAX_STEPS],
    }
}

fn preset_synthetic_tears() -> SynthPreset {
    SynthPreset {
        name: "Synthetic Tears Pad".into(),
        category: "Pad".into(),
        author: "DanimanX".into(),
        description: "Melancholic CS-80 inspired synthwave brass pad soaked in tape delay.".into(),
        osc1_table: 0, // Analog Morph
        osc1_enabled: true,
        osc1_unison: 4,
        osc1_tune: 0.0,
        osc1_fine: -4.0,
        osc1_level: 0.9,
        osc1_pos: 0.18,
        osc1_detune: 0.14,
        osc1_spread: 0.8,
        osc1_pan: 0.0,
        osc1_formant: 0.0,
        osc1_bend: 0.05,
        osc2_table: 6, // PWM Hyper
        osc2_enabled: true,
        osc2_unison: 3,
        osc2_tune: 0.0,
        osc2_fine: 4.0,
        osc2_level: 0.8,
        osc2_pos: 0.4,
        osc2_detune: 0.12,
        osc2_spread: 0.7,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: false,
        sub_wave: 0,
        sub_octave: 0,
        sub_level: 0.0,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1, // LowPass24
        filter_cutoff: 1600.0,
        filter_reso: 0.28,
        filter_drive: 1.3,
        filter_key: 0.65,
        filter_vel: 0.4,
        filter_env_amount: 0.45,
        amp_attack: 0.15,
        amp_decay: 0.6,
        amp_sustain: 0.75,
        amp_release: 0.8,
        filter_attack: 0.2,
        filter_decay: 0.8,
        filter_sustain: 0.55,
        filter_release: 0.9,
        dist_enabled: false,
        dist_mode: 0,
        dist_amount: 0.15,
        dist_mix: 0.0,
        chorus_enabled: true,
        chorus_rate: 1.1,
        chorus_depth: 4.0,
        chorus_mix: 0.5,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 320.0,
        delay_fb: 0.45,
        delay_damp: 0.35,
        delay_mix: 0.35,
        reverb_enabled: true,
        reverb_size: 0.75,
        reverb_damp: 0.3,
        reverb_mix: 0.45,
        gate_enabled: false,
        seq_running: false,
        seq_mode: 3,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: vec![Step::default(); MAX_STEPS],
    }
}

fn preset_cyber_glitch_seq() -> SynthPreset {
    let mut pattern = Vec::new();
    let intervals = [0, 12, 3, 15, 7, 19, 10, 22, 0, 14, 2, 17, 5, 12, 7, 0];
    for (i, &note) in intervals.iter().cycle().take(64).enumerate() {
        let gate = i % 2 == 0 || i % 7 == 0 || i % 5 == 0;
        let slide = i % 8 == 6;
        let accent = i % 4 == 0;
        pattern.push((note, 110, gate, slide, accent));
    }
    let seq = create_default_steps(&pattern);

    SynthPreset {
        name: "Cyber Glitch 64-Step".into(),
        category: "Sequence".into(),
        author: "DanimanX".into(),
        description: "64-step fast bitcrushed modular sequence using sliding bar navigation."
            .into(),
        osc1_table: 2, // Digital FM
        osc1_enabled: true,
        osc1_unison: 1,
        osc1_tune: 0.0,
        osc1_fine: 0.0,
        osc1_level: 0.78,
        osc1_pos: 0.85,
        osc1_detune: 0.0,
        osc1_spread: 0.0,
        osc1_pan: 0.0,
        osc1_formant: 0.0,
        osc1_bend: 0.0,
        osc2_table: 5, // Dystopian Steel
        osc2_enabled: true,
        osc2_unison: 1,
        osc2_tune: 19.0,
        osc2_fine: 0.0,
        osc2_level: 0.6,
        osc2_pos: 0.7,
        osc2_detune: 0.0,
        osc2_spread: 0.0,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: false,
        sub_wave: 0,
        sub_octave: 0,
        sub_level: 0.0,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1,
        filter_cutoff: 2400.0,
        filter_reso: 0.6,
        filter_drive: 1.7,
        filter_key: 0.5,
        filter_vel: 0.6,
        filter_env_amount: 0.7,
        amp_attack: 0.002,
        amp_decay: 0.12,
        amp_sustain: 0.0,
        amp_release: 0.08,
        filter_attack: 0.004,
        filter_decay: 0.15,
        filter_sustain: 0.0,
        filter_release: 0.1,
        dist_enabled: true,
        dist_mode: 1, // BitCrusher
        dist_amount: 0.65,
        dist_mix: 0.85,
        chorus_enabled: false,
        chorus_rate: 1.0,
        chorus_depth: 2.0,
        chorus_mix: 0.0,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 200.0,
        delay_fb: 0.45,
        delay_damp: 0.4,
        delay_mix: 0.35,
        reverb_enabled: true,
        reverb_size: 0.5,
        reverb_damp: 0.4,
        reverb_mix: 0.25,
        gate_enabled: false,
        seq_running: true,
        seq_mode: 0,
        seq_length: 64, // Full 64 steps!
        seq_rate: 3,
        seq_swing: 0.08,
        seq_steps: seq,
    }
}

fn preset_kiroshi_scanner() -> SynthPreset {
    let mut pattern = Vec::new();
    let intervals = [0, 0, 12, 0, 7, 0, 10, 0, 3, 5, 7, 12, 10, 7, 5, 3];
    for (i, &note) in intervals.iter().cycle().take(64).enumerate() {
        pattern.push((note, 105, i % 2 == 0 || i % 3 == 0, i % 4 == 3, i % 4 == 0));
    }
    let seq = create_default_steps(&pattern);

    SynthPreset {
        name: "Kiroshi Scanner Sequence".into(),
        category: "Sequence".into(),
        author: "DanimanX".into(),
        description: "Tactical cybernetic scanner audio sweep with 64-step bandpass modulation."
            .into(),
        osc1_table: 6, // PWM Hyper
        osc1_enabled: true,
        osc1_unison: 2,
        osc1_tune: 0.0,
        osc1_fine: -5.0,
        osc1_level: 0.95,
        osc1_pos: 0.6,
        osc1_detune: 0.1,
        osc1_spread: 0.7,
        osc1_pan: 0.0,
        osc1_formant: 0.1,
        osc1_bend: 0.0,
        osc2_table: 4, // Cyber Acid 303
        osc2_enabled: true,
        osc2_unison: 2,
        osc2_tune: 7.0,
        osc2_fine: 5.0,
        osc2_level: 0.75,
        osc2_pos: 0.4,
        osc2_detune: 0.12,
        osc2_spread: 0.8,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: false,
        sub_wave: 0,
        sub_octave: 0,
        sub_level: 0.0,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 5, // BandPass24
        filter_cutoff: 2100.0,
        filter_reso: 0.72,
        filter_drive: 2.1,
        filter_key: 0.7,
        filter_vel: 0.5,
        filter_env_amount: 0.65,
        amp_attack: 0.003,
        amp_decay: 0.18,
        amp_sustain: 0.0,
        amp_release: 0.12,
        filter_attack: 0.005,
        filter_decay: 0.2,
        filter_sustain: 0.0,
        filter_release: 0.15,
        dist_enabled: true,
        dist_mode: 0,
        dist_amount: 0.35,
        dist_mix: 0.6,
        chorus_enabled: false,
        chorus_rate: 1.0,
        chorus_depth: 2.0,
        chorus_mix: 0.0,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 250.0,
        delay_fb: 0.5,
        delay_damp: 0.35,
        delay_mix: 0.4,
        reverb_enabled: true,
        reverb_size: 0.65,
        reverb_damp: 0.3,
        reverb_mix: 0.3,
        gate_enabled: false,
        seq_running: true,
        seq_mode: 0,
        seq_length: 64, // 64 steps
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: seq,
    }
}

fn preset_overclocked_sync() -> SynthPreset {
    SynthPreset {
        name: "Overclocked Sync Screamer".into(),
        category: "Lead".into(),
        author: "DanimanX".into(),
        description: "Hard-synced screamer lead that cuts cleanly through dense cyberpunk mixes."
            .into(),
        osc1_table: 0, // Analog Morph
        osc1_enabled: true,
        osc1_unison: 4,
        osc1_tune: 0.0,
        osc1_fine: 0.0,
        osc1_level: 0.95,
        osc1_pos: 0.1,
        osc1_detune: 0.18,
        osc1_spread: 0.85,
        osc1_pan: 0.0,
        osc1_formant: 0.65,
        osc1_bend: 0.75,
        osc2_table: 2, // Digital FM
        osc2_enabled: true,
        osc2_unison: 2,
        osc2_tune: 12.0,
        osc2_fine: 0.0,
        osc2_level: 0.75,
        osc2_pos: 0.5,
        osc2_detune: 0.1,
        osc2_spread: 0.6,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: true,
        sub_wave: 2,
        sub_octave: -1,
        sub_level: 0.4,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1,
        filter_cutoff: 4500.0,
        filter_reso: 0.5,
        filter_drive: 2.5,
        filter_key: 0.8,
        filter_vel: 0.5,
        filter_env_amount: 0.5,
        amp_attack: 0.005,
        amp_decay: 0.4,
        amp_sustain: 0.7,
        amp_release: 0.3,
        filter_attack: 0.008,
        filter_decay: 0.45,
        filter_sustain: 0.45,
        filter_release: 0.35,
        dist_enabled: true,
        dist_mode: 2, // Fire Amp
        dist_amount: 0.5,
        dist_mix: 0.75,
        chorus_enabled: false,
        chorus_rate: 1.0,
        chorus_depth: 3.0,
        chorus_mix: 0.0,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 250.0,
        delay_fb: 0.4,
        delay_damp: 0.35,
        delay_mix: 0.35,
        reverb_enabled: true,
        reverb_size: 0.7,
        reverb_damp: 0.3,
        reverb_mix: 0.35,
        gate_enabled: false,
        seq_running: false,
        seq_mode: 3,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: vec![Step::default(); MAX_STEPS],
    }
}

fn preset_afterlife_rave() -> SynthPreset {
    SynthPreset {
        name: "Afterlife Future Rave".into(),
        category: "Rave".into(),
        author: "DanimanX".into(),
        description: "Underground Afterlife club pumping bass/pad with trance gate rhythm.".into(),
        osc1_table: 1, // Cyber Neuro
        osc1_enabled: true,
        osc1_unison: 4,
        osc1_tune: 0.0,
        osc1_fine: 0.0,
        osc1_level: 0.95,
        osc1_pos: 0.45,
        osc1_detune: 0.22,
        osc1_spread: 0.9,
        osc1_pan: 0.0,
        osc1_formant: 0.0,
        osc1_bend: 0.0,
        osc2_table: 0,
        osc2_enabled: true,
        osc2_unison: 3,
        osc2_tune: 7.0,
        osc2_fine: 0.0,
        osc2_level: 0.8,
        osc2_pos: 0.3,
        osc2_detune: 0.15,
        osc2_spread: 0.8,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: true,
        sub_wave: 2,
        sub_octave: -1,
        sub_level: 0.7,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1,
        filter_cutoff: 1600.0,
        filter_reso: 0.42,
        filter_drive: 2.0,
        filter_key: 0.4,
        filter_vel: 0.5,
        filter_env_amount: 0.6,
        amp_attack: 0.01,
        amp_decay: 0.4,
        amp_sustain: 0.65,
        amp_release: 0.25,
        filter_attack: 0.015,
        filter_decay: 0.45,
        filter_sustain: 0.3,
        filter_release: 0.3,
        dist_enabled: true,
        dist_mode: 0,
        dist_amount: 0.4,
        dist_mix: 0.7,
        chorus_enabled: false,
        chorus_rate: 1.0,
        chorus_depth: 3.0,
        chorus_mix: 0.0,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 250.0,
        delay_fb: 0.4,
        delay_damp: 0.4,
        delay_mix: 0.3,
        reverb_enabled: true,
        reverb_size: 0.6,
        reverb_damp: 0.35,
        reverb_mix: 0.3,
        gate_enabled: true,
        seq_running: false,
        seq_mode: 3,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: vec![Step::default(); MAX_STEPS],
    }
}

fn preset_braindance_memory() -> SynthPreset {
    SynthPreset {
        name: "Braindance Memory".into(),
        category: "Atmosphere".into(),
        author: "DanimanX".into(),
        description: "Nostalgic digital artifacts and shimmering space echoing in virtual memory."
            .into(),
        osc1_table: 5, // Dystopian Steel
        osc1_enabled: true,
        osc1_unison: 3,
        osc1_tune: 0.0,
        osc1_fine: -6.0,
        osc1_level: 0.85,
        osc1_pos: 0.5,
        osc1_detune: 0.18,
        osc1_spread: 0.85,
        osc1_pan: -0.15,
        osc1_formant: 0.1,
        osc1_bend: 0.0,
        osc2_table: 3, // Vocaloid Synth
        osc2_enabled: true,
        osc2_unison: 3,
        osc2_tune: -12.0,
        osc2_fine: 6.0,
        osc2_level: 0.75,
        osc2_pos: 0.7,
        osc2_detune: 0.14,
        osc2_spread: 0.85,
        osc2_pan: 0.15,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: true,
        sub_wave: 0,
        sub_octave: -1,
        sub_level: 0.4,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 0, // LowPass12
        filter_cutoff: 2800.0,
        filter_reso: 0.22,
        filter_drive: 1.2,
        filter_key: 0.5,
        filter_vel: 0.3,
        filter_env_amount: 0.4,
        amp_attack: 0.12,
        amp_decay: 0.7,
        amp_sustain: 0.7,
        amp_release: 1.1,
        filter_attack: 0.18,
        filter_decay: 0.9,
        filter_sustain: 0.5,
        filter_release: 1.2,
        dist_enabled: true,
        dist_mode: 1, // Bitcrusher
        dist_amount: 0.25,
        dist_mix: 0.3,
        chorus_enabled: true,
        chorus_rate: 0.7,
        chorus_depth: 5.5,
        chorus_mix: 0.6,
        delay_enabled: true,
        delay_sync: true,
        delay_time: 420.0,
        delay_fb: 0.5,
        delay_damp: 0.3,
        delay_mix: 0.4,
        reverb_enabled: true,
        reverb_size: 0.85,
        reverb_damp: 0.25,
        reverb_mix: 0.5,
        gate_enabled: false,
        seq_running: false,
        seq_mode: 3,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: vec![Step::default(); MAX_STEPS],
    }
}

// =============================================================================
// NIGHT CITY EXPANSION PACK — 22 additional factory patches
// =============================================================================

/// Neutral, safe starting point used by the compact expansion patches.
fn base_preset(name: &str, category: &str, description: &str) -> SynthPreset {
    SynthPreset {
        name: name.into(),
        category: category.into(),
        author: "DanimanX".into(),
        description: description.into(),
        osc1_table: 0,
        osc1_enabled: true,
        osc1_unison: 2,
        osc1_tune: 0.0,
        osc1_fine: 0.0,
        osc1_level: 0.8,
        osc1_pos: 0.15,
        osc1_detune: 0.12,
        osc1_spread: 0.55,
        osc1_pan: 0.0,
        osc1_formant: 0.0,
        osc1_bend: 0.0,
        osc2_table: 0,
        osc2_enabled: false,
        osc2_unison: 1,
        osc2_tune: 0.0,
        osc2_fine: 0.0,
        osc2_level: 0.0,
        osc2_pos: 0.0,
        osc2_detune: 0.0,
        osc2_spread: 0.0,
        osc2_pan: 0.0,
        osc2_formant: 0.0,
        osc2_bend: 0.0,
        sub_enabled: false,
        sub_wave: 0,
        sub_octave: -1,
        sub_level: 0.0,
        sub_pan: 0.0,
        noise_enabled: false,
        noise_type: 0,
        noise_level: 0.0,
        noise_pan: 0.0,
        filter_enabled: true,
        filter_type: 1,
        filter_cutoff: 4200.0,
        filter_reso: 0.18,
        filter_drive: 1.1,
        filter_key: 0.25,
        filter_vel: 0.3,
        filter_env_amount: 0.25,
        amp_attack: 0.01,
        amp_decay: 0.35,
        amp_sustain: 0.75,
        amp_release: 0.45,
        filter_attack: 0.01,
        filter_decay: 0.4,
        filter_sustain: 0.2,
        filter_release: 0.4,
        dist_enabled: false,
        dist_mode: 0,
        dist_amount: 0.2,
        dist_mix: 0.0,
        chorus_enabled: false,
        chorus_rate: 1.0,
        chorus_depth: 3.0,
        chorus_mix: 0.0,
        delay_enabled: false,
        delay_sync: true,
        delay_time: 380.0,
        delay_fb: 0.35,
        delay_damp: 0.3,
        delay_mix: 0.0,
        reverb_enabled: false,
        reverb_size: 0.6,
        reverb_damp: 0.3,
        reverb_mix: 0.0,
        gate_enabled: false,
        seq_running: false,
        seq_mode: 3,
        seq_length: 16,
        seq_rate: 3,
        seq_swing: 0.0,
        seq_steps: create_default_steps(&[]),
    }
}

fn preset_neon_rain_pad() -> SynthPreset {
    let mut p = base_preset(
        "Neon Rain Pad",
        "Pad",
        "Wide FM rain over a vocal ghost, drenched in night-city reverb.",
    );
    p.osc1_table = 2;
    p.osc1_unison = 5;
    p.osc1_pos = 0.35;
    p.osc1_detune = 0.26;
    p.osc1_spread = 0.95;
    p.osc1_level = 0.7;
    p.osc2_table = 3;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_unison = 3;
    p.osc2_pos = 0.62;
    p.osc2_detune = 0.2;
    p.osc2_spread = 0.9;
    p.osc2_pan = 0.18;
    p.osc2_level = 0.52;
    p.filter_type = 0;
    p.filter_cutoff = 3100.0;
    p.filter_reso = 0.24;
    p.filter_env_amount = 0.18;
    p.amp_attack = 1.1;
    p.amp_decay = 1.4;
    p.amp_sustain = 0.78;
    p.amp_release = 3.4;
    p.filter_attack = 1.6;
    p.filter_decay = 2.0;
    p.filter_sustain = 0.45;
    p.filter_release = 2.6;
    p.chorus_enabled = true;
    p.chorus_rate = 0.45;
    p.chorus_depth = 5.5;
    p.chorus_mix = 0.55;
    p.delay_enabled = true;
    p.delay_fb = 0.38;
    p.delay_mix = 0.24;
    p.reverb_enabled = true;
    p.reverb_size = 0.94;
    p.reverb_mix = 0.46;
    p
}

fn preset_corpo_cathedral() -> SynthPreset {
    let mut p = base_preset(
        "Corpo Cathedral",
        "Pad",
        "A corporate temple of glass: six detuned voices and an endless tail.",
    );
    p.osc1_table = 3;
    p.osc1_unison = 6;
    p.osc1_pos = 0.24;
    p.osc1_detune = 0.32;
    p.osc1_spread = 1.0;
    p.osc1_level = 0.55;
    p.osc2_table = 4;
    p.osc2_enabled = true;
    p.osc2_tune = 12.0;
    p.osc2_pos = 0.48;
    p.osc2_unison = 2;
    p.osc2_detune = 0.18;
    p.osc2_level = 0.25;
    p.filter_type = 1;
    p.filter_cutoff = 2600.0;
    p.filter_reso = 0.3;
    p.filter_key = 0.35;
    p.amp_attack = 1.8;
    p.amp_decay = 1.8;
    p.amp_sustain = 0.8;
    p.amp_release = 4.6;
    p.filter_attack = 2.4;
    p.filter_decay = 2.6;
    p.filter_sustain = 0.5;
    p.filter_release = 3.8;
    p.chorus_enabled = true;
    p.chorus_rate = 0.32;
    p.chorus_depth = 6.5;
    p.chorus_mix = 0.5;
    p.delay_enabled = true;
    p.delay_fb = 0.42;
    p.delay_mix = 0.28;
    p.reverb_enabled = true;
    p.reverb_size = 0.97;
    p.reverb_mix = 0.56;
    p
}

fn preset_ghost_protocol_choir() -> SynthPreset {
    let mut p = base_preset(
        "Ghost Protocol Choir",
        "Pad",
        "Metallic voices leaking through a broken neural uplink.",
    );
    p.osc1_table = 5;
    p.osc1_unison = 4;
    p.osc1_pos = 0.22;
    p.osc1_detune = 0.24;
    p.osc1_spread = 0.9;
    p.osc1_formant = 0.28;
    p.osc1_level = 0.78;
    p.osc2_table = 3;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_pos = 0.7;
    p.osc2_unison = 4;
    p.osc2_detune = 0.22;
    p.osc2_spread = 0.95;
    p.osc2_level = 0.68;
    p.filter_type = 4;
    p.filter_cutoff = 1800.0;
    p.filter_reso = 0.28;
    p.filter_drive = 1.3;
    p.filter_env_amount = 0.2;
    p.amp_attack = 1.4;
    p.amp_decay = 2.0;
    p.amp_sustain = 0.72;
    p.amp_release = 4.2;
    p.filter_attack = 1.8;
    p.filter_decay = 2.4;
    p.filter_sustain = 0.5;
    p.filter_release = 3.0;
    p.chorus_enabled = true;
    p.chorus_rate = 0.28;
    p.chorus_depth = 7.0;
    p.chorus_mix = 0.62;
    p.reverb_enabled = true;
    p.reverb_size = 0.96;
    p.reverb_mix = 0.55;
    p
}

fn preset_cryo_sleep_pad() -> SynthPreset {
    let mut p = base_preset(
        "Cryo Sleep Pad",
        "Pad",
        "Frozen sine layers and a sub breathing under the ice.",
    );
    p.osc1_table = 0;
    p.osc1_unison = 6;
    p.osc1_pos = 0.72;
    p.osc1_detune = 0.2;
    p.osc1_spread = 1.0;
    p.osc1_level = 0.55;
    p.osc2_table = 7;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_pos = 0.4;
    p.osc2_unison = 2;
    p.osc2_detune = 0.12;
    p.osc2_level = 0.45;
    p.sub_enabled = true;
    p.sub_wave = 0;
    p.sub_octave = -1;
    p.sub_level = 0.3;
    p.filter_type = 0;
    p.filter_cutoff = 2200.0;
    p.filter_reso = 0.2;
    p.amp_attack = 2.6;
    p.amp_decay = 2.2;
    p.amp_sustain = 0.85;
    p.amp_release = 6.0;
    p.filter_attack = 3.0;
    p.filter_sustain = 0.6;
    p.filter_release = 5.0;
    p.reverb_enabled = true;
    p.reverb_size = 0.95;
    p.reverb_mix = 0.5;
    p.delay_enabled = true;
    p.delay_fb = 0.35;
    p.delay_mix = 0.2;
    p
}

fn preset_blackwall_drone() -> SynthPreset {
    let mut p = base_preset(
        "Blackwall Drone",
        "Drone",
        "A hostile entity behind the firewall. Slow, dark and unstable.",
    );
    p.osc1_table = 5;
    p.osc1_unison = 3;
    p.osc1_pos = 0.14;
    p.osc1_detune = 0.3;
    p.osc1_spread = 0.8;
    p.osc1_level = 0.5;
    p.osc2_table = 1;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_pos = 0.7;
    p.osc2_unison = 2;
    p.osc2_detune = 0.24;
    p.osc2_level = 0.4;
    p.sub_enabled = true;
    p.sub_wave = 2;
    p.sub_octave = -2;
    p.sub_level = 0.3;
    p.filter_type = 1;
    p.filter_cutoff = 900.0;
    p.filter_reso = 0.46;
    p.filter_drive = 1.9;
    p.filter_key = 0.1;
    p.filter_env_amount = 0.1;
    p.amp_attack = 3.0;
    p.amp_decay = 2.4;
    p.amp_sustain = 0.9;
    p.amp_release = 5.5;
    p.filter_attack = 3.6;
    p.filter_decay = 3.0;
    p.filter_sustain = 0.7;
    p.filter_release = 5.0;
    p.reverb_enabled = true;
    p.reverb_size = 0.97;
    p.reverb_damp = 0.4;
    p.reverb_mix = 0.48;
    p.delay_enabled = true;
    p.delay_fb = 0.35;
    p.delay_mix = 0.22;
    p
}

fn preset_toxic_fog_drone() -> SynthPreset {
    let mut p = base_preset(
        "Toxic Fog Drone",
        "Drone",
        "Pink dust, acid harmonics and a band-passed industrial fog.",
    );
    p.osc1_table = 1;
    p.osc1_unison = 5;
    p.osc1_pos = 0.64;
    p.osc1_detune = 0.36;
    p.osc1_spread = 0.95;
    p.osc1_level = 0.52;
    p.osc2_table = 4;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_pos = 0.4;
    p.osc2_unison = 3;
    p.osc2_detune = 0.28;
    p.osc2_level = 0.42;
    p.noise_enabled = true;
    p.noise_type = 1;
    p.noise_level = 0.14;
    p.noise_pan = -0.2;
    p.filter_type = 5;
    p.filter_cutoff = 1400.0;
    p.filter_reso = 0.5;
    p.filter_drive = 1.8;
    p.amp_attack = 4.0;
    p.amp_decay = 2.5;
    p.amp_sustain = 0.88;
    p.amp_release = 6.0;
    p.filter_attack = 2.5;
    p.filter_sustain = 0.6;
    p.filter_release = 5.0;
    p.chorus_enabled = true;
    p.chorus_rate = 0.2;
    p.chorus_depth = 8.0;
    p.chorus_mix = 0.55;
    p.reverb_enabled = true;
    p.reverb_size = 0.95;
    p.reverb_mix = 0.48;
    p
}

fn preset_singularity_hum() -> SynthPreset {
    let mut p = base_preset(
        "Singularity Hum",
        "Drone",
        "Gravity-well sub and folded harmonics from the event horizon.",
    );
    p.osc1_table = 7;
    p.osc1_unison = 2;
    p.osc1_pos = 0.4;
    p.osc1_detune = 0.18;
    p.osc1_level = 0.62;
    p.osc2_table = 2;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_pos = 0.65;
    p.osc2_level = 0.38;
    p.sub_enabled = true;
    p.sub_wave = 0;
    p.sub_octave = -2;
    p.sub_level = 0.55;
    p.filter_type = 0;
    p.filter_cutoff = 620.0;
    p.filter_reso = 0.32;
    p.filter_drive = 2.6;
    p.amp_attack = 5.0;
    p.amp_decay = 3.0;
    p.amp_sustain = 0.9;
    p.amp_release = 7.0;
    p.filter_attack = 4.0;
    p.filter_sustain = 0.7;
    p.filter_release = 6.0;
    p.reverb_enabled = true;
    p.reverb_size = 0.98;
    p.reverb_mix = 0.45;
    p
}

fn preset_arasaka_orbital_drone() -> SynthPreset {
    let mut p = base_preset(
        "Arasaka Orbital Drone",
        "Drone",
        "PWM solar winds circling a corporate station.",
    );
    p.osc1_table = 6;
    p.osc1_unison = 3;
    p.osc1_pos = 0.5;
    p.osc1_detune = 0.22;
    p.osc1_spread = 0.9;
    p.osc1_level = 0.55;
    p.osc2_table = 5;
    p.osc2_enabled = true;
    p.osc2_tune = 7.0;
    p.osc2_pos = 0.3;
    p.osc2_unison = 2;
    p.osc2_detune = 0.26;
    p.osc2_level = 0.35;
    p.noise_enabled = true;
    p.noise_type = 2;
    p.noise_level = 0.06;
    p.filter_type = 0;
    p.filter_cutoff = 1200.0;
    p.filter_reso = 0.28;
    p.amp_attack = 3.5;
    p.amp_decay = 2.5;
    p.amp_sustain = 0.86;
    p.amp_release = 6.0;
    p.filter_attack = 2.0;
    p.filter_sustain = 0.55;
    p.filter_release = 5.0;
    p.chorus_enabled = true;
    p.chorus_rate = 0.15;
    p.chorus_depth = 7.5;
    p.chorus_mix = 0.5;
    p.reverb_enabled = true;
    p.reverb_size = 0.97;
    p.reverb_mix = 0.5;
    p
}

fn preset_combat_zone_bass() -> SynthPreset {
    let mut p = base_preset(
        "Combat Zone Bass",
        "Bass",
        "Saturated neuro bass built to cut through gunfire.",
    );
    p.osc1_table = 1;
    p.osc1_unison = 3;
    p.osc1_pos = 0.45;
    p.osc1_detune = 0.14;
    p.osc1_spread = 0.6;
    p.osc1_level = 0.85;
    p.osc2_table = 7;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_pos = 0.55;
    p.osc2_level = 0.55;
    p.sub_enabled = true;
    p.sub_wave = 3;
    p.sub_octave = -1;
    p.sub_level = 0.45;
    p.filter_type = 1;
    p.filter_cutoff = 720.0;
    p.filter_reso = 0.32;
    p.filter_drive = 2.4;
    p.filter_env_amount = 0.7;
    p.amp_attack = 0.004;
    p.amp_decay = 0.26;
    p.amp_sustain = 0.55;
    p.amp_release = 0.18;
    p.filter_attack = 0.01;
    p.filter_decay = 0.28;
    p.filter_sustain = 0.2;
    p.filter_release = 0.2;
    p.dist_enabled = true;
    p.dist_mode = 2;
    p.dist_amount = 0.45;
    p.dist_mix = 0.7;
    p
}

fn preset_chrome_sub_assault() -> SynthPreset {
    let mut p = base_preset(
        "Chrome Sub Assault",
        "Bass",
        "Pure chrome sub pressure with a wavefolded edge.",
    );
    p.osc1_table = 7;
    p.osc1_unison = 1;
    p.osc1_pos = 0.55;
    p.osc1_level = 0.8;
    p.osc2_table = 0;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_pos = 0.2;
    p.osc2_unison = 2;
    p.osc2_detune = 0.08;
    p.osc2_level = 0.45;
    p.sub_enabled = true;
    p.sub_wave = 0;
    p.sub_octave = -2;
    p.sub_level = 0.65;
    p.filter_type = 1;
    p.filter_cutoff = 480.0;
    p.filter_reso = 0.26;
    p.filter_drive = 2.8;
    p.filter_env_amount = 0.5;
    p.amp_attack = 0.002;
    p.amp_decay = 0.3;
    p.amp_sustain = 0.6;
    p.amp_release = 0.14;
    p.filter_attack = 0.004;
    p.filter_decay = 0.22;
    p.filter_sustain = 0.3;
    p.filter_release = 0.16;
    p.dist_enabled = true;
    p.dist_mode = 3;
    p.dist_amount = 0.28;
    p.dist_mix = 0.45;
    p
}

fn preset_neuro_reaper_bass() -> SynthPreset {
    let mut p = base_preset(
        "Neuro Reaper Bass",
        "Bass",
        "Reese-style neuro movement with a 24 dB resonant snarl.",
    );
    p.osc1_table = 1;
    p.osc1_unison = 4;
    p.osc1_pos = 0.62;
    p.osc1_detune = 0.22;
    p.osc1_spread = 0.7;
    p.osc1_level = 0.85;
    p.osc2_table = 2;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_pos = 0.4;
    p.osc2_unison = 2;
    p.osc2_detune = 0.16;
    p.osc2_level = 0.5;
    p.sub_enabled = true;
    p.sub_wave = 2;
    p.sub_octave = -1;
    p.sub_level = 0.4;
    p.filter_type = 1;
    p.filter_cutoff = 1200.0;
    p.filter_reso = 0.52;
    p.filter_drive = 1.9;
    p.filter_env_amount = 0.8;
    p.amp_attack = 0.006;
    p.amp_decay = 0.3;
    p.amp_sustain = 0.62;
    p.amp_release = 0.22;
    p.filter_attack = 0.008;
    p.filter_decay = 0.35;
    p.filter_sustain = 0.25;
    p.filter_release = 0.24;
    p.dist_enabled = true;
    p.dist_mode = 2;
    p.dist_amount = 0.55;
    p.dist_mix = 0.62;
    p.delay_enabled = true;
    p.delay_fb = 0.25;
    p.delay_mix = 0.12;
    p
}

fn preset_acid_rain_bass() -> SynthPreset {
    let mut p = base_preset(
        "Acid Rain Bass",
        "Bass",
        "Silver-box acid: squelchy resonance, slide-ready envelopes.",
    );
    p.osc1_table = 4;
    p.osc1_unison = 1;
    p.osc1_pos = 0.3;
    p.osc1_level = 0.72;
    p.sub_enabled = true;
    p.sub_wave = 3;
    p.sub_octave = -1;
    p.sub_level = 0.28;
    p.filter_type = 1;
    p.filter_cutoff = 950.0;
    p.filter_reso = 0.58;
    p.filter_drive = 2.0;
    p.filter_env_amount = 0.85;
    p.amp_attack = 0.002;
    p.amp_decay = 0.18;
    p.amp_sustain = 0.35;
    p.amp_release = 0.12;
    p.filter_attack = 0.002;
    p.filter_decay = 0.2;
    p.filter_sustain = 0.15;
    p.filter_release = 0.16;
    p.dist_enabled = true;
    p.dist_mode = 0;
    p.dist_amount = 0.3;
    p.dist_mix = 0.45;
    p.delay_enabled = true;
    p.delay_fb = 0.3;
    p.delay_mix = 0.12;
    p
}

fn preset_nightside_razor_lead() -> SynthPreset {
    let mut p = base_preset(
        "Nightside Razor Lead",
        "Lead",
        "A sharp neon blade with tempo echoes and a short bite.",
    );
    p.osc1_table = 6;
    p.osc1_unison = 3;
    p.osc1_pos = 0.3;
    p.osc1_detune = 0.16;
    p.osc1_spread = 0.8;
    p.osc1_level = 0.72;
    p.osc2_table = 4;
    p.osc2_enabled = true;
    p.osc2_tune = 7.0;
    p.osc2_pos = 0.5;
    p.osc2_level = 0.32;
    p.filter_type = 1;
    p.filter_cutoff = 5200.0;
    p.filter_reso = 0.28;
    p.filter_drive = 1.4;
    p.filter_env_amount = 0.3;
    p.amp_attack = 0.005;
    p.amp_decay = 0.3;
    p.amp_sustain = 0.7;
    p.amp_release = 0.3;
    p.filter_attack = 0.01;
    p.filter_decay = 0.4;
    p.filter_sustain = 0.4;
    p.filter_release = 0.35;
    p.chorus_enabled = true;
    p.chorus_rate = 0.8;
    p.chorus_mix = 0.3;
    p.delay_enabled = true;
    p.delay_fb = 0.38;
    p.delay_mix = 0.32;
    p.reverb_enabled = true;
    p.reverb_size = 0.55;
    p.reverb_mix = 0.16;
    p
}

fn preset_laser_katana() -> SynthPreset {
    let mut p = base_preset(
        "Laser Katana",
        "Lead",
        "Digital FM blade with a cutting band-pass and hard transients.",
    );
    p.osc1_table = 2;
    p.osc1_unison = 2;
    p.osc1_pos = 0.55;
    p.osc1_detune = 0.1;
    p.osc1_level = 0.75;
    p.osc2_table = 5;
    p.osc2_enabled = true;
    p.osc2_tune = 12.0;
    p.osc2_pos = 0.4;
    p.osc2_level = 0.32;
    p.filter_type = 4;
    p.filter_cutoff = 3800.0;
    p.filter_reso = 0.42;
    p.filter_drive = 1.6;
    p.filter_env_amount = 0.4;
    p.amp_attack = 0.003;
    p.amp_decay = 0.2;
    p.amp_sustain = 0.6;
    p.amp_release = 0.25;
    p.filter_attack = 0.005;
    p.filter_decay = 0.25;
    p.filter_sustain = 0.3;
    p.filter_release = 0.2;
    p.dist_enabled = true;
    p.dist_mode = 0;
    p.dist_amount = 0.3;
    p.dist_mix = 0.45;
    p.delay_enabled = true;
    p.delay_fb = 0.42;
    p.delay_mix = 0.36;
    p.reverb_enabled = true;
    p.reverb_size = 0.5;
    p.reverb_mix = 0.14;
    p
}

fn preset_neon_samurai_lead() -> SynthPreset {
    let mut p = base_preset(
        "Neon Samurai Lead",
        "Lead",
        "Vocal formant lead with a wide chorus and a long neon trail.",
    );
    p.osc1_table = 3;
    p.osc1_unison = 4;
    p.osc1_pos = 0.5;
    p.osc1_detune = 0.2;
    p.osc1_spread = 0.95;
    p.osc1_formant = 0.15;
    p.osc1_level = 0.85;
    p.osc2_table = 0;
    p.osc2_enabled = true;
    p.osc2_tune = 12.0;
    p.osc2_pos = 0.3;
    p.osc2_unison = 3;
    p.osc2_detune = 0.16;
    p.osc2_level = 0.5;
    p.filter_type = 0;
    p.filter_cutoff = 6200.0;
    p.filter_reso = 0.25;
    p.filter_env_amount = 0.2;
    p.amp_attack = 0.012;
    p.amp_decay = 0.4;
    p.amp_sustain = 0.8;
    p.amp_release = 0.55;
    p.filter_attack = 0.02;
    p.filter_decay = 0.5;
    p.filter_sustain = 0.5;
    p.filter_release = 0.5;
    p.chorus_enabled = true;
    p.chorus_rate = 0.6;
    p.chorus_depth = 4.0;
    p.chorus_mix = 0.42;
    p.delay_enabled = true;
    p.delay_fb = 0.4;
    p.delay_mix = 0.3;
    p.reverb_enabled = true;
    p.reverb_size = 0.72;
    p.reverb_mix = 0.26;
    p
}

fn preset_void_screamer_lead() -> SynthPreset {
    let mut p = base_preset(
        "Void Screamer Lead",
        "Lead",
        "A hostile lead screaming through the Blackwall.",
    );
    p.osc1_table = 1;
    p.osc1_unison = 2;
    p.osc1_pos = 0.82;
    p.osc1_detune = 0.24;
    p.osc1_level = 0.52;
    p.osc2_table = 5;
    p.osc2_enabled = true;
    p.osc2_tune = 7.0;
    p.osc2_pos = 0.5;
    p.osc2_unison = 2;
    p.osc2_detune = 0.2;
    p.osc2_level = 0.28;
    p.filter_type = 1;
    p.filter_cutoff = 3000.0;
    p.filter_reso = 0.55;
    p.filter_drive = 2.0;
    p.filter_env_amount = 0.45;
    p.amp_attack = 0.005;
    p.amp_decay = 0.22;
    p.amp_sustain = 0.7;
    p.amp_release = 0.32;
    p.filter_attack = 0.008;
    p.filter_decay = 0.3;
    p.filter_sustain = 0.35;
    p.filter_release = 0.3;
    p.dist_enabled = true;
    p.dist_mode = 3;
    p.dist_amount = 0.34;
    p.dist_mix = 0.3;
    p.delay_enabled = true;
    p.delay_fb = 0.35;
    p.delay_mix = 0.22;
    p
}

fn preset_data_stream_arp() -> SynthPreset {
    let mut p = base_preset(
        "Data Stream Arp",
        "Arp",
        "A 16th-note data stream with glassy FM plucks.",
    );
    p.osc1_table = 2;
    p.osc1_unison = 2;
    p.osc1_pos = 0.4;
    p.osc1_detune = 0.1;
    p.osc1_level = 0.95;
    p.osc2_table = 0;
    p.osc2_enabled = true;
    p.osc2_tune = 12.0;
    p.osc2_pos = 0.25;
    p.osc2_level = 0.45;
    p.filter_type = 1;
    p.filter_cutoff = 3800.0;
    p.filter_reso = 0.35;
    p.filter_env_amount = 0.55;
    p.amp_attack = 0.002;
    p.amp_decay = 0.12;
    p.amp_sustain = 0.35;
    p.amp_release = 0.16;
    p.filter_attack = 0.002;
    p.filter_decay = 0.14;
    p.filter_sustain = 0.15;
    p.filter_release = 0.18;
    p.chorus_enabled = true;
    p.chorus_rate = 1.2;
    p.chorus_mix = 0.3;
    p.delay_enabled = true;
    p.delay_fb = 0.45;
    p.delay_mix = 0.35;
    p.reverb_enabled = true;
    p.reverb_size = 0.6;
    p.reverb_mix = 0.25;
    p.seq_mode = 0;
    p.seq_length = 16;
    p.seq_rate = 3;
    p.seq_swing = 0.06;
    p.seq_steps = create_default_steps(&[
        (0, 110, true, false, true),
        (12, 90, true, false, false),
        (7, 100, true, true, false),
        (12, 90, true, false, false),
        (3, 105, true, false, true),
        (15, 90, true, false, false),
        (10, 100, true, true, false),
        (15, 90, true, false, false),
        (0, 110, true, false, true),
        (12, 90, true, false, false),
        (7, 100, true, true, false),
        (12, 90, true, false, false),
        (5, 105, true, false, true),
        (17, 90, true, false, false),
        (12, 100, true, true, false),
        (19, 90, true, false, false),
    ]);
    p
}

fn preset_kiroshi_rain_arp() -> SynthPreset {
    let mut p = base_preset(
        "Kiroshi Rain Arp",
        "Arp",
        "Optical implant arpeggio: PWM droplets in a delay field.",
    );
    p.osc1_table = 6;
    p.osc1_unison = 2;
    p.osc1_pos = 0.4;
    p.osc1_detune = 0.14;
    p.osc1_level = 0.9;
    p.osc2_table = 3;
    p.osc2_enabled = true;
    p.osc2_tune = 12.0;
    p.osc2_pos = 0.65;
    p.osc2_level = 0.45;
    p.filter_type = 0;
    p.filter_cutoff = 2800.0;
    p.filter_reso = 0.3;
    p.filter_env_amount = 0.45;
    p.amp_attack = 0.004;
    p.amp_decay = 0.2;
    p.amp_sustain = 0.4;
    p.amp_release = 0.3;
    p.filter_attack = 0.006;
    p.filter_decay = 0.24;
    p.filter_sustain = 0.2;
    p.filter_release = 0.28;
    p.chorus_enabled = true;
    p.chorus_rate = 0.9;
    p.chorus_mix = 0.36;
    p.delay_enabled = true;
    p.delay_fb = 0.48;
    p.delay_mix = 0.4;
    p.reverb_enabled = true;
    p.reverb_size = 0.72;
    p.reverb_mix = 0.3;
    p.seq_mode = 0;
    p.seq_length = 16;
    p.seq_rate = 3;
    p.seq_swing = 0.12;
    p.seq_steps = create_default_steps(&[
        (0, 105, true, false, true),
        (7, 90, true, false, false),
        (12, 100, true, true, false),
        (15, 85, true, false, false),
        (3, 105, true, false, true),
        (10, 90, true, false, false),
        (15, 100, true, true, false),
        (19, 85, true, false, false),
        (0, 105, true, false, true),
        (7, 90, true, false, false),
        (12, 100, true, true, false),
        (15, 85, true, false, false),
        (5, 105, true, false, true),
        (12, 90, true, false, false),
        (17, 100, true, true, false),
        (24, 85, true, false, false),
    ]);
    p
}

fn preset_afterlife_run() -> SynthPreset {
    let mut p = base_preset(
        "Afterlife Run",
        "Sequence",
        "A full-throttle 16-step chase for the Afterlife club.",
    );
    p.osc1_table = 1;
    p.osc1_unison = 3;
    p.osc1_pos = 0.5;
    p.osc1_detune = 0.16;
    p.osc1_level = 0.8;
    p.osc2_table = 4;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_pos = 0.45;
    p.osc2_level = 0.4;
    p.sub_enabled = true;
    p.sub_wave = 3;
    p.sub_octave = -1;
    p.sub_level = 0.35;
    p.filter_type = 1;
    p.filter_cutoff = 1400.0;
    p.filter_reso = 0.4;
    p.filter_drive = 1.8;
    p.filter_env_amount = 0.75;
    p.amp_attack = 0.004;
    p.amp_decay = 0.2;
    p.amp_sustain = 0.5;
    p.amp_release = 0.18;
    p.filter_attack = 0.005;
    p.filter_decay = 0.22;
    p.filter_sustain = 0.2;
    p.filter_release = 0.2;
    p.dist_enabled = true;
    p.dist_mode = 2;
    p.dist_amount = 0.4;
    p.dist_mix = 0.55;
    p.delay_enabled = true;
    p.delay_fb = 0.32;
    p.delay_mix = 0.22;
    p.reverb_enabled = true;
    p.reverb_size = 0.45;
    p.reverb_mix = 0.15;
    p.seq_mode = 0;
    p.seq_length = 16;
    p.seq_rate = 3;
    p.seq_swing = 0.08;
    p.seq_steps = create_default_steps(&[
        (0, 120, true, false, true),
        (0, 95, true, false, false),
        (3, 105, true, true, false),
        (7, 115, true, false, true),
        (0, 95, true, false, false),
        (10, 100, true, false, false),
        (12, 115, true, true, true),
        (7, 95, true, false, false),
        (0, 120, true, false, true),
        (0, 95, true, false, false),
        (2, 100, true, false, false),
        (3, 105, true, true, false),
        (5, 110, true, false, true),
        (7, 95, true, false, false),
        (12, 120, true, true, true),
        (0, 90, true, false, false),
    ]);
    p
}

fn preset_braindance_64() -> SynthPreset {
    let mut p = base_preset(
        "Braindance 64",
        "Sequence",
        "A 64-step braindance memory that keeps mutating.",
    );
    p.osc1_table = 3;
    p.osc1_unison = 3;
    p.osc1_pos = 0.55;
    p.osc1_detune = 0.18;
    p.osc1_spread = 0.8;
    p.osc1_level = 0.65;
    p.osc2_table = 2;
    p.osc2_enabled = true;
    p.osc2_tune = 12.0;
    p.osc2_pos = 0.35;
    p.osc2_level = 0.3;
    p.filter_type = 1;
    p.filter_cutoff = 2600.0;
    p.filter_reso = 0.38;
    p.filter_drive = 1.5;
    p.filter_env_amount = 0.55;
    p.amp_attack = 0.006;
    p.amp_decay = 0.25;
    p.amp_sustain = 0.55;
    p.amp_release = 0.3;
    p.filter_attack = 0.01;
    p.filter_decay = 0.3;
    p.filter_sustain = 0.25;
    p.filter_release = 0.35;
    p.chorus_enabled = true;
    p.chorus_rate = 0.7;
    p.chorus_mix = 0.3;
    p.delay_enabled = true;
    p.delay_fb = 0.45;
    p.delay_mix = 0.32;
    p.reverb_enabled = true;
    p.reverb_size = 0.7;
    p.reverb_mix = 0.28;
    p.seq_mode = 0;
    p.seq_length = 64;
    p.seq_rate = 3;
    p.seq_swing = 0.1;
    let scale = [0i8, 3, 5, 7, 10, 12, 15, 17, 19, 22, 24];
    let mut pattern = Vec::with_capacity(64);
    for i in 0..64 {
        let note = scale[(i * 7 + i / 4) % scale.len()] - 12;
        let gate = i % 4 == 0 || i % 7 == 0 || i % 11 == 0;
        let accent = i % 8 == 0;
        let slide = i % 13 == 6;
        let vel = if accent { 120 } else { 92 };
        pattern.push((note, vel, gate, slide, accent));
    }
    p.seq_steps = create_default_steps(&pattern);
    p
}

fn preset_emp_burst() -> SynthPreset {
    let mut p = base_preset(
        "EMP Burst",
        "FX",
        "A magnetic pulse: white noise, a falling filter and crushed debris.",
    );
    p.osc1_table = 5;
    p.osc1_unison = 2;
    p.osc1_pos = 0.8;
    p.osc1_detune = 0.3;
    p.osc1_level = 0.35;
    p.osc2_enabled = false;
    p.noise_enabled = true;
    p.noise_type = 0;
    p.noise_level = 0.5;
    p.filter_type = 5;
    p.filter_cutoff = 1200.0;
    p.filter_reso = 0.6;
    p.filter_drive = 1.5;
    p.filter_env_amount = -0.8;
    p.amp_attack = 0.001;
    p.amp_decay = 0.8;
    p.amp_sustain = 0.0;
    p.amp_release = 1.4;
    p.filter_attack = 0.001;
    p.filter_decay = 0.9;
    p.filter_sustain = 0.0;
    p.filter_release = 1.4;
    p.dist_enabled = true;
    p.dist_mode = 1;
    p.dist_amount = 0.5;
    p.dist_mix = 0.55;
    p.delay_enabled = true;
    p.delay_fb = 0.45;
    p.delay_mix = 0.3;
    p.reverb_enabled = true;
    p.reverb_size = 0.85;
    p.reverb_mix = 0.38;
    p.seq_mode = 3;
    p
}

fn preset_cyberpsychosis() -> SynthPreset {
    let mut p = base_preset(
        "Cyberpsychosis",
        "FX",
        "Unstable acid and metallic noise. Do not operate heavy machinery.",
    );
    p.osc1_table = 4;
    p.osc1_unison = 3;
    p.osc1_pos = 0.9;
    p.osc1_detune = 0.42;
    p.osc1_spread = 1.0;
    p.osc1_level = 0.55;
    p.osc2_table = 5;
    p.osc2_enabled = true;
    p.osc2_tune = -12.0;
    p.osc2_pos = 0.6;
    p.osc2_unison = 2;
    p.osc2_detune = 0.35;
    p.osc2_level = 0.4;
    p.noise_enabled = true;
    p.noise_type = 2;
    p.noise_level = 0.1;
    p.filter_type = 3;
    p.filter_cutoff = 420.0;
    p.filter_reso = 0.5;
    p.filter_drive = 2.4;
    p.filter_env_amount = 0.6;
    p.amp_attack = 0.01;
    p.amp_decay = 0.4;
    p.amp_sustain = 0.6;
    p.amp_release = 0.8;
    p.filter_attack = 0.02;
    p.filter_decay = 0.5;
    p.filter_sustain = 0.4;
    p.filter_release = 0.9;
    p.dist_enabled = true;
    p.dist_mode = 3;
    p.dist_amount = 0.5;
    p.dist_mix = 0.6;
    p.delay_enabled = true;
    p.delay_fb = 0.5;
    p.delay_mix = 0.4;
    p.reverb_enabled = true;
    p.reverb_size = 0.8;
    p.reverb_mix = 0.4;
    p.seq_mode = 3;
    p
}
