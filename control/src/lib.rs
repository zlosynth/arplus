#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::let_and_return)]

mod arpeggiator;
mod chords;
mod inputs;
mod parameters;
mod random;
pub mod save;
mod scales;

use arpeggiator::{
    Arpeggiator, Configuration as ArpeggiatorConfiguration, Mode as ArpeggiatorMode,
};
use arplus_dsp::{Attributes as DSPAttributes, TriggerAttributes as DSPTriggerAttributes};
use chords::Chord;
pub use inputs::ControlInputSnapshot;
use inputs::Inputs;
use parameters::Parameters;
use random::RandomGenerator;
use save::Save;
use scales::{
    scale::{Scale, S, T},
    scale_note::ScaleNote,
    tonic::Tonic,
};

pub struct Controller {
    inputs: Inputs,
    parameters: Parameters,
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
        // TODO: This will require input snapshot to initialize itself as well.
        // TODO: This would be recovered from save.
        let arp_config = ArpeggiatorConfiguration {
            scale: Scale::new(Tonic::C, &[T, T, S, T, T, T, S]).unwrap(),
            root: ScaleNote::new(scales::quarter_tones::QuarterTone::C1, 0),
            chord: Chord::from_slice(&[0, 2, 4]).unwrap(),
            mode: ArpeggiatorMode::Root,
        };
        Self {
            inputs: Inputs::new(),
            parameters: Parameters::new(save.parameters),
            arp: Arpeggiator::new_with_configuration(arp_config),
            random_generator: RandomGenerator::with_seed(seed),
        }
    }

    pub fn apply_input_snapshot(&mut self, snapshot: ControlInputSnapshot) -> Result {
        self.inputs.apply_input_snapshot(snapshot);
        let needs_save = self.reconcile_parameters_with_inputs();
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

    fn reconcile_parameters_with_inputs(&mut self) -> bool {
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
        parameters
            .contour
            .reconcile(linear_sum(pots.contour.value, cvs.contour.value));
        parameters
            .gain
            .reconcile(linear_sum(pots.gain.value, cvs.gain.value));
        parameters
            .cutoff
            .reconcile(linear_sum(pots.cutoff.value, cvs.cutoff.value));
        parameters
            .resonance
            .reconcile(linear_sum(pots.resonance.value, cvs.resonance.value));
        needs_save |= parameters.scale.reconcile(buttons.tonic.released);
        needs_save |= parameters.mode.reconcile(buttons.mode.released);
        needs_save |= parameters.arp.reconcile(buttons.arp.released);
        parameters
            .trigger
            .reconcile(buttons.trigger.clicked || cvs.trigger.triggered);

        needs_save
    }

    fn generate_dsp_attributes(&mut self) -> DSPAttributes {
        let trigger_attributes = if self.parameters.trigger.triggered() {
            let note_index = self.parameters.tone.selected_value();
            let _ = self.parameters.chord.selected_value();
            let _ = self.parameters.scale.selected_value();
            let _ = self.parameters.mode.selected_value();
            let arp_index = self.parameters.arp.selected_value();
            // todo!("Pass that to arp");
            // todo!("Pop note and its frequency");
            // todo!("Pass proper random");

            // TODO: Figure out where to keep the scale. In control and pass it by
            // reference to arp, or fully in arp.
            let scale = Scale::new(Tonic::C, &[T, T, S, T, T, T, S]).unwrap();
            self.arp.apply_configuration(ArpeggiatorConfiguration {
                root: scale.get_note_by_index_ascending(note_index).unwrap(),
                scale,
                chord: Chord::from_slice(&[0, 2, 4]).unwrap(),
                mode: ArpeggiatorMode::try_from_index(arp_index).unwrap(),
            });

            if let Some(note) = self.arp.pop(&mut self.random_generator) {
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
            gain: self.parameters.gain.value(),
            resonance: self.parameters.resonance.value(),
            cutoff: self.parameters.cutoff.value(),
            trigger: trigger_attributes,
        }
    }

    pub fn tick(&mut self) -> ControlOutputState {
        ControlOutputState { leds: [true; 8] }
    }
}

fn linear_sum(pot: f32, cv: Option<f32>) -> f32 {
    let offset_cv = cv.unwrap_or(0.0) / 5.0;
    let sum = pot + offset_cv;
    let clamped = sum.clamp(0.0, 1.0);
    clamped
}
