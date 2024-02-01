#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::let_and_return)]

mod arpeggiator;
mod chords;
mod inputs;
mod parameters;
pub mod save;
mod scales;

use arplus_dsp::{Attributes as DSPAttributes, TriggerAttributes as DSPTriggerAttributes};
pub use inputs::ControlInputSnapshot;
use inputs::Inputs;
use parameters::Parameters;
use save::Save;

pub struct Controller {
    inputs: Inputs,
    parameters: Parameters,
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
    pub instrument_attributes: DSPAttributes,
}

pub struct ControlOutputState {
    pub leds: [bool; 8],
}

impl Controller {
    pub fn new(save: Save) -> Self {
        Self {
            inputs: Inputs::new(),
            parameters: Parameters::new(save.parameters),
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
        // TODO: Converge internal state - reconfigure arp
        let dsp_attributes = self.generate_dsp_attributes();

        Result {
            save,
            instrument_attributes: dsp_attributes,
        }
    }

    fn reconcile_parameters_with_inputs(&mut self) -> bool {
        let pots = &self.inputs.pots;
        let buttons = &self.inputs.buttons;
        let cvs = &self.inputs.cvs;
        let parameters = &mut self.parameters;

        let mut needs_save = false;

        let tone = linear_sum(pots.tone.value, cvs.tone.value);
        needs_save |= parameters.tone.reconcile(tone);

        let chord = linear_sum(pots.chord.value, cvs.chord.value);
        needs_save |= parameters.chord.reconcile(chord);

        let contour = linear_sum(pots.contour.value, cvs.contour.value);
        parameters.contour.reconcile(contour);

        let gain = linear_sum(pots.gain.value, cvs.gain.value);
        parameters.gain.reconcile(gain);

        let cutoff = linear_sum(pots.cutoff.value, cvs.cutoff.value);
        parameters.cutoff.reconcile(cutoff);

        let resonance = linear_sum(pots.resonance.value, cvs.resonance.value);
        parameters.resonance.reconcile(resonance);

        needs_save |= parameters.tonic.reconcile(buttons.tonic.released);

        needs_save |= parameters.mode.reconcile(buttons.mode.released);

        needs_save |= parameters.arp.reconcile(buttons.arp.released);

        let triggered = buttons.trigger.clicked || cvs.trigger.triggered;
        parameters.trigger.reconcile(triggered);

        needs_save
    }

    fn generate_dsp_attributes(&self) -> DSPAttributes {
        let trigger_attributes = if self.parameters.trigger.triggered() {
            let _ = self.parameters.tone.selected_value();
            let _ = self.parameters.chord.selected_value();
            let _ = self.parameters.tonic.selected_value();
            let _ = self.parameters.mode.selected_value();
            Some(DSPTriggerAttributes {
                frequency: 200.0,
                contour: 0.0,
            })
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
