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
use crate::parameters::{CvAssignment, Parameters, SCALE_OFFSET_MAX_STEPS};
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
    pending_replay_trigger: bool,
}

enum State {
    CalibratingTone(CalibrationPhase),
    CalibratingQuant(CalibrationPhase),
    Normal,
}

enum CalibrationPhase {
    Octave1,
    CountdownToOctave2(f32, u8),
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
            pending_replay_trigger: false,
        }
    }

    pub fn apply_input_snapshot(&mut self, snapshot: ControlInputSnapshot) -> Result {
        let mut needs_save = false;
        let mut display_request = display_request::DisplayRequest::new();

        self.inputs.apply_input_snapshot(snapshot);

        self.reconcile_tone_calibration(&mut display_request, &mut needs_save);
        self.reconcile_quant_calibration(&mut display_request, &mut needs_save);
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

    fn reconcile_tone_calibration(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let button = &self.inputs.buttons.rsnx;
        let state = &mut self.state;
        let tone_cv = &mut self.inputs.cvs.tone;

        match state {
            State::Normal => {
                if button.pressed() && tone_cv.just_plugged() {
                    display_request.set_tone_calibration_phase(Screen::tone_calibration_octave_1());
                    *state = State::CalibratingTone(CalibrationPhase::Octave1);
                }
            }
            State::CalibratingQuant(_) => (),
            State::CalibratingTone(CalibrationPhase::Octave1) => {
                if let Some(value) = tone_cv.raw_value() {
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
            State::CalibratingTone(CalibrationPhase::CountdownToOctave2(_, _)) => {
                // This phase is not used for tone calibration, only for quant calibration
            }
            State::CalibratingTone(CalibrationPhase::Octave2(octave_1)) => {
                if let Some(value) = tone_cv.raw_value() {
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
            State::Normal => {
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
                    // NOTE: It is set already here, so the CV gets a chance
                    // to stabilize before the countdown is complete.
                    self.quantized_output.force_octave_2();
                    *state =
                        State::CalibratingQuant(CalibrationPhase::CountdownToOctave2(value, 10));
                } else {
                    display_request.set_calibration_result(Screen::calibration_failure());
                    *state = State::Normal;
                }
            }
            State::CalibratingQuant(CalibrationPhase::CountdownToOctave2(octave_1, countdown)) => {
                if *countdown == 0 {
                    *state = State::CalibratingQuant(CalibrationPhase::Octave2(*octave_1));
                } else {
                    *countdown -= 1;
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

    fn reconcile_parameters_with_inputs(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        self.reconcile_chord(display_request, needs_save);
        self.reconcile_contour();
        self.reconcile_cutoff();
        self.reconcile_resonance();
        self.reconcile_pluck();
        self.reconcile_trigger(display_request);
        self.reconcile_reset_next();
        self.reconcile_scale(display_request, needs_save);
        self.reconcile_scale_offsets(display_request, needs_save);
        self.reconcile_arp_mode(display_request, needs_save);
        self.reconcile_stereo_mode(display_request, needs_save);
        self.reconcile_width();
        self.reconcile_strings(display_request, needs_save);
        self.reconcile_cv_assignment(display_request, needs_save);
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
        *needs_save |= (size_cv_value.is_none() && changed_size)
            || (size_cv_value.is_none() && chord_cv_value.is_none() && changed_chord);

        if size_pot.activation_movement()
            || (size_cv_value.is_none()
                && changed_size
                && !self.parameters.cv_assignment.just_changed())
        {
            let size = parameter.selected_size();
            display_request.set_queried_attribute(Screen::size(size));
        } else if chord_pot.activation_movement()
            || (chord_cv_value.is_none() && !changed_size && changed_chord)
        {
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
        let button = &self.inputs.buttons.rsnx;
        let cv = &self.inputs.gates.rsnx;
        let parameter = &mut self.parameters.reset_next;
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

    fn reconcile_width(&mut self) {
        let pot = &self.inputs.pots.width;
        let cv_value = self.width_cv();
        let parameter = &mut self.parameters.width;
        parameter.reconcile(pot.value(), cv_value);
    }

    fn reconcile_strings(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let pot = &self.inputs.pots.strings;
        let cv_value = self.strings_cv();
        let parameter = &mut self.parameters.strings;

        let changed_length = parameter.reconcile(pot.value(), cv_value);

        *needs_save |= cv_value.is_none() && changed_length;

        if pot.activation_movement()
            || (cv_value.is_none()
                && changed_length
                && !self.parameters.cv_assignment.just_changed())
        {
            let length = parameter.value();
            display_request.set_queried_attribute(Screen::strings(length));
        }
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
        let tone_cv = self.tone_cv();
        let group_button = &self.inputs.buttons.group;
        let scale_button = &self.inputs.buttons.scale;
        let tonic_pot = &self.inputs.pots.tonic;
        let group_cv = self.group_cv();
        let scale_cv = self.scale_cv();
        let tonic_cv = self.tonic_cv();
        let parameter = &mut self.parameters.scale;

        let group_held = is_button_held(group_button);
        let scale_held = is_button_held(scale_button);
        let group_tapped = was_button_tapped(group_button) && !self.inputs.buttons.rsnx.pressed();
        let scale_tapped = was_button_tapped(scale_button) && !self.inputs.buttons.rsnx.pressed();

        let (note_changed, octave_changed, group_changed, scale_changed, tonic_changed) = parameter
            .reconcile_note_tonic_group_and_scale(
                tone_pot.value(),
                tone_cv,
                group_tapped,
                scale_tapped,
                tonic_pot.value(),
                group_cv,
                scale_cv,
                tonic_cv,
            );

        *needs_save |= (group_cv.is_none() && group_changed)
            || (scale_cv.is_none() && scale_changed)
            || (tonic_cv.is_none() && tonic_changed)
            || octave_changed
            || (tone_cv.is_none()
                && group_cv.is_none()
                && scale_cv.is_none()
                && tonic_cv.is_none()
                && note_changed);

        if (group_changed && group_cv.is_none())
            || (scale_changed && scale_cv.is_none())
            || (tonic_changed && tonic_cv.is_none())
        {
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
        } else if tone_cv.is_none()
            && (tone_pot.activation_movement()
                || (note_changed
                    && !group_changed
                    && !scale_changed
                    && !self.parameters.cv_assignment.just_changed()))
        {
            let selected = parameter.selected_note().index();
            display_request.set_queried_attribute(Screen::note(selected as usize));
        } else if tone_cv.is_some() && (tone_pot.activation_movement() || octave_changed) {
            let selected = parameter.selected_octave_index();
            display_request.set_queried_attribute(Screen::octave(selected));
        } else if (tonic_cv.is_none()
            && tonic_changed
            && !self.parameters.cv_assignment.just_changed())
            || tonic_pot.activation_movement()
        {
            let selected = parameter.selected_tonic();
            display_request.set_queried_attribute(Screen::tonic(selected));
        }
    }

    fn reconcile_scale_offsets(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let rsnx_button = &self.inputs.buttons.rsnx;
        let group_button = &self.inputs.buttons.group;
        let scale_button = &self.inputs.buttons.scale;
        let stereo_button = &self.inputs.buttons.stereo;
        let cv_assignment_button = &self.inputs.buttons.cv_assignment;

        let requested_increase = rsnx_button.pressed() && group_button.clicked();
        let requested_decrease = rsnx_button.pressed() && scale_button.clicked();
        let requested_lock = rsnx_button.pressed() && cv_assignment_button.clicked();
        let requested_reset = rsnx_button.pressed() && stereo_button.held_for() > 1000;
        let queried = is_button_held(rsnx_button)
            && !(group_button.pressed()
                || scale_button.pressed()
                || stereo_button.pressed()
                || cv_assignment_button.pressed());

        let parameter = &mut self.parameters.scale_offsets;

        if queried || requested_increase || requested_decrease {
            let group_index = self.parameters.scale.selected_group_id() as usize;
            let scale_index = self.parameters.scale.selected_scale_index();
            let note_index = self.arp.last_note_index() as usize;
            let scale_size = self.parameters.scale.selected_scale_size();

            if !parameter.locked() && scale_size <= SCALE_OFFSET_MAX_STEPS {
                if requested_increase {
                    defmt::info!("Requested scales offset increase");
                    *needs_save |= parameter.request_increase(group_index, scale_index, note_index);
                    self.pending_replay_trigger = true;
                } else if requested_decrease {
                    defmt::info!("Requested scales offset decrease");
                    *needs_save |= parameter.request_decrease(group_index, scale_index, note_index);
                    self.pending_replay_trigger = true;
                }
            }

            let offset = parameter.offset(group_index, scale_index, note_index);
            display_request.set_queried_attribute(Screen::offset_query(offset));
        } else if requested_lock {
            let locked = parameter.toggle_lock();
            display_request.set_offset_animation(if locked {
                defmt::info!("Requested scales offset lock");
                Screen::offset_lock()
            } else {
                defmt::info!("Requested scales offset unlock");
                Screen::offset_unlock()
            });
        } else if requested_reset {
            defmt::info!("Requested scales offset reset");
            if !parameter.locked() {
                let group_index = self.parameters.scale.selected_group_id() as usize;
                let scale_index = self.parameters.scale.selected_scale_index();
                *needs_save |= parameter.reset_scale(group_index, scale_index);
                display_request.set_offset_animation(Screen::offset_reset());
            }
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

        if was_button_tapped(button) && cv_value.is_none() && !self.inputs.buttons.rsnx.pressed() {
            *needs_save |= parameter.reconcile_button(true);
        } else if let Some(cv_value) = cv_value {
            parameter.reconcile_cv(cv_value);
        }

        if is_button_held(button) || was_button_tapped(button) {
            display_request.set_queried_attribute(Screen::arp_mode(parameter.selected()));
        }
    }

    fn reconcile_stereo_mode(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let rsnx_button = &self.inputs.buttons.rsnx;
        if rsnx_button.pressed() {
            return;
        }

        let button = &self.inputs.buttons.stereo;
        let parameter = &mut self.parameters.stereo_mode;

        *needs_save |= parameter.reconcile_button(was_button_tapped(button));

        if is_button_held(button) || was_button_tapped(button) {
            display_request.set_queried_attribute(Screen::stereo_mode(parameter.selected()));
        }
    }

    fn reconcile_cv_assignment(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        needs_save: &mut bool,
    ) {
        let rsnx_button = &self.inputs.buttons.rsnx;
        if rsnx_button.pressed() {
            return;
        }

        let button = &self.inputs.buttons.cv_assignment;
        let parameter = &mut self.parameters.cv_assignment;

        *needs_save |= parameter.reconcile_button(was_button_tapped(button));

        if is_button_held(button) || was_button_tapped(button) {
            display_request.set_queried_attribute(Screen::cv_assignment(parameter.selected()));
        }
    }

    fn generate_dsp_attributes(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
    ) -> DSPAttributes {
        let trigger_attributes = if self.pending_replay_trigger {
            self.pending_replay_trigger = false;
            self.generate_note_trigger(display_request, true)
        } else if self.parameters.trigger.triggered_n4() {
            self.arp.apply_config(
                build_arp_config(&mut self.parameters),
                &mut self.random_generator,
            );
            self.generate_note_trigger(display_request, false)
        } else {
            None
        };

        DSPAttributes {
            resonance: self.parameters.resonance.value(),
            cutoff: self.parameters.cutoff.value(),
            trigger: trigger_attributes,
            // NOTE: The gain is set quite low, so even when all strings are
            // playing loud, it does not produce excessive clipping and rough
            // saturation.
            gain: 1.0 / 4.0,
            chord_size: self.parameters.chord.selected_size(),
            width: self.parameters.width.value(),
            stereo_mode: self.parameters.stereo_mode.selected().into(),
            strings: self.parameters.strings.value(),
        }
    }

    fn generate_note_trigger(
        &mut self,
        display_request: &mut display_request::DisplayRequest,
        replay: bool,
    ) -> Option<DSPTriggerAttributes> {
        let group_index = self.parameters.scale.selected_group_id() as usize;
        let scale_index = self.parameters.scale.selected_scale_index();
        let scale_offsets = self
            .parameters
            .scale_offsets
            .scale_offsets_ref(group_index, scale_index);

        let note_and_index = if replay {
            self.arp.replay_last(scale_offsets)
        } else {
            self.arp.pop(&mut self.random_generator, scale_offsets)
        };

        note_and_index.map(|(note, index)| {
            display_request.set_fallback_attribute(Screen::step(note.index() as usize));
            let dsp_trigger_attributes = DSPTriggerAttributes {
                frequency: note.tone().frequency(),
                contour: self.parameters.contour.value(),
                pluck: self.parameters.pluck.value(),
                is_root: index == 0,
            };
            defmt::debug!(
                "DSP {} trigger attributes={:?}",
                if replay { "replay" } else { "" },
                dsp_trigger_attributes
            );
            dsp_trigger_attributes
        })
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
        self.inputs.cvs.tone.value()
    }

    fn chord_cv(&self) -> Option<f32> {
        self.inputs.cvs.chord.value()
    }

    fn resonance_cv(&self) -> Option<f32> {
        self.inputs.cvs.resonance.value()
    }

    fn cutoff_cv(&self) -> Option<f32> {
        self.inputs.cvs.cutoff.value()
    }

    fn contour_cv(&self) -> Option<f32> {
        self.inputs.cvs.contour.value()
    }

    fn tonic_cv(&self) -> Option<f32> {
        self.assignable_cv(CvAssignment::Tonic)
    }

    fn chord_size_cv(&self) -> Option<f32> {
        self.assignable_cv(CvAssignment::Size)
    }

    fn arp_cv(&self) -> Option<f32> {
        self.assignable_cv(CvAssignment::Arp)
    }

    fn group_cv(&self) -> Option<f32> {
        self.assignable_cv(CvAssignment::Group)
    }

    fn scale_cv(&self) -> Option<f32> {
        self.assignable_cv(CvAssignment::Scale)
    }

    fn pluck_cv(&self) -> Option<f32> {
        self.assignable_cv(CvAssignment::Pluck)
    }

    fn strings_cv(&self) -> Option<f32> {
        self.assignable_cv(CvAssignment::Strings)
    }

    fn width_cv(&self) -> Option<f32> {
        self.assignable_cv(CvAssignment::Width)
    }

    fn assignable_cv(&self, seeked_assignment: CvAssignment) -> Option<f32> {
        if self.parameters.cv_assignment.selected() == seeked_assignment {
            self.inputs.cvs.assignable.value()
        } else {
            None
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
