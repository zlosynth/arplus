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

use arplus_dsp::{
    Attributes as DSPAttributes, StereoMode as DSPStereoMode,
    TriggerAttributes as DSPTriggerAttributes,
};

pub use crate::inputs::ControlInputSnapshot;
pub use crate::save::{Save, WrappedSave};

use crate::arpeggiator::{Arpeggiator, Configuration as ArpeggiatorConfiguration};
use crate::chords::Chords;
use crate::display::{Display, Screen};
use crate::inputs::{Button, Inputs};
use crate::parameters::{CvMappingSocket, Parameters};
use crate::random::RandomGenerator;
use crate::scales::Scales;

const HOLD_TO_QUERY: usize = 400;

pub struct Controller {
    display: Display,
    inputs: Inputs,
    parameters: Parameters,
    arp: Arpeggiator,
    random_generator: RandomGenerator,
    state: State,
}

enum State {
    Calibrating(CalibrationPhase),
    Configuring,
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
        self.reconcile_configuration(&mut display_request, &mut needs_save);
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
            State::Normal | State::Configuring => {
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
                        if tone_cv.update_calibration(*octave_1, value).is_ok() {
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

    fn reconcile_configuration(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let state = &mut self.state;

        match state {
            State::Calibrating(_) => (),
            State::Normal => {
                if self.inputs.buttons.scale_group.held_for() > 3000
                    && self.inputs.buttons.scale.held_for() > 3000
                    && self.inputs.buttons.arp.held_for() > 3000
                {
                    defmt::info!("Entering configuration");
                    self.state = State::Configuring;
                }
            }
            State::Configuring => {
                if self.inputs.buttons.scale_group.clicked()
                    || self.inputs.buttons.scale.clicked()
                    || self.inputs.buttons.arp.clicked()
                {
                    self.state = State::Normal
                } else {
                    display_request.set_fallback_attribute(Screen::configuration());

                    if self.inputs.pots.chord_group.activation_movement() {
                        let changed = self
                            .parameters
                            .gain
                            .reconcile(self.inputs.pots.chord_group.value());
                        if changed {
                            display_request.set_active_attribute(Screen::gain(
                                self.parameters.gain.selected_index(),
                            ));
                            *needs_save |= true;
                        }
                        display_request.set_queried_attribute(Screen::gain(
                            self.parameters.gain.selected_index(),
                        ));
                    }

                    if self.inputs.pots.tone.activation_movement() {
                        let changed = self
                            .parameters
                            .cv_mapping
                            .reconcile_scale_group_mapping(self.inputs.pots.chord_group.value());
                        if changed {
                            display_request.set_active_attribute(Screen::cv_mapping(
                                self.parameters.cv_mapping.scale_group_socket(),
                            ));
                            *needs_save |= true;
                        }
                        display_request.set_queried_attribute(Screen::cv_mapping(
                            self.parameters.cv_mapping.scale_group_socket(),
                        ));
                    }

                    if self.inputs.pots.resonance.activation_movement() {
                        let changed = self
                            .parameters
                            .cv_mapping
                            .reconcile_scale_mapping(self.inputs.pots.resonance.value());
                        if changed {
                            display_request.set_active_attribute(Screen::cv_mapping(
                                self.parameters.cv_mapping.scale_socket(),
                            ));
                            *needs_save |= true;
                        }
                        display_request.set_queried_attribute(Screen::cv_mapping(
                            self.parameters.cv_mapping.scale_socket(),
                        ));
                    }

                    if self.inputs.pots.chord.activation_movement() {
                        let changed = self
                            .parameters
                            .cv_mapping
                            .reconcile_arp_mapping(self.inputs.pots.chord.value());
                        if changed {
                            display_request.set_active_attribute(Screen::cv_mapping(
                                self.parameters.cv_mapping.arp_socket(),
                            ));
                            *needs_save |= true;
                        }
                        display_request.set_queried_attribute(Screen::cv_mapping(
                            self.parameters.cv_mapping.arp_socket(),
                        ));
                    }

                    if self.inputs.pots.cutoff.activation_movement() {
                        let changed = self
                            .parameters
                            .cv_mapping
                            .reconcile_tonic_mapping(self.inputs.pots.cutoff.value());
                        if changed {
                            display_request.set_active_attribute(Screen::cv_mapping(
                                self.parameters.cv_mapping.tonic_socket(),
                            ));
                            *needs_save |= true;
                        }
                        display_request.set_queried_attribute(Screen::cv_mapping(
                            self.parameters.cv_mapping.tonic_socket(),
                        ));
                    }

                    if self.inputs.pots.contour.activation_movement() {
                        let changed = self
                            .parameters
                            .stereo_mode
                            .reconcile(self.inputs.pots.contour.value());
                        if changed {
                            display_request.set_active_attribute(Screen::stereo_mode(
                                self.parameters.stereo_mode.selected(),
                            ));
                            *needs_save |= true;
                        }
                        display_request.set_queried_attribute(Screen::stereo_mode(
                            self.parameters.stereo_mode.selected(),
                        ));
                    }
                }
            }
        }
    }

    fn reconcile_parameters_with_inputs(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        self.reconcile_chord(display_request, needs_save);
        self.reconcile_contour();
        self.reconcile_cutoff();
        self.reconcile_resonance();
        self.reconcile_trigger();
        self.reconcile_scale(display_request, needs_save);
        self.reconcile_arp_mode(display_request, needs_save);
    }

    fn reconcile_chord(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let group_pot = &self.inputs.pots.chord_group;
        let group_cv_value = self.chord_group_cv();
        let chord_pot = &self.inputs.pots.chord;
        let chord_cv_value = self.chord_cv();
        let scale_size = self.parameters.scale.selected_scale_size();
        let parameter = &mut self.parameters.chord;

        let (changed_group, changed_chord) = parameter.reconcile_group_chord_and_scale_size(
            group_pot.value(),
            group_cv_value,
            chord_pot.value(),
            chord_cv_value,
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

    fn reconcile_trigger(&mut self) {
        let button = &self.inputs.buttons.trigger;
        let cv = &self.inputs.gates.trigger;
        let parameter = &mut self.parameters.trigger;
        parameter.reconcile(button.clicked(), cv.triggered());
    }

    fn reconcile_resonance(&mut self) {
        let pot = &self.inputs.pots.resonance;
        let cv_value = self.resonance_cv();
        let parameter = &mut self.parameters.resonance;
        parameter.reconcile(pot.value(), cv_value);
    }

    fn reconcile_cutoff(&mut self) {
        let pot = &self.inputs.pots.cutoff;
        let cv_value = self.cutoff_cv();
        let parameter = &mut self.parameters.cutoff;
        parameter.reconcile(pot.value(), cv_value);
    }

    fn reconcile_contour(&mut self) {
        let pot = &self.inputs.pots.contour;
        let cv_value = self.contour_cv();
        let parameter = &mut self.parameters.contour;
        parameter.reconcile(pot.value(), cv_value);
    }

    fn reconcile_scale(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let tone_pot = &self.inputs.pots.tone;
        let tone_cv_value = self.tone_cv();
        let group_button = &self.inputs.buttons.scale_group;
        let scale_button = &self.inputs.buttons.scale;
        let trigger_button = &self.inputs.buttons.trigger;
        let group_cv = self.scale_group_cv();
        let scale_cv = self.scale_cv();
        let tonic_cv = self.tonic_cv();
        let parameter = &mut self.parameters.scale;

        let group_held = is_button_held(group_button);
        let scale_held = is_button_held(scale_button);
        let trigger_held = is_button_held(trigger_button);
        let group_tapped = was_button_tapped(group_button);
        let scale_tapped = was_button_tapped(scale_button);

        if (group_tapped && group_cv.is_some()) || (scale_tapped && scale_cv.is_some()) {
            display_request.set_failure(Screen::failure());
        }

        let (note_changed, octave_changed, group_changed, scale_changed, tonic_changed) = parameter
            .reconcile_note_tonic_group_and_scale(
                tone_pot.value(),
                tone_cv_value,
                group_tapped,
                scale_tapped,
                trigger_held,
                group_cv,
                scale_cv,
                tonic_cv,
            );
        *needs_save |=
            note_changed || group_changed || scale_changed || octave_changed || tonic_changed;

        if group_changed && group_cv.is_none() {
            let selected = parameter.selected_group_id();
            display_request.set_active_attribute(Screen::scale_group(selected));
        } else if scale_changed && scale_cv.is_none() {
            let selected = parameter.selected_scale_index();
            display_request.set_active_attribute(Screen::scale(selected));
        } else if note_changed && tone_cv_value.is_none() {
            let selected = parameter.selected_note().index();
            display_request.set_active_attribute(Screen::note(selected as usize));
        } else if octave_changed && tone_cv_value.is_some() {
            let selected = parameter.selected_octave_index();
            display_request.set_active_attribute(Screen::octave(selected));
        } else if tonic_changed && tonic_cv.is_none() {
            let selected = parameter.selected_tonic();
            display_request.set_active_attribute(Screen::tonic(selected));
        }

        if group_changed || scale_changed {
            defmt::info!(
                "Selected scale_group={:?} scale={:?}",
                parameter.selected_group_id(),
                parameter.selected_scale_index()
            );
        }

        if group_held {
            let selected = parameter.selected_group_id();
            display_request.set_queried_attribute(Screen::scale_group(selected));
        } else if scale_held {
            let selected = parameter.selected_scale_index();
            display_request.set_queried_attribute(Screen::scale(selected));
        } else if tone_pot.activation_movement() && !trigger_held && tone_cv_value.is_none() {
            let selected = parameter.selected_note().index();
            display_request.set_queried_attribute(Screen::note(selected as usize));
        } else if tone_pot.activation_movement() && !trigger_held && tone_cv_value.is_some() {
            let selected = parameter.selected_octave_index();
            display_request.set_queried_attribute(Screen::octave(selected));
        } else if tone_pot.activation_movement() && trigger_held {
            let selected = parameter.selected_tonic();
            display_request.set_queried_attribute(Screen::tonic(selected));
        }
    }

    fn reconcile_arp_mode(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let button = &self.inputs.buttons.arp;
        let cv_value = self.arp_cv();
        let parameter = &mut self.parameters.arp_mode;

        parameter.set_cv_control(self.parameters.cv_mapping.arp_socket().is_some());

        if is_button_held(button) {
            let selected = parameter.selected();
            display_request.set_queried_attribute(Screen::arp_mode(selected));
        } else if was_button_tapped(button) && cv_value.is_none() {
            *needs_save |= parameter.reconcile_button(true);
            let selected = parameter.selected();
            display_request.set_active_attribute(Screen::arp_mode(selected));
        } else if was_button_tapped(button) && cv_value.is_some() {
            display_request.set_failure(Screen::failure());
        } else if let Some(cv_value) = cv_value {
            parameter.reconcile_cv(cv_value);
        };
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
                    is_root: index == 0,
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
            gain: self.parameters.gain.value(),
            stereo_mode: self.parameters.stereo_mode.selected(),
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

    fn tone_cv(&self) -> Option<f32> {
        self.socket_cv_unless_remapped(CvMappingSocket::Tone)
    }

    fn chord_cv(&self) -> Option<f32> {
        self.socket_cv_unless_remapped(CvMappingSocket::Chord)
    }

    fn resonance_cv(&self) -> Option<f32> {
        self.socket_cv_unless_remapped(CvMappingSocket::Resonance)
    }

    fn cutoff_cv(&self) -> Option<f32> {
        self.socket_cv_unless_remapped(CvMappingSocket::Cutoff)
    }

    fn chord_group_cv(&self) -> Option<f32> {
        self.socket_cv_unless_remapped(CvMappingSocket::ChordGroup)
    }

    fn contour_cv(&self) -> Option<f32> {
        self.socket_cv_unless_remapped(CvMappingSocket::Contour)
    }

    fn tonic_cv(&self) -> Option<f32> {
        self.socket_cv(self.parameters.cv_mapping.tonic_socket())
    }

    fn scale_group_cv(&self) -> Option<f32> {
        self.socket_cv(self.parameters.cv_mapping.scale_group_socket())
    }

    fn scale_cv(&self) -> Option<f32> {
        self.socket_cv(self.parameters.cv_mapping.scale_socket())
    }

    fn arp_cv(&self) -> Option<f32> {
        self.socket_cv(self.parameters.cv_mapping.arp_socket())
    }

    fn socket_cv_unless_remapped(&self, socket: CvMappingSocket) -> Option<f32> {
        if self.parameters.cv_mapping.is_socket_remapped(socket) {
            None
        } else {
            self.socket_cv(socket)
        }
    }

    fn socket_cv(&self, socket: CvMappingSocket) -> Option<f32> {
        match socket {
            CvMappingSocket::None => None,
            CvMappingSocket::Tone => self.inputs.cvs.tone.value(),
            CvMappingSocket::Chord => self.inputs.cvs.chord.value(),
            CvMappingSocket::ChordGroup => self.inputs.cvs.chord_group.value(),
            CvMappingSocket::Resonance => self.inputs.cvs.resonance.value(),
            CvMappingSocket::Cutoff => self.inputs.cvs.cutoff.value(),
            CvMappingSocket::Contour => self.inputs.cvs.contour.value(),
        }
    }
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
