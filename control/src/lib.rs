#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::let_and_return)]

#[cfg(test)]
#[macro_use]
extern crate approx;

mod arpeggiator;
mod chords;
mod display;
mod display_request;
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
use crate::display::Display;
use crate::display::Screen;
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
    // TODO: Implement configuration.
    state: State,
}

enum State {
    Calibrating(CalibrationPhase),
    Normal,
}

enum CalibrationPhase {
    Octave1,
    Octave2(f32),
}

pub struct Result {
    pub save: Option<Save>,
    pub dsp_attributes: DSPAttributes,
}

pub struct ControlOutputState {
    pub leds: [bool; 8],
}

impl Controller {
    pub fn new(seed: u64, save: Save) -> Self {
        // TODO: Recover them from an input snapshot too.
        let parameters = Parameters::new(save.parameters, Chords::new(), Scales::new());

        Self {
            display: Display::new(),
            arp: Arpeggiator::with_config(build_arp_config(&parameters)),
            parameters,
            inputs: Inputs::new(save.inputs),
            random_generator: RandomGenerator::with_seed(seed),
            state: State::Normal,
        }
    }

    pub fn apply_input_snapshot(&mut self, snapshot: ControlInputSnapshot) -> Result {
        let mut needs_save = false;
        let mut display_request = display_request::DisplayRequest::new();

        self.inputs.apply_input_snapshot(snapshot);

        self.reconcile_calibration(&mut display_request, &mut needs_save);
        self.reconcile_parameters_with_inputs(&mut display_request, &mut needs_save);

        let dsp_attributes = self.generate_dsp_attributes(&mut display_request);

        self.apply_display_request(display_request);

        let save = if needs_save {
            Some(self.generate_save())
        } else {
            None
        };

        Result {
            save,
            dsp_attributes,
        }
    }

    fn reconcile_calibration(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let trigger_button = &self.inputs.buttons.trigger;
        let state = &mut self.state;
        let tone_cv = &mut self.inputs.cvs.tone;

        match state {
            State::Normal => {
                if trigger_button.pressed() && tone_cv.just_plugged() {
                    display_request.set_calibration_phase(Screen::calibration_octave_1());
                    *state = State::Calibrating(CalibrationPhase::Octave1);
                }
            }
            State::Calibrating(CalibrationPhase::Octave1) => {
                if let Some(value) = tone_cv.value() {
                    if trigger_button.clicked() {
                        display_request.set_calibration_phase(Screen::calibration_octave_2());
                        *state = State::Calibrating(CalibrationPhase::Octave2(value));
                    }
                } else {
                    display_request.set_calibration_result(Screen::calibration_failure());
                    display_request.reset_calibration_phase();
                    *state = State::Normal;
                }
            }
            State::Calibrating(CalibrationPhase::Octave2(octave_1)) => {
                if let Some(value) = tone_cv.value() {
                    if trigger_button.clicked() {
                        if let Ok(_) = tone_cv.update_calibration(*octave_1, value) {
                            defmt::info!(
                                "Successfully completed calibration, O1={:?} O2={:?}",
                                *octave_1,
                                value
                            );
                            *needs_save |= true;
                            display_request.set_calibration_result(Screen::calibration_success());
                        } else {
                            defmt::info!("Failed calibration, O1={:?} O2={:?}", *octave_1, value);
                            display_request.set_calibration_result(Screen::calibration_failure());
                        }
                        display_request.reset_calibration_phase();
                        *state = State::Normal;
                    }
                } else {
                    display_request.set_calibration_result(Screen::calibration_failure());
                    display_request.reset_calibration_phase();
                    *state = State::Normal;
                }
            }
        };
    }

    fn reconcile_parameters_with_inputs(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let pots = &self.inputs.pots;
        let buttons = &self.inputs.buttons;
        let cvs = &mut self.inputs.cvs;
        let gates = &self.inputs.gates;
        let parameters = &mut self.parameters;

        reconcile_chord(
            &pots.chord_group,
            &cvs.chord_group,
            &pots.chord,
            &cvs.chord,
            parameters.scale.selected_scale_size(),
            &mut parameters.chord,
            display_request,
            needs_save,
        );
        reconcile_contour(&pots.contour, &cvs.contour, &mut parameters.contour);
        reconcile_cutoff(&pots.cutoff, &cvs.cutoff, &mut parameters.cutoff);
        reconcile_resonance(&pots.resonance, &cvs.resonance, &mut parameters.resonance);
        reconcile_scale(
            &pots.tone,
            &cvs.tone,
            &buttons.scale_group,
            &buttons.scale,
            &buttons.trigger,
            &mut parameters.scale,
            display_request,
            needs_save,
        );
        reconcile_arp_mode(
            &buttons.arp,
            &mut parameters.arp_mode,
            display_request,
            needs_save,
        );
        reconcile_trigger(&buttons.trigger, &gates.trigger, &mut parameters.trigger);
    }

    fn generate_dsp_attributes(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
    ) -> DSPAttributes {
        let trigger_attributes = if self.parameters.trigger.triggered() {
            self.arp.apply_config(build_arp_config(&self.parameters));

            if let Some((note, index)) = self.arp.pop(&mut self.random_generator) {
                display_request.set_fallback_attribute(Screen::step(index as usize));
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

    fn apply_display_request(&mut self, mut display_request: display_request::DisplayRequest) {
        display_request.apply(&mut self.display);
    }

    fn generate_save(&mut self) -> Save {
        let save = Save {
            parameters: self.parameters.copy_config(),
            inputs: self.inputs.copy_config(),
        };
        save
    }

    pub fn tick(&mut self) -> ControlOutputState {
        self.display.tick();
        ControlOutputState {
            leds: if let Some((active_screen, clock)) = self.display.active_screen_and_clock() {
                active_screen.leds(clock)
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
    scale_size: usize,
    parameter: &mut parameters::Chord,
    display_request: &mut display_request::DisplayRequest,
    needs_save: &mut bool,
) {
    let (changed_group, changed_chord) = parameter.reconcile_group_chord_and_scale_size(
        group_pot.value(),
        group_cv.value(),
        chord_pot.value(),
        chord_cv.value(),
        scale_size,
    );
    *needs_save |= changed_group || changed_chord;
    if changed_group {
        let size = parameter.selected_group_size();
        display_request.set_active_attribute(Screen::chord_group(size));
    } else if changed_chord {
        let chord = parameter.selected_chord();
        display_request.set_active_attribute(Screen::chord(chord, scale_size));
    } else if group_pot.activation_movement() {
        let size = parameter.selected_group_size();
        display_request.set_queried_attribute(Screen::chord_group(size));
    } else if chord_pot.activation_movement() {
        let chord = parameter.selected_chord();
        display_request.set_queried_attribute(Screen::chord(chord, scale_size));
    }
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
    trigger_button: &Button,
    parameter: &mut parameters::Scale,
    display_request: &mut display_request::DisplayRequest,
    needs_save: &mut bool,
) {
    let group_held = is_button_held(group_button);
    let scale_held = is_button_held(scale_button);
    let trigger_held = is_button_held(trigger_button);
    let group_tapped = was_button_tapped(group_button);
    let scale_tapped = was_button_tapped(scale_button);

    let (note_changed, group_changed, scale_changed) = parameter
        .reconcile_note_tonic_group_and_scale(
            tone_pot.value(),
            tone_cv.value(),
            group_tapped,
            scale_tapped,
            trigger_held,
        );
    *needs_save |= note_changed || group_changed || scale_changed;
    if group_changed {
        let selected = parameter.selected_group_id();
        display_request.set_active_attribute(Screen::scale_group(selected));
    } else if scale_changed {
        let selected = parameter.selected_scale_index();
        display_request.set_active_attribute(Screen::scale(selected));
    } else if note_changed {
        let selected = parameter.selected_note().index();
        display_request.set_active_attribute(Screen::note(selected as usize));
    }

    if group_changed || scale_changed {
        defmt::info!(
            "Selected scale_group={:?} scale={:?}",
            parameter.selected_group_id(),
            parameter.selected_scale_index()
        );
    }

    // TODO: Display tonic if it has changed
    if group_held {
        let selected = parameter.selected_group_id();
        display_request.set_queried_attribute(Screen::scale_group(selected));
    } else if scale_held {
        let selected = parameter.selected_scale_index();
        display_request.set_queried_attribute(Screen::scale(selected));
    } else if tone_pot.activation_movement() {
        let selected = parameter.selected_note().index();
        display_request.set_queried_attribute(Screen::note(selected as usize));
    }
}

fn reconcile_arp_mode(
    button: &Button,
    parameter: &mut parameters::ArpMode,
    display_request: &mut display_request::DisplayRequest,
    needs_save: &mut bool,
) {
    if is_button_held(button) {
        let selected = parameter.selected();
        display_request.set_queried_attribute(Screen::arp_mode(selected));
    } else if was_button_tapped(button) {
        *needs_save |= parameter.reconcile(true);
        let selected = parameter.selected();
        display_request.set_active_attribute(Screen::arp_mode(selected));
    };
}

fn was_button_tapped(button: &Button) -> bool {
    button.released() && button.released_after() <= HOLD_TO_QUERY
}

fn is_button_held(button: &Button) -> bool {
    button.held_for() > HOLD_TO_QUERY
}

fn build_arp_config(parameters: &Parameters) -> ArpeggiatorConfiguration {
    ArpeggiatorConfiguration {
        root: parameters.scale.selected_note(),
        scale: parameters.scale.selected_scale(),
        chord: parameters.chord.selected_chord(),
        mode: parameters.arp_mode.selected(),
    }
}
