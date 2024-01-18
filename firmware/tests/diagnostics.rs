#![no_main]
#![no_std]

use arplus_control::ControlInputSnapshot;
use arplus_firmware as _; // Panic handler
use arplus_firmware::system::System;

struct Statistics {
    pots: [PotStatistics; 6],
    input_cvs: [InputCvStatistics; 6],
    input_gates: [InputGatesStatistics; 1],
    buttons: [ButtonStatistics; 4],
}

impl Statistics {
    fn new() -> Self {
        Self {
            pots: [
                PotStatistics::new(),
                PotStatistics::new(),
                PotStatistics::new(),
                PotStatistics::new(),
                PotStatistics::new(),
                PotStatistics::new(),
            ],
            input_cvs: [
                InputCvStatistics::new(),
                InputCvStatistics::new(),
                InputCvStatistics::new(),
                InputCvStatistics::new(),
                InputCvStatistics::new(),
                InputCvStatistics::new(),
            ],
            input_gates: [InputGatesStatistics::new()],
            buttons: [
                ButtonStatistics::new(),
                ButtonStatistics::new(),
                ButtonStatistics::new(),
                ButtonStatistics::new(),
            ],
        }
    }

    fn sample(&mut self, snapshot: ControlInputSnapshot) {
        for i in 0..snapshot.pots.len() {
            self.pots[i].sample(snapshot.pots[i]);
        }
        for i in 0..snapshot.cvs.len() {
            self.input_cvs[i].sample(snapshot.cvs[i]);
        }
        self.input_gates[0].sample(snapshot.trigger);
        for i in 0..snapshot.buttons.len() {
            self.buttons[i].sample(snapshot.buttons[i]);
        }
    }
}

impl defmt::Format for Statistics {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "\x1B[2J\x1b[1;1H");

        defmt::write!(fmt, "Pot\tValue\t\tMin\t\tMax\t\tNoise\n");
        for (i, pot) in self.pots.iter().enumerate() {
            defmt::write!(
                fmt,
                "{}\t{}\t{}\t{}\t{}%\n",
                i + 1,
                pot.value,
                pot.min,
                pot.max,
                pot.buffer.delta() * 100.0
            );
        }

        defmt::write!(fmt, "\nI.CV\tValue\t\tMin\t\tMax\t\tNoise\n");
        for (i, cv) in self.input_cvs.iter().enumerate() {
            defmt::write!(
                fmt,
                "{}\t{}\t{}\t{}\t{}%\n",
                i + 1,
                cv.value.unwrap_or(f32::NAN),
                cv.min,
                cv.max,
                cv.buffer.delta() * 100.0 / 10.0
            );
        }

        defmt::write!(fmt, "\nI.trig\tValue\tTrig(total)\tTrig(recent)\n");
        for (i, gate) in self.input_gates.iter().enumerate() {
            defmt::write!(
                fmt,
                "{}\t{}\t{}\t\t{}\n",
                i + 1,
                gate.value,
                gate.triggered,
                gate.buffer.trues()
            );
        }

        defmt::write!(fmt, "\nButton\tValue\tTrig(total)\tTrig(recent)\n");
        for (i, button) in self.buttons.iter().enumerate() {
            defmt::write!(
                fmt,
                "{}\t{}\t{}\t\t{}\n",
                i + 1,
                button.value,
                button.triggered,
                button.buffer.trues()
            );
        }
    }
}

struct PotStatistics {
    min: f32,
    max: f32,
    value: f32,
    buffer: F32Buffer,
}

impl PotStatistics {
    fn new() -> Self {
        Self {
            min: f32::MAX,
            max: f32::MIN,
            value: 0.0,
            buffer: F32Buffer::new(),
        }
    }

    fn sample(&mut self, value: f32) {
        self.min = f32::min(self.min, value);
        self.max = f32::max(self.max, value);
        self.value = value;
        self.buffer.write(value);
    }
}

struct InputCvStatistics {
    min: f32,
    max: f32,
    value: Option<f32>,
    buffer: F32Buffer,
}

impl InputCvStatistics {
    fn new() -> Self {
        Self {
            min: f32::MAX,
            max: f32::MIN,
            value: None,
            buffer: F32Buffer::new(),
        }
    }

    fn sample(&mut self, value: Option<f32>) {
        if let Some(value) = value {
            self.min = f32::min(self.min, value);
            self.max = f32::max(self.max, value);
            self.buffer.write(value);
        }
        self.value = value;
    }
}

struct InputGatesStatistics {
    value: bool,
    triggered: u32,
    buffer: BoolBuffer,
}

impl InputGatesStatistics {
    fn new() -> Self {
        Self {
            value: false,
            triggered: 0,
            buffer: BoolBuffer::new(),
        }
    }

    fn sample(&mut self, value: bool) {
        if !self.value && value {
            self.triggered += 1;
            self.buffer.write(true);
        } else {
            self.buffer.write(false);
        }
        self.value = value;
    }
}

struct ButtonStatistics {
    value: bool,
    triggered: u32,
    buffer: BoolBuffer,
}

impl ButtonStatistics {
    fn new() -> Self {
        Self {
            value: false,
            triggered: 0,
            buffer: BoolBuffer::new(),
        }
    }

    fn sample(&mut self, value: bool) {
        if !self.value && value {
            self.triggered += 1;
            self.buffer.write(true);
        } else {
            self.buffer.write(false);
        }
        self.value = value;
    }
}

struct F32Buffer {
    values: [f32; 512],
    index: usize,
}

impl F32Buffer {
    fn new() -> Self {
        Self {
            values: [0.0; 512],
            index: 0,
        }
    }

    fn write(&mut self, value: f32) {
        self.values[self.index] = value;
        self.index += 1;
        if self.index >= self.values.len() {
            self.index -= self.values.len();
        }
    }

    fn delta(&self) -> f32 {
        let min: f32 = self
            .values
            .iter()
            .fold(f32::MAX, |a, b| if a < *b { a } else { *b });
        let max: f32 = self
            .values
            .iter()
            .fold(f32::MIN, |a, b| if a > *b { a } else { *b });
        max - min
    }
}

struct BoolBuffer {
    values: [bool; 512],
    index: usize,
}

impl BoolBuffer {
    fn new() -> Self {
        Self {
            values: [false; 512],
            index: 0,
        }
    }

    fn write(&mut self, value: bool) {
        self.values[self.index] = value;
        self.index += 1;
        if self.index >= self.values.len() {
            self.index -= self.values.len();
        }
    }

    fn trues(&self) -> usize {
        self.values.iter().filter(|x| **x).count()
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Running diagnostics");

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = daisy::pac::Peripherals::take().unwrap();
    let system = System::init(cp, dp);

    let mut statistics = Statistics::new();
    let mut control_input_interface = system.control_input_interface;

    // Warm up.
    for _ in 0..1000 {
        control_input_interface.sample();
        cortex_m::asm::delay(480_000);
    }

    loop {
        for _ in 0..100 {
            control_input_interface.sample();
            statistics.sample(control_input_interface.snapshot());
            cortex_m::asm::delay(1_000_000);
        }

        defmt::println!("{}", statistics);
    }
}
