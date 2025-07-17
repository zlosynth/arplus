use crate::display::{Display, Priority, Screen};

pub struct DisplayRequest {
    pub failure: ScreenRequest,
    pub calibration_result: ScreenRequest,
    pub calibration_phase: ScreenRequest,
    pub offset_locking: ScreenRequest,
    pub queried_attribute: ScreenRequest,
    pub fallback_attribute: ScreenRequest,
}

#[derive(defmt::Format)]
pub enum ScreenRequest {
    Set(Screen),
    Reset,
    Keep,
}

impl DisplayRequest {
    pub fn new() -> Self {
        Self {
            failure: ScreenRequest::Keep,
            calibration_result: ScreenRequest::Keep,
            calibration_phase: ScreenRequest::Keep,
            offset_locking: ScreenRequest::Keep,
            queried_attribute: ScreenRequest::Keep,
            fallback_attribute: ScreenRequest::Keep,
        }
    }

    pub fn apply(&mut self, display: &mut Display) {
        self.calibration_result
            .take()
            .process(display, Priority::Failure);
        self.failure.take().process(display, Priority::Failure);
        self.calibration_phase
            .take()
            .process(display, Priority::Dialog);
        self.offset_locking
            .take()
            .process(display, Priority::Animation);
        self.queried_attribute
            .take()
            .process(display, Priority::Queried);
        self.fallback_attribute
            .take()
            .process(display, Priority::Fallback);
    }

    pub fn set_calibration_result(&mut self, calibration_result: Screen) {
        self.calibration_result = ScreenRequest::Set(calibration_result);
    }

    pub fn set_tone_calibration_phase(&mut self, calibration_phase: Screen) {
        self.calibration_phase = ScreenRequest::Set(calibration_phase);
    }

    pub fn reset_calibration_phase(&mut self) {
        self.calibration_phase = ScreenRequest::Reset;
    }

    pub fn set_offset_animation(&mut self, offset_locking_screen: Screen) {
        self.offset_locking = ScreenRequest::Set(offset_locking_screen);
    }

    pub fn set_queried_attribute(&mut self, queried_attribute: Screen) {
        if !matches!(self.queried_attribute, ScreenRequest::Reset) {
            self.queried_attribute = ScreenRequest::Set(queried_attribute);
        }
    }

    pub fn reset_queried_attribute(&mut self) {
        self.queried_attribute = ScreenRequest::Reset;
    }

    pub fn set_fallback_attribute(&mut self, fallback_attribute: Screen) {
        self.fallback_attribute = ScreenRequest::Set(fallback_attribute);
    }
}

impl ScreenRequest {
    pub fn take(&mut self) -> Self {
        core::mem::replace(self, Self::Keep)
    }

    pub fn process(self, display: &mut Display, priority: Priority) {
        match self {
            ScreenRequest::Set(screen) => display.set(priority, screen),
            ScreenRequest::Reset => display.reset(priority),
            ScreenRequest::Keep => (),
        }
    }
}
