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
use inputs::{Button, Buttons, Cv, CvTrigger, Pot};
use parameters::{Continuous, Discrete, Trigger};

use crate::arpeggiator::{
    Arpeggiator, Configuration as ArpeggiatorConfiguration, Mode as ArpeggiatorMode,
};
use crate::chords::Chords;
use crate::display::{ArpModeScreen, Display, Priority, Screen, StepScreen};
pub use crate::inputs::ControlInputSnapshot;
use crate::inputs::Inputs;
use crate::parameters::{Parameters, Toggle};
use crate::random::RandomGenerator;
use crate::save::Save;
use crate::scales::Scales;
use crate::scales::{scale_note::ScaleNote, tonic::Tonic};

const HOLD_TO_QUERY: usize = 400;

pub struct Controller {
    display: Display,
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

struct DisplayRequest {
    prioritized: [Option<Screen>; 5],
}

impl Controller {
    pub fn new(seed: u64, save: Save) -> Self {
        let scales = Scales::new();
        let chords = Chords::new();
        // TODO: Recover them from an input snapshot too.
        let parameters = Parameters::new(save.parameters, chords, scales);

        // TODO: This will require input snapshot and save to initialize itself as well.
        // Otherwise the root would move during the start. Once done, take all options
        // from parameters.
        // TODO: Once everything is taken from parameters, this can be move into
        // a function shared with the attribute reconciliation.
        let arp = Arpeggiator::new_with_configuration(ArpeggiatorConfiguration {
            tonic: Tonic::C,
            scale: parameters.scale.selected_scale(),
            root: ScaleNote::new(scales::quarter_tones::QuarterTone::C1, 0),
            chord: parameters.chord.selected_chord(),
            mode: parameters.arp_mode.selected(),
        });

        Self {
            display: Display::new(),
            parameters,
            arp,
            inputs: Inputs::new(),
            random_generator: RandomGenerator::with_seed(seed),
        }
    }

    pub fn apply_input_snapshot(&mut self, snapshot: ControlInputSnapshot) -> Result {
        self.inputs.apply_input_snapshot(snapshot);
        let (needs_save, display_request_a) = self.reconcile_parameters_with_inputs();
        let save = if needs_save {
            Some(self.generate_save())
        } else {
            None
        };
        let (dsp_attributes, display_request_b) = self.generate_dsp_attributes();
        self.apply_display_request(display_request_a.merge(display_request_b));

        Result {
            save,
            dsp_attributes,
        }
    }

    fn reconcile_parameters_with_inputs(&mut self) -> (bool, DisplayRequest) {
        let pots = &self.inputs.pots;
        let buttons = &self.inputs.buttons;
        let cvs = &self.inputs.cvs;
        let parameters = &mut self.parameters;

        let mut needs_save = false;
        let mut display_request = DisplayRequest::new();

        reconcile_chord(
            &pots.chord_group,
            &cvs.chord_group,
            &pots.chord,
            &cvs.chord,
            &mut parameters.chord,
            &mut display_request,
            &mut needs_save,
        );
        reconcile_continuous(&pots.contour, &cvs.contour, &mut parameters.contour);
        reconcile_continuous(&pots.cutoff, &cvs.cutoff, &mut parameters.cutoff);
        reconcile_continuous(&pots.resonance, &cvs.resonance, &mut parameters.resonance);
        reconcile_scale(
            // TODO: Unify tone/note naming
            &pots.tone,
            &cvs.tone,
            &buttons.scale_group,
            &buttons.scale,
            &mut parameters.scale,
            &mut display_request,
            &mut needs_save,
        );
        reconcile_arp_mode(
            &buttons.arp,
            &mut parameters.arp_mode,
            &mut display_request,
            &mut needs_save,
        );
        reconcile_trigger(&buttons.trigger, &cvs.trigger, &mut parameters.trigger);

        // TODO: Move the calculation of chords and tones here. It will be used for display
        // and then returned as an output.

        (needs_save, display_request)
    }

    fn generate_dsp_attributes(&mut self) -> (DSPAttributes, DisplayRequest) {
        let trigger_attributes = if self.parameters.trigger.triggered() {
            let note_index = self.parameters.scale.selected_note_index();
            let scale = self.parameters.scale.selected_scale();

            self.arp.apply_configuration(ArpeggiatorConfiguration {
                // TODO
                tonic: Tonic::C,
                // TODO: No unwrap or safety note
                root: scale
                    .with_tonic(Tonic::C)
                    .get_note_by_index_ascending(note_index)
                    .unwrap(),
                scale,
                chord: self.parameters.chord.selected_chord(),
                mode: self.parameters.arp_mode.selected(),
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

        (
            DSPAttributes {
                resonance: self.parameters.resonance.value(),
                cutoff: self.parameters.cutoff.value(),
                trigger: trigger_attributes,
            },
            DisplayRequest::new(),
        )
    }

    fn apply_display_request(&mut self, mut display_request: DisplayRequest) {
        if let Some(active_screen) = display_request.take_active_screen() {
            self.display.set(Priority::Active, active_screen);
        };

        if let Some(queried_screen) = display_request.take_queried_screen() {
            self.display.set(Priority::Queried, queried_screen);
        } else {
            self.display.reset(Priority::Queried);
        };
    }

    fn generate_save(&mut self) -> Save {
        let save = Save {
            parameters: self.parameters.copy_config(),
        };
        save
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

fn reconcile_trigger(button: &Button, cv: &CvTrigger, parameter: &mut Trigger) {
    parameter.reconcile(button.clicked, cv.triggered);
}

fn reconcile_chord(
    group_pot: &Pot,
    group_cv: &Cv,
    chord_pot: &Pot,
    chord_cv: &Cv,
    parameter: &mut parameters::Chord,
    display_request: &mut DisplayRequest,
    needs_save: &mut bool,
) {
    let (changed_group, changed_chord) = parameter.reconcile_group_and_chord(
        group_pot.value,
        group_cv.value,
        chord_pot.value,
        chord_cv.value,
    );
    *needs_save |= changed_group || changed_chord;
    if changed_group {
        let size = parameter.selected_group_size();
        display_request.set(Priority::Active, Screen::chord_group(size));
    } else if changed_chord {
        // TODO: Update display
    }
    // TODO: If active above treshold, show it too
}

fn reconcile_continuous(pot: &Pot, cv: &Cv, parameter: &mut Continuous) {
    parameter.reconcile(linear_sum(pot.value, cv.value));
}

fn reconcile_scale(
    tone_pot: &Pot,
    tone_cv: &Cv,
    group_button: &Button,
    scale_button: &Button,
    parameter: &mut parameters::Scale,
    display_request: &mut DisplayRequest,
    needs_save: &mut bool,
) {
    let group_held = is_button_held(group_button);
    let scale_held = is_button_held(scale_button);
    let group_tapped = was_button_tapped(group_button);
    let scale_tapped = was_button_tapped(scale_button);

    if group_tapped || scale_tapped {
        let (note_changed, group_changed, scale_changed) = parameter
            .reconcile_note_group_and_scale(
                tone_pot.value,
                tone_cv.value,
                group_tapped,
                scale_tapped,
            );
        *needs_save |= note_changed || group_changed || scale_changed;
        if group_changed {
            let selected = parameter.selected_group_id();
            display_request.set(Priority::Active, Screen::scale_group(selected));
        } else if scale_changed {
            let selected = parameter.selected_scale_index();
            display_request.set(Priority::Active, Screen::scale(selected));
        } else if note_changed {
            // TODO: Display the note. This will be easier once
            // the parameter retuns the actual note, instead of an index.
        }
    } else if group_held {
        let selected = parameter.selected_group_id();
        display_request.set(Priority::Queried, Screen::scale_group(selected));
    } else if scale_held {
        let selected = parameter.selected_scale_index();
        display_request.set(Priority::Queried, Screen::scale(selected));
    }
}

fn reconcile_arp_mode(
    button: &Button,
    parameter: &mut parameters::ArpMode,
    display_request: &mut DisplayRequest,
    needs_save: &mut bool,
) {
    if is_button_held(button) {
        let selected = parameter.selected();
        display_request.set(Priority::Queried, Screen::arp_mode(selected));
    } else if was_button_tapped(button) {
        *needs_save |= parameter.reconcile(true);
        let selected = parameter.selected();
        display_request.set(Priority::Active, Screen::arp_mode(selected));
    };
}

fn was_button_tapped(button: &Button) -> bool {
    button.released && button.released_after <= HOLD_TO_QUERY
}

fn is_button_held(button: &Button) -> bool {
    button.held_for > HOLD_TO_QUERY
}

impl DisplayRequest {
    fn new() -> Self {
        Self {
            prioritized: [None, None, None, None, None],
        }
    }

    fn set(&mut self, priority: Priority, screen: Screen) {
        self.prioritized[priority as usize] = Some(screen);
    }

    // TODO: This may not be needed in the end. And if it is not,
    // is the whole structure needed?
    fn merge(mut self, mut other: Self) -> Self {
        for (i, screen) in self.prioritized.iter_mut().enumerate() {
            *screen = screen.take().or(other.prioritized[i].take());
        }
        self
    }

    fn take_active_screen(&mut self) -> Option<Screen> {
        self.prioritized[Priority::Active as usize].take()
    }

    fn take_queried_screen(&mut self) -> Option<Screen> {
        self.prioritized[Priority::Queried as usize].take()
    }
}

fn linear_sum(pot: f32, cv: Option<f32>) -> f32 {
    let offset_cv = cv.unwrap_or(0.0) / 5.0;
    let sum = pot + offset_cv;
    let clamped = sum.clamp(0.0, 1.0);
    clamped
}
