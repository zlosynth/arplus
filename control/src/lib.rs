#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::let_and_return)]

#[cfg(test)]
#[macro_use]
extern crate approx;

mod arpeggiator;
mod calibration;
mod chords;
mod display;
mod display_request;
mod inputs;
mod parameters;
mod quantized_output;
mod random;
mod save;
mod scales;

use arplus_dsp::{Attributes as DSPAttributes, TriggerAttributes as DSPTriggerAttributes};

pub use crate::inputs::ControlInputSnapshot;
pub use crate::save::{Save, WrappedSave};

use crate::arpeggiator::{Arpeggiator, Configuration as ArpeggiatorConfiguration};
use crate::chords::Chords;
use crate::display::{Display, Screen};
use crate::inputs::{Button, Inputs};
use crate::parameters::{CvMappingSocket, Parameters};
use crate::quantized_output::QuantizedOutput;
use crate::random::RandomGenerator;
use crate::scales::Scales;

const HOLD_TO_QUERY: usize = 400;

pub struct Controller {
    display: Display,
    inputs: Inputs,
    quantized_output: QuantizedOutput,
    parameters: Parameters,
    arp: Arpeggiator,
    random_generator: RandomGenerator,
    state: State,
}

enum State {
    CalibratingTone(CalibrationPhase),
    CalibratingQuant(CalibrationPhase),
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
    pub cv: f32,
}

impl Controller {
    pub fn new(seed: u64, save: Save) -> Self {
        let mut parameters = Parameters::new(save.parameters, Chords::new(), Scales::new());

        Self {
            display: Display::new(),
            arp: Arpeggiator::with_config(build_arp_config(&mut parameters)),
            parameters,
            inputs: Inputs::new(save.inputs),
            quantized_output: QuantizedOutput::with_config(save.quantized_output),
            random_generator: RandomGenerator::with_seed(seed),
            state: State::Normal,
        }
    }

    pub fn apply_input_snapshot(&mut self, snapshot: ControlInputSnapshot) -> Result {
        let mut needs_save = false;
        let mut display_request = display_request::DisplayRequest::new();

        self.inputs.apply_input_snapshot(snapshot);

        self.reconcile_tone_calibration(&mut display_request, &mut needs_save);
        self.reconcile_quant_calibration(&mut display_request, &mut needs_save);
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

    // TODO: Implement output calibration
    // - [X] Rename all Calibration stuff to ToneCalibration
    // - [X] Use RSNX button for ToneCalibration
    // - [ ] Add calibration attributes to the Quan output abstraction
    // - [ ] Enter quan calibration and exit right away
    // - [ ] Confirm that quant is connected to TONE with a probe sequence
    // - [ ] Read two octaves
    // - [ ] Add animation to all steps

    fn reconcile_tone_calibration(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let button = &self.inputs.buttons.rsnx;
        let state = &mut self.state;
        let tone_cv = &mut self.inputs.cvs.tone;

        match state {
            State::Normal | State::Configuring => {
                if button.pressed() && tone_cv.just_plugged() {
                    display_request.set_tone_calibration_phase(Screen::tone_calibration_octave_1());
                    *state = State::CalibratingTone(CalibrationPhase::Octave1);
                }
            }
            State::CalibratingQuant(_) => (),
            State::CalibratingTone(CalibrationPhase::Octave1) => {
                if let Some(value) = tone_cv.value() {
                    if button.clicked() {
                        display_request
                            .set_tone_calibration_phase(Screen::tone_calibration_octave_2());
                        *state = State::CalibratingTone(CalibrationPhase::Octave2(value));
                    }
                } else {
                    display_request.set_calibration_result(Screen::calibration_failure());
                    display_request.reset_calibration_phase();
                    *state = State::Normal;
                }
            }
            State::CalibratingTone(CalibrationPhase::Octave2(octave_1)) => {
                if let Some(value) = tone_cv.value() {
                    if button.clicked() {
                        if tone_cv.update_calibration(*octave_1, value).is_ok() {
                            defmt::info!(
                                "Successfully completed tone calibration, O1={:?} O2={:?}",
                                *octave_1,
                                value
                            );
                            *needs_save |= true;
                            display_request.set_calibration_result(Screen::calibration_success());
                        } else {
                            defmt::info!(
                                "Failed tone calibration, O1={:?} O2={:?}",
                                *octave_1,
                                value
                            );
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

    fn reconcile_quant_calibration(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let button = &self.inputs.buttons.arp;
        let state = &mut self.state;
        let tone_cv = &mut self.inputs.cvs.tone;

        match state {
            State::Normal | State::Configuring => {
                if button.pressed() && tone_cv.just_plugged() {
                    *state = State::CalibratingQuant(CalibrationPhase::Octave1);
                    self.quantized_output.force_octave_1();
                }
            }
            State::CalibratingTone(_) => (),
            State::CalibratingQuant(CalibrationPhase::Octave1) => {
                // NOTE: Start calibrating on release, so the CV is already stabilized.
                if button.pressed() {
                    return;
                }

                if let Some(value) = tone_cv.value() {
                    *state = State::CalibratingQuant(CalibrationPhase::Octave2(value));
                    self.quantized_output.force_octave_2();
                } else {
                    display_request.set_calibration_result(Screen::calibration_failure());
                    *state = State::Normal;
                }
            }
            State::CalibratingQuant(CalibrationPhase::Octave2(octave_1)) => {
                if let Some(value) = tone_cv.value() {
                    self.quantized_output.remove_force();
                    if self
                        .quantized_output
                        .update_calibration(*octave_1, value)
                        .is_ok()
                    {
                        defmt::info!(
                            "Successfully completed quant calibration, O1={:?} O2={:?}",
                            *octave_1,
                            value
                        );
                        *needs_save |= true;
                        display_request.set_calibration_result(Screen::calibration_success());
                    } else {
                        defmt::info!(
                            "Failed quant calibration, O1={:?} O2={:?}",
                            *octave_1,
                            value
                        );
                        display_request.set_calibration_result(Screen::calibration_failure());
                    }
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
            State::CalibratingTone(_) => (),
            State::CalibratingQuant(_) => (),
            State::Normal => {
                if self.inputs.buttons.group.held_for() > 3000
                    && self.inputs.buttons.scale.held_for() > 3000
                {
                    defmt::info!("Entering configuration");
                    self.state = State::Configuring;
                }
            }
            State::Configuring => {
                if self.inputs.buttons.group.clicked() || self.inputs.buttons.scale.clicked() {
                    defmt::info!("Exiting configuration");
                    self.state = State::Normal;
                    display_request.reset_fallback_attribute();
                } else {
                    // Configuration
                    // =============
                    // POT -> ATTRIBUTE
                    // Tone -> Tonic CV mapping
                    // Chord -> Gain
                    // Chord Size -> Chord size CV mapping
                    // Resonance -> Group CV mapping
                    // Cutoff -> Scale CV mapping
                    // Contour -> Arp CV mapping
                    // Pluck -> Pluck CV mapping

                    display_request.set_fallback_attribute(Screen::configuration());

                    let cv_mapping = &mut self.parameters.cv_mapping;
                    let pots = &mut self.inputs.pots;

                    // Tone knob selects tonic CV.
                    if pots.tone.activation_movement() {
                        *needs_save |= cv_mapping.reconcile_tonic_mapping(pots.tone.value());
                        display_request
                            .set_queried_attribute(Screen::cv_mapping(cv_mapping.tonic_socket()));
                    }

                    // Chord knob sets gain.
                    if pots.chord.activation_movement() {
                        *needs_save |= self.parameters.gain.reconcile(pots.chord.value());
                        display_request.set_queried_attribute(Screen::gain(
                            self.parameters.gain.selected_index(),
                        ));
                    }

                    // Chord size knob selects chord size CV.
                    if pots.chord_size.activation_movement() {
                        *needs_save |= cv_mapping.reconcile_size_mapping(pots.chord_size.value());
                        display_request
                            .set_queried_attribute(Screen::cv_mapping(cv_mapping.size_socket()));
                    }

                    // Resonance knob selects group CV.
                    if pots.resonance.activation_movement() {
                        *needs_save |= cv_mapping.reconcile_group_mapping(pots.resonance.value());
                        display_request
                            .set_queried_attribute(Screen::cv_mapping(cv_mapping.group_socket()));
                    }

                    // Cutoff knob selects scale CV.
                    if pots.cutoff.activation_movement() {
                        *needs_save |= cv_mapping.reconcile_scale_mapping(pots.cutoff.value());
                        display_request
                            .set_queried_attribute(Screen::cv_mapping(cv_mapping.scale_socket()));
                    }

                    // Contour knob selects arp CV.
                    if pots.contour.activation_movement() {
                        *needs_save |= cv_mapping.reconcile_arp_mapping(pots.contour.value());
                        display_request
                            .set_queried_attribute(Screen::cv_mapping(cv_mapping.arp_socket()));
                    }

                    // Pluck knob selects pluck CV.
                    if pots.pluck.activation_movement() {
                        *needs_save |= cv_mapping.reconcile_pluck_mapping(pots.pluck.value());
                        display_request
                            .set_queried_attribute(Screen::cv_mapping(cv_mapping.pluck_socket()));
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
        // NOTE: Things get confusing otherwise.
        if matches!(self.state, State::Configuring) {
            return;
        }

        self.reconcile_chord(display_request, needs_save);
        self.reconcile_contour();
        self.reconcile_cutoff();
        self.reconcile_resonance();
        self.reconcile_pluck();
        self.reconcile_trigger(display_request);
        self.reconcile_reset_next();
        self.reconcile_scale(display_request, needs_save);
        self.reconcile_arp_mode(display_request, needs_save);
    }

    fn reconcile_chord(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let size_pot = &self.inputs.pots.chord_size;
        let size_cv_value = self.chord_size_cv();
        let chord_pot = &self.inputs.pots.chord;
        let chord_cv_value = self.chord_cv();
        let scale_size = self.parameters.scale.selected_scale_size();
        let parameter = &mut self.parameters.chord;

        let (changed_size, changed_chord) = parameter.reconcile_size_chord_and_scale_size(
            size_pot.value(),
            size_cv_value,
            chord_pot.value(),
            chord_cv_value,
            scale_size,
        );
        *needs_save |=
            size_cv_value.is_none() && chord_cv_value.is_none() && (changed_size || changed_chord);

        // TODO FIXME: Changing size using CV trigers chord display
        if size_pot.activation_movement() || (size_cv_value.is_none() && changed_size) {
            let size = parameter.selected_size();
            display_request.set_queried_attribute(Screen::size(size));
        } else if chord_pot.activation_movement() || (chord_cv_value.is_none() && changed_chord) {
            let chord = parameter.selected_chord();
            display_request.set_queried_attribute(Screen::chord(chord, scale_size));
        }
    }

    fn reconcile_trigger(&mut self, display_request: &mut display_request::DisplayRequest) {
        let button = &self.inputs.buttons.trigger;
        let cv = &self.inputs.gates.trigger;
        let parameter = &mut self.parameters.trigger;
        parameter.reconcile(button.clicked(), cv.triggered());
        if button.clicked() {
            display_request.reset_queried_attribute();
        }
    }

    fn reconcile_reset_next(&mut self) {
        // TODO XXX: This must be preserved until the trigger is received.
        let button = &self.inputs.buttons.rsnx;
        let cv = &self.inputs.gates.rsnx;
        let parameter = &mut self.parameters.reset_next;
        // TODO: This can only set it high. It is reset when read
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

    fn reconcile_pluck(&mut self) {
        let pot = &self.inputs.pots.pluck;
        let cv_value = self.pluck_cv();
        let parameter = &mut self.parameters.pluck;
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
        // TODO: Tonic button
        let tone_cv_value = self.tone_cv();
        let group_button = &self.inputs.buttons.group;
        let scale_button = &self.inputs.buttons.scale;
        let tonic_button = &self.inputs.buttons.tonic;
        let group_cv = self.group_cv();
        let scale_cv = self.scale_cv();
        let tonic_cv = self.tonic_cv();
        let parameter = &mut self.parameters.scale;

        let group_held = is_button_held(group_button);
        let scale_held = is_button_held(scale_button);
        let tonic_held = is_button_held(tonic_button);
        let group_tapped = was_button_tapped(group_button);
        let scale_tapped = was_button_tapped(scale_button);

        let (note_changed, octave_changed, group_changed, scale_changed, tonic_changed) = parameter
            .reconcile_note_tonic_group_and_scale(
                tone_pot.value(),
                tone_pot.activation_movement(),
                tone_cv_value,
                group_tapped,
                scale_tapped,
                tonic_held,
                group_cv,
                scale_cv,
                tonic_cv,
            );

        *needs_save |= (group_cv.is_none() && group_changed)
            || (scale_cv.is_none() && scale_changed)
            || (tonic_cv.is_none() && tonic_changed)
            || octave_changed
            || (tone_cv_value.is_none()
                && group_cv.is_none()
                && scale_cv.is_none()
                && tonic_cv.is_none()
                && note_changed);

        if group_changed || scale_changed || tonic_changed {
            defmt::info!(
                "Selected group={:?} scale={:?} tonic={:?}",
                parameter.selected_group_id(),
                parameter.selected_scale_index(),
                parameter.selected_tonic(),
            );
        }

        if group_held || group_tapped || (group_changed && group_cv.is_none()) {
            let selected = parameter.selected_group_id();
            display_request.set_queried_attribute(Screen::group(selected));
        } else if scale_held || scale_tapped || (scale_changed && scale_cv.is_none()) {
            let selected = parameter.selected_scale_index();
            display_request.set_queried_attribute(Screen::scale(selected));
        } else if !tonic_held
            && tone_cv_value.is_none()
            && (tone_pot.activation_movement()
                || (note_changed && !group_changed && !scale_changed))
        {
            let selected = parameter.selected_note().index();
            display_request.set_queried_attribute(Screen::note(selected as usize));
        } else if !tonic_held
            && tone_cv_value.is_some()
            && (tone_pot.activation_movement() || octave_changed)
        {
            let selected = parameter.selected_octave_index();
            display_request.set_queried_attribute(Screen::octave(selected));
        } else if tonic_held {
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

        parameter.set_cv_control(cv_value.is_some());

        if is_button_held(button) || was_button_tapped(button) && cv_value.is_some() {
            let selected = parameter.selected();
            display_request.set_queried_attribute(Screen::arp_mode(selected));
        } else if was_button_tapped(button) && cv_value.is_none() {
            *needs_save |= parameter.reconcile_button(true);
            let selected = parameter.selected();
            display_request.set_queried_attribute(Screen::arp_mode(selected));
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
        let trigger_attributes = if self.parameters.trigger.triggered_n1() {
            self.arp.apply_config(
                build_arp_config(&mut self.parameters),
                &mut self.random_generator,
            );

            if let Some((note, index)) = self.arp.pop(&mut self.random_generator) {
                display_request.set_fallback_attribute(Screen::step(note.index() as usize));
                let dsp_trigger_attributes = DSPTriggerAttributes {
                    frequency: note.tone().frequency(),
                    contour: self.parameters.contour.value(),
                    pluck: self.parameters.pluck.value(),
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
            chord_size: self.parameters.chord.selected_size(),
        }
    }

    fn apply_display_request(&mut self, mut display_request: display_request::DisplayRequest) {
        display_request.apply(&mut self.display);
    }

    fn generate_save(&mut self) -> Save {
        let save = Save {
            parameters: self.parameters.copy_config(),
            inputs: self.inputs.copy_config(),
            quantized_output: self.quantized_output.copy_config(),
        };
        save
    }

    pub fn tick(&mut self) -> ControlOutputState {
        self.display.tick();
        // TODO: Find a better place for this. Probably under reconcile methods.
        self.quantized_output.reconcile(self.arp.last_voct_output());
        ControlOutputState {
            leds: if let Some((active_screen, clock)) = self.display.active_screen_and_clock() {
                active_screen.leds(clock)
            } else {
                [false; 8]
            },
            cv: self.quantized_output.value(),
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

    fn pluck_cv(&self) -> Option<f32> {
        self.socket_cv(self.parameters.cv_mapping.pluck_socket())
    }

    fn chord_size_cv(&self) -> Option<f32> {
        self.socket_cv(self.parameters.cv_mapping.size_socket())
    }

    fn contour_cv(&self) -> Option<f32> {
        self.socket_cv_unless_remapped(CvMappingSocket::Contour)
    }

    fn tonic_cv(&self) -> Option<f32> {
        self.socket_cv(self.parameters.cv_mapping.tonic_socket())
    }

    fn group_cv(&self) -> Option<f32> {
        self.socket_cv(self.parameters.cv_mapping.group_socket())
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

fn build_arp_config(parameters: &mut Parameters) -> ArpeggiatorConfiguration {
    ArpeggiatorConfiguration {
        root: parameters.scale.selected_note(),
        scale: parameters.scale.selected_scale(),
        chord: parameters.chord.selected_chord(),
        mode: parameters.arp_mode.selected(),
        reset_next: parameters.reset_next.pop(),
    }
}
