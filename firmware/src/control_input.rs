use arplus_control::ControlInputSnapshot;

pub struct ControlInputInterface;

impl ControlInputInterface {
    pub fn sample(&mut self) {}

    pub fn snapshot(&self) -> ControlInputSnapshot {
        todo!();
    }
}
