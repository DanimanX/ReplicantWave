pub mod audio;
pub mod gui;
pub mod params;
pub mod presets;

use audio::SynthEngine;
use gui::{render_cyberpunk_gui, GuiSharedState, MIN_EDITOR_HEIGHT, MIN_EDITOR_WIDTH};
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui::vec2, resizable_window::ResizableWindow};
use params::ReplicantWaveParams;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct ReplicantWave {
    params: Arc<ReplicantWaveParams>,
    engine: Arc<Mutex<SynthEngine>>,
    /// Last pattern version copied into the engine.
    pattern_version: u64,
    sample_rate: f32,
}

impl Default for ReplicantWave {
    fn default() -> Self {
        Self {
            params: Arc::new(ReplicantWaveParams::default()),
            engine: Arc::new(Mutex::new(SynthEngine::default())),
            pattern_version: 0,
            sample_rate: 44100.0,
        }
    }
}

impl Plugin for ReplicantWave {
    const NAME: &'static str = "ReplicantWave";
    const VENDOR: &'static str = "DanimanX";
    const URL: &'static str = "https://github.com/DanimanX/ReplicantWave";
    const EMAIL: &'static str = "daniel.rojas@chileatiende.cl";
    const VERSION: &'static str = "1.2.4";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

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
        let params = self.params.clone();

        create_egui_editor(
            editor_state.clone(),
            GuiSharedState::new(engine, params),
            |_, _| {},
            move |egui_ctx, setter, state| {
                // The HUD below is laid out proportionally, so it follows whatever
                // size the window has. Dragging the grip in the bottom-right corner
                // requests a new size through the host (`GuiContext::request_resize`),
                // and `EguiState` (a `#[persist]` field on the params) keeps the last
                // size with the session.
                ResizableWindow::new("replicantwave-editor")
                    .min_size(vec2(MIN_EDITOR_WIDTH, MIN_EDITOR_HEIGHT))
                    .show(egui_ctx, editor_state.as_ref(), |ui| {
                        render_cyberpunk_gui(ui, setter, state);
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
        let mut engine = self.engine.lock();
        engine.set_sample_rate(self.sample_rate);
        // Push the persisted pattern and the current parameter values so a
        // restored session sounds exactly like it did when saved.
        self.params.sync_to_engine(&mut engine);
        self.params
            .apply_pattern_if_changed(&mut engine, &mut self.pattern_version);
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
        // Once per audio thread: flush denormals before any DSP runs.
        thread_local! {
            static DENORMALS: () = audio::denormal::flush_denormals_to_zero();
        }
        DENORMALS.with(|_| {});

        let mut engine = self.engine.lock();

        // Every host-visible parameter is mirrored into the engine here. This
        // runs before MIDI so automation and MIDI land in the same block.
        self.params.sync_to_engine(&mut engine);
        self.params
            .apply_pattern_if_changed(&mut engine, &mut self.pattern_version);

        // Sync host transport & BPM
        let host_playing = context.transport().playing;
        let host_bpm = context.transport().tempo.map(|t| t as f32).unwrap_or(0.0);

        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn { note, velocity, .. } => {
                    engine.handle_midi_note_on(note, velocity);
                }
                NoteEvent::NoteOff { note, .. } => {
                    engine.handle_midi_note_off(note);
                }
                NoteEvent::MidiPitchBend { value, .. } => {
                    engine.set_pitch_bend(value, 2.0);
                }
                // Every control is a host parameter now, so MIDI is handled by
                // the host's own MIDI-learn / modulation matrix. The previous
                // hidden CC 1 -> distortion mapping was removed because it
                // silently fought the distortion parameter and automation.
                _ => {}
            }
        }

        let num_samples = buffer.samples();
        let outputs = buffer.as_slice();
        let (left_channel, right_channel) = outputs.split_at_mut(1);
        let left_slice = &mut left_channel[0][..num_samples];
        let right_slice = &mut right_channel[0][..num_samples];

        engine.process_block(left_slice, right_slice, host_bpm, host_playing);

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
nih_export_vst3! {ReplicantWave}
