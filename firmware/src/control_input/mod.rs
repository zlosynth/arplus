mod buttons;
mod cvs;
mod debouncer;
mod one_pole_filter;
mod pots;
mod probe;
mod triggers;

use arplus_control::ControlInputSnapshot;

pub use self::buttons::Pins as ButtonsPins;
pub use self::cvs::Pins as CvsPins;
pub use self::pots::Pins as PotsPins;
pub use self::probe::BroadcasterPin as ProbeBroadcasterPin;
pub use self::triggers::Pins as TriggersPins;

use self::buttons::Buttons;
use self::cvs::Cvs;
use self::pots::Pots;
use self::probe::Broadcaster as ProbeBroadcaster;
use self::triggers::Triggers;
use crate::system::hal::adc::{Adc, Enabled};
use crate::system::hal::pac::{ADC1, ADC2};

pub struct ControlInputInterface {
    pots: Pots,
    buttons: Buttons,
    cvs: Cvs,
    triggers: Triggers,
    probe: ProbeBroadcaster,
    adc_1: Adc<ADC1, Enabled>,
    adc_2: Adc<ADC2, Enabled>,
}

pub struct Config {
    pub pots_pins: PotsPins,
    pub buttons_pins: ButtonsPins,
    pub cvs_pins: CvsPins,
    pub triggers_pins: TriggersPins,
    pub probe_pin: ProbeBroadcasterPin,
    pub adc_1: Adc<ADC1, Enabled>,
    pub adc_2: Adc<ADC2, Enabled>,
}

impl ControlInputInterface {
    pub fn new(config: Config) -> Self {
        Self {
            pots: Pots::new(config.pots_pins),
            buttons: Buttons::new(config.buttons_pins),
            cvs: Cvs::new(config.cvs_pins),
            triggers: Triggers::new(config.triggers_pins),
            probe: ProbeBroadcaster::new(config.probe_pin),
            adc_1: config.adc_1,
            adc_2: config.adc_2,
        }
    }

    pub fn sample(&mut self) {
        self.pots.sample(&mut self.adc_1, &mut self.adc_2);
        self.buttons.sample();
        self.cvs.sample(&mut self.adc_1, &mut self.adc_2);
        self.triggers.sample();

        // XXX: Selection happens at the end so the signal gets a chance
        // to propagate to probe detectors before the next reading cycle.
        self.probe.tick();
    }

    pub fn snapshot(&self) -> ControlInputSnapshot {
        ControlInputSnapshot {
            pots: self.pots.values(),
            buttons: self.buttons.values(),
            cvs: self.cvs.values(),
            trigger: self.triggers.values()[0],
        }
    }
}
