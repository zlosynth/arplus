use arplus_control::OutputState;

pub struct ControlOutputInterface;

impl ControlOutputInterface {
    pub fn set_state(&mut self, _state: &OutputState) {}
}
