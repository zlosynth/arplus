#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::let_and_return)]

mod arpeggiator;
mod chords;
mod display;
mod inputs;
mod parameters;
mod random;
mod save;
mod scales;

use arplus_dsp::{Attributes as DSPAttributes, TriggerAttributes as DSPTriggerAttributes};

pub use crate::inputs::ControlInputSnapshot;
pub use crate::save::{Save, WrappedSave};

use crate::arpeggiator::{Arpeggiator, Configuration as ArpeggiatorConfiguration};
use crate::chords::Chords;
use crate::display::Screen;
use crate::display::{Display, Priority, StepScreen};
use crate::inputs::Inputs;
use crate::inputs::{Button, Cv, Gate, Pot};
use crate::parameters::Parameters;
use crate::random::RandomGenerator;
use crate::scales::Scales;

const HOLD_TO_QUERY: usize = 400;

pub struct Controller {
    display: Display,
    inputs: Inputs,
    parameters: Parameters,
    arp: Arpeggiator,
    random_generator: RandomGenerator,
    // TODO: Implement calibration.
    // TODO: Implement configuration.
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
        // TODO: Recover them from an input snapshot too.
        let parameters = Parameters::new(save.parameters, Chords::new(), Scales::new());

        Self {
            display: Display::new(),
            arp: Arpeggiator::with_config(build_arp_config(&parameters)),
            parameters,
            inputs: Inputs::new(),
            random_generator: RandomGenerator::with_seed(seed),
        }
    }

    pub fn apply_input_snapshot(&mut self, snapshot: ControlInputSnapshot) -> Result {
        self.inputs.apply_input_snapshot(snapshot);
        let (needs_save, display_request) = self.reconcile_parameters_with_inputs();
        let save = if needs_save {
            Some(self.generate_save())
        } else {
            None
        };
        self.apply_display_request(display_request);
        let dsp_attributes = self.generate_dsp_attributes();

        Result {
            save,
            dsp_attributes,
        }
    }

    fn reconcile_parameters_with_inputs(&mut self) -> (bool, DisplayRequest) {
        let pots = &self.inputs.pots;
        let buttons = &self.inputs.buttons;
        let cvs = &self.inputs.cvs;
        let gates = &self.inputs.gates;
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
        reconcile_contour(&pots.contour, &cvs.contour, &mut parameters.contour);
        reconcile_cutoff(&pots.cutoff, &cvs.cutoff, &mut parameters.cutoff);
        reconcile_resonance(&pots.resonance, &cvs.resonance, &mut parameters.resonance);
        reconcile_scale(
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
        reconcile_trigger(&buttons.trigger, &gates.trigger, &mut parameters.trigger);

        (needs_save, display_request)
    }

    fn generate_dsp_attributes(&mut self) -> DSPAttributes {
        let trigger_attributes = if self.parameters.trigger.triggered() {
            self.arp.apply_config(build_arp_config(&self.parameters));

            if let Some(note) = self.arp.pop(&mut self.random_generator) {
                self.display.set(
                    Priority::Fallback,
                    Screen::Step(StepScreen::with_step(note.index() as usize)),
                );
                let dsp_trigger_attributes = DSPTriggerAttributes {
                    frequency: note.tone().frequency(),
                    contour: self.parameters.contour.value(),
                };
                defmt::debug!("DSP trigger attributes={:?}", dsp_trigger_attributes);
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

fn reconcile_trigger(button: &Button, cv: &Gate, parameter: &mut parameters::Trigger) {
    parameter.reconcile(button.clicked(), cv.triggered());
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
        group_pot.value(),
        group_cv.value(),
        chord_pot.value(),
        chord_cv.value(),
    );
    *needs_save |= changed_group || changed_chord;
    if changed_group {
        let size = parameter.selected_group_size();
        display_request.set(Priority::Active, Screen::chord_group(size));
    } else if changed_chord {
        let chord = parameter.selected_chord();
        display_request.set(Priority::Active, Screen::chord(chord));
    }
    // TODO: If active above treshold, show it too
}

fn reconcile_resonance(pot: &Pot, cv: &Cv, parameter: &mut parameters::Resonance) {
    parameter.reconcile(pot.value(), cv.value());
}

fn reconcile_cutoff(pot: &Pot, cv: &Cv, parameter: &mut parameters::Cutoff) {
    parameter.reconcile(pot.value(), cv.value());
}

fn reconcile_contour(pot: &Pot, cv: &Cv, parameter: &mut parameters::Contour) {
    parameter.reconcile(pot.value(), cv.value());
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
                tone_pot.value(),
                tone_cv.value(),
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
            let selected = parameter.selected_note().index();
            display_request.set(Priority::Active, Screen::note(selected as usize));
        }
    } else if group_held {
        let selected = parameter.selected_group_id();
        display_request.set(Priority::Queried, Screen::scale_group(selected));
    } else if scale_held {
        let selected = parameter.selected_scale_index();
        display_request.set(Priority::Queried, Screen::scale(selected));
    }
    // TODO: If tone active above treshold, show it too
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
    button.released() && button.released_after() <= HOLD_TO_QUERY
}

fn is_button_held(button: &Button) -> bool {
    button.held_for() > HOLD_TO_QUERY
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

    fn take_active_screen(&mut self) -> Option<Screen> {
        self.prioritized[Priority::Active as usize].take()
    }

    fn take_queried_screen(&mut self) -> Option<Screen> {
        self.prioritized[Priority::Queried as usize].take()
    }
}

fn build_arp_config(parameters: &Parameters) -> ArpeggiatorConfiguration {
    ArpeggiatorConfiguration {
        root: parameters.scale.selected_note(),
        scale: parameters.scale.selected_scale(),
        chord: parameters.chord.selected_chord(),
        mode: parameters.arp_mode.selected(),
    }
}
