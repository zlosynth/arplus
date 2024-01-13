pub struct Instrument;

pub struct Attributes;

impl Instrument {
    pub fn new(_sample_rate: f32) -> Self {
        Self
    }

    pub fn process(&mut self, _buffer: &mut [(f32, f32); 32]) {}

    pub fn set_attributes(&mut self, _attributes: Attributes) {}
}
