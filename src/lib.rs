pub mod audio;
pub mod gui;
pub mod presets;

use audio::SynthEngine;
use gui::{render_cyberpunk_gui, GuiSharedState};
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, EguiState};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct ReplicantWave {
    params: Arc<ReplicantWaveParams>,
    engine: Arc<Mutex<SynthEngine>>,
    sample_rate: f32,
}

#[derive(Params)]
pub struct ReplicantWaveParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "cutoff"]
    pub cutoff: FloatParam,

    #[id = "resonance"]
    pub resonance: FloatParam,
}

impl Default for ReplicantWave {
    fn default() -> Self {
        Self {
            params: Arc::new(ReplicantWaveParams::default()),
            engine: Arc::new(Mutex::new(SynthEngine::default())),
            sample_rate: 44100.0,
        }
    }
}

impl Default for ReplicantWaveParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(1024, 700),
            gain: FloatParam::new(
                "Master Gain",
                util::db_to_gain(0.0),
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.5,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            cutoff: FloatParam::new(
                "Filter Cutoff",
                2500.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(1)),

            resonance: FloatParam::new(
                "Filter Resonance",
                0.2,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }
}

impl Plugin for ReplicantWave {
    const NAME: &'static str = "ReplicantWave";
    const VENDOR: &'static str = "DanimanX";
    const URL: &'static str = "https://github.com/DanimanX/ReplicantWave";
    const EMAIL: &'static str = "daniel.rojas@chileatiende.cl";
    const VERSION: &'static str = "1.0.1";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let editor_state = self.params.editor_state.clone();
        let engine = self.engine.clone();

        create_egui_editor(
            editor_state,
            GuiSharedState::new(engine),
            |_, _| {},
            move |egui_ctx, _setter, state| {
                nih_plug_egui::egui::CentralPanel::default().show(egui_ctx, |ui| {
                    render_cyberpunk_gui(ui, state);
                });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.engine.lock().set_sample_rate(self.sample_rate);
        true
    }

    fn reset(&mut self) {
        self.engine.lock().reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut engine = self.engine.lock();

        // Sync host BPM
        let host_bpm = context
            .transport()
            .tempo
            .map(|t| t as f32)
            .unwrap_or(125.0);

        // Process incoming MIDI events
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn { note, velocity, .. } => {
                    engine.note_on(note, velocity);
                }
                NoteEvent::NoteOff { note, .. } => {
                    engine.note_off(note);
                }
                NoteEvent::MidiPitchBend { value, .. } => {
                    // value is -1.0 to 1.0
                    engine.set_pitch_bend(value, 2.0);
                }
                NoteEvent::MidiCC { cc, value, .. } => {
                    // CC 1 (Modulation Wheel)
                    if cc == 1 {
                        let norm = value as f32 / 127.0;
                        engine.fx.distortion.amount = norm;
                    }
                }
                _ => {}
            }
        }

        // Render audio
        let num_samples = buffer.samples();
        let outputs = buffer.as_slice();
        let (left_channel, right_channel) = outputs.split_at_mut(1);
        let left_slice = &mut left_channel[0][..num_samples];
        let right_slice = &mut right_channel[0][..num_samples];

        engine.process_block(left_slice, right_slice, host_bpm);

        ProcessStatus::Normal
    }
}

impl ClapPlugin for ReplicantWave {
    const CLAP_ID: &'static str = "com.danimanx.replicantwave";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Cyberpunk Wavetable Synthesizer with 64-Step Sequencer");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for ReplicantWave {
    const VST3_CLASS_ID: [u8; 16] = *b"ReplicantWav1337";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(ReplicantWave);
nih_export_vst3!(ReplicantWave);
