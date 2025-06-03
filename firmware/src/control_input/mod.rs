mod buttons;
mod cvs;
mod debouncer;
mod gates;
mod multiplexer;
mod one_pole_filter;
mod pots;
mod probe;

use arplus_control::ControlInputSnapshot;

pub use self::buttons::Pins as ButtonsPins;
pub use self::cvs::Pins as CvsPins;
pub use self::gates::Pins as GatesPins;
pub use self::multiplexer::Pins as MultiplexerPins;
pub use self::pots::Pins as PotsPins;
pub use self::probe::BroadcasterPin as ProbeBroadcasterPin;

use self::buttons::Buttons;
use self::cvs::Cvs;
use self::gates::Gates;
use self::multiplexer::Multiplexer;
use self::pots::Pots;
use self::probe::Broadcaster as ProbeBroadcaster;
use crate::system::hal::adc::{Adc, Enabled};
use crate::system::hal::pac::{ADC1, ADC2};

// To avoid crosstalk, it is necessary to let multiplexer settle after
// the source was changed. With 3 ticks crosstalk gets down to 0.5 %,
// with 4 ticks it is not measured.
const STABILIZATION_TICKS: u8 = 4;

pub struct ControlInputInterface {
    pots: Pots,
    buttons: Buttons,
    cvs: Cvs,
    gates: Gates,
    probe: ProbeBroadcaster,
    multiplexer: Multiplexer,
    adc_1: Adc<ADC1, Enabled>,
    adc_2: Adc<ADC2, Enabled>,
    cycle: u8,
    stabilization: u8,
}

pub struct Config {
    pub pots_pins: PotsPins,
    pub buttons_pins: ButtonsPins,
    pub cvs_pins: CvsPins,
    pub gates_pins: GatesPins,
    pub probe_pin: ProbeBroadcasterPin,
    pub multiplexer_pins: MultiplexerPins,
    pub adc_1: Adc<ADC1, Enabled>,
    pub adc_2: Adc<ADC2, Enabled>,
}

impl ControlInputInterface {
    pub fn new(config: Config) -> Self {
        Self {
            pots: Pots::new(config.pots_pins),
            buttons: Buttons::new(config.buttons_pins),
            cvs: Cvs::new(config.cvs_pins),
            gates: Gates::new(config.gates_pins),
            probe: ProbeBroadcaster::new(config.probe_pin),
            multiplexer: Multiplexer::new(config.multiplexer_pins),
            adc_1: config.adc_1,
            adc_2: config.adc_2,
            cycle: 0,
            stabilization: 0,
        }
    }

    pub fn sample(&mut self) {
        self.cvs.sample(&mut self.adc_1, &mut self.adc_2);
        self.gates.sample();

        self.stabilization += 1;
        if self.stabilization == STABILIZATION_TICKS {
            self.stabilization = 0;

            self.pots
                .sample(self.cycle, &mut self.adc_1, &mut self.adc_2);
            self.buttons.sample(self.cycle);

            // XXX: Selection happens at the end so the signal gets a chance
            // to propagate to mux before the next reading cycle.
            self.cycle = Multiplexer::next_position(self.cycle);
            self.multiplexer.select(self.cycle);
        }

        // XXX: Selection happens at the end so the signal gets a chance
        // to propagate to probe detectors before the next reading cycle.
        self.probe.tick();
    }

    pub fn snapshot(&self) -> ControlInputSnapshot {
        ControlInputSnapshot {
            pots: self.pots.values(),
            buttons: self.buttons.values(),
            cvs: self.cvs.values(),
            gates: self.gates.values(),
        }
    }
}
