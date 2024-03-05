#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::let_and_return)]

mod arpeggiator;
mod chords;
mod display;
mod inputs;
mod parameters;
mod random;
pub mod save;
mod scales;

use arplus_dsp::{Attributes as DSPAttributes, TriggerAttributes as DSPTriggerAttributes};

use crate::arpeggiator::{
    Arpeggiator, Configuration as ArpeggiatorConfiguration, Mode as ArpeggiatorMode,
};
use crate::chords::Chords;
use crate::display::{ArpModeScreen, Display, Priority, Screen, StepScreen};
pub use crate::inputs::ControlInputSnapshot;
use crate::inputs::Inputs;
use crate::parameters::Parameters;
use crate::random::RandomGenerator;
use crate::save::Save;
use crate::scales::Scales;
use crate::scales::{scale_note::ScaleNote, tonic::Tonic};

pub struct Controller {
    display: Display,
    inputs: Inputs,
    parameters: Parameters,
    chords: Chords,
    scales: Scales,
    arp: Arpeggiator,
    random_generator: RandomGenerator,
    // state: State,
    // queue: Queue,
}

// enum State {
//     Calibrating(StateCalibrating),
//     Normal,
// }

// struct StateCalibrating {
//     input: usize,
//     phase: CalibrationPhase,
// }

// enum CalibrationPhase {
//     Octave1,
//     Octave2(f32),
// }

// struct Queue {
//     queue: Vec<ControlAction; 8>,
// }

// enum ControlAction {
//     Calibrate(usize),
// }

pub struct Result {
    pub save: Option<Save>,
    pub dsp_attributes: DSPAttributes,
}

pub struct ControlOutputState {
    pub leds: [bool; 8],
}

impl Controller {
    pub fn new(seed: u64, save: Save) -> Self {
        let scales = Scales::new();
        let chords = Chords::new();
        // TODO: Recover them from an input snapshot too.
        let parameters = Parameters::new(save.parameters, &chords, &scales);

        // SAFETY: Parameter values are always limited based on the selected
        // chord group.
        let selected_chord = chords
            .chord(
                parameters.chord_group.selected_value(),
                parameters.chord.selected_value(),
            )
            .unwrap();

        // SAFETY: Parameter values are always limited based on the selected
        // scale group.
        let selected_scale = scales
            .scale(
                parameters.scale_group.selected_value(),
                parameters.scale.selected_value(),
            )
            .unwrap();

        // TODO: This will require input snapshot and save to initialize itself as well.
        // Otherwise the root would move during the start. Once done, take all options
        // from parameters.
        // TODO: Once everything is taken from parameters, this can be move into
        // a function shared with the attribute reconciliation.
        let arp = Arpeggiator::new_with_configuration(ArpeggiatorConfiguration {
            tonic: Tonic::C,
            scale: selected_scale,
            root: ScaleNote::new(scales::quarter_tones::QuarterTone::C1, 0),
            chord: selected_chord,
            // SAFETY: Parameter values used to get arp index are
            // statically limited by the maximum number of modes.
            mode: ArpeggiatorMode::try_from_index(parameters.arp.selected_value()).unwrap(),
        });

        Self {
            display: Display::new(),
            parameters,
            scales,
            chords,
            arp,
            inputs: Inputs::new(),
            random_generator: RandomGenerator::with_seed(seed),
        }
    }

    pub fn apply_input_snapshot(&mut self, snapshot: ControlInputSnapshot) -> Result {
        self.inputs.apply_input_snapshot(snapshot);
        let needs_save = self.reconcile_display_and_parameters_with_inputs();
        let save = if needs_save {
            Some(Save {
                parameters: self.parameters.copy_config(),
            })
        } else {
            None
        };
        let dsp_attributes = self.generate_dsp_attributes();

        Result {
            save,
            dsp_attributes,
        }
    }

    fn reconcile_display_and_parameters_with_inputs(&mut self) -> bool {
        let pots = &self.inputs.pots;
        let buttons = &self.inputs.buttons;
        let cvs = &self.inputs.cvs;
        let parameters = &mut self.parameters;

        let mut needs_save = false;

        needs_save |= parameters
            .tone
            .reconcile(linear_sum(pots.tone.value, cvs.tone.value));
        needs_save |= parameters
            .chord
            .reconcile(linear_sum(pots.chord.value, cvs.chord.value));
        needs_save |= parameters
            .chord_group
            .reconcile(linear_sum(pots.size.value, cvs.size.value));
        parameters
            .contour
            .reconcile(linear_sum(pots.contour.value, cvs.contour.value));
        parameters
            .cutoff
            .reconcile(linear_sum(pots.cutoff.value, cvs.cutoff.value));
        parameters
            .resonance
            .reconcile(linear_sum(pots.resonance.value, cvs.resonance.value));
        needs_save |= parameters.scale_group.reconcile(buttons.tonic.released);
        needs_save |= parameters.scale.reconcile(buttons.mode.released);

        let mut queried_display = None;
        let mut active_display = None;
        needs_save |= if buttons.arp.held_for > 400 {
            queried_display = Some(Screen::ArpMode(ArpModeScreen::with_selected(
                parameters.arp.selected_value(),
            )));
            false
        } else if buttons.arp.released && buttons.arp.released_after <= 400 {
            // TODO: Use a constant
            // TODO: Set a temporary display
            let changed = parameters.arp.reconcile(buttons.arp.released);
            active_display = Some(Screen::ArpMode(ArpModeScreen::with_selected(
                parameters.arp.selected_value(),
            )));
            changed
        } else {
            false
        };

        if let Some(active_display) = active_display {
            self.display.set(Priority::Active, active_display);
        };

        if let Some(queried_display) = queried_display {
            self.display.set(Priority::Queried, queried_display);
        } else {
            self.display.reset(Priority::Queried);
        };

        parameters
            .trigger
            .reconcile(buttons.trigger.clicked || cvs.trigger.triggered);

        // SAFETY: Chord group index parameter is always limited by the maximum
        // number of chord groups.
        let chord_group_index = self.parameters.chord_group.selected_value();
        self.parameters
            .chord
            .set_output_values(self.chords.number_of_chords(chord_group_index).unwrap());

        // SAFETY: Scale group index parameter is always limited by the maximum
        // number of scale groups.
        let scale_group_index = self.parameters.scale_group.selected_value();
        self.parameters
            .scale
            .set_output_values(self.scales.number_of_scales(scale_group_index).unwrap());

        // TODO: Safety
        const OCTAVES: usize = 7;
        // TODO: No unwrap or safety note
        let steps_in_scale = self
            .scales
            .number_of_steps_in_group(self.parameters.scale_group.selected_value())
            .unwrap();
        self.parameters
            .tone
            .set_output_values(steps_in_scale * OCTAVES);

        needs_save
    }

    fn generate_dsp_attributes(&mut self) -> DSPAttributes {
        let trigger_attributes = if self.parameters.trigger.triggered() {
            let note_index = self.parameters.tone.selected_value();
            let chord_group_index = self.parameters.chord_group.selected_value();
            let chord_index = self.parameters.chord.selected_value();
            let scale_group_index = self.parameters.scale_group.selected_value();
            let scale_index = self.parameters.scale.selected_value();
            let arp_index = self.parameters.arp.selected_value();

            // TODO: Figure out where to keep the scale. In control and pass it by
            // reference to arp, or fully in arp.
            let scale = self.scales.scale(scale_group_index, scale_index).unwrap();

            self.arp.apply_configuration(ArpeggiatorConfiguration {
                // TODO
                tonic: Tonic::C,
                // TODO: No unwrap or safety note
                root: scale
                    .with_tonic(Tonic::C)
                    .get_note_by_index_ascending(note_index)
                    .unwrap(),
                scale,
                // SAFETY: Parameter values used to get group and chord index
                // are always limited based on the selected chord group.
                chord: self.chords.chord(chord_group_index, chord_index).unwrap(),
                // SAFETY: Parameter values used to get arp index are
                // statically limited by the maximum number of modes.
                mode: ArpeggiatorMode::try_from_index(arp_index).unwrap(),
            });

            if let Some(note) = self.arp.pop(&mut self.random_generator) {
                self.display.set(
                    Priority::Fallback,
                    Screen::Step(StepScreen::with_step(note.index as usize)),
                );
                let contour = self.parameters.contour.value();
                let dsp_trigger_attributes = DSPTriggerAttributes {
                    frequency: note.tone.frequency(),
                    contour,
                };
                defmt::info!("DSP trigger attributes={:?}", dsp_trigger_attributes);
                Some(dsp_trigger_attributes)
            } else {
                None
            }
        } else {
            None
        };

        DSPAttributes {
            resonance: self.parameters.resonance.value(),
            cutoff: self.parameters.cutoff.value(),
            trigger: trigger_attributes,
        }
    }

    pub fn tick(&mut self) -> ControlOutputState {
        self.display.tick();
        ControlOutputState {
            leds: if let Some(active_screen) = self.display.active_screen() {
                active_screen.leds()
            } else {
                [false; 8]
            },
        }
    }
}

fn linear_sum(pot: f32, cv: Option<f32>) -> f32 {
    let offset_cv = cv.unwrap_or(0.0) / 5.0;
    let sum = pot + offset_cv;
    let clamped = sum.clamp(0.0, 1.0);
    clamped
}
