use super::buffer::Buffer;

pub struct Pot {
    buffer: Buffer<32>,
    last_activation_movement: u32,
}

impl Pot {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            last_activation_movement: 0,
        }
    }

    pub fn reconcile(&mut self, value: f32) {
        self.buffer.write(value);

        self.last_activation_movement = if self.traveled_more_than(0.013) {
            0
        } else {
            self.last_activation_movement.saturating_add(1)
        };
    }

    pub fn value(&self) -> f32 {
        // NOTE: Raw value is enough, since a LPF is already applied on the
        // pot input.
        self.buffer.read_raw()
    }

    pub fn activation_movement(&self) -> bool {
        self.last_activation_movement == 0
    }

    fn traveled_more_than(&self, toleration: f32) -> bool {
        libm::fabsf(self.buffer.traveled()) > toleration
    }
}
