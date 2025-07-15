use crate::system::hal::gpio;

// XXX: If the length of this changes, it must be reflected in `mod_32`.
const SEQUENCE: [bool; 32] = [
    true, true, false, true, false, false, true, false, true, true, true, false, true, false,
    false, false, false, false, false, true, true, false, true, false, true, true, true, false,
    false, false, false, true,
];

#[derive(defmt::Format)]
pub struct AudioProbeInterface {
    pub broadcaster: Broadcaster,
    pub detector: Detector,
}

impl AudioProbeInterface {
    pub fn new(broadcaster_pin: BroadcasterPin) -> Self {
        Self {
            broadcaster: Broadcaster::new(broadcaster_pin),
            detector: Detector::default(),
        }
    }
}

#[derive(defmt::Format)]
pub struct Broadcaster {
    position: usize,
    pin: BroadcasterPin,
}

pub type BroadcasterPin = gpio::gpiob::PB9<gpio::Output>;

impl Broadcaster {
    pub fn new(pin: BroadcasterPin) -> Self {
        let mut broadcaster = Self { position: 0, pin };
        broadcaster.tick(); // Make sure to start in the first position
        broadcaster
    }

    pub fn tick(&mut self) {
        let value = SEQUENCE[self.position];
        self.position = mod_32(self.position + 1);
        self.pin.set_state(value.into());
    }
}

#[derive(Default, defmt::Format)]
pub struct Detector {
    position: usize,
    queue: [bool; SEQUENCE.len()],
    detected_cache: bool,
}

impl Detector {
    pub fn write_from_left(&mut self, buffer: &[(f32, f32)]) {
        let last_sample_index = buffer.len() - 1;
        self.write(buffer[last_sample_index].0 > 0.0);
    }

    fn write(&mut self, value: bool) {
        // NOTE: The audio input is read inverted.
        self.queue[self.position] = !value;
        self.position = mod_32(self.position + 1);
    }

    pub fn detected(&mut self) -> bool {
        if self.position == 0 {
            self.detected_cache =
                compare_circular_buffers_with_tolerance(&self.queue, &SEQUENCE, 4);
        }
        self.detected_cache
    }
}

fn compare_circular_buffers_with_tolerance(
    a: &[bool; 32],
    b: &[bool; 32],
    tolerance: usize,
) -> bool {
    'outter: for offset in 0..32 {
        let mut misses = 0;
        for i in 0..32 {
            if a[mod_32(offset + i)] != b[i] {
                misses += 1;
                if misses > tolerance {
                    continue 'outter;
                }
            }
        }
        return true;
    }
    false
}

fn mod_32(x: usize) -> usize {
    x & 0b1_1111
}
